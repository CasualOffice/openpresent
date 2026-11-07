//! The shape tree as plain typed values (`docs/spec/SHAPE.md`): identity,
//! transform, locks, non-visual properties, hyperlinks and actions, for `sp`,
//! `grpSp`, `cxnSp`, `pic`, `graphicFrame` and `contentPart`.
//!
//! Geometry, fills, lines, effects and 3D are in [`ShapeNode::properties`], and a
//! shape's text body in [`crate::text`]. What a node holds beyond those (table and
//! chart content) stays in the part's tree,
//! preserved but not modelled, and is named in the import report. `Option` means
//! the file did not state a value, or stated one that could not be used; a schema
//! default is never filled in here. Effective values come from `casual-slide-style`.
//!
//! # Identity
//!
//! Every node has an [`ObjectId`], independent of `cNvPr/@id` (the file's `spid`),
//! which is unique only per slide, may be duplicated by a damaged file, and may be
//! renumbered by PowerPoint. An id is the part's ordinal in the deck and the node's
//! place in a depth-first walk of the part's shape tree, so the same file always
//! gives the same ids, whatever order its slides are loaded in. Ids are stable for
//! a session: an op that adds a node gives it an unused one, and nothing renumbers
//! (`docs/22` §Identity). They are not written to the file.

use std::fmt;

use crate::presentation::{Point, PositiveSize};
use crate::properties::token_enum;
use crate::text::TextBody;

/// A stable internal id: the part's ordinal in the deck, and the node within it.
/// Node 0 is the part itself; 1 is its shape tree's root group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId {
    /// The part's ordinal in the deck's structure.
    pub part: u32,
    /// The node within the part.
    pub node: u32,
}

impl ObjectId {
    /// The id of a part itself.
    pub const fn part(part: u32) -> Self {
        Self { part, node: 0 }
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.part, self.node)
    }
}

token_enum!(
    /// `a:ST_BlackWhiteMode`. How a shape renders in black-and-white mode; it does
    /// not make the shape black and white (`MS-OI29500` 2.1.1118).
    BlackWhiteMode {
        Clr = "clr",
        Auto = "auto",
        Gray = "gray",
        LtGray = "ltGray",
        InvGray = "invGray",
        GrayWhite = "grayWhite",
        BlackGray = "blackGray",
        BlackWhite = "blackWhite",
        Black = "black",
        White = "white",
        Hidden = "hidden",
    }
);

token_enum!(
    /// `p:ST_PlaceholderType`.
    PlaceholderType {
        Title = "title",
        Body = "body",
        CtrTitle = "ctrTitle",
        SubTitle = "subTitle",
        Dt = "dt",
        SldNum = "sldNum",
        Ftr = "ftr",
        Hdr = "hdr",
        Obj = "obj",
        Chart = "chart",
        Tbl = "tbl",
        ClipArt = "clipArt",
        Dgm = "dgm",
        Media = "media",
        SldImg = "sldImg",
        Pic = "pic",
    }
);

impl PlaceholderType {
    /// Every value, in schema order.
    pub const ALL: [Self; 16] = [
        Self::Title,
        Self::Body,
        Self::CtrTitle,
        Self::SubTitle,
        Self::Dt,
        Self::SldNum,
        Self::Ftr,
        Self::Hdr,
        Self::Obj,
        Self::Chart,
        Self::Tbl,
        Self::ClipArt,
        Self::Dgm,
        Self::Media,
        Self::SldImg,
        Self::Pic,
    ];
}

token_enum!(
    /// `p:ST_PlaceholderSize`.
    PlaceholderSize {
        Full = "full",
        Half = "half",
        Quarter = "quarter",
    }
);

/// `p:CT_Placeholder`. `extLst` stays in the tree.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Placeholder {
    /// `@type`. The schema default, `obj`, is not filled in.
    pub placeholder_type: Option<PlaceholderType>,
    /// `@orient`.
    pub orient: Option<crate::presentation::Direction>,
    /// `@sz`.
    pub size: Option<PlaceholderSize>,
    /// `@idx`. The schema default, 0, is not filled in.
    pub idx: Option<u32>,
    /// `@hasCustomPrompt`.
    pub has_custom_prompt: Option<bool>,
}

