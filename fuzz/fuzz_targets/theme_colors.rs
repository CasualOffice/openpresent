#![no_main]
//! COLOR-01: any bytes as a theme, a theme override or a clipboard style sheet. Import
//! never panics and the report stays within its bound.
//! A name set on a theme that reads, reads back.

use casual_slide_export::theme::set_theme_name;
use casual_slide_import::color::Findings;
use casual_slide_import::theme::{
    import_clipboard_style_sheet, import_theme, import_theme_limited, import_theme_override,
};
use casual_slide_ooxml::{OoxmlLimits, XmlTree};
use libfuzzer_sys::fuzz_target;

const PART: &str = "part";

fn bounded(findings: &Findings, limits: &OoxmlLimits) {
    assert!(findings.items.len() <= limits.max_color_findings);
    assert!(findings.total >= findings.items.len());
}

fuzz_target!(|data: &[u8]| {
    let limits = OoxmlLimits::default();
    let Ok(tree) = XmlTree::parse(data, PART, &limits) else {
        return;
    };
    if let Ok(got) = import_theme(&tree) {
        bounded(&got.findings, &limits);
        let mut edited = tree.clone();
        if matches!(set_theme_name(&mut edited, "Fuzz"), Ok(true)) {
            let bytes = edited.to_bytes();
            let again = XmlTree::parse(&bytes, PART, &limits).unwrap();
            assert_eq!(import_theme(&again).unwrap().value.name.as_deref(), Some("Fuzz"));
        }
    }
    let tight = OoxmlLimits {
        max_color_transforms: 1,
        max_theme_list_entries: 1,
        max_color_findings: 1,
        ..OoxmlLimits::default()
    };
    if let Ok(got) = import_theme_limited(&tree, &tight) {
        bounded(&got.findings, &tight);
    }
    if let Ok(got) = import_theme_override(&tree) {
        bounded(&got.findings, &limits);
    }
    let got = import_clipboard_style_sheet(&tree);
    bounded(&got.findings, &limits);
});
