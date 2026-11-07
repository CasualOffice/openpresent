//! Every fixture against the expectation recorded for it in `fixtures/manifest.tsv`.

use std::path::{Path, PathBuf};
use std::process::Command;

use casual_slide_ooxml::{
    AlternateResolution, Branch, Compatibility, Conformance, FindingKind, OoxmlLimits,
    PresentationPackage, STRICT_TO_TRANSITIONAL, XmlTree, is_understood, resolve_compatibility,
    to_strict, to_transitional, understood_namespaces,
};
use casual_slide_package::PartEdits;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// Each fixture with the limits its manifest row names, else the defaults.
fn limited_cases() -> Vec<(String, String, OoxmlLimits)> {
    let manifest = std::fs::read_to_string(fixtures().join("manifest.tsv")).unwrap();
    manifest
        .lines()
        .skip(1)
        .map(|line| {
            let mut cols = line.split('\t');
            let path = cols.next().unwrap().to_owned();
            cols.next();
            let expect = cols.next().unwrap().to_owned();
            let limits = cols
                .next()
                .map_or_else(OoxmlLimits::default, |l| limits_from(&path, l));
            (path, expect, limits)
        })
        .collect()
}

fn limits_from(path: &str, column: &str) -> OoxmlLimits {
    let mut limits = OoxmlLimits::default();
    for pair in column.split(',') {
        let (field, value) = pair
            .split_once('=')
            .unwrap_or_else(|| panic!("{path}: {pair}"));
        let value: usize = value.parse().unwrap_or_else(|_| panic!("{path}: {pair}"));
        match field {
            "max_xml_part_bytes" => limits.max_xml_part_bytes = value,
            "max_xml_elements" => limits.max_xml_elements = value,
            "max_xml_depth" => limits.max_xml_depth = value,
            "max_xml_attributes" => limits.max_xml_attributes = value,
            "max_part_relationships" => limits.max_part_relationships = value,
            "max_total_relationships" => limits.max_total_relationships = value,
            "max_content_types" => limits.max_content_types = value,
            other => panic!("{path}: no limit {other}"),
        }
    }
    limits
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

#[test]
fn every_fixture_meets_its_expectation() {
    let cases = limited_cases();
    assert!(cases.len() > 30, "only {} fixtures", cases.len());
    for (path, expect, limits) in cases {
        let result = PresentationPackage::open(read(&path), limits);
        // What only docs/29's rules refuse, whichever way it is opened.
        if let Some(code) = expect.strip_prefix("refuse:") {
            match result {
                Ok(_) => panic!("{path}: admitted, expected {code}"),
                Err(err) => assert_eq!(err.code(), code, "{path}: {err}"),
            }
            match PresentationPackage::open_strict(read(&path), limits) {
                Ok(_) => panic!("{path}: admitted strictly, expected {code}"),
                Err(err) => assert_eq!(err.code(), code, "{path}: {err}"),
            }
            continue;
        }
        // A fault: a host asking for strictness gets the refusal it always got.
        let expect = match expect.strip_prefix("recover:") {
            Some(rest) => {
                let (code, findings) = rest.split_once(' ').unwrap_or((rest, ""));
                match PresentationPackage::open_strict(read(&path), limits) {
                    Ok(_) => panic!("{path}: admitted strictly, expected {code}"),
                    Err(err) => assert_eq!(err.code(), code, "{path}: {err}"),
                }
                format!("open {findings}").trim_end().to_owned()
            }
            None => expect,
        };
        let pkg = result.unwrap_or_else(|err| panic!("{path}: {err}"));
        let mut codes: Vec<&str> = pkg.findings().iter().map(|f| f.kind.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        let wanted: Vec<&str> = expect.split(' ').skip(1).collect();
        assert_eq!(codes, wanted, "{path}: {:?}", pkg.findings());
    }
}

fn opening_decks() -> Vec<String> {
    limited_cases()
        .into_iter()
        // A repaired deck has its own tests, in `recovery.rs`: it is not meant to keep every
        // part as it was.
        .filter(|(_, e, _)| e.starts_with("open") && !e.contains("CS-REC"))
        .map(|(p, _, _)| p)
        .collect()
}

/// `SEC-016`: a deck refused under lowered limits is refused because of them, so
/// the default limits admit it. Otherwise it tests something else.
#[test]
fn a_deck_with_lowered_limits_is_refused_only_by_them() {
    let mut seen = 0;
    for line in std::fs::read_to_string(fixtures().join("manifest.tsv"))
        .unwrap()
        .lines()
        .skip(1)
    {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 4 {
            continue;
        }
        PresentationPackage::open_strict(read(cols[0]), OoxmlLimits::default())
            .unwrap_or_else(|e| panic!("{}: {e}", cols[0]));
        seen += 1;
    }
    assert!(seen >= 5, "{seen}");
}

/// Fidelity F1: an unedited save is the file.
#[test]
fn an_unedited_save_is_byte_identical() {
    for path in opening_decks() {
        let bytes = read(&path);
        let pkg = PresentationPackage::open(bytes.clone(), OoxmlLimits::default()).unwrap();
        assert_eq!(pkg.save_unedited(), bytes, "{path}");
    }
}

/// Fidelity F2: an edit leaves every other part's bytes alone, and the result
/// still opens with the same graph.
#[test]
fn an_edit_leaves_every_other_part_intact() {
    for path in opening_decks() {
        let mut before = PresentationPackage::open(read(&path), OoxmlLimits::default()).unwrap();
        let edited = before.main_part().to_owned();
        let replacement = before.read_part(&edited).unwrap();
        let mut edits = PartEdits::default();
        edits.replace(edited.clone(), replacement);
        let out = before.save_with(&edits).unwrap();

        let mut after = PresentationPackage::open(out, OoxmlLimits::default()).unwrap();
        assert_eq!(before.part_names(), after.part_names(), "{path}");
        assert_eq!(before.findings(), after.findings(), "{path}");
        for name in before.part_names() {
            assert_eq!(
                before.read_part(&name).unwrap(),
                after.read_part(&name).unwrap(),
                "{path}: {name}"
            );
        }
    }
}

/// Declared sizes, by entry name.
fn declared_sizes(bytes: &[u8]) -> std::collections::BTreeMap<String, u64> {
    let mut package =
        casual_slide_package::Package::open(bytes.to_vec(), Default::default()).unwrap();
    package
        .entries()
        .unwrap()
        .into_iter()
        .map(|e| (e.name, e.uncompressed))
        .collect()
}

#[test]
fn opening_a_deck_does_not_read_its_media() {
    // Only structure is read on open, so a deck full of media opens cheaply and
    // no decoder is reachable from opening a file. Proven on the read ledger
    // (`SEC-017`): opening charges exactly the content types, the main part and
    // the relationship parts, and nothing else.
    let bytes = read("generated/image.pptx");
    let sizes = declared_sizes(&bytes);
    let structure: u64 = sizes
        .iter()
        .filter(|(n, _)| {
            n.as_str() == "[Content_Types].xml"
                || n.as_str() == "ppt/presentation.xml"
                || n.ends_with(".rels")
        })
        .map(|(_, s)| s)
        .sum();
    let media = sizes["ppt/media/image1.png"];
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    assert_eq!(pkg.consumed(), structure);
    assert!(
        pkg.read_part("PPT/MEDIA/IMAGE1.PNG")
            .unwrap()
            .starts_with(b"\x89PNG")
    );
    assert_eq!(pkg.consumed(), structure + media);
}

/// `PKG-04`: a part read for parsing is refused on its declaration, with nothing
/// decompressed, so the walker never has to refuse what has already been read.
#[test]
fn an_oversized_xml_part_is_refused_before_it_is_read() {
    let big = slide(&"<p:sp/>".repeat(20_000));
    let deck = with_part("generated/minimal.pptx", "ppt/slides/slide1.xml", &big);
    let limits = OoxmlLimits {
        max_xml_part_bytes: 100_000,
        ..OoxmlLimits::default()
    };
    let mut pkg = PresentationPackage::open_strict(deck.clone(), limits).unwrap();
    let before = pkg.consumed();
    let err = pkg.read_xml_part("ppt/slides/slide1.xml").unwrap_err();
    assert_eq!(err.code(), "CS-PKG-0009");
    assert_eq!(pkg.consumed(), before);
    let err = pkg.read_resolved("ppt/slides/slide1.xml").unwrap_err();
    assert_eq!(err.code(), "CS-PKG-0009");
    assert_eq!(pkg.consumed(), before);
    assert!(pkg.read_xml_part("ppt/slides/slideLayout1.xml").is_err());
    let layout = pkg
        .read_xml_part("ppt/slideLayouts/slideLayout1.xml")
        .unwrap();
    assert_eq!(pkg.consumed(), before + layout.len() as u64);

    // Without `strict` the slide is an empty one, marked, and what it held is not read: the
    // bytes kept for it are its compressed bytes (`SEC-P13`).
    let mut pkg = PresentationPackage::open(deck, limits).unwrap();
    let served = pkg.read_xml_part("ppt/slides/slide1.xml").unwrap();
    assert!(served.len() < 2048, "{}", served.len());
    assert!(pkg.is_damaged("ppt/slides/slide1.xml"));
    assert!(pkg.consumed() < 100_000, "{}", pkg.consumed());
}

#[test]
fn the_part_graph_finds_a_deck_structure() {
    let pkg =
        PresentationPackage::open(read("generated/three-slides.pptx"), OoxmlLimits::default())
            .unwrap();
    assert_eq!(pkg.main_part(), "ppt/presentation.xml");
    assert_eq!(pkg.relationships_of("ppt/presentation.xml").len(), 5);
    assert!(pkg.findings().is_empty());
    assert_eq!(
        pkg.content_types().resolve("ppt/slides/slide2.xml"),
        Some("application/vnd.openxmlformats-officedocument.presentationml.slide+xml")
    );
}

/// Every XML part in every deck, including third-party ones, writes back byte for byte.
#[test]
fn every_xml_part_round_trips_through_the_tree() {
    let mut parts = 0;
    for path in opening_decks() {
        let mut pkg = PresentationPackage::open(read(&path), OoxmlLimits::default()).unwrap();
        for name in pkg.part_names() {
            if !(name.ends_with(".xml") || name.ends_with(".rels")) {
                continue;
            }
            let bytes = pkg.read_part(&name).unwrap();
            let tree = casual_slide_ooxml::XmlTree::parse(&bytes, &name, &OoxmlLimits::default())
                .unwrap_or_else(|e| panic!("{path}: {name}: {e}"));
            assert_eq!(tree.to_bytes(), bytes, "{path}: {name}");
            parts += 1;
        }
    }
    assert!(parts > 200, "only {parts} parts");
}

// --- OPC-01: part-name equivalence --------------------------------------------------

#[test]
fn a_part_is_found_however_its_name_is_spelled() {
    let mut pkg = PresentationPackage::open(
        read("generated/percent-encoded-names.pptx"),
        OoxmlLimits::default(),
    )
    .unwrap();
    assert!(pkg.findings().is_empty(), "{:?}", pkg.findings());
    for spelling in [
        "ppt/media/my image.png",
        "ppt/media/my%20image.png",
        "/PPT/MEDIA/MY%20IMAGE.PNG",
    ] {
        assert!(pkg.read_part(spelling).unwrap().starts_with(b"\x89PNG"));
    }
    assert!(
        pkg.read_part("ppt/media/é.png")
            .unwrap()
            .starts_with(b"\x89PNG")
    );
    // The graph went through a part with an encoded name, to that part's own rels.
    assert_eq!(pkg.relationships_of("ppt/slides/slide%202.xml").len(), 1);

    let pkg = PresentationPackage::open(
        read("generated/percent-encoded-targets.pptx"),
        OoxmlLimits::default(),
    )
    .unwrap();
    assert!(pkg.findings().is_empty(), "{:?}", pkg.findings());
}

#[test]
fn a_target_that_decodes_to_traversal_is_reported_and_not_followed() {
    let pkg = PresentationPackage::open(
        read("hostile/rel-percent-traversal.bin"),
        OoxmlLimits::default(),
    )
    .unwrap();
    let ids: Vec<&str> = pkg
        .findings()
        .iter()
        .filter(|f| f.kind == FindingKind::DisguisedPartName)
        .filter_map(|f| f.rel_id.as_deref())
        .collect();
    assert_eq!(ids, ["rId2", "rId3", "rId4", "rId5", "rId6"]);
    assert!(
        pkg.findings()
            .iter()
            .all(|f| f.kind.code() == "CS-FND-0011")
    );
}

// --- MCE-01 and SEC-X05 ------------------------------------------------------------

fn with_part(deck: &str, name: &str, xml: &str) -> Vec<u8> {
    let mut pkg = PresentationPackage::open(read(deck), OoxmlLimits::default()).unwrap();
    let mut edits = PartEdits::default();
    edits.replace(name.to_owned(), xml.as_bytes().to_vec());
    pkg.save_with(&edits).unwrap()
}

const SLIDE_OPEN: &str = "<p:sld xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" \
    xmlns:mc=\"http://schemas.openxmlformats.org/markup-compatibility/2006\" \
    xmlns:p14=\"http://schemas.microsoft.com/office/powerpoint/2010/main\" \
    xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
    xmlns:x=\"urn:x\" xmlns:y=\"urn:y\">";
const TREE_OPEN: &str = "<p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>";
const TREE_CLOSE: &str = "</p:spTree></p:cSld></p:sld>";

fn slide(body: &str) -> String {
    format!("{SLIDE_OPEN}{TREE_OPEN}{body}{TREE_CLOSE}")
}

#[test]
fn reading_a_part_picks_the_branch_and_keeps_both() {
    let mut pkg = PresentationPackage::open(
        read("generated/mce-alternate-content.pptx"),
        OoxmlLimits::default(),
    )
    .unwrap();
    let bytes = pkg.read_part("ppt/slides/slide1.xml").unwrap();
    let (tree, compatibility) = pkg.read_resolved("ppt/slides/slide1.xml").unwrap();
    assert_eq!(compatibility.alternates.len(), 1);
    assert!(matches!(
        compatibility.alternates[0].branch,
        Branch::Fallback { .. }
    ));
    assert_eq!(compatibility.alternates[0].not_taken.len(), 1);
    assert_eq!(compatibility.ignored_attributes, 1);
    assert_eq!(tree.to_bytes(), bytes);
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("notInAnySchema") && text.contains("Fallback"));
}

#[test]
fn a_must_understand_we_do_not_know_refuses_only_where_it_is_reached() {
    let refused = slide("<p:grpSp mc:MustUnderstand=\"p14\"/>");
    let deck = with_part("generated/minimal.pptx", "ppt/slides/slide1.xml", &refused);
    // Without `strict`, the demand is set aside and the part reads.
    let mut lenient = PresentationPackage::open(deck.clone(), OoxmlLimits::default()).unwrap();
    let (tree, compatibility) = lenient.read_resolved("ppt/slides/slide1.xml").unwrap();
    assert!(compatibility.must_understand.is_empty());
    assert!(!tree.to_bytes().windows(14).any(|w| w == b"MustUnderstand"));
    let mut pkg = PresentationPackage::open_strict(deck, OoxmlLimits::default()).unwrap();
    let err = pkg.read_resolved("ppt/slides/slide1.xml").unwrap_err();
    assert_eq!(err.code(), "CS-MCE-0001");
    // The namespace is the document's text, so the message does not carry it
    // (`SEC-015`); resolving the part names it.
    assert!(!err.to_string().contains("powerpoint/2010/main"), "{err}");
    let bytes = pkg.read_part("ppt/slides/slide1.xml").unwrap();
    let tree = XmlTree::parse(&bytes, "ppt/slides/slide1.xml", &OoxmlLimits::default()).unwrap();
    assert!(
        resolve_compatibility(&tree)
            .must_understand
            .contains("http://schemas.microsoft.com/office/powerpoint/2010/main")
    );

    // In a branch not taken, it is retained and it does not matter.
    let hidden = slide(
        "<mc:AlternateContent><mc:Choice Requires=\"p14\"><p:grpSp mc:MustUnderstand=\"p14\"/></mc:Choice></mc:AlternateContent>",
    );
    let deck = with_part("generated/minimal.pptx", "ppt/slides/slide1.xml", &hidden);
    let mut pkg = PresentationPackage::open(deck, OoxmlLimits::default()).unwrap();
    let (tree, _) = pkg.read_resolved("ppt/slides/slide1.xml").unwrap();
    assert_eq!(tree.to_bytes(), hidden.as_bytes());
}

// --- STRICT-01 ---------------------------------------------------------------------

fn schema_namespace(path: &Path) -> String {
    let text = std::fs::read_to_string(path).unwrap();
    let (_, rest) = text.split_once("targetNamespace=\"").unwrap();
    rest.split('"').next().unwrap().to_owned()
}

/// The table is held to the schemas it describes: each file in `ooxml-strict` and its
/// namesake in `ooxml` are one vocabulary.
#[test]
fn the_strict_map_agrees_with_the_vendored_schemas() {
    let schemas = fixtures().join("../schemas");
    let mut checked = 0;
    for entry in std::fs::read_dir(schemas.join("ooxml-strict")).unwrap() {
        let path = entry.unwrap().path();
        let strict = schema_namespace(&path);
        let transitional = schema_namespace(&schemas.join("ooxml").join(path.file_name().unwrap()));
        assert!(
            strict.starts_with("http://purl.oclc.org/ooxml/"),
            "{path:?}"
        );
        assert_eq!(to_transitional(&strict), transitional, "{path:?}");
        assert_eq!(to_strict(&transitional), strict, "{path:?}");
        assert!(is_understood(&strict) && is_understood(&transitional));
        checked += 1;
    }
    assert_eq!(checked, 14);
}

/// Synthetic: mapped from a Transitional deck by the generator.
#[test]
fn a_strict_deck_reads_as_transitional_and_writes_back_as_strict() {
    let path = "generated/strict-synthetic.pptx";
    let bytes = read(path);
    let mut pkg = PresentationPackage::open(bytes.clone(), OoxmlLimits::default()).unwrap();
    assert_eq!(pkg.conformance(), Conformance::Strict);
    assert!(pkg.findings().is_empty(), "{:?}", pkg.findings());

    let part = pkg.read_part("ppt/slides/slide1.xml").unwrap();
    let (tree, compatibility) = pkg.read_resolved("ppt/slides/slide1.xml").unwrap();
    assert!(compatibility.unknown_namespaces.is_empty());
    assert_eq!(
        tree.namespace(tree.root()).unwrap(),
        "http://purl.oclc.org/ooxml/presentationml/main"
    );
    assert_eq!(
        tree.normalized_namespace(tree.root()).unwrap(),
        "http://schemas.openxmlformats.org/presentationml/2006/main"
    );
    assert_eq!(tree.to_bytes(), part);
    assert!(
        !String::from_utf8(part)
            .unwrap()
            .contains("schemas.openxmlformats.org/presentationml")
    );
    assert_eq!(pkg.save_unedited(), bytes);

    let pml = "http://schemas.openxmlformats.org/presentationml/2006/main";
    assert_eq!(pkg.conformance().spell(pml), to_strict(pml));
    assert_eq!(Conformance::Transitional.spell(to_strict(pml)), pml);
}

// --- WR-01: forced rewrite through the tree -----------------------------------------

/// Every XML part, parsed and written back through the tree.
fn forced_rewrite(pkg: &mut PresentationPackage, path: &str) -> Vec<u8> {
    let limits = OoxmlLimits::default();
    let mut edits = PartEdits::default();
    let mut parts = 0;
    for name in pkg.part_names() {
        if name.ends_with(".xml") || name.ends_with(".rels") {
            let bytes = pkg.read_part(&name).unwrap();
            let tree = XmlTree::parse(&bytes, &name, &limits)
                .unwrap_or_else(|e| panic!("{path}: {name}: {e}"));
            edits.replace(name, tree.to_bytes());
            parts += 1;
        }
    }
    assert!(parts > 5, "{path}: only {parts} XML parts");
    pkg.save_with(&edits).unwrap()
}

/// Nothing external runs on `fixtures/hostile`, so F4 is compared on the others.
fn f4_may_run_on(path: &str) -> bool {
    path.starts_with("generated/") || path.starts_with("producers/")
}

/// Fidelity through the tree: every part byte for byte, the same findings, and the
/// same F4 result.
#[test]
fn a_forced_rewrite_through_the_tree_changes_nothing() {
    let f4 = f4_ready();
    let work = scratch("forced-rewrite");
    let mut compared = 0;
    for path in opening_decks() {
        let source = read(&path);
        let mut before = PresentationPackage::open(source.clone(), OoxmlLimits::default()).unwrap();
        let rewritten = forced_rewrite(&mut before, &path);
        // Every XML entry was recompressed, or this would prove nothing.
        assert_ne!(rewritten, source, "{path}: the rewrite did not happen");

        let mut after =
            PresentationPackage::open(rewritten.clone(), OoxmlLimits::default()).unwrap();
        assert_eq!(before.part_names(), after.part_names(), "{path}");
        assert_eq!(before.findings(), after.findings(), "{path}");
        for name in before.part_names() {
            assert_eq!(
                before.read_part(&name).unwrap(),
                after.read_part(&name).unwrap(),
                "{path}: {name}"
            );
        }

        if f4 && f4_may_run_on(&path) {
            let file_name = Path::new(&path).file_name().unwrap();
            let copy = work.join(file_name);
            std::fs::write(&copy, &rewritten).unwrap();
            let (want, _) = run_f4(&["--json", fixtures().join(&path).to_str().unwrap()]);
            let (got, _) = run_f4(&["--json", copy.to_str().unwrap()]);
            assert_eq!(want, got, "{path}: F4 differs after the rewrite");
            compared += 1;
        }
    }
    if f4 {
        assert!(compared >= 14, "F4 compared on only {compared} decks");
    }
}

// --- DOC-009 --------------------------------------------------------------------------

/// A finding repeats a relationship id only in the form the schema gives one.
#[test]
fn a_finding_leaves_out_an_id_that_is_not_an_ncname() {
    let rels = |id: &str| {
        format!(
            "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
             <Relationship Id=\"{id}\" Type=\"urn:unknown-type\" Target=\"http://example.com/\" TargetMode=\"External\"/>\
             </Relationships>"
        )
    };
    let findings = |id: &str| {
        let deck = with_part(MINIMAL, "ppt/slides/_rels/slide1.xml.rels", &rels(id));
        let pkg = PresentationPackage::open(deck, OoxmlLimits::default()).unwrap();
        pkg.findings()
            .iter()
            .filter(|f| f.part == "ppt/slides/slide1.xml")
            .map(|f| f.rel_id.clone())
            .collect::<Vec<_>>()
    };
    let plain = findings("rId9");
    assert!(plain.contains(&Some("rId9".to_owned())), "{plain:?}");
    let odd = findings("a secret sentence, not an id");
    assert!(!odd.is_empty());
    assert!(odd.iter().all(Option::is_none), "{odd:?}");
}

// --- F4-01 ---------------------------------------------------------------------------

/// F4 needs `python3` and `xmllint`, which not every CI runner has. Set `F4_REQUIRED`
/// where they must be there, so a missing tool fails instead of skipping.
fn f4_ready() -> bool {
    let has = |tool: &str, arg: &str| {
        Command::new(tool)
            .arg(arg)
            .output()
            .is_ok_and(|o| o.status.success())
    };
    let ready = has("python3", "--version") && has("xmllint", "--version");
    if !ready {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
        eprintln!("SKIPPED: python3 or xmllint is missing, so F4 was not run");
    }
    ready
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_f4(args: &[&str]) -> (String, i32) {
    let script = fixtures().join("../tools/f4/validate.py");
    let out = Command::new("python3")
        .arg(script)
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let code = out.status.code().unwrap();
    assert!(
        code == 0 || code == 1,
        "F4 could not run: {code}: {stdout}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    (stdout, code)
}

fn f4_on(dir: &Path, name: &str, deck: &[u8]) -> (String, i32) {
    let path = dir.join(name);
    std::fs::write(&path, deck).unwrap();
    run_f4(&[path.to_str().unwrap()])
}

fn part_status<'a>(report: &'a str, part: &str) -> &'a str {
    report
        .lines()
        .find(|l| l.split_whitespace().nth(1) == Some(part))
        .and_then(|l| l.split_whitespace().next())
        .unwrap_or_else(|| panic!("{part} is not in: {report}"))
}

fn part_line<'a>(report: &'a str, part: &str) -> &'a str {
    report
        .lines()
        .find(|l| l.split_whitespace().nth(1) == Some(part))
        .unwrap_or_else(|| panic!("{part} is not in: {report}"))
}

