//! Export patches named nodes: edit one property, save, reopen, and only that node differs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use casual_slide_export::presentation::save_deck;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::ImportLimits;
use casual_slide_model::presentation::{
    Coordinate, CustomShow, Deck, Guide, Percentage, PrintWhat, ShowType, SlideId, SlideListChoice,
    SlideSize, SlideSizeType, ViewType,
};
use casual_slide_model::properties::{
    CoreProperties, CustomProperty, Lexical, Variant, VectorBaseType, VtVector, W3cDateTime,
};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};
use casual_slide_package::PartEdits;

const RICH: &str = "generated/rich-properties.pptx";
const STRICT: &str = "generated/strict-properties.pptx";
const FMTID: &str = "{D5CDD505-2E9C-101B-9397-08002B2CF9AE}";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn limits() -> ImportLimits {
    ImportLimits::default()
}

fn model(bytes: &[u8]) -> Deck {
    import_deck(&mut open(bytes.to_vec()), &limits())
        .unwrap()
        .deck
}

fn parts(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut pkg = open(bytes.to_vec());
    pkg.part_names()
        .into_iter()
        .map(|n| {
            let b = pkg.read_part(&n).unwrap();
            (n, b)
        })
        .collect()
}

/// Edit, save, reopen. Returns the new bytes, having checked the value reads back, a
/// second save of the same model changes nothing, and every part not named is identical.
fn edit(source: &[u8], change: impl FnOnce(&mut Deck)) -> Vec<u8> {
    let mut pkg = open(source.to_vec());
    let mut deck = import_deck(&mut pkg, &limits()).unwrap().deck;
    change(&mut deck);
    let outcome = save_deck(&mut pkg, &deck, &limits()).unwrap();
    assert_eq!(model(&outcome.bytes), deck, "the edit reads back");

    let mut again = open(outcome.bytes.clone());
    let second = save_deck(&mut again, &deck, &limits()).unwrap();
    assert_eq!(second.bytes, outcome.bytes, "fixed point");
    assert!(second.changed_parts.is_empty());

    let (old, new) = (parts(source), parts(&outcome.bytes));
    for (name, bytes) in &old {
        if !outcome.changed_parts.contains(name) {
            assert_eq!(new.get(name), Some(bytes), "{name} changed");
        }
    }
    assert_eq!(
        old.len()
            + outcome
                .changed_parts
                .iter()
                .filter(|p| !old.contains_key(*p))
                .count(),
        new.len(),
        "parts were lost or invented"
    );
    outcome.bytes
}

/// `from` occurs once in `part`, and the edit turned it into `to` and did nothing else.
fn assert_only_this_changed(source: &[u8], edited: &[u8], part: &str, from: &str, to: &str) {
    let old = String::from_utf8(parts(source)[part].clone()).unwrap();
    let new = String::from_utf8(parts(edited)[part].clone()).unwrap();
    assert_eq!(old.matches(from).count(), 1, "{from} in {old}");
    assert_eq!(new, old.replace(from, to));
    let changed: Vec<String> = {
        let (a, b) = (parts(source), parts(edited));
        b.into_iter()
            .filter(|(n, v)| a.get(n) != Some(v))
            .map(|(n, _)| n)
            .collect()
    };
    assert_eq!(changed, [part]);
}

const CORE: &str = "docProps/core.xml";
const APP: &str = "docProps/app.xml";
const CUSTOM: &str = "docProps/custom.xml";
const MAIN: &str = "ppt/presentation.xml";
const PROPS: &str = "ppt/presProps.xml";
const VIEW: &str = "ppt/viewProps.xml";

type Change = fn(&mut Deck);

