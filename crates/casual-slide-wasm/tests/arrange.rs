//! Arrange (`UX-006`): z-order and flips saved, read back as the model said, a fixed
//! point, and undone to the opened bytes.

use casual_slide_wasm::{Insert, Order, Viewer};

const BOX: [i64; 4] = [914_400, 914_400, 2_743_200, 914_400];

fn order(v: &Viewer) -> Vec<u32> {
    v.shapes(0).unwrap().iter().map(|b| b.node).collect()
}

#[test]
fn z_order_and_flips_save_read_back_and_undo() {
    let mut v = Viewer::blank().unwrap();
    let a = v.insert_shape(0, Insert::Rectangle, BOX).unwrap();
    let b = v.insert_shape(0, Insert::Oval, BOX).unwrap();
    let source = v.save().unwrap();
    let mut v = Viewer::open(source.clone()).unwrap();
    let before = order(&v);
    let (ia, ib) = (
        before.iter().position(|&n| n == a).unwrap(),
        before.iter().position(|&n| n == b).unwrap(),
    );
    assert!(ia < ib);
    assert!(v.reorder(0, a, Order::Front).unwrap());
    assert!(!v.reorder(0, a, Order::Front).unwrap(), "already in front");
    v.flip(0, b, true).unwrap();
    v.flip(0, b, false).unwrap();
    let saved = v.save().unwrap();
    let again = Viewer::open(saved.clone()).unwrap();
    // Nodes are numbered again on reading, in file order: name the shapes instead.
    let names = |w: &Viewer| -> Vec<String> {
        order(w)
            .into_iter()
            .map(|n| w.shape_name(0, n).unwrap())
            .collect()
    };
    let (name_a, name_b) = (v.shape_name(0, a).unwrap(), v.shape_name(0, b).unwrap());
    let read = names(&again);
    assert_eq!(read, names(&v));
    assert_eq!(read.last(), Some(&name_a), "{read:?}");
    let b_again = order(&again)[read.iter().position(|n| *n == name_b).unwrap()];
    assert_eq!(again.shape_flips(0, b_again), (true, true));
    assert_eq!(again.save().unwrap(), saved, "not a fixed point");
    while v.can_undo_redo().0 {
        v.undo().unwrap();
    }
    assert_eq!(v.save().unwrap(), source, "undone");
}
