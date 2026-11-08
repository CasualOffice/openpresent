//! `casual-slide-wasm`: the `wasm-bindgen` bridge for the browser viewer.
//!
//! Started from OpenCalc's `casual-calc-wasm` (PROVENANCE.md, owner instruction 42):
//! a thin transport over the host-agnostic engine, holding one open document in a
//! thread-local session, so the browser keeps no model of its own. Here the session
//! is a [`Deck`] opened by `casual-slide-render`, and the surface is the first
//! vertical slice (owner instruction 44): open a deck, count its slides, draw one at
//! a given width, and say what that drawing left out. Slice 4 (`VS-04`) adds Save,
//! through the one writer, and PDF export, through `-render`'s PDF backend.
//!
//! Everything runs inside the page. Nothing here fetches, and the deck's bytes
//! never leave the browser.
//!
//! The logic lives in [`Viewer`], which is plain Rust and testable natively; the
//! exported functions only move values across the boundary.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use casual_slide_import::slide::SlideLimits;
use casual_slide_model::shape::ObjectId;
use casual_slide_model::slide::SlidePart;
use casual_slide_transaction::{History, TextOp};

use casual_slide_export::writer::SavePlan;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_render::{Deck, PdfMetadata, RenderError};
use wasm_bindgen::prelude::*;

mod edit;
pub use edit::{Move, Toggle};

/// The widest picture a host may ask for, in pixels. Twice a 2K screen, which is
/// more than a slide pane or a full-screen show at device pixels needs.
pub const MAX_WIDTH_PX: u32 = 4096;

/// The most pixels one picture may have. A slide size from the file sets the
/// aspect ratio, so a tall, thin slide at [`MAX_WIDTH_PX`] would otherwise ask for
/// a surface of `MAX_WIDTH_PX` by 16384; past this bound the width is reduced
/// instead, and the picture is smaller rather than refused.
pub const MAX_PIXELS: u64 = 4096 * 4096;

const EMU_PER_INCH: f64 = 914_400.0;

