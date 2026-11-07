//! Editing a theme: only the named node changes, in the part and in the package.

use std::collections::BTreeMap;
use std::path::Path;

use casual_slide_export::color::{ExportError, set_color};
use casual_slide_export::theme::{
    FontRole, FontSlot, add_custom_color, part_edit, set_color_map_override, set_color_map_slot,
    set_color_scheme_name, set_custom_color, set_extra_scheme_color, set_font_scheme_name,
    set_format_scheme_name, set_override_scheme_color, set_scheme_color, set_supplemental_font,
    set_theme_name, set_typeface,
};
use casual_slide_import::color::Findings;
use casual_slide_import::theme::{import_theme, read_color_map, read_theme_parts};
use casual_slide_model::color::{
    Angle, Color, ColorTransform, ColorValue, MAX_COLOR_TRANSFORMS, Percentage, Rgb, SchemeColor,
    SystemColor, TransformKind as K,
};
use casual_slide_model::theme::{ColorMapOverride, ColorMapSlot, ColorSchemeIndex as I, Theme};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};
use casual_slide_package::PartEdits;

const RICH: &str = include_str!("../../casual-slide-import/tests/data/rich-theme.xml");
const THEME_PART: &str = "ppt/theme/theme1.xml";
const A: &str = "xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"";

fn parse(xml: &str) -> XmlTree {
    XmlTree::parse(xml.as_bytes(), "t", &OoxmlLimits::default()).unwrap()
}

fn text(tree: &XmlTree) -> String {
    String::from_utf8(tree.to_bytes()).unwrap()
}

fn model(tree: &XmlTree) -> Theme {
    import_theme(tree).unwrap().value
}

/// `RICH` with `from` replaced by `to`, where `from` must occur exactly once.
fn rich_with(from: &str, to: &str) -> String {
    assert_eq!(RICH.matches(from).count(), 1, "{from}");
    RICH.replacen(from, to, 1)
}

fn all_parts(pkg: &mut PresentationPackage) -> BTreeMap<String, Vec<u8>> {
    pkg.part_names()
        .into_iter()
        .map(|name| {
            let bytes = pkg.read_part(&name).unwrap();
            (name, bytes)
        })
        .collect()
}

fn fixture(path: &str) -> Vec<u8> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    std::fs::read(root.join(path)).unwrap()
}

#[test]
fn editing_a_theme_colour_changes_that_value_and_nothing_else() {
    let mut tree = parse(RICH);
    let changed = set_scheme_color(&mut tree, I::Accent1, &Color::srgb(0x12, 0x34, 0x56)).unwrap();
    assert!(changed);
    assert_eq!(
        text(&tree),
        rich_with(
            r#"<a:accent1><a:srgbClr val="4472C4"/></a:accent1>"#,
            r#"<a:accent1><a:srgbClr val="123456"/></a:accent1>"#,
        )
    );
}