/// What a `ppaction://` action asks for (`MS-OI29500` 2.1.1395). Parsed and kept;
/// nothing here runs a program or a macro (`SEC-N04`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `ppaction://customshow?id=…`, with `&return=true` if present.
    CustomShow {
        /// The show's id, as written.
        id: Option<String>,
        /// Return to the calling slide afterwards.
        return_after: bool,
    },
    /// `ppaction://hlinkfile`: open the file `r:id` names.
    File,
    /// `ppaction://hlinkpres?slideindex=…`: another presentation.
    Presentation {
        /// The slide index, as written.
        slide_index: Option<String>,
    },
    /// `ppaction://hlinkshowjump?jump=…`.
    ShowJump(Option<String>),
    /// `ppaction://hlinksldjump`: the slide `r:id` names.
    SlideJump,
    /// `ppaction://macro?name=…`. Never run.
    Macro(Option<String>),
    /// `ppaction://program`. Never run.
    Program,
    /// `ppaction://noaction`.
    NoAction,
    /// `ppaction://media`.
    Media,
    /// `ppaction://ole?verb=…`. Never run.
    Ole(Option<String>),
    /// Another `ppaction://` verb, or another scheme. Kept as written.
    Other,
}

impl Action {
    /// Classify an `action` attribute. Case of the scheme and verb is ignored.
    pub fn parse(action: &str) -> Self {
        let lower = action.to_ascii_lowercase();
        let Some(rest) = lower.strip_prefix("ppaction://") else {
            return Self::Other;
        };
        let (verb, query) = rest.split_once('?').unwrap_or((rest, ""));
        // From the original text, so a value keeps its case.
        let original_query = action
            .get(action.len().saturating_sub(query.len())..)
            .unwrap_or_default();
        let param = |name: &str| {
            original_query.split('&').find_map(|pair| {
                let (k, v) = pair.split_once('=')?;
                k.eq_ignore_ascii_case(name).then(|| v.to_owned())
            })
        };
        match verb {
            "customshow" => Self::CustomShow {
                id: param("id"),
                return_after: param("return").is_some_and(|v| v.eq_ignore_ascii_case("true")),
            },
            "hlinkfile" => Self::File,
            "hlinkpres" => Self::Presentation {
                slide_index: param("slideindex"),
            },
            "hlinkshowjump" => Self::ShowJump(param("jump")),
            "hlinksldjump" => Self::SlideJump,
            "macro" => Self::Macro(param("name")),
            "program" => Self::Program,
            "noaction" => Self::NoAction,
            "media" => Self::Media,
            "ole" => Self::Ole(param("verb")),
            _ => Self::Other,
        }
    }

    /// Whether following it would run code: a program, a macro or an OLE verb.
    pub fn runs_code(&self) -> bool {
        matches!(self, Self::Program | Self::Macro(_) | Self::Ole(_))
    }
}

/// `a:CT_Hyperlink`, for `hlinkClick` and `hlinkHover`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hyperlink {
    /// `@r:id`. Empty is meaningful: an action with no target part.
    pub rel_id: Option<String>,
    /// `@invalidUrl`.
    pub invalid_url: Option<String>,
    /// `@action`, as written. [`Action::parse`] classifies it.
    pub action: Option<String>,
    /// `@tgtFrame`.
    pub target_frame: Option<String>,
    /// `@tooltip`.
    pub tooltip: Option<String>,
    /// `@history`.
    pub history: Option<bool>,
    /// `@highlightClick`.
    pub highlight_click: Option<bool>,
    /// `@endSnd`.
    pub end_sound: Option<bool>,
    /// `snd`.
    pub sound: Option<crate::media::EmbeddedWavAudioFile>,
}

/// `a:CT_NonVisualDrawingProps`, `cNvPr`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NonVisualDrawing {
    /// `@id`, the file's `spid`. Kept for writing back and for what timing and
    /// connectors reference; not this engine's identity.
    pub spid: Option<u32>,
    /// `@name`.
    pub name: Option<String>,
    /// `@descr`, alt text.
    pub descr: Option<String>,
    /// `@hidden`.
    pub hidden: Option<bool>,
    /// `@title`.
    pub title: Option<String>,
    /// `hlinkClick`.
    pub hyperlink_click: Option<Hyperlink>,
    /// `hlinkHover`.
    pub hyperlink_hover: Option<Hyperlink>,
    /// `extLst/ext/a16:creationId/@id`, PowerPoint's persistent id for a shape.
    // spec-audit: x:drawing/2014/main:<creationId>
    pub creation_id: Option<String>,
    /// `extLst/ext/adec:decorative/@val`.
    // spec-audit: x:drawing/2017/decorative:<decorative>
    pub decorative: Option<bool>,
}

