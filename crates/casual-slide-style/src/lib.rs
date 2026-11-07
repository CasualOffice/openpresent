//! `casual-slide-style`: effective values, resolved at read time through the
//! inheritance chain (ADR-006, `docs/22` §Inheritance). Nothing here changes the
//! model; every answer says which level it came from.
//!
//! The chain for a slide is slide, layout, master, then the presentation's
//! defaults and the schema's; for a notes slide it is notes slide, notes master.
//! Tier 1D resolves what Tier 1D models: placeholder matching, transform, colour
//! map, background, header and footer flags, the visibility of master and layout
//! shapes, a slide's hidden flag, and which master text style applies. Tier 1F adds
//! every text property, in [`text`]. Fills and lines join as their tiers model them.
//!
//! # Placeholder matching (BEH-01)
//!
//! - A slide placeholder inherits from the layout placeholder with the same `idx`.
//!   Documented: `MS-OI29500` 2.1.1127, "PowerPoint defines a placeholder shape on
//!   a slide to inherit from the slide layout placeholder with the same idx value".
//!   An absent `idx` is 0, the schema default. `idx` 4294967295, which `MS-OI29500`
//!   exempts from the uniqueness rule, matches nothing (Inferred: PowerPoint
//!   writes it on placeholders that have lost their layout counterpart).
//! - A notes slide placeholder inherits from the notes master placeholder with the
//!   same `type`. Documented: the same note.
//! - A layout placeholder inherits from the master placeholder of the same type,
//!   else of the same class: `ctrTitle` from `title`, and `subTitle`, `obj`,
//!   `chart`, `tbl`, `clipArt`, `dgm`, `media` and `pic` from `body`. **Inferred**:
//!   the standard and `MS-OI29500` are silent; this is how LibreOffice and
//!   python-pptx resolve it, and how master placeholders are laid out (one per type).
//! - A slide placeholder's master counterpart is its layout counterpart's.
//! - Connectors and groups are never placeholders (`MS-OI29500` 2.1.1126), so a
//!   `ph` on one matches nothing.
//!
//! # Inheritance by property family (BEH-02)
//!
//! | Family | Rule | Evidence |
//! | --- | --- | --- |
//! | Transform | The nearest level whose `xfrm` has both `off` and `ext`, as a unit; `rot` and flips from that same `xfrm`, absent meaning the schema default | Inferred |
//! | Colour map | Slide override, else layout override, else the master's `clrMap` | `docs/22`; `effective_color_map`; open under BEH-04 |
//! | Background | Slide `bg`, else layout `bg`, else master `bg` | ECMA-376 19.3.1.1; inheritance Inferred |
//! | Header, footer | Layout `hf`, else master `hf`, as a unit; an absent attribute of a present `hf` is the schema's `true` | Inferred |
//! | Master shapes | Hidden if the slide or the layout says `showMasterSp="0"`; layout shapes hidden if the slide does | ECMA-376 19.3.1.38, 19.3.1.39; the slide hiding both is Inferred |
//! | Text style | Title placeholders take `titleStyle`; `dt`, `ftr`, `sldNum`, `hdr` and non-placeholders `otherStyle`; other placeholders `bodyStyle`; a notes body `notesStyle`; else the presentation default | ECMA-376 19.3.1.51 for title, body, other; the footer types Inferred |
//! | Text properties | Run or paragraph, shape list style, the shape's `fontRef`, layout placeholder, the theme's object defaults, master placeholder, master text styles, presentation default | [`text`] |

pub mod text;

use casual_slide_model::presentation::{Point, PositiveSize};
use casual_slide_model::shape::{NodeKind, ObjectId, Placeholder, PlaceholderType, ShapeNode};
use casual_slide_model::slide::{Background, HeaderFooter, SlideKind, SlidePart, SlideParts};
use casual_slide_model::theme::{ColorMap, ColorMapOverride};

/// Where an effective value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Level {
    /// The slide, or notes slide, itself.
    Slide,
    /// Its layout.
    Layout,
    /// Its master, or notes master.
    Master,
    /// The presentation's defaults.
    Presentation,
    /// No level said, so the schema default applies.
    SchemaDefault,
}

/// An effective value and the level it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resolved<T> {
    /// The value.
    pub value: T,
    /// Where it came from.
    pub from: Level,
}

impl<T> Resolved<T> {
    fn new(value: T, from: Level) -> Self {
        Self { value, from }
    }
}

/// `idx` that matches nothing.
pub const UNMATCHED_INDEX: u32 = u32::MAX;

/// A placeholder's type, with the schema default `obj` when absent.
pub fn placeholder_type(ph: &Placeholder) -> PlaceholderType {
    ph.placeholder_type.unwrap_or(PlaceholderType::Obj)
}

