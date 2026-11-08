//! Building a deck (`VS-08`): New Slide from one of the deck's layouts, Duplicate,
//! Delete, Hide and reorder, and inserting a text box, a preset shape, a line or a
//! picture. Each is an operation of `casual-slide-transaction` with its inverse, so
//! undo and redo are the one history's.
//!
//! # What a new thing is
//!
//! A new slide is the part the writer's own `AddSlide` would write for its layout, read
//! back by the reader; a new shape is a small XML template, read back the same way. So
//! the model never holds a node the reader would not give, and the save's patch, which
//! reads every part back, agrees with it.
//!
//! # Saving
//!
//! A save starts from the file as opened and runs three plans through the one writer,
//! each over what the last wrote:
//!
//! 1. **Slides.** New slides (`AddSlide` on the layout, or `DuplicateSlide` of the file's
//!    slide they copy), then deleted ones, then moves until the order is the model's.
//! 2. **Shapes.** At each slide's top level, shapes the model no longer has are removed
//!    and new ones inserted empty, with their ids, in z-order.
//! 3. **Patch.** Each slide part is patched to its model: shapes are paired by
//!    `cNvPr/@id`, so a new shape gets its look and text through the same patch an
//!    edited one does (ADR-013).
//!
//! With nothing changed, no plan runs and the file comes back byte for byte.

use std::collections::BTreeSet;

use casual_slide_export::writer::{MediaBytes, NewShape, Op, SavePlan, SlideRef, new_slide_part};
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_part};
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode};
use casual_slide_model::slide::{PartEntry, SlideKind, SlidePart, SlideParts};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};
use casual_slide_render::Deck;
use casual_slide_transaction::Operation;

use crate::{BridgeError, Viewer};

/// The largest picture a user may insert, as the writer's own media bound.
pub const MAX_PICTURE_BYTES: usize = 64 * 1024 * 1024;

/// The prefix of a relationship id minted for a picture inserted in this session. It
/// never reaches the file: the writer names the relationship it adds.
pub(crate) const NEW_REL: &str = "rIdOpenPresentNew";

pub(crate) const NAMESPACES: &str = "xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
     xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
     xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"";

pub(crate) const TREE_HEAD: &str =
    "<p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>";

/// What Insert puts on a slide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Insert {
    /// Insert > Text Box: no fill, no line, a box that grows with its text.
    TextBox,
    /// A rectangle, styled from the theme as PowerPoint's shapes are.
    Rectangle,
    /// An oval.
    Oval,
    /// A straight line.
    Line,
    /// A line ending in an arrowhead.
    Arrow,
    /// Any preset of the Shapes gallery, by its table name, which is also its
    /// `ST_ShapeType` token (`SHAPE-01`). A line preset is a connector; its ends are
    /// [`Ends`].
    Preset(&'static str, Ends),
    /// Insert > WordArt (`UX-006`): a text box holding "Your text here" in style
    /// [`WORDART_STYLES`]`[n]`, its fill, outline and text warp.
    WordArt(usize),
}

/// The WordArt gallery (`UX-006`), our own simple styles by name: the text's fill and
/// outline as `RRGGBB` (empty for none), and its `prstTxWarp` (`textNoShape` for none).
/// Bold, 54 point, centred, as PowerPoint inserts WordArt.
pub const WORDART_STYLES: [(&str, &str, &str, &str); 8] = [
    ("Fill: Black", "000000", "", "textNoShape"),
    ("Fill: Blue", "156082", "", "textNoShape"),
    (
        "Fill: Gray; Outline: Blue",
        "595959",
        "156082",
        "textNoShape",
    ),
    (
        "Fill: Orange; Outline: Black",
        "E97132",
        "000000",
        "textNoShape",
    ),
    ("Arch Up", "156082", "", "textArchUp"),
    ("Wave", "196B24", "", "textWave1"),
    ("Inflate", "A02B93", "", "textInflate"),
    ("Chevron Up", "E97132", "", "textChevron"),
];

/// The arrowheads a line preset is drawn with, as the Lines gallery offers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Ends {
    /// None.
    #[default]
    Plain,
    /// One at the end the drag finished on.
    Tail,
    /// One at each end.
    Both,
}

/// The presets drawn as a connector, `p:cxnSp`, as PowerPoint's Lines gallery does.
pub(crate) fn is_connector(prst: &str) -> bool {
    matches!(
        prst,
        "line"
            | "straightConnector1"
            | "bentConnector2"
            | "bentConnector3"
            | "bentConnector4"
            | "bentConnector5"
            | "curvedConnector2"
            | "curvedConnector3"
            | "curvedConnector4"
            | "curvedConnector5"
    )
}

/// The presets that are outlines rather than solids: PowerPoint inserts them with the
/// theme's thin line and no fill.
fn is_outline(prst: &str) -> bool {
    matches!(
        prst,
        "arc"
            | "leftBracket"
            | "rightBracket"
            | "leftBrace"
            | "rightBrace"
            | "bracketPair"
            | "bracePair"
    )
}

