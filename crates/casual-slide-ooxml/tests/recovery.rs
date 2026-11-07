//! Recover before refusing (`docs/29-RECOVERY.md`, `REC-01` to `REC-05`): every hostile
//! fixture still exists, and its expectation is "opens, and the attack did nothing".

use std::io::{Cursor, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};

use casual_slide_ooxml::{
    Finding, FindingKind, OoxmlLimits, PresentationPackage, Repair, XmlRepair, XmlTree,
    resolve_compatibility, walk,
};
use casual_slide_package::{
    ContainerFaultKind, Package, PartEdits, RETAINED_DIR, is_safe_part_path,
};
use zip::CompressionMethod;
use zip::write::{SimpleFileOptions, ZipWriter};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn minimal() -> Vec<u8> {
    read("generated/minimal.pptx")
}

fn with(deck: &[u8], parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut package = Package::open(deck.to_vec(), Default::default()).unwrap();
    let mut edits = PartEdits::default();
    for (name, bytes) in parts {
        edits.replace(*name, bytes.to_vec());
    }
    package.rewrite(&edits).unwrap()
}

fn without(deck: &[u8], parts: &[&str]) -> Vec<u8> {
    let mut package = Package::open(deck.to_vec(), Default::default()).unwrap();
    let mut edits = PartEdits::default();
    for name in parts {
        edits.remove(name);
    }
    package.rewrite(&edits).unwrap()
}

fn repaired(pkg: &PresentationPackage) -> Vec<Repair> {
    pkg.findings()
        .iter()
        .filter_map(|f: &Finding| match f.kind {
            FindingKind::Repaired(r) => Some(r),
            _ => None,
        })
        .collect()
}

/// Rows of the manifest whose attack is neutralized, not refused.
fn recovering_rows() -> Vec<(String, OoxmlLimits)> {
    let manifest = std::fs::read_to_string(fixtures().join("manifest.tsv")).unwrap();
    manifest
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut cols = line.split('\t');
            let path = cols.next()?.to_owned();
            cols.next();
            let expect = cols.next()?;
            if !expect.starts_with("recover:") {
                return None;
            }
            let mut limits = OoxmlLimits::default();
            for pair in cols.next().unwrap_or_default().split(',') {
                let Some((field, value)) = pair.split_once('=') else {
                    continue;
                };
                let value: usize = value.parse().unwrap();
                match field {
                    "max_xml_part_bytes" => limits.max_xml_part_bytes = value,
                    "max_xml_elements" => limits.max_xml_elements = value,
                    "max_xml_depth" => limits.max_xml_depth = value,
                    "max_xml_attributes" => limits.max_xml_attributes = value,
                    "max_part_relationships" => limits.max_part_relationships = value,
                    "max_total_relationships" => limits.max_total_relationships = value,
                    "max_content_types" => limits.max_content_types = value,
                    other => panic!("{path}: {other}"),
                }
            }
            Some((path, limits))
        })
        .collect()
}

#[test]
fn every_hostile_fixture_opens_and_says_what_was_done() {
    let rows = recovering_rows();
    assert!(rows.len() >= 35, "only {}", rows.len());
    for (path, limits) in rows {
        let bytes = read(&path);
        let mut pkg = PresentationPackage::open(bytes.clone(), limits)
            .unwrap_or_else(|err| panic!("{path}: {err}"));
        assert!(pkg.was_repaired(), "{path}");
        let report = pkg.recovery_report();
        assert!(!report.is_empty(), "{path}");
        assert_eq!(pkg.save_unedited(), bytes, "{path}");
        // An explicit repair is a file the strict reader admits.
        let clean = pkg.save_repaired().unwrap();
        let mut again = PresentationPackage::open_strict(clean, limits)
            .unwrap_or_else(|err| panic!("{path}: {err}"));
        assert!(!again.was_repaired(), "{path}");
        // Nothing is left that reading it would have to mend.
        for name in again.part_names() {
            if name.ends_with(".xml") || name.ends_with(".rels") {
                let part = again.read_part(&name).unwrap();
                walk(&part, &name, &limits, |_, _| Ok(()))
                    .unwrap_or_else(|err| panic!("{path} {name}: {err}"));
            }
        }
    }
}

