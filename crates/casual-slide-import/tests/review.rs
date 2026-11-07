//! Tier 1M import: legacy and modern comments, threads, mentions, reactions, tasks; sections
//! against `sldIdLst`; zoom with its fallback; classification marks and labels; tags, custom XML,
//! change tracking and revision info named as retained. Damaged parts set aside. Nothing from a
//! document reaches a finding. A silent drop fails here, and every row of the tier's five areas
//! is accounted for.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use casual_slide_import::comment::{ReviewFinding, ReviewLimits};
use casual_slide_import::custom_data::{
    ImportedReview, import_datastore_item, import_label_info, import_review, import_schema_library,
    import_tags,
};
use casual_slide_import::presentation::import_deck;
use casual_slide_import::properties::{Findings, ImportFindingKind as K, ImportLimits};
use casual_slide_import::section::import_part_marks;
use casual_slide_model::comment::{CommentAnchor, CommentStatus, TaskAction};
use casual_slide_model::custom_data::RetainedKind;
use casual_slide_model::section::{
    ClassificationOutcome, DesignerMark, LabelSource, SummaryLayout, Zoom, ZoomFallback,
    ZoomImageType,
};
use casual_slide_ooxml::{ElementId, OoxmlLimits, PresentationPackage, XmlTree};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(fixtures().join(path)).unwrap()
}

fn open(bytes: Vec<u8>) -> PresentationPackage {
    PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap()
}

fn review_of(
    bytes: Vec<u8>,
) -> (
    casual_slide_model::presentation::Deck,
    Findings,
    ImportedReview,
) {
    let mut pkg = open(bytes);
    let import = ImportLimits::default();
    let imported = import_deck(&mut pkg, &import).unwrap();
    let review = import_review(&mut pkg, &imported.deck, &ReviewLimits::default(), &import);
    (imported.deck, imported.findings, review)
}

fn codes(findings: &Findings) -> Vec<String> {
    findings
        .iter()
        .map(|f| {
            format!(
                "{} {} {} {:?}",
                f.kind.code(),
                f.part,
                f.element,
                f.attribute
            )
        })
        .collect()
}

#[test]
fn legacy_comments_authors_presence_and_threads_read() {
    let (_, _, r) = review_of(read_fixture("generated/comments-legacy.pptx"));
    let authors = r.comments.comment_authors.as_ref().unwrap();
    assert_eq!(authors.authors.len(), 2);
    let ann = &authors.authors[0];
    assert_eq!(ann.id, Some(1));
    assert_eq!(ann.name.as_deref(), Some("Ann Lee"));
    assert_eq!(ann.initials.as_deref(), Some("AL"));
    assert_eq!(ann.last_idx, Some(2));
    assert_eq!(ann.clr_idx, Some(0));
    let presence = ann.presence.as_ref().unwrap();
    assert_eq!(presence.user_id.as_deref(), Some("S::ann@example.org::1"));
    assert_eq!(presence.provider_id.as_deref(), Some("AD"));
    assert_eq!(r.comments.legacy.len(), 2);
    let slide = &r.comments.legacy[0];
    assert_eq!(slide.slide_part, "ppt/slides/slide1.xml");
    let first = &slide.list.comments[0];
    assert_eq!(first.author_id, Some(1));
    assert_eq!(first.idx, Some(1));
    assert_eq!(first.text.as_deref(), Some("First note"));
    assert_eq!(
        first.dt.as_ref().unwrap().as_str(),
        "2024-03-17T08:18:03.526"
    );
    assert_eq!(first.pos.unwrap().x.0, 10);
    assert_eq!(first.threading.as_ref().unwrap().time_zone_bias, Some(-330));
    let reply = &slide.list.comments[1];
    let parent = reply.threading.as_ref().unwrap().parent.unwrap();
    assert_eq!((parent.author_id, parent.idx), (Some(1), Some(1)));
    assert_eq!(r.comments.legacy[1].list.comments[0].dt, None);
    assert!(r.comments.set_aside.is_empty());
}

#[test]
fn modern_comments_threads_mentions_reactions_and_tasks_read() {
    let (_, _, r) = review_of(read_fixture("generated/comments-modern.pptx"));
    let authors = r.comments.authors.as_ref().unwrap();
    assert_eq!(
        authors.authors[0].user_id.as_deref(),
        Some("S::ann@example.org::1")
    );
    assert_eq!(authors.authors[0].provider_id.as_deref(), Some("AD"));
    assert_eq!(authors.authors[0].initials.as_deref(), Some("AL"));
    assert_eq!(authors.authors[1].initials, None);
    let list = &r.comments.threaded[0].list;
    assert_eq!(list.comments.len(), 4);
    let first = &list.comments[0];
    assert_eq!(first.status, Some(CommentStatus::Active));
    assert!(matches!(&first.anchor, Some(CommentAnchor::Slide(m)) if m[1].kind == "sldMk"));
    assert_eq!(first.pos.unwrap().y.0, 200);
    assert_eq!(
        first.mentions(),
        vec!["{22222222-2222-4222-8222-222222222222}"],
        "a mention is a field of type mention, Inferred"
    );
    let replies = first.reply_lst.as_ref().unwrap();
    assert_eq!(replies[0].status, Some(CommentStatus::Resolved));
    assert!(replies[0].reactions.is_some());
    assert!(replies[0].tx_body.is_some());
    let reaction = &first.reactions.as_ref().unwrap().rxn[0];
    assert_eq!(reaction.reaction_type.as_deref(), Some("\u{1F44D}"));
    assert_eq!(
        reaction.instance[0].time.as_ref().unwrap().as_str(),
        "2024-03-17T10:00:00"
    );
    assert!(reaction.instance[0].author_id.is_some());
    assert!(matches!(
        list.comments[1].anchor,
        Some(CommentAnchor::Drawing(_))
    ));
    let task = &list.comments[2];
    assert_eq!(task.status, Some(CommentStatus::Closed));
    assert!(matches!(task.anchor, Some(CommentAnchor::Text(_))));
    assert_eq!(task.title.as_deref(), Some("Check the numbers"));
    assert_eq!(task.assigned_to.as_ref().unwrap().len(), 2);
    assert_eq!(task.complete.as_ref().unwrap().value().0, 50_000);
    assert!(task.start_date.is_some() && task.due_date.is_some());
    let events = &task.task.as_ref().unwrap().history.as_ref().unwrap().event;
    assert_eq!(events.len(), 8);
    assert!(events[0].atrbtn.is_some() && events[0].anchr.is_some() && events[0].id.is_some());
    assert!(matches!(
        events[0].action,
        Some(TaskAction::Assign(Some(_)))
    ));
    assert!(matches!(events[1].action, Some(TaskAction::Add)));
    assert!(
        matches!(&events[2].action, Some(TaskAction::Title(Some(t))) if t == "Check the numbers")
    );
    assert!(matches!(
        events[3].action,
        Some(TaskAction::Date {
            st_dt: Some(_),
            end_dt: Some(_)
        })
    ));
    assert!(matches!(
        events[4].action,
        Some(TaskAction::PercentComplete(Some(_)))
    ));
    assert!(matches!(events[5].action, Some(TaskAction::UnassignAll)));
    assert!(matches!(events[6].action, Some(TaskAction::Undo(Some(_)))));
    assert!(matches!(events[7].action, Some(TaskAction::Unknown)));
    assert!(matches!(
        list.comments[3].anchor,
        Some(CommentAnchor::Unknown)
    ));
    assert_eq!(list.comments[3].status, None, "absent stays absent");
    assert_eq!(r.marks[0].1.comment_rel.as_deref(), Some("rId2"));
    assert!(
        !codes(&r.findings).iter().any(|c| c.contains("CS-RVW")),
        "{:?}",
        codes(&r.findings)
    );
}

