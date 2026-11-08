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
use std::collections::BTreeMap;

use casual_slide_export::writer::Op;
use casual_slide_model::presentation::{Coordinate, POSITIVE_COORDINATE_MAX, Point, PositiveSize};
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode};
use casual_slide_model::slide::{SlidePart, SlideParts};
use casual_slide_ooxml::{PresentationPackage, XmlTree};
use casual_slide_render::{Deck, PdfMetadata, RenderError};
use wasm_bindgen::prelude::*;

mod design;
use design::DeckDoc;
pub use design::{BackgroundChoice, BuiltinTheme, Fit, THEMES};

mod animate;
mod build;
mod collab;
pub use collab::MAX_PARTICIPANT;
mod clipboard;
mod edit;
mod file;
pub use file::{CustomKind, PROPERTY_KEYS};
mod format;
#[cfg(test)]
mod home_tests;
mod picture;
pub use picture::{MAX_CROP, MAX_PAD, PICTURE_STYLES};
mod review;
mod rich;
mod shapes;
pub use animate::AnimationItem;
pub use build::{Ends, Insert, MAX_PICTURE_BYTES};
pub use clipboard::{Clip, MAX_CLIP_SHAPES, PASTE_OFFSET};
pub use edit::{Move, Toggle};
pub use format::{MAX_FONT_POINTS, MIN_FONT_POINTS, RunSetting, SIZE_LADDER, step_size};
pub use rich::{DEFAULT_TABLE_STYLE, TableEdit};
pub use shapes::{MAX_LINE_WIDTH, parse_color, preset_outline, theme_style};
mod show;
pub use show::{MAX_NOTES_CHARS, MAX_TRANSITION_MS, ShowSlide, ShowTransition};

use casual_slide_transaction::{History, Operation, TxnError};

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
    /// An edit the model refused; its own code (`CS-TXN-*`).
    Edit(TxnError),
    /// A shape this build cannot remove from the file yet: animated, without an
    /// id of its own, or sharing one (`VS-06`).
    CannotDelete,
    /// The shape has no text body, or no caret could be placed in it (`VS-05`).
    NoText,
    /// A bridge number named no caret move or attribute.
    BadArgument,
    /// A new slide or shape could not be built: its layout could not be read, or the
    /// reader refused the part built (`VS-08`); the reader's code, or `CS-WSM-0007`.
    Build(&'static str),
    /// Insert > Picture was given bytes that are not a PNG or JPEG picture, or more of
    /// them than [`MAX_PICTURE_BYTES`] (`VS-08`).
    NotAPicture,
    /// Co-editing refused a message or an arrival; the collaboration code
    /// (`CS-COL-*`, `COL-04`).
    Collab(&'static str),
    /// A shape Copy cannot carry yet, or Paste cannot put on this slide: a group, a
    /// table, a chart or another frame; a picture that is not PNG or JPEG; or, onto
    /// another slide, a shape naming a link or another relationship of its own
    /// slide (`UX-003`).
    CannotCopy,
}

impl BridgeError {
    /// The stable diagnostic code (`docs/20`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoDeck => "CS-WSM-0001",
            Self::InvalidWidth(_) => "CS-WSM-0002",
            Self::Decode => "CS-WSM-0003",
            Self::Render(e) => e.code(),
            Self::Save(code) => code,
            Self::Edit(e) => e.code(),
            Self::CannotDelete => "CS-WSM-0004",
            Self::NoText => "CS-WSM-0005",
            Self::BadArgument => "CS-WSM-0006",
            Self::Build(code) => code,
            Self::NotAPicture => "CS-WSM-0008",
            Self::Collab(code) => code,
            Self::CannotCopy => "CS-WSM-0010",
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
            Self::Edit(e) => e.fmt(f),
            Self::CannotDelete => write!(
                f,
                "[{}] this shape is animated or has no id of its own, so it cannot be deleted yet",
                self.code()
            ),
            Self::NoText => write!(f, "[{}] that shape has no text to edit", self.code()),
            Self::BadArgument => write!(f, "[{}] an argument named nothing", self.code()),
            Self::Build(code) => write!(f, "[{code}] the new slide or shape could not be built"),
            Self::Collab(code) => write!(f, "[{code}] the shared session could not take that"),
            Self::NotAPicture => write!(
                f,
                "[{}] only PNG and JPEG pictures up to 64 MB can be inserted",
                self.code()
            ),
            Self::CannotCopy => write!(
                f,
                "[{}] this shape cannot be copied or pasted here yet",
                self.code()
            ),
        }
    }
}

impl std::error::Error for BridgeError {}

impl From<TxnError> for BridgeError {
    fn from(e: TxnError) -> Self {
        Self::Edit(e)
    }
}

impl From<RenderError> for BridgeError {
    fn from(e: RenderError) -> Self {
        Self::Render(e)
    }
}

/// One shape a slide shows, where the editor can select it: its node, its
/// effective box in EMU, and what its locks allow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShapeBox {
    /// The node within the slide's part.
    pub node: u32,
    /// `off`, EMU, own or inherited.
    pub x: i64,
    /// `off`, EMU.
    pub y: i64,
    /// `ext`, EMU.
    pub cx: i64,
    /// `ext`, EMU.
    pub cy: i64,
    /// `@rot`, 60000ths of a degree.
    pub rotation: i32,
    /// [`ShapeBox::FLIP_H`] and the other bits.
    pub flags: u32,
}

impl ShapeBox {
    /// `@flipH`.
    pub const FLIP_H: u32 = 1;
    /// `@flipV`.
    pub const FLIP_V: u32 = 2;
    /// `noMove`.
    pub const NO_MOVE: u32 = 4;
    /// `noResize`.
    pub const NO_RESIZE: u32 = 8;
    /// `noRot`.
    pub const NO_ROTATE: u32 = 16;
    /// `noChangeAspect`: a corner handle keeps the aspect ratio.
    pub const KEEP_ASPECT: u32 = 32;
    /// A group.
    pub const GROUP: u32 = 64;
    /// A placeholder (`p:ph`): the only shape an empty text body prompts on, as
    /// PowerPoint shows "Click to add text" on placeholders only (`SHAPE-02`).
    pub const PLACEHOLDER: u32 = 128;
    /// A graphic frame holding a table (`VS-11`).
    pub const TABLE: u32 = 256;
    /// A graphic frame holding a chart (`VS-11`).
    pub const CHART: u32 = 512;
    /// A picture: Picture Format shows while one is selected (`UX-003`).
    pub const PICTURE: u32 = 1024;
}

/// A Design edit for the save plan.
enum DesignEdit {
    /// A file-level operation: a theme's new bytes.
    Op(Op),
    /// A part's new bytes: the presentation part with its new `sldSz`.
    Replace(String, Vec<u8>),
}

/// One opened deck, what each slide's last drawing left out, and its edits.
#[derive(Debug)]
pub struct Viewer {
    deck: Deck,
    reports: BTreeMap<usize, Vec<String>>,
    /// The parts as opened: what a save patches from.
    original: SlideParts,
    /// Each master's theme as opened, and the slide size (`VS-12`).
    original_themes: BTreeMap<u32, casual_slide_model::theme::Theme>,
    original_size: (i64, i64),
    /// The document properties as opened: what a save patches `docProps` from.
    original_properties: casual_slide_model::properties::DocumentProperties,
    history: History,
    /// Chart parts of the file whose data was edited (`VS-11`), patched at save.
    edited_charts: std::collections::BTreeSet<String>,
    /// The undo mark of the typing burst in progress, if any.
    typing: Option<u64>,
    /// The last undo or redo changed more than one slide: a theme, the slide size, or
    /// several backgrounds (`VS-12`).
    wide: bool,
    /// Where each part added in this session came from (`VS-08`).
    origins: BTreeMap<u32, build::Origin>,
    /// Pictures inserted in this session, by the relationship id the model gives them.
    new_media: BTreeMap<String, casual_slide_export::writer::MediaBytes>,
    /// Pictures inserted so far, which names the next one.
    minted: u32,
    /// A shared session, while one is on (`COL-04`).
    shared: Option<collab::Shared>,
    /// This deck's number in the session, so a paste knows whether it came from it.
    serial: u64,
    /// The deck's modern comments and authors, edited (`UX-011`), and as opened.
    review: casual_slide_transaction::Review,
    original_review: casual_slide_transaction::Review,
    /// Where each slide's comment part and the author list were found, as opened.
    review_parts: review::ReviewParts,
}

