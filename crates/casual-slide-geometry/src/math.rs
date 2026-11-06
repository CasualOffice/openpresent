//! Reproducible arithmetic for guide evaluation.
//!
//! Guides are `f64`, not integers: `*/`, `sin` and `sqrt` have fractional results
//! that later guides multiply back up, and truncating them would shift outlines
//! by accumulated error. Floats are reproducible here because of what is used:
//!
//! * `+ - * /`, `sqrt` and `%` are exact or correctly rounded by IEEE 754 on every
//!   target we build for (`wasm32`, `x86_64`, `aarch64`). Rust does not contract
//!   them into fused operations and these targets have no x87 excess precision.
//! * `sin`, `cos`, `tan` and `atan2` from the platform are NOT reproducible: `std`
//!   defers to the system libm, which differs across platforms and versions. They
//!   are not called. The versions below use only the operations above.
//! * NaN payloads and `-0.0` vary across targets, so [`saturate`] removes both
//!   from every value that leaves an operation.
//!
//! Angles are in 60000ths of a degree, as in the file format. Keeping them there
//! through reduction makes quarter turns exact: `cos(5400000)` is exactly 0, which
//! a route through radians would not give.

/// Largest guide magnitude. Far above any coordinate, because the presets need it:
/// circularArrow's guides reach the fourth power of the shape size, around 1e21 for
/// a one-inch shape and 1e51 at the largest legal extent, and clipping those bends
/// its outline. The square of this, and a sum of three squares, stay finite in
/// `f64`, so no product of two guides overflows.
pub const VALUE_LIMIT: f64 = 1e150;

/// Largest coordinate or angle magnitude in a result. Fits `i64` without wrapping.
pub const COORD_LIMIT: f64 = 4_611_686_018_427_387_904.0; // 2^62

/// One full turn, in 60000ths of a degree.
pub const TURN: f64 = 21_600_000.0;
const QUARTER: f64 = TURN / 4.0;
const EIGHTH: f64 = TURN / 8.0;
const RADIANS_PER_UNIT: f64 = std::f64::consts::PI / 10_800_000.0;
const UNITS_PER_RADIAN: f64 = 10_800_000.0 / std::f64::consts::PI;

/// Finite, in range, and never `-0.0`. NaN becomes 0: every operation with no
/// defined result (division by zero included) yields 0, so one bad guide cannot
/// poison the guides after it with NaN.
pub fn saturate(value: f64) -> f64 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(-VALUE_LIMIT, VALUE_LIMIT) + 0.0
    }
}

/// A guide value as a result coordinate: `saturate` to the narrower range.
pub fn coordinate(value: f64) -> f64 {
    saturate(value).clamp(-COORD_LIMIT, COORD_LIMIT)
}

/// `sin` on `|x| <= pi/4`: Taylor series to x^17, whose first omitted term is under
/// 1e-19, below half an ulp of the largest result.
fn sin_kernel(x: f64) -> f64 {
    const C: [f64; 8] = [
        -1.0 / 6.0,
        1.0 / 120.0,
        -1.0 / 5040.0,
        1.0 / 362_880.0,
        -1.0 / 39_916_800.0,
        1.0 / 6_227_020_800.0,
        -1.0 / 1_307_674_368_000.0,
        1.0 / 355_687_428_096_000.0,
    ];
    let z = x * x;
    let mut sum = 0.0;
    for coefficient in C.iter().rev() {
        sum = sum * z + coefficient;
    }
    x + x * z * sum
}

/// `cos` on `|x| <= pi/4`: Taylor series to x^18.
fn cos_kernel(x: f64) -> f64 {
    const C: [f64; 8] = [
        1.0 / 24.0,
        -1.0 / 720.0,
        1.0 / 40_320.0,
        -1.0 / 3_628_800.0,
        1.0 / 479_001_600.0,
        -1.0 / 87_178_291_200.0,
        1.0 / 20_922_789_888_000.0,
        -1.0 / 6_402_373_705_728_000.0,
    ];
    let z = x * x;
    let mut sum = 0.0;
    for coefficient in C.iter().rev() {
        sum = sum * z + coefficient;
    }
    1.0 - z / 2.0 + z * z * sum
}

