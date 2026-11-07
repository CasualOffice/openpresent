//! Text bodies as plain typed values (`docs/spec/TEXT.md`, Tier 1F): `bodyPr`,
//! `lstStyle`, paragraphs, runs, breaks and fields, and every run and paragraph
//! property.
//!
//! `Option` means the file did not state a value, or stated one that could not be
//! used (the import report says which). A schema default is never filled in, and
//! nothing is flattened: effective values come from `casual-slide-style` at read
//! time. Where a type's written form is a union, [`Written`] keeps what the file
//! wrote beside the value, so a `normAutofit/@fontScale` of `62.5%` stays `62.5%`.
//!
//! A run's fill, line, effects and underline line and fill are Tier 1E's own types, as
//! are a body's `prstTxWarp`, `scene3d` and `EG_Text3D`. What is still another tier's
//! stays in the part's tree, preserved but not modelled, and named in the report: a
//! picture bullet's `a:blip` beyond its relationship, and `extLst`.
//!
//! Text content is document data: it lives here and in the tree, and never reaches
//! a diagnostic.

use crate::color::Color;
use crate::d3::{FlatText, Scene3D, Shape3D};
use crate::effect::EffectProperties;
use crate::fill::Fill;
use crate::geometry::PresetTextShape;
use crate::line::Line;
use crate::presentation::{Coordinate, Percentage, TextFont};
use crate::properties::{Lexical, token_enum, trim_xml};
use crate::shape::Hyperlink;

/// A value whose type has more than one written form, with the form the file used.
///
/// Compared by value: `62.5%` and `62500` say the same thing, so changing one to
/// the other is not an edit, and the tree's bytes stay as they are.
#[derive(Debug, Clone)]
pub struct Written<T> {
    value: T,
    written: Option<String>,
}

impl<T> Written<T> {
    /// A new value, with no written form yet. Export spells it.
    pub fn new(value: T) -> Self {
        Self {
            value,
            written: None,
        }
    }

    /// The value.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// The text the file held, exactly. `None` for a value made by [`Self::new`].
    pub fn written(&self) -> Option<&str> {
        self.written.as_deref()
    }
}

impl<T: PartialEq> PartialEq for Written<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for Written<T> {}

impl<T: Lexical> Lexical for Written<T> {
    fn parse_lexical(s: &str) -> Option<Self> {
        Some(Self {
            value: T::parse_lexical(s)?,
            written: Some(s.to_owned()),
        })
    }
    fn to_lexical(&self) -> String {
        self.written
            .clone()
            .unwrap_or_else(|| self.value.to_lexical())
    }
}

/// `a:ST_TextPoint`, in hundredths of a point: a bare integer, or a universal
/// measure such as `1pt`, rounded to the nearest hundredth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextPoint(pub i32);

/// EMU in a hundredth of a point.
const EMU_PER_HUNDREDTH_POINT: i64 = 127;

impl Lexical for TextPoint {
    fn parse_lexical(s: &str) -> Option<Self> {
        let s = trim_xml(s);
        if let Ok(v) = s.parse::<i32>() {
            return Some(Self(v));
        }
        // A bare integer too large for `i32` is not a measure.
        if s.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
            return None;
        }
        let emu = Coordinate::parse_lexical(s)?.0;
        let half = EMU_PER_HUNDREDTH_POINT / 2;
        let rounded = if emu < 0 {
            emu.checked_sub(half)?
        } else {
            emu.checked_add(half)?
        };
        i32::try_from(rounded / EMU_PER_HUNDREDTH_POINT)
            .ok()
            .map(Self)
    }
    fn to_lexical(&self) -> String {
        self.0.to_string()
    }
}

