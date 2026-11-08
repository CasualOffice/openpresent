//! A deck opened for drawing: the model, each master's theme, every part's internal
//! relationships, and the media its slides draw, read only when a slide needs it.
//!
//! Opening goes through the same readers as everything else (`-ooxml`, `-import`),
//! so a damaged deck is recovered, not refused (`docs/29`). Nothing is fetched:
//! an external relationship resolves to nothing, and its picture is reported.

use std::collections::{BTreeMap, BTreeSet};

use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_import::theme::import_theme_limited;
use casual_slide_model::slide::{SlideKind, SlideParts};
use casual_slide_model::text::TextListStyle;
use casual_slide_model::theme::Theme;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, TargetMode, XmlTree, resolve_internal};

use crate::display::{DisplayItem, DisplayList, Paint};
use crate::raster::{BundledOutlines, GlyphOutlines, render_png};
use crate::scene::{MediaResolver, SceneReport, SlideScene, build_slide};
use crate::{RenderError, RenderReport};

/// The largest media part read for drawing. Over it, the picture is reported as not
/// supplied.
pub const MAX_MEDIA_BYTES: u64 = 64 * 1024 * 1024;

/// The slide size used when the deck states none, or one of no size: the schema
/// requires `sldSz`, and PowerPoint's 4:3 default is assumed.
pub const DEFAULT_SLIDE_SIZE: (i64, i64) = (9_144_000, 6_858_000);

/// Internal relationship targets, by part ordinal then relationship id.
#[derive(Debug, Default)]
struct Relationships(BTreeMap<u32, BTreeMap<String, String>>);

impl MediaResolver for Relationships {
    fn media_part(&self, ordinal: u32, rel_id: &str) -> Option<String> {
        self.0.get(&ordinal)?.get(rel_id).cloned()
    }
}

/// A deck ready to draw.
pub struct Deck {
    package: PresentationPackage,
    parts: SlideParts,
    themes: BTreeMap<u32, Theme>,
    rels: Relationships,
    size: (i64, i64),
    default_text_style: Option<TextListStyle>,
    media: BTreeMap<String, Vec<u8>>,
    unreadable_media: BTreeSet<String>,
    /// Masters whose theme could not be read.
    pub themes_unread: u32,
}

impl core::fmt::Debug for Deck {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Deck")
            .field("slides", &self.slide_count())
            .field("size", &self.size)
            .field("themes", &self.themes.len())
            .finish_non_exhaustive()
    }
}

impl Deck {
    /// Open a deck's bytes under the default limits.
    pub fn open(bytes: Vec<u8>) -> Result<Deck, RenderError> {
        let limits = ImportLimits::default();
        let mut package = PresentationPackage::open(bytes, OoxmlLimits::default())
            .map_err(|e| RenderError::Open(e.code()))?;
        let deck = import_deck(&mut package, &limits)
            .map_err(|e| RenderError::Open(e.code()))?
            .deck;
        let parts = import_slide_parts(
            &mut package,
            &deck.presentation,
            &SlideLimits::default(),
            &limits,
        )
        .map_err(|e| RenderError::Open(e.code()))?
        .parts;
        let size = deck
            .presentation
            .slide_size
            .map(|s| (i64::from(s.cx), i64::from(s.cy)))
            // A size of nothing cannot be drawn; the default stands in, as for none.
            .filter(|&(w, h)| w > 0 && h > 0)
            .unwrap_or(DEFAULT_SLIDE_SIZE);

        let mut themes = BTreeMap::new();
        let mut themes_unread = 0u32;
        let xml_limits = *package.limits();
        for entry in &parts.structure.parts {
            if entry.kind != SlideKind::Master && entry.kind != SlideKind::NotesMaster {
                continue;
            }
            let Some(name) = entry.theme.as_deref() else {
                continue;
            };
            let theme = package
                .read_xml_part(name)
                .ok()
                .and_then(|bytes| XmlTree::parse(&bytes, name, &xml_limits).ok())
                .and_then(|tree| import_theme_limited(&tree, &xml_limits).ok());
            match theme {
                Some(t) => {
                    themes.insert(entry.id.part, t.value);
                }
                None => themes_unread = themes_unread.saturating_add(1),
            }
        }

        let mut rels = Relationships::default();
        for entry in &parts.structure.parts {
            let mut map = BTreeMap::new();
            for rel in package.relationships_of(&entry.part) {
                if rel.mode != TargetMode::Internal {
                    continue;
                }
                if let Some(target) = resolve_internal(&entry.part, &rel.target) {
                    map.insert(rel.id.clone(), target);
                }
            }
            rels.0.insert(entry.id.part, map);
        }

        Ok(Deck {
            package,
            parts,
            themes,
            rels,
            size,
            default_text_style: deck.presentation.default_text_style.clone(),
            media: BTreeMap::new(),
            unreadable_media: BTreeSet::new(),
            themes_unread,
        })
    }

    /// How many slides the deck has.
    pub fn slide_count(&self) -> usize {
        self.parts.structure.slides.len()
    }

    /// The slide size, EMU.
    pub fn size(&self) -> (i64, i64) {
        self.size
    }

    /// The display list of slide `index`, counted from 0.
    pub fn display_list(&self, index: usize) -> Result<(DisplayList, SceneReport), RenderError> {
        let ordinal = self
            .parts
            .structure
            .slide(index)
            .ok_or(RenderError::NoSuchSlide(index))?;
        Ok(self.display_list_of_part(ordinal))
    }

    /// The display list of any slide-like part by ordinal: a layout or a master is
    /// drawn as a slide of its own.
    pub fn display_list_of_part(&self, ordinal: u32) -> (DisplayList, SceneReport) {
        let master = casual_slide_style::Chain::of(&self.parts, ordinal)
            .master
            .map(|m| m.id.part);
        let scene = SlideScene {
            parts: &self.parts,
            theme: master.and_then(|m| self.themes.get(&m)),
            size: self.size,
            media: &self.rels,
            default_text_style: self.default_text_style.as_ref(),
        };
        build_slide(scene, ordinal)
    }

    /// The deck's slide-like parts, as read.
    pub fn parts(&self) -> &SlideParts {
        &self.parts
    }

    /// Read the media a display list draws, once each.
    fn load_media(&mut self, list: &DisplayList) {
        for item in &list.items {
            let DisplayItem::Path { fill, stroke, .. } = item else {
                continue;
            };
            let paints = fill.iter().chain(stroke.iter().map(|s| &s.paint));
            for paint in paints {
                let Paint::Picture { part, .. } = paint else {
                    continue;
                };
                if self.media.contains_key(part) || self.unreadable_media.contains(part) {
                    continue;
                }
                match self.package.read_part_capped(part, MAX_MEDIA_BYTES) {
                    Ok(bytes) => {
                        self.media.insert(part.clone(), bytes);
                    }
                    Err(_) => {
                        self.unreadable_media.insert(part.clone());
                    }
                }
            }
        }
    }

    /// Render slide `index` to PNG at `dpi`. Glyphs are outlined by `glyphs` if
    /// given, else from `casual-slide-text`'s faces.
    pub fn render_slide_png(
        &mut self,
        index: usize,
        dpi: f64,
        glyphs: Option<&dyn GlyphOutlines>,
    ) -> Result<(Vec<u8>, RenderReport), RenderError> {
        let (list, scene) = self.display_list(index)?;
        self.load_media(&list);
        let glyphs = glyphs.unwrap_or(&BundledOutlines);
        let (png, mut report) = render_png(&list, dpi, &self.media, Some(glyphs))?;
        report.scene = scene;
        Ok((png, report))
    }
}
