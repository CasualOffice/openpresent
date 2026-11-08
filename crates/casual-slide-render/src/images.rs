//! Pictures: where a backend gets their bytes, and what it says about the ones it
//! could not draw.
//!
//! Copied from OpenCalc's `casual-calc-render/src/images.rs` (see PROVENANCE.md) and
//! changed only where slides need it: the PDF glyph report is gone, there is no PDF
//! backend yet; JPEG is the one codec beyond PNG (ADR-019); and header reads use
//! checked slicing, since the bytes are a deck's.
//!
//! **The report is the point, not a nicety.** A picture that decodes to nothing and a
//! picture that was never there produce the same blank rectangle, and the second is a
//! bug while the first is a file this cannot read.

use std::collections::BTreeMap;

/// Where a renderer gets the bytes of a media part.
pub trait ImageSource {
    /// The bytes of the part at `path`, or `None` if this source has none.
    fn part_bytes(&self, path: &str) -> Option<&[u8]>;
}

/// A source with no media at all: every picture is reported as not supplied.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoImages;

impl ImageSource for NoImages {
    fn part_bytes(&self, _path: &str) -> Option<&[u8]> {
        None
    }
}

impl ImageSource for BTreeMap<String, Vec<u8>> {
    fn part_bytes(&self, path: &str) -> Option<&[u8]> {
        self.get(path).map(Vec::as_slice)
    }
}

impl<T: ImageSource + ?Sized> ImageSource for &T {
    fn part_bytes(&self, path: &str) -> Option<&[u8]> {
        (**self).part_bytes(path)
    }
}

/// Why a picture in the display list did not reach the surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum UndrawnReason {
    /// The [`ImageSource`] had no bytes under that path.
    NotSupplied,
    /// A picture in a format this backend does not decode, named: `"emf"`, `"svg"`.
    UnsupportedFormat(&'static str),
    /// Bytes this could not read: a truncated or corrupt part.
    Undecodable,
    /// A picture declaring more pixels than [`MAX_IMAGE_PIXELS`], refused before
    /// decoding.
    TooLarge {
        /// The declared width in pixels.
        width: u32,
        /// The declared height in pixels.
        height: u32,
    },
}

impl UndrawnReason {
    /// A stable key for a compatibility report.
    #[must_use]
    pub fn feature(&self) -> &'static str {
        match self {
            UndrawnReason::NotSupplied => "image (media not supplied)",
            UndrawnReason::UnsupportedFormat(_) => "image (format not decoded)",
            UndrawnReason::Undecodable => "image (undecodable)",
            UndrawnReason::TooLarge { .. } => "image (over the pixel limit)",
        }
    }
}

impl core::fmt::Display for UndrawnReason {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            UndrawnReason::NotSupplied => f.write_str("no bytes were supplied for the part"),
            UndrawnReason::UnsupportedFormat(name) => {
                write!(f, "{name} is not an image format this backend decodes")
            }
            UndrawnReason::Undecodable => f.write_str("the picture could not be decoded"),
            UndrawnReason::TooLarge { width, height } => write!(
                f,
                "{width}x{height} is over the {MAX_IMAGE_PIXELS}-pixel limit"
            ),
        }
    }
}

/// One picture the renderer could not draw, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndrawnImage {
    /// The package path from the display list.
    pub part: String,
    /// What stopped it.
    pub reason: UndrawnReason,
}

/// What a render did with the pictures in its display list. `undrawn` is
/// deduplicated by part and reason.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImageReport {
    /// How many pictures were painted.
    pub drawn: u32,
    /// The ones that were not, named and reasoned, in first-seen order.
    pub undrawn: Vec<UndrawnImage>,
}

impl ImageReport {
    /// Whether every picture in the display list reached the surface.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.undrawn.is_empty()
    }

    pub(crate) fn drew(&mut self) {
        self.drawn = self.drawn.saturating_add(1);
    }

    pub(crate) fn missed(&mut self, part: &str, reason: UndrawnReason) {
        if self
            .undrawn
            .iter()
            .any(|u| u.part == part && u.reason == reason)
        {
            return;
        }
        self.undrawn.push(UndrawnImage {
            part: part.to_owned(),
            reason,
        });
    }
}

/// The most pixels a picture may declare before this refuses to decode it. A
/// security bound, checked against the header before any allocation.
pub const MAX_IMAGE_PIXELS: u64 = 40_000_000;

/// The longest a single side may be, for the decoder given a limit.
pub const MAX_IMAGE_SIDE: u32 = 20_000;

const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

fn is_jpeg(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0xff, 0xd8, 0xff])
}