/// A placeholder's index, with the schema default 0 when absent.
pub fn placeholder_index(ph: &Placeholder) -> u32 {
    ph.idx.unwrap_or(0)
}

/// What a master holds one placeholder for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlaceholderClass {
    /// `title`, `ctrTitle`.
    Title,
    /// `body` and every content type.
    Body,
    /// `dt`.
    DateTime,
    /// `ftr`.
    Footer,
    /// `sldNum`.
    SlideNumber,
    /// `hdr`.
    Header,
    /// `sldImg`.
    SlideImage,
}

/// The class a type falls in for master matching.
pub fn class_of(t: PlaceholderType) -> PlaceholderClass {
    use PlaceholderType as T;
    match t {
        T::Title | T::CtrTitle => PlaceholderClass::Title,
        T::Dt => PlaceholderClass::DateTime,
        T::Ftr => PlaceholderClass::Footer,
        T::SldNum => PlaceholderClass::SlideNumber,
        T::Hdr => PlaceholderClass::Header,
        T::SldImg => PlaceholderClass::SlideImage,
        T::Body
        | T::SubTitle
        | T::Obj
        | T::Chart
        | T::Tbl
        | T::ClipArt
        | T::Dgm
        | T::Media
        | T::Pic => PlaceholderClass::Body,
    }
}

/// A node's placeholder, if it can be one.
pub fn placeholder_of(node: &ShapeNode) -> Option<&Placeholder> {
    if matches!(node.kind, NodeKind::Connector | NodeKind::Group) {
        return None;
    }
    node.placeholder()
}

/// Every placeholder in a part's shape tree, in document order.
pub fn placeholders(part: &SlidePart) -> impl Iterator<Item = (&ShapeNode, &Placeholder)> {
    part.shape_tree()
        .into_iter()
        .flat_map(ShapeNode::descendants)
        .filter_map(|n| placeholder_of(n).map(|ph| (n, ph)))
}

/// The placeholder on `layout` a slide placeholder inherits from: the same `idx`.
pub fn match_by_index<'a>(ph: &Placeholder, layout: &'a SlidePart) -> Option<&'a ShapeNode> {
    let idx = placeholder_index(ph);
    if idx == UNMATCHED_INDEX {
        return None;
    }
    placeholders(layout)
        .find(|(_, other)| placeholder_index(other) == idx)
        .map(|(n, _)| n)
}

/// The placeholder on `master` of the same type, else of the same class.
pub fn match_by_type<'a>(ph: &Placeholder, master: &'a SlidePart) -> Option<&'a ShapeNode> {
    let t = placeholder_type(ph);
    placeholders(master)
        .find(|(_, other)| placeholder_type(other) == t)
        .or_else(|| {
            placeholders(master).find(|(_, other)| class_of(placeholder_type(other)) == class_of(t))
        })
        .map(|(n, _)| n)
}

/// The placeholder on `master` of exactly the same type, for notes slides.
pub fn match_by_exact_type<'a>(ph: &Placeholder, master: &'a SlidePart) -> Option<&'a ShapeNode> {
    let t = placeholder_type(ph);
    placeholders(master)
        .find(|(_, other)| placeholder_type(other) == t)
        .map(|(n, _)| n)
}

/// The placeholders a shape inherits from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Inheritance<'a> {
    /// On the layout.
    pub layout: Option<&'a ShapeNode>,
    /// On the master.
    pub master: Option<&'a ShapeNode>,
}

/// A transform with every field effective.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectiveTransform {
    /// `off`.
    pub offset: Point,
    /// `ext`.
    pub extent: PositiveSize,
    /// `@rot`, 0 when absent.
    pub rotation: i32,
    /// `@flipH`, false when absent.
    pub flip_h: bool,
    /// `@flipV`, false when absent.
    pub flip_v: bool,
}

/// Header and footer flags with every field effective.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectiveHeaderFooter {
    /// `@sldNum`.
    pub slide_number: bool,
    /// `@hdr`.
    pub header: bool,
    /// `@ftr`.
    pub footer: bool,
    /// `@dt`.
    pub date_time: bool,
}

/// Which list style a shape's text starts from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextStyle {
    /// The master's `titleStyle`.
    Title,
    /// The master's `bodyStyle`.
    Body,
    /// The master's `otherStyle`.
    Other,
    /// The notes master's `notesStyle`.
    Notes,
    /// The presentation's `defaultTextStyle`.
    PresentationDefault,
}

/// One slide's inheritance chain. Any level may be missing: a damaged deck, or a
/// chain asked of a layout or a master.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Chain<'a> {
    /// The slide or notes slide.
    pub slide: Option<&'a SlidePart>,
    /// The layout.
    pub layout: Option<&'a SlidePart>,
    /// The master, notes master or handout master.
    pub master: Option<&'a SlidePart>,
}

