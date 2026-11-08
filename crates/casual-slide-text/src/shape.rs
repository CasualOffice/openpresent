//! Turning text into positioned glyphs.
//!
//! Copied from OpenCalc's `casual-calc-render/src/shape.rs` (see `PROVENANCE.md`).
//! There it was behind a `shaping` feature, off for WebAssembly, under OpenCalc's
//! ADR-018. Here it is always compiled: a slide's lines are broken by the engine on
//! every host, so the shaper must be on every host too (`docs/42`). The one other
//! change: each glyph carries its cluster, the byte offset of the text it came
//! from, which line breaking needs to measure a span of text.
//!
//! # Why the glyph ids are safe to hand to `skrifa`
//!
//! The shaper turns text into glyph *ids* and `skrifa` outlines them. That
//! sounds like two sources of truth and is not: a glyph id is an index into the
//! font's own tables, so the same bytes give the same ids to both. The
//! alternative — shaping to characters and mapping them back — is what loses
//! ligatures, because the ligature has no character.
//!
//! It used to be two *parsers* that happened to agree: `rustybuzz` read the
//! font with `ttf-parser` while `skrifa` read it with `read-fonts`. Both of
//! those crates are unmaintained (`DEP-14`), and `harfrust` — the same shaping
//! algorithm, maintained under the harfbuzz organisation — is built on
//! `read-fonts` itself. So the agreement is now structural: one parser, one set
//! of tables, and a single `read-fonts` in the lock file rather than two
//! libraries kept in step by luck.

use harfrust::{Direction, FontRef, ShapeOptions, ShaperData, UnicodeBuffer};

/// One glyph, placed.
///
/// Offsets are separate from the advance because shaping needs both: a mark
/// sitting over the letter before it advances nothing and is displaced, and
/// collapsing the two would stack every diacritic at the origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    /// The glyph's id in the face it was shaped with.
    pub id: u16,
    /// How far the pen moves after drawing it.
    pub advance: f32,
    /// Horizontal displacement from the pen, before drawing.
    pub x_offset: f32,
    /// Vertical displacement from the baseline, before drawing, positive up.
    pub y_offset: f32,
    /// Byte offset in the shaped text of the first character this glyph draws.
    pub cluster: u32,
}

/// Shape `text` with `bytes` at `size_px`.
///
/// Returns `None` when the bytes are not a face this can read, which leaves the
/// caller on its unshaped path rather than dropping the text.
///
/// The result is in the **visual** order glyphs are drawn in, which for a
/// right-to-left run is not the order the characters were typed in. That
/// reordering is the whole point: it is what the per-`char` path cannot do, and
/// what makes Arabic render as words rather than as letters in reverse.
#[must_use]
pub fn run(bytes: &[u8], text: &str, size_px: f32) -> Option<Vec<Placed>> {
    run_directed(bytes, text, size_px, None)
}

