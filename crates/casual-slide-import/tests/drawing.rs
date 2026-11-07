//! Tier 1E reading: every element and attribute of the areas, hostile values and limits,
//! and what real decks hold. Edits are tested in `casual-slide-export/tests/drawing.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use casual_slide_import::drawing::{DrawingFinding as D, DrawingLimits, read_fill};
use casual_slide_import::geometry::{compile_custom, read_preset_text_shape};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{
    Elem, Findings, ImportFindingKind as K, ImportLimits, Reader,
};
use casual_slide_import::slide::{SlideLimits, import_slide_parts};
use casual_slide_model::color::Percentage;
use casual_slide_model::d3::FlatText;
use casual_slide_model::drawing::ShapeProperties;
use casual_slide_model::effect::{Effect, EffectProperties, PictureEffectKind};
use casual_slide_model::fill::Fill;
use casual_slide_model::geometry::{Geometry, TextShapeType};
use casual_slide_model::line::{LineDash, LineJoin, LineSketchType};
use casual_slide_model::presentation::Coordinate;
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::ShapeNode;
use casual_slide_model::slide::{SlideKind, SlidePart};
use casual_slide_model::text::{Autofit, Text3D};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};

fn lex<T: Lexical>(v: Option<&T>) -> Option<String> {
    v.map(Lexical::to_lexical)
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

struct Deck {
    parts: Vec<SlidePart>,
    findings: Findings,
}

fn load_bytes(bytes: Vec<u8>) -> Deck {
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    let deck = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    let s = import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    Deck {
        parts: s.parts.parts.iter().flatten().cloned().collect(),
        findings: s.findings.clone(),
    }
}

fn load(path: &str) -> Deck {
    load_bytes(std::fs::read(repo().join("fixtures").join(path)).unwrap())
}

impl Deck {
    fn slide(&self) -> &SlidePart {
        self.parts
            .iter()
            .find(|p| p.kind == SlideKind::Slide)
            .unwrap()
    }

    fn named(&self, name: &str) -> &ShapeNode {
        self.slide()
            .shape_tree()
            .unwrap()
            .descendants()
            .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some(name))
            .unwrap_or_else(|| panic!("no {name}"))
    }

    fn first(&self) -> &ShapeProperties {
        &self.named("s").properties
    }
}

/// Rows the fixture does not hold, each with why. Everything else in the areas' rows must
/// be in the fixture and read with nothing retained.
fn elsewhere(id: &str) -> Option<&'static str> {
    const OTHER_TIERS: [(&str, &str); 14] = [
        (
            "CT_BackgroundFormatting",
            "a background is Tier 1D's model, and T1C-02",
        ),
        (
            "CT_WholeE2oFormatting",
            "table cell and theme object defaults, T1C-02",
        ),
        ("CT_FlatText", "a body's: generated/text-warp-and-3d.pptx"),
        ("EG_Text3D", "a body's: generated/text-warp-and-3d.pptx"),
        (
            "CT_PresetTextShape",
            "a body's: generated/text-warp-and-3d.pptx",
        ),
        (
            "EG_TextGeometry",
            "a body's: generated/text-warp-and-3d.pptx",
        ),
        (
            "CT_Connection",
            "stCxn and endCxn: generated/text-object-defaults.pptx",
        ),
        ("CT_TextMath", "a14:m, equations: Tier 1K"),
        ("CT_ContentPartLocking", "a14 content parts: Tier 1K"),
        (
            "CT_NonVisualInkContentPartProperties",
            "a14 content parts: Tier 1K",
        ),
        ("CT_GvmlContentPart", "a14 content parts: Tier 1K"),
        (
            "CT_CameraTool",
            "a spreadsheet's camera tool: not in a slide part",
        ),
        ("CT_CompatExt", "a14:compatExt: OLE, Tier 1H"),
        ("CT_IsGvmlCanvas", "a14:isCanvas: Word, not a slide part"),
    ];
    OTHER_TIERS
        .iter()
        .find(|(t, _)| id.contains(t))
        .map(|(_, why)| *why)
        .or_else(|| {
            [
                "@legacySpreadsheetColorIndex",
                "ST_LegacySpreadsheetColorIndex",
                "<m>",
                "<cameraTool>",
                "<compatExt>",
                "<isCanvas>",
                "<contentPart>",
            ]
            .iter()
            .any(|n| id.contains(n))
            .then_some("Excel and Word roots in the a14 schema, or later tiers")
        })
}

