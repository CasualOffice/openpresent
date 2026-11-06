//! Colour values as the file holds them, and the one resolver that turns them
//! into a final sRGB value and alpha.
//!
//! The model is integers: percentages in thousandths of a percent and angles in
//! 60000ths of a degree, exactly the units of `ST_Percentage` and `ST_Angle`.
//! Floats exist only inside [`resolve`], which uses no platform maths library,
//! only IEEE operations that are exactly rounded, so the result is the same on
//! every target.
//!
//! # Arithmetic, and where it comes from
//!
//! ECMA-376 Part 1, 5th ed., 20.1.2.3. Its worked examples settle more than its
//! prose does:
//!
//! - `tint`, `shade`: linear light through the sRGB transfer. Examples: `00FF00`
//!   with `shade` 50% is `00BC00`, with `tint` 50% is `BCFFBC`.
//! - `scrgbClr`: linear, "a linear gamma of 1.0"; 50% is `BCBCBC`.
//! - `red`, `green`, `blue` and their `Off`, `Mod`: encoded sRGB. Examples:
//!   `FF0000` with `redMod` 50% is `800000`; with `redOff` -20% is `CC0000`.
//! - `hue`, `sat`, `lum` and their `Off`, `Mod`: HSL over encoded sRGB. Examples:
//!   `00FF00` with `lum` 20% is `006600`; with `satMod` 20% is `669966`.
//! - `hueOff`: limited to 0 to 360 degrees, not wrapped; the clause says so.
//!
//! The standard is silent on the following, each a decision pinned by a test
//! named `open_*`, to be settled by the render-fidelity work (BEH-03):
//!
//! - `inv`: encoded sRGB, like the other channel transforms. LibreOffice inverts
//!   in linear light. The only example is red to cyan, which fits both.
//! - `gray`: Rec. 709 weights over encoded sRGB. The clause gives no weights.
//!   LibreOffice uses 22, 72, 6 over encoded sRGB.
//! - `gamma`, `invGamma`: the IEC 61966-2-1 transfer and its inverse applied to
//!   the encoded value. LibreOffice has the same shape with a 2.3 power.
//! - `hueMod` above 360 degrees: limited, as `hueOff` is. LibreOffice agrees.
//! - Rounding to 8 bits: half up. The standard's examples disagree with each
//!   other: `blueMod` 50% gives `80` (up), `satOff` -20% gives `19` and `E5`
//!   (down), `sat` 50% gives `C0` where 191.25 is nearer `BF`.
//! - Consecutive HSL transforms keep their HSL state, so `lumMod` 0 followed by
//!   `lumOff` keeps the hue.
//! - `sysClr`: a host palette if given, then `lastClr` (the clause for
//!   `ST_SystemColorVal` requires it where system colours are unsupported), then
//!   [`SystemColor::fallback_rgb`], which is not from the standard.
//!
//! Defects in the standard's own examples, where computed values are used instead:
//! `hslClr` 240, 100%, 50% is given as `000080` and is `0000FF`; `lumMod` 50% of
//! `00FF00` is given as `007500` and is `008000`.

use crate::theme::{ColorMap, ColorMapSlot, ColorScheme, ColorSchemeIndex};

/// The most transforms one colour may carry. Real decks use fewer than ten, and
/// each costs a colour-space conversion at resolve time.
pub const MAX_COLOR_TRANSFORMS: usize = 64;

/// One hundred percent, in the file's units.
pub const PERCENT_100: i32 = 100_000;
/// One full turn, in the file's units.
pub const ANGLE_360: i32 = 21_600_000;

macro_rules! xml_enum {
    ($(#[$m:meta])* $name:ident { $($var:ident = $xml:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name { $(#[doc = $xml] $var),+ }

        impl $name {
            /// Every value, in schema order.
            pub const ALL: &'static [Self] = &[$(Self::$var),+];

            /// The value for this token, if it is one.
            pub fn from_xml(token: &str) -> Option<Self> {
                match token {
                    $($xml => Some(Self::$var),)+
                    _ => None,
                }
            }

            /// The token as the schema spells it.
            pub fn as_xml(self) -> &'static str {
                match self {
                    $(Self::$var => $xml,)+
                }
            }
        }
    };
}
pub(crate) use xml_enum;

/// An opaque 8-bit sRGB value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
}

impl Rgb {
    /// From components.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// `ST_HexColorRGB`: exactly six hex digits, either case.
    pub fn from_hex(text: &str) -> Option<Self> {
        let text = text.trim_matches(XML_SPACE);
        if text.len() != 6 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let byte = |at: usize| u8::from_str_radix(text.get(at..at + 2)?, 16).ok();
        Some(Self::new(byte(0)?, byte(2)?, byte(4)?))
    }

    /// Six upper-case hex digits.
    pub fn to_hex(self) -> String {
        format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    fn from_tuple((r, g, b): (u8, u8, u8)) -> Self {
        Self::new(r, g, b)
    }
}

const XML_SPACE: &[char] = &[' ', '\t', '\r', '\n'];

/// A value of `ST_Percentage` in thousandths of a percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Percentage(pub i32);

/// What was read from an `ST_Percentage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexicalPercentage {
    /// Thousandths of a percent.
    pub value: i32,
    /// The text had more precision than the model holds, so this was rounded.
    pub rounded: bool,
}

/// Parse the union `ST_Percentage`: an `xsd:int` in thousandths, or the Strict
/// form `-?[0-9]+(\.[0-9]+)?%`. `None` for anything else, an overflow included.
pub fn parse_percentage(text: &str) -> Option<LexicalPercentage> {
    let text = text.trim_matches(XML_SPACE);
    let Some(body) = text.strip_suffix('%') else {
        return text.parse::<i32>().ok().map(|value| LexicalPercentage {
            value,
            rounded: false,
        });
    };
    let (negative, body) = match body.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, body),
    };
    let (whole, fraction) = match body.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (body, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(whole) || fraction.is_some_and(|f| !digits(f)) {
        return None;
    }
    let mut total: i64 = 0;
    for b in whole.bytes() {
        total = total.checked_mul(10)?.checked_add(i64::from(b - b'0'))?;
        if total > i64::from(i32::MAX) {
            return None;
        }
    }
    total *= 1000;
    let mut rounded = false;
    if let Some(fraction) = fraction {
        let mut scale = 100;
        for (i, b) in fraction.bytes().enumerate() {
            let d = i64::from(b - b'0');
            match i {
                0..=2 => {
                    total += d * scale;
                    scale /= 10;
                }
                3 => {
                    rounded |= d != 0;
                    total += i64::from(d >= 5);
                }
                _ => rounded |= d != 0,
            }
        }
    }
    let value = i32::try_from(if negative { -total } else { total }).ok()?;
    Some(LexicalPercentage { value, rounded })
}