/// Across the whole corpus: edit, save, reopen. Every other part is byte-identical,
/// and the theme differs only in that value.
#[test]
fn save_and_reopen_across_the_corpus() {
    for path in [
        "generated/minimal.pptx",
        "generated/three-slides.pptx",
        "generated/image.pptx",
        "generated/slideshow.ppsx",
        "generated/template.potx",
        "generated/stored-not-deflated.pptx",
        "generated/utf16-rels.pptx",
        "generated/case-differs.pptx",
        "producers/libreoffice-26.8-three-slides.pptx",
        "producers/libreoffice-26.8-image.pptx",
    ] {
        let bytes = fixture(path);
        let mut pkg = PresentationPackage::open(bytes.clone(), OoxmlLimits::default()).unwrap();
        let before = all_parts(&mut pkg);
        let parts = read_theme_parts(&mut pkg).unwrap();
        let (part, mut tree) = parts.into_iter().next().unwrap();
        let original = tree.to_bytes();
        assert_eq!(original, before[&part], "{path}: tree writes back");
        let theme_before = model(&tree);

        let new = Color::srgb(0xAB, 0xCD, 0xEF);
        assert!(
            set_scheme_color(&mut tree, I::Accent1, &new).unwrap(),
            "{path}"
        );
        let saved = pkg.save_with(&part_edit(&part, &original, &tree)).unwrap();
        assert_ne!(saved, bytes);

        let mut reopened = PresentationPackage::open(saved, OoxmlLimits::default()).unwrap();
        let after = all_parts(&mut reopened);
        assert_eq!(
            before.keys().collect::<Vec<_>>(),
            after.keys().collect::<Vec<_>>(),
            "{path}"
        );
        for (name, old) in &before {
            if *name == part {
                let want = String::from_utf8(old.clone()).unwrap().replacen(
                    r#"<a:accent1><a:srgbClr val="4472C4"/>"#,
                    r#"<a:accent1><a:srgbClr val="ABCDEF"/>"#,
                    1,
                );
                assert_eq!(after[name], want.as_bytes(), "{path}: {name}");
            } else {
                assert_eq!(&after[name], old, "{path}: {name} changed");
            }
        }
        let reread = XmlTree::parse(&after[&part], "t", &OoxmlLimits::default()).unwrap();
        let mut want = theme_before;
        let scheme = want
            .elements
            .as_mut()
            .unwrap()
            .color_scheme
            .as_mut()
            .unwrap();
        scheme.slots[I::Accent1 as usize] = Some(new);
        assert_eq!(model(&reread), want, "{path}: model after reopen");
    }
}

#[test]
fn saving_without_an_edit_returns_the_input() {
    let bytes = fixture("generated/minimal.pptx");
    let mut pkg = PresentationPackage::open(bytes.clone(), OoxmlLimits::default()).unwrap();
    assert_eq!(pkg.save_unedited(), bytes);
    let (part, tree) = read_theme_parts(&mut pkg).unwrap().remove(0);
    assert!(part_edit(&part, &tree.to_bytes(), &tree).is_empty());
}

/// A rich theme carried in a real package: the file's unknown extensions come back
/// byte for byte after an edit, and other parts keep their bytes.
#[test]
fn unknown_extensions_survive_an_edit_in_a_package() {
    let mut pkg =
        PresentationPackage::open(fixture("generated/minimal.pptx"), OoxmlLimits::default())
            .unwrap();
    let mut edits = PartEdits::default();
    edits.replace(THEME_PART, RICH.as_bytes().to_vec());
    let with_rich = pkg.save_with(&edits).unwrap();

    let mut pkg = PresentationPackage::open(with_rich, OoxmlLimits::default()).unwrap();
    let before = all_parts(&mut pkg);
    let (part, mut tree) = read_theme_parts(&mut pkg).unwrap().remove(0);
    set_scheme_color(&mut tree, I::Hlink, &Color::srgb(1, 2, 3)).unwrap();
    let saved = pkg
        .save_with(&part_edit(&part, RICH.as_bytes(), &tree))
        .unwrap();
    let mut reopened = PresentationPackage::open(saved, OoxmlLimits::default()).unwrap();
    let after = all_parts(&mut reopened);
    let theme = String::from_utf8(after[&part].clone()).unwrap();
    for kept in [
        r#"<a:ext uri="{UNKNOWN-TO-US}"><z:anything xmlns:z="urn:z" deep="1"><z:child>kept &amp; exact</z:child></z:anything></a:ext>"#,
        r#"<a:ext uri="{CLR-SCHEME-EXT}"><u:keep xmlns:u="urn:unknown" a="1"/></a:ext>"#,
        r#"<thm15:themeFamily xmlns:thm15="http://schemas.microsoft.com/office/thememl/2012/main""#,
        "<!-- Self-authored",
    ] {
        assert!(theme.contains(kept), "{kept}");
    }
    assert_eq!(
        theme,
        rich_with(
            r#"<a:hlink><a:srgbClr val="0563C1"/>"#,
            r#"<a:hlink><a:srgbClr val="010203"/>"#,
        )
    );
    for (name, old) in &before {
        if *name != part {
            assert_eq!(&after[name], old, "{name}");
        }
    }
}