/// Smallest `ST_TextFontSize`.
pub const FONT_SIZE_MIN: i32 = 100;
/// Largest `ST_TextFontSize`, `ST_TextNonNegativePoint` and `ST_TextPointUnqualified`.
pub const TEXT_POINT_MAX: i32 = 400_000;
/// Largest `ST_TextSpacingPoint`.
pub const SPACING_POINT_MAX: i32 = 158_400;
/// Largest `ST_TextSpacingPercent`, in thousandths.
pub const SPACING_PERCENT_MAX: i32 = 13_200_000;
/// Smallest `ST_TextFontScalePercent`, in thousandths.
pub const FONT_SCALE_MIN: i32 = 1_000;
/// Largest `ST_TextFontScalePercent`, in thousandths.
pub const FONT_SCALE_MAX: i32 = 100_000;
/// Smallest `ST_TextBulletSizeDecimal` and `ST_TextBulletSizePercent`, in thousandths.
pub const BULLET_SIZE_MIN: i32 = 25_000;
/// Largest `ST_TextBulletSizeDecimal` and `ST_TextBulletSizePercent`, in thousandths.
pub const BULLET_SIZE_MAX: i32 = 400_000;
/// Largest `ST_TextMargin`, and the magnitude of the largest `ST_TextIndent`.
pub const TEXT_MARGIN_MAX: i32 = 51_206_400;
/// Largest `ST_TextIndentLevelType`.
pub const TEXT_LEVEL_MAX: u8 = 8;
/// Largest `ST_TextColumnCount`.
pub const TEXT_COLUMNS_MAX: u8 = 16;
/// Largest `ST_TextBulletStartAtNum`.
pub const BULLET_START_MAX: i32 = 32_767;

token_enum!(
    /// `a:ST_TextAnchoringType`.
    TextAnchor {
        Top = "t",
        Center = "ctr",
        Bottom = "b",
        Justified = "just",
        Distributed = "dist",
    }
);

token_enum!(
    /// `a:ST_TextVertOverflowType`.
    VerticalOverflow {
        Overflow = "overflow",
        Ellipsis = "ellipsis",
        Clip = "clip",
    }
);

token_enum!(
    /// `a:ST_TextHorzOverflowType`.
    HorizontalOverflow {
        Overflow = "overflow",
        Clip = "clip",
    }
);

token_enum!(
    /// `a:ST_TextVerticalType`.
    TextVertical {
        Horz = "horz",
        Vert = "vert",
        Vert270 = "vert270",
        WordArtVert = "wordArtVert",
        EaVert = "eaVert",
        MongolianVert = "mongolianVert",
        WordArtVertRtl = "wordArtVertRtl",
    }
);

token_enum!(
    /// `a:ST_TextWrappingType`.
    TextWrap {
        None = "none",
        Square = "square",
    }
);

token_enum!(
    /// `a:ST_TextAutonumberScheme`: all 41 schemes.
    AutonumberScheme {
        AlphaLcParenBoth = "alphaLcParenBoth",
        AlphaUcParenBoth = "alphaUcParenBoth",
        AlphaLcParenR = "alphaLcParenR",
        AlphaUcParenR = "alphaUcParenR",
        AlphaLcPeriod = "alphaLcPeriod",
        AlphaUcPeriod = "alphaUcPeriod",
        ArabicParenBoth = "arabicParenBoth",
        ArabicParenR = "arabicParenR",
        ArabicPeriod = "arabicPeriod",
        ArabicPlain = "arabicPlain",
        RomanLcParenBoth = "romanLcParenBoth",
        RomanUcParenBoth = "romanUcParenBoth",
        RomanLcParenR = "romanLcParenR",
        RomanUcParenR = "romanUcParenR",
        RomanLcPeriod = "romanLcPeriod",
        RomanUcPeriod = "romanUcPeriod",
        CircleNumDbPlain = "circleNumDbPlain",
        CircleNumWdBlackPlain = "circleNumWdBlackPlain",
        CircleNumWdWhitePlain = "circleNumWdWhitePlain",
        ArabicDbPeriod = "arabicDbPeriod",
        ArabicDbPlain = "arabicDbPlain",
        Ea1ChsPeriod = "ea1ChsPeriod",
        Ea1ChsPlain = "ea1ChsPlain",
        Ea1ChtPeriod = "ea1ChtPeriod",
        Ea1ChtPlain = "ea1ChtPlain",
        Ea1JpnChsDbPeriod = "ea1JpnChsDbPeriod",
        Ea1JpnKorPlain = "ea1JpnKorPlain",
        Ea1JpnKorPeriod = "ea1JpnKorPeriod",
        Arabic1Minus = "arabic1Minus",
        Arabic2Minus = "arabic2Minus",
        Hebrew2Minus = "hebrew2Minus",
        ThaiAlphaPeriod = "thaiAlphaPeriod",
        ThaiAlphaParenR = "thaiAlphaParenR",
        ThaiAlphaParenBoth = "thaiAlphaParenBoth",
        ThaiNumPeriod = "thaiNumPeriod",
        ThaiNumParenR = "thaiNumParenR",
        ThaiNumParenBoth = "thaiNumParenBoth",
        HindiAlphaPeriod = "hindiAlphaPeriod",
        HindiNumPeriod = "hindiNumPeriod",
        HindiNumParenR = "hindiNumParenR",
        HindiAlpha1Period = "hindiAlpha1Period",
    }
);

