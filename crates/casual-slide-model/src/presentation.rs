//! The presentation part, its properties and saved view state, as plain typed
//! values (`docs/spec/PRES.md`, `docs/spec/VIEW.md`).
//!
//! `Option` means the file did not state a value, or stated one that could not be
//! used; the import report says which. A schema default is never filled in, so
//! absent stays absent. Relationship ids are held as written: what they point at
//! belongs to the part graph.

use crate::properties::{DocumentProperties, Lexical, token_enum, trim_xml};

/// Smallest `ST_SlideId`.
pub const SLIDE_ID_MIN: u32 = 256;
/// Largest `ST_SlideId`.
pub const SLIDE_ID_MAX: u32 = 2_147_483_647;
/// Smallest `ST_SlideMasterId`.
pub const SLIDE_MASTER_ID_MIN: u32 = 2_147_483_648;
/// Smallest `ST_SlideSizeCoordinate`.
pub const SLIDE_SIZE_MIN: i32 = 914_400;
/// Largest `ST_SlideSizeCoordinate`.
pub const SLIDE_SIZE_MAX: i32 = 51_206_400;
/// Largest `ST_PositiveCoordinate`.
pub const POSITIVE_COORDINATE_MAX: i64 = 27_273_042_316_900;
/// Smallest `ST_CoordinateUnqualified`.
pub const COORDINATE_MIN: i64 = -27_273_042_329_600;

/// A percentage in thousandths, `50000` for 50%. Reads both forms of the union
/// in `a:ST_Percentage`, the bare integer and `50%`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percentage(pub i32);

impl Percentage {
    /// The `N%` form, which Strict requires.
    pub fn to_percent_form(self) -> String {
        let sign = if self.0 < 0 { "-" } else { "" };
        let v = self.0.unsigned_abs();
        match v % 1000 {
            0 => format!("{sign}{}%", v / 1000),
            frac => {
                let text = format!("{sign}{}.{frac:03}", v / 1000);
                format!("{}%", text.trim_end_matches('0'))
            }
        }
    }
}

impl Lexical for Percentage {
    fn parse_lexical(s: &str) -> Option<Self> {
        let s = trim_xml(s);
        let Some(number) = s.strip_suffix('%') else {
            return s.parse().ok().map(Self);
        };
        let (negative, number) = match number.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, number),
        };
        let (whole, frac) = number.split_once('.').unwrap_or((number, ""));
        let all_digits = |t: &str| t.bytes().all(|b| b.is_ascii_digit());
        if whole.is_empty()
            || !all_digits(whole)
            || !all_digits(frac)
            || (number.contains('.') && frac.is_empty())
        {
            return None;
        }
        let mut thousandths = whole.parse::<i64>().ok()?.checked_mul(1000)?;
        // Beyond a thousandth is dropped, as nothing here can hold it.
        for (place, b) in frac.bytes().take(3).enumerate() {
            let unit = [100, 10, 1][place];
            thousandths = thousandths.checked_add(i64::from(b - b'0') * unit)?;
        }
        let signed = if negative { -thousandths } else { thousandths };
        i32::try_from(signed).ok().map(Self)
    }
    fn to_lexical(&self) -> String {
        self.0.to_string()
    }
}

/// A length in EMU. Reads `a:ST_Coordinate`: the bare integer, or a number and a
/// unit such as `2.5in`, which is rounded to the nearest EMU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Coordinate(pub i64);