const SLIDE_PART: &str = "ppt/slides/slide1.xml";
const MINIMAL: &str = "generated/minimal.pptx";

#[test]
fn f4_fails_a_seeded_invalid_part_and_only_that_part() {
    if !f4_ready() {
        return;
    }
    let work = scratch("f4-seeded");
    let (report, code) = run_f4(&[fixtures().join(MINIMAL).to_str().unwrap()]);
    assert_eq!(code, 0, "{report}");
    assert_eq!(part_status(&report, SLIDE_PART), "valid");

    let deck = with_part(MINIMAL, SLIDE_PART, &slide("<p:notAnElement/>"));
    let (seeded, code) = f4_on(&work, "minimal.pptx", &deck);
    assert_eq!(code, 1, "{seeded}");
    assert_eq!(part_status(&seeded, SLIDE_PART), "invalid", "{seeded}");
    assert_eq!(part_status(&seeded, "ppt/presentation.xml"), "valid");
    assert!(seeded.contains("notAnElement"));
    // What the forced rewrite compares can tell these two apart.
    let path = work.join("minimal.pptx");
    let (seeded_json, _) = run_f4(&["--json", path.to_str().unwrap()]);
    let (minimal_json, _) = run_f4(&["--json", fixtures().join(MINIMAL).to_str().unwrap()]);
    assert_ne!(seeded_json, minimal_json);
}

