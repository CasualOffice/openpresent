//! The written forms the model accepts, by the schema's rules rather than Rust's.

use casual_slide_model::presentation::{Coordinate, Percentage, SlideSizeType};
use casual_slide_model::properties::{Float32, Float64, Lexical, W3cDateTime};

#[test]
fn booleans_take_the_four_schema_forms_and_no_other() {
    for (s, v) in [
        ("1", true),
        ("true", true),
        ("0", false),
        (" false\n", false),
    ] {
        assert_eq!(bool::parse_lexical(s), Some(v), "{s:?}");
    }
    for s in ["", "yes", "TRUE", "2", "on"] {
        assert_eq!(bool::parse_lexical(s), None, "{s:?}");
    }
    assert_eq!(true.to_lexical(), "1");
}

#[test]
fn integers_refuse_overflow_rather_than_wrap() {
    assert_eq!(i32::parse_lexical("99999999999"), None);
    assert_eq!(u32::parse_lexical("-1"), None);
    assert_eq!(u32::parse_lexical("4294967296"), None);
    assert_eq!(i8::parse_lexical("128"), None);
    assert_eq!(i32::parse_lexical(" 7 "), Some(7));
    // Unicode space is not schema whitespace.
    assert_eq!(i32::parse_lexical("\u{a0}7"), None);
}

#[test]
fn floats_take_the_schema_forms_and_a_nan_equals_itself() {
    for s in ["inf", "infinity", "nan", "Inf", "NAN", "0x10", ""] {
        assert_eq!(Float64::parse_lexical(s), None, "{s:?}");
    }
    let nan = Float64::parse_lexical("NaN").unwrap();
    assert_eq!(nan, Float64::new(f64::NAN));
    assert_eq!(nan.to_lexical(), "NaN");
    assert_eq!(Float32::parse_lexical("-INF").unwrap().to_lexical(), "-INF");
    assert_eq!(Float64::parse_lexical("1E3").unwrap().get(), 1000.0);
    assert_eq!(Float64::parse_lexical("1e999").unwrap().to_lexical(), "INF");
}

#[test]
fn percentages_read_both_forms_of_the_union() {
    for (s, v) in [
        ("50000", 50_000),
        ("50%", 50_000),
        ("15.62%", 15_620),
        ("-1.5%", -1500),
        ("0.001%", 1),
    ] {
        assert_eq!(Percentage::parse_lexical(s), Some(Percentage(v)), "{s:?}");
    }
    for s in [
        "",
        "%",
        "lots",
        "1.%",
        ".5%",
        "9999999999%",
        "99999999999",
        "1-%",
        "--1%",
    ] {
        assert_eq!(Percentage::parse_lexical(s), None, "{s:?}");
    }
    assert_eq!(Percentage(15_620).to_percent_form(), "15.62%");
    assert_eq!(Percentage(50_000).to_percent_form(), "50%");
    assert_eq!(Percentage(-1500).to_percent_form(), "-1.5%");
    assert_eq!(Percentage(i32::MIN).to_percent_form(), "-2147483.648%");
}

#[test]
fn coordinates_read_units_and_refuse_nonsense() {
    for (s, v) in [
        ("914400", 914_400),
        ("1in", 914_400),
        ("2.5cm", 900_000),
        ("-1pt", -12_700),
        ("1pc", 152_400),
        ("1pi", 152_400),
        ("10mm", 360_000),
    ] {
        assert_eq!(Coordinate::parse_lexical(s), Some(Coordinate(v)), "{s:?}");
    }
    for s in [
        "",
        "in",
        "1e3in",
        "1.in",
        ".5in",
        "--1in",
        "1zz",
        "é",
        "1é",
        "99999999999999999999",
        "1e400in",
    ] {
        assert_eq!(Coordinate::parse_lexical(s), None, "{s:?}");
    }
    let huge = format!("{}in", "9".repeat(400));
    assert_eq!(Coordinate::parse_lexical(&huge), None);
}

#[test]
fn dates_are_checked_against_the_calendar() {
    for s in [
        "2020",
        "2020-02",
        "2020-02-29",
        "2020-02-29T23:59Z",
        "2020-02-29T23:59:60Z",
        "2021-02-03T04:05:06.5+05:30",
        "2021-02-03T04:05:06",
    ] {
        assert!(W3cDateTime::parse_lexical(s).is_some(), "{s:?}");
    }
    for s in [
        "",
        "yesterday",
        "2021-02-29",
        "2020-13",
        "2020-00",
        "2020-01-00",
        "2020-01-32",
        "2020-01-01T24:00Z",
        "2020-01-01T00:60Z",
        "2020-01-01T00:00:61Z",
        "2020-01-01T00:00:00.Z",
        "2020-01-01T00:00:00+15:00",
        "2020-01-01T00:00:00X",
        "20é0",
        "2020-é",
    ] {
        assert!(W3cDateTime::parse_lexical(s).is_none(), "{s:?}");
    }
    assert_eq!(
        W3cDateTime::parse_lexical(" 2020 ").unwrap().as_str(),
        "2020"
    );
    assert!(!W3cDateTime::is_full_date_time("2020"));
    assert!(W3cDateTime::is_full_date_time("2020-01-02T03:04:05Z"));
}

#[test]
fn tokens_match_exactly() {
    assert_eq!(SlideSizeType::parse_lexical("A4"), Some(SlideSizeType::A4));
    assert_eq!(SlideSizeType::parse_lexical("a4"), None);
    assert_eq!(SlideSizeType::Mm35.to_lexical(), "35mm");
}
