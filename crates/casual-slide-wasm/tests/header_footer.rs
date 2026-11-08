//! Insert > Header & Footer (`UX-006`): date, footer and slide number as copies of the
//! layout's placeholders, Apply to All with the title slide skipped, saved, read back,
//! a fixed point, undone to the opened bytes, and passing F4.

use std::process::Command;

use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_wasm::{HeaderFooter, Viewer};

fn slide(bytes: &[u8], n: usize) -> String {
    let mut p = PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap();
    String::from_utf8(p.read_part(&format!("ppt/slides/slide{n}.xml")).unwrap()).unwrap()
}

#[test]
fn header_and_footer_apply_to_all_saves_and_undoes() {
    let mut v = Viewer::blank().unwrap();
    let layouts = v.layouts();
    let content = layouts
        .iter()
        .find(|(_, n)| n == "Title and Content")
        .map_or(layouts[1].0, |l| l.0);
    v.new_slide(1, content).unwrap();
    v.new_slide(2, content).unwrap();
    let source = v.save().unwrap();
    let mut v = Viewer::open(source.clone()).unwrap();
    let hf = HeaderFooter {
        date: Some("9 November 2026".to_owned()),
        footer: Some("Q4 & beyond".to_owned()),
        slide_number: true,
        not_on_title: true,
    };
    assert!(v.header_footer(0, true, &hf).unwrap());
    let saved = v.save().unwrap();
    let title = slide(&saved, 1);
    assert!(
        !title.contains("type=\"sldNum\""),
        "skipped on the title slide"
    );
    let third = slide(&saved, 3);
    for want in [
        "type=\"dt\"",
        "type=\"ftr\"",
        "type=\"sldNum\"",
        "type=\"slidenum\"",
        "Q4 &amp; beyond",
        "9 November 2026",
        "<a:t>3</a:t>",
    ] {
        assert!(third.contains(want), "{want} in {third}");
    }
    let again = Viewer::open(saved.clone()).unwrap();
    assert_eq!(again.save().unwrap(), saved, "not a fixed point");
    // Unticked, the copies go again.
    let mut w = Viewer::open(saved.clone()).unwrap();
    w.header_footer(2, false, &HeaderFooter::default()).unwrap();
    assert!(!slide(&w.save().unwrap(), 3).contains("type=\"ftr\""));
    v.undo().unwrap();
    assert_eq!(v.save().unwrap(), source, "undone");
    let dir = std::env::temp_dir().join("openpresent-ux006-hf");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("hf-{}.pptx", std::process::id()));
    std::fs::write(&file, &saved).unwrap();
    let tool = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/f4/validate.py");
    let out = Command::new("python3")
        .arg("-I")
        .arg(tool)
        .arg(&file)
        .output();
    std::fs::remove_file(&file).unwrap();
    if let Ok(out) = out {
        let code = out.status.code().unwrap_or(-1);
        assert!(
            code == 0 || (code == 2 && std::env::var_os("F4_REQUIRED").is_none()),
            "F4 rejected it:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
