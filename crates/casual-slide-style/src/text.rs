//! Effective text properties: run, paragraph, shape list style, the shape's `fontRef`,
//! layout placeholder, the theme's object defaults, master placeholder, master text
//! styles, presentation default (ADR-006, `docs/22`).
//! Nothing here changes the model or fills anything in it. Every answer says which
//! level it came from, so what the ribbon says and what is drawn cannot differ.
//!
//! # The chain, and what it rests on
//!
//! | Rule | Evidence |
//! | --- | --- |
//! | An attribute absent from `pPr` or `rPr` comes from `lstStyle` at the paragraph's level in the same part, then, in PowerPoint, the master | `MS-OI29500` 2.1.1388 and 2.1.1399 |
//! | `bodyPr` resolves the same way, attribute by attribute, ending at the master | `MS-OI29500` 2.1.1379 |
//! | Layout placeholder and master placeholder sit between the shape and the master text styles | `docs/22`; the placement of the layout is **Inferred**: the notes name the part, the theme and the master, never a layout |
//! | The theme's object defaults come after what the shape's own part says and before the master: `txDef`, then `lnDef`, then `spDef` | `MS-OI29500` 2.1.1379 for `bodyPr`, which names all three in this order; 2.1.1388 and 2.1.1399 for `pPr` and `rPr` name the theme's `lstStyle` without saying which, so the same order is **Inferred**. Taking all three for every shape is the note read as written, and whether "corresponding" narrows it to one by shape kind is open until `PV-01`. The layout going before them is **Inferred** |
//! | A shape's `p:style/fontRef` gives a run its colour and font, under whatever its own `txBody` says | `MS-OI29500` 20.1.2.2.37 (c): "child elements from txBody ... will override similar style properties defined in the fontRef". That it sits above the layout and the rest is **Inferred** |
//! | `fontRef` on a picture or a connector gives nothing | `MS-OI29500` 20.1.2.2.37 (a) |
//! | `defPPr` takes no part | `MS-OI29500` 2.1.1383, 2.1.1387: "Office ignores the defPPr element" |
//! | `lvl` inside `lvlNpPr` takes no part: a level's slot says its level | `MS-OI29500` 2.1.1406 (b) |
//! | `pPr/defRPr` takes no part in a run's chain | **Inferred** from `MS-OI29500` 2.1.1398 and 2.1.1399, whose order names only `lstStyle` ancestors |
//! | The presentation's `defaultTextStyle` comes last, after the master text styles | `docs/22`; **Inferred**: no note names it in the order |
//! | A non-placeholder takes `otherStyle`, then the default | ECMA-376 19.3.1.51 for `txStyles`; **Inferred** as to how far PowerPoint uses it, open until `PV-01` |
//! | A property family is one unit where the schema makes it a choice: spacing, each bullet group, fill, effects, underline line and fill, autofit. A nearer `gradFill` hides a further `solidFill` | **Inferred** from the schema's `choice`; `tabLst` replaces rather than merges, also Inferred |
//! | `grpFill` in text is `noFill` | `MS-OI29500` 2.1.1298 |

use std::borrow::Cow;

use casual_slide_model::fill::Fill;
use casual_slide_model::presentation::TextFont;
use casual_slide_model::shape::{NodeKind, ShapeNode};
use casual_slide_model::text::{
    BodyProperties, CharacterProperties, ParagraphProperties, TextListStyle,
};
use casual_slide_model::theme::{FontCollectionIndex, FontRef, ObjectDefaults};

use crate::{Chain, TextStyle};

/// Where an effective text value came from, nearest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextLevel {
    /// The run's `rPr`, or `endParaRPr` for the paragraph mark.
    Run,
    /// The paragraph's `pPr`.
    Paragraph,
    /// The shape's own `bodyPr`.
    ShapeBody,
    /// The shape's own `lstStyle`.
    ShapeListStyle,
    /// The shape's `p:style/fontRef`. Only a run's fill and fonts ask it.
    ShapeStyle,
    /// The matching placeholder on the layout: its `bodyPr` or `lstStyle`.
    LayoutPlaceholder,
    /// The theme's `txDef`: its `bodyPr` or `lstStyle`.
    ThemeTextDefault,
    /// The theme's `lnDef`: its `bodyPr` or `lstStyle`.
    ThemeLineDefault,
    /// The theme's `spDef`: its `bodyPr` or `lstStyle`.
    ThemeShapeDefault,
    /// The matching placeholder on the master: its `bodyPr` or `lstStyle`.
    MasterPlaceholder,
    /// The master's `titleStyle`, `bodyStyle`, `otherStyle`, or `notesStyle`.
    MasterTextStyles,
    /// The presentation's `defaultTextStyle`.
    PresentationDefault,
    /// No level said, so the documented default applies.
    SchemaDefault,
}

