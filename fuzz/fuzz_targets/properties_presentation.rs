#![no_main]
//! Any bytes as any Tier 1A or 1B part: import never panics; a patch never panics, and
//! what it returns as done reads back as it was asked, and an unchanged model changes
//! nothing.

use casual_slide_export::presentation::{
    patch_presentation, patch_presentation_properties, patch_view_properties,
};
use casual_slide_export::properties::{
    patch_app_properties, patch_core_properties, patch_custom_properties,
};
use casual_slide_import::presentation::{
    import_presentation, import_presentation_properties, import_view_properties,
};
use casual_slide_import::properties::{
    import_app_properties, import_core_properties, import_custom_properties, Findings,
    ImportLimits,
};
use casual_slide_ooxml::{OoxmlLimits, XmlTree};
use libfuzzer_sys::fuzz_target;

const PART: &str = "part";

fuzz_target!(|data: &[u8]| {
    let xml = OoxmlLimits::default();
    let limits = ImportLimits::default();
    let Ok(tree) = XmlTree::parse(data, PART, &xml) else {
        return;
    };
    let fresh = || Findings::new(limits.max_findings);

    macro_rules! exercise {
        ($import:expr, $patch:expr, $edit:expr) => {
            if let Ok(before) = $import(&tree, &mut fresh()) {
                let mut same = tree.clone();
                let changes = $patch(&mut same, &before).unwrap();
                assert_eq!(changes, 0, "an unchanged model changed something");
                assert_eq!(same.to_bytes(), tree.to_bytes());

                let mut wanted = before.clone();
                let edit: fn(&mut _) = $edit;
                edit(&mut wanted);
                let mut edited = tree.clone();
                if $patch(&mut edited, &wanted).is_ok() {
                    let bytes = edited.to_bytes();
                    let again = XmlTree::parse(&bytes, PART, &xml).unwrap();
                    assert_eq!($import(&again, &mut fresh()).unwrap(), wanted);
                }
            }
        };
    }

    exercise!(
        |t: &XmlTree, f: &mut Findings| import_core_properties(t, PART, &limits, f),
        |t: &mut XmlTree, m: &_| patch_core_properties(t, PART, false, m, &limits),
        |m: &mut casual_slide_model::properties::CoreProperties| {
            m.title = Some("edited".to_owned());
            m.subject = None;
        }
    );
    exercise!(
        |t: &XmlTree, f: &mut Findings| import_app_properties(t, PART, &limits, f),
        |t: &mut XmlTree, m: &_| patch_app_properties(t, PART, false, m, &limits),
        |m: &mut casual_slide_model::properties::AppProperties| {
            m.slides = Some(1);
            m.titles_of_parts = None;
        }
    );
    exercise!(
        |t: &XmlTree, f: &mut Findings| import_custom_properties(t, PART, &limits, f),
        |t: &mut XmlTree, m: &_| patch_custom_properties(t, PART, false, m, &limits),
        |m: &mut casual_slide_model::properties::CustomProperties| {
            m.properties.reverse();
            m.properties.pop();
        }
    );
    exercise!(
        |t: &XmlTree, f: &mut Findings| import_presentation(t, PART, &[], &limits, f),
        |t: &mut XmlTree, m: &_| patch_presentation(t, PART, false, &[], m, &limits),
        |m: &mut casual_slide_model::presentation::Presentation| {
            m.first_slide_num = Some(2);
            if let Some(s) = m.slides.as_mut() {
                s.reverse();
                s.pop();
            }
            m.kinsoku = None;
        }
    );
    exercise!(
        |t: &XmlTree, f: &mut Findings| import_presentation_properties(t, PART, &[], &limits, f),
        |t: &mut XmlTree, m: &_| patch_presentation_properties(t, PART, false, &[], m, &limits),
        |m: &mut casual_slide_model::presentation::PresentationProperties| {
            m.print = Some(Default::default());
            m.show = None;
        }
    );
    exercise!(
        |t: &XmlTree, f: &mut Findings| import_view_properties(t, PART, &[], &limits, f),
        |t: &mut XmlTree, m: &_| patch_view_properties(t, PART, false, &[], m, &limits),
        |m: &mut casual_slide_model::presentation::ViewProperties| {
            m.show_comments = Some(true);
            m.outline = None;
        }
    );
});
