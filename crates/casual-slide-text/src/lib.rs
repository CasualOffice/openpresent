//! `casual-slide-text`: fonts, shaping, line breaking and the layout of a text
//! body into positioned glyph runs (Phase 2, tier 2D; ADR-016).
//!
//! # Where this came from
//!
//! Started from OpenCalc's `casual-calc-text`, copied with provenance and changed
//! only where slides need it (owner instruction 42); see `PROVENANCE.md`. The
//! bundled faces, the substitution table, the CJK slices and the per-character
//! face resolution are OpenCalc's. Its shaper came from `casual-calc-render`.
//! Line breaking and paragraph and body layout are new: a spreadsheet cell does
//! not wrap the way a slide's text body does, so OpenCalc has neither.
//!
//! Tracker ids, ADR numbers and `docs/` links in the copied modules refer to
//! OpenCalc's documents unless they exist in this repository.
//!
//! # What is here
//!
//! - [`substitution`]: which bundled family stands in for a requested one.
//! - [`faces`]: the bundled bytes and per-character face resolution.
//! - [`cjk`] and [`woff2`]: the East Asian slices a host supplies.
//! - [`shape`]: text to positioned glyph ids, with `harfrust`.
//! - [`layout`]: paragraphs and a body, already resolved, to [`layout::GlyphRun`]s.
//! - [`body`]: a model text body, through the style chain, to that layout. The
//!   one call a renderer makes is [`layout_text_body`].
//! - [`outline`]: a run's face by a stable key, and its glyphs' outlines, which
//!   is all a backend needs to draw a run.
//!
//! Units are points throughout, `y` growing downwards, positions in the
//! coordinate space of the frame the caller passes. Outlines scale with the size
//! they are asked at, so a backend working in EMU asks in EMU.

pub mod body;
mod breaks;
pub mod cjk;
pub mod faces;
pub mod layout;
pub mod outline;
pub mod shape;
pub mod substitution;
pub mod woff2;

pub use body::{TextEnv, layout_text_body};
pub use layout::{CaretLine, Frame, Glyph, GlyphRun, TextLayout};
pub use outline::{OutlineOp, face_by_key, glyph_outline, outline_by_key};

use skrifa::{FontRef, MetadataProvider, instance::LocationRef, prelude::Size};

/// How wide `text` is, in pixels, in the face a request resolves to.
///
/// The sum of per-glyph advances, with the **same coverage fallback the
/// renderer draws with**: a character the chosen face has no glyph for is
/// measured in whichever face will actually be used for it. Measuring in one
/// face and drawing in another is how a string comes out a different width than
/// the box reserved for it.
///
/// Zero for an empty string, a non-positive size, or bytes that will not parse
/// as a font — every caller is sizing a box, and a box of nothing is a better
/// answer than a panic.
///
/// # A note on what this is *not*
///
/// This is not what a browser would measure. The editor canvas draws with the
/// system UI face and `measureText`; this measures the bundled face. The two
/// differ, and the headless render has always differed from the canvas in
/// exactly that way for cell text. What matters is that a string is measured in
/// the face it is drawn in, which is what makes a rectangle fit its contents.
#[must_use]
pub fn advance_width(text: &str, px: f32, bold: bool, italic: bool, family: Option<&str>) -> f32 {
    if text.is_empty() || !px.is_finite() || px <= 0.0 {
        return 0.0;
    }
    let faces = faces::TextFaces::new(family, bold, italic, text);
    text.chars()
        .map(|ch| char_advance(&faces.resolve(ch), ch, px))
        .sum()
}

/// How far one character advances the pen, in pixels at `px`, in the face it
/// resolved to.
///
/// With no face — a CJK slice that has not loaded yet — the width is the one
/// every CJK face gives it ([`cjk::nominal_advance_em`]): one em for an
/// ideograph, kana or hangul syllable, half for the half-width forms. So a
/// column measured before the face arrives is already the right width for
/// that text, and the renderers leave the same gap a drawn glyph would.
#[must_use]
pub fn char_advance(resolved: &faces::Resolved, ch: char, px: f32) -> f32 {
    let drawn = resolved.face.and_then(|bytes| {
        let font = FontRef::new(bytes).ok()?;
        let g = font.charmap().map(ch)?;
        font.glyph_metrics(Size::new(px), LocationRef::default())
            .advance_width(g)
    });
    drawn
        .or_else(|| resolved.nominal_em.map(|em| em * px))
        .unwrap_or(0.0)
}

#[cfg(test)]
mod cjk_tests;
#[cfg(test)]
mod tests;