impl Insert {
    /// From a bridge number: 0 text box, 1 rectangle, 2 oval, 3 line, 4 arrow.
    pub fn from_u32(n: u32) -> Option<Self> {
        Some(match n {
            0 => Self::TextBox,
            1 => Self::Rectangle,
            2 => Self::Oval,
            3 => Self::Line,
            4 => Self::Arrow,
            _ => return None,
        })
    }

    fn base_name(self) -> &'static str {
        match self {
            Self::TextBox => "TextBox",
            Self::Rectangle => "Rectangle",
            Self::Oval => "Oval",
            Self::Line => "Straight Connector",
            Self::Arrow => "Straight Arrow Connector",
            Self::Preset(..) => "Shape",
            Self::WordArt(_) => "TextBox",
        }
    }
}

/// Where a part this session added came from, for the save's first plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Origin {
    /// A new slide on this layout.
    Layout(u32),
    /// A copy of this slide.
    Copy(u32),
}

/// A picture's type, from its first bytes, never from the user's file name.
pub(crate) fn sniff(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some(("png", "image/png"))
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some(("jpeg", "image/jpeg"))
    } else {
        None
    }
}

pub(crate) fn spid(node: &ShapeNode) -> Option<u32> {
    node.non_visual.as_ref()?.spid
}

pub(crate) fn embed(node: &ShapeNode) -> Option<&str> {
    node.properties
        .picture
        .as_ref()?
        .blip
        .as_ref()?
        .embed
        .as_deref()
}

/// Read a slide part's XML as the reader does.
pub(crate) fn read_slide(xml: &[u8], part: &str, ordinal: u32) -> Result<SlidePart, BridgeError> {
    read_slide_kind(xml, part, ordinal, SlideKind::Slide)
}

/// Read a slide-like part's XML of this kind as the reader does: a new notes slide
/// (`UX-011`).
pub(crate) fn read_slide_kind(
    xml: &[u8],
    part: &str,
    ordinal: u32,
    kind: SlideKind,
) -> Result<SlidePart, BridgeError> {
    let xml_limits = OoxmlLimits::default();
    let tree = XmlTree::parse(xml, part, &xml_limits).map_err(|e| BridgeError::Build(e.code()))?;
    let import_limits = ImportLimits::default();
    let mut findings = Findings::new(import_limits.max_findings);
    let mut color = casual_slide_import::color::Findings::default();
    import_slide_part(
        &tree,
        part,
        &[],
        kind,
        ordinal,
        &SlideLimits::default(),
        &import_limits,
        &mut findings,
        &mut color,
    )
    .map_err(|e| BridgeError::Build(e.code()))
}

/// Give every node of a part this ordinal.
pub(crate) fn renumber_part(part: &mut SlidePart, ordinal: u32) {
    part.id = ObjectId::part(ordinal);
    if let Some(root) = part.shape_tree_mut() {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            node.id.part = ordinal;
            stack.extend(node.children.iter_mut());
        }
    }
}

