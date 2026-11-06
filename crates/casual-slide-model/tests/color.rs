//! The resolver against worked values: the standard's own examples first, then
//! values recalled from PowerPoint's output, then decisions the standard leaves
//! open, which are pinned under `open_*` so that changing one is a visible act.

use casual_slide_model::color::{
    Angle, Color, ColorTransform, ColorValue, MAX_COLOR_TRANSFORMS, PERCENT_100, Percentage,
    PresetColor, ResolveContext, ResolveError, Rgb, SchemeColor, SystemColor, TransformKind as K,
    format_percentage, parse_percentage, resolve,
};
use casual_slide_model::theme::{
    ColorMap, ColorMapOverride, ColorMapSlot, ColorScheme, ColorSchemeIndex, FontCollectionIndex,
    FormatScheme, OnOffStyle, StyleFill, ThemeOverride, effective_color_map,
    effective_color_scheme,
};

include!("standard_presets.inc");

fn hex(s: &str) -> Color {
    Color::new(ColorValue::Srgb(Rgb::from_hex(s).unwrap()))
}

fn with(mut c: Color, ts: &[(K, i32)]) -> Color {
    c.transforms
        .extend(ts.iter().map(|&(k, v)| ColorTransform::new(k, v)));
    c
}

fn out(c: &Color) -> (String, u32) {
    let r = resolve(c, &ResolveContext::default()).unwrap();
    (r.rgb.to_hex(), r.alpha)
}

fn rgb_of(base: &str, ts: &[(K, i32)]) -> String {
    out(&with(hex(base), ts)).0
}

/// Every example in ECMA-376 Part 1, 5th ed., 20.1.2.3 that states a result and
/// is arithmetically consistent, one per transform.
#[test]
fn the_standards_worked_examples() {
    let cases: &[(&str, K, i32, &str)] = &[
        ("00FF00", K::Tint, 50_000, "BCFFBC"),
        ("00FF00", K::Shade, 50_000, "00BC00"),
        ("FF0000", K::Comp, 0, "00FFFF"),
        ("FF0000", K::Inv, 0, "00FFFF"),
        ("00FF00", K::Blue, 100_000, "00FFFF"),
        ("0000FF", K::BlueMod, 50_000, "000080"),
        ("0000FF", K::BlueOff, -20_000, "0000CC"),
        ("0000FF", K::Green, 100_000, "00FFFF"),
        ("00FF00", K::GreenMod, 50_000, "008000"),
        ("00FF00", K::GreenOff, -20_000, "00CC00"),
        ("00FF00", K::Red, 100_000, "FFFF00"),
        ("FF0000", K::RedMod, 50_000, "800000"),
        ("FF0000", K::RedOff, -20_000, "CC0000"),
        ("00FF00", K::HueMod, 50_000, "FFFF00"),
        ("00FF00", K::Lum, 20_000, "006600"),
        ("00FF00", K::LumOff, -20_000, "009900"),
        ("00FF00", K::SatMod, 20_000, "669966"),
    ];
    for &(base, kind, val, want) in cases {
        assert_eq!(rgb_of(base, &[(kind, val)]), want, "{base} {kind:?} {val}");
    }
    // 20.1.2.3.30: 50% scRGB is BCBCBC.
    let half = Percentage(50_000);
    let sc = Color::new(ColorValue::ScRgb {
        r: half,
        g: half,
        b: half,
    });
    assert_eq!(out(&sc).0, "BCBCBC");
}

