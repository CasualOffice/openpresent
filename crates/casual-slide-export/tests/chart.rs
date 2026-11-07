//! Tier 1J export. Edits touch named nodes alone; every row is written from nothing, and
//! removed, and put back; real and generated decks are a fixed point; edited output validates;
//! what the model only names cannot be made.
//!
//! `OPENPRESENT_REAL_DECKS` names the fetched decks of `fixtures/real/README.md` where a run has
//! them, and CI has none, so a run without it holds the committed decks only. Edited decks stay
//! under `CHART_EDITED_OUT` where that is set, for Microsoft's validator.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use casual_slide_export::chart::{patch_chart_part, rewrite_from_model, save_chart_parts};
use casual_slide_export::properties::ExportError;
use casual_slide_import::chart::{
    ChartLimits, ChartStatus, DeckCharts, NS_C, NS_CS, NS_CX, import_chart_part,
};
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::drawing::DrawingLimits;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportLimits, ns};
use casual_slide_import::slide::{ImportedSlides, SlideLimits, import_slide_parts};
use casual_slide_model::chart::{ChartPart, ExtItem, Retained, c};
use casual_slide_model::color::{Color, ColorValue, Rgb};
use casual_slide_model::fill::Fill;
use casual_slide_ooxml::{
    Conformance, ElementId, OoxmlLimits, PresentationPackage, Relationship, TargetMode, XmlTree,
};

const CLASSIC: &str = "generated/chart-every-row-classic.pptx";
const EX: &str = "generated/chart-every-row-ex.pptx";
const FAMILIES: &str = "generated/chart-families.pptx";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

fn imported(pkg: &mut PresentationPackage) -> ImportedSlides {
    let import = ImportLimits::default();
    let deck = import_deck(pkg, &import).unwrap().deck;
    import_slide_parts(pkg, &deck.presentation, &SlideLimits::default(), &import).unwrap()
}

fn decks(dir: &Path) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = read
        .filter_map(|e| Some(e.ok()?.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "pptx"))
        .collect();
    out.sort();
    out
}

fn real_decks() -> Vec<PathBuf> {
    let mut out = decks(&fixtures().join("real"));
    if let Some(dir) = std::env::var_os("OPENPRESENT_REAL_DECKS") {
        out.extend(decks(Path::new(&dir)));
    }
    out
}

fn read_part(tree: &XmlTree, part: &str) -> ChartPart {
    let import = ImportLimits::default();
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    import_chart_part(
        tree,
        part,
        &[],
        &ChartLimits::default(),
        DrawingLimits::default(),
        &import,
        &mut findings,
        &mut color,
    )
    .unwrap()
}

/// The ids the every-row parts name, so a node holding one can be put back.
fn known_rels() -> Vec<Relationship> {
    ["rId1", "rId2", "rId5"]
        .iter()
        .map(|id| Relationship {
            id: (*id).to_owned(),
            rel_type: String::new(),
            target: "x".to_owned(),
            mode: TargetMode::Internal,
        })
        .collect()
}

fn patch(tree: &mut XmlTree, part: &str, after: &ChartPart) -> Result<usize, ExportError> {
    patch_chart_part(
        tree,
        part,
        false,
        &known_rels(),
        after,
        &ChartLimits::default(),
        &ImportLimits::default(),
        &OoxmlLimits::default(),
    )
}

fn parts_of(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut pkg = open(bytes.to_vec());
    pkg.part_names()
        .into_iter()
        .map(|n| {
            let data = pkg.read_part(&n).unwrap();
            (n, data)
        })
        .collect()
}

