//! Patch slide-like parts and their shape trees in place (ADR-013, Tier 1D).
//!
//! The part is read again with a map from each `ObjectId` to the element it came
//! from; the wanted model is compared with what the tree says, and only the
//! attributes and children that differ are touched. Every other byte stays as it
//! was read, and a part not handed in is not opened at all.
//!
//! What this tier writes: a node's transform, `cNvPr` attributes, hyperlink
//! attributes, locks, `txBox`, placeholder attributes, `isPhoto`, `userDrawn`,
//! `useBgFill`, `bwMode`, through `drawing.rs` its geometry, fill, line, effects
//! and 3D, and through `text.rs` its text body; a part's root attributes,
//! `cSld/@name`, `clrMap` slots, `clrMapOvr`, `hf`, `txStyles` and `notesStyle`.
//! Anything else that differs (a node added, removed
//! or reordered, a background, a layout list) is refused as `CS-SLE-0001`, and
//! nothing is saved: those edits belong to the writer tier (1N) and to Phase 4.

use std::collections::HashSet;
use std::fmt;

use casual_slide_import::properties::{Elem, Findings, ImportError, ImportLimits, ns};
use casual_slide_import::shape::SourceMap;
use casual_slide_import::slide::{
    SlideError, SlideLimits, import_slide_part, import_slide_part_mapped,
};
use casual_slide_model::presentation::{Point, PositiveSize};
use casual_slide_model::shape::{
    AppNonVisual, Connection, Hyperlink, Locks, NodeKind, NonVisualDrawing, Placeholder, ShapeNode,
    Transform,
};
use casual_slide_model::slide::{
    DeckStructure, HeaderFooter, MasterTextStyles, SlideKind, SlidePart,
};
use casual_slide_model::theme::{ColorMapOverride, ColorMapSlot};
use casual_slide_ooxml::{
    Conformance, FindingKind, OoxmlLimits, PresentationPackage, Relationship, XmlTree,
};
use casual_slide_package::PartEdits;

use crate::presentation::SaveOutcome;
use crate::properties::{ExportError, Writer};
use crate::text::{patch_list_style, patch_text_body};
use crate::theme::{set_color_map_override, set_color_map_slot};

/// Why a slide-like part could not be written. Nothing was saved.
#[derive(Debug)]
#[non_exhaustive]
pub enum SlideExportError {
    /// Writing or reading back refused.
    Export(ExportError),
    /// Reading the part refused.
    Slide(SlideError),
    /// The model asks for an edit this tier does not write.
    Unsupported {
        /// What.
        what: &'static str,
        /// The part.
        part: String,
    },
    /// The node sits in an `mc:AlternateContent` branch, so its other branches
    /// would go stale (BEH-25).
    StaleAlternate {
        /// The part.
        part: String,
    },
    /// Two nodes of the part would share a `cNvPr/@id`.
    DuplicateShapeId {
        /// The part.
        part: String,
    },
    /// The part's ordinal is not in the deck's structure.
    NotInStructure {
        /// The ordinal.
        ordinal: u32,
    },
}

impl SlideExportError {
    /// The stable diagnostic code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Export(err) => err.code(),
            Self::Slide(err) => err.code(),
            Self::Unsupported { .. } => "CS-SLE-0001",
            Self::StaleAlternate { .. } => "CS-SLE-0002",
            Self::DuplicateShapeId { .. } => "CS-SLE-0003",
            Self::NotInStructure { .. } => "CS-SLE-0004",
        }
    }
}

impl fmt::Display for SlideExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::Export(err) => write!(f, "{err}"),
            Self::Slide(err) => write!(f, "{err}"),
            Self::Unsupported { what, part } => {
                write!(f, "[{code}] {part:?}: editing {what} is not supported")
            }
            Self::StaleAlternate { part } => write!(
                f,
                "[{code}] {part:?}: the node has alternate content an edit would leave stale"
            ),
            Self::DuplicateShapeId { part } => {
                write!(f, "[{code}] {part:?}: two shapes would share an id")
            }
            Self::NotInStructure { ordinal } => {
                write!(f, "[{code}] part {ordinal} is not in the deck")
            }
        }
    }
}

impl std::error::Error for SlideExportError {}

impl From<ExportError> for SlideExportError {
    fn from(err: ExportError) -> Self {
        Self::Export(err)
    }
}

impl From<SlideError> for SlideExportError {
    fn from(err: SlideError) -> Self {
        Self::Slide(err)
    }
}

impl From<ImportError> for SlideExportError {
    fn from(err: ImportError) -> Self {
        Self::Export(ExportError::Import(err))
    }
}

