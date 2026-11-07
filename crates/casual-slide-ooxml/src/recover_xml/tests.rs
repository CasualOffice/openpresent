//! The recovering reader on what `walk` refuses. Whatever comes in, what comes out is what
//! `walk` admits; nothing is expanded; and the parse is one loop however deep the input.

use super::*;
use crate::{XmlTree, walk};

const MARKER: &str = "QzMarkerQz";

fn limits() -> OoxmlLimits {
    OoxmlLimits::default()
}

fn loose(input: &[u8]) -> RecoveredXml {
    let out = recover_xml(input, &limits(), Prune::Nothing);
    assert_admitted(&out, &limits());
    out
}

fn assert_admitted(out: &RecoveredXml, limits: &OoxmlLimits) {
    if out.has_root() {
        walk(&out.bytes, "t", limits, |_, _| Ok(())).unwrap_or_else(|err| {
            panic!("{err}: {:?}", String::from_utf8_lossy(&out.bytes));
        });
    } else {
        assert!(out.bytes.is_empty());
    }
}

fn text(out: &RecoveredXml) -> &str {
    std::str::from_utf8(&out.bytes).unwrap()
}

#[test]
fn a_document_type_is_skipped_and_never_interpreted() {
    let bomb = format!(
        "<?xml version=\"1.0\"?><!DOCTYPE r [<!ENTITY a \"{MARKER}\"><!ENTITY b \"&a;&a;&a;\">\
         <!ENTITY x SYSTEM \"http://evil.example/{MARKER}\">]><r v=\"&b;\">&b;&x;</r>"
    );
    let out = loose(bomb.as_bytes());
    assert!(out.repairs.contains(&XmlRepair::DoctypeSkipped));
    assert!(out.repairs.contains(&XmlRepair::ReferenceLiteral));
    let written = text(&out);
    // Nothing the entity holds, and nothing it names, reached the reading.
    assert!(!written.contains(MARKER), "{written}");
    assert!(!written.contains("ENTITY"), "{written}");
    // The references stand as the literal text they are.
    let tree = XmlTree::parse(&out.bytes, "t", &limits()).unwrap();
    assert_eq!(tree.own_text(tree.root()).as_deref(), Some("&b;&x;"));
    assert_eq!(
        tree.plain_attribute(tree.root(), "v").as_deref(),
        Some("&b;")
    );
}

#[test]
fn a_document_type_never_closed_costs_no_more_than_its_length() {
    let mut input = b"<!DOCTYPE r [".to_vec();
    input.extend(std::iter::repeat_n(b'[', 1 << 20));
    input.extend_from_slice(b"<r/>");
    let out = loose(&input);
    assert_eq!(out.root_local.as_deref(), Some("r"));
}

#[test]
fn broken_nesting_is_closed() {
    for (input, wanted) in [
        ("<a><b>", "<a><b></b></a>"),
        ("<a><b></a>", "<a><b></b></a>"),
        ("<a></b></a>", "<a></a>"),
        ("<a><b></c></b></a>", "<a><b></b></a>"),
        ("<a><b", "<a><b></b></a>"),
        ("<a><b x=\"1", "<a><b x=\"1\"></b></a>"),
        ("<a>< b/></a>", "<a>&lt; b/&gt;</a>"),
        ("<a>x<</a>", "<a>x&lt;</a>"),
    ] {
        let out = loose(input.as_bytes());
        assert_eq!(text(&out), wanted, "{input}");
        assert!(!out.repairs.is_empty(), "{input}");
    }
}

#[test]
fn more_than_one_root_keeps_the_first() {
    let out = loose(b"<a/><b>more</b>");
    assert_eq!(text(&out), "<a/>");
    assert!(out.repairs.contains(&XmlRepair::OutsideRootDropped));
    let out = loose(b"stray<a/>stray");
    assert_eq!(text(&out), "<a/>");
}

