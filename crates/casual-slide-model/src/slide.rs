//! Slides, layouts, masters, notes slides, the notes master and the handout
//! master as plain typed values (`docs/spec/SLIDE.md`), and the deck's structure:
//! which part is which, and how they chain.
//!
//! The six part kinds share most of their shape, so one [`SlidePart`] holds them,
//! with a field per attribute and child; a field a kind does not have stays `None`.
//! Transitions and timing are Tier 1I's, in [`crate::transition`] and [`crate::timing`].
//! Inheritance between these parts is resolved
//! by `casual-slide-style` at read time, never stored here (ADR-006).

use crate::properties::token_enum;
use crate::shape::{BlackWhiteMode, ObjectId, ShapeNode};
use crate::text::TextListStyle;
use crate::theme::{ColorMap, ColorMapOverride, StyleMatrixRef};
use crate::timing::{SlideTiming, Slot};
use crate::transition::SlideTransition;

/// Smallest `ST_SlideLayoutId`. Layout and master ids share one pool (`INV-ID-03`).
pub const SLIDE_LAYOUT_ID_MIN: u32 = 2_147_483_648;

/// The six kinds of slide-like part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SlideKind {
    /// `p:sldMaster`.
    Master,
    /// `p:sldLayout`.
    Layout,
    /// `p:sld`.
    Slide,
    /// `p:notes`.
    Notes,
    /// `p:notesMaster`.
    NotesMaster,
    /// `p:handoutMaster`.
    HandoutMaster,
}

impl SlideKind {
    /// The part's root element.
    pub fn root(self) -> &'static str {
        match self {
            Self::Master => "sldMaster",
            Self::Layout => "sldLayout",
            Self::Slide => "sld",
            Self::Notes => "notes",
            Self::NotesMaster => "notesMaster",
            Self::HandoutMaster => "handoutMaster",
        }
    }

    /// The relationship type's last segment.
    pub fn rel_type(self) -> &'static str {
        match self {
            Self::Master => "slideMaster",
            Self::Layout => "slideLayout",
            Self::Slide => "slide",
            Self::Notes => "notesSlide",
            Self::NotesMaster => "notesMaster",
            Self::HandoutMaster => "handoutMaster",
        }
    }

    /// The part's content type.
    pub fn content_type(self) -> &'static str {
        match self {
            Self::Master => {
                "application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"
            }
            Self::Layout => {
                "application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"
            }
            Self::Slide => "application/vnd.openxmlformats-officedocument.presentationml.slide+xml",
            Self::Notes => {
                "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml"
            }
            Self::NotesMaster => {
                "application/vnd.openxmlformats-officedocument.presentationml.notesMaster+xml"
            }
            Self::HandoutMaster => {
                "application/vnd.openxmlformats-officedocument.presentationml.handoutMaster+xml"
            }
        }
    }

    /// Whether the part is a top-level slide, with its own `clrMap`.
    pub fn is_top_level(self) -> bool {
        matches!(self, Self::Master | Self::NotesMaster | Self::HandoutMaster)
    }
}

token_enum!(
    /// `p:ST_SlideLayoutType`.
    SlideLayoutType {
        Title = "title",
        Tx = "tx",
        TwoColTx = "twoColTx",
        Tbl = "tbl",
        TxAndChart = "txAndChart",
        ChartAndTx = "chartAndTx",
        Dgm = "dgm",
        Chart = "chart",
        TxAndClipArt = "txAndClipArt",
        ClipArtAndTx = "clipArtAndTx",
        TitleOnly = "titleOnly",
        Blank = "blank",
        TxAndObj = "txAndObj",
        ObjAndTx = "objAndTx",
        ObjOnly = "objOnly",
        Obj = "obj",
        TxAndMedia = "txAndMedia",
        MediaAndTx = "mediaAndTx",
        ObjOverTx = "objOverTx",
        TxOverObj = "txOverObj",
        TxAndTwoObj = "txAndTwoObj",
        TwoObjAndTx = "twoObjAndTx",
        TwoObjOverTx = "twoObjOverTx",
        FourObj = "fourObj",
        VertTx = "vertTx",
        ClipArtAndVertTx = "clipArtAndVertTx",
        VertTitleAndTx = "vertTitleAndTx",
        VertTitleAndTxOverChart = "vertTitleAndTxOverChart",
        TwoObj = "twoObj",
        ObjAndTwoObj = "objAndTwoObj",
        TwoObjAndObj = "twoObjAndObj",
        Cust = "cust",
        SecHead = "secHead",
        TwoTxTwoObj = "twoTxTwoObj",
        ObjTx = "objTx",
        PicTx = "picTx",
    }
);