/// Write a percentage in the Strict form, losing nothing.
pub fn format_percentage(value: i32) -> String {
    let abs = i64::from(value).abs();
    let (whole, thousandths) = (abs / 1000, abs % 1000);
    let sign = if value < 0 { "-" } else { "" };
    if thousandths == 0 {
        format!("{sign}{whole}%")
    } else {
        let frac = format!("{thousandths:03}");
        format!("{sign}{whole}.{}%", frac.trim_end_matches('0'))
    }
}

/// An `ST_Angle`, in 60000ths of a degree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Angle(pub i32);

/// Parse an `xsd:int`.
pub fn parse_int(text: &str) -> Option<i32> {
    text.trim_matches(XML_SPACE).parse().ok()
}

xml_enum! {
    /// `ST_SchemeColorVal`.
    SchemeColor {
        Bg1 = "bg1", Tx1 = "tx1", Bg2 = "bg2", Tx2 = "tx2",
        Accent1 = "accent1", Accent2 = "accent2", Accent3 = "accent3",
        Accent4 = "accent4", Accent5 = "accent5", Accent6 = "accent6",
        Hlink = "hlink", FolHlink = "folHlink", PhClr = "phClr",
        Dk1 = "dk1", Lt1 = "lt1", Dk2 = "dk2", Lt2 = "lt2",
    }
}

/// Where a scheme colour value leads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemeTarget {
    /// Through the colour map.
    Mapped(ColorMapSlot),
    /// Straight to a theme slot, with no mapping.
    Direct(ColorSchemeIndex),
    /// To the colour of whatever referenced this style.
    Placeholder,
}

impl SchemeColor {
    /// Where this value leads.
    pub fn target(self) -> SchemeTarget {
        use ColorMapSlot as M;
        use ColorSchemeIndex as I;
        use SchemeTarget::{Direct, Mapped, Placeholder};
        match self {
            Self::Bg1 => Mapped(M::Bg1),
            Self::Tx1 => Mapped(M::Tx1),
            Self::Bg2 => Mapped(M::Bg2),
            Self::Tx2 => Mapped(M::Tx2),
            Self::Accent1 => Mapped(M::Accent1),
            Self::Accent2 => Mapped(M::Accent2),
            Self::Accent3 => Mapped(M::Accent3),
            Self::Accent4 => Mapped(M::Accent4),
            Self::Accent5 => Mapped(M::Accent5),
            Self::Accent6 => Mapped(M::Accent6),
            Self::Hlink => Mapped(M::Hlink),
            Self::FolHlink => Mapped(M::FolHlink),
            Self::PhClr => Placeholder,
            Self::Dk1 => Direct(I::Dk1),
            Self::Lt1 => Direct(I::Lt1),
            Self::Dk2 => Direct(I::Dk2),
            Self::Lt2 => Direct(I::Lt2),
        }
    }
}