/// The XML of one new shape, at a box in EMU.
fn shape_xml(
    kind: Insert,
    id: u32,
    name: &str,
    rel: Option<&str>,
    b: [i64; 4],
    flip: (bool, bool),
) -> String {
    let [x, y, cx, cy] = b;
    let flips = match flip {
        (false, false) => "",
        (true, false) => " flipH=\"1\"",
        (false, true) => " flipV=\"1\"",
        (true, true) => " flipH=\"1\" flipV=\"1\"",
    };
    let xfrm = format!(
        "<a:xfrm{flips}><a:off x=\"{x}\" y=\"{y}\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm>"
    );
    let nv = |container: &str, own: &str| {
        format!(
            "<p:{container}><p:cNvPr id=\"{id}\" name=\"{name}\"/>{own}<p:nvPr/></p:{container}>"
        )
    };
    let style = |ln: u8, fill: u8, font: &str, shade: bool| {
        let line_color = if shade {
            "<a:schemeClr val=\"accent1\"><a:shade val=\"50000\"/></a:schemeClr>"
        } else {
            "<a:schemeClr val=\"accent1\"/>"
        };
        format!(
            "<p:style><a:lnRef idx=\"{ln}\">{line_color}</a:lnRef>\
             <a:fillRef idx=\"{fill}\"><a:schemeClr val=\"accent1\"/></a:fillRef>\
             <a:effectRef idx=\"0\"><a:schemeClr val=\"accent1\"/></a:effectRef>\
             <a:fontRef idx=\"minor\"><a:schemeClr val=\"{font}\"/></a:fontRef></p:style>"
        )
    };
    match (kind, rel) {
        (_, Some(rel)) => format!(
            "<p:pic>{}<p:blipFill><a:blip r:embed=\"{rel}\"/><a:stretch><a:fillRect/></a:stretch>\
             </p:blipFill><p:spPr>{xfrm}<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></p:spPr></p:pic>",
            nv(
                "nvPicPr",
                "<p:cNvPicPr><a:picLocks noChangeAspect=\"1\"/></p:cNvPicPr>"
            )
        ),
        (Insert::WordArt(n), None) => {
            let (_, fill, line, warp) = WORDART_STYLES.get(n).copied().unwrap_or(WORDART_STYLES[0]);
            let ln = if line.is_empty() {
                String::new()
            } else {
                format!(
                    "<a:ln w=\"12700\"><a:solidFill><a:srgbClr val=\"{line}\"/></a:solidFill></a:ln>"
                )
            };
            format!(
                "<p:sp>{}<p:spPr>{xfrm}<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:noFill/></p:spPr>\
                 <p:txBody><a:bodyPr wrap=\"none\" rtlCol=\"0\"><a:prstTxWarp prst=\"{warp}\"><a:avLst/></a:prstTxWarp>\
                 <a:spAutoFit/></a:bodyPr><a:lstStyle/><a:p><a:pPr algn=\"ctr\"/><a:r><a:rPr lang=\"en-US\" sz=\"5400\" b=\"1\" dirty=\"0\">\
                 {ln}<a:solidFill><a:srgbClr val=\"{fill}\"/></a:solidFill></a:rPr><a:t>Your text here</a:t></a:r></a:p></p:txBody></p:sp>",
                nv("nvSpPr", "<p:cNvSpPr txBox=\"1\"/>")
            )
        }
        (Insert::TextBox, None) => format!(
            "<p:sp>{}<p:spPr>{xfrm}<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:noFill/></p:spPr>\
             <p:txBody><a:bodyPr wrap=\"square\" rtlCol=\"0\"><a:spAutoFit/></a:bodyPr><a:lstStyle/>\
             <a:p><a:endParaRPr lang=\"en-US\" dirty=\"0\"/></a:p></p:txBody></p:sp>",
            nv("nvSpPr", "<p:cNvSpPr txBox=\"1\"/>")
        ),
        (Insert::Rectangle | Insert::Oval, None) => {
            let prst = if kind == Insert::Oval {
                "ellipse"
            } else {
                "rect"
            };
            format!(
                "<p:sp>{}<p:spPr>{xfrm}<a:prstGeom prst=\"{prst}\"><a:avLst/></a:prstGeom></p:spPr>{}\
                 <p:txBody><a:bodyPr rtlCol=\"0\" anchor=\"ctr\"/><a:lstStyle/>\
                 <a:p><a:pPr algn=\"ctr\"/><a:endParaRPr lang=\"en-US\" dirty=\"0\"/></a:p></p:txBody></p:sp>",
                nv("nvSpPr", "<p:cNvSpPr/>"),
                style(2, 1, "lt1", true)
            )
        }
        (Insert::Preset(prst, ends), None) if is_connector(prst) => {
            let line = match ends {
                Ends::Plain => "",
                Ends::Tail => "<a:ln><a:tailEnd type=\"triangle\"/></a:ln>",
                Ends::Both => {
                    "<a:ln><a:headEnd type=\"triangle\"/><a:tailEnd type=\"triangle\"/></a:ln>"
                }
            };
            format!(
                "<p:cxnSp>{}<p:spPr>{xfrm}<a:prstGeom prst=\"{prst}\"><a:avLst/></a:prstGeom>{line}</p:spPr>{}</p:cxnSp>",
                nv("nvCxnSpPr", "<p:cNvCxnSpPr/>"),
                style(1, 0, "tx1", false)
            )
        }
        (Insert::Preset(prst, _), None) => {
            let (look, font) = if is_outline(prst) {
                (style(1, 0, "tx1", false), "")
            } else {
                (style(2, 1, "lt1", true), "<a:pPr algn=\"ctr\"/>")
            };
            format!(
                "<p:sp>{}<p:spPr>{xfrm}<a:prstGeom prst=\"{prst}\"><a:avLst/></a:prstGeom></p:spPr>{look}\
                 <p:txBody><a:bodyPr rtlCol=\"0\" anchor=\"ctr\"/><a:lstStyle/>\
                 <a:p>{font}<a:endParaRPr lang=\"en-US\" dirty=\"0\"/></a:p></p:txBody></p:sp>",
                nv("nvSpPr", "<p:cNvSpPr/>")
            )
        }
        (Insert::Line | Insert::Arrow, None) => {
            let line = if kind == Insert::Arrow {
                "<a:ln><a:tailEnd type=\"triangle\"/></a:ln>"
            } else {
                ""
            };
            format!(
                "<p:cxnSp>{}<p:spPr>{xfrm}<a:prstGeom prst=\"line\"><a:avLst/></a:prstGeom>{line}</p:spPr>{}</p:cxnSp>",
                nv("nvCxnSpPr", "<p:cNvCxnSpPr/>"),
                style(1, 0, "tx1", false)
            )
        }
    }
}