#[test]
fn bad_attributes_stay_readable_and_the_first_value_wins() {
    let out = loose(b"<a x=\"1\" x=\"2\" y=3 hidden z='q\"r' w=\"a<b\"/>");
    let tree = XmlTree::parse(&out.bytes, "t", &limits()).unwrap();
    let root = tree.root();
    assert_eq!(tree.plain_attribute(root, "x").as_deref(), Some("1"));
    assert_eq!(tree.plain_attribute(root, "y").as_deref(), Some("3"));
    assert_eq!(tree.plain_attribute(root, "hidden").as_deref(), Some(""));
    assert_eq!(tree.plain_attribute(root, "z").as_deref(), Some("q\"r"));
    assert!(out.repairs.contains(&XmlRepair::AttributeRepaired));
}

#[test]
fn attributes_run_together_are_separated() {
    let out = loose(b"<a x=\"1\"y=\"2\"/>");
    let tree = XmlTree::parse(&out.bytes, "t", &limits()).unwrap();
    assert_eq!(tree.plain_attribute(tree.root(), "y").as_deref(), Some("2"));
}

#[test]
fn a_repeat_by_expanded_name_is_left_out() {
    let out = loose(b"<r xmlns:x=\"urn:x\" xmlns:y=\"urn:x\" x:a=\"1\" y:a=\"2\"/>");
    assert!(text(&out).contains("x:a=\"1\""));
    assert!(!text(&out).contains("y:a"));
}

#[test]
fn an_undeclared_prefix_is_bound_so_it_stays_readable() {
    let out = loose(b"<a><q:x q:y=\"1\"/></a>");
    assert!(text(&out).contains("q:x"));
    // A standard prefix is bound to its own namespace, and the form follows the root's.
    let out =
        loose(b"<p:sld xmlns:p=\"http://purl.oclc.org/ooxml/presentationml/main\"><a:x/></p:sld>");
    assert!(
        text(&out).contains("xmlns:a=\"http://purl.oclc.org/ooxml/drawingml/main\""),
        "{}",
        text(&out)
    );
}

#[test]
fn forbidden_declarations_are_left_out() {
    let out = loose(
        b"<a xmlns:xmlns=\"u\" xmlns:xml=\"u\" xmlns:e=\"\" \
          xmlns=\"http://www.w3.org/2000/xmlns/\" xmlns:ok=\"u\"/>",
    );
    assert_eq!(text(&out), "<a xmlns:ok=\"u\"/>");
    let out = loose(b"<xmlns:a/>");
    assert_eq!(text(&out), "<xmlns_a/>");
}

#[test]
fn encodings_are_detected() {
    let encoded = |units: &str, mark: &[u8], word: fn(u16) -> [u8; 2]| {
        let mut b = mark.to_vec();
        units.encode_utf16().for_each(|u| b.extend(word(u)));
        b
    };
    let want = "<a>\u{e9}\u{4e16}</a>";
    for input in [
        // UTF-16 with its mark, truncated by one byte, and with none at all.
        encoded(want, &[0xFF, 0xFE], u16::to_le_bytes),
        encoded(want, &[0xFE, 0xFF], u16::to_be_bytes),
        encoded(want, &[], u16::to_le_bytes),
        encoded(want, &[], u16::to_be_bytes),
    ] {
        let out = loose(&input);
        assert_eq!(text(&out), want);
        let mut cut = input.clone();
        cut.pop();
        let out = loose(&cut);
        assert_eq!(out.root_local.as_deref(), Some("a"));
    }
    // A declared single-byte encoding, believed since the bytes are not UTF-8.
    let out = loose(b"<?xml version=\"1.0\" encoding=\"windows-1252\"?><a>caf\xE9 \x80</a>");
    assert_eq!(
        text(&out),
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><a>caf\u{e9} \u{20ac}</a>"
    );
    // UTF-8 declared as something else: the declaration is what is wrong.
    let out = loose("<?xml version=\"1.0\" encoding=\"UTF-16\"?><a>\u{e9}</a>".as_bytes());
    assert!(out.repairs.contains(&XmlRepair::TextRepaired));
    assert!(text(&out).contains("encoding=\"UTF-8\""));
    // UTF-32, and stray high bytes with no declaration.
    let mut utf32 = vec![0xFF, 0xFE, 0, 0];
    for c in "<a/>".chars() {
        utf32.extend((c as u32).to_le_bytes());
    }
    assert_eq!(text(&loose(&utf32)), "<a/>");
    assert_eq!(text(&loose(b"<a>x\xE9y</a>")), "<a>x\u{e9}y</a>");
}

