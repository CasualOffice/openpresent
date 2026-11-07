//! The guide language: operators, built-ins, and how names resolve.

use casual_slide_geometry::math::VALUE_LIMIT;
use casual_slide_geometry::{
    Diagnostic, Extents, Op, RawGeometry, RawGuide, apply, builtin_guide_names,
};

fn guide(name: &str, fmla: &str) -> RawGuide {
    RawGuide {
        name: name.to_owned(),
        fmla: fmla.to_owned(),
    }
}

/// Evaluates `result = fmla` at 1200000 x 800000.
fn eval(fmla: &str) -> f64 {
    let raw = RawGeometry {
        gd: vec![guide("result", fmla)],
        ..RawGeometry::default()
    };
    let compiled = raw.compile().unwrap();
    compiled
        .def
        .evaluate(Extents::new(1_200_000, 800_000), &[])
        .guide("result")
        .unwrap()
}

/// Every operator through the formula text, so parsing and arity are covered too.
/// Angles: 5400000 = 90 degrees. Expected values are worked by hand.
#[test]
fn operators_by_table() {
    let root2 = 2f64.sqrt();
    let cases: &[(&str, f64)] = &[
        ("val 42", 42.0),
        ("val -42", -42.0),
        ("val +42", 42.0),
        ("val w", 1_200_000.0),
        ("abs -7", 7.0),
        ("abs 7", 7.0),
        ("sqrt 144", 12.0),
        ("sqrt 2", root2),
        ("sqrt 0", 0.0),
        ("max 3 9", 9.0),
        ("max -3 -9", -3.0),
        ("min 3 9", 3.0),
        ("min -3 -9", -9.0),
        ("*/ 6 7 3", 14.0),
        ("*/ 1 1 3", 1.0 / 3.0),
        ("*/ w 1 2", 600_000.0),
        ("+- 10 5 3", 12.0),
        ("+- 0 0 3", -3.0),
        ("+/ 10 5 3", 5.0),
        ("+/ 1 2 4", 0.75),
        ("?: 1 10 20", 10.0),
        ("?: 0 10 20", 20.0),
        ("?: -1 10 20", 20.0),
        ("mod 3 4 0", 5.0),
        ("mod 2 3 6", 7.0),
        ("mod -3 -4 0", 5.0),
        ("pin 0 5 10", 5.0),
        ("pin 0 -5 10", 0.0),
        ("pin 0 15 10", 10.0),
        ("pin 10 5 0", 10.0), // lower bound is tested first
        ("cos 100 0", 100.0),
        ("cos 100 5400000", 0.0),
        ("cos 100 10800000", -100.0),
        ("sin 100 0", 0.0),
        ("sin 100 5400000", 100.0),
        ("sin 100 16200000", -100.0),
        ("sin 100 21600000", 0.0),
        ("sin 100 -5400000", -100.0),
        ("tan 100 0", 0.0),
        ("at2 1 0", 0.0),
        ("at2 0 1", 5_400_000.0),
        ("at2 -1 0", 10_800_000.0),
        ("at2 0 -1", -5_400_000.0),
        ("at2 1 1", 2_700_000.0),
        ("at2 -1 -1", -8_100_000.0),
        ("at2 0 0", 0.0),
        ("cat2 100 3 4", 60.0), // 100 * cos(atan(4/3)) = 100 * 3/5
        ("sat2 100 3 4", 80.0), // 100 * sin(atan(4/3)) = 100 * 4/5
        ("cat2 100 -3 4", -60.0),
        ("sat2 100 3 -4", -80.0),
        ("cat2 100 0 0", 100.0), // atan2(0, 0) = 0
        ("sat2 100 0 0", 0.0),
        ("cat2 100 0 5", 0.0),
        ("sat2 100 0 5", 100.0),
    ];
    for (fmla, want) in cases {
        let got = eval(fmla);
        assert!((got - want).abs() < 1e-9, "{fmla}: got {got}, want {want}");
    }
    assert!(
        (eval("tan 100 2700000") - 100.0).abs() < 1e-9,
        "tan 45 degrees"
    );
    assert!((eval("cos 100 2700000") - 50.0 * root2).abs() < 1e-9);
    assert!((eval("sin 100 1800000") - 50.0).abs() < 1e-9);
}

/// Operators with no value at this input give 0, finite, whichever way they fail.
#[test]
fn undefined_results_are_zero() {
    for fmla in [
        "*/ 1 1 0",
        "*/ 0 0 0",
        "*/ -5 9 0",
        "+/ 1 1 0",
        "+/ -1 -1 0",
        "sqrt -1",
        "tan 100 5400000",
        "tan 100 16200000",
    ] {
        assert_eq!(eval(fmla), 0.0, "{fmla}");
    }
}