#[test]
fn every_element_and_attribute_row_in_the_areas_is_in_the_fixture_and_read() {
    let d = load("generated/drawing-every-element.pptx");
    let retained: Vec<_> = d
        .findings
        .iter()
        .filter(|f| f.part.contains("slides/slide"))
        .collect();
    assert!(retained.is_empty(), "{retained:#?}");

    let mut pkg = PresentationPackage::open(
        std::fs::read(repo().join("fixtures/generated/drawing-every-element.pptx")).unwrap(),
        OoxmlLimits::default(),
    )
    .unwrap();
    let xml = String::from_utf8(pkg.read_part("ppt/slides/slide1.xml").unwrap()).unwrap();

    let text = std::fs::read_to_string(repo().join("docs/spec/checklist.tsv")).unwrap();
    let (mut checked, mut skipped) = (0, 0);
    for line in text.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        let (Some(id), Some(area), Some(kind)) = (cols.first(), cols.get(1), cols.get(3)) else {
            continue;
        };
        if !matches!(*area, "GEOM" | "FILL" | "LINE" | "EFFECT" | "D3") {
            continue;
        }
        if elsewhere(id).is_some() {
            skipped += 1;
            continue;
        }
        let tail = id.rsplit('/').next().unwrap();
        match *kind {
            "attribute" => {
                let name = tail.strip_prefix('@').unwrap();
                let name = name.rsplit(':').next().unwrap();
                assert!(
                    xml.contains(&format!(" {name}=\"")) || xml.contains(&format!(":{name}=\"")),
                    "{id} is not in the fixture"
                );
            }
            "element" => {
                let name = tail.rsplit(':').next().unwrap();
                // In the schema or the extension, under whatever prefix.
                let found = [":", ""].iter().any(|_| {
                    xml.contains(&format!(":{name} "))
                        || xml.contains(&format!(":{name}>"))
                        || xml.contains(&format!(":{name}/>"))
                });
                assert!(found, "{id} is not in the fixture");
            }
            _ => continue,
        }
        checked += 1;
    }
    assert!(checked > 300, "{checked}");
    assert!(skipped > 0 && skipped < 80, "{skipped}");
}

#[test]
fn the_fixture_reads_as_written_for_a_sample_of_each_area() {
    let d = load("generated/drawing-every-element.pptx");
    let line = d
        .named("solid and full line")
        .properties
        .line
        .clone()
        .unwrap();
    assert_eq!(line.width, Some(12_700));
    assert_eq!(line.join, Some(LineJoin::Round));
    let sketch = line.sketch.unwrap();
    assert_eq!(sketch.seed_attribute, Some(12_345));
    assert_eq!(sketch.sketch_type, Some(Some(LineSketchType::Scribble)));
    assert!(matches!(sketch.geometry, Some(Geometry::Preset(_))));

    let dashed = d.named("linear gradient").properties.line.clone().unwrap();
    let Some(LineDash::Custom(stops)) = dashed.dash else {
        panic!()
    };
    assert_eq!(stops[1].space, Some(Percentage(60_000)));

    let Some(Geometry::Custom(custom)) = d.named("custom").properties.geometry.clone() else {
        panic!()
    };
    assert_eq!(custom.paths.as_ref().unwrap()[0].commands.len(), 6);
    let compiled = compile_custom(&custom).unwrap();
    assert!(
        compiled.diagnostics.is_empty(),
        "{:?}",
        compiled.diagnostics
    );

    let Some(Fill::Blip(fill)) = d.named("stretched picture").properties.fill.clone() else {
        panic!()
    };
    let blip = fill.blip.unwrap();
    assert_eq!(blip.effects.len(), 17, "every CT_Blip effect");
    let layer = blip.image_properties.unwrap().layer.unwrap();
    assert_eq!(layer.embed.as_deref(), Some("rId2"));
    assert_eq!(layer.effects.len(), 27);
    assert!(layer.effects.iter().all(|e| e.kind.is_some()));
    assert!(matches!(
        layer.effects[26].kind,
        Some(PictureEffectKind::BackgroundRemoval(_))
    ));
    assert_eq!(blip.use_local_dpi, Some(false));

    let Some(EffectProperties::List(list)) =
        d.named("effect list and 3D").properties.effects.clone()
    else {
        panic!()
    };
    assert_eq!(list.len(), 8);
    let Some(EffectProperties::Dag(dag)) = d.named("effect dag").properties.effects.clone() else {
        panic!()
    };
    assert_eq!(dag.effects.len(), 30, "every EG_Effect");
    assert_eq!(dag.name.as_deref(), Some("top"));

    let hidden = d.named("hidden").properties.hidden.clone().unwrap();
    assert!(hidden.fill.is_some() && hidden.line.is_some() && hidden.effects.is_some());
    assert!(hidden.scene3d.is_some() && hidden.sp3d.is_some());
    assert_eq!(hidden.shadow_obscured, Some(true));

    let group = &d.named("group").properties;
    assert!(group.fill.is_some() && group.effects.is_some() && group.scene3d.is_some());
    assert_eq!(d.named("in group").properties.fill, Some(Fill::Group));
    assert!(d.named("picture").properties.picture.is_some());
    assert!(d.named("connector").properties.line.is_some());
}

