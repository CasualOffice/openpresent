//! The generated table: completeness against the schema, GEO-01 and GEO-02a to GEO-02e, sanity of
//! all 187 + 41, hand-computed goldens, reproducibility.

use std::path::{Path, PathBuf};
use std::process::Command;

use casual_slide_geometry::{
    Adjust, AdjustHandle, Arc, ConnectionSite, Extents, FillMode, GeometryDef, Point,
    ResolvedGeometry, ResolvedPath, Segment, preset_shape, preset_shapes, table_fingerprint,
    text_warp, text_warps,
};

const CD4: f64 = 5_400_000.0;
const CD2: f64 = 10_800_000.0;
const C3D4: f64 = 16_200_000.0;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Enumeration values of one simple type, read straight from the schema text.
fn xsd_values(simple_type: &str) -> Vec<String> {
    let xsd = std::fs::read_to_string(repo().join("schemas/ooxml/dml-main.xsd")).unwrap();
    let open = format!("<xsd:simpleType name=\"{simple_type}\"");
    let start = xsd.find(&open).unwrap();
    let body = &xsd[start..start + xsd[start..].find("</xsd:simpleType>").unwrap()];
    body.split("<xsd:enumeration value=\"")
        .skip(1)
        .map(|rest| rest[..rest.find('"').unwrap()].to_owned())
        .collect()
}

fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names
}

#[test]
fn table_equals_st_shape_type() {
    let want = sorted(xsd_values("ST_ShapeType"));
    assert_eq!(want.len(), 187);
    let have: Vec<String> = preset_shapes().iter().map(|d| d.name.to_string()).collect();
    assert_eq!(have, want);
}

#[test]
fn table_equals_st_text_shape_type() {
    let want = sorted(xsd_values("ST_TextShapeType"));
    assert_eq!(want.len(), 41);
    let have: Vec<String> = text_warps().iter().map(|d| d.name.to_string()).collect();
    assert_eq!(have, want);
}

#[test]
fn lookup_finds_each_name_and_nothing_else() {
    for def in preset_shapes() {
        assert_eq!(preset_shape(&def.name), Some(def));
    }
    for def in text_warps() {
        assert_eq!(text_warp(&def.name), Some(def));
    }
    assert_eq!(preset_shape("RECT"), None);
    assert_eq!(preset_shape(""), None);
    assert_eq!(preset_shape("textPlain"), None);
    assert_eq!(text_warp("rect"), None);
}

#[test]
fn checked_in_table_is_current() {
    let output = Command::new("python3")
        .arg(repo().join("tools/geometry-presets/generate.py"))
        .arg("--check")
        .output()
        .expect("python3 is required to verify the generated table");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn table_is_built_from_data_only() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let dependencies = manifest.split("[dependencies]").nth(1).unwrap();
    let dependencies = dependencies.split("[lints]").next().unwrap();
    assert_eq!(dependencies.trim(), "", "no runtime dependencies");
}

// ---------------------------------------------------------------- every preset

const SIZES: [(i64, i64); 10] = [
    (914_400, 914_400),
    (12_192_000, 6_858_000),
    (3_000_000, 500_000),
    (500_000, 3_000_000),
    (1, 1),
    (1, 1_000_000),
    (1_000_000, 1),
    (0, 0),
    (0, 1_000_000),
    (27_273_042_316_900, 27_273_042_316_900),
];

fn points_of(path: &ResolvedPath) -> Vec<Point> {
    let mut out = Vec::new();
    for segment in &path.segments {
        match *segment {
            Segment::MoveTo(p) | Segment::LineTo(p) => out.push(p),
            Segment::QuadTo { ctrl, end } => out.extend([ctrl, end]),
            Segment::CubicTo { c1, c2, end } => out.extend([c1, c2, end]),
            Segment::ArcTo(arc) => out.extend([arc.start, arc.end]),
            Segment::Close => {}
        }
    }
    out
}

/// The most any preset reaches from the origin at default adjusts, in multiples of
/// the longer side. The largest is a Bezier control point of `heart` at 1.52, and
/// control points are not on the outline. Anything past this is a fault.
const REACH: f64 = 1.6;

fn check_bounds(def: &GeometryDef, w: i64, h: i64) {
    let g = def.evaluate(Extents::new(w, h), &[]);
    assert!(
        g.diagnostics.is_empty(),
        "{}: {:?}",
        def.name,
        g.diagnostics
    );
    let limit = REACH * w.max(h) as f64 + 1e-6;
    let check = |label: &str, p: Point| {
        assert!(p.x.is_finite() && p.y.is_finite(), "{} {label}", def.name);
        assert!(
            p.x.abs() <= limit && p.y.abs() <= limit,
            "{} at {w}x{h}: {label} {p:?} beyond {limit}",
            def.name
        );
    };
    for path in &g.paths {
        let scaled = path.scaled_to(g.extents);
        for p in points_of(&scaled) {
            check("point", p);
        }
    }
    for site in &g.connection_sites {
        check("site", site.pos);
        assert!(site.ang.is_finite());
    }
    let t = g.text_rect;
    check("rect", Point { x: t.l, y: t.t });
    check("rect", Point { x: t.r, y: t.b });
    for handle in &g.adjust_handles {
        let (AdjustHandle::Xy { pos, .. } | AdjustHandle::Polar { pos, .. }) = handle;
        check("handle", *pos);
    }
}

