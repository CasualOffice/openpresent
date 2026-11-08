//! `casual-slide-wasm`: the `wasm-bindgen` bridge for the browser viewer.
//!
//! Started from OpenCalc's `casual-calc-wasm` (PROVENANCE.md, owner instruction 42):
//! a thin transport over the host-agnostic engine, holding one open document in a
//! thread-local session, so the browser keeps no model of its own. Here the session
//! is a [`Deck`] opened by `casual-slide-render`, and the surface is the first
//! vertical slice (owner instruction 44): open a deck, count its slides, draw one at
//! a given width, and say what that drawing left out.
//!
//! Everything runs inside the page. Nothing here fetches, and the deck's bytes
//! never leave the browser.
//!
//! The logic lives in [`Viewer`], which is plain Rust and testable natively; the
//! exported functions only move values across the boundary.

use std::cell::RefCell;
use std::collections::BTreeMap;

use casual_slide_render::{Deck, RenderError};
use wasm_bindgen::prelude::*;

/// The widest picture a host may ask for, in pixels. Twice a 2K screen, which is
/// more than a slide pane or a full-screen show at device pixels needs.
pub const MAX_WIDTH_PX: u32 = 4096;

/// The most pixels one picture may have. A slide size from the file sets the
/// aspect ratio, so a tall, thin slide at [`MAX_WIDTH_PX`] would otherwise ask for
/// a surface of `MAX_WIDTH_PX` by 16384; past this bound the width is reduced
/// instead, and the picture is smaller rather than refused.
pub const MAX_PIXELS: u64 = 4096 * 4096;

const EMU_PER_INCH: f64 = 914_400.0;

/// An error from the bridge. Each has a stable code (`docs/20`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BridgeError {
    /// A call that needs an open deck came before any was opened.
    NoDeck,
    /// The requested width is zero or over [`MAX_WIDTH_PX`].
    InvalidWidth(u32),
    /// The PNG the renderer wrote could not be decoded back for RGBA.
    Decode,
    /// The renderer's own error, with its own code.
    Render(RenderError),
}

impl BridgeError {
    /// The stable diagnostic code (`docs/20`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoDeck => "CS-WSM-0001",
            Self::InvalidWidth(_) => "CS-WSM-0002",
            Self::Decode => "CS-WSM-0003",
            Self::Render(e) => e.code(),
        }
    }
}