xml_enum! {
    /// `ST_PresetColorVal`, all 190 values.
    PresetColor {
        AliceBlue = "aliceBlue",
        AntiqueWhite = "antiqueWhite",
        Aqua = "aqua",
        Aquamarine = "aquamarine",
        Azure = "azure",
        Beige = "beige",
        Bisque = "bisque",
        Black = "black",
        BlanchedAlmond = "blanchedAlmond",
        Blue = "blue",
        BlueViolet = "blueViolet",
        Brown = "brown",
        BurlyWood = "burlyWood",
        CadetBlue = "cadetBlue",
        Chartreuse = "chartreuse",
        Chocolate = "chocolate",
        Coral = "coral",
        CornflowerBlue = "cornflowerBlue",
        Cornsilk = "cornsilk",
        Crimson = "crimson",
        Cyan = "cyan",
        DarkBlue = "darkBlue",
        DarkCyan = "darkCyan",
        DarkGoldenrod = "darkGoldenrod",
        DarkGray = "darkGray",
        DarkGrey = "darkGrey",
        DarkGreen = "darkGreen",
        DarkKhaki = "darkKhaki",
        DarkMagenta = "darkMagenta",
        DarkOliveGreen = "darkOliveGreen",
        DarkOrange = "darkOrange",
        DarkOrchid = "darkOrchid",
        DarkRed = "darkRed",
        DarkSalmon = "darkSalmon",
        DarkSeaGreen = "darkSeaGreen",
        DarkSlateBlue = "darkSlateBlue",
        DarkSlateGray = "darkSlateGray",
        DarkSlateGrey = "darkSlateGrey",
        DarkTurquoise = "darkTurquoise",
        DarkViolet = "darkViolet",
        DkBlue = "dkBlue",
        DkCyan = "dkCyan",
        DkGoldenrod = "dkGoldenrod",
        DkGray = "dkGray",
        DkGrey = "dkGrey",
        DkGreen = "dkGreen",
        DkKhaki = "dkKhaki",
        DkMagenta = "dkMagenta",
        DkOliveGreen = "dkOliveGreen",
        DkOrange = "dkOrange",
        DkOrchid = "dkOrchid",
        DkRed = "dkRed",
        DkSalmon = "dkSalmon",
        DkSeaGreen = "dkSeaGreen",
        DkSlateBlue = "dkSlateBlue",
        DkSlateGray = "dkSlateGray",
        DkSlateGrey = "dkSlateGrey",
        DkTurquoise = "dkTurquoise",
        DkViolet = "dkViolet",
        DeepPink = "deepPink",
        DeepSkyBlue = "deepSkyBlue",
        DimGray = "dimGray",
        DimGrey = "dimGrey",
        DodgerBlue = "dodgerBlue",
        Firebrick = "firebrick",
        FloralWhite = "floralWhite",
        ForestGreen = "forestGreen",
        Fuchsia = "fuchsia",
        Gainsboro = "gainsboro",
        GhostWhite = "ghostWhite",
        Gold = "gold",
        Goldenrod = "goldenrod",
        Gray = "gray",
        Grey = "grey",
        Green = "green",
        GreenYellow = "greenYellow",
        Honeydew = "honeydew",
        HotPink = "hotPink",
        IndianRed = "indianRed",
        Indigo = "indigo",
        Ivory = "ivory",
        Khaki = "khaki",
        Lavender = "lavender",
        LavenderBlush = "lavenderBlush",
        LawnGreen = "lawnGreen",
        LemonChiffon = "lemonChiffon",
        LightBlue = "lightBlue",
        LightCoral = "lightCoral",
        LightCyan = "lightCyan",
        LightGoldenrodYellow = "lightGoldenrodYellow",
        LightGray = "lightGray",
        LightGrey = "lightGrey",
        LightGreen = "lightGreen",
        LightPink = "lightPink",
        LightSalmon = "lightSalmon",
        LightSeaGreen = "lightSeaGreen",
        LightSkyBlue = "lightSkyBlue",
        LightSlateGray = "lightSlateGray",
        LightSlateGrey = "lightSlateGrey",
        LightSteelBlue = "lightSteelBlue",
        LightYellow = "lightYellow",
        LtBlue = "ltBlue",
        LtCoral = "ltCoral",
        LtCyan = "ltCyan",
        LtGoldenrodYellow = "ltGoldenrodYellow",
        LtGray = "ltGray",
        LtGrey = "ltGrey",
        LtGreen = "ltGreen",
        LtPink = "ltPink",
        LtSalmon = "ltSalmon",
        LtSeaGreen = "ltSeaGreen",
        LtSkyBlue = "ltSkyBlue",
        LtSlateGray = "ltSlateGray",
        LtSlateGrey = "ltSlateGrey",
        LtSteelBlue = "ltSteelBlue",
        LtYellow = "ltYellow",
        Lime = "lime",
        LimeGreen = "limeGreen",
        Linen = "linen",
        Magenta = "magenta",
        Maroon = "maroon",
        MedAquamarine = "medAquamarine",
        MedBlue = "medBlue",
        MedOrchid = "medOrchid",
        MedPurple = "medPurple",
        MedSeaGreen = "medSeaGreen",
        MedSlateBlue = "medSlateBlue",
        MedSpringGreen = "medSpringGreen",
        MedTurquoise = "medTurquoise",
        MedVioletRed = "medVioletRed",
        MediumAquamarine = "mediumAquamarine",
        MediumBlue = "mediumBlue",
        MediumOrchid = "mediumOrchid",
        MediumPurple = "mediumPurple",
        MediumSeaGreen = "mediumSeaGreen",
        MediumSlateBlue = "mediumSlateBlue",
        MediumSpringGreen = "mediumSpringGreen",
        MediumTurquoise = "mediumTurquoise",
        MediumVioletRed = "mediumVioletRed",
        MidnightBlue = "midnightBlue",
        MintCream = "mintCream",
        MistyRose = "mistyRose",
        Moccasin = "moccasin",
        NavajoWhite = "navajoWhite",
        Navy = "navy",
        OldLace = "oldLace",
        Olive = "olive",
        OliveDrab = "oliveDrab",
        Orange = "orange",
        OrangeRed = "orangeRed",
        Orchid = "orchid",
        PaleGoldenrod = "paleGoldenrod",
        PaleGreen = "paleGreen",
        PaleTurquoise = "paleTurquoise",
        PaleVioletRed = "paleVioletRed",
        PapayaWhip = "papayaWhip",
        PeachPuff = "peachPuff",
        Peru = "peru",
        Pink = "pink",
        Plum = "plum",
        PowderBlue = "powderBlue",
        Purple = "purple",
        Red = "red",
        RosyBrown = "rosyBrown",
        RoyalBlue = "royalBlue",
        SaddleBrown = "saddleBrown",
        Salmon = "salmon",
        SandyBrown = "sandyBrown",
        SeaGreen = "seaGreen",
        SeaShell = "seaShell",
        Sienna = "sienna",
        Silver = "silver",
        SkyBlue = "skyBlue",
        SlateBlue = "slateBlue",
        SlateGray = "slateGray",
        SlateGrey = "slateGrey",
        Snow = "snow",
        SpringGreen = "springGreen",
        SteelBlue = "steelBlue",
        Tan = "tan",
        Teal = "teal",
        Thistle = "thistle",
        Tomato = "tomato",
        Turquoise = "turquoise",
        Violet = "violet",
        Wheat = "wheat",
        White = "white",
        WhiteSmoke = "whiteSmoke",
        Yellow = "yellow",
        YellowGreen = "yellowGreen",
    }
}

