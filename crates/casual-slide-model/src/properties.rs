//! Document properties as plain typed values (`docs/spec/PKG.md`: `core:`, `ep:`, `cp:`, `vt:`).
//!
//! No XML lives here. `Option` means the file did not state a value, or stated
//! one that could not be used (the import report says which). A schema default
//! is never filled in.

/// A value with one written form in the file formats this engine reads.
///
/// Parsing is by the XML Schema rules for the type, so `inf`, which Rust would
/// take for a float, is refused, and `INF` is not.
pub trait Lexical: Sized {
    /// The value a written form denotes, if it is one.
    fn parse_lexical(s: &str) -> Option<Self>;
    /// A written form of the value.
    fn to_lexical(&self) -> String;
}

/// XML Schema's whitespace collapse at the edges. Not Unicode's, which would
/// accept values the schema does not.
pub fn trim_xml(s: &str) -> &str {
    s.trim_matches([' ', '\t', '\r', '\n'])
}

impl Lexical for String {
    fn parse_lexical(s: &str) -> Option<Self> {
        Some(s.to_owned())
    }
    fn to_lexical(&self) -> String {
        self.clone()
    }
}

impl Lexical for bool {
    fn parse_lexical(s: &str) -> Option<Self> {
        match trim_xml(s) {
            "1" | "true" => Some(true),
            "0" | "false" => Some(false),
            _ => None,
        }
    }
    fn to_lexical(&self) -> String {
        if *self { "1" } else { "0" }.to_owned()
    }
}

macro_rules! lexical_int {
    ($($t:ty),*) => {$(
        impl Lexical for $t {
            fn parse_lexical(s: &str) -> Option<Self> {
                trim_xml(s).parse().ok()
            }
            fn to_lexical(&self) -> String {
                self.to_string()
            }
        }
    )*};
}
lexical_int!(i8, i16, i32, i64, u8, u16, u32, u64);

