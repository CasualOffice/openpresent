// Slice 12's tests (`VS-12`): the Design tab's edits through the bridge, round-tripped
// through save.

use std::path::Path;
use std::process::Command;

use casual_slide_model::fill::Fill;
use casual_slide_model::slide::BackgroundFill;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

use super::*;

const DECK: &str = "area-text.pptx";

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../fixtures/generated/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(path).unwrap()
}

fn package(bytes: &[u8]) -> PresentationPackage {
    PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap()
}

/// Every part of two decks, by name, that differs, and those only one has.
fn changed_parts(a: &[u8], b: &[u8]) -> Vec<String> {
    let (mut pa, mut pb) = (package(a), package(b));
    let mut names = pa.part_names();
    for n in pb.part_names() {
        if !names.contains(&n) {
            names.push(n);
        }
    }
    names.sort();
    names
        .into_iter()
        .filter(|n| pa.read_part(n).ok() != pb.read_part(n).ok())
        .collect()
}

fn part_text(bytes: &[u8], name: &str) -> String {
    String::from_utf8(package(bytes).read_part(name).unwrap()).unwrap()
}

fn background(v: &Viewer, index: usize) -> Option<Fill> {
    let part = v.deck.parts().slide(index)?;
    match part.common.as_ref()?.background.as_ref()?.fill.as_ref()? {
        BackgroundFill::Properties(p) => p.fill.clone(),
        BackgroundFill::Reference(_) => None,
    }
}

#[test]
fn a_theme_changes_colours_and_fonts_and_saves_only_the_theme_part() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    let before = v.theme_colors();
    assert!(v.apply_theme(1, 0, false).unwrap());
    let colors = v.theme_colors();
    assert_ne!(colors, before);
    assert_eq!(colors.get(4), Some(&0x1F6FB2), "Harbour's accent 1");
    let saved = v.save().unwrap();
    let changed = changed_parts(&source, &saved);
    assert_eq!(changed.len(), 1, "{changed:?}");
    assert!(changed[0].contains("theme"), "{changed:?}");
    let theme = part_text(&saved, &changed[0]);
    assert!(theme.contains(r#"<a:clrScheme name="Harbour">"#), "{theme}");
    assert!(
        theme.contains(r#"<a:latin typeface="Georgia"/>"#),
        "{theme}"
    );

    let reopened = Viewer::open(saved).unwrap();
    assert_eq!(reopened.theme_colors(), colors, "the colours read back");
}

#[test]
fn a_variant_changes_colours_only_and_undo_gives_the_file_back() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    assert!(v.apply_theme(1, 2, true).unwrap());
    let saved = v.save().unwrap();
    let theme = part_text(&saved, &changed_parts(&source, &saved)[0]);
    assert!(
        theme.contains(r#"<a:latin typeface="Arial"/>"#),
        "fonts kept: {theme}"
    );
    assert!(theme.contains("16807A"), "{theme}");
    assert_eq!(v.undo().unwrap(), None);
    assert!(v.undid_wide(), "a theme redraws every slide");
    assert_eq!(v.save().unwrap(), source);
    v.redo().unwrap();
    assert_eq!(v.save().unwrap(), saved);
}

#[test]
fn a_bad_theme_or_variant_is_refused_and_changes_nothing() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    assert_eq!(
        v.apply_theme(99, 0, false).unwrap_err().code(),
        "CS-WSM-0006"
    );
    assert_eq!(
        v.apply_theme(0, 99, false).unwrap_err().code(),
        "CS-WSM-0006"
    );
    assert!(!v.can_undo_redo().0);
    assert_eq!(v.save().unwrap(), source);
}

#[test]
fn slide_size_scales_the_content_and_round_trips() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    let (ow, oh) = v.slide_size();
    let shapes_before = v.shapes(0).unwrap();
    assert!(
        v.set_slide_size(9_144_000, 6_858_000, Fit::EnsureFit)
            .unwrap()
    );
    assert_eq!(v.slide_size(), (9_144_000, 6_858_000));
    let k = (9_144_000.0 / ow as f64).min(6_858_000.0 / oh as f64);
    for (a, b) in shapes_before.iter().zip(v.shapes(0).unwrap()) {
        assert_eq!(a.node, b.node);
        assert!(
            ((a.cx as f64 * k) - b.cx as f64).abs() <= 1.0,
            "{a:?} {b:?}"
        );
        assert!(b.x >= 0 && b.x + b.cx <= 9_144_000, "on the slide: {b:?}");
    }
    let saved = v.save().unwrap();
    let main = package(&saved).main_part().to_owned();
    let pres = part_text(&saved, &main);
    assert!(
        pres.contains(r#"<p:sldSz cx="9144000" cy="6858000" type="screen4x3"/>"#),
        "{pres}"
    );
    let reopened = Viewer::open(saved).unwrap();
    assert_eq!(reopened.slide_size(), (9_144_000, 6_858_000));
    assert_eq!(reopened.shapes(0).unwrap(), v.shapes(0).unwrap());

    v.undo().unwrap();
    assert!(v.undid_wide(), "a new size redraws every slide");
    assert_eq!(v.slide_size(), (ow, oh));
    assert_eq!(v.save().unwrap(), source);
}

#[test]
fn a_slide_size_outside_the_schema_is_refused() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    let err = v.set_slide_size(100, 6_858_000, Fit::Maximize).unwrap_err();
    assert_eq!(err.code(), "CS-TXN-0007");
    assert_eq!(v.save().unwrap(), source);
}

