//! Tier 1M export: every comment form, section list, zoom, mark, tag and property edited; only
//! those nodes change, and what is retained whole stays byte for byte. Every deck is a fixed
//! point, and forcing a part through the writer from nothing gives back what it said. Edited
//! decks validate through F4. The writer's slide operations keep sections and zooms in step.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use casual_slide_export::comment::{
    patch_authors, patch_comment_authors, patch_comments, patch_threaded_comments,
};
use casual_slide_export::presentation::{patch_presentation, save_deck};
use casual_slide_export::custom_data::{
    patch_datastore_item, patch_schema_library, patch_tags, save_review,
};
use casual_slide_export::section::patch_part_marks;
use casual_slide_export::writer::{Op, SavePlan, SlideRef};
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::comment::{
    ReviewLimits, import_authors, import_comment_authors, import_comments,
    import_threaded_comments,
};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::custom_data::{
    ImportedReview, import_datastore_item, import_review, import_schema_library, import_tags,
};
use casual_slide_import::section::import_part_marks;
use casual_slide_model::comment::{
    Comment, CommentAnchor, CommentReply, CommentStatus, Moniker, ParentComment, ThreadedComment,
};
use casual_slide_model::presentation::Deck;
use casual_slide_model::custom_data::StringTag;
use casual_slide_model::section::{
    ClassificationOutcome, DesignerMark, Section, Zoom, ZoomImageType,
};
use casual_slide_model::text::TextItem;
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree};

const LEGACY: &str = "generated/comments-legacy.pptx";
const MODERN: &str = "generated/comments-modern.pptx";
const SECTIONS: &str = "generated/sections-zoom-labels.pptx";
const CUSTOM: &str = "generated/custom-tags-labels.pptx";
const REVIEW: &str = "generated/review-every-change.pptx";
const RETAINED: &str = "generated/extensions-retained.pptx";
const MODERN_PART: &str = "ppt/comments/modernComment_100_0.xml";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn read_all(bytes: &[u8]) -> (Deck, ImportedReview) {
    let mut pkg = open(bytes.to_vec());
    let import = ImportLimits::default();
    let deck = import_deck(&mut pkg, &import).unwrap().deck;
    let review = import_review(&mut pkg, &deck, &ReviewLimits::default(), &import);
    (deck, review)
}