macro_rules! float_value {
    ($(#[$doc:meta])* $name:ident, $t:ty) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name($t);

        impl $name {
            /// Wrap a value. Every NaN becomes the one NaN, so equal in the file
            /// is equal here.
            pub fn new(value: $t) -> Self {
                Self(if value.is_nan() { <$t>::NAN } else { value })
            }
            /// The value.
            pub fn get(self) -> $t {
                self.0
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0.to_bits() == other.0.to_bits()
            }
        }
        impl Eq for $name {}

        impl Lexical for $name {
            fn parse_lexical(s: &str) -> Option<Self> {
                let s = trim_xml(s);
                match s {
                    "INF" | "+INF" => return Some(Self(<$t>::INFINITY)),
                    "-INF" => return Some(Self(<$t>::NEG_INFINITY)),
                    "NaN" => return Some(Self(<$t>::NAN)),
                    _ => {}
                }
                // Rust also takes `inf`, `nan` and `infinity`.
                if !s
                    .bytes()
                    .all(|b| b.is_ascii_digit() || matches!(b, b'+' | b'-' | b'.' | b'e' | b'E'))
                {
                    return None;
                }
                s.parse().ok().map(Self)
            }
            fn to_lexical(&self) -> String {
                if self.0.is_nan() {
                    "NaN".to_owned()
                } else if self.0.is_infinite() {
                    if self.0 > 0.0 { "INF" } else { "-INF" }.to_owned()
                } else {
                    self.0.to_string()
                }
            }
        }
    };
}
float_value!(
    /// An `xsd:float`, compared by bits so that a NaN equals itself.
    Float32,
    f32
);
float_value!(
    /// An `xsd:double`, compared by bits so that a NaN equals itself.
    Float64,
    f64
);

/// Declare an enumeration of tokens with the written form of each.
macro_rules! token_enum {
    ($(#[$doc:meta])* $name:ident { $($variant:ident = $token:literal),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum $name {
            $(#[doc = concat!("`", $token, "`")] $variant,)+
        }

        impl $crate::properties::Lexical for $name {
            fn parse_lexical(s: &str) -> Option<Self> {
                match $crate::properties::trim_xml(s) {
                    $($token => Some(Self::$variant),)+
                    _ => None,
                }
            }
            fn to_lexical(&self) -> String {
                match self {
                    $(Self::$variant => $token,)+
                }
                .to_owned()
            }
        }
    };
}
pub(crate) use token_enum;

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn digits(s: &str, n: usize) -> Option<(u32, &str)> {
    let head = s.get(..n)?;
    if !head.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((head.parse().ok()?, &s[n..]))
}

fn parse_zone(s: &str) -> Option<()> {
    match s.as_bytes().first() {
        None => Some(()),
        Some(b'Z') if s.len() == 1 => Some(()),
        Some(b'+' | b'-') => {
            let (hh, rest) = digits(&s[1..], 2)?;
            let rest = rest.strip_prefix(':')?;
            let (mm, rest) = digits(rest, 2)?;
            (rest.is_empty() && hh <= 14 && mm <= 59).then_some(())
        }
        _ => None,
    }
}

fn parse_time(s: &str) -> Option<()> {
    let (hh, rest) = digits(s, 2)?;
    let rest = rest.strip_prefix(':')?;
    let (mm, rest) = digits(rest, 2)?;
    if hh > 23 || mm > 59 {
        return None;
    }
    let Some(rest) = rest.strip_prefix(':') else {
        return parse_zone(rest);
    };
    let (ss, rest) = digits(rest, 2)?;
    // 60 is a leap second, which W3CDTF allows.
    if ss > 60 {
        return None;
    }
    let rest = match rest.strip_prefix('.') {
        Some(frac) => {
            let n = frac.bytes().take_while(u8::is_ascii_digit).count();
            if n == 0 {
                return None;
            }
            &frac[n..]
        }
        None => rest,
    };
    parse_zone(rest)
}

/// A date in the W3CDTF profile of ISO 8601: `YYYY`, `YYYY-MM`, `YYYY-MM-DD`, or
/// a full date and time. Holds the text it was read from, checked, so an unedited
/// value writes back as written.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct W3cDateTime(String);

impl W3cDateTime {
    /// The text this value was parsed from.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether `s` is a full date and time, which a `vt:date` must be.
    pub fn is_full_date_time(s: &str) -> bool {
        Self::parse_lexical(s).is_some() && trim_xml(s).contains('T')
    }
}

impl Lexical for W3cDateTime {
    fn parse_lexical(s: &str) -> Option<Self> {
        let s = trim_xml(s);
        let (year, rest) = digits(s, 4)?;
        if rest.is_empty() {
            return Some(Self(s.to_owned()));
        }
        let (month, rest) = digits(rest.strip_prefix('-')?, 2)?;
        if !(1..=12).contains(&month) {
            return None;
        }
        if rest.is_empty() {
            return Some(Self(s.to_owned()));
        }
        let (day, rest) = digits(rest.strip_prefix('-')?, 2)?;
        if day == 0 || day > days_in_month(year, month) {
            return None;
        }
        if rest.is_empty() {
            return Some(Self(s.to_owned()));
        }
        parse_time(rest.strip_prefix('T')?)?;
        Some(Self(s.to_owned()))
    }
    fn to_lexical(&self) -> String {
        self.0.clone()
    }
}

/// `core:CT_Keyword`: a keyword, possibly in a stated language.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Keyword {
    /// `@xml:lang`.
    pub lang: Option<String>,
    /// The text.
    pub text: String,
}

/// `core:CT_Keywords`: mixed content, so plain text may sit beside `value` children.
/// The usual file holds only the text.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Keywords {
    /// `@xml:lang`.
    pub lang: Option<String>,
    /// The element's own text, joined across any `value` children.
    pub text: String,
    /// `value` children.
    pub values: Vec<Keyword>,
}

