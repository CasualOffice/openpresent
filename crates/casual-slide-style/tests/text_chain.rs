//! BEH-02 for text, as tables: every property family, at every level of the chain, by
//! building the chain from model values, so each level can say or not say each thing.
//! Then the same chain through import, from `fixtures/generated/text-inheritance.pptx`.

use std::path::Path;

use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_model::color::Color;
use casual_slide_model::d3::{Backdrop, FlatText, Point3D, Scene3D};
use casual_slide_model::effect::EffectProperties;
use casual_slide_model::fill::Fill;
use casual_slide_model::geometry::{GeomGuide, PresetTextShape};
use casual_slide_model::line::Line;
use casual_slide_model::presentation::{Coordinate, Percentage, TextFont};
use casual_slide_model::shape::{
    Hyperlink, NodeKind, ObjectId, Placeholder, PlaceholderType, ShapeNode,
};
use casual_slide_model::slide::{CommonSlideData, MasterTextStyles, SlideKind, SlidePart};
use casual_slide_model::text::{
    Autofit, BodyProperties, BooleanElement, Bullet, BulletColor, BulletFont, BulletSize, CapsType,
    CharacterProperties, FontAlign, HorizontalOverflow, Paragraph, ParagraphProperties, StrikeType,
    TabStop, Text3D, TextAlign, TextAnchor, TextBody, TextItem, TextListStyle, TextPoint, TextRun,
    TextSpacing, TextVertical, TextWrap, UnderlineFill, UnderlineLine, UnderlineType,
    VerticalOverflow, Written,
};
use casual_slide_model::theme::{
    FontCollectionIndex, FontRef, ObjectDefaults, ShapeDefinition, ShapeStyle,
};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_style::text::{FontScript, RunFont, TextChain, TextLevel as L, paragraph_level};
use casual_slide_style::{Chain, TextStyle};

/// The levels below a run or a paragraph, nearest first.
const STYLE_LEVELS: [L; 8] = [
    L::ShapeListStyle,
    L::LayoutPlaceholder,
    L::ThemeTextDefault,
    L::ThemeLineDefault,
    L::ThemeShapeDefault,
    L::MasterPlaceholder,
    L::MasterTextStyles,
    L::PresentationDefault,
];

/// One deck's worth of levels, with everything empty until a test says otherwise.
struct Levels {
    slide: SlidePart,
    layout: SlidePart,
    master: SlidePart,
    default: TextListStyle,
    theme: ObjectDefaults,
}

/// One of a short list, whatever level a family's value was made for.
fn pick<T: Copy>(list: &[T], i: i32) -> T {
    list[i as usize % list.len()]
}

fn definition() -> Option<ShapeDefinition> {
    Some(ShapeDefinition {
        body_properties: Some(BodyProperties::default()),
        list_style: Some(TextListStyle::default()),
        ..ShapeDefinition::default()
    })
}

fn body_placeholder(part: u32, node: u32, ph: Placeholder) -> ShapeNode {
    let mut n = ShapeNode::new(ObjectId { part, node }, NodeKind::Shape);
    n.app = Some(casual_slide_model::shape::AppNonVisual {
        placeholder: Some(ph),
        ..Default::default()
    });
    n.text_body = Some(Box::new(TextBody {
        properties: Some(BodyProperties::default()),
        list_style: Some(TextListStyle::default()),
        paragraphs: vec![Paragraph::default()],
    }));
    n
}

fn part_holding(kind: SlideKind, part: u32, shape: ShapeNode) -> SlidePart {
    let mut p = SlidePart::new(ObjectId::part(part), kind);
    let mut root = ShapeNode::new(ObjectId { part, node: 1 }, NodeKind::Group);
    root.children.push(shape);
    p.common = Some(CommonSlideData {
        name: None,
        background: None,
        shape_tree: Some(root),
        controls: Vec::new(),
    });
    p
}

fn levels() -> Levels {
    let ph = |t, idx| Placeholder {
        placeholder_type: Some(t),
        idx: Some(idx),
        ..Placeholder::default()
    };
    let mut master = part_holding(
        SlideKind::Master,
        0,
        body_placeholder(0, 2, ph(PlaceholderType::Body, 1)),
    );
    master.text_styles = Some(MasterTextStyles {
        title: Some(TextListStyle::default()),
        body: Some(TextListStyle::default()),
        other: Some(TextListStyle::default()),
    });
    Levels {
        slide: part_holding(
            SlideKind::Slide,
            2,
            body_placeholder(2, 2, ph(PlaceholderType::Body, 1)),
        ),
        layout: part_holding(
            SlideKind::Layout,
            1,
            body_placeholder(1, 2, ph(PlaceholderType::Body, 1)),
        ),
        master,
        default: TextListStyle::default(),
        theme: ObjectDefaults {
            shape: definition(),
            line: definition(),
            text: definition(),
            ..ObjectDefaults::default()
        },
    }
}

fn shape_mut(part: &mut SlidePart) -> &mut ShapeNode {
    &mut part.shape_tree_mut().unwrap().children[0]
}

fn list_mut(part: &mut SlidePart) -> &mut TextListStyle {
    shape_mut(part)
        .text_body
        .as_mut()
        .unwrap()
        .list_style
        .as_mut()
        .unwrap()
}