#[test]
fn absent_stays_absent_and_a_schema_default_is_never_filled_in() {
    let d = load("generated/drawing-every-element.pptx");
    let p = &d.named("in group").properties;
    assert_eq!(p.line, None);
    assert_eq!(p.effects, None);
    assert_eq!(p.scene3d, None);
    assert_eq!(p.sp3d, None);
    assert_eq!(p.hidden, None);
    let Some(Geometry::Preset(g)) = p.geometry.as_ref() else {
        panic!()
    };
    assert_eq!(
        g.adjust.as_deref(),
        Some(&[][..]),
        "an empty avLst is not an absent one"
    );
    let Some(EffectProperties::Dag(dag)) = d.named("effect dag").properties.effects.clone() else {
        panic!()
    };
    let Effect::Container(inner) = &dag.effects[0] else {
        panic!()
    };
    assert_eq!(inner.effects, [Effect::AlphaFloor]);
    let Effect::Blur { grow, .. } = &dag.effects[12] else {
        panic!("{:?}", dag.effects[12])
    };
    assert_eq!(*grow, Some(true));
    let Effect::InnerShadow {
        distance,
        direction,
        ..
    } = &dag.effects[21]
    else {
        panic!("{:?}", dag.effects[21])
    };
    assert_eq!((*distance, *direction), (None, None), "not 0");
}

#[test]
fn a_list_over_its_limit_leaves_the_property_out_and_reports_it() {
    let over = K::Drawing(D::OverLimit);
    for (file, element, gone) in [
        (
            "drawing-limit-gradient-stops",
            "gradFill",
            (|p: &ShapeProperties| p.fill.is_none()) as fn(&ShapeProperties) -> bool,
        ),
        ("drawing-limit-dash-stops", "ln", |p| p.line.is_none()),
        ("drawing-limit-effects", "effectDag", |p| {
            p.effects.is_none()
        }),
        ("drawing-limit-effect-depth", "effectDag", |p| {
            p.effects.is_none()
        }),
        ("drawing-limit-fill-depth", "blipFill", |p| p.fill.is_none()),
        ("drawing-limit-guides", "custGeom", |p| p.geometry.is_none()),
        ("drawing-limit-paths", "custGeom", |p| p.geometry.is_none()),
        ("drawing-limit-path-commands", "custGeom", |p| {
            p.geometry.is_none()
        }),
        ("drawing-limit-handles", "custGeom", |p| {
            p.geometry.is_none()
        }),
        ("drawing-limit-sites", "custGeom", |p| p.geometry.is_none()),
    ] {
        let d = load(&format!("hostile/{file}.bin"));
        assert!(d.findings.has(over, element), "{file}: {:?}", d.findings);
        assert!(gone(d.first()), "{file}");
    }
    assert_eq!(over.code(), "CS-DRW-0001");
}

#[test]
fn one_under_each_limit_is_read() {
    // The limit is the default's own number: one under reads, and the fixtures above go
    // one over. A smaller limit, here, shows the bound is the field and not a constant.
    let xml = br#"<a:gradFill xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:gsLst><a:gs pos="0"><a:srgbClr val="000000"/></a:gs><a:gs pos="1"><a:srgbClr val="000000"/></a:gs></a:gsLst></a:gradFill>"#;
    let tree = XmlTree::parse(xml, "x", &OoxmlLimits::default()).unwrap();
    let import = ImportLimits::default();
    let read = |max: usize| {
        let mut findings = Findings::new(100);
        let mut r = Reader::new(&tree, "x", &import, &mut findings);
        let root = Elem::root(&tree, "x").unwrap();
        let limits = DrawingLimits {
            max_gradient_stops: max,
            ..DrawingLimits::default()
        };
        read_fill(&mut r, &[], limits, &root).unwrap()
    };
    assert!(read(2).is_some());
    assert!(read(1).is_none());
}