/// `core:CT_CoreProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CoreProperties {
    /// `cp:category`.
    pub category: Option<String>,
    /// `cp:contentStatus`.
    pub content_status: Option<String>,
    /// `dcterms:created`.
    pub created: Option<W3cDateTime>,
    /// `dc:creator`.
    pub creator: Option<String>,
    /// `dc:description`.
    pub description: Option<String>,
    /// `dc:identifier`.
    pub identifier: Option<String>,
    /// `cp:keywords`.
    pub keywords: Option<Keywords>,
    /// `dc:language`.
    pub language: Option<String>,
    /// `cp:lastModifiedBy`.
    pub last_modified_by: Option<String>,
    /// `cp:lastPrinted`.
    pub last_printed: Option<W3cDateTime>,
    /// `dcterms:modified`.
    pub modified: Option<W3cDateTime>,
    /// `cp:revision`. A string in the schema, whatever it looks like.
    pub revision: Option<String>,
    /// `dc:subject`.
    pub subject: Option<String>,
    /// `dc:title`.
    pub title: Option<String>,
    /// `cp:version`.
    pub version: Option<String>,
}

token_enum!(
    /// `vt:ST_VectorBaseType`.
    VectorBaseType {
        Variant = "variant",
        I1 = "i1",
        I2 = "i2",
        I4 = "i4",
        I8 = "i8",
        Ui1 = "ui1",
        Ui2 = "ui2",
        Ui4 = "ui4",
        Ui8 = "ui8",
        R4 = "r4",
        R8 = "r8",
        Lpstr = "lpstr",
        Lpwstr = "lpwstr",
        Bstr = "bstr",
        Date = "date",
        Filetime = "filetime",
        Bool = "bool",
        Cy = "cy",
        Error = "error",
        Clsid = "clsid",
    }
);

token_enum!(
    /// `vt:ST_ArrayBaseType`.
    ArrayBaseType {
        Variant = "variant",
        I1 = "i1",
        I2 = "i2",
        I4 = "i4",
        Int = "int",
        Ui1 = "ui1",
        Ui2 = "ui2",
        Ui4 = "ui4",
        Uint = "uint",
        R4 = "r4",
        R8 = "r8",
        Decimal = "decimal",
        Bstr = "bstr",
        Date = "date",
        Bool = "bool",
        Error = "error",
        Cy = "cy",
    }
);

/// `vt:CT_Vector`. The file's `@size` is only a claim: it is checked against the
/// items and reported, never used to allocate, and rewritten if the items change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VtVector {
    /// `@baseType`.
    pub base_type: VectorBaseType,
    /// The items.
    // spec-audit: vt:CT_Vector/variant
    // spec-audit: vt:CT_Vector/i1
    // spec-audit: vt:CT_Vector/i2
    // spec-audit: vt:CT_Vector/i4
    // spec-audit: vt:CT_Vector/i8
    // spec-audit: vt:CT_Vector/ui1
    // spec-audit: vt:CT_Vector/ui2
    // spec-audit: vt:CT_Vector/ui4
    // spec-audit: vt:CT_Vector/ui8
    // spec-audit: vt:CT_Vector/r4
    // spec-audit: vt:CT_Vector/r8
    // spec-audit: vt:CT_Vector/lpstr
    // spec-audit: vt:CT_Vector/lpwstr
    // spec-audit: vt:CT_Vector/bstr
    // spec-audit: vt:CT_Vector/date
    // spec-audit: vt:CT_Vector/filetime
    // spec-audit: vt:CT_Vector/bool
    // spec-audit: vt:CT_Vector/cy
    // spec-audit: vt:CT_Vector/error
    // spec-audit: vt:CT_Vector/clsid
    pub items: Vec<Variant>,
}

