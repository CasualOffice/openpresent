//! Tier 1F import: every enum value and every row of `docs/spec/TEXT.md`, read from the
//! checklist at test time; stored lexemes; hostile text; nothing from a document in a
//! diagnostic.

use std::path::{Path, PathBuf};

use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportFindingKind as K, ImportLimits, TextBound};
use casual_slide_import::slide::{
    ImportedSlides, SlideLimits, import_slide_part, import_slide_parts,
};
use casual_slide_import::text::TextLimits;
use casual_slide_model::effect::EffectProperties;
use casual_slide_model::fill::Fill;
use casual_slide_model::shape::ShapeNode;
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_model::text::{
    Autofit, BulletColor, BulletFont, BulletSize, CharacterProperties, TextSpacing, UnderlineFill,
    UnderlineLine,
};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, Relationship, TargetMode, XmlTree};

include!("text_tables.inc");

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn rels() -> Vec<Relationship> {
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

/// A body, as the shape on a slide reads it, and what was noticed.
fn read(body: &str) -> (TextBody, Findings, ColorFindings) {
    let xml = slide_with_body(body);
    let tree = XmlTree::parse(xml.as_bytes(), "s.xml", &OoxmlLimits::default())
        .unwrap_or_else(|e| panic!("{e}: {body}"));
    let limits = ImportLimits::default();
    let mut findings = Findings::new(limits.max_findings);
    let mut color = ColorFindings::default();
    let part = import_slide_part(
        &tree,
        "s.xml",
        &rels(),
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &limits,
        &mut findings,
        &mut color,
    )
    .unwrap();
    let shape = &part.shape_tree().unwrap().children[0];
    (
        (**shape.text_body.as_ref().unwrap()).clone(),
        findings,
        color,
    )
}

fn deck(path: &str) -> ImportedSlides {
    deck_with(path, &ImportLimits::default()).unwrap()
}

fn deck_with(
    path: &str,
    limits: &ImportLimits,
) -> Result<ImportedSlides, casual_slide_import::slide::SlideError> {
    let bytes = std::fs::read(fixtures().join(path)).unwrap();
    let mut pkg = PresentationPackage::open_strict(bytes, OoxmlLimits::default()).unwrap();
    let presentation = import_deck(&mut pkg, limits).unwrap().deck.presentation;
    import_slide_parts(&mut pkg, &presentation, &SlideLimits::default(), limits)
}

fn named<'a>(part: &'a SlidePart, name: &str) -> &'a ShapeNode {
    part.shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some(name))
        .unwrap_or_else(|| panic!("no {name}"))
}

fn body_of<'a>(part: &'a SlidePart, name: &str) -> &'a TextBody {
    named(part, name).text_body.as_ref().unwrap()
}

#[test]
fn every_enum_value_in_the_text_rows_is_read_and_a_bad_one_is_reported() {
    let rows = enum_rows();
    assert!(rows.len() >= 100, "{} enum values in TEXT", rows.len());
    assert_eq!(
        rows.iter()
            .filter(|(t, _)| t == "ST_TextAutonumberScheme")
            .count(),
        41
    );
    for (simple_type, value) in &rows {
        let case = enum_case(simple_type);
        let (body, findings, _) = read(&(case.xml)(value));
        assert_eq!(
            (case.get)(&body).as_deref(),
            Some(value.as_str()),
            "{simple_type} {value}"
        );
        assert!(
            !findings
                .iter()
                .any(|f| matches!(f.kind, K::MalformedValue | K::OutOfRange)),
            "{simple_type} {value}: {findings:?}"
        );
    }
    let mut types: Vec<&String> = rows.iter().map(|(t, _)| t).collect();
    types.dedup();
    for simple_type in types {
        let case = enum_case(simple_type);
        let (body, findings, _) = read(&(case.xml)("notAValue"));
        assert_eq!((case.get)(&body), None, "{simple_type}");
        assert!(
            findings.iter().any(|f| f.kind == K::MalformedValue),
            "{simple_type}: a value no schema allows is reported"
        );
    }
}