/// Every locking attribute of `AG_Locking`, `CT_ShapeLocking`, `CT_GroupLocking`,
/// `CT_GraphicalObjectFrameLocking`, `CT_ConnectorLocking`,
/// `CT_ContentPartLocking` and `CT_PictureLocking`. Each kind's element allows a
/// subset; one it does not allow is preserved, not modelled.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Locks {
    /// `@noGrp`.
    pub no_grp: Option<bool>,
    /// `@noUngrp`, groups only.
    pub no_ungrp: Option<bool>,
    /// `@noSelect`.
    pub no_select: Option<bool>,
    /// `@noRot`.
    pub no_rot: Option<bool>,
    /// `@noChangeAspect`.
    pub no_change_aspect: Option<bool>,
    /// `@noMove`.
    pub no_move: Option<bool>,
    /// `@noResize`.
    pub no_resize: Option<bool>,
    /// `@noEditPoints`.
    pub no_edit_points: Option<bool>,
    /// `@noAdjustHandles`.
    pub no_adjust_handles: Option<bool>,
    /// `@noChangeArrowheads`.
    pub no_change_arrowheads: Option<bool>,
    /// `@noChangeShapeType`.
    pub no_change_shape_type: Option<bool>,
    /// `@noTextEdit`, shapes only.
    pub no_text_edit: Option<bool>,
    /// `@noDrilldown`, graphic frames only.
    pub no_drilldown: Option<bool>,
    /// `@noCrop`, pictures only.
    pub no_crop: Option<bool>,
}

impl Locks {
    /// Every locking attribute's name, as written.
    pub const NAMES: [&'static str; 14] = [
        "noGrp",
        "noUngrp",
        "noSelect",
        "noRot",
        "noChangeAspect",
        "noMove",
        "noResize",
        "noEditPoints",
        "noAdjustHandles",
        "noChangeArrowheads",
        "noChangeShapeType",
        "noTextEdit",
        "noDrilldown",
        "noCrop",
    ];

    /// An attribute's value, by its name as written.
    pub fn get(&self, name: &str) -> Option<bool> {
        let mut copy = *self;
        copy.slot(name).and_then(|v| *v)
    }

    /// An attribute's field, by its name as written.
    pub fn slot(&mut self, name: &str) -> Option<&mut Option<bool>> {
        Some(match name {
            "noGrp" => &mut self.no_grp,
            "noUngrp" => &mut self.no_ungrp,
            "noSelect" => &mut self.no_select,
            "noRot" => &mut self.no_rot,
            "noChangeAspect" => &mut self.no_change_aspect,
            "noMove" => &mut self.no_move,
            "noResize" => &mut self.no_resize,
            "noEditPoints" => &mut self.no_edit_points,
            "noAdjustHandles" => &mut self.no_adjust_handles,
            "noChangeArrowheads" => &mut self.no_change_arrowheads,
            "noChangeShapeType" => &mut self.no_change_shape_type,
            "noTextEdit" => &mut self.no_text_edit,
            "noDrilldown" => &mut self.no_drilldown,
            "noCrop" => &mut self.no_crop,
            _ => return None,
        })
    }
}

/// `AG_Locking`.
const AG_LOCKING: [&str; 10] = [
    "noGrp",
    "noSelect",
    "noRot",
    "noChangeAspect",
    "noMove",
    "noResize",
    "noEditPoints",
    "noAdjustHandles",
    "noChangeArrowheads",
    "noChangeShapeType",
];

/// `a:CT_Connection`, `stCxn` and `endCxn`: a connector's end, by `spid` and
/// connection site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connection {
    /// `@id`, a `spid` on the same slide.
    pub spid: u32,
    /// `@idx`, the connection site.
    pub site: u32,
}

/// Which `EG_Media` child `nvPr` holds. [`crate::media::Media`] holds what it says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    /// `a:audioCd`.
    AudioCd,
    /// `a:wavAudioFile`.
    WavAudioFile,
    /// `a:audioFile`.
    AudioFile,
    /// `a:videoFile`.
    VideoFile,
    /// `a:quickTimeFile`.
    QuickTimeFile,
}

