use std::path::Path;
use std::process::Command;

use casual_slide_model::text::TextItem;
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

/// The first text box on slide 0 whose body has text, and a point in its middle.
fn first_box(v: &Viewer) -> (ObjectId, f64, f64) {
    let text = v.deck.slide_text(0, None).unwrap();
    let b = text
        .boxes
        .iter()
        .find(|b| v.text_length(b.object).is_ok_and(|n| n > 3))
        .expect("a text box with text on slide 1");
    let (l, t, r, bottom) = b.rect;
    let (x, y) = b.transform.apply((l + r) / 2.0, (t + bottom) / 2.0);
    (b.object, x, y)
}

fn text_of(v: &Viewer, object: ObjectId) -> String {
    v.body(object)
        .unwrap()
        .paragraphs
        .iter()
        .map(|p| {
            p.items
                .iter()
                .map(|i| match i {
                    TextItem::Run(r) => r.text.clone().unwrap_or_default(),
                    _ => String::new(),
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every part of two decks, by name, that differs.
fn changed_parts(a: &[u8], b: &[u8]) -> Vec<String> {
    let mut pa = PresentationPackage::open(a.to_vec(), OoxmlLimits::default()).unwrap();
    let mut pb = PresentationPackage::open(b.to_vec(), OoxmlLimits::default()).unwrap();
    let mut names = pa.part_names();
    names.sort();
    let mut other = pb.part_names();
    other.sort();
    assert_eq!(names, other, "no part added or removed");
    names
        .into_iter()
        .filter(|n| pa.read_part(n).unwrap() != pb.read_part(n).unwrap())
        .collect()
}

fn part_text(bytes: &[u8], name: &str) -> String {
    let mut p = PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap();
    String::from_utf8(p.read_part(name).unwrap()).unwrap()
}

/// A click, typing and Bold, as a user makes them through the bridge.
fn edit_like_a_user(v: &mut Viewer) -> (ObjectId, bool) {
    let (object, x, y) = first_box(v);
    let (hit, at) = v
        .hit_text(0, x, y)
        .unwrap()
        .expect("the click lands in the box");
    assert_eq!(hit, object);
    v.break_typing();
    for (n, ch) in "Typed".chars().enumerate() {
        v.edit(
            Operation::InsertText {
                object,
                at: n,
                text: ch.to_string(),
            },
            true,
        )
        .unwrap();
    }
    let was_bold = v.format_state(object, 3).unwrap() & 1 == 1;
    v.toggle(object, 0, 5, Toggle::Bold).unwrap();
    let bold = v.format_state(object, 3).unwrap() & 1 == 1;
    assert_eq!(bold, !was_bold, "toggled");
    assert!(at <= v.text_length(object).unwrap());
    (object, bold)
}

#[test]
fn a_click_places_a_caret_inside_the_box_it_lands_in() {
    let v = Viewer::open(fixture(DECK)).unwrap();
    let (object, x, y) = first_box(&v);
    let (hit, at) = v.hit_text(0, x, y).unwrap().unwrap();
    assert_eq!(hit, object);
    let [x1, y1, x2, y2] = v.caret_rect(0, object, at).unwrap();
    assert!((x1 - x2).abs() < 1.0, "an upright caret");
    assert!(y2 > y1, "of some height");
    // Clicking where the caret is gives back the same offset.
    let again = v.offset_near(0, object, x1, (y1 + y2) / 2.0).unwrap();
    assert_eq!(again, at);
    // Home and End stay on the line; the selection of the line is one quad.
    let home = v.navigate(0, object, at, Move::Home).unwrap();
    let end = v.navigate(0, object, at, Move::End).unwrap();
    assert!(home <= at && at <= end);
    if home < end {
        assert_eq!(v.selection_rects(0, object, home, end).unwrap().len(), 8);
    }
}

#[test]
fn typed_and_bolded_text_round_trips_through_save() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    let (object, bold) = edit_like_a_user(&mut v);
    let typed = text_of(&v, object);
    assert!(typed.starts_with("Typed"));
    let after = v.format_state(object, 7).unwrap();
    let saved = v.save().unwrap();
    assert_ne!(saved, source);

    let reopened = Viewer::open(saved).unwrap();
    assert_eq!(text_of(&reopened, object), typed, "the text read back");
    assert_eq!(
        reopened.format_state(object, 3).unwrap() & 1 == 1,
        bold,
        "bold read back"
    );
    assert_eq!(
        reopened.format_state(object, 7).unwrap(),
        after,
        "and the text after it as it was"
    );
}

#[test]
fn only_the_edited_shape_changes_in_the_saved_file() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    let _ = edit_like_a_user(&mut v);
    let saved = v.save().unwrap();
    let changed = changed_parts(&source, &saved);
    assert_eq!(changed.len(), 1, "one part changed: {changed:?}");
    let name = &changed[0];
    assert!(
        name.starts_with("/ppt/slides/") || name.starts_with("ppt/slides/"),
        "{name}"
    );
    // In that part, every shape but the edited one is byte for byte as it was.
    let before = part_text(&source, name);
    let after = part_text(&saved, name);
    let pieces = |s: &str| s.split("<p:sp>").map(str::to_owned).collect::<Vec<_>>();
    let (a, b) = (pieces(&before), pieces(&after));
    assert_eq!(a.len(), b.len());
    let differing = a.iter().zip(&b).filter(|(x, y)| x != y).count();
    assert_eq!(differing, 1, "only the edited shape's element differs");
}

#[test]
fn undoing_every_edit_saves_the_file_that_was_opened() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    let _ = edit_like_a_user(&mut v);
    // Typing is one step, bold another.
    assert!(v.undo().unwrap().is_some());
    assert!(v.undo().unwrap().is_some());
    assert!(!v.can_undo_redo().0, "nothing left to undo");
    assert_eq!(v.save().unwrap(), source);
    // And redo brings both back.
    v.redo().unwrap();
    v.redo().unwrap();
    assert_ne!(v.save().unwrap(), source);
}

#[test]
fn a_bad_offset_is_refused_with_its_code_and_changes_nothing() {
    let source = fixture(DECK);
    let mut v = Viewer::open(source.clone()).unwrap();
    let (object, _, _) = first_box(&v);
    let err = v
        .edit(
            Operation::DeleteRange {
                object,
                start: 0,
                end: 1_000_000,
            },
            false,
        )
        .unwrap_err();
    assert_eq!(err.code(), "CS-TXN-0004");
    assert_eq!(v.save().unwrap(), source);
}

/// Microsoft's Open XML SDK validator (`tools/ooxml-sdk`) on the saved deck. It skips
/// where `dotnet` is missing, unless `OOXML_SDK_REQUIRED` is set; then it must run.
#[test]
fn the_saved_deck_passes_the_sdk_validator() {
    let mut v = Viewer::open(fixture(DECK)).unwrap();
    let _ = edit_like_a_user(&mut v);
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
    let dir = std::env::temp_dir().join("openpresent-vs05-sdk");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("edited-{}.pptx", std::process::id()));
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
        // Could not run: no usable SDK on this machine.
        Some(2) if !required => eprintln!("SDK validator could not run: skipped\n{report}"),
        _ => panic!("the SDK validator rejected the saved deck:\n{report}"),
    }
}