/// An effective text value and the level it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextResolved<T> {
    /// The value.
    pub value: T,
    /// Where it came from.
    pub from: TextLevel,
}

impl<T> TextResolved<T> {
    fn new(value: T, from: TextLevel) -> Self {
        Self { value, from }
    }

    fn map<U>(self, f: impl FnOnce(T) -> U) -> TextResolved<U> {
        TextResolved {
            value: f(self.value),
            from: self.from,
        }
    }
}

/// Which of a run's fonts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontScript {
    /// `latin`.
    Latin,
    /// `ea`.
    EastAsian,
    /// `cs`.
    ComplexScript,
}

/// A run's effective font, as the level says it. Nothing here looks a font up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunFont<'a> {
    /// `latin`, `ea` or `cs`, as a level states it.
    Stated(&'a TextFont),
    /// `fontRef/@idx`: the theme's major or minor font.
    Theme(FontCollectionIndex),
}

/// Every level one shape's text inherits from. Costs a few references; build it once
/// for a shape and ask it as many questions as needed.
#[derive(Debug, Clone, Default)]
pub struct TextChain<'a> {
    /// `bodyPr` of the shape's inherited placeholders, nearest first.
    bodies: Vec<(TextLevel, &'a BodyProperties)>,
    /// List styles, nearest first.
    styles: Vec<(TextLevel, &'a TextListStyle)>,
    /// The shape's `p:style/fontRef`, on a shape and not a picture or connector.
    font_ref: Option<&'a FontRef>,
}

/// `lvl` as a paragraph states it, from 0. An absent or unusable one is level 0.
pub fn paragraph_level(own: Option<&ParagraphProperties>) -> u8 {
    own.and_then(|p| p.level).unwrap_or(0)
}

impl<'a> Chain<'a> {
    /// The levels a shape's text inherits from. `default` is the presentation's
    /// `defaultTextStyle`, which belongs to the deck rather than to a slide part, and
    /// `theme` the object defaults of the theme in force.
    pub fn text_chain(
        &self,
        shape: &'a ShapeNode,
        default: Option<&'a TextListStyle>,
        theme: Option<&'a ObjectDefaults>,
    ) -> TextChain<'a> {
        let mut chain = TextChain::default();
        if shape.kind == NodeKind::Shape {
            chain.font_ref = shape.style.as_ref().and_then(|s| s.font.as_ref());
        }
        if let Some(style) = shape.text_body.as_ref().and_then(|b| b.list_style.as_ref()) {
            chain.styles.push((TextLevel::ShapeListStyle, style));
        }
        let inherited = self.inherited(shape);
        let placeholder = |chain: &mut TextChain<'a>, level, node: Option<&'a ShapeNode>| {
            let Some(body) = node.and_then(|n| n.text_body.as_deref()) else {
                return;
            };
            if let Some(props) = &body.properties {
                chain.bodies.push((level, props));
            }
            if let Some(style) = &body.list_style {
                chain.styles.push((level, style));
            }
        };
        placeholder(&mut chain, TextLevel::LayoutPlaceholder, inherited.layout);
        for (level, definition) in [
            (
                TextLevel::ThemeTextDefault,
                theme.and_then(|t| t.text.as_ref()),
            ),
            (
                TextLevel::ThemeLineDefault,
                theme.and_then(|t| t.line.as_ref()),
            ),
            (
                TextLevel::ThemeShapeDefault,
                theme.and_then(|t| t.shape.as_ref()),
            ),
        ] {
            let Some(definition) = definition else {
                continue;
            };
            if let Some(props) = &definition.body_properties {
                chain.bodies.push((level, props));
            }
            if let Some(style) = &definition.list_style {
                chain.styles.push((level, style));
            }
        }
        placeholder(&mut chain, TextLevel::MasterPlaceholder, inherited.master);
        if let Some(style) = self.master_list_style(self.text_style(shape).value) {
            chain.styles.push((TextLevel::MasterTextStyles, style));
        }
        if let Some(style) = default {
            chain.styles.push((TextLevel::PresentationDefault, style));
        }
        chain
    }

    /// A master list style by name. `None` for the presentation's, which the deck holds.
    pub fn master_list_style(&self, style: TextStyle) -> Option<&'a TextListStyle> {
        let master = self.master?;
        match style {
            TextStyle::Title => master.text_styles.as_ref()?.title.as_ref(),
            TextStyle::Body => master.text_styles.as_ref()?.body.as_ref(),
            TextStyle::Other => master.text_styles.as_ref()?.other.as_ref(),
            TextStyle::Notes => master.notes_style.as_ref(),
            TextStyle::PresentationDefault => None,
        }
    }
}

