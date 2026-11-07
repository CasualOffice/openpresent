//! Themes as the file holds them: colour, font and format schemes, object
//! defaults, extra schemes, custom colours, colour maps and overrides.
//!
//! Absent stays absent, schema-required items included, because a hostile or
//! damaged part may lack them. Fills, lines and effects inside a format scheme
//! are shown here only as far as theme colours are concerned: the rest belongs to
//! their own areas and stays in the part's tree, preserved but not modelled.

use crate::color::{Color, xml_enum};
use crate::text::{BodyProperties, TextListStyle};

xml_enum! {
    /// `ST_ColorSchemeIndex`: the twelve slots of a colour scheme.
    ColorSchemeIndex {
        Dk1 = "dk1", Lt1 = "lt1", Dk2 = "dk2", Lt2 = "lt2",
        Accent1 = "accent1", Accent2 = "accent2", Accent3 = "accent3",
        Accent4 = "accent4", Accent5 = "accent5", Accent6 = "accent6",
        Hlink = "hlink", FolHlink = "folHlink",
    }
}

xml_enum! {
    /// The attributes of `CT_ColorMapping`: what a slide calls a colour.
    ColorMapSlot {
        Bg1 = "bg1", Tx1 = "tx1", Bg2 = "bg2", Tx2 = "tx2",
        Accent1 = "accent1", Accent2 = "accent2", Accent3 = "accent3",
        Accent4 = "accent4", Accent5 = "accent5", Accent6 = "accent6",
        Hlink = "hlink", FolHlink = "folHlink",
    }
}

xml_enum! {
    /// `ST_FontCollectionIndex`.
    FontCollectionIndex { Major = "major", Minor = "minor", None = "none" }
}

xml_enum! {
    /// `ST_OnOffStyleType`.
    OnOffStyle { On = "on", Off = "off", Def = "def" }
}

/// `CT_ColorScheme`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColorScheme {
    /// `@name`.
    pub name: Option<String>,
    /// By [`ColorSchemeIndex`].
    pub slots: [Option<Color>; 12],
    /// `extLst`, by `uri`; the content stays in the part.
    pub extensions: Vec<String>,
}

impl ColorScheme {
    /// A slot's colour.
    pub fn get(&self, index: ColorSchemeIndex) -> Option<&Color> {
        self.slots[index as usize].as_ref()
    }
}

/// `CT_ColorMapping`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColorMap {
    /// By [`ColorMapSlot`].
    pub slots: [Option<ColorSchemeIndex>; 12],
    /// `extLst`, by `uri`; the content stays in the part.
    pub extensions: Vec<String>,
}

impl ColorMap {
    /// Where a slot points.
    pub fn get(&self, slot: ColorMapSlot) -> Option<ColorSchemeIndex> {
        self.slots[slot as usize]
    }
}

/// `CT_ColorMappingOverride`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorMapOverride {
    /// `masterClrMapping`.
    Master,
    /// `overrideClrMapping`.
    Override(ColorMap),
}

/// The colour map in force on a slide.
///
/// An override on the slide wins, then one on its layout, then the master's map.
/// ECMA-376 19.3.1.7 reads as if `masterClrMapping` on a slide skipped the layout;
/// docs/22 specifies this chain instead, and it is how dark layouts are recalled
/// to behave in PowerPoint. Open under BEH-04 until a PowerPoint fixture settles
/// it.
pub fn effective_color_map<'a>(
    master: Option<&'a ColorMap>,
    layout: Option<&'a ColorMapOverride>,
    slide: Option<&'a ColorMapOverride>,
) -> Option<&'a ColorMap> {
    let overriding = |o: Option<&'a ColorMapOverride>| match o {
        Some(ColorMapOverride::Override(map)) => Some(map),
        _ => None,
    };
    overriding(slide).or(overriding(layout)).or(master)
}

/// The colour scheme in force: an override part's scheme replaces the theme's
/// whole, since `CT_ColorScheme` always carries all twelve.
pub fn effective_color_scheme<'a>(
    theme: Option<&'a ColorScheme>,
    overriding: Option<&'a ThemeOverride>,
) -> Option<&'a ColorScheme> {
    overriding.and_then(|o| o.color_scheme.as_ref()).or(theme)
}