impl Levels {
    /// The list style at one of [`STYLE_LEVELS`].
    fn style_mut(&mut self, level: L) -> &mut TextListStyle {
        match level {
            L::ShapeListStyle => list_mut(&mut self.slide),
            L::LayoutPlaceholder => list_mut(&mut self.layout),
            L::MasterPlaceholder => list_mut(&mut self.master),
            L::MasterTextStyles => self
                .master
                .text_styles
                .as_mut()
                .unwrap()
                .body
                .as_mut()
                .unwrap(),
            L::PresentationDefault => &mut self.default,
            L::ThemeTextDefault | L::ThemeLineDefault | L::ThemeShapeDefault => {
                self.theme_definition(level).list_style.as_mut().unwrap()
            }
            other => panic!("{other:?} holds no list style"),
        }
    }

    fn theme_definition(&mut self, level: L) -> &mut ShapeDefinition {
        match level {
            L::ThemeTextDefault => self.theme.text.as_mut().unwrap(),
            L::ThemeLineDefault => self.theme.line.as_mut().unwrap(),
            L::ThemeShapeDefault => self.theme.shape.as_mut().unwrap(),
            other => panic!("{other:?} is no object default"),
        }
    }

    /// The `bodyPr` at one of the levels that holds one.
    fn body_mut(&mut self, level: L) -> &mut BodyProperties {
        let part = match level {
            L::ShapeBody => &mut self.slide,
            L::LayoutPlaceholder => &mut self.layout,
            L::MasterPlaceholder => &mut self.master,
            other => {
                return self
                    .theme_definition(other)
                    .body_properties
                    .as_mut()
                    .unwrap();
            }
        };
        shape_mut(part)
            .text_body
            .as_mut()
            .unwrap()
            .properties
            .as_mut()
            .unwrap()
    }

    fn chain(&self) -> TextChain<'_> {
        let chain = Chain {
            slide: Some(&self.slide),
            layout: Some(&self.layout),
            master: Some(&self.master),
        };
        chain.text_chain(
            &self.slide.shape_tree().unwrap().children[0],
            Some(&self.default),
            Some(&self.theme),
        )
    }
}

/// A paragraph property's setter and reader, with its value made from a number, so that
/// each level's is distinct.
struct ParagraphFamily {
    name: &'static str,
    set: fn(&mut ParagraphProperties, i32),
    get: fn(&ParagraphProperties) -> Option<String>,
}

macro_rules! paragraph_family {
    ($($name:literal, $field:ident, $make:expr;)+) => {
        vec![$(ParagraphFamily {
            name: $name,
            set: |p, i| {
                let make: fn(i32) -> _ = $make;
                p.$field = Some(make(i));
            },
            get: |p| p.$field.as_ref().map(|v| format!("{v:?}")),
        },)+]
    };
}

fn spacing(i: i32) -> TextSpacing {
    TextSpacing::Points(i)
}

fn paragraph_families() -> Vec<ParagraphFamily> {
    paragraph_family!(
        "marL", margin_left, |i| i;
        "marR", margin_right, |i| i;
        "indent", indent, |i| i;
        "algn", align, |i| pick(&[TextAlign::Left, TextAlign::Center, TextAlign::Right, TextAlign::Justified, TextAlign::Distributed, TextAlign::ThaiDistributed, TextAlign::JustifiedLow], i);
        "defTabSz", default_tab_size, |i| Written::new(Coordinate(i64::from(i)));
        "rtl", rtl, |i| i % 2 == 0;
        "eaLnBrk", east_asian_line_break, |i| i % 2 == 0;
        "fontAlgn", font_align, |i| pick(&[FontAlign::Auto, FontAlign::Top, FontAlign::Center, FontAlign::Baseline, FontAlign::Bottom, FontAlign::Auto, FontAlign::Top], i);
        "latinLnBrk", latin_line_break, |i| i % 2 == 0;
        "hangingPunct", hanging_punctuation, |i| i % 2 == 0;
        "lnSpc", line_spacing, spacing;
        "spcBef", space_before, spacing;
        "spcAft", space_after, spacing;
        "buClr", bullet_color, |i| BulletColor::Color(Some(Color::srgb(i as u8, 0, 0)));
        "buSz", bullet_size, |i| BulletSize::Points(1000 + i);
        "buFont", bullet_font, |i| BulletFont::Font(TextFont { typeface: format!("F{i}"), ..TextFont::default() });
        "bullet", bullet, |i| Bullet::Char(Some(format!("{i}")));
        "tabLst", tabs, |i| vec![TabStop { position: Some(Written::new(Coordinate(i64::from(i)))), align: None }];
    )
}

/// Set this family on every level from `first` outwards, and read the nearest back.
#[test]
fn every_paragraph_family_comes_from_the_nearest_level_that_says() {
    for level in [0u8, 3] {
        for family in paragraph_families() {
            for first in 0..=STYLE_LEVELS.len() {
                let mut l = levels();
                for (i, style_level) in STYLE_LEVELS
                    .iter()
                    .enumerate()
                    .skip(first.saturating_sub(1))
                {
                    let mut props = ParagraphProperties::default();
                    (family.set)(&mut props, i as i32 + 1);
                    l.style_mut(*style_level).levels[usize::from(level)] = Some(Box::new(props));
                }
                let mut own = ParagraphProperties {
                    level: Some(level),
                    ..ParagraphProperties::default()
                };
                if first == 0 {
                    (family.set)(&mut own, 0);
                }
                let chain = l.chain();
                let found = chain
                    .paragraph(Some(&own), |p| (family.get)(p))
                    .unwrap_or_else(|| panic!("{}: nothing at level {first}", family.name));
                let (want_from, want_value) = if first == 0 {
                    (L::Paragraph, 0)
                } else {
                    (STYLE_LEVELS[first - 1], first as i32)
                };
                let mut expected = ParagraphProperties::default();
                (family.set)(&mut expected, want_value);
                assert_eq!(
                    (found.from, Some(found.value)),
                    (want_from, (family.get)(&expected)),
                    "{} at paragraph level {level}, first said at {first}",
                    family.name
                );
            }
            // A paragraph at another level sees none of it, and no level means none.
            let mut l = levels();
            let mut props = ParagraphProperties::default();
            (family.set)(&mut props, 1);
            l.style_mut(L::ShapeListStyle).levels[usize::from(level) + 1] = Some(Box::new(props));
            let own = ParagraphProperties {
                level: Some(level),
                ..ParagraphProperties::default()
            };
            assert_eq!(
                l.chain().paragraph(Some(&own), |p| (family.get)(p)),
                None,
                "{}: another level's value leaked",
                family.name
            );
        }
    }
}