token_enum!(
    /// `a:ST_TextUnderlineType`.
    UnderlineType {
        None = "none",
        Words = "words",
        Sng = "sng",
        Dbl = "dbl",
        Heavy = "heavy",
        Dotted = "dotted",
        DottedHeavy = "dottedHeavy",
        Dash = "dash",
        DashHeavy = "dashHeavy",
        DashLong = "dashLong",
        DashLongHeavy = "dashLongHeavy",
        DotDash = "dotDash",
        DotDashHeavy = "dotDashHeavy",
        DotDotDash = "dotDotDash",
        DotDotDashHeavy = "dotDotDashHeavy",
        Wavy = "wavy",
        WavyHeavy = "wavyHeavy",
        WavyDbl = "wavyDbl",
    }
);

token_enum!(
    /// `a:ST_TextStrikeType`.
    StrikeType {
        NoStrike = "noStrike",
        SngStrike = "sngStrike",
        DblStrike = "dblStrike",
    }
);

token_enum!(
    /// `a:ST_TextCapsType`.
    CapsType {
        None = "none",
        Small = "small",
        All = "all",
    }
);

token_enum!(
    /// `a:ST_TextTabAlignType`.
    TabAlign {
        Left = "l",
        Center = "ctr",
        Right = "r",
        Decimal = "dec",
    }
);

token_enum!(
    /// `a:ST_TextAlignType`.
    TextAlign {
        Left = "l",
        Center = "ctr",
        Right = "r",
        Justified = "just",
        JustifiedLow = "justLow",
        Distributed = "dist",
        ThaiDistributed = "thaiDist",
    }
);

token_enum!(
    /// `a:ST_TextFontAlignType`.
    FontAlign {
        Auto = "auto",
        Top = "t",
        Center = "ctr",
        Baseline = "base",
        Bottom = "b",
    }
);

/// `a:EG_TextAutofit`. Stored as written; recomputation is Phase 2.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Autofit {
    /// `noAutofit`.
    None,
    /// `normAutofit`, with the scale the writer last computed.
    Normal {
        /// `@fontScale`, as written. Absent is not filled in with 100%.
        font_scale: Option<Written<Percentage>>,
        /// `@lnSpcReduction`, as written. Absent is not filled in with 0%.
        line_spacing_reduction: Option<Written<Percentage>>,
    },
    /// `spAutoFit`.
    Shape,
}

/// The `EG_Text3D` choice: a body's text is extruded, or flat at a height.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Text3D {
    /// `sp3d`.
    Shape(Box<Shape3D>),
    /// `flatTx`.
    Flat(FlatText),
}