impl PresetColor {
    /// The value ECMA-376 Part 1, 20.1.10.48 gives.
    pub fn rgb(self) -> Rgb {
        Rgb::from_tuple(match self {
            Self::AliceBlue => (240, 248, 255),
            Self::AntiqueWhite => (250, 235, 215),
            Self::Aqua => (0, 255, 255),
            Self::Aquamarine => (127, 255, 212),
            Self::Azure => (240, 255, 255),
            Self::Beige => (245, 245, 220),
            Self::Bisque => (255, 228, 196),
            Self::Black => (0, 0, 0),
            Self::BlanchedAlmond => (255, 235, 205),
            Self::Blue => (0, 0, 255),
            Self::BlueViolet => (138, 43, 226),
            Self::Brown => (165, 42, 42),
            Self::BurlyWood => (222, 184, 135),
            Self::CadetBlue => (95, 158, 160),
            Self::Chartreuse => (127, 255, 0),
            Self::Chocolate => (210, 105, 30),
            Self::Coral => (255, 127, 80),
            Self::CornflowerBlue => (100, 149, 237),
            Self::Cornsilk => (255, 248, 220),
            Self::Crimson => (220, 20, 60),
            Self::Cyan => (0, 255, 255),
            Self::DarkBlue => (0, 0, 139),
            Self::DarkCyan => (0, 139, 139),
            Self::DarkGoldenrod => (184, 134, 11),
            Self::DarkGray => (169, 169, 169),
            Self::DarkGrey => (169, 169, 169),
            Self::DarkGreen => (0, 100, 0),
            Self::DarkKhaki => (189, 183, 107),
            Self::DarkMagenta => (139, 0, 139),
            Self::DarkOliveGreen => (85, 107, 47),
            Self::DarkOrange => (255, 140, 0),
            Self::DarkOrchid => (153, 50, 204),
            Self::DarkRed => (139, 0, 0),
            Self::DarkSalmon => (233, 150, 122),
            Self::DarkSeaGreen => (143, 188, 143),
            Self::DarkSlateBlue => (72, 61, 139),
            Self::DarkSlateGray => (47, 79, 79),
            Self::DarkSlateGrey => (47, 79, 79),
            Self::DarkTurquoise => (0, 206, 209),
            Self::DarkViolet => (148, 0, 211),
            Self::DkBlue => (0, 0, 139),
            Self::DkCyan => (0, 139, 139),
            Self::DkGoldenrod => (184, 134, 11),
            Self::DkGray => (169, 169, 169),
            Self::DkGrey => (169, 169, 169),
            Self::DkGreen => (0, 100, 0),
            Self::DkKhaki => (189, 183, 107),
            Self::DkMagenta => (139, 0, 139),
            Self::DkOliveGreen => (85, 107, 47),
            Self::DkOrange => (255, 140, 0),
            Self::DkOrchid => (153, 50, 204),
            Self::DkRed => (139, 0, 0),
            Self::DkSalmon => (233, 150, 122),
            Self::DkSeaGreen => (143, 188, 139),
            Self::DkSlateBlue => (72, 61, 139),
            Self::DkSlateGray => (47, 79, 79),
            Self::DkSlateGrey => (47, 79, 79),
            Self::DkTurquoise => (0, 206, 209),
            Self::DkViolet => (148, 0, 211),
            Self::DeepPink => (255, 20, 147),
            Self::DeepSkyBlue => (0, 191, 255),
            Self::DimGray => (105, 105, 105),
            Self::DimGrey => (105, 105, 105),
            Self::DodgerBlue => (30, 144, 255),
            Self::Firebrick => (178, 34, 34),
            Self::FloralWhite => (255, 250, 240),
            Self::ForestGreen => (34, 139, 34),
            Self::Fuchsia => (255, 0, 255),
            Self::Gainsboro => (220, 220, 220),
            Self::GhostWhite => (248, 248, 255),
            Self::Gold => (255, 215, 0),
            Self::Goldenrod => (218, 165, 32),
            Self::Gray => (128, 128, 128),
            Self::Grey => (128, 128, 128),
            Self::Green => (0, 128, 0),
            Self::GreenYellow => (173, 255, 47),
            Self::Honeydew => (240, 255, 240),
            Self::HotPink => (255, 105, 180),
            Self::IndianRed => (205, 92, 92),
            Self::Indigo => (75, 0, 130),
            Self::Ivory => (255, 255, 240),
            Self::Khaki => (240, 230, 140),
            Self::Lavender => (230, 230, 250),
            Self::LavenderBlush => (255, 240, 245),
            Self::LawnGreen => (124, 252, 0),
            Self::LemonChiffon => (255, 250, 205),
            Self::LightBlue => (173, 216, 230),
            Self::LightCoral => (240, 128, 128),
            Self::LightCyan => (224, 255, 255),
            Self::LightGoldenrodYellow => (250, 250, 210),
            Self::LightGray => (211, 211, 211),
            Self::LightGrey => (211, 211, 211),
            Self::LightGreen => (144, 238, 144),
            Self::LightPink => (255, 182, 193),
            Self::LightSalmon => (255, 160, 122),
            Self::LightSeaGreen => (32, 178, 170),
            Self::LightSkyBlue => (135, 206, 250),
            Self::LightSlateGray => (119, 136, 153),
            Self::LightSlateGrey => (119, 136, 153),
            Self::LightSteelBlue => (176, 196, 222),
            Self::LightYellow => (255, 255, 224),
            Self::LtBlue => (173, 216, 230),
            Self::LtCoral => (240, 128, 128),
            Self::LtCyan => (224, 255, 255),
            Self::LtGoldenrodYellow => (250, 250, 120),
            Self::LtGray => (211, 211, 211),
            Self::LtGrey => (211, 211, 211),
            Self::LtGreen => (144, 238, 144),
            Self::LtPink => (255, 182, 193),
            Self::LtSalmon => (255, 160, 122),
            Self::LtSeaGreen => (32, 178, 170),
            Self::LtSkyBlue => (135, 206, 250),
            Self::LtSlateGray => (119, 136, 153),
            Self::LtSlateGrey => (119, 136, 153),
            Self::LtSteelBlue => (176, 196, 222),
            Self::LtYellow => (255, 255, 224),
            Self::Lime => (0, 255, 0),
            Self::LimeGreen => (50, 205, 50),
            Self::Linen => (250, 240, 230),
            Self::Magenta => (255, 0, 255),
            Self::Maroon => (128, 0, 0),
            Self::MedAquamarine => (102, 205, 170),
            Self::MedBlue => (0, 0, 205),
            Self::MedOrchid => (186, 85, 211),
            Self::MedPurple => (147, 112, 219),
            Self::MedSeaGreen => (60, 179, 113),
            Self::MedSlateBlue => (123, 104, 238),
            Self::MedSpringGreen => (0, 250, 154),
            Self::MedTurquoise => (72, 209, 204),
            Self::MedVioletRed => (199, 21, 133),
            Self::MediumAquamarine => (102, 205, 170),
            Self::MediumBlue => (0, 0, 205),
            Self::MediumOrchid => (186, 85, 211),
            Self::MediumPurple => (147, 112, 219),
            Self::MediumSeaGreen => (60, 179, 113),
            Self::MediumSlateBlue => (123, 104, 238),
            Self::MediumSpringGreen => (0, 250, 154),
            Self::MediumTurquoise => (72, 209, 204),
            Self::MediumVioletRed => (199, 21, 133),
            Self::MidnightBlue => (25, 25, 112),
            Self::MintCream => (245, 255, 250),
            Self::MistyRose => (255, 228, 225),
            Self::Moccasin => (255, 228, 181),
            Self::NavajoWhite => (255, 222, 173),
            Self::Navy => (0, 0, 128),
            Self::OldLace => (253, 245, 230),
            Self::Olive => (128, 128, 0),
            Self::OliveDrab => (107, 142, 35),
            Self::Orange => (255, 165, 0),
            Self::OrangeRed => (255, 69, 0),
            Self::Orchid => (218, 112, 214),
            Self::PaleGoldenrod => (238, 232, 170),
            Self::PaleGreen => (152, 251, 152),
            Self::PaleTurquoise => (175, 238, 238),
            Self::PaleVioletRed => (219, 112, 147),
            Self::PapayaWhip => (255, 239, 213),
            Self::PeachPuff => (255, 218, 185),
            Self::Peru => (205, 133, 63),
            Self::Pink => (255, 192, 203),
            Self::Plum => (221, 160, 221),
            Self::PowderBlue => (176, 224, 230),
            Self::Purple => (128, 0, 128),
            Self::Red => (255, 0, 0),
            Self::RosyBrown => (188, 143, 143),
            Self::RoyalBlue => (65, 105, 225),
            Self::SaddleBrown => (139, 69, 19),
            Self::Salmon => (250, 128, 114),
            Self::SandyBrown => (244, 164, 96),
            Self::SeaGreen => (46, 139, 87),
            Self::SeaShell => (255, 245, 238),
            Self::Sienna => (160, 82, 45),
            Self::Silver => (192, 192, 192),
            Self::SkyBlue => (135, 206, 235),
            Self::SlateBlue => (106, 90, 205),
            Self::SlateGray => (112, 128, 144),
            Self::SlateGrey => (112, 128, 144),
            Self::Snow => (255, 250, 250),
            Self::SpringGreen => (0, 255, 127),
            Self::SteelBlue => (70, 130, 180),
            Self::Tan => (210, 180, 140),
            Self::Teal => (0, 128, 128),
            Self::Thistle => (216, 191, 216),
            Self::Tomato => (255, 99, 71),
            Self::Turquoise => (64, 224, 208),
            Self::Violet => (238, 130, 238),
            Self::Wheat => (245, 222, 179),
            Self::White => (255, 255, 255),
            Self::WhiteSmoke => (245, 245, 245),
            Self::Yellow => (255, 255, 0),
            Self::YellowGreen => (154, 205, 50),
        })
    }
}