struct RunFamily {
    name: &'static str,
    set: fn(&mut CharacterProperties, i32),
    get: fn(&CharacterProperties) -> Option<String>,
}

macro_rules! run_family {
    ($($name:literal, $field:ident, $make:expr;)+) => {
        vec![$(RunFamily {
            name: $name,
            set: |c, i| {
                let make: fn(i32) -> _ = $make;
                c.$field = Some(make(i));
            },
            get: |c| c.$field.as_ref().map(|v| format!("{v:?}")),
        },)+]
    };
}

fn font(i: i32) -> Box<TextFont> {
    Box::new(TextFont {
        typeface: format!("F{i}"),
        ..TextFont::default()
    })
}

fn link(i: i32) -> Box<Hyperlink> {
    Box::new(Hyperlink {
        tooltip: Some(format!("{i}")),
        ..Hyperlink::default()
    })
}

fn run_families() -> Vec<RunFamily> {
    const UNDERLINES: [UnderlineType; 6] = [
        UnderlineType::None,
        UnderlineType::Sng,
        UnderlineType::Dbl,
        UnderlineType::Heavy,
        UnderlineType::Words,
        UnderlineType::Wavy,
    ];
    const STRIKES: [StrikeType; 3] = [
        StrikeType::NoStrike,
        StrikeType::SngStrike,
        StrikeType::DblStrike,
    ];
    const CAPS: [CapsType; 3] = [CapsType::None, CapsType::Small, CapsType::All];
    run_family!(
        "kumimoji", kumimoji, |i| i % 2 == 0;
        "lang", lang, |i| format!("l{i}");
        "altLang", alt_lang, |i| format!("a{i}");
        "sz", size, |i| 1000 + i;
        "b", bold, |i| i % 2 == 0;
        "i", italic, |i| i % 2 == 0;
        "u", underline, |i| UNDERLINES[i as usize % 6];
        "strike", strike, |i| STRIKES[i as usize % 3];
        "kern", kerning, |i| i;
        "cap", caps, |i| CAPS[i as usize % 3];
        "spc", spacing, |i| Written::new(TextPoint(i));
        "normalizeH", normalize_height, |i| i % 2 == 0;
        "baseline", baseline, |i| Written::new(Percentage(i));
        "noProof", no_proof, |i| i % 2 == 0;
        "dirty", dirty, |i| i % 2 == 0;
        "err", error, |i| i % 2 == 0;
        "smtClean", smart_tag_clean, |i| i % 2 == 0;
        "smtId", smart_tag_id, |i| i as u32;
        "bmk", bookmark, |i| format!("b{i}");
        "ln", line, |i| Box::new(Line { width: Some(i), ..Line::default() });
        "fill", fill, |i| Fill::Solid(Some(Color::srgb(i as u8, 0, 0)));
        "effects", effects, |i| Box::new(EffectProperties::List(vec![casual_slide_model::effect::Effect::Reference(Some(format!("{i}")))]));
        "highlight", highlight, |i| Some(Color::srgb(0, i as u8, 0));
        "uLn", underline_line, |i| UnderlineLine::Line(Box::new(Line { width: Some(i), ..Line::default() }));
        "uFill", underline_fill, |i| UnderlineFill::Fill(Some(Fill::Solid(Some(Color::srgb(0, 0, i as u8)))));
        "latin", latin, font;
        "ea", east_asian, font;
        "cs", complex_script, font;
        "sym", symbol, font;
        "hlinkClick", hyperlink_click, link;
        "hlinkMouseOver", hyperlink_mouse_over, link;
        "rtl", rtl, |i| BooleanElement { value: Some(i % 2 == 0) };
    )
}

#[test]
fn every_run_family_comes_from_the_nearest_level_that_says() {
    for level in [0u8, 4] {
        for family in run_families() {
            for first in 0..=STYLE_LEVELS.len() {
                let mut l = levels();
                for (i, style_level) in STYLE_LEVELS
                    .iter()
                    .enumerate()
                    .skip(first.saturating_sub(1))
                {
                    let mut c = CharacterProperties::default();
                    (family.set)(&mut c, i as i32 + 1);
                    l.style_mut(*style_level).levels[usize::from(level)] =
                        Some(Box::new(ParagraphProperties {
                            default_run: Some(Box::new(c)),
                            ..ParagraphProperties::default()
                        }));
                }
                let mut run = CharacterProperties::default();
                if first == 0 {
                    (family.set)(&mut run, 0);
                }
                let paragraph = ParagraphProperties {
                    level: Some(level),
                    ..ParagraphProperties::default()
                };
                let chain = l.chain();
                let found = chain
                    .run(Some(&run), Some(&paragraph), |c| (family.get)(c))
                    .unwrap_or_else(|| panic!("{}: nothing at level {first}", family.name));
                let (want_from, want_value) = if first == 0 {
                    (L::Run, 0)
                } else {
                    (STYLE_LEVELS[first - 1], first as i32)
                };
                let mut expected = CharacterProperties::default();
                (family.set)(&mut expected, want_value);
                assert_eq!(
                    (found.from, Some(found.value)),
                    (want_from, (family.get)(&expected)),
                    "{} at paragraph level {level}, first said at {first}",
                    family.name
                );
            }
        }
    }
}