/// `VS-19`: a toggle reads the run's state as drawn, inherited included. In
/// `text-inheritance.pptx` the body placeholder's runs are italic from the master's
/// `bodyStyle` and say nothing themselves: Ctrl+I turns italic off, as PowerPoint,
/// and the ribbon shows italic pressed before it.
#[test]
fn a_toggle_turns_off_what_the_run_inherits() {
    let mut v = Viewer::open(fixture("text-inheritance.pptx")).unwrap();
    let object = ObjectId {
        part: v.slide_part(0).unwrap(),
        node: v.shapes(0).unwrap()[0].node,
    };
    // "b", the second character, states nothing of its own.
    assert_eq!(
        v.format_state(object, 2).unwrap() & 2,
        2,
        "inherited italic shows pressed"
    );
    assert!(v.toggle(object, 1, 2, Toggle::Italic).unwrap());
    let body = v.body(object).unwrap();
    let run = props_at(body, 1).unwrap();
    assert_eq!(
        run.italic,
        Some(false),
        "italic is turned off, not stated on"
    );
    assert_eq!(v.format_state(object, 2).unwrap() & 2, 0);
    // And on again.
    assert!(v.toggle(object, 1, 2, Toggle::Italic).unwrap());
    assert_eq!(v.format_state(object, 2).unwrap() & 2, 2);
}
