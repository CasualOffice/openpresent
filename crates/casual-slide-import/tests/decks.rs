//! Import of generated and third-party decks, value by value.

use std::path::{Path, PathBuf};

use casual_slide_import::presentation::{ImportedDeck, import_deck};
use casual_slide_import::properties::{ImportFindingKind as K, ImportLimits};
use casual_slide_model::presentation::{
    ConformanceClass, Coordinate, Direction, Percentage, PrintColorMode, PrintWhat, ShowType,
    SlideId, SlideListChoice, SlideMasterId, SlideSizeType, SplitterBarState, ViewType,
    WebColorType, WebScreenSize,
};
use casual_slide_model::properties::{Float64, Variant, VectorBaseType, VtVector};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn import(path: &str) -> ImportedDeck {
    let bytes = std::fs::read(fixtures().join(path)).unwrap();
    let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
    import_deck(&mut pkg, &ImportLimits::default()).unwrap()
}

#[test]
fn a_minimal_deck_materializes_no_default() {
    let d = import("generated/minimal.pptx");
    let p = &d.deck.presentation;
    assert_eq!(
        p.first_slide_num, None,
        "schema default is 1, file says nothing"
    );
    assert_eq!(p.show_special_pls_on_title_sld, None);
    assert_eq!(p.server_zoom, None);
    assert_eq!(p.auto_compress_pictures, None);
    assert_eq!(p.bookmark_id_seed, None);
    assert_eq!(p.notes_masters, None);
    assert_eq!(p.custom_shows, None);
    let size = p.slide_size.unwrap();
    assert_eq!(
        (size.cx, size.cy, size.size_type),
        (12_192_000, 6_858_000, None)
    );
    assert_eq!(p.slides.as_ref().unwrap().len(), 1);
    assert_eq!(d.deck.properties.core, None);
    assert_eq!(d.deck.properties.app, None);
    assert_eq!(d.deck.properties.custom, None);
    assert_eq!(d.deck.presentation_properties, None);
    assert_eq!(d.deck.view_properties, None);
    assert!(d.findings.is_empty(), "{:?}", d.findings);
}

#[test]
fn a_libreoffice_deck_reads_as_written() {
    let d = import("producers/libreoffice-26.8-image.pptx");
    let core = d.deck.properties.core.as_ref().unwrap();
    assert_eq!(
        core.title.as_deref(),
        Some(""),
        "present and empty is not absent"
    );
    assert_eq!(core.language.as_deref(), Some("en-IN"));
    assert_eq!(core.revision.as_deref(), Some("0"));
    assert_eq!(core.created, None);
    let app = d.deck.properties.app.as_ref().unwrap();
    assert_eq!(app.template.as_deref(), Some(""));
    assert_eq!(app.total_time, Some(0));
    assert_eq!(app.app_version.as_deref(), Some("15.0000"));
    assert_eq!(app.slides, None);
    let p = &d.deck.presentation;
    assert_eq!(
        p.slide_masters.as_ref().unwrap()[0],
        SlideMasterId {
            id: Some(2_147_483_648),
            rel_id: "rId2".to_owned()
        }
    );
    assert_eq!(
        p.slides.as_ref().unwrap()[0],
        SlideId {
            id: 256,
            rel_id: "rId3".to_owned()
        }
    );
    assert_eq!(p.notes_size.unwrap().cy, 10_691_813);
    let show = d.deck.presentation_properties.unwrap().show.unwrap();
    assert_eq!(show.show_narration, Some(true));
    assert_eq!(show.loop_show, None);
    assert_eq!(d.deck.view_properties, None);
    let three = import("producers/libreoffice-26.8-three-slides.pptx");
    assert_eq!(three.deck.presentation.slides.unwrap().len(), 3);
}