struct BodyFamily {
    name: &'static str,
    set: fn(&mut BodyProperties, i32),
    get: fn(&BodyProperties) -> Option<String>,
}

macro_rules! body_family {
    ($($name:literal, $field:ident, $make:expr;)+) => {
        vec![$(BodyFamily {
            name: $name,
            set: |b, i| {
                let make: fn(i32) -> _ = $make;
                b.$field = Some(make(i));
            },
            get: |b| b.$field.as_ref().map(|v| format!("{v:?}")),
        },)+]
    };
}

fn inset(i: i32) -> Written<Coordinate> {
    Written::new(Coordinate(i64::from(i)))
}

fn body_families() -> Vec<BodyFamily> {
    body_family!(
        "rot", rotation, |i| i;
        "spcFirstLastPara", space_first_last_paragraph, |i| i % 2 == 0;
        "vertOverflow", vertical_overflow, |i| pick(&[VerticalOverflow::Overflow, VerticalOverflow::Ellipsis, VerticalOverflow::Clip], i);
        "horzOverflow", horizontal_overflow, |i| pick(&[HorizontalOverflow::Overflow, HorizontalOverflow::Clip, HorizontalOverflow::Overflow], i);
        "vert", vertical, |i| pick(&[TextVertical::Horz, TextVertical::Vert, TextVertical::EaVert], i);
        "wrap", wrap, |i| pick(&[TextWrap::None, TextWrap::Square, TextWrap::None], i);
        "lIns", inset_left, inset;
        "tIns", inset_top, inset;
        "rIns", inset_right, inset;
        "bIns", inset_bottom, inset;
        "numCol", columns, |i| i as u8 + 1;
        "spcCol", column_spacing, |i| i;
        "rtlCol", rtl_columns, |i| i % 2 == 0;
        "fromWordArt", from_word_art, |i| i % 2 == 0;
        "anchor", anchor, |i| pick(&[TextAnchor::Top, TextAnchor::Center, TextAnchor::Bottom], i);
        "anchorCtr", anchor_center, |i| i % 2 == 0;
        "forceAA", force_antialias, |i| i % 2 == 0;
        "upright", upright, |i| i % 2 == 0;
        "compatLnSpc", compat_line_spacing, |i| i % 2 == 0;
        "prstTxWarp", warp, |i| PresetTextShape { preset: None, adjust: Some(vec![GeomGuide { name: Some(format!("a{i}")), formula: None }]) };
        "scene3d", scene3d, |i| Box::new(Scene3D { backdrop: Some(Backdrop { anchor: Some(Point3D { x: Some(Coordinate(i64::from(i))), ..Point3D::default() }), ..Backdrop::default() }), ..Scene3D::default() });
        "text3d", text_3d, |i| Text3D::Flat(FlatText { z: Some(Coordinate(i64::from(i))) });
        "autofit", autofit, |i| match i { 0 => Autofit::None, 1 => Autofit::Shape, _ => Autofit::Normal { font_scale: None, line_spacing_reduction: None } };
    )
}

#[test]
fn every_body_family_comes_from_the_shape_its_placeholders_or_the_themes_defaults() {
    let places = [
        L::ShapeBody,
        L::LayoutPlaceholder,
        L::ThemeTextDefault,
        L::ThemeLineDefault,
        L::ThemeShapeDefault,
        L::MasterPlaceholder,
    ];
    for family in body_families() {
        for first in 0..places.len() {
            let mut l = levels();
            for (i, place) in places.iter().enumerate().skip(first) {
                (family.set)(l.body_mut(*place), i as i32);
            }
            let own = l.slide.shape_tree().unwrap().children[0]
                .text_body
                .as_ref()
                .unwrap()
                .properties
                .as_ref()
                .unwrap()
                .clone();
            let chain = l.chain();
            let found = chain
                .body(Some(&own), |b| (family.get)(b))
                .unwrap_or_else(|| panic!("{}: nothing", family.name));
            let mut expected = BodyProperties::default();
            (family.set)(&mut expected, first as i32);
            assert_eq!(
                (found.from, Some(found.value)),
                (places[first], (family.get)(&expected)),
                "{} first said at {first}",
                family.name
            );
        }
        let l = levels();
        assert_eq!(
            l.chain()
                .body(Some(&BodyProperties::default()), |b| (family.get)(b)),
            None,
            "{}: the master text styles hold no body properties",
            family.name
        );
    }
}

