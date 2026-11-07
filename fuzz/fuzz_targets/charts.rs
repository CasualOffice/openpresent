#![no_main]
//! Any bytes as a chart-family part, and as what `c:chartSpace` and `cx:chartSpace` hold.
//! Import never panics. A patch of an unchanged model changes nothing. A write from the model
//! never panics. An edit the exporter accepts reads back as it was asked.

use casual_slide_export::chart::{patch_chart_part, rewrite_from_model};
use casual_slide_import::chart::{import_chart_part, ChartLimits, NS_C, NS_CX};
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::drawing::DrawingLimits;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_model::chart::{c, ChartPart};
use casual_slide_ooxml::{OoxmlLimits, XmlTree};
use libfuzzer_sys::fuzz_target;

const PART: &str = "ppt/charts/chart1.xml";
const DML: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

fn read(tree: &XmlTree) -> Option<ChartPart> {
    let import = ImportLimits::default();
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    import_chart_part(
        tree,
        PART,
        &[],
        &ChartLimits::default(),
        DrawingLimits::default(),
        &import,
        &mut findings,
        &mut color,
    )
    .ok()
}

fn patch(tree: &mut XmlTree, after: &ChartPart) -> Option<usize> {
    patch_chart_part(
        tree,
        PART,
        false,
        &[],
        after,
        &ChartLimits::default(),
        &ImportLimits::default(),
        &OoxmlLimits::default(),
    )
    .ok()
}

fn exercise(bytes: &[u8]) {
    let limits = OoxmlLimits::default();
    let Ok(tree) = XmlTree::parse(bytes, PART, &limits) else {
        return;
    };
    let Some(model) = read(&tree) else {
        return;
    };
    let mut same = tree.clone();
    assert_eq!(patch(&mut same, &model), Some(0), "an unchanged model patched something");
    assert_eq!(same.to_bytes(), tree.to_bytes());

    let root = tree.qualified_name(tree.root()).to_owned();
    let declared: String = tree
        .attributes(tree.root(), PART)
        .unwrap_or_default()
        .iter()
        .filter(|(k, _)| k.starts_with("xmlns"))
        .map(|(k, v)| format!(" {k}=\"{}\"", v.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;")))
        .collect();
    if let Ok(mut empty) = XmlTree::parse(format!("<{root}{declared}/>").as_bytes(), PART, &limits) {
        let _ = rewrite_from_model(&mut empty, PART, &[], &model);
    }

    if let ChartPart::Chart(space) = &model {
        let mut edited = space.clone();
        edited.rounded_corners = Some(Box::new(c::Boolean { val: Some(true) }));
        let edited = ChartPart::Chart(edited);
        let mut tree = tree.clone();
        if patch(&mut tree, &edited).is_some() {
            let again = XmlTree::parse(&tree.to_bytes(), PART, &limits).expect("what was patched parses");
            assert_eq!(read(&again).as_ref(), Some(&edited), "the edit reads back");
        }
    }
}

fuzz_target!(|data: &[u8]| {
    exercise(data);
    let Ok(inner) = std::str::from_utf8(data) else {
        return;
    };
    let c_part = format!(r#"<c:chartSpace xmlns:c="{}" xmlns:a="{DML}" xmlns:r="{REL}">{inner}</c:chartSpace>"#, NS_C[0]);
    exercise(c_part.as_bytes());
    let cx_part = format!(r#"<cx:chartSpace xmlns:cx="{}" xmlns:a="{DML}" xmlns:r="{REL}">{inner}</cx:chartSpace>"#, NS_CX[0]);
    exercise(cx_part.as_bytes());
});