/// `p:CT_HeaderFooter`. `extLst` stays in the tree.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HeaderFooter {
    /// `@sldNum`.
    pub slide_number: Option<bool>,
    /// `@hdr`.
    pub header: Option<bool>,
    /// `@ftr`.
    pub footer: Option<bool>,
    /// `@dt`.
    pub date_time: Option<bool>,
}

/// `p:CT_BackgroundProperties`. The fill and effects are Tier 1E, and stay in the
/// tree; what is here is whether each is stated.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BackgroundProperties {
    /// `@shadeToTitle`.
    pub shade_to_title: Option<bool>,
    /// An `EG_FillProperties` child is present.
    pub has_fill: bool,
    /// An `EG_EffectProperties` child is present.
    pub has_effects: bool,
}

/// `p:EG_Background`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundFill {
    /// `bgPr`.
    Properties(BackgroundProperties),
    /// `bgRef`: a theme background fill and the colour standing for `phClr`.
    Reference(StyleMatrixRef),
}

/// `p:CT_Background`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Background {
    /// `@bwMode`.
    pub bw_mode: Option<BlackWhiteMode>,
    /// The choice. `None` if neither is there, which the schema does not allow.
    pub fill: Option<BackgroundFill>,
}

/// `p:CT_CommonSlideData`, `cSld`. `custDataLst` and `extLst` stay in
/// the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonSlideData {
    /// `@name`.
    pub name: Option<String>,
    /// `bg`.
    pub background: Option<Background>,
    /// `spTree`, a group whose id is node 1 of the part.
    pub shape_tree: Option<ShapeNode>,
    /// `p:controls`: identity and fallback of each ActiveX control, which has no node of its
    /// own in the shape tree. Never instantiated.
    pub controls: Vec<crate::retained::RetainedObject>,
}

/// `p:CT_SlideLayoutIdListEntry`. An id out of range is kept and reported, as the
/// entry still names a real layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideLayoutId {
    /// `@id`, optional.
    pub id: Option<u32>,
    /// `@r:id`.
    pub rel_id: String,
}

/// `p:CT_SlideMasterTextStyles`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MasterTextStyles {
    /// `titleStyle`.
    pub title: Option<TextListStyle>,
    /// `bodyStyle`.
    pub body: Option<TextListStyle>,
    /// `otherStyle`.
    pub other: Option<TextListStyle>,
}

/// One slide-like part: `sld`, `sldLayout`, `sldMaster`, `notes`, `notesMaster` or
/// `handoutMaster`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlidePart {
    /// Identity: the part's ordinal, node 0.
    pub id: ObjectId,
    /// Which kind.
    pub kind: SlideKind,
    /// `cSld`, which every kind requires.
    pub common: Option<CommonSlideData>,
    /// `clrMap`: masters, the notes master and the handout master.
    pub color_map: Option<ColorMap>,
    /// `clrMapOvr`: slides, layouts and notes slides.
    pub color_map_override: Option<ColorMapOverride>,
    /// `@showMasterSp`: slides, layouts and notes slides.
    pub show_master_shapes: Option<bool>,
    /// `@showMasterPhAnim`: slides, layouts and notes slides.
    pub show_master_placeholder_animations: Option<bool>,
    /// `@show`: slides.
    pub show: Option<bool>,
    /// `@matchingName`: layouts.
    pub matching_name: Option<String>,
    /// `@type`: layouts.
    pub layout_type: Option<SlideLayoutType>,
    /// `@preserve`: layouts and masters.
    pub preserve: Option<bool>,
    /// `@userDrawn`: layouts.
    pub user_drawn: Option<bool>,
    /// `hf`: layouts and every master.
    pub header_footer: Option<HeaderFooter>,
    /// `sldLayoutIdLst`: masters.
    // spec-audit: p:CT_SlideLayoutIdList/sldLayoutId
    pub layout_ids: Option<Vec<SlideLayoutId>>,
    /// `txStyles`: masters.
    pub text_styles: Option<MasterTextStyles>,
    /// `notesStyle`: the notes master.
    pub notes_style: Option<TextListStyle>,
    /// `transition`, on a slide, layout or master. Tier 1I.
    pub transition: Option<Slot<SlideTransition>>,
    /// `timing`, on a slide, layout or master. Tier 1I.
    pub timing: Option<Slot<SlideTiming>>,
}

impl SlidePart {
    /// A part with nothing stated.
    pub fn new(id: ObjectId, kind: SlideKind) -> Self {
        Self {
            id,
            kind,
            common: None,
            color_map: None,
            color_map_override: None,
            show_master_shapes: None,
            show_master_placeholder_animations: None,
            show: None,
            matching_name: None,
            layout_type: None,
            preserve: None,
            user_drawn: None,
            header_footer: None,
            layout_ids: None,
            text_styles: None,
            notes_style: None,
            transition: None,
            timing: None,
        }
    }

    /// The shape tree's root, if there is one.
    pub fn shape_tree(&self) -> Option<&ShapeNode> {
        self.common.as_ref()?.shape_tree.as_ref()
    }

