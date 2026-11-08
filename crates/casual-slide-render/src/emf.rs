//! The one EMF this backend draws: a single bitmap wrapped in a metafile (`T2E-03`'s
//! first slice, `FH-05`).
//!
//! Office and many converters save a pasted raster logo as an EMF holding a header,
//! one `EMR_STRETCHDIBITS` and an end record. Such a picture is that bitmap, so it is
//! decoded here into a pixmap and drawn as any other picture. Every other metafile,
//! including one whose bitmap does not cover its bounds or whose records could move,
//! clip or blend it, gives `None` and stays reported as an EMF this backend does not
//! decode. Our own reader (`DEC-18`): no code from another EMF implementation.
//!
//! Every read is checked: the bytes are a deck's. The bitmap's pixel count is bounded
//! by [`MAX_IMAGE_PIXELS`](crate::images::MAX_IMAGE_PIXELS) before any allocation.

use crate::images::{MAX_IMAGE_PIXELS, UndrawnReason};

/// The most records read before the metafile is left undrawn.
pub(crate) const MAX_EMF_RECORDS: usize = 100_000;

const EMR_HEADER: u32 = 1;
const EMR_EOF: u32 = 14;
const EMR_STRETCHDIBITS: u32 = 81;
/// `SRCCOPY`: the bitmap replaces what is under it.
const SRCCOPY: u32 = 0x00CC_0020;
const BI_RGB: u32 = 0;
const BI_JPEG: u32 = 4;
const BI_PNG: u32 = 5;

/// Records that change nothing a lone bitmap's pixels depend on: mapping-mode,
/// text, background and stretch-mode state, save and restore, and comments.
fn harmless(kind: u32) -> bool {
    matches!(
        kind,
        9..=12 // SETWINDOWEXTEX, SETWINDOWORGEX, SETVIEWPORTEXTEX, SETVIEWPORTORGEX
            | 17..=22 // SETMAPMODE .. SETTEXTALIGN
            | 24 | 25 // SETTEXTCOLOR, SETBKCOLOR
            | 33 | 34 // SAVEDC, RESTOREDC
            | 58 // SETMITERLIMIT
            | 70 // GDICOMMENT
            | 98 // SETICMMODE
    )
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    let end = at.checked_add(4)?;
    Some(u32::from_le_bytes(b.get(at..end)?.try_into().ok()?))
}

fn i32_at(b: &[u8], at: usize) -> Option<i32> {
    let end = at.checked_add(4)?;
    Some(i32::from_le_bytes(b.get(at..end)?.try_into().ok()?))
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    let end = at.checked_add(2)?;
    Some(u16::from_le_bytes(b.get(at..end)?.try_into().ok()?))
}

/// A `RectL` at `at`: left, top, right, bottom.
fn rect_at(b: &[u8], at: usize) -> Option<[i32; 4]> {
    Some([
        i32_at(b, at)?,
        i32_at(b, at.checked_add(4)?)?,
        i32_at(b, at.checked_add(8)?)?,
        i32_at(b, at.checked_add(12)?)?,
    ])
}

/// The metafile's bitmap, when it is a lone bitmap covering the picture: `None`
/// for any other EMF, `Some(Err)` for a bitmap too large or that cannot be read.
pub(crate) fn lone_bitmap(bytes: &[u8]) -> Option<Result<tiny_skia::Pixmap, UndrawnReason>> {
    if u32_at(bytes, 0)? != EMR_HEADER {
        return None;
    }
    let bounds = rect_at(bytes, 8)?;
    let mut at = 0usize;
    let mut bitmap = None;
    for _ in 0..MAX_EMF_RECORDS {
        let kind = u32_at(bytes, at)?;
        let size = usize::try_from(u32_at(bytes, at.checked_add(4)?)?).ok()?;
        let end = at.checked_add(size)?;
        if size < 8 || size % 4 != 0 || end > bytes.len() {
            return None;
        }
        let record = bytes.get(at..end)?;
        match kind {
            EMR_HEADER if at == 0 => {}
            EMR_EOF => return bitmap,
            EMR_STRETCHDIBITS if bitmap.is_none() => bitmap = Some(stretch_dibits(record, bounds)?),
            k if harmless(k) => {}
            _ => return None,
        }
        at = end;
    }
    None
}