/// `p:CT_ApplicationNonVisualDrawingProps`, `nvPr`. `custDataLst` and other `extLst`
/// entries stay in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppNonVisual {
    /// `ph`.
    pub placeholder: Option<Placeholder>,
    /// The `EG_Media` child, if any.
    pub media: Option<crate::media::Media>,
    /// `extLst/ext/p14:media`.
    pub media_extension: Option<Box<crate::media::MediaExtension>>,
    /// `@isPhoto`.
    pub is_photo: Option<bool>,
    /// `@userDrawn`.
    pub user_drawn: Option<bool>,
}

/// `a:CT_Transform2D` and `a:CT_GroupTransform2D`. The child fields are only read
/// for a group.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Transform {
    /// `off`.
    pub offset: Option<Point>,
    /// `ext`, EMU.
    pub extent: Option<PositiveSize>,
    /// `chOff`, groups only.
    pub child_offset: Option<Point>,
    /// `chExt`, groups only.
    pub child_extent: Option<PositiveSize>,
    /// `@rot`, 60000ths of a degree.
    pub rotation: Option<i32>,
    /// `@flipH`.
    pub flip_h: Option<bool>,
    /// `@flipV`.
    pub flip_v: Option<bool>,
}

/// Which element a node is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// `p:sp`.
    Shape,
    /// `p:grpSp`, and the slide's `spTree` itself.
    Group,
    /// `p:cxnSp`.
    Connector,
    /// `p:pic`.
    Picture,
    /// `p:graphicFrame`: a table, chart, diagram or OLE object.
    GraphicFrame,
    /// `p:contentPart`.
    ContentPart,
}

impl NodeKind {
    /// The non-visual container: `nvSpPr`, `nvGrpSpPr` and so on.
    pub fn non_visual_container(self) -> &'static str {
        match self {
            Self::Shape => "nvSpPr",
            Self::Group => "nvGrpSpPr",
            Self::Connector => "nvCxnSpPr",
            Self::Picture => "nvPicPr",
            Self::GraphicFrame => "nvGraphicFramePr",
            Self::ContentPart => "nvContentPartPr",
        }
    }

    /// The kind's own non-visual properties: `cNvSpPr`, `cNvGrpSpPr` and so on.
    pub fn non_visual_properties(self) -> &'static str {
        match self {
            Self::Shape => "cNvSpPr",
            Self::Group => "cNvGrpSpPr",
            Self::Connector => "cNvCxnSpPr",
            Self::Picture => "cNvPicPr",
            Self::GraphicFrame => "cNvGraphicFramePr",
            Self::ContentPart => "cNvContentPartPr",
        }
    }

    /// The locking element in [`Self::non_visual_properties`].
    pub fn lock_element(self) -> &'static str {
        match self {
            Self::Shape => "spLocks",
            Self::Group => "grpSpLocks",
            Self::Connector => "cxnSpLocks",
            Self::Picture => "picLocks",
            Self::GraphicFrame => "graphicFrameLocks",
            Self::ContentPart => "cpLocks",
        }
    }

    /// The locking attributes the schema allows on [`Self::lock_element`].
    pub fn lock_attributes(self) -> &'static [&'static str] {
        const SHAPE: [&str; 11] = [
            "noGrp",
            "noSelect",
            "noRot",
            "noChangeAspect",
            "noMove",
            "noResize",
            "noEditPoints",
            "noAdjustHandles",
            "noChangeArrowheads",
            "noChangeShapeType",
            "noTextEdit",
        ];
        const PICTURE: [&str; 11] = [
            "noGrp",
            "noSelect",
            "noRot",
            "noChangeAspect",
            "noMove",
            "noResize",
            "noEditPoints",
            "noAdjustHandles",
            "noChangeArrowheads",
            "noChangeShapeType",
            "noCrop",
        ];
        const GROUP: [&str; 7] = [
            "noGrp",
            "noUngrp",
            "noSelect",
            "noRot",
            "noChangeAspect",
            "noMove",
            "noResize",
        ];
        const FRAME: [&str; 6] = [
            "noGrp",
            "noDrilldown",
            "noSelect",
            "noChangeAspect",
            "noMove",
            "noResize",
        ];
        match self {
            Self::Shape => &SHAPE,
            Self::Picture => &PICTURE,
            Self::Group => &GROUP,
            Self::GraphicFrame => &FRAME,
            Self::Connector | Self::ContentPart => &AG_LOCKING,
        }
    }

    /// The PresentationML element name.
    pub fn element(self) -> &'static str {
        match self {
            Self::Shape => "sp",
            Self::Group => "grpSp",
            Self::Connector => "cxnSp",
            Self::Picture => "pic",
            Self::GraphicFrame => "graphicFrame",
            Self::ContentPart => "contentPart",
        }
    }
}

