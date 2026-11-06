//! SEC-C08: custom geometry from a hostile file. Nothing here may panic, loop or
//! produce a non-finite number, and each cap must reject at its limit.

use casual_slide_geometry::math::COORD_LIMIT;
use casual_slide_geometry::{
    Adjust, Diagnostic, Extents, GeometryDef, GeometryError, RawCommand, RawGeometry, RawGuide,
    RawHandle, RawPath, RawPoint, RawRect, RawSite, ResolvedGeometry, Segment, limits,
    preset_shape,
};

fn guide(name: &str, fmla: &str) -> RawGuide {
    RawGuide {
        name: name.to_owned(),
        fmla: fmla.to_owned(),
    }
}

fn pt(x: &str, y: &str) -> RawPoint {
    RawPoint {
        x: x.to_owned(),
        y: y.to_owned(),
    }
}

fn arc(wr: &str, hr: &str, st: &str, sw: &str) -> RawCommand {
    RawCommand::ArcTo {
        wr: wr.to_owned(),
        hr: hr.to_owned(),
        st_ang: st.to_owned(),
        sw_ang: sw.to_owned(),
    }
}

fn assert_sane(out: &ResolvedGeometry<'_>) {
    let check = |value: f64| {
        assert!(value.is_finite(), "{value}");
        assert!(value.abs() <= COORD_LIMIT, "{value}");
    };
    let point = |p: casual_slide_geometry::Point| {
        check(p.x);
        check(p.y);
    };
    for path in &out.paths {
        for candidate in [
            path.clone(),
            path.scaled_to(out.extents),
            path.with_arcs_as_cubics(),
        ] {
            for segment in &candidate.segments {
                match *segment {
                    Segment::MoveTo(p) | Segment::LineTo(p) => point(p),
                    Segment::QuadTo { ctrl, end } => {
                        point(ctrl);
                        point(end);
                    }
                    Segment::CubicTo { c1, c2, end } => {
                        point(c1);
                        point(c2);
                        point(end);
                    }
                    Segment::ArcTo(arc) => {
                        check(arc.wr);
                        check(arc.hr);
                        check(arc.st_ang);
                        check(arc.sw_ang);
                        assert!(arc.start_param.is_finite());
                        assert!(arc.sweep_param.is_finite());
                        point(arc.center);
                        point(arc.start);
                        point(arc.end);
                    }
                    Segment::Close => {}
                }
            }
        }
    }
    let t = out.text_rect;
    for value in [t.l, t.t, t.r, t.b] {
        check(value);
    }
    for site in &out.connection_sites {
        point(site.pos);
        check(site.ang);
    }
}

fn with_guides(count: usize) -> RawGeometry {
    RawGeometry {
        gd: (0..count)
            .map(|i| guide(&format!("g{i}"), "val 1"))
            .collect(),
        ..RawGeometry::default()
    }
}

#[test]
fn guide_cap_is_exact_and_counts_both_lists() {
    assert!(with_guides(limits::MAX_GUIDES).compile().is_ok());
    assert_eq!(
        with_guides(limits::MAX_GUIDES + 1).compile().unwrap_err(),
        GeometryError::TooManyGuides
    );
    let split = RawGeometry {
        av: (0..limits::MAX_GUIDES)
            .map(|i| guide(&format!("a{i}"), "val 1"))
            .collect(),
        gd: vec![guide("over", "val 1")],
        ..RawGeometry::default()
    };
    assert_eq!(split.compile().unwrap_err(), GeometryError::TooManyGuides);
}

#[test]
fn point_cap_is_exact_and_counts_bezier_points() {
    let lines = |count: usize| RawGeometry {
        paths: vec![RawPath {
            commands: vec![RawCommand::LineTo(pt("0", "0")); count],
            ..RawPath::default()
        }],
        ..RawGeometry::default()
    };
    assert!(lines(limits::MAX_PATH_POINTS).compile().is_ok());
    assert_eq!(
        lines(limits::MAX_PATH_POINTS + 1).compile().unwrap_err(),
        GeometryError::TooManyPathPoints
    );

    let cubic = RawCommand::CubicBezTo(pt("0", "0"), pt("0", "0"), pt("0", "0"));
    let beziers = RawGeometry {
        paths: vec![RawPath {
            commands: vec![cubic; limits::MAX_PATH_POINTS / 3 + 1],
            ..RawPath::default()
        }],
        ..RawGeometry::default()
    };
    assert_eq!(
        beziers.compile().unwrap_err(),
        GeometryError::TooManyPathPoints
    );

    let closes = RawGeometry {
        paths: vec![RawPath {
            commands: vec![RawCommand::Close; limits::MAX_PATH_POINTS + 1],
            ..RawPath::default()
        }],
        ..RawGeometry::default()
    };
    assert_eq!(
        closes.compile().unwrap_err(),
        GeometryError::TooManyPathPoints
    );

    let spread = RawGeometry {
        paths: vec![
            RawPath {
                commands: vec![RawCommand::LineTo(pt("0", "0")); limits::MAX_PATH_POINTS / 2 + 1],
                ..RawPath::default()
            };
            2
        ],
        ..RawGeometry::default()
    };
    assert_eq!(
        spread.compile().unwrap_err(),
        GeometryError::TooManyPathPoints
    );
}

