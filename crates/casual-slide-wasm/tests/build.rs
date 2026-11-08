//! Building a deck like PowerPoint (`VS-08`): New Slide, Duplicate, Delete, Hide,
//! reorder, Insert Text Box, Shape and Picture, and File > New. Each op undoes and
//! redoes, a save writes a deck that opens again as the model said, undoing every edit
//! saves the opened bytes, and a deck built from blank passes the SDK validator (F4).

use std::path::Path;
use std::process::Command;

use casual_slide_model::shape::ObjectId;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_transaction::Operation;
use casual_slide_wasm::{Insert, Viewer};

/// A 1 by 1 PNG.
const PNG: [u8; 70] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0xDA, 0x63, 0x64, 0x60, 0xF8, 0x5F,
    0x0F, 0x00, 0x02, 0x87, 0x01, 0x80, 0xEB, 0x47, 0xBA, 0x92, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

const BOX: [i64; 4] = [914_400, 914_400, 2_743_200, 914_400];

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated")
        .join(name);
    std::fs::read(path).unwrap()
}

fn slide_xml(bytes: &[u8], index: usize) -> String {
    let v = Viewer::open(bytes.to_vec()).unwrap();
    let _ = v.slide_count();
    let mut p = PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap();
    // The slide's part, by the deck's own order.
    let main = String::from_utf8(p.read_part("ppt/presentation.xml").unwrap()).unwrap();
    let rels = String::from_utf8(p.read_part("ppt/_rels/presentation.xml.rels").unwrap()).unwrap();
    let ids: Vec<&str> = main
        .split("<p:sldId ")
        .skip(1)
        .filter_map(|s| s.split("r:id=\"").nth(1)?.split('"').next())
        .collect();
    let rel = ids[index];
    let target = rels
        .split("<Relationship ")
        .find(|r| r.contains(&format!("Id=\"{rel}\"")))
        .and_then(|r| r.split("Target=\"").nth(1)?.split('"').next())
        .unwrap();
    String::from_utf8(p.read_part(&format!("ppt/{target}")).unwrap()).unwrap()
}

fn type_into(v: &mut Viewer, index: usize, node: u32, text: &str) {
    let object = ObjectId {
        part: v.slide_part(index).unwrap(),
        node,
    };
    v.break_typing();
    v.edit(
        Operation::InsertText {
            object,
            at: 0,
            text: text.to_owned(),
        },
        true,
    )
    .unwrap();
    v.break_typing();
}

/// Everything a user does to build a small deck from File > New.
fn build_a_deck(v: &mut Viewer) {
    let layouts = v.layouts();
    let names: Vec<&str> = layouts.iter().map(|(_, n)| n.as_str()).collect();
    assert_eq!(
        names,
        ["Title Slide", "Title and Content", "Title Only", "Blank"]
    );
    // Slide 1: type a title into the Title Slide's placeholder.
    let title = v.shapes(0).unwrap()[0].node;
    type_into(v, 0, title, "Built here");
    // Slide 2: Title and Content, with a text box, shapes, lines and a picture.
    v.new_slide(1, layouts[1].0).unwrap();
    let text_box = v.insert_shape(1, Insert::TextBox, BOX).unwrap();
    type_into(v, 1, text_box, "In a text box");
    v.insert_shape(1, Insert::Rectangle, BOX).unwrap();
    v.insert_shape(1, Insert::Oval, BOX).unwrap();
    v.insert_shape(1, Insert::Line, [0, 0, 1_828_800, 0])
        .unwrap();
    v.insert_shape(1, Insert::Arrow, [0, 457_200, 1_828_800, 0])
        .unwrap();
    v.insert_picture(1, PNG.to_vec(), [4_572_000, 0, 914_400, 914_400])
        .unwrap();
    // Slide 3: Blank; then duplicate slide 2, hide the copy, move it first.
    v.new_slide(2, layouts[3].0).unwrap();
    v.duplicate_slide(1).unwrap();
    v.set_hidden(2, true).unwrap();
    v.move_slide(2, 0).unwrap();
    assert_eq!(v.slide_count(), 4);
}

#[test]
fn the_blank_deck_opens_and_saves_unchanged() {
    let v = Viewer::blank().unwrap();
    assert_eq!(v.slide_count(), 1);
    assert_eq!(v.slide_size(), (12_192_000, 6_858_000));
    let bytes = casual_slide_export::blank::blank_deck().unwrap();
    assert_eq!(v.save().unwrap(), bytes);
    // The same build gives the same bytes.
    assert_eq!(casual_slide_export::blank::blank_deck().unwrap(), bytes);
}

#[test]
fn a_deck_built_from_blank_saves_and_opens_as_built() {
    let mut v = Viewer::blank().unwrap();
    build_a_deck(&mut v);
    let saved = v.save().unwrap();
    let mut again = Viewer::open(saved.clone()).unwrap();
    assert_eq!(again.slide_count(), 4);
    assert!(again.is_hidden(0).unwrap(), "the copy is hidden");
    assert!(!again.is_hidden(2).unwrap());
    // The copy and its original hold the same shapes: two placeholders and six more.
    assert_eq!(again.shapes(0).unwrap().len(), 8);
    assert_eq!(again.shapes(2).unwrap().len(), 8);
    let first = slide_xml(&saved, 1);
    assert!(first.contains("Built here"), "{first}");
    let built = slide_xml(&saved, 2);
    for wanted in [
        "In a text box",
        "txBox=\"1\"",
        "prst=\"ellipse\"",
        "<p:cxnSp>",
        "tailEnd type=\"triangle\"",
        "<p:pic>",
        "r:embed=",
    ] {
        assert!(built.contains(wanted), "{wanted} missing from {built}");
    }
    // A save of the reopened deck changes nothing, and every slide draws.
    assert_eq!(again.save().unwrap(), saved);
    for i in 0..4 {
        again.render_png(i, 160).unwrap();
    }
}

