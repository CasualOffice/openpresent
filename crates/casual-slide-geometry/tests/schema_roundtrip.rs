//! Two routes from the definition files to a result must agree exactly: Python
//! generator to static table, and this file's reader through `RawGeometry::compile`.
//! Each checks the other, and together they catch a wrong table or a wrong compiler
//! on all 228 definitions. The reader is test-only and handles just these two files.

use std::collections::BTreeMap;
use std::path::Path;

use casual_slide_geometry::{
    Adjust, Extents, FillMode, GeometryDef, RawAxis, RawCommand, RawGeometry, RawGuide, RawHandle,
    RawPath, RawPoint, RawRect, RawSite, preset_shape, text_warp,
};

#[derive(Debug, Default)]
struct Node {
    name: String,
    attrs: BTreeMap<String, String>,
    children: Vec<Node>,
}

impl Node {
    fn attr(&self, key: &str) -> String {
        self.attrs.get(key).cloned().unwrap_or_default()
    }

    fn opt(&self, key: &str) -> Option<String> {
        self.attrs.get(key).cloned()
    }

    fn child(&self, name: &str) -> Option<&Node> {
        self.children.iter().find(|c| c.name == name)
    }
}

/// Elements and double-quoted attributes. Nothing else occurs in these files.
fn parse(text: &str) -> Node {
    let mut stack = vec![Node::default()];
    let mut rest = text;
    while let Some(open) = rest.find('<') {
        rest = &rest[open + 1..];
        let close = rest.find('>').unwrap();
        let tag = &rest[..close];
        rest = &rest[close + 1..];
        if tag.starts_with('?') {
            continue;
        }
        if tag.starts_with('/') {
            let done = stack.pop().unwrap();
            stack.last_mut().unwrap().children.push(done);
            continue;
        }
        let self_closed = tag.ends_with('/');
        let tag = tag.trim_end_matches('/').trim();
        let (name, mut attrs_text) = tag.split_once(char::is_whitespace).unwrap_or((tag, ""));
        let mut node = Node {
            name: name.to_owned(),
            ..Node::default()
        };
        while let Some((key, value)) = attrs_text.split_once("=\"") {
            let end = value.find('"').unwrap();
            let (key, value_text) = (key.trim(), &value[..end]);
            // The only entity these files use.
            assert!(!value_text.contains('&') || value_text.contains("&lt;"));
            node.attrs.insert(key.to_owned(), value_text.to_owned());
            attrs_text = &value[end + 1..];
        }
        if self_closed {
            stack.last_mut().unwrap().children.push(node);
        } else {
            stack.push(node);
        }
    }
    let mut root = stack.pop().unwrap();
    root.children.pop().unwrap()
}

fn guides(node: Option<&Node>) -> Vec<RawGuide> {
    node.map(|list| {
        list.children
            .iter()
            .map(|g| RawGuide {
                name: g.attr("name"),
                fmla: g.attr("fmla"),
            })
            .collect()
    })
    .unwrap_or_default()
}

fn point(node: &Node) -> RawPoint {
    RawPoint {
        x: node.attr("x"),
        y: node.attr("y"),
    }
}

fn axis(node: &Node, reference: &str, min: &str, max: &str) -> RawAxis {
    RawAxis {
        guide: node.opt(reference),
        min: node.opt(min),
        max: node.opt(max),
    }
}

fn to_raw(shape: &Node) -> RawGeometry {
    let mut raw = RawGeometry {
        av: guides(shape.child("avLst")),
        gd: guides(shape.child("gdLst")),
        ..RawGeometry::default()
    };
    if let Some(list) = shape.child("ahLst") {
        for handle in &list.children {
            let pos = point(&handle.children[0]);
            raw.handles.push(if handle.name == "ahXY" {
                RawHandle::Xy {
                    x: axis(handle, "gdRefX", "minX", "maxX"),
                    y: axis(handle, "gdRefY", "minY", "maxY"),
                    pos,
                }
            } else {
                RawHandle::Polar {
                    r: axis(handle, "gdRefR", "minR", "maxR"),
                    ang: axis(handle, "gdRefAng", "minAng", "maxAng"),
                    pos,
                }
            });
        }
    }
    if let Some(list) = shape.child("cxnLst") {
        for site in &list.children {
            raw.sites.push(RawSite {
                ang: site.attr("ang"),
                pos: point(&site.children[0]),
            });
        }
    }
    if let Some(rect) = shape.child("rect") {
        raw.rect = Some(RawRect {
            l: rect.attr("l"),
            t: rect.attr("t"),
            r: rect.attr("r"),
            b: rect.attr("b"),
        });
    }
    for path in &shape.child("pathLst").unwrap().children {
        let mut out = RawPath {
            w: path.opt("w").map_or(0, |v| v.parse().unwrap()),
            h: path.opt("h").map_or(0, |v| v.parse().unwrap()),
            fill: path
                .opt("fill")
                .map_or(FillMode::Norm, |v| FillMode::parse(&v).unwrap()),
            stroke: path.attr("stroke") != "false",
            extrusion_ok: path.attr("extrusionOk") != "false",
            commands: Vec::new(),
        };
        for command in &path.children {
            let p = |i: usize| point(&command.children[i]);
            out.commands.push(match command.name.as_str() {
                "moveTo" => RawCommand::MoveTo(p(0)),
                "lnTo" => RawCommand::LineTo(p(0)),
                "quadBezTo" => RawCommand::QuadBezTo(p(0), p(1)),
                "cubicBezTo" => RawCommand::CubicBezTo(p(0), p(1), p(2)),
                "close" => RawCommand::Close,
                "arcTo" => RawCommand::ArcTo {
                    wr: command.attr("wR"),
                    hr: command.attr("hR"),
                    st_ang: command.attr("stAng"),
                    sw_ang: command.attr("swAng"),
                },
                other => panic!("{other}"),
            });
        }
        raw.paths.push(out);
    }
    raw
}

