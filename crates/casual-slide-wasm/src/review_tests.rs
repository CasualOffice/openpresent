// `UX-011`'s engine tests: comments, notes and the Selection Pane through the bridge's own
// methods, saved and read back, and undone to the file as opened.

use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use serde_json::Value;

use super::*;

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../fixtures/generated/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(path).unwrap()
}

const ME: Writer<'static> = Writer {
    id: "{11111111-2222-3333-4444-555555555555}",
    name: "Ada Lovelace",
};
const WHEN: &str = "2026-11-08T10:00:00.000Z";

fn json(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

fn part_names(bytes: &[u8]) -> Vec<String> {
    PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default())
        .unwrap()
        .part_names()
}

fn first_shape(v: &Viewer) -> u32 {
    v.shapes(0).unwrap().first().unwrap().node
}

#[test]
fn comments_replies_and_resolve_save_as_a_new_part_and_read_back() {
    let input = fixture("area-text.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    assert_eq!(v.comments_json(0).unwrap(), "[]");
    let node = first_shape(&v);
    let thread = "{AAAAAAAA-0000-0000-0000-000000000001}";
    assert!(
        v.add_comment(
            0,
            None,
            (100, 200),
            "On the slide @Grace",
            &ME,
            thread,
            WHEN
        )
        .unwrap()
    );
    let on_shape = "{AAAAAAAA-0000-0000-0000-000000000002}";
    assert!(
        v.add_comment(0, Some(node), (0, 0), "On a shape", &ME, on_shape, WHEN)
            .unwrap()
    );
    let reply = "{AAAAAAAA-0000-0000-0000-000000000003}";
    assert!(
        v.reply_comment(0, thread, "A reply", &ME, reply, WHEN)
            .unwrap()
    );
    assert!(v.resolve_comment(0, thread, true).unwrap());

    let saved = v.save().unwrap();
    let names = part_names(&saved);
    assert!(
        names.iter().any(|n| n.starts_with("ppt/comments/")),
        "{names:?}"
    );
    assert!(
        names.iter().any(|n| n.starts_with("ppt/authors")),
        "{names:?}"
    );

    let back = Viewer::open(saved).unwrap();
    let threads = json(&back.comments_json(0).unwrap());
    let threads = threads.as_array().unwrap();
    assert_eq!(threads.len(), 2);
    assert_eq!(threads[0]["text"], "On the slide @Grace");
    assert_eq!(threads[0]["author"]["name"], "Ada Lovelace");
    assert_eq!(threads[0]["author"]["initials"], "AL");
    assert_eq!(threads[0]["created"], WHEN);
    assert_eq!(threads[0]["resolved"], true);
    assert_eq!(threads[0]["node"], -1);
    assert_eq!(threads[0]["replies"][0]["text"], "A reply");
    assert_eq!(threads[1]["node"], i64::from(node));
    assert_eq!(back.comment_counts().first(), Some(&2));

    // Undone to the deck as opened, the save is the input, byte for byte.
    while v.can_undo_redo().0 {
        v.undo().unwrap();
    }
    assert_eq!(v.save().unwrap(), input);
}

#[test]
fn a_comment_deleted_from_a_saved_part_patches_that_part() {
    let mut v = Viewer::open(fixture("area-text.pptx")).unwrap();
    let a = "{AAAAAAAA-0000-0000-0000-00000000000A}";
    let b = "{AAAAAAAA-0000-0000-0000-00000000000B}";
    v.add_comment(0, None, (0, 0), "first", &ME, a, WHEN)
        .unwrap();
    v.add_comment(0, None, (0, 0), "second", &ME, b, WHEN)
        .unwrap();
    let saved = v.save().unwrap();
    let mut again = Viewer::open(saved.clone()).unwrap();
    assert!(again.delete_comment(0, a, None).unwrap());
    let twice = again.save().unwrap();
    assert_eq!(
        part_names(&twice),
        part_names(&saved),
        "no part added or removed"
    );
    let threads = json(&Viewer::open(twice).unwrap().comments_json(0).unwrap());
    assert_eq!(threads.as_array().unwrap().len(), 1);
    assert_eq!(threads[0]["text"], "second");
}

#[test]
fn what_a_comment_cannot_take_is_refused_and_changes_nothing() {
    let mut v = Viewer::open(fixture("area-text.pptx")).unwrap();
    let good = "{AAAAAAAA-0000-0000-0000-000000000001}";
    for (text, id, when) in [
        ("   ", good, WHEN),
        ("text", "<script>", WHEN),
        ("text", good, "yesterday"),
    ] {
        assert!(v.add_comment(0, None, (0, 0), text, &ME, id, when).is_err());
    }
    assert!(!v.can_undo_redo().0);
    assert!(
        v.reply_comment(0, good, "no thread", &ME, good, WHEN)
            .is_err()
    );
}

#[test]
fn notes_typed_on_a_slide_without_any_are_saved_to_a_new_notes_slide() {
    let input = fixture("area-text.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    assert_eq!(v.notes_text(0).unwrap(), "");
    assert_eq!(v.notes_body(0, false).unwrap(), None);
    let body = v.notes_body(0, true).unwrap().expect("a notes body");
    v.edit(
        Operation::InsertText {
            object: body,
            at: 0,
            text: "Say hello".to_owned(),
        },
        true,
    )
    .unwrap();
    v.toggle(body, 4, 9, crate::Toggle::Bold).unwrap();
    assert_eq!(v.notes_text(0).unwrap(), "Say hello");
    let runs = json(&v.body_json(body).unwrap());
    assert_eq!(runs[0][1], serde_json::json!(["hello", 1]));

    let saved = v.save().unwrap();
    assert!(
        part_names(&saved)
            .iter()
            .any(|n| n.starts_with("ppt/notesSlides/"))
    );
    let mut back = Viewer::open(saved).unwrap();
    assert_eq!(back.notes_text(0).unwrap(), "Say hello");
    let body = back.notes_body(0, false).unwrap().unwrap();
    assert_eq!(json(&back.body_json(body).unwrap())[0][1][1], 1);

    // Undone, the notes slide goes and the save is the input.
    while v.can_undo_redo().0 {
        v.undo().unwrap();
    }
    assert_eq!(v.notes_body(0, false).unwrap(), None);
    assert_eq!(v.save().unwrap(), input);
}

#[test]
fn a_new_notes_slide_left_empty_is_not_written() {
    let input = fixture("area-text.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    v.notes_body(0, true).unwrap().unwrap();
    assert_eq!(v.save().unwrap(), input);
}

#[test]
fn rename_hide_and_reorder_save_and_read_back() {
    let input = fixture("area-shape-tree.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    let before = json(&v.selection_list(0).unwrap());
    let list = before.as_array().unwrap();
    assert!(list.len() >= 2, "two shapes to reorder");
    let front = u32::try_from(list[0]["node"].as_u64().unwrap()).unwrap();
    let back_node = u32::try_from(list[list.len() - 1]["node"].as_u64().unwrap()).unwrap();
    assert!(
        v.set_shape_shown(0, front, Some("Front shape"), None)
            .unwrap()
    );
    assert!(v.set_shape_shown(0, back_node, None, Some(true)).unwrap());
    // The front shape to the back.
    assert!(v.move_shape_z(0, front, 0).unwrap());
    let edited = json(&v.selection_list(0).unwrap());

    let saved = v.save().unwrap();
    let reread = Viewer::open(saved).unwrap();
    // Nodes are numbered again when a file is read; names, visibility and order are kept.
    let shown = |v: &Value| -> Vec<(Value, Value)> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|s| (s["name"].clone(), s["hidden"].clone()))
            .collect()
    };
    assert_eq!(
        shown(&json(&reread.selection_list(0).unwrap())),
        shown(&edited)
    );
    let last = &edited.as_array().unwrap()[list.len() - 1];
    assert_eq!(last["name"], "Front shape");

    while v.can_undo_redo().0 {
        v.undo().unwrap();
    }
    assert_eq!(json(&v.selection_list(0).unwrap()), before);
    assert_eq!(v.save().unwrap(), input);
}