/// F4-04: VML is schema-checked. Before, `.vml` parts were never read, and
/// `vml-main.xsd` could not compile.
#[test]
fn f4_fails_a_seeded_invalid_vml_part() {
    if !f4_ready() {
        return;
    }
    const OLE: &str = "generated/area-ole.pptx";
    const VML_PART: &str = "ppt/drawings/vmlDrawing1.vml";
    let work = scratch("f4-vml");
    let (report, code) = run_f4(&[fixtures().join(OLE).to_str().unwrap()]);
    assert_eq!(code, 0, "{report}");
    assert_eq!(part_status(&report, VML_PART), "valid", "{report}");

    let vml = "<xml xmlns:v=\"urn:schemas-microsoft-com:vml\" \
        xmlns:o=\"urn:schemas-microsoft-com:office:office\">\
        <o:shapelayout v:ext=\"edit\"><o:idmap v:ext=\"edit\" data=\"1\"/></o:shapelayout>\
        <v:shape id=\"s\" v:notAnAttribute=\"1\"><v:notAnElement/></v:shape></xml>";
    let (seeded, code) = f4_on(&work, "area-ole.pptx", &with_part(OLE, VML_PART, vml));
    assert_eq!(code, 1, "{seeded}");
    assert_eq!(part_status(&seeded, VML_PART), "invalid", "{seeded}");
    assert!(seeded.contains("notAnElement"), "{seeded}");
}