impl core::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoDeck => write!(f, "[{}] no deck is open", self.code()),
            Self::InvalidWidth(w) => write!(
                f,
                "[{}] width {w} is outside 1 to {MAX_WIDTH_PX} pixels",
                self.code()
            ),
            Self::Decode => write!(
                f,
                "[{}] the rendered picture could not be read back",
                self.code()
            ),
            Self::Render(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for BridgeError {}

impl From<RenderError> for BridgeError {
    fn from(e: RenderError) -> Self {
        Self::Render(e)
    }
}

/// One opened deck, and what each slide's last drawing left out.
#[derive(Debug)]
pub struct Viewer {
    deck: Deck,
    reports: BTreeMap<usize, Vec<String>>,
}

impl Viewer {
    /// Open a deck's bytes. A deck that cannot be read at all is refused with its
    /// reader's code; a damaged one opens with a best reading (`docs/29`).
    pub fn open(bytes: Vec<u8>) -> Result<Self, BridgeError> {
        Ok(Self {
            deck: Deck::open(bytes)?,
            reports: BTreeMap::new(),
        })
    }

    /// How many slides the deck has.
    pub fn slide_count(&self) -> usize {
        self.deck.slide_count()
    }

    /// The slide size in EMU, width then height. Always positive: a deck that
    /// gives none, or a size of nothing, gets PowerPoint's default.
    pub fn slide_size(&self) -> (i64, i64) {
        self.deck.size()
    }

    /// The dots per inch that draw a slide `width` pixels wide, after the
    /// [`MAX_PIXELS`] bound.
    fn dpi_for(&self, width: u32) -> Result<f64, BridgeError> {
        if width == 0 || width > MAX_WIDTH_PX {
            return Err(BridgeError::InvalidWidth(width));
        }
        let (cx, cy) = self.slide_size();
        // Both are positive (`Deck::size`); `max(1)` keeps that true if it changes.
        let (cx, cy) = (cx.max(1) as f64, cy.max(1) as f64);
        let mut w = f64::from(width);
        let h = w * cy / cx;
        let pixels = w * h;
        if pixels > MAX_PIXELS as f64 {
            w *= (MAX_PIXELS as f64 / pixels).sqrt();
        }
        Ok(w.max(1.0) * EMU_PER_INCH / cx)
    }

    /// Draw slide `index`, counted from 0, `width` pixels wide, as PNG. The
    /// height follows the slide's aspect ratio.
    pub fn render_png(&mut self, index: usize, width: u32) -> Result<Vec<u8>, BridgeError> {
        let dpi = self.dpi_for(width)?;
        let (png, report) = self.deck.render_slide_png(index, dpi, None)?;
        self.reports.insert(index, report.lines());
        Ok(png)
    }

    /// Draw slide `index` as straight (not premultiplied) RGBA, four bytes a
    /// pixel, row by row; returns the width and the pixels. For a host that paints
    /// a canvas from `ImageData`.
    pub fn render_rgba(&mut self, index: usize, width: u32) -> Result<(u32, Vec<u8>), BridgeError> {
        let png = self.render_png(index, width)?;
        let pixmap = tiny_skia::Pixmap::decode_png(&png).map_err(|_| BridgeError::Decode)?;
        let mut out = Vec::with_capacity(pixmap.data().len());
        for p in pixmap.pixels() {
            let c = p.demultiply();
            out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
        }
        Ok((pixmap.width(), out))
    }

    /// What slide `index` left out, as plain lines with counts and feature names
    /// and no document content. Taken from its last drawing; a slide not yet
    /// drawn is drawn small to find out.
    pub fn report(&mut self, index: usize) -> Result<Vec<String>, BridgeError> {
        if !self.reports.contains_key(&index) {
            self.render_png(index, 64)?;
        }
        Ok(self.reports.get(&index).cloned().unwrap_or_default())
    }
}

thread_local! {
    static SESSION: RefCell<Option<Viewer>> = const { RefCell::new(None) };
}

fn js(e: BridgeError) -> JsError {
    JsError::new(&e.to_string())
}

fn with_viewer<T>(f: impl FnOnce(&mut Viewer) -> Result<T, BridgeError>) -> Result<T, JsError> {
    SESSION.with(|s| match s.borrow_mut().as_mut() {
        Some(v) => f(v).map_err(js),
        None => Err(js(BridgeError::NoDeck)),
    })
}

/// The engine's version.
#[wasm_bindgen]
#[must_use]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Open a deck's bytes into the session, replacing any deck open before. A deck
/// that cannot be opened leaves the previous one in place.
///
/// # Errors
///
/// The reader's own code when the bytes cannot be read as a presentation at all.
#[wasm_bindgen]
pub fn open(bytes: Vec<u8>) -> Result<(), JsError> {
    let viewer = Viewer::open(bytes).map_err(js)?;
    SESSION.with(|s| *s.borrow_mut() = Some(viewer));
    Ok(())
}

/// Close the open deck, dropping its bytes.
#[wasm_bindgen]
pub fn close() {
    SESSION.with(|s| *s.borrow_mut() = None);
}

/// How many slides the open deck has.
///
/// # Errors
///
/// `CS-WSM-0001` when no deck is open.
#[wasm_bindgen]
pub fn slide_count() -> Result<u32, JsError> {
    with_viewer(|v| Ok(u32::try_from(v.slide_count()).unwrap_or(u32::MAX)))
}

/// The slide size in EMU, `[width, height]`.
///
/// # Errors
///
/// `CS-WSM-0001` when no deck is open.
#[wasm_bindgen]
pub fn slide_size() -> Result<Vec<f64>, JsError> {
    with_viewer(|v| {
        let (cx, cy) = v.slide_size();
        Ok(vec![cx as f64, cy as f64])
    })
}

/// Draw slide `index` (from 0) `width` pixels wide, as PNG bytes.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck, `CS-WSM-0002` for a width outside 1 to 4096,
/// `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn render_slide_png(index: u32, width: u32) -> Result<Vec<u8>, JsError> {
    with_viewer(|v| v.render_png(index as usize, width))
}

