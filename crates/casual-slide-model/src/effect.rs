//! Effects as plain typed values (`docs/spec/EFFECT.md`): effect lists, effect DAGs,
//! blip effects, and the `a14` picture effects.
//!
//! The one [`Effect`] serves all three places, as `EG_Effect` does in the schema. A
//! DAG is a tree of containers and named references to them, which is why a reference
//! holds a name and never a pointer: nothing follows one here, and a resolver that
//! does must carry its own visited set.
//!
//! Compositing order (BEH-17) belongs to the renderer. The list holds what the file
//! holds, in the file's order.

use crate::color::{Angle, Color, Percentage};
use crate::fill::{Fill, RectAlignment};
use crate::presentation::Coordinate;
use crate::properties::token_enum;

token_enum!(
    /// `a:ST_PresetShadowVal`.
    PresetShadow {
        Shdw1 = "shdw1",
        Shdw2 = "shdw2",
        Shdw3 = "shdw3",
        Shdw4 = "shdw4",
        Shdw5 = "shdw5",
        Shdw6 = "shdw6",
        Shdw7 = "shdw7",
        Shdw8 = "shdw8",
        Shdw9 = "shdw9",
        Shdw10 = "shdw10",
        Shdw11 = "shdw11",
        Shdw12 = "shdw12",
        Shdw13 = "shdw13",
        Shdw14 = "shdw14",
        Shdw15 = "shdw15",
        Shdw16 = "shdw16",
        Shdw17 = "shdw17",
        Shdw18 = "shdw18",
        Shdw19 = "shdw19",
        Shdw20 = "shdw20",
    }
);

token_enum!(
    /// `a:ST_BlendMode`.
    BlendMode {
        Over = "over",
        Mult = "mult",
        Screen = "screen",
        Darken = "darken",
        Lighten = "lighten",
    }
);

token_enum!(
    /// `a:ST_EffectContainerType`.
    EffectContainerType {
        Sib = "sib",
        Tree = "tree",
    }
);

/// `a:CT_EffectContainer`: `cont`, and `effectDag`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectContainer {
    /// `@type`.
    pub container_type: Option<EffectContainerType>,
    /// `@name`. `MS-OI29500` 2.1.1291: unique throughout the top-level container.
    pub name: Option<String>,
    /// The effects, in order.
    pub effects: Vec<Effect>,
}

/// `a:CT_OuterShadowEffect`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OuterShadow {
    /// `@blurRad`.
    pub blur_radius: Option<Coordinate>,
    /// `@dist`.
    pub distance: Option<Coordinate>,
    /// `@dir`.
    pub direction: Option<Angle>,
    /// `@sx`. `MS-OI29500` 2.1.1303: Office draws other than 100% only in a group's.
    pub sx: Option<Percentage>,
    /// `@sy`.
    pub sy: Option<Percentage>,
    /// `@kx`.
    pub kx: Option<Angle>,
    /// `@ky`.
    pub ky: Option<Angle>,
    /// `@algn`.
    pub align: Option<RectAlignment>,
    /// `@rotWithShape`.
    pub rotate_with_shape: Option<bool>,
    /// The colour.
    pub color: Option<Color>,
}

/// `a:CT_ReflectionEffect`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Reflection {
    /// `@blurRad`.
    pub blur_radius: Option<Coordinate>,
    /// `@stA`.
    pub start_alpha: Option<Percentage>,
    /// `@stPos`.
    pub start_position: Option<Percentage>,
    /// `@endA`.
    pub end_alpha: Option<Percentage>,
    /// `@endPos`.
    pub end_position: Option<Percentage>,
    /// `@dist`.
    pub distance: Option<Coordinate>,
    /// `@dir`. `MS-OI29500` 2.1.1304: in Office this offsets the reflection.
    pub direction: Option<Angle>,
    /// `@fadeDir`. In Office, the direction of the alpha ramp.
    pub fade_direction: Option<Angle>,
    /// `@sx`.
    pub sx: Option<Percentage>,
    /// `@sy`.
    pub sy: Option<Percentage>,
    /// `@kx`.
    pub kx: Option<Angle>,
    /// `@ky`.
    pub ky: Option<Angle>,
    /// `@algn`.
    pub align: Option<RectAlignment>,
    /// `@rotWithShape`.
    pub rotate_with_shape: Option<bool>,
}