/// `vt:CT_Array`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VtArray {
    /// `@lBounds`.
    pub lower_bound: i32,
    /// `@uBounds`.
    pub upper_bound: i32,
    /// `@baseType`.
    pub base_type: ArrayBaseType,
    /// The items.
    // spec-audit: vt:CT_Array/variant
    // spec-audit: vt:CT_Array/i1
    // spec-audit: vt:CT_Array/i2
    // spec-audit: vt:CT_Array/i4
    // spec-audit: vt:CT_Array/int
    // spec-audit: vt:CT_Array/ui1
    // spec-audit: vt:CT_Array/ui2
    // spec-audit: vt:CT_Array/ui4
    // spec-audit: vt:CT_Array/uint
    // spec-audit: vt:CT_Array/r4
    // spec-audit: vt:CT_Array/r8
    // spec-audit: vt:CT_Array/decimal
    // spec-audit: vt:CT_Array/bstr
    // spec-audit: vt:CT_Array/date
    // spec-audit: vt:CT_Array/bool
    // spec-audit: vt:CT_Array/error
    // spec-audit: vt:CT_Array/cy
    pub items: Vec<Variant>,
}

/// `vt:CT_Variant`, the value of a custom property or vector item. Text forms are
/// checked against their type on import and held as written; binary forms stay
/// base64 and are never decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Variant {
    /// `vt:variant`
    Variant(Box<Variant>),
    /// `vt:vector`
    Vector(VtVector),
    /// `vt:array`
    Array(VtArray),
    /// `vt:blob`
    Blob(String),
    /// `vt:oblob`
    OBlob(String),
    /// `vt:empty`
    Empty,
    /// `vt:null`
    Null,
    /// `vt:i1`
    I1(i8),
    /// `vt:i2`
    I2(i16),
    /// `vt:i4`
    I4(i32),
    /// `vt:i8`
    I8(i64),
    /// `vt:int`
    Int(i32),
    /// `vt:ui1`
    Ui1(u8),
    /// `vt:ui2`
    Ui2(u16),
    /// `vt:ui4`
    Ui4(u32),
    /// `vt:ui8`
    Ui8(u64),
    /// `vt:uint`
    Uint(u32),
    /// `vt:r4`
    R4(Float32),
    /// `vt:r8`
    R8(Float64),
    /// `vt:decimal`
    Decimal(String),
    /// `vt:lpstr`
    Lpstr(String),
    /// `vt:lpwstr`
    Lpwstr(String),
    /// `vt:bstr`
    Bstr(String),
    /// `vt:date`
    Date(String),
    /// `vt:filetime`
    Filetime(String),
    /// `vt:bool`
    Bool(bool),
    /// `vt:cy`
    Cy(String),
    /// `vt:error`
    Error(String),
    /// `vt:stream`
    Stream(String),
    /// `vt:ostream`
    OStream(String),
    /// `vt:storage`
    Storage(String),
    /// `vt:ostorage`
    OStorage(String),
    /// `vt:vstream`
    Vstream {
        /// `@version`.
        version: Option<String>,
        /// The base64 content.
        data: String,
    },
    /// `vt:clsid`
    Clsid(String),
}

/// `ep:CT_Properties`, the extended properties.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppProperties {
    /// `Template`.
    pub template: Option<String>,
    /// `Manager`.
    pub manager: Option<String>,
    /// `Company`.
    pub company: Option<String>,
    /// `Pages`.
    pub pages: Option<i32>,
    /// `Words`.
    pub words: Option<i32>,
    /// `Characters`.
    pub characters: Option<i32>,
    /// `PresentationFormat`.
    pub presentation_format: Option<String>,
    /// `Lines`.
    pub lines: Option<i32>,
    /// `Paragraphs`.
    pub paragraphs: Option<i32>,
    /// `Slides`.
    pub slides: Option<i32>,
    /// `Notes`.
    pub notes: Option<i32>,
    /// `TotalTime`.
    pub total_time: Option<i32>,
    /// `HiddenSlides`.
    pub hidden_slides: Option<i32>,
    /// `MMClips`.
    pub mm_clips: Option<i32>,
    /// `ScaleCrop`.
    pub scale_crop: Option<bool>,
    /// `HeadingPairs`.
    pub heading_pairs: Option<VtVector>,
    /// `TitlesOfParts`.
    pub titles_of_parts: Option<VtVector>,
    /// `LinksUpToDate`.
    pub links_up_to_date: Option<bool>,
    /// `CharactersWithSpaces`.
    pub characters_with_spaces: Option<i32>,
    /// `SharedDoc`.
    pub shared_doc: Option<bool>,
    /// `HyperlinkBase`.
    pub hyperlink_base: Option<String>,
    /// `HLinks`.
    pub hlinks: Option<VtVector>,
    /// `HyperlinksChanged`.
    pub hyperlinks_changed: Option<bool>,
    /// `DigSig`: the base64 content of its `vt:blob`.
    // spec-audit: ep:CT_DigSigBlob/vt:blob
    pub dig_sig: Option<String>,
    /// `Application`.
    pub application: Option<String>,
    /// `AppVersion`.
    pub app_version: Option<String>,
    /// `DocSecurity`.
    pub doc_security: Option<i32>,
}

