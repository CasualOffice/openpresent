//! Theme and colour import: every construct, the corpus, and hostile input.

use std::path::{Path, PathBuf};

use casual_slide_import::color::{
    FindingKind, Findings, MAX_FINDINGS, MAX_LIST_ENTRIES, read_color, read_color_change,
    read_color_choice, read_color_mru, read_color_replace,
};
use casual_slide_import::theme::{
    Imported, ThemeImportError, import_clipboard_style_sheet, import_theme, import_theme_limited,
    import_theme_manager, import_theme_override, read_color_map, read_color_map_override,
    read_effect_reference, read_theme_parts, read_themeable_font, read_themeable_style,
};
use casual_slide_model::color::{
    Color, ColorTransform, ColorValue, MAX_COLOR_TRANSFORMS, Percentage, PresetColor,
    ResolveContext, Rgb, SchemeColor, SystemColor, TransformKind as K, resolve,
};
use casual_slide_model::theme::{
    ColorMapOverride, ColorMapSlot, ColorSchemeIndex as I, FontCollectionIndex, StyleFill, Theme,
    ThemeableFont, ThemeableStyle,
};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};

const RICH: &str = include_str!("data/rich-theme.xml");
const A: &str = "xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"";
const STRICT: &str = "xmlns:a=\"http://purl.oclc.org/ooxml/drawingml/main\"";

fn parse(xml: &str) -> XmlTree {
    XmlTree::parse(xml.as_bytes(), "t", &OoxmlLimits::default()).unwrap()
}

fn theme_of(xml: &str) -> Imported<Theme> {
    import_theme(&parse(xml)).unwrap()
}

fn rich() -> Imported<Theme> {
    theme_of(RICH)
}

/// One colour element, as a holder's child, read through the public path.
fn color(inner: &str) -> (Option<Color>, Findings) {
    let tree = parse(&format!("<a:h {A}>{inner}</a:h>"));
    let mut findings = Findings::default();
    let got = read_color_choice(&tree, tree.root(), &mut findings);
    (got, findings)
}

fn codes(f: &Findings) -> Vec<&'static str> {
    f.codes()
}

#[test]
fn a_full_theme_reads_with_nothing_to_report() {
    let Imported { value: t, findings } = rich();
    assert_eq!(findings, Findings::default());
    assert_eq!(t.name.as_deref(), Some("Rich & Full"));
    let e = t.elements.as_ref().unwrap();
    let s = e.color_scheme.as_ref().unwrap();
    assert_eq!(s.name.as_deref(), Some("Rich Colours"));
    assert_eq!(s.extensions, ["{CLR-SCHEME-EXT}"]);
    assert_eq!(e.extensions, ["{ELEMENTS-EXT}"]);
    assert_eq!(
        s.get(I::Dk1).unwrap().value,
        ColorValue::System {
            val: SystemColor::WindowText,
            last: Some(Rgb::new(0, 0, 0)),
        }
    );
    assert_eq!(
        s.get(I::Dk2).unwrap().value,
        ColorValue::Srgb(Rgb::new(0x44, 0x54, 0x6A))
    );
    assert_eq!(
        s.get(I::Accent2).unwrap().value,
        ColorValue::ScRgb {
            r: Percentage(50_000),
            g: Percentage(50_000),
            b: Percentage(12_500),
        }
    );
    assert_eq!(
        s.get(I::Accent4).unwrap(),
        &Color {
            value: ColorValue::Preset(PresetColor::Tomato),
            transforms: vec![ColorTransform::new(K::Alpha, 50_000)],
        }
    );
    assert_eq!(
        s.get(I::Accent5).unwrap(),
        &Color {
            value: ColorValue::Scheme(SchemeColor::Dk2),
            transforms: vec![
                ColorTransform::new(K::Tint, 40_000),
                ColorTransform::new(K::SatMod, 300_000),
            ],
        }
    );
    let kinds: Vec<K> = s
        .get(I::Accent6)
        .unwrap()
        .transforms
        .iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(kinds, [K::Comp, K::Inv, K::Gray, K::Gamma, K::InvGamma]);
}

