//! Outline View (`UX-006`): each slide's title and body found, their text replaced
//! from the outline as one undo step, saved and read back.

use casual_slide_model::shape::ObjectId;
use casual_slide_wasm::Viewer;

fn text(v: &Viewer, part: u32, node: u32) -> String {
    let o = ObjectId { part, node };
    let n = v.text_length(o).unwrap();
    v.text_slice(o, 0, n).unwrap()
}

#[test]
fn the_outline_finds_title_and_body_and_an_edit_saves_and_undoes() {
    let mut v = Viewer::blank().unwrap();
    let layouts = v.layouts();
    let content = layouts
        .iter()
        .find(|(_, n)| n == "Title and Content")
        .map_or(layouts[1].0, |l| l.0);
    v.new_slide(1, content).unwrap();
    let source = v.save().unwrap();
    let mut v = Viewer::open(source.clone()).unwrap();
    for i in 0..2 {
        let o = v.outline(i).unwrap();
        assert!(o.title.is_some(), "slide {i} has a title");
        assert!(o.body.is_some(), "slide {i} has a body");
    }
    let o = v.outline(1).unwrap();
    let (title, body) = (o.title.unwrap(), o.body.unwrap());
    v.replace_text(
        ObjectId {
            part: o.part,
            node: title,
        },
        "Plan",
    )
    .unwrap();
    v.replace_text(
        ObjectId {
            part: o.part,
            node: body,
        },
        "One\nTwo",
    )
    .unwrap();
    assert_eq!(text(&v, o.part, body), "One\nTwo");
    let saved = v.save().unwrap();
    let again = Viewer::open(saved).unwrap();
    let a = again.outline(1).unwrap();
    assert_eq!(text(&again, a.part, a.title.unwrap()), "Plan");
    assert_eq!(text(&again, a.part, a.body.unwrap()), "One\nTwo");
    v.undo().unwrap();
    assert_eq!(text(&v, o.part, body), "");
    v.undo().unwrap();
    assert_eq!(v.save().unwrap(), source, "undone");
}