/// The rest of each transform, by value worked outside this code.
#[test]
fn every_transform_has_a_worked_value() {
    let cases: &[(&str, K, i32, &str, u32)] = &[
        ("4472C4", K::Comp, 0, "C49644", 100_000),
        ("4472C4", K::Hue, 7_200_000, "44C444", 100_000),
        ("00FF00", K::HueOff, 7_200_000, "0000FF", 100_000),
        ("00FF00", K::Sat, 0, "808080", 100_000),
        ("00FF00", K::SatOff, -100_000, "808080", 100_000),
        ("808080", K::LumMod, 0, "000000", 100_000),
        ("000000", K::Tint, 0, "FFFFFF", 100_000),
        ("FFFFFF", K::Shade, 0, "000000", 100_000),
        ("4472C4", K::Alpha, 25_000, "4472C4", 25_000),
        ("4472C4", K::AlphaOff, -10_000, "4472C4", 90_000),
        ("4472C4", K::AlphaMod, 50_000, "4472C4", 50_000),
    ];
    for &(base, kind, val, want, alpha) in cases {
        let got = out(&with(hex(base), &[(kind, val)]));
        assert_eq!(got, (want.to_owned(), alpha), "{base} {kind:?} {val}");
    }
}

/// Limits the standard states: offsets and modulations never leave 0 to 100%.
#[test]
fn results_stay_in_range() {
    assert_eq!(rgb_of("FF0000", &[(K::RedMod, 200_000)]), "FF0000");
    assert_eq!(rgb_of("FF0000", &[(K::RedOff, 50_000)]), "FF0000");
    assert_eq!(rgb_of("000000", &[(K::RedOff, -50_000)]), "000000");
    assert_eq!(rgb_of("808080", &[(K::LumMod, 900_000)]), "FFFFFF");
    let a = out(&with(hex("000000"), &[(K::AlphaOff, 100_000)]));
    assert_eq!(a.1, 100_000, "10% offset to 100% opaque stays 100%");
    let a = out(&with(hex("000000"), &[(K::AlphaMod, 200_000)]));
    assert_eq!(a.1, 100_000, "never beyond 100%");
    let a = out(&with(
        hex("000000"),
        &[(K::Alpha, 50_000), (K::AlphaOff, -100_000)],
    ));
    assert_eq!(a.1, 0);
    // 20.1.2.3.16: past 360 is treated as 360, which is red again, not a wrap.
    assert_eq!(rgb_of("0000FF", &[(K::HueOff, 21_600_000)]), "FF0000");
    assert_eq!(rgb_of("00FF00", &[(K::HueOff, -21_600_000)]), "FF0000");
}

/// The Office 2013 to 2022 accents, as PowerPoint's colour picker and exported
/// SVG are recalled to show them. Recalled, not produced by a PowerPoint run here.
#[test]
fn powerpoint_palette_lum_transforms() {
    let rows = [
        ("4472C4", ["DAE3F3", "B4C7E7", "8FAADC", "2F5597", "203864"]),
        ("ED7D31", ["FBE5D6", "F8CBAD", "F4B183", "C55A11", "843C0C"]),
        ("A5A5A5", ["EDEDED", "DBDBDB", "C9C9C9", "7C7C7C", "525252"]),
        ("FFC000", ["FFF2CC", "FFE699", "FFD966", "BF9000", "7F6000"]),
        ("5B9BD5", ["DEEBF7", "BDD7EE", "9DC3E6", "2E75B6", "1F4E79"]),
        ("70AD47", ["E2F0D9", "C5E0B4", "A9D18E", "548235", "385723"]),
    ];
    let mut misses = Vec::new();
    for (base, [l80, l60, l40, d25, d50]) in rows {
        let cases: [(&[(K, i32)], &str); 5] = [
            (&[(K::LumMod, 20_000), (K::LumOff, 80_000)], l80),
            (&[(K::LumMod, 40_000), (K::LumOff, 60_000)], l60),
            (&[(K::LumMod, 60_000), (K::LumOff, 40_000)], l40),
            (&[(K::LumMod, 75_000)], d25),
            (&[(K::LumMod, 50_000)], d50),
        ];
        for (ts, want) in cases {
            let got = rgb_of(base, ts);
            if got != want {
                misses.push((base, got, want));
            }
        }
    }
    // Four lumMod 50% values differ from the recollection by one step, all at or
    // near an exact half. Open: PowerPoint's rounding may not be half up, or the
    // recollection may be wrong. A PowerPoint fixture settles it (BEH-03).
    let expected = [
        ("ED7D31", "843C0B".to_owned(), "843C0C"),
        ("A5A5A5", "535353".to_owned(), "525252"),
        ("FFC000", "806000".to_owned(), "7F6000"),
        ("70AD47", "385724".to_owned(), "385723"),
    ];
    assert_eq!(misses, expected);
}

