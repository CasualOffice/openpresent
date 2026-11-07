//! Read a shape tree from an `XmlTree`: `sp`, `grpSp`, `cxnSp`, `pic`,
//! `graphicFrame` and `contentPart`, each a node with an [`ObjectId`].
//!
//! Each node's identity, transform, locks, non-visual properties and hyperlinks are
//! read here; geometry, fills, lines, effects and 3D by [`crate::drawing`], text bodies
//! by [`crate::text`], and a frame's table by [`crate::table`]. Everything else a node
//! holds (a chart reference, a diagram) stays in the tree and is reported as retained.
//!
//! The walk follows the input to a depth of [`SlideLimits::max_group_depth`] and
//! no further, and counts nodes against [`SlideLimits::max_shapes_per_part`]. An
//! `mc:AlternateContent` among a group's children reads as the branch markup
//! compatibility selects, by `casual-slide-ooxml`'s one rule.

use std::collections::HashSet;

use casual_slide_model::presentation::{
    COORDINATE_MIN, Coordinate, POSITIVE_COORDINATE_MAX, Point,
};
use casual_slide_model::shape::{
    AlternateBranch, AppNonVisual, Connection, Hyperlink, Locks, NodeKind, NonVisualDrawing,
    ObjectId, Placeholder, ShapeNode, Transform,
};
use casual_slide_model::table::Table;
use casual_slide_model::text::TextItem;
use casual_slide_ooxml::{AlternateResolution, Branch, MARKUP_COMPATIBILITY, Relationship};

use crate::color::Findings as ColorFindings;
use crate::drawing::{Ctx, note_dag_siblings, read_blip_fill};
use crate::media::{MediaLimits, is_media_element, read_app_extensions, read_media, read_wav};
use crate::presentation::read_positive_size;
use crate::properties::{Elem, ImportError, ImportFindingKind as K, Reader, ns};
use crate::slide::{SlideError, SlideLimits};
use crate::table::{TableCounts, TableReader};
use crate::text::{TextCounts, TextReader};
use crate::theme::read_shape_style;

/// `a16:` Office 2016 drawing, where `creationId` lives.
pub const A16: &[&str] = &["http://schemas.microsoft.com/office/drawing/2014/main"];
/// `adec:`, where `decorative` lives.
pub const ADEC: &[&str] = &["http://schemas.microsoft.com/office/drawing/2017/decorative"];
/// Markup compatibility.
use crate::retained::{model_beside_picture, read_frame_content};
use casual_slide_model::retained::RetainedObject;

const MC: &[&str] = &[MARKUP_COMPATIBILITY];

/// Where a node came from in the tree, for an export that patches it in place.
#[derive(Debug, Clone)]
pub struct MappedNode {
    /// The node's element, as read, its attributes untaken.
    pub element: Elem,
    /// It sits inside an `mc:AlternateContent` branch, so the branches not taken
    /// hold a copy an edit here would not change.
    pub in_alternate: bool,
}

/// Each node read, by [`ObjectId::node`]; index 0 is node 1, the tree's root.
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    /// The nodes, in the order of their ids.
    pub nodes: Vec<MappedNode>,
}

impl SourceMap {
    /// A node's source, by id.
    pub fn get(&self, id: ObjectId) -> Option<&MappedNode> {
        let at = usize::try_from(id.node).ok()?.checked_sub(1)?;
        self.nodes.get(at)
    }
}

/// The kind a PresentationML child of a group is, if it is a node.
pub fn node_kind(el: &Elem) -> Option<NodeKind> {
    if !el.in_namespace(ns::PML) {
        return None;
    }
    Some(match el.local.as_str() {
        "sp" => NodeKind::Shape,
        "grpSp" => NodeKind::Group,
        "cxnSp" => NodeKind::Connector,
        "pic" => NodeKind::Picture,
        "graphicFrame" => NodeKind::GraphicFrame,
        "contentPart" => NodeKind::ContentPart,
        _ => return None,
    })
}