#[test]
fn f4_refuses_a_document_type_and_never_reports_its_text() {
    if !f4_ready() {
        return;
    }
    let work = scratch("f4-doctype");
    let xml = "<!DOCTYPE r [<!ENTITY x SYSTEM \"file:///etc/passwd\">]><r>&x;</r>";
    let (report, code) = f4_on(&work, "doctype.pptx", &with_part(MINIMAL, SLIDE_PART, xml));
    assert_eq!(code, 1, "{report}");
    assert_eq!(part_status(&report, SLIDE_PART), "refused", "{report}");
    assert!(!report.contains("passwd"));
}

#[test]
fn f4_validates_the_branch_that_applies_and_not_the_other() {
    if !f4_ready() {
        return;
    }
    let work = scratch("f4-branches");
    // Garbage in the branch not taken, and a namespace declared only on the branch
    // that is: valid.
    let fine = slide(
        "<mc:AlternateContent><mc:Choice Requires=\"p14\"><p:notAnElement/></mc:Choice>\
         <mc:Fallback xmlns:q=\"http://schemas.openxmlformats.org/presentationml/2006/main\"><q:grpSp><q:nvGrpSpPr><q:cNvPr id=\"2\" name=\"\"/><q:cNvGrpSpPr/><q:nvPr/></q:nvGrpSpPr><q:grpSpPr/></q:grpSp></mc:Fallback>\
         </mc:AlternateContent>",
    );
    let (report, code) = f4_on(&work, "fine.pptx", &with_part(MINIMAL, SLIDE_PART, &fine));
    assert_eq!(code, 0, "{report}");
    assert!(
        part_line(&report, SLIDE_PART).contains("branches=fallback"),
        "{report}"
    );

    // The same garbage in the branch that applies: invalid.
    let bad = slide(
        "<mc:AlternateContent><mc:Choice Requires=\"p14\"><p:grpSp/></mc:Choice>\
         <mc:Fallback><p:notAnElement/></mc:Fallback></mc:AlternateContent>",
    );
    let (report, code) = f4_on(&work, "bad.pptx", &with_part(MINIMAL, SLIDE_PART, &bad));
    assert_eq!(code, 1, "{report}");
    assert_eq!(part_status(&report, SLIDE_PART), "invalid", "{report}");

    // A must-understand namespace we do not know.
    let must = slide("<p:grpSp mc:MustUnderstand=\"p14\"/>");
    let (report, code) = f4_on(&work, "must.pptx", &with_part(MINIMAL, SLIDE_PART, &must));
    assert_eq!(code, 1, "{report}");
    assert_eq!(
        part_status(&report, SLIDE_PART),
        "must-understand",
        "{report}"
    );
}