/// A choice is one property: a nearer gradient hides a further solid colour, and a
/// nearer `buNone` hides a further character.
#[test]
fn a_choice_family_is_one_unit() {
    let mut l = levels();
    let run_with = |fill| ParagraphProperties {
        default_run: Some(Box::new(CharacterProperties {
            fill: Some(fill),
            ..CharacterProperties::default()
        })),
        ..ParagraphProperties::default()
    };
    l.style_mut(L::MasterTextStyles).levels[0] =
        Some(Box::new(run_with(Fill::Solid(Some(Color::srgb(1, 2, 3))))));
    l.style_mut(L::LayoutPlaceholder).levels[0] =
        Some(Box::new(run_with(Fill::Gradient(Box::default()))));
    let chain = l.chain();
    let found = chain.run(None, None, |c| c.fill.as_ref()).unwrap();
    assert!(matches!(found.value, Fill::Gradient(_)));
    assert_eq!(found.from, L::LayoutPlaceholder);

    let mut l = levels();
    l.style_mut(L::MasterTextStyles).levels[0] = Some(Box::new(ParagraphProperties {
        bullet: Some(Bullet::Char(Some("x".to_owned()))),
        ..ParagraphProperties::default()
    }));
    let own = ParagraphProperties {
        bullet: Some(Bullet::None),
        ..ParagraphProperties::default()
    };
    let chain = l.chain();
    assert_eq!(
        chain
            .paragraph(Some(&own), |p| p.bullet.as_ref())
            .unwrap()
            .value,
        &Bullet::None
    );
}

/// `MS-OI29500` 2.1.1298: `grpFill` in text is `noFill`.
#[test]
fn a_group_fill_on_a_run_is_no_fill() {
    let l = levels();
    let run = CharacterProperties {
        fill: Some(Fill::Group),
        ..CharacterProperties::default()
    };
    let chain = l.chain();
    assert_eq!(*chain.run_fill(Some(&run), None).unwrap().value, Fill::None);
    let solid = CharacterProperties {
        fill: Some(Fill::Solid(None)),
        ..CharacterProperties::default()
    };
    assert_eq!(
        *chain.run_fill(Some(&solid), None).unwrap().value,
        Fill::Solid(None)
    );
}

/// BEH-02, `p:style/fontRef` (`MS-OI29500` 20.1.2.2.37): who answers a run's colour and
/// font, with each level saying or not.
#[test]
fn a_shapes_font_reference_sits_under_its_own_text_body_and_over_the_rest() {
    let color = |i| Fill::Solid(Some(Color::srgb(i, 0, 0)));
    let says = |i| CharacterProperties {
        fill: Some(color(i)),
        latin: Some(font(i32::from(i))),
        ..CharacterProperties::default()
    };
    struct Case {
        what: &'static str,
        kind: NodeKind,
        run: bool,
        shape_list: bool,
        font_ref: Option<(u8, FontCollectionIndex)>,
        layout: bool,
        theme: bool,
        fill_from: Option<L>,
        font_from: Option<L>,
    }
    let base = || Case {
        what: "",
        kind: NodeKind::Shape,
        run: false,
        shape_list: false,
        font_ref: Some((9, FontCollectionIndex::Minor)),
        layout: true,
        theme: true,
        fill_from: Some(L::ShapeStyle),
        font_from: Some(L::ShapeStyle),
    };
    let cases = [
        Case {
            what: "the run says",
            run: true,
            fill_from: Some(L::Run),
            font_from: Some(L::Run),
            ..base()
        },
        Case {
            what: "the shape's list style says",
            shape_list: true,
            fill_from: Some(L::ShapeListStyle),
            font_from: Some(L::ShapeListStyle),
            ..base()
        },
        Case {
            what: "only the reference, over the layout and the theme",
            ..base()
        },
        Case {
            what: "a reference with idx none gives no font, and still a colour",
            font_ref: Some((9, FontCollectionIndex::None)),
            font_from: Some(L::LayoutPlaceholder),
            ..base()
        },
        Case {
            what: "no reference: the layout",
            font_ref: None,
            fill_from: Some(L::LayoutPlaceholder),
            font_from: Some(L::LayoutPlaceholder),
            ..base()
        },
        Case {
            what: "no reference, no layout: the theme's defaults",
            font_ref: None,
            layout: false,
            fill_from: Some(L::ThemeTextDefault),
            font_from: Some(L::ThemeTextDefault),
            ..base()
        },
        Case {
            what: "a connector's reference is not used",
            kind: NodeKind::Connector,
            layout: false,
            theme: false,
            fill_from: None,
            font_from: None,
            ..base()
        },
        Case {
            what: "a picture's reference is not used",
            kind: NodeKind::Picture,
            layout: false,
            theme: false,
            fill_from: None,
            font_from: None,
            ..base()
        },
    ];
    for case in cases {
        let mut l = levels();
        let at = |list: &mut TextListStyle, i| {
            list.levels[0] = Some(Box::new(ParagraphProperties {
                default_run: Some(Box::new(says(i))),
                ..ParagraphProperties::default()
            }));
        };
        if case.shape_list {
            at(l.style_mut(L::ShapeListStyle), 1);
        }
        if case.layout {
            at(l.style_mut(L::LayoutPlaceholder), 2);
        }
        if case.theme {
            at(l.style_mut(L::ThemeTextDefault), 3);
        }
        let shape = shape_mut(&mut l.slide);
        shape.kind = case.kind;
        shape.style = case.font_ref.map(|(i, idx)| {
            Box::new(ShapeStyle {
                font: Some(FontRef {
                    idx: Some(idx),
                    color: Some(Color::srgb(i, 0, 0)),
                }),
                ..ShapeStyle::default()
            })
        });
        let run = says(0);
        let run = case.run.then_some(&run);
        let chain = l.chain();
        assert_eq!(
            chain.run_fill(run, None).map(|f| f.from),
            case.fill_from,
            "fill: {}",
            case.what
        );
        assert_eq!(
            chain.run_font(run, None, FontScript::Latin).map(|f| f.from),
            case.font_from,
            "font: {}",
            case.what
        );
        if case.fill_from == Some(L::ShapeStyle) {
            assert_eq!(*chain.run_fill(run, None).unwrap().value, color(9));
        }
        if case.font_from == Some(L::ShapeStyle) {
            assert_eq!(
                chain.run_font(run, None, FontScript::Latin).unwrap().value,
                RunFont::Theme(FontCollectionIndex::Minor)
            );
        }
    }
}