/// `a:CT_TransformEffect`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TransformEffect {
    /// `@sx`.
    pub sx: Option<Percentage>,
    /// `@sy`.
    pub sy: Option<Percentage>,
    /// `@kx`.
    pub kx: Option<Angle>,
    /// `@ky`.
    pub ky: Option<Angle>,
    /// `@tx`.
    pub tx: Option<Coordinate>,
    /// `@ty`.
    pub ty: Option<Coordinate>,
}

/// `EG_Effect`, and each child of `CT_EffectList` and `CT_Blip`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// `cont`.
    Container(EffectContainer),
    /// `effect`, with its `@ref`: a container's name, or a special reference.
    Reference(Option<String>),
    /// `alphaBiLevel/@thresh`.
    AlphaBiLevel(Option<Percentage>),
    /// `alphaCeiling`.
    AlphaCeiling,
    /// `alphaFloor`.
    AlphaFloor,
    /// `alphaInv`.
    AlphaInverse(Option<Color>),
    /// `alphaMod/cont`.
    AlphaModulate(Option<EffectContainer>),
    /// `alphaModFix/@amt`.
    AlphaModulateFixed(Option<Percentage>),
    /// `alphaOutset/@rad`.
    AlphaOutset(Option<Coordinate>),
    /// `alphaRepl/@a`.
    AlphaReplace(Option<Percentage>),
    /// `biLevel/@thresh`.
    BiLevel(Option<Percentage>),
    /// `blend`.
    Blend {
        /// `@blend`.
        blend: Option<BlendMode>,
        /// `cont`.
        container: Option<EffectContainer>,
    },
    /// `blur`.
    Blur {
        /// `@rad`.
        radius: Option<Coordinate>,
        /// `@grow`.
        grow: Option<bool>,
    },
    /// `clrChange`.
    ColorChange {
        /// `@useA`.
        use_alpha: Option<bool>,
        /// `clrFrom`.
        from: Option<Color>,
        /// `clrTo`.
        to: Option<Color>,
    },
    /// `clrRepl`.
    ColorReplace(Option<Color>),
    /// `duotone`: its colours, of which the schema wants two.
    Duotone(Vec<Color>),
    /// `fill`.
    Fill(Option<Fill>),
    /// `fillOverlay`.
    FillOverlay {
        /// `@blend`.
        blend: Option<BlendMode>,
        /// The fill.
        fill: Option<Fill>,
    },
    /// `glow`.
    Glow {
        /// `@rad`.
        radius: Option<Coordinate>,
        /// The colour.
        color: Option<Color>,
    },
    /// `grayscl`.
    Grayscale,
    /// `hsl`. `MS-OI29500` 2.1.1299: Office adjusts by these amounts.
    Hsl {
        /// `@hue`.
        hue: Option<Angle>,
        /// `@sat`.
        sat: Option<Percentage>,
        /// `@lum`.
        lum: Option<Percentage>,
    },
    /// `innerShdw`.
    InnerShadow {
        /// `@blurRad`.
        blur_radius: Option<Coordinate>,
        /// `@dist`.
        distance: Option<Coordinate>,
        /// `@dir`. `MS-OI29500` 2.1.1300: clockwise from the left.
        direction: Option<Angle>,
        /// The colour.
        color: Option<Color>,
    },
    /// `lum`.
    Luminance {
        /// `@bright`.
        bright: Option<Percentage>,
        /// `@contrast`.
        contrast: Option<Percentage>,
    },
    /// `outerShdw`.
    OuterShadow(Box<OuterShadow>),
    /// `prstShdw`.
    PresetShadow {
        /// `@prst`.
        preset: Option<PresetShadow>,
        /// `@dist`.
        distance: Option<Coordinate>,
        /// `@dir`.
        direction: Option<Angle>,
        /// The colour.
        color: Option<Color>,
    },
    /// `reflection`.
    Reflection(Box<Reflection>),
    /// `relOff`.
    RelativeOffset {
        /// `@tx`.
        tx: Option<Percentage>,
        /// `@ty`.
        ty: Option<Percentage>,
    },
    /// `softEdge/@rad`.
    SoftEdge(Option<Coordinate>),
    /// `tint`.
    Tint {
        /// `@hue`.
        hue: Option<Angle>,
        /// `@amt`.
        amount: Option<Percentage>,
    },
    /// `xfrm`.
    Transform(Box<TransformEffect>),
}

