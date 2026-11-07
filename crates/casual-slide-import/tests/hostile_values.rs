//! Every file value is hostile: counts refuse, bad values are reported and left out,
//! and nothing panics.

use std::path::{Path, PathBuf};

use casual_slide_import::presentation::{ImportedDeck, import_deck, import_presentation};
use casual_slide_import::properties::{
    Findings, ImportError, ImportFindingKind as K, ImportLimits, import_app_properties,
    import_core_properties, import_custom_properties,
};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};
use casual_slide_package::PartEdits;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn open(path: &str) -> PresentationPackage {
    let bytes = std::fs::read(fixtures().join(path)).unwrap();
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn import(path: &str) -> Result<ImportedDeck, ImportError> {
    import_deck(&mut open(path), &ImportLimits::default())
}

/// One fixture for each bound in `ImportLimits`, just past its default.
#[test]
fn a_count_past_its_bound_refuses_with_its_code() {
    for (name, code, what) in [
        ("slides", "CS-IMP-0001", "slides"),
        ("masters", "CS-IMP-0001", "slide masters"),
        ("custom-shows", "CS-IMP-0001", "custom shows"),
        ("show-slides", "CS-IMP-0001", "custom show slides"),
        ("fonts", "CS-IMP-0001", "embedded fonts"),
        ("customer-data", "CS-IMP-0001", "customer data"),
        ("guides", "CS-IMP-0001", "guides"),
        ("custom-properties", "CS-IMP-0001", "custom properties"),
        ("keywords", "CS-IMP-0001", "keywords"),
        ("variant-items", "CS-IMP-0001", "variant items"),
        ("variant-depth", "CS-IMP-0001", "variant nesting"),
        ("value-bytes", "CS-IMP-0002", ""),
    ] {
        let err = import(&format!("hostile/import-limit-{name}.bin"))
            .err()
            .unwrap_or_else(|| panic!("{name}: imported"));
        assert_eq!(err.code(), code, "{name}: {err}");
        assert!(err.to_string().contains(what), "{name}: {err}");
    }
}

/// `PKG-02`: a target percent-encoded, as Part 2 allows, is the part it names. Compared
/// with ASCII case alone, this deck's properties were dropped as unusable.
#[test]
fn a_percent_encoded_target_reaches_its_part() {
    const RELS: &str = "ppt/_rels/presentation.xml.rels";
    let bytes = std::fs::read(fixtures().join("generated/rich-properties.pptx")).unwrap();
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    let plain = import_deck(&mut pkg, &ImportLimits::default()).unwrap();
    assert!(plain.deck.presentation_properties.is_some());

    let rels = String::from_utf8(pkg.read_part(RELS).unwrap()).unwrap();
    let encoded = rels.replace("presProps.xml", "pres%50rops.xml");
    assert_ne!(rels, encoded);
    let mut edits = PartEdits::default();
    edits.replace(RELS, encoded.into_bytes());
    let saved = pkg.save_with(&edits).unwrap();

    let mut pkg = PresentationPackage::open(saved, OoxmlLimits::default()).unwrap();
    let imported = import_deck(&mut pkg, &ImportLimits::default()).unwrap();
    assert_eq!(
        imported.deck.presentation_properties,
        plain.deck.presentation_properties
    );
    assert!(!imported.findings.has(K::UnusableRelationship, ""));
}

#[test]
fn a_limit_tightened_by_the_host_applies() {
    let limits = ImportLimits {
        max_slides: 2,
        ..ImportLimits::default()
    };
    let err = import_deck(&mut open("generated/three-slides.pptx"), &limits).unwrap_err();
    assert_eq!(err.code(), "CS-IMP-0001");
    let limits = ImportLimits {
        max_slides: 3,
        ..ImportLimits::default()
    };
    assert!(import_deck(&mut open("generated/three-slides.pptx"), &limits).is_ok());
}

#[test]
fn the_report_is_bounded() {
    let d = import("hostile/import-findings-flood.bin").unwrap();
    assert!(d.findings.len() <= ImportLimits::default().max_findings + 1);
    assert!(d.findings.iter().any(|f| f.kind == K::FindingsTruncated));
}

#[test]
fn bad_values_in_the_presentation_are_left_out_and_reported() {
    let d = import("hostile/import-bad-values.bin").unwrap();
    let p = &d.deck.presentation;
    assert_eq!(p.server_zoom, None);
    assert_eq!(p.first_slide_num, None, "overflow does not wrap");
    assert_eq!(p.rtl, None);
    assert_eq!(p.bookmark_id_seed, None, "zero is out of range");
    assert_eq!(p.conformance, None);
    assert_eq!(p.compat_mode, None);
    assert_eq!(
        p.slide_size, None,
        "the first sldSz is out of range, and the second is ignored"
    );
    assert_eq!(p.notes_size, None);
    assert!(d.findings.has(K::DuplicateElement, "sldSz"));
    assert!(d.findings.has(K::OutOfRange, "sldSz"));
    assert!(d.findings.has(K::MalformedValue, "presentation"));

    // In range or not, an id still names a slide. One that cannot be read does not.
    let ids: Vec<u32> = p.slides.as_ref().unwrap().iter().map(|s| s.id).collect();
    assert_eq!(ids, [4_294_967_295, 5, 256, 256]);
    assert!(d.findings.has(K::OutOfRange, "sldId"));
    assert!(d.findings.has(K::MalformedValue, "sldId"));
    assert!(d.findings.has(K::MissingRequired, "sldId"));
    assert!(d.findings.has(K::DanglingRelationship, "sldId"));
    assert!(d.findings.has(K::WrongRelationship, "sldId"));
    assert!(d.findings.has(K::DuplicateId, "sldIdLst"));
    let masters = p.slide_masters.as_ref().unwrap();
    assert_eq!(masters.len(), 1);
    assert_eq!(masters[0].id, Some(5));
    assert!(d.findings.has(K::OutOfRange, "sldMasterId"));

    let fonts = p.embedded_fonts.as_ref().unwrap();
    assert_eq!(fonts.len(), 1, "an entry without a font is left out");
    assert_eq!(fonts[0].font.panose, None);
    assert_eq!(fonts[0].font.pitch_family, None);
    assert_eq!(fonts[0].font.charset, None);
    assert!(d.findings.has(K::WrongRelationship, "regular"));
    let shows = p.custom_shows.as_ref().unwrap();
    assert_eq!(shows.len(), 2, "a show without an id is left out");
    assert!(d.findings.has(K::DuplicateId, "custShowLst"));
    assert!(
        d.findings.has(K::UnresolvedReference, "sld"),
        "{:?}",
        d.findings
    );
    assert_eq!(p.photo_album.unwrap().layout, None);
    assert_eq!(p.kinsoku, None, "both required sets are missing");
    let verifier = p.modify_verifier.as_ref().unwrap();
    assert_eq!(verifier.spin_count, None);
    assert_eq!(verifier.crypt_provider_type, None);
}

#[test]
fn bad_values_in_properties_and_view_state_are_left_out_and_reported() {
    let d = import("hostile/import-bad-values.bin").unwrap();
    let props = d.deck.presentation_properties.as_ref().unwrap();
    assert_eq!(props.web.as_ref().unwrap().image_size, None);
    assert_eq!(props.print.unwrap().print_what, None);
    let show = props.show.unwrap();
    assert_eq!(show.loop_show, None);
    assert!(matches!(
        show.show_type,
        Some(casual_slide_model::presentation::ShowType::Kiosk { restart: None })
    ));
    assert!(d.findings.has(K::DuplicateElement, "browse"));
    assert!(d.findings.has(K::OutOfRange, "sldRg"));
    assert!(d.findings.has(K::DuplicateElement, "custShow"));
    assert!(d.findings.has(K::DanglingRelationship, "htmlPubPr"));

    let view = d.deck.view_properties.as_ref().unwrap();
    assert_eq!(view.last_view, None);
    assert_eq!(view.show_comments, None);
    let normal = view.normal.unwrap();
    assert_eq!(normal.restored_left, None, "over 100%");
    assert_eq!(normal.restored_top, None, "`sz` is required");
    let common = view.slide.as_ref().unwrap().common.unwrap();
    assert_eq!(common.scale, None);
    assert_eq!(common.origin, None);
    let guides = view.slide.as_ref().unwrap().guides.as_ref().unwrap();
    assert_eq!(guides.len(), 2);
    assert_eq!(guides[0].pos, None, "past 32 bits");
    assert_eq!(guides[1].orient, None);
    assert_eq!(guides[1].pos, None);
    assert_eq!(
        view.outline
            .as_ref()
            .unwrap()
            .slides
            .as_ref()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(view.grid_spacing, None);

    let core = d.deck.properties.core.as_ref().unwrap();
    assert_eq!(
        core.title, None,
        "markup where text belongs, and the second is not used"
    );
    assert_eq!(core.created, None);
    assert_eq!(core.modified, None, "not a day, not an hour");
    assert_eq!(core.last_printed, None);
    assert!(d.findings.has(K::MissingRequired, "created"));
    assert!(d.findings.has(K::DuplicateElement, "title"));

    let app = d.deck.properties.app.as_ref().unwrap();
    assert_eq!(app.slides, None);
    assert_eq!(app.notes, None);
    assert_eq!(app.scale_crop, None);
    // A claim of 4294967295 items costs nothing, and is reported.
    assert_eq!(app.titles_of_parts.as_ref().unwrap().items.len(), 1);
    assert!(d.findings.has(K::CountMismatch, "vector"));
    assert_eq!(
        app.heading_pairs, None,
        "a vector with a bad item is not half a vector"
    );
    assert_eq!(app.hlinks, None);
    assert_eq!(app.dig_sig, None);

    let custom = d.deck.properties.custom.as_ref().unwrap();
    let pids: Vec<i32> = custom.properties.iter().map(|p| p.pid).collect();
    assert_eq!(pids, [1, 3, 3, 4, 5, 6, 7, 8]);
    assert!(custom.properties.iter().all(|p| p.value.is_none()));
    assert!(d.findings.has(K::DuplicateId, "property"));
    assert!(d.findings.has(K::OutOfRange, "property"));
    assert!(
        d.findings.has(K::MalformedValue, "r8"),
        "`inf` is not a schema float"
    );
    assert!(d.findings.has(K::MalformedValue, "i1"));
    assert!(d.findings.has(K::MalformedValue, "cy"));
    assert!(d.findings.has(K::MalformedValue, "date"));
    assert!(d.findings.has(K::MalformedValue, "blob"));
    assert!(d.findings.has(K::MalformedValue, "array"));
    assert!(d.findings.has(K::RetainedElement, "wat"));
}

/// A finding is structure. No value from the document is in the report.
#[test]
fn findings_carry_no_document_text() {
    let d = import("generated/rich-properties.pptx").unwrap();
    let text = format!("{:?}", d.findings);
    for secret in [
        "Old & title",
        "Old &amp;",
        "Co & Co",
        "Short",
        "Author",
        "BBBB",
    ] {
        assert!(!text.contains(secret), "{secret}");
    }
}

#[test]
fn an_unreadable_optional_part_is_reported_and_the_deck_opens() {
    let mut pkg = open("generated/rich-properties.pptx");
    let mut edits = casual_slide_package::PartEdits::default();
    edits.replace("docProps/core.xml", b"<workbook/>".to_vec());
    edits.replace(
        "docProps/app.xml",
        b"<Properties><a b=1></Properties>".to_vec(),
    );
    edits.replace("ppt/viewProps.xml", b"not xml <".to_vec());
    let bytes = pkg.save_with(&edits).unwrap();
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    let d = import_deck(&mut pkg, &ImportLimits::default()).unwrap();
    assert_eq!(d.deck.properties.core, None);
    assert_eq!(d.deck.view_properties, None);
    assert!(d.deck.properties.custom.is_some());
    assert!(
        d.findings
            .iter()
            .any(|f| f.kind == K::UnreadablePart && f.part == "docProps/core.xml")
    );
    assert!(
        d.findings
            .iter()
            .any(|f| f.kind == K::UnreadablePart && f.part == "ppt/viewProps.xml")
    );
}

#[test]
fn a_document_type_in_an_optional_part_still_refuses_the_deck() {
    let mut pkg = open("generated/rich-properties.pptx");
    let mut edits = casual_slide_package::PartEdits::default();
    edits.replace("docProps/core.xml", b"<!DOCTYPE a><a/>".to_vec());
    let bytes = pkg.save_with(&edits).unwrap();
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    let err = import_deck(&mut pkg, &ImportLimits::default()).unwrap_err();
    assert_eq!(err.code(), "CS-XML-0005");
}

#[test]
fn names_resolve_through_namespaces_not_prefixes() {
    let limits = ImportLimits::default();
    let xml_limits = OoxmlLimits::default();
    // The usual names, in the wrong namespaces, and the right ones under odd prefixes.
    let core = br#"<q:coreProperties xmlns:q="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="urn:not-dublin-core" xmlns:z="http://purl.org/dc/elements/1.1/"><dc:title>wrong</dc:title><z:title>right</z:title><q:revision>3</q:revision><revision>none</revision></q:coreProperties>"#;
    let tree = XmlTree::parse(core, "c", &xml_limits).unwrap();
    let mut f = Findings::new(100);
    let c = import_core_properties(&tree, "c", &limits, &mut f).unwrap();
    assert_eq!(c.title.as_deref(), Some("right"));
    assert_eq!(c.revision.as_deref(), Some("3"));

    let pres = br#"<x:presentation xmlns:x="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:rr="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:r="urn:not-relationships"><x:sldIdLst><x:sldId r:id="wrong" id="256" rr:id="rId1"/></x:sldIdLst></x:presentation>"#;
    let tree = XmlTree::parse(pres, "p", &xml_limits).unwrap();
    let mut f = Findings::new(100);
    let p = import_presentation(&tree, "p", &[], &limits, &mut f).unwrap();
    let slides = p.slides.unwrap();
    assert_eq!(slides[0].rel_id, "rId1");
    assert!(f.has(K::RetainedAttribute, "sldId"));

    // A root in the wrong namespace is not this part at all.
    let tree = XmlTree::parse(b"<Properties xmlns=\"urn:evil\"/>", "a", &xml_limits).unwrap();
    for err in [
        import_app_properties(&tree, "a", &limits, &mut f).unwrap_err(),
        import_custom_properties(&tree, "a", &limits, &mut f).unwrap_err(),
    ] {
        assert_eq!(err.code(), "CS-IMP-0003");
    }
    // An undeclared prefix is no namespace: the walker refuses the part (`XML-03`).
    let err = XmlTree::parse(b"<p:presentation/>", "p", &xml_limits).unwrap_err();
    assert_eq!(err.code(), "CS-XML-0004");
}

/// Every attribute value of the rich parts replaced by every hostile value, one at a
/// time. Whatever comes back, nothing panics. Not a fuzzer: a floor under one.
#[test]
fn hostile_values_at_every_attribute_do_not_panic() {
    const VALUES: &[&str] = &[
        "",
        " ",
        "-",
        "%",
        "1e999",
        "NaN",
        "99999999999999999999999999",
        "-99999999999999999999",
        "\u{e9}",
        "\u{1F600}%",
        "1\u{e9}in",
        "&#0;",
        "&#xD800;",
        "\u{e9}\u{e9}in",
        "4294967295",
        "2147483648",
    ];
    let mut pkg = open("generated/rich-properties.pptx");
    let limits = ImportLimits::default();
    let xml_limits = OoxmlLimits::default();
    let mut cases = 0usize;
    for part in [
        "ppt/presentation.xml",
        "ppt/presProps.xml",
        "ppt/viewProps.xml",
        "docProps/core.xml",
        "docProps/app.xml",
        "docProps/custom.xml",
    ] {
        let text = String::from_utf8(pkg.read_part(part).unwrap()).unwrap();
        let quotes: Vec<usize> = text.match_indices('"').map(|(i, _)| i).collect();
        for pair in quotes.chunks_exact(2) {
            for value in VALUES {
                let mutated = format!("{}{}{}", &text[..=pair[0]], value, &text[pair[1]..]);
                let Ok(tree) = XmlTree::parse(mutated.as_bytes(), part, &xml_limits) else {
                    continue;
                };
                let mut f = Findings::new(100);
                let _ = casual_slide_import::presentation::import_presentation(
                    &tree,
                    part,
                    &[],
                    &limits,
                    &mut f,
                );
                let _ = casual_slide_import::presentation::import_presentation_properties(
                    &tree,
                    part,
                    &[],
                    &limits,
                    &mut f,
                );
                let _ = casual_slide_import::presentation::import_view_properties(
                    &tree,
                    part,
                    &[],
                    &limits,
                    &mut f,
                );
                let _ = import_core_properties(&tree, part, &limits, &mut f);
                let _ = import_app_properties(&tree, part, &limits, &mut f);
                let _ = import_custom_properties(&tree, part, &limits, &mut f);
                cases += 1;
            }
        }
    }
    assert!(cases > 2000, "{cases}");
}