/// `CT_TextFont`, as a theme font scheme uses it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextFont {
    /// `@typeface`. An empty string is meaningful: no font for this script.
    pub typeface: Option<String>,
    /// `@panose`, ten bytes as hex text.
    pub panose: Option<String>,
    /// `@pitchFamily`.
    pub pitch_family: Option<i8>,
    /// `@charset`.
    pub charset: Option<i8>,
}

/// `CT_SupplementalFont`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SupplementalFont {
    /// `@script`.
    pub script: Option<String>,
    /// `@typeface`.
    pub typeface: Option<String>,
}

/// `CT_FontCollection`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FontCollection {
    /// `latin`.
    pub latin: Option<TextFont>,
    /// `ea`.
    pub ea: Option<TextFont>,
    /// `cs`.
    pub cs: Option<TextFont>,
    /// `font`, in order.
    pub fonts: Vec<SupplementalFont>,
    /// `extLst`, by `uri`; the content stays in the part.
    pub extensions: Vec<String>,
}

/// `CT_FontScheme`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FontScheme {
    /// `@name`.
    pub name: Option<String>,
    /// `majorFont`.
    pub major: Option<FontCollection>,
    /// `minorFont`.
    pub minor: Option<FontCollection>,
    /// `extLst`, by `uri`; the content stays in the part.
    pub extensions: Vec<String>,
}

/// One gradient stop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradientStop {
    /// `@pos` in thousandths of a percent.
    pub position: Option<i32>,
    /// The stop's colour.
    pub color: Option<Color>,
}

/// A fill in a format scheme. Colours are shown; the rest of each fill (angles,
/// tiling, the picture) stays in the part, preserved but not modelled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StyleFill {
    /// `noFill`.
    None,
    /// `solidFill`.
    Solid(Option<Color>),
    /// `gradFill`.
    Gradient(Vec<GradientStop>),
    /// `blipFill`.
    Picture,
    /// `pattFill`.
    Pattern {
        /// `fgClr`.
        foreground: Option<Color>,
        /// `bgClr`.
        background: Option<Color>,
    },
    /// `grpFill`.
    Group,
}

/// A line in a format scheme.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleLine {
    /// `@w` in EMU.
    pub width: Option<i32>,
    /// Its fill.
    pub fill: Option<StyleFill>,
}

/// An effect style. Its content stays in the part.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StyleEffect {
    /// `effectLst` or `effectDag` is present.
    pub has_effects: bool,
    /// `scene3d` is present.
    pub has_scene3d: bool,
    /// `sp3d` is present.
    pub has_sp3d: bool,
}

/// `CT_StyleMatrix`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FormatScheme {
    /// `@name`.
    pub name: Option<String>,
    /// `fillStyleLst`.
    pub fills: Option<Vec<StyleFill>>,
    /// `lnStyleLst`.
    pub lines: Option<Vec<StyleLine>>,
    /// `effectStyleLst`.
    pub effects: Option<Vec<StyleEffect>>,
    /// `bgFillStyleLst`.
    pub background_fills: Option<Vec<StyleFill>>,
}

impl FormatScheme {
    /// A style matrix fill reference: 1 to 999 is `fillStyleLst`, 1001 and up is
    /// `bgFillStyleLst`, and 0 or 1000 is no fill. `None` also for an index past
    /// the end of its list.
    pub fn fill(&self, idx: u32) -> Option<&StyleFill> {
        let (list, at) = match idx {
            0 | 1000 => return None,
            1..=999 => (&self.fills, idx - 1),
            _ => (&self.background_fills, idx - 1001),
        };
        list.as_ref()?.get(at as usize)
    }

    /// A style matrix line reference, 1-based; 0 is no line.
    pub fn line(&self, idx: u32) -> Option<&StyleLine> {
        self.lines.as_ref()?.get(idx.checked_sub(1)? as usize)
    }

    /// A style matrix effect reference, 1-based; 0 is no effect.
    pub fn effect(&self, idx: u32) -> Option<&StyleEffect> {
        self.effects.as_ref()?.get(idx.checked_sub(1)? as usize)
    }
}

/// `CT_BaseStyles`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BaseStyles {
    /// `clrScheme`.
    pub color_scheme: Option<ColorScheme>,
    /// `fontScheme`.
    pub font_scheme: Option<FontScheme>,
    /// `fmtScheme`.
    pub format_scheme: Option<FormatScheme>,
    /// `extLst`, by `uri`; the content stays in the part.
    pub extensions: Vec<String>,
}

