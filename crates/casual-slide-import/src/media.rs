//! Read audio and video references (`docs/spec/MEDIA.md`), and list a deck's media.
//!
//! A reference is a relationship id, checked against the part's relationships and no
//! further. An external target is reported as `CS-MED-0001` and never followed
//! (`SEC-N01`); nothing here can follow one, since the engine has no network.
//!
//! [`catalog_media`] is the only thing that reads a media part, and only when asked:
//! opening a deck never does (`docs/28`). It hashes the bytes, and for a raster image
//! reads the header for its size, in bounds and without decoding (`SEC-C01`). An SVG is
//! hashed like any other part and otherwise not looked at: the XML walker never sees it,
//! so nothing in it can act (`SEC-C03`). Audio and video likewise (`SEC-C06`).
//!
//! Evidence: `MS-PPTX` "Media Part", "Track Part", "Media", "TracksInfo Extensions"; `MS-OI29500`
//! v20260818 20.1.3.2 and 20.1.3.6. The relationship types `a:audioFile`, `a:videoFile`
//! and `a:quickTimeFile` may use are Inferred from those and the standard's part list,
//! so any of `audio`, `video` and `media` is accepted on all three. The content types
//! compared with an image's first bytes, and the default bounds, are Inferred.

use std::collections::BTreeMap;

use casual_slide_model::media::{
    AudioCd, AudioCdTime, ContentHash, EmbeddedWav, ExternalMedia, ImageDimensions, ImageFormat,
    Media, MediaBookmark, MediaCatalog, MediaClass, MediaExtension, MediaFade, MediaFile,
    MediaPart, MediaReferrer, MediaTrack, MediaTrim, TracksInfo,
};
use casual_slide_ooxml::{
    PresentationPackage, Relationship, TargetMode, part_name_key, resolve_internal,
};

use crate::properties::{Elem, ImportError, ImportFindingKind as K, Reader, is_guid, ns};

/// `p14:`, PowerPoint 2010.
pub const P14: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2010/main"];
/// `p173:`, PowerPoint 2017, where caption tracks live.
pub const P173: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2017/3/main"];
/// `asvg:`, Office 2016 SVG.
pub const ASVG: &[&str] = &["http://schemas.microsoft.com/office/drawing/2016/SVG/main"];
/// `woe:`, online content.
pub const WOE: &[&str] = &["http://schemas.microsoft.com/office/word/2020/oembed"];
/// `aif:`, a picture's formula.
pub const AIF: &[&str] = &["http://schemas.microsoft.com/office/drawing/2022/imageformula"];

/// Bytes of a part looked through for an image header. A JPEG's size comes after its
/// other segments; past this it is not read, and stays unknown.
pub const MAX_HEADER_SCAN: usize = 1 << 20;

/// Bounds on media references and on [`catalog_media`]. Over one, what crossed it is
/// left out and reported, and stays in the tree or the package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaLimits {
    /// `bmk` in one `bmkLst`. The whole `p14:media` is left out over it. `CS-MED-0002`.
    pub max_bookmarks: usize,
    /// `track` in one `trackLst`. The whole `p14:media` is left out over it. `CS-MED-0002`.
    pub max_tracks: usize,
    /// Parts listed by one catalog. `CS-MED-0008`.
    pub max_catalog_parts: usize,
    /// Bytes read by one catalog. It stops well short of the package's budget, which is
    /// cumulative (`PKG-01`), so listing media cannot use up what editing needs.
    /// `CS-MED-0003`.
    pub max_catalog_bytes: u64,
    /// Pixels an image may declare before it is reported as too large to decode. Nothing
    /// decodes here; a renderer is to check this answer first (`SEC-C01`). `CS-MED-0004`.
    pub max_image_pixels: u64,
}

impl Default for MediaLimits {
    fn default() -> Self {
        Self {
            max_bookmarks: 4_096,
            max_tracks: 1_024,
            max_catalog_parts: 100_000,
            max_catalog_bytes: 2 << 30,
            max_image_pixels: 1 << 28,
        }
    }
}