/// `MS-OI29500` 2.1.1383: Office ignores `defPPr`. So a `defPPr` at the nearest level
/// says nothing, even with nothing further to say.
#[test]
fn def_ppr_takes_no_part() {
    let mut l = levels();
    for level in STYLE_LEVELS {
        l.style_mut(level).default = Some(Box::new(ParagraphProperties {
            align: Some(TextAlign::Right),
            default_run: Some(Box::new(CharacterProperties {
                size: Some(9000),
                ..CharacterProperties::default()
            })),
            ..ParagraphProperties::default()
        }));
    }
    let chain = l.chain();
    assert_eq!(chain.paragraph(None, |p| p.align), None);
    assert_eq!(chain.run(None, None, |c| c.size), None);
}

/// Inferred from `MS-OI29500` 2.1.1398 and 2.1.1399, whose order names only `lstStyle`
/// ancestors: a paragraph's own `defRPr` is not in a run's chain.
#[test]
fn a_paragraph_default_run_takes_no_part() {
    let l = levels();
    let own = ParagraphProperties {
        default_run: Some(Box::new(CharacterProperties {
            size: Some(9000),
            ..CharacterProperties::default()
        })),
        ..ParagraphProperties::default()
    };
    assert_eq!(l.chain().run(None, Some(&own), |c| c.size), None);
}

/// `MS-OI29500` 2.1.1406 (b): `lvl` inside a level's own properties is not used.
#[test]
fn lvl_inside_a_level_says_nothing() {
    let mut l = levels();
    l.style_mut(L::ShapeListStyle).levels[0] = Some(Box::new(ParagraphProperties {
        level: Some(5),
        margin_left: Some(1),
        ..ParagraphProperties::default()
    }));
    let chain = l.chain();
    assert_eq!(chain.paragraph(None, |p| p.margin_left).unwrap().value, 1);
    assert_eq!(paragraph_level(None), 0);
    let own = ParagraphProperties {
        level: Some(5),
        ..ParagraphProperties::default()
    };
    assert_eq!(chain.paragraph(Some(&own), |p| p.margin_left), None);
}

/// Each default below is from `MS-OI29500`, and comes only after every level was asked.
#[test]
fn documented_defaults_come_last_and_say_so() {
    let l = levels();
    let chain = l.chain();
    let d = L::SchemaDefault;
    assert_eq!(
        (
            chain.font_size(None, None).value,
            chain.font_size(None, None).from
        ),
        (1800, d)
    );
    assert_eq!(
        (
            chain.baseline(None, None).value,
            chain.baseline(None, None).from
        ),
        (0, d)
    );
    assert_eq!(
        (chain.margin_left(None).value, chain.margin_left(None).from),
        (0, d)
    );
    assert_eq!((chain.indent(None).value, chain.indent(None).from), (0, d));
    assert_eq!(
        (
            chain.default_tab_size(None).value,
            chain.default_tab_size(None).from
        ),
        (914_400, d)
    );
    assert_eq!(
        (
            chain.east_asian_line_break(None).value,
            chain.hanging_punctuation(None).value,
            chain.latin_line_break(None).value
        ),
        (true, true, false)
    );
    let mut l = levels();
    l.style_mut(L::PresentationDefault).levels[0] = Some(Box::new(ParagraphProperties {
        margin_left: Some(7),
        hanging_punctuation: Some(false),
        default_run: Some(Box::new(CharacterProperties {
            size: Some(1234),
            baseline: Some(Written::new(Percentage(5))),
            ..CharacterProperties::default()
        })),
        ..ParagraphProperties::default()
    }));
    let chain = l.chain();
    let from = L::PresentationDefault;
    assert_eq!(
        (
            chain.font_size(None, None).value,
            chain.font_size(None, None).from
        ),
        (1234, from)
    );
    assert_eq!(chain.baseline(None, None).value, 5);
    assert_eq!(chain.margin_left(None).value, 7);
    assert!(!chain.hanging_punctuation(None).value);
}

fn fixture_deck() -> (casual_slide_model::slide::SlideParts, TextListStyle) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated/text-inheritance.pptx");
    let mut pkg =
        PresentationPackage::open(std::fs::read(path).unwrap(), OoxmlLimits::default()).unwrap();
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    let parts = import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap()
    .parts;
    (parts, deck.presentation.default_text_style.unwrap())
}

fn named<'a>(part: &'a SlidePart, name: &str) -> &'a ShapeNode {
    part.shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some(name))
        .unwrap()
}