impl Lexical for Coordinate {
    fn parse_lexical(s: &str) -> Option<Self> {
        let s = trim_xml(s);
        if let Ok(v) = s.parse::<i64>() {
            return Some(Self(v));
        }
        let split = s.len().checked_sub(2)?;
        let (number, unit) = (s.get(..split)?, s.get(split..)?);
        let per_unit = match unit {
            "mm" => 36_000.0,
            "cm" => 360_000.0,
            "in" => 914_400.0,
            "pt" => 12_700.0,
            "pc" | "pi" => 152_400.0,
            _ => return None,
        };
        let body = number.strip_prefix('-').unwrap_or(number);
        if body.is_empty()
            || body.starts_with('.')
            || body.ends_with('.')
            || !body.bytes().all(|b| b.is_ascii_digit() || b == b'.')
        {
            return None;
        }
        let emu = (number.parse::<f64>().ok()? * per_unit).round();
        // `as` saturates, and anything that large is outside every coordinate range.
        (emu.is_finite() && emu.abs() < 9.0e18).then_some(Self(emu as i64))
    }
    fn to_lexical(&self) -> String {
        self.0.to_string()
    }
}

token_enum!(
    /// `s:ST_ConformanceClass`.
    ConformanceClass {
        Strict = "strict",
        Transitional = "transitional",
    }
);

token_enum!(
    /// `p:ST_SlideSizeType`.
    SlideSizeType {
        Screen4x3 = "screen4x3",
        Letter = "letter",
        A4 = "A4",
        Mm35 = "35mm",
        Overhead = "overhead",
        Banner = "banner",
        Custom = "custom",
        Ledger = "ledger",
        A3 = "A3",
        B4Iso = "B4ISO",
        B5Iso = "B5ISO",
        B4Jis = "B4JIS",
        B5Jis = "B5JIS",
        HagakiCard = "hagakiCard",
        Screen16x9 = "screen16x9",
        Screen16x10 = "screen16x10",
    }
);

token_enum!(
    /// `p:ST_PhotoAlbumLayout`.
    PhotoAlbumLayout {
        FitToSlide = "fitToSlide",
        Pic1 = "1pic",
        Pic2 = "2pic",
        Pic4 = "4pic",
        Pic1Title = "1picTitle",
        Pic2Title = "2picTitle",
        Pic4Title = "4picTitle",
    }
);

token_enum!(
    /// `p:ST_PhotoAlbumFrameShape`.
    PhotoAlbumFrameShape {
        FrameStyle1 = "frameStyle1",
        FrameStyle2 = "frameStyle2",
        FrameStyle3 = "frameStyle3",
        FrameStyle4 = "frameStyle4",
        FrameStyle5 = "frameStyle5",
        FrameStyle6 = "frameStyle6",
        FrameStyle7 = "frameStyle7",
    }
);

token_enum!(
    /// `s:ST_CryptProv`.
    CryptProvider {
        RsaAes = "rsaAES",
        RsaFull = "rsaFull",
        Custom = "custom",
    }
);

token_enum!(
    /// `s:ST_AlgClass`.
    AlgorithmClass {
        Hash = "hash",
        Custom = "custom",
    }
);

token_enum!(
    /// `s:ST_AlgType`.
    AlgorithmType {
        TypeAny = "typeAny",
        Custom = "custom",
    }
);

token_enum!(
    /// `p:ST_WebColorType`.
    WebColorType {
        None = "none",
        Browser = "browser",
        PresentationText = "presentationText",
        PresentationAccent = "presentationAccent",
        WhiteTextOnBlack = "whiteTextOnBlack",
        BlackTextOnWhite = "blackTextOnWhite",
    }
);

token_enum!(
    /// `p:ST_WebScreenSize`.
    WebScreenSize {
        S544x376 = "544x376",
        S640x480 = "640x480",
        S720x512 = "720x512",
        S800x600 = "800x600",
        S1024x768 = "1024x768",
        S1152x882 = "1152x882",
        S1152x900 = "1152x900",
        S1280x1024 = "1280x1024",
        S1600x1200 = "1600x1200",
        S1800x1400 = "1800x1400",
        S1920x1200 = "1920x1200",
    }
);

token_enum!(
    /// `p:ST_PrintWhat`.
    PrintWhat {
        Slides = "slides",
        Handouts1 = "handouts1",
        Handouts2 = "handouts2",
        Handouts3 = "handouts3",
        Handouts4 = "handouts4",
        Handouts6 = "handouts6",
        Handouts9 = "handouts9",
        Notes = "notes",
        Outline = "outline",
    }
);