/// What a media reader noticed, under its own codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MediaFinding {
    /// A picture, clip, track or online-content URL outside the package. Never fetched.
    ExternalTarget,
    /// A bookmark or track list over a [`MediaLimits`] bound. The whole `p14:media` is
    /// left out of the model, and stays in the tree.
    OverLimit,
    /// A part over the capped read, or the catalog's bytes spent. Not hashed.
    PartOverCap,
    /// An image declaring more pixels than [`MediaLimits::max_image_pixels`].
    OverPixelCap,
    /// A raster part whose first bytes are not what its content type says. A file
    /// claiming to be a PNG and holding an SVG is one. Nothing acts on it.
    FormatMismatch,
    /// A relationship's target is not in the package.
    MissingPart,
    /// `asvg:svgBlip` on a blip naming no picture to fall back on. Kept.
    SvgWithoutFallback,
    /// More parts than [`MediaLimits::max_catalog_parts`]. The rest are not listed.
    CatalogOverBound,
    /// A part that would not read for a reason other than a bound.
    UnreadablePart,
}

impl MediaFinding {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::ExternalTarget => "CS-MED-0001",
            Self::OverLimit => "CS-MED-0002",
            Self::PartOverCap => "CS-MED-0003",
            Self::OverPixelCap => "CS-MED-0004",
            Self::FormatMismatch => "CS-MED-0005",
            Self::MissingPart => "CS-MED-0006",
            Self::SvgWithoutFallback => "CS-MED-0007",
            Self::CatalogOverBound => "CS-MED-0008",
            Self::UnreadablePart => "CS-MED-0009",
        }
    }
}

/// What a relationship id should reach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Expect {
    /// `.../image`.
    Image,
    /// `.../audio`, `.../video` or `.../media`.
    Clip,
    /// `.../audio`.
    Wav,
    /// `.../media`.
    Media,
    /// `.../track`.
    Track,
}

impl Expect {
    fn allows(self, rel_type: &str) -> bool {
        let tail = rel_type.rsplit('/').next().unwrap_or_default();
        match self {
            Self::Image => tail == "image",
            Self::Clip => matches!(tail, "audio" | "video" | "media"),
            Self::Wav => tail == "audio",
            Self::Media => tail == "media",
            Self::Track => tail == "track",
        }
    }
}

/// Take a relationship attribute and check what it names. An `r:embed` reaching outside
/// the package is wrong; an `r:link` doing so is what a link is, and is reported. Neither
/// is followed. Both are kept, since the tree says so.
pub(crate) fn read_rel(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    e: &mut Elem,
    local: &'static str,
    expect: Expect,
) -> Option<String> {
    let id = e.take_ns(ns::REL, local)?;
    if id.is_empty() {
        return Some(id);
    }
    let Some(rel) = rels.iter().find(|r| r.id == id) else {
        r.note(K::DanglingRelationship, e, Some(local));
        return Some(id);
    };
    let external = rel.mode == TargetMode::External;
    if external {
        r.note(K::Media(MediaFinding::ExternalTarget), e, Some(local));
    }
    if !expect.allows(&rel.rel_type) || (external && local == "embed") {
        r.note(K::WrongRelationship, e, Some(local));
    }
    Some(id)
}

/// `ST_UniversalTimeOffset`'s form in `MS-PPTX`'s example: a decimal. The schema says
/// only that it is a string, so another form is reported and kept as written.
fn time(r: &mut Reader<'_>, e: &mut Elem, name: &str) -> Result<Option<String>, ImportError> {
    let value: Option<String> = r.attr(e, name)?;
    if let Some(v) = &value {
        let digits = v.strip_prefix(['+', '-']).unwrap_or(v);
        let mut pieces = digits.splitn(2, '.');
        let all_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        let whole = pieces.next().is_some_and(all_digits);
        if !whole || !pieces.next().is_none_or(all_digits) {
            r.note(K::MalformedValue, e, Some(name));
        }
    }
    Ok(value)
}

fn report_kids(r: &mut Reader<'_>, el: &Elem) -> Result<(), ImportError> {
    for kid in r.children(el)? {
        r.retained(&kid);
    }
    Ok(())
}

fn cd_time(r: &mut Reader<'_>, el: &Elem) -> Result<AudioCdTime, ImportError> {
    let mut e = el.clone();
    let time = AudioCdTime {
        track: r.required(&mut e, "track", |_: &u8| true)?,
        time: r.attr(&mut e, "time")?,
    };
    r.finish(&e);
    report_kids(r, el)?;
    Ok(time)
}

