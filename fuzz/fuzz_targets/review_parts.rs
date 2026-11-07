#![no_main]
//! Any bytes as a part of Tier 1M: both comment forms, both author lists, tags, datastore
//! properties, a schema library, label metadata, a slide's zooms and marks, and a section list.
//! Import never panics; a patch of an unchanged model changes nothing; an edit the exporter
//! accepts reads back as it was asked.

use casual_slide_export::comment::{patch_comments, patch_threaded_comments};
use casual_slide_export::custom_data::patch_tags;
use casual_slide_export::section::patch_part_marks;
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::comment::{
    import_authors, import_comment_authors, import_comments, import_threaded_comments,
    ReviewLimits,
};
use casual_slide_import::custom_data::{
    import_datastore_item, import_label_info, import_schema_library, import_tags,
};
use casual_slide_import::presentation::import_presentation;
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_import::section::import_part_marks;
use casual_slide_ooxml::{OoxmlLimits, XmlTree};
use libfuzzer_sys::fuzz_target;

const PART: &str = "p";

fuzz_target!(|data: &[u8]| {
    let xml = OoxmlLimits::default();
    let Ok(tree) = XmlTree::parse(data, PART, &xml) else {
        return;
    };
    let limits = ReviewLimits::default();
    let import = ImportLimits::default();
    let quiet = || Findings::new(import.max_findings);

    let _ = import_comment_authors(&tree, PART, &limits, &import, &mut quiet());
    let _ = import_authors(&tree, PART, &limits, &import, &mut quiet());
    let _ = import_datastore_item(&tree, PART, &limits, &import, &mut quiet());
    let _ = import_schema_library(&tree, PART, &limits, &import, &mut quiet());
    let _ = import_label_info(&tree, PART, &limits, &import, &mut quiet());
    let _ = import_presentation(&tree, PART, &[], &import, &mut quiet());

    if let Ok(mut list) = import_comments(&tree, PART, &limits, &import, &mut quiet()) {
        let mut same = tree.clone();
        assert_eq!(
            patch_comments(&mut same, PART, false, &list, &limits, &import, &xml).ok(),
            Some(0)
        );
        if let Some(first) = list.comments.first_mut() {
            first.text = Some("edit".to_owned());
            let mut edited = tree.clone();
            if patch_comments(&mut edited, PART, false, &list, &limits, &import, &xml).is_ok() {
                let again = XmlTree::parse(&edited.to_bytes(), PART, &xml).unwrap();
                let read = import_comments(&again, PART, &limits, &import, &mut quiet()).unwrap();
                assert_eq!(read, list);
            }
        }
    }
    if let Ok(list) = import_threaded_comments(
        &tree,
        PART,
        &[],
        &limits,
        &import,
        &mut quiet(),
        &mut ColorFindings::default(),
    ) {
        let mut same = tree.clone();
        assert_eq!(
            patch_threaded_comments(&mut same, PART, false, &[], &list, &limits, &import, &xml)
                .ok(),
            Some(0)
        );
        for comment in &list.comments {
            let _ = comment.mentions();
        }
    }
    if let Ok(list) = import_tags(&tree, PART, &limits, &import, &mut quiet()) {
        let mut same = tree.clone();
        assert_eq!(
            patch_tags(&mut same, PART, false, &list, &limits, &import, &xml).ok(),
            Some(0)
        );
    }
    if let Ok(marks) = import_part_marks(&tree, PART, &[], &limits, &import, &mut quiet()) {
        let mut same = tree.clone();
        assert_eq!(
            patch_part_marks(&mut same, PART, false, &[], &marks, &limits, &import, &xml).ok(),
            Some(0)
        );
    }
});
