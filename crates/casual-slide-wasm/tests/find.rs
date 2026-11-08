//! Find, Replace and Reset Slide (`UX-006`): matches across slides, Replace All as
//! one undo step, a reset slide's placeholders back on their layout, each saved and
//! read back.

use casual_slide_model::shape::ObjectId;
use casual_slide_wasm::Viewer;

fn text(v: &Viewer, o: ObjectId) -> String {
    let n = v.text_length(o).unwrap();
    v.text_slice(o, 0, n).unwrap()
}

/// A blank deck of two slides, each title "Plan the plan".
fn deck() -> (Vec<u8>, Vec<ObjectId>) {
    let mut v = Viewer::blank().unwrap();
    let layouts = v.layouts();
    v.new_slide(1, layouts[1].0).unwrap();
    let mut titles = Vec::new();
    for i in 0..2 {
        let o = v.outline(i).unwrap();
        let t = ObjectId {
            part: o.part,
            node: o.title.unwrap(),
        };
        v.replace_text(t, "Plan the plan").unwrap();
        titles.push(t);
    }
    (v.save().unwrap(), titles)
}

#[test]
fn find_lists_matches_and_replace_all_is_one_undo_step() {
    let (source, _) = deck();
    let mut v = Viewer::open(source.clone()).unwrap();
    let found = v.find_text("plan", false).unwrap();
    assert_eq!(found.len(), 4);
    assert_eq!((found[1].start, found[1].end), (9, 13));
    assert_eq!(v.find_text("plan", true).unwrap().len(), 2);
    assert_eq!(v.replace_text_all("Plan", "Map", true).unwrap(), 2);
    let saved = v.save().unwrap();
    let again = Viewer::open(saved).unwrap();
    let titles: Vec<String> = (0..2)
        .map(|i| {
            let o = again.outline(i).unwrap();
            text(
                &again,
                ObjectId {
                    part: o.part,
                    node: o.title.unwrap(),
                },
            )
        })
        .collect();
    assert_eq!(titles, vec!["Map the plan", "Map the plan"]);
    v.undo().unwrap();
    assert_eq!(v.save().unwrap(), source, "one undo step");
}

#[test]
fn reset_slide_takes_a_moved_placeholder_back_to_its_layout() {
    let (source, titles) = deck();
    let mut v = Viewer::open(source.clone()).unwrap();
    let node = titles[0].node;
    let at = |v: &Viewer| {
        v.shapes(0)
            .unwrap()
            .into_iter()
            .find(|b| b.node == node)
            .map(|b| (b.x, b.y, b.cx, b.cy))
            .unwrap()
    };
    let home = at(&v);
    v.set_transform(0, node, (10, 20, 3_000_000, 900_000), 0)
        .unwrap();
    assert_ne!(at(&v), home);
    assert!(v.reset_slide(0).unwrap());
    assert_eq!(at(&v), home);
    let saved = v.save().unwrap();
    let again = Viewer::open(saved).unwrap();
    assert_eq!(at(&again), home);
    assert!(!v.reset_slide(0).unwrap(), "nothing left to reset");
}