#[test]
fn f4_validates_strict_against_the_strict_schemas() {
    if !f4_ready() {
        return;
    }
    let deck = fixtures().join("generated/strict-synthetic.pptx");
    let (report, code) = run_f4(&[deck.to_str().unwrap()]);
    assert_eq!(code, 0, "{report}");
    assert!(
        part_line(&report, SLIDE_PART).contains("[ooxml-strict/pml.xsd]"),
        "{report}"
    );

    // A Transitional element in a Strict slide is not Strict.
    let work = scratch("f4-strict");
    let mixed = slide("<a:notInStrict/>").replace(
        "http://schemas.openxmlformats.org/presentationml/2006/main",
        "http://purl.oclc.org/ooxml/presentationml/main",
    );
    let (report, code) = f4_on(
        &work,
        "mixed.pptx",
        &with_part("generated/strict-synthetic.pptx", SLIDE_PART, &mixed),
    );
    assert_eq!(code, 1, "{report}");
    assert_eq!(part_status(&report, SLIDE_PART), "invalid", "{report}");
}

#[test]
fn f4_will_not_run_on_the_hostile_fixtures() {
    if !f4_ready() {
        return;
    }
    let script = fixtures().join("../tools/f4/validate.py");
    let out = Command::new("python3")
        .arg(script)
        .arg(fixtures().join("hostile"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("refusing fixtures/hostile"));
}

#[test]
fn the_f4_tables_are_the_engines_tables() {
    if !f4_ready() {
        return;
    }
    let (tables, _) = run_f4(&["--print-tables"]);
    let mut map = Vec::new();
    let mut understood = Vec::new();
    for line in tables.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        match cols[0] {
            "map" => map.push((cols[1], cols[2])),
            "understood" => understood.push(cols[1]),
            other => panic!("{other}"),
        }
    }
    assert_eq!(map, STRICT_TO_TRANSITIONAL);
    let mut ours = understood_namespaces();
    ours.sort_unstable();
    assert_eq!(understood, ours);
}

