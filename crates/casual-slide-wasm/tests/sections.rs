//! Home > Section (`UX-006`): Add, Rename and Remove Section, each one undo step,
//! saved into `p14:sectionLst`, read back as the model said, a fixed point.

use casual_slide_wasm::Viewer;

const A: &str = "{11111111-2222-3333-4444-555555555555}";
const B: &str = "{AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE}";
const C: &str = "{0F0F0F0F-1234-5678-9ABC-DEF012345678}";

fn listed(v: &Viewer) -> Vec<(String, usize)> {
    v.sections().into_iter().map(|(s, i)| (s.name, i)).collect()
}

#[test]
fn sections_add_rename_remove_save_and_undo() {
    let mut v = Viewer::blank().unwrap();
    let layouts = v.layouts();
    v.new_slide(1, layouts[1].0).unwrap();
    v.new_slide(2, layouts[1].0).unwrap();
    let source = v.save().unwrap();
    let mut v = Viewer::open(source.clone()).unwrap();
    assert!(listed(&v).is_empty());
    assert!(v.add_section(2, "Results", A, B).is_ok());
    assert_eq!(
        listed(&v),
        vec![("Default Section".to_owned(), 0), ("Results".to_owned(), 2)]
    );
    assert!(
        v.add_section(2, "Again", C, B).is_err(),
        "a slide starts one section"
    );
    assert!(v.add_section(1, "Bad id", "not-a-guid", B).is_err());
    v.rename_section(0, "Intro").unwrap();
    let saved = v.save().unwrap();
    let again = Viewer::open(saved.clone()).unwrap();
    assert_eq!(
        listed(&again),
        vec![("Intro".to_owned(), 0), ("Results".to_owned(), 2)]
    );
    assert_eq!(again.save().unwrap(), saved, "not a fixed point");
    let mut w = Viewer::open(saved.clone()).unwrap();
    w.remove_section(1).unwrap();
    let removed = w.save().unwrap();
    assert_eq!(
        listed(&Viewer::open(removed).unwrap()),
        vec![("Intro".to_owned(), 0)]
    );
    w.undo().unwrap();
    assert_eq!(w.save().unwrap(), saved, "remove undone");
    v.undo().unwrap();
    v.undo().unwrap();
    assert!(listed(&v).is_empty());
    assert_eq!(v.save().unwrap(), source, "undone");
}