xml_enum! {
    /// `ST_SystemColorVal`.
    SystemColor {
        ScrollBar = "scrollBar",
        Background = "background",
        ActiveCaption = "activeCaption",
        InactiveCaption = "inactiveCaption",
        Menu = "menu",
        Window = "window",
        WindowFrame = "windowFrame",
        MenuText = "menuText",
        WindowText = "windowText",
        CaptionText = "captionText",
        ActiveBorder = "activeBorder",
        InactiveBorder = "inactiveBorder",
        AppWorkspace = "appWorkspace",
        Highlight = "highlight",
        HighlightText = "highlightText",
        BtnFace = "btnFace",
        BtnShadow = "btnShadow",
        GrayText = "grayText",
        BtnText = "btnText",
        InactiveCaptionText = "inactiveCaptionText",
        BtnHighlight = "btnHighlight",
        ThreeDDkShadow = "3dDkShadow",
        ThreeDLight = "3dLight",
        InfoText = "infoText",
        InfoBk = "infoBk",
        HotLight = "hotLight",
        GradientActiveCaption = "gradientActiveCaption",
        GradientInactiveCaption = "gradientInactiveCaption",
        MenuHighlight = "menuHighlight",
        MenuBar = "menuBar",
    }
}

impl SystemColor {
    /// A fixed value for a `sysClr` with no `lastClr` and no host palette.
    ///
    /// **Not from the standard**, which leaves these to the viewing system. They
    /// are the Windows 10 defaults as recalled, so a document renders the same
    /// everywhere; a host that knows better passes a palette.
    pub fn fallback_rgb(self) -> Rgb {
        Rgb::from_tuple(match self {
            Self::ScrollBar => (200, 200, 200),
            Self::Background => (0, 0, 0),
            Self::ActiveCaption => (153, 180, 209),
            Self::InactiveCaption => (191, 205, 219),
            Self::Menu => (240, 240, 240),
            Self::Window => (255, 255, 255),
            Self::WindowFrame => (100, 100, 100),
            Self::MenuText => (0, 0, 0),
            Self::WindowText => (0, 0, 0),
            Self::CaptionText => (0, 0, 0),
            Self::ActiveBorder => (180, 180, 180),
            Self::InactiveBorder => (244, 247, 252),
            Self::AppWorkspace => (171, 171, 171),
            Self::Highlight => (0, 120, 215),
            Self::HighlightText => (255, 255, 255),
            Self::BtnFace => (240, 240, 240),
            Self::BtnShadow => (160, 160, 160),
            Self::GrayText => (109, 109, 109),
            Self::BtnText => (0, 0, 0),
            Self::InactiveCaptionText => (0, 0, 0),
            Self::BtnHighlight => (255, 255, 255),
            Self::ThreeDDkShadow => (105, 105, 105),
            Self::ThreeDLight => (227, 227, 227),
            Self::InfoText => (0, 0, 0),
            Self::InfoBk => (255, 255, 225),
            Self::HotLight => (0, 102, 204),
            Self::GradientActiveCaption => (185, 209, 234),
            Self::GradientInactiveCaption => (215, 228, 242),
            Self::MenuHighlight => (0, 120, 215),
            Self::MenuBar => (240, 240, 240),
        })
    }
}

/// The values a transform's `val` may take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueDomain {
    /// The transform has no value.
    None,
    /// `ST_PositiveFixedPercentage`: 0 to 100%.
    PositiveFixedPercentage,
    /// `ST_FixedPercentage`: -100% to 100%.
    FixedPercentage,
    /// `ST_PositivePercentage`: 0 and up.
    PositivePercentage,
    /// `ST_Percentage`: any.
    Percentage,
    /// `ST_PositiveFixedAngle`: 0 up to, not including, 360 degrees.
    PositiveFixedAngle,
    /// `ST_Angle`: any.
    Angle,
}