token_enum!(
    /// `p:ST_PrintColorMode`.
    PrintColorMode {
        Bw = "bw",
        Gray = "gray",
        Clr = "clr",
    }
);

token_enum!(
    /// `p:ST_SplitterBarState`.
    SplitterBarState {
        Minimized = "minimized",
        Restored = "restored",
        Maximized = "maximized",
    }
);

token_enum!(
    /// `p:ST_ViewType`.
    ViewType {
        SldView = "sldView",
        SldMasterView = "sldMasterView",
        NotesView = "notesView",
        HandoutView = "handoutView",
        NotesMasterView = "notesMasterView",
        OutlineView = "outlineView",
        SldSorterView = "sldSorterView",
        SldThumbnailView = "sldThumbnailView",
    }
);

token_enum!(
    /// `p:ST_Direction`.
    Direction {
        Horz = "horz",
        Vert = "vert",
    }
);

/// `p:CT_SlideIdListEntry`. An `id` outside `ST_SlideId` is kept and reported, as
/// the entry still names a real slide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideId {
    /// `@id`.
    pub id: u32,
    /// `@r:id`.
    pub rel_id: String,
}

/// `p:CT_SlideMasterIdListEntry`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideMasterId {
    /// `@id`, which is optional.
    pub id: Option<u32>,
    /// `@r:id`.
    pub rel_id: String,
}

/// `p:sldSz`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlideSize {
    /// `@cx`, EMU, 914400 to 51206400.
    pub cx: i32,
    /// `@cy`, EMU, 914400 to 51206400.
    pub cy: i32,
    /// `@type`.
    pub size_type: Option<SlideSizeType>,
}

/// `a:CT_PositiveSize2D`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositiveSize {
    /// `@cx`, EMU.
    pub cx: i64,
    /// `@cy`, EMU.
    pub cy: i64,
}

/// `a:CT_TextFont`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextFont {
    /// `@typeface`.
    pub typeface: String,
    /// `@panose`, 20 hex digits as written.
    pub panose: Option<String>,
    /// `@pitchFamily`.
    pub pitch_family: Option<i8>,
    /// `@charset`.
    pub charset: Option<i8>,
}

/// `p:CT_EmbeddedFontListEntry`. The ids only: a font file is never read here (`SEC-C05`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EmbeddedFont {
    /// `font`.
    pub font: TextFont,
    /// `regular/@r:id`.
    pub regular: Option<String>,
    /// `bold/@r:id`.
    pub bold: Option<String>,
    /// `italic/@r:id`.
    pub italic: Option<String>,
    /// `boldItalic/@r:id`.
    pub bold_italic: Option<String>,
}

/// `p:CT_CustomShow`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomShow {
    /// `@name`.
    pub name: String,
    /// `@id`.
    pub id: u32,
    /// `sldLst/sld/@r:id`, in show order.
    pub slides: Vec<String>,
}

/// `p:CT_PhotoAlbum`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhotoAlbum {
    /// `@bw`.
    pub bw: Option<bool>,
    /// `@showCaptions`.
    pub show_captions: Option<bool>,
    /// `@layout`.
    pub layout: Option<PhotoAlbumLayout>,
    /// `@frame`.
    pub frame: Option<PhotoAlbumFrameShape>,
}

/// `p:CT_CustomerDataList`: references only. What they point at is Tier 1M.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomerDataList {
    /// `custData/@r:id`.
    pub customer_data: Vec<String>,
    /// `tags/@r:id`.
    pub tags: Option<String>,
}

/// `p:CT_Kinsoku`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Kinsoku {
    /// `@lang`.
    pub lang: Option<String>,
    /// `@invalStChars`.
    pub invalid_start_chars: String,
    /// `@invalEndChars`.
    pub invalid_end_chars: String,
}

