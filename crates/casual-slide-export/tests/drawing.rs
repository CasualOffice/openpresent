//! Tier 1E: geometry, fills, lines, effects and 3D round trip through the tree.
//!
//! The enumerations are read from `docs/spec/checklist.tsv` when the test runs, so a row
//! added to the checklist with no handling here fails, and no value can be left out by
//! recollection. Real decks are the point: every edit test runs over `fixtures/real`,
//! `fixtures/producers` and `fixtures/generated`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use casual_slide_export::slide::{patch_slide_part, save_slide_parts};
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{SlideLimits, import_slide_part, import_slide_parts};
use casual_slide_model::color::{Color, ColorValue, Percentage, Rgb, SchemeColor};
use casual_slide_model::drawing::ShapeProperties;
use casual_slide_model::effect::{Effect, EffectProperties};
use casual_slide_model::fill::{BlipFillMode, Fill, GradientFill, GradientShade, GradientStop};
use casual_slide_model::geometry::{Geometry, PathCommand};
use casual_slide_model::line::{CompoundLine, Line, LineCap, LineDash, LineJoin, PresetLineDash};
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::{NodeKind, ShapeNode};
use casual_slide_model::slide::{SlideKind, SlidePart, SlideParts};
use casual_slide_model::theme::FontCollectionIndex;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};

const PART: &str = "ppt/slides/slide1.xml";
const NS: &str = r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
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
            let b = pkg.read_part(&n).unwrap();
            (n, b)
        })
        .collect()
}

/// `(st, values)` for the simple types of the areas, read from the checklist.
fn checklist_values() -> BTreeMap<String, Vec<String>> {
    let text = std::fs::read_to_string(repo().join("docs/spec/checklist.tsv")).unwrap();
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in text.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        let (Some(id), Some(area), Some(kind)) = (cols.first(), cols.get(1), cols.get(3)) else {
            continue;
        };
        if !matches!(*area, "GEOM" | "FILL" | "LINE" | "EFFECT" | "D3") || *kind != "value" {
            continue;
        }
        let (st, value) = id.split_once("/=").unwrap();
        out.entry(st.to_owned()).or_default().push(value.to_owned());
    }
    out
}

fn spt(props: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>{props}</p:spPr></p:sp></p:spTree></p:cSld></p:sld>"#
    )
}

struct Read {
    tree: XmlTree,
    part: SlidePart,
    findings: Findings,
}

fn read(xml: &str) -> Read {
    let tree = XmlTree::parse(xml.as_bytes(), PART, &OoxmlLimits::default()).unwrap();
    let mut findings = Findings::new(10_000);
    let mut color = ColorFindings::default();
    let part = import_slide_part(
        &tree,
        PART,
        &[],
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &mut findings,
        &mut color,
    )
    .unwrap();
    Read {
        tree,
        part,
        findings,
    }
}

fn props_of(part: &SlidePart) -> &ShapeProperties {
    &part.shape_tree().unwrap().children[0].properties
}

/// Write `after` into `tree`, returning the changes.
fn patch(tree: &mut XmlTree, after: &SlidePart) -> Result<usize, String> {
    patch_slide_part(
        tree,
        PART,
        false,
        &[],
        after,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &OoxmlLimits::default(),
    )
    .map_err(|e| e.code().to_owned())
}

/// For an enumeration: the `spPr` content holding one value, and how to get it back out.
type Template = (
    &'static str,
    fn(&str) -> String,
    fn(&ShapeProperties) -> Option<String>,
);

fn lex<T: Lexical>(v: Option<&T>) -> Option<String> {
    v.map(Lexical::to_lexical)
}

fn first_effect(p: &ShapeProperties) -> Option<&Effect> {
    match p.effects.as_ref()? {
        EffectProperties::List(l) => l.first(),
        EffectProperties::Dag(c) => c.effects.first(),
    }
}

fn gradient(p: &ShapeProperties) -> Option<&GradientFill> {
    match p.fill.as_ref()? {
        Fill::Gradient(g) => Some(g),
        _ => None,
    }
}

fn blip_tile(p: &ShapeProperties) -> Option<casual_slide_model::fill::TileInfo> {
    match p.fill.as_ref()? {
        Fill::Blip(b) => match b.mode? {
            BlipFillMode::Tile(t) => Some(t),
            BlipFillMode::Stretch(_) => None,
        },
        _ => None,
    }
}