impl From<casual_slide_ooxml::OoxmlError> for SlideExportError {
    fn from(err: casual_slide_ooxml::OoxmlError) -> Self {
        Self::Export(ExportError::Ooxml(err))
    }
}

type Result<T> = std::result::Result<T, SlideExportError>;

const SP_PR_ORDER: &[&str] = &[
    "xfrm",
    "custGeom",
    "prstGeom",
    "noFill",
    "solidFill",
    "gradFill",
    "blipFill",
    "pattFill",
    "grpFill",
    "ln",
    "effectLst",
    "effectDag",
    "scene3d",
    "sp3d",
    "extLst",
];
const FRAME_ORDER: &[&str] = &["nvGraphicFramePr", "xfrm", "graphic", "extLst"];
const XFRM_ORDER: &[&str] = &["off", "ext", "chOff", "chExt"];
const SHAPE_ORDER: &[&str] = &["nvSpPr", "spPr", "style", "txBody", "extLst"];
const TEXT_STYLES_ORDER: &[&str] = &["titleStyle", "bodyStyle", "otherStyle", "extLst"];

fn root_order(kind: SlideKind) -> &'static [&'static str] {
    match kind {
        SlideKind::Slide => &["cSld", "clrMapOvr", "transition", "timing", "extLst"],
        SlideKind::Layout => &["cSld", "clrMapOvr", "transition", "timing", "hf", "extLst"],
        SlideKind::Master => &[
            "cSld",
            "clrMap",
            "sldLayoutIdLst",
            "transition",
            "timing",
            "hf",
            "txStyles",
            "extLst",
        ],
        SlideKind::Notes => &["cSld", "clrMapOvr", "extLst"],
        SlideKind::NotesMaster => &["cSld", "clrMap", "hf", "notesStyle", "extLst"],
        SlideKind::HandoutMaster => &["cSld", "clrMap", "hf", "extLst"],
    }
}

/// One part being patched.
struct Patch<'w, 'a> {
    w: Writer<'w>,
    map: &'a SourceMap,
}

impl Patch<'_, '_> {
    fn unsupported(&self, what: &'static str) -> SlideExportError {
        SlideExportError::Unsupported {
            what,
            part: self.w.part.to_owned(),
        }
    }

    fn child(&self, parent: &Elem, uris: &[&str], local: &str) -> Result<Option<Elem>> {
        Ok(parent.first_child(self.w.tree, self.w.part, uris, local)?)
    }

    fn need(&self, found: Option<Elem>, what: &'static str) -> Result<Elem> {
        found.ok_or_else(|| self.unsupported(what))
    }

    fn color_error(&self, _: crate::color::ExportError) -> SlideExportError {
        SlideExportError::Export(ExportError::MalformedElement {
            part: self.w.part.to_owned(),
        })
    }