/// `shade` 50% is the default shape outline over each accent, and `tint` 40%, 20%
/// the bands of the default table style. Recalled as above.
#[test]
fn powerpoint_defaults_tint_and_shade() {
    for (base, line) in [
        ("4472C4", "2F528F"),
        ("5B9BD5", "41719C"),
        ("ED7D31", "AE5A21"),
        ("A5A5A5", "787878"),
        ("FFC000", "BC8C00"),
        ("70AD47", "507E32"),
    ] {
        assert_eq!(rgb_of(base, &[(K::Shade, 50_000)]), line, "{base}");
    }
    assert_eq!(rgb_of("4472C4", &[(K::Tint, 40_000)]), "CFD5EA");
    assert_eq!(rgb_of("4472C4", &[(K::Tint, 20_000)]), "E9EBF5");
    assert_eq!(rgb_of("5B9BD5", &[(K::Tint, 40_000)]), "D2DEEF");
    assert_eq!(rgb_of("5B9BD5", &[(K::Tint, 20_000)]), "EAEFF7");
}

/// Decisions the standard leaves open. A change here is a decision, not a fix.
#[test]
fn open_inverse_is_in_encoded_srgb() {
    assert_eq!(rgb_of("4472C4", &[(K::Inv, 0)]), "BB8D3B");
}

#[test]
fn open_gray_is_rec709_over_encoded_srgb() {
    assert_eq!(rgb_of("FF0000", &[(K::Gray, 0)]), "363636");
    assert_eq!(rgb_of("4472C4", &[(K::Gray, 0)]), "6E6E6E");
}

#[test]
fn open_gamma_is_the_srgb_transfer_on_the_encoded_value() {
    assert_eq!(rgb_of("808080", &[(K::Gamma, 0)]), "BCBCBC");
    assert_eq!(rgb_of("808080", &[(K::InvGamma, 0)]), "373737");
}

#[test]
fn open_hue_mod_limits_at_360() {
    assert_eq!(rgb_of("0000FF", &[(K::HueMod, 200_000)]), "FF0000");
}

#[test]
fn open_rounding_is_half_up_where_the_standards_examples_disagree() {
    // The standard gives 19,E5,19 here and 40,C0,40 for `sat` 50%.
    assert_eq!(rgb_of("00FF00", &[(K::SatOff, -20_000)]), "1AE61A");
    assert_eq!(rgb_of("00FF00", &[(K::Sat, 50_000)]), "40BF40");
}

#[test]
fn computed_where_the_standards_example_is_defective() {
    // Given as 007500 in 20.1.2.3.20 and as 000080 in 20.1.2.3.13.
    assert_eq!(rgb_of("00FF00", &[(K::LumMod, 50_000)]), "008000");
    let blue = Color::new(ColorValue::Hsl {
        hue: Angle(14_400_000),
        sat: Percentage(100_000),
        lum: Percentage(50_000),
    });
    assert_eq!(out(&blue).0, "0000FF");
}

#[test]
fn the_standards_hsl_equivalences_hold() {
    let hsl = |hue, sat, lum| {
        Color::new(ColorValue::Hsl {
            hue: Angle(hue),
            sat: Percentage(sat),
            lum: Percentage(lum),
        })
    };
    let blue = out(&hsl(14_400_000, 100_000, 50_000));
    assert_eq!(
        out(&with(hsl(0, 100_000, 50_000), &[(K::Hue, 14_400_000)])),
        blue
    );
    assert_eq!(
        out(&with(hsl(14_400_000, 100_000, 0), &[(K::Lum, 50_000)])),
        blue
    );
    assert_eq!(
        out(&with(hsl(14_400_000, 0, 50_000), &[(K::Sat, 100_000)])),
        blue
    );
}