/// Applies the generator's documented corrections, so that what remains to compare
/// is everything the corrections do not touch.
fn apply_corrections(name: &str, raw: &mut RawGeometry) {
    if name == "curvedLeftArrow" {
        assert_eq!(raw.sites[2].ang, "cd3", "GEO-02b no longer applies");
        raw.sites[2].ang = "cd2".to_owned();
    }
    if name == "gear6" || name == "gear9" {
        let mut fixed = 0;
        for guide in raw.gd.iter_mut().filter(|g| g.fmla.starts_with("+/ r 0 ")) {
            guide.fmla = guide.fmla.replace("+/ r 0 ", "+- r 0 ");
            fixed += 1;
        }
        assert_eq!(fixed, if name == "gear6" { 1 } else { 4 }, "GEO-02e");
    }
    if name == "pie" {
        let rect = raw.rect.as_mut().unwrap();
        assert_eq!((rect.t.as_str(), rect.r.as_str()), ("ir", "it"), "GEO-02d");
        std::mem::swap(&mut rect.t, &mut rect.r);
    }
}

/// The definitions with surplus operands, each reported once per formula: GEO-02c.
fn expected_diagnostics(name: &str) -> usize {
    match name {
        "circularArrow" | "leftCircularArrow" => 2,
        "leftRightCircularArrow" => 4,
        _ => 0,
    }
}

fn compare(name: &str, shape: &Node, table: &GeometryDef) {
    let mut raw = to_raw(shape);
    apply_corrections(name, &mut raw);
    let compiled = raw.compile().unwrap();
    assert_eq!(
        compiled.diagnostics.total(),
        expected_diagnostics(name),
        "{name}: {:?}",
        compiled.diagnostics
    );
    for (w, h) in [
        (914_400, 914_400),
        (3_000_000, 500_000),
        (333, 7_777_777),
        (0, 0),
    ] {
        for adjust in [None, Some(12_345)] {
            let adjusts: Vec<Adjust<'_>> = adjust
                .into_iter()
                .flat_map(|value| {
                    table
                        .av
                        .iter()
                        .map(move |g| Adjust::val(g.name.as_ref(), value))
                })
                .collect();
            let (a, b) = (
                table.evaluate(Extents::new(w, h), &adjusts),
                compiled.def.evaluate(Extents::new(w, h), &adjusts),
            );
            assert_eq!(a.paths, b.paths, "{name} at {w}x{h}");
            assert_eq!(a.text_rect, b.text_rect, "{name} at {w}x{h}");
            assert_eq!(a.connection_sites, b.connection_sites, "{name} at {w}x{h}");
            assert_eq!(a.adjust_handles, b.adjust_handles, "{name} at {w}x{h}");
            assert_eq!(a.fingerprint(), b.fingerprint(), "{name} at {w}x{h}");
        }
    }
}

fn definitions(file: &str) -> Node {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas/geometry")
        .join(file);
    parse(&std::fs::read_to_string(path).unwrap())
}

#[test]
fn shape_table_matches_the_definitions_file() {
    let root = definitions("presetShapeDefinitions.xml");
    assert_eq!(
        root.children.len(),
        187,
        "the file's own count, duplicate included"
    );
    let mut seen = 0;
    for shape in &root.children {
        compare(&shape.name, shape, preset_shape(&shape.name).unwrap());
        seen += 1;
    }
    assert_eq!(seen, 187);
    assert!(
        root.children.iter().all(|s| s.name != "upArrow"),
        "GEO-01 no longer applies"
    );
}

#[test]
fn warp_table_matches_the_definitions_file() {
    let root = definitions("presetTextWarpDefinitions.xml");
    assert_eq!(
        root.children.len(),
        40,
        "GEO-02a: 40 here and 41 in the schema"
    );
    for shape in &root.children {
        compare(&shape.name, shape, text_warp(&shape.name).unwrap());
    }
}

/// GEO-01's evidence, kept executable: the two copies are the same text.
#[test]
fn the_duplicate_up_down_arrows_are_identical() {
    let root = definitions("presetShapeDefinitions.xml");
    let copies: Vec<&Node> = root
        .children
        .iter()
        .filter(|s| s.name == "upDownArrow")
        .collect();
    assert_eq!(copies.len(), 2);
    assert_eq!(format!("{:?}", copies[0]), format!("{:?}", copies[1]));
}