#[test]
fn apply_is_total_on_extreme_operands() {
    let values = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        1e-300,
        -1e-300,
        VALUE_LIMIT,
        -VALUE_LIMIT,
        f64::MAX,
        f64::MIN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ];
    let ops = [
        Op::Val,
        Op::Abs,
        Op::Sqrt,
        Op::At2,
        Op::Cos,
        Op::Sin,
        Op::Tan,
        Op::Max,
        Op::Min,
        Op::MulDiv,
        Op::AddSub,
        Op::AddDiv,
        Op::If,
        Op::Cat2,
        Op::Sat2,
        Op::Mod,
        Op::Pin,
    ];
    for op in ops {
        for x in values {
            for y in values {
                for z in values {
                    // The evaluator only ever passes saturated operands, but `apply` is
                    // public, so it must hold for anything.
                    let result = apply(op, x, y, z);
                    assert!(result.is_finite(), "{op:?}({x},{y},{z})");
                    assert!(result.abs() <= VALUE_LIMIT, "{op:?}({x},{y},{z})");
                    assert_ne!(result.to_bits(), (-0.0f64).to_bits(), "{op:?}({x},{y},{z})");
                }
            }
        }
    }
}

#[test]
fn saturates_rather_than_overflowing() {
    let raw = RawGeometry {
        gd: vec![
            guide("a", "val 9223372036854775807"),
            guide("b", "*/ a a 1"),
            guide("c", "*/ b b 1"),
            guide("d", "*/ c c 1"),
            guide("e", "*/ d d 1"),
            guide("f", "*/ e e 1"),
            guide("g", "*/ f f 1"),
            guide("big", "+- g g 0"),
            guide("small", "+- 0 0 big"),
            guide("huge", "val 99999999999999999999999999999999"),
        ],
        ..RawGeometry::default()
    };
    let def = raw.compile().unwrap().def;
    let out = def.evaluate(Extents::new(100, 100), &[]);
    assert_eq!(out.guide("big"), Some(VALUE_LIMIT));
    assert_eq!(out.guide("small"), Some(-VALUE_LIMIT));
    assert_eq!(
        out.guide("huge"),
        Some(i64::MAX as f64),
        "literal saturates at parse"
    );
}

/// ECMA's table, plus `hd10`, which two presets rely on.
#[test]
fn built_in_guides() {
    let (w, h) = (1_200_000.0, 800_000.0);
    let (ss, ls) = (h, w);
    let want: &[(&str, f64)] = &[
        ("3cd4", 16_200_000.0),
        ("3cd8", 8_100_000.0),
        ("5cd8", 13_500_000.0),
        ("7cd8", 18_900_000.0),
        ("b", h),
        ("cd2", 10_800_000.0),
        ("cd4", 5_400_000.0),
        ("cd8", 2_700_000.0),
        ("h", h),
        ("hc", w / 2.0),
        ("hd10", h / 10.0),
        ("hd2", h / 2.0),
        ("hd3", h / 3.0),
        ("hd4", h / 4.0),
        ("hd5", h / 5.0),
        ("hd6", h / 6.0),
        ("hd8", h / 8.0),
        ("l", 0.0),
        ("ls", ls),
        ("r", w),
        ("ss", ss),
        ("ssd16", ss / 16.0),
        ("ssd2", ss / 2.0),
        ("ssd32", ss / 32.0),
        ("ssd4", ss / 4.0),
        ("ssd6", ss / 6.0),
        ("ssd8", ss / 8.0),
        ("t", 0.0),
        ("vc", h / 2.0),
        ("w", w),
        ("wd10", w / 10.0),
        ("wd12", w / 12.0),
        ("wd16", w / 16.0),
        ("wd2", w / 2.0),
        ("wd3", w / 3.0),
        ("wd32", w / 32.0),
        ("wd4", w / 4.0),
        ("wd5", w / 5.0),
        ("wd6", w / 6.0),
        ("wd8", w / 8.0),
    ];
    let names: Vec<&str> = want.iter().map(|(name, _)| *name).collect();
    assert_eq!(builtin_guide_names(), names.as_slice(), "slot order");
    let def = RawGeometry::default().compile().unwrap().def;
    let out = def.evaluate(Extents::new(1_200_000, 800_000), &[]);
    for (name, value) in want {
        assert_eq!(out.guide(name), Some(*value), "{name}");
        // Through a formula as well: the slot and the name must agree.
        assert_eq!(eval(&format!("val {name}")), *value, "val {name}");
    }
}