/// The decision as `validate.py --mce-report` prints it.
fn summary(c: &Compatibility) -> String {
    let branch = |a: &AlternateResolution| match a.branch {
        Branch::Choice { index, .. } => format!("choice:{index}"),
        Branch::Fallback { .. } => "fallback".to_owned(),
        Branch::None => "none".to_owned(),
    };
    let alternates: Vec<String> = c.alternates.iter().map(branch).collect();
    let must: Vec<&str> = c.must_understand.iter().map(String::as_str).collect();
    let unknown: Vec<&str> = c.unknown_namespaces.iter().map(String::as_str).collect();
    let mut broken: Vec<&str> = c.nonconformances.iter().map(|n| n.name()).collect();
    broken.sort_unstable();
    format!(
        "alternates={} must_understand={} unknown={} nonconformances={} ignored={} unwrapped={} ignored_attributes={} extensions={}",
        alternates.join(","),
        must.join("|"),
        unknown.join("|"),
        broken.join(","),
        c.ignored_elements.len(),
        c.unwrapped_elements.len(),
        c.ignored_attributes,
        c.extensions.len(),
    )
}

/// Two implementations of Part 3, one engine and one tool, must not disagree: what F4
/// validates has to be what the engine reads.
#[test]
fn the_engine_and_f4_resolve_markup_compatibility_alike() {
    if !f4_ready() {
        return;
    }
    let limits = OoxmlLimits::default();
    let work = scratch("f4-parity");
    let mut inputs: Vec<(String, Vec<u8>)> = [
        "<a mc:MustUnderstand=\"p14 p\"/>",
        "<a mc:MustUnderstand=\"nope\"/>",
        "<a mc:Ignorable=\"mc\"/>",
        "<a mc:ProcessContent=\"x\"/>",
        "<a mc:ProcessContent=\"x:b\"/>",
        "<a mc:Other=\"1\"/>",
        "<nope:a/>",
        "<x:a/><y:b y:c=\"1\"/>",
        "<k mc:Ignorable=\"x\" x:at=\"1\"><x:a/><x:b/></k>",
        "<k mc:Ignorable=\"x\" mc:ProcessContent=\"x:*\"><x:a><x:b/></x:a></k><x:c/>",
        "<k mc:Ignorable=\"x\" mc:ProcessContent=\"x:a\"><x:a xml:lang=\"en\"/><x:b/></k>",
        "<extLst><ext><mc:AlternateContent/><nope:b/><p14:c/></ext></extLst>",
        "<xfrm><ext><p14:c/></ext></xfrm>",
        "<mc:Choice Requires=\"p\"/><mc:Fallback/><mc:Other/>",
        "<mc:AlternateContent><mc:Choice Requires=\"p14\"/><mc:Choice Requires=\"p\"/><mc:Choice Requires=\"p\"/><mc:Fallback/></mc:AlternateContent>",
        "<mc:AlternateContent><mc:Choice Requires=\"p p14\"/><mc:Fallback/></mc:AlternateContent>",
        "<mc:AlternateContent><mc:Choice Requires=\"p14\"/></mc:AlternateContent>",
        "<mc:AlternateContent><mc:Choice/><mc:Choice Requires=\" \"/><mc:Choice Requires=\"nope\"/><mc:Choice Requires=\"mc\"/></mc:AlternateContent>",
        "<mc:AlternateContent><mc:Fallback/><mc:Choice Requires=\"p\"/><mc:Fallback/></mc:AlternateContent>",
        "<mc:AlternateContent k=\"1\" xml:lang=\"en\"><q/><mc:Other/><mc:Fallback p14:z=\"1\"/></mc:AlternateContent>",
        "<mc:AlternateContent mc:Ignorable=\"x\" x:ok=\"1\"><x:future/><mc:Choice Requires=\"p\" mc:MustUnderstand=\"p14\"><mc:AlternateContent><mc:Choice Requires=\"p14\"/><mc:Fallback/></mc:AlternateContent></mc:Choice></mc:AlternateContent>",
        "<mc:AlternateContent><mc:Choice Requires=\"p14\"><evil mc:MustUnderstand=\"p14\"><mc:AlternateContent/></evil></mc:Choice></mc:AlternateContent>",
        "<q xmlns=\"\"/><q xmlns:p14=\"urn:rebound\"><mc:AlternateContent><mc:Choice Requires=\"p14\"/></mc:AlternateContent></q>",
    ]
    .iter()
    .enumerate()
    .map(|(i, body)| (format!("vector {i}"), slide(body).into_bytes()))
    .collect();
    for deck in [
        "generated/mce-alternate-content.pptx",
        "generated/strict-synthetic.pptx",
        "producers/libreoffice-26.8-three-slides.pptx",
        "producers/libreoffice-26.8-image.pptx",
    ] {
        let mut pkg = PresentationPackage::open(read(deck), OoxmlLimits::default()).unwrap();
        for name in pkg.part_names() {
            if name.ends_with(".xml") {
                let bytes = pkg.read_part(&name).unwrap();
                inputs.push((format!("{deck}: {name}"), bytes));
            }
        }
    }
    assert!(inputs.len() > 50);
    let part = work.join("part.xml");
    for (label, bytes) in inputs {
        std::fs::write(&part, &bytes).unwrap();
        let (theirs, _) = run_f4(&["--mce-report", part.to_str().unwrap()]);
        // An undeclared element prefix is not namespace-well-formed, so the walker
        // refuses the part (`XML-03`). F4 reads without namespaces and reports it;
        // the two agree that the part is broken.
        let tree = match XmlTree::parse(&bytes, "t", &limits) {
            Ok(tree) => tree,
            Err(err) => {
                assert_eq!(err.code(), "CS-XML-0004", "{label}");
                assert!(
                    theirs.contains("undeclared-element-prefix"),
                    "{label}: {theirs}"
                );
                continue;
            }
        };
        assert_eq!(
            summary(&resolve_compatibility(&tree)),
            theirs.trim_end(),
            "{label}"
        );
    }
}