/// One entry per `ST_*` with values in the areas. A type not here fails the test.
fn templates() -> Vec<Template> {
    vec![
        (
            "a:ST_ShapeType",
            |v| format!(r#"<a:prstGeom prst="{v}"><a:avLst/></a:prstGeom>"#),
            |p| match p.geometry.as_ref()? {
                Geometry::Preset(g) => lex(g.preset.as_ref()),
                Geometry::Custom(_) => None,
            },
        ),
        (
            "a:ST_PathFillMode",
            |v| {
                format!(
                    r#"<a:custGeom><a:pathLst><a:path fill="{v}"><a:close/></a:path></a:pathLst></a:custGeom>"#
                )
            },
            |p| match p.geometry.as_ref()? {
                Geometry::Custom(g) => lex(g.paths.as_ref()?.first()?.fill.as_ref()),
                Geometry::Preset(_) => None,
            },
        ),
        (
            "a:ST_PathShadeType",
            |v| format!(r#"<a:gradFill><a:path path="{v}"/></a:gradFill>"#),
            |p| match gradient(p)?.shade? {
                GradientShade::Path { path, .. } => lex(path.as_ref()),
                GradientShade::Linear { .. } => None,
            },
        ),
        (
            "a:ST_TileFlipMode",
            |v| format!(r#"<a:gradFill flip="{v}"/>"#),
            |p| lex(gradient(p)?.flip.as_ref()),
        ),
        (
            "a:ST_PresetPatternVal",
            |v| format!(r#"<a:pattFill prst="{v}"/>"#),
            |p| match p.fill.as_ref()? {
                Fill::Pattern(f) => lex(f.preset.as_ref()),
                _ => None,
            },
        ),
        (
            "a:ST_RectAlignment",
            |v| format!(r#"<a:blipFill><a:tile algn="{v}"/></a:blipFill>"#),
            |p| lex(blip_tile(p)?.align.as_ref()),
        ),
        (
            "a:ST_LineEndType",
            |v| format!(r#"<a:ln><a:headEnd type="{v}"/></a:ln>"#),
            |p| lex(p.line.as_ref()?.head_end?.end_type.as_ref()),
        ),
        (
            "a:ST_LineEndWidth",
            |v| format!(r#"<a:ln><a:tailEnd w="{v}"/></a:ln>"#),
            |p| lex(p.line.as_ref()?.tail_end?.width.as_ref()),
        ),
        (
            "a:ST_LineEndLength",
            |v| format!(r#"<a:ln><a:tailEnd len="{v}"/></a:ln>"#),
            |p| lex(p.line.as_ref()?.tail_end?.length.as_ref()),
        ),
        (
            "a:ST_PresetLineDashVal",
            |v| format!(r#"<a:ln><a:prstDash val="{v}"/></a:ln>"#),
            |p| match p.line.as_ref()?.dash.as_ref()? {
                LineDash::Preset(d) => lex(d.as_ref()),
                LineDash::Custom(_) => None,
            },
        ),
        (
            "a:ST_LineCap",
            |v| format!(r#"<a:ln cap="{v}"/>"#),
            |p| lex(p.line.as_ref()?.cap.as_ref()),
        ),
        (
            "a:ST_PenAlignment",
            |v| format!(r#"<a:ln algn="{v}"/>"#),
            |p| lex(p.line.as_ref()?.align.as_ref()),
        ),
        (
            "a:ST_CompoundLine",
            |v| format!(r#"<a:ln cmpd="{v}"/>"#),
            |p| lex(p.line.as_ref()?.compound.as_ref()),
        ),
        (
            "a:ST_PresetCameraType",
            |v| format!(r#"<a:scene3d><a:camera prst="{v}"/></a:scene3d>"#),
            |p| lex(p.scene3d.as_ref()?.camera?.preset.as_ref()),
        ),
        (
            "a:ST_LightRigDirection",
            |v| format!(r#"<a:scene3d><a:lightRig dir="{v}"/></a:scene3d>"#),
            |p| lex(p.scene3d.as_ref()?.light_rig?.direction.as_ref()),
        ),
        (
            "a:ST_LightRigType",
            |v| format!(r#"<a:scene3d><a:lightRig rig="{v}"/></a:scene3d>"#),
            |p| lex(p.scene3d.as_ref()?.light_rig?.rig.as_ref()),
        ),
        (
            "a:ST_BevelPresetType",
            |v| format!(r#"<a:sp3d><a:bevelT prst="{v}"/></a:sp3d>"#),
            |p| lex(p.sp3d.as_ref()?.bevel_top?.preset.as_ref()),
        ),
        (
            "a:ST_PresetMaterialType",
            |v| format!(r#"<a:sp3d prstMaterial="{v}"/>"#),
            |p| lex(p.sp3d.as_ref()?.material.as_ref()),
        ),
        (
            "a:ST_PresetShadowVal",
            |v| {
                format!(
                    r#"<a:effectLst><a:prstShdw prst="{v}"><a:srgbClr val="000000"/></a:prstShdw></a:effectLst>"#
                )
            },
            |p| match first_effect(p)? {
                Effect::PresetShadow { preset, .. } => lex(preset.as_ref()),
                _ => None,
            },
        ),
        (
            "a:ST_BlendMode",
            |v| format!(r#"<a:effectDag><a:blend blend="{v}"/></a:effectDag>"#),
            |p| match first_effect(p)? {
                Effect::Blend { blend, .. } => lex(blend.as_ref()),
                _ => None,
            },
        ),
        (
            "a:ST_EffectContainerType",
            |v| format!(r#"<a:effectDag><a:cont type="{v}"/></a:effectDag>"#),
            |p| match first_effect(p)? {
                Effect::Container(c) => lex(c.container_type.as_ref()),
                _ => None,
            },
        ),
    ]
}

/// `a:ST_TextShapeType` is reached through a text body, and read by its own function.
const READ_ELSEWHERE: [&str; 1] = ["a:ST_TextShapeType"];

#[test]
fn every_enum_value_in_the_areas_is_read_written_and_a_bad_one_reported() {
    let values = checklist_values();
    assert!(
        values.len() >= 22,
        "the checklist has lost rows: {:?}",
        values.keys()
    );
    let table = templates();
    let handled: BTreeSet<&str> = table.iter().map(|t| t.0).collect();
    for st in values.keys() {
        assert!(
            handled.contains(st.as_str()) || READ_ELSEWHERE.contains(&st.as_str()),
            "{st} has values in the checklist and no handling here"
        );
    }
    let mut total = 0;
    for (st, make, get) in table {
        let list = values
            .get(st)
            .unwrap_or_else(|| panic!("{st} is not in the checklist"));
        let base = read(&spt(&make(&list[0])));
        for value in list {
            let xml = spt(&make(value));
            let one = read(&xml);
            assert_eq!(
                get(props_of(&one.part)).as_deref(),
                Some(value.as_str()),
                "{st} = {value} is not read: {:?}",
                one.findings
            );
            assert!(
                !one.findings.iter().any(|f| matches!(
                    f.kind,
                    casual_slide_import::properties::ImportFindingKind::MalformedValue
                        | casual_slide_import::properties::ImportFindingKind::OutOfRange
                )),
                "{st} = {value}: {:?}",
                one.findings
            );

            // Written: from the first value's tree to this value's model, byte for byte.
            let mut tree = base.tree.clone();
            patch(&mut tree, &one.part).unwrap_or_else(|e| panic!("{st} = {value}: {e}"));
            assert_eq!(
                String::from_utf8(tree.to_bytes()).unwrap(),
                xml,
                "{st} = {value} is not written"
            );
            total += 1;
        }

        // A token the schema does not have is reported, left out, and kept in the tree.
        let xml = spt(&make("notAValue"));
        let bad = read(&xml);
        assert_eq!(get(props_of(&bad.part)), None, "{st}");
        assert!(
            bad.findings.iter().any(|f| f.kind
                == casual_slide_import::properties::ImportFindingKind::MalformedValue
                && f.attribute.is_some()),
            "{st}: a bad token is not reported: {:?}",
            bad.findings
        );
        let mut tree = bad.tree.clone();
        assert_eq!(patch(&mut tree, &bad.part), Ok(0));
        assert_eq!(String::from_utf8(tree.to_bytes()).unwrap(), xml);
    }
    assert!(total >= 300, "only {total} values were exercised");
}

#[test]
fn every_preset_token_is_known_to_the_geometry_engine() {
    let values = checklist_values();
    let shapes = &values["a:ST_ShapeType"];
    assert_eq!(shapes.len(), 187);
    for token in shapes {
        assert!(
            casual_slide_geometry::preset_shape(token).is_some(),
            "{token}"
        );
    }
    let warps = &values["a:ST_TextShapeType"];
    assert_eq!(warps.len(), 41);
    for token in warps {
        assert!(casual_slide_geometry::text_warp(token).is_some(), "{token}");
        let parsed = casual_slide_model::geometry::TextShapeType::parse_lexical(token).unwrap();
        assert_eq!(&parsed.to_lexical(), token);
    }
}

#[test]
fn all_187_presets_round_trip_through_the_deck_that_shows_them() {
    let path = repo().join("fixtures/generated/preset-shapes.pptx");
    let source = std::fs::read(path).unwrap();
    let before = model(&source);
    let tokens = |m: &SlideParts| -> BTreeSet<String> {
        m.parts
            .iter()
            .flatten()
            .filter_map(|p| p.shape_tree())
            .flat_map(ShapeNode::descendants)
            .filter_map(|n| match n.properties.geometry.as_ref()? {
                Geometry::Preset(g) => lex(g.preset.as_ref()),
                Geometry::Custom(_) => None,
            })
            .collect()
    };
    let wanted: BTreeSet<String> = checklist_values()["a:ST_ShapeType"]
        .iter()
        .cloned()
        .collect();
    assert_eq!(tokens(&before), wanted, "the deck shows every preset");

    // Turn every shape into a `rect`, save, reopen, and turn them back: same bytes.
    let rect = casual_slide_model::geometry::ShapeType::Rect;
    let mut flat = before.clone();
    let mut changed = 0;
    let ordinals: Vec<u32> = flat.parts.iter().flatten().map(|p| p.id.part).collect();
    for ordinal in &ordinals {
        let part = flat
            .parts
            .iter_mut()
            .flatten()
            .find(|p| p.id.part == *ordinal)
            .unwrap();
        let ids: Vec<_> = part
            .shape_tree()
            .into_iter()
            .flat_map(ShapeNode::descendants)
            .map(|n| n.id)
            .collect();
        for id in ids {
            if let Some(Geometry::Preset(g)) =
                part.node_mut(id).unwrap().properties.geometry.as_mut()
                && g.preset != Some(rect)
            {
                g.preset = Some(rect);
                changed += 1;
            }
        }
    }
    assert!(changed >= 186, "{changed}");
    let save = |bytes: &[u8], wanted: &SlideParts| {
        let mut pkg = open(bytes.to_vec());
        let refs: Vec<&SlidePart> = wanted.parts.iter().flatten().collect();
        save_slide_parts(
            &mut pkg,
            &wanted.structure,
            &refs,
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap()
        .bytes
    };
    let flattened = save(&source, &flat);
    assert_eq!(model(&flattened), flat, "reads back as rects");
    assert_eq!(tokens(&model(&flattened)).len(), 1);
    let restored = save(&flattened, &before);
    assert_eq!(parts(&restored), parts(&source), "every part byte for byte");
}

/// The nodes of a deck's slide-like parts that can take a fill and a line edit.
fn editable(part: &SlidePart) -> Vec<&ShapeNode> {
    part.shape_tree()
        .into_iter()
        .flat_map(ShapeNode::descendants)
        .filter(|n| matches!(n.kind, NodeKind::Shape | NodeKind::Connector))
        .collect()
}

fn new_fill() -> Fill {
    Fill::Gradient(Box::new(GradientFill {
        rotate_with_shape: Some(true),
        stops: Some(vec![
            GradientStop {
                position: Some(Percentage(0)),
                color: Some(Color::srgb(1, 2, 3)),
            },
            GradientStop {
                position: Some(Percentage(100_000)),
                color: Some(Color::new(ColorValue::Scheme(SchemeColor::Accent1))),
            },
        ]),
        shade: Some(GradientShade::Linear {
            angle: Some(casual_slide_model::color::Angle(5_400_000)),
            scaled: Some(false),
        }),
        ..GradientFill::default()
    }))
}

fn edit_line(line: &mut Option<Box<Line>>) {
    let l = line.get_or_insert_with(Box::default);
    l.width = Some(31_750);
    l.cap = Some(LineCap::Sq);
    l.compound = Some(CompoundLine::ThickThin);
    l.fill = Some(Fill::Solid(Some(Color::new(ColorValue::Srgb(Rgb::new(
        9, 8, 7,
    ))))));
    l.dash = Some(LineDash::Preset(Some(PresetLineDash::LgDash)));
    l.join = Some(LineJoin::Miter(Some(Percentage(800_000))));
}

/// A part's text before, inside and after the `spPr` that follows this name, as a
/// statement that nothing outside it moved.
fn sp_pr_of_named(xml: &str, name_attr: &str) -> (String, String, String) {
    let at = xml
        .find(name_attr)
        .unwrap_or_else(|| panic!("{name_attr} not in the part"));
    let start = at + xml[at..].find("<p:spPr").unwrap();
    let tag_end = start + xml[start..].find('>').unwrap();
    let end = if xml[..tag_end].ends_with('/') {
        tag_end + 1
    } else {
        start + xml[start..].find("</p:spPr>").unwrap() + "</p:spPr>".len()
    };
    (
        xml[..start].to_owned(),
        xml[start..end].to_owned(),
        xml[end..].to_owned(),
    )
}

/// Edit a fill and a line on one node of one part. The result reads back as asked, only
/// that part's bytes changed, only inside that node's `spPr`, a second save is a fixed
/// point, and F4 passes. `None` if the deck has nothing to edit.
fn edit_one(source: &[u8], label: &str) -> Option<Vec<u8>> {
    let wanted = model(source);
    // A part whose patch must not be refused: a node not in alternate content.
    let mut chosen = None;
    for part in wanted.parts.iter().flatten().rev() {
        for node in editable(part) {
            let name = node.non_visual.as_ref()?.name.clone()?;
            if name.is_empty() || name.contains(['<', '>', '&', '"']) {
                continue;
            }
            chosen = Some((part.id.part, node.id, name));
            break;
        }
        if chosen.is_some() {
            break;
        }
    }
    let (ordinal, id, name) = chosen?;
    let mut wanted = wanted;
    let part = wanted
        .parts
        .iter_mut()
        .flatten()
        .find(|p| p.id.part == ordinal)?;
    {
        let node = part.node_mut(id)?;
        node.properties.fill = Some(new_fill());
        edit_line(&mut node.properties.line);
    }
    let edited: Vec<&SlidePart> = wanted
        .parts
        .iter()
        .flatten()
        .filter(|p| p.id.part == ordinal)
        .collect();
    let mut pkg = open(source.to_vec());
    let outcome = match save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &edited,
        &SlideLimits::default(),
        &ImportLimits::default(),
    ) {
        Ok(outcome) => outcome,
        // A node in alternate content is refused by Tier 1D's rule, and that is a pass.
        Err(e) if e.code() == "CS-SLE-0002" => return None,
        Err(e) => panic!("{label}: {e}"),
    };
    assert_eq!(
        model(&outcome.bytes),
        wanted,
        "{label}: reads back as asked"
    );
    assert_eq!(outcome.changed_parts.len(), 1, "{label}");

    // Every other part byte for byte.
    let (old, new) = (parts(source), parts(&outcome.bytes));
    assert_eq!(
        old.keys().collect::<Vec<_>>(),
        new.keys().collect::<Vec<_>>(),
        "{label}"
    );
    for (name, bytes) in &old {
        if !outcome.changed_parts.contains(name) {
            assert_eq!(new.get(name), Some(bytes), "{label}: {name} changed");
        }
    }

    // In the part, nothing outside this node's `spPr`.
    let changed = &outcome.changed_parts[0];
    let needle = format!("name=\"{name}\"");
    let (a, b) = (
        String::from_utf8_lossy(&old[changed]).into_owned(),
        String::from_utf8_lossy(&new[changed]).into_owned(),
    );
    if a.matches(&needle).count() == 1 {
        let (before_a, inside_a, after_a) = sp_pr_of_named(&a, &needle);
        let (before_b, inside_b, after_b) = sp_pr_of_named(&b, &needle);
        assert_eq!(before_a, before_b, "{label}: before the node's spPr");
        assert_eq!(after_a, after_b, "{label}: after the node's spPr");
        assert_ne!(inside_a, inside_b);
    }

    // Fixed point.
    let mut again = open(outcome.bytes.clone());
    let second = save_slide_parts(
        &mut again,
        &wanted.structure,
        &edited,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(second.bytes, outcome.bytes, "{label}: fixed point");
    assert!(second.changed_parts.is_empty());
    Some(outcome.bytes)
}

fn decks(folder: &str) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(repo().join("fixtures").join(folder)).unwrap() {
        let path = entry.unwrap().path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if matches!(ext, "pptx" | "potx" | "ppsx" | "pptm") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            out.push((name, std::fs::read(&path).unwrap()));
        }
    }
    out.sort();
    out
}

/// Each part's status and error count, from `tools/f4/validate.py --json`.
fn f4_parts(bytes: &[u8], label: &str) -> Option<BTreeMap<String, (String, u64)>> {
    let required = std::env::var("F4_REQUIRED").is_ok_and(|v| v == "1");
    let have = |tool: &str| Command::new(tool).arg("--version").output().is_ok();
    if !have("xmllint") || !have("python3") {
        assert!(!required, "F4_REQUIRED, and xmllint or python3 is missing");
        eprintln!("xmllint or python3 not installed: F4 skipped");
        return None;
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("f4-drawing");
    std::fs::create_dir_all(&dir).unwrap();
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let file = dir.join(format!("deck-{}-{n}.pptx", std::process::id()));
    std::fs::write(&file, bytes).unwrap();
    let out = Command::new("python3")
        .arg("-I")
        .arg(repo().join("tools/f4/validate.py"))
        .arg("--json")
        .arg(&file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();
    assert_ne!(out.status.code(), Some(2), "{label}: F4 could not run");
    // Its form is stable and flat enough to read by line; a record's keys are sorted.
    let text = String::from_utf8(out.stdout).unwrap();
    let value = |line: &str| {
        line.split_once(": ")
            .map(|(_, v)| v.trim_end_matches(',').trim_matches('"').to_owned())
    };
    let mut parts = BTreeMap::new();
    let (mut errors, mut part) = (0, String::new());
    for line in text.lines().map(str::trim) {
        if line.starts_with("\"error_count\"") {
            errors = value(line).unwrap().parse().unwrap();
        } else if line.starts_with("\"part\"") {
            part = value(line).unwrap();
        } else if line.starts_with("\"status\"") {
            // The record's own, or with no part pending, the deck's.
            let key = if part.is_empty() {
                "<deck>".to_owned()
            } else {
                std::mem::take(&mut part)
            };
            parts.insert(key, (value(line).unwrap(), errors));
        }
    }
    assert!(!parts.is_empty(), "{label}: F4 reported nothing: {text}");
    Some(parts)
}

/// F4 on the edited deck, against F4 on the deck it came from. Several real decks do not
/// validate as written (`fixtures/real`: a `tint` out of range, a `buSzPct` under 25%),
/// in parts this edit does not touch. So: nothing an edit touched may get worse, and a
/// deck that passed still passes.
fn f4_validates(source: &[u8], edited: &[u8], label: &str) {
    let (Some(before), Some(after)) = (f4_parts(source, label), f4_parts(edited, label)) else {
        return;
    };
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>(),
        "{label}"
    );
    for (part, (status, errors)) in &after {
        let (old_status, old_errors) = &before[part];
        assert!(
            status == old_status && errors <= old_errors,
            "{label}: {part} went from {old_status} ({old_errors}) to {status} ({errors})"
        );
    }
}

#[test]
fn a_fill_and_a_line_edit_over_real_decks_changes_only_those_nodes() {
    let mut edited = 0;
    for (name, source) in decks("real") {
        if let Some(bytes) = edit_one(&source, &name) {
            f4_validates(&source, &bytes, &name);
            edited += 1;
        }
    }
    assert!(edited >= 10, "only {edited} real decks took the edit");
}

#[test]
fn a_fill_and_a_line_edit_over_producer_and_generated_decks_changes_only_those_nodes() {
    let mut edited = 0;
    for folder in ["producers", "generated"] {
        for (name, source) in decks(folder) {
            if name == "five-hundred-slides.pptx" {
                continue;
            }
            if let Some(bytes) = edit_one(&source, &name) {
                f4_validates(&source, &bytes, &name);
                edited += 1;
            }
        }
    }
    assert!(edited >= 20, "only {edited} decks took the edit");
}

#[test]
fn unedited_real_decks_patch_to_nothing_and_read_identically_twice() {
    let mut nodes_with_drawing = 0usize;
    for folder in ["real", "producers", "generated"] {
        for (name, source) in decks(folder) {
            if name == "five-hundred-slides.pptx" {
                continue;
            }
            let wanted = model(&source);
            let mut pkg = open(source.clone());
            let refs: Vec<&SlidePart> = wanted.parts.iter().flatten().collect();
            let out = save_slide_parts(
                &mut pkg,
                &wanted.structure,
                &refs,
                &SlideLimits::default(),
                &ImportLimits::default(),
            )
            .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(out.changed_parts.is_empty(), "{name}");
            assert_eq!(out.bytes, source, "{name}: unedited save");
            nodes_with_drawing += wanted
                .parts
                .iter()
                .flatten()
                .filter_map(|p| p.shape_tree())
                .flat_map(ShapeNode::descendants)
                .filter(|n| n.properties != ShapeProperties::default())
                .count();
        }
    }
    assert!(nodes_with_drawing > 1000, "{nodes_with_drawing}");
}

fn every_element() -> Vec<u8> {
    std::fs::read(repo().join("fixtures/generated/drawing-every-element.pptx")).unwrap()
}

fn named<'a>(part: &'a mut SlidePart, name: &str) -> &'a mut ShapeNode {
    let id = part
        .shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some(name))
        .unwrap_or_else(|| panic!("no {name}"))
        .id;
    part.node_mut(id).unwrap()
}

fn ordinal_of_slide(m: &SlideParts) -> u32 {
    m.parts
        .iter()
        .flatten()
        .find(|p| p.kind == SlideKind::Slide)
        .unwrap()
        .id
        .part
}

/// Edit the fixture's slide and return the slide's new text, having checked the rest.
fn edit_every_element(change: impl FnOnce(&mut SlidePart)) -> String {
    let source = every_element();
    let mut wanted = model(&source);
    let ordinal = ordinal_of_slide(&wanted);
    let part = wanted
        .parts
        .iter_mut()
        .flatten()
        .find(|p| p.id.part == ordinal)
        .unwrap();
    change(part);
    let edited: Vec<&SlidePart> = wanted
        .parts
        .iter()
        .flatten()
        .filter(|p| p.id.part == ordinal)
        .collect();
    let mut pkg = open(source.clone());
    let out = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &edited,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(model(&out.bytes), wanted);
    f4_validates(&source, &out.bytes, "every element");
    assert!(
        f4_parts(&out.bytes, "every element")
            .is_none_or(|p| p.values().all(|(s, _)| s == "valid" || s == "pass"))
    );
    String::from_utf8(parts(&out.bytes)[PART].clone()).unwrap()
}

fn source_slide() -> String {
    String::from_utf8(parts(&every_element())[PART].clone()).unwrap()
}

/// `from` occurs once; the edit turned it into `to`, and nothing else in the part moved.
fn assert_only(new: &str, from: &str, to: &str) {
    let old = source_slide();
    assert_eq!(old.matches(from).count(), 1, "{from}");
    assert_eq!(new, old.replace(from, to));
}

#[test]
fn a_changed_attribute_changes_that_attribute_and_no_other_byte() {
    let new = edit_every_element(|part| {
        let line = named(part, "solid and full line")
            .properties
            .line
            .as_mut()
            .unwrap();
        line.width = Some(25_400);
    });
    assert_only(
        &new,
        r#"<a:ln w="12700" cap="rnd""#,
        r#"<a:ln w="25400" cap="rnd""#,
    );

    let new = edit_every_element(|part| {
        let Some(Fill::Pattern(p)) = named(part, "pattern").properties.fill.as_mut() else {
            panic!()
        };
        p.preset = Some(casual_slide_model::fill::PresetPattern::Wave);
    });
    assert_only(&new, r#"prst="dkDnDiag""#, r#"prst="wave""#);

    let new = edit_every_element(|part| {
        let s = named(part, "effect list and 3D")
            .properties
            .sp3d
            .as_mut()
            .unwrap();
        s.material = Some(casual_slide_model::d3::PresetMaterial::Clear);
    });
    assert_eq!(
        new.matches(r#"prstMaterial="clear""#).count(),
        1,
        "the shape's and not the hidden one's"
    );
}

#[test]
fn a_gradient_stop_edit_keeps_the_other_stop_and_the_fill_kind() {
    let new = edit_every_element(|part| {
        let Some(Fill::Gradient(g)) = named(part, "linear gradient").properties.fill.as_mut()
        else {
            panic!()
        };
        g.stops.as_mut().unwrap()[1].position = Some(Percentage(90_000));
    });
    assert_only(
        &new,
        r#"<a:gs pos="100000"><a:schemeClr val="accent1"/></a:gs>"#,
        r#"<a:gs pos="90000"><a:schemeClr val="accent1"/></a:gs>"#,
    );
}

#[test]
fn replacing_a_fill_replaces_that_element_in_schema_order() {
    let new = edit_every_element(|part| {
        named(part, "pattern").properties.fill = Some(Fill::None);
    });
    let at = new.find(r#"name="pattern""#).unwrap();
    let slice = &new[at..];
    assert!(
        slice.contains(r#"</a:prstGeom><a:noFill/><a:ln>"#),
        "noFill sits between geometry and line: {}",
        &slice[..300]
    );
}

#[test]
fn removing_and_adding_a_line_leaves_the_other_properties() {
    let new = edit_every_element(|part| {
        named(part, "pattern").properties.line = None;
    });
    assert!(
        !new[new.find(r#"name="pattern""#).unwrap()..new.find(r#"name="tiled picture""#).unwrap()]
            .contains("<a:ln")
    );
    let new = edit_every_element(|part| {
        named(part, "tiled picture").properties.line = Some(Box::new(Line {
            width: Some(1),
            ..Line::default()
        }));
    });
    let tile_at = new.find(r#"name="tiled picture""#).unwrap();
    assert!(new[tile_at..].contains(r#"</a:blipFill><a:ln w="1"/></p:spPr>"#));
}

#[test]
fn blip_effects_a14_picture_effects_and_hidden_extensions_are_edited_in_place() {
    let new = edit_every_element(|part| {
        let Some(Fill::Blip(b)) = named(part, "stretched picture").properties.fill.as_mut() else {
            panic!()
        };
        let blip = b.blip.as_mut().unwrap();
        let layer = blip
            .image_properties
            .as_mut()
            .unwrap()
            .layer
            .as_mut()
            .unwrap();
        layer.effects[0].visible = Some(false);
    });
    assert!(new.contains(r#"<a14:imgEffect visible="0"><a14:artisticBlur radius="10"/>"#));
    assert!(new.contains(r#"{BEBA8EAE-BF5A-486C-A8C5-ECC9F3942E4B}"#));

    let new = edit_every_element(|part| {
        named(part, "hidden")
            .properties
            .hidden
            .as_mut()
            .unwrap()
            .line = None;
    });
    assert!(!new.contains("hiddenLine"), "the entry goes");
    assert!(!new.contains("{91240B29"), "and its holder");
    assert!(new.contains("hiddenFill") && new.contains("hiddenScene3d"));

    let new = edit_every_element(|part| {
        let line = named(part, "solid and full line")
            .properties
            .line
            .as_mut()
            .unwrap();
        line.sketch.as_mut().unwrap().seed_attribute = Some(7);
    });
    assert_only(&new, r#"sd="12345""#, r#"sd="7""#);
}

#[test]
fn a_path_command_edit_in_a_custom_geometry_is_in_place() {
    let new = edit_every_element(|part| {
        let Some(Geometry::Custom(g)) = named(part, "custom").properties.geometry.as_mut() else {
            panic!()
        };
        let path = &mut g.paths.as_mut().unwrap()[0];
        if let PathCommand::LineTo(Some(p)) = &mut path.commands[1] {
            p.x = Some("26".to_owned());
        }
    });
    assert_only(&new, r#"<a:lnTo><a:pt x="25""#, r#"<a:lnTo><a:pt x="26""#);

    // Commands reordered: the list is rebuilt, and reads back in the new order.
    edit_every_element(|part| {
        let Some(Geometry::Custom(g)) = named(part, "custom").properties.geometry.as_mut() else {
            panic!()
        };
        g.paths.as_mut().unwrap()[0].commands.swap(1, 2);
    });
}

#[test]
fn a_picture_blip_fill_is_edited_on_the_picture() {
    let new = edit_every_element(|part| {
        let pic = named(part, "picture").properties.picture.as_mut().unwrap();
        pic.rotate_with_shape = Some(false);
    });
    assert_only(
        &new,
        r#"<p:blipFill rotWithShape="1">"#,
        r#"<p:blipFill rotWithShape="0">"#,
    );
}

#[test]
fn unknown_attributes_children_and_extensions_survive_an_edit() {
    let props = r#"<a:prstGeom prst="rect" foo="kept"><a:avLst/></a:prstGeom><a:solidFill bar="kept"><a:srgbClr val="112233" baz="kept"/><x:n xmlns:x="urn:x"/></a:solidFill><a:ln w="1" qux="kept"><x:m xmlns:x="urn:x"/><a:extLst><a:ext uri="{K}"><y:keep xmlns:y="urn:y"/></a:ext></a:extLst></a:ln><a:extLst><a:ext uri="{J}"><y:keep2 xmlns:y="urn:y"/></a:ext></a:extLst>"#;
    let xml = spt(props);
    let r = read(&xml);
    let mut after = r.part.clone();
    let node = after
        .shape_tree_mut()
        .unwrap()
        .children
        .first_mut()
        .unwrap();
    let Some(Fill::Solid(Some(color))) = node.properties.fill.as_mut() else {
        panic!()
    };
    color.value = ColorValue::Srgb(Rgb::new(0xAA, 0xBB, 0xCC));
    node.properties.line.as_mut().unwrap().width = Some(2);
    let mut tree = r.tree.clone();
    patch(&mut tree, &after).unwrap();
    let new = String::from_utf8(tree.to_bytes()).unwrap();
    assert_eq!(
        new,
        xml.replace(r#"val="112233""#, r#"val="AABBCC""#)
            .replace(r#"<a:ln w="1""#, r#"<a:ln w="2""#)
    );
    // Each thing the reader could not model was named.
    for name in ["n", "m", "keep", "keep2"] {
        assert!(
            r.findings.has(
                casual_slide_import::properties::ImportFindingKind::RetainedElement,
                name
            ),
            "{name}: {:?}",
            r.findings
        );
    }
}

#[test]
fn a_strict_part_writes_percentages_as_percentages() {
    let mut tree = XmlTree::parse(
        spt(r#"<a:ln><a:miter lim="50%"/></a:ln>"#).as_bytes(),
        PART,
        &OoxmlLimits::default(),
    )
    .unwrap();
    let r = read(&String::from_utf8(tree.to_bytes()).unwrap());
    let mut after = r.part.clone();
    let line = after.shape_tree_mut().unwrap().children[0]
        .properties
        .line
        .as_mut()
        .unwrap();
    line.join = Some(LineJoin::Miter(Some(Percentage(25_000))));
    patch(&mut tree, &after).unwrap();
    assert!(
        String::from_utf8(tree.to_bytes())
            .unwrap()
            .contains(r#"lim="25%""#),
        "follows the spelling it replaces"
    );
}

#[test]
fn a_value_the_reader_would_not_believe_is_refused_and_nothing_changes() {
    let xml = spt(r#"<a:ln w="1"/>"#);
    let r = read(&xml);
    let refused = |change: fn(&mut ShapeNode)| {
        let mut after = r.part.clone();
        change(
            after
                .shape_tree_mut()
                .unwrap()
                .children
                .first_mut()
                .unwrap(),
        );
        let mut tree = r.tree.clone();
        patch(&mut tree, &after).unwrap_err()
    };
    let code = refused(|n| n.properties.line.as_mut().unwrap().width = Some(-5));
    assert_eq!(code, "CS-EXP-0007", "a width below zero does not read back");
    let code = refused(|n| {
        n.properties.line.as_mut().unwrap().width = Some(30_000_000);
    });
    assert_eq!(code, "CS-EXP-0007", "a width over the schema's");
    let code = refused(|n| {
        n.properties.geometry = Some(Geometry::Custom(Box::new(
            casual_slide_model::geometry::CustomGeometry {
                guides: Some(vec![casual_slide_model::geometry::GeomGuide {
                    name: Some("a\u{1}b".to_owned()),
                    formula: None,
                }]),
                ..Default::default()
            },
        )));
    });
    assert_eq!(code, "CS-EXP-0001", "a character XML cannot carry");
    let code = refused(|n| {
        n.properties.fill = Some(Fill::Blip(Box::new(casual_slide_model::fill::BlipFill {
            blip: Some(casual_slide_model::fill::Blip {
                embed: Some("rId404".to_owned()),
                ..Default::default()
            }),
            ..Default::default()
        })));
    });
    assert_eq!(code, "CS-EXP-0003", "a relationship the part does not have");
    let code = refused(|n| {
        n.properties.hidden = Some(Box::new(casual_slide_model::drawing::HiddenProperties {
            shadow_obscured: Some(true),
            ..Default::default()
        }));
    });
    assert_eq!(
        code, "CS-EXP-0002",
        "an extension with no known uri cannot be added"
    );
}

#[test]
fn replacing_an_oversized_fill_the_model_could_not_hold_replaces_it() {
    let source =
        std::fs::read(repo().join("fixtures/hostile/drawing-limit-gradient-stops.bin")).unwrap();
    let mut wanted = model(&source);
    let ordinal = ordinal_of_slide(&wanted);
    let part = wanted
        .parts
        .iter_mut()
        .flatten()
        .find(|p| p.id.part == ordinal)
        .unwrap();
    let node = named(part, "s");
    assert_eq!(node.properties.fill, None, "left out of the model");
    node.properties.fill = Some(Fill::None);
    let edited: Vec<&SlidePart> = wanted
        .parts
        .iter()
        .flatten()
        .filter(|p| p.id.part == ordinal)
        .collect();
    let mut pkg = open(source.clone());
    let out = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &edited,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(model(&out.bytes), wanted);
    let text = String::from_utf8(parts(&out.bytes)[PART].clone()).unwrap();
    assert!(!text.contains("gradFill") && text.contains("<a:noFill/>"));
}

#[test]
fn a_node_in_alternate_content_is_not_edited_for_its_fill() {
    let xml = format!(
        r#"<p:sld {NS} xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><mc:AlternateContent><mc:Fallback><p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr><a:noFill/></p:spPr></p:sp></mc:Fallback></mc:AlternateContent></p:spTree></p:cSld></p:sld>"#
    );
    let r = read(&xml);
    let mut after = r.part.clone();
    let node = after
        .shape_tree_mut()
        .unwrap()
        .children
        .first_mut()
        .unwrap();
    assert_eq!(node.properties.fill, Some(Fill::None));
    node.properties.fill = Some(Fill::Group);
    let mut tree = r.tree.clone();
    assert_eq!(patch(&mut tree, &after).unwrap_err(), "CS-SLE-0002");
}

#[test]
fn a_sketch_and_a_hidden_fill_can_be_added_where_a_uri_is_known() {
    let new = edit_every_element(|part| {
        let node = named(part, "pattern");
        node.properties.line.as_mut().unwrap().sketch =
            Some(casual_slide_model::line::LineSketch {
                seed_attribute: Some(9),
                sketch_type: Some(Some(casual_slide_model::line::LineSketchType::Curved)),
                ..Default::default()
            });
        node.properties.hidden = Some(Box::new(casual_slide_model::drawing::HiddenProperties {
            fill: Some(Fill::None),
            ..Default::default()
        }));
    });
    let at = new.find(r#"name="pattern""#).unwrap();
    let slice = &new[at..new.find(r#"name="tiled picture""#).unwrap()];
    assert!(slice.contains("<ask:lineSketchCurved/>"), "{slice}");
    assert!(
        slice.contains("</a:ln><a:extLst>"),
        "extLst last, after ln: {slice}"
    );
    assert!(slice.contains("hiddenFill"), "{slice}");
}

#[test]
fn a_long_path_is_replaced_without_costing_its_square() {
    // Every name-keyed step here would be 400 000 000 compares as a scan of a scan.
    let cmds = r#"<a:lnTo><a:pt x="1" y="1"/></a:lnTo>"#.repeat(20_000);
    let xml = spt(&format!(
        "<a:custGeom><a:pathLst><a:path>{cmds}</a:path></a:pathLst></a:custGeom>"
    ));
    let r = read(&xml);
    let mut after = r.part.clone();
    let node = after
        .shape_tree_mut()
        .unwrap()
        .children
        .first_mut()
        .unwrap();
    let Some(Geometry::Custom(g)) = node.properties.geometry.as_mut() else {
        panic!()
    };
    g.paths.as_mut().unwrap()[0].commands = vec![PathCommand::Close; 20_000];
    let mut tree = r.tree.clone();
    patch(&mut tree, &after).unwrap();
    let new = String::from_utf8(tree.to_bytes()).unwrap();
    assert_eq!(new.matches("<a:close/>").count(), 20_000);
    assert!(!new.contains("lnTo"));
}

fn body_slide(body: &str, style: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><p:sp><p:nvSpPr><p:cNvPr id="2" name="s"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/>{style}<p:txBody>{body}<a:p/></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#
    )
}

/// From what one spelling reads as to another's model, and back: each way the bytes that
/// result read as asked, and hold the `wanted` strings in that order.
fn writes(from: &str, to: &str, wanted: &[&str]) -> String {
    let mut tree = read(from).tree;
    let after = read(to).part;
    patch(&mut tree, &after).unwrap_or_else(|e| panic!("{e}: {to}"));
    let out = String::from_utf8(tree.to_bytes()).unwrap();
    assert_eq!(read(&out).part, after, "{out}");
    let mut at = 0;
    for piece in wanted {
        let found = out[at..]
            .find(piece)
            .unwrap_or_else(|| panic!("{piece} not after {at} in {out}"));
        at += found + piece.len();
    }
    out
}

const WARP: &str = r#"<a:prstTxWarp prst="textWave1"><a:avLst><a:gd name="adj1" fmla="val 12500"/><a:gd name="adj2" fmla="val 0"/></a:avLst></a:prstTxWarp>"#;
const SCENE_XML: &str = r#"<a:scene3d><a:camera prst="orthographicFront"/><a:lightRig rig="threePt" dir="t"/></a:scene3d>"#;
const SP3D_XML: &str = r#"<a:sp3d z="1" extrusionH="2"><a:bevelT w="3" h="4"/></a:sp3d>"#;

#[test]
fn a_bodys_warp_scene_and_3d_are_made_from_nothing_at_their_schema_places() {
    let base = body_slide("<a:bodyPr/>", "");
    let full = body_slide(
        &format!(r#"<a:bodyPr wrap="none">{WARP}<a:noAutofit/>{SCENE_XML}{SP3D_XML}</a:bodyPr>"#),
        "",
    );
    writes(
        &base,
        &full,
        &[
            "prstTxWarp",
            "avLst",
            "adj1",
            "adj2",
            "noAutofit",
            "scene3d",
            "sp3d",
        ],
    );
    // And taken away again, all of it.
    let out = writes(&full, &base, &["bodyPr"]);
    assert!(!out.contains("prstTxWarp") && !out.contains("scene3d") && !out.contains("sp3d"));
    // With what is already there.
    let autofit = body_slide(
        r#"<a:bodyPr><a:spAutoFit/><a:extLst><a:ext uri="u"/></a:extLst></a:bodyPr>"#,
        "",
    );
    writes(
        &autofit,
        &body_slide(
            &format!(
                r#"<a:bodyPr>{WARP}<a:spAutoFit/>{SCENE_XML}<a:flatTx z="9"/><a:extLst><a:ext uri="u"/></a:extLst></a:bodyPr>"#
            ),
            "",
        ),
        &["prstTxWarp", "spAutoFit", "scene3d", "flatTx", "extLst"],
    );
}

#[test]
fn a_warp_adjust_edit_changes_that_attribute_and_no_other_byte() {
    let xml = body_slide(&format!("<a:bodyPr>{WARP}</a:bodyPr>"), "");
    let r = read(&xml);
    let mut after = r.part.clone();
    let body = after.shape_tree_mut().unwrap().children[0]
        .text_body
        .as_mut()
        .unwrap();
    body.properties
        .as_mut()
        .unwrap()
        .warp
        .as_mut()
        .unwrap()
        .adjust
        .as_mut()
        .unwrap()[1]
        .formula = Some("val 7".to_owned());
    let mut tree = r.tree;
    patch(&mut tree, &after).unwrap();
    assert_eq!(
        String::from_utf8(tree.to_bytes()).unwrap(),
        xml.replace(r#"name="adj2" fmla="val 0""#, r#"name="adj2" fmla="val 7""#)
    );
    // An absent list and an empty one differ: an emptied list stays.
    let emptied = body_slide(
        r#"<a:bodyPr><a:prstTxWarp prst="textWave1"><a:avLst/></a:prstTxWarp></a:bodyPr>"#,
        "",
    );
    let out = writes(&xml, &emptied, &["avLst"]);
    assert!(!out.contains("adj1"));
    let bare = body_slide(
        r#"<a:bodyPr><a:prstTxWarp prst="textWave1"/></a:bodyPr>"#,
        "",
    );
    let out = writes(&emptied, &bare, &["prstTxWarp"]);
    assert!(!out.contains("avLst"));
}

#[test]
fn the_text_3d_choice_replaces_itself_and_leaves_no_other() {
    let sp3d = body_slide(&format!("<a:bodyPr>{SP3D_XML}</a:bodyPr>"), "");
    let flat = body_slide(r#"<a:bodyPr><a:flatTx z="5"/></a:bodyPr>"#, "");
    let out = writes(&sp3d, &flat, &["flatTx"]);
    assert!(!out.contains("sp3d"), "{out}");
    let out = writes(&flat, &sp3d, &["sp3d"]);
    assert!(!out.contains("flatTx"), "{out}");
    // A file holding both reads the first and reports the second, which stays in the tree.
    // An edit choosing the second leaves only that, or the reader would find both.
    let both = body_slide(r#"<a:bodyPr><a:sp3d/><a:flatTx z="5"/></a:bodyPr>"#, "");
    assert!(!read(&both).findings.is_empty());
    let out = writes(&both, &flat, &["flatTx"]);
    assert!(!out.contains("sp3d"), "{out}");
}

/// `DRAW-01`, `MS-OI29500` 2.1.1330: every `ST_PositiveCoordinate` a shape's properties
/// hold stops at 2147483647 when it is new, and what a file already holds stays.
#[test]
fn a_new_coordinate_past_what_powerpoint_accepts_is_refused_and_nothing_changes() {
    const LIMIT: i64 = 2_147_483_647;
    let bodies: [&str; 14] = [
        r#"<a:sp3d extrusionH="{}"/>"#,
        r#"<a:sp3d contourW="{}"/>"#,
        r#"<a:sp3d><a:bevelT w="{}"/></a:sp3d>"#,
        r#"<a:sp3d><a:bevelB h="{}"/></a:sp3d>"#,
        r#"<a:effectLst><a:blur rad="{}"/></a:effectLst>"#,
        r#"<a:effectLst><a:glow rad="{}"><a:srgbClr val="000000"/></a:glow></a:effectLst>"#,
        r#"<a:effectLst><a:softEdge rad="{}"/></a:effectLst>"#,
        r#"<a:effectLst><a:innerShdw blurRad="{}"><a:srgbClr val="000000"/></a:innerShdw></a:effectLst>"#,
        r#"<a:effectLst><a:outerShdw dist="{}"><a:srgbClr val="000000"/></a:outerShdw></a:effectLst>"#,
        r#"<a:effectLst><a:prstShdw prst="shdw1" dist="{}"><a:srgbClr val="000000"/></a:prstShdw></a:effectLst>"#,
        r#"<a:effectLst><a:reflection blurRad="{}"/></a:effectLst>"#,
        r#"<a:effectLst><a:reflection dist="{}"/></a:effectLst>"#,
        r#"<a:custGeom><a:pathLst><a:path w="{}"/></a:pathLst></a:custGeom>"#,
        r#"<a:custGeom><a:pathLst><a:path h="{}"/></a:pathLst></a:custGeom>"#,
    ];
    for template in bodies {
        let with = |v: i64| spt(&template.replace("{}", &v.to_string()));
        let small = read(&with(1));
        let huge = read(&with(LIMIT + 1));
        let mut tree = small.tree.clone();
        let before = tree.to_bytes();
        assert_eq!(
            patch(&mut tree, &huge.part).unwrap_err(),
            "CS-EXP-0002",
            "{template}"
        );
        // Refusing happens before any save: the saved form of the attempt is not offered.
        let mut fresh = small.tree.clone();
        let exact = read(&with(LIMIT));
        patch(&mut fresh, &exact.part).unwrap_or_else(|e| panic!("{e}: {template}"));
        assert_eq!(
            read(std::str::from_utf8(&fresh.to_bytes()).unwrap()).part,
            exact.part
        );
        assert_ne!(fresh.to_bytes(), before);
        // A file saying more keeps it: an edit elsewhere leaves it, byte for byte.
        let mut tree = huge.tree.clone();
        let mut after = huge.part.clone();
        after.shape_tree_mut().unwrap().children[0].properties.line = Some(Box::new(Line {
            width: Some(1),
            ..Line::default()
        }));
        patch(&mut tree, &after).unwrap_or_else(|e| panic!("{e}: {template}"));
        let out = String::from_utf8(tree.to_bytes()).unwrap();
        assert!(out.contains(&(LIMIT + 1).to_string()), "{out}");
    }

    // On a body's 3D too, and on a shape's extent.
    let body = |v: i64| {
        body_slide(
            &format!(r#"<a:bodyPr><a:sp3d extrusionH="{v}"/></a:bodyPr>"#),
            "",
        )
    };
    let mut tree = read(&body(1)).tree;
    assert_eq!(
        patch(&mut tree, &read(&body(LIMIT + 1)).part).unwrap_err(),
        "CS-EXP-0002"
    );

    let size = |cx: i64, cy: i64| {
        spt(&format!(
            r#"<a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>"#
        ))
    };
    let mut tree = read(&size(1, 1)).tree;
    assert_eq!(
        patch(&mut tree, &read(&size(LIMIT + 1, 1)).part).unwrap_err(),
        "CS-EXP-0002"
    );
    let mut tree = read(&size(1, 1)).tree;
    assert_eq!(
        patch(&mut tree, &read(&size(1, LIMIT + 1)).part).unwrap_err(),
        "CS-EXP-0002"
    );
    let mut tree = read(&size(1, 1)).tree;
    patch(&mut tree, &read(&size(LIMIT, LIMIT)).part).unwrap();
    // An extent the file already holds does not stop a change to the other.
    let mut tree = read(&size(LIMIT + 1, 1)).tree;
    patch(&mut tree, &read(&size(LIMIT + 1, 2)).part).unwrap();
    assert!(
        String::from_utf8(tree.to_bytes())
            .unwrap()
            .contains(&(LIMIT + 1).to_string())
    );
}

/// `DRAW-01`, `MS-OI29500` 2.1.1292: a dash stop of zero or less is kept where a file has
/// one, and never written new.
#[test]
fn a_new_dash_stop_of_zero_or_less_is_refused() {
    let with = |d: i32, sp: i32| {
        spt(&format!(
            r#"<a:ln><a:custDash><a:ds d="{d}" sp="{sp}"/></a:custDash></a:ln>"#
        ))
    };
    for (d, sp) in [(0, 1), (1, 0)] {
        let mut tree = read(&with(1, 1)).tree;
        assert_eq!(
            patch(&mut tree, &read(&with(d, sp)).part).unwrap_err(),
            "CS-EXP-0002",
            "{d} {sp}"
        );
    }
    // Below zero the reader does not believe, so only a model says it.
    let r = read(&with(1, 1));
    let mut after = r.part.clone();
    let line = after.shape_tree_mut().unwrap().children[0]
        .properties
        .line
        .as_mut()
        .unwrap();
    let Some(LineDash::Custom(stops)) = line.dash.as_mut() else {
        panic!("no dash")
    };
    stops[0].dash = Some(Percentage(-5));
    let mut tree = r.tree.clone();
    assert_eq!(patch(&mut tree, &after).unwrap_err(), "CS-EXP-0002");
    let mut tree = read(&with(1, 1)).tree;
    patch(&mut tree, &read(&with(2, 3)).part).unwrap();
    let held = read(&with(0, 1));
    let mut tree = held.tree.clone();
    assert_eq!(patch(&mut tree, &held.part), Ok(0));
}

const STYLE_XML: &str = r#"<p:style><a:lnRef idx="1"><a:schemeClr val="accent1"/></a:lnRef><a:fillRef idx="2"><a:schemeClr val="accent2"/></a:fillRef><a:effectRef idx="0"><a:schemeClr val="accent3"/></a:effectRef><a:fontRef idx="minor"><a:schemeClr val="lt1"/></a:fontRef></p:style>"#;

#[test]
fn a_shape_style_is_edited_in_place_and_made_at_its_schema_place() {
    let xml = body_slide("<a:bodyPr/>", STYLE_XML);
    let r = read(&xml);
    assert!(r.findings.is_empty(), "{:?}", r.findings);
    let mut after = r.part.clone();
    let style = after.shape_tree_mut().unwrap().children[0]
        .style
        .as_mut()
        .unwrap();
    style.font.as_mut().unwrap().idx = FontCollectionIndex::from_xml("major");
    style.line.as_mut().unwrap().idx = Some(3);
    let mut tree = r.tree;
    patch(&mut tree, &after).unwrap();
    assert_eq!(
        String::from_utf8(tree.to_bytes()).unwrap(),
        xml.replace(r#"fontRef idx="minor""#, r#"fontRef idx="major""#)
            .replace(r#"lnRef idx="1""#, r#"lnRef idx="3""#)
    );

    // A colour is a choice inside its reference.
    let green = xml.replace(
        r#"<a:lnRef idx="1"><a:schemeClr val="accent1"/></a:lnRef>"#,
        r#"<a:lnRef idx="1"><a:srgbClr val="00FF00"/></a:lnRef>"#,
    );
    let out = writes(&xml, &green, &["lnRef", "srgbClr", "fillRef"]);
    assert!(!out.contains(r#"schemeClr val="accent1""#));

    // Made between `spPr` and `txBody`, and taken away.
    let bare = body_slide("<a:bodyPr/>", "");
    writes(
        &bare,
        &xml,
        &[
            "p:spPr",
            "p:style",
            "lnRef",
            "fillRef",
            "effectRef",
            "fontRef",
            "p:txBody",
        ],
    );
    let out = writes(&xml, &bare, &["p:txBody"]);
    assert!(!out.contains("p:style"));
}

#[test]
fn a_style_short_of_a_reference_or_on_a_group_is_refused() {
    let xml = body_slide("<a:bodyPr/>", STYLE_XML);
    let r = read(&xml);
    let mut after = r.part.clone();
    after.shape_tree_mut().unwrap().children[0]
        .style
        .as_mut()
        .unwrap()
        .effect = None;
    let mut tree = r.tree.clone();
    assert_eq!(patch(&mut tree, &after).unwrap_err(), "CS-EXP-0009");

    let mut after = r.part.clone();
    after.shape_tree_mut().unwrap().style = after.shape_tree().unwrap().children[0].style.clone();
    let mut tree = r.tree.clone();
    assert_eq!(patch(&mut tree, &after).unwrap_err(), "CS-SLE-0001");
}

/// `CT_Connection`, on the deck that shows it: edited in place, schema-valid, and it
/// reads back.
#[test]
fn a_connectors_ends_are_edited_in_place() {
    let source =
        std::fs::read(repo().join("fixtures/generated/text-object-defaults.pptx")).unwrap();
    let mut wanted = model(&source);
    let ordinal = ordinal_of_slide(&wanted);
    let slide = wanted.get_mut(ordinal).unwrap();
    let connector = named(slide, "Connector");
    assert_eq!(
        connector.start_connection,
        Some(casual_slide_model::shape::Connection { spid: 10, site: 1 })
    );
    connector.end_connection = Some(casual_slide_model::shape::Connection { spid: 11, site: 2 });
    let mut pkg = open(source.clone());
    let out = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &[wanted.get(ordinal).unwrap()],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(model(&out.bytes), wanted);
    let (old, new) = (parts(&source), parts(&out.bytes));
    let old = String::from_utf8(old["ppt/slides/slide1.xml"].clone()).unwrap();
    let new = String::from_utf8(new["ppt/slides/slide1.xml"].clone()).unwrap();
    assert_eq!(
        new,
        old.replace(
            r#"<a:endCxn id="12" idx="3"/>"#,
            r#"<a:endCxn id="11" idx="2"/>"#
        )
    );
    f4_validates(&source, &out.bytes, "connector ends");
}

/// Forced from a model through the writer, on the decks made for this: every part of a
/// shape can be taken away and put back, and each pass validates (per-tier gate 3).
#[test]
fn the_shape_decks_rewrite_from_the_model_and_stay_valid() {
    for deck in [
        "shape-fills",
        "text-warp-and-3d",
        "text-object-defaults",
        "drawing-extension-lists",
        "drawing-every-element",
    ] {
        let source = std::fs::read(repo().join(format!("fixtures/generated/{deck}.pptx"))).unwrap();
        let wanted = model(&source);
        let ordinal = ordinal_of_slide(&wanted);
        let mut emptied = wanted.clone();
        let tree_of = emptied.get_mut(ordinal).unwrap().shape_tree_mut().unwrap();
        fn strip(n: &mut ShapeNode) {
            // What cannot be added is left: a known `uri` is needed for an extension, and a
            // blip's `imgProps` cannot be made (T1E-04).
            let picture = n.properties.picture.take();
            let hidden = n.properties.hidden.take();
            let blip = n
                .properties
                .fill
                .take()
                .filter(|f| matches!(f, Fill::Blip(_)));
            n.properties = ShapeProperties {
                picture,
                hidden,
                fill: blip,
                ..ShapeProperties::default()
            };
            n.style = None;
            n.children.iter_mut().for_each(strip);
        }
        tree_of.children.iter_mut().for_each(strip);
        let mut pkg = open(source.clone());
        let stripped = save_slide_parts(
            &mut pkg,
            &emptied.structure,
            &[emptied.get(ordinal).unwrap()],
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap_or_else(|e| panic!("{deck}: {e}"));
        let mut pkg = open(stripped.bytes.clone());
        let rewritten = save_slide_parts(
            &mut pkg,
            &wanted.structure,
            &[wanted.get(ordinal).unwrap()],
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap_or_else(|e| panic!("{deck}: {e}"));
        assert_eq!(model(&rewritten.bytes), wanted, "{deck}: a fixed point");
        f4_validates(&source, &rewritten.bytes, deck);
        // For Microsoft's validator, which needs `dotnet` and runs outside this test.
        if let Some(dir) = std::env::var_os("REWRITTEN_DECKS_TO") {
            let dir = Path::new(&dir);
            std::fs::write(dir.join(format!("{deck}-rewritten.pptx")), &rewritten.bytes).unwrap();
            std::fs::write(dir.join(format!("{deck}-stripped.pptx")), &stripped.bytes).unwrap();
        }
    }
}