impl<'a> TextChain<'a> {
    /// How many list styles this chain holds.
    pub fn list_style_count(&self) -> usize {
        self.styles.len()
    }

    /// A paragraph property: `pPr`, then each list style's level.
    ///
    /// `own` is the paragraph's `pPr`; its `lvl` picks the level.
    pub fn paragraph<T>(
        &self,
        own: Option<&'a ParagraphProperties>,
        get: impl Fn(&'a ParagraphProperties) -> Option<T>,
    ) -> Option<TextResolved<T>> {
        if let Some(value) = own.and_then(&get) {
            return Some(TextResolved::new(value, TextLevel::Paragraph));
        }
        let level = paragraph_level(own);
        self.styles.iter().find_map(|(from, style)| {
            let value = get(style.level(level)?)?;
            Some(TextResolved::new(value, *from))
        })
    }

    /// A run property: the run's `rPr` (or `endParaRPr` for the paragraph mark), then
    /// each list style's `defRPr` at the paragraph's level. `pPr/defRPr` is not asked,
    /// nor the shape's `fontRef`, which only [`Self::run_fill`] and [`Self::run_font`]
    /// can answer from.
    pub fn run<T>(
        &self,
        run: Option<&'a CharacterProperties>,
        paragraph: Option<&ParagraphProperties>,
        get: impl Fn(&'a CharacterProperties) -> Option<T>,
    ) -> Option<TextResolved<T>> {
        if let Some(value) = run.and_then(&get) {
            return Some(TextResolved::new(value, TextLevel::Run));
        }
        let level = paragraph_level(paragraph);
        self.styles.iter().find_map(|(from, style)| {
            let value = get(style.level(level)?.default_run.as_deref()?)?;
            Some(TextResolved::new(value, *from))
        })
    }

    /// A body property: the shape's `bodyPr`, its layout placeholder's, the theme's
    /// object defaults', its master placeholder's. The master text styles and the
    /// presentation default hold no body properties.
    pub fn body<T>(
        &self,
        own: Option<&'a BodyProperties>,
        get: impl Fn(&'a BodyProperties) -> Option<T>,
    ) -> Option<TextResolved<T>> {
        if let Some(value) = own.and_then(&get) {
            return Some(TextResolved::new(value, TextLevel::ShapeBody));
        }
        self.bodies.iter().find_map(|(from, props)| {
            let value = get(props)?;
            Some(TextResolved::new(value, *from))
        })
    }

    /// As [`Self::run`], with the shape's `fontRef` asked straight after what the shape's
    /// own `txBody` says: `MS-OI29500` 20.1.2.2.37 (c) puts that above it.
    fn run_or_style<T>(
        &self,
        run: Option<&'a CharacterProperties>,
        paragraph: Option<&ParagraphProperties>,
        get: impl Fn(&'a CharacterProperties) -> Option<T>,
        style: impl Fn(&'a FontRef) -> Option<T>,
    ) -> Option<TextResolved<T>> {
        if let Some(value) = run.and_then(&get) {
            return Some(TextResolved::new(value, TextLevel::Run));
        }
        let level = paragraph_level(paragraph);
        let at = |from: TextLevel, list: &'a TextListStyle| {
            let value = get(list.level(level)?.default_run.as_deref()?)?;
            Some(TextResolved::new(value, from))
        };
        let (own, others) = match self.styles.split_first() {
            Some((first, rest)) if first.0 == TextLevel::ShapeListStyle => (Some(first), rest),
            _ => (None, self.styles.as_slice()),
        };
        own.and_then(|(from, list)| at(*from, list))
            .or_else(|| {
                let value = style(self.font_ref?)?;
                Some(TextResolved::new(value, TextLevel::ShapeStyle))
            })
            .or_else(|| others.iter().find_map(|(from, list)| at(*from, list)))
    }

    /// A run's fill, with `grpFill` taken as `noFill`, as Office does in text. A shape's
    /// `fontRef` colour answers as the solid fill it stands for.
    pub fn run_fill(
        &self,
        run: Option<&'a CharacterProperties>,
        paragraph: Option<&ParagraphProperties>,
    ) -> Option<TextResolved<Cow<'a, Fill>>> {
        self.run_or_style(
            run,
            paragraph,
            |c| c.fill.as_ref().map(Cow::Borrowed),
            |font| font.color.clone().map(|c| Cow::Owned(Fill::Solid(Some(c)))),
        )
        .map(|fill| {
            fill.map(|f| {
                if *f == Fill::Group {
                    Cow::Owned(Fill::None)
                } else {
                    f
                }
            })
        })
    }

    /// A run's font in one script. A shape's `fontRef` answers with the theme font it
    /// names, unless that is `none`.
    pub fn run_font(
        &self,
        run: Option<&'a CharacterProperties>,
        paragraph: Option<&ParagraphProperties>,
        script: FontScript,
    ) -> Option<TextResolved<RunFont<'a>>> {
        self.run_or_style(
            run,
            paragraph,
            |c| {
                let font = match script {
                    FontScript::Latin => &c.latin,
                    FontScript::EastAsian => &c.east_asian,
                    FontScript::ComplexScript => &c.complex_script,
                };
                font.as_deref().map(RunFont::Stated)
            },
            |font| match font.idx {
                Some(FontCollectionIndex::None) | None => None,
                Some(idx) => Some(RunFont::Theme(idx)),
            },
        )
    }

    /// `@sz`, 1800 if no level says (`MS-OI29500` 2.1.1399 (e)).
    pub fn font_size(
        &self,
        run: Option<&'a CharacterProperties>,
        paragraph: Option<&ParagraphProperties>,
    ) -> TextResolved<i32> {
        self.run(run, paragraph, |c| c.size)
            .unwrap_or(TextResolved::new(1800, TextLevel::SchemaDefault))
    }

    /// `@baseline`, 0 if no level says (`MS-OI29500` 2.1.1399 (d)).
    pub fn baseline(
        &self,
        run: Option<&'a CharacterProperties>,
        paragraph: Option<&ParagraphProperties>,
    ) -> TextResolved<i32> {
        self.run(run, paragraph, |c| c.baseline.as_ref().map(|b| b.value().0))
            .unwrap_or(TextResolved::new(0, TextLevel::SchemaDefault))
    }

    /// `@marL`, 0 if no level says (`MS-OI29500` 2.1.1406 (a)).
    pub fn margin_left(&self, own: Option<&'a ParagraphProperties>) -> TextResolved<i32> {
        self.paragraph(own, |p| p.margin_left)
            .unwrap_or(TextResolved::new(0, TextLevel::SchemaDefault))
    }

    /// `@indent`, 0 if no level says (`MS-OI29500` 2.1.1406 (f)).
    pub fn indent(&self, own: Option<&'a ParagraphProperties>) -> TextResolved<i32> {
        self.paragraph(own, |p| p.indent)
            .unwrap_or(TextResolved::new(0, TextLevel::SchemaDefault))
    }

    /// `@defTabSz`, 914400 if no level says (`MS-OI29500` 2.1.1406 (e)).
    pub fn default_tab_size(&self, own: Option<&'a ParagraphProperties>) -> TextResolved<i64> {
        self.paragraph(own, |p| {
            p.default_tab_size.as_ref().map(|size| size.value().0)
        })
        .unwrap_or(TextResolved::new(914_400, TextLevel::SchemaDefault))
    }

    /// `@eaLnBrk`, true if no level says (`MS-OI29500` 2.1.1406 (c)).
    pub fn east_asian_line_break(
        &self,
        own: Option<&'a ParagraphProperties>,
    ) -> TextResolved<bool> {
        self.paragraph(own, |p| p.east_asian_line_break)
            .unwrap_or(TextResolved::new(true, TextLevel::SchemaDefault))
    }

    /// `@hangingPunct`, true if no level says (`MS-OI29500` 2.1.1406 (d)).
    pub fn hanging_punctuation(&self, own: Option<&'a ParagraphProperties>) -> TextResolved<bool> {
        self.paragraph(own, |p| p.hanging_punctuation)
            .unwrap_or(TextResolved::new(true, TextLevel::SchemaDefault))
    }

    /// `@latinLnBrk`, false if no level says (`MS-OI29500` 2.1.1406 (g)).
    pub fn latin_line_break(&self, own: Option<&'a ParagraphProperties>) -> TextResolved<bool> {
        self.paragraph(own, |p| p.latin_line_break)
            .unwrap_or(TextResolved::new(false, TextLevel::SchemaDefault))
    }
}