/// `(sin, cos)` of an angle in 60000ths of a degree. Exact at quarter turns.
pub fn sin_cos(angle: f64) -> (f64, f64) {
    if !angle.is_finite() {
        return (0.0, 1.0);
    }
    let mut reduced = angle % TURN;
    if reduced < 0.0 {
        reduced += TURN;
    }
    let quadrant = (reduced / QUARTER).floor();
    let within = reduced - quadrant * QUARTER; // 0 <= within < QUARTER, exact
    let (s, c) = if within == 0.0 {
        (0.0, 1.0)
    } else if within <= EIGHTH {
        let x = within * RADIANS_PER_UNIT;
        (sin_kernel(x), cos_kernel(x))
    } else {
        let x = (QUARTER - within) * RADIANS_PER_UNIT;
        (cos_kernel(x), sin_kernel(x))
    };
    if quadrant < 1.0 {
        (s, c)
    } else if quadrant < 2.0 {
        (c, -s + 0.0)
    } else if quadrant < 3.0 {
        (-s + 0.0, -c + 0.0)
    } else {
        (-c + 0.0, s)
    }
}

/// `atan` on `0 <= t <= 1`, in radians. Three halvings with
/// `atan t = 2 atan(t / (1 + sqrt(1 + t^2)))` reach `|u| <= tan(pi/32)` (under 0.1),
/// where the series to u^17 leaves an error below 1e-20.
fn atan_unit(t: f64) -> f64 {
    let mut u = t;
    for _ in 0..3 {
        u /= 1.0 + (1.0 + u * u).sqrt();
    }
    let z = u * u;
    let mut sum = 0.0;
    for n in (1..=8).rev() {
        let term = 1.0 / f64::from(2 * n + 1);
        sum = (if n % 2 == 0 { term } else { -term }) + z * sum;
    }
    8.0 * (u + u * z * sum)
}

/// `atan2(y, x)` in 60000ths of a degree, in `[-10800000, 10800000]`. 0 when both
/// are 0, and exact on the axes and diagonals.
pub fn atan2(y: f64, x: f64) -> f64 {
    if y == 0.0 && x >= 0.0 || !y.is_finite() || !x.is_finite() {
        return 0.0;
    }
    let (ay, ax) = (y.abs(), x.abs());
    let first_quadrant = if ay == 0.0 {
        0.0
    } else if ax == 0.0 {
        QUARTER
    } else if ay == ax {
        EIGHTH
    } else if ay < ax {
        atan_unit(ay / ax) * UNITS_PER_RADIAN
    } else {
        QUARTER - atan_unit(ax / ay) * UNITS_PER_RADIAN
    };
    let half_plane = if x < 0.0 {
        2.0 * QUARTER - first_quadrant
    } else {
        first_quadrant
    };
    if y < 0.0 { -half_plane } else { half_plane }
}