#[test]
fn characters_xml_does_not_allow_go() {
    let out = loose(b"\xEF\xBB\xBF\xEF\xBB\xBF<a\0b>x\x01y</a\0b>");
    assert_eq!(text(&out), "<ab>xy</ab>");
    assert!(out.repairs.contains(&XmlRepair::TextRepaired));
}

#[test]
fn every_bound_stops_the_reading_and_keeps_what_was_read() {
    let lowered = OoxmlLimits {
        max_xml_depth: 3,
        ..limits()
    };
    let out = recover_xml(
        b"<a><b><c><d><e/></d></c></b></a>",
        &lowered,
        Prune::Nothing,
    );
    assert_admitted(&out, &lowered);
    assert_eq!(text(&out), "<a><b><c></c></b></a>");
    assert!(out.repairs.contains(&XmlRepair::LimitStopped));

    let lowered = OoxmlLimits {
        max_xml_elements: 2,
        ..limits()
    };
    let out = recover_xml(b"<a><b/><c/><d/></a>", &lowered, Prune::Nothing);
    assert_admitted(&out, &lowered);
    assert_eq!(text(&out), "<a><b/></a>");

    let lowered = OoxmlLimits {
        max_xml_attributes: 2,
        ..limits()
    };
    let out = recover_xml(b"<a x=\"1\" y=\"2\" z=\"3\"/>", &lowered, Prune::Nothing);
    assert_admitted(&out, &lowered);
    assert_eq!(text(&out), "<a x=\"1\" y=\"2\"/>");

    // A part over the size bound is not read at all.
    let lowered = OoxmlLimits {
        max_xml_part_bytes: 4,
        ..limits()
    };
    let out = recover_xml(b"<a></a>", &lowered, Prune::Nothing);
    assert!(!out.has_root());
    // A reading that grew past it is given up, rather than handed to `walk` to refuse.
    let lowered = OoxmlLimits {
        max_xml_part_bytes: 12,
        ..limits()
    };
    let out = recover_xml(b"<a>&&&&&</a>", &lowered, Prune::Nothing);
    assert!(!out.has_root());
}

#[test]
fn deep_nesting_uses_no_stack() {
    let mut input = Vec::new();
    for _ in 0..200_000 {
        input.extend_from_slice(b"<a>");
    }
    let out = loose(&input);
    assert!(out.repairs.contains(&XmlRepair::LimitStopped));
    assert_eq!(out.root_local.as_deref(), Some("a"));
}

/// What is outside any text the part holds does not appear in a repair.
#[test]
fn a_repair_holds_no_document_text() {
    let input = format!("<!DOCTYPE {MARKER}><{MARKER}:a {MARKER}={MARKER}>&{MARKER};</b>");
    let out = recover_xml(input.as_bytes(), &limits(), Prune::Nothing);
    let debug = format!("{:?}", out.repairs);
    assert!(!debug.contains(MARKER), "{debug}");
}

const SLIDE_OPEN: &str = "<p:sld xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" \
    xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"><p:cSld><p:spTree>\
    <p:nvGrpSpPr/><p:grpSpPr/>";
const SHAPE: &str = "<p:sp><p:nvSpPr/></p:sp>";

fn slide_like(input: &str) -> RecoveredXml {
    let out = recover_xml(input.as_bytes(), &limits(), Prune::SlideLike);
    assert_admitted(&out, &limits());
    out
}

#[test]
fn a_damaged_shape_is_left_out_and_its_neighbours_stay() {
    let good = "<p:sp><p:good/></p:sp>";
    let input =
        format!("{SLIDE_OPEN}{good}<p:sp><p:bad x=1/></p:sp>{good}</p:spTree></p:cSld></p:sld>");
    let out = slide_like(&input);
    assert_eq!(out.left_out, 1);
    let written = text(&out);
    assert_eq!(written.matches(good).count(), 2, "{written}");
    assert!(!written.contains("bad"), "{written}");
    assert!(out.repairs.contains(&XmlRepair::SubtreeLeftOut));
}

