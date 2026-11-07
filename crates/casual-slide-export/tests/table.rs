//! Tier 1G export: edit a cell's text and fill, save, reopen; only those nodes changed
//! and every other part is byte-identical. Every modelled row is written and removed;
//! decks are a fixed point; edited output validates; hostile models are refused with
//! nothing saved.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use casual_slide_export::slide::{patch_slide_part, save_slide_parts};
use casual_slide_export::table::{patch_table_styles, save_table_styles};
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::slide::{
    ImportedSlides, SlideLimits, import_slide_part, import_slide_parts,
};
use casual_slide_import::table::import_table_styles;
use casual_slide_model::color::Color;
use casual_slide_model::fill::Fill;
use casual_slide_model::shape::ShapeNode;
use casual_slide_model::slide::{SlideKind, SlidePart, SlideParts};
use casual_slide_model::table::{
    Table, TableCell, TablePartKind, TableStyleChoice, TableStyleList,
};
use casual_slide_model::text::TextItem;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};

const NS: &str = concat!(
    r#"xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" "#,
    r#"xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#
);
const EVERY: &str = "generated/table-every-property.pptx";
const MERGED: &str = "generated/table-merged-cells.pptx";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn imported(bytes: &[u8]) -> ImportedSlides {
    let mut pkg = open(bytes.to_vec());
    let import = ImportLimits::default();
    let deck = import_deck(&mut pkg, &import).unwrap().deck;
    import_slide_parts(
        &mut pkg,
        &deck.presentation,
        &SlideLimits::default(),
        &import,
    )
    .unwrap()
}

fn model(bytes: &[u8]) -> SlideParts {
    imported(bytes).parts
}

fn parts(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut pkg = open(bytes.to_vec());
    pkg.part_names()
        .into_iter()
        .map(|n| {
            let data = pkg.read_part(&n).unwrap();
            (n, data)
        })
        .collect()
}

fn named_mut<'a>(part: &'a mut SlidePart, name: &str) -> &'a mut ShapeNode {
    let id = part
        .shape_tree()
        .unwrap()
        .descendants()
        .find(|n| n.non_visual.as_ref().and_then(|v| v.name.as_deref()) == Some(name))
        .unwrap_or_else(|| panic!("no {name}"))
        .id;
    part.node_mut(id).unwrap()
}

fn all_decks() -> Vec<String> {
    let mut out = Vec::new();
    for set in ["generated", "producers", "real"] {
        let mut names: Vec<String> = std::fs::read_dir(fixtures().join(set))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| !p.extension().is_some_and(|e| e == "tsv" || e == "md"))
            .map(|p| format!("{set}/{}", p.file_name().unwrap().to_string_lossy()))
            .collect();
        names.sort();
        out.extend(names);
    }
    out
}

/// Save a deck with its first slide edited, returning the bytes or the code.
fn edit_slide(source: &[u8], edit: impl FnOnce(&mut SlidePart)) -> Result<Vec<u8>, String> {
    let mut wanted = model(source);
    let ordinal = wanted.structure.slides[0];
    edit(wanted.get_mut(ordinal).unwrap());
    let mut pkg = open(source.to_vec());
    let outcome = save_slide_parts(
        &mut pkg,
        &wanted.structure,
        &[wanted.get(ordinal).unwrap()],
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .map_err(|e| e.code().to_owned())?;
    assert_eq!(model(&outcome.bytes), wanted, "reopened as asked");
    Ok(outcome.bytes)
}

fn table_mut<'a>(part: &'a mut SlidePart, name: &str) -> &'a mut Table {
    named_mut(part, name).table.as_mut().unwrap()
}

fn set_first_run(cell: &mut TableCell, text: &str) {
    let body = cell.text.as_mut().unwrap();
    let TextItem::Run(run) = &mut body.paragraphs[0].items[0] else {
        panic!("no run");
    };
    run.text = Some(text.to_owned());
}

