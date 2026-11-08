//! Tier 1F export: edit a run's text and a property, save, reopen; only those nodes
//! changed and every other part is byte-identical. Every row and enum value of TEXT is
//! written; real decks are a fixed point; edited output validates; hostile models are
//! refused with nothing saved.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use casual_slide_export::presentation::{patch_presentation, save_deck};
use casual_slide_export::slide::{patch_slide_part, save_slide_parts};
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::presentation::{import_deck, import_presentation};
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_part, import_slide_parts};
use casual_slide_model::color::Color;
use casual_slide_model::d3::{
    Camera, FlatText, LightRig, LightRigDirection, LightRigType, PresetCamera, Scene3D,
};
use casual_slide_model::effect::{Effect, EffectProperties};
use casual_slide_model::fill::{Fill, GradientFill};
use casual_slide_model::line::Line;
use casual_slide_model::shape::ShapeNode;
use casual_slide_model::slide::{SlideKind, SlidePart, SlideParts};
use casual_slide_model::text::{
    Autofit, BodyProperties, CharacterProperties, Paragraph, Text3D, TextListStyle, TextRun,
    UnderlineLine, Written,
};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, Relationship, TargetMode, XmlTree};

include!("../../casual-slide-import/tests/text_tables.inc");

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn model(bytes: &[u8]) -> SlideParts {
    let mut pkg = open(bytes.to_vec());
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap()
    .parts
}

fn parts(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut pkg = open(bytes.to_vec());
    pkg.part_names()
        .into_iter()
        .map(|n| {
            let data = pkg.read_part(&n).unwrap();
            (n, data)
        })
        .collect()
}

fn named_mut<'a>(part: &'a mut SlidePart, name: &str) -> &'a mut ShapeNode {
    let id = part
        .shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some(name))
        .unwrap_or_else(|| panic!("no {name}"))
        .id;
    part.node_mut(id).unwrap()
}

fn all_decks() -> Vec<String> {
    let mut out = Vec::new();
    for set in ["generated", "producers", "real"] {
        let mut names: Vec<String> = std::fs::read_dir(fixtures().join(set))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| !p.extension().is_some_and(|e| e == "tsv" || e == "md"))
            .map(|p| format!("{set}/{}", p.file_name().unwrap().to_string_lossy()))
            .collect();
        names.sort();
        out.extend(names);
    }
    out
}

fn body_rels() -> Vec<Relationship> {
    let make = |id: &str, kind: &str, target: &str, mode| Relationship {
        id: id.to_owned(),
        rel_type: format!("{NS_R}/{kind}"),
        target: target.to_owned(),
        mode,
    };
    vec![
        make(
            "rId2",
            "hyperlink",
            "https://example.invalid/",
            TargetMode::External,
        ),
        make("rId3", "image", "../media/image1.png", TargetMode::Internal),
    ]
}

fn read_part(xml: &[u8]) -> SlidePart {
    let tree = XmlTree::parse(xml, "s.xml", &OoxmlLimits::default()).unwrap();
    let limits = ImportLimits::default();
    let mut findings = Findings::new(limits.max_findings);
    let mut color = ColorFindings::default();
    import_slide_part(
        &tree,
        "s.xml",
        &body_rels(),
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &limits,
        &mut findings,
        &mut color,
    )
    .unwrap()
}

/// Patch a slide holding `from` to the model of `to`, returning the bytes or the code.
fn patch_between(from: &str, to: &SlidePart) -> Result<Vec<u8>, String> {
    let xml = slide_with_body(from);
    let mut tree = XmlTree::parse(xml.as_bytes(), "s.xml", &OoxmlLimits::default()).unwrap();
    patch_slide_part(
        &mut tree,
        "s.xml",
        false,
        &body_rels(),
        to,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &OoxmlLimits::default(),
    )
    .map_err(|e| e.code().to_owned())?;
    Ok(tree.to_bytes())
}

#[test]
fn every_row_of_the_text_area_is_written_or_refused_or_left_alone() {
    let mut written = 0;
    for id in property_rows() {
        let Some(row) = text_row(&id) else { continue };
        let base = read_part(slide_with_body(&row.base).as_bytes());
        let with = read_part(slide_with_body(&row.with).as_bytes());

        // Adding it: the patch reads back as asked, by `patch_slide_part`'s own check
        // and again here, in a second place.
        match (row.outcome, patch_between(&row.base, &with)) {
            (Outcome::Modelled, Ok(bytes)) => {
                assert_eq!(read_part(&bytes), with, "{id}");
                written += 1;
            }
            (Outcome::Reported, Ok(bytes)) => {
                assert_eq!(
                    bytes,
                    slide_with_body(&row.base).into_bytes(),
                    "{id}: nothing in the model, so nothing to write"
                );
            }
            (outcome, other) => panic!("{id}: {outcome:?} gave {other:?}"),
        }

        // Taking it away: back to the base, or refused where the schema requires it.
        let required = matches!(
            id.as_str(),
            "CT_TextBody/bodyPr" | "CT_TextBody/p" | "CT_RegularTextRun/t" | "CT_TextField/@id"
        );
        match patch_between(&row.with, &base) {
            Ok(bytes) => {
                assert!(!required, "{id}: required, and removed");
                assert_eq!(read_part(&bytes), base, "{id}");
            }
            Err(code) => {
                assert!(required, "{id}: refused with {code}");
                assert_eq!(code, "CS-EXP-0009", "{id}");
            }
        }
    }
    assert!(written >= 110, "{written} rows written");
}