impl Viewer {
    /// Open a deck's bytes. A deck that cannot be read at all is refused with its
    /// reader's code; a damaged one opens with a best reading where the reader recovers one (`docs/29`).
    pub fn open(bytes: Vec<u8>) -> Result<Self, BridgeError> {
        let deck = Deck::open(bytes)?;
        let (original_review, review_parts) = review::read(&deck);
        let original = deck.parts().clone();
        let original_themes = deck.themes().clone();
        let original_size = deck.size();
        let original_properties = deck.properties().clone();
        Ok(Self {
            deck,
            reports: BTreeMap::new(),
            original,
            original_themes,
            original_size,
            original_properties,
            history: History::with_depth(Some(UNDO_DEPTH)),
            edited_charts: std::collections::BTreeSet::new(),
            typing: None,
            wide: false,
            origins: BTreeMap::new(),
            new_media: BTreeMap::new(),
            minted: 0,
            shared: None,
            serial: clipboard::next_serial(),
            review: original_review.clone(),
            original_review,
            review_parts,
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
    /// edited, the bytes that were opened. Edits reach the file as patches of the
    /// named nodes only (ADR-013): new, duplicated, deleted and moved slides go through
    /// the writer's slide operations, a removed shape is [`casual_slide_export::writer::Op::RemoveShape`], a new one
    /// is inserted empty, and then every slide part is patched to its model, run by run
    /// for text (`VS-05`, `VS-08`); every other byte stays.
    pub fn save(&self) -> Result<Vec<u8>, BridgeError> {
        self.save_built()
    }

    /// What Design changed (`VS-12`), as the writer takes it: each master's theme part
    /// patched node by node from the part as opened, and `sldSz`.
    fn design_edits(
        &self,
        package: &mut PresentationPackage,
    ) -> Result<Vec<DesignEdit>, BridgeError> {
        let save = |code: &'static str| BridgeError::Save(code);
        let limits = *package.limits();
        let mut out = Vec::new();
        for (master, now) in self.deck.themes() {
            let Some(before) = self.original_themes.get(master) else {
                continue;
            };
            if before == now {
                continue;
            }
            let (Some(b), Some(a)) = (before.elements.as_ref(), now.elements.as_ref()) else {
                continue;
            };
            let Some(entry) = self.original.structure.entry(*master) else {
                continue;
            };
            let Some(name) = entry.theme.as_deref() else {
                continue;
            };
            let bytes = package.read_xml_part(name).map_err(|e| save(e.code()))?;
            let mut tree = XmlTree::parse(&bytes, name, &limits).map_err(|e| save(e.code()))?;
            if casual_slide_export::design::patch_theme_schemes(&mut tree, b, a)
                .map_err(|e| save(e.code()))?
            {
                out.push(DesignEdit::Op(Op::EditTheme {
                    master: entry.part.clone(),
                    bytes: tree.to_bytes(),
                }));
            }
        }
        let (cx, cy) = self.deck.size();
        if (cx, cy) != self.original_size {
            let name = package.main_part().to_owned();
            let bytes = package.read_xml_part(&name).map_err(|e| save(e.code()))?;
            let mut tree = XmlTree::parse(&bytes, &name, &limits).map_err(|e| save(e.code()))?;
            if casual_slide_export::design::set_slide_size(&mut tree, cx, cy)
                .map_err(|e| save(e.code()))?
            {
                out.push(DesignEdit::Replace(name, tree.to_bytes()));
            }
        }
        Ok(out)
    }

    fn slide_ordinal(&self, index: usize) -> Result<u32, BridgeError> {
        self.deck
            .parts()
            .structure
            .slide(index)
            .ok_or(BridgeError::Render(RenderError::NoSuchSlide(index)))
    }

    /// The shapes on slide `index` the editor can select, back to front: the
    /// slide's own top-level shapes that are placed and not hidden or
    /// `noSelect`. A group is one shape, as a first click in PowerPoint selects it.
    pub fn shapes(&self, index: usize) -> Result<Vec<ShapeBox>, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let parts = self.deck.parts();
        let chain = casual_slide_style::Chain::of(parts, ordinal);
        let Some(root) = parts.get(ordinal).and_then(SlidePart::shape_tree) else {
            return Ok(Vec::new());
        };
        let mut out = Vec::new();
        for shape in &root.children {
            let hidden = shape.non_visual.as_ref().and_then(|n| n.hidden) == Some(true);
            let locks = shape.locks.unwrap_or_default();
            if hidden || locks.no_select == Some(true) {
                continue;
            }
            let Some(t) = chain.transform(shape) else {
                continue;
            };
            let t = t.value;
            let bit = |on: Option<bool>, b: u32| if on == Some(true) { b } else { 0 };
            let flags = bit(Some(t.flip_h), ShapeBox::FLIP_H)
                | bit(Some(t.flip_v), ShapeBox::FLIP_V)
                | bit(locks.no_move, ShapeBox::NO_MOVE)
                | bit(locks.no_resize, ShapeBox::NO_RESIZE)
                | bit(locks.no_rot, ShapeBox::NO_ROTATE)
                | bit(locks.no_change_aspect, ShapeBox::KEEP_ASPECT)
                | bit(Some(shape.kind == NodeKind::Group), ShapeBox::GROUP)
                | bit(Some(shape.placeholder().is_some()), ShapeBox::PLACEHOLDER)
                | bit(Some(shape.table.is_some()), ShapeBox::TABLE)
                | bit(Some(shape.chart.is_some()), ShapeBox::CHART)
                | bit(
                    Some(shape.kind == NodeKind::Picture && shape.properties.picture.is_some()),
                    ShapeBox::PICTURE,
                );
            out.push(ShapeBox {
                node: shape.id.node,
                x: t.offset.x.0,
                y: t.offset.y.0,
                cx: t.extent.cx,
                cy: t.extent.cy,
                rotation: t.rotation,
                flags,
            });
        }
        Ok(out)
    }

    /// Place shape `node` of slide `index` at this box: one property op, one undo
    /// step, as a whole drag commits once (`docs/24`). A transform the shape only
    /// inherited becomes its own; a rotation or flip it inherited is kept.
    pub fn set_transform(
        &mut self,
        index: usize,
        node: u32,
        b: (i64, i64, i64, i64),
        rotation: i32,
    ) -> Result<(), BridgeError> {
        let op = self.transform_op(index, node, b, rotation)?;
        self.history.apply(self.deck.parts_mut(), op)?;
        self.reports.remove(&index);
        Ok(())
    }

    /// The operation that places shape `node` of slide `index` at this box.
    pub(crate) fn transform_op(
        &self,
        index: usize,
        node: u32,
        (x, y, cx, cy): (i64, i64, i64, i64),
        rotation: i32,
    ) -> Result<Operation, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let id = ObjectId {
            part: ordinal,
            node,
        };
        let parts = self.deck.parts();
        let shape = parts
            .get(ordinal)
            .and_then(|p| p.node(id))
            .ok_or(TxnError::NoSuchObject(id))?;
        let inherited = casual_slide_style::Chain::of(parts, ordinal)
            .transform(shape)
            .map(|t| t.value);
        let own = shape.transform;
        let mut t = own.unwrap_or_default();
        let coordinate =
            |v: i64| Coordinate(v.clamp(-POSITIVE_COORDINATE_MAX, POSITIVE_COORDINATE_MAX));
        let size = |v: i64| v.clamp(0, POSITIVE_COORDINATE_MAX);
        t.offset = Some(Point {
            x: coordinate(x),
            y: coordinate(y),
        });
        t.extent = Some(PositiveSize {
            cx: size(cx),
            cy: size(cy),
        });
        let rotation = rotation.rem_euclid(21_600_000);
        // Absent stays absent: an unrotated shape that stays unrotated gets no `@rot`.
        if rotation != 0 || t.rotation.is_some() {
            t.rotation = Some(rotation);
        }
        if own.is_none()
            && let Some(e) = inherited
        {
            if e.flip_h {
                t.flip_h = Some(true);
            }
            if e.flip_v {
                t.flip_v = Some(true);
            }
        }
        Ok(Operation::SetTransform {
            node: id,
            transform: Some(t),
        })
    }

    /// Delete shape `node` of slide `index`, as one undo step. Refused, before
    /// anything changes, for a shape the writer cannot yet remove from the file:
    /// one an animation names, or one without an id of its own.
    pub fn delete(&mut self, index: usize, node: u32) -> Result<(), BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        self.check_removable(ordinal, node)?;
        self.history.apply(
            self.deck.parts_mut(),
            Operation::DeleteNode {
                node: ObjectId {
                    part: ordinal,
                    node,
                },
            },
        )?;
        self.reports.remove(&index);
        Ok(())
    }