#[test]
fn all_shapes_are_finite_and_in_bounds_at_every_size() {
    for def in preset_shapes() {
        for (w, h) in SIZES {
            check_bounds(def, w, h);
        }
    }
}

#[test]
fn all_warps_are_finite_and_in_bounds_at_every_size() {
    for def in text_warps() {
        for (w, h) in SIZES {
            check_bounds(def, w, h);
        }
    }
}

/// Each real preset draws something, and its outline is not collapsed.
#[test]
fn every_definition_draws_at_a_normal_size() {
    for def in preset_shapes().iter().chain(text_warps()) {
        let g = def.evaluate(Extents::new(1_200_000, 800_000), &[]);
        if def.name == "textNoShape" {
            assert!(g.paths.is_empty());
            continue;
        }
        assert!(!g.paths.is_empty(), "{}", def.name);
        for path in &g.paths {
            assert!(
                matches!(path.segments[0], Segment::MoveTo(_)),
                "{}",
                def.name
            );
        }
        let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
        let (mut min_y, mut max_y) = (f64::MAX, f64::MIN);
        for path in &g.paths {
            for p in points_of(&path.scaled_to(g.extents)) {
                min_x = min_x.min(p.x);
                max_x = max_x.max(p.x);
                min_y = min_y.min(p.y);
                max_y = max_y.max(p.y);
            }
        }
        assert!(
            max_x - min_x > 0.0 || max_y - min_y > 0.0,
            "{} is a point",
            def.name
        );
    }
}

/// GEO-02d is the rule this enforces for every shape: the text rectangle is never
/// inside out, here at the default adjusts.
#[test]
fn text_rectangles_are_not_inverted() {
    for def in preset_shapes() {
        for (w, h) in [
            (914_400, 914_400),
            (3_000_000, 500_000),
            (500_000, 3_000_000),
        ] {
            let t = def.evaluate(Extents::new(w, h), &[]).text_rect;
            assert!(t.l <= t.r + 1e-6, "{} at {w}x{h}: {t:?}", def.name);
            assert!(t.t <= t.b + 1e-6, "{} at {w}x{h}: {t:?}", def.name);
        }
    }
}

/// Arcs must join what the preset's own guides expect. `pie` computes where the arc
/// should finish in x2, y2; a parametric reading of the angle would miss on any
/// non-circular box. This is the check on the arc model that matters.
#[test]
fn arc_endpoints_meet_the_guides() {
    for (w, h) in [(2_000_000, 1_000_000), (700_000, 2_300_000)] {
        for (adj1, adj2) in [
            (0, 16_200_000),
            (1_000_000, 7_000_000),
            (15_000_000, 2_000_000),
        ] {
            let g = preset_shape("pie").unwrap().evaluate(
                Extents::new(w, h),
                &[Adjust::val("adj1", adj1), Adjust::val("adj2", adj2)],
            );
            let Segment::ArcTo(arc) = g.paths[0].segments[1] else {
                panic!();
            };
            near(arc.end.x, g.guide("x2").unwrap());
            near(arc.end.y, g.guide("y2").unwrap());
            near(arc.center.x, w as f64 / 2.0);
            near(arc.center.y, h as f64 / 2.0);
        }
    }
}

/// Geometry is homogeneous: scale a shape and each position scales with it, because
/// guides combine lengths with dimensionless ratios. A guide that divides where it
/// should subtract (GEO-02e) breaks this and little else notices, since a wrong
/// coordinate near zero is well within the shape. Paths with their own w/h keep raw
/// coordinates in path units, so those stay put.
#[test]
fn every_position_scales_with_the_shape() {
    const FACTOR: f64 = 10.0;
    let close = |a: Point, b: Point, k: f64, name: &str, what: &str| {
        assert!(
            (a.x * k - b.x).abs() < 1e-3 && (a.y * k - b.y).abs() < 1e-3,
            "{name} {what}: {a:?} x {k} vs {b:?}"
        );
    };
    for def in preset_shapes().iter().chain(text_warps()) {
        let small = def.evaluate(Extents::new(1_200_000, 800_000), &[]);
        let large = def.evaluate(Extents::new(12_000_000, 8_000_000), &[]);
        for (p, q) in small.paths.iter().zip(&large.paths) {
            let k = if p.w > 0 { 1.0 } else { FACTOR };
            for (a, b) in points_of(p).iter().zip(&points_of(q)) {
                close(*a, *b, k, &def.name, "path");
            }
        }
        for (a, b) in small.connection_sites.iter().zip(&large.connection_sites) {
            close(a.pos, b.pos, FACTOR, &def.name, "site");
            assert_eq!(a.ang, b.ang, "{}", def.name);
        }
        let (a, b) = (small.text_rect, large.text_rect);
        close(
            Point { x: a.l, y: a.t },
            Point { x: b.l, y: b.t },
            FACTOR,
            &def.name,
            "rect",
        );
        close(
            Point { x: a.r, y: a.b },
            Point { x: b.r, y: b.b },
            FACTOR,
            &def.name,
            "rect",
        );
    }
}

