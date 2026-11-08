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
use casual_slide_model::shape::{ObjectId, ShapeNode};
use casual_slide_model::slide::{SlideKind, SlideParts};
use casual_slide_model::table::TableStyleList;
use casual_slide_model::text::TextListStyle;
use casual_slide_model::theme::Theme;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, TargetMode, XmlTree, resolve_internal};

use crate::display::{DisplayItem, DisplayList, Paint};
use crate::raster::{BundledOutlines, GlyphOutlines, render_png};
use crate::scene::{
    MediaResolver, SceneReport, SceneText, SlideScene, build_slide, build_slide_text,
};
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
    table_styles: Option<TableStyleList>,
    charts: casual_slide_import::chart::DeckCharts,
    diagrams: BTreeMap<ObjectId, Vec<ShapeNode>>,
    media: BTreeMap<String, Vec<u8>>,
    unreadable_media: BTreeSet<String>,
    /// Masters whose theme could not be read.
    pub themes_unread: u32,
    /// The property parts, as read or as edited since (File > Info).
    properties: casual_slide_model::properties::DocumentProperties,
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
        let imported = import_slide_parts(
            &mut package,
            &deck.presentation,
            &SlideLimits::default(),
            &limits,
        )
        .map_err(|e| RenderError::Open(e.code()))?;
        let parts = imported.parts;
        let table_styles = imported.table_styles;
        let charts = imported.charts;
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

        // Each SmartArt frame's stored drawing (`SEC-C12`: no layout is run).
        let mut diagrams = BTreeMap::new();
        let graphic_limits = casual_slide_import::diagram::GraphicLimits::default();
        let mut findings = casual_slide_import::properties::Findings::new(limits.max_findings);
        for entry in &parts.structure.parts {
            let Some(part) = parts.get(entry.id.part) else {
                continue;
            };
            let found = casual_slide_import::diagram::import_slide_diagrams(
                &mut package,
                &entry.part,
                part,
                &graphic_limits,
                &limits,
                &mut findings,
            );
            for (frame, diagram) in found {
                if let Some(drawing) = diagram.drawing.as_ref() {
                    diagrams.insert(frame, crate::diagram::drawing_nodes(frame, drawing));
                }
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
            table_styles,
            charts,
            diagrams,
            media: BTreeMap::new(),
            unreadable_media: BTreeSet::new(),
            themes_unread,
            properties: deck.properties,
        })
    }

    /// How many slides the deck has.
    pub fn slide_count(&self) -> usize {
        self.parts.structure.slides.len()
    }

    /// The document properties (`docProps/core.xml`, `app.xml`, `custom.xml`), as read
    /// or as edited since. The file is not touched until a save.
    pub fn properties(&self) -> &casual_slide_model::properties::DocumentProperties {
        &self.properties
    }

    /// The document properties, to edit (File > Info). The file is not touched until
    /// a save.
    pub fn properties_mut(&mut self) -> &mut casual_slide_model::properties::DocumentProperties {
        &mut self.properties
    }

    /// The slide size, EMU.
    pub fn size(&self) -> (i64, i64) {
        self.size
    }

    /// Every character the deck's text uses, in slides, layouts, masters, tables and
    /// stored SmartArt drawings: what a host looks up before rendering, to load the
    /// CJK slices (`casual_slide_text::cjk::SliceDirectory`) that draw them.
    pub fn characters(&self) -> BTreeSet<char> {
        use casual_slide_model::text::{TextBody, TextItem};
        fn body(out: &mut BTreeSet<char>, b: &TextBody) {
            for p in &b.paragraphs {
                for item in &p.items {
                    let text = match item {
                        TextItem::Run(r) => r.text.as_deref(),
                        TextItem::Field(f) => f.text.as_deref(),
                        _ => None,
                    };
                    out.extend(text.unwrap_or_default().chars());
                }
            }
        }
        fn node(out: &mut BTreeSet<char>, n: &ShapeNode) {
            for n in n.descendants() {
                if let Some(b) = n.text_body.as_deref() {
                    body(out, b);
                }
                let cells = n
                    .table
                    .iter()
                    .flat_map(|t| t.rows.iter())
                    .flat_map(|r| &r.cells);
                for b in cells.filter_map(|c| c.tx_body.as_deref()) {
                    body(out, b);
                }
            }
        }
        let mut out = BTreeSet::new();
        let trees = self
            .parts
            .parts
            .iter()
            .flatten()
            .filter_map(|p| p.common.as_ref()?.shape_tree.as_ref());
        for tree in trees {
            node(&mut out, tree);
        }
        for drawn in self.diagrams.values().flatten() {
            node(&mut out, drawn);
        }
        out
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
        build_slide(self.scene(ordinal), ordinal)
    }

    /// The text boxes of slide `index`, and the caret lines of `capture` if it is one
    /// of them (`VS-05`).
    pub fn slide_text(
        &self,
        index: usize,
        capture: Option<ObjectId>,
    ) -> Result<SceneText, RenderError> {
        let ordinal = self
            .parts
            .structure
            .slide(index)
            .ok_or(RenderError::NoSuchSlide(index))?;
        Ok(build_slide_text(self.scene(ordinal), ordinal, capture).2)
    }

    /// The levels shape `object`'s text inherits from, as the slide draws it: what an
    /// editor reads to know a run's state, its own or inherited (`VS-19`). `None`
    /// when no such shape is in the deck.
    pub fn text_chain(&self, object: ObjectId) -> Option<casual_slide_style::text::TextChain<'_>> {
        let node = self.parts.get(object.part)?.node(object)?;
        let chain = casual_slide_style::Chain::of(&self.parts, object.part);
        let defaults = chain
            .master
            .and_then(|m| self.themes.get(&m.id.part))
            .and_then(|t| t.object_defaults.as_ref());
        Some(chain.text_chain(node, self.default_text_style.as_ref(), defaults))
    }

    fn scene(&self, ordinal: u32) -> SlideScene<'_> {
        let master = casual_slide_style::Chain::of(&self.parts, ordinal)
            .master
            .map(|m| m.id.part);
        SlideScene {
            parts: &self.parts,
            theme: master.and_then(|m| self.themes.get(&m)),
            size: self.size,
            media: &self.rels,
            default_text_style: self.default_text_style.as_ref(),
            table_styles: self.table_styles.as_ref(),
            charts: Some(&self.charts),
            diagrams: Some(&self.diagrams),
        }
    }

    /// The theme slide `index` draws with, its master's: for a host's colour pickers
    /// (`SHAPE-01`).
    pub fn theme_of_slide(&self, index: usize) -> Option<&Theme> {
        let ordinal = self.parts.structure.slide(index)?;
        self.scene(ordinal).theme
    }

    /// The file's bytes exactly as they arrived, for a save that starts from them.
    pub fn source_bytes(&self) -> Vec<u8> {
        self.package.save_unedited()
    }

    /// The deck's slide-like parts, as read, or as edited since.
    pub fn parts(&self) -> &SlideParts {
        &self.parts
    }

    /// The deck's slide-like parts, to edit (`VS-06`). The next drawing draws them as
    /// they then stand; the file is not touched until a save.
    pub fn parts_mut(&mut self) -> &mut SlideParts {
        &mut self.parts
    }

    /// Each master's theme by the master's part ordinal, as read or as edited since
    /// (`VS-12`). A master whose theme could not be read has none.
    pub fn themes(&self) -> &BTreeMap<u32, Theme> {
        &self.themes
    }

    /// The theme of the master with this ordinal, to edit (`VS-12`). The next drawing
    /// uses it; the file is not touched until a save.
    pub fn theme_mut(&mut self, master: u32) -> Option<&mut Theme> {
        self.themes.get_mut(&master)
    }

    /// The deck's table styles (`tableStyles`), as read (`VS-11`).
    pub fn table_styles(&self) -> Option<&TableStyleList> {
        self.table_styles.as_ref()
    }

    /// A chart part as read, or as edited since (`VS-11`). `None` for one set aside.
    pub fn chart(&self, part: &str) -> Option<&casual_slide_model::chart::ChartPart> {
        match &self.charts.get(part)?.status {
            casual_slide_import::chart::ChartStatus::Read(p) => Some(p),
            casual_slide_import::chart::ChartStatus::SetAside { .. } => None,
        }
    }

    /// A chart part, to edit (`VS-11`). The next drawing draws it; the file is not
    /// touched until a save.
    pub fn chart_mut(&mut self, part: &str) -> Option<&mut casual_slide_model::chart::ChartPart> {
        let chart = self.charts.parts.iter_mut().find(|p| p.part == part)?;
        match &mut chart.status {
            casual_slide_import::chart::ChartStatus::Read(p) => Some(p),
            casual_slide_import::chart::ChartStatus::SetAside { .. } => None,
        }
    }

    /// Draw relationship `rel_id` of part `ordinal` as this chart: one inserted since the
    /// deck was opened, which no part of the file holds yet (`VS-11`). `key` names it
    /// within this session and never reaches the file.
    pub fn add_chart(
        &mut self,
        ordinal: u32,
        rel_id: &str,
        key: &str,
        chart: casual_slide_model::chart::ChartPart,
    ) {
        self.rels
            .0
            .entry(ordinal)
            .or_default()
            .insert(rel_id.to_owned(), key.to_owned());
        self.charts.parts.retain(|p| p.part != key);
        self.charts
            .parts
            .push(casual_slide_import::chart::DeckChart {
                part: key.to_owned(),
                owner: String::new(),
                status: casual_slide_import::chart::ChartStatus::Read(chart),
            });
    }

    /// The part relationship `rel_id` of part `ordinal` lands on.
    pub fn related_part(&self, ordinal: u32, rel_id: &str) -> Option<String> {
        self.rels.media_part(ordinal, rel_id)
    }

    /// The slide size, EMU, to edit (`VS-12`). The caller keeps both sides positive.
    pub fn size_mut(&mut self) -> &mut (i64, i64) {
        &mut self.size
    }

    /// A part's bytes, as the file holds them, for a host building a new part from one
    /// (`VS-08`). `None` for a part the file does not have or that cannot be read.
    pub fn read_part(&mut self, name: &str) -> Option<Vec<u8>> {
        self.package.read_xml_part(name).ok()
    }

    /// The file's conformance class, which a new part's namespaces follow.
    pub fn conformance(&self) -> casual_slide_ooxml::Conformance {
        self.package.conformance()
    }

    /// Give part `ordinal` the relationships part `from` has, so a copy of a slide
    /// draws the pictures its original does before it is saved (`VS-08`).
    pub fn share_relationships(&mut self, from: u32, ordinal: u32) {
        let map = self.rels.0.get(&from).cloned().unwrap_or_default();
        self.rels.0.insert(ordinal, map);
    }

    /// Draw relationship `rel_id` of part `ordinal` as these image bytes: a picture
    /// inserted since the deck was opened, which no part of the file holds yet (`VS-08`).
    /// `key` names the media within this session and never reaches the file.
    pub fn add_media(&mut self, ordinal: u32, rel_id: &str, key: &str, bytes: Vec<u8>) {
        self.rels
            .0
            .entry(ordinal)
            .or_default()
            .insert(rel_id.to_owned(), key.to_owned());
        self.unreadable_media.remove(key);
        self.media.insert(key.to_owned(), bytes);
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

    /// Write every slide as one PDF, a page per slide, with real text in embedded,
    /// subset faces. Returns the file and, per slide, what it did not draw or drew
    /// approximately.
    ///
    /// # Errors
    ///
    /// [`RenderError::InvalidSize`] only if the slide size were zero, which
    /// [`Deck::open`] never leaves it.
    pub fn render_pdf(
        &mut self,
        meta: &crate::pdf::PdfMetadata,
    ) -> Result<(Vec<u8>, Vec<RenderReport>), RenderError> {
        let mut lists = Vec::with_capacity(self.slide_count());
        let mut scenes = Vec::with_capacity(self.slide_count());
        for index in 0..self.slide_count() {
            let (list, scene) = self.display_list(index)?;
            self.load_media(&list);
            lists.push(list);
            scenes.push(scene);
        }
        let refs: Vec<&DisplayList> = lists.iter().collect();
        let (pdf, mut reports) = crate::pdf::write_pdf(&refs, &self.media, meta)?;
        for (report, scene) in reports.iter_mut().zip(scenes) {
            // The writer's own notes join the scene's, which come first.
            let own = std::mem::take(&mut report.scene.features);
            report.scene = scene;
            for (feature, n) in own {
                let total = report.scene.features.entry(feature).or_insert(0);
                *total = total.saturating_add(n);
            }
        }
        Ok((pdf, reports))
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