/// `cos(atan2(z, y))` and `sin(atan2(z, y))`, without going through an angle: the
/// shape of `cat2` and `sat2`, exact where the ratio is. `(1, 0)` when both are 0,
/// matching `atan2(0, 0) = 0`.
pub fn unit_vector(y: f64, z: f64) -> (f64, f64) {
    let length = (y * y + z * z).sqrt();
    if length == 0.0 {
        (1.0, 0.0)
    } else {
        (y / length, z / length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEG: f64 = 60_000.0;

    #[test]
    fn quarter_turns_are_exact() {
        for k in -8i32..=8 {
            let a = f64::from(k) * QUARTER;
            let (s, c) = sin_cos(a);
            let want = [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)][k.rem_euclid(4) as usize];
            assert_eq!((s, c), want, "k={k}");
            assert_eq!(s.to_bits(), want.0.to_bits(), "no negative zero, k={k}");
            assert_eq!(c.to_bits(), want.1.to_bits(), "no negative zero, k={k}");
        }
    }

    #[test]
    fn known_trig_values() {
        let half_root3 = 3f64.sqrt() / 2.0;
        let half_root2 = 2f64.sqrt() / 2.0;
        for (degrees, s, c) in [
            (30.0, 0.5, half_root3),
            (45.0, half_root2, half_root2),
            (60.0, half_root3, 0.5),
            (120.0, half_root3, -0.5),
            (210.0, -0.5, -half_root3),
            (315.0, -half_root2, half_root2),
            (-30.0, -0.5, half_root3),
            (390.0, 0.5, half_root3),
        ] {
            let (gs, gc) = sin_cos(degrees * DEG);
            assert!((gs - s).abs() < 3e-16, "sin {degrees}: {gs} vs {s}");
            assert!((gc - c).abs() < 3e-16, "cos {degrees}: {gc} vs {c}");
        }
    }

    #[test]
    fn known_atan2_values() {
        for (y, x, degrees) in [
            (0.0, 0.0, 0.0),
            (0.0, 5.0, 0.0),
            (5.0, 0.0, 90.0),
            (0.0, -5.0, 180.0),
            (-5.0, 0.0, -90.0),
            (1.0, 1.0, 45.0),
            (-1.0, 1.0, -45.0),
            (1.0, -1.0, 135.0),
            (-1.0, -1.0, -135.0),
        ] {
            assert_eq!(atan2(y, x), degrees * DEG, "exact for atan2({y},{x})");
        }
        let root3 = 3f64.sqrt();
        assert!((atan2(1.0, root3) - 30.0 * DEG).abs() < 1e-8);
        assert!((atan2(root3, 1.0) - 60.0 * DEG).abs() < 1e-8);
        assert!((atan2(-root3, -1.0) + 120.0 * DEG).abs() < 1e-8);
    }

    /// The bit patterns are the reproducibility claim, pinned: a kernel change or a
    /// stray call into the platform library shows up here on whichever target
    /// disagrees. The full-table digest in tests/ extends this to all presets.
    #[test]
    fn bit_patterns_are_pinned() {
        let (s, c) = sin_cos(1_234_567.0);
        let a = atan2(3.0, 7.0);
        let got = [s.to_bits(), c.to_bits(), a.to_bits()];
        let want = [
            0x3fd6_7e2e_2187_3740,
            0x3fed_f566_87e4_27fb,
            0x4135_3d2b_6e4a_2599,
        ];
        assert_eq!(got, want, "{got:x?}");
    }

    /// Agreement with the platform library is a sanity check of accuracy, not of
    /// reproducibility, so it is tolerance-based and the platform calls stay here.
    #[test]
    fn agrees_with_the_platform_within_rounding() {
        // Within one turn each way: past that the platform's input, in radians, has
        // lost more than this reduction, which is exact, and the fault would be theirs.
        let mut a = -TURN;
        while a < TURN {
            let r = a * RADIANS_PER_UNIT;
            let (s, c) = sin_cos(a);
            assert!((s - r.sin()).abs() < 2e-15, "sin({a}) {s} vs {}", r.sin());
            assert!((c - r.cos()).abs() < 2e-15, "cos({a}) {c} vs {}", r.cos());
            a += 12_345.678;
        }
        let mut k = -200.0f64;
        while k < 200.0 {
            for scale in [1e-3, 1.0, 917.0, 1e9] {
                let (y, x) = (k * 0.37 * scale, (k * 1.13 + 3.1) * scale);
                let platform = y.atan2(x) * UNITS_PER_RADIAN;
                assert!((atan2(y, x) - platform).abs() < 1e-7, "atan2({y},{x})");
            }
            k += 0.77;
        }
    }

    #[test]
    fn saturate_cleans_everything() {
        assert_eq!(saturate(f64::NAN), 0.0);
        assert_eq!(saturate(f64::INFINITY), VALUE_LIMIT);
        assert_eq!(saturate(f64::NEG_INFINITY), -VALUE_LIMIT);
        assert_eq!(saturate(-0.0).to_bits(), 0.0f64.to_bits());
        assert_eq!(saturate(7.5), 7.5);
        assert!((VALUE_LIMIT * VALUE_LIMIT * 3.0).is_finite());
        assert_eq!(coordinate(VALUE_LIMIT), COORD_LIMIT);
        assert_eq!(coordinate(f64::NAN), 0.0);
    }

    #[test]
    fn hostile_angles_do_not_misbehave() {
        for a in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(sin_cos(a), (0.0, 1.0));
        }
        for a in [VALUE_LIMIT, -VALUE_LIMIT, COORD_LIMIT] {
            let (s, c) = sin_cos(a);
            assert!(s.abs() <= 1.0 && c.abs() <= 1.0);
        }
        assert_eq!(atan2(f64::NAN, 1.0), 0.0);
        assert_eq!(unit_vector(0.0, 0.0), (1.0, 0.0));
    }
}