/// `EG_EffectProperties`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectProperties {
    /// `effectLst`, children in document order. Empty means no effect, overriding
    /// what would be inherited, which absence does not.
    List(Vec<Effect>),
    /// `effectDag`.
    Dag(EffectContainer),
}

token_enum!(
    /// The `a14:CT_PictureEffect` choices with the common shape: `@trans` and one
    /// parameter, as [`ArtisticEffect::parameter`] names it.
    ArtisticEffect {
        Blur = "artisticBlur",
        Cement = "artisticCement",
        ChalkSketch = "artisticChalkSketch",
        CrisscrossEtching = "artisticCrisscrossEtching",
        Cutout = "artisticCutout",
        FilmGrain = "artisticFilmGrain",
        Glass = "artisticGlass",
        GlowDiffused = "artisticGlowDiffused",
        GlowEdges = "artisticGlowEdges",
        LightScreen = "artisticLightScreen",
        LineDrawing = "artisticLineDrawing",
        Marker = "artisticMarker",
        MosiaicBubbles = "artisticMosiaicBubbles",
        PaintStrokes = "artisticPaintStrokes",
        PaintBrush = "artisticPaintBrush",
        PastelsSmooth = "artisticPastelsSmooth",
        PencilGrayscale = "artisticPencilGrayscale",
        PencilSketch = "artisticPencilSketch",
        Photocopy = "artisticPhotocopy",
        PlasticWrap = "artisticPlasticWrap",
        Texturizer = "artisticTexturizer",
        WatercolorSponge = "artisticWatercolorSponge",
    }
);

impl ArtisticEffect {
    /// The parameter's attribute, and its largest value (`ST_ArtisticEffectParam*`).
    pub fn parameter(self) -> (&'static str, i32) {
        match self {
            Self::Blur => ("radius", 100),
            Self::Cement => ("crackSpacing", 100),
            Self::ChalkSketch => ("pressure", 4),
            Self::CrisscrossEtching | Self::MosiaicBubbles | Self::PencilSketch => {
                ("pressure", 100)
            }
            Self::Cutout => ("numberOfShades", 6),
            Self::FilmGrain => ("grainSize", 100),
            Self::Glass | Self::PastelsSmooth | Self::Texturizer => ("scaling", 100),
            Self::GlowDiffused | Self::PaintStrokes => ("intensity", 10),
            Self::GlowEdges | Self::PlasticWrap => ("smoothness", 10),
            Self::LightScreen => ("gridSize", 10),
            Self::LineDrawing | Self::PencilGrayscale => ("pencilSize", 100),
            Self::Marker => ("size", 100),
            Self::PaintBrush | Self::WatercolorSponge => ("brushSize", 10),
            Self::Photocopy => ("detail", 10),
        }
    }

    /// Whether the effect has `@trans`: all but `artisticBlur`.
    pub fn has_transparency(self) -> bool {
        self != Self::Blur
    }
}

/// `a14:CT_PictureEffectBackgroundRemovalForegroundMark`, and its twin
/// `a14:CT_PictureEffectBackgroundRemovalBackgroundMark`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RemovalMark {
    /// `@x1`.
    pub x1: Option<Percentage>,
    /// `@y1`.
    pub y1: Option<Percentage>,
    /// `@x2`.
    pub x2: Option<Percentage>,
    /// `@y2`.
    pub y2: Option<Percentage>,
}

/// `a14:CT_PictureEffectBackgroundRemoval`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BackgroundRemoval {
    /// `@t`.
    pub t: Option<Percentage>,
    /// `@b`.
    pub b: Option<Percentage>,
    /// `@l`.
    pub l: Option<Percentage>,
    /// `@r`.
    pub r: Option<Percentage>,
    /// `foregroundMark`.
    pub foreground: Vec<RemovalMark>,
    /// `backgroundMark`.
    pub background: Vec<RemovalMark>,
}