/// Writing what was read changes nothing at all, spelling included: lower-case hex,
/// `50%`, single quotes, `40%` in a transform.
#[test]
fn writing_back_the_model_is_a_fixed_point() {
    let mut tree = parse(RICH);
    let theme = model(&tree);
    let scheme = theme
        .elements
        .as_ref()
        .unwrap()
        .color_scheme
        .as_ref()
        .unwrap();
    for slot in I::ALL {
        let color = scheme.get(*slot).unwrap();
        let changed = set_scheme_color(&mut tree, *slot, color).unwrap();
        assert!(!changed, "{slot:?} reported a change");
    }
    assert_eq!(text(&tree), RICH);
    for (i, custom) in theme.custom_colors.as_ref().unwrap().iter().enumerate() {
        assert!(!set_custom_color(&mut tree, i, custom.color.as_ref().unwrap()).unwrap());
    }
    assert!(!set_theme_name(&mut tree, "Rich & Full").unwrap());
    assert!(!set_color_scheme_name(&mut tree, "Rich Colours").unwrap());
    assert!(!set_font_scheme_name(&mut tree, "Rich Fonts").unwrap());
    assert!(!set_format_scheme_name(&mut tree, "Rich Format").unwrap());
    assert!(!set_typeface(&mut tree, FontRole::Minor, FontSlot::Latin, "Calibri").unwrap());
    assert!(!set_supplemental_font(&mut tree, FontRole::Major, "Arab", "Times New Roman").unwrap());
    assert_eq!(text(&tree), RICH);
}

/// Edit, read, write again: the second round moves nothing.
#[test]
fn an_edit_is_stable_once_made() {
    let color = Color {
        value: ColorValue::Hsl {
            hue: Angle(3_600_000),
            sat: Percentage(12_345),
            lum: Percentage(50_000),
        },
        transforms: vec![
            ColorTransform::new(K::LumMod, 75_000),
            ColorTransform::new(K::Comp, 0),
            ColorTransform::new(K::HueOff, -60_000),
        ],
    };
    let mut tree = parse(RICH);
    assert!(set_scheme_color(&mut tree, I::Accent2, &color).unwrap());
    let once = text(&tree);
    let mut again = parse(&once);
    assert_eq!(
        model(&again)
            .elements
            .unwrap()
            .color_scheme
            .unwrap()
            .get(I::Accent2),
        Some(&color)
    );
    assert!(!set_scheme_color(&mut again, I::Accent2, &color).unwrap());
    assert_eq!(text(&again), once);
}

#[test]
fn changing_a_colours_kind_replaces_that_element_only() {
    let mut tree = parse(RICH);
    set_scheme_color(&mut tree, I::Dk1, &Color::srgb(1, 2, 3)).unwrap();
    assert_eq!(
        text(&tree),
        rich_with(
            r#"<a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>"#,
            r#"<a:dk1><a:srgbClr val="010203"/></a:dk1>"#,
        )
    );
}

#[test]
fn every_colour_model_writes_and_reads_back() {
    let values = [
        ColorValue::ScRgb {
            r: Percentage(1),
            g: Percentage(-2),
            b: Percentage(300_000),
        },
        ColorValue::Srgb(Rgb::new(9, 8, 7)),
        ColorValue::Hsl {
            hue: Angle(0),
            sat: Percentage(1),
            lum: Percentage(2),
        },
        ColorValue::System {
            val: SystemColor::ThreeDDkShadow,
            last: Some(Rgb::new(1, 1, 1)),
        },
        ColorValue::System {
            val: SystemColor::MenuBar,
            last: None,
        },
        ColorValue::Scheme(SchemeColor::PhClr),
        ColorValue::Preset(casual_slide_model::color::PresetColor::LtGoldenrodYellow),
    ];
    for from in values {
        for to in values {
            let mut tree = parse(&format!("<a:h {A}><a:srgbClr val=\"000000\"/></a:h>"));
            let root = tree.root();
            set_color(&mut tree, root, &Color::new(from)).unwrap();
            let want = Color::new(to);
            set_color(&mut tree, root, &want).unwrap();
            let mut f = Findings::default();
            let reread = parse(&text(&tree));
            let got = casual_slide_import::color::read_color_choice(&reread, reread.root(), &mut f);
            assert_eq!(got, Some(want), "{from:?} to {to:?}");
            assert_eq!(f, Findings::default());
        }
    }
}