// --- F4-02, F4-03: what PowerPoint writes, and the real errors next to it ------------

fn bullet_size(val: &str) -> String {
    slide(&format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"2\" name=\"\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr/>\
         <p:txBody><a:bodyPr/><a:p><a:pPr><a:buSzPct val=\"{val}\"/></a:pPr></a:p></p:txBody></p:sp>"
    ))
}

/// A chart with the named axis ids, `tail` after its plot area and `series` in its bar chart.
fn chart(axis: &str, cross: &str, series: &str, tail: &str) -> String {
    format!(
        "<c:chartSpace xmlns:c=\"http://schemas.openxmlformats.org/drawingml/2006/chart\"><c:chart>\
         <c:plotArea><c:barChart><c:barDir val=\"bar\"/>{series}<c:axId val=\"{axis}\"/><c:axId val=\"{cross}\"/></c:barChart>\
         <c:catAx><c:axId val=\"{axis}\"/><c:scaling/><c:delete val=\"0\"/><c:axPos val=\"b\"/><c:crossAx val=\"{cross}\"/></c:catAx>\
         <c:valAx><c:axId val=\"{cross}\"/><c:scaling/><c:delete val=\"0\"/><c:axPos val=\"l\"/><c:crossAx val=\"{axis}\"/></c:valAx>\
         </c:plotArea>{tail}</c:chart></c:chartSpace>"
    )
}

fn plain_chart(tail: &str) -> String {
    chart("1", "2", "", tail)
}

/// The status F4 gives the slide part, with and without the allowances.
fn f4_with_and_without(work: &Path, xml: &str) -> (String, String) {
    let (with, _) = f4_on(work, "seeded.pptx", &with_part(MINIMAL, SLIDE_PART, xml));
    let path = work.join("seeded.pptx");
    let (without, _) = run_f4(&["--no-allowances", path.to_str().unwrap()]);
    (
        part_status(&with, SLIDE_PART).to_owned(),
        part_status(&without, SLIDE_PART).to_owned(),
    )
}

#[test]
fn f4_allowance_each_accepts_what_powerpoint_writes_and_is_needed() {
    if !f4_ready() {
        return;
    }
    let work = scratch("f4-allowed");
    let c15 = "xmlns:c15=\"http://schemas.microsoft.com/office/drawing/2012/chart\"";
    let cases = [
        ("F4A-1 numeric bullet size", bullet_size("130000")),
        ("F4A-1 lowest numeric", bullet_size("25000")),
        ("F4A-1 highest numeric", bullet_size("400000")),
        (
            "F4A-3 and 4 signed axis ids",
            chart("-1884094432", "-1884097184", "", ""),
        ),
        (
            "F4A-5 empty extension",
            plain_chart("<c:extLst><c:ext uri=\"{X}\"/></c:extLst>"),
        ),
        (
            "F4A-5 extension with two children",
            plain_chart(&format!(
                "<c:extLst><c:ext uri=\"{{X}}\" {c15}><c15:layout/><c15:showLeaderLines val=\"0\"/></c:ext></c:extLst>"
            )),
        ),
        (
            "F4A-6 showDLblsOverMax after the extension list",
            plain_chart(
                "<c:extLst><c:ext uri=\"{X}\" xmlns:u=\"urn:u\"><u:v/></c:ext></c:extLst><c:showDLblsOverMax val=\"0\"/>",
            ),
        ),
    ];
    for (name, xml) in cases {
        let (with, without) = f4_with_and_without(&work, &xml);
        assert_eq!(with, "valid", "{name} should pass F4");
        assert_eq!(without, "invalid", "{name}: this allowance is not needed");
    }
    // What the XSD always accepted still passes.
    for xml in [
        bullet_size("100%"),
        plain_chart("<c:showDLblsOverMax val=\"0\"/><c:extLst/>"),
    ] {
        let (with, without) = f4_with_and_without(&work, &xml);
        assert_eq!((with.as_str(), without.as_str()), ("valid", "valid"));
    }
}