/// One part's shape tree being read.
pub(crate) struct ShapeReader<'s, 'r, 'a> {
    pub r: &'s mut Reader<'r>,
    pub rels: &'s [Relationship],
    pub limits: &'s SlideLimits,
    pub part_ordinal: u32,
    pub alternates: &'a [AlternateResolution],
    pub cursor: usize,
    pub next_node: u32,
    pub spids: HashSet<u32>,
    pub map: Option<&'s mut SourceMap>,
    pub color: &'s mut ColorFindings,
    pub text_counts: TextCounts,
    pub table_counts: TableCounts,
}

impl ShapeReader<'_, '_, '_> {
    fn too_many(&self) -> SlideError {
        SlideError::TooManyShapes {
            part: self.r.part.to_owned(),
            limit: self.limits.max_shapes_per_part,
        }
    }

    /// Read `spTree`, or any group, as a node.
    // spec-audit: owner=spTree
    pub fn tree(&mut self, el: &Elem) -> Result<ShapeNode, SlideError> {
        self.node(el, NodeKind::Group, 0, false)
    }

    // spec-audit: owner=sp,cxnSp,pic,graphicFrame,grpSp,contentPart
    fn node(
        &mut self,
        el: &Elem,
        kind: NodeKind,
        depth: usize,
        in_alternate: bool,
    ) -> Result<ShapeNode, SlideError> {
        if depth > self.limits.max_group_depth {
            return Err(SlideError::GroupTooDeep {
                part: self.r.part.to_owned(),
                limit: self.limits.max_group_depth,
            });
        }
        let count = usize::try_from(self.next_node).unwrap_or(usize::MAX);
        if count >= self.limits.max_shapes_per_part {
            return Err(self.too_many());
        }
        let node = self
            .next_node
            .checked_add(1)
            .ok_or_else(|| self.too_many())?;
        let id = ObjectId {
            part: self.part_ordinal,
            node,
        };
        self.next_node = node;
        if let Some(map) = self.map.as_deref_mut() {
            map.nodes.push(MappedNode {
                element: el.clone(),
                in_alternate,
            });
        }
        let mut n = ShapeNode::new(id, kind);
        let mut el = el.clone();
        match kind {
            NodeKind::Shape => n.use_bg_fill = self.r.attr(&mut el, "useBgFill")?,
            NodeKind::GraphicFrame => n.bw_mode = self.r.attr(&mut el, "bwMode")?,
            NodeKind::ContentPart => {
                n.rel_id = el.take_ns(ns::REL, "id");
                match &n.rel_id {
                    None => self.r.note(K::MissingRequired, &el, Some("r:id")),
                    Some(id) => self.check_rel(&el, id),
                }
            }
            _ => {}
        }
        self.r.finish(&el);
        let mut seen = HashSet::new();
        for child in self.r.children(&el)? {
            if kind == NodeKind::Group {
                if let Some(child_kind) = node_kind(&child) {
                    let c = self.node(&child, child_kind, depth + 1, in_alternate)?;
                    n.children.push(c);
                    continue;
                }
                if child.is(MC, "AlternateContent") {
                    self.alternate(&child, &mut n, depth)?;
                    continue;
                }
            }
            if !child.in_namespace(ns::PML) && !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            let name = child.local.as_str();
            let modelled = name == kind.non_visual_container()
                || matches!(
                    (kind, name),
                    (
                        NodeKind::Shape | NodeKind::Connector | NodeKind::Picture,
                        "spPr"
                    ) | (NodeKind::Group, "grpSpPr")
                        | (NodeKind::GraphicFrame, "xfrm" | "graphic")
                        | (NodeKind::Picture, "blipFill")
                        | (NodeKind::Shape, "txBody")
                        | (
                            NodeKind::Shape | NodeKind::Connector | NodeKind::Picture,
                            "style"
                        )
                );
            if !modelled {
                // `extLst` and the unknown.
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(name.to_owned()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            match name {
                "spPr" | "grpSpPr" => self.properties(&child, &mut n)?,
                "blipFill" => {
                    let fill = read_blip_fill(self.r, self.rels, self.limits.drawing, &child)?;
                    n.properties.picture = fill.map(Box::new);
                }
                "xfrm" => n.transform = Some(self.transform(&child, false)?),
                "graphic" => {
                    let mut diagram = None;
                    n.graphic_uri =
                        self.graphic(&child, &mut n.table, &mut diagram, &mut n.retained)?;
                    n.diagram = diagram;
                }
                "txBody" => {
                    let mut text = TextReader {
                        r: &mut *self.r,
                        rels: self.rels,
                        color: &mut *self.color,
                        counts: &mut self.text_counts,
                        drawing: self.limits.drawing,
                    };
                    n.text_body = Some(Box::new(text.body(&child)?));
                }
                "style" => {
                    let style = read_shape_style(self.r.tree, child.id, &mut *self.color);
                    n.style = Some(Box::new(style));
                }
                _ => self.non_visual(&child, &mut n)?,
            }
        }
        Ok(n)
    }

    /// The branch markup compatibility selects, read in place among a group's
    /// children. The resolution list is in document order, as this walk is.
    fn alternate(
        &mut self,
        el: &Elem,
        group: &mut ShapeNode,
        depth: usize,
    ) -> Result<(), SlideError> {
        let found = self
            .alternates
            .get(self.cursor..)
            .unwrap_or_default()
            .iter()
            .position(|a| a.element == el.id);
        let Some(offset) = found else {
            self.r.retained(el);
            return Ok(());
        };
        let at = self.cursor.saturating_add(offset);
        self.cursor = at.saturating_add(1);
        let Some(resolution) = self.alternates.get(at) else {
            return Ok(());
        };
        let not_taken = resolution.not_taken.clone();
        self.not_taken_branches(el, &not_taken, group)?;
        let Some(resolution) = self.alternates.get(at) else {
            return Ok(());
        };
        let branch = match resolution.branch {
            Branch::Choice { element, .. } | Branch::Fallback { element } => element,
            Branch::None => {
                self.r.retained(el);
                return Ok(());
            }
        };
        if !resolution.not_taken.is_empty() {
            self.r.retained(el);
        }
        let branch = self.r.children(el)?.into_iter().find(|c| c.id == branch);
        let Some(branch) = branch else {
            return Ok(());
        };
        for child in self.r.children(&branch)? {
            match node_kind(&child) {
                Some(kind) => {
                    let mut c = self.node(&child, kind, depth + 1, true)?;
                    if kind == NodeKind::Picture {
                        c.retained =
                            model_beside_picture(self.r, self.rels, el, &child)?.map(Box::new);
                    }
                    group.children.push(c);
                }
                None => self.r.retained(&child),
            }
        }
        Ok(())
    }

    /// Read what a branch not taken holds of an equation or of ink. The branch stays in
    /// the tree, and is reported as before.
    fn not_taken_branches(
        &mut self,
        el: &Elem,
        not_taken: &[casual_slide_ooxml::ElementId],
        group: &mut ShapeNode,
    ) -> Result<(), SlideError> {
        for branch in self.r.children(el)? {
            if !not_taken.contains(&branch.id) || !branch.is(MC, "Choice") {
                continue;
            }
            let mut found = AlternateBranch {
                requires: branch.get("Requires").map(str::to_owned),
                ..AlternateBranch::default()
            };
            for child in self.r.children(&branch)? {
                if child.is(ns::PML, "contentPart") {
                    if let Some(id) = child.clone().take_ns(ns::REL, "id") {
                        self.check_rel(&child, &id);
                        found.content_parts.push(id);
                    }
                } else if child.is(ns::PML, "sp") {
                    self.branch_equations(&child, &mut found)?;
                }
            }
            if !found.content_parts.is_empty() || !found.equations.is_empty() {
                group.alternates.push(found);
            }
        }
        Ok(())
    }

    fn branch_equations(
        &mut self,
        sp: &Elem,
        found: &mut AlternateBranch,
    ) -> Result<(), SlideError> {
        for body in self.r.children(sp)? {
            if !body.is(ns::PML, "txBody") {
                continue;
            }
            for paragraph in self.r.children(&body)? {
                if !paragraph.is(ns::DML, "p") {
                    continue;
                }
                for item in self.r.children(&paragraph)? {
                    if !item.is(crate::math::A14, "m") {
                        continue;
                    }
                    let mut text = TextReader {
                        r: &mut *self.r,
                        rels: self.rels,
                        color: &mut *self.color,
                        counts: &mut self.text_counts,
                        drawing: self.limits.drawing,
                    };
                    if let TextItem::Math(math) = text.inline_math(&item)?
                        && let Some(math) = math.math
                    {
                        found.equations.push(math);
                    }
                }
            }
        }
        Ok(())
    }

    fn check_rel(&mut self, el: &Elem, id: &str) {
        if !id.is_empty() && !self.rels.iter().any(|r| r.id == id) {
            self.r.note(K::DanglingRelationship, el, Some("r:id"));
        }
    }

    // spec-audit: owner=nvSpPr,nvCxnSpPr,nvPicPr,nvGraphicFramePr,nvGrpSpPr
    fn non_visual(&mut self, el: &Elem, n: &mut ShapeNode) -> Result<(), SlideError> {
        self.r.finish(el);
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            let name = child.local.clone();
            let known = name == "cNvPr" || name == "nvPr" || name == n.kind.non_visual_properties();
            if !known {
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(name.clone()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            match name.as_str() {
                "cNvPr" => n.non_visual = Some(self.drawing(&child)?),
                "nvPr" => n.app = Some(self.app(&child, n.kind)?),
                _ => self.kind_properties(&child, n)?,
            }
        }
        Ok(())
    }

    // spec-audit: owner=cNvPr
    fn drawing(&mut self, el: &Elem) -> Result<NonVisualDrawing, SlideError> {
        let mut el = el.clone();
        let mut d = NonVisualDrawing {
            spid: self.r.required(&mut el, "id", |_| true)?,
            name: self.r.required(&mut el, "name", |_| true)?,
            descr: self.r.attr(&mut el, "descr")?,
            hidden: self.r.attr(&mut el, "hidden")?,
            title: self.r.attr(&mut el, "title")?,
            ..NonVisualDrawing::default()
        };
        if let Some(spid) = d.spid
            && !self.spids.insert(spid)
        {
            self.r.note(K::DuplicateId, &el, Some("id"));
        }
        self.r.finish(&el);
        let mut seen = HashSet::new();
        for child in self.r.children(&el)? {
            if child.in_namespace(ns::DML)
                && matches!(child.local.as_str(), "hlinkClick" | "hlinkHover")
            {
                if !seen.insert(child.local.clone()) {
                    self.r.note(K::DuplicateElement, &child, None);
                    continue;
                }
                let link = self.hyperlink(&child)?;
                if child.local == "hlinkClick" {
                    d.hyperlink_click = Some(link);
                } else {
                    d.hyperlink_hover = Some(link);
                }
            } else if child.is(ns::DML, "extLst") {
                self.drawing_extensions(&child, &mut d)?;
            } else {
                self.r.retained(&child);
            }
        }
        Ok(d)
    }

    /// `a16:creationId` and `adec:decorative` from `cNvPr/extLst`. Other extensions
    /// stay in the tree.
    // spec-audit: x:drawing/2014/main:<creationId>
    // spec-audit: x:drawing/2017/decorative:<decorative>
    fn drawing_extensions(
        &mut self,
        el: &Elem,
        d: &mut NonVisualDrawing,
    ) -> Result<(), SlideError> {
        for ext in self.r.children(el)? {
            if !ext.is(ns::DML, "ext") {
                self.r.retained(&ext);
                continue;
            }
            for item in self.r.children(&ext)? {
                let mut item = item.clone();
                if item.is(A16, "creationId") && d.creation_id.is_none() {
                    d.creation_id = self.r.attr(&mut item, "id")?;
                    self.r.finish(&item);
                } else if item.is(ADEC, "decorative") && d.decorative.is_none() {
                    d.decorative = self.r.attr(&mut item, "val")?;
                    self.r.finish(&item);
                } else {
                    self.r.retained(&item);
                }
            }
        }
        Ok(())
    }

    // spec-audit: owner=hlinkClick,hlinkHover
    fn hyperlink(&mut self, el: &Elem) -> Result<Hyperlink, SlideError> {
        Ok(read_hyperlink(self.r, self.rels, el)?)
    }

    // spec-audit: owner=cNvSpPr,cNvCxnSpPr,cNvPicPr,cNvGraphicFramePr,cNvGrpSpPr
    fn kind_properties(&mut self, el: &Elem, n: &mut ShapeNode) -> Result<(), SlideError> {
        let mut el = el.clone();
        if n.kind == NodeKind::Shape {
            n.text_box = self.r.attr(&mut el, "txBox")?;
        }
        if n.kind == NodeKind::Picture {
            return self.c_nv_pic_pr(&el, n);
        }
        self.r.finish(&el);
        let mut seen = HashSet::new();
        for child in self.r.children(&el)? {
            let name = child.local.as_str();
            let known = child.in_namespace(ns::DML)
                && (name == n.kind.lock_element()
                    || (n.kind == NodeKind::Connector && matches!(name, "stCxn" | "endCxn")));
            if !known {
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(name.to_owned()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            match name {
                "stCxn" => n.start_connection = self.connection(&child)?,
                "endCxn" => n.end_connection = self.connection(&child)?,
                _ => n.locks = Some(self.locks(&child, n.kind)?),
            }
        }
        Ok(())
    }

    fn c_nv_pic_pr(&mut self, el: &Elem, n: &mut ShapeNode) -> Result<(), SlideError> {
        let mut e = el.clone();
        n.prefer_relative_resize = self.r.attr(&mut e, "preferRelativeResize")?;
        self.r.finish(&e);
        let mut seen = false;
        for child in self.r.children(&e)? {
            if !child.is(ns::DML, "picLocks") {
                self.r.retained(&child);
            } else if seen {
                self.r.note(K::DuplicateElement, &child, None);
            } else {
                seen = true;
                n.locks = Some(self.pic_locks(&child)?);
            }
        }
        Ok(())
    }

    /// `AG_Locking` as any node has it, and `noCrop`, which only a picture does.
    fn pic_locks(&mut self, el: &Elem) -> Result<Locks, SlideError> {
        let mut e = el.clone();
        let no_crop = self.r.attr(&mut e, "noCrop")?;
        let mut locks = self.locks(&e, NodeKind::Picture)?;
        locks.no_crop = no_crop;
        Ok(locks)
    }

    // spec-audit: owner=spLocks,grpSpLocks,cxnSpLocks,picLocks,graphicFrameLocks
    fn locks(&mut self, el: &Elem, kind: NodeKind) -> Result<Locks, SlideError> {
        let mut el = el.clone();
        let mut locks = Locks::default();
        for name in kind.lock_attributes() {
            let value = self.r.attr::<bool>(&mut el, name)?;
            if let Some(slot) = locks.slot(name) {
                *slot = value;
            }
        }
        self.r.finish(&el);
        for child in self.r.children(&el)? {
            self.r.retained(&child);
        }
        Ok(locks)
    }

    // spec-audit: owner=stCxn,endCxn
    fn connection(&mut self, el: &Elem) -> Result<Option<Connection>, SlideError> {
        let mut el = el.clone();
        let spid = self.r.required::<u32>(&mut el, "id", |_| true)?;
        let site = self.r.required::<u32>(&mut el, "idx", |_| true)?;
        self.r.finish_leaf(&el)?;
        Ok(spid.zip(site).map(|(spid, site)| Connection { spid, site }))
    }

    // spec-audit: owner=nvPr
    fn app(&mut self, el: &Elem, _kind: NodeKind) -> Result<AppNonVisual, SlideError> {
        let mut el = el.clone();
        let mut app = AppNonVisual {
            is_photo: self.r.attr(&mut el, "isPhoto")?,
            user_drawn: self.r.attr(&mut el, "userDrawn")?,
            ..AppNonVisual::default()
        };
        self.r.finish(&el);
        let mut seen = HashSet::new();
        for child in self.r.children(&el)? {
            if is_media_element(&child) {
                if app.media.is_some() {
                    self.r.note(K::DuplicateElement, &child, None);
                } else {
                    app.media = Some(read_media(self.r, self.rels, &child)?);
                }
            } else if child.is(ns::PML, "ph") {
                if !seen.insert("ph") {
                    self.r.note(K::DuplicateElement, &child, None);
                    continue;
                }
                // MS-OI29500, Part 1 Section 19.3.1.33: PowerPoint allows none on a connector or a
                // group. Read as written all the same; the resolver matches none.
                app.placeholder = Some(self.placeholder(&child)?);
            } else if child.is(ns::PML, "extLst") {
                if !seen.insert("extLst") {
                    self.r.note(K::DuplicateElement, &child, None);
                    continue;
                }
                app.media_extension =
                    read_app_extensions(self.r, self.rels, &MediaLimits::default(), &child)?;
            } else {
                // `custDataLst` is Tier 1M.
                self.r.retained(&child);
            }
        }
        Ok(app)
    }

    // spec-audit: owner=ph
    fn placeholder(&mut self, el: &Elem) -> Result<Placeholder, SlideError> {
        let mut el = el.clone();
        let ph = Placeholder {
            placeholder_type: self.r.attr(&mut el, "type")?,
            orient: self.r.attr(&mut el, "orient")?,
            size: self.r.attr(&mut el, "sz")?,
            idx: self.r.attr(&mut el, "idx")?,
            has_custom_prompt: self.r.attr(&mut el, "hasCustomPrompt")?,
        };
        self.r.finish(&el);
        for child in self.r.children(&el)? {
            self.r.retained(&child);
        }
        Ok(ph)
    }

    // spec-audit: owner=spPr,grpSpPr
    fn properties(&mut self, el: &Elem, n: &mut ShapeNode) -> Result<(), SlideError> {
        let mut el = el.clone();
        n.bw_mode = self.r.attr(&mut el, "bwMode")?;
        self.r.finish(&el);
        let mut seen = false;
        let mut slots = HashSet::new();
        let group = n.kind == NodeKind::Group;
        for child in self.r.children(&el)? {
            if !child.is(ns::DML, "xfrm") {
                let mut drawing = Ctx::new(&mut *self.r, self.rels, self.limits.drawing);
                if !drawing.property(&child, &mut n.properties, group, &mut slots)? {
                    self.r.retained(&child);
                }
                continue;
            }
            if seen {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            seen = true;
            n.transform = Some(self.transform(&child, n.kind == NodeKind::Group)?);
        }
        note_dag_siblings(self.r, &el, &n.properties);
        Ok(())
    }

    // spec-audit: owner=xfrm
    fn transform(&mut self, el: &Elem, group: bool) -> Result<Transform, SlideError> {
        let mut el = el.clone();
        let mut t = Transform {
            rotation: self.r.attr(&mut el, "rot")?,
            flip_h: self.r.attr(&mut el, "flipH")?,
            flip_v: self.r.attr(&mut el, "flipV")?,
            ..Transform::default()
        };
        self.r.finish(&el);
        let mut seen = HashSet::new();
        for child in self.r.children(&el)? {
            let name = child.local.as_str();
            let known = child.in_namespace(ns::DML)
                && (matches!(name, "off" | "ext") || (group && matches!(name, "chOff" | "chExt")));
            if !known {
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(name.to_owned()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            match name {
                "off" => t.offset = read_point(self.r, &child)?,
                "ext" => t.extent = read_positive_size(self.r, &child)?,
                "chOff" => t.child_offset = read_point(self.r, &child)?,
                _ => t.child_extent = read_positive_size(self.r, &child)?,
            }
        }
        Ok(t)
    }

    fn rel_ids(
        &mut self,
        el: &Elem,
    ) -> Result<casual_slide_model::diagram::schema::RelIds, SlideError> {
        let limits = self.r.limits.graphic;
        let mut text_counts = TextCounts::default();
        let mut c = crate::diagram::SchemaReader::new(
            &mut *self.r,
            self.rels,
            &mut *self.color,
            &mut text_counts,
            self.limits.drawing,
            limits,
        );
        crate::diagram::schema::read_rel_ids(&mut c, el).map_err(|err| match err {
            crate::diagram::GraphicError::Import(err) => SlideError::from(err),
            // One element, with no children to count.
            crate::diagram::GraphicError::Bound { part, .. } => {
                SlideError::from(ImportError::TooMany {
                    what: "diagram references",
                    part,
                    limit: 1,
                })
            }
        })
    }

    // spec-audit: owner=graphic
    // spec-audit: a:<graphic>
    fn graphic(
        &mut self,
        el: &Elem,
        table: &mut Option<Box<Table>>,
        diagram: &mut Option<Box<casual_slide_model::diagram::schema::RelIds>>,
        retained: &mut Option<Box<RetainedObject>>,
    ) -> Result<Option<String>, SlideError> {
        self.r.finish(el);
        let mut uri = None;
        for child in self.r.children(el)? {
            if !child.is(ns::DML, "graphicData") || uri.is_some() {
                self.r.retained(&child);
                continue;
            }
            let mut data = child.clone();
            uri = self.r.required::<String>(&mut data, "uri", |_| true)?;
            self.r.finish(&data);
            // A chart reference, a diagram, an OLE object: later tiers.
            for inner in self.r.children(&data)? {
                if inner.is(crate::diagram::schema::NS_DGM, "relIds") {
                    if diagram.is_some() {
                        self.r.note(K::DuplicateElement, &inner, None);
                    } else {
                        *diagram = Some(Box::new(self.rel_ids(&inner)?));
                    }
                } else if !inner.is(ns::DML, "tbl") {
                    self.r.retained(&inner);
                    if retained.is_none() {
                        *retained = read_frame_content(self.r, self.rels, &inner)?.map(Box::new);
                    }
                } else if table.is_some() {
                    self.r.note(K::DuplicateElement, &inner, None);
                } else {
                    let mut reader = TableReader {
                        r: &mut *self.r,
                        rels: self.rels,
                        limits: &self.limits.table,
                        drawing: self.limits.drawing,
                        color: &mut *self.color,
                        text_counts: &mut self.text_counts,
                        counts: &mut self.table_counts,
                    };
                    *table = Some(Box::new(reader.tbl(&inner)?));
                }
            }
        }
        Ok(uri)
    }
}

/// `a:CT_Hyperlink`, on a shape or a run.
// spec-audit: owner=hlinkClick,hlinkHover
pub(crate) fn read_hyperlink(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    el: &Elem,
) -> Result<Hyperlink, ImportError> {
    let mut el = el.clone();
    let rel_id = el.take_ns(ns::REL, "id");
    if let Some(id) = &rel_id
        && !id.is_empty()
        && !rels.iter().any(|r| r.id == *id)
    {
        r.note(K::DanglingRelationship, &el, Some("r:id"));
    }
    let mut link = Hyperlink {
        rel_id,
        invalid_url: r.attr(&mut el, "invalidUrl")?,
        action: r.attr(&mut el, "action")?,
        target_frame: r.attr(&mut el, "tgtFrame")?,
        tooltip: r.attr(&mut el, "tooltip")?,
        history: r.attr(&mut el, "history")?,
        highlight_click: r.attr(&mut el, "highlightClick")?,
        end_sound: r.attr(&mut el, "endSnd")?,
        sound: None,
    };
    r.finish(&el);
    for child in r.children(&el)? {
        if !child.is(ns::DML, "snd") {
            // `extLst` stays.
            r.retained(&child);
        } else if link.sound.is_some() {
            r.note(K::DuplicateElement, &child, None);
        } else {
            link.sound = Some(read_wav(r, rels, &child)?);
        }
    }
    Ok(link)
}

/// `a:CT_Point2D`. `None` unless both coordinates are usable.
// spec-audit: owner=off,chOff
pub fn read_point(
    r: &mut Reader<'_>,
    el: &Elem,
) -> Result<Option<Point>, crate::properties::ImportError> {
    let mut el = el.clone();
    let range = |c: &Coordinate| (COORDINATE_MIN..=POSITIVE_COORDINATE_MAX).contains(&c.0);
    let x = r.required::<Coordinate>(&mut el, "x", range)?;
    let y = r.required::<Coordinate>(&mut el, "y", range)?;
    r.finish_leaf(&el)?;
    Ok(x.zip(y).map(|(x, y)| Point { x, y }))
}