/// The longest shape name a host may give, in characters.
const MAX_NAME_CHARS: usize = 128;

/// A host's shape name as XML attribute text: control characters dropped, the rest
/// escaped, at most [`MAX_NAME_CHARS`].
fn escape_name(name: &str) -> String {
    let mut out = String::new();
    for c in name
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_NAME_CHARS)
    {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

/// The highest `cNvPr/@id` and node number in a part.
pub(crate) fn highest(part: &SlidePart) -> (u32, u32) {
    part.shape_tree()
        .into_iter()
        .flat_map(ShapeNode::descendants)
        .fold((1, 0), |(s, n), node| {
            (s.max(spid(node).unwrap_or(0)), n.max(node.id.node))
        })
}

impl Viewer {
    /// File > New: the built-in blank presentation, one Title Slide on a 16:9 master
    /// with four layouts (`casual_slide_export::blank`).
    pub fn blank() -> Result<Self, BridgeError> {
        let bytes =
            casual_slide_export::blank::blank_deck().map_err(|e| BridgeError::Build(e.code()))?;
        Self::open(bytes)
    }

    /// File > New from a template (`PAR-18`): the blank presentation in built-in theme
    /// `theme`'s colours and fonts, written and opened again, so it starts as a deck
    /// just opened does, with nothing to undo.
    pub fn template(theme: usize) -> Result<Self, BridgeError> {
        let mut deck = Self::blank()?;
        deck.apply_theme(theme, 0, false)?;
        Self::open(deck.save()?)
    }

    /// The part ordinal of slide `index`, which names its shapes' [`ObjectId`]s.
    pub fn slide_part(&self, index: usize) -> Result<u32, BridgeError> {
        self.slide_ordinal(index)
    }

    /// The deck's layouts, as `(ordinal, name)`: each master's in order, as PowerPoint's
    /// New Slide gallery lists them. A layout with no name is listed by its type.
    pub fn layouts(&self) -> Vec<(u32, String)> {
        let parts = self.deck.parts();
        parts
            .structure
            .layouts
            .iter()
            .filter_map(|&o| {
                let part = parts.get(o)?;
                let name = part
                    .common
                    .as_ref()
                    .and_then(|c| c.name.clone())
                    .filter(|n| !n.is_empty())
                    .unwrap_or_else(|| "Layout".to_owned());
                Some((o, name))
            })
            .collect()
    }

    /// The layout slide `index` is on, by ordinal.
    pub fn layout_of(&self, index: usize) -> Option<u32> {
        let parts = self.deck.parts();
        parts.structure.entry(parts.structure.slide(index)?)?.layout
    }

    pub(crate) fn next_ordinal(&self) -> Result<u32, BridgeError> {
        u32::try_from(self.deck.parts().parts.len()).map_err(|_| BridgeError::BadArgument)
    }

    fn add_slide_part(
        &mut self,
        at: usize,
        entry: PartEntry,
        part: SlidePart,
        origin: Origin,
    ) -> Result<usize, BridgeError> {
        self.add_slide_parts(vec![(at, entry, part, origin)])?;
        Ok(at)
    }

    /// Add several new slides, each `(place, entry, part, origin)`, as one undo step.
    pub(crate) fn add_slide_parts(
        &mut self,
        slides: Vec<(usize, PartEntry, SlidePart, Origin)>,
    ) -> Result<(), BridgeError> {
        let mut ops = Vec::with_capacity(slides.len() * 2);
        let mut origins = Vec::with_capacity(slides.len());
        for (at, entry, part, origin) in slides {
            let ordinal = entry.id.part;
            ops.push(Operation::AddPart {
                entry: Box::new(entry),
                part: Box::new(part),
            });
            ops.push(Operation::InsertSlide { index: at, ordinal });
            origins.push((ordinal, origin));
        }
        self.typing = None;
        self.history
            .apply(self.deck.parts_mut(), Operation::Batch(ops))?;
        self.origins.extend(origins);
        self.reports.clear();
        Ok(())
    }

    /// A new slide part on layout `layout` at part ordinal `ordinal`, with an empty copy
    /// of each of the layout's placeholders, and its entry: what New Slide adds.
    pub(crate) fn layout_slide(
        &mut self,
        layout: u32,
        ordinal: u32,
    ) -> Result<(PartEntry, SlidePart), BridgeError> {
        let layout_entry = self
            .deck
            .parts()
            .structure
            .entry(layout)
            .filter(|e| e.kind == SlideKind::Layout)
            .ok_or(BridgeError::BadArgument)?
            .clone();
        let xml = self
            .deck
            .read_part(&layout_entry.part)
            .ok_or(BridgeError::Build("CS-WSM-0007"))?;
        let tree = XmlTree::parse(&xml, &layout_entry.part, &OoxmlLimits::default())
            .map_err(|e| BridgeError::Build(e.code()))?;
        let name = format!("ppt/slides/openpresent-new-{ordinal}.xml");
        let slide = new_slide_part(self.deck.conformance(), &tree);
        let part = read_slide(&slide, &name, ordinal)?;
        let entry = PartEntry {
            id: ObjectId::part(ordinal),
            kind: SlideKind::Slide,
            part: name,
            layout: Some(layout),
            master: None,
            notes: None,
            theme: None,
        };
        Ok((entry, part))
    }

    /// New Slide: a slide on layout `layout` (an ordinal from [`Viewer::layouts`]) at
    /// place `at`, with an empty copy of each of the layout's placeholders. One undo step.
    pub fn new_slide(&mut self, at: usize, layout: u32) -> Result<usize, BridgeError> {
        let parts = self.deck.parts();
        if at > parts.structure.slides.len() {
            return Err(BridgeError::Edit(
                casual_slide_transaction::TxnError::NoSuchSlide(at),
            ));
        }
        let ordinal = self.next_ordinal()?;
        let (entry, part) = self.layout_slide(layout, ordinal)?;
        self.add_slide_part(at, entry, part, Origin::Layout(layout))
    }

    /// Duplicate Slide: a copy of slide `index`, just after it. Its notes are not
    /// copied before a save; the file's copy keeps them.
    pub fn duplicate_slide(&mut self, index: usize) -> Result<usize, BridgeError> {
        let source = self.slide_ordinal(index)?;
        let parts = self.deck.parts();
        let mut part = parts.get(source).ok_or(BridgeError::BadArgument)?.clone();
        let mut entry = parts
            .structure
            .entry(source)
            .ok_or(BridgeError::BadArgument)?
            .clone();
        let ordinal = self.next_ordinal()?;
        renumber_part(&mut part, ordinal);
        entry.id = ObjectId::part(ordinal);
        entry.part = format!("ppt/slides/openpresent-new-{ordinal}.xml");
        entry.notes = None;
        self.deck.share_relationships(source, ordinal);
        self.add_slide_part(index.saturating_add(1), entry, part, Origin::Copy(source))
    }

    /// Delete Slide: one undo step. The deck may be left with no slide, as PowerPoint's.
    pub fn delete_slide(&mut self, index: usize) -> Result<(), BridgeError> {
        self.typing = None;
        self.history
            .apply(self.deck.parts_mut(), Operation::RemoveSlide { index })?;
        self.reports.clear();
        Ok(())
    }

    /// Move slide `from` to place `to`: one undo step.
    pub fn move_slide(&mut self, from: usize, to: usize) -> Result<(), BridgeError> {
        self.typing = None;
        self.history
            .apply(self.deck.parts_mut(), Operation::MoveSlide { from, to })?;
        self.reports.clear();
        Ok(())
    }

    /// Whether slide `index` is hidden in a slide show.
    pub fn is_hidden(&self, index: usize) -> Result<bool, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        Ok(self.deck.parts().get(ordinal).and_then(|p| p.show) == Some(false))
    }

    /// Hide Slide, or show it again: one undo step. Shown again, `@show` is removed, as
    /// absent means shown.
    pub fn set_hidden(&mut self, index: usize, hidden: bool) -> Result<(), BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        self.typing = None;
        let show = if hidden { Some(false) } else { None };
        self.history.apply(
            self.deck.parts_mut(),
            Operation::SetSlideShow { ordinal, show },
        )?;
        Ok(())
    }

    pub(crate) fn insert_node(
        &mut self,
        index: usize,
        kind: Insert,
        rel: Option<&str>,
        b: [i64; 4],
        flip: (bool, bool),
        label: Option<&str>,
    ) -> Result<u32, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        // In a shared session, an id from this participant's own range.
        let mint = self.shared_mint();
        let parts = self.deck.parts();
        let part = parts.get(ordinal).ok_or(BridgeError::BadArgument)?;
        let root = part.shape_tree().ok_or(BridgeError::BadArgument)?;
        let (parent, at) = (root.id, root.children.len());
        let (top_spid, top_node) = highest(part);
        let id = match mint {
            Some(m) => m,
            None => top_spid.checked_add(1).ok_or(BridgeError::BadArgument)?,
        };
        let name = if rel.is_some() {
            format!("Picture {}", id.saturating_sub(1))
        } else if let Some(label) = label {
            format!("{} {}", escape_name(label), id.saturating_sub(1))
        } else {
            format!("{} {}", kind.base_name(), id.saturating_sub(1))
        };
        let clamp = |v: i64| v.clamp(0, i64::from(i32::MAX));
        let b = [clamp(b[0]), clamp(b[1]), clamp(b[2]), clamp(b[3])];
        let element = shape_xml(kind, id, &name, rel, b, flip);
        let xml = format!(
            "<p:sld {NAMESPACES}><p:cSld><p:spTree>{TREE_HEAD}{element}</p:spTree></p:cSld></p:sld>"
        );
        let read = read_slide(xml.as_bytes(), "ppt/slides/openpresent-insert.xml", ordinal)?;
        let mut node = read
            .shape_tree()
            .and_then(|t| t.children.first())
            .cloned()
            .ok_or(BridgeError::Build("CS-WSM-0007"))?;
        let node_number = match mint {
            Some(m) => m,
            None => top_node.checked_add(1).ok_or(BridgeError::BadArgument)?,
        };
        node.id = ObjectId {
            part: ordinal,
            node: node_number,
        };
        self.shared_remember(node.id, &element);
        node.children.clear();
        self.typing = None;
        self.history.apply(
            self.deck.parts_mut(),
            Operation::InsertNode {
                parent,
                index: at,
                node: Box::new(node),
            },
        )?;
        self.reports.remove(&index);
        Ok(node_number)
    }

    /// Insert a text box, shape, line or arrow on slide `index` at this box (EMU), on
    /// top of the others: one undo step. Returns its node.
    pub fn insert_shape(
        &mut self,
        index: usize,
        kind: Insert,
        b: [i64; 4],
    ) -> Result<u32, BridgeError> {
        self.insert_node(index, kind, None, b, (false, false), None)
    }

    /// Insert > Picture: PNG or JPEG bytes, told apart by their signature, at this box.
    /// Anything else is refused with `CS-WSM-0008`. Returns its node.
    pub fn insert_picture(
        &mut self,
        index: usize,
        bytes: Vec<u8>,
        b: [i64; 4],
    ) -> Result<u32, BridgeError> {
        let (extension, content_type) = sniff(&bytes).ok_or(BridgeError::NotAPicture)?;
        if bytes.len() > MAX_PICTURE_BYTES {
            return Err(BridgeError::NotAPicture);
        }
        let ordinal = self.slide_ordinal(index)?;
        // Shared, its bytes cross with it, within the wire's bound (`COL-04`).
        self.shared_media_fits(bytes.len())?;
        let (rel, key) = self.mint_media();
        self.deck.add_media(ordinal, &rel, &key, bytes.clone());
        self.new_media.insert(
            rel.clone(),
            MediaBytes {
                bytes,
                extension: extension.to_owned(),
                content_type: content_type.to_owned(),
            },
        );
        self.insert_node(
            index,
            Insert::Rectangle,
            Some(&rel),
            b,
            (false, false),
            None,
        )
    }

    /// The part a new part's copy chain starts from: a layout, or a slide of the file.
    fn root_origin(&self, mut ordinal: u32) -> Result<Origin, BridgeError> {
        let original = self.original_count();
        for _ in 0..=self.origins.len() {
            match self.origins.get(&ordinal) {
                Some(Origin::Layout(l)) => return Ok(Origin::Layout(*l)),
                Some(Origin::Copy(s)) if *s >= original => ordinal = *s,
                Some(Origin::Copy(s)) => return Ok(Origin::Copy(*s)),
                None => break,
            }
        }
        Err(BridgeError::Save("CS-WSM-0009"))
    }

    pub(crate) fn original_count(&self) -> u32 {
        u32::try_from(self.original.parts.len()).unwrap_or(u32::MAX)
    }

    /// The deck as a file: see the module's notes.
    pub(crate) fn save_built(&self) -> Result<Vec<u8>, BridgeError> {
        let bytes = self.slides_pass(self.deck.source_bytes())?;
        let bytes = self.shapes_pass(bytes)?;
        let bytes = self.order_pass(bytes)?;
        let bytes = self.notes_pass(bytes)?;
        let (bytes, links) = self.links_pass(bytes)?;
        let bytes = self.patch_pass(bytes, &links)?;
        let bytes = self.sections_pass(bytes)?;
        let bytes = self.chart_pass(bytes)?;
        let bytes = self.design_pass(bytes)?;
        let bytes = self.review_pass(bytes)?;
        self.properties_pass(bytes)
    }

    /// What Design changed in themes and the slide size (`VS-12`), last, on the file the
    /// other passes left.
    fn design_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let mut package = PresentationPackage::open(bytes.clone(), OoxmlLimits::default())
            .map_err(|e| BridgeError::Save(e.code()))?;
        let edits = self.design_edits(&mut package)?;
        if edits.is_empty() {
            return Ok(bytes);
        }
        let mut plan = SavePlan::new();
        for edit in edits {
            plan = match edit {
                crate::DesignEdit::Op(op) => plan.op(op),
                crate::DesignEdit::Replace(part, bytes) => plan.replace_part(part, bytes),
            };
        }
        Ok(plan
            .save(&mut package)
            .map_err(|e| BridgeError::Save(e.code()))?
            .bytes)
    }

    pub(crate) fn run(plan: &SavePlan<'_>, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let mut package = PresentationPackage::open(bytes, OoxmlLimits::default())
            .map_err(|e| BridgeError::Save(e.code()))?;
        Ok(plan
            .save(&mut package)
            .map_err(|e| BridgeError::Save(e.code()))?
            .bytes)
    }

    fn slides_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let now = &self.deck.parts().structure.slides;
        let before = &self.original.structure.slides;
        if now == before {
            return Ok(bytes);
        }
        let original = self.original_count();
        let name = |o: u32| -> Result<String, BridgeError> {
            self.original
                .structure
                .entry(o)
                .map(|e| e.part.clone())
                .ok_or(BridgeError::Save("CS-WSM-0009"))
        };
        let mut plan = SavePlan::new();
        // What the file's slide list holds, op by op.
        let mut list: Vec<u32> = before.clone();
        for &o in now.iter().filter(|&&o| o >= original) {
            match self.root_origin(o)? {
                Origin::Layout(l) => {
                    let layout = name(l)?;
                    plan = plan.op(Op::AddSlide {
                        position: None,
                        layout: Some(layout),
                    });
                    list.push(o);
                }
                Origin::Copy(s) => {
                    let at = list
                        .iter()
                        .position(|&x| x == s)
                        .ok_or(BridgeError::Save("CS-WSM-0009"))?;
                    plan = plan.op(Op::DuplicateSlide {
                        slide: SlideRef::Part(name(s)?),
                    });
                    list.insert(at.saturating_add(1), o);
                }
            }
        }
        let kept: BTreeSet<u32> = now.iter().copied().collect();
        for &o in before.iter().filter(|o| !kept.contains(o)) {
            plan = plan.op(Op::DeleteSlide {
                slide: SlideRef::Part(name(o)?),
            });
            list.retain(|&x| x != o);
        }
        for (i, &o) in now.iter().enumerate() {
            let j = list
                .iter()
                .position(|&x| x == o)
                .ok_or(BridgeError::Save("CS-WSM-0009"))?;
            if j != i {
                plan = plan.op(Op::MoveSlide {
                    slide: SlideRef::Index(j),
                    position: i,
                });
                let moved = list.remove(j);
                list.insert(i, moved);
            }
        }
        Self::run(&plan, bytes)
    }

    /// Each slide of the file paired with the model's, by place.
    pub(crate) fn pairs<'a>(
        &'a self,
        file: &'a SlideParts,
    ) -> Result<Vec<(String, &'a SlidePart, &'a SlidePart)>, BridgeError> {
        let model = self.deck.parts();
        if file.structure.slides.len() != model.structure.slides.len() {
            return Err(BridgeError::Save("CS-WSM-0009"));
        }
        let mut out = Vec::new();
        for (i, &o) in file.structure.slides.iter().enumerate() {
            let (Some(entry), Some(f), Some(m)) =
                (file.structure.entry(o), file.get(o), model.slide(i))
            else {
                continue;
            };
            out.push((entry.part.clone(), f, m));
        }
        Ok(out)
    }

    fn shapes_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let file = Deck::open(bytes.clone())?;
        let mut removals = Vec::new();
        let mut inserts = Vec::new();
        for (name, f, m) in self.pairs(file.parts())? {
            let (Some(ft), Some(mt)) = (f.shape_tree(), m.shape_tree()) else {
                continue;
            };
            let in_model: BTreeSet<u32> = mt.children.iter().filter_map(spid).collect();
            let in_file: BTreeSet<u32> = ft.children.iter().filter_map(spid).collect();
            for node in &ft.children {
                if let Some(id) = spid(node).filter(|id| !in_model.contains(id)) {
                    removals.push(Op::RemoveShape {
                        slide: SlideRef::Part(name.clone()),
                        shape_id: id,
                    });
                }
            }
            for (index, node) in mt.children.iter().enumerate() {
                let Some(id) = spid(node).filter(|id| !in_file.contains(id)) else {
                    continue;
                };
                let placeholder = node.placeholder();
                let shape = match node.kind {
                    // A footer placeholder Header & Footer added (`UX-006`).
                    NodeKind::Shape if placeholder.is_some() => NewShape::Placeholder {
                        kind: placeholder
                            .and_then(|p| p.placeholder_type)
                            .map(|t| {
                                use casual_slide_model::properties::Lexical;
                                t.to_lexical()
                            })
                            .unwrap_or_default(),
                        idx: placeholder.and_then(|p| p.idx),
                    },
                    NodeKind::Shape => NewShape::Shape,
                    NodeKind::Connector => NewShape::Connector,
                    NodeKind::Picture => {
                        let media = embed(node)
                            .and_then(|rel| self.new_media.get(rel))
                            .ok_or(BridgeError::Save("CS-WSM-0009"))?;
                        NewShape::Picture(media.clone())
                    }
                    NodeKind::GraphicFrame => self.new_frame(node)?,
                    _ => return Err(BridgeError::Save("CS-WSM-0009")),
                };
                inserts.push(Op::InsertShape {
                    slide: SlideRef::Part(name.clone()),
                    index,
                    shape,
                    shape_id: id,
                    name: node
                        .non_visual
                        .as_ref()
                        .and_then(|n| n.name.clone())
                        .unwrap_or_default(),
                });
            }
        }
        if removals.is_empty() && inserts.is_empty() {
            return Ok(bytes);
        }
        let mut plan = SavePlan::new();
        for op in removals.into_iter().chain(inserts) {
            plan = plan.op(op);
        }
        Self::run(&plan, bytes)
    }

    fn patch_pass(
        &self,
        bytes: Vec<u8>,
        links: &std::collections::BTreeMap<String, String>,
    ) -> Result<Vec<u8>, BridgeError> {
        let file = Deck::open(bytes.clone())?;
        let mut changed: Vec<SlidePart> = Vec::new();
        let mut pairs = self.pairs(file.parts())?;
        // Notes slides, edited under the slide (`UX-011`).
        pairs.extend(self.notes_pairs(file.parts()));
        // Layouts and masters, by name: Slide Size scales their shapes too (`VS-12`).
        let model = self.deck.parts();
        for entry in &file.parts().structure.parts {
            if !matches!(entry.kind, SlideKind::Layout | SlideKind::Master) {
                continue;
            }
            let Some(m) = model
                .structure
                .parts
                .iter()
                .find(|e| e.kind == entry.kind && e.part == entry.part)
                .and_then(|e| model.get(e.id.part))
            else {
                continue;
            };
            if let Some(f) = file.parts().get(entry.id.part) {
                pairs.push((entry.part.clone(), f, m));
            }
        }
        for (_, f, m) in pairs {
            let mut after = f.clone();
            after.show = m.show;
            // Transitions and animations (`VS-10`): the writer patches only what differs.
            after.transition.clone_from(&m.transition);
            after.timing.clone_from(&m.timing);
            // A master's text styles, which Slide Size scales (`UX-015`).
            after.text_styles.clone_from(&m.text_styles);
            // A background set by Format Background (`VS-12`).
            if let (Some(c), Some(k)) = (after.common.as_mut(), m.common.as_ref()) {
                c.background.clone_from(&k.background);
            }
            if let (Some(root), Some(model_root)) = (after.shape_tree_mut(), m.shape_tree()) {
                overlay(root, model_root, links);
            }
            if &after != f {
                changed.push(after);
            }
        }
        if changed.is_empty() {
            return Ok(bytes);
        }
        let refs: Vec<&SlidePart> = changed.iter().collect();
        let limits = SlideLimits::default();
        let plan = SavePlan::new().slide_parts(&file.parts().structure, &refs, &limits);
        Self::run(&plan, bytes)
    }
}