/// `a:CT_EmbeddedWAVAudioFile`, as `wavAudioFile` and `snd`.
pub(crate) fn read_wav(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    el: &Elem,
) -> Result<EmbeddedWav, ImportError> {
    let mut e = el.clone();
    let embed = read_rel(r, rels, &mut e, "embed", Expect::Wav);
    if embed.is_none() {
        r.note(K::MissingRequired, el, Some("r:embed"));
    }
    let wav = EmbeddedWav {
        embed,
        name: r.attr(&mut e, "name")?,
    };
    r.finish(&e);
    report_kids(r, el)?;
    Ok(wav)
}

/// The element of `a:EG_Media` this is, if it is one.
pub(crate) fn is_media_element(el: &Elem) -> bool {
    el.in_namespace(ns::DML)
        && matches!(
            el.local.as_str(),
            "audioCd" | "wavAudioFile" | "audioFile" | "videoFile" | "quickTimeFile"
        )
}

/// One element of `a:EG_Media`, which [`is_media_element`] has said it is.
pub(crate) fn read_media(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    el: &Elem,
) -> Result<Media, ImportError> {
    match el.local.as_str() {
        "audioCd" => {
            r.finish(el);
            let mut cd = AudioCd::default();
            for kid in r.children(el)? {
                let slot = match (kid.in_namespace(ns::DML), kid.local.as_str()) {
                    (true, "st") => &mut cd.start,
                    (true, "end") => &mut cd.end,
                    _ => {
                        r.retained(&kid);
                        continue;
                    }
                };
                if slot.is_some() {
                    r.note(K::DuplicateElement, &kid, None);
                } else {
                    *slot = Some(cd_time(r, &kid)?);
                }
            }
            Ok(Media::AudioCd(cd))
        }
        "wavAudioFile" => Ok(Media::WavAudioFile(read_wav(r, rels, el)?)),
        local => {
            let mut e = el.clone();
            let link = read_rel(r, rels, &mut e, "link", Expect::Clip);
            if link.is_none() {
                r.note(K::MissingRequired, el, Some("r:link"));
            }
            let file = MediaFile {
                link,
                content_type: match local {
                    "quickTimeFile" => None,
                    _ => r.attr(&mut e, "contentType")?,
                },
            };
            r.finish(&e);
            report_kids(r, el)?;
            Ok(match local {
                "audioFile" => Media::AudioFile(file),
                "videoFile" => Media::VideoFile(file),
                _ => Media::QuickTimeFile(file),
            })
        }
    }
}

/// `extLst/p:ext/`, the items inside.
fn extension_items(r: &mut Reader<'_>, el: &Elem) -> Result<Vec<Elem>, ImportError> {
    r.finish(el);
    let mut items = Vec::new();
    for ext in r.children(el)? {
        if ext.is(ns::PML, "ext") {
            items.extend(r.children(&ext)?);
        } else {
            r.retained(&ext);
        }
    }
    Ok(items)
}

/// Over a list bound anywhere in this `p14:media`, before any of it is read.
fn over_limit(r: &Reader<'_>, el: &Elem, limits: &MediaLimits) -> Result<bool, ImportError> {
    for kid in r.children(el)? {
        if kid.is(P14, "bmkLst") && r.tree.child_elements(kid.id).len() > limits.max_bookmarks {
            return Ok(true);
        }
        if !kid.is(P14, "extLst") {
            continue;
        }
        for ext in r.children(&kid)? {
            for info in r.children(&ext)? {
                for list in r.children(&info)? {
                    if list.is(P173, "trackLst")
                        && r.tree.child_elements(list.id).len() > limits.max_tracks
                    {
                        return Ok(true);
                    }
                }
            }
        }
    }
    Ok(false)
}

fn track(r: &mut Reader<'_>, rels: &[Relationship], el: &Elem) -> Result<MediaTrack, ImportError> {
    let mut e = el.clone();
    let id: Option<String> = r.required(&mut e, "id", |_: &String| true)?;
    if id.as_deref().is_some_and(|id| !is_guid(id)) {
        r.note(K::MalformedValue, el, Some("id"));
    }
    let t = MediaTrack {
        id,
        label: r.required(&mut e, "label", |_: &String| true)?,
        lang: r.attr(&mut e, "lang")?,
        embed: read_rel(r, rels, &mut e, "embed", Expect::Track),
        link: read_rel(r, rels, &mut e, "link", Expect::Track),
    };
    r.finish(&e);
    report_kids(r, el)?;
    Ok(t)
}