/// `cp:CT_Property`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomProperty {
    /// `@fmtid`.
    pub fmtid: String,
    /// `@pid`, which must be unique in the part.
    pub pid: i32,
    /// `@name`.
    pub name: Option<String>,
    /// `@linkTarget`.
    pub link_target: Option<String>,
    /// The single value child. `None` when it was absent or unusable.
    // spec-audit: ep:CT_VectorVariant/vt:vector
    // spec-audit: ep:CT_VectorLpstr/vt:vector
    // spec-audit: cp:CT_Property/vt:vector
    // spec-audit: cp:CT_Property/vt:array
    // spec-audit: cp:CT_Property/vt:blob
    // spec-audit: cp:CT_Property/vt:oblob
    // spec-audit: cp:CT_Property/vt:empty
    // spec-audit: cp:CT_Property/vt:null
    // spec-audit: cp:CT_Property/vt:i1
    // spec-audit: cp:CT_Property/vt:i2
    // spec-audit: cp:CT_Property/vt:i4
    // spec-audit: cp:CT_Property/vt:i8
    // spec-audit: cp:CT_Property/vt:int
    // spec-audit: cp:CT_Property/vt:ui1
    // spec-audit: cp:CT_Property/vt:ui2
    // spec-audit: cp:CT_Property/vt:ui4
    // spec-audit: cp:CT_Property/vt:ui8
    // spec-audit: cp:CT_Property/vt:uint
    // spec-audit: cp:CT_Property/vt:r4
    // spec-audit: cp:CT_Property/vt:r8
    // spec-audit: cp:CT_Property/vt:decimal
    // spec-audit: cp:CT_Property/vt:lpstr
    // spec-audit: cp:CT_Property/vt:lpwstr
    // spec-audit: cp:CT_Property/vt:bstr
    // spec-audit: cp:CT_Property/vt:date
    // spec-audit: cp:CT_Property/vt:filetime
    // spec-audit: cp:CT_Property/vt:bool
    // spec-audit: cp:CT_Property/vt:cy
    // spec-audit: cp:CT_Property/vt:error
    // spec-audit: cp:CT_Property/vt:stream
    // spec-audit: cp:CT_Property/vt:ostream
    // spec-audit: cp:CT_Property/vt:storage
    // spec-audit: cp:CT_Property/vt:ostorage
    // spec-audit: cp:CT_Property/vt:vstream
    // spec-audit: cp:CT_Property/vt:clsid
    pub value: Option<Variant>,
}

/// `cp:CT_Properties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomProperties {
    /// `property`, in file order.
    pub properties: Vec<CustomProperty>,
}

/// The three property parts. `None` means the part is absent or unreadable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentProperties {
    /// `docProps/core.xml`, usually.
    pub core: Option<CoreProperties>,
    /// `docProps/app.xml`, usually.
    pub app: Option<AppProperties>,
    /// `docProps/custom.xml`, usually.
    pub custom: Option<CustomProperties>,
}