/// An `EMR_STRETCHDIBITS` copying its bitmap over the whole picture: `None` when it
/// covers less, uses a raster operation other than a copy or a palette by index.
fn stretch_dibits(r: &[u8], bounds: [i32; 4]) -> Option<Result<tiny_skia::Pixmap, UndrawnReason>> {
    // Its own bounds, in device units as the header's, inclusive; one unit of
    // rounding either way is still the whole picture.
    let own = rect_at(r, 8)?;
    if own
        .iter()
        .zip(bounds)
        .any(|(a, b)| (i64::from(*a) - i64::from(b)).abs() > 1)
    {
        return None;
    }
    let (x_src, y_src) = (i32_at(r, 32)?, i32_at(r, 36)?);
    let (cx_src, cy_src) = (i32_at(r, 40)?, i32_at(r, 44)?);
    let off_bmi = usize::try_from(u32_at(r, 48)?).ok()?;
    let off_bits = usize::try_from(u32_at(r, 56)?).ok()?;
    let cb_bits = usize::try_from(u32_at(r, 60)?).ok()?;
    if u32_at(r, 64)? != 0 || u32_at(r, 68)? != SRCCOPY {
        return None;
    }
    let bits = r.get(off_bits..off_bits.checked_add(cb_bits)?)?;
    let header = r.get(off_bmi..)?;
    let bmi_size = usize::try_from(u32_at(header, 0)?).ok()?;
    let width = i32_at(header, 4)?;
    let height = i32_at(header, 8)?;
    let bit_count = u16_at(header, 14)?;
    let compression = u32_at(header, 16)?;
    let colors_used = u32_at(header, 32)?;
    if bmi_size < 40 || width <= 0 || height == 0 {
        return None;
    }
    let w = u32::try_from(width).ok()?;
    let h = height.unsigned_abs();
    if u64::from(w).saturating_mul(u64::from(h)) > MAX_IMAGE_PIXELS {
        return Some(Err(UndrawnReason::TooLarge {
            width: w,
            height: h,
        }));
    }
    let full = match compression {
        BI_JPEG | BI_PNG => {
            let mut source = std::collections::BTreeMap::new();
            source.insert(String::new(), bits.to_vec());
            crate::images::decode("", &source)
        }
        BI_RGB => {
            let palette = header.get(bmi_size..).unwrap_or(&[]);
            rgb(bits, palette, w, h, height < 0, bit_count, colors_used)
                .ok_or(UndrawnReason::Undecodable)
        }
        _ => return None,
    };
    Some(full.and_then(|p| crop(p, x_src, y_src, cx_src, cy_src)))
}

/// The source rectangle of `full`, clamped to it.
fn crop(
    full: tiny_skia::Pixmap,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32,
) -> Result<tiny_skia::Pixmap, UndrawnReason> {
    let (fw, fh) = (i64::from(full.width()), i64::from(full.height()));
    let l = i64::from(x).clamp(0, fw);
    let t = i64::from(y).clamp(0, fh);
    let r = (i64::from(x) + i64::from(cx)).clamp(0, fw);
    let b = (i64::from(y) + i64::from(cy)).clamp(0, fh);
    if (l, t, r, b) == (0, 0, fw, fh) {
        return Ok(full);
    }
    let rect = tiny_skia::IntRect::from_ltrb(
        i32::try_from(l).map_err(|_| UndrawnReason::Undecodable)?,
        i32::try_from(t).map_err(|_| UndrawnReason::Undecodable)?,
        i32::try_from(r).map_err(|_| UndrawnReason::Undecodable)?,
        i32::try_from(b).map_err(|_| UndrawnReason::Undecodable)?,
    )
    .ok_or(UndrawnReason::Undecodable)?;
    full.clone_rect(rect).ok_or(UndrawnReason::Undecodable)
}