#[test]
fn order_is_document_order() {
    let a = rgb_of("4472C4", &[(K::LumMod, 50_000), (K::LumOff, 40_000)]);
    let b = rgb_of("4472C4", &[(K::LumOff, 40_000), (K::LumMod, 50_000)]);
    assert_ne!(a, b);
    let a = out(&with(
        hex("000000"),
        &[(K::Alpha, 50_000), (K::AlphaMod, 50_000)],
    ));
    let b = out(&with(
        hex("000000"),
        &[(K::AlphaMod, 50_000), (K::Alpha, 50_000)],
    ));
    assert_eq!((a.1, b.1), (25_000, 50_000));
}

#[test]
fn hue_survives_black_within_hsl() {
    // Staying in HSL keeps the hue that a trip through RGB would lose.
    let got = rgb_of("FF0000", &[(K::LumMod, 0), (K::LumOff, 50_000)]);
    assert_eq!(got, "FF0000");
}

#[test]
fn scrgb_out_of_gamut_is_limited_only_on_leaving_linear() {
    let sc = |r| {
        Color::new(ColorValue::ScRgb {
            r: Percentage(r),
            g: Percentage(-20_000),
            b: Percentage(0),
        })
    };
    assert_eq!(out(&sc(150_000)).0, "FF0000");
    assert_eq!(out(&with(sc(200_000), &[(K::Shade, 50_000)])).0, "FF0000");
}

#[test]
fn all_190_presets_have_the_standards_values() {
    assert_eq!(PresetColor::ALL.len(), 190);
    assert_eq!(STANDARD_PRESETS.len(), 190);
    for (name, r, g, b) in STANDARD_PRESETS {
        let preset = PresetColor::from_xml(name).unwrap_or_else(|| panic!("{name}"));
        assert_eq!(preset.as_xml(), name);
        assert_eq!(preset.rgb(), Rgb::new(r, g, b), "{name}");
        let got = resolve(&Color::new(ColorValue::Preset(preset)), &Default::default());
        assert_eq!(got.unwrap().rgb, Rgb::new(r, g, b), "{name} resolved");
    }
}

/// `dk` and `med` are spellings of `dark` and `medium`, and `grey` of `gray`. The
/// standard's table breaks this twice. It is followed as written and these two are
/// reported: `dkSeaGreen` is (143,188,139) against `darkSeaGreen` (143,188,143),
/// and `ltGoldenrodYellow` is (250,250,120) against `lightGoldenrodYellow`
/// (250,250,210).
#[test]
fn preset_spellings_agree() {
    let get = |n: &str| PresetColor::from_xml(n).unwrap().rgb();
    for p in PresetColor::ALL {
        let name = p.as_xml();
        let long = if let Some(s) = name.strip_prefix("dk") {
            format!("dark{s}")
        } else if let Some(s) = name.strip_prefix("lt") {
            format!("light{s}")
        } else if let Some(s) = name.strip_prefix("med").filter(|s| !s.starts_with("ium")) {
            format!("medium{s}")
        } else {
            continue;
        };
        if name != "dkSeaGreen" && name != "ltGoldenrodYellow" {
            assert_eq!(p.rgb(), get(&long), "{name}");
        }
    }
    for p in PresetColor::ALL {
        if p.as_xml().contains("Grey") || p.as_xml() == "grey" {
            let gray = p.as_xml().replace("Grey", "Gray").replace("grey", "gray");
            assert_eq!(p.rgb(), get(&gray), "{gray}");
        }
    }
}