    fn part(&mut self, root: &Elem, before: &SlidePart, after: &SlidePart) -> Result<()> {
        if before.kind != after.kind || before.id != after.id {
            return Err(self.unsupported("a part's kind or identity"));
        }
        let w = &mut self.w;
        w.attr(
            root,
            "showMasterSp",
            before.show_master_shapes.as_ref(),
            after.show_master_shapes.as_ref(),
        )?;
        w.attr(
            root,
            "showMasterPhAnim",
            before.show_master_placeholder_animations.as_ref(),
            after.show_master_placeholder_animations.as_ref(),
        )?;
        w.attr(root, "show", before.show.as_ref(), after.show.as_ref())?;
        w.attr(
            root,
            "matchingName",
            before.matching_name.as_ref(),
            after.matching_name.as_ref(),
        )?;
        w.attr(
            root,
            "type",
            before.layout_type.as_ref(),
            after.layout_type.as_ref(),
        )?;
        w.attr(
            root,
            "preserve",
            before.preserve.as_ref(),
            after.preserve.as_ref(),
        )?;
        w.attr(
            root,
            "userDrawn",
            before.user_drawn.as_ref(),
            after.user_drawn.as_ref(),
        )?;
        if before.layout_ids != after.layout_ids {
            return Err(self.unsupported("a layout list"));
        }
        let order = root_order(after.kind);
        sync_ordered(
            &mut self.w,
            root,
            ns::PML,
            "p",
            order,
            "txStyles",
            before.text_styles.as_ref(),
            after.text_styles.as_ref(),
            patch_text_styles,
        )?;
        sync_ordered(
            &mut self.w,
            root,
            ns::PML,
            "p",
            order,
            "notesStyle",
            before.notes_style.as_ref(),
            after.notes_style.as_ref(),
            patch_list_style,
        )?;
        sync_ordered(
            &mut self.w,
            root,
            ns::PML,
            "p",
            order,
            "hf",
            before.header_footer.as_ref(),
            after.header_footer.as_ref(),
            patch_header_footer,
        )?;
        if before.color_map != after.color_map {
            let (Some(b), Some(a)) = (&before.color_map, &after.color_map) else {
                return Err(self.unsupported("adding or removing a colour map"));
            };
            if b.extensions != a.extensions {
                return Err(self.unsupported("colour map extensions"));
            }
            let el = self.child(root, ns::PML, "clrMap")?;
            let el = self.need(el, "a colour map")?;
            for slot in ColorMapSlot::ALL {
                if b.get(*slot) == a.get(*slot) {
                    continue;
                }
                let Some(to) = a.get(*slot) else {
                    return Err(self.unsupported("removing a colour map slot"));
                };
                let changed = set_color_map_slot(self.w.tree, el.id, *slot, to)
                    .map_err(|e| self.color_error(e))?;
                self.w.changes += usize::from(changed);
            }
        }
        if before.color_map_override != after.color_map_override {
            self.color_map_override(root, order, after.color_map_override.as_ref())?;
        }
        match (&before.common, &after.common) {
            (Some(b), Some(a)) if b != a => {
                let el = self.child(root, ns::PML, "cSld")?;
                let el = self.need(el, "cSld")?;
                self.w.attr(&el, "name", b.name.as_ref(), a.name.as_ref())?;
                if b.background != a.background {
                    return Err(self.unsupported("a background"));
                }
                match (&b.shape_tree, &a.shape_tree) {
                    (Some(bt), Some(at)) => self.node(bt, at)?,
                    (None, None) => {}
                    _ => return Err(self.unsupported("adding or removing a shape tree")),
                }
            }
            (b, a) if b.is_some() != a.is_some() => {
                return Err(self.unsupported("adding or removing cSld"));
            }
            _ => {}
        }
        Ok(())
    }

    fn color_map_override(
        &mut self,
        root: &Elem,
        order: &[&str],
        after: Option<&ColorMapOverride>,
    ) -> Result<()> {
        let Some(wanted) = after else {
            self.w.remove_all(root, ns::PML, "clrMapOvr")?;
            return Ok(());
        };
        let holder = match self.child(root, ns::PML, "clrMapOvr")? {
            Some(el) => el,
            None => {
                let el = self.w.new_child(root, ns::PML, "p", "clrMapOvr")?;
                self.w.insert_in_order(root, order, "clrMapOvr", &el)?;
                el
            }
        };
        let changed = set_color_map_override(self.w.tree, holder.id, wanted)
            .map_err(|e| self.color_error(e))?;
        self.w.changes += usize::from(changed);
        Ok(())
    }

    fn node(&mut self, b: &ShapeNode, a: &ShapeNode) -> Result<()> {
        if b == a {
            return Ok(());
        }
        let same_shape = b.id == a.id
            && b.kind == a.kind
            && b.children.len() == a.children.len()
            && b.children
                .iter()
                .zip(&a.children)
                .all(|(x, y)| x.id == y.id && x.kind == y.kind);
        if !same_shape {
            return Err(self.unsupported("the shape tree's structure"));
        }
        let own_changed = b.non_visual != a.non_visual
            || b.locks != a.locks
            || b.text_box != a.text_box
            || b.start_connection != a.start_connection
            || b.end_connection != a.end_connection
            || b.app != a.app
            || b.transform != a.transform
            || b.bw_mode != a.bw_mode
            || b.use_bg_fill != a.use_bg_fill
            || b.properties != a.properties
            || b.graphic_uri != a.graphic_uri
            || b.rel_id != a.rel_id
            || b.text_body != a.text_body
            || b.style != a.style;
        if own_changed {
            let mapped = self
                .map
                .get(a.id)
                .ok_or_else(|| self.unsupported("a node not in the part"))?
                .clone();
            if mapped.in_alternate {
                return Err(SlideExportError::StaleAlternate {
                    part: self.w.part.to_owned(),
                });
            }
            self.own(&mapped.element, b, a)?;
        }
        for (bc, ac) in b.children.iter().zip(&a.children) {
            self.node(bc, ac)?;
        }
        Ok(())
    }