#[test]
fn font_scheme_reads_as_written() {
    let t = rich().value;
    let fonts = t.elements.unwrap().font_scheme.unwrap();
    assert_eq!(fonts.name.as_deref(), Some("Rich Fonts"));
    let major = fonts.major.unwrap();
    let latin = major.latin.unwrap();
    assert_eq!(latin.typeface.as_deref(), Some("Calibri Light"));
    assert_eq!(latin.panose.as_deref(), Some("020F0302020204030204"));
    assert_eq!(latin.pitch_family, Some(34));
    assert_eq!(latin.charset, Some(0));
    assert_eq!(major.ea.unwrap().typeface.as_deref(), Some(""));
    assert_eq!(major.fonts.len(), 2);
    assert_eq!(major.fonts[0].script.as_deref(), Some("Jpan"));
    assert_eq!(
        major.fonts[0].typeface.as_deref(),
        Some("\u{6E38}\u{30B4}\u{30B7}\u{30C3}\u{30AF} Light")
    );
    let minor = fonts.minor.unwrap();
    assert_eq!(minor.extensions, ["{FONT-EXT}"]);
    let latin = minor.latin.unwrap();
    assert_eq!(latin.pitch_family, None, "absent stays absent");
    assert_eq!(
        latin.charset, None,
        "the schema default is not materialized"
    );
    assert_eq!(latin.panose, None);
}

#[test]
fn format_scheme_shows_its_colours_and_kinds() {
    let t = rich().value;
    let f = t.elements.unwrap().format_scheme.unwrap();
    assert_eq!(f.name.as_deref(), Some("Rich Format"));
    let fills = f.fills.as_ref().unwrap();
    assert_eq!(
        fills[0],
        StyleFill::Solid(Some(Color::new(ColorValue::Scheme(SchemeColor::PhClr))))
    );
    let StyleFill::Gradient(stops) = &fills[1] else {
        panic!("{:?}", fills[1]);
    };
    assert_eq!(stops[1].position, Some(100_000));
    assert_eq!(
        stops[1].color.as_ref().unwrap().transforms,
        [ColorTransform::new(K::Shade, 78_000)]
    );
    let StyleFill::Pattern { background, .. } = &fills[2] else {
        panic!("{:?}", fills[2]);
    };
    assert_eq!(
        background.as_ref().unwrap().value,
        ColorValue::Preset(PresetColor::White)
    );
    let lines = f.lines.as_ref().unwrap();
    assert_eq!(lines[0].width, Some(6350));
    assert_eq!(lines[1].fill, Some(StyleFill::None));
    assert_eq!(lines[2].fill, None);
    let effects = f.effects.as_ref().unwrap();
    assert!(effects[1].has_scene3d && !effects[1].has_sp3d);
    assert!(effects[2].has_sp3d);
    let bg = f.background_fills.as_ref().unwrap();
    assert_eq!(bg[1], StyleFill::Picture);
    assert_eq!(bg[2], StyleFill::Group);
    assert_eq!(f.fill(1001), Some(&bg[0]));
}

#[test]
fn object_defaults_extra_schemes_custom_colours_and_family() {
    let t = rich().value;
    let d = t.object_defaults.as_ref().unwrap();
    assert_eq!(d.extensions, ["{DEFAULTS-EXT}"]);
    let sp = d.shape.as_ref().unwrap();
    assert!(sp.has_shape_properties && sp.body_properties.is_some() && sp.list_style.is_some());
    let style = sp.style.as_ref().unwrap();
    assert_eq!(style.line.as_ref().unwrap().idx, Some(2));
    assert_eq!(style.fill.as_ref().unwrap().idx, Some(1001));
    assert_eq!(style.effect.as_ref().unwrap().color, None);
    let font = style.font.as_ref().unwrap();
    assert_eq!(font.idx, Some(FontCollectionIndex::Minor));
    assert!(d.line.as_ref().unwrap().style.is_none());
    assert!(d.text.is_none(), "absent stays absent");

    let extra = &t.extra_color_schemes.as_ref().unwrap()[0];
    let scheme = extra.color_scheme.as_ref().unwrap();
    assert_eq!(scheme.get(I::Hlink), Some(&Color::srgb(7, 7, 7)));
    let map = extra.map.as_ref().unwrap();
    assert_eq!(map.get(ColorMapSlot::Bg1), Some(I::Dk1));

    let custom = t.custom_colors.as_ref().unwrap();
    assert_eq!(custom[0].name.as_deref(), Some("Brand Red"));
    assert_eq!(custom[0].color, Some(Color::srgb(0xC0, 0, 0)));
    assert_eq!(custom[1].name, None);

    let family = t.family.as_ref().unwrap();
    assert_eq!(family.name.as_deref(), Some("Office Theme"));
    assert_eq!(
        family.vid.as_deref(),
        Some("{4A3C46E8-61CC-4603-A589-7422A47A8E4A}")
    );
    assert_eq!(t.extensions.len(), 2);
    assert_eq!(t.extensions[1], "{UNKNOWN-TO-US}");
}