/// `p:CT_ModifyVerifier`. Held, never checked: this engine neither sets nor
/// enforces a modify password.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModifyVerifier {
    /// `@algorithmName`.
    pub algorithm_name: Option<String>,
    /// `@hashValue`.
    pub hash_value: Option<String>,
    /// `@saltValue`.
    pub salt_value: Option<String>,
    /// `@spinValue`.
    pub spin_value: Option<u32>,
    /// `@cryptProviderType`.
    pub crypt_provider_type: Option<CryptProvider>,
    /// `@cryptAlgorithmClass`.
    pub crypt_algorithm_class: Option<AlgorithmClass>,
    /// `@cryptAlgorithmType`.
    pub crypt_algorithm_type: Option<AlgorithmType>,
    /// `@cryptAlgorithmSid`.
    pub crypt_algorithm_sid: Option<u32>,
    /// `@spinCount`.
    pub spin_count: Option<u32>,
    /// `@saltData`.
    pub salt_data: Option<String>,
    /// `@hashData`.
    pub hash_data: Option<String>,
    /// `@cryptProvider`.
    pub crypt_provider: Option<String>,
    /// `@algIdExt`.
    pub alg_id_ext: Option<u32>,
    /// `@algIdExtSource`.
    pub alg_id_ext_source: Option<String>,
    /// `@cryptProviderTypeExt`.
    pub crypt_provider_type_ext: Option<u32>,
    /// `@cryptProviderTypeExtSource`.
    pub crypt_provider_type_ext_source: Option<String>,
}

/// `p:CT_Presentation`. `defaultTextStyle` and `extLst` stay in the tree, and are
/// reported as retained; neither has a value here until Tier 1F and 1M.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Presentation {
    /// `@serverZoom`.
    pub server_zoom: Option<Percentage>,
    /// `@firstSlideNum`.
    pub first_slide_num: Option<i32>,
    /// `@showSpecialPlsOnTitleSld`.
    pub show_special_pls_on_title_sld: Option<bool>,
    /// `@rtl`.
    pub rtl: Option<bool>,
    /// `@removePersonalInfoOnSave`.
    pub remove_personal_info_on_save: Option<bool>,
    /// `@compatMode`.
    pub compat_mode: Option<bool>,
    /// `@strictFirstAndLastChars`.
    pub strict_first_and_last_chars: Option<bool>,
    /// `@embedTrueTypeFonts`.
    pub embed_true_type_fonts: Option<bool>,
    /// `@saveSubsetFonts`.
    pub save_subset_fonts: Option<bool>,
    /// `@autoCompressPictures`.
    pub auto_compress_pictures: Option<bool>,
    /// `@bookmarkIdSeed`, 1 to 2147483647.
    pub bookmark_id_seed: Option<u32>,
    /// `@conformance`.
    pub conformance: Option<ConformanceClass>,
    /// `sldMasterIdLst`.
    pub slide_masters: Option<Vec<SlideMasterId>>,
    /// `notesMasterIdLst/notesMasterId/@r:id`.
    pub notes_masters: Option<Vec<String>>,
    /// `handoutMasterIdLst/handoutMasterId/@r:id`.
    pub handout_masters: Option<Vec<String>>,
    /// `sldIdLst`.
    pub slides: Option<Vec<SlideId>>,
    /// `sldSz`.
    pub slide_size: Option<SlideSize>,
    /// `notesSz`.
    pub notes_size: Option<PositiveSize>,
    /// `smartTags/@r:id`.
    pub smart_tags: Option<String>,
    /// `embeddedFontLst`.
    pub embedded_fonts: Option<Vec<EmbeddedFont>>,
    /// `custShowLst`.
    pub custom_shows: Option<Vec<CustomShow>>,
    /// `photoAlbum`.
    pub photo_album: Option<PhotoAlbum>,
    /// `custDataLst`.
    pub customer_data: Option<CustomerDataList>,
    /// `kinsoku`.
    pub kinsoku: Option<Kinsoku>,
    /// `modifyVerifier`.
    pub modify_verifier: Option<ModifyVerifier>,
}