/// Draw slide `index` (from 0) about `width` pixels wide, as straight RGBA. The
/// width actually drawn is `pixels.length / 4 / height`; ask
/// [`last_rgba_width`] for it.
///
/// # Errors
///
/// As [`render_slide_png`], and `CS-WSM-0003` if the picture cannot be read back.
#[wasm_bindgen]
pub fn render_slide_rgba(index: u32, width: u32) -> Result<Vec<u8>, JsError> {
    with_viewer(|v| {
        let (w, pixels) = v.render_rgba(index as usize, width)?;
        LAST_RGBA_WIDTH.with(|c| c.set(w));
        Ok(pixels)
    })
}

thread_local! {
    static LAST_RGBA_WIDTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// The width of the picture the last [`render_slide_rgba`] returned.
#[wasm_bindgen]
#[must_use]
pub fn last_rgba_width() -> u32 {
    LAST_RGBA_WIDTH.with(std::cell::Cell::get)
}

/// What slide `index` left out when drawn, one line each: counts, feature names
/// and part names, never document content.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn slide_report(index: u32) -> Result<Vec<String>, JsError> {
    with_viewer(|v| v.report(index as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = format!(
            "{}/../../fixtures/generated/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read(path).unwrap()
    }

    #[test]
    fn opens_a_deck_and_draws_each_slide_at_the_asked_width() {
        let mut v = Viewer::open(fixture("area-text.pptx")).unwrap();
        assert!(v.slide_count() > 0);
        let (w, rgba) = v.render_rgba(0, 320).unwrap();
        assert_eq!(w, 320);
        let (cx, cy) = v.slide_size();
        let h = rgba.len() / 4 / w as usize;
        let expected = (320.0 * cy as f64 / cx as f64).round() as usize;
        assert!(h.abs_diff(expected) <= 1, "height {h}, expected {expected}");
        let png = v.render_png(0, 200).unwrap();
        assert_eq!(&png[1..4], b"PNG");
    }

    #[test]
    fn refuses_a_width_of_zero_or_past_the_bound() {
        let mut v = Viewer::open(fixture("area-text.pptx")).unwrap();
        assert_eq!(v.render_png(0, 0).unwrap_err().code(), "CS-WSM-0002");
        assert_eq!(
            v.render_png(0, MAX_WIDTH_PX + 1).unwrap_err().code(),
            "CS-WSM-0002"
        );
    }

    #[test]
    fn a_slide_past_the_end_is_its_render_code() {
        let mut v = Viewer::open(fixture("area-text.pptx")).unwrap();
        let past = v.slide_count();
        assert_eq!(v.render_png(past, 100).unwrap_err().code(), "CS-RND-0003");
        assert_eq!(v.report(past).unwrap_err().code(), "CS-RND-0003");
    }

    #[test]
    fn bytes_that_are_no_deck_are_refused_with_the_readers_code() {
        let err = Viewer::open(b"not a zip".to_vec()).unwrap_err();
        assert!(err.code().starts_with("CS-"), "{}", err.code());
        assert_ne!(err.code(), "CS-WSM-0001");
    }

    #[test]
    fn the_report_of_an_undrawn_slide_draws_it_first() {
        let mut v = Viewer::open(fixture("image.pptx")).unwrap();
        assert!(v.reports.is_empty());
        v.report(0).unwrap();
        assert!(v.reports.contains_key(&0));
    }

    #[test]
    fn the_pixel_bound_shrinks_the_width_rather_than_refusing() {
        let v = Viewer::open(fixture("area-text.pptx")).unwrap();
        let dpi = v.dpi_for(MAX_WIDTH_PX).unwrap();
        let (cx, cy) = v.slide_size();
        let w = cx as f64 * dpi / EMU_PER_INCH;
        let h = cy as f64 * dpi / EMU_PER_INCH;
        assert!(w * h <= MAX_PIXELS as f64 + 1.0);
    }
}