    /// Refuse a shape the writer cannot yet remove from the file, as [`Viewer::delete`] says.
    pub(crate) fn check_removable(&self, ordinal: u32, node: u32) -> Result<(), BridgeError> {
        let id = ObjectId {
            part: ordinal,
            node,
        };
        let parts = self.deck.parts();
        let part = parts.get(ordinal).ok_or(TxnError::NoSuchObject(id))?;
        let shape = part.node(id).ok_or(TxnError::NoSuchObject(id))?;
        let animated = part.animated_spids();
        let spid_of = |n: &ShapeNode| n.non_visual.as_ref().and_then(|v| v.spid);
        let removable = spid_of(shape).is_some_and(|spid| {
            let shared = part.shape_tree().map_or(0, |root| {
                root.descendants()
                    .filter(|n| spid_of(n) == Some(spid))
                    .count()
            }) > 1;
            // An animated child would go with its group.
            let names_animated = shape
                .descendants()
                .any(|n| spid_of(n).is_some_and(|s| animated.contains(&s)));
            !shared && !names_animated
        });
        if !removable {
            return Err(BridgeError::CannotDelete);
        }
        Ok(())
    }

    /// The slide an operation edited, if it is a slide.
    fn slide_of(&mut self, op: Option<Operation>) -> Option<usize> {
        self.wide = op
            .as_ref()
            .is_some_and(|op| design::redraws_all(op, &self.deck.parts().structure));
        let part = op?.part()?;
        self.deck
            .parts()
            .structure
            .slides
            .iter()
            .position(|&o| o == part)
    }

    /// Undo the latest edit; the slide it changed, if any.
    pub fn undo(&mut self) -> Result<Option<usize>, BridgeError> {
        self.typing = None;
        let op = self
            .history
            .undo(&mut DeckDoc(&mut self.deck, &mut self.review))?;
        self.reports.clear();
        Ok(self.slide_of(op))
    }

    /// Redo the latest undone edit; the slide it changed, if any.
    pub fn redo(&mut self) -> Result<Option<usize>, BridgeError> {
        self.typing = None;
        let op = self
            .history
            .redo(&mut DeckDoc(&mut self.deck, &mut self.review))?;
        self.reports.clear();
        Ok(self.slide_of(op))
    }

    /// Whether the last undo or redo changed more than one slide, so a host redraws them
    /// all (`VS-12`).
    pub fn undid_wide(&self) -> bool {
        self.wide
    }