/// How many steps undo reaches back, as PowerPoint's default of 20 and then some.
pub const UNDO_DEPTH: usize = 100;

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
    /// The writer refused to save; its own code (`CS-SAV-*`, or the reader's).
    Save(&'static str),
    /// The shape has no text body, or no caret could be placed in it.
    NoText,
    /// An edit was refused; its own code (`CS-TXN-*`).
    Edit(&'static str),
    /// A bridge number named no move or attribute.
    BadArgument,
}

impl BridgeError {
    /// The stable diagnostic code (`docs/20`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoDeck => "CS-WSM-0001",
            Self::InvalidWidth(_) => "CS-WSM-0002",
            Self::Decode => "CS-WSM-0003",
            Self::Render(e) => e.code(),
            Self::Save(code) | Self::Edit(code) => code,
            Self::NoText => "CS-WSM-0004",
            Self::BadArgument => "CS-WSM-0005",
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
            Self::Save(code) => write!(f, "[{code}] the presentation could not be saved"),
            Self::NoText => write!(f, "[{}] that shape has no text to edit", self.code()),
            Self::Edit(code) => write!(f, "[{code}] the edit could not be made"),
            Self::BadArgument => write!(f, "[{}] an argument named nothing", self.code()),
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
    history: History,
    /// Ordinals of the parts an edit has touched: the only ones a save patches.
    edited: BTreeSet<u32>,
    /// The undo mark of the typing burst in progress, if any.
    typing: Option<u64>,
}

impl Viewer {
    /// Open a deck's bytes. A deck that cannot be read at all is refused with its
    /// reader's code; a damaged one opens with a best reading where the reader recovers one (`docs/29`).
    pub fn open(bytes: Vec<u8>) -> Result<Self, BridgeError> {
        Ok(Self {
            deck: Deck::open(bytes)?,
            reports: BTreeMap::new(),
            history: History::with_depth(Some(UNDO_DEPTH)),
            edited: BTreeSet::new(),
            typing: None,
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

    /// The deck as a file, through the one writer (`SavePlan`). With nothing
    /// edited, the bytes that were opened. After text edits, each edited part is
    /// patched to its model (ADR-013): only the nodes that differ change, and a part
    /// whose edits were all undone is written as it was.
    pub fn save(&self) -> Result<Vec<u8>, BridgeError> {
        let mut package =
            PresentationPackage::open(self.deck.source_bytes(), OoxmlLimits::default())
                .map_err(|e| BridgeError::Save(e.code()))?;
        let parts = self.deck.parts();
        let edited: Vec<&SlidePart> = self
            .edited
            .iter()
            .filter_map(|&ordinal| parts.get(ordinal))
            .collect();
        let limits = SlideLimits::default();
        let mut plan = SavePlan::new();
        if !edited.is_empty() {
            // The user chose to edit; a signed deck's signatures go with the edit, as
            // in PowerPoint, and the writer reports it.
            plan = plan
                .slide_parts(&parts.structure, &edited, &limits)
                .acknowledge_signature();
        }
        let outcome = plan
            .save(&mut package)
            .map_err(|e| BridgeError::Save(e.code()))?;
        Ok(outcome.bytes)
    }

    /// Every slide as one PDF, a page per slide, with real text. What each page
    /// left out replaces that slide's report.
    pub fn export_pdf(&mut self) -> Result<Vec<u8>, BridgeError> {
        let (pdf, reports) = self.deck.render_pdf(&PdfMetadata::default())?;
        for (index, report) in reports.iter().enumerate() {
            self.reports.insert(index, report.lines());
        }
        Ok(pdf)
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

/// The open deck as a `.pptx` file, through the writer. Unedited, it is the file
/// that was opened, byte for byte.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; the writer's own code if it refuses.
#[wasm_bindgen]
pub fn save_deck() -> Result<Vec<u8>, JsError> {
    with_viewer(|v| v.save())
}

/// Every slide of the open deck as one PDF, a page per slide, with real text.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck.
#[wasm_bindgen]
pub fn export_pdf() -> Result<Vec<u8>, JsError> {
    with_viewer(|v| v.export_pdf())
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

/* ── Text editing (`VS-05`) ─────────────────────────────────────────────── */

fn object(part: u32, node: u32) -> ObjectId {
    ObjectId { part, node }
}

fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// The text box under a point of slide `index`, in slide EMU: `[part, node, offset]`,
/// or empty when there is none.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn text_hit(index: u32, x: f64, y: f64) -> Result<Vec<f64>, JsError> {
    with_viewer(|v| {
        Ok(v.hit_text(index as usize, x, y)?
            .map(|(o, at)| vec![f64::from(o.part), f64::from(o.node), at as f64])
            .unwrap_or_default())
    })
}

/// The offset in a shape's body nearest a point in slide EMU, for a drag.
///
/// # Errors
///
/// `CS-WSM-0004` when the shape has no text.
#[wasm_bindgen]
pub fn text_offset_near(index: u32, part: u32, node: u32, x: f64, y: f64) -> Result<u32, JsError> {
    with_viewer(|v| {
        Ok(to_u32(v.offset_near(
            index as usize,
            object(part, node),
            x,
            y,
        )?))
    })
}

/// The caret at `offset`: `[x1, y1, x2, y2]`, its top and bottom in slide EMU.
///
/// # Errors
///
/// `CS-WSM-0004` when the shape has no text.
#[wasm_bindgen]
pub fn text_caret(index: u32, part: u32, node: u32, offset: u32) -> Result<Vec<f64>, JsError> {
    with_viewer(|v| {
        Ok(
            v.caret_rect(index as usize, object(part, node), offset as usize)?
                .to_vec(),
        )
    })
}

/// The selection `[start, end)` as quadrilaterals, eight numbers each, in slide EMU.
///
/// # Errors
///
/// `CS-WSM-0004` when the shape has no text.
#[wasm_bindgen]
pub fn text_selection(
    index: u32,
    part: u32,
    node: u32,
    start: u32,
    end: u32,
) -> Result<Vec<f64>, JsError> {
    with_viewer(|v| {
        v.selection_rects(
            index as usize,
            object(part, node),
            start as usize,
            end as usize,
        )
    })
}

/// Where the caret goes on Home (0), End (1), Up (2) or Down (3).
///
/// # Errors
///
/// `CS-WSM-0005` for another number; `CS-WSM-0004` when the shape has no text.
#[wasm_bindgen]
pub fn text_navigate(
    index: u32,
    part: u32,
    node: u32,
    offset: u32,
    how: u32,
) -> Result<u32, JsError> {
    with_viewer(|v| {
        let how = Move::from_code(how).ok_or(BridgeError::BadArgument)?;
        Ok(to_u32(v.navigate(
            index as usize,
            object(part, node),
            offset as usize,
            how,
        )?))
    })
}

/// A shape's text length in characters, a paragraph break counting one.
///
/// # Errors
///
/// `CS-WSM-0004` when the shape has no text.
#[wasm_bindgen]
pub fn text_length(part: u32, node: u32) -> Result<u32, JsError> {
    with_viewer(|v| Ok(to_u32(v.text_length(object(part, node))?)))
}

/// Insert `text` at `at`. `typing` joins it to the keystrokes before it in one undo step.
/// Returns whether anything changed.
///
/// # Errors
///
/// The edit's own `CS-TXN` code; nothing changes.
#[wasm_bindgen]
pub fn text_insert(
    part: u32,
    node: u32,
    at: u32,
    text: String,
    typing: bool,
) -> Result<bool, JsError> {
    with_viewer(|v| {
        v.edit(
            TextOp::InsertText {
                object: object(part, node),
                at: at as usize,
                text,
            },
            typing,
        )
    })
}

/// Delete `[start, end)`. Returns whether anything changed.
///
/// # Errors
///
/// The edit's own `CS-TXN` code; nothing changes.
#[wasm_bindgen]
pub fn text_delete(part: u32, node: u32, start: u32, end: u32) -> Result<bool, JsError> {
    with_viewer(|v| {
        v.edit(
            TextOp::DeleteRange {
                object: object(part, node),
                start: start.min(end) as usize,
                end: start.max(end) as usize,
            },
            false,
        )
    })
}

/// Toggle bold (0), italic (1) or underline (2) over `[start, end)`.
///
/// # Errors
///
/// `CS-WSM-0005` for another number; the edit's own `CS-TXN` code.
#[wasm_bindgen]
pub fn text_toggle(
    part: u32,
    node: u32,
    start: u32,
    end: u32,
    which: u32,
) -> Result<bool, JsError> {
    with_viewer(|v| {
        let which = Toggle::from_code(which).ok_or(BridgeError::BadArgument)?;
        v.toggle(object(part, node), start as usize, end as usize, which)
    })
}

/// Bits 1, 2 and 4: bold, italic and underline of the character before `offset`.
///
/// # Errors
///
/// `CS-WSM-0004` when the shape has no text.
#[wasm_bindgen]
pub fn text_format_state(part: u32, node: u32, offset: u32) -> Result<u32, JsError> {
    with_viewer(|v| v.format_state(object(part, node), offset as usize))
}

/// Undo the last edit: `[part, node]` of the shape it changed, or empty.
///
/// # Errors
///
/// The edit's own `CS-TXN` code, if the step no longer applies.
#[wasm_bindgen]
pub fn edit_undo() -> Result<Vec<u32>, JsError> {
    with_viewer(|v| Ok(v.undo()?.map(|o| vec![o.part, o.node]).unwrap_or_default()))
}

/// Redo the last undone edit: `[part, node]`, or empty.
///
/// # Errors
///
/// The edit's own `CS-TXN` code.
#[wasm_bindgen]
pub fn edit_redo() -> Result<Vec<u32>, JsError> {
    with_viewer(|v| Ok(v.redo()?.map(|o| vec![o.part, o.node]).unwrap_or_default()))
}

/// Bit 1: something to undo; bit 2: something to redo. Zero with no deck.
#[wasm_bindgen]
#[must_use]
pub fn edit_state() -> u32 {
    with_viewer(|v| Ok(v.undo_state())).unwrap_or(0)
}

/// End the typing burst: the next keystroke is its own undo step.
#[wasm_bindgen]
pub fn edit_break_typing() {
    let _ = with_viewer(|v| {
        v.break_typing();
        Ok(())
    });
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
    fn an_unedited_save_gives_back_the_bytes_that_were_opened() {
        for name in [
            "area-text.pptx",
            "image.pptx",
            "minimal.pptx",
            "chart-families.pptx",
            "comments-modern.pptx",
            "rr-notes.pptx",
            "graphics-diagram-every-row.pptx",
            "media-every-reference.pptx",
        ] {
            let bytes = fixture(name);
            let mut v = Viewer::open(bytes.clone()).unwrap();
            // Drawing first must not change what is saved.
            v.render_png(0, 64).unwrap();
            assert!(v.save().unwrap() == bytes, "{name}: byte-identical");
        }
    }

    #[test]
    fn the_pdf_has_a_page_per_slide_and_replaces_the_reports() {
        let mut v = Viewer::open(fixture("area-text.pptx")).unwrap();
        let pdf = v.export_pdf().unwrap();
        assert!(pdf.starts_with(b"%PDF-"));
        let text = String::from_utf8_lossy(&pdf);
        assert_eq!(text.matches("/Type /Page ").count(), v.slide_count());
        assert_eq!(v.reports.len(), v.slide_count());
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
