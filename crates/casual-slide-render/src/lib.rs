//! `casual-slide-render`: a slide's display list, and the CPU backend that paints it
//! to PNG.
//!
//! Started from OpenCalc's `casual-calc-render` (PROVENANCE.md, owner instruction
//! 42): the backend pattern, the picture source and its report, and the bounded
//! decoders are its; the item set and the scene are new, because a slide is vector
//! art rather than a grid.
//!
//! - [`display`]: the backend-neutral display list (`docs/42` §Display list).
//! - [`scene`]: a slide's resolved shapes to that list, through `casual-slide-style`
//!   and `casual-slide-geometry`.
//! - [`raster`]: the list onto a `tiny-skia` pixmap, then PNG.
//! - [`pdf`]: the same list as PDF, one page per slide, with real, extractable text
//!   in embedded, subset faces.
//! - [`Deck`]: a deck's bytes opened for all of them.
//!
//! Text is laid out by `casual-slide-text` into [`DisplayItem::GlyphRun`]s and
//! outlined from its faces ([`raster::BundledOutlines`]); a body that lays out to no
//! glyphs is counted in [`SceneReport::text_bodies_undrawn`], and
//! [`RenderReport::lines`] says so.

// rustdoc proves auto traits through the generated chart and equation types (here
// `scene::chart::SerParts` borrowing `c::DLbls`), which nest deeper than the default
// limit of 128 allows (E0275 on `math::schema::MC`). Compilation itself never needs
// it; `casual-slide-import` carries the same attribute for the same reason.
#![recursion_limit = "256"]

pub mod deck;
mod diagram;
pub mod display;
mod emf;
mod images;
pub mod pdf;
pub mod raster;
pub mod scene;
mod subset;

pub use deck::Deck;
pub use display::{DisplayItem, DisplayList};
pub use images::{
    ImageReport, ImageSource, MAX_IMAGE_PIXELS, MAX_IMAGE_SIDE, NoImages, UndrawnImage,
    UndrawnReason,
};
pub use pdf::{PdfMetadata, write_pdf};
pub use raster::{BundledOutlines, GlyphOutlines, render_pixmap, render_png};
pub use scene::{
    MediaResolver, NoMedia, SceneReport, SceneText, SlideScene, TextBox, build_slide,
    build_slide_text,
};

/// An error rendering a slide.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RenderError {
    /// The surface size is zero or too large.
    InvalidSize {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
    },
    /// PNG encoding failed.
    Encode,
    /// The deck has no slide at this index.
    NoSuchSlide(usize),
    /// The deck could not be opened; the reader's own code.
    Open(&'static str),
}

impl RenderError {
    /// The stable diagnostic code (`docs/20`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidSize { .. } => "CS-RND-0001",
            Self::Encode => "CS-RND-0002",
            Self::NoSuchSlide(_) => "CS-RND-0003",
            Self::Open(code) => code,
        }
    }
}

impl core::fmt::Display for RenderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidSize { width, height } => {
                write!(f, "[{}] invalid surface size {width}x{height}", self.code())
            }
            Self::Encode => write!(f, "[{}] PNG encoding failed", self.code()),
            Self::NoSuchSlide(i) => write!(f, "[{}] no slide at index {i}", self.code()),
            Self::Open(code) => write!(f, "[{code}] the deck could not be opened"),
        }
    }
}

impl std::error::Error for RenderError {}

/// What one render drew and did not. Nothing is left out silently.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderReport {
    /// Pictures drawn and not.
    pub images: ImageReport,
    /// Glyph runs not drawn because no outlines were supplied.
    pub glyph_runs_undrawn: u32,
    /// Glyphs whose outline was not found.
    pub glyphs_undrawn: u32,
    /// What the scene left out or approximated.
    pub scene: SceneReport,
}

impl RenderReport {
    /// The report as plain lines: counts and feature names, no document content.
    pub fn lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.scene.text_bodies_undrawn > 0 {
            out.push(format!(
                "text: {} text bodies laid out to no glyphs",
                self.scene.text_bodies_undrawn
            ));
        }
        if self.glyph_runs_undrawn > 0 {
            out.push(format!(
                "text: {} glyph runs not drawn, no outlines supplied",
                self.glyph_runs_undrawn
            ));
        }
        if self.glyphs_undrawn > 0 {
            out.push(format!(
                "text: {} glyphs had no outline",
                self.glyphs_undrawn
            ));
        }
        for (feature, n) in &self.scene.features {
            out.push(format!("{feature}: {n}"));
        }
        for miss in &self.images.undrawn {
            out.push(format!(
                "{}: {} ({})",
                miss.reason.feature(),
                miss.part,
                miss.reason
            ));
        }
        out
    }
}