#[test]
fn hostile_values_are_reported_and_left_out_never_a_panic() {
    let d = load("hostile/drawing-bad-values.bin");
    let f = &d.findings;
    for (kind, element, attribute) in [
        (K::MalformedValue, "prstGeom", "prst"),
        (K::MissingRequired, "gd", "name"),
        (K::OutOfRange, "gs", "pos"),
        (K::MalformedValue, "gs", "pos"),
        (K::MissingRequired, "gs", "pos"),
        (K::MalformedValue, "gradFill", "flip"),
        (K::OutOfRange, "lin", "ang"),
        (K::OutOfRange, "ln", "w"),
        (K::MalformedValue, "ln", "cap"),
        (K::MalformedValue, "prstDash", "val"),
        (K::MalformedValue, "headEnd", "type"),
        (K::OutOfRange, "outerShdw", "blurRad"),
        (K::OutOfRange, "outerShdw", "kx"),
        (K::MalformedValue, "camera", "prst"),
        (K::OutOfRange, "camera", "fov"),
        (K::MissingRequired, "rot", "lon"),
        (K::MalformedValue, "lightRig", "rig"),
        (K::OutOfRange, "bevelT", "w"),
        (K::MalformedValue, "sp3d", "prstMaterial"),
        (K::MissingRequired, "softEdge", "rad"),
        (K::MissingRequired, "fillOverlay", "blend"),
    ] {
        let found = f.iter().any(|x| {
            x.kind == kind && x.element == element && x.attribute.as_deref() == Some(attribute)
        });
        assert!(found, "{kind:?} {element} {attribute}: {f:#?}");
    }
    for element in [
        // `unknownThing` and `foreign` are names the part made up (`SEC-018`).
        "(unknown)",
        "blipFill",
        "tint",
        "extrusionClr",
    ] {
        assert!(
            f.iter().any(|x| x.element.ends_with(element)),
            "{element} is not named: {f:#?}"
        );
    }
    assert!(f.has(K::DuplicateElement, "miter"), "{f:#?}");
    assert!(f.has(K::DuplicateElement, "effectDag"), "{f:#?}");
    assert!(f.has(K::DuplicateElement, "custDash"), "{f:#?}");
    assert!(f.has(K::DuplicateElement, "outerShdw"), "{f:#?}");
    assert!(f.has(K::DuplicateElement, "solidFill"), "{f:#?}");
    assert!(
        !f.has(K::Drawing(D::NonPositiveDashStop), "ds"),
        "custDash was second"
    );
    let p = d.first();
    assert_eq!(p.line.as_ref().unwrap().width, None);
    let Some(Fill::Gradient(g)) = p.fill.as_ref() else {
        panic!()
    };
    assert_eq!(
        g.stops.as_ref().unwrap().len(),
        3,
        "each kept, with what could be read"
    );
    assert!(
        g.stops
            .as_ref()
            .unwrap()
            .iter()
            .all(|s| s.position.is_none() && s.color.is_none())
    );

    let d = load("hostile/drawing-bad-custom-geometry.bin");
    assert!(
        d.findings.has(K::Drawing(D::GeometryRepaired), "custGeom"),
        "{:#?}",
        d.findings
    );
    assert!(
        d.findings.has(K::CountMismatch, "quadBezTo"),
        "{:#?}",
        d.findings
    );
    assert!(
        d.findings.has(K::CountMismatch, "cubicBezTo"),
        "{:#?}",
        d.findings
    );
    let Some(Geometry::Custom(custom)) = d.first().geometry.as_ref() else {
        panic!()
    };
    assert_eq!(
        custom.paths.as_ref().unwrap()[0].commands.len(),
        4,
        "the unknown one is left out"
    );

    let d = load("hostile/drawing-dangling-and-cycle.bin");
    assert!(
        d.findings.has(K::DanglingRelationship, "blip"),
        "{:#?}",
        d.findings
    );
    assert!(
        !d.findings.has(K::Drawing(D::EffectReference), "effectDag"),
        "a reference to a container that is there is not dangling, even in a cycle"
    );
}