impl ValueDomain {
    /// Whether `value` is legal here.
    pub fn contains(self, value: i32) -> bool {
        match self {
            Self::None | Self::Percentage | Self::Angle => true,
            Self::PositiveFixedPercentage => (0..=PERCENT_100).contains(&value),
            Self::FixedPercentage => (-PERCENT_100..=PERCENT_100).contains(&value),
            Self::PositivePercentage => value >= 0,
            Self::PositiveFixedAngle => (0..ANGLE_360).contains(&value),
        }
    }

    /// Whether the value is a percentage, rather than an angle or nothing.
    pub fn is_percentage(self) -> bool {
        matches!(
            self,
            Self::PositiveFixedPercentage
                | Self::FixedPercentage
                | Self::PositivePercentage
                | Self::Percentage
        )
    }
}

xml_enum! {
    /// `EG_ColorTransform`.
    TransformKind {
        Tint = "tint", Shade = "shade", Comp = "comp", Inv = "inv", Gray = "gray",
        Alpha = "alpha", AlphaOff = "alphaOff", AlphaMod = "alphaMod",
        Hue = "hue", HueOff = "hueOff", HueMod = "hueMod",
        Sat = "sat", SatOff = "satOff", SatMod = "satMod",
        Lum = "lum", LumOff = "lumOff", LumMod = "lumMod",
        Red = "red", RedOff = "redOff", RedMod = "redMod",
        Green = "green", GreenOff = "greenOff", GreenMod = "greenMod",
        Blue = "blue", BlueOff = "blueOff", BlueMod = "blueMod",
        Gamma = "gamma", InvGamma = "invGamma",
    }
}

impl TransformKind {
    /// The schema's type for this transform's `val`.
    pub fn domain(self) -> ValueDomain {
        use ValueDomain as D;
        match self {
            Self::Comp | Self::Inv | Self::Gray | Self::Gamma | Self::InvGamma => D::None,
            Self::Tint | Self::Shade | Self::Alpha => D::PositiveFixedPercentage,
            Self::AlphaOff => D::FixedPercentage,
            Self::AlphaMod | Self::HueMod => D::PositivePercentage,
            Self::Hue => D::PositiveFixedAngle,
            Self::HueOff => D::Angle,
            Self::Sat
            | Self::SatOff
            | Self::SatMod
            | Self::Lum
            | Self::LumOff
            | Self::LumMod
            | Self::Red
            | Self::RedOff
            | Self::RedMod
            | Self::Green
            | Self::GreenOff
            | Self::GreenMod
            | Self::Blue
            | Self::BlueOff
            | Self::BlueMod => D::Percentage,
        }
    }
}

/// One transform. `value` is in the units of its kind's [`ValueDomain`], and 0
/// for a kind with no value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorTransform {
    /// Which transform.
    pub kind: TransformKind,
    /// Its `val`, as the file has it.
    pub value: i32,
}

impl ColorTransform {
    /// A transform with a value.
    pub const fn new(kind: TransformKind, value: i32) -> Self {
        Self { kind, value }
    }
}

/// `EG_ColorChoice`, without its transforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorValue {
    /// `scrgbClr`, linear; components may lie outside 0 to 100%.
    ScRgb {
        /// Red.
        r: Percentage,
        /// Green.
        g: Percentage,
        /// Blue.
        b: Percentage,
    },
    /// `srgbClr`.
    Srgb(Rgb),
    /// `hslClr`.
    Hsl {
        /// Hue.
        hue: Angle,
        /// Saturation.
        sat: Percentage,
        /// Luminance.
        lum: Percentage,
    },
    /// `sysClr`.
    System {
        /// Which system colour.
        val: SystemColor,
        /// `lastClr`, if the file has it.
        last: Option<Rgb>,
    },
    /// `schemeClr`.
    Scheme(SchemeColor),
    /// `prstClr`.
    Preset(PresetColor),
}

impl ColorValue {
    /// The element's local name.
    pub fn element_name(&self) -> &'static str {
        match self {
            Self::ScRgb { .. } => "scrgbClr",
            Self::Srgb(_) => "srgbClr",
            Self::Hsl { .. } => "hslClr",
            Self::System { .. } => "sysClr",
            Self::Scheme(_) => "schemeClr",
            Self::Preset(_) => "prstClr",
        }
    }
}

/// A colour: a value and its transforms, in document order.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Color {
    /// The base value.
    pub value: ColorValue,
    /// Applied first to last.
    pub transforms: Vec<ColorTransform>,
}

impl Color {
    /// A colour with no transforms.
    pub fn new(value: ColorValue) -> Self {
        Self {
            value,
            transforms: Vec::new(),
        }
    }

    /// An `srgbClr`.
    pub fn srgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(ColorValue::Srgb(Rgb::new(r, g, b)))
    }

    /// Why this colour is not one the schema allows, if it is not.
    pub fn validate(&self) -> Result<(), ResolveError> {
        if self.transforms.len() > MAX_COLOR_TRANSFORMS {
            return Err(ResolveError::TooManyTransforms);
        }
        if let ColorValue::Hsl { hue, .. } = self.value
            && !ValueDomain::PositiveFixedAngle.contains(hue.0)
        {
            return Err(ResolveError::ValueOutOfRange);
        }
        if self
            .transforms
            .iter()
            .all(|t| t.kind.domain().contains(t.value))
        {
            Ok(())
        } else {
            Err(ResolveError::ValueOutOfRange)
        }
    }
}

/// `CT_ColorChangeEffect`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorChange {
    /// `clrFrom`.
    pub from: Option<Color>,
    /// `clrTo`.
    pub to: Option<Color>,
    /// `@useA`, if the file has it.
    pub use_alpha: Option<bool>,
}

/// The final colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResolvedColor {
    /// 8-bit sRGB.
    pub rgb: Rgb,
    /// Opacity in thousandths of a percent, 0 to 100000.
    pub alpha: u32,
}