    /// The shape tree's root, to change.
    pub fn shape_tree_mut(&mut self) -> Option<&mut ShapeNode> {
        self.common.as_mut()?.shape_tree.as_mut()
    }

    /// The node with this id, anywhere in the shape tree.
    pub fn node(&self, id: ObjectId) -> Option<&ShapeNode> {
        self.shape_tree()?.find(id)
    }

    /// The node with this id, to change.
    pub fn node_mut(&mut self, id: ObjectId) -> Option<&mut ShapeNode> {
        self.shape_tree_mut()?.find_mut(id)
    }
}

/// One slide-like part of a deck, located from relationships alone, before any of
/// it is read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartEntry {
    /// The part's id; `id.part` is its place in [`DeckStructure::parts`].
    pub id: ObjectId,
    /// Which kind, from the relationship that reached it.
    pub kind: SlideKind,
    /// The part's name, as the archive spells it.
    pub part: String,
    /// A slide's layout, by place in `parts`.
    pub layout: Option<u32>,
    /// A layout's master, or a notes slide's notes master, by place in `parts`.
    pub master: Option<u32>,
    /// A slide's notes slide, by place in `parts`.
    pub notes: Option<u32>,
    /// A master's theme part, as the archive spells it.
    pub theme: Option<String>,
}

/// The deck's slide-like parts and how they chain. Built from relationships, so a
/// 500-slide deck is indexed without reading a slide.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeckStructure {
    /// Every part, once each, by ordinal.
    pub parts: Vec<PartEntry>,
    /// Masters, in `sldMasterIdLst` order.
    pub masters: Vec<u32>,
    /// Layouts: each master's in relationship order, then any reached only from a slide.
    pub layouts: Vec<u32>,
    /// Slides, in `sldIdLst` order. A part listed twice appears twice.
    pub slides: Vec<u32>,
    /// Notes slides, in the order their slides are.
    pub notes: Vec<u32>,
    /// The notes master.
    pub notes_master: Option<u32>,
    /// The handout master.
    pub handout_master: Option<u32>,
}

impl DeckStructure {
    /// A part by ordinal.
    pub fn entry(&self, ordinal: u32) -> Option<&PartEntry> {
        self.parts.get(usize::try_from(ordinal).ok()?)
    }

    /// A slide's ordinal by its place in the deck.
    pub fn slide(&self, index: usize) -> Option<u32> {
        self.slides.get(index).copied()
    }
}

/// Every slide-like part of a deck, read. Held by ordinal, as in [`DeckStructure`];
/// `None` for a part that was not read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SlideParts {
    /// Where everything is.
    pub structure: DeckStructure,
    /// By ordinal.
    pub parts: Vec<Option<SlidePart>>,
}

impl SlideParts {
    /// A part by ordinal.
    pub fn get(&self, ordinal: u32) -> Option<&SlidePart> {
        self.parts.get(usize::try_from(ordinal).ok()?)?.as_ref()
    }

    /// A part by ordinal, to change.
    pub fn get_mut(&mut self, ordinal: u32) -> Option<&mut SlidePart> {
        self.parts.get_mut(usize::try_from(ordinal).ok()?)?.as_mut()
    }

    /// The slide at a place in the deck.
    pub fn slide(&self, index: usize) -> Option<&SlidePart> {
        self.get(self.structure.slide(index)?)
    }

    /// A slide's layout and that layout's master, as far as each is there; for a
    /// notes slide, no layout and its notes master.
    pub fn chain(&self, ordinal: u32) -> (Option<&SlidePart>, Option<&SlidePart>) {
        let entry = self.structure.entry(ordinal);
        if let Some(e) = entry.filter(|e| e.kind == SlideKind::Notes) {
            return (None, e.master.and_then(|o| self.get(o)));
        }
        let layout_ordinal = entry.and_then(|e| e.layout);
        let layout = layout_ordinal.and_then(|o| self.get(o));
        let master = layout_ordinal
            .and_then(|o| self.structure.entry(o))
            .and_then(|l| l.master)
            .and_then(|o| self.get(o));
        (layout, master)
    }
}

#[cfg(test)]
mod tests {
    use super::SlideKind;

    /// The import's cycle guard (`SEC-X07`) rests on this: a part's content type
    /// names exactly one kind.
    #[test]
    fn every_slide_kind_has_its_own_content_type() {
        let kinds = [
            SlideKind::Master,
            SlideKind::Layout,
            SlideKind::Slide,
            SlideKind::Notes,
            SlideKind::NotesMaster,
            SlideKind::HandoutMaster,
        ];
        let types: std::collections::BTreeSet<_> = kinds.iter().map(|k| k.content_type()).collect();
        assert_eq!(types.len(), kinds.len());
    }
}