#[test]
fn a_dag_with_a_repeated_name_or_a_dangling_reference_is_reported() {
    let d = load("hostile/drawing-bad-values.bin");
    // The second effects element is the DAG, left unread as a repeat of `effectLst`.
    assert!(d.findings.has(K::DuplicateElement, "effectDag"));

    let dml = "http://schemas.openxmlformats.org/drawingml/2006/main";
    for (dag, reported) in [
        (
            r#"<a:effectDag name="a"><a:cont name="a"/></a:effectDag>"#,
            true,
        ),
        (
            r#"<a:effectDag><a:effect ref="nowhere"/></a:effectDag>"#,
            true,
        ),
        (
            r#"<a:effectDag><a:effect ref="fillLine"/></a:effectDag>"#,
            false,
        ),
        (
            r#"<a:effectDag><a:cont name="b"/><a:effect ref="b"/></a:effectDag>"#,
            false,
        ),
    ] {
        let xml = dag.replacen(
            "<a:effectDag",
            &format!(r#"<a:effectDag xmlns:a="{dml}""#),
            1,
        );
        let tree = XmlTree::parse(xml.as_bytes(), "x", &OoxmlLimits::default()).unwrap();
        let import = ImportLimits::default();
        let mut findings = Findings::new(100);
        let mut r = Reader::new(&tree, "x", &import, &mut findings);
        let root = Elem::root(&tree, "x").unwrap();
        casual_slide_import::drawing::read_effects(&mut r, &[], DrawingLimits::default(), &root)
            .unwrap()
            .unwrap();
        assert_eq!(
            findings.has(K::Drawing(D::EffectReference), "effectDag"),
            reported,
            "{dag}"
        );
    }
}

#[test]
fn ms_oi29500_departures_are_reported_as_they_are_documented() {
    let dml = "http://schemas.openxmlformats.org/drawingml/2006/main";
    // 2.1.1292: Office wants dash stops above zero. The schema does not, so it is kept.
    let xml =
        format!(r#"<a:ln xmlns:a="{dml}"><a:custDash><a:ds d="0" sp="100"/></a:custDash></a:ln>"#);
    let tree = XmlTree::parse(xml.as_bytes(), "x", &OoxmlLimits::default()).unwrap();
    let import = ImportLimits::default();
    let mut findings = Findings::new(100);
    let mut r = Reader::new(&tree, "x", &import, &mut findings);
    let root = Elem::root(&tree, "x").unwrap();
    let line =
        casual_slide_import::drawing::read_line(&mut r, &[], DrawingLimits::default(), &root)
            .unwrap()
            .unwrap();
    assert!(findings.has(K::Drawing(D::NonPositiveDashStop), "ds"));
    assert_eq!(
        line.dash,
        Some(LineDash::Custom(vec![casual_slide_model::line::DashStop {
            dash: Some(Percentage(0)),
            space: Some(Percentage(100)),
        }]))
    );
    assert_eq!(D::NonPositiveDashStop.code(), "CS-DRW-0002");
    assert_eq!(D::DagWithSiblings.code(), "CS-DRW-0003");
    assert_eq!(D::GeometryRepaired.code(), "CS-DRW-0004");
    assert_eq!(D::EffectReference.code(), "CS-DRW-0005");
    assert_eq!(D::PercentRounded.code(), "CS-DRW-0006");
}

#[test]
fn every_text_warp_is_read() {
    let text = std::fs::read_to_string(repo().join("docs/spec/checklist.tsv")).unwrap();
    let dml = "http://schemas.openxmlformats.org/drawingml/2006/main";
    let mut seen = 0;
    for line in text.lines() {
        let Some(value) = line
            .split('\t')
            .next()
            .and_then(|id| id.strip_prefix("a:ST_TextShapeType/="))
        else {
            continue;
        };
        let xml =
            format!(r#"<a:prstTxWarp xmlns:a="{dml}" prst="{value}"><a:avLst/></a:prstTxWarp>"#);
        let tree = XmlTree::parse(xml.as_bytes(), "x", &OoxmlLimits::default()).unwrap();
        let import = ImportLimits::default();
        let mut findings = Findings::new(100);
        let mut r = Reader::new(&tree, "x", &import, &mut findings);
        let root = Elem::root(&tree, "x").unwrap();
        let warp = read_preset_text_shape(&mut r, DrawingLimits::default(), &root)
            .unwrap()
            .unwrap();
        use casual_slide_model::properties::Lexical;
        assert_eq!(warp.preset, TextShapeType::parse_lexical(value), "{value}");
        assert!(
            warp.preset.is_some() && findings.is_empty(),
            "{value}: {findings:?}"
        );
        seen += 1;
    }
    assert_eq!(seen, 41);
}

/// What real decks hold, as a pinned census: PowerPoint's own output is what this tier is
/// for, and a reader that quietly stopped modelling something would move these numbers.
#[test]
fn real_decks_are_modelled_and_what_is_not_is_named() {
    let mut census: BTreeMap<&str, usize> = BTreeMap::new();
    let mut codes: BTreeMap<String, usize> = BTreeMap::new();
    let mut retained: BTreeSet<String> = BTreeSet::new();
    let mut decks = 0;
    for entry in std::fs::read_dir(repo().join("fixtures/real")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("pptx") {
            continue;
        }
        decks += 1;
        let d = load_bytes(std::fs::read(&path).unwrap());
        for f in d.findings.iter() {
            let code = f.kind.code().to_owned();
            *codes.entry(code).or_default() += 1;
            if f.kind == K::RetainedElement
                && !matches!(f.element.as_str(), "txBody" | "style" | "tbl" | "extLst")
            {
                retained.insert(f.element.clone());
            }
        }
        for node in d
            .parts
            .iter()
            .filter_map(|p| p.shape_tree())
            .flat_map(ShapeNode::descendants)
        {
            let p = &node.properties;
            let mut add = |name, yes: bool| {
                if yes {
                    *census.entry(name).or_default() += 1;
                }
            };
            add("preset", matches!(p.geometry, Some(Geometry::Preset(_))));
            add("custom", matches!(p.geometry, Some(Geometry::Custom(_))));
            add("solid", matches!(p.fill, Some(Fill::Solid(_))));
            add("gradient", matches!(p.fill, Some(Fill::Gradient(_))));
            add("noFill", matches!(p.fill, Some(Fill::None)));
            add("blipFill", matches!(p.fill, Some(Fill::Blip(_))));
            add("line", p.line.is_some());
            add("effects", p.effects.is_some());
            add("scene3d", p.scene3d.is_some());
            add("sp3d", p.sp3d.is_some());
            add("hidden", p.hidden.is_some());
            add("picture", p.picture.is_some());
            if let Some(Geometry::Custom(g)) = &p.geometry {
                let compiled = compile_custom(g).expect("a real deck's geometry compiles");
                assert!(
                    compiled.diagnostics.is_empty(),
                    "{:?}",
                    compiled.diagnostics
                );
            }
        }
    }
    eprintln!("decks {decks}\ncensus {census:#?}\ncodes {codes:#?}\nretained {retained:#?}");
    assert_eq!(decks, 12);
    for name in [
        "preset", "custom", "solid", "line", "effects", "hidden", "picture",
    ] {
        assert!(
            census.get(name).copied().unwrap_or(0) > 0,
            "{name}: {census:?}"
        );
    }
    assert!(census["preset"] > 200, "{census:?}");
    // No real deck's lists come near a limit, and none repeats a choice.
    assert_eq!(codes.get("CS-DRW-0001"), None, "{codes:?}");
    assert_eq!(codes.get("CS-DRW-0004"), None, "{codes:?}");
}

/// `MS-OI29500` 2.1.1379's order needs these on the model: a body's warp with its adjusts,
/// its scene, and the `EG_Text3D` choice either way (T1E-06, T1F-04).
#[test]
fn a_bodys_warp_scene_and_3d_are_read_with_nothing_retained() {
    let d = load("generated/text-warp-and-3d.pptx");
    let on_slide: Vec<_> = d
        .findings
        .iter()
        .filter(|f| f.part.contains("slides/slide"))
        .collect();
    assert!(on_slide.is_empty(), "{on_slide:#?}");
    let props = |name| {
        d.named(name)
            .text_body
            .as_ref()
            .unwrap()
            .properties
            .clone()
            .unwrap()
    };

    let warp = props("Warp only").warp.unwrap();
    assert_eq!(lex(warp.preset.as_ref()).as_deref(), Some("textArchUp"));
    let adjust = warp.adjust.unwrap();
    assert_eq!(adjust[0].name.as_deref(), Some("adj"));
    assert_eq!(adjust[0].formula.as_deref(), Some("val 10800000"));
    assert_eq!(props("Warp no list").warp.unwrap().adjust, None);
    assert_eq!(props("Warp empty list").warp.unwrap().adjust, Some(vec![]));

    let extruded = props("Scene and extrusion");
    assert!(extruded.scene3d.as_ref().unwrap().camera.is_some());
    let Some(Text3D::Shape(s)) = extruded.text_3d else {
        panic!("no sp3d")
    };
    assert_eq!(s.z, Some(Coordinate(12_700)));
    assert_eq!(
        s.bevel_top.as_ref().unwrap().width,
        Some(Coordinate(63_500))
    );
    assert!(s.contour_color.is_some());

    assert_eq!(
        props("Scene and flat").text_3d,
        Some(Text3D::Flat(FlatText {
            z: Some(Coordinate(50_800))
        }))
    );
    assert_eq!(
        props("Flat bare").text_3d,
        Some(Text3D::Flat(FlatText { z: None })),
        "absent stays absent"
    );
    let all = props("Everything");
    assert!(all.warp.is_some() && all.scene3d.is_some());
    assert!(matches!(all.text_3d, Some(Text3D::Shape(_))));
    assert_eq!(all.autofit, Some(Autofit::None));
}

#[test]
fn hostile_warps_and_3d_are_reported_and_a_duplicate_of_the_choice_is_not_read() {
    let d = load("hostile/text-bad-3d.bin");
    let props = d
        .named("s")
        .text_body
        .as_ref()
        .unwrap()
        .properties
        .clone()
        .unwrap();
    assert!(d.findings.has(K::DuplicateElement, "prstTxWarp"));
    assert!(d.findings.has(K::DuplicateElement, "scene3d"));
    assert!(
        d.findings.has(K::DuplicateElement, "flatTx"),
        "{:?}",
        d.findings
    );
    assert!(
        d.findings.has(K::MalformedValue, "prstTxWarp"),
        "{:?}",
        d.findings
    );
    assert!(d.findings.has(K::RetainedElement, "(unknown)"));
    assert!(d.findings.has(K::MissingRequired, "gd"), "{:?}", d.findings);
    assert!(
        matches!(props.text_3d, Some(Text3D::Shape(_))),
        "first of the choice"
    );

    // Too many transforms on a colour in a body's `sp3d`, under the colour's own code.
    assert!(
        d.findings.iter().any(|f| f.kind.code() == "CS-CLR-0002"),
        "{:?}",
        d.findings
    );

    let d = load("hostile/text-limit-warp-guides.bin");
    let body = d.named("s").text_body.as_ref().unwrap();
    assert_eq!(body.properties.as_ref().unwrap().warp, None);
    assert!(
        d.findings.has(K::Drawing(D::OverLimit), "prstTxWarp"),
        "{:?}",
        d.findings
    );

    let d = load("hostile/text-limit-3d-depth.bin");
    assert!(
        d.findings.has(K::RetainedElement, "(unknown)"),
        "{:?}",
        d.findings
    );

    let d = load("hostile/drawing-limit-picture-effects.bin");
    assert!(d.first().fill.is_none());
    assert!(
        d.findings.has(K::Drawing(D::OverLimit), "blipFill"),
        "{:?}",
        d.findings
    );

    let d = load("hostile/shape-style-bad-values.bin");
    let style = d.named("s").style.as_ref().unwrap();
    assert!(style.line.as_ref().unwrap().idx.is_none());
    assert!(
        d.findings.has(K::DuplicateElement, "style"),
        "{:?}",
        d.findings
    );
}

/// `MS-OI29500` has them under `CS-CLR` on a theme; a slide part's report is the same list.
#[test]
fn a_colour_finding_in_a_shapes_properties_keeps_its_own_code() {
    let d = load("hostile/drawing-bad-values.bin");
    let codes: Vec<_> = d.findings.iter().map(|f| f.kind.code()).collect();
    assert!(codes.iter().any(|c| c.starts_with("CS-CLR-")), "{codes:?}");
    assert!(
        d.findings
            .iter()
            .any(|f| f.kind == K::Color(casual_slide_import::color::FindingKind::BadColorValue)),
        "{codes:?}"
    );
}

/// A deck's limits reach the drawing readers (T1E-06): a tighter `SlideLimits` leaves out
/// what the default reads, on a shape, a picture, a run, a warp and a master's text style.
#[test]
fn slide_limits_reach_every_drawing_reader() {
    let open_with = |path: &str, drawing: DrawingLimits| {
        let bytes = std::fs::read(repo().join("fixtures").join(path)).unwrap();
        let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
        let deck = import_deck(&mut pkg, &ImportLimits::default())
            .unwrap()
            .deck;
        let limits = SlideLimits {
            drawing,
            ..SlideLimits::default()
        };
        import_slide_parts(
            &mut pkg,
            &deck.presentation,
            &limits,
            &ImportLimits::default(),
        )
        .unwrap()
    };
    let tight = DrawingLimits {
        max_gradient_stops: 1,
        max_guides: 0,
        max_items_per_property: 1,
        ..DrawingLimits::default()
    };
    let over = K::Drawing(D::OverLimit);

    // On a shape, and with every limit at its default the same deck reads clean.
    let every = open_with("generated/drawing-every-element.pptx", tight);
    assert!(every.findings.has(over, "gradFill"), "{:?}", every.findings);
    assert!(every.findings.has(over, "custGeom"));
    let same = open_with(
        "generated/drawing-every-element.pptx",
        DrawingLimits::default(),
    );
    assert!(!same.findings.has(over, "gradFill"));
    // A picture's own `p:blipFill`, which the node holds beside `spPr`.
    let picture = |parts: &casual_slide_import::slide::ImportedSlides| {
        parts
            .parts
            .parts
            .iter()
            .flatten()
            .filter_map(|p| p.shape_tree())
            .flat_map(ShapeNode::descendants)
            .find(|n| n.kind == casual_slide_model::shape::NodeKind::Picture)
            .unwrap()
            .properties
            .picture
            .clone()
    };
    assert_eq!(picture(&every), None, "left out over the bound");
    assert!(picture(&same).is_some());

    // A body's warp and 3D.
    let warp = open_with("generated/text-warp-and-3d.pptx", tight);
    assert!(warp.findings.has(over, "prstTxWarp"), "{:?}", warp.findings);
    assert!(warp.findings.has(over, "scene3d"), "{:?}", warp.findings);

    // A run's own fill, and one in the master's text styles.
    let text = open_with("generated/text-every-property.pptx", tight);
    assert!(text.findings.has(over, "ln"), "{:?}", text.findings);
    const LEVEL: &str = r#"<a:lvl1pPr><a:defRPr><a:gradFill><a:gsLst><a:gs pos="0"><a:srgbClr val="000000"/></a:gs><a:gs pos="1000"><a:srgbClr val="000000"/></a:gs></a:gsLst></a:gradFill></a:defRPr></a:lvl1pPr>"#;
    let master = |root: &str, kind, limits: DrawingLimits| {
        let styles = if root == "sldMaster" {
            format!("<p:txStyles><p:titleStyle>{LEVEL}</p:titleStyle></p:txStyles>")
        } else {
            format!("<p:notesStyle>{LEVEL}</p:notesStyle>")
        };
        let xml = format!(
            r#"<p:{root} xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cSld><p:spTree/></p:cSld>{styles}</p:{root}>"#
        );
        let tree = XmlTree::parse(xml.as_bytes(), "m", &OoxmlLimits::default()).unwrap();
        let import = ImportLimits::default();
        let mut findings = Findings::new(100);
        let mut color = casual_slide_import::color::Findings::default();
        let part = casual_slide_import::slide::import_slide_part(
            &tree,
            "m",
            &[],
            kind,
            0,
            &SlideLimits {
                drawing: limits,
                ..SlideLimits::default()
            },
            &import,
            &mut findings,
            &mut color,
        )
        .unwrap();
        let style = part
            .notes_style
            .clone()
            .or_else(|| part.text_styles.clone().and_then(|t| t.title))
            .unwrap();
        style.levels[0].clone().unwrap().default_run.unwrap().fill
    };
    for (root, kind) in [
        ("sldMaster", SlideKind::Master),
        ("notesMaster", SlideKind::NotesMaster),
    ] {
        assert!(
            master(root, kind, DrawingLimits::default()).is_some(),
            "{root}"
        );
        assert_eq!(master(root, kind, tight), None, "{root}");
    }
}

/// The real deck that uses the older namespace, and a synthetic with hidden properties in
/// it: both read as the released one does.
#[test]
fn the_older_a14_namespace_reads_as_the_released_one() {
    let d = load("generated/drawing-extension-lists.pptx");
    let hidden = d
        .named("Older namespace")
        .properties
        .hidden
        .clone()
        .unwrap();
    assert!(matches!(hidden.fill, Some(Fill::Solid(Some(_)))));
    assert_eq!(hidden.line.unwrap().width, Some(9525));
    assert!(
        !d.findings.has(K::RetainedElement, "hiddenFill"),
        "{:?}",
        d.findings
    );

    let real = load("real/sdk-animation.pptx");
    assert!(
        !real.findings.has(K::RetainedElement, "useLocalDpi"),
        "the real deck's own use of it"
    );
}

/// What this deck says is reported where it should be, and nowhere else.
#[test]
fn every_extension_slot_in_3d_and_a_sketch_is_retained_and_named() {
    let d = load("generated/drawing-extension-lists.pptx");
    let slide: Vec<_> = d
        .findings
        .iter()
        .filter(|f| f.part.contains("slides/slide"))
        .map(|f| f.element.clone())
        .collect();
    assert_eq!(
        slide,
        ["extLst"],
        "scene3d, backdrop, sp3d, the sketch, by name"
    );
    for mode in [
        "none",
        "norm",
        "lighten",
        "lightenLess",
        "darken",
        "darkenLess",
    ] {
        let node = d.named(&format!("Path {mode}"));
        let Some(Geometry::Custom(g)) = &node.properties.geometry else {
            panic!("{mode}")
        };
        let path = &g.paths.as_ref().unwrap()[0];
        assert_eq!(lex(path.fill.as_ref()).as_deref(), Some(mode));
    }
}

/// Each gradient kind and pattern, as a shape holding text, the kind no real deck here has.
#[test]
fn gradient_and_pattern_fills_on_placed_shapes_read_with_their_text() {
    let d = load("generated/shape-fills.pptx");
    assert_eq!(
        d.findings
            .iter()
            .filter(|f| f.part.contains("slides/slide"))
            .count(),
        0,
        "{:?}",
        d.findings
    );
    let Some(Fill::Gradient(g)) = &d.named("Radial").properties.fill else {
        panic!("radial")
    };
    assert_eq!(g.stops.as_ref().unwrap().len(), 2);
    assert!(d.named("Radial").text_body.is_some());
    assert!(d.named("Radial").transform.is_some());
    let text = std::fs::read_to_string(repo().join("docs/spec/checklist.tsv")).unwrap();
    let patterns: Vec<_> = text
        .lines()
        .filter_map(|l| {
            l.split('\t')
                .next()?
                .strip_prefix("a:ST_PresetPatternVal/=")
        })
        .collect();
    assert!(patterns.len() >= 54);
    for pattern in patterns {
        let node = d.named(pattern);
        assert!(node.text_body.is_some(), "{pattern}");
        let Some(Fill::Pattern(p)) = &node.properties.fill else {
            panic!("{pattern}")
        };
        assert_eq!(lex(p.preset.as_ref()).as_deref(), Some(pattern));
        assert!(
            p.foreground.is_some() && p.background.is_some(),
            "{pattern}"
        );
    }
}

/// `p:style` on the kinds that hold one; and in a theme's object defaults, a `bodyPr` and
/// `lstStyle` that are text, and bad values reported in the theme's own list.
#[test]
fn a_shape_style_and_a_themes_object_defaults_are_read() {
    let d = load("generated/text-object-defaults.pptx");
    assert!(
        !d.findings.has(K::RetainedElement, "style"),
        "{:?}",
        d.findings
    );
    let style = d.named("Styled").style.as_ref().unwrap();
    assert_eq!(style.line.as_ref().unwrap().idx, Some(1));
    assert_eq!(
        style.font.as_ref().unwrap().idx,
        Some(casual_slide_model::theme::FontCollectionIndex::Major)
    );
    assert!(d.named("Connector").style.is_some());
    assert_eq!(d.named("No style").style, None);
    assert!(d.named("Connector").start_connection.is_some());
    assert!(d.named("Connector").end_connection.is_some());

    let theme = |file: &str| {
        let bytes = std::fs::read(repo().join("fixtures").join(file)).unwrap();
        let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
        let xml = pkg.read_part("ppt/theme/theme1.xml").unwrap();
        let tree = XmlTree::parse(&xml, "ppt/theme/theme1.xml", &OoxmlLimits::default()).unwrap();
        casual_slide_import::theme::import_theme(&tree).unwrap()
    };
    let good = theme("generated/text-object-defaults.pptx");
    assert!(good.findings.items.is_empty(), "{:?}", good.findings);
    let defaults = good.value.object_defaults.unwrap();
    let tx = defaults.text.unwrap();
    assert_eq!(
        lex(tx.body_properties.unwrap().anchor.as_ref()).as_deref(),
        Some("ctr")
    );
    let run = tx.list_style.unwrap().levels[0]
        .clone()
        .unwrap()
        .default_run
        .unwrap();
    assert_eq!(run.size, Some(2100));
    assert_eq!(
        defaults.line.unwrap().body_properties.unwrap().columns,
        Some(3)
    );
    assert!(defaults.shape.unwrap().style.is_some());

    let bad = theme("hostile/theme-object-defaults-bad-values.bin");
    let codes = bad.findings.codes();
    assert!(codes.contains(&"CS-CLR-0005"), "{codes:?}");
    assert!(codes.contains(&"CS-CLR-0009"), "{codes:?}");
    assert!(
        codes.contains(&"CS-CLR-0010"),
        "a tab list over its bound: {codes:?}"
    );
    let defaults = bad.value.object_defaults.unwrap();
    let sp = defaults.shape.unwrap();
    assert_eq!(
        sp.body_properties.unwrap().columns,
        None,
        "99 is out of range"
    );
    assert!(sp.list_style.is_some());
    assert_eq!(defaults.line.unwrap().list_style, None);
}