#[test]
fn other_caps() {
    let paths = RawGeometry {
        paths: vec![RawPath::default(); limits::MAX_PATHS + 1],
        ..RawGeometry::default()
    };
    assert_eq!(paths.compile().unwrap_err(), GeometryError::TooManyPaths);

    let handle = RawHandle::Xy {
        x: Default::default(),
        y: Default::default(),
        pos: pt("0", "0"),
    };
    let handles = RawGeometry {
        handles: vec![handle; limits::MAX_HANDLES + 1],
        ..RawGeometry::default()
    };
    assert_eq!(
        handles.compile().unwrap_err(),
        GeometryError::TooManyHandles
    );

    let sites = RawGeometry {
        sites: vec![RawSite::default(); limits::MAX_SITES + 1],
        ..RawGeometry::default()
    };
    assert_eq!(sites.compile().unwrap_err(), GeometryError::TooManySites);
}

#[test]
fn oversized_text_is_rejected_without_losing_the_shape() {
    let long_name = "n".repeat(limits::MAX_NAME_LEN + 1);
    let long_formula = format!("val {}", "9".repeat(limits::MAX_FORMULA_LEN));
    let raw = RawGeometry {
        gd: vec![
            guide(&long_name, "val 7"),
            guide("x", &long_formula),
            guide("y", "val 3"),
        ],
        ..RawGeometry::default()
    };
    let compiled = raw.compile().unwrap();
    assert_eq!(compiled.diagnostics.total(), 2);
    let out = compiled.def.evaluate(Extents::new(10, 10), &[]);
    assert_eq!(out.guide("x"), Some(0.0));
    assert_eq!(out.guide("y"), Some(3.0));
}

#[test]
fn diagnostics_are_bounded_and_text_clipped() {
    // A multi-byte character straddling the clip point must not panic either.
    let name = "é".repeat(200);
    let raw = RawGeometry {
        gd: (0..limits::MAX_GUIDES)
            .map(|i| guide(&format!("g{i}"), &format!("val {name}")))
            .collect(),
        ..RawGeometry::default()
    };
    let compiled = raw.compile().unwrap();
    assert_eq!(compiled.diagnostics.total(), limits::MAX_GUIDES);
    assert!(compiled.diagnostics.kept().len() <= 32);
    let Diagnostic::UnknownGuide(subject) = &compiled.diagnostics.kept()[0] else {
        panic!("{:?}", compiled.diagnostics.kept()[0]);
    };
    assert!(subject.len() <= 64);
}

/// The long chain is the attack on a recursive evaluator: each guide names the one
/// before, as deep as the cap allows. This also has to be quick.
#[test]
fn a_chain_at_the_cap_evaluates() {
    let mut gd = vec![guide("g0", "val 1")];
    for i in 1..limits::MAX_GUIDES {
        gd.push(guide(&format!("g{i}"), &format!("+- g{} 1 0", i - 1)));
    }
    let raw = RawGeometry {
        gd,
        ..RawGeometry::default()
    };
    let def = raw.compile().unwrap().def;
    let out = def.evaluate(Extents::new(100, 100), &[]);
    let last = format!("g{}", limits::MAX_GUIDES - 1);
    assert_eq!(out.guide(&last), Some(limits::MAX_GUIDES as f64));
}

