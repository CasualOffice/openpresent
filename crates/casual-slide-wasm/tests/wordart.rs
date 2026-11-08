//! Insert > WordArt (`UX-006`): each style is a text box whose run has the style's
//! fill and outline and whose body has its warp, saved, read back as written, a fixed
//! point, undone to the opened bytes, and passing F4.

use std::path::Path;
use std::process::Command;

use casual_slide_model::shape::ObjectId;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_wasm::{Insert, Viewer, WORDART_STYLES};

const BOX: [i64; 4] = [1_000_000, 1_000_000, 6_000_000, 1_200_000];

fn slide_xml(bytes: &[u8]) -> String {
    let mut p = PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap();
    String::from_utf8(p.read_part("ppt/slides/slide1.xml").unwrap()).unwrap()
}

#[test]
fn every_wordart_style_saves_with_its_fill_outline_and_warp() {
    let source = Viewer::blank().unwrap().save().unwrap();
    let mut v = Viewer::open(source.clone()).unwrap();
    let mut nodes = Vec::new();
    for n in 0..WORDART_STYLES.len() {
        nodes.push(v.insert_shape(0, Insert::WordArt(n), BOX).unwrap());
    }
    let part = v.outline(0).unwrap().part;
    let o = ObjectId {
        part,
        node: nodes[0],
    };
    let len = v.text_length(o).unwrap();
    assert_eq!(v.text_slice(o, 0, len).unwrap(), "Your text here");
    let saved = v.save().unwrap();
    let xml = slide_xml(&saved);
    for (_, fill, line, warp) in WORDART_STYLES {
        assert!(xml.contains(&format!("prst=\"{warp}\"")), "{warp}");
        assert!(xml.contains(&format!("val=\"{fill}\"")), "{fill}");
        if !line.is_empty() {
            assert!(xml.contains(&format!("val=\"{line}\"")), "{line}");
        }
    }
    let again = Viewer::open(saved.clone()).unwrap();
    assert_eq!(again.save().unwrap(), saved, "not a fixed point");
    while v.can_undo_redo().0 {
        v.undo().unwrap();
    }
    assert_eq!(v.save().unwrap(), source, "undone");
    // F4: xmllint and the schemas, where python and xmllint are present.
    let tool = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/f4/validate.py");
    let dir = std::env::temp_dir().join("openpresent-ux006");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("wordart-{}.pptx", std::process::id()));
    std::fs::write(&file, &saved).unwrap();
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
            "F4 rejected the WordArt deck:\n{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