fn spec_values(doc: &str, st: &str) -> Vec<String> {
    let mut inside = false;
    let mut out = Vec::new();
    for line in doc.lines() {
        if line.starts_with("## ") {
            inside = line == format!("## `a:{st}`");
        } else if inside && let Some(v) = line.strip_prefix("- `").and_then(|l| l.strip_suffix('`'))
        {
            out.push(v.to_owned());
        }
    }
    out
}

macro_rules! every_spec_value {
    ($doc:expr, $st:literal, $ty:ty) => {{
        let wanted = spec_values($doc, $st);
        assert!(!wanted.is_empty(), $st);
        assert_eq!(wanted.len(), <$ty>::ALL.len(), $st);
        for v in wanted {
            let got = <$ty>::from_xml(&v).unwrap_or_else(|| panic!("{} {}", $st, v));
            assert_eq!(got.as_xml(), v);
        }
        assert_eq!(<$ty>::from_xml("notAValue"), None);
    }};
}

/// Against the generated checklist, so the list comes from the schema, not us.
#[test]
fn every_enum_value_in_the_spec_rows_is_handled() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/spec/");
    let color = std::fs::read_to_string(format!("{root}COLOR.md")).unwrap();
    let theme = std::fs::read_to_string(format!("{root}THEME.md")).unwrap();
    every_spec_value!(&color, "ST_SystemColorVal", SystemColor);
    every_spec_value!(&color, "ST_SchemeColorVal", SchemeColor);
    every_spec_value!(&color, "ST_PresetColorVal", PresetColor);
    every_spec_value!(&theme, "ST_ColorSchemeIndex", ColorSchemeIndex);
    every_spec_value!(&theme, "ST_FontCollectionIndex", FontCollectionIndex);
    every_spec_value!(&theme, "ST_OnOffStyleType", OnOffStyle);
    // EG_ColorTransform: 28 element rows.
    let rows: Vec<&str> = color
        .split("## `a:EG_ColorTransform`")
        .nth(1)
        .unwrap()
        .split("## ")
        .next()
        .unwrap()
        .lines()
        .filter_map(|l| l.strip_prefix("| element | `"))
        .filter_map(|l| l.split('`').next())
        .collect();
    assert_eq!(rows.len(), 28);
    assert_eq!(K::ALL.len(), 28);
    for name in rows {
        assert_eq!(K::from_xml(name).unwrap().as_xml(), name);
    }
}

fn full_scheme() -> ColorScheme {
    let mut s = ColorScheme::default();
    for (i, slot) in ColorSchemeIndex::ALL.iter().enumerate() {
        s.slots[*slot as usize] = Some(Color::srgb(i as u8 + 1, 0, 0));
    }
    s
}

fn office_map() -> ColorMap {
    let mut m = ColorMap::default();
    for slot in ColorMapSlot::ALL {
        let to = match slot {
            ColorMapSlot::Bg1 => "lt1",
            ColorMapSlot::Tx1 => "dk1",
            ColorMapSlot::Bg2 => "lt2",
            ColorMapSlot::Tx2 => "dk2",
            other => other.as_xml(),
        };
        m.slots[*slot as usize] = ColorSchemeIndex::from_xml(to);
    }
    m
}

fn scheme(s: SchemeColor) -> Color {
    Color::new(ColorValue::Scheme(s))
}

#[test]
fn scheme_colours_go_through_the_map_and_dk_lt_do_not() {
    let s = full_scheme();
    let mut map = office_map();
    map.slots[ColorMapSlot::Bg1 as usize] = Some(ColorSchemeIndex::Dk1);
    let ctx = ResolveContext {
        scheme: Some(&s),
        map: Some(&map),
        ..Default::default()
    };
    let r = |c: SchemeColor| resolve(&scheme(c), &ctx).unwrap().rgb.r;
    assert_eq!(r(SchemeColor::Bg1), 1, "bg1 is mapped to dk1");
    assert_eq!(r(SchemeColor::Lt1), 2, "lt1 skips the map");
    assert_eq!(r(SchemeColor::Tx2), 3);
    assert_eq!(r(SchemeColor::Accent6), 10);
    assert_eq!(r(SchemeColor::FolHlink), 12);
}

