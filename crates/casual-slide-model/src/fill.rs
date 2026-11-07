//! Fills as plain typed values (`docs/spec/FILL.md`): `EG_FillProperties`.
//!
//! A fill is a value here, not XML. The relationship a picture fill names is an id:
//! nothing is decoded or fetched. `MS-OI29500` notes that bear on what a renderer does
//! with these (2.1.1296 `fillToRect` is relative to the shape's box, 2.1.1297 Office
//! ignores `gradFill/@flip` and tiles with `xy`, 2.1.1298 `grpFill` in text is
//! `noFill`) change no stored value: the file's value is what is held.

use crate::color::{Angle, Color, Percentage};
use crate::effect::{Effect, ImageProperties};
use crate::picture::{ImageFormula, OEmbed, SvgBlip};
use crate::presentation::Coordinate;
use crate::properties::token_enum;

token_enum!(
    /// `a:ST_PathShadeType`.
    PathShadeType {
        Shape = "shape",
        Circle = "circle",
        Rect = "rect",
    }
);

token_enum!(
    /// `a:ST_TileFlipMode`.
    TileFlipMode {
        None = "none",
        X = "x",
        Y = "y",
        Xy = "xy",
    }
);

token_enum!(
    /// `a:ST_PresetPatternVal`.
    PresetPattern {
        Pct5 = "pct5",
        Pct10 = "pct10",
        Pct20 = "pct20",
        Pct25 = "pct25",
        Pct30 = "pct30",
        Pct40 = "pct40",
        Pct50 = "pct50",
        Pct60 = "pct60",
        Pct70 = "pct70",
        Pct75 = "pct75",
        Pct80 = "pct80",
        Pct90 = "pct90",
        Horz = "horz",
        Vert = "vert",
        LtHorz = "ltHorz",
        LtVert = "ltVert",
        DkHorz = "dkHorz",
        DkVert = "dkVert",
        NarHorz = "narHorz",
        NarVert = "narVert",
        DashHorz = "dashHorz",
        DashVert = "dashVert",
        Cross = "cross",
        DnDiag = "dnDiag",
        UpDiag = "upDiag",
        LtDnDiag = "ltDnDiag",
        LtUpDiag = "ltUpDiag",
        DkDnDiag = "dkDnDiag",
        DkUpDiag = "dkUpDiag",
        WdDnDiag = "wdDnDiag",
        WdUpDiag = "wdUpDiag",
        DashDnDiag = "dashDnDiag",
        DashUpDiag = "dashUpDiag",
        DiagCross = "diagCross",
        SmCheck = "smCheck",
        LgCheck = "lgCheck",
        SmGrid = "smGrid",
        LgGrid = "lgGrid",
        DotGrid = "dotGrid",
        SmConfetti = "smConfetti",
        LgConfetti = "lgConfetti",
        HorzBrick = "horzBrick",
        DiagBrick = "diagBrick",
        SolidDmnd = "solidDmnd",
        OpenDmnd = "openDmnd",
        DotDmnd = "dotDmnd",
        Plaid = "plaid",
        Sphere = "sphere",
        Weave = "weave",
        Divot = "divot",
        Shingle = "shingle",
        Wave = "wave",
        Trellis = "trellis",
        ZigZag = "zigZag",
    }
);

token_enum!(
    /// `a:ST_RectAlignment`.
    RectAlignment {
        Tl = "tl",
        T = "t",
        Tr = "tr",
        L = "l",
        Ctr = "ctr",
        R = "r",
        Bl = "bl",
        B = "b",
        Br = "br",
    }
);

token_enum!(
    /// `a:ST_BlipCompression`.
    BlipCompression {
        Email = "email",
        Screen = "screen",
        Print = "print",
        Hqprint = "hqprint",
        None = "none",
    }
);

/// `a:CT_RelativeRect`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelativeRect {
    /// `@l`.
    pub l: Option<Percentage>,
    /// `@t`.
    pub t: Option<Percentage>,
    /// `@r`.
    pub r: Option<Percentage>,
    /// `@b`.
    pub b: Option<Percentage>,
}