fn red() -> Fill {
    Fill::Solid(Some(Color::srgb(0xAB, 0xCD, 0xEF)))
}

/// The bytes of `part` are the source's with exactly these replacements, and no other
/// part differs.
fn only_this_changed(source: &[u8], edited: &[u8], part: &str, changes: &[(&str, &str)]) {
    let old = String::from_utf8(parts(source)[part].clone()).unwrap();
    let new = String::from_utf8(parts(edited)[part].clone()).unwrap();
    let mut expected = old.clone();
    for (from, to) in changes {
        assert_eq!(expected.matches(from).count(), 1, "{from} in {part}");
        expected = expected.replace(from, to);
    }
    assert_eq!(new, expected);
    let (a, b) = (parts(source), parts(edited));
    let changed: Vec<&String> = b
        .iter()
        .filter(|(n, v)| a.get(*n) != Some(v))
        .map(|(n, _)| n)
        .collect();
    assert_eq!(changed, [part]);
    assert_eq!(a.len(), b.len());
}

/// F4 is a question about a deck, and some real decks were not valid before we saw them.
fn f4_failures(bytes: &[u8]) -> Option<String> {
    let have = |tool: &str| Command::new(tool).arg("--version").output().is_ok();
    if !have("xmllint") || !have("python3") {
        assert!(
            std::env::var_os("F4_REQUIRED").is_none(),
            "F4_REQUIRED is set, and python3 or xmllint is missing"
        );
        eprintln!("xmllint or python3 not installed: F4 skipped");
        return None;
    }
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("f4-table");
    std::fs::create_dir_all(&dir).unwrap();
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let file = dir.join(format!("edited-{}-{n}.pptx", std::process::id()));
    std::fs::write(&file, bytes).unwrap();
    let tool = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/f4/validate.py");
    let out = Command::new("python3")
        .arg("-I")
        .arg(tool)
        .arg(&file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();
    (!out.status.success()).then(|| {
        let mut errors: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| l.contains("Schemas validity error"))
            .map(str::to_owned)
            .collect();
        errors.sort();
        errors.join("\n")
    })
}

fn f4_validates(bytes: &[u8]) {
    if let Some(errors) = f4_failures(bytes) {
        panic!("{errors}");
    }
}