    fn own(&mut self, el: &Elem, b: &ShapeNode, a: &ShapeNode) -> Result<()> {
        if b.graphic_uri != a.graphic_uri {
            return Err(self.unsupported("a graphic frame's content"));
        }
        if b.rel_id != a.rel_id {
            return Err(self.unsupported("a content part's target"));
        }
        self.w.attr(
            el,
            "useBgFill",
            b.use_bg_fill.as_ref(),
            a.use_bg_fill.as_ref(),
        )?;
        let changed_nv = b.non_visual != a.non_visual
            || b.locks != a.locks
            || b.text_box != a.text_box
            || b.start_connection != a.start_connection
            || b.end_connection != a.end_connection
            || b.app != a.app;
        if changed_nv {
            let container = self.child(el, ns::PML, a.kind.non_visual_container())?;
            let container = self.need(container, "non-visual properties")?;
            self.non_visual(&container, b, a)?;
        }
        if b.transform != a.transform || b.bw_mode != a.bw_mode {
            self.transform(el, b, a)?;
        }
        if a.text_body.is_some() && a.kind != NodeKind::Shape {
            return Err(self.unsupported("a text body on a node that holds none"));
        }
        sync_ordered(
            &mut self.w,
            el,
            ns::PML,
            "p",
            SHAPE_ORDER,
            "txBody",
            b.text_body.as_ref(),
            a.text_body.as_ref(),
            |w, el, before, after| patch_text_body(w, el, before.map(|b| &**b), after),
        )?;
        if a.style.is_some()
            && !matches!(
                a.kind,
                NodeKind::Shape | NodeKind::Connector | NodeKind::Picture
            )
        {
            return Err(self.unsupported("a shape style on a node that holds none"));
        }
        sync_ordered(
            &mut self.w,
            el,
            ns::PML,
            "p",
            SHAPE_ORDER,
            "style",
            b.style.as_ref(),
            a.style.as_ref(),
            |w, el, before, after| crate::drawing::patch_style(w, el, before.map(|b| &**b), after),
        )?;
        if b.properties != a.properties {
            crate::drawing::patch_properties(
                &mut self.w,
                el,
                a.kind,
                &b.properties,
                &a.properties,
            )?;
        }
        Ok(())
    }

    fn non_visual(&mut self, container: &Elem, b: &ShapeNode, a: &ShapeNode) -> Result<()> {
        if b.non_visual != a.non_visual {
            let (Some(bn), Some(an)) = (&b.non_visual, &a.non_visual) else {
                return Err(self.unsupported("adding or removing cNvPr"));
            };
            let el = self.child(container, ns::PML, "cNvPr")?;
            let el = self.need(el, "cNvPr")?;
            self.drawing(&el, bn, an)?;
        }
        if b.start_connection != a.start_connection || b.end_connection != a.end_connection {
            if a.kind != NodeKind::Connector {
                return Err(self.unsupported("a connection on a node that is no connector"));
            }
            let el = self.child(container, ns::PML, a.kind.non_visual_properties())?;
            let el = self.need(el, "the connector's non-visual properties")?;
            let order = [a.kind.lock_element(), "stCxn", "endCxn", "extLst"];
            for (local, before, after) in [
                ("stCxn", &b.start_connection, &a.start_connection),
                ("endCxn", &b.end_connection, &a.end_connection),
            ] {
                sync_ordered(
                    &mut self.w,
                    &el,
                    ns::DML,
                    "a",
                    &order,
                    local,
                    before.as_ref(),
                    after.as_ref(),
                    patch_connection,
                )?;
            }
        }
        if b.locks != a.locks || b.text_box != a.text_box {
            let el = self.child(container, ns::PML, a.kind.non_visual_properties())?;
            let el = self.need(el, "the kind's non-visual properties")?;
            self.w
                .attr(&el, "txBox", b.text_box.as_ref(), a.text_box.as_ref())?;
            let lock = a.kind.lock_element();
            let order = [lock, "stCxn", "endCxn", "extLst"];
            let kind = a.kind;
            sync_ordered(
                &mut self.w,
                &el,
                ns::DML,
                "a",
                &order,
                lock,
                b.locks.as_ref(),
                a.locks.as_ref(),
                |w, el, before, after| patch_locks(w, el, kind, before, after),
            )?;
        }
        if b.app != a.app {
            let (Some(ba), Some(aa)) = (&b.app, &a.app) else {
                return Err(self.unsupported("adding or removing nvPr"));
            };
            let el = self.child(container, ns::PML, "nvPr")?;
            let el = self.need(el, "nvPr")?;
            self.app(&el, ba, aa)?;
        }
        Ok(())
    }