#[test]
fn every_field_of_the_presentation_part_is_read() {
    let d = import("generated/rich-properties.pptx");
    let p = &d.deck.presentation;
    assert_eq!(p.server_zoom, Some(Percentage(80_000)));
    assert_eq!(p.first_slide_num, Some(5));
    assert_eq!(p.show_special_pls_on_title_sld, Some(false));
    assert_eq!(p.rtl, Some(false));
    assert_eq!(p.remove_personal_info_on_save, Some(true));
    assert_eq!(p.compat_mode, Some(false));
    assert_eq!(p.strict_first_and_last_chars, Some(true));
    assert_eq!(p.embed_true_type_fonts, Some(true));
    assert_eq!(p.save_subset_fonts, Some(true));
    assert_eq!(p.auto_compress_pictures, Some(false));
    assert_eq!(p.bookmark_id_seed, Some(7));
    assert_eq!(p.conformance, Some(ConformanceClass::Transitional));
    assert_eq!(p.notes_masters.as_deref(), Some(&["rId11".to_owned()][..]));
    assert_eq!(
        p.handout_masters.as_deref(),
        Some(&["rId12".to_owned()][..])
    );
    // `r:id` first in the file: `id` must not find it.
    assert_eq!(
        p.slides.as_ref().unwrap()[0],
        SlideId {
            id: 256,
            rel_id: "rId3".to_owned()
        }
    );
    assert_eq!(p.slides.as_ref().unwrap().len(), 3);
    assert_eq!(
        p.slide_size.unwrap().size_type,
        Some(SlideSizeType::Screen16x9)
    );
    assert_eq!(p.notes_size.unwrap().cx, 6_858_000);
    assert_eq!(p.smart_tags.as_deref(), Some("rId8"));
    let font = &p.embedded_fonts.as_ref().unwrap()[0];
    assert_eq!(font.font.typeface, "Fixture Sans");
    assert_eq!(font.font.panose.as_deref(), Some("020B0604020202020204"));
    assert_eq!(font.font.pitch_family, Some(34));
    assert_eq!(font.font.charset, Some(0));
    assert_eq!(font.regular.as_deref(), Some("rId9"));
    assert_eq!(font.bold, None);
    let show = &p.custom_shows.as_ref().unwrap()[0];
    assert_eq!(show.name, "Short & sweet");
    assert_eq!(
        (show.id, show.slides.as_slice()),
        (3, &["rId5".to_owned(), "rId3".to_owned()][..])
    );
    let album = p.photo_album.unwrap();
    assert_eq!(album.bw, Some(true));
    assert_eq!(album.show_captions, Some(false));
    let data = p.customer_data.as_ref().unwrap();
    assert_eq!(data.customer_data, ["rId10"]);
    assert_eq!(data.tags.as_deref(), Some("rId8"));
    let kinsoku = p.kinsoku.as_ref().unwrap();
    assert_eq!(kinsoku.lang.as_deref(), Some("ja-JP"));
    assert_eq!(kinsoku.invalid_start_chars, "!)");
    let verifier = p.modify_verifier.as_ref().unwrap();
    assert_eq!(verifier.spin_count, Some(100_000));
    assert_eq!(verifier.hash_data.as_deref(), Some("BBBB"));

    // Preserved, not modelled, and said so.
    for element in ["extLst", "clrMru", "penClr"] {
        assert!(d.findings.has(K::RetainedElement, element), "{element}");
    }
    assert!(d.findings.has(K::RetainedAttribute, "presentation"));
    let relationship_trouble = d.findings.iter().filter(|f| {
        matches!(
            f.kind,
            K::DanglingRelationship | K::WrongRelationship | K::MalformedValue | K::OutOfRange
        )
    });
    assert_eq!(relationship_trouble.count(), 0, "{:?}", d.findings);
}