/// `p:EG_SlideListChoice`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideListChoice {
    /// `sldAll`.
    All,
    /// `sldRg`.
    Range {
        /// `@st`.
        start: u32,
        /// `@end`.
        end: u32,
    },
    /// `custShow/@id`.
    CustomShow(u32),
}

/// `p:CT_HtmlPublishProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HtmlPublishProperties {
    /// `@showSpeakerNotes`.
    pub show_speaker_notes: Option<bool>,
    /// `@target`.
    pub target: Option<String>,
    /// `@title`.
    pub title: Option<String>,
    /// `@r:id`.
    pub rel_id: Option<String>,
    /// The slide list.
    pub slide_list: Option<SlideListChoice>,
}

/// `p:CT_WebProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WebProperties {
    /// `@showAnimation`.
    pub show_animation: Option<bool>,
    /// `@resizeGraphics`.
    pub resize_graphics: Option<bool>,
    /// `@allowPng`.
    pub allow_png: Option<bool>,
    /// `@relyOnVml`.
    pub rely_on_vml: Option<bool>,
    /// `@organizeInFolders`.
    pub organize_in_folders: Option<bool>,
    /// `@useLongFilenames`.
    pub use_long_filenames: Option<bool>,
    /// `@imgSz`.
    pub image_size: Option<WebScreenSize>,
    /// `@encoding`.
    pub encoding: Option<String>,
    /// `@clr`.
    pub color: Option<WebColorType>,
}

/// `p:CT_PrintProperties`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PrintProperties {
    /// `@prnWhat`.
    pub print_what: Option<PrintWhat>,
    /// `@clrMode`.
    pub color_mode: Option<PrintColorMode>,
    /// `@hiddenSlides`.
    pub hidden_slides: Option<bool>,
    /// `@scaleToFitPaper`.
    pub scale_to_fit_paper: Option<bool>,
    /// `@frameSlides`.
    pub frame_slides: Option<bool>,
}

/// `p:EG_ShowType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShowType {
    /// `present`.
    Present,
    /// `browse`.
    Browse {
        /// `@showScrollbar`.
        show_scrollbar: Option<bool>,
    },
    /// `kiosk`.
    Kiosk {
        /// `@restart`.
        restart: Option<u32>,
    },
}

/// `p:CT_ShowProperties`. `penClr` stays in the tree, reported as retained.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShowProperties {
    /// `@loop`.
    pub loop_show: Option<bool>,
    /// `@showNarration`.
    pub show_narration: Option<bool>,
    /// `@showAnimation`.
    pub show_animation: Option<bool>,
    /// `@useTimings`.
    pub use_timings: Option<bool>,
    /// The show type.
    pub show_type: Option<ShowType>,
    /// The slide list.
    pub slide_list: Option<SlideListChoice>,
}

/// `p:CT_PresentationProperties`. `clrMru` and `extLst` stay in the tree, reported.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PresentationProperties {
    /// `htmlPubPr`.
    pub html_publish: Option<HtmlPublishProperties>,
    /// `webPr`.
    pub web: Option<WebProperties>,
    /// `prnPr`.
    pub print: Option<PrintProperties>,
    /// `showPr`.
    pub show: Option<ShowProperties>,
}

/// `p:CT_NormalViewPortion`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalViewPortion {
    /// `@sz`, 0 to 100000 thousandths.
    pub size: Percentage,
    /// `@autoAdjust`.
    pub auto_adjust: Option<bool>,
}