/// A hue of 240 with `lumMod` 75%, and scheme indirection, end to end.
#[test]
fn the_imported_theme_resolves() {
    let t = rich().value;
    let scheme = t.elements.as_ref().unwrap().color_scheme.as_ref().unwrap();
    let ctx = ResolveContext {
        scheme: Some(scheme),
        ..Default::default()
    };
    let r = |i: I| resolve(scheme.get(i).unwrap(), &ctx).unwrap();
    assert_eq!(r(I::Dk1).rgb, Rgb::new(0, 0, 0));
    assert_eq!(r(I::Accent1).rgb.to_hex(), "4472C4");
    assert_eq!(r(I::Accent3).rgb.to_hex(), "0000BF");
    assert_eq!(r(I::Accent4).alpha, 50_000);
    assert_eq!(r(I::Accent4).rgb.to_hex(), "FF6347");
    for i in I::ALL {
        resolve(scheme.get(*i).unwrap(), &ctx).unwrap();
    }
}

#[test]
fn strict_namespace_and_percent_forms_read() {
    let (got, f) = color(r#"<a:srgbClr val="000000"><a:alpha val="33.5%"/></a:srgbClr>"#);
    assert_eq!(f, Findings::default());
    assert_eq!(got.unwrap().transforms[0].value, 33_500);

    let tree = parse(&format!(
        r#"<a:h {STRICT}><a:scrgbClr r="50%" g="0%" b="100%"/></a:h>"#
    ));
    let mut findings = Findings::default();
    let got = read_color_choice(&tree, tree.root(), &mut findings).unwrap();
    assert_eq!(findings, Findings::default());
    assert!(matches!(
        got.value,
        ColorValue::ScRgb {
            r: Percentage(50_000),
            ..
        }
    ));
}

#[test]
fn every_transform_element_reads() {
    for kind in K::ALL {
        let name = kind.as_xml();
        // A transform with no value has no `val` to read: one there is reported, below.
        let value = if kind.domain() == casual_slide_model::color::ValueDomain::None {
            ""
        } else {
            r#" val="1000""#
        };
        let inner = format!(r#"<a:srgbClr val="000000"><a:{name}{value}/></a:srgbClr>"#);
        let (got, f) = color(&inner);
        assert_eq!(f, Findings::default(), "{name}");
        let got = got.unwrap();
        let want = if kind.domain() == casual_slide_model::color::ValueDomain::None {
            0
        } else {
            1000
        };
        assert_eq!(got.transforms, [ColorTransform::new(*kind, want)], "{name}");
    }
}

#[test]
fn every_enum_value_reads_from_xml() {
    for v in SchemeColor::ALL {
        let (got, _) = color(&format!(r#"<a:schemeClr val="{}"/>"#, v.as_xml()));
        assert_eq!(got.unwrap().value, ColorValue::Scheme(*v));
    }
    for v in PresetColor::ALL {
        let (got, _) = color(&format!(r#"<a:prstClr val="{}"/>"#, v.as_xml()));
        assert_eq!(got.unwrap().value, ColorValue::Preset(*v));
    }
    for v in SystemColor::ALL {
        let (got, _) = color(&format!(r#"<a:sysClr val="{}"/>"#, v.as_xml()));
        assert_eq!(
            got.unwrap().value,
            ColorValue::System {
                val: *v,
                last: None
            }
        );
    }
}

#[test]
fn colour_map_and_override() {
    let tree = parse(&format!(
        r#"<o {A}><a:overrideClrMapping bg1="dk1" tx1="lt1" bg2="dk2" tx2="lt2" accent1="accent2" accent2="accent1" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="folHlink" folHlink="hlink"><a:extLst><a:ext uri="{{M}}"/></a:extLst></a:overrideClrMapping></o>"#
    ));
    let mut f = Findings::default();
    let got = read_color_map_override(&tree, tree.root(), &mut f).unwrap();
    assert_eq!(f, Findings::default());
    let ColorMapOverride::Override(map) = got else {
        panic!("{got:?}");
    };
    assert_eq!(map.get(ColorMapSlot::Hlink), Some(I::FolHlink));
    assert_eq!(map.extensions, ["{M}"]);

    let tree = parse(&format!("<o {A}><a:masterClrMapping/></o>"));
    assert_eq!(
        read_color_map_override(&tree, tree.root(), &mut f),
        Some(ColorMapOverride::Master)
    );

    let tree = parse(&format!(
        "<o {A}><a:masterClrMapping/><a:masterClrMapping/></o>"
    ));
    let mut f = Findings::default();
    read_color_map_override(&tree, tree.root(), &mut f);
    assert_eq!(codes(&f), ["CS-CLR-0008"]);

    let tree = parse(&format!("<o {A}/>"));
    let mut f = Findings::default();
    assert_eq!(read_color_map_override(&tree, tree.root(), &mut f), None);
    assert_eq!(codes(&f), ["CS-CLR-0007"]);

    // A bad target and a missing one stay absent, not defaulted.
    let tree = parse(r#"<m bg1="purple" tx1="dk1"/>"#);
    let mut f = Findings::default();
    let map = read_color_map(&tree, tree.root(), &mut f);
    assert_eq!(map.get(ColorMapSlot::Bg1), None);
    assert_eq!(map.get(ColorMapSlot::Tx1), Some(I::Dk1));
    assert_eq!(map.get(ColorMapSlot::Accent1), None);
    assert_eq!(codes(&f), ["CS-CLR-0005", "CS-CLR-0007"]);
}

#[test]
fn override_part_and_clipboard_sheet() {
    let body = RICH
        .split("<a:clrScheme name=\"Rich Colours\">")
        .nth(1)
        .unwrap()
        .split("</a:clrScheme>")
        .next()
        .unwrap();
    let scheme = format!(r#"<a:clrScheme name="O">{body}</a:clrScheme>"#);
    let part = format!("<a:themeOverride {A}>{scheme}</a:themeOverride>");
    let got = import_theme_override(&parse(&part)).unwrap();
    assert_eq!(got.findings, Findings::default());
    assert_eq!(got.value.color_scheme.unwrap().name.as_deref(), Some("O"));
    assert_eq!(got.value.font_scheme, None);

    let empty = import_theme_override(&parse(&format!("<a:themeOverride {A}/>"))).unwrap();
    assert_eq!(empty.value, Default::default());

    let sheet = format!(
        r#"<a:c {A}><a:themeElements>{scheme}</a:themeElements><a:clrMap bg1="lt1"/></a:c>"#
    );
    let got = import_clipboard_style_sheet(&parse(&sheet));
    assert!(got.value.elements.unwrap().color_scheme.is_some());
    assert_eq!(got.value.map.unwrap().get(ColorMapSlot::Bg1), Some(I::Lt1));
}

#[test]
fn colour_effects_mru_and_themeable_choices() {
    let tree = parse(&format!(
        r#"<a:clrChange {A} useA="0"><a:clrFrom><a:srgbClr val="010203"/></a:clrFrom><a:clrTo><a:prstClr val="red"/></a:clrTo></a:clrChange>"#
    ));
    let mut f = Findings::default();
    let got = read_color_change(&tree, tree.root(), &mut f);
    assert_eq!(got.use_alpha, Some(false));
    assert_eq!(got.from, Some(Color::srgb(1, 2, 3)));
    assert!(got.to.is_some());

    let tree = parse(&format!("<a:clrChange {A}/>"));
    assert_eq!(
        read_color_change(&tree, tree.root(), &mut f).use_alpha,
        None
    );

    let tree = parse(&format!(
        r#"<a:clrRepl {A}><a:srgbClr val="010203"/></a:clrRepl>"#
    ));
    assert_eq!(
        read_color_replace(&tree, tree.root(), &mut f),
        Some(Color::srgb(1, 2, 3))
    );

    let tree = parse(&format!(
        r#"<a:mru {A}><a:srgbClr val="010203"/><a:srgbClr val="bad"/><a:prstClr val="red"/></a:mru>"#
    ));
    let mut f = Findings::default();
    assert_eq!(read_color_mru(&tree, tree.root(), &mut f).len(), 2);
    assert_eq!(codes(&f), ["CS-CLR-0001"]);

    let tree = parse(&format!(
        r#"<t {A}><a:fillRef idx="1"><a:srgbClr val="010203"/></a:fillRef></t>"#
    ));
    let mut f = Findings::default();
    let Some(ThemeableStyle::Ref(r)) = read_themeable_style(&tree, tree.root(), &mut f) else {
        panic!();
    };
    assert_eq!((r.idx, r.color), (Some(1), Some(Color::srgb(1, 2, 3))));

    let tree = parse(&format!(r#"<t {A}><a:fill/></t>"#));
    assert_eq!(
        read_themeable_style(&tree, tree.root(), &mut f),
        Some(ThemeableStyle::Inline)
    );

    let tree = parse(&format!(r#"<t {A}><a:fontRef idx="none"/></t>"#));
    let Some(ThemeableFont::Ref(r)) = read_themeable_font(&tree, tree.root(), &mut f) else {
        panic!();
    };
    assert_eq!(r.idx, Some(FontCollectionIndex::None));

    let tree = parse(&format!(
        r#"<t {A}><a:font><a:latin typeface="X"/></a:font></t>"#
    ));
    assert!(matches!(
        read_themeable_font(&tree, tree.root(), &mut f),
        Some(ThemeableFont::Inline(_))
    ));
}

fn fixture_paths() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let mut out = Vec::new();
    for dir in ["generated", "producers"] {
        for entry in std::fs::read_dir(root.join(dir)).unwrap() {
            let path = entry.unwrap().path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if matches!(ext, "pptx" | "potx" | "ppsx") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Every deck in the corpus, and its master's map with its theme, to final values.
#[test]
fn themes_in_the_corpus_import_and_resolve() {
    let paths = fixture_paths();
    assert!(paths.len() >= 10, "{paths:?}");
    let mut seen_producer = false;
    for path in paths {
        let name = path.display().to_string();
        seen_producer |= name.contains("producers");
        let bytes = std::fs::read(&path).unwrap();
        let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default())
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        let parts = read_theme_parts(&mut pkg).unwrap();
        assert!(!parts.is_empty(), "{name}: no theme part");
        for (part, tree) in parts {
            if part.contains("themeOverride") {
                let got = import_theme_override(&tree).unwrap_or_else(|e| panic!("{part}: {e}"));
                assert_eq!(got.findings, Findings::default(), "{name} {part}");
                continue;
            }
            let got = import_theme(&tree).unwrap_or_else(|e| panic!("{name} {part}: {e}"));
            assert_eq!(got.findings, Findings::default(), "{name} {part}");
            let scheme = got.value.elements.unwrap().color_scheme.unwrap();
            let master = pkg.read_part("ppt/slideMasters/slideMaster1.xml").unwrap();
            let master = XmlTree::parse(&master, "m", &OoxmlLimits::default()).unwrap();
            let el = master.child(master.root(), "clrMap").unwrap();
            let mut f = Findings::default();
            let map = read_color_map(&master, el, &mut f);
            assert_eq!(f, Findings::default(), "{name}");
            let ctx = ResolveContext {
                scheme: Some(&scheme),
                map: Some(&map),
                ..Default::default()
            };
            let sc = |s| Color::new(ColorValue::Scheme(s));
            let tx = resolve(&sc(SchemeColor::Tx1), &ctx).unwrap();
            let acc = resolve(&sc(SchemeColor::Accent1), &ctx).unwrap();
            assert_eq!(tx.rgb.to_hex(), "000000", "{name}");
            assert_eq!(acc.rgb.to_hex(), "4472C4", "{name}");
        }
    }
    assert!(seen_producer);
}

#[test]
fn wrong_root_is_refused() {
    assert!(matches!(
        import_theme(&parse(&format!("<a:notTheme {A}/>"))),
        Err(ThemeImportError::WrongRoot { expected: "theme" })
    ));
    // Right name, foreign namespace.
    assert!(import_theme(&parse(r#"<a:theme xmlns:a="urn:x"/>"#)).is_err());
    assert!(import_theme(&parse("<theme/>")).is_err());
    assert!(import_theme_override(&parse(&format!("<a:theme {A}/>"))).is_err());
}

#[test]
fn an_empty_theme_is_absent_all_the_way_down() {
    let got = theme_of(&format!("<a:theme {A}/>"));
    assert_eq!(got.value, Theme::default());
    assert_eq!(codes(&got.findings), ["CS-CLR-0007"]);
}

#[test]
fn bad_values_are_refused_and_reported() {
    for (inner, code) in [
        (r#"<a:srgbClr val="GGGGGG"/>"#, "CS-CLR-0001"),
        (r#"<a:srgbClr val="12345"/>"#, "CS-CLR-0001"),
        (r#"<a:srgbClr val="1234567"/>"#, "CS-CLR-0001"),
        (r#"<a:srgbClr val=""/>"#, "CS-CLR-0001"),
        (r#"<a:srgbClr/>"#, "CS-CLR-0001"),
        (r#"<a:scrgbClr r="1" g="2"/>"#, "CS-CLR-0001"),
        (
            r#"<a:scrgbClr r="99999999999" g="0" b="0"/>"#,
            "CS-CLR-0001",
        ),
        (
            r#"<a:hslClr hue="21600000" sat="0" lum="0"/>"#,
            "CS-CLR-0001",
        ),
        (r#"<a:hslClr hue="-1" sat="0" lum="0"/>"#, "CS-CLR-0001"),
        (r#"<a:hslClr hue="0" sat="x" lum="0"/>"#, "CS-CLR-0001"),
        (r#"<a:schemeClr val="accent7"/>"#, "CS-CLR-0001"),
        (r#"<a:schemeClr val="ACCENT1"/>"#, "CS-CLR-0001"),
        (r#"<a:prstClr val="notAColor"/>"#, "CS-CLR-0001"),
        (r#"<a:sysClr val="notSystem"/>"#, "CS-CLR-0001"),
        ("", "CS-CLR-0007"),
    ] {
        let (got, f) = color(inner);
        assert_eq!(got, None, "{inner}");
        assert_eq!(codes(&f), [code], "{inner}");
    }
    // Malformed attributes: the walker refuses the part (`XML-02`), and an element
    // an edit made malformed is still reported, not read.
    for start in [
        r#"a:srgbClr val="1" val="2""#,
        r#"a:srgbClr val=123456"#,
        r#"a:srgbClr val="&bad;""#,
    ] {
        let source = format!("<a:h {A}><{start}/></a:h>");
        let err = XmlTree::parse(source.as_bytes(), "t", &OoxmlLimits::default()).unwrap_err();
        assert_eq!(err.code(), "CS-XML-0004", "{start}");
        let mut tree = parse(&format!("<a:h {A}/>"));
        let root = tree.root();
        let bad = tree.new_element(start);
        tree.insert_child(root, None, bad);
        let mut f = Findings::default();
        assert_eq!(read_color_choice(&tree, root, &mut f), None, "{start}");
        assert_eq!(codes(&f), ["CS-CLR-0004"], "{start}");
    }
}

#[test]
fn a_bad_last_colour_loses_only_itself() {
    let (got, f) = color(r#"<a:sysClr val="window" lastClr="nope"/>"#);
    assert_eq!(
        got.unwrap().value,
        ColorValue::System {
            val: SystemColor::Window,
            last: None,
        }
    );
    assert_eq!(codes(&f), ["CS-CLR-0005"]);
}

#[test]
fn a_bad_transform_is_dropped_and_reported() {
    for t in [
        r#"<a:tint val="100001"/>"#,
        r#"<a:shade val="-1"/>"#,
        r#"<a:alpha val="200%"/>"#,
        r#"<a:alphaOff val="100001"/>"#,
        r#"<a:alphaMod val="-1"/>"#,
        r#"<a:hue val="21600000"/>"#,
        r#"<a:hueMod val="-1"/>"#,
        r#"<a:lumMod val="abc"/>"#,
        r#"<a:lumMod/>"#,
        r#"<a:satMod val="2147483648"/>"#,
    ] {
        let inner = format!(r#"<a:srgbClr val="000000">{t}<a:lumOff val="5"/></a:srgbClr>"#);
        let (got, f) = color(&inner);
        assert_eq!(
            got.unwrap().transforms,
            [ColorTransform::new(K::LumOff, 5)],
            "{t}"
        );
        assert_eq!(codes(&f), ["CS-CLR-0003"], "{t}");
    }
}

#[test]
fn over_precise_percentages_are_rounded_and_reported() {
    let (got, f) = color(r#"<a:srgbClr val="000000"><a:lumMod val="12.3456%"/></a:srgbClr>"#);
    assert_eq!(got.unwrap().transforms[0].value, 12_346);
    assert_eq!(codes(&f), ["CS-CLR-0006"]);
}

#[test]
fn the_transform_list_is_capped() {
    let list = |n: usize| {
        let items = "<a:lumMod val=\"99000\"/>".repeat(n);
        format!(r#"<a:srgbClr val="000000">{items}</a:srgbClr>"#)
    };
    let (got, f) = color(&list(MAX_COLOR_TRANSFORMS));
    assert_eq!(got.unwrap().transforms.len(), MAX_COLOR_TRANSFORMS);
    assert_eq!(f, Findings::default());
    let (got, f) = color(&list(MAX_COLOR_TRANSFORMS + 1));
    assert_eq!(got, None, "over the cap, the colour is absent");
    assert_eq!(codes(&f), ["CS-CLR-0002"]);
    let (got, f) = color(&list(200_000));
    assert_eq!(got, None);
    assert_eq!(
        f.total, 2,
        "the list cap and the colour's refusal, not per element"
    );
}

#[test]
fn foreign_look_alikes_are_not_believed() {
    // A colour in another namespace is not a colour.
    let (got, f) = color(r#"<x:srgbClr xmlns:x="urn:x" val="123456"/>"#);
    assert_eq!(got, None);
    assert_eq!(codes(&f), ["CS-CLR-0007", "CS-CLR-0009"]);
    // A foreign attribute does not stand in for the schema's.
    let (got, f) = color(r#"<a:srgbClr xmlns:x="urn:x" x:val="000000" val="4472C4"/>"#);
    assert_eq!(
        got.unwrap().value,
        ColorValue::Srgb(Rgb::new(0x44, 0x72, 0xC4))
    );
    assert_eq!(
        codes(&f),
        ["CS-CLR-0009"],
        "the foreign attribute is reported, not read"
    );
    let (got, _) = color(r#"<a:srgbClr xmlns:x="urn:x" x:val="000000"/>"#);
    assert_eq!(got, None);
    // A foreign child of a colour is reported, and the colour stands.
    let (got, f) =
        color(r#"<a:srgbClr xmlns:x="urn:x" val="000000"><x:tint val="1"/></a:srgbClr>"#);
    assert_eq!(got.unwrap().transforms, []);
    assert_eq!(codes(&f), ["CS-CLR-0009"]);
}

/// `COLOR-01`: a hostile deck past each bound, read through the package, and the bound
/// the host set is the one that applies.
#[test]
fn a_hostile_deck_past_each_colour_bound_is_capped_and_reported() {
    let read = |name: &str, limits: &OoxmlLimits| {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/hostile")
            .join(format!("{name}.bin"));
        let bytes = std::fs::read(path).unwrap();
        let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
        let parts = read_theme_parts(&mut pkg).unwrap();
        import_theme_limited(&parts[0].1, limits).unwrap()
    };
    let limits = OoxmlLimits::default();

    let got = read("color-too-many-transforms", &limits);
    assert!(codes(&got.findings).contains(&"CS-CLR-0002"));
    let scheme = got.value.elements.unwrap().color_scheme.unwrap();
    assert_eq!(
        scheme.get(casual_slide_model::theme::ColorSchemeIndex::Dk1),
        None
    );
    // With room, the same deck reads.
    let roomy = OoxmlLimits {
        max_color_transforms: limits.max_color_transforms + 1,
        ..limits
    };
    let got = read("color-too-many-transforms", &roomy);
    let scheme = got.value.elements.unwrap().color_scheme.unwrap();
    assert_eq!(
        scheme
            .get(casual_slide_model::theme::ColorSchemeIndex::Dk1)
            .unwrap()
            .transforms
            .len(),
        limits.max_color_transforms + 1
    );

    let got = read("color-list-capped", &limits);
    assert!(
        codes(&got.findings).contains(&"CS-CLR-0010"),
        "{:?}",
        codes(&got.findings)
    );
    let colors = got.value.custom_colors.unwrap();
    assert_eq!(colors.len(), limits.max_theme_list_entries);
    let tight = OoxmlLimits {
        max_theme_list_entries: 3,
        ..limits
    };
    assert_eq!(
        read("color-list-capped", &tight)
            .value
            .custom_colors
            .unwrap()
            .len(),
        3
    );

    let got = read("color-findings-flood", &limits);
    assert_eq!(got.findings.items.len(), limits.max_color_findings);
    assert!(got.findings.total > limits.max_color_findings);
    let tight = OoxmlLimits {
        max_color_findings: 7,
        ..limits
    };
    let got = read("color-findings-flood", &tight);
    assert_eq!(got.findings.items.len(), 7);
    assert!(got.findings.total > 7);
}

#[test]
fn the_default_bounds_are_the_published_constants() {
    let limits = OoxmlLimits::default();
    assert_eq!(limits.max_color_transforms, MAX_COLOR_TRANSFORMS);
    assert_eq!(limits.max_theme_list_entries, MAX_LIST_ENTRIES);
    assert_eq!(limits.max_color_findings, MAX_FINDINGS);
}

#[test]
fn unknown_children_are_reported() {
    let got = theme_of(&format!(
        "<a:theme {A}><a:themeElements/><a:surprise/><u:other xmlns:u='urn:u'/></a:theme>"
    ));
    assert!(
        got.findings
            .items
            .iter()
            .any(|f| { f.kind == FindingKind::Unmodelled && f.element == "(unknown)" })
    );
    assert!(!format!("{:?}", got.findings).contains("surprise"));
    assert!(!format!("{:?}", got.findings).contains("other"));
}

#[test]
fn duplicates_keep_the_first() {
    let (got, f) = color(r#"<a:srgbClr val="010203"/><a:srgbClr val="040506"/>"#);
    assert_eq!(got, Some(Color::srgb(1, 2, 3)));
    assert_eq!(codes(&f), ["CS-CLR-0008"]);
}

#[test]
fn findings_hold_no_document_values() {
    let got = theme_of(&format!(
        r#"<a:theme {A} name="x"><a:themeElements><a:clrScheme name="SECRET"><a:dk1><a:srgbClr val="SECRET"/></a:dk1></a:clrScheme></a:themeElements></a:theme>"#
    ));
    let text = format!("{:?}", got.findings);
    assert!(!text.contains("SECRET"), "{text}");
    assert!(got.findings.total > 0);
}

#[test]
fn lists_and_findings_are_bounded() {
    let many = format!(
        r#"<a:theme {A}><a:themeElements/><a:custClrLst>{}</a:custClrLst></a:theme>"#,
        r#"<a:custClr><a:srgbClr val="bad"/></a:custClr>"#.repeat(MAX_LIST_ENTRIES * 2)
    );
    let got = theme_of(&many);
    let list = got.value.custom_colors.unwrap();
    assert!(list.len() <= MAX_LIST_ENTRIES + 8, "{}", list.len());
    assert!(got.findings.items.len() <= MAX_FINDINGS);
    assert!(got.findings.total > MAX_FINDINGS);
    let tree = parse(&format!(
        "<a:m {A}>{}</a:m>",
        r#"<a:srgbClr val="000000"/>"#.repeat(MAX_LIST_ENTRIES + 1)
    ));
    let mut f = Findings::default();
    assert_eq!(
        read_color_mru(&tree, tree.root(), &mut f).len(),
        MAX_LIST_ENTRIES
    );
    assert!(codes(&f).contains(&"CS-CLR-0010"));
}

#[test]
fn a_colour_element_reads_directly() {
    let tree = parse(&format!(r#"<a:srgbClr {A} val="010203"/>"#));
    let mut f = Findings::default();
    assert_eq!(
        read_color(&tree, tree.root(), &mut f),
        Some(Color::srgb(1, 2, 3))
    );
}

/// A deterministic mutation campaign over the rich theme. Whatever survives
/// parsing must import and resolve without panicking, and write back identically.
#[test]
fn mutated_themes_never_panic() {
    let original = RICH.as_bytes();
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let tokens: [&[u8]; 8] = [b"\"", b"<", b"&", b"9", b"%", b"-", b" ", b"/"];
    let mut admitted = 0;
    for _ in 0..4000 {
        let mut bytes = original.to_vec();
        for _ in 0..(next() % 4 + 1) {
            let at = (next() % bytes.len() as u64) as usize;
            match next() % 3 {
                0 => bytes[at] = next() as u8,
                1 => {
                    let t = tokens[(next() % tokens.len() as u64) as usize];
                    bytes.splice(at..at, t.iter().copied());
                }
                _ => {
                    let len = ((next() % 16) as usize).min(bytes.len() - at);
                    bytes.drain(at..at + len);
                }
            }
        }
        let Ok(tree) = XmlTree::parse(&bytes, "m", &OoxmlLimits::default()) else {
            continue;
        };
        assert_eq!(tree.to_bytes(), bytes);
        let Ok(got) = import_theme(&tree) else {
            continue;
        };
        admitted += 1;
        if let Some(scheme) = got.value.elements.and_then(|e| e.color_scheme) {
            let ctx = ResolveContext {
                scheme: Some(&scheme),
                ..Default::default()
            };
            for i in I::ALL {
                if let Some(c) = scheme.get(*i) {
                    let _ = resolve(c, &ctx);
                }
            }
        }
    }
    assert!(
        admitted > 100,
        "the campaign reached import {admitted} times"
    );
}

// --- T1C-01 ---------------------------------------------------------------------------

/// `CT_EffectReference`: `@ref` names an effect by a token, and is required.
#[test]
fn an_effect_reference_reads_its_ref() {
    let tree = parse(&format!(r#"<a:effect {A} ref="fillRef"/>"#));
    let mut f = Findings::default();
    let got = read_effect_reference(&tree, tree.root(), &mut f);
    assert_eq!(got.reference.as_deref(), Some("fillRef"));
    assert!(f.items.is_empty());

    let tree = parse(&format!("<a:effect {A}/>"));
    let mut f = Findings::default();
    let got = read_effect_reference(&tree, tree.root(), &mut f);
    assert_eq!(got.reference, None);
    assert_eq!(codes(&f), ["CS-CLR-0007"]);
}

/// `a:themeManager` is `CT_EmptyElement`. A part with anything inside is reported.
#[test]
fn a_theme_manager_part_is_recognised_and_holds_nothing() {
    let got = import_theme_manager(&parse(&format!("<a:themeManager {A}/>"))).unwrap();
    assert!(got.findings.items.is_empty());

    let got = import_theme_manager(&parse(&format!(
        "<a:themeManager {A}><a:surprise/></a:themeManager>"
    )))
    .unwrap();
    assert_eq!(codes(&got.findings), ["CS-CLR-0009"]);

    assert!(matches!(
        import_theme_manager(&parse(&format!("<a:theme {A}/>"))),
        Err(ThemeImportError::WrongRoot {
            expected: "themeManager"
        })
    ));
}