#[test]
fn every_field_of_show_print_and_view_state_is_read() {
    let d = import("generated/rich-properties.pptx");
    let props = d.deck.presentation_properties.as_ref().unwrap();
    let html = props.html_publish.as_ref().unwrap();
    assert_eq!(html.show_speaker_notes, Some(false));
    assert_eq!(html.rel_id.as_deref(), Some("rId1"));
    assert_eq!(html.slide_list, Some(SlideListChoice::All));
    let web = props.web.as_ref().unwrap();
    assert_eq!(web.image_size, Some(WebScreenSize::S1024x768));
    assert_eq!(web.color, Some(WebColorType::Browser));
    assert_eq!(web.encoding.as_deref(), Some("utf-8"));
    let print = props.print.unwrap();
    assert_eq!(print.print_what, Some(PrintWhat::Handouts6));
    assert_eq!(print.color_mode, Some(PrintColorMode::Gray));
    assert_eq!(print.frame_slides, Some(true));
    let show = props.show.unwrap();
    assert_eq!(show.loop_show, Some(true));
    assert_eq!(show.use_timings, Some(false));
    assert_eq!(
        show.show_type,
        Some(ShowType::Kiosk {
            restart: Some(60_000)
        })
    );
    assert_eq!(show.slide_list, Some(SlideListChoice::CustomShow(3)));

    let v = d.deck.view_properties.as_ref().unwrap();
    assert_eq!(v.last_view, Some(ViewType::SldThumbnailView));
    assert_eq!(v.show_comments, Some(false));
    let normal = v.normal.unwrap();
    assert_eq!(normal.vert_bar_state, Some(SplitterBarState::Minimized));
    assert_eq!(normal.restored_left.unwrap().size, Percentage(15_620));
    assert_eq!(normal.restored_left.unwrap().auto_adjust, Some(false));
    assert_eq!(normal.restored_top.unwrap().auto_adjust, None);
    let slide = v.slide.as_ref().unwrap();
    assert_eq!(slide.snap_to_objects, Some(true));
    let common = slide.common.unwrap();
    assert_eq!(common.var_scale, Some(true));
    assert_eq!(common.scale.unwrap().x.n, 88);
    assert_eq!(common.origin.unwrap().x, Coordinate(-1104));
    let guides = slide.guides.as_ref().unwrap();
    assert_eq!(guides[0].orient, Some(Direction::Horz));
    assert_eq!(
        guides[1].orient, None,
        "schema default is vert, file says nothing"
    );
    assert_eq!(guides[1].pos, Some(Coordinate(3840)));
    let outline = v.outline.as_ref().unwrap();
    assert_eq!(outline.slides.as_ref().unwrap()[0].collapse, Some(true));
    assert!(v.notes_text.is_some());
    assert_eq!(v.sorter.unwrap().show_formatting, Some(false));
    assert!(v.notes.is_some());
    assert_eq!(v.grid_spacing.unwrap().cx, 76_200);
}

