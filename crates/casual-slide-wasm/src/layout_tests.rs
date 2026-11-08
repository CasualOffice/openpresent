// `PAR-07`'s engine tests: Home > Layout on an existing slide, through the bridge's own
// methods, saved and read back, and undone to the file as opened.

use super::*;
use casual_slide_model::shape::PlaceholderType;

type Seen = (Option<PlaceholderType>, Option<u32>, String, bool);

/// Slide 0's placeholders: each one's type, `idx`, text and whether it has its own
/// position.
fn placeholders(v: &Viewer) -> Vec<Seen> {
    let ordinal = v.slide_ordinal(0).unwrap();
    let root = v.deck.parts().get(ordinal).unwrap().shape_tree().unwrap();
    root.children
        .iter()
        .filter_map(|n| {
            let p = n.placeholder()?;
            let text = n
                .text_body
                .as_deref()
                .map(|b| {
                    b.paragraphs
                        .iter()
                        .flat_map(|p| &p.items)
                        .filter_map(|i| match i {
                            casual_slide_model::text::TextItem::Run(r) => r.text.clone(),
                            _ => None,
                        })
                        .collect::<String>()
                })
                .unwrap_or_default();
            Some((p.placeholder_type, p.idx, text, n.transform.is_some()))
        })
        .collect()
}

fn layout_named(v: &Viewer, name: &str) -> u32 {
    v.layouts()
        .into_iter()
        .find(|(_, n)| n == name)
        .unwrap_or_else(|| panic!("no layout {name}: {:?}", v.layouts()))
        .0
}

fn layout_name(v: &Viewer) -> String {
    let l = v.layout_of(0).unwrap();
    v.layouts().into_iter().find(|(o, _)| *o == l).unwrap().1
}

#[test]
fn change_layout_remaps_placeholders_saves_reads_back_and_undoes() {
    let mut v = Viewer::blank().unwrap();
    let input = v.save().unwrap();
    let ordinal = v.slide_ordinal(0).unwrap();
    let before = placeholders(&v);
    let nodes: Vec<u32> = v
        .deck
        .parts()
        .get(ordinal)
        .unwrap()
        .shape_tree()
        .unwrap()
        .children
        .iter()
        .map(|n| n.id.node)
        .collect();
    assert_eq!(before.len(), 2, "{before:?}");
    let id = |node| ObjectId {
        part: ordinal,
        node,
    };
    v.replace_text(id(nodes[0]), "Quarterly review").unwrap();
    v.replace_text(id(nodes[1]), "Prepared by Ada").unwrap();
    assert_eq!(layout_name(&v), "Title Slide");

    let target = layout_named(&v, "Title and Content");
    assert!(v.change_layout(0, target).unwrap());
    assert_eq!(layout_name(&v), "Title and Content");
    let after = placeholders(&v);
    // The title takes the new layout's title, and its place; the subtitle, which the
    // new layout has no match for, keeps its text where it was drawn; the content
    // placeholder is added, empty.
    assert_eq!(after[0].0, Some(PlaceholderType::Title), "{after:?}");
    assert_eq!(after[0].2, "Quarterly review");
    assert!(!after[0].3);
    assert_eq!(after[1].0, Some(PlaceholderType::SubTitle));
    assert_eq!(after[1].2, "Prepared by Ada");
    assert!(after[1].3);
    assert_eq!(after[2].1, Some(1));
    assert_eq!(after[2].2, "");
    assert!(!v.change_layout(0, target).unwrap());

    let reread = Viewer::open(v.save().unwrap()).unwrap();
    assert_eq!(layout_name(&reread), "Title and Content");
    assert_eq!(placeholders(&reread), after);

    v.undo().unwrap();
    assert_eq!(layout_name(&v), "Title Slide");
    v.undo().unwrap();
    v.undo().unwrap();
    assert_eq!(v.save().unwrap(), input);
}

#[test]
fn a_part_that_is_not_a_layout_is_refused() {
    let mut v = Viewer::blank().unwrap();
    let slide = v.slide_ordinal(0).unwrap();
    assert!(v.change_layout(0, slide).is_err());
}