#[test]
fn a_same_kind_edit_patches_attributes_not_the_element() {
    let mut tree = parse(&format!(
        r#"<a:h {A}><a:scrgbClr future="keep" r = '50%' g="1" b="2"/></a:h>"#
    ));
    let root = tree.root();
    let c = Color::new(ColorValue::ScRgb {
        r: Percentage(25_000),
        g: Percentage(1),
        b: Percentage(3),
    });
    assert!(set_color(&mut tree, root, &c).unwrap());
    assert_eq!(
        text(&tree),
        format!(r#"<a:h {A}><a:scrgbClr future="keep" r = '25%' g="1" b="3"/></a:h>"#)
    );
}

#[test]
fn removing_last_colour_removes_only_that_attribute() {
    let mut tree = parse(&format!(
        r#"<a:h {A}><a:sysClr val="window" lastClr="FFFFFF"/></a:h>"#
    ));
    let root = tree.root();
    let c = Color::new(ColorValue::System {
        val: SystemColor::Window,
        last: None,
    });
    assert!(set_color(&mut tree, root, &c).unwrap());
    assert_eq!(
        text(&tree),
        format!(r#"<a:h {A}><a:sysClr val="window"/></a:h>"#)
    );
}

#[test]
fn transform_edits_keep_foreign_children_and_their_place() {
    let mut tree = parse(&format!(
        r#"<a:h {A} xmlns:x="urn:x"><a:srgbClr val="000000"><a:tint val="1"/><x:keep/><a:shade val="2"/></a:srgbClr></a:h>"#
    ));
    let root = tree.root();
    let mut c = Color::srgb(0, 0, 0);
    c.transforms = vec![
        ColorTransform::new(K::Inv, 0),
        ColorTransform::new(K::Alpha, 5),
    ];
    assert!(set_color(&mut tree, root, &c).unwrap());
    let out = text(&tree);
    assert!(
        out.contains(r#"<a:srgbClr val="000000"><a:inv/><a:alpha val="5"/><x:keep/></a:srgbClr>"#),
        "{out}"
    );
    c.transforms.clear();
    assert!(set_color(&mut tree, root, &c).unwrap());
    assert!(text(&tree).contains(r#"<a:srgbClr val="000000"><x:keep/></a:srgbClr>"#));
}

#[test]
fn a_strict_part_gets_strict_percentages() {
    let strict = r#"xmlns:a="http://purl.oclc.org/ooxml/drawingml/main""#;
    let mut tree = parse(&format!("<a:h {strict}><a:srgbClr val=\"000000\"/></a:h>"));
    let root = tree.root();
    let mut c = Color::new(ColorValue::ScRgb {
        r: Percentage(50_000),
        g: Percentage(12_500),
        b: Percentage(-1),
    });
    c.transforms = vec![ColorTransform::new(K::Tint, 40_000)];
    set_color(&mut tree, root, &c).unwrap();
    assert_eq!(
        text(&tree),
        format!(
            r#"<a:h {strict}><a:scrgbClr r="50%" g="12.5%" b="-0.001%"><a:tint val="40%"/></a:scrgbClr></a:h>"#
        )
    );
}

#[test]
fn the_prefix_comes_from_scope() {
    let mut tree = parse(
        r#"<h xmlns="http://schemas.openxmlformats.org/drawingml/2006/main"><srgbClr val="000000"/></h>"#,
    );
    let root = tree.root();
    set_color(
        &mut tree,
        root,
        &Color::new(ColorValue::Scheme(SchemeColor::Tx1)),
    )
    .unwrap();
    assert!(text(&tree).contains(r#"<schemeClr val="tx1"/>"#));

    let mut tree = parse(r#"<h xmlns="urn:other"/>"#);
    let root = tree.root();
    assert_eq!(
        set_color(&mut tree, root, &Color::srgb(0, 0, 0)),
        Err(ExportError::NoNamespace)
    );
}

#[test]
fn names_fonts_and_custom_colours() {
    let mut tree = parse(RICH);
    assert!(set_theme_name(&mut tree, "A \"B\" <C> & D").unwrap());
    assert!(text(&tree).contains(r#"name="A &quot;B&quot; &lt;C&gt; &amp; D">"#));
    assert_eq!(
        model(&parse(&text(&tree))).name.as_deref(),
        Some("A \"B\" <C> & D")
    );

    let mut tree = parse(RICH);
    assert!(set_typeface(&mut tree, FontRole::Major, FontSlot::Latin, "Aptos").unwrap());
    assert_eq!(
        text(&tree),
        rich_with(r#"typeface="Calibri Light""#, r#"typeface="Aptos""#)
    );

    let mut tree = parse(RICH);
    assert!(set_supplemental_font(&mut tree, FontRole::Major, "Arab", "Arial").unwrap());
    assert_eq!(
        text(&tree),
        rich_with(
            r#"<a:font script="Arab" typeface="Times New Roman"/>"#,
            r#"<a:font script="Arab" typeface="Arial"/>"#
        )
    );

    // A new script goes before extLst, where the schema puts it.
    let mut tree = parse(RICH);
    assert!(set_supplemental_font(&mut tree, FontRole::Minor, "Thai", "Tahoma").unwrap());
    assert!(text(&tree).contains(
        r#"<a:font script="Thai" typeface="Tahoma"/><a:extLst><a:ext uri="{FONT-EXT}"/>"#
    ));

    let mut tree = parse(RICH);
    assert!(set_custom_color(&mut tree, 1, &Color::srgb(1, 1, 1)).unwrap());
    assert!(text(&tree).contains(r#"<a:custClr><a:srgbClr val="010101"/></a:custClr>"#));

    let mut tree = parse(RICH);
    add_custom_color(&mut tree, Some("New"), &Color::srgb(2, 2, 2)).unwrap();
    assert!(
        text(&tree).contains(
            r#"<a:custClr name="New"><a:srgbClr val="020202"/></a:custClr></a:custClrLst>"#
        )
    );

    // With no list, the new list goes before the root's extLst.
    let bare = rich_with(
        r#"<a:custClr name="Brand Red"><a:srgbClr val="C00000"/></a:custClr>"#,
        "",
    );
    let bare = bare.split("<a:custClrLst>").next().unwrap().to_owned()
        + bare.split("</a:custClrLst>").nth(1).unwrap();
    let mut tree = parse(&bare);
    add_custom_color(&mut tree, None, &Color::srgb(2, 2, 2)).unwrap();
    let out = text(&tree);
    let list = out.find("<a:custClrLst>").unwrap();
    assert!(list < out.rfind("<a:extLst>").unwrap(), "{out}");
    assert_eq!(model(&parse(&out)).custom_colors.unwrap().len(), 1);

    let mut tree = parse(RICH);
    assert!(set_extra_scheme_color(&mut tree, 0, I::Accent1, &Color::srgb(7, 7, 7)).unwrap());
    assert_eq!(
        text(&tree),
        rich_with(
            r#"<a:accent1><a:srgbClr val="010101"/></a:accent1>"#,
            r#"<a:accent1><a:srgbClr val="070707"/></a:accent1>"#,
        )
    );
}

#[test]
fn colour_map_edits() {
    let master = fixture("generated/minimal.pptx");
    let mut pkg = PresentationPackage::open(master, OoxmlLimits::default()).unwrap();
    let bytes = pkg.read_part("ppt/slideMasters/slideMaster1.xml").unwrap();
    let source = String::from_utf8(bytes).unwrap();
    let mut tree = parse(&source);
    let el = tree.child(tree.root(), "clrMap").unwrap();
    assert!(set_color_map_slot(&mut tree, el, ColorMapSlot::Bg1, I::Dk1).unwrap());
    assert_eq!(
        text(&tree),
        source.replacen(r#"bg1="lt1""#, r#"bg1="dk1""#, 1)
    );
    assert!(!set_color_map_slot(&mut tree, el, ColorMapSlot::Bg1, I::Dk1).unwrap());
    let mut f = Findings::default();
    let reread = parse(&text(&tree));
    let el = reread.child(reread.root(), "clrMap").unwrap();
    assert_eq!(
        read_color_map(&reread, el, &mut f).get(ColorMapSlot::Bg1),
        Some(I::Dk1)
    );
}

#[test]
fn colour_map_override_edits() {
    let source = format!("<p:clrMapOvr xmlns:p=\"urn:p\" {A}><a:masterClrMapping/></p:clrMapOvr>");
    let mut tree = parse(&source);
    let root = tree.root();
    assert!(!set_color_map_override(&mut tree, root, &ColorMapOverride::Master).unwrap());
    assert_eq!(text(&tree), source);

    let mut map = casual_slide_model::theme::ColorMap::default();
    map.slots[ColorMapSlot::Bg1 as usize] = Some(I::Dk1);
    let over = ColorMapOverride::Override(map);
    assert!(set_color_map_override(&mut tree, root, &over).unwrap());
    assert!(text(&tree).contains(r#"<a:overrideClrMapping bg1="dk1"/>"#));
    assert!(!set_color_map_override(&mut tree, root, &over).unwrap());
    assert!(set_color_map_override(&mut tree, root, &ColorMapOverride::Master).unwrap());
    assert_eq!(text(&tree), source);

    let mut empty = parse(&format!("<p:clrMapOvr xmlns:p=\"urn:p\" {A}/>"));
    let root = empty.root();
    assert!(set_color_map_override(&mut empty, root, &ColorMapOverride::Master).unwrap());
    assert!(text(&empty).contains("<a:masterClrMapping/></p:clrMapOvr>"));
}

#[test]
fn override_part_edit() {
    let body = format!(
        r#"<a:themeOverride {A}><a:clrScheme name="O"><a:dk1><a:srgbClr val="000000"/></a:dk1></a:clrScheme></a:themeOverride>"#
    );
    let mut tree = parse(&body);
    assert!(set_override_scheme_color(&mut tree, I::Dk1, &Color::srgb(1, 1, 1)).unwrap());
    assert_eq!(text(&tree), body.replace("000000", "010101"));
}

/// A refused edit leaves the bytes alone.
#[test]
fn bad_edits_are_refused_and_change_nothing() {
    let mut tree = parse(RICH);
    let mut too_many = Color::srgb(0, 0, 0);
    too_many.transforms = vec![ColorTransform::new(K::Comp, 0); MAX_COLOR_TRANSFORMS + 1];
    assert_eq!(
        set_scheme_color(&mut tree, I::Accent1, &too_many),
        Err(ExportError::TooManyTransforms)
    );
    let mut out_of_range = Color::srgb(0, 0, 0);
    out_of_range.transforms = vec![ColorTransform::new(K::Tint, 100_001)];
    assert!(matches!(
        set_scheme_color(&mut tree, I::Accent1, &out_of_range),
        Err(ExportError::InvalidValue(_))
    ));
    let bad_hue = Color::new(ColorValue::Hsl {
        hue: Angle(21_600_000),
        sat: Percentage(0),
        lum: Percentage(0),
    });
    assert!(set_scheme_color(&mut tree, I::Accent1, &bad_hue).is_err());
    assert!(matches!(
        set_theme_name(&mut tree, "bad\u{1}name"),
        Err(ExportError::InvalidValue(_))
    ));
    assert!(matches!(
        add_custom_color(&mut tree, Some("\u{0}"), &Color::srgb(0, 0, 0)),
        Err(ExportError::InvalidValue(_))
    ));
    assert!(add_custom_color(&mut tree, None, &too_many).is_err());
    assert_eq!(text(&tree), RICH);

    // Refused before the list is created, too.
    let source = format!("<a:theme {A}/>");
    let mut tree = parse(&source);
    assert!(add_custom_color(&mut tree, None, &too_many).is_err());
    assert_eq!(text(&tree), source);
}

/// A transform the importer dropped, and a value it rounded, are still in the part.
/// Writing the model back keeps both, exactly.
#[test]
fn what_import_set_aside_survives_writing_the_model_back() {
    let source = format!(
        r#"<a:h {A}><a:srgbClr val="000000"><a:tint val="999999"/><a:lumMod val="12.3456%"/></a:srgbClr></a:h>"#
    );
    let mut tree = parse(&source);
    let root = tree.root();
    let mut f = Findings::default();
    let read = casual_slide_import::color::read_color_choice(&tree, root, &mut f).unwrap();
    assert_eq!(read.transforms, [ColorTransform::new(K::LumMod, 12_346)]);
    assert!(!set_color(&mut tree, root, &read).unwrap());
    assert_eq!(text(&tree), source);
}

#[test]
fn a_utf16_part_edits_and_stays_utf16() {
    let mut bytes = vec![0xFF, 0xFE];
    RICH.encode_utf16()
        .for_each(|u| bytes.extend(u.to_le_bytes()));
    let mut tree = XmlTree::parse(&bytes, "t", &OoxmlLimits::default()).unwrap();
    assert_eq!(tree.to_bytes(), bytes);
    set_scheme_color(&mut tree, I::Accent1, &Color::srgb(1, 2, 3)).unwrap();
    let mut want = vec![0xFF, 0xFE];
    rich_with(r#"val="4472C4""#, r#"val="010203""#)
        .encode_utf16()
        .for_each(|u| want.extend(u.to_le_bytes()));
    assert_eq!(tree.to_bytes(), want);
}

#[test]
fn a_missing_node_is_refused_and_absent_stays_absent() {
    let mut tree = parse(&format!("<a:theme {A}/>"));
    assert_eq!(
        set_scheme_color(&mut tree, I::Dk1, &Color::srgb(0, 0, 0)),
        Err(ExportError::NodeMissing("themeElements"))
    );
    assert!(set_custom_color(&mut tree, 0, &Color::srgb(0, 0, 0)).is_err());
    assert!(set_extra_scheme_color(&mut tree, 9, I::Dk1, &Color::srgb(0, 0, 0)).is_err());
    assert_eq!(text(&tree), format!("<a:theme {A}/>"));
    let mut tree = parse(RICH);
    assert!(set_custom_color(&mut tree, 99, &Color::srgb(0, 0, 0)).is_err());
    assert_eq!(text(&tree), RICH);
}

#[test]
fn a_malformed_colour_element_is_not_patched() {
    // The walker refuses such a part (`XML-02`), so no parsed tree holds one.
    let source = format!(r#"<a:h {A}><a:srgbClr val="1" val="2"/></a:h>"#);
    let refused = XmlTree::parse(source.as_bytes(), "t", &OoxmlLimits::default());
    assert_eq!(refused.unwrap_err().code(), "CS-XML-0004");
    // One an edit made is still not patched.
    let mut tree = parse(&format!("<a:h {A}/>"));
    let root = tree.root();
    let bad = tree.new_element(r#"a:srgbClr val="1" val="2""#);
    tree.insert_child(root, None, bad);
    let before = text(&tree);
    assert_eq!(
        set_color(&mut tree, root, &Color::srgb(1, 1, 1)),
        Err(ExportError::MalformedNode("colour"))
    );
    assert_eq!(text(&tree), before);
}

/// Mutated parts: an edit either refuses or leaves a part that still parses.
#[test]
fn edits_on_mutated_themes_never_panic_or_corrupt() {
    let mut state: u64 = 0xDEAD_BEEF_CAFE_F00D;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut edited = 0;
    for _ in 0..3000 {
        let mut bytes = RICH.as_bytes().to_vec();
        for _ in 0..(next() % 3 + 1) {
            let at = (next() % bytes.len() as u64) as usize;
            bytes[at] = (next() % 128) as u8;
        }
        let Ok(mut tree) = XmlTree::parse(&bytes, "m", &OoxmlLimits::default()) else {
            continue;
        };
        let before = tree.to_bytes();
        match set_scheme_color(&mut tree, I::Accent1, &Color::srgb(1, 2, 3)) {
            Ok(true) => {
                edited += 1;
                XmlTree::parse(&tree.to_bytes(), "m", &OoxmlLimits::default())
                    .expect("an accepted edit keeps the part parseable");
            }
            Ok(false) => assert_eq!(tree.to_bytes(), before),
            Err(_) => assert_eq!(tree.to_bytes(), before),
        }
    }
    assert!(edited > 100, "{edited}");
}