// ---------------------------------------------------------------- GEO-01 and GEO-02a to GEO-02e

/// Reflection in the horizontal midline.
fn mirrored(p: Point, h: f64) -> Point {
    Point { x: p.x, y: h - p.y }
}

/// Twice the signed area of a polygon. The sign is the winding.
fn signed_area(points: &[Point]) -> f64 {
    let mut sum = 0.0;
    for (i, a) in points.iter().enumerate() {
        let b = points[(i + 1) % points.len()];
        sum += a.x * b.y - b.x * a.y;
    }
    sum
}

/// GEO-01. Pins the override: upArrow is downArrow reflected top to bottom at every
/// adjust and size, as a set of vertices, with the winding of its siblings kept.
#[test]
fn up_arrow_is_the_mirror_of_down_arrow() {
    let up = preset_shape("upArrow").unwrap();
    let down = preset_shape("downArrow").unwrap();
    for (w, h) in [
        (1_000_000, 2_000_000),
        (2_000_000, 1_000_000),
        (914_400, 914_400),
    ] {
        for (adj1, adj2) in [
            (50_000, 50_000),
            (20_000, 80_000),
            (100_000, 0),
            (0, 400_000),
        ] {
            let adjusts = [Adjust::val("adj1", adj1), Adjust::val("adj2", adj2)];
            let (u, d) = (
                up.evaluate(Extents::new(w, h), &adjusts),
                down.evaluate(Extents::new(w, h), &adjusts),
            );
            let hf = h as f64;

            let up_points = points_of(&u.paths[0]);
            let down_points = points_of(&d.paths[0]);
            assert_eq!(up_points.len(), 7);
            for p in &down_points {
                let m = mirrored(*p, hf);
                assert!(
                    up_points
                        .iter()
                        .any(|q| (q.x - m.x).abs() < 1e-6 && (q.y - m.y).abs() < 1e-6),
                    "{w}x{h} adj {adj1},{adj2}: no vertex at {m:?} in {up_points:?}"
                );
            }
            let (a, b) = (signed_area(&up_points), signed_area(&down_points));
            assert!((a.abs() - b.abs()).abs() < 1e-3, "same area");
            assert!(a * b >= 0.0, "same winding as downArrow: {a} vs {b}");

            // Text rectangle: reflected.
            near(u.text_rect.l, d.text_rect.l);
            near(u.text_rect.r, d.text_rect.r);
            near(u.text_rect.t, hf - d.text_rect.b);
            near(u.text_rect.b, hf - d.text_rect.t);

            // Connection sites keep their index: top, left, bottom, right. Each is
            // the reflection of the site on the opposite vertical side.
            assert_eq!(u.connection_sites.len(), 4);
            let mirrored_site = |i: usize| mirrored(d.connection_sites[i].pos, hf);
            for (ui, di) in [(0, 2), (1, 1), (2, 0), (3, 3)] {
                near(u.connection_sites[ui].pos.x, mirrored_site(di).x);
                near(u.connection_sites[ui].pos.y, mirrored_site(di).y);
            }
            let angles: Vec<f64> = u.connection_sites.iter().map(|s| s.ang).collect();
            assert_eq!(angles, [C3D4, CD2, CD4, 0.0]);

            // Handles edit the same guides at the reflected positions.
            for (a, b) in u.adjust_handles.iter().zip(&d.adjust_handles) {
                let (
                    AdjustHandle::Xy {
                        x: ax,
                        y: ay,
                        pos: pa,
                    },
                    AdjustHandle::Xy {
                        x: bx,
                        y: by,
                        pos: pb,
                    },
                ) = (a, b)
                else {
                    panic!();
                };
                assert_eq!(ax, bx);
                assert_eq!(ay, by);
                near(pa.x, pb.x);
                near(pa.y, hf - pb.y);
            }
        }
    }
}

#[test]
fn up_arrow_golden() {
    // 1000000 x 2000000, defaults. ss = 1000000; a1 = a2 = 50000.
    // dy2 = 1000000 * 50000 / 100000 = 500000; y2 = 500000.
    // dx1 = 1000000 * 50000 / 200000 = 250000; x1 = 250000; x2 = 750000.
    // dy1 = x1 * dy2 / wd2 = 250000; y1 = y2 - dy1 = 250000.
    let g = preset_shape("upArrow")
        .unwrap()
        .evaluate(Extents::new(1_000_000, 2_000_000), &[]);
    assert_polyline(
        &g.paths[0],
        &[
            (0.0, 500_000.0),
            (500_000.0, 0.0),
            (1_000_000.0, 500_000.0),
            (750_000.0, 500_000.0),
            (750_000.0, 2_000_000.0),
            (250_000.0, 2_000_000.0),
            (250_000.0, 500_000.0),
        ],
    );
    assert_rect(&g, 250_000.0, 250_000.0, 750_000.0, 2_000_000.0);
    assert_sites(
        &g,
        &[
            (500_000.0, 0.0, C3D4),
            (0.0, 500_000.0, CD2),
            (500_000.0, 2_000_000.0, CD4),
            (1_000_000.0, 500_000.0, 0.0),
        ],
    );
}