/// Why a colour has no final value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveError {
    /// More than [`MAX_COLOR_TRANSFORMS`] on one colour.
    TooManyTransforms,
    /// A value outside its schema type.
    ValueOutOfRange,
    /// `bg1` and the like with no colour map, or one lacking this slot.
    Unmapped(ColorMapSlot),
    /// A scheme colour with no colour scheme, or one lacking this slot.
    MissingSchemeSlot(ColorSchemeIndex),
    /// `phClr` with no placeholder colour.
    NoPlaceholderColor,
    /// Theme slots or placeholder colours that lead back to themselves.
    Cycle,
}

impl core::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooManyTransforms => write!(f, "too many colour transforms"),
            Self::ValueOutOfRange => write!(f, "colour value out of range"),
            Self::Unmapped(slot) => write!(f, "no colour map entry for {}", slot.as_xml()),
            Self::MissingSchemeSlot(i) => write!(f, "no theme colour {}", i.as_xml()),
            Self::NoPlaceholderColor => write!(f, "phClr with no placeholder colour"),
            Self::Cycle => write!(f, "scheme colours form a cycle"),
        }
    }
}

impl std::error::Error for ResolveError {}

/// What a colour resolves against. Everything absent stays absent: a scheme
/// colour with no scheme is an error, never a default.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResolveContext<'a> {
    /// The effective colour scheme.
    pub scheme: Option<&'a ColorScheme>,
    /// The effective colour map.
    pub map: Option<&'a ColorMap>,
    /// What `phClr` stands for.
    pub placeholder: Option<&'a Color>,
    /// The host's system colours, where it has them.
    pub system_palette: Option<fn(SystemColor) -> Option<Rgb>>,
}

/// Resolve to final sRGB and alpha. Pure and deterministic: the same arguments
/// give the same result on every platform.
pub fn resolve(color: &Color, ctx: &ResolveContext<'_>) -> Result<ResolvedColor, ResolveError> {
    let mut walk = Walk {
        visited: [false; 12],
        placeholder_used: false,
    };
    let state = walk.evaluate(color, ctx)?;
    let [r, g, b] = state.work.into_srgb();
    Ok(ResolvedColor {
        rgb: Rgb::new(to_byte(r), to_byte(g), to_byte(b)),
        alpha: (state.alpha * f64::from(PERCENT_100) + 0.5).floor() as u32,
    })
}

struct Walk {
    visited: [bool; 12],
    placeholder_used: bool,
}

impl Walk {
    fn evaluate(&mut self, color: &Color, ctx: &ResolveContext<'_>) -> Result<State, ResolveError> {
        color.validate()?;
        let mut state = self.base(&color.value, ctx)?;
        for t in &color.transforms {
            state.apply(t);
        }
        Ok(state)
    }

    fn base(
        &mut self,
        value: &ColorValue,
        ctx: &ResolveContext<'_>,
    ) -> Result<State, ResolveError> {
        let work = match *value {
            ColorValue::ScRgb { r, g, b } => Work::Linear([pct(r.0), pct(g.0), pct(b.0)]),
            ColorValue::Srgb(rgb) => Work::from_rgb(rgb),
            ColorValue::Hsl { hue, sat, lum } => Work::Hsl {
                h: f64::from(hue.0) / 60_000.0,
                s: unit(pct(sat.0)),
                l: unit(pct(lum.0)),
            },
            ColorValue::System { val, last } => {
                let host = ctx.system_palette.and_then(|p| p(val));
                Work::from_rgb(host.or(last).unwrap_or_else(|| val.fallback_rgb()))
            }
            ColorValue::Preset(p) => Work::from_rgb(p.rgb()),
            ColorValue::Scheme(s) => return self.scheme(s, ctx),
        };
        Ok(State { work, alpha: 1.0 })
    }

    fn scheme(&mut self, s: SchemeColor, ctx: &ResolveContext<'_>) -> Result<State, ResolveError> {
        let index = match s.target() {
            SchemeTarget::Placeholder => {
                if self.placeholder_used {
                    return Err(ResolveError::Cycle);
                }
                self.placeholder_used = true;
                let held = ctx.placeholder.ok_or(ResolveError::NoPlaceholderColor)?;
                let state = self.evaluate(held, ctx);
                self.placeholder_used = false;
                return state;
            }
            SchemeTarget::Direct(index) => index,
            SchemeTarget::Mapped(slot) => ctx
                .map
                .and_then(|m| m.get(slot))
                .ok_or(ResolveError::Unmapped(slot))?,
        };
        let at = index as usize;
        if core::mem::replace(&mut self.visited[at], true) {
            return Err(ResolveError::Cycle);
        }
        let held = ctx
            .scheme
            .and_then(|c| c.get(index))
            .ok_or(ResolveError::MissingSchemeSlot(index))?;
        let state = self.evaluate(held, ctx);
        self.visited[at] = false;
        state
    }
}

fn pct(value: i32) -> f64 {
    f64::from(value) / f64::from(PERCENT_100)
}

