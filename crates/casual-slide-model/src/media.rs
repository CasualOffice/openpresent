//! Audio, video and the parts that hold pictures and media (`docs/spec/MEDIA.md`).
//!
//! Every reference is a relationship id: the engine never parses media (`SEC-C06`) and
//! never fetches a target (`SEC-N01`). A media part is known by its [`ContentHash`], and
//! several relationships can share it, so nothing here can name "the" owner of a part.
//!
//! The poster frame of a video is the blip of the `p:pic` it sits on (`MS-PPTX` "Media").
//! [`ShapeNode::poster`](crate::shape::ShapeNode::poster) answers it.
//!
//! `MS-OI29500` v20260818, 20.1.3.2 and 20.1.3.6: Office ignores `contentType` on
//! `audioFile` and `videoFile` and does not round trip it. It is kept here all the same,
//! since nothing the file says is dropped.

use std::fmt;

use crate::properties::token_enum;
use crate::shape::MediaKind;

/// `a:CT_AudioFile`, `a:CT_VideoFile` and `a:CT_QuickTimeFile`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaFile {
    /// `@r:link`, which the schema requires. An internal or an external relationship.
    pub link: Option<String>,
    /// `@contentType`. `a:CT_QuickTimeFile` has none.
    pub content_type: Option<String>,
}

/// `a:CT_EmbeddedWAVAudioFile`, as `wavAudioFile` and as a hyperlink's `snd`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EmbeddedWav {
    /// `@r:embed`, which the schema requires.
    pub embed: Option<String>,
    /// `@name`.
    pub name: Option<String>,
}

/// `a:CT_AudioCDTime`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AudioCdTime {
    /// `@track`, which the schema requires.
    pub track: Option<u8>,
    /// `@time`. The schema default, 0, is not filled in.
    pub time: Option<u32>,
}

/// `a:CT_AudioCD`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AudioCd {
    /// `st`.
    pub start: Option<AudioCdTime>,
    /// `end`.
    pub end: Option<AudioCdTime>,
}

/// `a:EG_Media`: the one media child `nvPr` holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Media {
    /// `audioCd`.
    AudioCd(AudioCd),
    /// `wavAudioFile`.
    WavAudioFile(EmbeddedWav),
    /// `audioFile`.
    AudioFile(MediaFile),
    /// `videoFile`.
    VideoFile(MediaFile),
    /// `quickTimeFile`.
    QuickTimeFile(MediaFile),
}

impl Media {
    /// Which element it is.
    pub fn kind(&self) -> MediaKind {
        match self {
            Self::AudioCd(_) => MediaKind::AudioCd,
            Self::WavAudioFile(_) => MediaKind::WavAudioFile,
            Self::AudioFile(_) => MediaKind::AudioFile,
            Self::VideoFile(_) => MediaKind::VideoFile,
            Self::QuickTimeFile(_) => MediaKind::QuickTimeFile,
        }
    }
}

/// `p14:CT_MediaTrim`. Times are `ST_UniversalTimeOffset`, a string in the schema and
/// milliseconds in `MS-PPTX`'s example, and are kept as written.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaTrim {
    /// `@st`.
    pub start: Option<String>,
    /// `@end`.
    pub end: Option<String>,
}

/// `p14:CT_MediaFade`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaFade {
    /// `@in`.
    pub fade_in: Option<String>,
    /// `@out`.
    pub fade_out: Option<String>,
}

/// `p14:CT_MediaBookmark`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaBookmark {
    /// `@name`.
    pub name: Option<String>,
    /// `@time`.
    pub time: Option<String>,
}

token_enum!(
    /// `p173:ST_DisplayLocation`.
    DisplayLocation {
        Media = "media",
        Slide = "slide",
    }
);

/// `p173:CT_Track`: a caption file, itself a relationship.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaTrack {
    /// `@id`, a GUID.
    pub id: Option<String>,
    /// `@label`.
    pub label: Option<String>,
    /// `@lang`.
    pub lang: Option<String>,
    /// `@r:embed`.
    pub embed: Option<String>,
    /// `@r:link`. Never followed.
    pub link: Option<String>,
}

/// `p173:CT_TracksInfo`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TracksInfo {
    /// `@displayLoc`, which the schema requires.
    pub display_location: Option<DisplayLocation>,
    /// `trackLst`. An absent list and an empty one differ.
    pub tracks: Option<Vec<MediaTrack>>,
}

