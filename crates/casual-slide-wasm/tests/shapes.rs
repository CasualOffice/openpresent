//! PowerPoint's shape experience (`SHAPE-01`): every preset of the Shapes gallery inserts
//! and saves a deck that opens again and passes the SDK validator (F4); a dragged
//! adjust handle saves its `a:avLst` and reads back to the same handle; Shape Fill,
//! Outline and Styles save and undo.

use std::path::Path;
use std::process::Command;

use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_wasm::{Ends, Viewer};

/// The first slide's XML: the blank deck's one slide.
fn first_slide_xml(bytes: &[u8]) -> String {
    let mut p = PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap();
    String::from_utf8(p.read_part("ppt/slides/slide1.xml").unwrap()).unwrap()
}

/// F4, as `tests/build.rs` runs it: skipped where `dotnet` is missing, unless
/// `OOXML_SDK_REQUIRED` is set.
fn sdk_validates(saved: &[u8], tag: &str) {
    let required = std::env::var_os("OOXML_SDK_REQUIRED").is_some();
    let python = ["python3", "python"]
        .into_iter()
        .find(|p| Command::new(p).arg("--version").output().is_ok());
    let Some(python) = python else {
        assert!(!required, "OOXML_SDK_REQUIRED is set and python is missing");
        return;
    };
    let dir = std::env::temp_dir().join("openpresent-vs09-sdk");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{tag}-{}.pptx", std::process::id()));
    std::fs::write(&file, saved).unwrap();
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
        _ => panic!("the SDK validator rejected the deck:\n{report}"),
    }
}

#[test]
fn every_preset_inserts_and_the_deck_saves_opens_and_validates() {
    let mut v = Viewer::blank().unwrap();
    let names: Vec<String> = casual_slide_geometry::preset_shapes()
        .iter()
        .map(|d| d.name.to_string())
        .collect();
    assert_eq!(names.len(), 187);
    for (i, name) in names.iter().enumerate() {
        let i = i as i64;
        let b = [(i % 16) * 700_000, (i / 16) * 520_000, 600_000, 450_000];
        let ends = if name == "straightConnector1" {
            Ends::Tail
        } else {
            Ends::Plain
        };
        v.insert_preset(0, name, ends, "Shape", b, (i % 3 == 1, i % 5 == 2))
            .unwrap();
    }
    assert!(
        v.insert_preset(0, "notAShape", Ends::Plain, "x", [0; 4], (false, false))
            .is_err()
    );
    let saved = v.save().unwrap();
    let xml = first_slide_xml(&saved);
    for name in &names {
        assert!(
            xml.contains(&format!("prst=\"{name}\"")),
            "{name} is not in the saved slide"
        );
    }
    assert!(xml.contains("<p:cxnSp>"));
    assert!(xml.contains("tailEnd type=\"triangle\""));
    assert!(xml.contains("flipH=\"1\""));
    let again = Viewer::open(saved.clone()).unwrap();
    assert_eq!(again.shapes(0).unwrap().len(), v.shapes(0).unwrap().len());
    sdk_validates(&saved, "presets");
}

#[test]
fn a_dragged_handle_saves_its_adjust_value_and_reads_back_the_same() {
    let mut v = Viewer::blank().unwrap();
    let (x, y) = (1_000_000.0, 2_000_000.0);
    let node = v
        .insert_preset(
            0,
            "roundRect",
            Ends::Plain,
            "Rectangle: Rounded Corners",
            [1_000_000, 2_000_000, 2_000_000, 1_000_000],
            (false, false),
        )
        .unwrap();
    let before = v.handles(0, node).unwrap();
    assert_eq!(before.len(), 2, "one handle, x and y");
    // The corner handle at a quarter of the short side: adj 25000.
    assert!(v.drag_handle(0, node, 0, (x + 300_000.0, y), true).unwrap());
    assert!(
        v.drag_handle(0, node, 0, (x + 250_000.0, y), false)
            .unwrap()
    );
    v.break_typing();
    let after = v.handles(0, node).unwrap();
    assert!((after[0] - (x + 250_000.0)).abs() < 1_000.0, "{after:?}");
    let saved = v.save().unwrap();
    let xml = first_slide_xml(&saved);
    assert!(
        xml.contains("<a:avLst><a:gd name=\"adj\" fmla=\"val 25000\"/></a:avLst>"),
        "{xml}"
    );
    let reopened = Viewer::open(saved.clone()).unwrap();
    assert_eq!(reopened.handles(0, node).unwrap(), after);
    // The whole drag is one undo step.
    v.undo().unwrap();
    assert_eq!(v.handles(0, node).unwrap(), before);

    // A shape with two handles keeps the one not dragged.
    let arrow = v
        .insert_preset(
            0,
            "rightArrow",
            Ends::Plain,
            "Arrow: Right",
            [0, 0, 2_000_000, 1_000_000],
            (false, false),
        )
        .unwrap();
    let h = v.handles(0, arrow).unwrap();
    assert_eq!(h.len(), 4);
    assert!(
        v.drag_handle(0, arrow, 1, (1_200_000.0, h[3]), true)
            .unwrap()
    );
    v.break_typing();
    let saved = v.save().unwrap();
    let xml = first_slide_xml(&saved);
    assert!(xml.contains("name=\"adj1\""), "{xml}");
    assert!(xml.contains("name=\"adj2\""), "{xml}");
    sdk_validates(&saved, "handles");
}

#[test]
fn fill_outline_and_style_save_and_undo() {
    let mut v = Viewer::blank().unwrap();
    let node = v
        .insert_preset(
            0,
            "ellipse",
            Ends::Plain,
            "Oval",
            [0, 0, 1_000_000, 1_000_000],
            (false, false),
        )
        .unwrap();
    let opened = v.save().unwrap();
    assert!(v.set_fill(0, node, "scheme accent2 lumMod 75000").unwrap());
    assert!(v.set_outline(0, node, 0, "srgb FF0000").unwrap());
    assert!(v.set_outline(0, node, 1, "38100").unwrap());
    assert!(v.set_outline(0, node, 2, "dash").unwrap());
    assert!(v.set_fill(0, node, "srgb nonsense").is_err());
    assert!(v.set_outline(0, node, 2, "wiggly").is_err());
    let saved = v.save().unwrap();
    let xml = first_slide_xml(&saved);
    assert!(
        xml.contains("<a:schemeClr val=\"accent2\"><a:lumMod val=\"75000\"/>"),
        "{xml}"
    );
    assert!(xml.contains("<a:ln w=\"38100\">"), "{xml}");
    assert!(xml.contains("<a:srgbClr val=\"FF0000\"/>"), "{xml}");
    assert!(xml.contains("<a:prstDash val=\"dash\"/>"), "{xml}");
    sdk_validates(&saved, "format");
    // A theme style takes the shape's own fill and line away.
    assert!(v.set_theme_style(0, node, 4, 3).unwrap());
    let xml = first_slide_xml(&v.save().unwrap());
    assert!(!xml.contains("FF0000"), "{xml}");
    assert!(
        xml.contains("<a:fillRef idx=\"3\"><a:schemeClr val=\"accent3\"/>"),
        "{xml}"
    );
    for _ in 0..5 {
        v.undo().unwrap();
    }
    assert_eq!(v.save().unwrap(), opened);
    assert_eq!(v.palette(0).unwrap().len(), 10);
    assert!(v.palette(0).unwrap().iter().all(|c| c.len() == 6));
}