/// The whole chain, from a deck, by what each level says in the fixture.
#[test]
fn the_chain_through_import_gives_each_level_its_own_value() {
    let (parts, default) = fixture_deck();
    let ordinal = parts.structure.slide(0).unwrap();
    let c = Chain::of(&parts, ordinal);
    let slide = parts.get(ordinal).unwrap();
    let shape = named(slide, "Inheriting");
    let text = c.text_chain(shape, Some(&default), None);
    assert_eq!(text.list_style_count(), 5);
    let body = shape.text_body.as_ref().unwrap();
    let props = |at: usize| body.paragraphs[at].properties.as_deref();
    let run = |at: usize, item: usize| -> Option<&CharacterProperties> {
        let TextItem::Run(TextRun { properties, .. }) = &body.paragraphs[at].items[item] else {
            panic!("not a run");
        };
        properties.as_deref()
    };

    // Run, then each level, for size.
    let size = text.font_size(run(0, 0), None);
    assert_eq!((size.value, size.from), (6000, L::Run));
    let size = text.font_size(run(0, 1), None);
    assert_eq!((size.value, size.from), (5000, L::ShapeListStyle));
    let size = text.font_size(run(1, 0), props(1));
    assert_eq!(
        (size.value, size.from),
        (5001, L::ShapeListStyle),
        "level 2"
    );
    // `pPr/defRPr` says 7000, and is not asked.
    let size = text.font_size(run(2, 0), props(2));
    assert_eq!((size.value, size.from), (5000, L::ShapeListStyle));

    // One property nothing nearer says, per level, at levels 1 and 2.
    let cap = text.run(None, None, |c| c.caps).unwrap();
    assert_eq!((cap.value, cap.from), (CapsType::All, L::LayoutPlaceholder));
    let cap = text.run(None, props(1), |c| c.caps).unwrap();
    assert_eq!(
        (cap.value, cap.from),
        (CapsType::Small, L::LayoutPlaceholder)
    );
    let spc = text.run(None, None, |c| c.spacing.as_ref()).unwrap();
    assert_eq!((spc.value.value().0, spc.from), (1, L::MasterPlaceholder));
    let i = text.run(None, None, |c| c.italic).unwrap();
    assert_eq!((i.value, i.from), (true, L::MasterTextStyles));
    let i = text.run(None, props(1), |c| c.italic).unwrap();
    assert_eq!((i.value, i.from), (false, L::MasterTextStyles));
    let kern = text.run(None, None, |c| c.kerning).unwrap();
    assert_eq!((kern.value, kern.from), (1, L::PresentationDefault));
    // `titleStyle` and `otherStyle` say these, and are not this shape's.
    assert_eq!(text.run(None, None, |c| c.bold), None);
    assert_eq!(text.run(None, None, |c| c.underline), None);
    // `defPPr` in the default says `thaiDist`, and is not asked.
    assert_eq!(text.paragraph(None, |p| p.align), None);
    let align = text.paragraph(props(1), |p| p.align).unwrap();
    assert_eq!((align.value, align.from), (TextAlign::Right, L::Paragraph));

    let paragraph =
        |get: fn(&ParagraphProperties) -> Option<i64>| text.paragraph(None, get).unwrap();
    let got = paragraph(|p| p.indent.map(i64::from));
    assert_eq!((got.value, got.from), (1, L::PresentationDefault));
    let got = paragraph(|p| p.margin_right.map(i64::from));
    assert_eq!((got.value, got.from), (3, L::MasterTextStyles));
    let got = paragraph(|p| p.margin_left.map(i64::from));
    assert_eq!((got.value, got.from), (1, L::MasterPlaceholder));
    let got = paragraph(|p| p.default_tab_size.as_ref().map(|t| t.value().0));
    assert_eq!((got.value, got.from), (1, L::LayoutPlaceholder));
    let align = text.paragraph(None, |p| p.font_align).unwrap();
    assert_eq!(
        (align.value, align.from),
        (FontAlign::Top, L::ShapeListStyle)
    );

    // Body properties: own, layout, master, nothing further.
    let own = body.properties.as_ref();
    let got = text.body(own, |b| b.inset_right.as_ref()).unwrap();
    assert_eq!((got.value.value().0, got.from), (33, L::ShapeBody));
    let got = text.body(own, |b| b.inset_top.as_ref()).unwrap();
    assert_eq!((got.value.value().0, got.from), (22, L::LayoutPlaceholder));
    let got = text.body(own, |b| b.anchor).unwrap();
    assert_eq!(
        (got.value, got.from),
        (TextAnchor::Center, L::LayoutPlaceholder)
    );
    let got = text.body(own, |b| b.inset_left.as_ref()).unwrap();
    assert_eq!((got.value.value().0, got.from), (11, L::MasterPlaceholder));
    let got = text.body(own, |b| b.wrap).unwrap();
    assert_eq!(
        (got.value, got.from),
        (TextWrap::None, L::MasterPlaceholder)
    );
    assert_eq!(text.body(own, |b| b.columns), None);

    // A shape with no placeholder skips both placeholders, and takes `otherStyle`.
    let free = named(slide, "Free Text");
    assert_eq!(c.text_style(free).value, TextStyle::Other);
    let text = c.text_chain(free, Some(&default), None);
    assert_eq!(text.list_style_count(), 2);
    let u = text.run(None, None, |c| c.underline).unwrap();
    assert_eq!((u.value, u.from), (UnderlineType::Sng, L::MasterTextStyles));
    assert_eq!(text.font_size(None, None).value, 2200);
    assert_eq!(text.run(None, None, |c| c.caps), None);
    assert_eq!(text.body(None, |b| b.anchor), None);

    // A bare placeholder, with neither `lstStyle` nor own values: the layout's first.
    let bare = named(slide, "Bare Placeholder");
    let text = c.text_chain(bare, Some(&default), None);
    assert_eq!(text.list_style_count(), 4);
    let size = text.font_size(None, None);
    assert_eq!((size.value, size.from), (4000, L::LayoutPlaceholder));

    // Asked of the layout itself, it is at the shape's level and its master's next.
    let layout_part = c.layout.unwrap();
    let on_layout = Chain {
        slide: None,
        layout: c.layout,
        master: c.master,
    };
    let held = named(layout_part, "Layout Body");
    let text = on_layout.text_chain(held, Some(&default), None);
    let size = text.font_size(None, None);
    assert_eq!((size.value, size.from), (4000, L::ShapeListStyle));
    let spc = text.run(None, None, |c| c.spacing.as_ref()).unwrap();
    assert_eq!(spc.from, L::MasterPlaceholder);
    // And of the master: its placeholder's style, then the styles, then the default.
    let on_master = Chain {
        slide: None,
        layout: None,
        master: c.master,
    };
    let held = named(c.master.unwrap(), "Master Body");
    let text = on_master.text_chain(held, Some(&default), None);
    let size = text.font_size(None, None);
    assert_eq!((size.value, size.from), (3000, L::ShapeListStyle));
    let i = text.run(None, None, |c| c.italic).unwrap();
    assert_eq!(i.from, L::MasterTextStyles);
    let kern = text.run(None, None, |c| c.kerning).unwrap();
    assert_eq!(kern.from, L::PresentationDefault);
    // No default given, and the chain ends sooner, with no panic.
    let text = on_master.text_chain(held, None, None);
    assert_eq!(text.run(None, None, |c| c.kerning), None);
}