/// One choice of `a14:CT_PictureEffect`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PictureEffectKind {
    // `effect` names the element; `transparency` and `parameter` hold its attributes.
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticBlur
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticCement
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticChalkSketch
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticCrisscrossEtching
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticCutout
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticFilmGrain
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticGlass
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticGlowDiffused
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticGlowEdges
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticLightScreen
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticLineDrawing
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticMarker
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticMosiaicBubbles
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticPaintStrokes
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticPaintBrush
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticPastelsSmooth
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticPencilGrayscale
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticPencilSketch
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticPhotocopy
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticPlasticWrap
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticTexturizer
    // spec-audit: x:drawing/2010/main:CT_PictureEffect/artisticWatercolorSponge
    // spec-audit: x:drawing/2010/main:CT_PictureEffectBlur/@radius
    // spec-audit: x:drawing/2010/main:CT_PictureEffectCement/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectCement/@crackSpacing
    // spec-audit: x:drawing/2010/main:CT_PictureEffectChalkSketch/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectChalkSketch/@pressure
    // spec-audit: x:drawing/2010/main:CT_PictureEffectCrisscrossEtching/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectCrisscrossEtching/@pressure
    // spec-audit: x:drawing/2010/main:CT_PictureEffectCutout/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectCutout/@numberOfShades
    // spec-audit: x:drawing/2010/main:CT_PictureEffectFilmGrain/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectFilmGrain/@grainSize
    // spec-audit: x:drawing/2010/main:CT_PictureEffectGlass/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectGlass/@scaling
    // spec-audit: x:drawing/2010/main:CT_PictureEffectGlowDiffused/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectGlowDiffused/@intensity
    // spec-audit: x:drawing/2010/main:CT_PictureEffectGlowEdges/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectGlowEdges/@smoothness
    // spec-audit: x:drawing/2010/main:CT_PictureEffectLightScreen/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectLightScreen/@gridSize
    // spec-audit: x:drawing/2010/main:CT_PictureEffectLineDrawing/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectLineDrawing/@pencilSize
    // spec-audit: x:drawing/2010/main:CT_PictureEffectMarker/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectMarker/@size
    // spec-audit: x:drawing/2010/main:CT_PictureEffectMosiaicBubbles/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectMosiaicBubbles/@pressure
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPaintBrush/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPaintBrush/@brushSize
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPaintStrokes/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPaintStrokes/@intensity
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPastelsSmooth/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPastelsSmooth/@scaling
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPencilGrayscale/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPencilGrayscale/@pencilSize
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPencilSketch/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPencilSketch/@pressure
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPhotocopy/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPhotocopy/@detail
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPlasticWrap/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectPlasticWrap/@smoothness
    // spec-audit: x:drawing/2010/main:CT_PictureEffectTexturizer/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectTexturizer/@scaling
    // spec-audit: x:drawing/2010/main:CT_PictureEffectWatercolorSponge/@trans
    // spec-audit: x:drawing/2010/main:CT_PictureEffectWatercolorSponge/@brushSize
    // spec-audit: x:drawing/2010/main:CT_PictureEffectBrightnessContrast/@bright
    // spec-audit: x:drawing/2010/main:CT_PictureEffectBrightnessContrast/@contrast
    // spec-audit: x:drawing/2010/main:CT_PictureEffectColorTemperature/@colorTemp
    // spec-audit: x:drawing/2010/main:CT_PictureEffectSaturation/@sat
    // spec-audit: x:drawing/2010/main:CT_PictureEffectSharpenSoften/@amount
    /// One of the artistic effects.
    Artistic {
        /// Which.
        effect: ArtisticEffect,
        /// `@trans`.
        transparency: Option<Percentage>,
        /// The one parameter, an integer.
        parameter: Option<i32>,
    },
    /// `backgroundRemoval`.
    BackgroundRemoval(BackgroundRemoval),
    /// `brightnessContrast`.
    BrightnessContrast {
        /// `@bright`.
        bright: Option<Percentage>,
        /// `@contrast`.
        contrast: Option<Percentage>,
    },
    /// `colorTemperature/@colorTemp`.
    ColorTemperature(Option<i32>),
    /// `saturation/@sat`.
    Saturation(Option<Percentage>),
    /// `sharpenSoften/@amount`.
    SharpenSoften(Option<Percentage>),
}

/// `a14:CT_PictureEffect`: `imgEffect`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PictureEffect {
    /// `@visible`.
    pub visible: Option<bool>,
    /// The choice. `None` if it holds none this reader knows; the tree keeps it.
    pub kind: Option<PictureEffectKind>,
}

/// `a14:CT_PictureLayer`: `imgLayer`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageLayer {
    /// `@r:embed`: the original the effects are applied to.
    pub embed: Option<String>,
    /// `imgEffect`, in order.
    pub effects: Vec<PictureEffect>,
}

/// `a14:CT_Photo`: `imgProps`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageProperties {
    /// `imgLayer`.
    pub layer: Option<ImageLayer>,
}
