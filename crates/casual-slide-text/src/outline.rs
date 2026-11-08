//! Glyph outlines, and a stable name for each face a glyph run uses.
//!
//! A display list is plain data that a backend may hold apart from this crate, so
//! a run names its face by a string key rather than by its bytes. The key of a
//! bundled face is its file's name (`Carlito-Bold`); any other face, supplied by
//! the host or a CJK slice, is named by a hash of its bytes, so the same face has
//! the same key in every process. A face is remembered when a run first names it,
//! and [`face_by_key`] finds it again for the backend.
//!
//! The pen is OpenCalc's `GlyphPen` in `casual-calc-render`, which maps font
//! space, y up, to device space, y down; here it appends to a plain list so the
//! backend chooses its own path type.

use std::sync::RwLock;

use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider as _;
use skrifa::{FontRef, GlyphId, MetadataProvider, instance::LocationRef, prelude::Size};

use crate::faces::FAMILIES;

/// One step of a glyph's outline, y down, origin at the glyph's pen position on
/// the baseline, in the unit the size was given in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutlineOp {
    /// Start a contour.
    MoveTo(f32, f32),
    /// A straight segment.
    LineTo(f32, f32),
    /// A quadratic segment: control, end.
    QuadTo(f32, f32, f32, f32),
    /// A cubic segment: two controls, end.
    CubicTo(f32, f32, f32, f32, f32, f32),
    /// Close the contour.
    Close,
}

/// The style suffixes of a bundled family's four faces, by `bold | italic << 1`.
const STYLES: [&str; 4] = ["Regular", "Bold", "Italic", "BoldItalic"];

/// Faces named by a hash, remembered as runs name them.
static HASHED: RwLock<Vec<(&'static [u8], String)>> = RwLock::new(Vec::new());

/// Whether two faces are one: the same address, or the same bytes. A `const`
/// family's `include_bytes!` may be copied per codegen unit, so address alone
/// can miss a bundled face (`CI-16`). Lengths differ between faces, so the
/// byte compare runs only for the face itself.
fn same_face(a: &[u8], b: &[u8]) -> bool {
    core::ptr::eq(a, b) || a == b
}

fn bundled_key(face: &[u8]) -> Option<String> {
    FAMILIES.iter().find_map(|family| {
        STYLES.iter().enumerate().find_map(|(i, style)| {
            let bytes = family.face_bytes(i & 1 == 1, i & 2 == 2);
            same_face(bytes, face).then(|| format!("{}-{style}", family.name.replace(' ', "")))
        })
    })
}

/// FNV-1a over the bytes, with their length: the same face gives the same key in
/// every process and on every host.
fn hash_key(face: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in face {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("face-{:x}-{hash:016x}", face.len())
}

/// The key a glyph run names `face` by, remembering the face so that
/// [`face_by_key`] can find it.
#[must_use]
pub fn face_key(face: &'static [u8]) -> String {
    if let Some(key) = bundled_key(face) {
        return key;
    }
    if let Ok(known) = HASHED.read()
        && let Some((_, key)) = known.iter().find(|(f, _)| core::ptr::eq(*f, face))
    {
        return key.clone();
    }
    let key = hash_key(face);
    if let Ok(mut known) = HASHED.write()
        && !known.iter().any(|(f, _)| core::ptr::eq(*f, face))
    {
        known.push((face, key.clone()));
    }
    key
}

/// The face a key names: a bundled face, or one a run has named before.
#[must_use]
pub fn face_by_key(key: &str) -> Option<&'static [u8]> {
    let bundled = FAMILIES.iter().find_map(|family| {
        STYLES.iter().enumerate().find_map(|(i, style)| {
            let name = format!("{}-{style}", family.name.replace(' ', ""));
            (name == key).then(|| family.face_bytes(i & 1 == 1, i & 2 == 2))
        })
    });
    bundled.or_else(|| {
        HASHED
            .read()
            .ok()?
            .iter()
            .find(|(_, k)| k == key)
            .map(|(f, _)| *f)
    })
}

/// Writes outline commands y down, scaled from font units by `k`.
struct Pen<'a> {
    out: &'a mut Vec<OutlineOp>,
    k: f32,
}

impl OutlinePen for Pen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        let k = self.k;
        self.out.push(OutlineOp::MoveTo(x * k, -y * k));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let k = self.k;
        self.out.push(OutlineOp::LineTo(x * k, -y * k));
    }
    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let k = self.k;
        self.out
            .push(OutlineOp::QuadTo(cx0 * k, -cy0 * k, x * k, -y * k));
    }
    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let k = self.k;
        self.out.push(OutlineOp::CubicTo(
            cx0 * k,
            -cy0 * k,
            cx1 * k,
            -cy1 * k,
            x * k,
            -y * k,
        ));
    }
    fn close(&mut self) {
        self.out.push(OutlineOp::Close);
    }
}