fn tracks_info(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    el: &Elem,
) -> Result<TracksInfo, ImportError> {
    let mut e = el.clone();
    let mut info = TracksInfo {
        display_location: r.required(&mut e, "displayLoc", |_| true)?,
        tracks: None,
    };
    r.finish(&e);
    for kid in r.children(el)? {
        if !kid.is(P173, "trackLst") {
            r.retained(&kid);
        } else if info.tracks.is_some() {
            r.note(K::DuplicateElement, &kid, None);
        } else {
            r.finish(&kid);
            let mut list = Vec::new();
            for item in r.children(&kid)? {
                if item.is(P173, "track") {
                    list.push(track(r, rels, &item)?);
                } else {
                    r.retained(&item);
                }
            }
            info.tracks = Some(list);
        }
    }
    Ok(info)
}

/// `p14:CT_Media`. `None`, reported, if a list in it crosses a bound.
pub(crate) fn read_media_extension(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    limits: &MediaLimits,
    el: &Elem,
) -> Result<Option<MediaExtension>, ImportError> {
    if over_limit(r, el, limits)? {
        r.note(K::Media(MediaFinding::OverLimit), el, None);
        return Ok(None);
    }
    let mut e = el.clone();
    let mut media = MediaExtension {
        embed: read_rel(r, rels, &mut e, "embed", Expect::Media),
        link: read_rel(r, rels, &mut e, "link", Expect::Media),
        ..MediaExtension::default()
    };
    r.finish(&e);
    let mut seen = std::collections::HashSet::new();
    for kid in r.children(el)? {
        // `extLst` is `p14:` too, by the schema's `elementFormDefault`, and holds `p:ext`.
        let known = kid.in_namespace(P14)
            && matches!(kid.local.as_str(), "trim" | "fade" | "bmkLst" | "extLst");
        if !known {
            r.retained(&kid);
            continue;
        }
        if !seen.insert(kid.local.clone()) {
            r.note(K::DuplicateElement, &kid, None);
            continue;
        }
        let mut k = kid.clone();
        match kid.local.as_str() {
            "trim" => {
                media.trim = Some(MediaTrim {
                    start: time(r, &mut k, "st")?,
                    end: time(r, &mut k, "end")?,
                });
                r.finish(&k);
                report_kids(r, &kid)?;
            }
            "fade" => {
                media.fade = Some(MediaFade {
                    fade_in: time(r, &mut k, "in")?,
                    fade_out: time(r, &mut k, "out")?,
                });
                r.finish(&k);
                report_kids(r, &kid)?;
            }
            "bmkLst" => {
                r.finish(&k);
                let mut list = Vec::new();
                for item in r.children(&kid)? {
                    if !item.is(P14, "bmk") {
                        r.retained(&item);
                        continue;
                    }
                    let mut b = item.clone();
                    list.push(MediaBookmark {
                        name: r.attr(&mut b, "name")?,
                        time: time(r, &mut b, "time")?,
                    });
                    r.finish(&b);
                    report_kids(r, &item)?;
                }
                media.bookmarks = Some(list);
            }
            _ => {
                for item in extension_items(r, &kid)? {
                    if item.is(P173, "tracksInfo") && media.tracks.is_none() {
                        media.tracks = Some(tracks_info(r, rels, &item)?);
                    } else {
                        r.retained(&item);
                    }
                }
            }
        }
    }
    Ok(Some(media))
}

/// `nvPr/p:extLst`: `p14:media`, and the rest reported.
pub(crate) fn read_app_extensions(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    limits: &MediaLimits,
    el: &Elem,
) -> Result<Option<Box<MediaExtension>>, ImportError> {
    let mut found = None;
    let mut seen = false;
    for item in extension_items(r, el)? {
        if !item.is(P14, "media") {
            r.retained(&item);
        } else if seen {
            r.note(K::DuplicateElement, &item, None);
        } else {
            seen = true;
            found = read_media_extension(r, rels, limits, &item)?.map(Box::new);
        }
    }
    Ok(found)
}