/// Bring a node of the file to its model's values, keeping the file's own ids and what
/// only the file knows: a new picture's relationship, which the writer named.
fn overlay(
    file: &mut ShapeNode,
    model: &ShapeNode,
    links: &std::collections::BTreeMap<String, String>,
) {
    let children = core::mem::take(&mut file.children);
    let id = file.id;
    let picture = file.properties.picture.clone();
    let rel_id = file.rel_id.clone();
    let mut copy = model.clone();
    copy.children = Vec::new();
    copy.id = id;
    copy.rel_id = rel_id;
    if embed(model).is_some_and(|e| e.starts_with(NEW_REL)) {
        copy.properties.picture = picture;
    }
    // A new chart's relationship is the writer's too (`VS-11`).
    let model_chart_rel = model.chart.as_ref().and_then(|c| c.rel_id.as_deref());
    if model_chart_rel.is_some_and(|r| r.starts_with(NEW_REL)) {
        copy.chart.clone_from(&file.chart);
    }
    *file = copy;
    // A link added this session names the relationship the links pass gave it
    // (`UX-006`).
    if let Some(link) = file
        .non_visual
        .as_mut()
        .and_then(|nv| nv.hyperlink_click.as_mut())
        && let Some(id) = link.rel_id.as_ref().and_then(|r| links.get(r))
    {
        link.rel_id = Some(id.clone());
    }
    let mut used = BTreeSet::new();
    file.children = children
        .into_iter()
        .map(|mut child| {
            let found = model.children.iter().enumerate().find(|(i, c)| {
                !used.contains(i)
                    && match spid(&child) {
                        Some(s) => spid(c) == Some(s),
                        None => spid(c).is_none() && c.id.node == child.id.node,
                    }
            });
            if let Some((i, m)) = found {
                used.insert(i);
                overlay(&mut child, m, links);
            }
            child
        })
        .collect();
}