/// GEO-01's other half: the duplicate must remain a single, correct upDownArrow.
#[test]
fn up_down_arrow_is_present_once_and_is_not_an_up_arrow() {
    let count = preset_shapes()
        .iter()
        .filter(|d| d.name == "upDownArrow")
        .count();
    assert_eq!(count, 1);
    let up_down = preset_shape("upDownArrow").unwrap();
    assert_ne!(up_down.gd, preset_shape("upArrow").unwrap().gd);
    assert_eq!(
        up_down
            .evaluate(Extents::new(1000, 4000), &[])
            .connection_sites
            .len(),
        8
    );
}

#[test]
fn text_no_shape_exists_and_draws_nothing() {
    let def = text_warp("textNoShape").unwrap();
    assert!(def.paths.is_empty() && def.gd.is_empty());
    let g = def.evaluate(Extents::new(1000, 500), &[]);
    assert_rect(&g, 0.0, 0.0, 1000.0, 500.0);
}

/// GEO-02b: `cd3` in the source is corrected to `cd2`.
#[test]
fn curved_left_arrow_site_faces_left() {
    let g = preset_shape("curvedLeftArrow")
        .unwrap()
        .evaluate(Extents::new(1000, 2000), &[]);
    let site = g.connection_sites[2];
    assert_eq!(site.ang, CD2);
    assert_eq!(site.pos.x, 0.0, "on the left edge");
}

/// GEO-02c: the surplus operand is dropped, leaving `xH - dxB`.
#[test]
fn circular_arrow_surplus_operands_are_dropped() {
    let g = preset_shape("circularArrow")
        .unwrap()
        .evaluate(Extents::new(1_200_000, 800_000), &[]);
    near(
        g.guide("xB").unwrap(),
        g.guide("xH").unwrap() - g.guide("dxB").unwrap(),
    );
    near(
        g.guide("yB").unwrap(),
        g.guide("yH").unwrap() - g.guide("dyB").unwrap(),
    );
}

/// Regression, found by the LibreOffice cross-check: this preset's guides reach the
/// fourth power of the size, and clipping them at 2^62 turned a sweep of 163.26
/// degrees into exactly 90. The expected value was computed independently from the
/// definition file in Python with doubles and agrees with LibreOffice's table.
#[test]
fn circular_arrow_keeps_large_intermediates() {
    let g = preset_shape("circularArrow")
        .unwrap()
        .evaluate(Extents::new(1_200_000, 800_000), &[]);
    assert!(g.guide("q5").unwrap() > 1e21);
    assert!((g.guide("swAng").unwrap() - 9_795_553.510_414).abs() < 1e-3);
    // Angles do not depend on scale, so the largest legal extent, where those
    // guides peak, must give the same sweep as an ordinary size.
    let sweep = |side: i64| {
        let def = preset_shape("circularArrow").unwrap();
        let g = def.evaluate(Extents::new(side, side), &[]);
        (g.guide("swAng").unwrap(), g.guide("q5").unwrap())
    };
    let (ordinary, _) = sweep(914_400);
    let (largest, q5) = sweep(27_273_042_316_900);
    assert!(q5 > 1e50, "{q5}");
    assert!((ordinary - largest).abs() < 1e-3, "{ordinary} vs {largest}");
}

/// GEO-02d: pie's rectangle, by hand. idx = 1000000 cos 45 = 707106.78;
/// idy = 500000 sin 45 = 353553.39.
#[test]
fn pie_text_rectangle_is_corrected() {
    let g = preset_shape("pie")
        .unwrap()
        .evaluate(Extents::new(2_000_000, 1_000_000), &[]);
    assert_rect(
        &g,
        1_000_000.0 - 707_106.781_186_5,
        500_000.0 - 353_553.390_593_3,
        1_000_000.0 + 707_106.781_186_5,
        500_000.0 + 353_553.390_593_3,
    );
}

/// GEO-02e: the gear sites are left-right pairs, reflected in the vertical midline.
#[test]
fn gear_connection_sites_are_mirrored() {
    for (name, pairs) in [
        ("gear6", &[(0, 4), (1, 3)][..]),
        ("gear9", &[(0, 7), (1, 6), (2, 5), (3, 4)][..]),
    ] {
        let g = preset_shape(name)
            .unwrap()
            .evaluate(Extents::new(1_200_000, 800_000), &[]);
        for (left, right) in pairs {
            let (a, b) = (
                g.connection_sites[*left].pos,
                g.connection_sites[*right].pos,
            );
            near(a.x + b.x, 1_200_000.0);
            near(a.y, b.y);
        }
    }
}