#[test]
fn short_side_follows_the_shape() {
    let def = RawGeometry::default().compile().unwrap().def;
    let tall = def.evaluate(Extents::new(300, 900), &[]);
    assert_eq!(tall.guide("ss"), Some(300.0));
    assert_eq!(tall.guide("ls"), Some(900.0));
}

#[test]
fn malformed_formulas_are_zero_and_reported() {
    for (fmla, expect) in [
        ("", Diagnostic::UnknownOperator),
        ("   ", Diagnostic::UnknownOperator),
        ("nope 1 2", Diagnostic::UnknownOperator),
        ("VAL 1", Diagnostic::UnknownOperator),
        ("*/ 1 2", Diagnostic::WrongOperandCount),
        ("val", Diagnostic::WrongOperandCount),
    ] {
        let raw = RawGeometry {
            gd: vec![guide("x", fmla)],
            ..RawGeometry::default()
        };
        let compiled = raw.compile().unwrap();
        assert_eq!(compiled.diagnostics.kept(), &[expect], "{fmla:?}");
        let out = compiled.def.evaluate(Extents::new(10, 10), &[]);
        assert_eq!(out.guide("x"), Some(0.0), "{fmla:?}");
    }
}

/// Eight formulas in the shipped presets have one operand too many (GEO-04), so the
/// surplus has to be tolerated; it is reported for files.
#[test]
fn surplus_operands_are_ignored_and_reported() {
    let raw = RawGeometry {
        gd: vec![guide("x", "+- 10 0 3 999")],
        ..RawGeometry::default()
    };
    let compiled = raw.compile().unwrap();
    assert_eq!(compiled.diagnostics.total(), 1);
    let out = compiled.def.evaluate(Extents::new(10, 10), &[]);
    assert_eq!(out.guide("x"), Some(7.0));
}

/// No cycle can be written: each name resolves to what is already defined.
#[test]
fn self_and_forward_references_read_zero() {
    let raw = RawGeometry {
        gd: vec![
            guide("a", "+- a 1 0"),  // self: a is not yet defined
            guide("b", "+- c 1 0"),  // forward
            guide("c", "+- b 1 0"),  // sees b; the would-be cycle is cut at b
            guide("a", "+- a 10 0"), // redefinition reads the previous a
        ],
        ..RawGeometry::default()
    };
    let compiled = raw.compile().unwrap();
    assert_eq!(
        compiled.diagnostics.kept(),
        &[Diagnostic::UnknownGuide, Diagnostic::UnknownGuide]
    );
    let out = compiled.def.evaluate(Extents::new(10, 10), &[]);
    assert_eq!(out.guide("b"), Some(1.0));
    assert_eq!(out.guide("c"), Some(2.0));
    assert_eq!(out.guide("a"), Some(11.0), "latest definition");
}

/// textRingInside rebinds `r`, so a name's meaning changes partway through.
#[test]
fn a_guide_can_shadow_a_built_in() {
    let raw = RawGeometry {
        gd: vec![
            guide("before", "val r"),
            guide("r", "val 5"),
            guide("after", "val r"),
        ],
        ..RawGeometry::default()
    };
    let out_def = raw.compile().unwrap().def;
    let out = out_def.evaluate(Extents::new(1000, 1000), &[]);
    assert_eq!(out.guide("before"), Some(1000.0));
    assert_eq!(out.guide("after"), Some(5.0));
    assert_eq!(out.guide("r"), Some(5.0));
}

/// `ST_AdjCoordinate` allows `ST_UniversalMeasure` wherever a path takes a coordinate.
#[test]
fn universal_measures_convert_to_emu() {
    use casual_slide_geometry::{RawCommand, RawPath, RawPoint, Segment};
    for (text, want) in [
        ("1in", 914_400.0),
        ("-1in", -914_400.0),
        ("2.5cm", 900_000.0),
        ("10mm", 360_000.0),
        ("1pt", 12_700.0),
        ("-2.5pt", -31_750.0),
        ("1pc", 152_400.0),
        ("1pi", 152_400.0),
    ] {
        let raw = RawGeometry {
            paths: vec![RawPath {
                commands: vec![RawCommand::MoveTo(RawPoint {
                    x: text.to_owned(),
                    y: "0".to_owned(),
                })],
                ..RawPath::default()
            }],
            ..RawGeometry::default()
        };
        let compiled = raw.compile().unwrap();
        assert!(compiled.diagnostics.is_empty(), "{text}");
        let out = compiled.def.evaluate(Extents::new(10, 10), &[]);
        let Segment::MoveTo(p) = out.paths[0].segments[0] else {
            panic!();
        };
        assert_eq!(p.x, want, "{text}");
    }
}
