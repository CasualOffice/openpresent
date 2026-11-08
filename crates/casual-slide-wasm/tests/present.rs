//! Presenting (`VS-07`): what the slide show reads from the deck. Hidden slides,
//! each slide's transition, and the speaker notes Presenter View shows.

use casual_slide_package::{Package, PackageLimits, PartEdits};
use casual_slide_wasm::Viewer;

fn fixture(name: &str) -> Vec<u8> {
    let path = format!(
        "{}/../../fixtures/generated/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(path).unwrap()
}

/// `rr-notes.pptx` with one of its slide parts changed by `edit`.
fn edited(part: &str, edit: impl Fn(String) -> String) -> Vec<u8> {
    let bytes = fixture("rr-notes.pptx");
    let mut package = Package::open(bytes, PackageLimits::default()).unwrap();
    let xml = String::from_utf8(package.read_part(part).unwrap()).unwrap();
    let mut edits = PartEdits::default();
    edits.replace(part, edit(xml).into_bytes());
    package.rewrite(&edits).unwrap()
}

/// `rr-notes.pptx` with slide 2 marked `show="0"`.
fn with_hidden_second_slide() -> Vec<u8> {
    edited("ppt/slides/slide2.xml", |xml| {
        assert!(
            !xml.contains(" show="),
            "the fixture states no show attribute"
        );
        xml.replacen("<p:sld ", "<p:sld show=\"0\" ", 1)
    })
}

#[test]
fn a_slide_marked_show_0_is_hidden_and_the_others_are_not() {
    let v = Viewer::open(with_hidden_second_slide()).unwrap();
    assert!(!v.show_slide(0).unwrap().hidden);
    assert!(v.show_slide(1).unwrap().hidden);
}

#[test]
fn a_slide_without_a_show_attribute_is_shown() {
    let v = Viewer::open(fixture("rr-notes.pptx")).unwrap();
    for i in 0..v.slide_count() {
        assert!(!v.show_slide(i).unwrap().hidden, "slide {i}");
    }
}

#[test]
fn notes_are_the_notes_body_text_and_not_the_slide_image_label() {
    let v = Viewer::open(fixture("rr-notes.pptx")).unwrap();
    assert_eq!(v.notes_text(0).unwrap(), "Notes");
    // Slide 2 has no notes page.
    assert_eq!(v.notes_text(1).unwrap(), "");
}

#[test]
fn notes_of_a_slide_that_does_not_exist_is_an_error() {
    let v = Viewer::open(fixture("rr-notes.pptx")).unwrap();
    assert_eq!(v.notes_text(99).unwrap_err().code(), "CS-RND-0003");
    assert_eq!(v.show_slide(99).unwrap_err().code(), "CS-RND-0003");
}

#[test]
fn a_slide_without_a_transition_has_none() {
    let v = Viewer::open(fixture("rr-notes.pptx")).unwrap();
    assert_eq!(v.show_slide(0).unwrap().transition, None);
}

#[test]
fn transitions_carry_their_effect_and_the_speed_as_a_duration() {
    let v = Viewer::open(fixture("area-trans.pptx")).unwrap();
    // `<p:transition spd="slow"><p:circle/>`: played as a stand-in fade, a second long.
    let t = v.show_slide(0).unwrap().transition.unwrap();
    assert_eq!(t.effect, "circle");
    assert_eq!(t.duration_ms, 1000);
    assert!(!t.played_as_is);
    // `spd="med"`, dissolve.
    let t = v.show_slide(1).unwrap().transition.unwrap();
    assert_eq!((t.effect, t.duration_ms), ("dissolve", 750));
    // `spd="fast"`, blinds.
    let t = v.show_slide(8).unwrap().transition.unwrap();
    assert_eq!((t.effect, t.duration_ms), ("blinds", 500));
}

#[test]
fn a_fade_is_played_as_it_is_and_a_cut_is_instant() {
    let v = Viewer::open(fixture("area-trans.pptx")).unwrap();
    let all: Vec<_> = (0..v.slide_count())
        .filter_map(|i| v.show_slide(i).unwrap().transition)
        .collect();
    let fade = all.iter().find(|t| t.effect == "fade").expect("a fade");
    assert!(fade.played_as_is);
    assert!(fade.duration_ms > 0);
    let cut = all.iter().find(|t| t.effect == "cut").expect("a cut");
    assert!(cut.played_as_is);
    assert_eq!(cut.duration_ms, 0);
}

#[test]
fn a_p14_duration_overrides_the_speed() {
    let bytes = edited("ppt/slides/slide1.xml", |xml| {
        xml.replacen(
            "</p:cSld>",
            "</p:cSld><p:transition xmlns:p14=\"http://schemas.microsoft.com/office/powerpoint/2010/main\" spd=\"fast\" p14:dur=\"1250\"><p:push dir=\"u\"/></p:transition>",
            1,
        )
    });
    let v = Viewer::open(bytes).unwrap();
    let t = v.show_slide(0).unwrap().transition.unwrap();
    assert_eq!(
        (t.effect, t.duration_ms, t.played_as_is),
        ("push", 1250, false)
    );
}

#[test]
fn an_alternate_transition_plays_the_branch_the_deck_resolves_to() {
    let v = Viewer::open(fixture("transition-every-effect.pptx")).unwrap();
    // Slide 9: `p14:ferris` in a `Choice`, a slow fade in the `Fallback`. The
    // renderer resolves to the fallback, and the show follows it.
    let t = v.show_slide(8).unwrap().transition.unwrap();
    assert_eq!(
        (t.effect, t.duration_ms, t.played_as_is),
        ("fade", 1000, true)
    );
}

#[test]
fn notes_past_the_bound_are_cut_and_marked() {
    let long = "n".repeat(casual_slide_wasm::MAX_NOTES_CHARS + 10);
    let bytes = edited("ppt/notesSlides/notesSlide1.xml", |xml| {
        xml.replacen("<a:t>Notes</a:t>", &format!("<a:t>{long}</a:t>"), 1)
    });
    let notes = Viewer::open(bytes).unwrap().notes_text(0).unwrap();
    assert_eq!(
        notes.chars().count(),
        casual_slide_wasm::MAX_NOTES_CHARS + 1
    );
    assert!(notes.ends_with('\u{2026}'));
}