/// SHA-256 (FIPS 180-4). Written here because a new dependency in the engine needs an
/// ADR, and this is sixty lines with test vectors beside it.
pub fn sha256(data: &[u8]) -> ContentHash {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut block = |chunk: &[u8]| {
        let mut w = [0u32; 64];
        for (slot, word) in w.iter_mut().zip(chunk.chunks_exact(4)) {
            *slot = word.iter().fold(0, |acc, b| (acc << 8) | u32::from(*b));
        }
        for i in 16..64 {
            let (a, b) = (w[i - 15], w[i - 2]);
            let s0 = a.rotate_right(7) ^ a.rotate_right(18) ^ (a >> 3);
            let s1 = b.rotate_right(17) ^ b.rotate_right(19) ^ (b >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let mut s = h;
        for (k, word) in K.iter().zip(w) {
            let [a, b, c, d, e, f, g, hh] = s;
            let t1 = hh
                .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                .wrapping_add((e & f) ^ (!e & g))
                .wrapping_add(*k)
                .wrapping_add(word);
            let t2 = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                .wrapping_add((a & b) ^ (a & c) ^ (b & c));
            s = [t1.wrapping_add(t2), a, b, c, d.wrapping_add(t1), e, f, g];
        }
        for (slot, value) in h.iter_mut().zip(s) {
            *slot = slot.wrapping_add(value);
        }
    };
    let whole = data.chunks_exact(64);
    let rest = whole.remainder();
    whole.for_each(&mut block);
    // The tail: what is left, `0x80`, zeros to 56 mod 64, and the length in bits.
    let mut tail = rest.to_vec();
    tail.push(0x80);
    while tail.len() % 64 != 56 {
        tail.push(0);
    }
    let bits = u64::try_from(data.len())
        .unwrap_or(u64::MAX)
        .wrapping_mul(8);
    tail.extend_from_slice(&bits.to_be_bytes());
    tail.chunks_exact(64).for_each(&mut block);
    let mut out = [0u8; 32];
    for (slot, word) in out.chunks_exact_mut(4).zip(h) {
        slot.copy_from_slice(&word.to_be_bytes());
    }
    ContentHash(out)
}

fn be16(b: &[u8], at: usize) -> Option<u32> {
    let s = b.get(at..at.checked_add(2)?)?;
    Some(s.iter().fold(0, |acc, v| (acc << 8) | u32::from(*v)))
}

fn be32(b: &[u8], at: usize) -> Option<u32> {
    let s = b.get(at..at.checked_add(4)?)?;
    Some(s.iter().fold(0, |acc, v| (acc << 8) | u32::from(*v)))
}

fn le16(b: &[u8], at: usize) -> Option<u32> {
    let s = b.get(at..at.checked_add(2)?)?;
    Some(s.iter().rev().fold(0, |acc, v| (acc << 8) | u32::from(*v)))
}

fn le32(b: &[u8], at: usize) -> Option<u32> {
    let s = b.get(at..at.checked_add(4)?)?;
    Some(s.iter().rev().fold(0, |acc, v| (acc << 8) | u32::from(*v)))
}

fn jpeg_dimensions(b: &[u8]) -> Option<ImageDimensions> {
    let b = b.get(..b.len().min(MAX_HEADER_SCAN))?;
    let mut at = 2usize;
    // Each pass moves on by at least two, so the loop ends within the slice's length.
    loop {
        if *b.get(at)? != 0xFF {
            return None;
        }
        let marker = *b.get(at.checked_add(1)?)?;
        if marker == 0xFF {
            at = at.checked_add(1)?;
            continue;
        }
        if matches!(marker, 0xC0..=0xCF) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
            return Some(ImageDimensions {
                height: be16(b, at.checked_add(5)?)?,
                width: be16(b, at.checked_add(7)?)?,
            });
        }
        let length = usize::try_from(be16(b, at.checked_add(2)?)?).ok()?;
        if length < 2 {
            return None;
        }
        at = at.checked_add(2)?.checked_add(length)?;
    }
}

/// What a part's first bytes say, and its size where the header gives one. Reads a
/// bounded prefix and allocates nothing: this is not a decoder, and an SVG, or anything
/// else unrecognized, gets `None`. It is never given a part declared as SVG.
pub fn sniff_image(b: &[u8]) -> Option<(ImageFormat, Option<ImageDimensions>)> {
    if b.starts_with(b"\x89PNG\r\n\x1a\n") {
        let size = if b.get(12..16) == Some(b"IHDR") {
            be32(b, 16)
                .zip(be32(b, 20))
                .map(|(width, height)| ImageDimensions { width, height })
        } else {
            None
        };
        return Some((ImageFormat::Png, size));
    }
    if b.starts_with(b"\xFF\xD8\xFF") {
        return Some((ImageFormat::Jpeg, jpeg_dimensions(b)));
    }
    if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        let size = le16(b, 6)
            .zip(le16(b, 8))
            .map(|(width, height)| ImageDimensions { width, height });
        return Some((ImageFormat::Gif, size));
    }
    if b.starts_with(b"BM") {
        // `BITMAPINFOHEADER` and later. A negative height means top-down; a negative
        // width means nothing, and is left unknown.
        let size = if le32(b, 14).is_some_and(|dib| dib >= 40) {
            le32(b, 18)
                .filter(|w| i32::try_from(*w).is_ok())
                .zip(le32(b, 22))
                .map(|(width, h)| ImageDimensions {
                    width,
                    height: (h as i32).unsigned_abs(),
                })
        } else {
            None
        };
        return Some((ImageFormat::Bmp, size));
    }
    if b.starts_with(b"II*\0") || b.starts_with(b"MM\0*") {
        return Some((ImageFormat::Tiff, None));
    }
    if b.starts_with(b"RIFF") && b.get(8..12) == Some(b"WEBP") {
        return Some((ImageFormat::Webp, None));
    }
    if b.get(40..44) == Some(b" EMF") {
        return Some((ImageFormat::Emf, None));
    }
    if b.starts_with(b"\xD7\xCD\xC6\x9A") || b.starts_with(b"\x01\x00\x09\x00") {
        return Some((ImageFormat::Wmf, None));
    }
    None
}