#[test]
fn undoing_every_build_step_saves_the_opened_bytes_and_redo_saves_the_same_deck() {
    let mut v = Viewer::blank().unwrap();
    let source = v.save().unwrap();
    build_a_deck(&mut v);
    let built = v.save().unwrap();
    while v.can_undo_redo().0 {
        v.undo().unwrap();
    }
    assert_eq!(v.slide_count(), 1);
    assert_eq!(v.save().unwrap(), source);
    while v.can_undo_redo().1 {
        v.redo().unwrap();
    }
    assert_eq!(v.save().unwrap(), built);
}

#[test]
fn each_slide_op_round_trips_on_a_deck_from_the_corpus() {
    let source = fixture("minimal.pptx");
    type Step = fn(&mut Viewer);
    let steps: [(&str, Step, usize); 6] = [
        (
            "duplicate",
            |v| {
                v.duplicate_slide(0).unwrap();
            },
            2,
        ),
        (
            "new slide",
            |v| {
                let layout = v.layout_of(0).unwrap();
                v.new_slide(0, layout).unwrap();
            },
            2,
        ),
        ("delete", |v| v.delete_slide(0).unwrap(), 0),
        ("hide", |v| v.set_hidden(0, true).unwrap(), 1),
        (
            "insert",
            |v| {
                v.insert_shape(0, Insert::Oval, BOX).unwrap();
            },
            1,
        ),
        (
            "move",
            |v| {
                v.duplicate_slide(0).unwrap();
                v.move_slide(1, 0).unwrap();
            },
            2,
        ),
    ];
    for (what, step, count) in steps {
        let mut v = Viewer::open(source.clone()).unwrap();
        step(&mut v);
        let saved = v.save().unwrap();
        assert_ne!(saved, source, "{what} changed nothing");
        let again = Viewer::open(saved.clone()).unwrap();
        assert_eq!(again.slide_count(), count, "{what}");
        assert_eq!(again.save().unwrap(), saved, "{what} is not a fixed point");
        while v.can_undo_redo().0 {
            v.undo().unwrap();
        }
        assert_eq!(v.save().unwrap(), source, "{what} undone");
    }
}

#[test]
fn a_file_that_is_not_a_picture_is_refused_and_changes_nothing() {
    let mut v = Viewer::blank().unwrap();
    let err = v
        .insert_picture(0, b"GIF89a not ours".to_vec(), BOX)
        .unwrap_err();
    assert_eq!(err.code(), "CS-WSM-0008");
    assert!(!v.can_undo_redo().0);
}

/// F4: Microsoft's Open XML SDK validator (`tools/ooxml-sdk`) on a deck built from
/// blank. It skips where `dotnet` is missing, unless `OOXML_SDK_REQUIRED` is set.
#[test]
fn a_deck_built_from_blank_passes_the_sdk_validator() {
    let mut v = Viewer::blank().unwrap();
    build_a_deck(&mut v);
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
    let dir = std::env::temp_dir().join("openpresent-vs08-sdk");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("built-{}.pptx", std::process::id()));
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
        _ => panic!("the SDK validator rejected the built deck:\n{report}"),
    }
}

/// `VS-18`: PowerPoint's blank Title Slide centres its title and subtitle; the blank
/// deck's layout must say so, since its master's `titleStyle` says left.
#[test]
fn the_blank_decks_title_and_subtitle_draw_centred() {
    let mut v = Viewer::blank().unwrap();
    let shapes = v.shapes(0).unwrap();
    for (i, text) in [(0, "Hi"), (1, "Sub")] {
        let node = shapes[i].node;
        type_into(&mut v, 0, node, text);
        let object = ObjectId {
            part: v.slide_part(0).unwrap(),
            node,
        };
        let start = v.caret_rect(0, object, 0).unwrap()[0];
        let end = v.caret_rect(0, object, text.len()).unwrap()[0];
        // Both placeholders run from 1524000 to 10668000 EMU: the middle is 6096000.
        let middle = (start + end) / 2.0;
        assert!(
            (middle - 6_096_000.0).abs() < 20_000.0,
            "{text}: drawn from {start} to {end}"
        );
    }
}

/// `VS-18`: Ctrl+U draws a line under the run, right across the text.
#[test]
fn an_underlined_title_draws_a_line_under_its_text() {
    let mut v = Viewer::blank().unwrap();
    let node = v.shapes(0).unwrap()[0].node;
    type_into(&mut v, 0, node, "HELLO");
    let object = ObjectId {
        part: v.slide_part(0).unwrap(),
        node,
    };
    let width = 960u32;
    let scale = f64::from(width) / 12_192_000.0;
    let start = v.caret_rect(0, object, 0).unwrap();
    let end = v.caret_rect(0, object, 5).unwrap();
    let (x0, x1) = (
        (start[0] * scale) as usize + 2,
        (end[0] * scale) as usize - 2,
    );
    // The most dark pixels in any one row across the text.
    let longest = |v: &mut Viewer| {
        let (w, rgba) = v.render_rgba(0, width).unwrap();
        let w = w as usize;
        (0..rgba.len() / 4 / w)
            .map(|y| (x0..x1).filter(|x| rgba[(y * w + x) * 4] < 100).count())
            .max()
            .unwrap()
    };
    let before = longest(&mut v);
    assert!(
        before < (x1 - x0) * 9 / 10,
        "no row is dark across: {before}"
    );
    v.toggle(object, 0, 5, casual_slide_wasm::Toggle::Underline)
        .unwrap();
    let after = longest(&mut v);
    assert_eq!(after, x1 - x0, "a row is dark right across the text");
}