/// [`run`], in the direction the bidi levels gave the text (`Some(true)` right to
/// left), or guessed from it with `None`.
///
/// New here (`T2D1-04`): a slide paragraph's runs are split by bidi level first,
/// so digits or spaces inside Arabic text are shaped in the direction their level
/// says, not the one their own characters would suggest.
#[must_use]
pub fn run_directed(
    bytes: &[u8],
    text: &str,
    size_px: f32,
    rtl: Option<bool>,
) -> Option<Vec<Placed>> {
    let font = FontRef::from_index(bytes, 0).ok()?;

    // Built per call rather than cached. The tables it derives are a real cost
    // and caching them is a real optimisation — and it is one to make against a
    // measurement, not on the way past: this path is off entirely in the
    // browser, and the native one draws a viewport of cells rather than a book.
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();

    // From the shaper rather than the font, because it is the shaper's own
    // notion of the design grid that its advances are expressed in.
    let upem = shaper.units_per_em();
    if upem <= 0 {
        return None;
    }
    let scale = size_px / upem as f32;

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    if let Some(rtl) = rtl {
        buffer.set_direction(if rtl {
            Direction::RightToLeft
        } else {
            Direction::LeftToRight
        });
    }
    // Direction and script are guessed from the text itself. Deliberately: the
    // alternative is a per-cell setting nobody fills in, and a wrong guess is
    // visible while an unset field is not.
    buffer.guess_segment_properties();

    let shaped = shaper.shape(buffer, ShapeOptions::new());
    let positions = shaped.glyph_positions();
    let infos = shaped.glyph_infos();
    Some(
        infos
            .iter()
            .zip(positions)
            .map(|(info, pos)| Placed {
                // `glyph_id` is a `u32` in the buffer and a `u16` in every font
                // table it indexes; a face with more glyphs than that cannot be
                // represented, so saturating is the honest conversion.
                id: u16::try_from(info.glyph_id).unwrap_or(u16::MAX),
                advance: pos.x_advance as f32 * scale,
                x_offset: pos.x_offset as f32 * scale,
                y_offset: pos.y_offset as f32 * scale,
                cluster: info.cluster,
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hebrew is right-to-left, and the bundled fonts cover it.
    ///
    /// This is the case that shaping actually fixes today. The per-`char` path
    /// walks the string in memory order and advances left to right, so a Hebrew
    /// word comes out **backwards**. Shaping returns glyphs in visual order, so
    /// the first glyph drawn is the last character typed.
    ///
    /// Asserted against a naive `cmap` lookup rather than against fixed ids,
    /// because the ids belong to whichever face is bundled and the *relationship*
    /// is what the feature is for.
    #[test]
    fn hebrew_comes_back_in_visual_order_which_is_the_reverse_of_memory_order() {
        use skrifa::MetadataProvider as _;

        let bytes = crate::faces::coverage_face_bytes('\u{05d0}', false, false)
            .expect("a bundled face covers Hebrew; if this fails the premise has moved");
        let glyphs = run(bytes, "\u{05d0}\u{05d1}\u{05d2}", 16.0).expect("shaped");

        let font = skrifa::FontRef::new(bytes).expect("readable");
        let charmap = font.charmap();
        let naive: Vec<u16> = "\u{05d0}\u{05d1}\u{05d2}"
            .chars()
            .map(|c| charmap.map(c).map(|g| g.to_u32() as u16).unwrap_or(0))
            .collect();
        let shaped: Vec<u16> = glyphs.iter().map(|g| g.id).collect();

        assert_eq!(shaped.len(), naive.len(), "same glyphs, different order");
        assert_ne!(
            shaped, naive,
            "shaping must not return memory order for a right-to-left run"
        );
        let mut reversed = naive.clone();
        reversed.reverse();
        assert_eq!(
            shaped, reversed,
            "visual order for this run is exactly memory order reversed"
        );
    }

    /// What is *not* fixed, stated as a test so it cannot be quietly assumed.
    ///
    /// Shaping is necessary and not sufficient: a script also needs a font that
    /// covers it, and the bundled families — Caladea, Carlito, Liberation,
    /// Roboto — cover Latin and Hebrew and not Devanagari, Thai or CJK; Arabic has
    /// the Noto Arabic faces since `T2D1-04`.
    /// Those render as `.notdef` today and would still do so with a shaper,
    /// because there are no glyphs to shape.
    #[test]
    fn the_scripts_the_bundled_fonts_do_not_cover_are_known() {
        let covered = |ch| crate::faces::coverage_face_bytes(ch, false, false).is_some();
        assert!(covered('A'), "Latin");
        assert!(covered('\u{05d0}'), "Hebrew");
        // Not a wish list — a record of what a bundle decision would have to
        // change, so that adding a font is a deliberate act with a size cost
        // rather than something discovered from a screenshot.
        //
        // CJK is not in this list: its faces ship as slices a host supplies
        // (`XL-FEAT-140`), so whether `中` is covered depends on what this
        // process has registered, which another test in this binary does.
        // Arabic is covered since `T2D1-04` (Noto Sans and Naskh Arabic).
        assert!(covered('\u{0645}'), "Arabic");
        for (script, ch) in [("Devanagari", '\u{0915}'), ("Thai", '\u{0e01}')] {
            assert!(!covered(ch), "{script} is now covered — update ADR-018");
        }
    }

    #[test]
    fn latin_still_shapes_to_one_glyph_per_letter() {
        // The regression guard. Shaping must not change what Latin looks like,
        // or every existing reference image is wrong.
        let bytes = crate::faces::face_bytes_for(None, false, false);
        let Some(glyphs) = run(bytes, "Total", 16.0) else {
            return;
        };
        assert_eq!(glyphs.len(), 5, "no ligature in this word: {glyphs:?}");
        assert!(glyphs.iter().all(|g| g.advance > 0.0));
    }
}