#[test]
fn every_scheme_value_resolves() {
    let s = full_scheme();
    let map = office_map();
    let ph = Color::srgb(9, 9, 9);
    let ctx = ResolveContext {
        scheme: Some(&s),
        map: Some(&map),
        placeholder: Some(&ph),
        ..Default::default()
    };
    for v in SchemeColor::ALL {
        resolve(&scheme(*v), &ctx).unwrap_or_else(|e| panic!("{v:?}: {e}"));
    }
}

#[test]
fn absent_is_an_error_not_a_default() {
    let c = Default::default();
    assert_eq!(
        resolve(&scheme(SchemeColor::Bg1), &c),
        Err(ResolveError::Unmapped(ColorMapSlot::Bg1))
    );
    assert_eq!(
        resolve(&scheme(SchemeColor::Dk1), &c),
        Err(ResolveError::MissingSchemeSlot(ColorSchemeIndex::Dk1))
    );
    assert_eq!(
        resolve(&scheme(SchemeColor::PhClr), &c),
        Err(ResolveError::NoPlaceholderColor)
    );
}

#[test]
fn placeholder_colour_carries_its_transforms_into_the_style() {
    let s = full_scheme();
    let ph = with(hex("4472C4"), &[(K::Alpha, 50_000)]);
    let ctx = ResolveContext {
        scheme: Some(&s),
        placeholder: Some(&ph),
        ..Default::default()
    };
    let style = with(scheme(SchemeColor::PhClr), &[(K::Shade, 50_000)]);
    let got = resolve(&style, &ctx).unwrap();
    assert_eq!((got.rgb.to_hex().as_str(), got.alpha), ("2F528F", 50_000));
}

#[test]
fn cycles_are_refused() {
    let mut s = full_scheme();
    s.slots[ColorSchemeIndex::Dk1 as usize] = Some(scheme(SchemeColor::Lt1));
    s.slots[ColorSchemeIndex::Lt1 as usize] = Some(scheme(SchemeColor::Dk1));
    let ctx = ResolveContext {
        scheme: Some(&s),
        ..Default::default()
    };
    assert_eq!(
        resolve(&scheme(SchemeColor::Dk1), &ctx),
        Err(ResolveError::Cycle)
    );

    let ph = scheme(SchemeColor::PhClr);
    let ctx = ResolveContext {
        placeholder: Some(&ph),
        ..Default::default()
    };
    assert_eq!(resolve(&ph, &ctx), Err(ResolveError::Cycle));
}

#[test]
fn a_theme_slot_may_refer_to_another_and_transforms_compose() {
    let mut s = full_scheme();
    s.slots[ColorSchemeIndex::Dk1 as usize] = Some(hex("FFFFFF"));
    s.slots[ColorSchemeIndex::Dk2 as usize] =
        Some(with(scheme(SchemeColor::Dk1), &[(K::Shade, 50_000)]));
    let ctx = ResolveContext {
        scheme: Some(&s),
        ..Default::default()
    };
    let got = resolve(&with(scheme(SchemeColor::Dk2), &[(K::Alpha, 1)]), &ctx).unwrap();
    assert_eq!((got.rgb.to_hex().as_str(), got.alpha), ("BCBCBC", 1));
}

#[test]
fn system_colours_prefer_host_then_last_then_fallback() {
    let c = |last| {
        Color::new(ColorValue::System {
            val: SystemColor::WindowText,
            last,
        })
    };
    let last = Some(Rgb::new(1, 2, 3));
    assert_eq!(out(&c(last)).0, "010203");
    assert_eq!(out(&c(None)).0, "000000");
    let ctx = ResolveContext {
        system_palette: Some(|_| Some(Rgb::new(9, 9, 9))),
        ..Default::default()
    };
    assert_eq!(resolve(&c(last), &ctx).unwrap().rgb, Rgb::new(9, 9, 9));
    let none = ResolveContext {
        system_palette: Some(|_| None),
        ..Default::default()
    };
    assert_eq!(resolve(&c(last), &none).unwrap().rgb, Rgb::new(1, 2, 3));
}