/// An uncompressed DIB: 1, 4 and 8 bits through the palette, 16 bits as 5-5-5,
/// 24 and 32 bits as blue, green, red. The fourth byte of a 32-bit pixel is not
/// alpha to `StretchDIBits`, so the bitmap is opaque.
fn rgb(
    bits: &[u8],
    palette: &[u8],
    w: u32,
    h: u32,
    top_down: bool,
    bit_count: u16,
    colors_used: u32,
) -> Option<tiny_skia::Pixmap> {
    let bpp = usize::from(bit_count);
    if !matches!(bpp, 1 | 4 | 8 | 16 | 24 | 32) {
        return None;
    }
    let wu = usize::try_from(w).ok()?;
    let hu = usize::try_from(h).ok()?;
    let stride = wu.checked_mul(bpp)?.checked_add(31)? / 32 * 4;
    if stride.checked_mul(hu)? > bits.len() {
        return None;
    }
    let entries = if bpp <= 8 {
        let max = 1usize << bpp;
        match usize::try_from(colors_used).ok()? {
            0 => max,
            n => n.min(max),
        }
    } else {
        0
    };
    let colour = |i: usize| -> Option<[u8; 3]> {
        let at = i.checked_mul(4)?;
        match palette.get(at..at.checked_add(3)?)? {
            &[b, g, r] => Some([r, g, b]),
            _ => None,
        }
    };
    if palette.len() < entries.checked_mul(4)? {
        return None;
    }
    let mut pixmap = tiny_skia::Pixmap::new(w, h)?;
    let pixels = pixmap.pixels_mut();
    for row in 0..hu {
        let src_row = if top_down { row } else { hu - 1 - row };
        let start = src_row.checked_mul(stride)?;
        let line = bits.get(start..start.checked_add(stride)?)?;
        for col in 0..wu {
            let [r, g, b] = match bpp {
                24 | 32 => {
                    let at = col * (bpp / 8);
                    match line.get(at..at + 3)? {
                        &[b, g, r] => [r, g, b],
                        _ => return None,
                    }
                }
                16 => {
                    let v = u16::from_le_bytes(line.get(col * 2..col * 2 + 2)?.try_into().ok()?);
                    let five = |s: u16| ((((v >> s) & 31) * 255 + 15) / 31) as u8;
                    [five(10), five(5), five(0)]
                }
                _ => {
                    let bit = col * bpp;
                    let byte = *line.get(bit / 8)?;
                    let shift = 8 - bpp - bit % 8;
                    let index = usize::from(byte >> shift) & ((1 << bpp) - 1);
                    if index >= entries {
                        [0, 0, 0]
                    } else {
                        colour(index)?
                    }
                }
            };
            *pixels.get_mut(row * wu + col)? =
                tiny_skia::PremultipliedColorU8::from_rgba(r, g, b, 255)?;
        }
    }
    Some(pixmap)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A metafile of a header, one `EMR_STRETCHDIBITS` of a `w` by 2 bottom-up
    /// 24-bit bitmap, and an end record, with `extra` before the end.
    pub(crate) fn wrapped_bitmap(w: u32, extra: &[u8]) -> Vec<u8> {
        let stride = (w * 24).div_ceil(32) * 4;
        let mut bits = vec![0u8; (stride * 2) as usize];
        // Bottom row (stored first) blue, top row red.
        for x in 0..w as usize {
            bits[x * 3] = 255;
            bits[stride as usize + x * 3 + 2] = 255;
        }
        let mut header = vec![0u8; 88];
        header[0..4].copy_from_slice(&EMR_HEADER.to_le_bytes());
        header[4..8].copy_from_slice(&88u32.to_le_bytes());
        header[16..20].copy_from_slice(&(w as i32 - 1).to_le_bytes());
        header[20..24].copy_from_slice(&1i32.to_le_bytes());
        header[40..44].copy_from_slice(b" EMF");
        let mut rec = vec![0u8; 80];
        let mut bmi = vec![0u8; 40];
        bmi[0..4].copy_from_slice(&40u32.to_le_bytes());
        bmi[4..8].copy_from_slice(&(w as i32).to_le_bytes());
        bmi[8..12].copy_from_slice(&2i32.to_le_bytes());
        bmi[12..14].copy_from_slice(&1u16.to_le_bytes());
        bmi[14..16].copy_from_slice(&24u16.to_le_bytes());
        let size = 80 + 40 + bits.len() as u32;
        rec[0..4].copy_from_slice(&EMR_STRETCHDIBITS.to_le_bytes());
        rec[4..8].copy_from_slice(&size.to_le_bytes());
        rec[16..20].copy_from_slice(&(w as i32 - 1).to_le_bytes());
        rec[20..24].copy_from_slice(&1i32.to_le_bytes());
        rec[40..44].copy_from_slice(&(w as i32).to_le_bytes());
        rec[44..48].copy_from_slice(&2i32.to_le_bytes());
        rec[48..52].copy_from_slice(&80u32.to_le_bytes());
        rec[52..56].copy_from_slice(&40u32.to_le_bytes());
        rec[56..60].copy_from_slice(&120u32.to_le_bytes());
        rec[60..64].copy_from_slice(&(bits.len() as u32).to_le_bytes());
        rec[68..72].copy_from_slice(&SRCCOPY.to_le_bytes());
        rec[72..76].copy_from_slice(&(w as i32).to_le_bytes());
        rec[76..80].copy_from_slice(&2i32.to_le_bytes());
        let mut out = header;
        out.extend(rec);
        out.extend(bmi);
        out.extend(bits);
        out.extend_from_slice(extra);
        out.extend_from_slice(&EMR_EOF.to_le_bytes());
        out.extend_from_slice(&20u32.to_le_bytes());
        out.extend_from_slice(&[0u8; 12]);
        out
    }

    #[test]
    fn a_metafile_holding_one_bitmap_decodes_to_that_bitmap() {
        let emf = wrapped_bitmap(3, &[]);
        let pixmap = lone_bitmap(&emf).expect("a lone bitmap").expect("decodes");
        assert_eq!((pixmap.width(), pixmap.height()), (3, 2));
        let top = pixmap.pixel(0, 0).expect("pixel");
        let bottom = pixmap.pixel(2, 1).expect("pixel");
        assert_eq!((top.red(), top.blue(), top.alpha()), (255, 0, 255));
        assert_eq!((bottom.red(), bottom.blue()), (0, 255));
    }

    #[test]
    fn a_metafile_that_draws_anything_else_is_left_undrawn() {
        // EMR_RECTANGLE (43) after the bitmap.
        let mut rect = 43u32.to_le_bytes().to_vec();
        rect.extend_from_slice(&24u32.to_le_bytes());
        rect.extend_from_slice(&[0u8; 16]);
        assert!(lone_bitmap(&wrapped_bitmap(3, &rect)).is_none());
    }

    #[test]
    fn truncated_and_hostile_metafiles_never_panic() {
        let emf = wrapped_bitmap(5, &[]);
        for cut in 0..emf.len() {
            let _ = lone_bitmap(&emf[..cut]);
        }
        let mut huge = emf.clone();
        // Declare a bitmap far over the pixel limit.
        huge[88 + 80 + 4..88 + 80 + 8].copy_from_slice(&1_000_000i32.to_le_bytes());
        huge[88 + 80 + 8..88 + 80 + 12].copy_from_slice(&(-1_000_000i32).to_le_bytes());
        assert!(matches!(
            lone_bitmap(&huge),
            Some(Err(UndrawnReason::TooLarge { .. }))
        ));
        let mut zero = emf;
        zero[88 + 4..88 + 8].copy_from_slice(&0u32.to_le_bytes());
        assert!(lone_bitmap(&zero).is_none());
    }
}