/// `a:CT_TextBodyProperties`. `extLst` stays in the tree, and is reported.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BodyProperties {
    /// `@rot`, 60000ths of a degree.
    pub rotation: Option<i32>,
    /// `@spcFirstLastPara`.
    pub space_first_last_paragraph: Option<bool>,
    /// `@vertOverflow`.
    pub vertical_overflow: Option<VerticalOverflow>,
    /// `@horzOverflow`.
    pub horizontal_overflow: Option<HorizontalOverflow>,
    /// `@vert`.
    pub vertical: Option<TextVertical>,
    /// `@wrap`.
    pub wrap: Option<TextWrap>,
    /// `@lIns`, EMU, as written.
    pub inset_left: Option<Written<Coordinate>>,
    /// `@tIns`, EMU, as written.
    pub inset_top: Option<Written<Coordinate>>,
    /// `@rIns`, EMU, as written.
    pub inset_right: Option<Written<Coordinate>>,
    /// `@bIns`, EMU, as written.
    pub inset_bottom: Option<Written<Coordinate>>,
    /// `@numCol`, 1 to 16.
    pub columns: Option<u8>,
    /// `@spcCol`, EMU.
    pub column_spacing: Option<i32>,
    /// `@rtlCol`.
    pub rtl_columns: Option<bool>,
    /// `@fromWordArt`.
    pub from_word_art: Option<bool>,
    /// `@anchor`.
    pub anchor: Option<TextAnchor>,
    /// `@anchorCtr`.
    pub anchor_center: Option<bool>,
    /// `@forceAA`.
    pub force_antialias: Option<bool>,
    /// `@upright`.
    pub upright: Option<bool>,
    /// `@compatLnSpc`.
    pub compat_line_spacing: Option<bool>,
    /// `prstTxWarp`, with its `avLst`.
    pub warp: Option<PresetTextShape>,
    /// The `EG_TextAutofit` choice.
    pub autofit: Option<Autofit>,
    /// `scene3d`.
    pub scene3d: Option<Box<Scene3D>>,
    /// The `EG_Text3D` choice.
    pub text_3d: Option<Text3D>,
}

/// `a:CT_TextSpacing`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextSpacing {
    /// `spcPct/@val`, in thousandths, as written.
    Percent(Written<Percentage>),
    /// `spcPts/@val`, in hundredths of a point.
    Points(i32),
}

/// `a:EG_TextBulletColor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BulletColor {
    /// `buClrTx`.
    FollowText,
    /// `buClr`. `None` if its colour could not be believed; the tree keeps it.
    Color(Option<Color>),
}

/// `a:EG_TextBulletSize`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BulletSize {
    /// `buSzTx`.
    FollowText,
    /// `buSzPct/@val`, in thousandths, as written.
    Percent(Written<Percentage>),
    /// `buSzPts/@val`, in hundredths of a point.
    Points(i32),
}

/// `a:EG_TextBulletTypeface`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BulletFont {
    /// `buFontTx`.
    FollowText,
    /// `buFont`.
    Font(TextFont),
}

/// `a:EG_TextBullet`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bullet {
    /// `buNone`.
    None,
    /// `buAutoNum`.
    AutoNumber {
        /// `@type`, which the schema requires.
        scheme: Option<AutonumberScheme>,
        /// `@startAt`, 1 to 32767. Absent is not filled in with 1.
        start_at: Option<i32>,
    },
    /// `buChar/@char`, which the schema requires.
    Char(Option<String>),
    /// `buBlip`: the picture's relationship. The rest of `a:blip` belongs to the
    /// picture tier and stays in the tree; nothing here reads the image.
    Picture {
        /// `a:blip/@r:embed`.
        embed: Option<String>,
    },
}

/// `a:CT_TextTabStop`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TabStop {
    /// `@pos`, EMU, as written.
    pub position: Option<Written<Coordinate>>,
    /// `@algn`.
    pub align: Option<TabAlign>,
}

