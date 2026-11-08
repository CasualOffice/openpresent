// `PAR-03`'s engine tests: Group and Ungroup through the bridge's own methods, drawn
// where they were, saved and read back, and undone to the file as opened.

use super::*;
use crate::ShapeBox;

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../fixtures/generated/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(path).unwrap()
}

/// The node named `name` anywhere on slide 0.
fn named(v: &Viewer, name: &str) -> u32 {
    let ordinal = v.slide_ordinal(0).unwrap();
    v.deck
        .parts()
        .get(ordinal)
        .unwrap()
        .shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|n| n.name.as_deref()) == Some(name))
        .unwrap_or_else(|| panic!("no shape named {name}"))
        .id
        .node
}

/// The names of the children of the node named `group`, or of the slide's tree.
fn children(v: &Viewer, group: Option<&str>) -> Vec<String> {
    let ordinal = v.slide_ordinal(0).unwrap();
    let part = v.deck.parts().get(ordinal).unwrap();
    let root = part.shape_tree().unwrap();
    let at = match group {
        Some(g) => {
            let n = named(v, g);
            root.descendants().find(|x| x.id.node == n).unwrap()
        }
        None => root,
    };
    at.children
        .iter()
        .map(|c| {
            c.non_visual
                .as_ref()
                .and_then(|n| n.name.clone())
                .unwrap_or_default()
        })
        .collect()
}

/// Every shape's box on slide 0 as drawn, by name, groups opened.
fn drawn(v: &Viewer) -> Vec<(String, i64, i64, i64, i64)> {
    let ordinal = v.slide_ordinal(0).unwrap();
    let part = v.deck.parts().get(ordinal).unwrap();
    let root = part.shape_tree().unwrap();
    let name = |node: u32| {
        root.descendants()
            .find(|x| x.id.node == node)
            .and_then(|x| x.non_visual.as_ref()?.name.clone())
            .unwrap_or_default()
    };
    let mut out = Vec::new();
    let mut todo: Vec<ShapeBox> = v.shapes(0).unwrap();
    while let Some(b) = todo.pop() {
        if b.flags & ShapeBox::GROUP != 0 {
            todo.extend(v.group_shapes(0, b.node).unwrap());
        } else {
            out.push((name(b.node), b.x, b.y, b.cx, b.cy));
        }
    }
    out.sort();
    out
}

fn near(a: &[(String, i64, i64, i64, i64)], b: &[(String, i64, i64, i64, i64)]) {
    assert_eq!(a.len(), b.len(), "{a:?} against {b:?}");
    for (x, y) in a.iter().zip(b) {
        assert_eq!(x.0, y.0);
        for (p, q) in [(x.1, y.1), (x.2, y.2), (x.3, y.3), (x.4, y.4)] {
            assert!((p - q).abs() <= 2, "{x:?} moved to {y:?}");
        }
    }
}

#[test]
fn group_puts_shapes_in_one_group_that_saves_reads_back_and_undoes() {
    let input = fixture("area-shape-tree.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    let before = drawn(&v);
    let (a, b) = (named(&v, "Node 7"), named(&v, "Node 8"));
    let g = v.group(0, &[b, a]).unwrap();
    let ordinal = v.slide_ordinal(0).unwrap();
    let grp_name = v
        .deck
        .parts()
        .get(ordinal)
        .unwrap()
        .node(ObjectId {
            part: ordinal,
            node: g,
        })
        .unwrap()
        .non_visual
        .as_ref()
        .unwrap()
        .name
        .clone()
        .unwrap();
    // At the place of the frontmost, its children back to front; nothing moved.
    assert_eq!(
        children(&v, None),
        [
            "Title 1",
            "Outer group",
            &grp_name,
            "Connector 9",
            "Hidden 10"
        ]
    );
    assert_eq!(children(&v, Some(&grp_name)), ["Node 7", "Node 8"]);
    near(&drawn(&v), &before);

    let saved = v.save().unwrap();
    let reread = Viewer::open(saved).unwrap();
    assert_eq!(children(&reread, None), children(&v, None));
    assert_eq!(children(&reread, Some(&grp_name)), ["Node 7", "Node 8"]);
    near(&drawn(&reread), &before);

    v.undo().unwrap();
    assert_eq!(v.save().unwrap(), input);
}

#[test]
fn ungroup_lifts_children_into_the_parent_where_they_were_drawn() {
    let input = fixture("area-shape-tree.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    let before = drawn(&v);
    // The outer group is locked with `noUngrp`, as PowerPoint keeps it whole.
    let outer = named(&v, "Outer group");
    assert!(v.ungroup(0, outer).is_err());
    let inner = named(&v, "Inner group");
    let out = v.ungroup(0, inner).unwrap();
    assert_eq!(out, [named(&v, "Child 6")]);
    assert_eq!(children(&v, Some("Outer group")), ["Child 5", "Child 6"]);
    near(&drawn(&v), &before);

    let saved = v.save().unwrap();
    let reread = Viewer::open(saved).unwrap();
    assert_eq!(
        children(&reread, Some("Outer group")),
        ["Child 5", "Child 6"]
    );
    near(&drawn(&reread), &before);

    v.undo().unwrap();
    assert_eq!(v.save().unwrap(), input);
}

#[test]
fn group_then_ungroup_at_the_top_round_trips() {
    let input = fixture("area-shape-tree.pptx");
    let mut v = Viewer::open(input).unwrap();
    let before = drawn(&v);
    let (a, b) = (named(&v, "Node 7"), named(&v, "Node 8"));
    let g = v.group(0, &[a, b]).unwrap();
    // Moved as one: the group's box shifts both children.
    let gb = v
        .shapes(0)
        .unwrap()
        .into_iter()
        .find(|s| s.node == g)
        .unwrap();
    let op = v
        .transform_op(0, g, (gb.x + 914_400, gb.y, gb.cx, gb.cy), 0)
        .unwrap();
    v.history.apply(v.deck.parts_mut(), op).unwrap();
    let moved = drawn(&v);
    let x = |d: &[(String, i64, i64, i64, i64)], n: &str| d.iter().find(|e| e.0 == n).unwrap().1;
    assert_eq!(x(&moved, "Node 7"), x(&before, "Node 7") + 914_400);
    assert_eq!(x(&moved, "Node 8"), x(&before, "Node 8") + 914_400);
    v.ungroup(0, g).unwrap();
    near(&drawn(&v), &moved);
    let reread = Viewer::open(v.save().unwrap()).unwrap();
    near(&drawn(&reread), &moved);
    assert!(children(&reread, None).contains(&"Node 7".to_owned()));
}

#[test]
fn placeholders_and_single_shapes_are_not_grouped() {
    let input = fixture("area-shape-tree.pptx");
    let mut v = Viewer::open(input).unwrap();
    let (t, a) = (named(&v, "Title 1"), named(&v, "Node 7"));
    assert!(v.group(0, &[t, a]).is_err());
    assert!(v.group(0, &[a]).is_err());
}