/// `a:CT_GradientStop`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GradientStop {
    /// `@pos`.
    pub position: Option<Percentage>,
    /// The colour.
    pub color: Option<Color>,
}

/// `EG_ShadeProperties`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientShade {
    /// `lin`.
    Linear {
        /// `@ang`.
        angle: Option<Angle>,
        /// `@scaled`.
        scaled: Option<bool>,
    },
    /// `path`.
    Path {
        /// `@path`.
        path: Option<PathShadeType>,
        /// `fillToRect`.
        fill_to_rect: Option<RelativeRect>,
    },
}

/// `a:CT_GradientFillProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GradientFill {
    /// `@flip`.
    pub flip: Option<TileFlipMode>,
    /// `@rotWithShape`.
    pub rotate_with_shape: Option<bool>,
    /// `gsLst/gs`. An absent list and an empty one differ.
    pub stops: Option<Vec<GradientStop>>,
    /// `lin` or `path`.
    pub shade: Option<GradientShade>,
    /// `tileRect`.
    pub tile_rect: Option<RelativeRect>,
}

/// `a:CT_PatternFillProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PatternFill {
    /// `@prst`.
    pub preset: Option<PresetPattern>,
    /// `fgClr`.
    pub foreground: Option<Color>,
    /// `bgClr`.
    pub background: Option<Color>,
}

/// `a:CT_TileInfoProperties`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TileInfo {
    /// `@tx`.
    pub tx: Option<Coordinate>,
    /// `@ty`.
    pub ty: Option<Coordinate>,
    /// `@sx`.
    pub sx: Option<Percentage>,
    /// `@sy`.
    pub sy: Option<Percentage>,
    /// `@flip`.
    pub flip: Option<TileFlipMode>,
    /// `@algn`.
    pub align: Option<RectAlignment>,
}

/// `EG_FillModeProperties`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlipFillMode {
    /// `tile`.
    Tile(TileInfo),
    /// `stretch/fillRect`.
    Stretch(Option<RelativeRect>),
}

/// `a:CT_Blip`: a reference to a picture, and what is done to it. Nothing here says what
/// the picture holds; `extLst` entries other than those here stay in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Blip {
    /// `@r:embed`.
    pub embed: Option<String>,
    /// `@r:link`. Never followed.
    pub link: Option<String>,
    /// `@cstate`.
    pub cstate: Option<BlipCompression>,
    /// The blip effects, in document order.
    pub effects: Vec<Effect>,
    /// `a14:imgProps`, from `extLst`.
    // spec-audit: x:drawing/2010/main:<imgProps>
    pub image_properties: Option<ImageProperties>,
    /// `a14:useLocalDpi/@val`, from `extLst`.
    // spec-audit: x:drawing/2010/main:<useLocalDpi>
    pub use_local_dpi: Option<bool>,
    /// `asvg:svgBlip`, from `extLst`, of which this blip's `@r:embed` is the fallback.
    pub svg: Option<SvgBlip>,
    /// `aif:imageFormula`, from `extLst`.
    pub image_formula: Option<ImageFormula>,
    /// `woe:oembed`, from `extLst`.
    pub oembed: Option<OEmbed>,
}

/// `a:CT_BlipFillProperties`, as a shape's fill and as a picture's `p:blipFill`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlipFill {
    /// `@dpi`.
    pub dpi: Option<u32>,
    /// `@rotWithShape`.
    pub rotate_with_shape: Option<bool>,
    /// `blip`.
    pub blip: Option<Blip>,
    /// `srcRect`.
    pub source_rect: Option<RelativeRect>,
    /// `tile` or `stretch`.
    pub mode: Option<BlipFillMode>,
}

/// `EG_FillProperties`.
/// A line holds these too, as `EG_LineFillProperties`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fill {
    /// `noFill`.
    None,
    /// `solidFill`. `None` if it holds no colour that could be believed.
    Solid(Option<Color>),
    /// `gradFill`.
    Gradient(Box<GradientFill>),
    /// `blipFill`.
    Blip(Box<BlipFill>),
    /// `pattFill`.
    Pattern(Box<PatternFill>),
    /// `grpFill`.
    Group,
}