/// `SEC-X01`: the bomb's document type is skipped, and what it would have made is not there.
#[test]
fn a_billion_laughs_opens_and_nothing_expands() {
    let mut pkg = open(read("hostile/doctype-billion-laughs.bin"));
    let found = repaired(&pkg);
    assert!(
        found.contains(&Repair::Xml(XmlRepair::DoctypeSkipped)),
        "{found:?}"
    );
    assert!(
        found.contains(&Repair::Xml(XmlRepair::ReferenceLiteral)),
        "{found:?}"
    );
    let main = pkg.main_part().to_owned();
    let part = pkg.read_xml_part(&main).unwrap();
    // Any expansion at all is a billion of something.
    assert!(part.len() < 4096, "{}", part.len());
    let text = String::from_utf8(part).unwrap();
    assert!(!text.contains("ENTITY"), "{text}");
    assert!(!text.contains("DOCTYPE"), "{text}");
    assert!(pkg.consumed() < 1 << 20, "{}", pkg.consumed());
}

/// `SEC-X02`, `SEC-N01`: a listener is the proof. Nothing connects to it, whatever the file
/// names: an external entity, an external image, a remote template.
#[test]
fn a_file_naming_a_server_never_reaches_it() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}/never", listener.local_addr().unwrap());
    let doctype =
        format!("<!DOCTYPE p [<!ENTITY x SYSTEM \"{url}\"><!ENTITY % y SYSTEM \"{url}\">%y;]>");
    let base = minimal();
    let mut source = Package::open(base.clone(), Default::default()).unwrap();
    let main = source.read_part("ppt/presentation.xml").unwrap();
    let main = String::from_utf8(main).unwrap();
    let at = main.find("<p:presentation").unwrap();
    let hostile_main = format!("{}{doctype}{}&x;", &main[..at], &main[at..]);
    let rels = format!(
        "<?xml version=\"1.0\"?>{doctype}<Relationships \
         xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
         <Relationship Id=\"rId1\" \
         Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" \
         Target=\"{url}\" TargetMode=\"External\"/></Relationships>"
    );
    let deck = with(
        &base,
        &[
            ("ppt/presentation.xml", hostile_main.as_bytes()),
            ("ppt/slides/_rels/slide1.xml.rels", rels.as_bytes()),
        ],
    );
    let mut pkg = open(deck);
    let main_name = pkg.main_part().to_owned();
    pkg.read_xml_part(&main_name).unwrap();
    pkg.read_xml_part("ppt/slides/slide1.xml").unwrap();
    pkg.save_repaired().unwrap();
    assert!(
        pkg.findings()
            .iter()
            .any(|f| matches!(f.kind, FindingKind::Repaired(_))),
        "the damage was seen"
    );
    // Accepted, had anything connected: a connect finishes before open returns.
    assert_eq!(
        listener.accept().map(|_| ()).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock,
    );
}

/// `SEC-P01`: the bomb is not decompressed, is not a part, and is kept as it arrived.
#[test]
fn a_zip_bomb_opens_and_is_kept_unopened() {
    let bytes = read("hostile/zip-bomb.bin");
    let mut pkg = open(bytes.clone());
    assert!(repaired(&pkg).contains(&Repair::Container(ContainerFaultKind::OverLimit)));
    assert!(pkg.read_part("ppt/media/bomb.bin").is_err());
    let kept: Vec<String> = pkg
        .part_names()
        .into_iter()
        .filter(|n| n.starts_with(RETAINED_DIR))
        .collect();
    assert_eq!(kept.len(), 1, "{kept:?}");
    let raw = pkg.read_part(&kept[0]).unwrap();
    assert!(raw.len() < 1 << 20, "{}", raw.len());
    assert!(
        bytes.windows(raw.len()).any(|w| w == raw),
        "the raw bytes are what arrived"
    );
    // Reading the deck spent nothing like what the bomb holds.
    assert!(pkg.consumed() < 8 << 20, "{}", pkg.consumed());
}

/// `SEC-P03`, `SEC-P07`: no name is used as a path, and none reaches a part.
#[test]
fn an_unsafe_name_is_opaque() {
    let mut pkg = open(read("hostile/path-traversal.bin"));
    assert!(repaired(&pkg).contains(&Repair::Container(ContainerFaultKind::UnsafeName)));
    for name in pkg.part_names() {
        assert!(is_safe_part_path(&name), "{name}");
        assert!(!name.contains(".."), "{name}");
    }
    assert!(pkg.read_part("../../etc/evil.xml").is_err());
}