#[test]
fn every_row_of_the_text_area_is_modelled_or_reported() {
    let rows = property_rows();
    assert!(rows.len() >= 130, "{} rows in TEXT", rows.len());
    let mut checked = 0;
    for id in &rows {
        let Some(row) = text_row(id) else { continue };
        checked += 1;
        let (base, _, _) = read(&row.base);
        let (with, findings, color) = read(&row.with);
        assert!(
            !findings.iter().any(|f| matches!(
                f.kind,
                K::MalformedValue | K::OutOfRange | K::RetainedAttribute
            )),
            "{id}: {findings:?}"
        );
        assert_eq!(color.total, 0, "{id}: {:?}", color.items);
        match row.outcome {
            Outcome::Modelled => {
                assert_ne!(base, with, "{id} is not in the model");
                assert!(
                    !findings.iter().any(|f| f.kind == K::RetainedElement),
                    "{id}: {findings:?}"
                );
            }
            Outcome::Stated => {
                assert_ne!(base, with, "{id}: that it is stated is not in the model");
                assert!(
                    findings.has(K::RetainedElement, row.element) || row.element == "buBlip",
                    "{id}: {findings:?}"
                );
            }
            Outcome::Reported => {
                assert_eq!(base, with, "{id}");
                assert!(
                    findings.has(K::RetainedElement, row.element),
                    "{id} is not reported: {findings:?}"
                );
            }
        }
    }
    assert!(checked >= 130, "{checked} rows checked");
}