/// F4, for a deck: `None` if valid or if the tools are not here.
fn f4_failures(bytes: &[u8]) -> Option<String> {
    let have = |tool: &str| Command::new(tool).arg("--version").output().is_ok();
    if !have("xmllint") || !have("python3") {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
        return None;
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("f4-chart");
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

fn keep_for_validator(name: &str, bytes: &[u8]) {
    if let Some(dir) = std::env::var_os("CHART_EDITED_OUT") {
        let dir = Path::new(&dir);
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(format!("{name}.pptx")), bytes).unwrap();
    }
}

fn assert_fixed_point(pkg: &mut PresentationPackage, charts: &DeckCharts, name: &str) -> usize {
    let strict = pkg.conformance() == Conformance::Strict;
    let xml = *pkg.limits();
    let mut read = 0;
    for chart in &charts.parts {
        let ChartStatus::Read(model) = &chart.status else {
            panic!("{name}: {} set aside: {:?}", chart.part, chart.status);
        };
        let bytes = pkg.read_part(&chart.part).unwrap();
        let mut tree = XmlTree::parse(&bytes, &chart.part, &xml).unwrap();
        let rels = pkg.relationships_of(&chart.part).to_vec();
        let changes = patch_chart_part(
            &mut tree,
            &chart.part,
            strict,
            &rels,
            model,
            &ChartLimits::default(),
            &ImportLimits::default(),
            &xml,
        )
        .unwrap();
        assert_eq!(changes, 0, "{name}: {} changed", chart.part);
        assert_eq!(tree.to_bytes(), bytes, "{name}: {}", chart.part);
        read += 1;
    }
    read
}

fn generated_chart_decks() -> Vec<PathBuf> {
    decks(&fixtures().join("generated"))
        .into_iter()
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("chart-"))
        })
        .collect()
}

