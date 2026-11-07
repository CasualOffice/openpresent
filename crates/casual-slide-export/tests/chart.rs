//! Tier 1J export. Real and generated decks are a fixed point with their charts read.

use std::path::{Path, PathBuf};

use casual_slide_export::chart::patch_chart_part;
use casual_slide_import::chart::{ChartLimits, ChartStatus, DeckCharts};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_import::slide::{ImportedSlides, SlideLimits, import_slide_parts};
use casual_slide_model::chart::ChartPart;
use casual_slide_ooxml::{Conformance, OoxmlLimits, PresentationPackage, XmlTree};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
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

/// An unedited model patches nothing, part by part.
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

#[test]
fn every_committed_chart_is_read_and_a_fixed_point() {
    let mut total = 0;
    for set in ["real", "generated", "producers"] {
        for deck in decks(&fixtures().join(set)) {
            let name = deck.display().to_string();
            let mut pkg = open(std::fs::read(&deck).unwrap());
            let slides = imported(&mut pkg);
            total += assert_fixed_point(&mut pkg, &slides.charts, &name);
        }
    }
    assert!(total >= 4, "only {total} chart parts in the corpus");
}

/// `OPENPRESENT_REAL_DECKS`: the fetched decks of `fixtures/real/README.md`, where a run has
/// them. CI has none, so a run without the variable passes on the committed decks above.
#[test]
fn every_fetched_chart_is_read_and_a_fixed_point() {
    let Some(dir) = std::env::var_os("OPENPRESENT_REAL_DECKS") else {
        return;
    };
    let mut total = 0;
    let mut kinds = [0usize; 5];
    for deck in decks(Path::new(&dir)) {
        let name = deck.display().to_string();
        let mut pkg = open(std::fs::read(&deck).unwrap());
        let slides = imported(&mut pkg);
        let mut retained: Vec<String> = slides
            .findings
            .iter()
            .filter(|f| f.part.contains("chart") || f.part.contains("drawing"))
            .map(|f| {
                format!(
                    "{} {} {} {:?}",
                    f.kind.code(),
                    f.part,
                    f.element,
                    f.attribute
                )
            })
            .collect();
        retained.truncate(40);
        eprintln!("{name}: {} parts", slides.charts.parts.len());
        for line in retained {
            eprintln!("  {line}");
        }
        for chart in &slides.charts.parts {
            if let ChartStatus::Read(model) = &chart.status {
                let at = match model {
                    ChartPart::Chart(_) => 0,
                    ChartPart::ChartEx(_) => 1,
                    ChartPart::UserShapes(_) => 2,
                    ChartPart::Style(_) => 3,
                    ChartPart::Colors(_) => 4,
                };
                kinds[at] += 1;
            }
        }
        total += assert_fixed_point(&mut pkg, &slides.charts, &name);
    }
    eprintln!("{total} parts: {kinds:?}");
    assert!(total > 0);
}