/// A `bodyPr` attribute must say what it says on its own, not leave its neighbour's.
#[test]
fn every_property_deck_reads_as_written() {
    let s = deck("generated/text-every-property.pptx");
    let slide = s.parts.slide(0).unwrap();
    let every = body_of(slide, "Every Property");
    let b = every.properties.as_ref().unwrap();
    assert_eq!(b.rotation, Some(60000));
    assert_eq!(b.columns, Some(2));
    assert_eq!(b.column_spacing, Some(91440));
    assert_eq!(b.inset_left.as_ref().unwrap().value().0, 91440);
    assert_eq!(b.inset_top.as_ref().unwrap().value().0, 914_400);
    assert_eq!(
        b.inset_top.as_ref().unwrap().written(),
        Some("1in"),
        "a universal measure keeps its unit"
    );
    assert_eq!(b.inset_right.as_ref().unwrap().written(), Some("2.5cm"));
    assert_eq!(
        b.warp.as_ref().unwrap().preset.as_deref(),
        Some("textPlain")
    );
    let Some(Autofit::Normal {
        font_scale,
        line_spacing_reduction,
    }) = &b.autofit
    else {
        panic!("no normAutofit");
    };
    assert_eq!(font_scale.as_ref().unwrap().written(), Some("62.5%"));
    assert_eq!(percent_value(font_scale.as_ref().unwrap()), 62_500);
    assert_eq!(
        line_spacing_reduction.as_ref().unwrap().written(),
        Some("10000")
    );

    let list = every.list_style.as_ref().unwrap();
    assert!(list.default.is_some(), "defPPr is kept");
    for (i, level) in list.levels.iter().enumerate() {
        let level = level.as_ref().unwrap();
        assert_eq!(level.margin_left, Some((i as i32 + 1) * 1000));
        assert_eq!(
            level.default_run.as_ref().unwrap().size,
            Some(1000 + (i as i32 + 1) * 100)
        );
    }

    let p = &every.paragraphs[0];
    let pp: &ParagraphProperties = p.properties.as_ref().unwrap();
    assert_eq!(pp.level, Some(1));
    assert_eq!(pp.margin_left, Some(342_900));
    assert_eq!(pp.indent, Some(-342_900));
    let Some(TextSpacing::Percent(line)) = &pp.line_spacing else {
        panic!("lnSpc");
    };
    assert_eq!(percent_value(line), 90_000);
    assert_eq!(pp.space_before, Some(TextSpacing::Points(1000)));
    let Some(TextSpacing::Percent(after)) = &pp.space_after else {
        panic!("spcAft");
    };
    assert_eq!(after.written(), Some("50%"));
    assert_eq!(percent_value(after), 50_000);
    assert!(matches!(pp.bullet_color, Some(BulletColor::Color(Some(_)))));
    let Some(BulletSize::Percent(size)) = &pp.bullet_size else {
        panic!("buSzPct");
    };
    assert_eq!(percent_value(size), 150_000);
    assert!(matches!(&pp.bullet_font, Some(BulletFont::Font(f)) if f.typeface == "Arial"));
    assert_eq!(pp.bullet, Some(Bullet::Char(Some("\u{2022}".to_owned()))));
    let tabs = pp.tabs.as_ref().unwrap();
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[1].position.as_ref().unwrap().written(), Some("1in"));

    let [
        TextItem::Run(run),
        TextItem::Break(br),
        TextItem::Field(field),
    ] = p.items.as_slice()
    else {
        panic!("{:?}", p.items);
    };
    assert_eq!(run.text.as_deref(), Some("Every property"));
    let c: &CharacterProperties = run.properties.as_ref().unwrap();
    assert_eq!(c.size, Some(2400));
    assert_eq!(c.kerning, Some(1200));
    assert_eq!(c.spacing.as_ref().unwrap().value().0, -50);
    assert_eq!(percent_value(c.baseline.as_ref().unwrap()), 30_000);
    assert_eq!(c.lang.as_deref(), Some("en-US"));
    assert_eq!(c.alt_lang.as_deref(), Some("de-DE"));
    assert_eq!(c.smart_tag_id, Some(7));
    assert_eq!(c.bookmark.as_deref(), Some("mark"));
    let line = c.line.as_ref().unwrap();
    assert_eq!(line.width, Some(12700));
    assert!(matches!(line.fill, Some(Fill::Solid(Some(_)))));
    assert!(matches!(c.fill, Some(Fill::Solid(Some(_)))));
    assert!(matches!(c.effects.as_deref(), Some(EffectProperties::List(e)) if e.len() == 1));
    assert!(matches!(c.highlight, Some(Some(_))));
    assert_eq!(c.underline_line, Some(UnderlineLine::FollowText));
    assert_eq!(c.underline_fill, Some(UnderlineFill::FollowText));
    let latin = c.latin.as_ref().unwrap();
    assert_eq!(
        (
            latin.typeface.as_str(),
            latin.panose.as_deref(),
            latin.pitch_family,
            latin.charset
        ),
        ("Arial", Some("020F0502020204030204"), Some(34), Some(0))
    );
    assert_eq!(c.east_asian.as_ref().unwrap().typeface, "+mn-ea");
    assert_eq!(c.symbol.as_ref().unwrap().typeface, "Wingdings");
    let click = c.hyperlink_click.as_ref().unwrap();
    assert_eq!(click.rel_id.as_deref(), Some("rId2"));
    assert_eq!(click.tooltip.as_deref(), Some("Go"));
    assert_eq!(click.target_frame.as_deref(), Some("_blank"));
    assert_eq!(click.history, Some(false));
    let hover = c.hyperlink_mouse_over.as_ref().unwrap();
    assert_eq!(hover.rel_id.as_deref(), Some(""));
    assert_eq!(hover.action.as_deref(), Some("ppaction://noaction"));
    assert_eq!(c.rtl.unwrap().value, Some(true));
    assert_eq!(br.properties.as_ref().unwrap().size, Some(1100));
    assert_eq!(field.field_type.as_deref(), Some("slidenum"));
    assert_eq!(field.text.as_deref(), Some("1"));
    assert_eq!(field.properties.as_ref().unwrap().size, Some(1200));
    assert!(field.paragraph_properties.is_some());
    assert_eq!(p.end_properties.as_ref().unwrap().dirty, Some(false));

    // Preserved, not modelled, and said so.
    for element in ["scene3d", "sp3d", "extLst", "avLst"] {
        assert!(
            s.findings.has(K::RetainedElement, element),
            "{element}: {:?}",
            s.findings
        );
    }

    // Each autofit, exactly.
    let fit = |name: &str| {
        body_of(slide, name)
            .properties
            .as_ref()
            .unwrap()
            .autofit
            .clone()
    };
    assert_eq!(fit("No Autofit"), Some(Autofit::None));
    assert_eq!(fit("Shape Autofit"), Some(Autofit::Shape));
    assert_eq!(
        fit("Bare Autofit"),
        Some(Autofit::Normal {
            font_scale: None,
            line_spacing_reduction: None,
        }),
        "absent stays absent: not 100%, not 0%"
    );
    let Some(Autofit::Normal {
        font_scale: Some(scale),
        line_spacing_reduction: Some(reduction),
    }) = fit("Percent Autofit")
    else {
        panic!("Percent Autofit");
    };
    assert_eq!(
        (scale.written(), reduction.written()),
        (Some("62.5%"), Some("10%"))
    );
    assert_eq!(
        (percent_value(&scale), percent_value(&reduction)),
        (62_500, 10_000)
    );

    // Each fill, through Tier 1E's reader, with what it holds.
    let fills: Vec<_> = body_of(slide, "Fills").paragraphs[0]
        .items
        .iter()
        .map(|item| match item {
            TextItem::Run(r) => r.properties.as_ref().unwrap().fill.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(fills[0], Some(Fill::None));
    let Some(Fill::Gradient(gradient)) = &fills[1] else {
        panic!("{:?}", fills[1]);
    };
    assert_eq!(gradient.stops.as_ref().map(Vec::len), Some(2));
    assert!(matches!(fills[2], Some(Fill::Blip(_))));
    assert!(matches!(fills[3], Some(Fill::Pattern(_))));
    assert_eq!(fills[4], Some(Fill::Group));
    assert_eq!(fills[5], None);
    let TextItem::Run(under) = &body_of(slide, "Fills").paragraphs[0].items[5] else {
        panic!("not a run");
    };
    let under = under.properties.as_ref().unwrap();
    assert!(matches!(&under.underline_line, Some(UnderlineLine::Line(l)) if l.width == Some(9525)));
    assert!(matches!(
        under.effects.as_deref(),
        Some(EffectProperties::Dag(_))
    ));
    assert!(matches!(
        under.underline_fill,
        Some(UnderlineFill::Fill(Some(Fill::Solid(Some(_)))))
    ));

    // Each bullet.
    let bullets: Vec<_> = body_of(slide, "Bullets")
        .paragraphs
        .iter()
        .map(|p| p.properties.as_ref().unwrap().clone())
        .collect();
    assert_eq!(bullets[0].bullet, Some(Bullet::None));
    let Some(Bullet::AutoNumber { start_at, .. }) = &bullets[1].bullet else {
        panic!("buAutoNum");
    };
    assert_eq!(*start_at, Some(3));
    assert_eq!(
        bullets[3].bullet,
        Some(Bullet::Picture {
            embed: Some("rId3".to_owned())
        })
    );
    assert_eq!(bullets[4].bullet_color, Some(BulletColor::FollowText));
    assert_eq!(bullets[4].bullet_size, Some(BulletSize::FollowText));
    assert_eq!(bullets[4].bullet_font, Some(BulletFont::FollowText));
    assert_eq!(bullets[5].bullet_size, Some(BulletSize::Points(1400)));

    assert!(s.parts.get(0).is_some());
    let bytes = std::fs::read(fixtures().join("generated/text-every-property.pptx")).unwrap();
    let mut pkg = PresentationPackage::open_strict(bytes, OoxmlLimits::default()).unwrap();
    let imported = import_deck(&mut pkg, &ImportLimits::default()).unwrap();
    let default = imported.deck.presentation.default_text_style.unwrap();
    assert!(default.default.is_some());
    assert_eq!(
        default.levels[0]
            .as_ref()
            .unwrap()
            .default_run
            .as_ref()
            .unwrap()
            .size,
        Some(1800)
    );
    assert!(
        !imported
            .findings
            .has(K::RetainedElement, "defaultTextStyle")
    );
}

#[test]
fn text_is_kept_exactly_and_an_empty_one_is_not_absent() {
    let s = deck("generated/text-every-property.pptx");
    let slide = s.parts.slide(0).unwrap();
    let body = body_of(slide, "Whitespace");
    let runs: Vec<_> = body.paragraphs[0]
        .items
        .iter()
        .map(|i| match i {
            TextItem::Run(r) => r.text.clone(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        runs,
        [Some("  two & <spaced> ".to_owned()), Some(String::new())]
    );
    assert_eq!(body.paragraphs[1].items, []);
    assert_eq!(body.paragraphs[1].properties, None);
    assert_eq!(body.list_style, None, "no lstStyle is not an empty one");
}

#[test]
fn an_equation_in_a_paragraph_stays_in_its_place_and_is_reported() {
    let s = deck("generated/text-every-property.pptx");
    let slide = s.parts.slide(0).unwrap();
    let items = &body_of(slide, "Equation").paragraphs[0].items;
    assert_eq!(items.len(), 3);
    assert_eq!(items[1], TextItem::Retained("AlternateContent".to_owned()));
    assert!(s.findings.has(K::RetainedElement, "AlternateContent"));
}

#[test]
fn absent_stays_absent_in_every_text_struct() {
    let (body, findings, _) = read("<a:bodyPr/><a:p><a:pPr/><a:r><a:rPr/><a:t>x</a:t></a:r></a:p>");
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(body.properties, Some(Default::default()));
    let p = &body.paragraphs[0];
    assert_eq!(
        p.properties.as_deref(),
        Some(&ParagraphProperties::default())
    );
    let TextItem::Run(run) = &p.items[0] else {
        panic!("not a run");
    };
    assert_eq!(
        run.properties.as_deref(),
        Some(&CharacterProperties::default())
    );
    assert_eq!(p.end_properties, None);
}

#[test]
fn master_text_styles_and_the_notes_style_hold_their_levels() {
    let s = deck("generated/text-inheritance.pptx");
    let master = s.parts.get(s.parts.structure.masters[0]).unwrap();
    let styles = master.text_styles.as_ref().unwrap();
    let size = |l: &casual_slide_model::text::TextListStyle, at: usize| {
        l.levels[at]
            .as_ref()
            .unwrap()
            .default_run
            .as_ref()
            .unwrap()
            .size
    };
    assert_eq!(size(styles.title.as_ref().unwrap(), 1), Some(2001));
    assert_eq!(size(styles.body.as_ref().unwrap(), 0), Some(2100));
    assert_eq!(size(styles.other.as_ref().unwrap(), 1), Some(2201));
    let rich = deck("generated/slides-masters-layouts.pptx");
    let notes = rich.parts.get(rich.parts.structure.notes_master.unwrap());
    assert!(notes.unwrap().notes_style.is_some());
}

#[test]
fn a_percentage_in_either_form_means_the_same() {
    for value in ["62500", "62.5%"] {
        let (body, findings, _) = read(&format!(
            r#"<a:bodyPr><a:normAutofit fontScale="{value}"/></a:bodyPr><a:p/>"#
        ));
        assert!(findings.is_empty(), "{findings:?}");
        let Some(Autofit::Normal { font_scale, .. }) = body.properties.unwrap().autofit else {
            panic!("no autofit");
        };
        let scale = font_scale.unwrap();
        assert_eq!(scale.written(), Some(value), "stored as written");
        assert_eq!(percent_value(&scale), 62_500);
    }
}

#[test]
fn a_value_out_of_its_range_is_reported_and_left_out() {
    for (body, attribute) in [
        (r#"<a:bodyPr numCol="17"/><a:p/>"#, "numCol"),
        (r#"<a:bodyPr numCol="0"/><a:p/>"#, "numCol"),
        (r#"<a:bodyPr spcCol="-1"/><a:p/>"#, "spcCol"),
        (r#"<a:bodyPr lIns="9999999999"/><a:p/>"#, "lIns"),
        (
            r#"<a:bodyPr><a:normAutofit fontScale="999"/></a:bodyPr><a:p/>"#,
            "fontScale",
        ),
        (
            r#"<a:bodyPr><a:normAutofit fontScale="100001"/></a:bodyPr><a:p/>"#,
            "fontScale",
        ),
        (r#"<a:bodyPr/><a:p><a:pPr lvl="9"/></a:p>"#, "lvl"),
        (r#"<a:bodyPr/><a:p><a:pPr marL="-1"/></a:p>"#, "marL"),
        (
            r#"<a:bodyPr/><a:p><a:pPr indent="51206401"/></a:p>"#,
            "indent",
        ),
        (
            r#"<a:bodyPr/><a:p><a:pPr><a:buAutoNum type="arabicPlain" startAt="32768"/></a:pPr></a:p>"#,
            "startAt",
        ),
        (
            r#"<a:bodyPr/><a:p><a:pPr><a:buSzPct val="24999"/></a:pPr></a:p>"#,
            "val",
        ),
        (
            r#"<a:bodyPr/><a:p><a:pPr><a:buSzPts val="99"/></a:pPr></a:p>"#,
            "val",
        ),
        (
            r#"<a:bodyPr/><a:p><a:pPr><a:lnSpc><a:spcPts val="158401"/></a:lnSpc></a:pPr></a:p>"#,
            "val",
        ),
        (
            r#"<a:bodyPr/><a:p><a:r><a:rPr sz="99"/><a:t>x</a:t></a:r></a:p>"#,
            "sz",
        ),
        (
            r#"<a:bodyPr/><a:p><a:r><a:rPr kern="400001"/><a:t>x</a:t></a:r></a:p>"#,
            "kern",
        ),
        (
            r#"<a:bodyPr/><a:p><a:r><a:rPr spc="400001"/><a:t>x</a:t></a:r></a:p>"#,
            "spc",
        ),
    ] {
        let (read_body, findings, _) = read(body);
        assert!(
            findings
                .iter()
                .any(|f| matches!(f.kind, K::OutOfRange | K::MalformedValue)
                    && f.attribute.as_deref() == Some(attribute)),
            "{body}: {findings:?}"
        );
        // Left out: the same as if the file had not said it.
        let start = body.find(&format!(r#" {attribute}=""#)).unwrap();
        let open = start + attribute.len() + 3;
        let end = open + body[open..].find('"').unwrap() + 1;
        let (without, _, _) = read(&format!("{}{}", &body[..start], &body[end..]));
        assert_eq!(read_body, without, "{body}");
    }
}

#[test]
fn hostile_text_refuses_with_its_code() {
    let code = |path: &str| {
        deck_with(path, &ImportLimits::default())
            .map(|_| ())
            .unwrap_err()
            .code()
    };
    assert_eq!(
        code("hostile/text-limit-paragraphs-in-body.bin"),
        "CS-TXT-0001"
    );
    assert_eq!(
        code("hostile/text-limit-runs-in-paragraph.bin"),
        "CS-TXT-0002"
    );
    assert_eq!(
        code("hostile/text-limit-paragraphs-in-part.bin"),
        "CS-TXT-0003"
    );
    assert_eq!(code("hostile/text-limit-tab-stops.bin"), "CS-TXT-0005");

    // One past a bound refuses, and at the bound it does not.
    let at = |limits: TextLimits| ImportLimits {
        text: limits,
        ..ImportLimits::default()
    };
    let paragraphs = at(TextLimits {
        max_paragraphs_per_body: 100_001,
        ..TextLimits::default()
    });
    assert!(deck_with("hostile/text-limit-paragraphs-in-body.bin", &paragraphs).is_ok());
    let runs = at(TextLimits {
        max_runs_per_paragraph: 100_001,
        ..TextLimits::default()
    });
    assert!(deck_with("hostile/text-limit-runs-in-paragraph.bin", &runs).is_ok());
    let part = at(TextLimits {
        max_paragraphs_per_part: 270_000,
        ..TextLimits::default()
    });
    assert!(deck_with("hostile/text-limit-paragraphs-in-part.bin", &part).is_ok());
    let tabs = at(TextLimits {
        max_tab_stops: 1_025,
        ..TextLimits::default()
    });
    assert!(deck_with("hostile/text-limit-tab-stops.bin", &tabs).is_ok());

    // The default is too large for a committed deck, so the bound comes down to it.
    let tight = at(TextLimits {
        max_runs_per_part: 100_000,
        max_runs_per_paragraph: 100_001,
        ..TextLimits::default()
    });
    let err = deck_with("hostile/text-limit-runs-in-paragraph.bin", &tight).unwrap_err();
    assert_eq!(err.code(), "CS-TXT-0004");
    assert_eq!(TextBound::RunsInPart.code(), "CS-TXT-0004");
    let message = err.to_string();
    assert!(
        message.contains("slide1.xml") && message.contains("100000"),
        "{message}"
    );
}

#[test]
fn more_tab_stops_than_the_schema_allows_are_kept_and_reported() {
    let tabs = "<a:tab/>".repeat(33);
    let (body, findings, _) = read(&format!(
        "<a:bodyPr/><a:p><a:pPr><a:tabLst>{tabs}</a:tabLst></a:pPr></a:p>"
    ));
    assert_eq!(first_paragraph(&body).tabs.as_ref().unwrap().len(), 33);
    assert!(findings.has(K::OutOfRange, "tabLst"), "{findings:?}");
}

#[test]
fn bad_values_are_reported_never_a_panic_and_stay_in_the_tree() {
    let s = deck("hostile/text-bad-values.bin");
    let slide = s.parts.slide(0).unwrap();
    let tree = slide.shape_tree().unwrap();
    let body = tree.children[0].text_body.as_ref().unwrap();
    let props = body.properties.as_ref().unwrap();
    assert_eq!(
        (props.rotation, props.columns, props.vertical, props.wrap),
        (None, None, None, None)
    );
    assert_eq!(
        props.warp.as_ref().unwrap().preset,
        None,
        "the first of two, with no prst"
    );
    assert!(s.findings.has(K::MissingRequired, "prstTxWarp"));
    for (kind, element) in [
        (K::DuplicateElement, "bodyPr"),
        (K::DuplicateElement, "lstStyle"),
        (K::DuplicateElement, "noAutofit"),
        (K::DuplicateElement, "lvl1pPr"),
        (K::DuplicateElement, "defPPr"),
        (K::DuplicateElement, "pPr"),
        (K::DuplicateElement, "t"),
        (K::DuplicateElement, "endParaRPr"),
        (K::DuplicateElement, "defRPr"),
        (K::DuplicateElement, "lnSpc"),
        (K::MissingRequired, "r"),
        (K::MissingRequired, "fld"),
        (K::DuplicateElement, "buSzPts"),
        (K::MissingRequired, "latin"),
        (K::MalformedValue, "t"),
        (K::DanglingRelationship, "hlinkClick"),
        (K::RetainedAttribute, "bodyPr"),
        (K::RetainedAttribute, "rPr"),
        (K::RetainedAttribute, "br"),
        (K::RetainedElement, "unknown"),
        (K::MissingRequired, "txBody"),
    ] {
        assert!(
            s.findings.has(kind, element),
            "{kind:?} {element}: {:?}",
            s.findings
        );
    }
    assert!(
        s.color_findings.total > 0,
        "a colour with no value keeps its own code"
    );
    assert!(s.color_findings.codes().contains(&"CS-CLR-0007"));
}

/// `SEC-015` for text: what a document holds in a text body stays out of every finding
/// and every error, and the bad-values deck plants a marker wherever text can stand.
#[test]
fn no_message_or_finding_carries_text_from_a_body() {
    const MARK: &str = "QzMarkerQz";
    let s = deck("hostile/text-bad-values.bin");
    assert!(!s.findings.is_empty());
    let report = format!("{:?} {:?}", s.findings, s.color_findings);
    assert!(!report.contains(MARK), "{report}");

    // The marker is there to be kept: the model holds what it could read.
    let model = format!("{:?}", s.parts.slide(0).unwrap());
    assert!(model.contains(MARK), "the plant did not take");

    // And on real decks, by their own words: any run's text with a space in it or
    // beyond ASCII is document content, and is in no report.
    let mut phrases = 0;
    for entry in std::fs::read_dir(fixtures().join("real")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "pptx") {
            continue;
        }
        let name = format!("real/{}", path.file_name().unwrap().to_string_lossy());
        let s = deck(&name);
        let report = format!("{:?} {:?}", s.findings, s.color_findings);
        for part in s.parts.parts.iter().flatten() {
            for node in part
                .shape_tree()
                .into_iter()
                .flat_map(ShapeNode::descendants)
            {
                for p in node.text_body.iter().flat_map(|b| &b.paragraphs) {
                    for item in &p.items {
                        let TextItem::Run(run) = item else { continue };
                        let Some(text) = run.text.as_deref() else {
                            continue;
                        };
                        if text.len() > 8 && (text.contains(' ') || !text.is_ascii()) {
                            phrases += 1;
                            assert!(!report.contains(text), "{name}: run text in the report");
                        }
                    }
                }
            }
        }
    }
    assert!(phrases > 1000, "{phrases} phrases");
}

#[test]
fn every_deck_reads_its_text() {
    let mut runs = 0usize;
    let mut bodies = 0usize;
    let mut decks = 0;
    for set in ["generated", "producers", "real"] {
        for entry in std::fs::read_dir(fixtures().join(set)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "tsv" || e == "md") {
                continue;
            }
            let name = format!("{set}/{}", path.file_name().unwrap().to_string_lossy());
            let s = deck(&name);
            decks += 1;
            if set == "real" {
                // Names only, from the report: what real decks hold that is unmodelled.
                let mut seen = std::collections::BTreeSet::new();
                for f in s.findings.iter() {
                    if !matches!(f.kind, K::RetainedAttribute | K::RetainedElement) {
                        seen.insert((f.kind, f.element.clone(), f.attribute.clone()));
                    }
                }
                eprintln!("{name}: {seen:?}; colour {:?}", s.color_findings.codes());
            }
            for part in s.parts.parts.iter().flatten() {
                for node in part
                    .shape_tree()
                    .into_iter()
                    .flat_map(ShapeNode::descendants)
                {
                    if let Some(body) = &node.text_body {
                        bodies += 1;
                        runs += body.paragraphs.iter().map(|p| p.items.len()).sum::<usize>();
                    }
                }
            }
        }
    }
    assert!(decks >= 35, "{decks}");
    assert!(bodies > 1000 && runs > 1000, "{bodies} bodies, {runs} runs");
    eprintln!("{decks} decks, {bodies} text bodies, {runs} runs, breaks and fields");
}

/// `docs/21` states what a run costs at the bound; this holds the figure.
#[test]
fn a_run_costs_what_the_limits_page_says() {
    assert!(
        size_of::<CharacterProperties>() <= 400,
        "{}",
        size_of::<CharacterProperties>()
    );
    assert!(size_of::<TextItem>() <= 96, "{}", size_of::<TextItem>());
    assert!(
        size_of::<ParagraphProperties>() <= 360,
        "{}",
        size_of::<ParagraphProperties>()
    );
}

/// Nothing a shape's text holds is dropped on the way to the model: each part's runs,
/// breaks, fields and paragraphs, counted in the tree without the importer, are all there.
/// A part with an `mc:AlternateContent` among its shapes is skipped, since only one
/// branch's shapes are read.
#[test]
fn every_paragraph_and_run_in_a_shapes_text_reaches_the_model() {
    use casual_slide_ooxml::ElementId;
    use std::collections::HashMap;
    fn count(
        tree: &XmlTree,
        at: ElementId,
        in_body: bool,
        totals: &mut HashMap<&'static str, usize>,
    ) -> bool {
        let name = tree.qualified_name(at).to_owned();
        let local = tree.local_name(at).to_owned();
        if local == "AlternateContent" && !in_body {
            return false;
        }
        if local == "graphicFrame"
            || local == "timing"
            || local == "txStyles"
            || local == "notesStyle"
        {
            return true;
        }
        let body = in_body || (local == "txBody" && name.starts_with("p:"));
        if in_body {
            match local.as_str() {
                "p" => *totals.entry("p").or_default() += 1,
                "r" | "br" | "fld" => *totals.entry("r").or_default() += 1,
                "AlternateContent" => {
                    *totals.entry("r").or_default() += 1;
                    return true;
                }
                _ => {}
            }
        }
        tree.child_elements(at)
            .into_iter()
            .all(|c| count(tree, c, body, totals))
    }
    let mut compared = 0usize;
    for entry in std::fs::read_dir(fixtures().join("real")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "pptx") {
            continue;
        }
        let name = format!("real/{}", path.file_name().unwrap().to_string_lossy());
        let s = deck(&name);
        let mut pkg =
            PresentationPackage::open_strict(std::fs::read(&path).unwrap(), OoxmlLimits::default())
                .unwrap();
        for entry in &s.parts.structure.parts {
            let Some(part) = s.parts.get(entry.id.part) else {
                continue;
            };
            let bytes = pkg.read_part(&entry.part).unwrap();
            let tree = XmlTree::parse(&bytes, &entry.part, &OoxmlLimits::default()).unwrap();
            let mut raw = HashMap::new();
            if !count(&tree, tree.root(), false, &mut raw) {
                continue;
            }
            let (mut paragraphs, mut runs) = (0, 0);
            for node in part
                .shape_tree()
                .into_iter()
                .flat_map(ShapeNode::descendants)
            {
                for p in node.text_body.iter().flat_map(|b| &b.paragraphs) {
                    paragraphs += 1;
                    runs += p.items.len();
                }
            }
            assert_eq!(
                raw.get("p").copied().unwrap_or(0),
                paragraphs,
                "{name} {}",
                entry.part
            );
            assert_eq!(
                raw.get("r").copied().unwrap_or(0),
                runs,
                "{name} {}",
                entry.part
            );
            compared += runs;
        }
    }
    assert!(compared > 2000, "{compared}");
}

#[test]
fn a_universal_measure_on_spc_keeps_its_unit() {
    let (body, findings, _) =
        read(r#"<a:bodyPr/><a:p><a:r><a:rPr spc="-1.5pt"/><a:t>x</a:t></a:r></a:p>"#);
    assert!(findings.is_empty(), "{findings:?}");
    let spacing = first_run(&body).spacing.as_ref().unwrap();
    assert_eq!(
        (spacing.value().0, spacing.written()),
        (-150, Some("-1.5pt"))
    );
}