/// `SEC-P05`, `SEC-P06`, `SEC-P14`: on save, one entry per part, so no scanner and editor
/// can disagree about which they saw.
#[test]
fn of_duplicates_a_save_writes_one() {
    for fixture in [
        "hostile/case-colliding-parts.bin",
        "hostile/percent-equivalent-parts.bin",
    ] {
        let mut pkg = open(read(fixture));
        assert!(
            repaired(&pkg).contains(&Repair::Container(ContainerFaultKind::DuplicateEntry)),
            "{fixture}"
        );
        let clean = pkg.save_repaired().unwrap();
        let strict = PresentationPackage::open_strict(clean, OoxmlLimits::default()).unwrap();
        // Strict admission refuses two entries for one part, so one is all there is.
        assert!(!strict.part_names().is_empty());
    }
    // The main part is the one that parses as one, though the other came last.
    let mut pkg = open(read("hostile/case-colliding-parts.bin"));
    let main = pkg.main_part().to_owned();
    let part = pkg.read_xml_part(&main).unwrap();
    let tree = XmlTree::parse(&part, &main, &OoxmlLimits::default()).unwrap();
    assert_eq!(tree.local_name(tree.root()), "presentation");
    assert!(!repaired(&pkg).contains(&Repair::MainPartBuilt));
}

/// `SEC-X05`: the demand is set aside, and what it demanded is left out.
#[test]
fn a_must_understand_is_set_aside_and_its_namespace_left_out() {
    let mut pkg = open(read("hostile/mce-must-understand-unknown.bin"));
    assert!(repaired(&pkg).contains(&Repair::MustUnderstandSetAside));
    let main = pkg.main_part().to_owned();
    let (tree, compatibility) = pkg.read_resolved(&main).unwrap();
    assert!(compatibility.must_understand.is_empty());
    assert!(resolve_compatibility(&tree).must_understand.is_empty());
}

#[test]
fn a_damaged_slide_starts_the_repair_when_it_is_read_and_not_before() {
    let deck = with(&minimal(), &[("ppt/slides/slide1.xml", b"<p:sld><broken")]);
    let mut pkg = open(deck.clone());
    assert!(!pkg.was_repaired(), "opening reads structure only");
    let part = pkg.read_xml_part("ppt/slides/slide1.xml").unwrap();
    let tree = XmlTree::parse(&part, "s", &OoxmlLimits::default()).unwrap();
    assert_eq!(tree.local_name(tree.root()), "sld");
    assert!(pkg.was_repaired());
    assert!(pkg.is_damaged("ppt/slides/slide1.xml"));
    assert_eq!(pkg.save_unedited(), deck);
}

#[test]
fn a_flipped_byte_in_a_stream_is_found_on_reading_it() {
    let mut deck = {
        let mut w = ZipWriter::new(Cursor::new(Vec::new()));
        let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let mut base = Package::open(minimal(), Default::default()).unwrap();
        for name in base.entry_names() {
            let bytes = base.read_part(&name).unwrap();
            w.start_file(name.as_str(), opts).unwrap();
            w.write_all(&bytes).unwrap();
        }
        w.finish().unwrap().into_inner()
    };
    let needle = b"<p:sld ";
    let at = deck
        .windows(needle.len())
        .position(|w| w == needle)
        .unwrap();
    // Still well-formed, in a stored slide, so only the checksum can say.
    deck[at + 3] = b'S';
    deck[at + 5] = b'D';
    let mut pkg = open(deck.clone());
    let part = pkg.read_xml_part("ppt/slides/slide1.xml").unwrap();
    assert!(!part.is_empty());
    assert!(
        repaired(&pkg).contains(&Repair::Container(ContainerFaultKind::ChecksumMismatch)),
        "{:?}",
        pkg.findings()
    );
    assert_eq!(pkg.save_unedited(), deck);
}

/// `REC-05`: `prepare_for_edit` finds what no reader had reached, so it cannot be written out.
#[test]
fn damage_no_reader_reached_is_found_before_a_save() {
    let broken = with(
        &minimal(),
        &[("ppt/theme/theme1.xml", b"<a:theme><unclosed")],
    );
    let mut pkg = open(broken);
    assert!(!pkg.was_repaired());
    let mut edits = PartEdits::default();
    edits.replace("docProps/extra.xml", b"<a/>".to_vec());
    let saved = pkg.save_with(&edits).unwrap();
    assert!(pkg.was_repaired());
    let mut again = PresentationPackage::open_strict(saved, OoxmlLimits::default()).unwrap();
    let theme = again.read_part("ppt/theme/theme1.xml").unwrap();
    walk(&theme, "t", &OoxmlLimits::default(), |_, _| Ok(())).unwrap();
}

