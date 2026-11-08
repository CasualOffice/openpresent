// `UX-003`'s engine tests: the Home tab's Font and Paragraph groups, and the
// clipboard and multiple-selection edits, each through the bridge's own methods,
// saved and read back.

use casual_slide_model::shape::NodeKind;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

use super::*;
use crate::format::RunSetting;

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../fixtures/generated/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(path).unwrap()
}

/// The first text box on slide 0 with more than three characters, and its length.
fn text_box(v: &Viewer) -> (ObjectId, usize) {
    let text = v.deck.slide_text(0, None).unwrap();
    text.boxes
        .iter()
        .find_map(|b| {
            let n = v.text_length(b.object).ok()?;
            (n > 3).then_some((b.object, n))
        })
        .expect("a text box with text on slide 1")
}

fn changed_parts(a: &[u8], b: &[u8]) -> Vec<String> {
    let mut pa = PresentationPackage::open(a.to_vec(), OoxmlLimits::default()).unwrap();
    let mut pb = PresentationPackage::open(b.to_vec(), OoxmlLimits::default()).unwrap();
    let names = pa.part_names();
    assert_eq!(names, pb.part_names(), "no part added or removed");
    names
        .into_iter()
        .filter(|n| pa.read_part(n).unwrap() != pb.read_part(n).unwrap())
        .collect()
}

fn part_xml(bytes: &[u8], name: &str) -> String {
    let mut p = PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap();
    String::from_utf8(p.read_part(name).unwrap()).unwrap()
}