/// Append glyph `id` of `face` at em size `size`, origin at 0,0 on the baseline,
/// y down, in the unit `size` is in. `false` if the bytes are not a face, the
/// glyph is not in it, or the size is not a positive number. A glyph with no
/// contours, a space, appends nothing and is `true`.
pub fn glyph_outline(face: &[u8], id: u32, size: f32, out: &mut Vec<OutlineOp>) -> bool {
    if !size.is_finite() || size <= 0.0 {
        return false;
    }
    let Ok(font) = FontRef::new(face) else {
        return false;
    };
    let Some(glyph) = font.outline_glyphs().get(GlyphId::new(id)) else {
        return false;
    };
    // Drawn in font units and scaled here: a slide asks for sizes in EMU, a 44 pt
    // em being 558,800, and a CFF outline scaled that far overflows skrifa's 16.16
    // fixed point and comes out upside down (FH-01).
    let upem = font
        .head()
        .map_or(1000.0, |h| f32::from(h.units_per_em().max(16)));
    let mut pen = Pen {
        out,
        k: size / upem,
    };
    glyph
        .draw(
            DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
            &mut pen,
        )
        .is_ok()
}

/// [`glyph_outline`] for a face named by [`face_key`].
pub fn outline_by_key(key: &str, id: u32, size: f32, out: &mut Vec<OutlineOp>) -> bool {
    face_by_key(key).is_some_and(|face| glyph_outline(face, id, size, out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_faces_round_trip_through_their_keys() {
        for family in FAMILIES {
            for (bold, italic) in [(false, false), (true, false), (false, true), (true, true)] {
                let face = family.face_bytes(bold, italic);
                let key = face_key(face);
                assert!(!key.contains(' '), "{key}");
                let back = face_by_key(&key).expect("found again");
                assert_eq!(back, face, "{key}");
            }
        }
        assert_eq!(
            face_key(crate::faces::CARLITO.face_bytes(true, false)),
            "Carlito-Bold"
        );
    }

    #[test]
    fn another_face_is_named_by_its_bytes_and_found_again() {
        let mut bytes = crate::faces::CARLITO.face_bytes(false, false).to_vec();
        bytes.push(0);
        let copy: &'static [u8] = Box::leak(bytes.into_boxed_slice());
        let key = face_key(copy);
        assert!(key.starts_with("face-"), "{key}");
        assert_eq!(key, hash_key(copy), "the same bytes, the same key");
        assert!(core::ptr::eq(face_by_key(&key).expect("remembered"), copy));
        assert_eq!(face_by_key("no-such-face"), None);
    }

    #[test]
    fn a_letter_has_an_outline_scaled_to_the_size_and_above_the_baseline() {
        let face = crate::faces::CARLITO.face_bytes(false, false);
        let id = FontRef::new(face)
            .unwrap()
            .charmap()
            .map('H')
            .unwrap()
            .to_u32();
        let mut small = Vec::new();
        let mut large = Vec::new();
        assert!(glyph_outline(face, id, 10.0, &mut small));
        assert!(glyph_outline(face, id, 20.0, &mut large));
        assert!(!small.is_empty());
        assert_eq!(small.len(), large.len());
        let top = |ops: &[OutlineOp]| {
            ops.iter()
                .filter_map(|op| match *op {
                    OutlineOp::MoveTo(_, y) | OutlineOp::LineTo(_, y) => Some(y),
                    _ => None,
                })
                .fold(f32::INFINITY, f32::min)
        };
        assert!(
            top(&small) < 0.0,
            "y down: the letter is above the baseline"
        );
        assert!(
            // Each outline is rounded to 26.6 fixed point, a 1/64 step; doubling the small
            // one doubles its rounding, so two steps bound the difference.
            (top(&large) - 2.0 * top(&small)).abs() <= 2.0 / 64.0,
            "linear within two 26.6 roundings"
        );
    }

    #[test]
    fn bad_input_gives_no_outline_and_no_panic() {
        let face = crate::faces::CARLITO.face_bytes(false, false);
        let mut out = Vec::new();
        assert!(!glyph_outline(b"not a font", 1, 10.0, &mut out));
        assert!(!glyph_outline(face, u32::MAX, 10.0, &mut out));
        assert!(!glyph_outline(face, 1, f32::NAN, &mut out));
        assert!(!glyph_outline(face, 1, -1.0, &mut out));
        assert!(out.is_empty());
    }
}