/// The format a declared content type promises. `None` for one with no promise to check.
fn declared_format(content_type: &str) -> Option<ImageFormat> {
    Some(match content_type.to_ascii_lowercase().as_str() {
        "image/png" | "image/x-png" => ImageFormat::Png,
        "image/jpeg" | "image/jpg" | "image/pjpeg" => ImageFormat::Jpeg,
        "image/gif" => ImageFormat::Gif,
        "image/bmp" | "image/x-bmp" | "image/x-ms-bmp" => ImageFormat::Bmp,
        "image/tiff" | "image/tif" => ImageFormat::Tiff,
        "image/webp" => ImageFormat::Webp,
        "image/x-emf" | "image/emf" => ImageFormat::Emf,
        "image/x-wmf" | "image/wmf" => ImageFormat::Wmf,
        _ => return None,
    })
}

fn is_svg_type(content_type: &str) -> bool {
    content_type.eq_ignore_ascii_case("image/svg+xml")
}

/// What a relationship reaches, by its type and then its target's content type.
fn classify(rel_type: &str, content_type: Option<&str>) -> Option<MediaClass> {
    if content_type.is_some_and(is_svg_type) {
        return Some(MediaClass::Svg);
    }
    let by_type = match rel_type.rsplit('/').next().unwrap_or_default() {
        "image" | "hdphoto" => Some(MediaClass::Image),
        "audio" => Some(MediaClass::Audio),
        "video" | "media" => Some(MediaClass::Video),
        "track" => Some(MediaClass::Track),
        _ => None,
    };
    by_type.or_else(|| {
        let lower = content_type?.to_ascii_lowercase();
        if lower.starts_with("image/") {
            Some(MediaClass::Image)
        } else if lower.starts_with("audio/") {
            Some(MediaClass::Audio)
        } else if lower.starts_with("video/") {
            Some(MediaClass::Video)
        } else {
            None
        }
    })
}

/// One thing a catalog noticed. Names a part and a relationship, never content.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CatalogFinding {
    /// What.
    pub kind: MediaFinding,
    /// The part it concerns.
    pub part: String,
    /// The relationship, where one is the subject.
    pub rel_id: Option<String>,
}