#[test]
fn font_size_colour_and_highlight_save_and_read_back_and_undo_to_the_input() {
    let input = fixture("area-text.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    let (object, len) = text_box(&v);
    let range = (0, len);
    assert!(
        v.set_run(object, range, RunSetting::Font, "Georgia")
            .unwrap()
    );
    assert!(v.set_run(object, range, RunSetting::Size, "28").unwrap());
    assert!(
        v.set_run(object, range, RunSetting::Color, "srgb C00000")
            .unwrap()
    );
    assert!(
        v.set_run(object, range, RunSetting::Highlight, "srgb FFFF00")
            .unwrap()
    );
    let state = v.run_state(object, 2).unwrap();
    assert_eq!(state[..2], ["Georgia".to_owned(), "28".to_owned()]);
    let saved = v.save().unwrap();
    let parts = changed_parts(&input, &saved);
    assert!(
        parts.iter().all(|p| p.starts_with("ppt/slides/slide")),
        "{parts:?}"
    );
    let xml = part_xml(&saved, &parts[0]);
    assert!(xml.contains("sz=\"2800\""), "size written");
    assert!(
        xml.contains("<a:latin typeface=\"Georgia\"/>"),
        "font written"
    );
    assert!(
        xml.contains("<a:srgbClr val=\"C00000\"/>"),
        "colour written"
    );
    assert!(
        xml.contains("<a:highlight><a:srgbClr val=\"FFFF00\"/></a:highlight>"),
        "highlight written"
    );
    let again = Viewer::open(saved).unwrap();
    assert_eq!(again.run_state(object, 2).unwrap()[..2], state[..2]);
    for _ in 0..4 {
        v.undo().unwrap();
    }
    assert!(v.save().unwrap() == input, "undone, the save is the input");
}

#[test]
fn a_bad_size_or_colour_is_refused_and_changes_nothing() {
    let mut v = Viewer::open(fixture("area-text.pptx")).unwrap();
    let (object, len) = text_box(&v);
    for (what, value) in [
        (RunSetting::Size, "0"),
        (RunSetting::Size, "4001"),
        (RunSetting::Size, "x"),
        (RunSetting::Color, "red"),
        (RunSetting::Highlight, "srgb 12345"),
    ] {
        assert_eq!(
            v.set_run(object, (0, len), what, value).unwrap_err().code(),
            "CS-WSM-0006"
        );
    }
    assert_eq!(v.can_undo_redo(), (false, false));
    // An empty range is no edit.
    assert!(!v.set_run(object, (2, 2), RunSetting::Size, "20").unwrap());
}

#[test]
fn increase_and_decrease_step_the_drawn_size_in_one_undo_step() {
    let mut v = Viewer::open(fixture("area-text.pptx")).unwrap();
    let (object, len) = text_box(&v);
    let size = |v: &Viewer| v.run_state(object, 2).unwrap()[1].clone();
    let before = size(&v);
    let points: f64 = before.parse().unwrap();
    let hundredths = (points * 100.0).round() as i32;
    assert!(v.grow_text(object, (0, len), true).unwrap());
    let up = step_size(hundredths, true);
    assert_eq!(size(&v), format!("{}", f64::from(up) / 100.0));
    assert!(v.grow_text(object, (0, len), false).unwrap());
    // One step each: two undos give the start back.
    v.undo().unwrap();
    v.undo().unwrap();
    assert_eq!(size(&v), before);
    assert_eq!(v.can_undo_redo(), (false, true));
}

#[test]
fn alignment_and_bullets_save_read_back_and_toggle_off() {
    let input = fixture("area-text.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    let (object, len) = text_box(&v);
    assert!(v.set_paragraphs(object, (0, len), 0, "ctr").unwrap());
    assert!(v.set_paragraphs(object, (1, 1), 1, "toggle").unwrap());
    let state = v.run_state(object, 1).unwrap();
    assert_eq!(state[2..], ["ctr".to_owned(), "1".to_owned()]);
    let saved = v.save().unwrap();
    let again = Viewer::open(saved.clone()).unwrap();
    assert_eq!(again.run_state(object, 1).unwrap()[2..], state[2..]);
    let parts = changed_parts(&input, &saved);
    let xml = part_xml(&saved, &parts[0]);
    assert!(
        xml.contains("<a:buChar char=\"\u{2022}\"/>"),
        "bullet written"
    );
    assert!(xml.contains("algn=\"ctr\""), "alignment written");
    assert!(v.set_paragraphs(object, (1, 1), 1, "toggle").unwrap());
    assert_eq!(v.run_state(object, 1).unwrap()[3], "0");
    assert_eq!(
        v.set_paragraphs(object, (0, 1), 0, "middle")
            .unwrap_err()
            .code(),
        "CS-WSM-0006"
    );
}

#[test]
fn a_slice_of_text_is_what_copy_puts_on_the_clipboard() {
    let mut v = Viewer::open(fixture("area-text.pptx")).unwrap();
    let (object, len) = text_box(&v);
    v.edit(
        Operation::InsertText {
            object,
            at: 0,
            text: "ab\ncd".to_owned(),
        },
        false,
    )
    .unwrap();
    assert_eq!(v.text_slice(object, 0, 5).unwrap(), "ab\ncd");
    assert_eq!(v.text_slice(object, 1, 4).unwrap(), "b\nc");
    assert_eq!(v.text_slice(object, 4, 1).unwrap(), "b\nc");
    assert!(v.text_slice(object, 0, len + 100).is_err());
}

fn line_node(v: &Viewer) -> ShapeBox {
    v.shapes(0).unwrap()[1]
}

#[test]
fn a_shape_pasted_onto_its_own_slide_lands_beside_it_and_saves() {
    let mut v = Viewer::open(fixture("preset-shapes.pptx")).unwrap();
    let count = v.shapes(0).unwrap().len();
    let l = line_node(&v);
    let clip = v.copy_shapes(0, &[l.node]).unwrap();
    assert_eq!(clipboard::clip_spids(&clip).len(), 1);
    let new = v.paste_shapes(0, &clip).unwrap();
    assert_eq!(new.len(), 1);
    let shapes = v.shapes(0).unwrap();
    assert_eq!(shapes.len(), count + 1);
    let pasted = shapes.last().unwrap();
    assert_eq!(pasted.node, new[0]);
    assert_eq!(
        (pasted.x, pasted.y, pasted.cx, pasted.cy),
        (l.x + PASTE_OFFSET, l.y + PASTE_OFFSET, l.cx, l.cy)
    );
    let saved = v.save().unwrap();
    let again = Viewer::open(saved).unwrap();
    let back = again.shapes(0).unwrap();
    assert_eq!(back.len(), count + 1);
    assert_eq!(back.last().unwrap().x, l.x + PASTE_OFFSET);
    // One undo takes the paste back.
    v.undo().unwrap();
    assert_eq!(v.shapes(0).unwrap().len(), count);
}

#[test]
fn shapes_paste_into_a_new_deck_where_they_were_copied_and_save() {
    let mut from = Viewer::open(fixture("preset-shapes.pptx")).unwrap();
    let nodes: Vec<u32> = from.shapes(0).unwrap().iter().map(|s| s.node).collect();
    let clip = from.copy_shapes(0, &nodes).unwrap();
    let mut to = Viewer::blank().unwrap();
    let before = to.shapes(0).unwrap().len();
    let new = to.paste_shapes(0, &clip).unwrap();
    assert_eq!(new.len(), nodes.len());
    let src = from.shapes(0).unwrap();
    let got = to.shapes(0).unwrap();
    assert_eq!(got.len(), before + nodes.len());
    // Another deck: where it was, not moved.
    for (a, b) in src.iter().zip(&got[before..]) {
        assert_eq!((a.x, a.y, a.cx, a.cy), (b.x, b.y, b.cx, b.cy));
    }
    let saved = to.save().unwrap();
    assert_eq!(
        Viewer::open(saved).unwrap().shapes(0).unwrap().len(),
        before + nodes.len()
    );
}

#[test]
fn a_picture_copies_with_its_image_into_another_deck() {
    let mut from = Viewer::open(fixture("area-pic.pptx")).unwrap();
    let ordinal = from.slide_ordinal(0).unwrap();
    let pic = from
        .deck
        .parts()
        .get(ordinal)
        .unwrap()
        .shape_tree()
        .unwrap()
        .children
        .iter()
        .find(|n| n.kind == NodeKind::Picture)
        .map(|n| n.id.node)
        .expect("a picture on slide 1");
    let clip = from.copy_shapes(0, &[pic]).unwrap();
    let mut to = Viewer::blank().unwrap();
    let before = to.shapes(0).unwrap().len();
    to.paste_shapes(0, &clip).unwrap();
    let saved = to.save().unwrap();
    let package = PresentationPackage::open(saved.clone(), OoxmlLimits::default()).unwrap();
    assert!(
        package
            .part_names()
            .iter()
            .any(|n| n.starts_with("ppt/media/")),
        "the image went with it"
    );
    let again = Viewer::open(saved).unwrap();
    assert_eq!(again.shapes(0).unwrap().len(), before + 1);
}

#[test]
fn duplicate_moves_deletes_and_aligns_several_shapes_in_one_undo_step_each() {
    let input = fixture("preset-shapes.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    let shapes = v.shapes(0).unwrap();
    let nodes: Vec<u32> = shapes.iter().map(|s| s.node).collect();
    let count = shapes.len();
    let copies = v.duplicate_shapes(0, &nodes).unwrap();
    assert_eq!(v.shapes(0).unwrap().len(), count * 2);
    // Move every copy to x = 0, as Align Left does.
    let boxes: Vec<[i64; 6]> = v
        .shapes(0)
        .unwrap()
        .iter()
        .filter(|s| copies.contains(&s.node))
        .map(|s| [i64::from(s.node), 0, s.y, s.cx, s.cy, i64::from(s.rotation)])
        .collect();
    v.set_transforms(0, &boxes).unwrap();
    assert!(
        v.shapes(0)
            .unwrap()
            .iter()
            .filter(|s| copies.contains(&s.node))
            .all(|s| s.x == 0)
    );
    v.delete_shapes(0, &copies).unwrap();
    assert_eq!(v.shapes(0).unwrap().len(), count);
    // Three steps: delete, move, duplicate.
    v.undo().unwrap();
    assert_eq!(v.shapes(0).unwrap().len(), count * 2);
    v.undo().unwrap();
    v.undo().unwrap();
    assert_eq!(v.can_undo_redo(), (false, true));
    assert!(v.save().unwrap() == input);
}

#[test]
fn a_move_or_delete_of_several_with_one_missing_shape_changes_nothing() {
    let mut v = Viewer::open(fixture("preset-shapes.pptx")).unwrap();
    let count = v.shapes(0).unwrap().len();
    let s = v.shapes(0).unwrap()[0];
    let boxes = [
        [i64::from(s.node), 0, 0, s.cx, s.cy, 0],
        [9999, 0, 0, 10, 10, 0],
    ];
    assert!(v.set_transforms(0, &boxes).is_err());
    assert_eq!(v.shapes(0).unwrap()[0].x, s.x);
    assert!(v.delete_shapes(0, &[s.node, 9999]).is_err());
    assert_eq!(v.shapes(0).unwrap().len(), count);
    assert_eq!(v.can_undo_redo(), (false, false));
}

#[test]
fn a_shape_with_a_link_does_not_paste_onto_another_slide() {
    let mut v = Viewer::open(fixture("preset-shapes.pptx")).unwrap();
    let l = line_node(&v);
    let ordinal = v.slide_ordinal(0).unwrap();
    let id = ObjectId {
        part: ordinal,
        node: l.node,
    };
    v.deck
        .parts_mut()
        .get_mut(ordinal)
        .unwrap()
        .node_mut(id)
        .unwrap()
        .non_visual
        .get_or_insert_with(Default::default)
        .hyperlink_click = Some(Default::default());
    let clip = v.copy_shapes(0, &[l.node]).unwrap();
    let mut other = Viewer::blank().unwrap();
    assert_eq!(
        other.paste_shapes(0, &clip).unwrap_err().code(),
        "CS-WSM-0010"
    );
    assert_eq!(other.can_undo_redo(), (false, false));
    // Onto its own slide its relationships still hold.
    assert_eq!(v.paste_shapes(0, &clip).unwrap().len(), 1);
}

/// What the Home tab shows for a shape's first character: font, size, alignment,
/// bulleted.
fn look(v: &Viewer, node: u32) -> Vec<String> {
    let part = v.slide_part(0).unwrap();
    v.run_state(ObjectId { part, node }, 0).unwrap()
}

#[test]
fn a_duplicated_or_pasted_placeholder_keeps_its_inherited_look_saved_and_read_back() {
    let mut v = Viewer::blank().unwrap();
    let shapes = v.shapes(0).unwrap();
    let (title, subtitle) = (shapes[0], shapes[1]);
    let title_look = look(&v, title.node);
    let subtitle_look = look(&v, subtitle.node);
    // The layout's centred title: not the 18 pt default a bare text box draws in.
    assert_ne!(title_look[1], "18");
    let new = v.duplicate_shapes(0, &[title.node, subtitle.node]).unwrap();
    assert_eq!(look(&v, new[0]), title_look);
    assert_eq!(look(&v, new[1]), subtitle_look);
    let got = v.shapes(0).unwrap();
    let copy = got.iter().find(|s| s.node == new[0]).unwrap();
    assert_eq!(
        (copy.x, copy.y, copy.cx, copy.cy),
        (
            title.x + PASTE_OFFSET,
            title.y + PASTE_OFFSET,
            title.cx,
            title.cy
        )
    );
    // Saved and opened again, the copies still draw as the originals.
    let again = Viewer::open(v.save().unwrap()).unwrap();
    let back = again.shapes(0).unwrap();
    assert_eq!(back.len(), 4);
    assert_eq!(look(&again, back[2].node), title_look);
    assert_eq!(look(&again, back[3].node), subtitle_look);
    // Pasted into another deck, too.
    let clip = v.copy_shapes(0, &[title.node]).unwrap();
    let mut other = Viewer::open(fixture("preset-shapes.pptx")).unwrap();
    let pasted = other.paste_shapes(0, &clip).unwrap();
    assert_eq!(look(&other, pasted[0])[1], title_look[1]);
    // One undo takes the duplicate back.
    v.undo().unwrap();
    assert_eq!(v.shapes(0).unwrap().len(), 2);
}