/// The acceptance test: a cell's text and fill. Those bytes change, and no others.
#[test]
fn editing_a_cells_text_and_fill_changes_only_those_nodes() {
    let source = read_fixture(EVERY);
    let edited = edit_slide(&source, |slide| {
        let cell = &mut table_mut(slide, "Every Property").rows[1].cells[0];
        set_first_run(cell, "Edited & <new>");
        cell.properties.get_or_insert_with(Default::default).fill = Some(red());
    })
    .unwrap();
    // The `tcPr/` of a plain cell becomes one holding the fill: the only structure touched.
    let old = String::from_utf8(parts(&source)["ppt/slides/slide1.xml"].clone()).unwrap();
    let new = String::from_utf8(parts(&edited)["ppt/slides/slide1.xml"].clone()).unwrap();
    let second_row = old.find(r#"<a:tr h="370000"><a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>Plain"#).unwrap();
    let (head, tail) = old.split_at(second_row);
    let tail = tail
        .replacen("<a:t>Plain</a:t>", "<a:t>Edited &amp; &lt;new&gt;</a:t>", 1)
        .replacen(
            "<a:tcPr/>",
            r#"<a:tcPr><a:solidFill><a:srgbClr val="ABCDEF"/></a:solidFill></a:tcPr>"#,
            1,
        );
    assert_eq!(new, format!("{head}{tail}"));
    let (a, b) = (parts(&source), parts(&edited));
    for (name, bytes) in &a {
        if name != "ppt/slides/slide1.xml" {
            assert_eq!(b.get(name), Some(bytes), "{name} changed");
        }
    }
    f4_validates(&edited);
}

/// An edit in a cell full of what the model does not see keeps all of it.
#[test]
fn an_edit_keeps_rowid_colid_and_every_extension_where_they_are() {
    let source = read_fixture(EVERY);
    let edited = edit_slide(&source, |slide| {
        let t = table_mut(slide, "Every Property");
        let p = t.rows[0].cells[0].properties.as_mut().unwrap();
        p.anchor = Some(casual_slide_model::text::TextAnchor::Bottom);
        t.rows[0].height = None;
        t.rows[0].height = Some(casual_slide_model::text::Written::new(
            casual_slide_model::presentation::Coordinate(400_000),
        ));
    })
    .unwrap();
    only_this_changed(
        &source,
        &edited,
        "ppt/slides/slide1.xml",
        &[
            (r#"anchor="ctr""#, r#"anchor="b""#),
            // The first row only: the second stays.
            (
                r#"<a:tr h="370000"><a:tc rowSpan="1""#,
                r#"<a:tr h="400000"><a:tc rowSpan="1""#,
            ),
        ],
    );
    let text = String::from_utf8(parts(&edited)["ppt/slides/slide1.xml"].clone()).unwrap();
    for kept in [
        "a16:rowId",
        "a16:colId",
        "{00000000-0000-0000-0000-000000000006}",
    ] {
        assert!(text.contains(kept), "{kept}");
    }
    f4_validates(&edited);
}

#[test]
fn another_spelling_of_the_same_value_is_not_an_edit() {
    let source = read_fixture(EVERY);
    // `1in` and 914400 are one value: an equal model changes nothing.
    let same = edit_slide(&source, |slide| {
        let p = table_mut(slide, "Every Property").rows[0].cells[0]
            .properties
            .as_mut()
            .unwrap();
        p.margin_right = Some(casual_slide_model::text::Written::new(
            casual_slide_model::presentation::Coordinate(914_400),
        ));
    })
    .unwrap();
    assert_eq!(same, source);
}

fn read_part(xml: &[u8]) -> SlidePart {
    let tree = XmlTree::parse(xml, "s.xml", &OoxmlLimits::default()).unwrap();
    let limits = ImportLimits::default();
    let mut findings = Findings::new(limits.max_findings);
    let mut color = ColorFindings::default();
    import_slide_part(
        &tree,
        "s.xml",
        &[],
        SlideKind::Slide,
        0,
        &SlideLimits::default(),
        &limits,
        &mut findings,
        &mut color,
    )
    .unwrap()
}

fn patch_between(from: &str, to: &SlidePart) -> Result<Vec<u8>, String> {
    let mut tree = XmlTree::parse(from.as_bytes(), "s.xml", &OoxmlLimits::default()).unwrap();
    patch_slide_part(
        &mut tree,
        "s.xml",
        false,
        &[],
        to,
        &SlideLimits::default(),
        &ImportLimits::default(),
        &OoxmlLimits::default(),
    )
    .map_err(|e| e.code().to_owned())?;
    Ok(tree.to_bytes())
}

/// The bare form of the fixture's table: the same rows and cells, stating nothing.
const BARE_EVERY: &str = concat!(
    r#"<a:tbl><a:tblGrid><a:gridCol w="2000000"/><a:gridCol w="2000000"/></a:tblGrid>"#,
    r#"<a:tr h="370000"><a:tc/><a:tc/></a:tr><a:tr h="370000"><a:tc/><a:tc/></a:tr></a:tbl>"#
);
const BARE_INLINE: &str = r#"<a:tbl><a:tblGrid><a:gridCol w="4000000"/></a:tblGrid><a:tr h="370000"><a:tc/></a:tr></a:tbl>"#;

fn two_frames(one: &str, two: &str) -> String {
    let frame = |id: u32, name: &str, t: &str| {
        format!(
            r#"<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="{id}" name="{name}"/><p:cNvGraphicFramePr><a:graphicFrameLocks noGrp="1"/></p:cNvGraphicFramePr><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x="0" y="0"/><a:ext cx="4000000" cy="1000000"/></p:xfrm><a:graphic><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table">{t}</a:graphicData></a:graphic></p:graphicFrame>"#
        )
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld {NS}><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>{}{}</p:spTree></p:cSld></p:sld>"#,
        frame(2, "Every Property", one),
        frame(3, "Inline Style", two)
    )
}

/// The creating path: every modelled row written from nothing, in schema order, and
/// valid; then taken away again. The model is the fixture's, whose every row the import
/// test holds is there, so this reaches them all.
#[test]
fn every_modelled_row_is_written_from_nothing_valid_and_removed_again() {
    let source = read_fixture(EVERY);
    let full = model(&source);
    let ordinal = full.structure.slides[0];
    let wanted = full.get(ordinal).unwrap().clone();

    let bare = two_frames(BARE_EVERY, BARE_INLINE);
    let bare_part = read_part(bare.as_bytes());
    let mut with_id = wanted.clone();
    with_id.id = bare_part.id;
    renumber(&mut with_id, &bare_part);
    let written = patch_between(&bare, &with_id).unwrap();
    let again = read_part(&written);
    assert_eq!(again, with_id);

    // In a deck, so F4 sees it in place.
    let mut entries = parts(&source);
    entries.insert("ppt/slides/slide1.xml".to_owned(), written.clone());
    let mut pkg = open(source.clone());
    let mut edits = casual_slide_package::PartEdits::default();
    edits.replace("ppt/slides/slide1.xml".to_owned(), written.clone());
    f4_validates(&pkg.save_with(&edits).unwrap());

    // And back: everything the model named goes, and it reads as the bare table.
    let text = String::from_utf8(written).unwrap();
    let mut back = bare_part.clone();
    back.id = with_id.id;
    let bytes = patch_between(&text, &back);
    let out = bytes.unwrap_or_else(|code| panic!("{code}"));
    assert_eq!(read_part(&out), back);
}

/// Ids carry a part's ordinal; a part read alone has another.
fn renumber(part: &mut SlidePart, like: &SlidePart) {
    let want = like.shape_tree().unwrap().id.part;
    fn walk(n: &mut ShapeNode, part: u32) {
        n.id.part = part;
        for c in &mut n.children {
            walk(c, part);
        }
    }
    walk(part.shape_tree_mut().unwrap(), want);
}

#[test]
fn every_style_row_is_written_from_nothing_valid_and_removed_again() {
    let source = read_fixture(EVERY);
    let wanted = imported(&source).table_styles.unwrap();
    let ids: Vec<&str> = wanted
        .styles
        .iter()
        .map(|s| s.id.as_deref().unwrap())
        .collect();
    let bare = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:tblStyleLst xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" def="{}"><a:tblStyle styleId="{}" styleName="Fixture Style"/><a:tblStyle styleId="{}" styleName="Second"/></a:tblStyleLst>"#,
        ids[0], ids[0], ids[1]
    );
    let patch = |from: &str, to: &TableStyleList| -> Vec<u8> {
        let mut tree = XmlTree::parse(from.as_bytes(), "t.xml", &OoxmlLimits::default()).unwrap();
        patch_table_styles(
            &mut tree,
            "t.xml",
            false,
            &[],
            to,
            &SlideLimits::default(),
            &ImportLimits::default(),
            &OoxmlLimits::default(),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        tree.to_bytes()
    };
    let written = patch(&bare, &wanted);
    let mut pkg = open(source.clone());
    let mut edits = casual_slide_package::PartEdits::default();
    edits.replace("ppt/tableStyles.xml".to_owned(), written.clone());
    let deck = pkg.save_with(&edits).unwrap();
    assert_eq!(imported(&deck).table_styles.as_ref(), Some(&wanted));
    f4_validates(&deck);

    // And back.
    let bare_model = {
        let tree = XmlTree::parse(bare.as_bytes(), "t.xml", &OoxmlLimits::default()).unwrap();
        let limits = ImportLimits::default();
        let mut f = Findings::new(limits.max_findings);
        let mut c = ColorFindings::default();
        import_table_styles(
            &tree,
            "t.xml",
            &[],
            &SlideLimits::default(),
            &limits,
            &mut f,
            &mut c,
        )
        .unwrap()
    };
    let out = patch(&String::from_utf8(written).unwrap(), &bare_model);
    let mut f = Findings::new(100);
    let mut c = ColorFindings::default();
    let tree = XmlTree::parse(&out, "t.xml", &OoxmlLimits::default()).unwrap();
    let read = import_table_styles(
        &tree,
        "t.xml",
        &[],
        &SlideLimits::default(),
        &ImportLimits::default(),
        &mut f,
        &mut c,
    )
    .unwrap();
    assert_eq!(read, bare_model);
}

/// A `tableStyles` part that is not XML costs that part and not the deck.
#[test]
fn a_tablestyles_part_that_is_not_xml_is_reported_and_the_deck_opens() {
    let mut pkg = open(read_fixture(EVERY));
    let mut edits = casual_slide_package::PartEdits::default();
    edits.replace("ppt/tableStyles.xml".to_owned(), b"<a:tblStyleLst".to_vec());
    let bytes = pkg.save_with(&edits).unwrap();
    let d = imported(&bytes);
    assert_eq!(d.table_styles, None);
    assert!(
        d.findings
            .iter()
            .any(|f| f.kind == casual_slide_import::properties::ImportFindingKind::UnreadablePart)
    );
}

/// Saving an edit to the list touches its part alone, and only the node named.
#[test]
fn editing_a_style_changes_its_part_alone() {
    let source = read_fixture(EVERY);
    let mut wanted = imported(&source).table_styles.unwrap();
    wanted.styles[1].name = Some("Renamed".to_owned());
    let mut pkg = open(source.clone());
    let outcome = save_table_styles(
        &mut pkg,
        &wanted,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(outcome.changed_parts, ["ppt/tableStyles.xml"]);
    only_this_changed(
        &source,
        &outcome.bytes,
        "ppt/tableStyles.xml",
        &[(r#"styleName="Second""#, r#"styleName="Renamed""#)],
    );
    f4_validates(&outcome.bytes);

    // Unchanged: the input bytes.
    let same = imported(&source).table_styles.unwrap();
    let mut pkg = open(source.clone());
    let outcome = save_table_styles(
        &mut pkg,
        &same,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    assert_eq!(outcome.bytes, source);
    assert!(outcome.changed_parts.is_empty());

    // No such part: refused, not invented.
    let no_part = read_fixture("generated/table-unknown-style.pptx");
    let mut pkg = open(no_part);
    let err = save_table_styles(
        &mut pkg,
        &same,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap_err();
    assert_eq!(err.code(), "CS-SLE-0001");
}

#[test]
fn a_change_of_which_choice_replaces_it() {
    let source = read_fixture(EVERY);
    let mut wanted = imported(&source).table_styles.unwrap();
    let whole = wanted.styles[0]
        .slot_mut(TablePartKind::WholeTable)
        .as_mut()
        .unwrap();
    let cell = whole.cell.as_mut().unwrap();
    // `fillRef` to `fill`, and a border's `lnRef` to `ln`.
    cell.fill = Some(casual_slide_model::table::TableStyleFill::Inline(
        Fill::None,
    ));
    cell.borders.as_mut().unwrap().right =
        Some(casual_slide_model::table::TableBorder::Line(Box::default()));
    let mut pkg = open(source);
    let outcome = save_table_styles(
        &mut pkg,
        &wanted,
        &SlideLimits::default(),
        &ImportLimits::default(),
    )
    .unwrap();
    let text = String::from_utf8(parts(&outcome.bytes)["ppt/tableStyles.xml"].clone()).unwrap();
    assert!(!text.contains("<a:fillRef idx=\"2\""), "{text}");
    assert!(!text.contains("<a:lnRef"), "{text}");
    f4_validates(&outcome.bytes);
    assert_eq!(imported(&outcome.bytes).table_styles.unwrap(), wanted);

    // And on a table: an id to a style held inline.
    let source = read_fixture(EVERY);
    let edited = edit_slide(&source, |slide| {
        let t = table_mut(slide, "Every Property");
        let style = wanted.styles[1].clone();
        t.properties.as_mut().unwrap().style = Some(TableStyleChoice::Inline(Box::new(style)));
    })
    .unwrap();
    let text = String::from_utf8(parts(&edited)["ppt/slides/slide1.xml"].clone()).unwrap();
    assert!(text.contains("<a:tableStyle styleId"));
    assert_eq!(text.matches("<a:tableStyleId>").count(), 0);
    f4_validates(&edited);
}

#[test]
fn a_merge_can_be_stated_on_cells_and_reads_back() {
    let source = read_fixture(MERGED);
    let edited = edit_slide(&source, |slide| {
        let t = table_mut(slide, "Merged");
        // Merge the two plain cells on the last row.
        t.rows[2].cells[0].grid_span = Some(2);
        t.rows[2].cells[1].h_merge = Some(true);
    })
    .unwrap();
    only_this_changed(
        &source,
        &edited,
        "ppt/slides/slide1.xml",
        &[
            (
                r#"<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>A"#,
                r#"<a:tc gridSpan="2"><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>A"#,
            ),
            (
                r#"<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>B"#,
                r#"<a:tc hMerge="1"><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>B"#,
            ),
        ],
    );
    let m = model(&edited);
    let part = m.get(m.structure.slides[0]).unwrap();
    let t = part.shape_tree().unwrap().children[0]
        .table
        .as_ref()
        .unwrap();
    assert_eq!(t.merges().faults, []);
    assert_eq!(t.merges().merges.len(), 4);
    f4_validates(&edited);
}

/// Each is refused with nothing saved, and says why.
#[test]
fn what_this_tier_does_not_write_is_refused_with_nothing_saved() {
    let source = read_fixture(EVERY);
    let code = |edit: fn(&mut Table)| {
        edit_slide(&source, |slide| edit(table_mut(slide, "Every Property"))).unwrap_err()
    };
    assert_eq!(
        code(|t| {
            t.rows.pop();
        }),
        "CS-EXP-0009"
    );
    assert_eq!(code(|t| t.rows.push(Default::default())), "CS-EXP-0009");
    assert_eq!(
        code(|t| {
            t.rows[0].cells.pop();
        }),
        "CS-EXP-0009"
    );
    assert_eq!(
        code(|t| t.rows[0].cells.push(Default::default())),
        "CS-EXP-0009"
    );
    assert_eq!(
        code(|t| {
            t.grid.as_mut().unwrap().pop();
        }),
        "CS-EXP-0009"
    );
    assert_eq!(code(|t| t.grid = None), "CS-EXP-0009");
    // A span past the grid: the reader would not believe it.
    assert_eq!(
        code(|t| t.rows[0].cells[0].grid_span = Some(9)),
        "CS-EXP-0007"
    );
    assert_eq!(
        code(|t| t.rows[0].cells[0].row_span = Some(0)),
        "CS-EXP-0007"
    );
    // A text body with no bodyPr, which the schema requires.
    assert_eq!(
        code(|t| t.rows[0].cells[0].text.as_mut().unwrap().properties = None),
        "CS-EXP-0009"
    );
    // A character XML cannot carry, and a malformed GUID.
    assert_eq!(
        code(|t| t.rows[0].cells[0].id = Some("\u{1}".to_owned())),
        "CS-EXP-0001"
    );
    assert_eq!(
        code(|t| {
            t.properties.as_mut().unwrap().style = Some(TableStyleChoice::Id("x".to_owned()));
        }),
        "CS-EXP-0007"
    );
    // A table on a frame with none, or gone from one.
    let none = edit_slide(&source, |slide| {
        named_mut(slide, "Every Property").table = None;
    });
    assert_eq!(none.unwrap_err(), "CS-EXP-0009");
}

#[test]
fn a_table_in_an_alternate_branch_is_refused_as_any_node_is() {
    let table =
        r#"<a:tbl><a:tblGrid><a:gridCol w="1"/></a:tblGrid><a:tr h="1"><a:tc/></a:tr></a:tbl>"#;
    let frame = format!(
        r#"<p:graphicFrame><p:nvGraphicFramePr><p:cNvPr id="2" name="t"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr><p:xfrm><a:off x="0" y="0"/><a:ext cx="1" cy="1"/></p:xfrm><a:graphic><a:graphicData uri="u">{table}</a:graphicData></a:graphic></p:graphicFrame>"#
    );
    let xml = format!(
        r#"<p:sld {NS} xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/><mc:AlternateContent><mc:Fallback>{frame}</mc:Fallback></mc:AlternateContent></p:spTree></p:cSld></p:sld>"#
    );
    let mut wanted = read_part(xml.as_bytes());
    table_mut(&mut wanted, "t").rows[0].cells[0].id = Some("x".to_owned());
    assert_eq!(patch_between(&xml, &wanted).unwrap_err(), "CS-SLE-0002");
}

/// Strict stays Strict: new children use the part's own prefix and the reader believes them.
#[test]
fn an_edit_in_a_strict_deck_stays_strict_and_valid() {
    let source = read_fixture("generated/table-strict.pptx");
    let edited = edit_slide(&source, |slide| {
        let cell = &mut table_mut(slide, "Every Property").rows[1].cells[1];
        set_first_run(cell, "Strict");
        cell.properties.get_or_insert_with(Default::default).fill = Some(red());
    })
    .unwrap();
    let text = String::from_utf8(parts(&edited)["ppt/slides/slide1.xml"].clone()).unwrap();
    assert!(text.contains("purl.oclc.org/ooxml/drawingml/main"));
    f4_validates(&edited);
}

/// Import, save, import: on every deck, the bytes come back and so does the model.
#[test]
fn every_deck_is_a_fixed_point_with_its_tables_read() {
    let mut count = 0;
    for path in all_decks() {
        let source = read_fixture(&path);
        let first = imported(&source);
        let all: Vec<&SlidePart> = first.parts.parts.iter().flatten().collect();
        let mut pkg = open(source.clone());
        let outcome = save_slide_parts(
            &mut pkg,
            &first.parts.structure,
            &all,
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(outcome.bytes, source, "{path}");
        let second = imported(&outcome.bytes);
        assert_eq!(second.parts, first.parts, "{path}");
        assert_eq!(second.table_styles, first.table_styles, "{path}");
        if let Some(list) = &first.table_styles {
            let mut pkg = open(source.clone());
            let out = save_table_styles(
                &mut pkg,
                list,
                &SlideLimits::default(),
                &ImportLimits::default(),
            )
            .unwrap_or_else(|e| panic!("{path}: {e}"));
            assert_eq!(out.bytes, source, "{path}");
        }
        count += 1;
    }
    assert!(count >= 40, "{count}");
}

fn first_table_cell(wanted: &SlideParts) -> Option<(u32, casual_slide_model::shape::ObjectId)> {
    wanted.structure.slides.iter().find_map(|&ordinal| {
        let part = wanted.get(ordinal)?;
        let node = part.shape_tree()?.descendants().find(|n| {
            n.table.as_ref().is_some_and(|t| {
                t.rows.first().is_some_and(|r| {
                    r.cells.first().is_some_and(|c| {
                        c.text.as_ref().is_some_and(|b| {
                            b.paragraphs.first().is_some_and(|p| {
                                p.items.iter().any(|i| matches!(i, TextItem::Run(_)))
                            })
                        })
                    })
                })
            })
        })?;
        Some((ordinal, node.id))
    })
}

/// On every deck with a table: a cell's text and fill. Only that part changes, it reopens
/// as asked, and F4 finds nothing it did not already.
#[test]
fn every_deck_with_a_table_takes_a_cell_edit_and_stays_as_valid_as_it_was() {
    let mut edited_decks = Vec::new();
    for path in all_decks() {
        let source = read_fixture(&path);
        let mut wanted = model(&source);
        let Some((ordinal, id)) = first_table_cell(&wanted) else {
            continue;
        };
        let node = wanted.get_mut(ordinal).unwrap().node_mut(id).unwrap();
        let cell = &mut node.table.as_mut().unwrap().rows[0].cells[0];
        let body = cell.text.as_mut().unwrap();
        for item in &mut body.paragraphs[0].items {
            if let TextItem::Run(run) = item {
                run.text = Some("Edited cell".to_owned());
                break;
            }
        }
        cell.properties.get_or_insert_with(Default::default).fill = Some(red());
        let mut pkg = open(source.clone());
        let outcome = match save_slide_parts(
            &mut pkg,
            &wanted.structure,
            &[wanted.get(ordinal).unwrap()],
            &SlideLimits::default(),
            &ImportLimits::default(),
        ) {
            Ok(outcome) => outcome,
            Err(e) if e.code() == "CS-SLE-0002" => continue,
            Err(e) => panic!("{path}: {e}"),
        };
        assert_eq!(model(&outcome.bytes), wanted, "{path}");
        assert_eq!(outcome.changed_parts.len(), 1, "{path}");
        let (old, new) = (parts(&source), parts(&outcome.bytes));
        for (name, bytes) in &old {
            if !outcome.changed_parts.contains(name) {
                assert_eq!(new.get(name), Some(bytes), "{path}: {name} changed");
            }
        }
        assert_eq!(f4_failures(&outcome.bytes), f4_failures(&source), "{path}");
        edited_decks.push(path);
    }
    for real in [
        "real/z19828779-education-ar.pptx",
        "generated/table-every-property.pptx",
        "generated/table-merged-cells.pptx",
    ] {
        assert!(
            edited_decks.iter().any(|p| p == real),
            "{real}: {edited_decks:?}"
        );
    }
    eprintln!("edited: {edited_decks:?}");
}

/// A real table with no run in its first cell still takes text at the paragraph.
#[test]
fn real_decks_take_a_fill_on_every_cell_and_stay_valid() {
    for path in [
        "real/z17310490-vierfelder-de.pptx",
        "real/z19828779-education-ar.pptx",
    ] {
        let source = read_fixture(path);
        let mut wanted = model(&source);
        let mut touched = Vec::new();
        let slides = wanted.structure.slides.clone();
        for ordinal in slides {
            let Some(part) = wanted.get_mut(ordinal) else {
                continue;
            };
            let ids: Vec<_> = part
                .shape_tree()
                .unwrap()
                .descendants()
                .filter(|n| n.table.is_some())
                .map(|n| n.id)
                .collect();
            for id in &ids {
                let t = part.node_mut(*id).unwrap().table.as_mut().unwrap();
                for row in &mut t.rows {
                    for cell in &mut row.cells {
                        cell.properties.get_or_insert_with(Default::default).fill = Some(red());
                    }
                }
            }
            if !ids.is_empty() {
                touched.push(ordinal);
            }
        }
        assert!(!touched.is_empty(), "{path}");
        let list: Vec<&SlidePart> = touched.iter().map(|o| wanted.get(*o).unwrap()).collect();
        let mut pkg = open(source.clone());
        let outcome = save_slide_parts(
            &mut pkg,
            &wanted.structure,
            &list,
            &SlideLimits::default(),
            &ImportLimits::default(),
        )
        .unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(outcome.changed_parts.len(), touched.len(), "{path}");
        assert_eq!(model(&outcome.bytes), wanted, "{path}");
        assert_eq!(f4_failures(&outcome.bytes), f4_failures(&source), "{path}");
    }
}