/// List a deck's pictures and clips: each part by content hash with every relationship
/// reaching it, and each external target. Reads every media part it can within bounds,
/// which charges the package's budget, so it is called when wanted and not by opening.
///
/// The read is `read_xml_part`, the only capped read `PresentationPackage` offers, so a
/// part over `OoxmlLimits::max_xml_part_bytes` is refused from its declared size with
/// nothing inflated. The name is wrong for this and the bound is right. The bytes go only
/// to the hash and the header sniff, and are dropped.
pub fn catalog_media(
    package: &mut PresentationPackage,
    limits: &MediaLimits,
) -> (MediaCatalog, Vec<CatalogFinding>) {
    let mut findings = Vec::new();
    let mut catalog = MediaCatalog::default();
    let names = package.part_names();
    let by_key: BTreeMap<String, &String> = names
        .iter()
        .filter_map(|n| Some((part_name_key(n)?, n)))
        .collect();
    let mut found: BTreeMap<String, MediaPart> = BTreeMap::new();
    let mut sources: Vec<&str> = names.iter().map(String::as_str).collect();
    sources.push("");
    let mut over = false;
    for source in sources {
        for rel in package.relationships_of(source) {
            let referrer = MediaReferrer {
                part: source.to_owned(),
                rel_id: rel.id.clone(),
            };
            if rel.mode == TargetMode::External {
                if let Some(class) = classify(&rel.rel_type, None) {
                    findings.push(CatalogFinding {
                        kind: MediaFinding::ExternalTarget,
                        part: source.to_owned(),
                        rel_id: Some(rel.id.clone()),
                    });
                    catalog.external.push(ExternalMedia { referrer, class });
                }
                continue;
            }
            let Some(resolved) = resolve_internal(source, &rel.target) else {
                continue;
            };
            let Some(name) = part_name_key(&resolved).and_then(|k| by_key.get(&k).copied()) else {
                if classify(&rel.rel_type, None).is_some() {
                    findings.push(CatalogFinding {
                        kind: MediaFinding::MissingPart,
                        part: source.to_owned(),
                        rel_id: Some(rel.id.clone()),
                    });
                }
                continue;
            };
            let content_type = package.content_types().resolve(name);
            let Some(class) = classify(&rel.rel_type, content_type) else {
                continue;
            };
            if let Some(entry) = found.get_mut(name) {
                entry.referrers.push(referrer);
            } else if found.len() >= limits.max_catalog_parts {
                over = true;
            } else {
                found.insert(
                    name.clone(),
                    MediaPart {
                        part: name.clone(),
                        class,
                        content_type: content_type.map(str::to_owned),
                        hash: None,
                        size: None,
                        format: None,
                        dimensions: None,
                        referrers: vec![referrer],
                    },
                );
            }
        }
    }
    if over {
        findings.push(CatalogFinding {
            kind: MediaFinding::CatalogOverBound,
            part: String::new(),
            rel_id: None,
        });
    }
    let mut spent = 0u64;
    for entry in found.values_mut() {
        entry.referrers.sort();
        let note = |kind| CatalogFinding {
            kind,
            part: entry.part.clone(),
            rel_id: None,
        };
        if spent >= limits.max_catalog_bytes {
            findings.push(note(MediaFinding::PartOverCap));
            continue;
        }
        let bytes = match package.read_xml_part(&entry.part) {
            Ok(bytes) => bytes,
            Err(err) => {
                findings.push(note(match err.code() {
                    "CS-PKG-0003" | "CS-PKG-0009" => MediaFinding::PartOverCap,
                    _ => MediaFinding::UnreadablePart,
                }));
                continue;
            }
        };
        let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        spent = spent.saturating_add(size);
        entry.size = Some(size);
        entry.hash = Some(sha256(&bytes));
        if entry.class != MediaClass::Image {
            continue;
        }
        let sniffed = sniff_image(&bytes);
        entry.format = sniffed.map(|s| s.0);
        entry.dimensions = sniffed.and_then(|s| s.1);
        let promised = entry.content_type.as_deref().and_then(declared_format);
        if promised.is_some() && promised != entry.format {
            findings.push(note(MediaFinding::FormatMismatch));
        }
        if entry
            .dimensions
            .is_some_and(|d| d.pixels() > limits.max_image_pixels)
        {
            findings.push(note(MediaFinding::OverPixelCap));
        }
    }
    catalog.parts = found.into_values().collect();
    catalog.external.sort();
    findings.sort();
    (catalog, findings)
}