/// A title takes `titleStyle`, a footer `otherStyle`, and without a master style, the
/// presentation default is all there is. This was `TextStyle::PresentationDefault`'s gap.
#[test]
fn the_list_style_chosen_is_the_one_in_force() {
    let (parts, default) = fixture_deck();
    let ordinal = parts.structure.slide(0).unwrap();
    let c = Chain::of(&parts, ordinal);
    let styles = c.master.unwrap().text_styles.as_ref().unwrap();
    assert_eq!(c.master_list_style(TextStyle::Title), styles.title.as_ref());
    assert_eq!(c.master_list_style(TextStyle::Body), styles.body.as_ref());
    assert_eq!(c.master_list_style(TextStyle::Other), styles.other.as_ref());
    assert_eq!(c.master_list_style(TextStyle::Notes), None);
    assert_eq!(c.master_list_style(TextStyle::PresentationDefault), None);

    let free = named(parts.get(ordinal).unwrap(), "Free Text");
    let mut master = c.master.unwrap().clone();
    master.text_styles.as_mut().unwrap().other = None;
    let without = Chain {
        slide: c.slide,
        layout: c.layout,
        master: Some(&master),
    };
    assert_eq!(
        without.text_style(free).value,
        TextStyle::PresentationDefault
    );
    let text = without.text_chain(free, Some(&default), None);
    assert_eq!(text.list_style_count(), 1);
    let size = text.font_size(None, None);
    assert_eq!((size.value, size.from), (1000, L::PresentationDefault));
}

/// The chain through import, from a theme's object defaults and a shape's `p:style`, in
/// `fixtures/generated/text-object-defaults.pptx`. The theme's `txDef` is what the shape's
/// own part cannot answer; without the theme or the style the answer differs.
#[test]
fn object_defaults_and_font_references_answer_through_import() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    let bytes = std::fs::read(root.join("text-object-defaults.pptx")).unwrap();
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    let theme_xml = pkg.read_part("ppt/theme/theme1.xml").unwrap();
    let xml = OoxmlLimits::default();
    let tree =
        casual_slide_ooxml::XmlTree::parse(&theme_xml, "ppt/theme/theme1.xml", &xml).unwrap();
    let theme = casual_slide_import::theme::import_theme(&tree).unwrap();
    assert!(theme.findings.items.is_empty(), "{:?}", theme.findings);
    let defaults = theme.value.object_defaults.as_ref().unwrap();
    let presentation =
        casual_slide_import::presentation::import_deck(&mut pkg, &ImportLimits::default())
            .unwrap()
            .deck
            .presentation;
    let parts = import_slide_parts(
        &mut pkg,
        &presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap()
    .parts;
    let ordinal = parts.structure.slide(0).unwrap();
    let c = Chain::of(&parts, ordinal);
    let slide = parts.get(ordinal).unwrap();

    let styled = named(slide, "Styled");
    let text = c.text_chain(styled, None, Some(defaults));
    let own = styled.text_body.as_ref().unwrap();
    let para = own.paragraphs.first().unwrap();
    let pp = para.properties.as_deref();
    assert_eq!(text.run_fill(None, pp).unwrap().from, L::ShapeStyle);
    assert_eq!(
        text.run_font(None, pp, FontScript::Latin).unwrap().value,
        RunFont::Theme(FontCollectionIndex::Major)
    );
    assert_eq!(text.font_size(None, pp).value, 2100, "txDef's defRPr");
    assert_eq!(text.font_size(None, pp).from, L::ThemeTextDefault);
    assert_eq!(
        text.body(own.properties.as_ref(), |b| b.columns).unwrap(),
        casual_slide_style::text::TextResolved {
            value: 3,
            from: L::ThemeLineDefault
        },
        "lnDef's bodyPr: txDef does not say"
    );
    assert_eq!(
        text.body(own.properties.as_ref(), |b| b.rotation)
            .unwrap()
            .from,
        L::ThemeShapeDefault,
        "spDef's bodyPr: neither txDef nor lnDef says"
    );

    let line = named(slide, "Connector");
    let text = c.text_chain(line, None, None);
    assert_eq!(
        text.run_fill(None, None),
        None,
        "`MS-OI29500` on 20.1.2.2.37 (a)"
    );

    let bare = named(slide, "No style");
    let with = c.text_chain(bare, None, Some(defaults));
    let without = c.text_chain(bare, None, None);
    assert_eq!(with.font_size(None, None).value, 2100);
    assert_eq!(without.font_size(None, None).from, L::SchemaDefault);
}