/// The six accent callouts have `close` straight after `moveTo`, then a line, which
/// must draw from the point they closed back to.
#[test]
fn accent_callout_line_after_close_starts_from_the_subpath() {
    let g = preset_shape("accentCallout1")
        .unwrap()
        .evaluate(Extents::new(1_200_000, 800_000), &[]);
    let segments = &g.paths[1].segments;
    let x1 = g.guide("x1").unwrap();
    assert_eq!(segments[0], Segment::MoveTo(Point { x: x1, y: 0.0 }));
    assert_eq!(segments[1], Segment::Close);
    assert_eq!(
        segments[2],
        Segment::LineTo(Point {
            x: x1,
            y: 800_000.0
        })
    );
    // An arc issued after a close has to start there too.
    let raw = casual_slide_geometry::RawGeometry {
        paths: vec![casual_slide_geometry::RawPath {
            commands: vec![
                casual_slide_geometry::RawCommand::MoveTo(casual_slide_geometry::RawPoint {
                    x: "10".into(),
                    y: "20".into(),
                }),
                casual_slide_geometry::RawCommand::LineTo(casual_slide_geometry::RawPoint {
                    x: "90".into(),
                    y: "90".into(),
                }),
                casual_slide_geometry::RawCommand::Close,
                casual_slide_geometry::RawCommand::ArcTo {
                    wr: "5".into(),
                    hr: "5".into(),
                    st_ang: "0".into(),
                    sw_ang: "cd2".into(),
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    };
    let def = raw.compile().unwrap().def;
    let out = def.evaluate(Extents::new(100, 100), &[]);
    let Segment::ArcTo(arc) = out.paths[0].segments[3] else {
        panic!();
    };
    assert_eq!(arc.start, Point { x: 10.0, y: 20.0 });
}

// ---------------------------------------------------------------- golden values

fn near(got: f64, want: f64) {
    assert!((got - want).abs() < 1e-4, "got {got}, want {want}");
}

fn assert_rect(g: &ResolvedGeometry<'_>, l: f64, t: f64, r: f64, b: f64) {
    let rect = g.text_rect;
    near(rect.l, l);
    near(rect.t, t);
    near(rect.r, r);
    near(rect.b, b);
}

fn assert_sites(g: &ResolvedGeometry<'_>, want: &[(f64, f64, f64)]) {
    assert_eq!(g.connection_sites.len(), want.len(), "site count");
    for (i, (site, (x, y, ang))) in g.connection_sites.iter().zip(want).enumerate() {
        let ConnectionSite { pos, ang: got } = site;
        assert!(
            (pos.x - x).abs() < 1e-4 && (pos.y - y).abs() < 1e-4 && got == ang,
            "site {i}: {site:?}, want ({x}, {y}, {ang})"
        );
    }
}

/// A move, lines and a final close through these vertices.
fn assert_polyline(path: &ResolvedPath, want: &[(f64, f64)]) {
    assert_eq!(path.segments.len(), want.len() + 1, "{:?}", path.segments);
    for (i, (segment, (x, y))) in path.segments.iter().zip(want).enumerate() {
        let (Segment::MoveTo(p) | Segment::LineTo(p)) = segment else {
            panic!("{segment:?}");
        };
        assert!(
            (p.x - x).abs() < 1e-4 && (p.y - y).abs() < 1e-4,
            "vertex {i}: {p:?}, want ({x}, {y})"
        );
    }
    assert_eq!(path.segments.last(), Some(&Segment::Close));
}

fn arc_at(path: &ResolvedPath, index: usize) -> Arc {
    let Segment::ArcTo(arc) = path.segments[index] else {
        panic!("{:?}", path.segments[index]);
    };
    arc
}

fn assert_arc_end(arc: Arc, x: f64, y: f64) {
    near(arc.end.x, x);
    near(arc.end.y, y);
}

#[test]
fn golden_rect() {
    let g = preset_shape("rect")
        .unwrap()
        .evaluate(Extents::new(1_000_000, 2_000_000), &[]);
    assert_eq!(g.paths.len(), 1);
    let path = &g.paths[0];
    assert_eq!((path.w, path.h), (0, 0));
    assert_eq!(path.fill, FillMode::Norm);
    assert!(path.stroke && path.extrusion_ok);
    assert_polyline(path, &[(0.0, 0.0), (1e6, 0.0), (1e6, 2e6), (0.0, 2e6)]);
    assert_rect(&g, 0.0, 0.0, 1e6, 2e6);
    assert_sites(
        &g,
        &[
            (5e5, 0.0, C3D4),
            (0.0, 1e6, CD2),
            (5e5, 2e6, CD4),
            (1e6, 1e6, 0.0),
        ],
    );
    assert!(g.adjust_handles.is_empty());
}

#[test]
fn golden_ellipse() {
    // 2000000 x 1000000. cos 45 = sin 45 = 0.70710678118654...
    // idx = 1000000 * that = 707106.7811865; idy = 500000 * that = 353553.3905933.
    let (idx, idy) = (707_106.781_186_5, 353_553.390_593_3);
    let (il, ir, it, ib) = (1e6 - idx, 1e6 + idx, 5e5 - idy, 5e5 + idy);
    let g = preset_shape("ellipse")
        .unwrap()
        .evaluate(Extents::new(2_000_000, 1_000_000), &[]);
    let path = &g.paths[0];
    assert_eq!(path.segments[0], Segment::MoveTo(Point { x: 0.0, y: 5e5 }));
    // Four quarter arcs, all exact: the ellipse's axes are where the angles land.
    let ends = [(1e6, 0.0), (2e6, 5e5), (1e6, 1e6), (0.0, 5e5)];
    for (i, (x, y)) in ends.iter().enumerate() {
        let arc = arc_at(path, i + 1);
        assert_eq!((arc.wr, arc.hr), (1e6, 5e5));
        assert_eq!(arc.center, Point { x: 1e6, y: 5e5 }, "arc {i}");
        assert_eq!(arc.end, Point { x: *x, y: *y }, "arc {i}");
        assert_eq!(arc.sw_ang, CD4);
    }
    assert_eq!(path.segments[5], Segment::Close);
    assert_rect(&g, il, it, ir, ib);
    assert_sites(
        &g,
        &[
            (1e6, 0.0, C3D4),
            (il, it, C3D4),
            (0.0, 5e5, CD2),
            (il, ib, CD4),
            (1e6, 1e6, CD4),
            (ir, ib, CD4),
            (2e6, 5e5, 0.0),
            (ir, it, C3D4),
        ],
    );
}

#[test]
fn golden_round_rect() {
    // 3000000 x 1000000, default adj 16667. x1 = 1000000 * 16667 / 100000 = 166670.
    // x2 = 3000000 - 166670 = 2833330; y2 = 1000000 - 166670 = 833330.
    // il = 166670 * 29289 / 100000 = 48815.9763.
    let g = preset_shape("roundRect")
        .unwrap()
        .evaluate(Extents::new(3_000_000, 1_000_000), &[]);
    let path = &g.paths[0];
    let (x1, x2, y2) = (166_670.0, 2_833_330.0, 833_330.0);
    assert_eq!(path.segments[0], Segment::MoveTo(Point { x: 0.0, y: x1 }));
    assert_arc_end(arc_at(path, 1), x1, 0.0);
    assert_eq!(path.segments[2], Segment::LineTo(Point { x: x2, y: 0.0 }));
    assert_arc_end(arc_at(path, 3), 3e6, x1);
    assert_eq!(path.segments[4], Segment::LineTo(Point { x: 3e6, y: y2 }));
    assert_arc_end(arc_at(path, 5), x2, 1e6);
    assert_eq!(path.segments[6], Segment::LineTo(Point { x: x1, y: 1e6 }));
    assert_arc_end(arc_at(path, 7), 0.0, y2);
    assert_eq!(path.segments[8], Segment::Close);
    let il = 48_815.976_3;
    assert_rect(&g, il, il, 3e6 - il, 1e6 - il);
    assert_sites(
        &g,
        &[
            (1.5e6, 0.0, C3D4),
            (0.0, 5e5, CD2),
            (1.5e6, 1e6, CD4),
            (3e6, 5e5, 0.0),
        ],
    );

    // adj 50000 is the pin's top: the short sides become half-circles.
    let pill = preset_shape("roundRect").unwrap().evaluate(
        Extents::new(3_000_000, 1_000_000),
        &[Adjust::val("adj", 90_000)],
    );
    assert_eq!(pill.guide("x1"), Some(5e5));
    assert_eq!(pill.adjust_handles.len(), 1);
    let AdjustHandle::Xy { x, y, pos } = pill.adjust_handles[0] else {
        panic!();
    };
    assert_eq!(x.guide, Some("adj"));
    assert_eq!((x.min, x.max), (Some(0.0), Some(50_000.0)));
    assert_eq!(y.guide, None);
    assert_eq!(pos, Point { x: 5e5, y: 0.0 });
}

#[test]
fn golden_right_arrow() {
    // 2000000 x 1000000, adj1 = adj2 = 50000. ss = 1000000.
    // maxAdj2 = 100000 * 2000000 / 1000000 = 200000.
    // dx1 = 1000000 * 50000 / 100000 = 500000; x1 = 1500000.
    // dy1 = 1000000 * 50000 / 200000 = 250000; y1 = 250000; y2 = 750000.
    // dx2 = y1 * dx1 / hd2 = 250000; x2 = 1750000.
    let g = preset_shape("rightArrow")
        .unwrap()
        .evaluate(Extents::new(2_000_000, 1_000_000), &[]);
    assert_polyline(
        &g.paths[0],
        &[
            (0.0, 250_000.0),
            (1_500_000.0, 250_000.0),
            (1_500_000.0, 0.0),
            (2_000_000.0, 500_000.0),
            (1_500_000.0, 1_000_000.0),
            (1_500_000.0, 750_000.0),
            (0.0, 750_000.0),
        ],
    );
    assert_rect(&g, 0.0, 250_000.0, 1_750_000.0, 750_000.0);
    assert_sites(
        &g,
        &[
            (1_500_000.0, 0.0, C3D4),
            (0.0, 500_000.0, CD2),
            (1_500_000.0, 1_000_000.0, CD4),
            (2_000_000.0, 500_000.0, 0.0),
        ],
    );

    // With adjusts: adj1 = 20000 narrows the shaft, adj2 past maxAdj2 pins to it.
    // a2 = 200000: dx1 = 2000000; x1 = 0. dy1 = 100000; y1 = 400000; y2 = 600000.
    let pinned = preset_shape("rightArrow").unwrap().evaluate(
        Extents::new(2_000_000, 1_000_000),
        &[Adjust::val("adj1", 20_000), Adjust::val("adj2", 999_999)],
    );
    assert_eq!(pinned.guide("a2"), Some(200_000.0));
    near(pinned.guide("x1").unwrap(), 0.0);
    near(pinned.guide("y1").unwrap(), 400_000.0);
    near(pinned.guide("y2").unwrap(), 600_000.0);
}

#[test]
fn golden_wedge_round_rect_callout_tail_below() {
    // 2000000 x 1000000, adj -20833, 62500, 16667.
    // dxPos = -416660; dyPos = 625000; xPos = 583340; yPos = 1125000.
    // dq = dxPos * h / w = -208330; dz = |dyPos| - |dq| = 416670 > 0: tail on an
    // end, and dyPos > 0: the bottom one. dxPos <= 0 picks xg 2, 5 of 12:
    // x1 = 333333.33, x2 = 833333.33. dyPos > 0 picks yg 7, 10: y1 = 583333.33,
    // y2 = 833333.33. u1 = 1000000 * 16667 / 100000 = 166670.
    // il = 166670 * 29289 / 100000 = 48815.9763.
    let g = preset_shape("wedgeRoundRectCallout")
        .unwrap()
        .evaluate(Extents::new(2_000_000, 1_000_000), &[]);
    let path = &g.paths[0];
    let (x1, x2) = (2e6 * 2.0 / 12.0, 2e6 * 5.0 / 12.0);
    let (y1, y2) = (1e6 * 7.0 / 12.0, 1e6 * 10.0 / 12.0);
    let (u1, u2, v2) = (166_670.0, 1_833_330.0, 833_330.0);

    let line = |index: usize, x: f64, y: f64| {
        let Segment::LineTo(p) = path.segments[index] else {
            panic!("{index}: {:?}", path.segments[index]);
        };
        near(p.x, x);
        near(p.y, y);
    };
    assert_eq!(path.segments[0], Segment::MoveTo(Point { x: 0.0, y: u1 }));
    assert_arc_end(arc_at(path, 1), u1, 0.0);
    line(2, x1, 0.0); // top edge: tail absent, so xt, yt stay on the edge
    line(3, x1, 0.0);
    line(4, x2, 0.0);
    line(5, u2, 0.0);
    assert_arc_end(arc_at(path, 6), 2e6, u1);
    line(7, 2e6, y1); // right edge: tail absent
    line(8, 2e6, y1);
    line(9, 2e6, y2);
    line(10, 2e6, v2);
    assert_arc_end(arc_at(path, 11), u2, 1e6);
    line(12, x2, 1e6); // bottom edge: here is the tail
    line(13, 583_340.0, 1_125_000.0);
    line(14, x1, 1e6);
    line(15, u1, 1e6);
    assert_arc_end(arc_at(path, 16), 0.0, v2);
    line(17, 0.0, y2); // left edge: tail absent
    line(18, 0.0, y1);
    line(19, 0.0, y1);
    assert_eq!(path.segments[20], Segment::Close);

    let il = 48_815.976_3;
    assert_rect(&g, il, il, 2e6 - il, 1e6 - il);
    assert_sites(
        &g,
        &[
            (1e6, 0.0, C3D4),
            (0.0, 5e5, CD2),
            (1e6, 1e6, CD4),
            (2e6, 5e5, 0.0),
            (583_340.0, 1_125_000.0, CD4),
        ],
    );
    let AdjustHandle::Xy { x, y, pos } = g.adjust_handles[0] else {
        panic!();
    };
    assert_eq!((x.guide, y.guide), (Some("adj1"), Some("adj2")));
    assert_eq!((pos.x, pos.y), (583_340.0, 1_125_000.0));
}

#[test]
fn golden_wedge_round_rect_callout_tail_right() {
    // adj1 = 70000, adj2 = 0. dxPos = 1400000; dyPos = 0; dq = 700000; dz = -700000:
    // tail on a side, and dxPos > 0: the right one. yg stays 2, 5 because dyPos is
    // not positive: y1 = 166666.67, y2 = 416666.67. xPos = 2400000; yPos = 500000.
    let g = preset_shape("wedgeRoundRectCallout").unwrap().evaluate(
        Extents::new(2_000_000, 1_000_000),
        &[Adjust::val("adj1", 70_000), Adjust::val("adj2", 0)],
    );
    let path = &g.paths[0];
    let Segment::LineTo(p) = path.segments[8] else {
        panic!();
    };
    near(p.x, 2_400_000.0);
    near(p.y, 500_000.0);
    near(g.guide("y1").unwrap(), 1e6 * 2.0 / 12.0);
    near(g.guide("y2").unwrap(), 1e6 * 5.0 / 12.0);
    assert_eq!(
        g.connection_sites[4].pos,
        Point {
            x: 2_400_000.0,
            y: 500_000.0
        }
    );
}

/// A path with its own w/h scales to the shape, arcs included. Worked on `can`-free
/// data: a 100 x 100 path space on a 1000 x 500 shape.
#[test]
fn path_coordinate_space_scales_per_axis() {
    use casual_slide_geometry::{RawCommand, RawGeometry, RawPath, RawPoint};
    let raw = RawGeometry {
        paths: vec![RawPath {
            w: 100,
            h: 100,
            commands: vec![
                RawCommand::MoveTo(RawPoint {
                    x: "0".into(),
                    y: "50".into(),
                }),
                RawCommand::ArcTo {
                    wr: "50".into(),
                    hr: "50".into(),
                    st_ang: "cd2".into(),
                    sw_ang: "cd4".into(),
                },
                RawCommand::LineTo(RawPoint {
                    x: "100".into(),
                    y: "100".into(),
                }),
            ],
            ..RawPath::default()
        }],
        ..RawGeometry::default()
    };
    let def = raw.compile().unwrap().def;
    let g = def.evaluate(Extents::new(1000, 500), &[]);
    let path = &g.paths[0];
    assert_eq!((path.w, path.h), (100, 100));
    assert_eq!(path.segments[0], Segment::MoveTo(Point { x: 0.0, y: 50.0 }));
    assert_eq!(path.scale(g.extents), (10.0, 5.0));

    let scaled = path.scaled_to(g.extents);
    assert_eq!((scaled.w, scaled.h), (0, 0));
    assert_eq!(
        scaled.segments[0],
        Segment::MoveTo(Point { x: 0.0, y: 250.0 })
    );
    let arc = arc_at(&scaled, 1);
    assert_eq!((arc.wr, arc.hr), (500.0, 250.0));
    assert_eq!(arc.center, Point { x: 500.0, y: 250.0 });
    assert_eq!(arc.end, Point { x: 500.0, y: 0.0 });
    assert_eq!(
        scaled.segments[2],
        Segment::LineTo(Point {
            x: 1000.0,
            y: 500.0
        })
    );
}

/// An arc on a non-circular ellipse. The authored angle is the direction of a ray, so
/// at 45 degrees on a 2:1 ellipse the end is where that ray meets the curve:
/// x = y = ab / sqrt(a^2 + b^2) = 200 / sqrt(5) = 89.4427.
#[test]
fn arc_angles_are_rays_not_parameters() {
    use casual_slide_geometry::{RawCommand, RawGeometry, RawPath, RawPoint};
    let raw = RawGeometry {
        paths: vec![RawPath {
            commands: vec![
                RawCommand::MoveTo(RawPoint {
                    x: "200".into(),
                    y: "100".into(),
                }),
                RawCommand::ArcTo {
                    wr: "200".into(),
                    hr: "100".into(),
                    st_ang: "0".into(),
                    sw_ang: "2700000".into(),
                },
            ],
            ..RawPath::default()
        }],
        ..RawGeometry::default()
    };
    let def = raw.compile().unwrap().def;
    let out = def.evaluate(Extents::new(400, 200), &[]);
    let arc = arc_at(&out.paths[0], 1);
    assert_eq!(arc.center, Point { x: 0.0, y: 100.0 });
    let leg = 200.0 / 5f64.sqrt();
    assert!((arc.end.x - leg).abs() < 1e-9);
    assert!((arc.end.y - (100.0 + leg)).abs() < 1e-9);
    assert_eq!(arc.sw_ang, 2_700_000.0, "as authored");
    assert!(
        arc.sweep_param > 3_000_000.0,
        "the parameter sweep is wider"
    );
}

/// Converted arcs follow the true ellipse, end exactly where the arc does, and stay
/// close to it in between.
#[test]
fn cubic_conversion_tracks_the_ellipse() {
    let g = preset_shape("ellipse")
        .unwrap()
        .evaluate(Extents::new(2_000_000, 1_000_000), &[]);
    let converted = g.paths[0].with_arcs_as_cubics();
    assert_eq!(converted.segments.len(), 6);
    let mut from = Point { x: 0.0, y: 5e5 };
    for segment in &converted.segments[1..5] {
        let Segment::CubicTo { c1, c2, end } = *segment else {
            panic!();
        };
        for step in 0..=32 {
            let t = f64::from(step) / 32.0;
            let u = 1.0 - t;
            let bx = u * u * u * from.x
                + 3.0 * u * u * t * c1.x
                + 3.0 * u * t * t * c2.x
                + t * t * t * end.x;
            let by = u * u * u * from.y
                + 3.0 * u * u * t * c1.y
                + 3.0 * u * t * t * c2.y
                + t * t * t * end.y;
            let (nx, ny) = ((bx - 1e6) / 1e6, (by - 5e5) / 5e5);
            let radius = (nx * nx + ny * ny).sqrt();
            assert!((radius - 1.0).abs() < 3e-4, "off the ellipse by {radius}");
        }
        from = end;
    }
    assert_eq!(from, Point { x: 0.0, y: 5e5 });
}

/// Reproducibility: the exact bits of the whole table. The same number was obtained
/// on aarch64-apple-darwin and wasm32-unknown-unknown (under Node). Anything that
/// changes the table or any result changes it, on purpose or not.
#[test]
fn table_fingerprint_is_pinned() {
    assert_eq!(table_fingerprint(), 3_356_062_406_242_477_444);
}