/// One node of a shape tree. Fields a kind does not have stay `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeNode {
    /// Identity.
    pub id: ObjectId,
    /// Which element.
    pub kind: NodeKind,
    /// `cNvPr`.
    // spec-audit: p:CT_ShapeNonVisual/cNvPr
    // spec-audit: p:CT_ConnectorNonVisual/cNvPr
    // spec-audit: p:CT_PictureNonVisual/cNvPr
    // spec-audit: p:CT_GraphicalObjectFrameNonVisual/cNvPr
    // spec-audit: p:CT_GroupShapeNonVisual/cNvPr
    pub non_visual: Option<NonVisualDrawing>,
    /// The kind's locking element in its `cNv…Pr`.
    // spec-audit: p:CT_ShapeNonVisual/cNvSpPr
    // spec-audit: p:CT_ConnectorNonVisual/cNvCxnSpPr
    // spec-audit: p:CT_PictureNonVisual/cNvPicPr
    // spec-audit: p:CT_GraphicalObjectFrameNonVisual/cNvGraphicFramePr
    // spec-audit: p:CT_GroupShapeNonVisual/cNvGrpSpPr
    pub locks: Option<Locks>,
    /// `cNvSpPr/@txBox`, shapes only.
    pub text_box: Option<bool>,
    /// `cNvCxnSpPr/stCxn`, connectors only.
    pub start_connection: Option<Connection>,
    /// `cNvCxnSpPr/endCxn`, connectors only.
    pub end_connection: Option<Connection>,
    /// `nvPr`.
    // spec-audit: p:CT_ShapeNonVisual/nvPr
    // spec-audit: p:CT_ConnectorNonVisual/nvPr
    // spec-audit: p:CT_PictureNonVisual/nvPr
    // spec-audit: p:CT_GraphicalObjectFrameNonVisual/nvPr
    // spec-audit: p:CT_GroupShapeNonVisual/nvPr
    pub app: Option<AppNonVisual>,
    /// `spPr/xfrm`, `grpSpPr/xfrm`, or a graphic frame's `p:xfrm`.
    // spec-audit: p:CT_GraphicalObjectFrame/xfrm
    pub transform: Option<Transform>,
    /// `spPr/@bwMode`, `grpSpPr/@bwMode`, or the graphic frame's own.
    // spec-audit: p:CT_GraphicalObjectFrame/@bwMode
    pub bw_mode: Option<BlackWhiteMode>,
    /// `@useBgFill`, shapes only.
    // spec-audit: p:CT_Shape/@useBgFill
    pub use_bg_fill: Option<bool>,
    /// `cNvPicPr/@preferRelativeResize`, pictures only.
    pub prefer_relative_resize: Option<bool>,
    /// Geometry, fill, line, effects and 3D: Tier 1E.
    // spec-audit: p:CT_Shape/spPr
    // spec-audit: p:CT_Connector/spPr
    // spec-audit: p:CT_Picture/spPr
    // spec-audit: p:CT_GroupShape/grpSpPr
    pub properties: crate::drawing::ShapeProperties,
    /// `a:graphic/a:graphicData/@uri`, graphic frames only: what the frame holds.
    // spec-audit: p:CT_GraphicalObjectFrame/a:graphic
    // spec-audit: a:CT_GraphicalObject/graphicData
    // spec-audit: a:<graphic>
    pub graphic_uri: Option<String>,
    /// `@r:id`, content parts only.
    // spec-audit: p:CT_Rel/@r:id
    pub rel_id: Option<String>,
    /// `txBody`, shapes only. A table cell's text is in [`Self::table`].
    // spec-audit: p:CT_Shape/txBody
    pub text_body: Option<Box<TextBody>>,
    /// `a:tbl` in the frame's `graphicData`, graphic frames only.
    pub table: Option<Box<crate::table::Table>>,
    /// `dgm:relIds` in the frame's `graphicData`: which parts its diagram is.
    pub diagram: Option<Box<crate::diagram::schema::RelIds>>,
    /// What a group's `mc:AlternateContent` holds in the branches not taken, where it is
    /// a content part's ink or an equation. The taken branch's nodes are [`Self::children`].
    pub alternates: Vec<AlternateBranch>,
    /// An OLE object or 3D model this node stands for: identity and fallback only.
    pub retained: Option<Box<crate::retained::RetainedObject>>,
    /// `p:style` on a shape, connector or picture: references into the theme.
    // spec-audit: p:CT_Shape/style
    // spec-audit: p:CT_Connector/style
    // spec-audit: p:CT_Picture/style
    pub style: Option<Box<crate::theme::ShapeStyle>>,
    /// A group's children, in document order, which is z-order.
    // spec-audit: p:CT_GroupShape/sp
    // spec-audit: p:CT_GroupShape/grpSp
    // spec-audit: p:CT_GroupShape/graphicFrame
    // spec-audit: p:CT_GroupShape/cxnSp
    // spec-audit: p:CT_GroupShape/pic
    // spec-audit: p:CT_GroupShape/contentPart
    pub children: Vec<ShapeNode>,
}