/// `p14:CT_Media`, from `nvPr/p:extLst`: where PowerPoint keeps embedded media, its
/// trim, fade and bookmarks. Beside `a:videoFile` or `a:audioFile`, not instead of it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaExtension {
    /// `@r:embed`.
    pub embed: Option<String>,
    /// `@r:link`. Never followed.
    pub link: Option<String>,
    /// `trim`.
    pub trim: Option<MediaTrim>,
    /// `fade`.
    pub fade: Option<MediaFade>,
    /// `bmkLst`. An absent list and an empty one differ.
    pub bookmarks: Option<Vec<MediaBookmark>>,
    /// `extLst/ext/p173:tracksInfo`.
    pub tracks: Option<TracksInfo>,
}

/// SHA-256 of a part's bytes: how the model knows a picture or a clip, whatever part
/// name or relationship it arrived under (`docs/22`, `INV-SP-03`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash(pub [u8; 32]);

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|b| write!(f, "{b:02x}"))
    }
}

/// What a part's first bytes say it is. SVG is never in this list: it is known only by
/// its declared content type, because nothing looks inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageFormat {
    /// PNG.
    Png,
    /// JPEG.
    Jpeg,
    /// GIF.
    Gif,
    /// BMP.
    Bmp,
    /// TIFF.
    Tiff,
    /// WebP.
    Webp,
    /// Enhanced Metafile.
    Emf,
    /// Windows Metafile with a placeable header.
    Wmf,
}

/// An image's size in pixels, read from its header and not decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageDimensions {
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
}

impl ImageDimensions {
    /// Width times height, which `u64` cannot overflow.
    pub fn pixels(self) -> u64 {
        u64::from(self.width).saturating_mul(u64::from(self.height))
    }
}

/// What kind of thing a relationship reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediaClass {
    /// A raster or metafile image.
    Image,
    /// An SVG: retained, unparsed, and never placed anywhere (`SEC-C03`).
    Svg,
    /// Audio.
    Audio,
    /// Video, or a clip whose type does not say which.
    Video,
    /// A caption track.
    Track,
}

/// One relationship reaching a media part or an external target.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MediaReferrer {
    /// The part holding the relationship.
    pub part: String,
    /// `Id`.
    pub rel_id: String,
}

/// One part holding a picture or a clip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaPart {
    /// The part's name.
    pub part: String,
    /// What it reaches, from its relationship type and declared content type.
    pub class: MediaClass,
    /// The declared content type.
    pub content_type: Option<String>,
    /// `None` if the part was not read: over a bound, or missing.
    pub hash: Option<ContentHash>,
    /// Bytes, if read.
    pub size: Option<u64>,
    /// What the first bytes say. Always `None` for an SVG, audio and video.
    pub format: Option<ImageFormat>,
    /// From the header, where one could be read in bounds without decoding.
    pub dimensions: Option<ImageDimensions>,
    /// Every relationship reaching it, sorted. More than one means it is shared: an edit
    /// must not overwrite it, and it goes only with its last (`INV-SP-02`, `INV-SP-04`).
    pub referrers: Vec<MediaReferrer>,
}

impl MediaPart {
    /// Whether more than one relationship reaches it.
    pub fn is_shared(&self) -> bool {
        self.referrers.len() > 1
    }

    /// Whether anything else reaches it once this one relationship is gone.
    pub fn outlives(&self, part: &str, rel_id: &str) -> bool {
        self.referrers
            .iter()
            .any(|r| !(r.part == part && r.rel_id == rel_id))
    }
}

/// A picture or clip outside the package. Never fetched, and always listed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExternalMedia {
    /// The relationship.
    pub referrer: MediaReferrer,
    /// What it reaches, by relationship type.
    pub class: MediaClass,
}

/// A deck's media, read on request and never by opening (`docs/28`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaCatalog {
    /// Parts by name, sorted.
    pub parts: Vec<MediaPart>,
    /// External targets, sorted.
    pub external: Vec<ExternalMedia>,
}

impl MediaCatalog {
    /// A part by name.
    pub fn part(&self, name: &str) -> Option<&MediaPart> {
        self.parts.iter().find(|p| p.part == name)
    }

    /// The part already holding these bytes, for an insert to reuse (`INV-SP-03`).
    pub fn with_hash(&self, hash: ContentHash) -> Option<&MediaPart> {
        self.parts.iter().find(|p| p.hash == Some(hash))
    }

    /// What a relationship reaches, by source part and `Id`.
    pub fn reached_by(&self, part: &str, rel_id: &str) -> Option<&MediaPart> {
        self.parts.iter().find(|p| {
            p.referrers
                .iter()
                .any(|r| r.part == part && r.rel_id == rel_id)
        })
    }
}
