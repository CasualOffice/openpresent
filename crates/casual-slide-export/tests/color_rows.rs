//! Every colour value and transform the checklist's `COLOR` area lists writes and reads back.
//! Reads the rows from `docs/spec/checklist.tsv`, so a value added there is held here too.

use std::path::Path;

use casual_slide_export::color::set_color;
use casual_slide_import::color::{Findings, read_color_choice};
use casual_slide_model::color::{
    Color, ColorTransform, ColorValue, Percentage, PresetColor, Rgb, SchemeColor, SystemColor,
    TransformKind,
};
use casual_slide_ooxml::{OoxmlLimits, XmlTree};

const AREA: &str = "COLOR";
const A: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";

/// The value rows of a simple type in the `COLOR` area.
fn values(simple: &str) -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/checklist.tsv");
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter_map(|line| {
            let cols: Vec<&str> = line.split('\t').collect();
            (cols.get(1) == Some(&AREA)
                && cols.get(2) == Some(&simple)
                && cols.get(3) == Some(&"value"))
            .then(|| cols[4].trim_start_matches('=').to_owned())
        })
        .collect()
}

fn holder() -> XmlTree {
    let xml = format!(r#"<a:solidFill xmlns:a="{A}"/>"#);
    XmlTree::parse(xml.as_bytes(), "fill", &OoxmlLimits::default()).unwrap()
}

fn round_trip(color: &Color) -> Color {
    let mut tree = holder();
    let root = tree.root();
    assert!(set_color(&mut tree, root, color).unwrap(), "{color:?}");
    let bytes = tree.to_bytes();
    let back = XmlTree::parse(&bytes, "fill", &OoxmlLimits::default()).unwrap();
    let mut findings = Findings::default();
    let read = read_color_choice(&back, back.root(), &mut findings).unwrap();
    assert_eq!(findings, Findings::default(), "{color:?}");
    read
}

fn plain(value: ColorValue) -> Color {
    Color {
        value,
        transforms: Vec::new(),
    }
}

#[test]
fn every_preset_scheme_and_system_value_writes_and_reads_back() {
    let preset = values("ST_PresetColorVal");
    assert!(preset.len() > 100, "{}", preset.len());
    for token in preset {
        let color = plain(ColorValue::Preset(PresetColor::from_xml(&token).unwrap()));
        assert_eq!(round_trip(&color), color, "{token}");
    }
    let scheme = values("ST_SchemeColorVal");
    assert!(scheme.len() > 10);
    for token in scheme {
        let color = plain(ColorValue::Scheme(SchemeColor::from_xml(&token).unwrap()));
        assert_eq!(round_trip(&color), color, "{token}");
    }
    let system = values("ST_SystemColorVal");
    assert!(system.len() > 10);
    for token in system {
        let color = plain(ColorValue::System {
            val: SystemColor::from_xml(&token).unwrap(),
            last: Some(Rgb::new(1, 2, 3)),
        });
        assert_eq!(round_trip(&color), color, "{token}");
    }
}

#[test]
fn each_colour_element_writes_and_reads_back() {
    for color in [
        plain(ColorValue::Srgb(Rgb::new(1, 2, 3))),
        plain(ColorValue::ScRgb {
            r: Percentage(1000),
            g: Percentage(2000),
            b: Percentage(3000),
        }),
        plain(ColorValue::Hsl {
            hue: casual_slide_model::color::Angle(60_000),
            sat: Percentage(1000),
            lum: Percentage(2000),
        }),
    ] {
        assert_eq!(round_trip(&color), color);
    }
}

#[test]
fn each_transform_writes_and_reads_back() {
    for kind in TransformKind::ALL {
        let value = if kind.domain() == casual_slide_model::color::ValueDomain::None {
            0
        } else {
            1000
        };
        let color = Color {
            value: ColorValue::Srgb(Rgb::new(1, 2, 3)),
            transforms: vec![ColorTransform::new(*kind, value)],
        };
        assert_eq!(round_trip(&color), color, "{}", kind.as_xml());
    }
}