    /// Whether there is an edit to undo, and one to redo.
    pub fn can_undo_redo(&self) -> (bool, bool) {
        (self.history.can_undo(), self.history.can_redo())
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

/// What an undo or redo changed: the slide, -1 for nothing shown, -2 for every slide.
fn changed_code(v: &Viewer, slide: Option<usize>) -> i32 {
    if v.undid_wide() {
        return -2;
    }
    slide.map_or(-1, |i| i32::try_from(i).unwrap_or(-1))
}

/// Start a shared session as participant `client` at `revision` (`COL-04`): the deck
/// open must be the room's, its seed with its log replayed.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-COL-0004` for a participant number out of range.
#[wasm_bindgen]
pub fn collab_begin(client: u64, revision: u64) -> Result<(), JsError> {
    with_viewer(|v| v.collab_begin(client, revision))
}

/// End the shared session.
#[wasm_bindgen]
pub fn collab_end() {
    let _ = with_viewer(|v| {
        v.collab_end();
        Ok(())
    });
}

/// The collaboration protocol's version.
#[wasm_bindgen]
#[must_use]
pub fn protocol_version() -> u32 {
    casual_slide_transaction::collab::wire::PROTOCOL_VERSION
}

/// The next submission to send, as its JSON message; empty when there is none.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-COL-0003` for an edit with no wire form.
#[wasm_bindgen]
pub fn collab_flush() -> Result<String, JsError> {
    with_viewer(|v| v.collab_flush().map(Option::unwrap_or_default))
}

/// Own chunks through `through` are committed; the room is at `revision`.
#[wasm_bindgen]
pub fn collab_acknowledge(through: u64, revision: u64) {
    let _ = with_viewer(|v| {
        v.collab_acknowledge(through, revision);
        Ok(())
    });
}

/// Whether any own edit is not yet acknowledged.
#[wasm_bindgen]
#[must_use]
pub fn collab_unacknowledged() -> bool {
    with_viewer(|v| Ok(v.collab_unacknowledged())).unwrap_or(false)
}

/// Apply another participant's operations, a JSON array, at `revision`. Returns the
/// slide indexes they touched, `-2` for one not named.
///
/// # Errors
///
/// `CS-COL-0002` when this copy no longer agrees and must join again; `CS-COL-0004`
/// for a malformed message.
#[wasm_bindgen]
pub fn collab_receive(ops: String, revision: u64) -> Result<Vec<i32>, JsError> {
    with_viewer(|v| v.collab_receive(&ops, revision))
}

/// Replay a room's log, a JSON array, on the deck just opened from its seed.
///
/// # Errors
///
/// `CS-COL-0002` when an operation does not apply.
#[wasm_bindgen]
pub fn collab_replay(ops: String) -> Result<(), JsError> {
    with_viewer(|v| v.collab_replay(&ops))
}

/// Where a caret at `offset` in a shape moves to after the last arrival.
#[wasm_bindgen]
#[must_use]
pub fn collab_map_offset(part: u32, node: u32, offset: u32) -> u32 {
    with_viewer(|v| Ok(v.collab_map_offset(part, node, offset as usize)))
        .ok()
        .and_then(|o| u32::try_from(o).ok())
        .unwrap_or(offset)
}

/// This participant's presence as the JSON message to send: the slide, the selected
/// shapes, and the caret (`node` 0 for none) with the selection's other end.
#[wasm_bindgen]
#[must_use]
pub fn collab_presence(
    slide: u32,
    shapes: Vec<u32>,
    node: u32,
    offset: u32,
    anchor: u32,
) -> String {
    Viewer::collab_presence(casual_slide_transaction::collab::Presence {
        slide,
        shapes,
        caret: (node != 0).then_some(casual_slide_transaction::collab::Caret {
            node,
            offset,
            anchor,
        }),
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

/// The shapes on slide `index` the editor can select, back to front, seven numbers
/// each: node, x, y, cx, cy (EMU), rotation (60000ths of a degree), and flags
/// (`ShapeBox`'s bits). No document text crosses.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn slide_shapes(index: u32) -> Result<Vec<f64>, JsError> {
    with_viewer(|v| {
        Ok(v.shapes(index as usize)?
            .into_iter()
            .flat_map(|b| {
                [
                    f64::from(b.node),
                    b.x as f64,
                    b.y as f64,
                    b.cx as f64,
                    b.cy as f64,
                    f64::from(b.rotation),
                    f64::from(b.flags),
                ]
            })
            .collect())
    })
}

/// Place a shape at a box, in EMU, rotated in 60000ths of a degree: one edit, one
/// undo step. A drag calls this once, at its end.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-RND-0003`; `CS-TXN-0001` for no such shape.
#[wasm_bindgen]
pub fn set_shape_transform(
    index: u32,
    node: u32,
    x: f64,
    y: f64,
    cx: f64,
    cy: f64,
    rotation: f64,
) -> Result<(), JsError> {
    // `as` saturates, and a NaN becomes 0: values from the page cannot trap.
    with_viewer(|v| {
        v.set_transform(
            index as usize,
            node,
            (x as i64, y as i64, cx as i64, cy as i64),
            rotation as i32,
        )
    })
}

/// Delete a shape: one edit, one undo step.
///
/// # Errors
///
/// `CS-WSM-0004` for a shape that cannot be removed from the file yet.
#[wasm_bindgen]
pub fn delete_shape(index: u32, node: u32) -> Result<(), JsError> {
    with_viewer(|v| v.delete(index as usize, node))
}

/// Undo the latest edit. Returns the slide it changed, or -1.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck.
#[wasm_bindgen]
pub fn undo() -> Result<i32, JsError> {
    with_viewer(|v| {
        let slide = v.undo()?;
        Ok(changed_code(v, slide))
    })
}

/// Redo the latest undone edit. Returns the slide it changed, or -1.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck.
#[wasm_bindgen]
pub fn redo() -> Result<i32, JsError> {
    with_viewer(|v| {
        let slide = v.redo()?;
        Ok(changed_code(v, slide))
    })
}

/// `[can undo, can redo]`, each 0 or 1.
#[wasm_bindgen]
#[must_use]
pub fn undo_state() -> Vec<u8> {
    SESSION.with(|s| match s.borrow().as_ref() {
        Some(v) => {
            let (u, r) = v.can_undo_redo();
            vec![u8::from(u), u8::from(r)]
        }
        None => vec![0, 0],
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

/* ── Presenting (`VS-07`) ─────────────────────────────────────────────── */

/// How each slide takes part in a slide show, two numbers a slide in order:
/// flags (1: hidden, 2: has a transition, 4: the transition is played as it is,
/// not as a stand-in fade), then the transition's duration in milliseconds.
///
/// # Errors
///
/// `CS-WSM-0001` when no deck is open.
#[wasm_bindgen]
pub fn show_plan() -> Result<Vec<u32>, JsError> {
    with_viewer(|v| {
        let mut out = Vec::with_capacity(v.slide_count().saturating_mul(2));
        for i in 0..v.slide_count() {
            let s = v.show_slide(i)?;
            let mut flags = u32::from(s.hidden);
            let mut ms = 0;
            if let Some(t) = &s.transition {
                flags |= 2;
                if t.played_as_is {
                    flags |= 4;
                }
                ms = t.duration_ms;
            }
            out.extend_from_slice(&[flags, ms]);
        }
        Ok(out)
    })
}

/// The transition effect into slide `index`, by its element name (`push`), or
/// empty when it has none. For the show's report of what it played as a fade.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn slide_transition(index: u32) -> Result<String, JsError> {
    with_viewer(|v| {
        Ok(v.show_slide(index as usize)?
            .transition
            .map_or(String::new(), |t| t.effect.to_string()))
    })
}

/// The speaker notes of slide `index` as plain text, for Presenter View; empty
/// when it has none.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn slide_notes(index: u32) -> Result<String, JsError> {
    with_viewer(|v| v.notes_text(index as usize))
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
            Operation::InsertText {
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
            Operation::DeleteRange {
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
    use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

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

    /// Every part of `saved` but `slide1.xml` is the input's, byte for byte; returns
    /// the slide's XML before and after.
    fn only_slide_one_changed(input: &[u8], saved: &[u8]) -> (String, String) {
        let mut a = PresentationPackage::open(input.to_vec(), OoxmlLimits::default()).unwrap();
        let mut b = PresentationPackage::open(saved.to_vec(), OoxmlLimits::default()).unwrap();
        assert_eq!(a.part_names(), b.part_names());
        let mut slide = None;
        for name in a.part_names() {
            let (x, y) = (a.read_part(&name).unwrap(), b.read_part(&name).unwrap());
            if name.ends_with("slides/slide1.xml") {
                slide = Some((String::from_utf8(x).unwrap(), String::from_utf8(y).unwrap()));
            } else {
                assert!(x == y, "{name} changed");
            }
        }
        slide.unwrap()
    }

    // `preset-shapes.pptx` slide 1: the title (spid 2), a line (spid 3) at
    // 274320,685800 sized 1375410 by 1257300, a label (spid 4).
    const LINE_OFF: &str = r#"<a:off x="274320" y="685800"/>"#;
    const LINE_EXT: &str = r#"<a:ext cx="1375410" cy="1257300"/>"#;

    fn line(v: &Viewer) -> ShapeBox {
        let shapes = v.shapes(0).unwrap();
        let line = shapes[1];
        assert_eq!(
            (line.x, line.y, line.cx, line.cy),
            (274_320, 685_800, 1_375_410, 1_257_300)
        );
        line
    }

    #[test]
    fn a_move_changes_only_the_shapes_offset_in_the_file() {
        let input = fixture("preset-shapes.pptx");
        let mut v = Viewer::open(input.clone()).unwrap();
        let l = line(&v);
        v.set_transform(0, l.node, (l.x + 100_000, l.y + 50_000, l.cx, l.cy), 0)
            .unwrap();
        let saved = v.save().unwrap();
        let (before, after) = only_slide_one_changed(&input, &saved);
        assert_eq!(
            after,
            before.replacen(LINE_OFF, r#"<a:off x="374320" y="735800"/>"#, 1)
        );
        // Read back, the shape is where it was put.
        let again = Viewer::open(saved).unwrap();
        let moved = again.shapes(0).unwrap()[1];
        assert_eq!((moved.x, moved.y), (374_320, 735_800));
    }

    #[test]
    fn a_resize_and_rotate_change_only_the_shapes_xfrm() {
        let input = fixture("preset-shapes.pptx");
        let mut v = Viewer::open(input.clone()).unwrap();
        let l = line(&v);
        v.set_transform(0, l.node, (l.x, l.y, 2_000_000, 1_000_000), 5_400_000)
            .unwrap();
        let saved = v.save().unwrap();
        let (before, after) = only_slide_one_changed(&input, &saved);
        let expected = before.replacen(
            &format!("<a:xfrm>{LINE_OFF}{LINE_EXT}"),
            &format!(r#"<a:xfrm rot="5400000">{LINE_OFF}<a:ext cx="2000000" cy="1000000"/>"#),
            1,
        );
        assert_ne!(expected, before);
        assert_eq!(after, expected);
    }

    #[test]
    fn a_delete_removes_only_that_shape_from_the_file() {
        let input = fixture("preset-shapes.pptx");
        let mut v = Viewer::open(input.clone()).unwrap();
        let count = v.shapes(0).unwrap().len();
        let l = line(&v);
        v.delete(0, l.node).unwrap();
        assert_eq!(v.shapes(0).unwrap().len(), count - 1);
        let saved = v.save().unwrap();
        let (before, after) = only_slide_one_changed(&input, &saved);
        let id = before.find(r#"<p:cNvPr id="3""#).unwrap();
        let start = before[..id].rfind("<p:sp>").unwrap();
        let end = id + before[id..].find("</p:sp>").unwrap() + "</p:sp>".len();
        let expected = format!("{}{}", &before[..start], &before[end..]);
        assert_eq!(after, expected);
        assert_eq!(
            Viewer::open(saved).unwrap().shapes(0).unwrap().len(),
            count - 1
        );
    }

    #[test]
    fn undo_and_redo_walk_a_move_and_a_delete_and_save_follows() {
        let input = fixture("preset-shapes.pptx");
        let mut v = Viewer::open(input.clone()).unwrap();
        let l = line(&v);
        v.set_transform(0, l.node, (0, 0, l.cx, l.cy), 0).unwrap();
        v.delete(0, l.node).unwrap();
        assert_eq!(v.can_undo_redo(), (true, false));
        assert_eq!(v.undo().unwrap(), Some(0));
        assert_eq!(v.shapes(0).unwrap()[1].x, 0);
        assert_eq!(v.undo().unwrap(), Some(0));
        assert_eq!(v.can_undo_redo(), (false, true));
        line(&v);
        // Undone to the start, the save is the input, byte for byte.
        assert!(v.save().unwrap() == input);
        assert_eq!(v.redo().unwrap(), Some(0));
        assert_eq!(v.shapes(0).unwrap()[1].x, 0);
        assert_eq!(v.undo().unwrap(), Some(0));
        assert_eq!(v.undo().unwrap(), None);
    }

    #[test]
    fn an_animated_shape_is_refused_before_anything_changes() {
        let mut v = Viewer::open(fixture("rr-animated.pptx")).unwrap();
        let ordinal = v.slide_ordinal(0).unwrap();
        let animated = v.deck.parts().get(ordinal).unwrap().animated_spids();
        let node = v
            .deck
            .parts()
            .get(ordinal)
            .unwrap()
            .shape_tree()
            .unwrap()
            .children
            .iter()
            .find(|n| {
                n.non_visual
                    .as_ref()
                    .and_then(|x| x.spid)
                    .is_some_and(|s| animated.contains(&s))
            })
            .unwrap()
            .id
            .node;
        assert_eq!(v.delete(0, node).unwrap_err().code(), "CS-WSM-0004");
        assert_eq!(v.can_undo_redo(), (false, false));
    }
}

/* ── Building a deck (`VS-08`) ─────────────────────────────────────────── */

/// File > New: open the built-in blank presentation, one Title Slide, replacing any
/// deck open before.
///
/// # Errors
///
/// `CS-WSM-0007` if the blank deck could not be built, which a correct build never does.
#[wasm_bindgen]
pub fn open_blank() -> Result<(), JsError> {
    let viewer = Viewer::blank().map_err(js)?;
    SESSION.with(|s| *s.borrow_mut() = Some(viewer));
    Ok(())
}

/// File > New from a template (`PAR-18`): the blank presentation in built-in theme
/// `theme` (an index of [`design_themes`]), replacing any deck open before.
///
/// # Errors
///
/// `CS-WSM-0006` for a theme that is not there; `CS-WSM-0007` if the blank deck could
/// not be built.
#[wasm_bindgen]
pub fn open_template(theme: u32) -> Result<(), JsError> {
    let viewer = Viewer::template(theme as usize).map_err(js)?;
    SESSION.with(|s| *s.borrow_mut() = Some(viewer));
    Ok(())
}

/// The deck's layouts for the New Slide gallery: `[ordinal, name, ordinal, name, …]`.
/// A name is document text: the page shows it through `textContent` only.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck.
#[wasm_bindgen]
pub fn slide_layouts() -> Result<Vec<String>, JsError> {
    with_viewer(|v| {
        Ok(v.layouts()
            .into_iter()
            .flat_map(|(o, n)| [o.to_string(), n])
            .collect())
    })
}

/// The layout ordinal slide `index` is on, or -1.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck.
#[wasm_bindgen]
pub fn slide_layout(index: u32) -> Result<i64, JsError> {
    with_viewer(|v| Ok(v.layout_of(index as usize).map_or(-1, i64::from)))
}

/// New Slide on layout `layout` at place `at`; returns the place.
///
/// # Errors
///
/// `CS-WSM-0006` for a layout that is not one; `CS-TXN-0006` for a bad place.
#[wasm_bindgen]
pub fn new_slide(at: u32, layout: u32) -> Result<u32, JsError> {
    with_viewer(|v| v.new_slide(at as usize, layout).map(to_u32))
}

/// Duplicate slide `index`; returns the copy's place.
///
/// # Errors
///
/// `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn duplicate_slide(index: u32) -> Result<u32, JsError> {
    with_viewer(|v| v.duplicate_slide(index as usize).map(to_u32))
}

/// Delete slide `index`.
///
/// # Errors
///
/// `CS-TXN-0006` for a slide that does not exist.
#[wasm_bindgen]
pub fn delete_slide(index: u32) -> Result<(), JsError> {
    with_viewer(|v| v.delete_slide(index as usize))
}

/// Move slide `from` to place `to`.
///
/// # Errors
///
/// `CS-TXN-0006` for a place past the end.
#[wasm_bindgen]
pub fn move_slide(from: u32, to: u32) -> Result<(), JsError> {
    with_viewer(|v| v.move_slide(from as usize, to as usize))
}

/// Which slides are hidden, one byte each, 1 for hidden.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck.
#[wasm_bindgen]
pub fn hidden_slides() -> Result<Vec<u8>, JsError> {
    with_viewer(|v| {
        (0..v.slide_count())
            .map(|i| v.is_hidden(i).map(u8::from))
            .collect()
    })
}

/// Hide slide `index` in a slide show, or show it again.
///
/// # Errors
///
/// `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn set_slide_hidden(index: u32, hidden: bool) -> Result<(), JsError> {
    with_viewer(|v| v.set_hidden(index as usize, hidden))
}

/// Insert a text box (0), rectangle (1), oval (2), line (3) or arrow (4) on slide
/// `index` at a box in EMU; returns its node.
///
/// # Errors
///
/// `CS-WSM-0006` for another kind.
#[wasm_bindgen]
pub fn insert_shape(
    index: u32,
    kind: u32,
    x: f64,
    y: f64,
    cx: f64,
    cy: f64,
) -> Result<u32, JsError> {
    with_viewer(|v| {
        let kind = Insert::from_u32(kind).ok_or(BridgeError::BadArgument)?;
        v.insert_shape(
            index as usize,
            kind,
            [x as i64, y as i64, cx as i64, cy as i64],
        )
    })
}

/// Insert > Picture: PNG or JPEG bytes on slide `index` at a box in EMU; returns its
/// node.
///
/// # Errors
///
/// `CS-WSM-0008` for bytes that are not a PNG or JPEG picture.
#[wasm_bindgen]
pub fn insert_picture(
    index: u32,
    bytes: Vec<u8>,
    x: f64,
    y: f64,
    cx: f64,
    cy: f64,
) -> Result<u32, JsError> {
    with_viewer(|v| {
        v.insert_picture(
            index as usize,
            bytes,
            [x as i64, y as i64, cx as i64, cy as i64],
        )
    })
}

/// The names of every preset shape the geometry engine has, sorted, for the Shapes
/// gallery (`SHAPE-01`).
#[wasm_bindgen]
#[must_use]
pub fn preset_names() -> Vec<String> {
    casual_slide_geometry::preset_shapes()
        .iter()
        .map(|d| d.name.to_string())
        .collect()
}

/// The Shapes gallery's icon for preset `name` at `w` by `h` units, as SVG path data
/// ([`preset_outline`]); empty for a name the table lacks (`SHAPE-01`).
#[wasm_bindgen]
pub fn preset_icon(name: &str, w: f64, h: f64) -> String {
    preset_outline(name, w, h).unwrap_or_default()
}

/// Insert gallery preset `preset` on slide `index` at a box in EMU; `flags` bit 0
/// flips it horizontally, bit 1 vertically, bits 2 and 3 give a line one arrowhead or
/// two. Named `label` and a number. Returns its node.
///
/// # Errors
///
/// `CS-WSM-0006` for a name the preset table lacks.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)] // a box and its flags cross the boundary as numbers
pub fn insert_preset(
    index: u32,
    preset: &str,
    label: &str,
    x: f64,
    y: f64,
    cx: f64,
    cy: f64,
    flags: u32,
) -> Result<u32, JsError> {
    with_viewer(|v| {
        let ends = match (flags >> 2) & 3 {
            0 => Ends::Plain,
            1 => Ends::Tail,
            _ => Ends::Both,
        };
        v.insert_preset(
            index as usize,
            preset,
            ends,
            label,
            [x as i64, y as i64, cx as i64, cy as i64],
            (flags & 1 != 0, flags & 2 != 0),
        )
    })
}

/// Shape `node`'s yellow adjust handles on slide `index`: x and y in slide EMU, each.
///
/// # Errors
///
/// `CS-WSM-0006` for a node the slide's top level lacks.
#[wasm_bindgen]
pub fn shape_handles(index: u32, node: u32) -> Result<Vec<f64>, JsError> {
    with_viewer(|v| v.handles(index as usize, node))
}

/// Drag handle `handle` of shape `node` to a point in slide EMU; `first` starts the
/// gesture, which `edit_break_typing` ends. Whether anything changed.
///
/// # Errors
///
/// `CS-WSM-0006` for a shape or handle that is not there.
#[wasm_bindgen]
pub fn drag_shape_handle(
    index: u32,
    node: u32,
    handle: u32,
    x: f64,
    y: f64,
    first: bool,
) -> Result<bool, JsError> {
    with_viewer(|v| v.drag_handle(index as usize, node, handle as usize, (x, y), first))
}

/// Shape Fill: `none` or a colour, as [`parse_color`] reads it.
///
/// # Errors
///
/// `CS-WSM-0006` for a colour it cannot read, or a node that takes no fill.
#[wasm_bindgen]
pub fn set_shape_fill(index: u32, node: u32, spec: &str) -> Result<bool, JsError> {
    with_viewer(|v| v.set_fill(index as usize, node, spec))
}

/// Shape Outline: `what` 0 colour, 1 weight in EMU, 2 dash, 3 arrowheads.
///
/// # Errors
///
/// `CS-WSM-0006` for a value it cannot read.
#[wasm_bindgen]
pub fn set_shape_outline(index: u32, node: u32, what: u32, value: &str) -> Result<bool, JsError> {
    with_viewer(|v| v.set_outline(index as usize, node, what, value))
}

/// Picture `node`'s crop: left, top, right, bottom, thousandths of a percent.
///
/// # Errors
///
/// `CS-WSM-0006` for a node that is not a picture.
#[wasm_bindgen]
pub fn picture_crop(index: u32, node: u32) -> Result<Vec<i32>, JsError> {
    with_viewer(|v| v.picture_crop(index as usize, node).map(|c| c.to_vec()))
}

/// Crop: picture `node` cropped to `l, t, r, b` and placed at `x, y, cx, cy`, as one
/// undo step (`UX-003`).
///
/// # Errors
///
/// `CS-WSM-0006` for a crop past the bounds, or a node that is not a picture.
#[wasm_bindgen]
#[allow(clippy::too_many_arguments)] // the crop's four sides and the frame's four values, as JS holds them
pub fn set_picture_crop(
    index: u32,
    node: u32,
    l: i32,
    t: i32,
    r: i32,
    b: i32,
    x: f64,
    y: f64,
    cx: f64,
    cy: f64,
) -> Result<(), JsError> {
    with_viewer(|v| {
        v.set_picture_crop(
            index as usize,
            node,
            [l, t, r, b],
            (x as i64, y as i64, cx as i64, cy as i64),
        )
    })
}

/// Reset Picture, or with `and_size` Reset Picture & Size.
///
/// # Errors
///
/// `CS-WSM-0006` for a node that is not a picture.
#[wasm_bindgen]
pub fn reset_picture(index: u32, node: u32, and_size: bool) -> Result<bool, JsError> {
    with_viewer(|v| v.reset_picture(index as usize, node, and_size))
}

/// Picture Styles: one of [`PICTURE_STYLES`], by name.
///
/// # Errors
///
/// `CS-WSM-0006` for a name the gallery lacks, or a node that is not a picture.
#[wasm_bindgen]
pub fn set_picture_style(index: u32, node: u32, name: &str) -> Result<bool, JsError> {
    with_viewer(|v| v.set_picture_style(index as usize, node, name))
}

/// Shape Styles: theme style `row`, `column` of the gallery.
///
/// # Errors
///
/// `CS-WSM-0006` for a style the gallery lacks.
#[wasm_bindgen]
pub fn set_shape_style(index: u32, node: u32, row: u32, column: u32) -> Result<bool, JsError> {
    with_viewer(|v| v.set_theme_style(index as usize, node, row, column))
}

/// Slide `index`'s ten theme colours, as the colour pickers show them.
///
/// # Errors
///
/// `CS-RND-*` for a slide that is not there.
#[wasm_bindgen]
pub fn slide_palette(index: u32) -> Result<Vec<String>, JsError> {
    with_viewer(|v| v.palette(index as usize))
}

/* ── Transitions and animations (`VS-10`) ─────────────────────────────── */

fn bad<T>(v: Option<T>) -> Result<T, BridgeError> {
    v.ok_or(BridgeError::BadArgument)
}

/// The transition into slide `index` for the Transitions tab: `[kind, ms]`, kind as
/// [`casual_slide_transaction::TransitionKind::from_code`] numbers it, 99 for an effect the tab does not offer.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn transition_choice(index: u32) -> Result<Vec<u32>, JsError> {
    with_viewer(|v| {
        let (kind, ms) = v.transition_choice(index as usize)?;
        Ok(vec![kind.map_or(99, |k| k.code()), ms])
    })
}

/// Give slide `index` a transition (0 none, 1 cut, 2 fade, 3 push, 4 wipe, 5 split)
/// lasting `ms` milliseconds.
///
/// # Errors
///
/// `CS-WSM-0006` for another kind.
#[wasm_bindgen]
pub fn set_slide_transition(index: u32, kind: u32, ms: u32) -> Result<bool, JsError> {
    with_viewer(|v| {
        let kind = bad(casual_slide_transaction::TransitionKind::from_code(kind))?;
        v.set_transition(index as usize, kind, ms)
    })
}

/// Apply To All: every slide gets slide `index`'s transition.
///
/// # Errors
///
/// `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn transition_to_all(index: u32) -> Result<bool, JsError> {
    with_viewer(|v| v.transition_to_all(index as usize))
}

/// The Animation Pane's list for slide `index`: nine numbers an effect, `[node (-1
/// for none), kind (-1 for another), class, start, ms, x, y, cx, cy]`, class 1
/// entrance, 2 emphasis, 3 exit, 0 other; the box in EMU, zero for a shape the
/// editor cannot select.
///
/// # Errors
///
/// `CS-TXN-0008` for a timing this build does not edit.
#[wasm_bindgen]
pub fn slide_animations(index: u32) -> Result<Vec<f64>, JsError> {
    with_viewer(|v| {
        let boxes = v.shapes(index as usize)?;
        let mut out = Vec::new();
        for a in v.animations(index as usize)? {
            let b = a.node.and_then(|n| boxes.iter().find(|b| b.node == n));
            let e = &a.entry;
            out.extend_from_slice(&[
                a.node.map_or(-1.0, f64::from),
                e.kind.map_or(-1.0, |k| f64::from(k.code())),
                f64::from(animate::class_code(e.class)),
                f64::from(e.start.code()),
                f64::from(e.duration_ms),
            ]);
            match b {
                Some(b) => {
                    out.extend_from_slice(&[b.x as f64, b.y as f64, b.cx as f64, b.cy as f64])
                }
                None => out.extend_from_slice(&[0.0; 4]),
            }
        }
        Ok(out)
    })
}

/// The name of shape `node` on slide `index`: document text, for `textContent` only.
///
/// # Errors
///
/// `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn shape_name(index: u32, node: u32) -> Result<String, JsError> {
    with_viewer(|v| v.shape_name(index as usize, node))
}

/// Add Animation: effect `kind` (0 Appear, 1 Fade, 2 Fly In, 3 Pulse, 4 Disappear,
/// 5 Fade Out) on shape `node`, starting `start` (0 on click, 1 with, 2 after previous).
///
/// # Errors
///
/// `CS-WSM-0006` for another number or a shape with no id; `CS-TXN-0008` for a
/// timing this build does not edit.
#[wasm_bindgen]
pub fn add_animation(index: u32, node: u32, kind: u32, start: u32) -> Result<bool, JsError> {
    with_viewer(|v| {
        let kind = bad(casual_slide_transaction::EffectKind::from_code(kind))?;
        let start = bad(casual_slide_transaction::EffectStart::from_code(start))?;
        v.add_animation(index as usize, node, kind, start)
    })
}

/// The Animation Styles gallery: shape `node`'s effects become `kind`; -1 removes them.
///
/// # Errors
///
/// As [`add_animation`].
#[wasm_bindgen]
pub fn replace_animation(index: u32, node: u32, kind: i32) -> Result<bool, JsError> {
    with_viewer(|v| {
        let kind = match u32::try_from(kind) {
            Ok(k) => Some(bad(casual_slide_transaction::EffectKind::from_code(k))?),
            Err(_) => None,
        };
        v.replace_animation(index as usize, node, kind)
    })
}

/// Delete effect `at` of slide `index`.
///
/// # Errors
///
/// `CS-TXN-0004` for a place past the end.
#[wasm_bindgen]
pub fn remove_animation(index: u32, at: u32) -> Result<bool, JsError> {
    with_viewer(|v| v.remove_animation(index as usize, at as usize))
}

/// Move effect `from` of slide `index` to `to`.
///
/// # Errors
///
/// `CS-TXN-0004` for a place past the end.
#[wasm_bindgen]
pub fn move_animation(index: u32, from: u32, to: u32) -> Result<bool, JsError> {
    with_viewer(|v| v.move_animation(index as usize, from as usize, to as usize))
}

/// Set when effect `at` of slide `index` starts.
///
/// # Errors
///
/// `CS-WSM-0006` for another number.
#[wasm_bindgen]
pub fn set_animation_start(index: u32, at: u32, start: u32) -> Result<bool, JsError> {
    with_viewer(|v| {
        let start = bad(casual_slide_transaction::EffectStart::from_code(start))?;
        v.set_animation_start(index as usize, at as usize, start)
    })
}

/// Slide `index` as PNG with the shapes `nodes` hidden, for a show's animation steps.
///
/// # Errors
///
/// As [`render_slide_png`].
#[wasm_bindgen]
pub fn render_slide_png_hiding(
    index: u32,
    width: u32,
    nodes: Vec<u32>,
) -> Result<Vec<u8>, JsError> {
    with_viewer(|v| v.render_png_hiding(index as usize, width, &nodes))
}

/// The direction of the transition into slide `index` (`l`, `u`, `horz out`), as
/// the show plays push, wipe and split; empty for an effect without one.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn transition_direction(index: u32) -> Result<String, JsError> {
    with_viewer(|v| {
        Ok(v.show_slide(index as usize)?
            .transition
            .map_or(String::new(), |t| t.direction))
    })
}

/* ── Rich objects (`VS-11`) ─────────────────────────────────────────────── */

/// Insert > Table: `rows` by `columns` on slide `index` in a box, EMU; returns its node.
///
/// # Errors
///
/// `CS-WSM-0006` for no rows or columns, or more than 75.
#[wasm_bindgen]
pub fn insert_table(
    index: u32,
    rows: u32,
    columns: u32,
    x: f64,
    y: f64,
    cx: f64,
    cy: f64,
) -> Result<u32, JsError> {
    with_viewer(|v| {
        v.insert_table(
            index as usize,
            rows as usize,
            columns as usize,
            [x as i64, y as i64, cx as i64, cy as i64],
        )
    })
}

/// A table's rows, columns, each column's width and each row's height, EMU.
///
/// # Errors
///
/// `CS-WSM-0006` for a node that holds no table.
#[wasm_bindgen]
pub fn table_shape(index: u32, node: u32) -> Result<Vec<f64>, JsError> {
    with_viewer(|v| {
        Ok(v.table_shape(index as usize, node)?
            .into_iter()
            .map(|n| n as f64)
            .collect())
    })
}

/// A cell's plain text, a line feed between paragraphs. Document text: the page shows
/// it through a form field's value only.
///
/// # Errors
///
/// `CS-WSM-0006` for a cell that is not there.
#[wasm_bindgen]
pub fn table_cell_text(index: u32, node: u32, row: u32, column: u32) -> Result<String, JsError> {
    with_viewer(|v| v.table_cell_text(index as usize, node, row as usize, column as usize))
}

/// Type a cell's text: one undo step. Returns whether anything changed.
///
/// # Errors
///
/// `CS-TXN-0013` for a cell that is not there.
#[wasm_bindgen]
pub fn set_table_cell_text(
    index: u32,
    node: u32,
    row: u32,
    column: u32,
    text: String,
) -> Result<bool, JsError> {
    with_viewer(|v| {
        v.set_table_cell_text(index as usize, node, row as usize, column as usize, &text)
    })
}

/// Insert a row (0) or column (2) at `at`, or delete one (1, 3): one undo step.
///
/// # Errors
///
/// `CS-TXN-0013` for an edit the table cannot take; `CS-WSM-0006` for another number.
#[wasm_bindgen]
pub fn edit_table(index: u32, node: u32, what: u32, at: u32) -> Result<bool, JsError> {
    with_viewer(|v| {
        let edit = TableEdit::from_u32(what).ok_or(BridgeError::BadArgument)?;
        v.edit_table(index as usize, node, edit, at as usize)
    })
}

/// Table Design's styles: `[id, name, id, name, …]`. Names are document text.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck.
#[wasm_bindgen]
pub fn table_styles() -> Result<Vec<String>, JsError> {
    with_viewer(|v| {
        Ok(v.table_styles()
            .into_iter()
            .flat_map(|(id, name)| [id, name])
            .collect())
    })
}

/// The style id a table names, or empty.
///
/// # Errors
///
/// `CS-WSM-0006` for a node that holds no table.
#[wasm_bindgen]
pub fn table_style(index: u32, node: u32) -> Result<String, JsError> {
    with_viewer(|v| v.table_style(index as usize, node))
}

/// Give a table a style by id: one undo step.
///
/// # Errors
///
/// `CS-WSM-0006` for an id that is not a GUID, or a node with no table.
#[wasm_bindgen]
pub fn set_table_style(index: u32, node: u32, style: String) -> Result<bool, JsError> {
    with_viewer(|v| v.set_table_style(index as usize, node, &style))
}

/// Insert > Chart: column (0), bar (1), line (2) or pie (3), with sample data, in a box,
/// EMU; returns its node.
///
/// # Errors
///
/// `CS-WSM-0006` for another kind.
#[wasm_bindgen]
pub fn insert_chart(
    index: u32,
    kind: u32,
    x: f64,
    y: f64,
    cx: f64,
    cy: f64,
) -> Result<u32, JsError> {
    with_viewer(|v| {
        let kind = casual_slide_export::new_chart::NewChartKind::from_u32(kind)
            .ok_or(BridgeError::BadArgument)?;
        v.insert_chart(
            index as usize,
            kind,
            [x as i64, y as i64, cx as i64, cy as i64],
        )
    })
}

/// A chart's data as `[kind, categories, series, category…, (name, value…)…]`, every
/// item a string and a blank value empty. Names are document text.
///
/// # Errors
///
/// `CS-WSM-0006` for a chart other than column, bar, line or pie.
#[wasm_bindgen]
pub fn chart_data(index: u32, node: u32) -> Result<Vec<String>, JsError> {
    with_viewer(|v| {
        let g = v.chart_grid(index as usize, node)?;
        let mut out = vec![
            g.kind.code().to_string(),
            g.categories.len().to_string(),
            g.series.len().to_string(),
        ];
        out.extend(g.categories.iter().cloned());
        for (name, values) in &g.series {
            out.push(name.clone());
            out.extend(
                values
                    .iter()
                    .map(|x| x.map_or(String::new(), |x| x.to_string())),
            );
        }
        Ok(out)
    })
}

/// Change a chart's data, in [`chart_data`]'s form: one undo step. A value that is not
/// a number is blank.
///
/// # Errors
///
/// `CS-WSM-0006` for a malformed list, or a chart of the file given another shape.
#[wasm_bindgen]
pub fn set_chart_data(index: u32, node: u32, data: Vec<String>) -> Result<bool, JsError> {
    with_viewer(|v| {
        let num = |i: usize| -> Result<usize, BridgeError> {
            data.get(i)
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|n| *n <= casual_slide_export::new_chart::MAX_GRID_SIDE)
                .ok_or(BridgeError::BadArgument)
        };
        let (n, m) = (num(1)?, num(2)?);
        let mut it = data.iter().skip(3);
        let categories: Vec<String> = it.by_ref().take(n).cloned().collect();
        let mut series = Vec::new();
        for _ in 0..m {
            let name = it.next().cloned().ok_or(BridgeError::BadArgument)?;
            let values = it
                .by_ref()
                .take(n)
                .map(|s| s.trim().parse::<f64>().ok().filter(|x| x.is_finite()))
                .collect();
            series.push((name, values));
        }
        if categories.len() != n {
            return Err(BridgeError::BadArgument);
        }
        let kind = v.chart_grid(index as usize, node)?.kind;
        v.set_chart_grid(
            index as usize,
            node,
            casual_slide_export::new_chart::ChartGrid {
                kind,
                categories,
                series,
            },
        )
    })
}

/* ── The Home tab's Font and Paragraph groups, and the clipboard (`UX-003`) ── */

/// What the Font, Size, alignment and Bullets controls show for the character before
/// `offset`: `[font, size in points, alignment token, "1" if bulleted]`.
///
/// # Errors
///
/// `CS-WSM-0005` when the shape has no text.
#[wasm_bindgen]
pub fn text_run_state(part: u32, node: u32, offset: u32) -> Result<Vec<String>, JsError> {
    with_viewer(|v| v.run_state(object(part, node), offset as usize))
}

/// Set a Font group value over `[start, end)`: `what` 0 font name, 1 size in points,
/// 2 font colour, 3 highlight (a colour as [`parse_color`] reads it, or empty to clear).
///
/// # Errors
///
/// `CS-WSM-0006` for a value it cannot read; the edit's own `CS-TXN` code.
#[wasm_bindgen]
pub fn text_set_run(
    part: u32,
    node: u32,
    start: u32,
    end: u32,
    what: u32,
    value: &str,
) -> Result<bool, JsError> {
    with_viewer(|v| {
        let what = format::RunSetting::from_code(what).ok_or(BridgeError::BadArgument)?;
        v.set_run(
            object(part, node),
            (start as usize, end as usize),
            what,
            value,
        )
    })
}

/// Increase (`up`) or Decrease Font Size over `[start, end)`.
///
/// # Errors
///
/// `CS-WSM-0006` for a range past the text; the edit's own `CS-TXN` code.
#[wasm_bindgen]
pub fn text_grow(part: u32, node: u32, start: u32, end: u32, up: bool) -> Result<bool, JsError> {
    with_viewer(|v| v.grow_text(object(part, node), (start as usize, end as usize), up))
}

/// Paragraph alignment (`what` 0: `l`, `ctr`, `r`, `just`), Bullets (`what` 1: `on`,
/// `off`, `toggle` or a gallery style), Numbering (`what` 2: `toggle`, `off` or a
/// scheme), List Level (`what` 3: `up`, `down`) or Line Spacing (`what` 4: lines) for
/// the paragraphs `[start, end)` touches (`PAR-06`).
///
/// # Errors
///
/// `CS-WSM-0006` for a value it cannot read; the edit's own `CS-TXN` code.
#[wasm_bindgen]
pub fn text_set_paragraphs(
    part: u32,
    node: u32,
    start: u32,
    end: u32,
    what: u32,
    value: &str,
) -> Result<bool, JsError> {
    with_viewer(|v| {
        v.set_paragraphs(
            object(part, node),
            (start as usize, end as usize),
            what,
            value,
        )
    })
}

/// Paste formatted text from another application over `[start, end)` (`PAR-11`), one
/// undo step: `texts` the runs, `looks` five numbers a run (bold, italic, underline as
/// -1 unsaid, 0 or 1; size in hundredths of a point and colour as `0xRRGGBB`, -1
/// unsaid), `lists` two a pasted paragraph (0 plain, 1 bulleted, 2 numbered; level).
/// Where the caret lands.
///
/// # Errors
///
/// `CS-WSM-0006` for a range past the text, lists not in twos or looks not in fives, or
/// past the paste bound of 262144 characters; the edit's own code.
#[wasm_bindgen]
pub fn text_paste_rich(
    part: u32,
    node: u32,
    start: u32,
    end: u32,
    texts: Vec<String>,
    looks: Vec<i32>,
    lists: Vec<u8>,
) -> Result<u32, JsError> {
    with_viewer(|v| {
        if looks.len() != texts.len().saturating_mul(5) || !lists.len().is_multiple_of(2) {
            return Err(BridgeError::BadArgument);
        }
        let flag = |n: i32| (n >= 0).then_some(n > 0);
        let pieces: Vec<format::RichPiece> = texts
            .into_iter()
            .zip(looks.chunks_exact(5))
            .map(|(text, l)| format::RichPiece {
                text,
                bold: l.first().copied().and_then(flag),
                italic: l.get(1).copied().and_then(flag),
                underline: l.get(2).copied().and_then(flag),
                size: l.get(3).copied().filter(|&n| n > 0),
                color: l.get(4).copied().and_then(|n| u32::try_from(n).ok()),
            })
            .collect();
        let paragraphs: Vec<format::RichParagraph> = lists
            .chunks_exact(2)
            .map(|p| format::RichParagraph {
                list: p.first().copied().unwrap_or(0),
                level: p.get(1).copied().unwrap_or(0),
            })
            .collect();
        v.paste_rich(
            object(part, node),
            (start as usize, end as usize),
            &pieces,
            &paragraphs,
        )
        .map(to_u32)
    })
}

/// The plain text of `[start, end)`, for Copy and Cut.
///
/// # Errors
///
/// `CS-WSM-0006` for a range past the text.
#[wasm_bindgen]
pub fn text_slice(part: u32, node: u32, start: u32, end: u32) -> Result<String, JsError> {
    with_viewer(|v| v.text_slice(object(part, node), start as usize, end as usize))
}

/// The Font box's list: OpenCalc's `font_families`, `[{n,f,k}, …]` as JSON.
#[wasm_bindgen]
pub fn font_families() -> String {
    format::font_families_json()
}

/// The CSS font stack a family is shown with: its bundled face first, as OpenCalc's.
#[wasm_bindgen]
pub fn font_css_stack(name: &str) -> String {
    casual_slide_text::substitution::css_stack(name)
}

/// The regular face of the bundled family `name` renders as, for the page to register
/// as a `FontFace` when the Font list first opens, so each name is shown in its face.
#[wasm_bindgen]
pub fn font_face_bytes(name: &str) -> Vec<u8> {
    casual_slide_text::faces::face_bytes_for(Some(name), false, false).to_vec()
}

/// Copy shapes `nodes` of slide `index` to the engine's clipboard, which outlives the
/// deck; how many it holds.
///
/// # Errors
///
/// `CS-WSM-0010` for a shape that cannot be copied yet.
#[wasm_bindgen]
pub fn copy_shapes(index: u32, nodes: Vec<u32>) -> Result<u32, JsError> {
    let clip = with_viewer(|v| v.copy_shapes(index as usize, &nodes))?;
    let n = to_u32(clip.len());
    clipboard::keep(clip);
    Ok(n)
}

/// Copy slides `indices` to the engine's clipboard for the thumbnail pane (`PAR-10`),
/// which outlives the deck; how many it holds.
///
/// # Errors
///
/// `CS-WSM-0006` for no slide, too many, or one that is not there.
#[wasm_bindgen]
pub fn copy_slides(indices: Vec<u32>) -> Result<u32, JsError> {
    let indices: Vec<usize> = indices.into_iter().map(|i| i as usize).collect();
    let clip = with_viewer(|v| v.copy_slides(&indices))?;
    let n = to_u32(clip.len());
    clipboard::keep_slides(clip);
    Ok(n)
}

/// How many slides the engine's clipboard holds.
#[wasm_bindgen]
pub fn clipboard_slides() -> u32 {
    clipboard::kept_slides().map_or(0, |c| to_u32(c.len()))
}

/// Paste the engine's slides at place `at`, one undo step: `[first place, how many,
/// objects left out]`, the last only when they came from another deck.
///
/// # Errors
///
/// `CS-WSM-0006` for a place past the end; the edit's own code.
#[wasm_bindgen]
pub fn paste_slides(at: u32) -> Result<Vec<u32>, JsError> {
    let Some(clip) = clipboard::kept_slides() else {
        return Ok(Vec::new());
    };
    with_viewer(|v| {
        let other = !clip.from(v);
        let first = v.paste_slides(at as usize, &clip)?;
        let skipped = if other { clip.skipped() } else { 0 };
        Ok(vec![to_u32(first), to_u32(clip.len()), to_u32(skipped)])
    })
}

/// How many shapes the engine's clipboard holds.
#[wasm_bindgen]
pub fn clipboard_shapes() -> u32 {
    clipboard::kept().map_or(0, |c| to_u32(c.len()))
}

/// Paste the engine's clipboard onto slide `index`, one undo step; the new nodes.
///
/// # Errors
///
/// `CS-WSM-0010` for a shape that cannot be pasted onto this slide.
#[wasm_bindgen]
pub fn paste_shapes(index: u32) -> Result<Vec<u32>, JsError> {
    let Some(clip) = clipboard::kept() else {
        return Ok(Vec::new());
    };
    with_viewer(|v| v.paste_shapes(index as usize, &clip))
}

/// Duplicate shapes `nodes` of slide `index` beside themselves; the new nodes.
///
/// # Errors
///
/// `CS-WSM-0010` for a shape that cannot be copied yet.
#[wasm_bindgen]
pub fn duplicate_shapes(index: u32, nodes: Vec<u32>) -> Result<Vec<u32>, JsError> {
    with_viewer(|v| v.duplicate_shapes(index as usize, &nodes))
}

/// Place several shapes at once, one undo step: `boxes` is `node, x, y, cx, cy, rot`
/// for each, rotation in 60000ths of a degree.
///
/// # Errors
///
/// `CS-WSM-0006` for a list not in sixes; the edit's own code.
#[wasm_bindgen]
pub fn set_shape_transforms(index: u32, boxes: Vec<f64>) -> Result<(), JsError> {
    with_viewer(|v| {
        if !boxes.len().is_multiple_of(6) {
            return Err(BridgeError::BadArgument);
        }
        let mut out = Vec::with_capacity(boxes.len() / 6);
        for b in boxes.chunks_exact(6) {
            let mut one = [0i64; 6];
            for (slot, &x) in one.iter_mut().zip(b) {
                if !x.is_finite() {
                    return Err(BridgeError::BadArgument);
                }
                // Finite and rounded; `as` saturates at the ends.
                #[allow(clippy::cast_possible_truncation)]
                {
                    *slot = x.round() as i64;
                }
            }
            out.push(one);
        }
        v.set_transforms(index as usize, &out)
    })
}

/// Delete several shapes at once, one undo step.
///
/// # Errors
///
/// `CS-WSM-0004` if any one cannot be deleted yet; nothing changes then.
#[wasm_bindgen]
pub fn delete_shapes(index: u32, nodes: Vec<u32>) -> Result<(), JsError> {
    with_viewer(|v| v.delete_shapes(index as usize, &nodes))
}