/// `CT_BaseStylesOverride`: a `themeOverride` part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThemeOverride {
    /// `clrScheme`.
    pub color_scheme: Option<ColorScheme>,
    /// `fontScheme`.
    pub font_scheme: Option<FontScheme>,
    /// `fmtScheme`.
    pub format_scheme: Option<FormatScheme>,
}

/// `CT_StyleMatrixReference`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleMatrixRef {
    /// `@idx`.
    pub idx: Option<u32>,
    /// The colour standing for `phClr`.
    pub color: Option<Color>,
}

/// `CT_FontReference`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FontRef {
    /// `@idx`.
    pub idx: Option<FontCollectionIndex>,
    /// The colour.
    pub color: Option<Color>,
}

/// `CT_ShapeStyle`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShapeStyle {
    /// `lnRef`.
    pub line: Option<StyleMatrixRef>,
    /// `fillRef`.
    pub fill: Option<StyleMatrixRef>,
    /// `effectRef`.
    pub effect: Option<StyleMatrixRef>,
    /// `fontRef`.
    pub font: Option<FontRef>,
}

/// `CT_EffectReference`: `a:effect` in an effect list, naming an effect by `@ref`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectReference {
    /// `@ref`, a token as written.
    pub reference: Option<String>,
}

/// `a:themeManager`, `CT_EmptyElement`: the root of a part that holds nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ThemeManager;

/// `EG_ThemeableFillStyle`, `EG_ThemeableEffectStyle`: a reference into the
/// format scheme, or an inline value that stays in the part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeableStyle {
    /// `fill` or `effect`, preserved but not modelled.
    Inline,
    /// `fillRef` or `effectRef`.
    Ref(StyleMatrixRef),
}

/// `EG_ThemeableFontStyles`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeableFont {
    /// `font`.
    Inline(FontCollection),
    /// `fontRef`.
    Ref(FontRef),
}

/// `CT_DefaultShapeDefinition`. `bodyPr` and `lstStyle` take part in a shape's text
/// inheritance (`MS-OI29500` 2.1.1379, 2.1.1388, 2.1.1399). `spPr` stays in the part,
/// preserved but not modelled here (`T1C-02`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShapeDefinition {
    /// `spPr` is present.
    pub has_shape_properties: bool,
    /// `bodyPr`.
    pub body_properties: Option<BodyProperties>,
    /// `lstStyle`.
    pub list_style: Option<TextListStyle>,
    /// `style`.
    pub style: Option<ShapeStyle>,
    /// `extLst`, by `uri`; the content stays in the part.
    pub extensions: Vec<String>,
}

/// `CT_ObjectStyleDefaults`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObjectDefaults {
    /// `spDef`.
    pub shape: Option<ShapeDefinition>,
    /// `lnDef`.
    pub line: Option<ShapeDefinition>,
    /// `txDef`.
    pub text: Option<ShapeDefinition>,
    /// `extLst`, by `uri`; the content stays in the part.
    pub extensions: Vec<String>,
}

/// `CT_ColorSchemeAndMapping`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExtraColorScheme {
    /// `clrScheme`.
    pub color_scheme: Option<ColorScheme>,
    /// `clrMap`.
    pub map: Option<ColorMap>,
}

/// `CT_CustomColor`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomColor {
    /// `@name`.
    pub name: Option<String>,
    /// The colour.
    pub color: Option<Color>,
}

/// `CT_ThemeFamily`, the Office 2013 extension.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThemeFamily {
    /// `@name`.
    pub name: Option<String>,
    /// `@id`.
    pub id: Option<String>,
    /// `@vid`.
    pub vid: Option<String>,
}

/// `CT_OfficeStyleSheet`: a theme part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Theme {
    /// `@name`.
    pub name: Option<String>,
    /// `themeElements`.
    pub elements: Option<BaseStyles>,
    /// `objectDefaults`.
    pub object_defaults: Option<ObjectDefaults>,
    /// `extraClrSchemeLst`.
    pub extra_color_schemes: Option<Vec<ExtraColorScheme>>,
    /// `custClrLst`.
    pub custom_colors: Option<Vec<CustomColor>>,
    /// `extLst`, by `uri`; the content stays in the part.
    pub extensions: Vec<String>,
    /// The `themeFamily` extension, where present.
    pub family: Option<ThemeFamily>,
}

/// `CT_ClipboardStyleSheet`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClipboardStyleSheet {
    /// `themeElements`.
    pub elements: Option<BaseStyles>,
    /// `clrMap`.
    pub map: Option<ColorMap>,
}