/// `p:CT_NormalViewProperties`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NormalViewProperties {
    /// `@showOutlineIcons`.
    pub show_outline_icons: Option<bool>,
    /// `@snapVertSplitter`.
    pub snap_vert_splitter: Option<bool>,
    /// `@vertBarState`.
    pub vert_bar_state: Option<SplitterBarState>,
    /// `@horzBarState`.
    pub horz_bar_state: Option<SplitterBarState>,
    /// `@preferSingleView`.
    pub prefer_single_view: Option<bool>,
    /// `restoredLeft`.
    pub restored_left: Option<NormalViewPortion>,
    /// `restoredTop`.
    pub restored_top: Option<NormalViewPortion>,
}

/// `a:CT_Ratio`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ratio {
    /// `@n`.
    pub n: i64,
    /// `@d`, unchecked: nothing here divides by it.
    pub d: i64,
}

/// `a:CT_Scale2D`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scale {
    /// `sx`.
    pub x: Ratio,
    /// `sy`.
    pub y: Ratio,
}

/// `a:CT_Point2D`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    /// `@x`.
    pub x: Coordinate,
    /// `@y`.
    pub y: Coordinate,
}

/// `p:CT_CommonViewProperties`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CommonViewProperties {
    /// `@varScale`.
    pub var_scale: Option<bool>,
    /// `scale`.
    pub scale: Option<Scale>,
    /// `origin`.
    pub origin: Option<Point>,
}

/// `p:CT_Guide`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Guide {
    /// `@orient`.
    pub orient: Option<Direction>,
    /// `@pos`.
    pub pos: Option<Coordinate>,
}

/// `p:CT_CommonSlideViewProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommonSlideViewProperties {
    /// `@snapToGrid`.
    pub snap_to_grid: Option<bool>,
    /// `@snapToObjects`.
    pub snap_to_objects: Option<bool>,
    /// `@showGuides`.
    pub show_guides: Option<bool>,
    /// `cViewPr`.
    pub common: Option<CommonViewProperties>,
    /// `guideLst`.
    pub guides: Option<Vec<Guide>>,
}

/// `p:CT_OutlineViewSlideEntry`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineSlide {
    /// `@r:id`.
    pub rel_id: String,
    /// `@collapse`.
    pub collapse: Option<bool>,
}

/// `p:CT_OutlineViewProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OutlineViewProperties {
    /// `cViewPr`.
    pub common: Option<CommonViewProperties>,
    /// `sldLst`.
    pub slides: Option<Vec<OutlineSlide>>,
}

/// `p:CT_SlideSorterViewProperties`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SorterViewProperties {
    /// `@showFormatting`.
    pub show_formatting: Option<bool>,
    /// `cViewPr`.
    pub common: Option<CommonViewProperties>,
}

/// `p:CT_ViewProperties`. `extLst` stays in the tree, reported as retained.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViewProperties {
    /// `@lastView`.
    pub last_view: Option<ViewType>,
    /// `@showComments`.
    pub show_comments: Option<bool>,
    /// `normalViewPr`.
    pub normal: Option<NormalViewProperties>,
    /// `slideViewPr/cSldViewPr`.
    pub slide: Option<CommonSlideViewProperties>,
    /// `outlineViewPr`.
    pub outline: Option<OutlineViewProperties>,
    /// `notesTextViewPr/cViewPr`.
    pub notes_text: Option<CommonViewProperties>,
    /// `sorterViewPr`.
    pub sorter: Option<SorterViewProperties>,
    /// `notesViewPr/cSldViewPr`.
    pub notes: Option<CommonSlideViewProperties>,
    /// `gridSpacing`.
    pub grid_spacing: Option<PositiveSize>,
}

/// What the model holds of a deck at this tier.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Deck {
    /// The main part.
    pub presentation: Presentation,
    /// `presProps`. `None` when absent or unreadable.
    pub presentation_properties: Option<PresentationProperties>,
    /// `viewProps`. `None` when absent or unreadable.
    pub view_properties: Option<ViewProperties>,
    /// The property parts.
    pub properties: DocumentProperties,
}