#[test]
fn every_property_part_is_read() {
    let d = import("generated/rich-properties.pptx");
    let core = d.deck.properties.core.as_ref().unwrap();
    assert_eq!(
        core.title.as_deref(),
        Some("Old & title"),
        "`other:title` is not `dc:title`"
    );
    assert_eq!(core.subject.as_deref(), Some("Subject"));
    assert_eq!(core.creator.as_deref(), Some("Author"));
    assert_eq!(core.description.as_deref(), Some("Words"));
    assert_eq!(core.identifier.as_deref(), Some("urn:x"));
    assert_eq!(core.language.as_deref(), Some("en-GB"));
    assert_eq!(core.last_modified_by.as_deref(), Some("Editor"));
    assert_eq!(core.revision.as_deref(), Some("12"));
    assert_eq!(core.category.as_deref(), Some("Cat"));
    assert_eq!(core.content_status.as_deref(), Some("Draft"));
    assert_eq!(core.version.as_deref(), Some("1.2"));
    assert_eq!(
        core.created.as_ref().unwrap().as_str(),
        "2020-01-02T03:04:05Z"
    );
    assert_eq!(
        core.modified.as_ref().unwrap().as_str(),
        "2021-02-03T04:05:06.5+05:30"
    );
    assert_eq!(
        core.last_printed.as_ref().unwrap().as_str(),
        "2019-12-31T23:59:59Z"
    );
    let keywords = core.keywords.as_ref().unwrap();
    assert_eq!(keywords.lang.as_deref(), Some("en"));
    assert_eq!(keywords.text, "one, two");
    assert_eq!(keywords.values[0].lang.as_deref(), Some("de"));
    assert_eq!(keywords.values[0].text, "drei");
    assert!(d.findings.has(K::RetainedElement, "title"));

    let app = d.deck.properties.app.as_ref().unwrap();
    assert_eq!(app.company.as_deref(), Some("Co & Co"));
    assert_eq!(app.manager.as_deref(), Some("M"));
    assert_eq!(app.template.as_deref(), Some("T.potx"));
    assert_eq!(
        (
            app.pages,
            app.words,
            app.characters,
            app.lines,
            app.paragraphs
        ),
        (Some(0), Some(120), Some(5), Some(2), Some(9))
    );
    assert_eq!(
        (
            app.slides,
            app.notes,
            app.total_time,
            app.hidden_slides,
            app.mm_clips
        ),
        (Some(3), Some(1), Some(14), Some(0), Some(0))
    );
    assert_eq!(app.characters_with_spaces, Some(6));
    assert_eq!(app.doc_security, Some(0));
    assert_eq!(app.scale_crop, Some(false));
    assert_eq!(app.links_up_to_date, Some(false));
    assert_eq!(app.shared_doc, Some(false));
    assert_eq!(app.hyperlinks_changed, Some(false));
    assert_eq!(app.hyperlink_base.as_deref(), Some("https://example.com/"));
    assert_eq!(app.presentation_format.as_deref(), Some("Widescreen"));
    assert_eq!(app.application.as_deref(), Some("Fixture"));
    assert_eq!(app.app_version.as_deref(), Some("16.0000"));
    assert_eq!(app.dig_sig.as_deref(), Some("QUJD"));
    assert_eq!(
        app.titles_of_parts,
        Some(VtVector {
            base_type: VectorBaseType::Lpstr,
            items: vec![Variant::Lpstr("One".to_owned())]
        })
    );
    let pairs = app.heading_pairs.as_ref().unwrap();
    assert_eq!(pairs.base_type, VectorBaseType::Variant);
    assert_eq!(pairs.items[1], Variant::Variant(Box::new(Variant::I4(1))));
    assert_eq!(app.hlinks.as_ref().unwrap().items.len(), 1);

    let custom = d.deck.properties.custom.as_ref().unwrap();
    assert_eq!(custom.properties.len(), 33, "one of every variant form");
    assert!(
        custom
            .properties
            .iter()
            .all(|p| p.value.is_some() && p.pid >= 2)
    );
    let by_name = |n: &str| {
        custom
            .properties
            .iter()
            .find(|p| p.name.as_deref() == Some(n))
            .and_then(|p| p.value.clone())
            .unwrap()
    };
    assert_eq!(by_name("lpwstr"), Variant::Lpwstr("text".to_owned()));
    assert_eq!(by_name("i8"), Variant::I8(-8));
    assert_eq!(by_name("r8"), Variant::R8(Float64::new(f64::NAN)));
    assert_eq!(by_name("bool"), Variant::Bool(true));
    assert_eq!(by_name("cy"), Variant::Cy("1.2500".to_owned()));
    assert_eq!(by_name("empty"), Variant::Empty);
    assert_eq!(by_name("null"), Variant::Null);
    assert!(matches!(
        by_name("vstream"),
        Variant::Vstream {
            version: Some(_),
            ..
        }
    ));
    assert!(matches!(by_name("vec"), Variant::Vector(v) if v.items.len() == 2));
    assert!(matches!(by_name("arr"), Variant::Array(a) if a.upper_bound == 1));
}

#[test]
fn strict_namespaces_read_the_same_values() {
    let plain = import("generated/rich-properties.pptx");
    let strict = import("generated/strict-properties.pptx");
    let mut wanted = plain.deck.clone();
    wanted.presentation.conformance = Some(ConformanceClass::Strict);
    assert_eq!(strict.deck, wanted);
}

#[test]
fn every_deck_that_opens_imports_or_refuses_with_a_code() {
    let manifest = std::fs::read_to_string(fixtures().join("manifest.tsv")).unwrap();
    let mut imported = 0;
    for line in manifest.lines().skip(1) {
        let mut cols = line.split('\t');
        let path = cols.next().unwrap();
        let expect = cols.nth(1).unwrap();
        if !expect.starts_with("open") || path.starts_with("hostile/import-limit") {
            continue;
        }
        let bytes = std::fs::read(fixtures().join(path)).unwrap();
        let mut pkg = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
        import_deck(&mut pkg, &ImportLimits::default()).unwrap_or_else(|e| panic!("{path}: {e}"));
        imported += 1;
    }
    assert!(imported > 20, "{imported}");
}