    fn drawing(&mut self, el: &Elem, b: &NonVisualDrawing, a: &NonVisualDrawing) -> Result<()> {
        if a.spid.is_none() || a.name.is_none() {
            return Err(self.unsupported("removing a required attribute"));
        }
        if b.creation_id != a.creation_id || b.decorative != a.decorative {
            return Err(self.unsupported("a drawing extension"));
        }
        let w = &mut self.w;
        w.attr(el, "id", b.spid.as_ref(), a.spid.as_ref())?;
        w.attr(el, "name", b.name.as_ref(), a.name.as_ref())?;
        w.attr(el, "descr", b.descr.as_ref(), a.descr.as_ref())?;
        w.attr(el, "hidden", b.hidden.as_ref(), a.hidden.as_ref())?;
        w.attr(el, "title", b.title.as_ref(), a.title.as_ref())?;
        for (local, before, after) in [
            ("hlinkClick", &b.hyperlink_click, &a.hyperlink_click),
            ("hlinkHover", &b.hyperlink_hover, &a.hyperlink_hover),
        ] {
            if before == after {
                continue;
            }
            let (Some(bl), Some(al)) = (before, after) else {
                return Err(self.unsupported("adding or removing a hyperlink"));
            };
            let link = self.child(el, ns::DML, local)?;
            let link = self.need(link, "a hyperlink")?;
            patch_hyperlink(&mut self.w, &link, bl, al)?;
        }
        Ok(())
    }

    fn app(&mut self, el: &Elem, b: &AppNonVisual, a: &AppNonVisual) -> Result<()> {
        if b.media != a.media {
            return Err(self.unsupported("media"));
        }
        self.w
            .attr(el, "isPhoto", b.is_photo.as_ref(), a.is_photo.as_ref())?;
        self.w.attr(
            el,
            "userDrawn",
            b.user_drawn.as_ref(),
            a.user_drawn.as_ref(),
        )?;
        if b.placeholder != a.placeholder {
            let (Some(bp), Some(ap)) = (&b.placeholder, &a.placeholder) else {
                return Err(self.unsupported("making a shape a placeholder or not"));
            };
            let ph = self.child(el, ns::PML, "ph")?;
            let ph = self.need(ph, "ph")?;
            patch_placeholder(&mut self.w, &ph, bp, ap)?;
        }
        Ok(())
    }

    fn transform(&mut self, el: &Elem, b: &ShapeNode, a: &ShapeNode) -> Result<()> {
        if a.kind == NodeKind::GraphicFrame {
            self.w
                .attr(el, "bwMode", b.bw_mode.as_ref(), a.bw_mode.as_ref())?;
            if b.transform != a.transform {
                if a.transform.is_none() {
                    return Err(self.unsupported("removing a graphic frame's required transform"));
                }
                let group = false;
                sync_ordered(
                    &mut self.w,
                    el,
                    ns::PML,
                    "p",
                    FRAME_ORDER,
                    "xfrm",
                    b.transform.as_ref(),
                    a.transform.as_ref(),
                    |w, el, before, after| patch_transform(w, el, group, before, after),
                )?;
            }
            return Ok(());
        }
        let holder = match a.kind {
            NodeKind::Group => "grpSpPr",
            NodeKind::ContentPart => return Err(self.unsupported("a content part's transform")),
            _ => "spPr",
        };
        let props = self.child(el, ns::PML, holder)?;
        let props = self.need(props, "shape properties")?;
        self.w
            .attr(&props, "bwMode", b.bw_mode.as_ref(), a.bw_mode.as_ref())?;
        let group = a.kind == NodeKind::Group;
        sync_ordered(
            &mut self.w,
            &props,
            ns::DML,
            "a",
            SP_PR_ORDER,
            "xfrm",
            b.transform.as_ref(),
            a.transform.as_ref(),
            |w, el, before, after| patch_transform(w, el, group, before, after),
        )?;
        Ok(())
    }
}

/// `a:CT_Connection`, a connector's end.
fn patch_connection(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&Connection>,
    a: &Connection,
) -> std::result::Result<(), ExportError> {
    w.attr(el, "id", b.map(|b| &b.spid), Some(&a.spid))?;
    w.attr(el, "idx", b.map(|b| &b.site), Some(&a.site))
}