fn save(source: &[u8], after: &ImportedReview) -> Result<casual_slide_export::presentation::SaveOutcome, String> {
    let mut pkg = open(source.to_vec());
    save_review(&mut pkg, after, &ReviewLimits::default(), &ImportLimits::default()).map_err(|e| e.code().to_owned())
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

/// Which parts differ.
fn changed(a: &[u8], b: &[u8]) -> Vec<String> {
    let (a, b) = (parts(a), parts(b));
    let mut out: Vec<String> = a.keys().chain(b.keys()).filter(|n| a.get(*n) != b.get(*n)).cloned().collect();
    out.sort();
    out.dedup();
    out
}

fn text_of(bytes: &[u8], part: &str) -> String {
    String::from_utf8(parts(bytes)[part].clone()).unwrap()
}

/// Lengths around the one place two texts differ, so a test can say how small an edit was.
fn differing_span<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    let start = a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count();
    let tail = a[start..].bytes().rev().zip(b[start..].bytes().rev()).take_while(|(x, y)| x == y).count();
    (&a[start..a.len() - tail], &b[start..b.len() - tail])
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

// ------------------------------------------------------------- unedited, fixed point

#[test]
fn every_deck_saves_unedited_as_its_own_bytes_and_is_a_fixed_point() {
    let mut count = 0;
    let mut held = (0, 0, 0);
    for path in all_decks() {
        let source = read_fixture(&path);
        let (deck, first) = read_all(&source);
        let out = save(&source, &first).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(out.bytes, source, "{path}");
        assert!(out.changed_parts.is_empty(), "{path}");
        let (deck2, second) = read_all(&out.bytes);
        assert_eq!(deck2, deck, "{path}");
        assert_eq!(second.comments, first.comments, "{path}");
        assert_eq!(second.custom, first.custom, "{path}");
        assert_eq!(second.marks, first.marks, "{path}");
        assert_eq!(second.labels, first.labels, "{path}");
        let mut pkg = open(source.clone());
        let saved = save_deck(&mut pkg, &deck, &ImportLimits::default()).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(saved.bytes, source, "{path}: a deck saved with its sections");
        held.0 += usize::from(first.comments.comment_authors.is_some() || first.comments.authors.is_some());
        held.1 += usize::from(deck.presentation.sections.is_some());
        held.2 += usize::from(!first.retained.is_empty());
        count += 1;
    }
    assert!(count >= 40, "{count}");
    assert!(held.0 >= 3 && held.1 >= 2 && held.2 >= 3, "{held:?}");
}

/// A save that goes through every part of this tier: each field of a copy stated again from
/// nothing, by a part with nothing in it, reads back as the same. This is the forced rewrite:
/// an unedited save proves nothing about the writer (`WR-01`).
#[test]
fn every_part_of_this_tier_is_written_from_nothing_and_reads_back_the_same() {
    let import = ImportLimits::default();
    let limits = ReviewLimits::default();
    let xml = OoxmlLimits::default();
    let quiet = || Findings::new(1000);
    let p = r#"xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main""#;
    let p188 = r#"xmlns:p188="http://schemas.microsoft.com/office/powerpoint/2018/8/main""#;

    let source = read_fixture(LEGACY);
    let (_, r) = read_all(&source);
    let mut tree = XmlTree::parse(format!("<p:cmAuthorLst {p}/>").as_bytes(), "a", &xml).unwrap();
    let wanted = r.comments.comment_authors.as_ref().unwrap();
    assert!(patch_comment_authors(&mut tree, "a", false, wanted, &limits, &import, &xml).unwrap() > 0);
    let again = XmlTree::parse(&tree.to_bytes(), "a", &xml).unwrap();
    assert_eq!(&import_comment_authors(&again, "a", &limits, &import, &mut quiet()).unwrap(), wanted);
    for slide in &r.comments.legacy {
        let mut tree = XmlTree::parse(format!("<p:cmLst {p}/>").as_bytes(), "c", &xml).unwrap();
        patch_comments(&mut tree, "c", false, &slide.list, &limits, &import, &xml).unwrap();
        let again = XmlTree::parse(&tree.to_bytes(), "c", &xml).unwrap();
        assert_eq!(import_comments(&again, "c", &limits, &import, &mut quiet()).unwrap(), slide.list);
    }

    let source = read_fixture(MODERN);
    let (_, r) = read_all(&source);
    let mut tree = XmlTree::parse(format!("<p188:authorLst {p188}/>").as_bytes(), "a", &xml).unwrap();
    let wanted = r.comments.authors.as_ref().unwrap();
    patch_authors(&mut tree, "a", false, wanted, &limits, &import, &xml).unwrap();
    let again = XmlTree::parse(&tree.to_bytes(), "a", &xml).unwrap();
    assert_eq!(&import_authors(&again, "a", &limits, &import, &mut quiet()).unwrap(), wanted);

    // A new comment, and everything it can hold: a list with one that has no extension, as an
    // entry's uri is what cannot be made for a reaction or a task (see the module docs).
    let comments = &r.comments.threaded[0].list;
    let mut wanted = comments.clone();
    for c in &mut wanted.comments {
        c.reactions = None;
        c.task = None;
        for reply in c.reply_lst.iter_mut().flatten() {
            reply.reactions = None;
        }
    }
    let mut tree = XmlTree::parse(
        format!(r#"<p188:cmLst {p188} xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"/>"#).as_bytes(),
        "c",
        &xml,
    )
    .unwrap();
    patch_threaded_comments(&mut tree, "c", false, &[], &wanted, &limits, &import, &xml).unwrap();
    let again = XmlTree::parse(&tree.to_bytes(), "c", &xml).unwrap();
    let read = import_threaded_comments(&again, "c", &[], &limits, &import, &mut quiet(), &mut ColorFindings::default()).unwrap();
    assert_eq!(read, wanted);

    let source = read_fixture(CUSTOM);
    let (_, r) = read_all(&source);
    let mut tree = XmlTree::parse(format!("<p:tagLst {p}/>").as_bytes(), "t", &xml).unwrap();
    let wanted = &r.custom.tags[0].list;
    patch_tags(&mut tree, "t", false, wanted, &limits, &import, &xml).unwrap();
    let again = XmlTree::parse(&tree.to_bytes(), "t", &xml).unwrap();
    assert_eq!(&import_tags(&again, "t", &limits, &import, &mut quiet()).unwrap(), wanted);
    let ds = r#"xmlns:ds="http://schemas.openxmlformats.org/officeDocument/2006/customXml""#;
    let mut tree = XmlTree::parse(format!("<ds:datastoreItem {ds}/>").as_bytes(), "d", &xml).unwrap();
    let wanted = r.custom.custom_xml[0].properties.as_ref().unwrap();
    patch_datastore_item(&mut tree, "d", false, wanted, &limits, &import, &xml).unwrap();
    let again = XmlTree::parse(&tree.to_bytes(), "d", &xml).unwrap();
    assert_eq!(&import_datastore_item(&again, "d", &limits, &import, &mut quiet()).unwrap(), wanted);
    let sl = r#"xmlns:sl="http://schemas.openxmlformats.org/schemaLibrary/2006/main""#;
    let mut tree = XmlTree::parse(format!("<sl:schemaLibrary {sl}/>").as_bytes(), "l", &xml).unwrap();
    let wanted = r.custom.custom_xml[1].schema_library.as_ref().unwrap();
    patch_schema_library(&mut tree, "l", false, wanted, &limits, &import, &xml).unwrap();
    let again = XmlTree::parse(&tree.to_bytes(), "l", &xml).unwrap();
    assert_eq!(&import_schema_library(&again, "l", &limits, &import, &mut quiet()).unwrap(), wanted);
}

/// Each modelled value stated again through the writer on real and generated decks, and undone:
/// the tree comes back to the same reading, which an unedited save could never show.
#[test]
fn a_change_undone_through_the_writer_reads_the_same_on_every_deck_holding_this_tier() {
    let mut touched = 0;
    for path in all_decks() {
        let source = read_fixture(&path);
        let (_, first) = read_all(&source);
        let mut edited = first.clone();
        let mut any = false;
        if let Some(list) = &mut edited.comments.comment_authors {
            for a in &mut list.authors {
                a.name = Some("Changed".to_owned());
                any = true;
            }
        }
        if let Some(list) = &mut edited.comments.authors {
            for a in &mut list.authors {
                a.provider_id = Some("Changed".to_owned());
                any = true;
            }
        }
        if !any {
            continue;
        }
        touched += 1;
        let out = save(&source, &edited).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert!(out.changed_parts.iter().all(|p| p.contains("uthors")), "{path}");
        let (_, second) = read_all(&out.bytes);
        assert_eq!(second.comments.comment_authors, edited.comments.comment_authors, "{path}");
        let undone = save(&out.bytes, &first).unwrap_or_else(|e| panic!("{path}: {e}"));
        let (_, third) = read_all(&undone.bytes);
        assert_eq!(third.comments, first.comments, "{path}");
    }
    assert!(touched >= 2, "{touched}");
}

// ------------------------------------------------------------- named nodes

#[test]
fn editing_a_legacy_comments_text_changes_only_that_text() {
    let source = read_fixture(LEGACY);
    let (_, mut r) = read_all(&source);
    r.comments.legacy[0].list.comments[0].text = Some("Edited".to_owned());
    let out = save(&source, &r).unwrap();
    assert_eq!(out.changed_parts, vec!["ppt/comments/comment1.xml".to_owned()]);
    assert_eq!(changed(&source, &out.bytes), out.changed_parts);
    let (before, after) = (text_of(&source, "ppt/comments/comment1.xml"), text_of(&out.bytes, "ppt/comments/comment1.xml"));
    assert_eq!(differing_span(&before, &after), ("First note", "Edited"));
}

#[test]
fn a_legacy_comment_taking_a_parent_keeps_the_rest_of_its_extension() {
    let source = read_fixture(LEGACY);
    let (_, mut r) = read_all(&source);
    let first = &mut r.comments.legacy[0].list.comments[0];
    first.threading.as_mut().unwrap().parent = Some(ParentComment {
        author_id: Some(2),
        idx: Some(1),
    });
    let out = save(&source, &r).unwrap();
    let (_, again) = read_all(&out.bytes);
    assert_eq!(again.comments.legacy[0], r.comments.legacy[0]);
    let after = text_of(&out.bytes, "ppt/comments/comment1.xml");
    assert_eq!(after.matches("{C676402C-5697-4E1C-873F-D02D1690AC5C}").count(), 2);
}

#[test]
fn a_legacy_comment_can_be_added_and_taken_away() {
    let source = read_fixture(LEGACY);
    let (_, mut r) = read_all(&source);
    r.comments.legacy[1].list.comments.push(Comment {
        author_id: Some(2),
        idx: Some(9),
        text: Some("New".to_owned()),
        pos: Some(casual_slide_model::presentation::Point {
            x: casual_slide_model::presentation::Coordinate(1),
            y: casual_slide_model::presentation::Coordinate(2),
        }),
        ..Comment::default()
    });
    r.comments.legacy[0].list.comments.remove(0);
    let out = save(&source, &r).unwrap();
    let (_, again) = read_all(&out.bytes);
    assert_eq!(again.comments.legacy, r.comments.legacy);
}

#[test]
fn editing_a_modern_comment_changes_only_what_was_named() {
    let source = read_fixture(MODERN);
    let (_, mut r) = read_all(&source);
    let list = &mut r.comments.threaded[0].list.comments;
    list[1].status = Some(CommentStatus::Closed);
    let out = save(&source, &r).unwrap();
    assert_eq!(out.changed_parts, vec![MODERN_PART.to_owned()]);
    let (before, after) = (text_of(&source, MODERN_PART), text_of(&out.bytes, MODERN_PART));
    assert_eq!(differing_span(&before, &after), ("resolv", "clos"));
    assert_eq!(after.matches("<ac:deMkLst>").count(), 1, "an anchor stays");
    assert_eq!(after.matches("p223:reactions").count(), before.matches("p223:reactions").count());
}

#[test]
fn a_replys_text_a_reaction_and_a_task_are_edited_where_the_file_has_them() {
    let source = read_fixture(MODERN);
    let (_, mut r) = read_all(&source);
    let list = &mut r.comments.threaded[0].list.comments;
    let reply = &mut list[0].reply_lst.as_mut().unwrap()[0];
    let body = reply.tx_body.as_mut().unwrap();
    let TextItem::Run(run) = &mut body.paragraphs[0].items[0] else { panic!() };
    run.text = Some("Edited reply".to_owned());
    reply.reactions.as_mut().unwrap().rxn[0].reaction_type = Some("+1".to_owned());
    list[2].task.as_mut().unwrap().history.as_mut().unwrap().event[2].action =
        Some(casual_slide_model::comment::TaskAction::Title(Some("New".to_owned())));
    list[2].title = Some("New".to_owned());
    list[2].complete = None;
    let out = save(&source, &r).unwrap();
    assert_eq!(out.changed_parts, vec![MODERN_PART.to_owned()]);
    let (_, again) = read_all(&out.bytes);
    assert_eq!(again.comments.threaded, r.comments.threaded);
}

#[test]
fn a_new_modern_comment_goes_in_with_its_anchor_and_a_reply_goes_with_it() {
    let source = read_fixture(MODERN);
    let (_, mut r) = read_all(&source);
    let new_id = "{ABABABAB-0000-4000-8000-000000000001}".to_owned();
    let anchors = [
        CommentAnchor::Slide(vec![Moniker { kind: "docMk".to_owned(), attributes: vec![] }, Moniker {
            kind: "sldMk".to_owned(),
            attributes: vec![("cId".to_owned(), "1".to_owned()), ("sldId".to_owned(), "256".to_owned())],
        }]),
        CommentAnchor::Drawing(vec![Moniker { kind: "spMk".to_owned(), attributes: vec![("id".to_owned(), "2".to_owned())] }]),
        CommentAnchor::Text(vec![Moniker { kind: "txMk".to_owned(), attributes: vec![("cp".to_owned(), "0".to_owned())] }]),
        CommentAnchor::Unknown,
    ];
    let body = r.comments.threaded[0].list.comments[1].tx_body.clone();
    let author = r.comments.authors.as_ref().unwrap().authors[0].id.clone();
    let date = r.comments.threaded[0].list.comments[1].created.clone();
    for (i, anchor) in anchors.into_iter().enumerate() {
        let mut id = new_id.clone();
        id.replace_range(1..2, &i.to_string());
        r.comments.threaded[0].list.comments.push(ThreadedComment {
            id: Some(id.clone()),
            author_id: author.clone(),
            created: date.clone(),
            anchor: Some(anchor),
            tx_body: body.clone(),
            reply_lst: Some(vec![CommentReply {
                id: Some(id.replace("ABAB", "CDCD")),
                author_id: author.clone(),
                created: date.clone(),
                tx_body: body.clone(),
                ..CommentReply::default()
            }]),
            ..ThreadedComment::default()
        });
    }
    let out = save(&source, &r).unwrap();
    let (_, again) = read_all(&out.bytes);
    assert_eq!(again.comments.threaded, r.comments.threaded);
    let text = text_of(&out.bytes, MODERN_PART);
    assert!(text.contains(r#"<pc:sldMk cId="1" sldId="256"/>"#), "the namespace it belongs in");
    assert!(text.contains(r#"<ac:spMk id="2"/>"#));
    // Taking one away leaves the others' elements exactly as they were.
    let removed_id = r.comments.threaded[0].list.comments[1].id.clone();
    r.comments.threaded[0].list.comments.remove(1);
    let gone = save(&out.bytes, &r).unwrap();
    let after = text_of(&gone.bytes, MODERN_PART);
    assert!(!after.contains(removed_id.as_deref().unwrap()));
    assert_eq!(after.matches("<ac:txMkLst>").count(), 2);
}

#[test]
fn a_reaction_or_task_with_no_known_uri_is_refused_and_a_changed_anchor_too() {
    let source = read_fixture(MODERN);
    let (_, base) = read_all(&source);
    let mut r = base.clone();
    r.comments.threaded[0].list.comments[1].reactions = base.comments.threaded[0].list.comments[0].reactions.clone();
    assert_eq!(save(&source, &r).unwrap_err(), "CS-EXP-0009");
    let mut r = base.clone();
    r.comments.threaded[0].list.comments[1].task = base.comments.threaded[0].list.comments[2].task.clone();
    assert_eq!(save(&source, &r).unwrap_err(), "CS-EXP-0009");
    let mut r = base.clone();
    r.comments.threaded[0].list.comments[1].anchor = Some(CommentAnchor::Unknown);
    assert_eq!(save(&source, &r).unwrap_err(), "CS-EXP-0009");
    let mut r = base.clone();
    r.comments.threaded[0].list.comments.reverse();
    assert_eq!(save(&source, &r).unwrap_err(), "CS-EXP-0009");
}

#[test]
fn a_value_the_reader_would_not_believe_is_refused_with_nothing_saved() {
    let source = read_fixture(MODERN);
    let (_, mut r) = read_all(&source);
    r.comments.threaded[0].list.comments[1].id = Some("not a guid".to_owned());
    assert_eq!(save(&source, &r).unwrap_err(), "CS-EXP-0007");
    let (_, mut r) = read_all(&read_fixture(LEGACY));
    r.comments.legacy[0].list.comments[0].text = Some("bell\u{7}".to_owned());
    assert_eq!(save(&read_fixture(LEGACY), &r).unwrap_err(), "CS-EXP-0001");
}

#[test]
fn a_new_part_for_a_deck_with_none_is_refused() {
    let source = read_fixture("generated/minimal.pptx");
    let (_, mut r) = read_all(&source);
    r.comments.authors = Some(casual_slide_model::comment::AuthorList::default());
    assert_eq!(save(&source, &r).unwrap_err(), "CS-EXP-0009");
}

// ------------------------------------------------------------- sections

fn with_sections(source: &[u8], edit: impl FnOnce(&mut casual_slide_model::section::SectionList)) -> (Vec<u8>, Vec<String>) {
    let mut pkg = open(source.to_vec());
    let import = ImportLimits::default();
    let mut deck = import_deck(&mut pkg, &import).unwrap().deck;
    edit(deck.presentation.sections.as_mut().unwrap());
    let out = save_deck(&mut pkg, &deck, &import).unwrap();
    (out.bytes, out.changed_parts)
}

#[test]
fn a_slide_moved_between_sections_changes_only_those_ids() {
    let source = read_fixture(SECTIONS);
    let (bytes, _) = with_sections(&source, |list| {
        let moved = list.sections[0].sld_id_lst.as_mut().unwrap().pop().unwrap();
        list.sections[1].sld_id_lst.as_mut().unwrap().insert(0, moved);
    });
    assert_eq!(changed(&source, &bytes), vec!["ppt/presentation.xml".to_owned()]);
    let before = text_of(&source, "ppt/presentation.xml");
    let after = text_of(&bytes, "ppt/presentation.xml");
    assert!(after.contains("p15:sldGuideLst"), "another ext stays");
    assert!(after.contains("p16:designElem"), "a section's own extLst stays");
    assert_eq!(before.len(), after.len(), "two ids change places: {after}");
    let mut pkg = open(bytes);
    let deck = import_deck(&mut pkg, &ImportLimits::default()).unwrap().deck;
    let list = deck.presentation.sections.unwrap();
    assert_eq!(list.sections[0].sld_id_lst.as_deref(), Some(&[256][..]));
    assert_eq!(list.sections[1].sld_id_lst.as_deref(), Some(&[257, 258][..]));
    assert!(list.check_against(&[256, 257, 258]).in_sync());
}

#[test]
fn a_sections_name_edit_changes_only_its_name_and_preflight_holds_what_was_not_asked() {
    let source = read_fixture(SECTIONS);
    let (bytes, _) = with_sections(&source, |list| {
        list.sections[1].name = Some("Later".to_owned());
    });
    let (a, b) = (text_of(&source, "ppt/presentation.xml"), text_of(&bytes, "ppt/presentation.xml"));
    assert_eq!(differing_span(&a, &b), ("Rest", "Later"));
    // Through a whole save, the writer's rule stands: a section goes by its operation.
    let mut pkg = open(source);
    let import = ImportLimits::default();
    let mut deck = import_deck(&mut pkg, &import).unwrap().deck;
    deck.presentation.sections.as_mut().unwrap().sections.remove(2);
    assert_eq!(save_deck(&mut pkg, &deck, &import).unwrap_err().code(), "CS-SAV-0012", "INV-SL-03");
}

/// The patch itself, as the writer's own section operations leave preflight satisfied.
fn patched_presentation(source: &[u8], edit: impl FnOnce(&mut Deck)) -> (String, Deck) {
    let mut pkg = open(source.to_vec());
    let import = ImportLimits::default();
    let mut deck = import_deck(&mut pkg, &import).unwrap().deck;
    edit(&mut deck);
    let main = pkg.main_part().to_owned();
    let rels = pkg.relationships_of(&main).to_vec();
    let bytes = pkg.read_part(&main).unwrap();
    let mut tree = XmlTree::parse(&bytes, &main, &OoxmlLimits::default()).unwrap();
    patch_presentation(&mut tree, &main, false, &rels, &deck.presentation, &import).unwrap();
    (String::from_utf8(tree.to_bytes()).unwrap(), deck)
}

#[test]
fn a_new_section_goes_in_and_taking_one_away_leaves_the_others() {
    let source = read_fixture(SECTIONS);
    let (text, _) = patched_presentation(&source, |deck| {
        let list = deck.presentation.sections.as_mut().unwrap();
        list.sections.push(Section {
            name: Some("New".to_owned()),
            id: Some("{AAAAAAA9-0000-4000-8000-000000000009}".to_owned()),
            sld_id_lst: Some(Vec::new()),
        });
        list.sections.remove(0);
    });
    assert!(text.contains(r#"name="New""#));
    assert!(!text.contains(r#"name="Opening""#));
    assert!(text.contains("p16:designElem"), "the section it was on stays, with its extLst");
}

#[test]
fn a_first_section_list_goes_in_at_its_uri_and_the_last_takes_its_ext_with_it() {
    let source = read_fixture("generated/three-slides.pptx");
    let (text, deck) = patched_presentation(&source, |deck| {
        let ids: Vec<u32> = deck.presentation.slides.iter().flatten().map(|s| s.id).collect();
        deck.presentation.sections = Some(casual_slide_model::section::SectionList {
            sections: vec![Section {
                name: Some("All".to_owned()),
                id: Some("{AAAAAAA1-0000-4000-8000-000000000001}".to_owned()),
                sld_id_lst: Some(ids),
            }],
        });
    });
    assert!(text.contains("{521415D9-36F7-43E2-AB2F-B90AF26B5E84}"), "INV-SL-01");
    let import = ImportLimits::default();
    let tree = XmlTree::parse(text.as_bytes(), "p", &OoxmlLimits::default()).unwrap();
    let rels = open(source.clone()).relationships_of("ppt/presentation.xml").to_vec();
    let mut f = Findings::new(100);
    let read = casual_slide_import::presentation::import_presentation(&tree, "p", &rels, &import, &mut f).unwrap();
    assert_eq!(read.sections, deck.presentation.sections);
    let mut tree = tree;
    let mut none = read.clone();
    none.sections = None;
    patch_presentation(&mut tree, "p", false, &rels, &none, &import).unwrap();
    let after = String::from_utf8(tree.to_bytes()).unwrap();
    assert!(!after.contains("extLst"), "INV-SL-02");
    // A list with nothing in it is what the schema does not allow: say none by removing it.
    let mut empty = read.clone();
    empty.sections = Some(casual_slide_model::section::SectionList::default());
    assert_eq!(patch_presentation(&mut tree, "p", false, &rels, &empty, &import).unwrap_err().code(), "CS-EXP-0009");
}

#[test]
fn the_writers_slide_operations_keep_sections_and_zooms_in_step() {
    let source = read_fixture(SECTIONS);
    // A slide a zoom names: the writer stops, rather than leave the zoom naming nothing.
    let mut pkg = open(source.clone());
    let blocked = SavePlan::new().op(Op::DeleteSlide { slide: SlideRef::Index(1) }).save(&mut pkg);
    assert!(blocked.is_err(), "a slide zoom names it");
    // A slide nothing names goes from its section.
    let mut pkg = open(source.clone());
    let out = SavePlan::new().op(Op::DeleteSlide { slide: SlideRef::Index(2) }).save(&mut pkg).unwrap();
    let mut pkg = open(out.bytes.clone());
    let deck = import_deck(&mut pkg, &ImportLimits::default()).unwrap().deck;
    let slides: Vec<u32> = deck.presentation.slides.iter().flatten().map(|s| s.id).collect();
    assert_eq!(slides, vec![256, 257]);
    let list = deck.presentation.sections.as_ref().unwrap();
    assert!(list.check_against(&slides).in_sync(), "{list:?}");
    assert_eq!(list.sections[1].sld_id_lst.as_deref(), Some(&[][..]), "INV-SL-03: emptied, kept");
    let review = import_review(&mut pkg, &deck, &ReviewLimits::default(), &ImportLimits::default());
    let marks = &review.marks.iter().find(|(p, _)| p.ends_with("slide1.xml")).unwrap().1;
    assert_eq!(marks.zooms.len(), 4, "zooms and fallbacks stay");
    // Moved: the list reads in `sldIdLst`'s order.
    let mut pkg = open(source);
    let out = SavePlan::new().op(Op::MoveSlide { slide: SlideRef::Index(2), position: 0 }).save(&mut pkg).unwrap();
    let mut pkg = open(out.bytes);
    let deck = import_deck(&mut pkg, &ImportLimits::default()).unwrap().deck;
    let slides: Vec<u32> = deck.presentation.slides.iter().flatten().map(|s| s.id).collect();
    assert!(deck.presentation.sections.as_ref().unwrap().check_against(&slides).in_sync());
}

#[test]
fn deleting_a_slide_takes_its_comment_parts_with_it() {
    let source = read_fixture(LEGACY);
    let mut pkg = open(source);
    let plan = SavePlan::new().op(Op::DeleteSlide { slide: SlideRef::Index(0) });
    let out = plan.save(&mut pkg).unwrap();
    let (_, r) = read_all(&out.bytes);
    assert_eq!(r.comments.legacy.len(), 1);
    assert_eq!(r.comments.legacy[0].slide_part, "ppt/slides/slide2.xml");
    assert!(r.comments.comment_authors.is_some(), "authors stay");
    assert!(!parts(&out.bytes).contains_key("ppt/comments/comment1.xml"));
}

// ------------------------------------------------------------- zooms and marks

#[test]
fn a_zoom_and_its_fallback_stand_apart_under_an_edit() {
    let source = read_fixture(SECTIONS);
    let (_, mut r) = read_all(&source);
    let at = r.marks.iter().position(|(p, _)| p.ends_with("slide1.xml")).unwrap();
    let marks = &mut r.marks[at].1;
    let Zoom::Slide(Some(slide)) = &mut marks.zooms[0].zoom else { panic!() };
    slide.sld_id = Some(258);
    let props = slide.zm_pr.as_mut().unwrap();
    props.image_type = Some(ZoomImageType::Cover);
    props.show_bg = Some(true);
    props.sp_pr.as_mut().unwrap().line = None;
    let Zoom::Summary { objects, layout } = &mut marks.zooms[3].zoom else { panic!() };
    objects.clear();
    *layout = Some(casual_slide_model::section::SummaryLayout::Grid);
    let out = save(&source, &r).unwrap();
    assert_eq!(out.changed_parts, vec!["ppt/slides/slide1.xml".to_owned()]);
    let (_, again) = read_all(&out.bytes);
    assert_eq!(again.marks[at], r.marks[at]);
    let before = text_of(&source, "ppt/slides/slide1.xml");
    let after = text_of(&out.bytes, "ppt/slides/slide1.xml");
    let fallback = |s: &str| {
        s.match_indices("<mc:Fallback>")
            .map(|(i, _)| s[i..].split_once("</mc:Fallback>").unwrap().0.to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(fallback(&before), fallback(&after), "BEH-25 keeps the branch a zoom edit leaves stale, and reports nothing lost");
    assert!(after.contains("<a:xfrm><a:off"), "xfrm in a zoom's spPr stays");
}

#[test]
fn classification_and_designer_marks_edit_in_place_and_structure_is_refused() {
    let source = read_fixture(SECTIONS);
    let (_, base) = read_all(&source);
    let at = base.marks.iter().position(|(p, _)| p.ends_with("slide1.xml")).unwrap();
    let mut r = base.clone();
    let marks = &mut r.marks[at].1;
    for m in &mut marks.classifications {
        m.outcome = Some(ClassificationOutcome::Watermark);
    }
    for m in &mut marks.designer {
        match m {
            DesignerMark::DesignElement { val, .. } => *val = Some(false),
            DesignerMark::DrawingProperties { edt_design_elem, design_tag_lst, .. } => {
                *edt_design_elem = Some(false);
                design_tag_lst.as_mut().unwrap()[0].val = Some("c".to_owned());
            }
        }
    }
    let out = save(&source, &r).unwrap();
    assert_eq!(out.changed_parts, vec!["ppt/slides/slide1.xml".to_owned()]);
    let (_, again) = read_all(&out.bytes);
    assert_eq!(again.marks[at], r.marks[at]);
    let text = text_of(&out.bytes, "ppt/slides/slide1.xml");
    assert!(text.contains(r#"<p184:classification xmlns:p184="http://schemas.microsoft.com/office/powerpoint/2018/4/main" val="watermark"/>"#));
    assert!(text.contains(r#"classificationOutcomeType="watermark""#));
    let refuse = |edit: &dyn Fn(&mut ImportedReview)| {
        let mut r = base.clone();
        edit(&mut r);
        assert_eq!(save(&source, &r).unwrap_err(), "CS-EXP-0009");
    };
    refuse(&|r| {
        r.marks[at].1.zooms.remove(0);
    });
    refuse(&|r| {
        r.marks[at].1.classifications.remove(0);
    });
    refuse(&|r| {
        r.marks[at].1.designer.remove(0);
    });
    refuse(&|r| {
        r.marks[at].1.customer_data.clear();
    });
    refuse(&|r| {
        r.marks[at].1.zooms[0].fallback = casual_slide_model::section::ZoomFallback::Absent;
    });
    refuse(&|r| {
        let zoom = r.marks[at].1.zooms[1].zoom.clone();
        r.marks[at].1.zooms[0].zoom = zoom;
    });
    // What patch_part_marks is given directly behaves as what save_review gives it.
    let tree_part = "ppt/slides/slide1.xml";
    let bytes = parts(&source)[tree_part].clone();
    let xml = OoxmlLimits::default();
    let mut tree = XmlTree::parse(&bytes, tree_part, &xml).unwrap();
    let rels = open(source.clone()).relationships_of(tree_part).to_vec();
    let same = patch_part_marks(&mut tree, tree_part, false, &rels, &base.marks[at].1, &ReviewLimits::default(), &ImportLimits::default(), &xml).unwrap();
    assert_eq!(same, 0);
    let read = import_part_marks(&tree, tree_part, &rels, &ReviewLimits::default(), &ImportLimits::default(), &mut Findings::new(10)).unwrap();
    assert_eq!(read, base.marks[at].1);
}

// ------------------------------------------------------------- tags, custom XML, retained

#[test]
fn a_tag_edit_changes_its_part_and_item_bodies_stay() {
    let source = read_fixture(CUSTOM);
    let (_, mut r) = read_all(&source);
    r.custom.tags[0].list.tags[0].val = Some("Beta".to_owned());
    r.custom.tags[0].list.tags.push(StringTag {
        name: Some("NEW".to_owned()),
        val: Some("x".to_owned()),
    });
    r.custom.custom_xml[0].properties.as_mut().unwrap().schema_refs.as_mut().unwrap()[1].uri = Some("urn:new".to_owned());
    r.custom.custom_xml[0].properties.as_mut().unwrap().item_id = Some("{CCCCCCCC-0000-4000-8000-000000000009}".to_owned());
    r.custom.custom_xml[1].schema_library.as_mut().unwrap().schemas[0].schema_location = Some("o.xsd".to_owned());
    let out = save(&source, &r).unwrap();
    assert_eq!(
        changed(&source, &out.bytes),
        vec!["customXml/item2.xml".to_owned(), "customXml/itemProps1.xml".to_owned(), "ppt/tags/tag1.xml".to_owned()]
    );
    let (_, again) = read_all(&out.bytes);
    assert_eq!(again.custom.tags, r.custom.tags);
    assert_eq!(again.custom.custom_xml, r.custom.custom_xml);
    let before = parts(&source);
    let after = parts(&out.bytes);
    assert_eq!(before["customXml/item1.xml"], after["customXml/item1.xml"], "an item's body");
    assert_eq!(before["docMetadata/LabelInfo.xml"], after["docMetadata/LabelInfo.xml"], "labels are read and kept");
    let text = text_of(&out.bytes, "customXml/itemProps1.xml");
    assert!(text.contains(r#"ds:itemID="{CCCCCCCC-0000-4000-8000-000000000009}""#), "its own prefix");
}

/// The retained-part claim: bytes compared, under an edit of something else in the deck.
#[test]
fn retained_parts_are_byte_identical_through_an_edit() {
    for (deck, retained) in [
        (REVIEW, vec!["ppt/changesInfos/changesInfo1.xml", "ppt/revisionInfo.xml"]),
        (RETAINED, vec!["ppt/slides/slide1.xml"]),
        ("real/z14230715-ddi-ko.pptx", vec!["ppt/changesInfos/changesInfo1.xml", "ppt/revisionInfo.xml"]),
        ("real/sdk-animation.pptx", vec!["customXml/item1.xml", "customXml/itemProps1.xml"]),
    ] {
        let source = read_fixture(deck);
        let mut pkg = open(source.clone());
        let import = ImportLimits::default();
        let mut d = import_deck(&mut pkg, &import).unwrap().deck;
        d.presentation.first_slide_num = Some(7);
        let out = save_deck(&mut pkg, &d, &import).unwrap();
        let (before, after) = (parts(&source), parts(&out.bytes));
        for part in retained {
            assert_eq!(before[part], after[part], "{deck}: {part}");
        }
        assert_eq!(out.changed_parts, vec!["ppt/presentation.xml".to_owned()], "{deck}");
    }
}

/// A damaged part is set aside, stays as it was through any edit, and the deck saves.
#[test]
fn a_set_aside_part_stays_byte_for_byte_through_an_edit() {
    let source = read_fixture(LEGACY);
    let mut pkg = open(source.clone());
    let mut edits = casual_slide_package::PartEdits::default();
    let damage = b"<p:cmLst not xml".to_vec();
    edits.replace("ppt/comments/comment1.xml".to_owned(), damage.clone());
    let damaged = pkg.save_with(&edits).unwrap();
    let (_, mut r) = read_all(&damaged);
    assert_eq!(r.comments.set_aside, vec!["ppt/comments/comment1.xml".to_owned()]);
    let unedited = save(&damaged, &r).unwrap();
    assert_eq!(unedited.bytes, damaged);
    r.comments.legacy[0].list.comments[0].text = Some("Edited".to_owned());
    let out = save(&damaged, &r).unwrap();
    assert_eq!(parts(&out.bytes)["ppt/comments/comment1.xml"], damage);
    assert_eq!(out.changed_parts, vec!["ppt/comments/comment2.xml".to_owned()]);
}

#[test]
fn a_set_aside_section_list_is_not_touched_by_an_edit_of_the_main_part() {
    let source = read_fixture("hostile/section-limit-sections.bin");
    let mut pkg = open(source);
    let import = ImportLimits::default();
    let mut d = import_deck(&mut pkg, &import).unwrap().deck;
    assert_eq!(d.presentation.sections, None);
    d.presentation.first_slide_num = Some(7);
    let out = save_deck(&mut pkg, &d, &import).unwrap();
    assert_eq!(text_of(&out.bytes, "ppt/presentation.xml").matches("<p14:section ").count(), 10_001);
}

// ------------------------------------------------------------- what validates

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

static SCRATCH: AtomicUsize = AtomicUsize::new(0);

/// F4 over edited decks. `xmllint` is required in CI, and asked for with `F4_REQUIRED`.
fn f4(decks: &[(String, Vec<u8>)]) -> Result<(), String> {
    let dir = std::env::temp_dir().join(format!(
        "review-f4-{}-{}",
        std::process::id(),
        SCRATCH.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    for (name, bytes) in decks {
        std::fs::write(dir.join(format!("{name}.pptx")), bytes).unwrap();
    }
    let output = Command::new("python3")
        .arg(repo().join("tools/f4/validate.py"))
        .arg(&dir)
        .output()
        .unwrap();
    let keep = std::env::var_os("REVIEW_KEEP_EDITED").is_some();
    if !keep {
        std::fs::remove_dir_all(&dir).ok();
    }
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// Every kind of edit here, saved, through F4; `REVIEW_EDITED_OUT` leaves them for the SDK run.
#[test]
fn every_edited_deck_validates_through_f4() {
    let mut decks = Vec::new();
    let source = read_fixture(LEGACY);
    let (_, mut r) = read_all(&source);
    r.comments.legacy[0].list.comments[0].text = Some("Edited".to_owned());
    r.comments.legacy[1].list.comments[0].threading = Some(Default::default());
    r.comments.comment_authors.as_mut().unwrap().authors[1].presence = Some(Default::default());
    r.comments.comment_authors.as_mut().unwrap().authors[1].presence.as_mut().unwrap().user_id = Some("u".to_owned());
    r.comments.comment_authors.as_mut().unwrap().authors[1].presence.as_mut().unwrap().provider_id = Some("None".to_owned());
    decks.push(("legacy".to_owned(), save(&source, &r).unwrap().bytes));

    let source = read_fixture(MODERN);
    let (_, mut r) = read_all(&source);
    let list = &mut r.comments.threaded[0].list.comments;
    list[1].status = Some(CommentStatus::Closed);
    list[2].title = Some("New".to_owned());
    let TextItem::Run(run) = &mut list[0].reply_lst.as_mut().unwrap()[0].tx_body.as_mut().unwrap().paragraphs[0].items[0] else { panic!() };
    run.text = Some("Edited".to_owned());
    r.comments.authors.as_mut().unwrap().authors[1].initials = Some("BC".to_owned());
    decks.push(("modern".to_owned(), save(&source, &r).unwrap().bytes));

    let source = read_fixture(SECTIONS);
    let (bytes, _) = with_sections(&source, |list| {
        let moved = list.sections[0].sld_id_lst.as_mut().unwrap().pop().unwrap();
        list.sections[1].sld_id_lst.as_mut().unwrap().insert(0, moved);
        list.sections[0].name = Some("First".to_owned());
    });
    let (_, mut r) = read_all(&bytes);
    let at = r.marks.iter().position(|(p, _)| p.ends_with("slide1.xml")).unwrap();
    let Zoom::Slide(Some(zoom)) = &mut r.marks[at].1.zooms[0].zoom else { panic!() };
    zoom.zm_pr.as_mut().unwrap().show_bg = Some(true);
    for m in &mut r.marks[at].1.classifications {
        m.outcome = Some(ClassificationOutcome::Watermark);
    }
    decks.push(("sections".to_owned(), save(&bytes, &r).unwrap().bytes));

    let source = read_fixture(CUSTOM);
    let (_, mut r) = read_all(&source);
    r.custom.tags[0].list.tags[0].val = Some("Beta".to_owned());
    r.custom.custom_xml[0].properties.as_mut().unwrap().schema_refs.as_mut().unwrap()[1].uri = Some("urn:new".to_owned());
    decks.push(("custom".to_owned(), save(&source, &r).unwrap().bytes));

    for path in ["real/z14230715-ddi-ko.pptx", "real/z17310490-vierfelder-de.pptx"] {
        let source = read_fixture(path);
        let (_, mut r) = read_all(&source);
        if let Some(l) = &mut r.comments.comment_authors {
            l.authors[0].initials = Some("XX".to_owned());
        }
        if let Some(l) = &mut r.comments.authors {
            l.authors[0].initials = Some("XX".to_owned());
        }
        let name = path.rsplit('/').next().unwrap().trim_end_matches(".pptx").to_owned();
        decks.push((name, save(&source, &r).unwrap().bytes));
    }
    if let Some(dir) = std::env::var_os("REVIEW_EDITED_OUT") {
        std::fs::create_dir_all(&dir).unwrap();
        for (name, bytes) in &decks {
            std::fs::write(Path::new(&dir).join(format!("{name}.pptx")), bytes).unwrap();
        }
    }
    f4(&decks).unwrap();
}