#[test]
fn what_arrived_of_a_damaged_part_stays_in_the_file() {
    let junk: &[u8] = b"\xFFnot XML, and the user's\x00only copy";
    let deck = with(&minimal(), &[("ppt/slides/slide1.xml", junk)]);
    let mut pkg = open(deck);
    let clean = pkg.save_repaired().unwrap();
    let mut again = PresentationPackage::open_strict(clean, OoxmlLimits::default()).unwrap();
    let kept: Vec<String> = again
        .part_names()
        .into_iter()
        .filter(|n| n.starts_with(RETAINED_DIR))
        .collect();
    assert_eq!(kept.len(), 1, "{kept:?}");
    assert_eq!(again.read_part(&kept[0]).unwrap(), junk);
    // And it is declared, so the file stays valid.
    assert!(again.content_types().resolve(&kept[0]).is_some());
}

#[test]
fn an_unedited_save_returns_what_arrived_however_it_was_repaired() {
    let cases: Vec<Vec<u8>> = vec![
        // Each of the layers: the container, the package, a part, the chain.
        minimal()[..minimal().len() - 30].to_vec(),
        without(&minimal(), &["[Content_Types].xml"]),
        without(&minimal(), &["_rels/.rels"]),
        with(&minimal(), &[("ppt/presentation.xml", b"<broken")]),
        without(&minimal(), &["ppt/slides/slide1.xml"]),
    ];
    for deck in cases {
        let mut pkg = open(deck.clone());
        assert!(pkg.was_repaired() || !pkg.findings().is_empty());
        assert_eq!(pkg.save_unedited(), deck);
        pkg.prepare_for_edit().unwrap();
        assert_eq!(pkg.save_unedited(), deck);
        assert_eq!(pkg.save_with(&PartEdits::default()).unwrap(), deck);
    }
}

#[test]
fn a_host_that_wants_refusal_gets_it() {
    let broken_slide = with(&minimal(), &[("ppt/slides/slide1.xml", b"<p:sld><broken")]);
    let mut strict =
        PresentationPackage::open_strict(broken_slide, OoxmlLimits::default()).unwrap();
    assert!(strict.is_strict());
    let err = strict
        .read_xml_part("ppt/slides/slide1.xml")
        .and_then(|bytes| {
            XmlTree::parse(&bytes, "s", &OoxmlLimits::default())?;
            Ok(())
        });
    assert_eq!(err.unwrap_err().code(), "CS-XML-0004");
    assert!(!strict.was_repaired());
    assert!(strict.set_part_aside("ppt/slides/slide1.xml").is_err());

    let cut = &minimal()[..minimal().len() - 30];
    let err = PresentationPackage::open_strict(cut.to_vec(), OoxmlLimits::default()).unwrap_err();
    assert_eq!(err.code(), "CS-PKG-0005");
}

#[test]
fn what_docs_29_lists_still_refuses() {
    let limits = OoxmlLimits::default();
    for (bytes, code) in [
        (read("hostile/not-a-zip.bin"), "CS-PKG-0005"),
        (read("hostile/compound-file.bin"), "CS-OPC-0005"),
        (vec![0xAB; 1 << 16], "CS-PKG-0005"),
        (Vec::new(), "CS-PKG-0005"),
    ] {
        let err = PresentationPackage::open(bytes, limits).unwrap_err();
        assert_eq!(err.code(), code);
    }
    let mut tight = limits;
    tight.package.max_input_bytes = 16;
    let err = PresentationPackage::open(minimal(), tight).unwrap_err();
    assert_eq!(err.code(), "CS-PKG-0001");
}

#[test]
fn a_report_is_plain_language() {
    let deck = without(&minimal(), &["ppt/slides/slide1.xml"]);
    let pkg = open(deck);
    let lines = pkg.recovery_report().plain_language();
    assert!(!lines.is_empty());
    assert!(lines.iter().any(|l| l.contains("shown empty")), "{lines:?}");
    for line in &lines {
        assert!(!line.contains("CS-"), "{line}");
        assert!(line.ends_with('.'), "{line}");
    }
    // A healthy deck has nothing to say.
    assert!(open(minimal()).recovery_report().is_empty());
}