/// One edit of each kind of node, on the deck full of things a parser would lose.
#[test]
fn an_edit_changes_only_that_node_and_every_other_part_stays_identical() {
    let cases: &[(&str, &str, &str, Change)] = &[
        (
            CORE,
            "<dc:title>Old &amp; title</dc:title>",
            "<dc:title>New &lt;title&gt;</dc:title>",
            |d| {
                d.properties.core.as_mut().unwrap().title = Some("New <title>".to_owned());
            },
        ),
        (
            CORE,
            "<cp:revision>12</cp:revision>",
            "<cp:revision>13</cp:revision>",
            |d| {
                d.properties.core.as_mut().unwrap().revision = Some("13".to_owned());
            },
        ),
        (
            CORE,
            ">2021-02-03T04:05:06.5+05:30<",
            ">2022-01-01T00:00:00Z<",
            |d| {
                d.properties.core.as_mut().unwrap().modified =
                    W3cDateTime::parse_lexical("2022-01-01T00:00:00Z");
            },
        ),
        (CORE, "  <dc:subject>Subject</dc:subject>\n", "  \n", |d| {
            d.properties.core.as_mut().unwrap().subject = None;
        }),
        (
            APP,
            "<Company>Co &amp; Co</Company>",
            "<Company>Acme</Company>",
            |d| {
                d.properties.app.as_mut().unwrap().company = Some("Acme".to_owned());
            },
        ),
        (APP, "<Slides>3</Slides>", "<Slides>4</Slides>", |d| {
            d.properties.app.as_mut().unwrap().slides = Some(4);
        }),
        (
            APP,
            r#"<vt:vector size="1" baseType="lpstr"><vt:lpstr>One</vt:lpstr></vt:vector>"#,
            r#"<vt:vector size="2" baseType="lpstr"><vt:lpstr>Uno</vt:lpstr><vt:lpstr>Dos</vt:lpstr></vt:vector>"#,
            |d| {
                d.properties.app.as_mut().unwrap().titles_of_parts = Some(VtVector {
                    base_type: VectorBaseType::Lpstr,
                    items: vec![
                        Variant::Lpstr("Uno".to_owned()),
                        Variant::Lpstr("Dos".to_owned()),
                    ],
                });
            },
        ),
        (
            CUSTOM,
            "<vt:lpwstr>text</vt:lpwstr>",
            "<vt:lpwstr>changed</vt:lpwstr>",
            |d| {
                d.properties.custom.as_mut().unwrap().properties[0].value =
                    Some(Variant::Lpwstr("changed".to_owned()));
            },
        ),
        (MAIN, "firstSlideNum=\"5\"", "firstSlideNum=\"6\"", |d| {
            d.presentation.first_slide_num = Some(6);
        }),
        (MAIN, " rtl=\"false\"", "", |d| d.presentation.rtl = None),
        (MAIN, "serverZoom='80%'", "serverZoom='75%'", |d| {
            d.presentation.server_zoom = Some(Percentage(75_000));
        }),
        (
            MAIN,
            r#"<p:sldSz cx="12192000" cy="6858000" type="screen16x9"/>"#,
            r#"<p:sldSz cx="9144000" cy="6858000" type="screen4x3"/>"#,
            |d| {
                d.presentation.slide_size = Some(SlideSize {
                    cx: 9_144_000,
                    cy: 6_858_000,
                    size_type: Some(SlideSizeType::Screen4x3),
                });
            },
        ),
        (
            MAIN,
            r#"invalStChars="!)""#,
            r#"invalStChars="a&quot;b""#,
            |d| {
                d.presentation.kinsoku.as_mut().unwrap().invalid_start_chars = "a\"b".to_owned();
            },
        ),
        (
            MAIN,
            r#"<p:sld r:id="rId5"/><p:sld r:id="rId3"/>"#,
            r#"<p:sld r:id="rId3"/><p:sld r:id="rId5"/>"#,
            |d| {
                d.presentation.custom_shows.as_mut().unwrap()[0]
                    .slides
                    .reverse()
            },
        ),
        (MAIN, r#"<p:photoAlbum bw="1" "#, r#"<p:photoAlbum "#, |d| {
            d.presentation.photo_album.as_mut().unwrap().bw = None;
        }),
        (PROPS, r#"prnWhat="handouts6""#, r#"prnWhat="notes""#, |d| {
            let props = d.presentation_properties.as_mut().unwrap();
            props.print.as_mut().unwrap().print_what = Some(PrintWhat::Notes);
        }),
        (PROPS, r#"restart="60000""#, r#"restart="5""#, |d| {
            let props = d.presentation_properties.as_mut().unwrap();
            props.show.as_mut().unwrap().show_type = Some(ShowType::Kiosk { restart: Some(5) });
        }),
        (
            PROPS,
            r#"<p:custShow id="3"/>"#,
            r#"<p:sldRg st="1" end="2"/>"#,
            |d| {
                let props = d.presentation_properties.as_mut().unwrap();
                props.show.as_mut().unwrap().slide_list =
                    Some(SlideListChoice::Range { start: 1, end: 2 });
            },
        ),
        (
            VIEW,
            r#"lastView="sldThumbnailView""#,
            r#"lastView="notesView""#,
            |d| {
                d.view_properties.as_mut().unwrap().last_view = Some(ViewType::NotesView);
            },
        ),
        (VIEW, r#"sz="15620""#, r#"sz="20000""#, |d| {
            let normal = d.view_properties.as_mut().unwrap().normal.as_mut().unwrap();
            normal.restored_left.as_mut().unwrap().size = Percentage(20_000);
        }),
        (
            VIEW,
            r#"<p:gridSpacing cx="76200""#,
            r#"<p:gridSpacing cx="1""#,
            |d| {
                d.view_properties
                    .as_mut()
                    .unwrap()
                    .grid_spacing
                    .as_mut()
                    .unwrap()
                    .cx = 1;
            },
        ),
    ];
    let source = read(RICH);
    for &(part, from, to, change) in cases {
        let edited = edit(&source, change);
        assert_only_this_changed(&source, &edited, part, from, to);
    }
}