#[test]
fn f4_allowance_none_lets_a_real_error_through() {
    if !f4_ready() {
        return;
    }
    let work = scratch("f4-refused");
    let idx = "<c:ser><c:idx val=\"-1\"/><c:order val=\"0\"/></c:ser>";
    let order = "<c:ser><c:idx val=\"0\"/><c:order val=\"-1\"/></c:ser>";
    let over = "<c:showDLblsOverMax val=\"0\"/>";
    let cases = [
        ("bullet size under 25%", bullet_size("24999")),
        ("bullet size over 400%", bullet_size("400001")),
        ("bullet size of text", bullet_size("large")),
        ("bullet size percent over 400%", bullet_size("401%")),
        ("axis id under 32 bits", chart("-2147483649", "2", "", "")),
        ("axis id over 32 bits", chart("4294967296", "2", "", "")),
        ("axis id not an integer", chart("1.5", "2", "", "")),
        ("cross axis not an integer", chart("1", "x", "", "")),
        ("a signed series index", chart("1", "2", idx, "")),
        ("a signed series order", chart("1", "2", order, "")),
        (
            "a second showDLblsOverMax",
            plain_chart(&format!("{over}<c:extLst/>{over}")),
        ),
        (
            "showDLblsOverMax twice, together",
            plain_chart(&format!("{over}{over}")),
        ),
        (
            "an unknown element where the empty extension is allowed",
            plain_chart("<c:ext uri=\"{X}\"/>"),
        ),
        ("an unknown chart element", plain_chart("<c:notAnElement/>")),
    ];
    for (name, xml) in cases {
        let (with, _) = f4_with_and_without(&work, &xml);
        assert_eq!(with, "invalid", "F4 accepted {name}");
    }
}

#[test]
fn f4_stops_on_a_stale_allowance_or_a_schema_that_will_not_compile() {
    if !f4_ready() {
        return;
    }
    let out = Command::new("python3")
        .arg(fixtures().join("../tools/f4/test_validate.py"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn f4_never_relaxes_strict() {
    if !f4_ready() {
        return;
    }
    let work = scratch("f4-strict-numeric");
    let transitional = bullet_size("130000");
    let strict_slide = transitional
        .replace(
            "http://schemas.openxmlformats.org/presentationml/2006/main",
            "http://purl.oclc.org/ooxml/presentationml/main",
        )
        .replace(
            "http://schemas.openxmlformats.org/drawingml/2006/main",
            "http://purl.oclc.org/ooxml/drawingml/main",
        );
    let (report, code) = f4_on(
        &work,
        "strict.pptx",
        &with_part("generated/strict-synthetic.pptx", SLIDE_PART, &strict_slide),
    );
    assert_eq!(code, 1, "{report}");
    assert_eq!(part_status(&report, SLIDE_PART), "invalid", "{report}");
}

#[test]
fn f4_passes_area_text_with_nothing_recorded() {
    if !f4_ready() {
        return;
    }
    let deck = fixtures().join("generated/area-text.pptx");
    let (report, code) = run_f4(&["--no-known", deck.to_str().unwrap()]);
    assert_eq!(code, 0, "{report}");
    assert!(!report.contains("recorded"), "{report}");
}

/// F4-06: the VML fixture, every element and value, is schema-valid but for the one
/// attribute no VML type allows, `o:wrapcoords`.
#[test]
fn f4_finds_only_wrapcoords_wrong_in_area_vml() {
    if !f4_ready() {
        return;
    }
    let part = "ppt/drawings/vmlDrawing1.vml";
    let deck = fixtures().join("generated/area-vml.pptx");
    let (report, code) = run_f4(&["--no-known", deck.to_str().unwrap()]);
    assert_eq!(code, 1, "{report}");
    let messages: Vec<&str> = report
        .lines()
        .skip_while(|l| l.split_whitespace().nth(1) != Some(part))
        .skip(1)
        .take_while(|l| l.starts_with("      "))
        .collect();
    assert_eq!(messages.len(), 1, "{report}");
    assert!(messages[0].contains("office}wrapcoords"), "{report}");
    let (report, code) = run_f4(&[deck.to_str().unwrap()]);
    assert_eq!(code, 0, "{report}");
    assert_eq!(part_status(&report, part), "recorded", "{report}");
}

#[test]
fn f4_recorded_failures_cover_only_what_they_say() {
    if !f4_ready() {
        return;
    }
    let name = "real/sdk-strict-animclr.pptx";
    let deck = fixtures().join(name);
    let (report, code) = run_f4(&[deck.to_str().unwrap()]);
    assert_eq!(code, 0, "{report}");
    assert_eq!(part_status(&report, "ppt/theme/theme1.xml"), "recorded");

    let (report, code) = run_f4(&["--no-known", deck.to_str().unwrap()]);
    assert_eq!(code, 1, "{report}");
    assert_eq!(part_status(&report, "ppt/theme/theme1.xml"), "invalid");

    // A real error in the same deck, in a part that has a recorded failure, still fails.
    let work = scratch("f4-recorded");
    let pkg = PresentationPackage::open(read(name), OoxmlLimits::default()).unwrap();
    let mut pkg = pkg;
    let theme = String::from_utf8(pkg.read_part("ppt/theme/theme1.xml").unwrap()).unwrap();
    let seeded = theme.replacen(
        "</a:themeElements>",
        "<a:notAnElement/></a:themeElements>",
        1,
    );
    assert_ne!(seeded, theme);
    let (report, code) = f4_on(
        &work,
        "sdk-strict-animclr.pptx",
        &with_part(name, "ppt/theme/theme1.xml", &seeded),
    );
    assert_eq!(code, 1, "{report}");
    assert_eq!(part_status(&report, "ppt/theme/theme1.xml"), "invalid");
    // The same part under another deck's name is not covered by the row.
    let (report, code) = f4_on(
        &work,
        "other.pptx",
        &with_part(name, "ppt/theme/theme1.xml", &theme),
    );
    assert_eq!(code, 1, "{report}");
}