#[test]
fn a_main_part_is_found_by_name_and_by_root() {
    let no_rels = without(&minimal(), &["_rels/.rels"]);
    let pkg = open(no_rels);
    assert_eq!(pkg.main_part(), "ppt/presentation.xml");
    assert!(repaired(&pkg).contains(&Repair::MainPartFound));

    // Under another name, and its type gone as well: the root says.
    let mut source = Package::open(minimal(), Default::default()).unwrap();
    let main = source.read_part("ppt/presentation.xml").unwrap();
    let moved = with(
        &without(&minimal(), &["_rels/.rels", "ppt/presentation.xml"]),
        &[("elsewhere/deck.xml", &main)],
    );
    let pkg = open(moved);
    assert_eq!(pkg.main_part(), "elsewhere/deck.xml");
}

#[test]
fn a_deck_is_built_from_whatever_slides_exist() {
    let bare = without(
        &read("generated/three-slides.pptx"),
        &[
            "ppt/presentation.xml",
            "ppt/_rels/presentation.xml.rels",
            "_rels/.rels",
            "[Content_Types].xml",
        ],
    );
    let mut pkg = open(bare);
    assert!(repaired(&pkg).contains(&Repair::MainPartBuilt));
    let main = pkg.main_part().to_owned();
    let slides: Vec<_> = pkg
        .relationships_of(&main)
        .iter()
        .filter(|r| r.rel_type.ends_with("/slide"))
        .map(|r| r.target.clone())
        .collect();
    assert_eq!(slides.len(), 3, "{slides:?}");
    assert!(slides[0].ends_with("slide1.xml"), "{slides:?}");
    assert!(slides[2].ends_with("slide3.xml"), "{slides:?}");
    let part = pkg.read_xml_part(&main).unwrap();
    let tree = XmlTree::parse(&part, &main, &OoxmlLimits::default()).unwrap();
    assert_eq!(tree.local_name(tree.root()), "presentation");
}

/// Entries on a list that name nothing, in a number no real deck has. Taken off one at a time,
/// that is a search of the list for each: its length squared, which here is minutes.
#[test]
fn a_flood_of_list_entries_costs_no_more_than_its_length() {
    let entries = "<p:sldId id=\"256\" r:id=\"rId999999\"/>".repeat(200_000);
    let main = format!(
        "<p:presentation xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" \
         xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
         <p:sldIdLst>{entries}</p:sldIdLst><p:notesSz cx=\"1\" cy=\"1\"/></p:presentation>"
    );
    // The part is whole. What starts the repair is the relationships gone from the package.
    let deck = with(
        &without(&minimal(), &["_rels/.rels"]),
        &[("ppt/presentation.xml", main.as_bytes())],
    );
    let started = std::time::Instant::now();
    let mut pkg = open(deck);
    pkg.save_repaired().unwrap();
    let took = started.elapsed();
    assert!(took.as_secs() < 60, "{took:?}");
    assert!(repaired(&pkg).contains(&Repair::ListEntryDropped));
}

/// Building edits, and writing them over as many entries: a search of the edits for each,
/// in either place, is a product. Seen to run past an hour with `PartEdits::replace` searching.
#[test]
fn many_edits_over_many_entries_cost_their_sum() {
    let names: Vec<String> = (0..20_000).map(|i| format!("ppt/x{i}.bin")).collect();
    let started = std::time::Instant::now();
    let parts: Vec<(&str, &[u8])> = names.iter().map(|n| (n.as_str(), &b"x"[..])).collect();
    let deck = with(&minimal(), &parts);
    let mut package = Package::open(deck, Default::default()).unwrap();
    let mut edits = PartEdits::default();
    for name in &names {
        edits.replace(name.clone(), b"y".to_vec());
    }
    package.rewrite(&edits).unwrap();
    let took = started.elapsed();
    assert!(took.as_secs() < 60, "{took:?}");
}

#[test]
fn opening_twice_gives_one_image() {
    let deck = without(&read("generated/three-slides.pptx"), &["_rels/.rels"]);
    let a = open(deck.clone());
    let b = open(deck);
    assert_eq!(a.base_bytes(), b.base_bytes());
}
