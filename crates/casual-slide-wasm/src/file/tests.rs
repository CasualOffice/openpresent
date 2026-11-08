// `FILE-01`'s engine tests: File > Info's properties and Advanced Properties, each
// edited through the bridge, saved, read back and undone; and Save As.

use casual_slide_ooxml::{OoxmlLimits, PresentationKind, PresentationPackage};

use super::*;

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../fixtures/generated/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(path).unwrap()
}

fn part_xml(bytes: &[u8], name: &str) -> Option<String> {
    let mut p = PresentationPackage::open(bytes.to_vec(), OoxmlLimits::default()).unwrap();
    p.read_part(name)
        .ok()
        .map(|b| String::from_utf8(b).unwrap())
}

fn changed_parts(a: &[u8], b: &[u8]) -> Vec<String> {
    let mut pa = PresentationPackage::open(a.to_vec(), OoxmlLimits::default()).unwrap();
    let mut pb = PresentationPackage::open(b.to_vec(), OoxmlLimits::default()).unwrap();
    let mut names = pa.part_names();
    for n in pb.part_names() {
        if !names.contains(&n) {
            names.push(n);
        }
    }
    names
        .into_iter()
        .filter(|n| pa.read_part(n).ok() != pb.read_part(n).ok())
        .collect()
}