impl<'a> Chain<'a> {
    /// The chain for any part of a deck, by ordinal, as far as the deck has it.
    pub fn of(parts: &'a SlideParts, ordinal: u32) -> Self {
        let Some(part) = parts.get(ordinal) else {
            return Self::default();
        };
        let entry = parts.structure.entry(ordinal);
        match part.kind {
            SlideKind::Slide | SlideKind::Notes => {
                let (layout, master) = parts.chain(ordinal);
                Self {
                    slide: Some(part),
                    layout,
                    master,
                }
            }
            SlideKind::Layout => Self {
                slide: None,
                layout: Some(part),
                master: entry.and_then(|e| e.master).and_then(|o| parts.get(o)),
            },
            SlideKind::Master | SlideKind::NotesMaster | SlideKind::HandoutMaster => Self {
                slide: None,
                layout: None,
                master: Some(part),
            },
        }
    }

    fn is_notes(&self) -> bool {
        self.slide.is_some_and(|s| s.kind == SlideKind::Notes)
    }

    fn level_of(&self, id: ObjectId) -> Option<Level> {
        let holds = |p: Option<&SlidePart>| p.is_some_and(|p| p.id.part == id.part);
        if holds(self.slide) {
            Some(Level::Slide)
        } else if holds(self.layout) {
            Some(Level::Layout)
        } else if holds(self.master) {
            Some(Level::Master)
        } else {
            None
        }
    }

