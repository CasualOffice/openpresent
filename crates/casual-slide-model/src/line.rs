//! Lines as plain typed values (`docs/spec/LINE.md`): `a:CT_LineProperties`, and the
//! Office 2018 sketch extension in its `extLst`.

use crate::color::Percentage;
use crate::fill::Fill;
use crate::geometry::Geometry;
use crate::properties::token_enum;

/// The largest `ST_LineWidth`, in EMU.
pub const LINE_WIDTH_LIMIT: i32 = 20_116_800;

token_enum!(
    /// `a:ST_LineEndType`.
    LineEndType {
        None = "none",
        Triangle = "triangle",
        Stealth = "stealth",
        Diamond = "diamond",
        Oval = "oval",
        Arrow = "arrow",
    }
);

token_enum!(
    /// `a:ST_LineEndWidth`.
    LineEndWidth {
        Sm = "sm",
        Med = "med",
        Lg = "lg",
    }
);

token_enum!(
    /// `a:ST_LineEndLength`.
    LineEndLength {
        Sm = "sm",
        Med = "med",
        Lg = "lg",
    }
);

token_enum!(
    /// `a:ST_PresetLineDashVal`.
    PresetLineDash {
        Solid = "solid",
        Dot = "dot",
        Dash = "dash",
        LgDash = "lgDash",
        DashDot = "dashDot",
        LgDashDot = "lgDashDot",
        LgDashDotDot = "lgDashDotDot",
        SysDash = "sysDash",
        SysDot = "sysDot",
        SysDashDot = "sysDashDot",
        SysDashDotDot = "sysDashDotDot",
    }
);

token_enum!(
    /// `a:ST_LineCap`.
    LineCap {
        Rnd = "rnd",
        Sq = "sq",
        Flat = "flat",
    }
);

token_enum!(
    /// `a:ST_PenAlignment`.
    PenAlignment {
        Ctr = "ctr",
        In = "in",
    }
);

token_enum!(
    /// `a:ST_CompoundLine`.
    CompoundLine {
        Sng = "sng",
        Dbl = "dbl",
        ThickThin = "thickThin",
        ThinThick = "thinThick",
        Tri = "tri",
    }
);

/// `a:CT_LineEndProperties`: `headEnd` and `tailEnd`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LineEnd {
    /// `@type`.
    pub end_type: Option<LineEndType>,
    /// `@w`.
    pub width: Option<LineEndWidth>,
    /// `@len`.
    pub length: Option<LineEndLength>,
}

/// `a:CT_DashStop`. `MS-OI29500` 2.1.1292: Office wants each above zero, which the
/// schema does not, so a zero is kept and reported.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DashStop {
    /// `@d`.
    pub dash: Option<Percentage>,
    /// `@sp`.
    pub space: Option<Percentage>,
}

/// `EG_LineDashProperties`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineDash {
    /// `prstDash/@val`.
    Preset(Option<PresetLineDash>),
    /// `custDash/ds`.
    Custom(Vec<DashStop>),
}

/// `EG_LineJoinProperties`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    /// `round`.
    Round,
    /// `bevel`.
    Bevel,
    /// `miter/@lim`.
    Miter(Option<Percentage>),
}

/// `ask:EG_LineSketchType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineSketchType {
    /// `lineSketchNone`.
    None,
    /// `lineSketchCurved`.
    Curved,
    /// `lineSketchFreehand`.
    Freehand,
    /// `lineSketchScribble`.
    Scribble,
}

/// `ask:CT_LineSketchStyleProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineSketch {
    /// `@sd`.
    pub seed_attribute: Option<u32>,
    /// `EG_Geometry`: the outline the sketch follows.
    pub geometry: Option<Geometry>,
    /// `ask:type`, and its choice if it has one.
    pub sketch_type: Option<Option<LineSketchType>>,
    /// `ask:seed`.
    pub seed: Option<u32>,
}

/// `a:CT_LineProperties`. A line with nothing stated is `Some` of the default:
/// `<a:ln/>` and no `a:ln` differ, as one overrides inheritance.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Line {
    /// `@w`, EMU.
    pub width: Option<i32>,
    /// `@cap`.
    pub cap: Option<LineCap>,
    /// `@cmpd`.
    pub compound: Option<CompoundLine>,
    /// `@algn`.
    pub align: Option<PenAlignment>,
    /// `EG_LineFillProperties`.
    pub fill: Option<Fill>,
    /// `prstDash` or `custDash`.
    pub dash: Option<LineDash>,
    /// `round`, `bevel` or `miter`.
    pub join: Option<LineJoin>,
    /// `headEnd`.
    pub head_end: Option<LineEnd>,
    /// `tailEnd`.
    pub tail_end: Option<LineEnd>,
    /// `ask:lineSketchStyleProps`, from `extLst`.
    pub sketch: Option<LineSketch>,
}