#[test]
fn the_cap_and_ranges_are_enforced() {
    let mut c = hex("808080");
    c.transforms = vec![ColorTransform::new(K::LumMod, 99_000); MAX_COLOR_TRANSFORMS];
    assert!(resolve(&c, &Default::default()).is_ok());
    c.transforms.push(ColorTransform::new(K::Comp, 0));
    assert_eq!(
        resolve(&c, &Default::default()),
        Err(ResolveError::TooManyTransforms)
    );
    for (k, v) in [
        (K::Tint, 100_001),
        (K::Shade, -1),
        (K::Alpha, 100_001),
        (K::AlphaOff, 100_001),
        (K::AlphaMod, -1),
        (K::HueMod, -1),
        (K::Hue, 21_600_000),
        (K::Hue, -1),
    ] {
        assert_eq!(
            resolve(&with(hex("000000"), &[(k, v)]), &Default::default()),
            Err(ResolveError::ValueOutOfRange),
            "{k:?} {v}"
        );
    }
    let bad_hue = Color::new(ColorValue::Hsl {
        hue: Angle(21_600_000),
        sat: Percentage(0),
        lum: Percentage(0),
    });
    assert_eq!(
        resolve(&bad_hue, &Default::default()),
        Err(ResolveError::ValueOutOfRange)
    );
}

/// Extreme integers, every transform: in range, no panic.
#[test]
fn extreme_values_never_panic() {
    for &kind in K::ALL {
        for &v in &[i32::MIN, -PERCENT_100, -1, 0, 1, PERCENT_100, i32::MAX] {
            let c = with(hex("4472C4"), &[(kind, v)]);
            match resolve(&c, &Default::default()) {
                Ok(r) => assert!(r.alpha <= 100_000),
                Err(e) => assert_eq!(e, ResolveError::ValueOutOfRange),
            }
        }
    }
    let wild = Color::new(ColorValue::ScRgb {
        r: Percentage(i32::MAX),
        g: Percentage(i32::MIN),
        b: Percentage(0),
    });
    let c = with(wild, &[(K::Tint, 50_000), (K::Lum, i32::MAX)]);
    assert!(resolve(&c, &Default::default()).is_ok());
}

/// A digest of a grid over every transform. Same bits on every platform, or this
/// fails on whichever differs: floats must never reach an answer through libm.
#[test]
fn results_are_deterministic() {
    let mut digest: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |b: u8| {
        digest ^= u64::from(b);
        digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
    };
    for base in ["000000", "FFFFFF", "4472C4", "ED7D31", "123456", "808081"] {
        for &kind in K::ALL {
            for v in [0, 1, 12_345, 50_000, 99_999, 100_000] {
                let c = with(
                    hex(base),
                    &[(kind, v), (K::Shade, 77_777), (K::SatMod, 123_456)],
                );
                let r = resolve(&c, &Default::default()).unwrap();
                feed(r.rgb.r);
                feed(r.rgb.g);
                feed(r.rgb.b);
                r.alpha.to_le_bytes().into_iter().for_each(&mut feed);
            }
        }
    }
    assert_eq!(digest, 8_230_097_184_434_426_797);
}