/// Every value below goes into every operand of a geometry using every command.
#[test]
fn extreme_values_everywhere() {
    let values = [
        "0",
        "-1",
        "1",
        "9223372036854775807",
        "-9223372036854775808",
        "99999999999999999999999999999",
        "not a number",
        "",
        "1e308",
        "NaN",
        "inf",
        "2147483647",
        "21600000",
        "-21600001",
        "1.5in",
        "-99999999999999.999999pt",
        "mm",
        "\u{00e9}\u{00e9}",
    ];
    let extents = [
        (0, 0),
        (1, 0),
        (0, 1),
        (1, 1),
        (914_400, 914_400),
        (i64::MAX, i64::MAX),
        (-5, -5),
    ];
    for value in values {
        let raw = RawGeometry {
            av: vec![guide("adj", &format!("val {value}"))],
            gd: vec![
                guide("a", &format!("*/ {value} {value} 0")),
                guide("b", &format!("*/ {value} {value} {value}")),
                guide("c", &format!("sin {value} {value}")),
                guide("d", &format!("tan {value} {value}")),
                guide("e", &format!("at2 {value} {value}")),
                guide("f", &format!("cat2 {value} {value} {value}")),
                guide("h1", &format!("mod {value} {value} {value}")),
                guide("big", "*/ adj adj adj"),
            ],
            handles: vec![
                RawHandle::Xy {
                    x: casual_slide_geometry::RawAxis {
                        guide: Some("adj".to_owned()),
                        min: Some(value.to_owned()),
                        max: Some(value.to_owned()),
                    },
                    y: Default::default(),
                    pos: pt(value, value),
                },
                RawHandle::Polar {
                    r: Default::default(),
                    ang: casual_slide_geometry::RawAxis {
                        guide: Some(value.to_owned()),
                        min: None,
                        max: None,
                    },
                    pos: pt("big", "big"),
                },
            ],
            sites: vec![RawSite {
                ang: value.to_owned(),
                pos: pt(value, "big"),
            }],
            rect: Some(RawRect {
                l: value.to_owned(),
                t: "big".to_owned(),
                r: value.to_owned(),
                b: "a".to_owned(),
            }),
            paths: vec![RawPath {
                w: 1,
                h: 1,
                commands: vec![
                    arc(value, value, value, value),
                    RawCommand::MoveTo(pt(value, value)),
                    RawCommand::LineTo(pt("big", "b")),
                    arc("big", value, "21600000", "-99999999999"),
                    arc("0", "0", "0", "21600000"),
                    arc("big", "1", "big", "big"),
                    RawCommand::QuadBezTo(pt(value, value), pt("big", "c")),
                    RawCommand::CubicBezTo(pt(value, "d"), pt("e", "f"), pt("h1", value)),
                    RawCommand::Close,
                    RawCommand::LineTo(pt("big", "big")),
                ],
                ..RawPath::default()
            }],
        };
        let compiled = raw.compile().unwrap();
        for (w, h) in extents {
            for adjusts in [
                &[][..],
                &[Adjust::val("adj", i64::MAX)][..],
                &[Adjust::formula("adj", value)][..],
                &[Adjust::formula("adj", "*/ w w 0")][..],
            ] {
                assert_sane(&compiled.def.evaluate(Extents::new(w, h), adjusts));
            }
        }
    }
}

/// A small deterministic generator, so a failure repeats and needs no dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn pick<'a>(&mut self, options: &[&'a str]) -> &'a str {
        options[(self.next() % options.len() as u64) as usize]
    }
}

fn random_geometry(rng: &mut Rng) -> RawGeometry {
    const OPS: [&str; 18] = [
        "val", "abs", "sqrt", "at2", "cos", "sin", "tan", "max", "min", "*/", "+-", "+/", "?:",
        "cat2", "sat2", "mod", "pin", "bogus",
    ];
    const ATOMS: [&str; 16] = [
        "w",
        "h",
        "ss",
        "cd4",
        "0",
        "-1",
        "1",
        "100000",
        "21600000",
        "-21600000",
        "9223372036854775807",
        "g0",
        "g1",
        "g5",
        "g9",
        "missing",
    ];
    let mut raw = RawGeometry::default();
    for i in 0..12 {
        let fmla = format!(
            "{} {} {} {}",
            rng.pick(&OPS),
            rng.pick(&ATOMS),
            rng.pick(&ATOMS),
            rng.pick(&ATOMS)
        );
        raw.gd.push(guide(&format!("g{i}"), &fmla));
    }
    let mut path = RawPath {
        w: (rng.next() % 4) as i64 * 1000,
        h: (rng.next() % 4) as i64 * 1000,
        ..RawPath::default()
    };
    for _ in 0..10 {
        let p = pt(rng.pick(&ATOMS), rng.pick(&ATOMS));
        let q = pt(rng.pick(&ATOMS), rng.pick(&ATOMS));
        path.commands.push(match rng.next() % 6 {
            0 => RawCommand::MoveTo(p),
            1 => RawCommand::LineTo(p),
            2 => RawCommand::QuadBezTo(p, q),
            3 => RawCommand::CubicBezTo(p.clone(), q, p),
            4 => RawCommand::Close,
            _ => arc(
                rng.pick(&ATOMS),
                rng.pick(&ATOMS),
                rng.pick(&ATOMS),
                rng.pick(&ATOMS),
            ),
        });
    }
    raw.paths.push(path);
    raw
}