#[test]
fn an_unclosed_shape_takes_only_itself_and_what_it_swallowed() {
    let input = format!("{SLIDE_OPEN}{SHAPE}<p:sp>{SHAPE}</p:spTree></p:cSld></p:sld>");
    let out = slide_like(&input);
    assert_eq!(out.left_out, 1);
    assert_eq!(text(&out).matches("<p:sp>").count(), 1, "{}", text(&out));
}

#[test]
fn the_slides_frame_survives_damage_to_its_own_tags() {
    let input = format!("<p:sld xmlns:p=\"u\" x=1><p:cSld name=2><p:spTree>{SHAPE}");
    let out = slide_like(&input);
    let written = text(&out);
    assert!(
        written.starts_with("<p:sld xmlns:p=\"u\"><p:cSld><p:spTree>"),
        "{written}"
    );
    assert!(
        written.ends_with("</p:spTree></p:cSld></p:sld>"),
        "{written}"
    );
    // A shape before the cut stands.
    assert!(written.contains(SHAPE), "{written}");
}

#[test]
fn text_where_only_elements_belong_goes() {
    let input = format!("{SLIDE_OPEN}stray{SHAPE}</p:spTree></p:cSld></p:sld>");
    let written = slide_like(&input);
    assert!(!text(&written).contains("stray"));
}

#[test]
fn nothing_panics_at_any_cut_or_any_byte() {
    let input = format!(
        "<?xml version=\"1.0\"?><!DOCTYPE a><!-- c --><?pi x?>\
         {SLIDE_OPEN}<a:t x=\"&amp;&#65;\">t&lt;<![CDATA[c]]></a:t></p:spTree></p:cSld></p:sld>"
    );
    let bytes = input.as_bytes();
    for at in 0..bytes.len() {
        for prune in [Prune::Nothing, Prune::SlideLike] {
            for replacement in [b'<', b'>', b'&', b'"', b'/', b'\0', 0xFF, b':', b'='] {
                let mut bad = bytes.to_vec();
                bad[at] = replacement;
                assert_admitted(&recover_xml(&bad, &limits(), prune), &limits());
            }
            assert_admitted(&recover_xml(&bytes[..at], &limits(), prune), &limits());
            let mut gone = bytes.to_vec();
            gone.remove(at);
            assert_admitted(&recover_xml(&gone, &limits(), prune), &limits());
        }
    }
}

/// `XmlTree` stays byte-faithful: an untouched part, damaged or not, writes back what came.
#[test]
fn an_untouched_recovered_tree_writes_back_the_original_bytes() {
    for input in [
        &b"<a><b></a>"[..],
        b"<a x=1 x=2>&evil;</b>",
        b"<!DOCTYPE a [<!ENTITY e 'x'>]><a>&e;</a>",
        b"\xEF\xBB\xBF<a>\xE9</a>",
        b"<?xml version=\"1.0\" encoding=\"UTF-16\"?><a/>",
    ] {
        let (tree, repairs) = XmlTree::parse_recovering(input, "t", &limits()).unwrap();
        assert!(!repairs.is_empty());
        assert_eq!(tree.to_bytes(), input);
        walk(&tree.to_repaired_bytes(), "t", &limits(), |_, _| Ok(())).unwrap();
    }
    // Strict first: a healthy part never reaches the recovering reader.
    let healthy = b"<a  x = '1' ><!-- c --></a >";
    let (tree, repairs) = XmlTree::parse_recovering(healthy, "t", &limits()).unwrap();
    assert!(repairs.is_empty());
    assert_eq!(tree.to_bytes(), healthy);
}

#[test]
fn an_edited_recovered_tree_writes_well_formed_text() {
    let (mut tree, _) = XmlTree::parse_recovering(b"<a><b></a>", "t", &limits()).unwrap();
    let root = tree.root();
    assert!(tree.set_attribute(root, "k", "v"));
    let written = tree.to_bytes();
    assert_eq!(written, b"<a k=\"v\"><b></b></a>");
    walk(&written, "t", &limits(), |_, _| Ok(())).unwrap();
}