/// `a:CT_TextParagraphProperties`: `pPr` and each list style level. `extLst` stays
/// in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParagraphProperties {
    /// `@marL`, EMU.
    pub margin_left: Option<i32>,
    /// `@marR`, EMU.
    pub margin_right: Option<i32>,
    /// `@lvl`, 0 to 8.
    pub level: Option<u8>,
    /// `@indent`, EMU.
    pub indent: Option<i32>,
    /// `@algn`.
    pub align: Option<TextAlign>,
    /// `@defTabSz`, EMU, as written.
    pub default_tab_size: Option<Written<Coordinate>>,
    /// `@rtl`.
    pub rtl: Option<bool>,
    /// `@eaLnBrk`.
    pub east_asian_line_break: Option<bool>,
    /// `@fontAlgn`.
    pub font_align: Option<FontAlign>,
    /// `@latinLnBrk`.
    pub latin_line_break: Option<bool>,
    /// `@hangingPunct`.
    pub hanging_punctuation: Option<bool>,
    /// `lnSpc`.
    pub line_spacing: Option<TextSpacing>,
    /// `spcBef`.
    pub space_before: Option<TextSpacing>,
    /// `spcAft`.
    pub space_after: Option<TextSpacing>,
    /// The `EG_TextBulletColor` choice.
    pub bullet_color: Option<BulletColor>,
    /// The `EG_TextBulletSize` choice.
    pub bullet_size: Option<BulletSize>,
    /// The `EG_TextBulletTypeface` choice.
    pub bullet_font: Option<BulletFont>,
    /// The `EG_TextBullet` choice.
    pub bullet: Option<Bullet>,
    /// `tabLst/tab`.
    pub tabs: Option<Vec<TabStop>>,
    /// `defRPr`.
    pub default_run: Option<Box<CharacterProperties>>,
}

/// `a:EG_TextUnderlineLine`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnderlineLine {
    /// `uLnTx`.
    FollowText,
    /// `uLn`.
    Line(Box<Line>),
}

/// `a:EG_TextUnderlineFill`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnderlineFill {
    /// `uFillTx`.
    FollowText,
    /// `uFill`, with its fill if it states one.
    Fill(Option<Fill>),
}

/// `a:CT_Boolean`, as `rtl` in `rPr`. An absent `@val` is not filled in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BooleanElement {
    /// `@val`.
    pub value: Option<bool>,
}

/// `a:CT_TextCharacterProperties`: `rPr`, `defRPr` and `endParaRPr`. `extLst` stays
/// in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CharacterProperties {
    /// `@kumimoji`.
    pub kumimoji: Option<bool>,
    /// `@lang`.
    pub lang: Option<String>,
    /// `@altLang`.
    pub alt_lang: Option<String>,
    /// `@sz`, hundredths of a point.
    pub size: Option<i32>,
    /// `@b`.
    pub bold: Option<bool>,
    /// `@i`.
    pub italic: Option<bool>,
    /// `@u`.
    pub underline: Option<UnderlineType>,
    /// `@strike`.
    pub strike: Option<StrikeType>,
    /// `@kern`, hundredths of a point.
    pub kerning: Option<i32>,
    /// `@cap`.
    pub caps: Option<CapsType>,
    /// `@spc`, hundredths of a point, as written.
    pub spacing: Option<Written<TextPoint>>,
    /// `@normalizeH`.
    pub normalize_height: Option<bool>,
    /// `@baseline`, thousandths of a percent, as written.
    pub baseline: Option<Written<Percentage>>,
    /// `@noProof`.
    pub no_proof: Option<bool>,
    /// `@dirty`.
    pub dirty: Option<bool>,
    /// `@err`.
    pub error: Option<bool>,
    /// `@smtClean`.
    pub smart_tag_clean: Option<bool>,
    /// `@smtId`.
    pub smart_tag_id: Option<u32>,
    /// `@bmk`.
    pub bookmark: Option<String>,
    /// `ln`.
    pub line: Option<Box<Line>>,
    /// The `EG_FillProperties` choice. `Fill::Group` is `noFill` in text: Office treats
    /// it so (`MS-OI29500` 2.1.1298), and the resolver says so.
    pub fill: Option<Fill>,
    /// The `EG_EffectProperties` choice.
    pub effects: Option<Box<EffectProperties>>,
    /// `highlight`. The outer `None` is absent, the inner one a colour that could
    /// not be believed; the tree keeps it.
    pub highlight: Option<Option<Color>>,
    /// The `EG_TextUnderlineLine` choice.
    pub underline_line: Option<UnderlineLine>,
    /// The `EG_TextUnderlineFill` choice.
    pub underline_fill: Option<UnderlineFill>,
    /// `latin`.
    pub latin: Option<Box<TextFont>>,
    /// `ea`.
    pub east_asian: Option<Box<TextFont>>,
    /// `cs`.
    pub complex_script: Option<Box<TextFont>>,
    /// `sym`.
    pub symbol: Option<Box<TextFont>>,
    /// `hlinkClick`.
    pub hyperlink_click: Option<Box<Hyperlink>>,
    /// `hlinkMouseOver`.
    pub hyperlink_mouse_over: Option<Box<Hyperlink>>,
    /// `rtl`.
    pub rtl: Option<BooleanElement>,
}

