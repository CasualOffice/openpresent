//! Tier 1J import: charts read to the row, damage set aside with the deck open, workbooks
//! never read, and nothing dropped without a word.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use casual_slide_import::chart::{
    ChartBound, ChartFinding, ChartLimits, ChartStatus, DeckCharts, chart_of, import_chart_part,
    read_deck_charts,
};
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::drawing::DrawingLimits;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportFindingKind as K, ImportLimits};
use casual_slide_import::slide::{ImportedSlides, SlideLimits, import_slide_parts};
use casual_slide_model::chart::{ChartPart, c};
use casual_slide_model::properties::Float64;
use casual_slide_ooxml::{ElementId, OoxmlLimits, PresentationPackage, XmlTree};

const CLASSIC: &str = "generated/chart-every-row-classic.pptx";
const EX: &str = "generated/chart-every-row-ex.pptx";
const FAMILIES: &str = "generated/chart-families.pptx";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn open(path: &str) -> PresentationPackage {
    open_bytes(std::fs::read(fixtures().join(path)).unwrap())
}

fn open_bytes(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn imported(pkg: &mut PresentationPackage) -> ImportedSlides {
    let import = ImportLimits::default();
    let deck = import_deck(pkg, &import).unwrap().deck;
    import_slide_parts(pkg, &deck.presentation, &SlideLimits::default(), &import).unwrap()
}

fn charts_with(path: &str, limits: &ChartLimits) -> (DeckCharts, Findings) {
    let import = ImportLimits::default();
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    let mut pkg = open(path);
    let charts = read_deck_charts(
        &mut pkg,
        limits,
        DrawingLimits::default(),
        &import,
        &mut findings,
        &mut color,
    );
    (charts, findings)
}

fn read_part(xml: &[u8], part: &str) -> Option<(ChartPart, Vec<String>)> {
    let tree = XmlTree::parse(xml, part, &OoxmlLimits::default()).ok()?;
    let import = ImportLimits::default();
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    let model = import_chart_part(
        &tree,
        part,
        &[],
        &ChartLimits::default(),
        DrawingLimits::default(),
        &import,
        &mut findings,
        &mut color,
    )
    .ok()?;
    let mut report: Vec<String> = findings
        .iter()
        .map(|f| format!("{} {} {:?}", f.kind.code(), f.element, f.attribute))
        .collect();
    report.extend(
        color
            .items
            .iter()
            .map(|f| format!("{} {}", f.kind.code(), f.element)),
    );
    Some((model, report))
}

fn read_chart(status: &ChartStatus) -> &ChartPart {
    match status {
        ChartStatus::Read(model) => model,
        ChartStatus::SetAside { code } => panic!("set aside: {code}"),
    }
}

/// Every name in a part: `element`, and `element@attribute`, by local name.
fn collect_names(xml: &[u8], part: &str, out: &mut HashSet<String>) {
    let tree = XmlTree::parse(xml, part, &OoxmlLimits::default()).unwrap();
    let mut stack = vec![tree.root()];
    while let Some(id) = stack.pop() {
        let local = tree.local_name(id).to_owned();
        for (name, _) in tree.attributes(id, part).unwrap() {
            let bare = name.rsplit(':').next().unwrap().to_owned();
            out.insert(format!("{local}@{bare}"));
        }
        out.insert(local);
        stack.extend(tree.child_elements(id));
    }
}

/// What holds no element in any schema, so no well-formed part can hold it. Each is a row with
/// a named reason in `docs/spec`, and each is declared in the model.
fn unreachable_owner(owner: &str) -> bool {
    matches!(
        owner,
        "CT_BooleanTrue"
            | "CT_NumFilteredLiteralCache"
            | "CT_StrFilteredLiteralCache"
            | "CT_MultiLvlStrFilteredLiteralCache"
            | "CT_LiteralDataChart"
            | "CT_FeatureExtension"
            | "CT_GeoHierarchyEntity"
    )
}

/// The checklist's own rows: every CHART and CHARTEX element, attribute and value row is in the
/// every-row decks, each read with nothing reported as unread or unbelieved.
#[test]
fn every_chart_row_in_the_checklist_is_held_and_read() {
    let mut names = HashSet::new();
    let mut values = HashSet::new();
    for deck in [CLASSIC, EX] {
        let mut pkg = open(deck);
        let slides = imported(&mut pkg);
        assert!(slides.charts.parts.len() >= 13, "{deck}");
        for chart in &slides.charts.parts {
            read_chart(&chart.status);
            let xml = pkg.read_part(&chart.part).unwrap();
            collect_names(&xml, &chart.part, &mut names);
            values.insert(String::from_utf8(xml).unwrap());
        }
        for finding in slides.findings.iter() {
            let code = finding.kind.code();
            let unread = matches!(finding.kind, K::RetainedAttribute);
            assert!(!unread || finding.attribute.as_deref() == Some("mc:Ignorable"), "{deck}: {code} {finding:?}");
            assert!(
                !matches!(
                    finding.kind,
                    K::MalformedValue | K::OutOfRange | K::MissingRequired | K::DuplicateElement
                ),
                "{deck}: {code} {finding:?}"
            );
            if finding.kind == K::RetainedElement && finding.part.contains("chart") {
                let foreign = RETAINED_DML.contains(&finding.element.as_str());
                assert!(foreign, "{deck}: {} {} not read", finding.part, finding.element);
            }
        }
    }
    let list = std::fs::read_to_string(fixtures().join("../docs/spec/checklist.tsv")).unwrap();
    let mut checked = 0;
    for row in list.lines().skip(1) {
        let cols: Vec<&str> = row.split('\t').collect();
        let (area, owner, kind, name) = (cols[1], cols[2], cols[3], cols[4]);
        if !["CHART", "CHARTEX"].contains(&area) || unreachable_owner(owner) {
            continue;
        }
        match kind {
            "element" => {
                // An element in a type holding only an `a:` type, or a group of them, is named.
                assert!(
                    names.contains(name.rsplit(':').next().unwrap()),
                    "{}: no fixture holds it",
                    cols[0]
                );
            }
            "attribute" => {
                let bare = name.trim_start_matches('@').rsplit(':').next().unwrap();
                assert!(
                    names.iter().any(|n| n.ends_with(&format!("@{bare}"))),
                    "{}: no fixture holds it",
                    cols[0]
                );
            }
            "value" => {
                let value = name.trim_start_matches('=');
                assert!(
                    values.iter().any(|x| x.contains(&format!("\"{value}\""))
                        || x.contains(&format!(" {value}\""))
                        || x.contains(&format!(">{value}<"))),
                    "{}: no fixture holds it",
                    cols[0]
                );
            }
            _ => continue,
        }
        checked += 1;
    }
    assert!(checked > 1000, "only {checked} rows checked");
}

/// `a:` content a chart names and does not read. Each stays in the tree, and is reported.
const RETAINED_DML: &[&str] = &[
    "clrMapOvr",
    "lumMod",
    "defRPr",
    "bodyPr",
    "cNvPr",
    "cNvSpPr",
    "cNvCxnSpPr",
    "cNvPicPr",
    "cNvGraphicFramePr",
    "cNvGrpSpPr",
    "cNvContentPartPr",
    "style",
    "blipFill",
    "xfrm",
    "grpSpPr",
    "graphic",
    "extLst",
    "ext",
    "maxColor",
    "minColor",
    "midColor",
    "solidFill",
];

fn bar(space: &c::ChartSpace) -> Option<&c::BarChart> {
    space.charts().iter().find_map(|chart| match chart {
        c::PlotAreaCharts::BarChart(bar) => Some(&**bar),
        _ => None,
    })
}

#[test]
fn the_sixteen_families_read_with_their_series_axes_and_caches() {
    let mut pkg = open(FAMILIES);
    let slides = imported(&mut pkg);
    assert_eq!(slides.charts.parts.len(), 16);
    let mut kinds = Vec::new();
    for chart in &slides.charts.parts {
        let space = chart_of(read_chart(&chart.status)).unwrap();
        let [family] = space.charts() else {
            panic!("{}: one family", chart.part);
        };
        kinds.push(std::mem::discriminant(family));
        let style = space.alternate_style.as_ref().unwrap();
        assert_eq!(style.requires.as_deref(), Some("c14"));
        assert_eq!(style.choice.as_ref().unwrap().val, Some(102));
        assert_eq!(style.fallback.as_ref().unwrap().val, Some(2));
        assert!(space.style.is_none(), "the one inside the branch is not a second");
        assert!(space.sp_pr.as_ref().unwrap().fill.is_some());
        assert!(space.tx_pr.is_some());
    }
    let distinct: HashSet<_> = kinds.into_iter().collect();
    assert_eq!(distinct.len(), 16);

    // chart11 is the bar chart.
    let at = slides.charts.get("ppt/charts/chart11.xml").unwrap();
    let space = chart_of(read_chart(&at.status)).unwrap();
    let family = bar(space).unwrap();
    assert_eq!(family.ser.len(), 2);
    let val = family.ser[1].val.as_ref().unwrap();
    let cache = val.num_ref.as_ref().unwrap().num_cache.as_ref().unwrap();
    assert_eq!(cache.pt_count.as_ref().unwrap().val, Some(3));
    assert_eq!(cache.pt[2].v.as_deref(), Some("22"));
    assert_eq!(family.ax_id[0].val, Some(111));
    let overlap = family.overlap.as_ref().unwrap().val.as_deref();
    assert_eq!(overlap, Some("-27"), "as written");
    let plot = space.chart.as_ref().unwrap().plot_area.as_ref().unwrap();
    assert_eq!(plot.axes.len(), 2);
    assert!(matches!(plot.axes[0], c::PlotAreaAxes::CatAx(_)));
    let id = space.external_data.as_ref().unwrap().id.as_deref();
    assert_eq!(id, Some("rId1"));
}

#[test]
fn a_frame_names_its_chart_and_a_series_keeps_its_unique_id() {
    let mut pkg = open(FAMILIES);
    let slides = imported(&mut pkg);
    let tree = slides.parts.parts.iter().flatten().find_map(|part| {
        let tree = part.shape_tree()?;
        tree.children.iter().any(|n| n.chart.is_some()).then_some(tree)
    });
    let frame = tree.unwrap().children.iter().find(|n| n.chart.is_some()).unwrap();
    let chart = frame.chart.as_ref().unwrap();
    assert!(!chart.extended);
    assert_eq!(chart.rel_id.as_deref(), Some("rId2"));

    let at = slides.charts.get("ppt/charts/chart1.xml").unwrap();
    let space = chart_of(read_chart(&at.status)).unwrap();
    let c::PlotAreaCharts::AreaChart(area) = &space.charts()[0] else {
        panic!("chart1 is the area chart");
    };
    let ext = area.ser[0].ext_lst.as_ref().unwrap();
    let item = &ext.ext[0].items[0];
    assert!(
        matches!(item, casual_slide_model::chart::ExtItem::C16UniqueId(_)),
        "{item:?}"
    );
}

#[test]
fn the_eight_extended_layouts_read_each_behind_its_fallback() {
    let mut pkg = open("generated/chart-ex-layouts.pptx");
    let slides = imported(&mut pkg);
    let mut layouts = Vec::new();
    for chart in &slides.charts.parts {
        let ChartPart::ChartEx(space) = read_chart(&chart.status) else {
            panic!("{}: not cx", chart.part);
        };
        let plot = space.chart.as_ref().unwrap().plot_area.as_ref().unwrap();
        let region = plot.plot_area_region.as_ref().unwrap();
        layouts.push(region.series[0].layout_id.clone().unwrap());
        let data = space.chart_data.as_ref().unwrap();
        assert_eq!(data.data.len(), 1);
    }
    assert_eq!(layouts.len(), 8);
    assert_eq!(
        layouts.into_iter().collect::<HashSet<_>>().len(),
        8,
        "each layout once"
    );
    assert_eq!(slides.charts.workbooks.len(), 8);
}

/// T1J-04: as `poi-bar-chart-mac`, whose ids are negative. Kept to the digit, and reported.
#[test]
fn a_negative_axis_id_is_kept_and_reported() {
    let mut pkg = open("generated/chart-signed-axis.pptx");
    let slides = imported(&mut pkg);
    let at = slides.charts.get("ppt/charts/chart1.xml").unwrap();
    let family = bar(chart_of(read_chart(&at.status)).unwrap()).unwrap().clone();
    assert_eq!(family.ax_id[0].val, Some(-1_884_094_432));
    assert_eq!(family.ax_id[1].val, Some(-2_096_236_376));
    let signed = K::Chart(ChartFinding::SignedAxisId);
    assert!(slides.findings.iter().any(|f| f.kind == signed));
    let unknown = K::Chart(ChartFinding::UnknownAxis);
    assert!(!slides.findings.iter().any(|f| f.kind == unknown));
}

#[test]
fn an_axis_id_outside_32_bits_is_left_out_and_reported() {
    let xml = format!(
        r#"<c:chartSpace xmlns:c="{}"><c:chart><c:plotArea><c:barChart><c:axId val="4294967296"/><c:axId val="-2147483649"/><c:axId val="4294967295"/></c:barChart></c:plotArea></c:chart></c:chartSpace>"#,
        casual_slide_import::chart::NS_C[0]
    );
    let (model, report) = read_part(xml.as_bytes(), "ppt/charts/chart1.xml").unwrap();
    let family = bar(chart_of(&model).unwrap()).unwrap().clone();
    let got: Vec<Option<i64>> = family.ax_id.iter().map(|a| a.val).collect();
    assert_eq!(got, [None, None, Some(4_294_967_295)]);
    assert!(report.iter().any(|f| f.starts_with("CS-IMF-0002 axId")), "{report:?}");
}

#[test]
fn a_chart_past_each_bound_is_set_aside_with_its_code_and_the_deck_opens() {
    for (name, part, bound) in [
        ("series", "ppt/charts/chart1.xml", ChartBound::Series),
        ("points", "ppt/charts/chart1.xml", ChartBound::Points),
        ("data-points", "ppt/charts/chart1.xml", ChartBound::DataPoints),
        ("data-labels", "ppt/charts/chart1.xml", ChartBound::DataLabels),
        ("levels", "ppt/charts/chart1.xml", ChartBound::Levels),
        ("extensions", "ppt/charts/chart1.xml", ChartBound::Extensions),
        ("anchors", "ppt/drawings/drawing1.xml", ChartBound::Anchors),
        ("group-depth", "ppt/drawings/drawing1.xml", ChartBound::GroupDepth),
    ] {
        let bytes = std::fs::read(fixtures().join(format!("hostile/chart-limit-{name}.bin"))).unwrap();
        let mut pkg = open_bytes(bytes.clone());
        let slides = imported(&mut pkg);
        let at = slides.charts.get(part).unwrap_or_else(|| panic!("{name}: no {part}"));
        assert_eq!(
            at.status,
            ChartStatus::SetAside { code: bound.code() },
            "{name}"
        );
        assert!(
            slides
                .findings
                .iter()
                .any(|f| f.kind == K::UnreadablePart && f.part == part),
            "{name}"
        );
        assert_eq!(pkg.save_unedited(), bytes, "{name}: set aside, kept");
    }
}

#[test]
fn just_below_each_bound_a_chart_is_read() {
    let limits = ChartLimits {
        max_series: 4_097,
        ..ChartLimits::default()
    };
    let (charts, _) = charts_with("hostile/chart-limit-series.bin", &limits);
    assert!(matches!(charts.parts[0].status, ChartStatus::Read(_)));
}

#[test]
fn more_chart_parts_than_the_bound_are_set_aside_and_the_rest_read() {
    let limits = ChartLimits {
        max_chart_parts: 3,
        ..ChartLimits::default()
    };
    let (charts, _) = charts_with(FAMILIES, &limits);
    assert_eq!(charts.parts.len(), 16);
    let aside = charts
        .parts
        .iter()
        .filter(|p| matches!(p.status, ChartStatus::SetAside { code: "CS-CHT-0001" }))
        .count();
    assert_eq!(aside, 13);
}

#[test]
fn damage_is_read_as_far_as_it_goes_reported_and_holds_no_document_text() {
    let mut pkg = open("hostile/chart-bad-structure.bin");
    let slides = imported(&mut pkg);
    let at = slides.charts.get("ppt/charts/chart1.xml").unwrap();
    let space = chart_of(read_chart(&at.status)).unwrap().clone();
    assert_eq!(space.date1904.as_ref().unwrap().val, None);
    let family = bar(&space).unwrap();
    let cache = family.ser[0].val.as_ref().unwrap();
    let cache = cache.num_ref.as_ref().unwrap().num_cache.as_ref().unwrap();
    assert_eq!(cache.pt.len(), 4, "every point kept");
    assert_eq!(cache.pt[2].v.as_deref(), Some("QzMarkerQz"), "as written: a cache is text");
    let codes: HashSet<&str> = slides.findings.iter().map(|f| f.kind.code()).collect();
    for code in [
        "CS-IMF-0001",
        "CS-IMF-0002",
        "CS-IMF-0003",
        "CS-IMF-0004",
        "CS-IMF-0006",
        "CS-IMF-0010",
        "CS-IMF-0011",
        "CS-CHF-0001",
        "CS-CHF-0002",
        "CS-CHF-0003",
        "CS-CHF-0004",
        "CS-CHF-0005",
    ] {
        assert!(codes.contains(code), "{code} not in {codes:?}");
    }
    let report = format!("{:?}", slides.findings);
    assert!(!report.contains("QzMarkerQz"), "{report}");
}

#[test]
fn a_chart_with_the_wrong_root_or_not_xml_is_set_aside_and_the_deck_opens() {
    let mut pkg = open("hostile/chart-wrong-root.bin");
    let slides = imported(&mut pkg);
    let at = slides.charts.get("ppt/charts/chart1.xml").unwrap();
    assert_eq!(at.status, ChartStatus::SetAside { code: "CS-IMP-0003" });

    let mut pkg = open(FAMILIES);
    let mut edits = casual_slide_package::PartEdits::default();
    edits.replace("ppt/charts/chart3.xml".to_owned(), b"<c:chartSpace".to_vec());
    let bytes = pkg.save_with(&edits).unwrap();
    let mut pkg = open_bytes(bytes.clone());
    let slides = imported(&mut pkg);
    let at = slides.charts.get("ppt/charts/chart3.xml").unwrap();
    assert!(matches!(at.status, ChartStatus::SetAside { .. }));
    let read = slides
        .charts
        .parts
        .iter()
        .filter(|p| matches!(p.status, ChartStatus::Read(_)))
        .count();
    assert_eq!(read, 15, "the others are read");
    assert_eq!(pkg.save_unedited(), bytes);
}

/// SEC-P09. If anything read a workbook, its size would show in what the package consumed.
#[test]
fn an_embedded_workbook_is_recorded_and_never_read() {
    let consumed = |workbook: &[u8]| {
        let mut pkg = open(FAMILIES);
        let mut edits = casual_slide_package::PartEdits::default();
        edits.replace("ppt/embeddings/Workbook1.xlsx".to_owned(), workbook.to_vec());
        let bytes = pkg.save_with(&edits).unwrap();
        let mut pkg = open_bytes(bytes);
        let before = pkg.consumed();
        let import = ImportLimits::default();
        let mut findings = Findings::new(import.max_findings);
        let mut color = ColorFindings::default();
        let charts = read_deck_charts(
            &mut pkg,
            &ChartLimits::default(),
            DrawingLimits::default(),
            &import,
            &mut findings,
            &mut color,
        );
        assert_eq!(charts.workbooks.len(), 16);
        let first = charts.workbooks.iter().find(|w| w.chart.ends_with("chart1.xml"));
        assert_eq!(
            first.unwrap().part.as_deref(),
            Some("ppt/embeddings/Workbook1.xlsx")
        );
        pkg.consumed() - before
    };
    // Bytes that do not compress, so the package's own expansion check is not what answers.
    let noise: Vec<u8> = (0..1u32 << 20)
        .map(|i| (i.wrapping_mul(2_654_435_761).rotate_left(13) >> 24) as u8)
        .collect();
    assert_eq!(consumed(&noise[..16]), consumed(&noise));
}

#[test]
fn a_workbook_nothing_could_read_leaves_the_chart_read() {
    let (charts, _) = charts_with("hostile/chart-workbook-corrupt.bin", &ChartLimits::default());
    assert!(matches!(charts.parts[0].status, ChartStatus::Read(_)));
    assert_eq!(charts.workbooks.len(), 1);
}

#[test]
fn chart_style_and_colors_read_with_their_chart() {
    let mut pkg = open(CLASSIC);
    let slides = imported(&mut pkg);
    let style = slides.charts.get("ppt/charts/style1.xml").unwrap();
    let ChartPart::Style(style) = read_chart(&style.status) else {
        panic!("a style part");
    };
    assert!(style.legend.is_some());
    assert_eq!(style.id, Some(1));
    let colors = slides.charts.get("ppt/charts/colors1.xml").unwrap();
    let ChartPart::Colors(colors) = read_chart(&colors.status) else {
        panic!("a colour part");
    };
    assert!(!colors.colors.is_empty());
    let shapes = slides.charts.get("ppt/drawings/drawing1.xml").unwrap();
    assert_eq!(shapes.owner, "ppt/charts/chart1.xml");
    assert!(matches!(read_chart(&shapes.status), ChartPart::UserShapes(_)));
}

#[test]
fn doubles_compare_by_value_and_a_nan_is_itself() {
    let xml = |val: &str| {
        format!(
            r#"<c:chartSpace xmlns:c="{}"><c:chart><c:plotArea><c:layout><c:manualLayout><c:x val="{val}"/></c:manualLayout></c:layout></c:plotArea></c:chart></c:chartSpace>"#,
            casual_slide_import::chart::NS_C[0]
        )
    };
    let read = |val: &str| read_part(xml(val).as_bytes(), "p").unwrap().0;
    assert_eq!(read("0.50"), read("0.5"));
    assert_eq!(read("NaN"), read("NaN"));
    assert_ne!(read("0.5"), read("0.6"));
    let model = read("-INF");
    let layout = chart_of(&model).unwrap().chart.as_ref().unwrap();
    let layout = layout.plot_area.as_ref().unwrap().layout.as_ref().unwrap();
    let x = layout.manual_layout.as_ref().unwrap().x.as_ref().unwrap();
    assert_eq!(x.val, Some(Float64::new(f64::NEG_INFINITY)));
}

fn elements(tree: &XmlTree) -> Vec<(ElementId, ElementId)> {
    let mut out = Vec::new();
    let mut stack = vec![tree.root()];
    while let Some(parent) = stack.pop() {
        for kid in tree.child_elements(parent) {
            out.push((parent, kid));
            stack.push(kid);
        }
    }
    out
}

/// A silent drop fails here (`docs/06`, per-tier gate 6). For each kind of element and attribute
/// in a part, by what holds it: take it out and read the part again. If neither the model nor the
/// report differs, the reader went past it and said nothing. DrawingML is Tier 1E's and 1F's,
/// which have the same test.
#[test]
fn taking_out_any_chart_node_changes_the_model_or_the_report() {
    let dml = casual_slide_import::properties::ns::DML;
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
        let mut pkg = open(deck);
        // By what holds it, across the deck: one read per kind of node.
        let mut seen = BTreeMap::new();
        for part in parts {
            let bytes = pkg.read_part(part).unwrap();
            let tree = XmlTree::parse(&bytes, part, &OoxmlLimits::default()).unwrap();
            let as_read = read_part(&bytes, part).unwrap();
            let is_dml = |id: ElementId| {
                tree.namespace(id).is_some_and(|n| dml.contains(&n.as_str()))
            };
            let mut checked = 0usize;
            for (parent, kid) in elements(&tree) {
                let above = tree.qualified_name(parent).to_owned();
                let name = tree.qualified_name(kid).to_owned();
                if is_dml(parent) || is_dml(kid) || name == "mc:Choice" || name == "mc:Fallback" {
                    continue;
                }
                // What the model only names is read no further: its own removal is noticed.
                let local = tree.local_name(parent);
                if RETAINED_DML.contains(&local) && local != "extLst" && local != "ext" && local != "style" {
                    continue;
                }
                if seen.insert((above.clone(), name.clone()), ()).is_none() {
                    let mut gone = tree.clone();
                    gone.remove_child(parent, kid);
                    let after = read_part(&gone.to_bytes(), part);
                    assert_ne!(after.as_ref(), Some(&as_read), "{part}: {above}/{name} went unnoticed");
                    checked += 1;
                }
                for (attr, _) in tree.attributes(kid, part).unwrap() {
                    let kid_local = tree.local_name(kid);
                    let named = RETAINED_DML.contains(&kid_local) && kid_local != "ext" && kid_local != "style"
                        || name == "cdr:ext";
                    if attr.starts_with("xmlns") || attr == "mc:Ignorable" || named {
                        continue;
                    }
                    let key = (name.clone(), format!("@{attr}"));
                    if seen.insert(key, ()).is_some() {
                        continue;
                    }
                    let mut gone = tree.clone();
                    gone.remove_attribute(kid, &attr);
                    let after = read_part(&gone.to_bytes(), part);
                    assert_ne!(after.as_ref(), Some(&as_read), "{part}: {name}@{attr} went unnoticed");
                    checked += 1;
                }
            }
            assert!(checked > 0 || part.ends_with("2.xml"), "{part}");
        }
    }
}
