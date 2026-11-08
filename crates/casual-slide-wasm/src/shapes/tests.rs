use super::*;

#[test]
fn every_preset_has_an_icon() {
    for def in casual_slide_geometry::preset_shapes() {
        let icon = preset_outline(&def.name, 32.0, 32.0).unwrap();
        assert!(icon.contains('M'), "{} has no outline", def.name);
    }
    assert!(preset_outline("notAShape", 32.0, 32.0).is_none());
}

#[test]
fn colours_parse_only_in_the_short_form() {
    let c = parse_color("scheme accent1 lumMod 75000 lumOff 25000").unwrap();
    assert_eq!(c.value, ColorValue::Scheme(SchemeColor::Accent1));
    assert_eq!(c.transforms.len(), 2);
    assert_eq!(
        parse_color("srgb 4472C4").unwrap(),
        Color::srgb(0x44, 0x72, 0xC4)
    );
    for bad in [
        "",
        "srgb 4472C",
        "srgb 4472C4 extra",
        "scheme nope",
        "scheme accent1 lumMod",
        "scheme accent1 lumMod 100001",
        "scheme accent1 alpha 5",
        "scheme accent1 lumMod 1 lumMod 1 lumMod 1 lumMod 1 lumMod 1",
    ] {
        assert!(parse_color(bad).is_none(), "{bad:?} was read");
    }
}

#[test]
fn every_gallery_style_exists_and_no_other() {
    for row in 0..6 {
        for column in 0..7 {
            assert!(theme_style(row, column).is_some());
        }
    }
    assert!(theme_style(6, 0).is_none());
    assert!(theme_style(0, 7).is_none());
}

#[test]
fn a_handle_dragged_lands_where_the_pointer_is() {
    let def = preset_shape("roundRect").unwrap();
    let extents = Extents::new(2_000_000, 1_000_000);
    // The corner handle runs along the top edge: half the short side is 50000.
    let list = solve_handle(def, extents, &[], 0, (250_000.0, 0.0)).unwrap();
    assert_eq!(list, vec![("adj".to_owned(), "val 25000".to_owned())]);
    // Past the end of its range it stops at the range's end.
    let list = solve_handle(def, extents, &[], 0, (5_000_000.0, 0.0)).unwrap();
    assert_eq!(list, vec![("adj".to_owned(), "val 50000".to_owned())]);
    // A rectangle has no handle.
    let rect = preset_shape("rect").unwrap();
    assert!(solve_handle(rect, extents, &[], 0, (0.0, 0.0)).is_none());
}

#[test]
fn a_frame_carries_points_there_and_back() {
    let f = Frame {
        x: 100.0,
        y: 200.0,
        w: 400.0,
        h: 300.0,
        rot: 0.7,
        flip_h: true,
        flip_v: false,
    };
    let (sx, sy) = f.to_slide(30.0, 40.0);
    let (px, py) = f.to_shape(sx, sy);
    assert!((px - 30.0).abs() < 1e-6 && (py - 40.0).abs() < 1e-6);
}