#[test]
fn every_chart_is_read_and_a_fixed_point_with_its_save_byte_identical() {
    let mut total = 0;
    let mut sets = real_decks();
    sets.extend(generated_chart_decks());
    sets.extend(decks(&fixtures().join("producers")));
    for deck in sets {
        let name = deck.display().to_string();
        let source = std::fs::read(&deck).unwrap();
        let mut pkg = open(source.clone());
        let slides = imported(&mut pkg);
        total += assert_fixed_point(&mut pkg, &slides.charts, &name);
        let edits: Vec<(String, ChartPart)> = slides
            .charts
            .parts
            .iter()
            .filter_map(|p| match &p.status {
                ChartStatus::Read(m) => Some((p.part.clone(), m.clone())),
                ChartStatus::SetAside { .. } => None,
            })
            .collect();
        let saved = save_chart_parts(
            &mut pkg,
            &edits,
            &ChartLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap();
        assert_eq!(saved.bytes, source, "{name}: unedited, byte identical");
        assert!(saved.changed_parts.is_empty(), "{name}");
    }
    assert!(total >= 100, "only {total} chart parts");
}

fn model_of(slides: &ImportedSlides, part: &str) -> ChartPart {
    match &slides.charts.get(part).unwrap().status {
        ChartStatus::Read(model) => model.clone(),
        ChartStatus::SetAside { code } => panic!("{part}: {code}"),
    }
}

fn first_bar(model: &mut ChartPart) -> &mut c::BarChart {
    let ChartPart::Chart(space) = model else {
        panic!("not a chart")
    };
    let plot = space.chart.as_mut().unwrap().plot_area.as_mut().unwrap();
    plot.charts
        .iter_mut()
        .find_map(|f| match f {
            c::PlotAreaCharts::BarChart(bar) => Some(&mut **bar),
            _ => None,
        })
        .unwrap()
}

/// Exactly this replacement, in this part, and every other part byte-identical.
fn only_this_changed(source: &[u8], edited: &[u8], part: &str, from: &str, to: &str) {
    let (old, new) = (parts_of(source), parts_of(edited));
    let before = String::from_utf8(old[part].clone()).unwrap();
    let after = String::from_utf8(new[part].clone()).unwrap();
    assert_eq!(before.matches(from).count(), 1, "{from} in {part}");
    assert_eq!(after, before.replace(from, to));
    assert_eq!(old.len(), new.len(), "parts lost or invented");
    for (name, bytes) in &old {
        if name != part {
            assert_eq!(new.get(name), Some(bytes), "{name} changed");
        }
    }
}

#[test]
fn an_edit_to_a_cached_value_changes_that_node_and_nothing_else() {
    let source = read_fixture(FAMILIES);
    let mut pkg = open(source.clone());
    let slides = imported(&mut pkg);
    let part = "ppt/charts/chart11.xml";
    let mut model = model_of(&slides, part);
    {
        let bar = first_bar(&mut model);
        let val = bar.ser[0].val.as_mut().unwrap();
        let cache = val.num_ref.as_mut().unwrap().num_cache.as_mut().unwrap();
        cache.pt[1].v = Some("999".to_owned());
    }
    let saved = save_chart_parts(
        &mut pkg,
        &[(part.to_owned(), model.clone())],
        &ChartLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(saved.changed_parts, [part]);
    only_this_changed(
        &source,
        &saved.bytes,
        part,
        r#"<c:pt idx="1"><c:v>11</c:v></c:pt><c:pt idx="2"><c:v>12</c:v>"#,
        r#"<c:pt idx="1"><c:v>999</c:v></c:pt><c:pt idx="2"><c:v>12</c:v>"#,
    );
    assert_eq!(f4_failures(&saved.bytes), None);
    keep_for_validator("edit-value", &saved.bytes);

    // Reopens as edited, and is then a fixed point.
    let mut again = open(saved.bytes.clone());
    let slides = imported(&mut again);
    assert_eq!(model_of(&slides, part), model);
    assert_fixed_point(&mut again, &slides.charts, "edited");
}

#[test]
fn an_edit_to_a_series_fill_goes_through_the_drawing_writer_and_keeps_the_rest() {
    let source = read_fixture(FAMILIES);
    let mut pkg = open(source.clone());
    let slides = imported(&mut pkg);
    let part = "ppt/charts/chart11.xml";
    let mut model = model_of(&slides, part);
    {
        let bar = first_bar(&mut model);
        let props = casual_slide_model::drawing::ShapeProperties {
            fill: Some(Fill::Solid(Some(Color::new(ColorValue::Srgb(Rgb::new(
                1, 2, 3,
            )))))),
            ..Default::default()
        };
        bar.ser[0].sp_pr = Some(Box::new(props));
    }
    let saved = save_chart_parts(
        &mut pkg,
        &[(part.to_owned(), model.clone())],
        &ChartLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(saved.changed_parts, [part]);
    let new = String::from_utf8(parts_of(&saved.bytes)[part].clone()).unwrap();
    assert!(new.contains(r#"<c:spPr><a:solidFill><a:srgbClr val="010203"/></a:solidFill></c:spPr><c:invertIfNegative"#), "{new}");
    assert!(new.contains("uniqueId"), "the extension stays");
    assert_eq!(f4_failures(&saved.bytes), None);
    keep_for_validator("edit-fill", &saved.bytes);
}

#[test]
fn a_negative_axis_id_edited_stays_signed_and_the_other_stays_as_written() {
    let source = read_fixture("generated/chart-signed-axis.pptx");
    let mut pkg = open(source.clone());
    let slides = imported(&mut pkg);
    let part = "ppt/charts/chart1.xml";
    let mut model = model_of(&slides, part);
    first_bar(&mut model).gap_width = Some(Box::new(c::GapAmount {
        val: Some("120".to_owned()),
    }));
    let saved = save_chart_parts(
        &mut pkg,
        &[(part.to_owned(), model)],
        &ChartLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    let new = String::from_utf8(parts_of(&saved.bytes)[part].clone()).unwrap();
    assert!(new.contains(r#"<c:gapWidth val="120"/><c:axId val="-1884094432"/>"#), "{new}");
    assert_eq!(f4_failures(&saved.bytes), None);
    keep_for_validator("edit-signed-axis", &saved.bytes);
}

#[test]
fn every_real_deck_with_a_chart_takes_an_edit_and_stays_as_valid_as_it_was() {
    let mut edited = 0;
    for deck in real_decks() {
        let name = deck.file_stem().unwrap().to_string_lossy().into_owned();
        let source = std::fs::read(&deck).unwrap();
        let mut pkg = open(source.clone());
        let slides = imported(&mut pkg);
        let mut edits = Vec::new();
        for chart in &slides.charts.parts {
            let ChartStatus::Read(model) = &chart.status else {
                panic!("{name}: {}", chart.part);
            };
            let mut model = model.clone();
            match &mut model {
                ChartPart::Chart(space) => {
                    let was = space.rounded_corners.as_ref().and_then(|b| b.val);
                    space.rounded_corners = Some(Box::new(c::Boolean {
                        val: Some(was != Some(true)),
                    }));
                }
                ChartPart::ChartEx(space) => {
                    let data = space.chart_data.as_mut().unwrap();
                    data.data[0].id = data.data[0].id.map(|i| i + 1);
                }
                _ => continue,
            }
            edits.push((chart.part.clone(), model));
        }
        if edits.is_empty() {
            continue;
        }
        let before_f4 = f4_failures(&source);
        let names: HashSet<String> = edits.iter().map(|e| e.0.clone()).collect();
        let saved = save_chart_parts(
            &mut pkg,
            &edits,
            &ChartLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap();
        let (old, new) = (parts_of(&source), parts_of(&saved.bytes));
        for (part, bytes) in &old {
            if !names.contains(part) {
                assert_eq!(new.get(part), Some(bytes), "{name}: {part} changed");
            }
        }
        // Only what the edit named: the part loses no byte beyond its own nodes.
        let mut again = open(saved.bytes.clone());
        let slides = imported(&mut again);
        for (part, model) in &edits {
            assert_eq!(&model_of(&slides, part), model, "{name}: {part}");
        }
        assert_fixed_point(&mut again, &slides.charts, &name);
        assert_eq!(f4_failures(&saved.bytes), before_f4, "{name}");
        keep_for_validator(&format!("real-{name}"), &saved.bytes);
        edited += 1;
    }
    assert!(edited >= 1, "no real deck with a chart");
}

/// The empty root of a part, in its own namespace.
fn empty_root(source: &XmlTree, part: &str) -> XmlTree {
    let name = source.qualified_name(source.root()).to_owned();
    let declared: String = source
        .attributes(source.root(), part)
        .unwrap()
        .iter()
        .filter(|(k, _)| k.starts_with("xmlns"))
        .map(|(k, v)| format!(r#" {k}="{v}""#))
        .collect();
    let xml = format!("<{name}{declared}/>");
    XmlTree::parse(xml.as_bytes(), part, &OoxmlLimits::default()).unwrap()
}

/// `WR-01`, for this writer: from nothing, by the path a new child takes. What comes out reads
/// back as the model. Every part in every chart deck, real ones too: a row written wrongly, in
/// the wrong order among its siblings, or not at all, fails here or in the next test.
#[test]
fn every_chart_part_written_from_nothing_reads_back_as_its_model() {
    let mut count = 0;
    let mut sets = generated_chart_decks();
    sets.extend(real_decks());
    for deck in sets {
        let mut pkg = open(std::fs::read(&deck).unwrap());
        let slides = imported(&mut pkg);
        for chart in &slides.charts.parts {
            let ChartStatus::Read(model) = &chart.status else {
                continue;
            };
            let bytes = pkg.read_part(&chart.part).unwrap();
            let source = XmlTree::parse(&bytes, &chart.part, &OoxmlLimits::default()).unwrap();
            let mut tree = empty_root(&source, &chart.part);
            rewrite_from_model(&mut tree, &chart.part, &[], model).unwrap();
            let again = XmlTree::parse(&tree.to_bytes(), &chart.part, &OoxmlLimits::default()).unwrap();
            assert_eq!(&read_part(&again, &chart.part), model, "{deck:?} {}", chart.part);
            count += 1;
        }
    }
    assert!(count >= 50, "only {count}");
}

/// An element with its attributes sorted, so two trees compare by content.
fn canonical(tree: &XmlTree, id: ElementId, part: &str, out: &mut String) {
    let mut attrs: Vec<(String, String)> = tree
        .attributes(id, part)
        .unwrap()
        .into_iter()
        .filter(|(k, _)| !k.starts_with("xmlns"))
        .collect();
    attrs.sort();
    out.push_str(&format!("<{} {attrs:?}>", tree.qualified_name(id)));
    let kids = tree.child_elements(id);
    if kids.is_empty() {
        out.push_str(&tree.own_text(id).unwrap_or_default());
    }
    for kid in kids {
        canonical(tree, kid, part, out);
    }
    out.push_str("</>");
}

fn canon(tree: &XmlTree, part: &str) -> String {
    let mut out = String::new();
    canonical(tree, tree.root(), part, &mut out);
    out
}

/// An element the model only names: its removal is a change, and what it holds is not read.
fn is_named_only(qualified: &str) -> bool {
    let (prefix, local) = qualified.split_once(':').unwrap_or(("", qualified));
    match prefix {
        "c" => local == "clrMapOvr",
        "cdr" | "cdr14" => matches!(
            local,
            "cNvPr" | "cNvSpPr" | "cNvCxnSpPr" | "cNvPicPr" | "cNvGraphicFramePr" | "cNvGrpSpPr"
                | "cNvContentPartPr" | "style" | "blipFill" | "xfrm" | "grpSpPr" | "ext" | "extLst"
        ),
        "cs" => matches!(local, "defRPr" | "bodyPr" | "extLst"),
        "cx" => matches!(local, "maxColor" | "minColor" | "midColor"),
        _ => false,
    }
}

fn is_dml(tree: &XmlTree, id: ElementId) -> bool {
    tree.namespace(id).is_some_and(|n| ns::DML.contains(&n.as_str()))
}

/// Every kind of chart node, in a part holding each once: take it out of the tree, and the model
/// that reads as, patched onto the whole part, takes out that node and nothing else; and the
/// other way, the whole model patched onto the part without it puts that node back, in its place
/// among its siblings. So each row is held to removal and to insertion, and to "named nodes".
/// The checklist's chart rows are walked by `CHART`, `CHARTEX`: see `docs/spec/checklist.tsv`.
#[test]
fn every_chart_row_comes_out_and_goes_back_as_the_only_change() {
    let list = std::fs::read_to_string(fixtures().join("../docs/spec/checklist.tsv")).unwrap();
    let rows = list
        .lines()
        .filter(|l| {
            let area = l.split('\t').nth(1).unwrap_or_default();
            area == "CHART" || area == "CHARTEX"
        })
        .count();
    assert_eq!(rows, 1764, "the rows this tier owns");
    let mut checked = 0usize;
    for (deck, parts) in [
        (
            CLASSIC,
            vec![
                "ppt/charts/chart1.xml",
                "ppt/charts/chart2.xml",
                "ppt/drawings/drawing1.xml",
                "ppt/drawings/drawing2.xml",
                "ppt/charts/style1.xml",
                "ppt/charts/colors1.xml",
            ],
        ),
        (EX, vec!["ppt/charts/chartEx1.xml", "ppt/charts/chartEx2.xml"]),
    ] {
        let mut pkg = open(read_fixture(deck));
        let mut seen = HashSet::new();
        for part in parts {
            let bytes = pkg.read_part(part).unwrap();
            let whole = XmlTree::parse(&bytes, part, &OoxmlLimits::default()).unwrap();
            let full = read_part(&whole, part);
            let mut stack = vec![whole.root()];
            while let Some(parent) = stack.pop() {
                let parent_name = whole.qualified_name(parent).to_owned();
                let parent_dml = is_dml(&whole, parent);
                if is_named_only(&parent_name) {
                    continue;
                }
                // The drawing writers hold DrawingML, in their own tiers.
                for (attr, _) in whole.attributes(parent, part).unwrap() {
                    if parent_dml || attr.starts_with("xmlns") || attr == "mc:Ignorable" {
                        continue;
                    }
                    if parent_name.starts_with("mc:") || !seen.insert((parent_name.clone(), attr.clone())) {
                        continue;
                    }
                    let mut without = whole.clone();
                    without.remove_attribute(parent, &attr);
                    checked += check_both_ways(&whole, &without, &full, part, &format!("{parent_name}@{attr}"));
                }
                for kid in whole.child_elements(parent) {
                    stack.push(kid);
                    let name = whole.qualified_name(kid).to_owned();
                    let skip = parent_dml
                        || is_dml(&whole, kid)
                        || name.starts_with("mc:")
                        || parent_name.starts_with("mc:")
                        || !seen.insert((parent_name.clone(), name.clone()));
                    if skip {
                        continue;
                    }
                    let mut without = whole.clone();
                    without.remove_child(parent, kid);
                    checked += check_both_ways(&whole, &without, &full, part, &format!("{parent_name}/{name}"));
                }
            }
        }
    }
    assert!(checked > 700, "only {checked} nodes taken out and put back");
}

fn check_both_ways(whole: &XmlTree, without: &XmlTree, full: &ChartPart, part: &str, what: &str) -> usize {
    // A tree that no longer reads as a chart at all, such as a wholly emptied root, says nothing.
    let parsed = XmlTree::parse(&without.to_bytes(), part, &OoxmlLimits::default()).unwrap();
    let less = read_part(&parsed, part);
    if &less == full {
        panic!("{part}: {what}: taken out and the model did not change");
    }
    // Out: from the whole part to the model without it.
    let mut tree = whole.clone();
    patch(&mut tree, part, &less).unwrap_or_else(|e| panic!("{part}: {what}: out: {e}"));
    assert_eq!(canon(&tree, part), canon(without, part), "{part}: {what}: out");
    // Back: from the part without it to the whole model.
    let mut tree = parsed;
    match patch(&mut tree, part, full) {
        Ok(_) => {
            let back = XmlTree::parse(&tree.to_bytes(), part, &OoxmlLimits::default()).unwrap();
            assert_eq!(canon(&back, part), canon(whole, part), "{part}: {what}: back");
        }
        // What holds a `Retained` cannot be made again, which is what that type says.
        Err(ExportError::UnsupportedEdit { .. }) => {}
        Err(err) => panic!("{part}: {what}: back: {err}"),
    }
    2
}

/// Every element a `c14`, `c15` or `c16` schema gives an `ext`, each read as itself and none
/// left as something only retained (`T1J-02`).
#[test]
fn every_extension_root_reads_as_itself() {
    const ROOTS: &[&str] = &[
        "pivotOptions", "invertSolidFillFmt", "style", "pivotSource", "numFmt", "spPr", "layout",
        "fullRef", "levelRef", "formulaRef", "filteredSeriesTitle", "filteredCategoryTitle",
        "filteredAreaSeries", "filteredBarSeries", "filteredBubbleSeries", "filteredLineSeries",
        "filteredPieSeries", "filteredRadarSeries", "filteredScatterSeries",
        "filteredSurfaceSeries", "datalabelsRange", "categoryFilterExceptions", "dlblFieldTable",
        "xForSave", "showDataLabelsRange", "tx", "showLeaderLines", "leaderLines", "autoCat",
        "datapointuniqueidmap", "explosion", "invertIfNegative", "bubble3D", "marker", "dLbl",
        "pivotOptions16", "multiLvlStrLit", "dataDisplayOptions16", "formatcode2", "contentPart",
        "chartSpace", "userShapes", "chart", "chartStyle", "colorStyle",
    ];
    let mut pkg = open(read_fixture(CLASSIC));
    let slides = imported(&mut pkg);
    let bytes = pkg.read_part("ppt/charts/chart1.xml").unwrap();
    let text = String::from_utf8(bytes).unwrap();
    let drawing = pkg.read_part("ppt/drawings/drawing1.xml").unwrap();
    let drawing = String::from_utf8(drawing).unwrap();
    for root in ROOTS {
        let tag = format!(":{root}");
        assert!(text.contains(&tag) || drawing.contains(&tag) || ["chartStyle", "colorStyle"].contains(root), "{root}");
    }
    let ChartPart::Chart(space) = model_of(&slides, "ppt/charts/chart1.xml") else {
        panic!("a chart")
    };
    let items = &space.ext_lst.as_ref().unwrap().ext;
    assert!(items.len() >= 39, "{}", items.len());
    for ext in items {
        for item in &ext.items {
            assert!(!matches!(item, ExtItem::Other(_)), "{item:?}");
        }
    }
}

#[test]
fn a_change_of_family_order_rebuilds_that_list_in_the_order_asked() {
    let source = read_fixture("generated/chart-every-row-classic.pptx");
    let mut pkg = open(source);
    let bytes = pkg.read_part("ppt/charts/chart3.xml").unwrap();
    let mut tree = XmlTree::parse(&bytes, "p", &OoxmlLimits::default()).unwrap();
    let mut model = read_part(&tree, "p");
    let ChartPart::Chart(space) = &mut model else {
        panic!("a chart")
    };
    let plot = space.chart.as_mut().unwrap().plot_area.as_mut().unwrap();
    plot.charts.reverse();
    match patch(&mut tree, "p", &model) {
        Ok(_) => assert_eq!(read_part(&tree, "p"), model),
        Err(ExportError::UnsupportedEdit { .. }) => {}
        Err(err) => panic!("{err}"),
    }
}

#[test]
fn what_the_model_only_names_cannot_be_made_and_nothing_is_saved() {
    let source = read_fixture(FAMILIES);
    let mut pkg = open(source);
    let slides = imported(&mut pkg);
    let part = "ppt/charts/chart1.xml";
    let mut model = model_of(&slides, part);
    let ChartPart::Chart(space) = &mut model else {
        panic!("a chart")
    };
    space.clr_map_ovr = Some(Retained {
        uri: Some(NS_C[0].to_owned()),
        local: "clrMapOvr".to_owned(),
    });
    let err = save_chart_parts(
        &mut pkg,
        &[(part.to_owned(), model)],
        &ChartLimits::default(),
        &ImportLimits::default(),
    )
    .err()
    .unwrap();
    assert_eq!(err.code(), "CS-EXP-0009");
}

#[test]
fn a_value_the_reader_would_not_believe_is_refused_with_nothing_saved() {
    let source = read_fixture(FAMILIES);
    let mut pkg = open(source);
    let slides = imported(&mut pkg);
    let part = "ppt/charts/chart11.xml";
    let mut model = model_of(&slides, part);
    first_bar(&mut model).gap_width = Some(Box::new(c::GapAmount {
        val: Some("99999".to_owned()),
    }));
    let err = save_chart_parts(
        &mut pkg,
        &[(part.to_owned(), model)],
        &ChartLimits::default(),
        &ImportLimits::default(),
    )
    .err()
    .unwrap();
    assert_eq!(err.code(), "CS-EXP-0007");

    let mut model = model_of(&slides, part);
    first_bar(&mut model).ser[0].val.as_mut().unwrap().num_ref.as_mut().unwrap().f =
        Some("a\u{1}b".to_owned());
    let err = save_chart_parts(
        &mut pkg,
        &[(part.to_owned(), model)],
        &ChartLimits::default(),
        &ImportLimits::default(),
    )
    .err()
    .unwrap();
    assert_eq!(err.code(), "CS-EXP-0001");
}

#[test]
fn a_part_is_not_turned_into_another_kind_of_part() {
    let bytes = format!(r#"<c:chartSpace xmlns:c="{}"/>"#, NS_C[0]);
    let mut tree = XmlTree::parse(bytes.as_bytes(), "p", &OoxmlLimits::default()).unwrap();
    let other = ChartPart::Colors(Box::default());
    assert!(matches!(
        patch(&mut tree, "p", &other),
        Err(ExportError::UnsupportedEdit { .. })
    ));
    let _ = (NS_CS, NS_CX);
}

#[test]
fn an_edit_in_a_strict_deck_stays_strict_and_valid() {
    let source = read_fixture("generated/chart-strict.pptx");
    let mut pkg = open(source);
    assert_eq!(pkg.conformance(), Conformance::Strict);
    let slides = imported(&mut pkg);
    let part = "ppt/charts/chart11.xml";
    let mut model = model_of(&slides, part);
    first_bar(&mut model).gap_width = Some(Box::new(c::GapAmount {
        val: Some("120%".to_owned()),
    }));
    first_bar(&mut model).ser_lines = vec![c::ChartLines::default()];
    let saved = save_chart_parts(
        &mut pkg,
        &[(part.to_owned(), model)],
        &ChartLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    let new = String::from_utf8(parts_of(&saved.bytes)[part].clone()).unwrap();
    assert!(new.contains("purl.oclc.org"), "{new}");
    assert!(new.contains(r#"<c:gapWidth val="120%"/>"#), "{new}");
    assert_eq!(f4_failures(&saved.bytes), None);
    keep_for_validator("edit-strict", &saved.bytes);
}