#[test]
fn a_solid_background_on_one_slide_changes_that_slide_alone() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    assert!(
        v.set_background(Some(1), BackgroundChoice::Solid(0x336699))
            .unwrap()
    );
    let saved = v.save().unwrap();
    let changed = changed_parts(&source, &saved);
    assert_eq!(changed.len(), 1, "{changed:?}");
    let slide = part_text(&saved, &changed[0]);
    assert!(
        slide.contains(
            r#"<p:bg><p:bgPr><a:solidFill><a:srgbClr val="336699"/></a:solidFill><a:effectLst/></p:bgPr></p:bg>"#
        ),
        "{slide}"
    );
    assert!(
        slide.find("<p:bg>") < slide.find("<p:spTree>"),
        "bg first in cSld"
    );
    let reopened = Viewer::open(saved).unwrap();
    assert_eq!(background(&reopened, 1), background(&v, 1));
    assert!(background(&reopened, 0).is_none());
    // Undo is the slide alone, not every slide.
    assert_eq!(v.undo().unwrap(), Some(1));
    assert!(!v.undid_wide());
    assert_eq!(v.save().unwrap(), source);
}

#[test]
fn a_gradient_applied_to_all_then_changed_and_reset_round_trips() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    assert!(
        v.set_background(None, BackgroundChoice::Gradient(0xFFFFFF, 0x1F6FB2, 90))
            .unwrap()
    );
    for i in 0..v.slide_count() {
        assert!(
            matches!(background(&v, i), Some(Fill::Gradient(_))),
            "slide {i}"
        );
    }
    // A background that is already there is patched, not added again.
    assert!(
        v.set_background(Some(0), BackgroundChoice::Solid(0x000000))
            .unwrap()
    );
    let saved = v.save().unwrap();
    assert_eq!(changed_parts(&source, &saved).len(), v.slide_count());
    let reopened = Viewer::open(saved.clone()).unwrap();
    for i in 0..v.slide_count() {
        assert_eq!(background(&reopened, i), background(&v, i), "slide {i}");
    }
    let first = part_text(&saved, &changed_parts(&source, &saved)[0]);
    assert_eq!(first.matches("<p:bg>").count(), 1, "{first}");
    assert!(
        !first.contains("gradFill"),
        "the gradient replaced: {first}"
    );
    // Reset takes the slide's own background away again.
    assert!(v.set_background(Some(0), BackgroundChoice::Reset).unwrap());
    assert!(background(&v, 0).is_none());
    v.undo().unwrap();
    v.undo().unwrap();
    v.undo().unwrap();
    assert!(!v.can_undo_redo().0);
    assert_eq!(v.save().unwrap(), source);
}

/// Microsoft's Open XML SDK validator (`tools/ooxml-sdk`) on a deck with every Design
/// edit. It skips where `dotnet` is missing, unless `OOXML_SDK_REQUIRED` is set.
#[test]
fn the_redesigned_deck_passes_the_sdk_validator() {
    let mut v = Viewer::open(fixture(DECK)).unwrap();
    v.apply_theme(3, 1, false).unwrap();
    v.set_slide_size(9_144_000, 5_143_500, Fit::Maximize)
        .unwrap();
    v.set_background(None, BackgroundChoice::Gradient(0x112233, 0xFFEEDD, 45))
        .unwrap();
    v.set_background(Some(0), BackgroundChoice::Solid(0xABCDEF))
        .unwrap();
    let saved = v.save().unwrap();
    let required = std::env::var_os("OOXML_SDK_REQUIRED").is_some();
    let python = ["python3", "python"]
        .into_iter()
        .find(|p| Command::new(p).arg("--version").output().is_ok());
    let Some(python) = python else {
        assert!(!required, "OOXML_SDK_REQUIRED is set and python is missing");
        eprintln!("python not installed: SDK validation skipped");
        return;
    };
    let dir = std::env::temp_dir().join("openpresent-vs12-sdk");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("design-{}.pptx", std::process::id()));
    std::fs::write(&file, &saved).unwrap();
    let tool = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/ooxml-sdk/validate.py");
    let out = Command::new(python)
        .arg("-I")
        .arg(tool)
        .arg(&file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();
    let report =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    match out.status.code() {
        Some(0) => {}
        Some(2) if !required => eprintln!("SDK validator could not run: skipped\n{report}"),
        _ => panic!("the SDK validator rejected the saved deck:\n{report}"),
    }
}