    /// The placeholders a shape inherits from. Empty for a shape that is not a
    /// placeholder, or that is not on a level of this chain.
    pub fn inherited(&self, shape: &ShapeNode) -> Inheritance<'a> {
        let Some(ph) = placeholder_of(shape) else {
            return Inheritance::default();
        };
        match self.level_of(shape.id) {
            Some(Level::Slide) if self.is_notes() => Inheritance {
                layout: None,
                master: self.master.and_then(|m| match_by_exact_type(ph, m)),
            },
            Some(Level::Slide) => {
                let layout = self.layout.and_then(|l| match_by_index(ph, l));
                let master = layout
                    .and_then(placeholder_of)
                    .and_then(|lph| self.master.and_then(|m| match_by_type(lph, m)));
                Inheritance { layout, master }
            }
            Some(Level::Layout) => Inheritance {
                layout: None,
                master: self.master.and_then(|m| match_by_type(ph, m)),
            },
            _ => Inheritance::default(),
        }
    }

    /// The shape itself and what it inherits from, nearest first, with levels.
    fn levels(&self, shape: &'a ShapeNode) -> Vec<(&'a ShapeNode, Level)> {
        let own = self.level_of(shape.id).unwrap_or(Level::Slide);
        let inherited = self.inherited(shape);
        let mut out = vec![(shape, own)];
        out.extend(inherited.layout.map(|n| (n, Level::Layout)));
        out.extend(inherited.master.map(|n| (n, Level::Master)));
        out
    }

    /// A shape's placeholder type: the nearest that states one, else `obj`.
    pub fn placeholder_type(&self, shape: &'a ShapeNode) -> Option<Resolved<PlaceholderType>> {
        placeholder_of(shape)?;
        let stated = self.levels(shape).into_iter().find_map(|(n, level)| {
            placeholder_of(n)?
                .placeholder_type
                .map(|t| Resolved::new(t, level))
        });
        Some(stated.unwrap_or(Resolved::new(PlaceholderType::Obj, Level::SchemaDefault)))
    }

    /// A shape's effective transform: its own, else its layout placeholder's, else
    /// its master placeholder's. `None` if no level places it.
    pub fn transform(&self, shape: &'a ShapeNode) -> Option<Resolved<EffectiveTransform>> {
        self.levels(shape).into_iter().find_map(|(n, level)| {
            let t = n.transform?;
            Some(Resolved::new(
                EffectiveTransform {
                    offset: t.offset?,
                    extent: t.extent?,
                    rotation: t.rotation.unwrap_or(0),
                    flip_h: t.flip_h.unwrap_or(false),
                    flip_v: t.flip_v.unwrap_or(false),
                },
                level,
            ))
        })
    }

    /// The colour map in force.
    pub fn color_map(&self) -> Option<Resolved<&'a ColorMap>> {
        let overriding = |p: Option<&'a SlidePart>| match p?.color_map_override.as_ref()? {
            ColorMapOverride::Override(map) => Some(map),
            ColorMapOverride::Master => None,
        };
        if let Some(map) = overriding(self.slide) {
            return Some(Resolved::new(map, Level::Slide));
        }
        if let Some(map) = overriding(self.layout) {
            return Some(Resolved::new(map, Level::Layout));
        }
        let map = self.master?.color_map.as_ref()?;
        Some(Resolved::new(map, Level::Master))
    }

    /// The background in force.
    pub fn background(&self) -> Option<Resolved<&'a Background>> {
        let bg = |p: Option<&'a SlidePart>| p?.common.as_ref()?.background.as_ref();
        [
            (self.slide, Level::Slide),
            (self.layout, Level::Layout),
            (self.master, Level::Master),
        ]
        .into_iter()
        .find_map(|(p, level)| bg(p).map(|b| Resolved::new(b, level)))
    }

    /// Whether the layout's non-placeholder shapes are drawn on the slide.
    pub fn layout_shapes_visible(&self) -> Resolved<bool> {
        match self.slide.and_then(|s| s.show_master_shapes) {
            Some(v) => Resolved::new(v, Level::Slide),
            None => Resolved::new(true, Level::SchemaDefault),
        }
    }

    /// Whether the master's non-placeholder shapes are drawn: hidden if the slide
    /// or the layout says so.
    pub fn master_shapes_visible(&self) -> Resolved<bool> {
        let slide = self.slide.and_then(|s| s.show_master_shapes);
        let layout = self.layout.and_then(|l| l.show_master_shapes);
        match (slide, layout) {
            (Some(false), _) => Resolved::new(false, Level::Slide),
            (_, Some(false)) => Resolved::new(false, Level::Layout),
            (Some(true), _) => Resolved::new(true, Level::Slide),
            (_, Some(true)) => Resolved::new(true, Level::Layout),
            (None, None) => Resolved::new(true, Level::SchemaDefault),
        }
    }

    /// The shapes inherited for drawing, back to front: the master's, then the
    /// layout's, each as far as visible. Placeholders are never drawn from these
    /// levels; a slide's own placeholders inherit from them instead.
    pub fn inherited_shapes(&self) -> Vec<&'a ShapeNode> {
        let drawable = |p: Option<&'a SlidePart>| {
            p.and_then(SlidePart::shape_tree)
                .into_iter()
                .flat_map(|t| t.children.iter())
                .filter(|n| placeholder_of(n).is_none())
        };
        let mut out = Vec::new();
        if self.slide.is_some() || self.layout.is_some() {
            if self.master_shapes_visible().value {
                out.extend(drawable(self.master));
            }
            if self.slide.is_some() && self.layout_shapes_visible().value {
                out.extend(drawable(self.layout));
            }
        }
        out
    }

    /// Whether the slide is hidden in a show.
    pub fn hidden(&self) -> Resolved<bool> {
        match self.slide.and_then(|s| s.show) {
            Some(show) => Resolved::new(!show, Level::Slide),
            None => Resolved::new(false, Level::SchemaDefault),
        }
    }

    /// Header and footer flags: the layout's `hf`, else the master's.
    pub fn header_footer(&self) -> Option<Resolved<EffectiveHeaderFooter>> {
        let effective = |hf: &HeaderFooter| EffectiveHeaderFooter {
            slide_number: hf.slide_number.unwrap_or(true),
            header: hf.header.unwrap_or(true),
            footer: hf.footer.unwrap_or(true),
            date_time: hf.date_time.unwrap_or(true),
        };
        [(self.layout, Level::Layout), (self.master, Level::Master)]
            .into_iter()
            .find_map(|(p, level)| {
                p?.header_footer
                    .as_ref()
                    .map(|hf| Resolved::new(effective(hf), level))
            })
    }

    /// The list style a shape's text starts from.
    pub fn text_style(&self, shape: &'a ShapeNode) -> Resolved<TextStyle> {
        let wanted = match self.placeholder_type(shape).map(|t| t.value) {
            None => TextStyle::Other,
            Some(t) => match class_of(t) {
                PlaceholderClass::Title => TextStyle::Title,
                PlaceholderClass::Body if self.is_notes() => TextStyle::Notes,
                PlaceholderClass::Body => TextStyle::Body,
                _ => TextStyle::Other,
            },
        };
        let present = match wanted {
            _ if self.is_notes() && wanted != TextStyle::Notes => false,
            slot => self.master_list_style(slot).is_some(),
        };
        if present {
            Resolved::new(wanted, Level::Master)
        } else {
            Resolved::new(TextStyle::PresentationDefault, Level::Presentation)
        }
    }

    /// `showMasterPhAnim`: whether the master's placeholder animations play.
    pub fn master_placeholder_animations(&self) -> Resolved<bool> {
        [(self.slide, Level::Slide), (self.layout, Level::Layout)]
            .into_iter()
            .find_map(|(p, level)| {
                p?.show_master_placeholder_animations
                    .map(|v| Resolved::new(v, level))
            })
            .unwrap_or(Resolved::new(true, Level::SchemaDefault))
    }
}
