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

fn cases() -> Vec<(String, String)> {
    let manifest = std::fs::read_to_string(fixtures().join("manifest.tsv")).unwrap();
    manifest
        .lines()
        .skip(1)
        .map(|line| {
            let mut cols = line.split('\t');
            let path = cols.next().unwrap().to_owned();
            cols.next();
            (path, cols.next().unwrap().to_owned())
        })
        .collect()
}

fn read(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

#[test]
fn every_fixture_meets_its_expectation() {
    let cases = cases();
    assert!(cases.len() > 30, "only {} fixtures", cases.len());
    for (path, expect) in cases {
        let result = PresentationPackage::open(read(&path), OoxmlLimits::default());
        if let Some(code) = expect.strip_prefix("refuse:") {
            match result {
                Ok(_) => panic!("{path}: admitted, expected {code}"),
                Err(err) => assert_eq!(err.code(), code, "{path}: {err}"),
            }
            continue;
        }
        let pkg = result.unwrap_or_else(|err| panic!("{path}: {err}"));
        let mut codes: Vec<&str> = pkg.findings().iter().map(|f| f.kind.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        let wanted: Vec<&str> = expect.split(' ').skip(1).collect();
        assert_eq!(codes, wanted, "{path}: {:?}", pkg.findings());
    }
}

fn opening_decks() -> Vec<String> {
    cases()
        .into_iter()
        .filter(|(_, e)| e.starts_with("open"))
        .map(|(p, _)| p)
        .collect()
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

#[test]
fn opening_a_deck_does_not_read_its_media() {
    // Only structure is read on open, so a deck full of media opens cheaply and
    // no decoder is reachable from opening a file.
    let mut pkg =
        PresentationPackage::open(read("generated/image.pptx"), OoxmlLimits::default()).unwrap();
    assert!(pkg.part_names().iter().any(|n| n.ends_with("image1.png")));
    assert!(
        pkg.read_part("PPT/MEDIA/IMAGE1.PNG")
            .unwrap()
            .starts_with(b"\x89PNG")
    );
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
    let mut pkg = PresentationPackage::open(deck, OoxmlLimits::default()).unwrap();
    let err = pkg.read_resolved("ppt/slides/slide1.xml").unwrap_err();
    assert_eq!(err.code(), "CS-MCE-0001");
    assert!(err.to_string().contains("powerpoint/2010/main"));

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
        let tree = XmlTree::parse(&bytes, "t", &limits).unwrap();
        std::fs::write(&part, &bytes).unwrap();
        let (theirs, _) = run_f4(&["--mce-report", part.to_str().unwrap()]);
        assert_eq!(
            summary(&resolve_compatibility(&tree)),
            theirs.trim_end(),
            "{label}"
        );
    }
}