/// As `Writer::sync_child`, but placing a new child among siblings of any
/// namespace: an `a:xfrm` goes before `a:prstGeom` in a `p:spPr`, and a `p:xfrm`
/// before `a:graphic`. Elements not in `order` are not landmarks.
#[allow(clippy::too_many_arguments)]
pub(crate) fn sync_ordered<T: PartialEq>(
    w: &mut Writer<'_>,
    parent: &Elem,
    uris: &[&str],
    preferred: &str,
    order: &[&str],
    local: &str,
    before: Option<&T>,
    after: Option<&T>,
    patch: impl FnOnce(&mut Writer<'_>, &Elem, Option<&T>, &T) -> std::result::Result<(), ExportError>,
) -> std::result::Result<(), ExportError> {
    if before == after {
        return Ok(());
    }
    let Some(wanted) = after else {
        return w.remove_all(parent, uris, local);
    };
    let el = match parent.first_child(w.tree, w.part, uris, local)? {
        Some(el) => el,
        None => {
            let el = w.new_child(parent, uris, preferred, local)?;
            let rank = |name: &str| order.iter().position(|o| *o == name);
            let mine = rank(local);
            let landmark = parent
                .children(w.tree, w.part)?
                .into_iter()
                .find(|c| {
                    (c.in_namespace(ns::PML) || c.in_namespace(ns::DML))
                        && matches!((rank(&c.local), mine), (Some(theirs), Some(mine)) if theirs > mine)
                })
                .map(|c| c.id);
            w.tree.insert_child(parent.id, landmark, el.id);
            el
        }
    };
    patch(w, &el, before, wanted)
}

fn patch_header_footer(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&HeaderFooter>,
    a: &HeaderFooter,
) -> std::result::Result<(), ExportError> {
    w.attr(
        el,
        "sldNum",
        b.and_then(|b| b.slide_number.as_ref()),
        a.slide_number.as_ref(),
    )?;
    w.attr(
        el,
        "hdr",
        b.and_then(|b| b.header.as_ref()),
        a.header.as_ref(),
    )?;
    w.attr(
        el,
        "ftr",
        b.and_then(|b| b.footer.as_ref()),
        a.footer.as_ref(),
    )?;
    w.attr(
        el,
        "dt",
        b.and_then(|b| b.date_time.as_ref()),
        a.date_time.as_ref(),
    )
}

fn patch_text_styles(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&MasterTextStyles>,
    a: &MasterTextStyles,
) -> std::result::Result<(), ExportError> {
    for (name, before, after) in [
        (
            "titleStyle",
            b.and_then(|b| b.title.as_ref()),
            a.title.as_ref(),
        ),
        (
            "bodyStyle",
            b.and_then(|b| b.body.as_ref()),
            a.body.as_ref(),
        ),
        (
            "otherStyle",
            b.and_then(|b| b.other.as_ref()),
            a.other.as_ref(),
        ),
    ] {
        sync_ordered(
            w,
            el,
            ns::PML,
            "p",
            TEXT_STYLES_ORDER,
            name,
            before,
            after,
            patch_list_style,
        )?;
    }
    Ok(())
}

fn patch_locks(
    w: &mut Writer<'_>,
    el: &Elem,
    kind: NodeKind,
    b: Option<&Locks>,
    a: &Locks,
) -> std::result::Result<(), ExportError> {
    for name in Locks::NAMES {
        let before = b.and_then(|b| b.get(name));
        let after = a.get(name);
        if before == after {
            continue;
        }
        if !kind.lock_attributes().contains(&name) {
            return Err(ExportError::ValueOutOfRange {
                what: "a lock this kind of node does not have",
                part: w.part.to_owned(),
            });
        }
        w.attr(el, name, before.as_ref(), after.as_ref())?;
    }
    Ok(())
}

pub(crate) fn patch_hyperlink(
    w: &mut Writer<'_>,
    el: &Elem,
    b: &Hyperlink,
    a: &Hyperlink,
) -> std::result::Result<(), ExportError> {
    if b.rel_id != a.rel_id {
        // An empty `r:id` is an action with no target, and names no relationship.
        if a.rel_id.as_deref() == Some("") {
            let Some(name) = el.name_of_ns(ns::REL, "id").map(str::to_owned) else {
                return Err(ExportError::UnknownRelationship {
                    part: w.part.to_owned(),
                });
            };
            w.attr(el, &name, b.rel_id.as_ref(), a.rel_id.as_ref())?;
        } else {
            w.rel_attr(el, b.rel_id.as_deref(), a.rel_id.as_deref())?;
        }
    }
    w.attr(
        el,
        "invalidUrl",
        b.invalid_url.as_ref(),
        a.invalid_url.as_ref(),
    )?;
    w.attr(el, "action", b.action.as_ref(), a.action.as_ref())?;
    w.attr(
        el,
        "tgtFrame",
        b.target_frame.as_ref(),
        a.target_frame.as_ref(),
    )?;
    w.attr(el, "tooltip", b.tooltip.as_ref(), a.tooltip.as_ref())?;
    w.attr(el, "history", b.history.as_ref(), a.history.as_ref())?;
    w.attr(
        el,
        "highlightClick",
        b.highlight_click.as_ref(),
        a.highlight_click.as_ref(),
    )?;
    w.attr(el, "endSnd", b.end_sound.as_ref(), a.end_sound.as_ref())
}

fn patch_placeholder(
    w: &mut Writer<'_>,
    el: &Elem,
    b: &Placeholder,
    a: &Placeholder,
) -> std::result::Result<(), ExportError> {
    w.attr(
        el,
        "type",
        b.placeholder_type.as_ref(),
        a.placeholder_type.as_ref(),
    )?;
    w.attr(el, "orient", b.orient.as_ref(), a.orient.as_ref())?;
    w.attr(el, "sz", b.size.as_ref(), a.size.as_ref())?;
    w.attr(el, "idx", b.idx.as_ref(), a.idx.as_ref())?;
    w.attr(
        el,
        "hasCustomPrompt",
        b.has_custom_prompt.as_ref(),
        a.has_custom_prompt.as_ref(),
    )
}

fn patch_point(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&Point>,
    a: &Point,
) -> std::result::Result<(), ExportError> {
    w.attr(el, "x", b.map(|b| &b.x), Some(&a.x))?;
    w.attr(el, "y", b.map(|b| &b.y), Some(&a.y))
}

fn patch_size(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&PositiveSize>,
    a: &PositiveSize,
) -> std::result::Result<(), ExportError> {
    if !(0..=casual_slide_model::presentation::POSITIVE_COORDINATE_MAX).contains(&a.cx)
        || !(0..=casual_slide_model::presentation::POSITIVE_COORDINATE_MAX).contains(&a.cy)
    {
        return Err(ExportError::ValueOutOfRange {
            what: "extent",
            part: w.part.to_owned(),
        });
    }
    // `DRAW-01`: what a file holds stays, and a new value stays within what PowerPoint
    // accepts (`MS-OI29500` 2.1.1330).
    let max = crate::drawing::POWERPOINT_COORDINATE_MAX;
    if (a.cx > max && b.is_none_or(|b| b.cx != a.cx))
        || (a.cy > max && b.is_none_or(|b| b.cy != a.cy))
    {
        return Err(ExportError::ValueOutOfRange {
            what: "an extent past what PowerPoint accepts",
            part: w.part.to_owned(),
        });
    }
    w.attr(el, "cx", b.map(|b| &b.cx), Some(&a.cx))?;
    w.attr(el, "cy", b.map(|b| &b.cy), Some(&a.cy))
}

fn patch_transform(
    w: &mut Writer<'_>,
    el: &Elem,
    group: bool,
    b: Option<&Transform>,
    a: &Transform,
) -> std::result::Result<(), ExportError> {
    if !group && (a.child_offset.is_some() || a.child_extent.is_some()) {
        return Err(ExportError::ValueOutOfRange {
            what: "a child transform on a node that is not a group",
            part: w.part.to_owned(),
        });
    }
    w.attr(
        el,
        "rot",
        b.and_then(|b| b.rotation.as_ref()),
        a.rotation.as_ref(),
    )?;
    w.attr(
        el,
        "flipH",
        b.and_then(|b| b.flip_h.as_ref()),
        a.flip_h.as_ref(),
    )?;
    w.attr(
        el,
        "flipV",
        b.and_then(|b| b.flip_v.as_ref()),
        a.flip_v.as_ref(),
    )?;
    let points = [
        ("off", b.and_then(|t| t.offset), a.offset),
        ("chOff", b.and_then(|t| t.child_offset), a.child_offset),
    ];
    for (local, before, after) in points {
        sync_ordered(
            w,
            el,
            ns::DML,
            "a",
            XFRM_ORDER,
            local,
            before.as_ref(),
            after.as_ref(),
            patch_point,
        )?;
    }
    let sizes = [
        ("ext", b.and_then(|t| t.extent), a.extent),
        ("chExt", b.and_then(|t| t.child_extent), a.child_extent),
    ];
    for (local, before, after) in sizes {
        sync_ordered(
            w,
            el,
            ns::DML,
            "a",
            XFRM_ORDER,
            local,
            before.as_ref(),
            after.as_ref(),
            patch_size,
        )?;
    }
    Ok(())
}

fn check_unique_ids(part: &SlidePart, before: &SlidePart, name: &str) -> Result<()> {
    let ids = |p: &SlidePart| -> Vec<u32> {
        p.shape_tree()
            .into_iter()
            .flat_map(ShapeNode::descendants)
            .filter_map(|n| n.non_visual.as_ref()?.spid)
            .collect()
    };
    let (old, new) = (ids(before), ids(part));
    if old == new {
        return Ok(());
    }
    // Only an id the edit changed may not collide; a file's own duplicates stay.
    let mut seen = HashSet::new();
    let duplicates: HashSet<u32> = new.iter().copied().filter(|id| !seen.insert(*id)).collect();
    let old_duplicates = {
        let mut seen = HashSet::new();
        old.iter()
            .copied()
            .filter(|id| !seen.insert(*id))
            .collect::<HashSet<u32>>()
    };
    if duplicates.is_subset(&old_duplicates) {
        Ok(())
    } else {
        Err(SlideExportError::DuplicateShapeId {
            part: name.to_owned(),
        })
    }
}

/// Patch one slide-like part's tree to say `after`. Returns the nodes changed;
/// with 0, the tree is as it was. What was asked is read back before returning.
#[allow(clippy::too_many_arguments)]
pub fn patch_slide_part(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &SlidePart,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
    xml_limits: &OoxmlLimits,
) -> Result<usize> {
    let (before, root, map) = import_slide_part_mapped(
        tree,
        part,
        rels,
        after.kind,
        after.id.part,
        limits,
        import_limits,
    )?;
    if &before == after {
        return Ok(0);
    }
    check_unique_ids(after, &before, part)?;
    let mut patch = Patch {
        w: Writer {
            tree,
            part,
            strict,
            known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
            changes: 0,
        },
        map: &map,
    };
    patch.part(&root, &before, after)?;
    let changes = patch.w.changes;
    let again = XmlTree::parse(&tree.to_bytes(), part, xml_limits)?;
    let mut findings = Findings::new(import_limits.max_findings);
    let mut color = casual_slide_import::color::Findings::default();
    let read = import_slide_part(
        &again,
        part,
        rels,
        after.kind,
        after.id.part,
        limits,
        import_limits,
        &mut findings,
        &mut color,
    )?;
    if &read != after {
        return Err(SlideExportError::Export(ExportError::RoundTrip {
            part: part.to_owned(),
        }));
    }
    Ok(changes)
}

/// The part edits that bring each given part to its model. Parts not given are
/// not read. Returns the edits and the names of the parts that changed.
pub fn slide_part_edits(
    package: &mut PresentationPackage,
    structure: &DeckStructure,
    parts: &[&SlidePart],
    limits: &SlideLimits,
    import_limits: &ImportLimits,
) -> Result<(PartEdits, Vec<String>)> {
    let strict = package.conformance() == Conformance::Strict;
    let xml_limits = *package.limits();
    let mut edits = PartEdits::default();
    let mut changed = Vec::new();
    for after in parts {
        let ordinal = after.id.part;
        let entry = structure
            .entry(ordinal)
            .ok_or(SlideExportError::NotInStructure { ordinal })?;
        if entry.kind != after.kind {
            return Err(SlideExportError::Unsupported {
                what: "a part's kind",
                part: entry.part.clone(),
            });
        }
        let bytes = package.read_part(&entry.part)?;
        let mut tree = XmlTree::parse(&bytes, &entry.part, &xml_limits)?;
        let rels = package.relationships_of(&entry.part).to_vec();
        let changes = patch_slide_part(
            &mut tree,
            &entry.part,
            strict,
            &rels,
            after,
            limits,
            import_limits,
            &xml_limits,
        )?;
        if changes > 0 {
            let out = tree.to_bytes();
            if out != bytes {
                edits.replace(entry.part.clone(), out);
                changed.push(entry.part.clone());
            }
        }
    }
    changed.sort();
    changed.dedup();
    Ok((edits, changed))
}

/// Save the given parts over the package they came from. Nothing else is read or
/// written; with nothing changed, the input bytes come back.
pub fn save_slide_parts(
    package: &mut PresentationPackage,
    structure: &DeckStructure,
    parts: &[&SlidePart],
    limits: &SlideLimits,
    import_limits: &ImportLimits,
) -> Result<SaveOutcome> {
    let (edits, changed) = slide_part_edits(package, structure, parts, limits, import_limits)?;
    if edits.is_empty() {
        return Ok(SaveOutcome {
            bytes: package.save_unedited(),
            changed_parts: changed,
            signature_invalidated: false,
        });
    }
    let signed = package
        .findings()
        .iter()
        .any(|f| f.kind == FindingKind::Signature);
    Ok(SaveOutcome {
        bytes: package.save_with(&edits)?,
        changed_parts: changed,
        signature_invalidated: signed,
    })
}