/// `a:CT_RegularTextRun`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextRun {
    /// `rPr`.
    pub properties: Option<Box<CharacterProperties>>,
    /// `t`, which the schema requires. Document data: never in a diagnostic.
    pub text: Option<String>,
}

/// `a:CT_TextLineBreak`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineBreak {
    /// `rPr`.
    pub properties: Option<Box<CharacterProperties>>,
}

/// `a:CT_TextField`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextField {
    /// `@id`, which the schema requires.
    pub id: Option<String>,
    /// `@type`, as written: `slidenum`, `datetime1` and so on (`MS-OI29500`, the note
    /// on ECMA-376 Part 1 21.1.2.2.4). The schema does not restrict it.
    pub field_type: Option<String>,
    /// `rPr`.
    pub properties: Option<Box<CharacterProperties>>,
    /// `pPr`.
    pub paragraph_properties: Option<Box<ParagraphProperties>>,
    /// `t`, the text last shown. Document data: never in a diagnostic.
    pub text: Option<String>,
}

/// One child of a paragraph, in document order.
/// In `EG_TextRun`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextItem {
    /// `r`.
    Run(TextRun),
    /// `br`.
    Break(LineBreak),
    /// `fld`.
    Field(TextField),
    /// An equation: `a14:m`, or the `mc:AlternateContent` around one, with its fallback.
    Math(Box<crate::math::InlineMath>),
    /// Anything else, by its local name. It stays in the tree where it was, and is
    /// reported.
    Retained(String),
}

/// `a:CT_TextParagraph`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Paragraph {
    /// `pPr`.
    pub properties: Option<Box<ParagraphProperties>>,
    /// Runs, breaks and fields, in order.
    pub items: Vec<TextItem>,
    /// `endParaRPr`.
    pub end_properties: Option<Box<CharacterProperties>>,
}

/// `a:CT_TextListStyle`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextListStyle {
    /// `defPPr`. Kept for round trip, and not used in resolution: Office ignores it
    /// (`MS-OI29500`, the note on ECMA-376 Part 1 21.1.2.2.2).
    pub default: Option<Box<ParagraphProperties>>,
    /// `lvl1pPr`, `lvl2pPr`, `lvl3pPr`, `lvl4pPr`, `lvl5pPr`, `lvl6pPr`, `lvl7pPr`, `lvl8pPr`,
    /// `lvl9pPr`, by level from 0.
    pub levels: [Option<Box<ParagraphProperties>>; 9],
}

impl TextListStyle {
    /// The element holding a level, from 0.
    pub fn level_element(level: usize) -> Option<&'static str> {
        const NAMES: [&str; 9] = [
            "lvl1pPr", "lvl2pPr", "lvl3pPr", "lvl4pPr", "lvl5pPr", "lvl6pPr", "lvl7pPr", "lvl8pPr",
            "lvl9pPr",
        ];
        NAMES.get(level).copied()
    }

    /// A level's properties, from 0. A level past 8 has none.
    pub fn level(&self, level: u8) -> Option<&ParagraphProperties> {
        self.levels.get(usize::from(level))?.as_deref()
    }
}

/// `a:CT_TextBody`. Any child beyond these stays in the tree, and is reported.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextBody {
    /// `bodyPr`, which the schema requires.
    pub properties: Option<BodyProperties>,
    /// `lstStyle`.
    pub list_style: Option<TextListStyle>,
    /// `p`, of which the schema requires one.
    pub paragraphs: Vec<Paragraph>,
}