#[test]
fn percentage_forms() {
    let p = |s| parse_percentage(s).map(|l| (l.value, l.rounded));
    assert_eq!(p("50000"), Some((50_000, false)));
    assert_eq!(p("-20000"), Some((-20_000, false)));
    assert_eq!(p("+5"), Some((5, false)));
    assert_eq!(p("50%"), Some((50_000, false)));
    assert_eq!(p("-20%"), Some((-20_000, false)));
    assert_eq!(p("12.5%"), Some((12_500, false)));
    assert_eq!(p(" 1.234% "), Some((1_234, false)));
    assert_eq!(p("1.2345%"), Some((1_235, true)));
    assert_eq!(p("1.23400000%"), Some((1_234, false)));
    assert_eq!(p("2147483.647%"), Some((i32::MAX, false)));
    for bad in [
        "",
        "%",
        "-%",
        "abc",
        "1.%",
        ".5%",
        "1e3",
        "1.5",
        "50%%",
        "--5%",
        "+5%",
        "2147483648",
        "2147483.648%",
        "99999999999999999999999%",
        "٣%",
    ] {
        assert_eq!(p(bad), None, "{bad:?}");
    }
    for v in [0, 1, 10, 100, 1000, 12_345, -1, -999, 100_000, i32::MAX] {
        let text = format_percentage(v);
        assert_eq!(parse_percentage(&text).unwrap().value, v, "{text}");
    }
    assert_eq!(format_percentage(50_000), "50%");
    assert_eq!(format_percentage(-12_500), "-12.5%");
}

#[test]
fn hex_forms() {
    assert_eq!(Rgb::from_hex("4472c4"), Some(Rgb::new(0x44, 0x72, 0xC4)));
    for bad in [
        "", "12345", "1234567", "GGGGGG", "#12345", "12 456", "é12345",
    ] {
        assert_eq!(Rgb::from_hex(bad), None, "{bad:?}");
    }
}

#[test]
fn colour_map_chain() {
    let master = office_map();
    let mut dark = office_map();
    dark.slots[ColorMapSlot::Bg1 as usize] = Some(ColorSchemeIndex::Dk1);
    let mut inverted = office_map();
    inverted.slots[ColorMapSlot::Tx1 as usize] = Some(ColorSchemeIndex::Lt1);
    let layout = ColorMapOverride::Override(dark.clone());
    let slide = ColorMapOverride::Override(inverted.clone());
    let inherit = ColorMapOverride::Master;
    let m = Some(&master);
    assert_eq!(effective_color_map(m, None, None), m);
    assert_eq!(effective_color_map(m, Some(&inherit), Some(&inherit)), m);
    assert_eq!(effective_color_map(m, None, Some(&slide)), Some(&inverted));
    assert_eq!(
        effective_color_map(m, Some(&layout), Some(&slide)),
        Some(&inverted)
    );
    // Open under BEH-04: see `effective_color_map`.
    assert_eq!(
        effective_color_map(m, Some(&layout), Some(&inherit)),
        Some(&dark)
    );
    assert_eq!(effective_color_map(None, None, None), None);
}

#[test]
fn override_part_scheme_replaces_the_themes() {
    let theme = full_scheme();
    let mut ours = ThemeOverride::default();
    assert_eq!(
        effective_color_scheme(Some(&theme), Some(&ours)),
        Some(&theme)
    );
    let other = ColorScheme {
        name: Some("o".to_owned()),
        ..Default::default()
    };
    ours.color_scheme = Some(other.clone());
    assert_eq!(
        effective_color_scheme(Some(&theme), Some(&ours)),
        Some(&other)
    );
    assert_eq!(effective_color_scheme(Some(&theme), None), Some(&theme));
}

#[test]
fn style_matrix_indexes() {
    let scheme = FormatScheme {
        fills: Some(vec![StyleFill::None, StyleFill::Group]),
        background_fills: Some(vec![StyleFill::Picture]),
        ..Default::default()
    };
    assert_eq!(scheme.fill(0), None);
    assert_eq!(scheme.fill(1000), None);
    assert_eq!(scheme.fill(1), Some(&StyleFill::None));
    assert_eq!(scheme.fill(2), Some(&StyleFill::Group));
    assert_eq!(scheme.fill(3), None);
    assert_eq!(scheme.fill(1001), Some(&StyleFill::Picture));
    assert_eq!(scheme.fill(u32::MAX), None);
    assert_eq!(scheme.line(0), None);
    assert_eq!(scheme.effect(u32::MAX), None);
}