/// The format `bytes` look like by magic number, when it is not one decoded here.
/// Sniffed, never taken from the part's extension or content type.
fn foreign_format(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(PNG) || is_jpeg(bytes) {
        return None;
    }
    let at = |from: usize, to: usize| bytes.get(from..to);
    let name = if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "gif"
    } else if bytes.starts_with(b"BM") {
        "bmp"
    } else if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        "tiff"
    } else if bytes.starts_with(b"RIFF") && at(8, 12) == Some(b"WEBP") {
        "webp"
    } else if bytes.starts_with(&[1, 0, 0, 0]) && at(40, 44) == Some(b" EMF") {
        "emf"
    } else if bytes.starts_with(&[0xd7, 0xcd, 0xc6, 0x9a]) || bytes.starts_with(&[1, 0, 9, 0]) {
        "wmf"
    } else if bytes.starts_with(b"<?xml") || bytes.starts_with(b"<svg") {
        "svg"
    } else {
        "an unrecognised format"
    };
    Some(name)
}

/// Decode a JPEG, bounded before it allocates.
fn decode_jpeg(bytes: &[u8]) -> Result<tiny_skia::Pixmap, UndrawnReason> {
    use image::ImageDecoder as _;

    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_SIDE);
    limits.max_image_height = Some(MAX_IMAGE_SIDE);
    limits.max_alloc = Some(MAX_IMAGE_PIXELS.saturating_mul(4));

    let reader =
        image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Jpeg);
    let mut decoder = reader
        .into_decoder()
        .map_err(|_| UndrawnReason::Undecodable)?;
    let (width, height) = decoder.dimensions();
    if u64::from(width).saturating_mul(u64::from(height)) > MAX_IMAGE_PIXELS {
        return Err(UndrawnReason::TooLarge { width, height });
    }
    decoder
        .set_limits(limits)
        .map_err(|_| UndrawnReason::TooLarge { width, height })?;
    let decoded = image::DynamicImage::from_decoder(decoder)
        .map_err(|_| UndrawnReason::Undecodable)?
        .into_rgba8();
    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or(UndrawnReason::Undecodable)?;
    for (dst, src) in pixmap.pixels_mut().iter_mut().zip(decoded.pixels()) {
        let [r, g, b, a] = src.0;
        let pre = |c: u8| ((u16::from(c) * u16::from(a)) / 255) as u8;
        if let Some(px) = tiny_skia::PremultipliedColorU8::from_rgba(pre(r), pre(g), pre(b), a) {
            *dst = px;
        }
    }
    Ok(pixmap)
}

/// The width and height a PNG's IHDR declares, without decoding it.
fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.get(12..16)? != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?);
    let height = u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?);
    Some((width, height))
}

/// Decode the media part at `path` into a pixmap, or say why not.
pub(crate) fn decode(
    path: &str,
    images: &dyn ImageSource,
) -> Result<tiny_skia::Pixmap, UndrawnReason> {
    let Some(bytes) = images.part_bytes(path) else {
        return Err(UndrawnReason::NotSupplied);
    };
    if let Some(format) = foreign_format(bytes) {
        return Err(UndrawnReason::UnsupportedFormat(format));
    }
    if is_jpeg(bytes) {
        return decode_jpeg(bytes);
    }
    // Before decoding, not after: the point of the limit is the allocation it
    // prevents.
    let (width, height) = png_dimensions(bytes).ok_or(UndrawnReason::Undecodable)?;
    if u64::from(width).saturating_mul(u64::from(height)) > MAX_IMAGE_PIXELS {
        return Err(UndrawnReason::TooLarge { width, height });
    }
    tiny_skia::Pixmap::decode_png(bytes).map_err(|_| UndrawnReason::Undecodable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_png_declaring_too_many_pixels_is_refused_before_decoding() {
        let mut bytes = PNG.to_vec();
        bytes.extend_from_slice(&[0, 0, 0, 13]);
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&60_000u32.to_be_bytes());
        bytes.extend_from_slice(&60_000u32.to_be_bytes());
        let mut source = BTreeMap::new();
        source.insert("p".to_owned(), bytes);
        assert_eq!(
            decode("p", &source).err(),
            Some(UndrawnReason::TooLarge {
                width: 60_000,
                height: 60_000
            })
        );
    }

    #[test]
    fn vector_and_short_inputs_are_named_not_decoded() {
        let mut source = BTreeMap::new();
        source.insert("svg".to_owned(), b"<svg/>".to_vec());
        source.insert("short".to_owned(), vec![1, 0, 0, 0]);
        assert_eq!(
            decode("svg", &source).err(),
            Some(UndrawnReason::UnsupportedFormat("svg"))
        );
        assert_eq!(
            decode("short", &source).err(),
            Some(UndrawnReason::UnsupportedFormat("an unrecognised format"))
        );
        assert_eq!(
            decode("absent", &source).err(),
            Some(UndrawnReason::NotSupplied)
        );
    }

    #[test]
    fn a_truncated_jpeg_is_undecodable_not_a_panic() {
        let mut source = BTreeMap::new();
        source.insert("j".to_owned(), vec![0xff, 0xd8, 0xff, 0xe0, 0, 0x10]);
        assert_eq!(decode("j", &source).err(), Some(UndrawnReason::Undecodable));
    }
}