#[test]
fn sections_read_and_agree_with_the_slide_list() {
    let (deck, findings, _) = review_of(read_fixture("generated/sections-zoom-labels.pptx"));
    let list = deck.presentation.sections.as_ref().unwrap();
    assert_eq!(list.sections.len(), 3);
    assert_eq!(list.sections[0].name.as_deref(), Some("Opening"));
    assert_eq!(
        list.sections[0].sld_id_lst.as_deref(),
        Some(&[256, 257][..])
    );
    assert_eq!(list.sections[2].sld_id_lst.as_deref(), Some(&[][..]));
    assert!(list.check_against(&[256, 257, 258]).in_sync());
    assert!(
        !codes(&findings).iter().any(|c| c.contains("CS-RVW")),
        "{:?}",
        codes(&findings)
    );
    // The other `ext` in the list is kept, and said.
    assert!(
        codes(&findings)
            .iter()
            .any(|c| c.contains("CS-IMF-0010") && c.contains("extLst"))
    );
}

#[test]
fn a_section_list_out_of_step_opens_and_each_way_is_reported() {
    let list = |sections: &str| {
        format!(
            r#"<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:sldIdLst><p:sldId id="256" r:id="rId1"/><p:sldId id="257" r:id="rId2"/></p:sldIdLst><p:extLst><p:ext uri="{{521415D9-36F7-43E2-AB2F-B90AF26B5E84}}"><p14:sectionLst xmlns:p14="http://schemas.microsoft.com/office/powerpoint/2010/main">{sections}</p14:sectionLst></p:ext></p:extLst></p:presentation>"#
        )
    };
    let section = |ids: &[u32]| {
        let entries: String = ids
            .iter()
            .map(|i| format!(r#"<p14:sldId id="{i}"/>"#))
            .collect();
        format!(
            r#"<p14:section name="s" id="{{AAAAAAA1-0000-4000-8000-000000000001}}"><p14:sldIdLst>{entries}</p14:sldIdLst></p14:section>"#
        )
    };
    let report = |ids: &[u32]| {
        let xml = list(&section(ids));
        let tree = XmlTree::parse(xml.as_bytes(), "p", &OoxmlLimits::default()).unwrap();
        let import = ImportLimits::default();
        let mut f = Findings::new(100);
        let rels: Vec<casual_slide_ooxml::Relationship> = Vec::new();
        let p = casual_slide_import::presentation::import_presentation(
            &tree, "p", &rels, &import, &mut f,
        )
        .unwrap();
        p.sections.unwrap().check_against(&[256, 257])
    };
    assert!(report(&[256, 257]).in_sync());
    assert_eq!(report(&[256, 257, 999]).unknown, vec![999]);
    assert_eq!(report(&[256]).unlisted, vec![257]);
    assert_eq!(report(&[256, 256, 257]).repeated, vec![256]);
    assert!(report(&[257, 256]).out_of_order);
    let (_, findings, _) = review_of(read_fixture("hostile/section-bad-values.bin"));
    let said = codes(&findings);
    for code in ["CS-RVW-0004", "CS-RVW-0005", "CS-RVW-0006", "CS-RVW-0007"] {
        assert!(said.iter().any(|c| c.contains(code)), "{code}: {said:?}");
    }
}

#[test]
fn zooms_read_with_their_fallbacks() {
    let (_, _, r) = review_of(read_fixture("generated/sections-zoom-labels.pptx"));
    let marks = &r
        .marks
        .iter()
        .find(|(p, _)| p.ends_with("slide1.xml"))
        .unwrap()
        .1;
    assert_eq!(marks.zooms.len(), 4);
    let Zoom::Slide(Some(slide)) = &marks.zooms[0].zoom else {
        panic!()
    };
    assert_eq!(slide.sld_id, Some(257));
    assert_eq!(slide.c_id, Some(42));
    let props = slide.zm_pr.as_ref().unwrap();
    assert_eq!(props.return_to_parent, Some(true));
    assert_eq!(props.image_type, Some(ZoomImageType::Preview));
    assert_eq!(props.transition_dur.as_deref(), Some("1000"));
    assert_eq!(props.show_bg, Some(false));
    assert!(props.id.is_some() && props.blip_fill.is_some() && props.sp_pr.is_some());
    assert_eq!(marks.zooms[0].fallback, ZoomFallback::Picture);
    assert_eq!(marks.zooms[0].shape_id, Some(5));
    let Zoom::Section(Some(section)) = &marks.zooms[1].zoom else {
        panic!()
    };
    assert!(section.section_id.is_some());
    assert_eq!(
        section.zm_pr.as_ref().unwrap().image_type,
        Some(ZoomImageType::Cover)
    );
    let Zoom::Summary { objects, layout } = &marks.zooms[2].zoom else {
        panic!()
    };
    assert_eq!(*layout, Some(SummaryLayout::Grid));
    assert_eq!(objects[0].title.as_deref(), Some("Opening"));
    assert_eq!(objects[0].descr.as_deref(), Some("First part"));
    assert_eq!(objects[0].offset_factor_x.unwrap().0, 10_000);
    assert_eq!(objects[0].offset_factor_y.unwrap().0, 20_000);
    assert_eq!(objects[0].scale_factor_x.unwrap().0, 90_000);
    assert_eq!(objects[0].scale_factor_y.unwrap().0, 80_000);
    assert_eq!(marks.zooms[2].fallback, ZoomFallback::Group);
    let Zoom::Summary { layout, .. } = &marks.zooms[3].zoom else {
        panic!()
    };
    assert_eq!(*layout, Some(SummaryLayout::Fixed));
    assert!(
        !codes(&r.findings).iter().any(|c| c.contains("CS-RVW-0009")),
        "{:?}",
        codes(&r.findings)
    );
}

#[test]
fn classification_designer_and_custom_data_marks_read() {
    let (_, _, r) = review_of(read_fixture("generated/sections-zoom-labels.pptx"));
    let marks = &r
        .marks
        .iter()
        .find(|(p, _)| p.ends_with("slide1.xml"))
        .unwrap()
        .1;
    let outcomes: Vec<_> = marks
        .classifications
        .iter()
        .map(|m| (m.shape_id, m.outcome))
        .collect();
    assert!(outcomes.contains(&(Some(9), Some(ClassificationOutcome::Header))));
    assert!(outcomes.contains(&(Some(9), Some(ClassificationOutcome::Footer))));
    assert!(outcomes.contains(&(Some(20), Some(ClassificationOutcome::None))));
    assert!(outcomes.contains(&(Some(21), Some(ClassificationOutcome::Watermark))));
    assert!(marks.designer.iter().any(|m| matches!(
        m,
        DesignerMark::DesignElement {
            val: Some(true),
            shape_id: Some(9)
        }
    )));
    assert!(marks.designer.iter().any(|m| matches!(m,
        DesignerMark::DrawingProperties { edt_design_elem: Some(true), design_tag_lst: Some(t), .. }
            if t[0].name.as_deref() == Some("a") && t[0].val.as_deref() == Some("b"))));
    assert_eq!(marks.customer_data, vec!["rId3".to_owned()]);
}

#[test]
fn tags_custom_xml_and_labels_read_and_item_bodies_do_not() {
    let (_, _, r) = review_of(read_fixture("generated/custom-tags-labels.pptx"));
    assert_eq!(r.custom.tags.len(), 1);
    let tags = &r.custom.tags[0].list.tags;
    assert_eq!(tags[0].name.as_deref(), Some("PROJECT"));
    assert_eq!(tags[0].val.as_deref(), Some("Alpha"));
    assert_eq!(r.custom.custom_xml.len(), 2);
    let item = &r.custom.custom_xml[0];
    let props = item.properties.as_ref().unwrap();
    assert_eq!(
        props.item_id.as_deref(),
        Some("{CCCCCCCC-0000-4000-8000-000000000001}")
    );
    let refs = props.schema_refs.as_ref().unwrap();
    assert_eq!(refs[0].uri.as_deref(), Some("urn:example:note"));
    assert_eq!(item.schema_library, None);
    let library = r.custom.custom_xml[1].schema_library.as_ref().unwrap();
    assert_eq!(library.schemas[0].uri.as_deref(), Some("urn:example:note"));
    assert_eq!(
        library.schemas[0].manifest_location.as_deref(),
        Some("m.xml")
    );
    assert_eq!(library.schemas[0].schema_location.as_deref(), Some("n.xsd"));
    assert!(library.schemas[0].schema_language.is_some());
    assert!(
        !format!("{r:?}").contains("Item text"),
        "an item's body stays out of the model"
    );
    let part = r
        .labels
        .labels
        .iter()
        .find(|l| l.source == LabelSource::LabelPart)
        .unwrap();
    assert_eq!(part.enabled, Some(true));
    assert_eq!(part.method.as_deref(), Some("Standard"));
    assert_eq!(part.content_bits, Some(3));
    assert_eq!(part.removed, Some(false));
    let props = r
        .labels
        .labels
        .iter()
        .find(|l| l.source == LabelSource::CustomProperties)
        .unwrap();
    assert_eq!(props.name.as_deref(), Some("Confidential"));
    assert_eq!(props.enabled, Some(true));
}

#[test]
fn retained_whole_parts_are_named_and_reported() {
    let (_, _, r) = review_of(read_fixture("generated/review-every-change.pptx"));
    let kinds: Vec<_> = r.retained.iter().map(|p| p.kind).collect();
    assert!(kinds.contains(&RetainedKind::ChangesInfo));
    assert!(kinds.contains(&RetainedKind::RevisionInfo));
    let said = codes(&r.findings);
    assert!(
        said.iter()
            .any(|c| c.contains("CS-RVW-0012") && c.contains("changesInfo")),
        "{said:?}"
    );
    assert!(
        said.iter()
            .any(|c| c.contains("CS-RVW-0012") && c.contains("revisionInfo")),
        "{said:?}"
    );
    let (_, _, custom) = review_of(read_fixture("generated/custom-tags-labels.pptx"));
    assert!(
        custom
            .retained
            .iter()
            .any(|p| p.kind == RetainedKind::CustomXml)
    );
    assert!(
        custom
            .retained
            .iter()
            .any(|p| p.kind == RetainedKind::LabelInfo)
    );
}

/// What `fixtures/real` holds of this tier: both author lists and change records in one deck,
/// and no modern comment or section anywhere in it.
#[test]
fn the_real_decks_of_this_tier_read_and_say_nothing_of_who_wrote_them() {
    let (_, _, ko) = review_of(read_fixture("real/z14230715-ddi-ko.pptx"));
    assert_eq!(ko.comments.authors.as_ref().unwrap().authors.len(), 2);
    let legacy = ko.comments.comment_authors.as_ref().unwrap();
    assert!(!legacy.authors.is_empty());
    assert!(legacy.authors[0].presence.is_some());
    let kinds: Vec<_> = ko.retained.iter().map(|p| p.kind).collect();
    assert!(kinds.contains(&RetainedKind::ChangesInfo));
    assert!(kinds.contains(&RetainedKind::RevisionInfo));
    assert!(ko.comments.set_aside.is_empty());
    let said = format!("{:?}", codes(&ko.findings));
    for private in [
        "Jieun",
        "JEONG",
        "sciencespo",
        "Petrinko",
        "Yearn",
        "528bff1fc84d75a6",
    ] {
        assert!(!said.contains(private), "{private} reached a finding");
    }
    let (_, _, de) = review_of(read_fixture("real/z17310490-vierfelder-de.pptx"));
    assert!(
        de.retained
            .iter()
            .any(|p| p.kind == RetainedKind::ChangesInfo)
    );
    let (_, _, sdk) = review_of(read_fixture("real/sdk-animation.pptx"));
    assert!(sdk.custom.custom_xml.iter().any(|i| i.properties.is_some()));
}

#[test]
fn a_damaged_comment_part_is_set_aside_and_the_deck_opens() {
    for (name, broken) in [
        ("ppt/comments/comment1.xml", &b"<p:cmLst not xml"[..]),
        (
            "ppt/comments/comment1.xml",
            &b"<a:wrong xmlns:a=\"urn:x\"/>"[..],
        ),
        ("ppt/commentAuthors.xml", &b"\xff\xfe garbage"[..]),
    ] {
        let source = read_fixture("generated/comments-legacy.pptx");
        let deck = rezip(&source, name, broken);
        let (_, _, r) = review_of(deck);
        assert_eq!(r.comments.set_aside, vec![name.to_owned()]);
        let said = codes(&r.findings);
        assert!(
            said.iter()
                .any(|c| c.contains("CS-IMF-0012") && c.contains(name)),
            "{said:?}"
        );
        // The other part is still read.
        if name.contains("comment1") {
            assert_eq!(r.comments.legacy.len(), 1);
            assert_eq!(r.comments.legacy[0].slide_part, "ppt/slides/slide2.xml");
        } else {
            assert_eq!(r.comments.legacy.len(), 2);
            assert_eq!(r.comments.comment_authors, None);
        }
    }
}

#[test]
fn a_damaged_tags_or_properties_part_is_set_aside_and_the_deck_opens() {
    for name in ["ppt/tags/tag1.xml", "customXml/itemProps1.xml"] {
        let source = read_fixture("generated/custom-tags-labels.pptx");
        let (_, _, r) = review_of(rezip(&source, name, b"<not xml"));
        assert_eq!(r.custom.set_aside, vec![name.to_owned()], "{name}");
    }
    let source = read_fixture("generated/custom-tags-labels.pptx");
    let (_, _, r) = review_of(rezip(&source, "docMetadata/LabelInfo.xml", b"<not xml"));
    assert_eq!(
        r.labels.set_aside.as_deref(),
        Some("docMetadata/LabelInfo.xml")
    );
    assert_eq!(
        r.labels.labels.len(),
        1,
        "the property labels are still read"
    );
}

/// The bytes of a deck with one part changed, through this crate's own ZIP view.
fn rezip(source: &[u8], name: &str, data: &[u8]) -> Vec<u8> {
    let mut pkg = open(source.to_vec());
    let mut edits = casual_slide_package::PartEdits::default();
    edits.replace(name.to_owned(), data.to_vec());
    pkg.save_with(&edits).unwrap()
}

#[test]
fn every_bound_sets_its_part_aside_with_its_code_and_the_deck_opens() {
    for (deck, part) in [
        (
            "hostile/comment-limit-comments.bin",
            "ppt/comments/comment1.xml",
        ),
        (
            "hostile/comment-limit-authors.bin",
            "ppt/commentAuthors.xml",
        ),
        (
            "hostile/comment-limit-replies.bin",
            "ppt/comments/modernComment_100_0.xml",
        ),
        (
            "hostile/comment-limit-reactions.bin",
            "ppt/comments/modernComment_100_0.xml",
        ),
        (
            "hostile/comment-limit-task-events.bin",
            "ppt/comments/modernComment_100_0.xml",
        ),
        (
            "hostile/comment-limit-monikers.bin",
            "ppt/comments/modernComment_100_0.xml",
        ),
    ] {
        let (_, _, r) = review_of(read_fixture(deck));
        assert!(r.comments.set_aside.iter().any(|p| p == part), "{deck}");
        let said = codes(&r.findings);
        assert!(
            said.iter()
                .any(|c| c.contains("CS-RVW-0001") && c.contains(part)),
            "{deck}: {said:?}"
        );
    }
    let (_, _, r) = review_of(read_fixture("hostile/tags-limit-tags.bin"));
    assert_eq!(r.custom.set_aside, vec!["ppt/tags/tag1.xml".to_owned()]);
    let (_, _, r) = review_of(read_fixture("hostile/custom-limit-schema-refs.bin"));
    assert_eq!(
        r.custom.set_aside,
        vec!["customXml/itemProps1.xml".to_owned()]
    );
    let (_, _, r) = review_of(read_fixture("hostile/custom-limit-labels.bin"));
    assert_eq!(
        r.labels.set_aside.as_deref(),
        Some("docMetadata/LabelInfo.xml")
    );
    // The marks of a part stop at their bound and stay in the tree: the part is still read.
    let (_, _, r) = review_of(read_fixture("hostile/marks-limit-part.bin"));
    assert_eq!(r.marks[0].1.customer_data.len(), 100_000);
    assert!(
        codes(&r.findings)
            .iter()
            .any(|c| c.contains("CS-RVW-0001") && c.contains("slide1"))
    );
    // Parts past their bound: at a lowered bound, as 100,001 parts cross `CS-PKG-0002` first.
    let source = read_fixture("generated/comments-legacy.pptx");
    let mut pkg = open(source);
    let import = ImportLimits::default();
    let d = import_deck(&mut pkg, &import).unwrap().deck;
    let few = ReviewLimits {
        max_review_parts: 1,
        ..ReviewLimits::default()
    };
    let r = import_review(&mut pkg, &d, &few, &import);
    assert_eq!(r.comments.legacy.len(), 1);
    assert_eq!(
        r.comments.set_aside,
        vec!["ppt/comments/comment2.xml".to_owned()]
    );
    for deck in [
        "hostile/section-limit-sections.bin",
        "hostile/section-limit-slides.bin",
    ] {
        let (d, findings, _) = review_of(read_fixture(deck));
        assert_eq!(d.presentation.sections, None, "{deck}");
        let said = codes(&findings);
        assert!(
            said.iter()
                .any(|c| c.contains("CS-RVW-0001") && c.contains("sectionLst")),
            "{deck}: {said:?}"
        );
    }
}

/// Modern comments and their author list, one past their bound and exactly at it, at a
/// lowered bound: the generated deck holds four comments and two authors.
#[test]
fn a_modern_comment_or_author_list_past_its_bound_is_set_aside() {
    let read = |limits: ReviewLimits| {
        let mut pkg = open(read_fixture("generated/comments-modern.pptx"));
        let import = ImportLimits::default();
        let d = import_deck(&mut pkg, &import).unwrap().deck;
        import_review(&mut pkg, &d, &limits, &import)
    };
    let at = read(ReviewLimits {
        max_comments_per_part: 4,
        max_authors: 2,
        ..ReviewLimits::default()
    });
    assert!(
        at.comments.set_aside.is_empty(),
        "{:?}",
        at.comments.set_aside
    );
    let over = read(ReviewLimits {
        max_comments_per_part: 3,
        ..ReviewLimits::default()
    });
    assert_eq!(
        over.comments.set_aside,
        vec!["ppt/comments/modernComment_100_0.xml".to_owned()]
    );
    let over = read(ReviewLimits {
        max_authors: 1,
        ..ReviewLimits::default()
    });
    assert_eq!(over.comments.set_aside, vec!["ppt/authors.xml".to_owned()]);
}

/// Reactions under one holder are bounded two ways: too many `rxn`, and `rxn` with their
/// `instance` counted together. At a lowered bound of two, each way alone refuses the part.
#[test]
fn reactions_past_their_bound_either_way_refuse_the_part() {
    let limits = ReviewLimits {
        max_reactions_per_holder: 2,
        ..ReviewLimits::default()
    };
    let read = |reactions: &str| {
        let xml = format!(
            r#"<p188:cmLst xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:p188="http://schemas.microsoft.com/office/powerpoint/2018/8/main" xmlns:pc="http://schemas.microsoft.com/office/powerpoint/2013/main/command"><p188:cm id="{{0000000A-AAAA-4BBB-8CCC-000000000000}}" authorId="{{11111111-1111-4111-8111-111111111111}}" created="2024-03-17T08:00:00.000"><pc:sldMkLst><pc:docMk/><pc:sldMk cId="1" sldId="256"/></pc:sldMkLst><p188:txBody><a:bodyPr/><a:lstStyle/><a:p/></p188:txBody><p188:extLst><p:ext uri="{{6F9A3E12-4B5D-4C7E-8F01-23456789ABCD}}"><p223:reactions xmlns:p223="http://schemas.microsoft.com/office/powerpoint/2022/03/main">{reactions}</p223:reactions></p:ext></p188:extLst></p188:cm></p188:cmLst>"#
        );
        let tree = XmlTree::parse(xml.as_bytes(), "c", &OoxmlLimits::default()).unwrap();
        casual_slide_import::comment::import_threaded_comments(
            &tree,
            "c",
            &[],
            &limits,
            &ImportLimits::default(),
            &mut Findings::new(100),
            &mut casual_slide_import::color::Findings::default(),
        )
    };
    let rxn = r#"<p223:rxn type="a"/>"#;
    let instance = r#"<p223:instance time="2024-03-17T10:00:00" authorId="{22222222-2222-4222-8222-222222222222}"/>"#;
    let with = |n: usize| format!(r#"<p223:rxn type="a">{}</p223:rxn>"#, instance.repeat(n));
    // At the bound: two reactions, or one holding one instance.
    assert!(read(&rxn.repeat(2)).is_ok());
    assert!(read(&with(1)).is_ok());
    // Three reactions, none with an instance.
    assert_eq!(read(&rxn.repeat(3)).unwrap_err().code(), "CS-IMP-0001");
    // One reaction and two instances: three together.
    assert_eq!(read(&with(2)).unwrap_err().code(), "CS-IMP-0001");
    // One reaction holding three children that are not instances: only the count of a
    // reaction's own children stops these, as the combined count sees instances alone.
    let unknown = r#"<p223:rxn type="a"><p223:x/><p223:x/><p223:x/></p223:rxn>"#;
    assert_eq!(read(unknown).unwrap_err().code(), "CS-IMP-0001");
}

fn review_at(deck: &str, limits: ReviewLimits) -> ImportedReview {
    let mut pkg = open(read_fixture(deck));
    let import = ImportLimits::default();
    let d = import_deck(&mut pkg, &import).unwrap().deck;
    import_review(&mut pkg, &d, &limits, &import)
}

/// `max_review_parts` counts modern comment parts too: one past it, the part is set aside.
#[test]
fn a_modern_comment_part_past_the_part_count_is_set_aside() {
    let at = review_at(
        "generated/comments-modern.pptx",
        ReviewLimits {
            max_review_parts: 1,
            ..ReviewLimits::default()
        },
    );
    assert_eq!(at.comments.threaded.len(), 1);
    let over = review_at(
        "generated/comments-modern.pptx",
        ReviewLimits {
            max_review_parts: 0,
            ..ReviewLimits::default()
        },
    );
    assert!(over.comments.threaded.is_empty());
    assert!(
        over.comments
            .set_aside
            .contains(&"ppt/comments/modernComment_100_0.xml".to_owned()),
        "{:?}",
        over.comments.set_aside
    );
}

/// `max_review_parts` counts tag parts and custom XML items, each on its own count.
#[test]
fn tag_parts_and_custom_xml_items_past_the_part_count_are_set_aside() {
    let deck = "generated/custom-tags-labels.pptx";
    let at = review_at(
        deck,
        ReviewLimits {
            max_review_parts: 2,
            ..ReviewLimits::default()
        },
    );
    assert!(at.custom.set_aside.is_empty(), "{:?}", at.custom.set_aside);
    let one = review_at(
        deck,
        ReviewLimits {
            max_review_parts: 1,
            ..ReviewLimits::default()
        },
    );
    assert_eq!(one.custom.set_aside, vec!["customXml/item2.xml".to_owned()]);
    let none = review_at(
        deck,
        ReviewLimits {
            max_review_parts: 0,
            ..ReviewLimits::default()
        },
    );
    assert!(
        none.custom
            .set_aside
            .contains(&"ppt/tags/tag1.xml".to_owned()),
        "{:?}",
        none.custom.set_aside
    );
}

/// A schema library past `max_schema_refs` is refused; exactly at it, it reads.
#[test]
fn a_schema_library_past_its_bound_is_refused() {
    let limits = ReviewLimits {
        max_schema_refs: 2,
        ..ReviewLimits::default()
    };
    let read = |n: usize| {
        let schemas = r#"<sl:schema sl:uri="urn:a"/>"#.repeat(n);
        let xml = format!(
            r#"<sl:schemaLibrary xmlns:sl="http://schemas.openxmlformats.org/schemaLibrary/2006/main">{schemas}</sl:schemaLibrary>"#
        );
        let tree = XmlTree::parse(xml.as_bytes(), "l", &OoxmlLimits::default()).unwrap();
        import_schema_library(
            &tree,
            "l",
            &limits,
            &ImportLimits::default(),
            &mut Findings::new(10),
        )
    };
    assert_eq!(read(2).unwrap().schemas.len(), 2);
    assert_eq!(read(3).unwrap_err().code(), "CS-IMP-0001");
}

/// Labels left in custom properties stop at `max_labels`.
#[test]
fn labels_in_custom_properties_stop_at_their_bound() {
    let mut pkg = open(read_fixture("generated/custom-tags-labels.pptx"));
    let d = import_deck(&mut pkg, &ImportLimits::default())
        .unwrap()
        .deck;
    let at = |n: usize| {
        casual_slide_import::custom_data::labels_in_custom_properties(
            &d,
            &ReviewLimits {
                max_labels: n,
                ..ReviewLimits::default()
            },
        )
    };
    assert_eq!(at(1).len(), 1);
    assert!(at(0).is_empty());
}

/// Designer tags and summary zoom objects are marks too: past the bound the rest stay in the
/// tree, unread, and the part is reported.
#[test]
fn designer_tags_past_the_mark_bound_stay_unread_and_are_reported() {
    let read = |n: usize| {
        let tags = r#"<p202:designTag name="a" val="b"/>"#.repeat(3);
        let xml = format!(
            r#"<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><p:cSld><p:spTree><p:sp><p:nvSpPr><p:cNvPr id="9" name="L"/><p:cNvSpPr/><p:nvPr><p:extLst><p:ext uri="{{3A5E9C8B-7D21-4F60-8E1A-5C4B3D2E1F00}}"><p202:designPr xmlns:p202="http://schemas.microsoft.com/office/powerpoint/2020/02/main"><p202:designTagLst>{tags}</p202:designTagLst></p202:designPr></p:ext></p:extLst></p:nvPr></p:nvSpPr><p:spPr/></p:sp></p:spTree></p:cSld></p:sld>"#
        );
        let tree = XmlTree::parse(xml.as_bytes(), "s", &OoxmlLimits::default()).unwrap();
        let limits = ReviewLimits {
            max_marks_per_part: n,
            ..ReviewLimits::default()
        };
        let mut f = Findings::new(100);
        let marks =
            import_part_marks(&tree, "s", &[], &limits, &ImportLimits::default(), &mut f).unwrap();
        let tags = match marks.designer.first() {
            Some(DesignerMark::DrawingProperties { design_tag_lst, .. }) => {
                design_tag_lst.as_ref().map_or(0, Vec::len)
            }
            other => panic!("{other:?}"),
        };
        (tags, codes(&f).iter().any(|c| c.contains("CS-RVW-0001")))
    };
    assert_eq!(read(3), (3, false));
    assert_eq!(read(2), (2, true));
}

/// One past each bound is what the hostile decks hold. Exactly at it, a part opens.
#[test]
fn a_part_exactly_at_its_bound_is_read() {
    let limits = ReviewLimits {
        max_tags: 2,
        ..ReviewLimits::default()
    };
    let import = ImportLimits::default();
    let read = |n: usize| {
        let tags = r#"<p:tag name="a" val="b"/>"#.repeat(n);
        let xml = format!(
            r#"<p:tagLst xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">{tags}</p:tagLst>"#
        );
        let tree = XmlTree::parse(xml.as_bytes(), "t", &OoxmlLimits::default()).unwrap();
        import_tags(&tree, "t", &limits, &import, &mut Findings::new(10))
    };
    assert_eq!(read(2).unwrap().tags.len(), 2);
    assert_eq!(read(3).unwrap_err().code(), "CS-IMP-0001");
}

#[test]
fn hostile_values_are_reported_and_reach_no_finding() {
    let mark = "QzMarkerQz";
    for deck in [
        "hostile/comment-bad-values.bin",
        "hostile/section-bad-values.bin",
        "hostile/custom-bad-values.bin",
    ] {
        let (_, opened, r) = review_of(read_fixture(deck));
        let said = format!("{:?}{:?}", codes(&opened), codes(&r.findings));
        assert!(!said.contains(mark), "{deck}: {said}");
        assert!(!said.is_empty(), "{deck}");
        assert!(
            said.contains("CS-IMF-0001") || said.contains("CS-RVW"),
            "{deck}: {said}"
        );
    }
    let (_, _, r) = review_of(read_fixture("hostile/comment-bad-values.bin"));
    let said = codes(&r.findings);
    for code in [
        "CS-IMF-0001",
        "CS-IMF-0003",
        "CS-IMF-0004",
        "CS-IMF-0010",
        "CS-IMF-0011",
        "CS-RVW-0002",
        "CS-RVW-0004",
    ] {
        assert!(said.iter().any(|c| c.contains(code)), "{code}: {said:?}");
    }
    let bad = &r.comments.threaded[0].list.comments[0];
    assert_eq!(bad.id, None);
    assert_eq!(bad.status, None);
    assert_eq!(bad.complete, None);
    assert_eq!(bad.assigned_to, None);
}

#[test]
fn a_zoom_with_no_fallback_or_no_target_is_reported() {
    let (_, _, r) = review_of(read_fixture("generated/sections-zoom-labels.pptx"));
    assert!(!codes(&r.findings).iter().any(|c| c.contains("CS-RVW-0010")));
    let source = read_fixture("generated/sections-zoom-labels.pptx");
    let mut pkg = open(source.clone());
    let slide = String::from_utf8(pkg.read_part("ppt/slides/slide1.xml").unwrap()).unwrap();
    let cut = slide
        .replace("<mc:Fallback>", "<mc:Fallback><!-- -->")
        .replace(r#"sldId="257""#, r#"sldId="999""#);
    let start = cut.find("<mc:Fallback>").unwrap();
    let end = cut.find("</mc:Fallback>").unwrap();
    let cut = format!("{}{}", &cut[..start], &cut[end + "</mc:Fallback>".len()..]);
    let (_, _, r) = review_of(rezip(&source, "ppt/slides/slide1.xml", cut.as_bytes()));
    let said = codes(&r.findings);
    assert!(said.iter().any(|c| c.contains("CS-RVW-0010")), "{said:?}");
    assert!(said.iter().any(|c| c.contains("CS-RVW-0009")), "{said:?}");
}

#[test]
fn a_comment_rel_naming_nothing_is_reported() {
    let source = read_fixture("generated/comments-modern.pptx");
    let mut pkg = open(source.clone());
    let slide = String::from_utf8(pkg.read_part("ppt/slides/slide1.xml").unwrap()).unwrap();
    let moved = slide.replace(r#"r:id="rId2""#, r#"r:id="rId9""#);
    let (_, _, r) = review_of(rezip(&source, "ppt/slides/slide1.xml", moved.as_bytes()));
    assert!(codes(&r.findings).iter().any(|c| c.contains("CS-RVW-0011")));
}

#[test]
fn a_comment_naming_nobody_is_reported() {
    let source = read_fixture("generated/comments-legacy.pptx");
    let moved = LEGACY_ORPHAN.as_bytes();
    let (_, _, r) = review_of(rezip(&source, "ppt/comments/comment2.xml", moved));
    let said = codes(&r.findings);
    assert!(
        said.iter()
            .any(|c| c.contains("CS-RVW-0002") && c.contains("comment2")),
        "{said:?}"
    );
    assert!(
        said.iter()
            .any(|c| c.contains("CS-RVW-0003") && c.contains("comment2")),
        "{said:?}"
    );
}

const LEGACY_ORPHAN: &str = r#"<p:cmLst xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cm authorId="9" idx="1"><p:extLst><p:ext uri="x"><p15:threadingInfo xmlns:p15="http://schemas.microsoft.com/office/powerpoint/2012/main"><p15:parentCm authorId="9" idx="77"/></p15:threadingInfo></p:ext></p:extLst></p:cm></p:cmLst>"#;

// ------------------------------------------------------------------- a silent drop fails

type Read = (String, Vec<String>);

fn local(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

/// A reader of one kind of part, as model text and the report.
type PartReader = fn(&XmlTree) -> Option<Read>;

fn lines(f: &Findings) -> Vec<String> {
    let mut out: Vec<String> = f
        .iter()
        .map(|x| format!("{} {} {:?}", x.kind.code(), x.element, x.attribute))
        .collect();
    out.sort();
    out
}

fn read_comments(tree: &XmlTree) -> Option<Read> {
    let mut f = Findings::new(1000);
    let v = casual_slide_import::comment::import_comments(
        tree,
        "p",
        &ReviewLimits::default(),
        &ImportLimits::default(),
        &mut f,
    )
    .ok()?;
    Some((format!("{v:?}"), lines(&f)))
}

fn read_cm_authors(tree: &XmlTree) -> Option<Read> {
    let mut f = Findings::new(1000);
    let v = casual_slide_import::comment::import_comment_authors(
        tree,
        "p",
        &ReviewLimits::default(),
        &ImportLimits::default(),
        &mut f,
    )
    .ok()?;
    Some((format!("{v:?}"), lines(&f)))
}

fn read_authors(tree: &XmlTree) -> Option<Read> {
    let mut f = Findings::new(1000);
    let v = casual_slide_import::comment::import_authors(
        tree,
        "p",
        &ReviewLimits::default(),
        &ImportLimits::default(),
        &mut f,
    )
    .ok()?;
    Some((format!("{v:?}"), lines(&f)))
}

fn read_threaded(tree: &XmlTree) -> Option<Read> {
    let mut f = Findings::new(1000);
    let mut c = casual_slide_import::color::Findings::default();
    let v = casual_slide_import::comment::import_threaded_comments(
        tree,
        "p",
        &[],
        &ReviewLimits::default(),
        &ImportLimits::default(),
        &mut f,
        &mut c,
    )
    .ok()?;
    Some((format!("{v:?}"), lines(&f)))
}

fn read_tags(tree: &XmlTree) -> Option<Read> {
    let mut f = Findings::new(1000);
    let v = import_tags(
        tree,
        "p",
        &ReviewLimits::default(),
        &ImportLimits::default(),
        &mut f,
    )
    .ok()?;
    Some((format!("{v:?}"), lines(&f)))
}

fn read_store(tree: &XmlTree) -> Option<Read> {
    let mut f = Findings::new(1000);
    let v = import_datastore_item(
        tree,
        "p",
        &ReviewLimits::default(),
        &ImportLimits::default(),
        &mut f,
    )
    .ok()?;
    Some((format!("{v:?}"), lines(&f)))
}

fn read_library(tree: &XmlTree) -> Option<Read> {
    let mut f = Findings::new(1000);
    let v = import_schema_library(
        tree,
        "p",
        &ReviewLimits::default(),
        &ImportLimits::default(),
        &mut f,
    )
    .ok()?;
    Some((format!("{v:?}"), lines(&f)))
}

fn read_labels(tree: &XmlTree) -> Option<Read> {
    let mut f = Findings::new(1000);
    let v = import_label_info(
        tree,
        "p",
        &ReviewLimits::default(),
        &ImportLimits::default(),
        &mut f,
    )
    .ok()?;
    Some((format!("{v:?}"), lines(&f)))
}

fn read_marks(tree: &XmlTree) -> Option<Read> {
    let mut f = Findings::new(1000);
    let v = import_part_marks(
        tree,
        "p",
        &[],
        &ReviewLimits::default(),
        &ImportLimits::default(),
        &mut f,
    )
    .ok()?;
    Some((format!("{v:?}"), lines(&f)))
}

fn read_presentation(tree: &XmlTree) -> Option<Read> {
    let mut f = Findings::new(1000);
    let v = casual_slide_import::presentation::import_presentation(
        tree,
        "p",
        &[],
        &ImportLimits::default(),
        &mut f,
    )
    .ok()?;
    Some((format!("{:?}", v.sections), lines(&f)))
}

/// Everything in this part that reads no differently for being gone, under `only` if given.
fn silent_in(xml: &[u8], read: PartReader, only: Option<&str>) -> Vec<String> {
    let whole = XmlTree::parse(xml, "p", &OoxmlLimits::default()).unwrap();
    let as_read = read(&whole).unwrap();
    let mut silent = Vec::new();
    let mut stack: Vec<(ElementId, bool, Vec<String>)> =
        vec![(whole.root(), only.is_none(), Vec::new())];
    while let Some((parent, inside, above)) = stack.pop() {
        for kid in whole.child_elements(parent) {
            let name = whole.qualified_name(kid).to_owned();
            let inside = inside || only.is_some_and(|o| local(&name) == o);
            let attrs = whole.attributes(kid, "p").unwrap();
            let mut path = above.clone();
            path.push(local(&name).to_owned());
            stack.push((kid, inside, path));
            if !inside {
                continue;
            }
            let mut gone = whole.clone();
            gone.remove_child(parent, kid);
            let again = XmlTree::parse(&gone.to_bytes(), "p", &OoxmlLimits::default()).ok();
            if again.as_ref().and_then(read).as_ref() == Some(&as_read)
                && !noticed_above(&as_read, &above, &name)
            {
                silent.push(format!("<{name}>"));
            }
            for (attr, _) in attrs {
                if attr == "xmlns" || attr.starts_with("xmlns:") || attr == "uri" {
                    continue;
                }
                let mut gone = whole.clone();
                gone.remove_attribute(kid, &attr);
                let again = XmlTree::parse(&gone.to_bytes(), "p", &OoxmlLimits::default()).ok();
                if again.as_ref().and_then(read).as_ref() == Some(&as_read)
                    && !noticed_above(&as_read, &above, "")
                {
                    silent.push(format!("<{name}>/@{attr}"));
                }
            }
        }
    }
    silent.sort();
    silent.dedup();
    silent
}

/// A repeat's removal changes no report that names its kind; what holds it answers for it. An
/// `ext` holds what its `extLst` read, and is answered by the item; an empty holder is nothing
/// lost; a moniker's text is the tree's own, and read as it stands.
fn noticed_above(read: &Read, above: &[String], name: &str) -> bool {
    let named = |n: &str| read.1.iter().any(|l| l.split(' ').nth(1) == Some(n));
    named(local(name)) || above.iter().any(|a| named(a))
}

fn part_of(deck: &str, part: &str) -> Vec<u8> {
    open(read_fixture(deck)).read_part(part).unwrap()
}

#[test]
fn nothing_in_a_comment_or_custom_part_can_go_without_the_model_or_the_report_changing() {
    let cases: [(&str, &str, PartReader); 8] = [
        (
            "generated/comments-legacy.pptx",
            "ppt/commentAuthors.xml",
            read_cm_authors,
        ),
        (
            "generated/comments-legacy.pptx",
            "ppt/comments/comment1.xml",
            read_comments,
        ),
        (
            "generated/comments-modern.pptx",
            "ppt/authors.xml",
            read_authors,
        ),
        (
            "generated/comments-modern.pptx",
            "ppt/comments/modernComment_100_0.xml",
            read_threaded,
        ),
        (
            "generated/custom-tags-labels.pptx",
            "ppt/tags/tag1.xml",
            read_tags,
        ),
        (
            "generated/custom-tags-labels.pptx",
            "customXml/itemProps1.xml",
            read_store,
        ),
        (
            "generated/custom-tags-labels.pptx",
            "customXml/item2.xml",
            read_library,
        ),
        (
            "generated/custom-tags-labels.pptx",
            "docMetadata/LabelInfo.xml",
            read_labels,
        ),
    ];
    let mut silent = Vec::new();
    for (deck, part, read) in cases {
        silent.extend(
            silent_in(&part_of(deck, part), read, None)
                .into_iter()
                .map(|s| format!("{part}: {s}")),
        );
    }
    // What a moniker holds is read as it stands, by name, so it is in the model; that is its test.
    assert!(
        silent.is_empty(),
        "{} read past:\n{}",
        silent.len(),
        silent.join("\n")
    );
}

#[test]
fn nothing_of_a_section_list_or_a_mark_can_go_unnoticed() {
    let deck = "generated/sections-zoom-labels.pptx";
    let mut silent = silent_in(
        &part_of(deck, "ppt/presentation.xml"),
        read_presentation,
        Some("sectionLst"),
    );
    let slide = part_of(deck, "ppt/slides/slide1.xml");
    for holder in [
        "sldZm",
        "sectionZm",
        "summaryZm",
        "classification",
        "designElem",
        "designPr",
    ] {
        silent.extend(silent_in(&slide, read_marks, Some(holder)));
    }
    let modern = part_of("generated/comments-modern.pptx", "ppt/slides/slide1.xml");
    silent.extend(silent_in(&modern, read_marks, Some("commentRel")));
    assert!(
        silent.is_empty(),
        "{} read past:\n{}",
        silent.len(),
        silent.join("\n")
    );
}

/// An attribute nobody has heard of, on every element of every part, changes the model or the
/// report. Taking away what is there cannot show this: a reader that never says what is left
/// over passes that, as the mutation of `read_cm_author` showed.
#[test]
fn an_unknown_attribute_on_any_element_is_noticed() {
    let cases: [(&str, &str, PartReader); 8] = [
        (
            "generated/comments-legacy.pptx",
            "ppt/commentAuthors.xml",
            read_cm_authors,
        ),
        (
            "generated/comments-legacy.pptx",
            "ppt/comments/comment1.xml",
            read_comments,
        ),
        (
            "generated/comments-modern.pptx",
            "ppt/authors.xml",
            read_authors,
        ),
        (
            "generated/comments-modern.pptx",
            "ppt/comments/modernComment_100_0.xml",
            read_threaded,
        ),
        (
            "generated/custom-tags-labels.pptx",
            "ppt/tags/tag1.xml",
            read_tags,
        ),
        (
            "generated/custom-tags-labels.pptx",
            "customXml/itemProps1.xml",
            read_store,
        ),
        (
            "generated/custom-tags-labels.pptx",
            "customXml/item2.xml",
            read_library,
        ),
        (
            "generated/custom-tags-labels.pptx",
            "docMetadata/LabelInfo.xml",
            read_labels,
        ),
    ];
    let mut silent = Vec::new();
    for (deck, part, read) in cases {
        let xml = part_of(deck, part);
        let whole = XmlTree::parse(&xml, "p", &OoxmlLimits::default()).unwrap();
        let as_read = read(&whole).unwrap();
        let mut stack = vec![whole.root()];
        while let Some(el) = stack.pop() {
            stack.extend(whole.child_elements(el));
            let mut marked = whole.clone();
            assert!(marked.set_attribute(el, "madeUp", "QzMarkerQz"));
            let again = XmlTree::parse(&marked.to_bytes(), "p", &OoxmlLimits::default()).unwrap();
            let got = read(&again).unwrap();
            if got == as_read {
                silent.push(format!("{part}: <{}>", whole.qualified_name(el)));
            }
            assert!(!got.1.iter().any(|l| l.contains("QzMarkerQz")), "{part}");
        }
    }
    silent.sort();
    silent.dedup();
    assert!(
        silent.is_empty(),
        "{} read past:\n{}",
        silent.len(),
        silent.join("\n")
    );
}

/// The check itself: something this reader has never heard of is noticed. A retained child
/// taken away changes the report, which is what would miss a real drop.
#[test]
fn something_unknown_in_a_comment_is_reported_and_so_is_its_absence() {
    let xml = |extra: &str| {
        format!(
            r#"<p:cmLst xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><p:cm authorId="1" idx="1"{extra}><p:madeUp/></p:cm></p:cmLst>"#
        )
    };
    let plain = XmlTree::parse(xml("").as_bytes(), "p", &OoxmlLimits::default()).unwrap();
    let got = read_comments(&plain).unwrap();
    assert!(
        got.1.iter().any(|l| l.starts_with("CS-IMF-0010")),
        "{:?}",
        got.1
    );
    assert!(silent_in(xml("").as_bytes(), read_comments, None).is_empty());
    let unknown = XmlTree::parse(
        xml(r#" madeUp="1""#).as_bytes(),
        "p",
        &OoxmlLimits::default(),
    )
    .unwrap();
    assert!(
        read_comments(&unknown)
            .unwrap()
            .1
            .iter()
            .any(|l| l.starts_with("CS-IMF-0011"))
    );
    assert!(silent_in(xml(r#" madeUp="1""#).as_bytes(), read_comments, None).is_empty());
}

#[test]
fn taking_a_retained_part_away_changes_the_report() {
    let source = read_fixture("generated/review-every-change.pptx");
    let (_, _, whole) = review_of(source.clone());
    let mut pkg = open(source);
    let names = pkg.part_names();
    let name = "ppt/_rels/presentation.xml.rels";
    assert!(names.iter().any(|n| n == name));
    let rels = String::from_utf8(pkg.read_part(name).unwrap()).unwrap();
    let cut: String = rels
        .split_inclusive("/>")
        .filter(|r| !r.contains("revisionInfo"))
        .collect();
    let (_, _, gone) = review_of(rezip(
        &read_fixture("generated/review-every-change.pptx"),
        name,
        cut.as_bytes(),
    ));
    assert_ne!(whole.retained, gone.retained);
    assert_ne!(codes(&whole.findings), codes(&gone.findings));
}

// ------------------------------------------------------------------- the five areas

fn rows() -> Vec<BTreeMap<String, String>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/checklist.tsv");
    let text = std::fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().unwrap().split('\t').collect();
    lines
        .map(|l| {
            head.iter()
                .map(|h| (*h).to_owned())
                .zip(l.split('\t').map(str::to_owned))
                .collect()
        })
        .filter(|r: &BTreeMap<String, String>| {
            ["COMMENT", "EXT", "REVIEW", "DESIGN", "CUSTOM"].contains(&r["area"].as_str())
        })
        .collect()
}

/// Blanks with their reason in `docs/spec/open-rows.tsv`, each naming a tracker row.
fn open_rows() -> std::collections::BTreeSet<(String, char)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec/open-rows.tsv");
    let text = std::fs::read_to_string(path).unwrap();
    let mut out = std::collections::BTreeSet::new();
    for line in text.lines().skip(1) {
        let mut cols = line.split('\t');
        let (Some(id), Some(columns)) = (cols.next(), cols.next()) else {
            continue;
        };
        out.extend(columns.chars().map(|c| (id.to_owned(), c)));
    }
    out
}

/// Every row of `COMMENT`, `EXT`, `REVIEW`, `DESIGN` and `CUSTOM` is marked, or has a recorded
/// reason. 876 rows, as `docs/06` says. The audit's fixture walk cannot reach the moniker types a
/// list holds through `xsd:any`, so `moniker_types_are_in_the_fixtures_and_kept` answers for them.
#[test]
fn every_row_of_the_five_areas_is_accounted_for() {
    let rows = rows();
    assert_eq!(rows.len(), 876, "docs/06 says 876");
    let open = open_rows();
    let mut blank = Vec::new();
    for row in &rows {
        for column in ['M', 'I', 'E'] {
            let mark = row[&column.to_string()].as_str();
            assert!(
                matches!(mark, "y" | "r" | "n/a" | "."),
                "{}: {mark}",
                row["id"]
            );
            if mark == "." && !open.contains(&(row["id"].clone(), column)) {
                blank.push(format!("{} {column}", row["id"]));
            }
        }
    }
    assert!(
        blank.is_empty(),
        "{} unaccounted:\n{}",
        blank.len(),
        blank.join("\n")
    );
    let y = rows.iter().filter(|r| r["I"] == "y").count();
    assert!(y > 100, "a tier that models this much, marked {y}");
}

/// Each moniker a change record or an anchor names, as text, in the fixtures that hold them.
#[test]
fn moniker_types_are_in_the_fixtures_and_kept() {
    let mut pkg = open(read_fixture("generated/review-every-change.pptx"));
    let bytes =
        String::from_utf8(pkg.read_part("ppt/changesInfos/changesInfo1.xml").unwrap()).unwrap();
    for needle in [
        "<pc:docMk",
        "<pc:sldMk",
        "<pc:sldMasterMk",
        "<pc:sldLayoutMk",
        "<pc:cmMk",
        "<pc:cmAuthorMk",
        "cId=",
        "sldId=",
        "authorId=",
        "idx=",
    ] {
        assert!(bytes.contains(needle), "{needle}");
    }
    let mut real = open(read_fixture("real/z14230715-ddi-ko.pptx"));
    let changes =
        String::from_utf8(real.read_part("ppt/changesInfos/changesInfo1.xml").unwrap()).unwrap();
    assert!(
        changes.contains("<pc:sldMk") && changes.contains("cId="),
        "as PowerPoint writes them"
    );
}

#[test]
fn a_report_code_is_what_its_finding_says() {
    assert_eq!(K::Review(ReviewFinding::OverBound).code(), "CS-RVW-0001");
    assert_eq!(
        K::Review(ReviewFinding::RetainedWhole).code(),
        "CS-RVW-0012"
    );
}