fn unit(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn to_byte(encoded: f64) -> u8 {
    // The small nudge makes half-up hold when a conversion lands a hair under.
    (unit(encoded) * 255.0 + 0.5 + 1e-9).floor() as u8
}

/// Colour during evaluation, held in whichever space the last transform used.
/// Staying there avoids round trips, and keeps a hue through black.
#[derive(Debug, Clone, Copy)]
enum Work {
    /// Encoded sRGB, 0 to 1.
    Srgb([f64; 3]),
    /// Linear light. May lie outside 0 to 1 until it leaves this space.
    Linear([f64; 3]),
    /// Degrees 0 to 360, then 0 to 1, over encoded sRGB.
    Hsl { h: f64, s: f64, l: f64 },
}

impl Work {
    fn from_rgb(rgb: Rgb) -> Self {
        Self::Srgb([rgb.r, rgb.g, rgb.b].map(|c| f64::from(c) / 255.0))
    }

    fn into_srgb(self) -> [f64; 3] {
        match self {
            Self::Srgb(c) => c,
            Self::Linear(c) => c.map(|v| encode_srgb(unit(v))),
            Self::Hsl { h, s, l } => hsl_to_srgb(h, s, l),
        }
    }

    fn into_linear(self) -> [f64; 3] {
        match self {
            Self::Linear(c) => c,
            other => other.into_srgb().map(|v| decode_srgb(unit(v))),
        }
    }

    fn into_hsl(self) -> (f64, f64, f64) {
        match self {
            Self::Hsl { h, s, l } => (h, s, l),
            other => srgb_to_hsl(other.into_srgb().map(unit)),
        }
    }
}

struct State {
    work: Work,
    alpha: f64,
}

impl State {
    fn apply(&mut self, t: &ColorTransform) {
        use TransformKind as K;
        let v = pct(t.value);
        match t.kind {
            K::Tint => {
                let c = self.work.into_linear();
                self.work = Work::Linear(c.map(|c| 1.0 - (1.0 - c) * v));
            }
            K::Shade => {
                let c = self.work.into_linear();
                self.work = Work::Linear(c.map(|c| c * v));
            }
            K::Comp => {
                let (h, s, l) = self.work.into_hsl();
                let turned = h + 180.0;
                let h = if turned >= 360.0 {
                    turned - 360.0
                } else {
                    turned
                };
                self.work = Work::Hsl { h, s, l };
            }
            K::Inv => self.channels(|c| [1.0 - c[0], 1.0 - c[1], 1.0 - c[2]]),
            K::Gray => self.channels(|c| {
                let y = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
                [y, y, y]
            }),
            K::Gamma => self.channels(|c| c.map(encode_srgb)),
            K::InvGamma => self.channels(|c| c.map(decode_srgb)),
            K::Alpha => self.alpha = unit(v),
            K::AlphaOff => self.alpha = unit(self.alpha + v),
            K::AlphaMod => self.alpha = unit(self.alpha * v),
            K::Hue => self.hsl(|c| c.0 = f64::from(t.value) / 60_000.0),
            K::HueOff => self.hsl(|c| {
                c.0 = (c.0 + f64::from(t.value) / 60_000.0).clamp(0.0, 360.0);
            }),
            K::HueMod => self.hsl(|c| c.0 = (c.0 * v).clamp(0.0, 360.0)),
            K::Sat => self.hsl(|c| c.1 = unit(v)),
            K::SatOff => self.hsl(|c| c.1 = unit(c.1 + v)),
            K::SatMod => self.hsl(|c| c.1 = unit(c.1 * v)),
            K::Lum => self.hsl(|c| c.2 = unit(v)),
            K::LumOff => self.hsl(|c| c.2 = unit(c.2 + v)),
            K::LumMod => self.hsl(|c| c.2 = unit(c.2 * v)),
            K::Red => self.channel(0, |_| v),
            K::RedOff => self.channel(0, |c| c + v),
            K::RedMod => self.channel(0, |c| c * v),
            K::Green => self.channel(1, |_| v),
            K::GreenOff => self.channel(1, |c| c + v),
            K::GreenMod => self.channel(1, |c| c * v),
            K::Blue => self.channel(2, |_| v),
            K::BlueOff => self.channel(2, |c| c + v),
            K::BlueMod => self.channel(2, |c| c * v),
        }
    }

    fn channels(&mut self, f: impl FnOnce([f64; 3]) -> [f64; 3]) {
        let c = self.work.into_srgb().map(unit);
        self.work = Work::Srgb(f(c).map(unit));
    }

    fn channel(&mut self, at: usize, f: impl FnOnce(f64) -> f64) {
        self.channels(|mut c| {
            c[at] = f(c[at]);
            c
        });
    }

    fn hsl(&mut self, f: impl FnOnce(&mut (f64, f64, f64))) {
        let mut c = self.work.into_hsl();
        f(&mut c);
        self.work = Work::Hsl {
            h: c.0,
            s: c.1,
            l: c.2,
        };
    }
}

fn srgb_to_hsl([r, g, b]: [f64; 3]) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let d = max - min;
    if d == 0.0 {
        return (0.0, 0.0, l);
    }
    let s = if l <= 0.5 {
        d / (max + min)
    } else {
        d / (2.0 - max - min)
    };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s, l)
}

fn hsl_to_srgb(h: f64, s: f64, l: f64) -> [f64; 3] {
    if s == 0.0 {
        return [l, l, l];
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let at = |offset: f64| {
        let mut t = h / 360.0 + offset;
        if t < 0.0 {
            t += 1.0;
        }
        if t >= 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    [at(1.0 / 3.0), at(0.0), at(-1.0 / 3.0)]
}

/// Linear light to encoded sRGB, IEC 61966-2-1. `c` is within 0 to 1.
fn encode_srgb(c: f64) -> f64 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        // c^(1/2.4) = c^(5/12) = 12th root of c^5.
        let c5 = c * c * c * c * c;
        1.055 * nth_root(c5, 3).sqrt().sqrt() - 0.055
    }
}

/// Encoded sRGB to linear light, IEC 61966-2-1. `c` is within 0 to 1.
fn decode_srgb(c: f64) -> f64 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        // x^2.4 = x^2 * 5th root of x^2.
        let x = (c + 0.055) / 1.055;
        let x2 = x * x;
        x2 * nth_root(x2, 5)
    }
}

/// The nth root of `v` in (0, 1], by Newton's method from above.
///
/// `powf` calls the platform's maths library, whose last bit differs between
/// targets. This uses only exactly rounded operations, so it does not.
fn nth_root(v: f64, n: u32) -> f64 {
    let mut x = 1.0_f64;
    let k = f64::from(n);
    for _ in 0..256 {
        let mut power = 1.0;
        for _ in 1..n {
            power *= x;
        }
        let next = ((k - 1.0) * x + v / power) / k;
        if next >= x {
            break;
        }
        x = next;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_transfer_matches_the_formula_it_replaces() {
        for i in 0..=4096 {
            let c = f64::from(i) / 4096.0;
            if c > 0.003_130_8 {
                let want = 1.055 * c.powf(1.0 / 2.4) - 0.055;
                assert!((encode_srgb(c) - want).abs() < 1e-12, "encode {c}");
            }
            if c > 0.040_45 {
                let want = ((c + 0.055) / 1.055).powf(2.4);
                assert!((decode_srgb(c) - want).abs() < 1e-12, "decode {c}");
            }
            assert!((encode_srgb(decode_srgb(c)) - c).abs() < 1e-10, "trip {c}");
        }
    }
}