/// An `mc:Choice` a reader of this model did not take: PowerPoint's own form of an
/// equation or of ink, whose `mc:Fallback` is the shape in the group. It stays in the
/// tree as it was, and reading it is all that is done with it (`INV-FB-03`, `INV-FB-06`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AlternateBranch {
    /// `mc:Choice/@Requires`, as written.
    pub requires: Option<String>,
    /// `r:id` of each `p:contentPart` in the branch.
    pub content_parts: Vec<String>,
    /// Each `a14:m` in the text of a `p:sp` in the branch, in document order.
    pub equations: Vec<crate::math::TextMath>,
}

impl ShapeNode {
    /// An empty node of a kind, with nothing stated.
    pub fn new(id: ObjectId, kind: NodeKind) -> Self {
        Self {
            id,
            kind,
            non_visual: None,
            locks: None,
            text_box: None,
            start_connection: None,
            end_connection: None,
            app: None,
            transform: None,
            bw_mode: None,
            use_bg_fill: None,
            prefer_relative_resize: None,
            properties: crate::drawing::ShapeProperties::default(),
            graphic_uri: None,
            rel_id: None,
            text_body: None,
            table: None,
            diagram: None,
            alternates: Vec::new(),
            retained: None,
            style: None,
            children: Vec::new(),
        }
    }

    /// The node's placeholder, if it is one.
    pub fn placeholder(&self) -> Option<&Placeholder> {
        self.app.as_ref()?.placeholder.as_ref()
    }

    /// A media node's poster frame: the blip of the picture its `nvPr` sits on. `None`
    /// on anything else, a picture with no media included.
    pub fn poster(&self) -> Option<&crate::fill::Blip> {
        let app = self.app.as_ref()?;
        if app.media.is_none() && app.media_extension.is_none() {
            return None;
        }
        self.properties.picture.as_ref()?.blip.as_ref()
    }

    /// Every node in the subtree, this one first, depth first. Iterative.
    pub fn descendants(&self) -> Descendants<'_> {
        Descendants { stack: vec![self] }
    }

    /// The node with this id in the subtree.
    pub fn find(&self, id: ObjectId) -> Option<&ShapeNode> {
        self.descendants().find(|n| n.id == id)
    }

    /// The node with this id in the subtree, to change. Iterative.
    pub fn find_mut(&mut self, id: ObjectId) -> Option<&mut ShapeNode> {
        let mut stack = vec![self];
        while let Some(node) = stack.pop() {
            if node.id == id {
                return Some(node);
            }
            stack.extend(node.children.iter_mut().rev());
        }
        None
    }
}

/// Depth-first iteration over a subtree, in document order.
#[derive(Debug)]
pub struct Descendants<'a> {
    stack: Vec<&'a ShapeNode>,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = &'a ShapeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        self.stack.extend(node.children.iter().rev());
        Some(node)
    }
}

/// A shape tree's nodes are dropped without recursion, so a deep tree that was
/// admitted cannot exhaust the stack on the way out.
impl Drop for ShapeNode {
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.children);
        while let Some(mut node) = pending.pop() {
            pending.append(&mut node.children);
        }
    }
}