#[test]
fn info_fields_save_into_core_and_app_and_read_back() {
    let input = fixture("rich-properties.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    assert_eq!(v.property("title").unwrap().as_deref(), Some("Old & title"));
    assert!(v.set_property("title", "New <title>").unwrap());
    assert!(v.set_property("author", "Ada").unwrap());
    assert!(v.set_property("categories", "Plans").unwrap());
    assert!(v.set_property("comments", "Two\nlines").unwrap());
    assert!(v.set_property("company", "Acme").unwrap());
    assert!(v.set_property("manager", "Grace").unwrap());
    let saved = v.save().unwrap();

    let mut changed = changed_parts(&input, &saved);
    changed.sort();
    assert_eq!(changed, ["docProps/app.xml", "docProps/core.xml"]);
    let core = part_xml(&saved, "docProps/core.xml").unwrap();
    // What nobody edited stays, unknown elements included.
    for kept in [
        "<other:title>not the title</other:title>",
        "<cp:contentStatus>Draft</cp:contentStatus>",
        "<cp:revision>12</cp:revision>",
    ] {
        assert!(core.contains(kept), "{kept} kept");
    }
    let app = part_xml(&saved, "docProps/app.xml").unwrap();
    assert!(app.contains("<Words>120</Words>"), "the counts stand");

    let back = Viewer::open(saved).unwrap();
    for (key, want) in [
        ("title", "New <title>"),
        ("author", "Ada"),
        ("categories", "Plans"),
        ("comments", "Two\nlines"),
        ("company", "Acme"),
        ("manager", "Grace"),
        ("subject", "Subject"),
    ] {
        assert_eq!(back.property(key).unwrap().as_deref(), Some(want), "{key}");
    }
}

#[test]
fn a_cleared_field_loses_its_element_and_undo_gives_back_the_opened_file() {
    let input = fixture("rich-properties.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    assert!(v.set_property("subject", "").unwrap());
    let saved = v.save().unwrap();
    assert!(
        !part_xml(&saved, "docProps/core.xml")
            .unwrap()
            .contains("dc:subject")
    );
    assert_eq!(
        Viewer::open(saved).unwrap().property("subject").unwrap(),
        None
    );

    // One undo step per edit, and undone the file is the one opened, byte for byte.
    assert!(v.set_property("title", "Other").unwrap());
    assert_eq!(v.undo().unwrap(), None);
    assert_eq!(v.undo().unwrap(), None);
    assert!(!v.can_undo_redo().0);
    assert_eq!(v.save().unwrap(), input);
}

#[test]
fn a_field_given_the_value_it_has_is_no_edit() {
    let input = fixture("rich-properties.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    assert!(!v.set_property("title", "Old & title").unwrap());
    assert!(!v.set_property("company", "Co & Co").unwrap());
    assert!(!v.can_undo_redo().0);
    assert_eq!(v.save().unwrap(), input);
    assert!(v.set_property("nonsense", "x").is_err());
}

#[test]
fn tags_replace_the_keywords_and_keep_their_language() {
    let mut v = Viewer::open(fixture("rich-properties.pptx")).unwrap();
    assert_eq!(
        v.property("tags").unwrap().as_deref(),
        Some("one, two; drei")
    );
    assert!(v.set_property("tags", "alpha; beta").unwrap());
    let saved = v.save().unwrap();
    let core = part_xml(&saved, "docProps/core.xml").unwrap();
    assert!(
        core.contains("<cp:keywords xml:lang=\"en\">alpha; beta</cp:keywords>"),
        "{core}"
    );
}

#[test]
fn a_deck_without_property_parts_gets_one_only_for_what_was_set() {
    let input = fixture("minimal.pptx");
    let mut v = Viewer::open(input.clone()).unwrap();
    assert!(v.set_property("title", "Made here").unwrap());
    let saved = v.save().unwrap();
    assert!(
        part_xml(&saved, "docProps/core.xml")
            .unwrap()
            .contains("<dc:title>Made here</dc:title>")
    );
    assert!(
        part_xml(&saved, "docProps/app.xml").is_none(),
        "no app part made"
    );
    assert_eq!(
        Viewer::open(saved)
            .unwrap()
            .property("title")
            .unwrap()
            .as_deref(),
        Some("Made here")
    );
    // Set and cleared again: nothing to make.
    assert!(v.set_property("title", "").unwrap());
    assert_eq!(v.save().unwrap(), input);
}

#[test]
fn custom_properties_are_added_changed_and_removed() {
    let input = fixture("rich-properties.pptx");
    let mut v = Viewer::open(input).unwrap();
    let before = v.custom_properties().len();
    assert!(
        v.set_custom_property("Client", CustomKind::Text, "Contoso")
            .unwrap()
    );
    assert!(
        v.set_custom_property("Budget", CustomKind::Number, "12.5")
            .unwrap()
    );
    assert!(
        v.set_custom_property("Seats", CustomKind::Number, "40")
            .unwrap()
    );
    assert!(
        v.set_custom_property("Due", CustomKind::Date, "2026-12-01")
            .unwrap()
    );
    assert!(
        v.set_custom_property("Signed", CustomKind::YesNo, "Yes")
            .unwrap()
    );
    // Same name, other case: the one property changes.
    assert!(
        v.set_custom_property("client", CustomKind::Text, "Fabrikam")
            .unwrap()
    );
    assert!(v.remove_custom_property("LPWSTR").unwrap());
    assert!(
        v.set_custom_property("Bad", CustomKind::Number, "twelve")
            .is_err()
    );
    assert!(
        v.set_custom_property("Bad", CustomKind::Date, "2026-02-30")
            .is_err()
    );
    assert!(v.set_custom_property(" ", CustomKind::Text, "x").is_err());
    let saved = v.save().unwrap();

    let custom = part_xml(&saved, "docProps/custom.xml").unwrap();
    assert!(
        custom.contains("<vt:lpwstr>Fabrikam</vt:lpwstr>"),
        "{custom}"
    );
    assert!(custom.contains("<vt:r8>12.5</vt:r8>"));
    assert!(custom.contains("<vt:i4>40</vt:i4>"));
    assert!(custom.contains("<vt:filetime>2026-12-01T00:00:00Z</vt:filetime>"));
    assert!(!custom.contains("name=\"lpwstr\""));

    let back = Viewer::open(saved).unwrap();
    let list = back.custom_properties();
    assert_eq!(list.len(), before + 5 - 1);
    assert!(list.contains(&("Client".to_owned(), "text", "Fabrikam".to_owned())));
    assert!(list.contains(&("Due".to_owned(), "date", "2026-12-01".to_owned())));
    assert!(list.contains(&("Signed".to_owned(), "yesno", "Yes".to_owned())));
    // Each pid is its own.
    let all = &back.deck.properties().custom.as_ref().unwrap().properties;
    let pids: std::collections::BTreeSet<i32> = all.iter().map(|p| p.pid).collect();
    assert_eq!(pids.len(), all.len());
}

#[test]
fn document_info_reports_the_fields_and_counts() {
    let v = Viewer::open(fixture("rich-properties.pptx")).unwrap();
    let info: serde_json::Value = serde_json::from_str(&v.document_info().unwrap()).unwrap();
    assert_eq!(info["title"], "Old & title");
    assert_eq!(info["created"], "2020-01-02T03:04:05Z");
    assert_eq!(info["lastModifiedBy"], "Editor");
    assert_eq!(info["format"], "pptx");
    assert_eq!(info["slides"], v.slide_count());
    assert!(info["custom"].as_array().unwrap().len() > 3);
    let words = Viewer::open(fixture("area-text.pptx"))
        .unwrap()
        .word_count();
    assert!(words > 0);
}

#[test]
fn save_as_retypes_through_the_grid() {
    let v = Viewer::open(fixture("minimal.pptx")).unwrap();
    for (ext, kind) in [
        ("potx", PresentationKind::Template),
        ("ppsx", PresentationKind::Slideshow),
        ("pptx", PresentationKind::Presentation),
    ] {
        let bytes = v.save_as(ext, false).unwrap();
        let p = PresentationPackage::open(bytes, OoxmlLimits::default()).unwrap();
        assert_eq!(p.kind(), kind, "{ext}");
    }
    assert!(v.save_as("pdf", false).is_err());
}

#[test]
fn values_past_the_bounds_are_cut_and_characters_xml_cannot_hold_are_dropped() {
    let mut v = Viewer::open(fixture("minimal.pptx")).unwrap();
    let long = "x".repeat(MAX_PROPERTY_CHARS + 10);
    assert!(v.set_property("title", &format!("a\u{1}b{long}")).unwrap());
    let title = v.property("title").unwrap().unwrap();
    assert_eq!(title.chars().count(), MAX_PROPERTY_CHARS);
    assert!(title.starts_with("abx"));
    let name = "n".repeat(MAX_CUSTOM_NAME_CHARS + 10);
    assert!(v.set_custom_property(&name, CustomKind::Text, "v").unwrap());
    assert_eq!(
        v.custom_properties()[0].0.chars().count(),
        MAX_CUSTOM_NAME_CHARS
    );
    // What is cut still saves and reads back.
    let saved = v.save().unwrap();
    assert_eq!(
        Viewer::open(saved).unwrap().property("title").unwrap(),
        Some(title)
    );
}