#[test]
fn randomized_definitions_stay_sane_and_repeat_exactly() {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    for _ in 0..3000 {
        let raw = random_geometry(&mut rng);
        let def: GeometryDef = raw.compile().unwrap().def;
        let out = def.evaluate(Extents::new(914_400, 457_200), &[]);
        assert_sane(&out);
        let again = def.evaluate(Extents::new(914_400, 457_200), &[]);
        assert_eq!(out.fingerprint(), again.fingerprint());
    }
}

#[test]
fn output_always_starts_with_move_to() {
    for first in [
        RawCommand::LineTo(pt("5", "5")),
        RawCommand::Close,
        arc("5", "5", "0", "cd4"),
    ] {
        let raw = RawGeometry {
            paths: vec![RawPath {
                commands: vec![first],
                ..RawPath::default()
            }],
            ..RawGeometry::default()
        };
        let def = raw.compile().unwrap().def;
        let out = def.evaluate(Extents::new(10, 10), &[]);
        assert!(matches!(out.paths[0].segments[0], Segment::MoveTo(_)));
        assert_eq!(out.diagnostics.kept(), &[Diagnostic::PathWithoutMoveTo]);
    }
}

#[test]
fn arc_sweep_is_bounded_to_one_turn() {
    let raw = RawGeometry {
        paths: vec![RawPath {
            commands: vec![
                RawCommand::MoveTo(pt("0", "0")),
                arc("100", "50", "0", "2147483647"),
            ],
            ..RawPath::default()
        }],
        ..RawGeometry::default()
    };
    let def = raw.compile().unwrap().def;
    let out = def.evaluate(Extents::new(10, 10), &[]);
    let Segment::ArcTo(arc) = out.paths[0].segments[1] else {
        panic!();
    };
    assert_eq!(arc.sw_ang, 21_600_000.0);
    let cubics = out.paths[0].with_arcs_as_cubics();
    assert_eq!(cubics.segments.len(), 5, "a move and at most four curves");
}

#[test]
fn bad_adjusts_on_a_preset_are_reported_not_fatal() {
    let def = preset_shape("roundRect").unwrap();
    let ext = Extents::new(1000, 1000);

    let unknown = def.evaluate(ext, &[Adjust::val("nope", 1)]);
    assert_eq!(
        unknown.diagnostics.kept(),
        &[Diagnostic::UnknownAdjust("nope".to_owned())]
    );
    assert_eq!(unknown.guide("adj"), Some(16667.0), "default survives");

    let bad = def.evaluate(ext, &[Adjust::formula("adj", "gibberish")]);
    assert_eq!(bad.diagnostics.total(), 1);
    assert_eq!(
        bad.guide("adj"),
        Some(16667.0),
        "default survives a bad formula"
    );

    // pin keeps a wild adjust inside the shape.
    let wild = def.evaluate(ext, &[Adjust::val("adj", i64::MAX)]);
    assert_eq!(wild.guide("a"), Some(50000.0));
    assert_sane(&wild);

    let many = vec![Adjust::val("adj", 1); limits::MAX_GUIDES + 1];
    let flood = def.evaluate(ext, &many);
    assert_eq!(flood.diagnostics.kept(), &[Diagnostic::TooManyAdjusts]);
}

/// Every preset, under adjust values chosen to break it.
#[test]
fn presets_survive_hostile_adjusts() {
    let values = [
        i64::MIN,
        -21_600_000,
        -1,
        0,
        1,
        50_000,
        21_600_000,
        i64::MAX,
    ];
    let names = [
        "adj", "adj1", "adj2", "adj3", "adj4", "adj5", "adj6", "adj7", "adj8",
    ];
    for def in casual_slide_geometry::preset_shapes()
        .iter()
        .chain(casual_slide_geometry::text_warps())
    {
        for value in values {
            let adjusts: Vec<Adjust<'_>> = names
                .iter()
                .filter(|name| def.av.iter().any(|g| g.name == **name))
                .map(|name| Adjust::val(name, value))
                .collect();
            for (w, h) in [(914_400, 457_200), (0, 0), (1, 27_273_042_316_900)] {
                let out = def.evaluate(Extents::new(w, h), &adjusts);
                assert_sane(&out);
                assert!(
                    out.diagnostics.is_empty(),
                    "{}: {:?}",
                    def.name,
                    out.diagnostics
                );
            }
        }
    }
}