/// `uLn` is `a:CT_LineProperties` under another name, so a new child goes in at the same
/// schema position: before an `extLst` that only the tree holds, and not after it.
#[test]
fn a_new_child_of_an_underline_line_goes_before_its_extension() {
    let from = concat!(
        r#"<a:bodyPr/><a:p><a:r><a:rPr><a:uLn w="1"><a:extLst><a:ext uri="{00000000-0000-0000-0000-000000000009}"/></a:extLst></a:uLn></a:rPr>"#,
        r#"<a:t>x</a:t></a:r></a:p>"#
    );
    let mut wanted = read_part(slide_with_body(from).as_bytes());
    let run = first_run_of_mut(&mut wanted.shape_tree_mut().unwrap().children[0]);
    let properties = run.properties.as_mut().unwrap();
    let Some(UnderlineLine::Line(line)) = properties.underline_line.as_mut() else {
        panic!("no uLn");
    };
    line.fill = Some(Fill::None);
    let bytes = patch_between(from, &wanted).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    let fill = text.find("<a:noFill").unwrap();
    let extension = text.find("<a:extLst").unwrap();
    assert!(fill < extension, "{text}");
    assert_eq!(read_part(&bytes), wanted);
}

#[test]
fn every_enum_value_in_the_text_rows_is_written_and_reads_back() {
    let rows = enum_rows();
    assert!(rows.len() >= 100);
    let mut by_type: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (t, v) in &rows {
        by_type.entry(t).or_default().push(v);
    }
    for (simple_type, values) in by_type {
        let case = enum_case(simple_type);
        for (at, value) in values.iter().enumerate() {
            let other = values[(at + 1) % values.len()];
            let xml = (case.xml)(other);
            let mut wanted = read_part(slide_with_body(&xml).as_bytes());
            let shape = &mut wanted.shape_tree_mut().unwrap().children[0];
            (case.set)(shape.text_body.as_mut().unwrap(), value);
            let bytes = patch_between(&xml, &wanted).unwrap_or_else(|c| panic!("{value}: {c}"));
            let read = read_part(&bytes);
            let body = read.shape_tree().unwrap().children[0].text_body.as_ref();
            assert_eq!(
                (case.get)(body.unwrap()).as_deref(),
                Some(*value),
                "{simple_type}"
            );
            let text = String::from_utf8(bytes).unwrap();
            let lexeme = format!(r#"="{value}""#);
            assert!(text.contains(&lexeme), "{simple_type}: {value} not spelled");
        }
    }
}

const FULL: &str = "generated/text-every-property.pptx";

fn slide_ordinal(parts: &SlideParts) -> u32 {
    parts.structure.slide(0).unwrap()
}

/// Save the first slide after `change`. Second-save and untouched-part checks as in
/// the Tier 1D tests; `parts` are compared byte for byte.
fn edit_slide(source: &[u8], change: impl FnOnce(&mut SlidePart)) -> Result<Vec<u8>, String> {
    let mut wanted = model(source);
    let ordinal = slide_ordinal(&wanted);
    change(wanted.get_mut(ordinal).unwrap());
    let mut pkg = open(source.to_vec());
    let outcome = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &[wanted.get(ordinal).unwrap()],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .map_err(|e| e.code().to_owned())?;
    assert_eq!(model(&outcome.bytes), wanted, "the edit reads back");
    let mut again = open(outcome.bytes.clone());
    let second = save_slide_parts(
        &mut again,
        &wanted.structure,
        &[wanted.get(ordinal).unwrap()],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(second.bytes, outcome.bytes, "a second save changes nothing");
    let (old, new) = (parts(source), parts(&outcome.bytes));
    assert_eq!(
        old.keys().collect::<Vec<_>>(),
        new.keys().collect::<Vec<_>>(),
        "parts were lost or invented"
    );
    for (name, bytes) in &old {
        if !outcome.changed_parts.contains(name) {
            assert_eq!(new.get(name), Some(bytes), "{name} changed");
        }
    }
    Ok(outcome.bytes)
}

fn first_run_of_mut(shape: &mut ShapeNode) -> &mut TextRun {
    let body = shape.text_body.as_mut().unwrap();
    body.paragraphs
        .iter_mut()
        .flat_map(|p| p.items.iter_mut())
        .find_map(|i| match i {
            TextItem::Run(r) => Some(r),
            _ => None,
        })
        .unwrap()
}

fn only_this_changed(source: &[u8], edited: &[u8], part: &str, changes: &[(&str, &str)]) {
    let old = String::from_utf8(parts(source)[part].clone()).unwrap();
    let new = String::from_utf8(parts(edited)[part].clone()).unwrap();
    let mut expected = old.clone();
    for (from, to) in changes {
        assert_eq!(expected.matches(from).count(), 1, "{from} in {part}");
        expected = expected.replace(from, to);
    }
    assert_eq!(new, expected);
    let (a, b) = (parts(source), parts(edited));
    let changed: Vec<&String> = b
        .iter()
        .filter(|(n, v)| a.get(*n) != Some(v))
        .map(|(n, _)| n)
        .collect();
    assert_eq!(changed, [part]);
}

/// The acceptance test: a run's text and a property. Those bytes change, and no others.
#[test]
fn editing_a_runs_text_and_a_property_changes_only_those_nodes() {
    let source = read_fixture(FULL);
    let edited = edit_slide(&source, |slide| {
        let run = first_run_of_mut(named_mut(slide, "Every Property"));
        run.text = Some("Edited & <new>".to_owned());
        run.properties.as_mut().unwrap().size = Some(3200);
    })
    .unwrap();
    only_this_changed(
        &source,
        &edited,
        "ppt/slides/slide1.xml",
        &[
            (
                "<a:t>Every property</a:t>",
                "<a:t>Edited &amp; &lt;new&gt;</a:t>",
            ),
            (
                r#"altLang="de-DE" sz="2400" b="1" i="0" u="dbl" strike="sngStrike" kern="1200" cap="small" spc="-50" normalizeH="1" baseline="30000" noProof="1" dirty="0" err="1" smtClean="0" smtId="7" bmk="mark"><a:ln"#,
                r#"altLang="de-DE" sz="3200" b="1" i="0" u="dbl" strike="sngStrike" kern="1200" cap="small" spc="-50" normalizeH="1" baseline="30000" noProof="1" dirty="0" err="1" smtClean="0" smtId="7" bmk="mark"><a:ln"#,
            ),
        ],
    );
    f4_validates(&edited);
}

#[test]
fn an_edit_on_one_run_leaves_its_neighbours_and_unknown_content_alone() {
    let source = read_fixture(FULL);
    // An equation's alternate content sits between the two runs; the second is edited.
    let edited = edit_slide(&source, |slide| {
        let shape = named_mut(slide, "Equation");
        let body = shape.text_body.as_mut().unwrap();
        let TextItem::Run(run) = &mut body.paragraphs[0].items[2] else {
            panic!("not a run");
        };
        run.text = Some("Later".to_owned());
    })
    .unwrap();
    only_this_changed(
        &source,
        &edited,
        "ppt/slides/slide1.xml",
        &[("<a:t>After</a:t>", "<a:t>Later</a:t>")],
    );
    f4_validates(&edited);
    let new = String::from_utf8(parts(&edited)["ppt/slides/slide1.xml"].clone()).unwrap();
    for kept in [
        "a14:m",
        "mc:AlternateContent",
        "scene3d",
        "gradFill",
        "pattFill",
        "<a:extLst>",
    ] {
        assert!(new.contains(kept), "{kept}");
    }
}

/// `normAutofit` keeps what it was written as across an edit of something else, and an
/// edit of it spells the new value in the form the element has.
#[test]
fn norm_autofit_is_kept_as_written_and_edited_in_its_form() {
    let source = read_fixture(FULL);
    let edited = edit_slide(&source, |slide| {
        let shape = named_mut(slide, "Percent Autofit");
        first_run_of_mut(shape).text = Some("x".to_owned());
    })
    .unwrap();
    let new = String::from_utf8(parts(&edited)["ppt/slides/slide1.xml"].clone()).unwrap();
    assert!(new.contains(r#"fontScale="62.5%" lnSpcReduction="10%""#));

    let edited = edit_slide(&source, |slide| {
        let body = named_mut(slide, "Percent Autofit")
            .text_body
            .as_mut()
            .unwrap();
        let Some(Autofit::Normal { font_scale, .. }) =
            &mut body.properties.as_mut().unwrap().autofit
        else {
            panic!("no autofit");
        };
        *font_scale = Some(Written::new(casual_slide_model::presentation::Percentage(
            50_000,
        )));
    })
    .unwrap();
    only_this_changed(
        &source,
        &edited,
        "ppt/slides/slide1.xml",
        &[(
            r#"<a:normAutofit fontScale="62.5%" lnSpcReduction="10%"/>"#,
            r#"<a:normAutofit fontScale="50%" lnSpcReduction="10%"/>"#,
        )],
    );
    let edited = edit_slide(&source, |slide| {
        let body = named_mut(slide, "Decimal Autofit")
            .text_body
            .as_mut()
            .unwrap();
        let Some(Autofit::Normal { font_scale, .. }) =
            &mut body.properties.as_mut().unwrap().autofit
        else {
            panic!("no autofit");
        };
        *font_scale = Some(Written::new(casual_slide_model::presentation::Percentage(
            50_000,
        )));
    })
    .unwrap();
    only_this_changed(
        &source,
        &edited,
        "ppt/slides/slide1.xml",
        &[(
            r#"fontScale="92500" lnSpcReduction="20000""#,
            r#"fontScale="50000" lnSpcReduction="20000""#,
        )],
    );
    f4_validates(&edited);
    // The written form is held by value: asking for the same value changes nothing.
    let same = edit_slide(&source, |slide| {
        let body = named_mut(slide, "Percent Autofit")
            .text_body
            .as_mut()
            .unwrap();
        let Some(Autofit::Normal { font_scale, .. }) =
            &mut body.properties.as_mut().unwrap().autofit
        else {
            panic!("no autofit");
        };
        *font_scale = Some(Written::new(casual_slide_model::presentation::Percentage(
            62_500,
        )));
    })
    .unwrap();
    assert_eq!(
        same, source,
        "the same value in another spelling is no edit"
    );
}

/// Choices swap, and a new child goes in at its schema position. Each is valid to the
/// schema, not only readable by us.
#[test]
fn property_edits_stay_valid_to_the_schema() {
    let source = read_fixture(FULL);
    let edited = edit_slide(&source, |slide| {
        let run = first_run_of_mut(named_mut(slide, "Every Property"));
        let c = run.properties.as_mut().unwrap();
        c.fill = Some(Fill::Solid(Some(Color::srgb(1, 2, 3))));
        c.underline_line = None;
        c.highlight = None;
        c.latin.as_mut().unwrap().typeface = "Verdana".to_owned();

        let bare = first_run_of_mut(named_mut(slide, "No Autofit"));
        bare.properties = Some(Box::new(CharacterProperties {
            size: Some(1000),
            fill: Some(Fill::Solid(Some(Color::srgb(9, 9, 9)))),
            underline_line: Some(UnderlineLine::FollowText),
            latin: Some(Box::new(casual_slide_model::presentation::TextFont {
                typeface: "Arial".to_owned(),
                ..Default::default()
            })),
            rtl: Some(Default::default()),
            hyperlink_click: Some(Box::new(casual_slide_model::shape::Hyperlink {
                rel_id: Some("rId2".to_owned()),
                tooltip: Some("t".to_owned()),
                ..Default::default()
            })),
            hyperlink_mouse_over: Some(Box::new(casual_slide_model::shape::Hyperlink {
                rel_id: Some(String::new()),
                action: Some("ppaction://noaction".to_owned()),
                ..Default::default()
            })),
            ..CharacterProperties::default()
        }));

        let body = named_mut(slide, "Bullets").text_body.as_mut().unwrap();
        for p in &mut body.paragraphs {
            let pp = p.properties.as_mut().unwrap();
            pp.bullet = Some(Bullet::AutoNumber {
                scheme: casual_slide_model::text::AutonumberScheme::parse_lexical("thaiNumPeriod"),
                start_at: Some(7),
            });
            pp.bullet_color = Some(casual_slide_model::text::BulletColor::Color(Some(
                Color::srgb(4, 5, 6),
            )));
            pp.bullet_size = Some(casual_slide_model::text::BulletSize::Percent(Written::new(
                casual_slide_model::presentation::Percentage(120_000),
            )));
            pp.bullet_font = Some(casual_slide_model::text::BulletFont::FollowText);
            pp.line_spacing = Some(casual_slide_model::text::TextSpacing::Points(1400));
            pp.tabs = Some(vec![TabStop::default(), TabStop::default()]);
            pp.default_run = Some(Box::new(CharacterProperties {
                bold: Some(true),
                ..CharacterProperties::default()
            }));
            pp.margin_left = Some(5);
        }

        let body = named_mut(slide, "Shape Autofit")
            .text_body
            .as_mut()
            .unwrap();
        let props = body.properties.as_mut().unwrap();
        props.autofit = Some(Autofit::Normal {
            font_scale: Some(Written::new(casual_slide_model::presentation::Percentage(
                70_000,
            ))),
            line_spacing_reduction: None,
        });
        props.anchor = casual_slide_model::text::TextAnchor::parse_lexical("b");
        props.inset_left = Some(Written::new(casual_slide_model::presentation::Coordinate(
            7,
        )));
        props.warp = Some(casual_slide_model::geometry::PresetTextShape {
            preset: casual_slide_model::geometry::TextShapeType::parse_lexical("textPlain"),
            adjust: Some(Vec::new()),
        });
    })
    .unwrap();
    f4_validates(&edited);
}

/// Paragraphs and runs grow and shrink at the tail, in schema order, and the end of a
/// paragraph stays last.
#[test]
fn paragraphs_and_runs_grow_and_shrink_at_the_tail() {
    let source = read_fixture(FULL);
    let edited = edit_slide(&source, |slide| {
        let shape = named_mut(slide, "Every Property");
        let body = shape.text_body.as_mut().unwrap();
        let first = &mut body.paragraphs[0];
        first.items.push(TextItem::Run(TextRun {
            properties: None,
            text: Some("appended".to_owned()),
        }));
        body.paragraphs.push(Paragraph {
            properties: Some(Box::new(ParagraphProperties {
                level: Some(2),
                ..ParagraphProperties::default()
            })),
            items: vec![
                TextItem::Run(TextRun {
                    properties: Some(Box::default()),
                    text: Some("new".to_owned()),
                }),
                TextItem::Break(Default::default()),
                TextItem::Field(casual_slide_model::text::TextField {
                    id: Some(GUID_FOR_TESTS.to_owned()),
                    field_type: Some("datetime1".to_owned()),
                    text: Some("today".to_owned()),
                    ..Default::default()
                }),
            ],
            end_properties: Some(Box::default()),
        });
        let two = named_mut(slide, "Bullets").text_body.as_mut().unwrap();
        two.paragraphs.truncate(1);
    })
    .unwrap();
    f4_validates(&edited);
    let new = String::from_utf8(parts(&edited)["ppt/slides/slide1.xml"].clone()).unwrap();
    assert!(
        new.contains("<a:t>appended</a:t></a:r><a:endParaRPr"),
        "before the end"
    );
    assert!(!new.contains("Bullet 1"));
}

#[test]
fn a_new_text_body_on_a_shape_with_none_is_written_at_its_place() {
    let source = read_fixture("generated/slides-masters-layouts.pptx");
    let edited = edit_slide(&source, |slide| {
        let tree = slide.shape_tree_mut().unwrap();
        let shape = tree.children.first_mut().unwrap();
        shape.text_body = Some(Box::new(TextBody {
            properties: Some(Default::default()),
            list_style: Some(Default::default()),
            paragraphs: vec![Paragraph {
                items: vec![TextItem::Run(TextRun {
                    properties: None,
                    text: Some("replaced".to_owned()),
                })],
                ..Paragraph::default()
            }],
        }));
    })
    .unwrap();
    f4_validates(&edited);
    let source = read_fixture(FULL);
    let edited = edit_slide(&source, |slide| {
        named_mut(slide, "Whitespace").text_body = None;
    })
    .unwrap();
    f4_validates(&edited);
    let new = String::from_utf8(parts(&edited)["ppt/slides/slide1.xml"].clone()).unwrap();
    assert!(!new.contains("two &amp;"));
}

/// The presentation default and the master text styles.
#[test]
fn default_text_style_and_master_styles_are_patched_in_place() {
    let source = read_fixture(FULL);
    let mut pkg = open(source.clone());
    let imported = import_deck(&mut pkg, &ImportLimits::default()).unwrap();
    let mut deck = imported.deck;
    deck.presentation
        .default_text_style
        .as_mut()
        .unwrap()
        .levels[0]
        .as_mut()
        .unwrap()
        .default_run
        .as_mut()
        .unwrap()
        .size = Some(2000);
    let outcome = save_deck(&mut pkg, &deck, &ImportLimits::default()).unwrap();
    only_this_changed(
        &source,
        &outcome.bytes,
        "ppt/presentation.xml",
        &[(r#"<a:defRPr sz="1800"/>"#, r#"<a:defRPr sz="2000"/>"#)],
    );
    f4_validates(&outcome.bytes);

    // A deck with none gains one, in schema position.
    let source = read_fixture("generated/minimal.pptx");
    let mut pkg = open(source.clone());
    let mut deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    let mut style = TextListStyle::default();
    style.levels[8] = Some(Box::new(ParagraphProperties {
        align: Some(casual_slide_model::text::TextAlign::Center),
        ..ParagraphProperties::default()
    }));
    deck.presentation.default_text_style = Some(style.clone());
    let outcome = save_deck(&mut pkg, &deck, &ImportLimits::default()).unwrap();
    f4_validates(&outcome.bytes);
    let mut again = open(outcome.bytes.clone());
    let read = import_deck(&mut again, &ImportLimits::default())
        .unwrap()
        .deck;
    assert_eq!(read.presentation.default_text_style, Some(style));

    // Master styles, by the part's own patch.
    let source = read_fixture("generated/text-inheritance.pptx");
    let mut wanted = model(&source);
    let master_ordinal = wanted.structure.masters[0];
    let master = wanted.get_mut(master_ordinal).unwrap();
    let body = master.text_styles.as_mut().unwrap().body.as_mut().unwrap();
    body.levels[0].as_mut().unwrap().margin_right = Some(99);
    master.text_styles.as_mut().unwrap().other = None;
    let mut pkg = open(source.clone());
    let outcome = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &[wanted.get(master_ordinal).unwrap()],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(model(&outcome.bytes), wanted);
    assert_eq!(outcome.changed_parts, ["ppt/slideMasters/slideMaster1.xml"]);
    f4_validates(&outcome.bytes);

    // `notesStyle`, on the notes master.
    let source = read_fixture("generated/slides-masters-layouts.pptx");
    let mut wanted = model(&source);
    let notes_master = wanted.structure.notes_master.unwrap();
    let part = wanted.get_mut(notes_master).unwrap();
    part.notes_style.as_mut().unwrap().levels[0]
        .as_mut()
        .unwrap()
        .indent = Some(-5);
    let mut pkg = open(source);
    let outcome = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &[wanted.get(notes_master).unwrap()],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(model(&outcome.bytes), wanted);
    f4_validates(&outcome.bytes);
}

#[test]
fn a_patch_of_an_unchanged_presentation_changes_nothing() {
    let source = read_fixture(FULL);
    let main = "ppt/presentation.xml";
    let original = parts(&source)[main].clone();
    let mut tree = XmlTree::parse(&original, main, &OoxmlLimits::default()).unwrap();
    let limits = ImportLimits::default();
    let mut findings = Findings::new(limits.max_findings);
    let pkg = open(source);
    let rels = pkg.relationships_of(main).to_vec();
    let before = import_presentation(&tree, main, &rels, &limits, &mut findings).unwrap();
    assert_eq!(
        patch_presentation(&mut tree, main, false, &rels, &before, &limits).unwrap(),
        0
    );
    assert_eq!(tree.to_bytes(), original);
}

/// What another tier owns can be kept or taken away, and not made from here.
#[test]
fn what_this_tier_cannot_write_is_refused_and_nothing_is_saved() {
    let source = read_fixture(FULL);
    let refused = |change: fn(&mut SlidePart)| edit_slide(&source, change).unwrap_err();
    for change in [
        (|s| {
            let body = named_mut(s, "Bullets").text_body.as_mut().unwrap();
            body.paragraphs[0].properties.as_mut().unwrap().bullet = Some(Bullet::Picture(None));
        }) as fn(&mut SlidePart),
        |s| {
            named_mut(s, "No Autofit")
                .text_body
                .as_mut()
                .unwrap()
                .properties = None
        },
        |s| {
            named_mut(s, "No Autofit")
                .text_body
                .as_mut()
                .unwrap()
                .paragraphs
                .clear()
        },
        |s| first_run_of_mut(named_mut(s, "No Autofit")).text = None,
        // Reordering, and a retained item.
        |s| {
            named_mut(s, "Equation")
                .text_body
                .as_mut()
                .unwrap()
                .paragraphs[0]
                .items
                .swap(0, 1)
        },
        |s| {
            named_mut(s, "Equation")
                .text_body
                .as_mut()
                .unwrap()
                .paragraphs[0]
                .items
                .push(TextItem::Retained("x".to_owned()))
        },
        // A new body with nothing the schema requires.
        |s| named_mut(s, "Whitespace").text_body = Some(Box::default()),
    ] {
        assert_eq!(refused(change), "CS-EXP-0009");
    }
    // An equation is in the model since `T1K-02`, so taking it away is an edit like another.
    edit_slide(&source, |s| {
        named_mut(s, "Equation")
            .text_body
            .as_mut()
            .unwrap()
            .paragraphs[0]
            .items
            .truncate(1)
    })
    .unwrap();
    // Over a type's range: not written, and the read-back says so.
    let code = edit_slide(&source, |s| {
        first_run_of_mut(named_mut(s, "No Autofit")).properties =
            Some(Box::new(CharacterProperties {
                size: Some(1),
                ..CharacterProperties::default()
            }));
    })
    .unwrap_err();
    assert_eq!(code, "CS-EXP-0007");
    // A character XML cannot carry.
    let code = edit_slide(&source, |s| {
        first_run_of_mut(named_mut(s, "No Autofit")).text = Some("bell\u{7}".to_owned());
    })
    .unwrap_err();
    assert_eq!(code, "CS-EXP-0001");
    // A new hyperlink with no `r:id`, which the schema requires.
    let code = edit_slide(&source, |s| {
        first_run_of_mut(named_mut(s, "No Autofit")).properties =
            Some(Box::new(CharacterProperties {
                hyperlink_click: Some(Box::default()),
                ..CharacterProperties::default()
            }));
    })
    .unwrap_err();
    assert_eq!(code, "CS-EXP-0009");
    // A relationship the part does not have.
    let code = edit_slide(&source, |s| {
        first_run_of_mut(named_mut(s, "No Autofit")).properties =
            Some(Box::new(CharacterProperties {
                hyperlink_click: Some(Box::new(casual_slide_model::shape::Hyperlink {
                    rel_id: Some("rId404".to_owned()),
                    ..Default::default()
                })),
                ..CharacterProperties::default()
            }));
    })
    .unwrap_err();
    assert_eq!(code, "CS-EXP-0003");
    // A hyperlink's colour extension is read, not written.
    let code = edit_slide(&source, |s| {
        let run = first_run_of_mut(named_mut(s, "Every Property"));
        let link = run
            .properties
            .as_mut()
            .unwrap()
            .hyperlink_click
            .as_mut()
            .unwrap();
        link.color = Some(casual_slide_model::shape::HyperlinkColor::Text);
    })
    .unwrap_err();
    assert_eq!(code, "CS-EXP-0009");
}

/// Tier 1E's writer, from a run: each kind is made, changed in place and taken away,
/// reads back, and stays valid to the schema.
#[test]
fn a_runs_fill_line_effects_and_underline_are_written_through_tier_1e() {
    let source = read_fixture(FULL);
    let stated = |c: CharacterProperties| {
        move |s: &mut SlidePart| {
            first_run_of_mut(named_mut(s, "No Autofit")).properties = Some(Box::new(c));
        }
    };
    let solid = |r| Some(Fill::Solid(Some(Color::srgb(r, 0, 0))));
    let cases = [
        CharacterProperties {
            fill: Some(Fill::Gradient(Box::new(GradientFill {
                rotate_with_shape: Some(true),
                stops: Some(Vec::new()),
                ..GradientFill::default()
            }))),
            line: Some(Box::new(Line {
                width: Some(12700),
                fill: solid(1),
                ..Line::default()
            })),
            effects: Some(Box::new(EffectProperties::List(vec![Effect::SoftEdge(
                Some(casual_slide_model::presentation::Coordinate(12700)),
            )]))),
            underline_line: Some(UnderlineLine::Line(Box::new(Line {
                width: Some(9525),
                ..Line::default()
            }))),
            underline_fill: Some(casual_slide_model::text::UnderlineFill::Fill(solid(2))),
            ..CharacterProperties::default()
        },
        CharacterProperties {
            fill: Some(Fill::Solid(None)),
            effects: Some(Box::new(EffectProperties::List(Vec::new()))),
            ..CharacterProperties::default()
        },
    ];
    for case in cases {
        // `edit_slide` checks the edit reads back, and is a fixed point.
        edit_slide(&source, stated(case)).unwrap();
    }

    // A real gradient, in the deck, changed in place: its stops' spelling stays.
    let edited = edit_slide(&source, |s| {
        let body = named_mut(s, "Fills").text_body.as_mut().unwrap();
        let TextItem::Run(run) = &mut body.paragraphs[0].items[1] else {
            panic!("not a run");
        };
        let Some(Fill::Gradient(g)) = &mut run.properties.as_mut().unwrap().fill else {
            panic!("no gradient");
        };
        g.rotate_with_shape = Some(true);
    })
    .unwrap();
    only_this_changed(
        &source,
        &edited,
        "ppt/slides/slide1.xml",
        &[(
            "<a:rPr><a:gradFill>",
            r#"<a:rPr><a:gradFill rotWithShape="1">"#,
        )],
    );
    f4_validates(&edited);

    // Kinds swapped, a line and effects taken away, an underline line changed.
    let edited = edit_slide(&source, |s| {
        let run = first_run_of_mut(named_mut(s, "Every Property"));
        let c = run.properties.as_mut().unwrap();
        c.line.as_mut().unwrap().width = Some(25400);
        c.effects = None;
        c.fill = Some(Fill::Group);
        let body = named_mut(s, "Fills").text_body.as_mut().unwrap();
        let TextItem::Run(under) = &mut body.paragraphs[0].items[5] else {
            panic!("not a run");
        };
        let c = under.properties.as_mut().unwrap();
        c.underline_line = Some(UnderlineLine::FollowText);
        c.effects = Some(Box::new(EffectProperties::List(Vec::new())));
        c.fill = solid(3);
    })
    .unwrap();
    f4_validates(&edited);
}

#[test]
fn strict_decks_are_patched_in_their_own_namespaces_and_percent_forms() {
    let source = read_fixture("generated/text-strict.pptx");
    f4_validates(&source);
    let edited = edit_slide(&source, |slide| {
        let shape = named_mut(slide, "Every Property");
        let run = first_run_of_mut(shape);
        run.text = Some("Strict".to_owned());
        let c = run.properties.as_mut().unwrap();
        c.baseline = Some(Written::new(casual_slide_model::presentation::Percentage(
            12_500,
        )));
        c.hyperlink_click.as_mut().unwrap().tooltip = None;
        let body = shape.text_body.as_mut().unwrap();
        let p = body.paragraphs[0].properties.as_mut().unwrap();
        p.line_spacing = Some(casual_slide_model::text::TextSpacing::Percent(
            Written::new(casual_slide_model::presentation::Percentage(80_000)),
        ));
        p.bullet_size = Some(casual_slide_model::text::BulletSize::Percent(Written::new(
            casual_slide_model::presentation::Percentage(125_000),
        )));
        let props = body.properties.as_mut().unwrap();
        props.autofit = Some(Autofit::Normal {
            font_scale: Some(Written::new(casual_slide_model::presentation::Percentage(
                70_000,
            ))),
            line_spacing_reduction: None,
        });
        // New elements in the part's own namespace.
        let bare = named_mut(slide, "Shape Autofit");
        first_run_of_mut(bare).properties = Some(Box::new(CharacterProperties {
            fill: Some(Fill::Solid(Some(Color::srgb(1, 1, 1)))),
            baseline: Some(Written::new(casual_slide_model::presentation::Percentage(
                5_000,
            ))),
            ..CharacterProperties::default()
        }));
    })
    .unwrap();
    f4_validates(&edited);
    let new = String::from_utf8(parts(&edited)["ppt/slides/slide1.xml"].clone()).unwrap();
    assert!(new.contains(r#"baseline="12.5%""#), "{new}");
    assert!(new.contains(r#"fontScale="70%""#));
    assert!(new.contains(r#"<a:spcPct val="80%"/>"#));
    assert!(new.contains(r#"<a:buSzPct val="125%"/>"#));
}

#[test]
fn every_deck_is_a_fixed_point_and_unedited_bytes_come_back() {
    let mut count = 0;
    for path in all_decks() {
        let source = read_fixture(&path);
        let parts_model = model(&source);
        assert_eq!(
            model(&source),
            parts_model,
            "{path}: same input, same model"
        );
        let mut pkg = open(source.clone());
        let all: Vec<&SlidePart> = parts_model.parts.iter().flatten().collect();
        let outcome = save_slide_parts(
            &mut pkg,
            &parts_model.structure,
            &all,
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(outcome.bytes, source, "{path}");
        assert!(outcome.changed_parts.is_empty(), "{path}");
        count += 1;
    }
    assert!(count >= 35, "{count}");
}

/// F4 is a question about a deck, and some real decks were not valid before we saw them.
/// Those are not this tier's to fix, so an edit there need only make nothing worse.
fn f4_failures(bytes: &[u8]) -> Option<String> {
    let have = |tool: &str| Command::new(tool).arg("--version").output().is_ok();
    if !have("xmllint") || !have("python3") {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
        eprintln!("xmllint or python3 not installed: F4 skipped");
        return None;
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("f4-text");
    std::fs::create_dir_all(&dir).unwrap();
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let file = dir.join(format!("edited-{}-{n}.pptx", std::process::id()));
    std::fs::write(&file, bytes).unwrap();
    let tool = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/f4/validate.py");
    let out = Command::new("python3")
        .arg("-I")
        .arg(tool)
        .arg(&file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();
    (!out.status.success()).then(|| {
        let mut errors: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| l.contains("Schemas validity error"))
            .map(str::to_owned)
            .collect();
        errors.sort();
        errors.join("\n")
    })
}

fn f4_validates(bytes: &[u8]) {
    if let Some(errors) = f4_failures(bytes) {
        panic!("{errors}");
    }
}

/// A run's text and a property, on the first slide with a run, on every deck: only that
/// part changes, and F4 finds nothing it did not already.
#[test]
fn every_deck_takes_a_text_edit_and_stays_as_valid_as_it_was() {
    let mut edited_decks = 0;
    let mut already_invalid = Vec::new();
    for path in all_decks() {
        let source = read_fixture(&path);
        let wanted = model(&source);
        let target = wanted.structure.slides.iter().find_map(|&ordinal| {
            let part = wanted.get(ordinal)?;
            let node = part.shape_tree()?.descendants().find(|n| {
                n.text_body.as_ref().is_some_and(|b| {
                    b.paragraphs
                        .iter()
                        .any(|p| p.items.iter().any(|i| matches!(i, TextItem::Run(_))))
                })
            })?;
            Some((ordinal, node.id))
        });
        let Some((ordinal, id)) = target else {
            continue;
        };
        let mut wanted = wanted;
        let part = wanted.get_mut(ordinal).unwrap();
        let run = first_run_of_mut(part.node_mut(id).unwrap());
        run.text = Some("Edited text".to_owned());
        let properties = run.properties.get_or_insert_with(Box::default);
        properties.bold = Some(properties.bold != Some(true));
        let mut pkg = open(source.clone());
        let result = save_slide_parts(
            &mut pkg,
            &wanted.structure,
            &[wanted.get(ordinal).unwrap()],
            &SlideLimits::default(),
            &ImportLimits::default(),
        );
        let outcome = match result {
            Ok(outcome) => outcome,
            // A shape in an alternate branch is refused, as the Tier 1D tests hold.
            Err(e) if e.code() == "CS-SLE-0002" => continue,
            Err(e) => panic!("{path}: {e}"),
        };
        assert_eq!(model(&outcome.bytes), wanted, "{path}");
        let (old, new) = (parts(&source), parts(&outcome.bytes));
        assert_eq!(edited_parts(&outcome.changed_parts), 1, "{path}");
        for (name, bytes) in &old {
            if !outcome.changed_parts.contains(name) {
                assert_eq!(new.get(name), Some(bytes), "{path}: {name} changed");
            }
        }
        edited_decks += 1;
        // Only real and producer decks are worth the wall time.
        if !path.starts_with("generated/area") && !path.contains("five-hundred") {
            let before = f4_failures(&source);
            let after = f4_failures(&outcome.bytes);
            if before.is_some() {
                already_invalid.push(path.clone());
            }
            assert_eq!(after, before, "{path}: the edit changed what F4 finds");
        }
    }
    assert!(edited_decks >= 30, "{edited_decks}");
    eprintln!("invalid before any edit: {already_invalid:?}");
}

/// The creating path, on everything real: take each text body away, then ask for it
/// back. Whatever a real deck holds is written from the model, read back the same, and
/// valid; a body holding what another tier owns is refused and counted, never mangled.
/// Warp, scene and 3D edits on a deck: only what was named changes, the edit reads back,
/// and the part stays schema-valid.
#[test]
fn a_bodys_warp_and_3d_edits_stay_valid_to_the_schema() {
    let source = read_fixture("generated/text-warp-and-3d.pptx");
    type Edit = fn(&mut BodyProperties);
    let edits: [(&str, Edit); 5] = [
        ("Warp only", |b| {
            b.warp.as_mut().unwrap().adjust.as_mut().unwrap()[0].formula =
                Some("val 5400000".to_owned());
        }),
        ("Scene and extrusion", |b| {
            b.text_3d = Some(Text3D::Flat(FlatText {
                z: Some(casual_slide_model::presentation::Coordinate(7)),
            }));
        }),
        ("Scene and flat", |b| {
            b.text_3d = Some(Text3D::Shape(Box::default()));
        }),
        ("Flat bare", |b| {
            b.scene3d = Some(Box::new(Scene3D {
                camera: Some(Camera {
                    preset: PresetCamera::parse_lexical("orthographicFront"),
                    ..Camera::default()
                }),
                light_rig: Some(LightRig {
                    rig: LightRigType::parse_lexical("flat"),
                    direction: LightRigDirection::parse_lexical("t"),
                    ..LightRig::default()
                }),
                ..Scene3D::default()
            }));
            b.text_3d = None;
        }),
        ("Warp no list", |b| b.warp = None),
    ];
    for (name, edit) in edits {
        let edited = edit_slide(&source, |part| {
            let body = named_mut(part, name).text_body.as_mut().unwrap();
            edit(body.properties.as_mut().unwrap());
        })
        .unwrap_or_else(|e| panic!("{name}: {e}"));
        f4_validates(&edited);
    }
}

#[test]
fn every_text_body_can_be_written_from_the_model() {
    /// What refusing a body rests on: the model holds no content for either, so it can
    /// only be told to keep what the tree has. A picture bullet's is Tier 1H's. A
    /// paragraph's `TextItem::Retained` is an inline equation's `mc:AlternateContent` or
    /// the like, Tier 1K's.
    /// A retained item: the model holds a name and not its content.
    fn cannot_be_made(body: &TextBody) -> bool {
        body.paragraphs
            .iter()
            .any(|p| p.items.iter().any(|i| matches!(i, TextItem::Retained(_))))
    }

    #[derive(Default)]
    struct Tally {
        written: usize,
        refused: Vec<String>,
    }

    let xml = OoxmlLimits::default();
    let limits = SlideLimits::default();
    let import_limits = ImportLimits::default();
    // A slide is written once however many decks hold its bytes, and counted for each.
    type Key = (Vec<u8>, Vec<String>);
    let mut seen: BTreeMap<Key, (usize, Vec<String>)> = BTreeMap::new();
    let mut tally = Tally::default();
    for path in all_decks() {
        if path.contains("five-hundred") {
            continue;
        }
        let source = read_fixture(&path);
        let original = model(&source);
        for &ordinal in &original.structure.slides {
            let Some(part) = original.get(ordinal) else {
                continue;
            };
            let name = original.structure.entry(ordinal).unwrap().part.clone();
            let mut pkg = open(source.clone());
            let bytes = pkg.read_part(&name).unwrap();
            let rels = pkg.relationships_of(&name).to_vec();
            let key = (bytes.clone(), rels.iter().map(|r| r.id.clone()).collect());
            let (written, refused) = seen.entry(key).or_insert_with(|| {
                let ids: Vec<_> = part
                    .shape_tree()
                    .into_iter()
                    .flat_map(ShapeNode::descendants)
                    .filter(|n| n.text_body.is_some())
                    .map(|n| n.id)
                    .collect();
                let patched = |tree: &mut XmlTree, to: &SlidePart| {
                    patch_slide_part(tree, &name, false, &rels, to, &limits, &import_limits, &xml)
                };
                // All of a part's bodies at once: each patch is read back whole against
                // the model, so every body is held to what one alone would be.
                let mut emptied = part.clone();
                for id in &ids {
                    emptied.node_mut(*id).unwrap().text_body = None;
                }
                let mut tree = XmlTree::parse(&bytes, &name, &xml).unwrap();
                if patched(&mut tree, &emptied).is_ok() && patched(&mut tree, part).is_ok() {
                    return (ids.len(), Vec::new());
                }
                // Something here will not go through: find out which, one at a time.
                let (mut written, mut refused) = (0, Vec::new());
                for id in ids {
                    let mut emptied = part.clone();
                    emptied.node_mut(id).unwrap().text_body = None;
                    let mut tree = XmlTree::parse(&bytes, &name, &xml).unwrap();
                    match patched(&mut tree, &emptied) {
                        Ok(_) => {}
                        // A shape in an alternate branch.
                        Err(e) if e.code() == "CS-SLE-0002" => continue,
                        Err(e) => panic!("{name}: {e}"),
                    }
                    match patched(&mut tree, part) {
                        Ok(_) => written += 1,
                        Err(e) if e.code() == "CS-EXP-0009" => {
                            let node = part
                                .shape_tree()
                                .into_iter()
                                .flat_map(ShapeNode::descendants)
                                .find(|n| n.id == id)
                                .unwrap();
                            assert!(
                                node.text_body.as_deref().is_some_and(cannot_be_made),
                                "{name}: refused, and holding nothing the model cannot make: {e}"
                            );
                            refused.push(format!("{e}"));
                        }
                        Err(e) => panic!("{name}: {e}"),
                    }
                }
                (written, refused)
            });
            tally.written += *written;
            tally.refused.extend(refused.iter().cloned());
        }
    }
    eprintln!(
        "written from the model: {}; refused: {:?}",
        tally.written, tally.refused
    );
    assert!(tally.written > 1000, "{}", tally.written);
    // A picture bullet is made now (`T1F-04`); only what is retained is refused.
    // `T1F-04`'s last two, the inline equations, are written from the model since `T1K-02`.
    assert!(tally.refused.is_empty(), "{:?}", tally.refused);
}

#[test]
fn run_and_text_edits_are_refused_inside_an_alternate_branch() {
    // Tier 1D's rule holds for text too: `CS-SLE-0002`, as its other branches go stale.
    let source = read_fixture("generated/slides-masters-layouts.pptx");
    let mut wanted = model(&source);
    let ordinal = wanted.structure.slides[1];
    let part = wanted.get_mut(ordinal).unwrap();
    let id = part
        .shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some("Fallback Shape"))
        .unwrap()
        .id;
    first_run_of_mut(part.node_mut(id).unwrap()).text = Some("x".to_owned());
    let mut pkg = open(source);
    let err = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &[wanted.get(ordinal).unwrap()],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-SLE-0002");
}

#[test]
fn nothing_from_a_body_reaches_an_export_message() {
    let source = read_fixture(FULL);
    let mut wanted = model(&source);
    let ordinal = slide_ordinal(&wanted);
    let part = wanted.get_mut(ordinal).unwrap();
    first_run_of_mut(named_mut(part, "No Autofit")).text = Some("QzMarkerQz\u{7}".to_owned());
    let mut pkg = open(source);
    let err = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &[wanted.get(ordinal).unwrap()],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap_err();
    let message = format!("{err} {err:?}");
    assert!(!message.contains("QzMarkerQz"), "{message}");
}

/// What an edit asked for, leaving out what any edit owes the package: `app.xml`, a
/// thumbnail, a signature set (`T1N-05`).
fn edited_parts(changed: &[String]) -> usize {
    changed
        .iter()
        .filter(|n| {
            !n.starts_with("docProps/")
                && !n.starts_with("_xmlsignatures/")
                && !matches!(n.as_str(), "_rels/.rels" | "[Content_Types].xml")
        })
        .count()
}