/// Repeated elements need the first one, so these use replace on a fragment that recurs.
#[test]
fn an_edit_inside_a_repeated_structure_changes_one_node() {
    let source = read(RICH);
    let edited = edit(&source, |d| {
        let view = d.view_properties.as_mut().unwrap();
        let guides = view.slide.as_mut().unwrap().guides.as_mut().unwrap();
        guides[0].pos = Some(Coordinate(100));
    });
    let old = String::from_utf8(parts(&source)[VIEW].clone()).unwrap();
    let new = String::from_utf8(parts(&edited)[VIEW].clone()).unwrap();
    assert_eq!(new, old.replacen(r#"pos="2160""#, r#"pos="100""#, 1));
    assert_ne!(new, old);
}

#[test]
fn a_slide_reorder_moves_elements_and_leaves_what_is_between_them() {
    let source = read(RICH);
    let edited = edit(&source, |d| {
        d.presentation.slides.as_mut().unwrap().swap(0, 2);
    });
    let new = String::from_utf8(parts(&edited)[MAIN].clone()).unwrap();
    assert!(new.contains(
        "<p:sldIdLst>\n    <p:sldId id=\"258\" r:id=\"rId5\"/>\n    <!-- between -->\n    <p:sldId id=\"257\" r:id=\"rId4\"><p:extLst><p:ext uri=\"{K}\"><x:y/></p:ext></p:extLst></p:sldId>\n    <p:sldId r:id=\"rId3\" id=\"256\"/>\n  </p:sldIdLst>"
    ), "{new}");
}

#[test]
fn changing_a_slide_id_keeps_what_the_entry_holds_beyond_the_model() {
    let source = read(RICH);
    let edited = edit(&source, |d| {
        d.presentation.slides.as_mut().unwrap()[1].id = 900;
    });
    assert_only_this_changed(
        &source,
        &edited,
        MAIN,
        r#"id="257" r:id="rId4"><p:extLst>"#,
        r#"id="900" r:id="rId4"><p:extLst>"#,
    );
}

#[test]
fn removing_a_slide_entry_leaves_the_rest_in_place() {
    let source = read(RICH);
    let edited = edit(&source, |d| {
        d.presentation.slides.as_mut().unwrap().remove(1);
    });
    let new = String::from_utf8(parts(&edited)[MAIN].clone()).unwrap();
    assert!(new.contains(r#"<p:sldId r:id="rId3" id="256"/>"#));
    assert!(!new.contains("rId4\""));
    assert!(new.contains("<!-- between -->"));
}

#[test]
fn a_new_attribute_and_element_land_in_schema_order() {
    let source = read("generated/minimal.pptx");
    let edited = edit(&source, |d| {
        d.presentation.embed_true_type_fonts = Some(true);
        d.presentation.kinsoku = Some(casual_slide_model::presentation::Kinsoku {
            lang: None,
            invalid_start_chars: "a".to_owned(),
            invalid_end_chars: "b".to_owned(),
        });
        d.presentation.notes_masters = Some(vec![]);
    });
    let new = String::from_utf8(parts(&edited)[MAIN].clone()).unwrap();
    assert!(new.contains(r#"embedTrueTypeFonts="1">"#), "{new}");
    let at = |needle: &str| {
        new.find(needle)
            .unwrap_or_else(|| panic!("{needle}: {new}"))
    };
    assert!(at("<p:sldMasterIdLst>") < at("<p:notesMasterIdLst"));
    assert!(at("<p:notesMasterIdLst") < at("<p:sldIdLst>"));
    assert!(at("<p:notesSz") < at("<p:kinsoku"));
}

#[test]
fn new_customer_data_goes_before_tags() {
    let source = read(RICH);
    let edited = edit(&source, |d| {
        let data = d.presentation.customer_data.as_mut().unwrap();
        data.customer_data.push("rId5".to_owned());
    });
    assert_only_this_changed(
        &source,
        &edited,
        MAIN,
        r#"<p:custData r:id="rId10"/><p:tags r:id="rId8"/>"#,
        r#"<p:custData r:id="rId10"/><p:custData r:id="rId5"/><p:tags r:id="rId8"/>"#,
    );
}

#[test]
fn a_new_property_part_is_wired_in_and_holds_only_what_was_set() {
    let source = read("generated/minimal.pptx");
    let mut deck = model(&source);
    assert_eq!(deck.properties.core, None);
    let edited = edit(&source, |d| {
        d.properties.core = Some(CoreProperties {
            title: Some("Fresh".to_owned()),
            created: W3cDateTime::parse_lexical("2020-01-02T03:04:05Z"),
            ..CoreProperties::default()
        });
        d.properties.custom = Some(casual_slide_model::properties::CustomProperties {
            properties: vec![CustomProperty {
                fmtid: FMTID.to_owned(),
                pid: 2,
                name: Some("K".to_owned()),
                link_target: None,
                value: Some(Variant::Lpwstr("V".to_owned())),
            }],
        });
    });
    let all = parts(&edited);
    let core = String::from_utf8(all[CORE].clone()).unwrap();
    assert!(core.contains("<dc:title>Fresh</dc:title>"), "{core}");
    assert!(
        core.contains(r#"<dcterms:created xsi:type="dcterms:W3CDTF">2020"#),
        "{core}"
    );
    assert!(!core.contains("creator"), "nothing materialized: {core}");
    let types = String::from_utf8(all["[Content_Types].xml"].clone()).unwrap();
    assert!(types.contains(r#"<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>"#), "{types}");
    let rels = String::from_utf8(all["_rels/.rels"].clone()).unwrap();
    assert!(rels.contains(r#"Target="docProps/core.xml""#), "{rels}");
    assert!(rels.contains(r#"Target="docProps/custom.xml""#), "{rels}");
    // The other absent parts stay absent.
    assert!(!all.contains_key(APP));
    assert!(!all.contains_key(PROPS));
    deck.properties.core = None;
    assert_eq!(deck.properties.core, None);
}

#[test]
fn a_new_presentation_properties_part_is_wired_in() {
    let source = read("generated/minimal.pptx");
    let edited = edit(&source, |d| {
        d.view_properties = Some(casual_slide_model::presentation::ViewProperties {
            last_view: Some(ViewType::OutlineView),
            ..Default::default()
        });
    });
    let all = parts(&edited);
    assert!(String::from_utf8_lossy(&all[VIEW]).contains(r#"lastView="outlineView""#));
    let rels = String::from_utf8(all["ppt/_rels/presentation.xml.rels"].clone()).unwrap();
    assert!(rels.contains(r#"Target="viewProps.xml""#), "{rels}");
}

#[test]
fn a_custom_property_can_be_added_changed_in_type_and_removed() {
    let source = read(RICH);
    let edited = edit(&source, |d| {
        let list = &mut d.properties.custom.as_mut().unwrap().properties;
        list[1].value = Some(Variant::Bool(true));
        list.remove(2);
        list.push(CustomProperty {
            fmtid: FMTID.to_owned(),
            pid: 1000,
            name: Some("New".to_owned()),
            link_target: None,
            value: Some(Variant::Vector(VtVector {
                base_type: VectorBaseType::Variant,
                items: vec![Variant::Variant(Box::new(Variant::Empty))],
            })),
        });
    });
    let new = String::from_utf8(parts(&edited)[CUSTOM].clone()).unwrap();
    assert!(
        new.contains(r#"name="i1"><vt:bool>1</vt:bool></property>"#),
        "{new}"
    );
    assert!(!new.contains(r#"name="i2""#));
    assert!(new.contains(
        r#"pid="1000" name="New"><vt:vector baseType="variant" size="1"><vt:variant><vt:empty/></vt:variant></vt:vector></property>"#
    ), "{new}");
}

#[test]
fn keywords_keep_their_values_when_only_text_changes_and_lose_them_when_asked() {
    let source = read(RICH);
    let edited = edit(&source, |d| {
        d.properties
            .core
            .as_mut()
            .unwrap()
            .keywords
            .as_mut()
            .unwrap()
            .lang = Some("fr".to_owned());
    });
    assert_only_this_changed(
        &source,
        &edited,
        CORE,
        r#"<cp:keywords xml:lang="en""#,
        r#"<cp:keywords xml:lang="fr""#,
    );
    let edited = edit(&source, |d| {
        let k = d
            .properties
            .core
            .as_mut()
            .unwrap()
            .keywords
            .as_mut()
            .unwrap();
        k.text = "x".to_owned();
        k.values.clear();
    });
    assert!(
        String::from_utf8_lossy(&parts(&edited)[CORE])
            .contains(r#"<cp:keywords xml:lang="en">x</cp:keywords>"#)
    );
}

#[test]
fn whitespace_and_control_like_characters_round_trip() {
    let source = read(RICH);
    let tricky = "tab\there\nline\rcr \u{e9} <&>\"' ]]>";
    edit(&source, |d| {
        d.properties.core.as_mut().unwrap().title = Some(tricky.to_owned());
        d.presentation.kinsoku.as_mut().unwrap().invalid_start_chars = tricky.to_owned();
        d.presentation.custom_shows.as_mut().unwrap()[0].name = tricky.to_owned();
    });
}

#[test]
fn an_unedited_model_returns_the_input_for_every_deck_that_opens() {
    let manifest = std::fs::read_to_string(fixtures().join("manifest.tsv")).unwrap();
    let mut seen = 0;
    for line in manifest.lines().skip(1) {
        let mut cols = line.split('\t');
        let path = cols.next().unwrap();
        let expect = cols.nth(1).unwrap();
        if !expect.starts_with("open") || path.starts_with("hostile/import-limit") {
            continue;
        }
        let source = read(path);
        let mut pkg = open(source.clone());
        let deck = import_deck(&mut pkg, &limits()).unwrap().deck;
        let outcome =
            save_deck(&mut pkg, &deck, &limits()).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(outcome.bytes, source, "{path}");
        assert!(outcome.changed_parts.is_empty(), "{path}");
        assert_eq!(model(&outcome.bytes), deck, "{path}");
        seen += 1;
    }
    assert!(seen > 20);
}

#[test]
fn a_producer_deck_takes_an_edit() {
    for path in [
        "producers/libreoffice-26.8-image.pptx",
        "producers/libreoffice-26.8-three-slides.pptx",
    ] {
        let source = read(path);
        let edited = edit(&source, |d| {
            d.properties.core.as_mut().unwrap().title = Some("Edited".to_owned());
        });
        assert_only_this_changed(
            &source,
            &edited,
            CORE,
            "<dc:title></dc:title>",
            "<dc:title>Edited</dc:title>",
        );
        let edited = edit(&source, |d| {
            let props = d.presentation_properties.as_mut().unwrap();
            props.show.as_mut().unwrap().loop_show = Some(true);
        });
        assert_only_this_changed(
            &source,
            &edited,
            PROPS,
            r#"showNarration="1">"#,
            r#"showNarration="1" loop="1">"#,
        );
    }
}

#[test]
fn strict_decks_are_written_strict() {
    let source = read(STRICT);
    let edited = edit(&source, |d| {
        d.presentation.server_zoom = Some(Percentage(12_500));
        d.view_properties
            .as_mut()
            .unwrap()
            .normal
            .as_mut()
            .unwrap()
            .restored_top = None;
    });
    assert!(String::from_utf8_lossy(&parts(&edited)[MAIN]).contains("serverZoom='12.5%'"));
    let edited = edit(&source, |d| {
        let normal = d.view_properties.as_mut().unwrap().normal.as_mut().unwrap();
        normal.restored_left.as_mut().unwrap().size = Percentage(20_000);
    });
    assert_only_this_changed(&source, &edited, VIEW, r#"sz="15.62%""#, r#"sz="20%""#);

    let minimal_strict = {
        // Parts wired in on a Strict deck use Strict's names.
        let mut pkg = open(source.clone());
        let mut edits = PartEdits::default();
        let rels = String::from_utf8(pkg.read_part("_rels/.rels").unwrap()).unwrap();
        let stripped: String = rels
            .split_inclusive("/>")
            .filter(|r| !r.contains("Target=\"docProps/custom.xml\""))
            .collect();
        edits.replace("_rels/.rels", stripped.into_bytes());
        edits.remove(CUSTOM);
        pkg.save_with(&edits).unwrap()
    };
    let wired = edit(&minimal_strict, |d| {
        d.properties.custom = Some(casual_slide_model::properties::CustomProperties {
            properties: vec![CustomProperty {
                fmtid: FMTID.to_owned(),
                pid: 2,
                name: None,
                link_target: None,
                value: Some(Variant::I4(1)),
            }],
        });
    });
    let all = parts(&wired);
    assert!(
        String::from_utf8_lossy(&all["_rels/.rels"])
            .contains("purl.oclc.org/ooxml/officeDocument/relationships/customProperties")
    );
    let custom = String::from_utf8_lossy(&all[CUSTOM]).into_owned();
    assert!(
        custom.contains("purl.oclc.org/ooxml/officeDocument/customProperties"),
        "{custom}"
    );
}

fn code_of(source: &[u8], change: impl FnOnce(&mut Deck)) -> &'static str {
    let mut pkg = open(source.to_vec());
    let mut deck = import_deck(&mut pkg, &limits()).unwrap().deck;
    change(&mut deck);
    save_deck(&mut pkg, &deck, &limits()).unwrap_err().code()
}

#[test]
fn a_value_that_would_corrupt_the_deck_is_refused_and_nothing_is_saved() {
    let source = read(RICH);
    assert_eq!(
        code_of(&source, |d| {
            d.properties.core.as_mut().unwrap().title = Some("bell\u{7}".to_owned());
        }),
        "CS-EXP-0001"
    );
    assert_eq!(
        code_of(&source, |d| {
            d.presentation.kinsoku.as_mut().unwrap().invalid_end_chars = "\u{FFFF}".to_owned();
        }),
        "CS-EXP-0001"
    );
    assert_eq!(
        code_of(&source, |d| {
            d.presentation.slides.as_mut().unwrap()[0].id = 5;
        }),
        "CS-EXP-0002"
    );
    assert_eq!(
        code_of(&source, |d| {
            d.presentation.slides.as_mut().unwrap().push(SlideId {
                id: 2_147_483_648,
                rel_id: "rId3".to_owned(),
            });
        }),
        "CS-EXP-0002"
    );
    assert_eq!(
        code_of(&source, |d| {
            d.presentation.slide_masters.as_mut().unwrap()[0].id = Some(300);
        }),
        "CS-EXP-0002"
    );
    assert_eq!(
        code_of(&source, |d| {
            d.properties.custom.as_mut().unwrap().properties[0].value =
                Some(Variant::Cy("nope".to_owned()));
        }),
        "CS-EXP-0002"
    );
    assert_eq!(
        code_of(&source, |d| {
            d.presentation.slides.as_mut().unwrap().push(SlideId {
                id: 999,
                rel_id: "rId404".to_owned(),
            });
        }),
        "CS-EXP-0003"
    );
    assert_eq!(
        code_of(&source, |d| d.properties.core = None),
        "CS-EXP-0005"
    );
    // Out of the type's range: written, read back as absent, and so refused.
    assert_eq!(
        code_of(&source, |d| {
            d.presentation.slide_size.as_mut().unwrap().cx = 1;
        }),
        "CS-EXP-0007"
    );
    assert_eq!(
        code_of(&source, |d| {
            d.presentation.bookmark_id_seed = Some(0);
        }),
        "CS-EXP-0007"
    );
    assert_eq!(
        code_of(&source, |d| {
            d.properties.custom.as_mut().unwrap().properties[0].fmtid = "nope".to_owned();
        }),
        "CS-EXP-0007"
    );
    let mut deep = Variant::Null;
    for _ in 0..40 {
        deep = Variant::Variant(Box::new(deep));
    }
    assert_eq!(
        code_of(&source, move |d| {
            d.properties.custom.as_mut().unwrap().properties[0].value = Some(deep);
        }),
        "CS-EXP-0008"
    );
    // A custom show naming no relationship.
    assert_eq!(
        code_of(&source, |d| {
            d.presentation
                .custom_shows
                .as_mut()
                .unwrap()
                .push(CustomShow {
                    name: "n".to_owned(),
                    id: 9,
                    slides: vec!["rId404".to_owned()],
                });
        }),
        "CS-EXP-0003"
    );
}

#[test]
fn a_part_that_cannot_be_read_is_left_alone_and_not_written_over() {
    let source = read(RICH);
    let mut pkg = open(source);
    let mut edits = PartEdits::default();
    edits.replace(CORE, b"<workbook/>".to_vec());
    let broken = pkg.save_with(&edits).unwrap();
    // Editing elsewhere leaves it as it is.
    let edited = edit(&broken, |d| d.presentation.first_slide_num = Some(9));
    assert_eq!(parts(&edited)[CORE], b"<workbook/>");
    // A value for it is refused, rather than replacing what is there.
    assert_eq!(
        code_of(&broken, |d| d.properties.core =
            Some(CoreProperties::default())),
        "CS-EXP-0006"
    );
}

#[test]
fn hostile_values_stay_in_the_file_through_an_edit_elsewhere() {
    let source = read("hostile/import-bad-values.bin");
    let edited = edit(&source, |d| d.presentation.first_slide_num = Some(3));
    let old = String::from_utf8(parts(&source)[MAIN].clone()).unwrap();
    let new = String::from_utf8(parts(&edited)[MAIN].clone()).unwrap();
    assert_eq!(
        new,
        old.replace(
            r#"firstSlideNum="99999999999999999999""#,
            r#"firstSlideNum="3""#
        )
    );
    // Other bad values are still there, as written.
    assert!(new.contains(r#"serverZoom="lots""#));
    assert!(new.contains(r#"<p:sldId id="4294967296" r:id="rId5"/>"#));
    // A property edit in a part full of them.
    let edited = edit(&source, |d| {
        d.properties
            .custom
            .as_mut()
            .unwrap()
            .properties
            .retain(|p| p.pid != 4);
    });
    let new = String::from_utf8(parts(&edited)[CUSTOM].clone()).unwrap();
    assert!(
        new.contains("<property/>"),
        "an entry the model cannot see stays"
    );
    assert!(!new.contains("pid=\"4\""));
}

/// Found by fuzzing: with `showPr` repeated, removing the first let the second appear.
#[test]
fn removing_a_repeated_element_removes_every_copy() {
    let source = read(RICH);
    let mut pkg = open(source);
    let mut edits = PartEdits::default();
    let props = String::from_utf8(pkg.read_part(PROPS).unwrap()).unwrap();
    let doubled = props.replace(
        "</p:presentationPr>",
        r#"<p:showPr loop="0"/></p:presentationPr>"#,
    );
    edits.replace(PROPS, doubled.into_bytes());
    let repeated = pkg.save_with(&edits).unwrap();
    let edited = edit(&repeated, |d| {
        d.presentation_properties.as_mut().unwrap().show = None
    });
    assert!(!String::from_utf8_lossy(&parts(&edited)[PROPS]).contains("showPr"));
}

#[test]
fn a_changed_guide_list_with_no_keys_pairs_in_order() {
    let source = read(RICH);
    edit(&source, |d| {
        let view = d.view_properties.as_mut().unwrap();
        let guides = view.slide.as_mut().unwrap().guides.as_mut().unwrap();
        guides.push(Guide {
            orient: None,
            pos: Some(Coordinate(1)),
        });
        guides.remove(0);
    });
}

#[test]
fn a_signed_deck_says_its_signature_is_invalidated() {
    let source = read("generated/minimal.pptx");
    let mut pkg = open(source);
    let unsigned = {
        let deck = import_deck(&mut pkg, &limits()).unwrap().deck;
        save_deck(&mut pkg, &deck, &limits()).unwrap()
    };
    assert!(!unsigned.signature_invalidated);
    let source = read(RICH);
    let mut pkg = open(source);
    let mut edits = PartEdits::default();
    let rels = String::from_utf8(pkg.read_part("_rels/.rels").unwrap()).unwrap();
    edits.replace(
        "_rels/.rels",
        rels.replace(
            "</Relationships>",
            r#"<Relationship Id="rIdSig" Type="http://schemas.openxmlformats.org/package/2006/relationships/digital-signature/origin" Target="_xmlsignatures/origin.sigs"/></Relationships>"#,
        )
        .into_bytes(),
    );
    edits.replace("_xmlsignatures/origin.sigs", Vec::new());
    let signed = pkg.save_with(&edits).unwrap();
    let mut pkg = open(signed);
    let mut deck = import_deck(&mut pkg, &limits()).unwrap().deck;
    let same = save_deck(&mut pkg, &deck, &limits()).unwrap();
    assert!(!same.signature_invalidated, "nothing was edited");
    deck.presentation.first_slide_num = Some(1);
    let outcome = save_deck(&mut pkg, &deck, &limits()).unwrap();
    assert!(outcome.signature_invalidated);
}

/// `xmllint`, if the machine has it. The parts here carry no schema of their own.
fn xmllint_validate(schema: &Path, part: &str, bytes: &[u8]) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("xsd");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(part.replace('/', "_"));
    std::fs::write(&file, bytes).unwrap();
    let out = Command::new("xmllint")
        .args(["--noout", "--nonet", "--schema"])
        .arg(schema)
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{part}: {}\n{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(bytes)
    );
}

/// Strip the rich model down to nothing, build it back on that, and hold the result to
/// the published schemas. This is what proves new elements land in schema order.
#[test]
fn parts_rebuilt_from_the_model_validate_against_the_schemas() {
    if Command::new("xmllint").arg("--version").output().is_err() {
        eprintln!("xmllint not installed: skipped");
        return;
    }
    let source = read(RICH);
    let mut pkg = open(source);
    // What this fixture holds on purpose to be outside the schema.
    let mut edits = PartEdits::default();
    let main = String::from_utf8(pkg.read_part(MAIN).unwrap()).unwrap();
    edits.replace(MAIN, main.replace(r#" x:unknown="kept""#, "").into_bytes());
    let conforming = pkg.save_with(&edits).unwrap();
    let wanted = model(&conforming);

    let stripped = edit(&conforming, |d| {
        let keep_required = d.presentation.notes_size;
        d.presentation = Default::default();
        d.presentation.notes_size = keep_required;
        d.presentation_properties = Some(Default::default());
        d.view_properties = Some(Default::default());
        d.properties.app = Some(Default::default());
        d.properties.custom = Some(Default::default());
    });
    let rebuilt = edit(&stripped, |d| *d = wanted.clone());
    assert_eq!(model(&rebuilt), wanted);

    let all = parts(&rebuilt);
    let schemas = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas/ooxml");
    for (part, schema) in [
        (MAIN, "pml.xsd"),
        (PROPS, "pml.xsd"),
        (VIEW, "pml.xsd"),
        (APP, "shared-documentPropertiesExtended.xsd"),
        (CUSTOM, "shared-documentPropertiesCustom.xsd"),
    ] {
        xmllint_validate(&schemas.join(schema), part, &all[part]);
    }
    // And single edits on a deck from another producer.
    let edited = edit(&read("producers/libreoffice-26.8-image.pptx"), |d| {
        d.presentation.first_slide_num = Some(2);
        d.presentation.slides.as_mut().unwrap()[0].id = 300;
        let show = d
            .presentation_properties
            .as_mut()
            .unwrap()
            .show
            .as_mut()
            .unwrap();
        show.show_type = Some(ShowType::Browse {
            show_scrollbar: Some(false),
        });
        show.slide_list = Some(SlideListChoice::All);
    });
    let all = parts(&edited);
    xmllint_validate(&schemas.join("pml.xsd"), MAIN, &all[MAIN]);
    xmllint_validate(&schemas.join("pml.xsd"), PROPS, &all[PROPS]);
}
