//! Transitions and animations (`VS-10`): each Transitions and Animations tab edit
//! saves through the writer, opens again as the model said, is a fixed point, undoes
//! back to the opened bytes, and writes a deck F4 and the SDK validator pass.

use std::path::Path;
use std::process::Command;

use casual_slide_transaction::{EffectKind, EffectStart, TransitionKind, TransitionOption};
use casual_slide_wasm::{Insert, Viewer};

const BOX: [i64; 4] = [914_400, 914_400, 2_743_200, 914_400];

fn fixture(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated")
        .join(name);
    std::fs::read(path).unwrap()
}

/// A blank deck of two slides with a text box and a rectangle on the first.
fn base() -> (Vec<u8>, u32, u32) {
    let mut v = Viewer::blank().unwrap();
    let layouts = v.layouts();
    v.new_slide(1, layouts[1].0).unwrap();
    let text = v.insert_shape(0, Insert::TextBox, BOX).unwrap();
    let rect = v.insert_shape(0, Insert::Rectangle, BOX).unwrap();
    (v.save().unwrap(), text, rect)
}

/// The tab's kind and duration for slide `index`.
fn choice(v: &Viewer, index: usize) -> (Option<TransitionKind>, u32) {
    let c = v.transition_choice(index).unwrap();
    (c.kind, c.duration_ms)
}

fn kinds(v: &Viewer, index: usize) -> Vec<(Option<EffectKind>, EffectStart)> {
    v.animations(index)
        .unwrap()
        .iter()
        .map(|a| (a.entry.kind, a.entry.start))
        .collect()
}

/// Every edit the two tabs make, on a deck built and saved first.
fn animate(v: &mut Viewer, text: u32, rect: u32) {
    v.set_transition(0, TransitionKind::Push, None, 1200)
        .unwrap();
    v.set_transition(
        0,
        TransitionKind::Reveal,
        Some(TransitionOption::ThroughBlackRight),
        1200,
    )
    .unwrap();
    v.set_transition(1, TransitionKind::Split, None, 1500)
        .unwrap();
    v.set_transition(
        1,
        TransitionKind::Cover,
        Some(TransitionOption::RightDown),
        800,
    )
    .unwrap();
    v.set_transition_advance(1, false, Some(3000)).unwrap();
    v.add_animation(0, text, EffectKind::Fade, EffectStart::OnClick)
        .unwrap();
    v.add_animation(0, rect, EffectKind::FlyIn, EffectStart::WithPrevious)
        .unwrap();
    v.add_animation(0, rect, EffectKind::Pulse, EffectStart::AfterPrevious)
        .unwrap();
    v.add_animation(0, text, EffectKind::Appear, EffectStart::OnClick)
        .unwrap();
    v.replace_animation(0, text, Some(EffectKind::FadeOut))
        .unwrap();
    v.add_animation(0, rect, EffectKind::Disappear, EffectStart::OnClick)
        .unwrap();
    v.move_animation(0, 3, 0).unwrap();
    v.set_animation_start(0, 1, EffectStart::AfterPrevious)
        .unwrap();
    v.remove_animation(0, 2).unwrap();
}

#[test]
fn each_edit_saves_opens_again_as_it_was_and_undoes_to_the_opened_bytes() {
    let (source, text, rect) = base();
    let mut v = Viewer::open(source.clone()).unwrap();
    animate(&mut v, text, rect);
    let expected = kinds(&v, 0);
    assert_eq!(
        expected,
        vec![
            (Some(EffectKind::Disappear), EffectStart::OnClick),
            (Some(EffectKind::FadeOut), EffectStart::AfterPrevious),
            (Some(EffectKind::Pulse), EffectStart::AfterPrevious),
        ]
    );
    let saved = v.save().unwrap();
    let again = Viewer::open(saved.clone()).unwrap();
    assert_eq!(kinds(&again, 0), expected);
    assert_eq!(choice(&again, 0), (Some(TransitionKind::Reveal), 1200));
    let one = again.transition_choice(1).unwrap();
    assert_eq!(
        (
            one.kind,
            one.option,
            one.duration_ms,
            one.on_click,
            one.after_ms
        ),
        (
            Some(TransitionKind::Cover),
            Some(TransitionOption::RightDown),
            800,
            false,
            Some(3000)
        )
    );
    assert_eq!(again.save().unwrap(), saved, "not a fixed point");
    while v.can_undo_redo().0 {
        v.undo().unwrap();
    }
    assert_eq!(v.save().unwrap(), source, "undone");
}

#[test]
fn apply_to_all_gives_every_slide_the_transition_in_one_undo_step() {
    let (source, _, _) = base();
    let mut v = Viewer::open(source).unwrap();
    v.set_transition(1, TransitionKind::Fade, None, 700)
        .unwrap();
    v.transition_to_all(1).unwrap();
    for i in 0..v.slide_count() {
        assert_eq!(choice(&v, i), (Some(TransitionKind::Fade), 700));
    }
    v.undo().unwrap();
    assert_eq!(choice(&v, 0), (Some(TransitionKind::None), 0));
}

#[test]
fn the_show_plays_the_new_transitions_as_themselves() {
    let (source, _, _) = base();
    let mut v = Viewer::open(source).unwrap();
    for (kind, option, effect, direction) in [
        (TransitionKind::Split, None, "split", "horz out"),
        (
            TransitionKind::Cover,
            Some(TransitionOption::LeftUp),
            "cover",
            "lu",
        ),
        (
            TransitionKind::Uncover,
            Some(TransitionOption::Down),
            "pull",
            "d",
        ),
        (
            TransitionKind::Zoom,
            Some(TransitionOption::Out),
            "zoom",
            "out",
        ),
        (
            TransitionKind::Reveal,
            Some(TransitionOption::Right),
            "reveal",
            "r",
        ),
    ] {
        v.set_transition(0, kind, option, 1500).unwrap();
        let t = v.show_slide(0).unwrap().transition.unwrap();
        assert_eq!(
            (
                t.effect,
                t.duration_ms,
                t.played_as_is,
                t.direction.as_str()
            ),
            (effect, 1500, true, direction)
        );
    }
    v.set_transition_advance(0, true, Some(2500)).unwrap();
    assert_eq!(v.show_slide(0).unwrap().advance_after_ms, Some(2500));
}

#[test]
fn effect_options_timing_and_the_painter_save_and_read_back() {
    use casual_slide_transaction::EffectOption;
    let (source, text, rect) = base();
    let mut v = Viewer::open(source.clone()).unwrap();
    for kind in [EffectKind::FloatIn, EffectKind::Spin] {
        v.add_animation(0, text, kind, EffectStart::OnClick)
            .unwrap();
    }
    v.set_animation(0, 0, Some(EffectOption::FromTop), 1500, 250)
        .unwrap();
    v.paint_animation(0, text, rect).unwrap();
    let saved = v.save().unwrap();
    let again = Viewer::open(saved.clone()).unwrap();
    let list = again.animations(0).unwrap();
    let read: Vec<_> = list
        .iter()
        .map(|a| {
            (
                a.node,
                a.entry.kind,
                a.entry.option,
                a.entry.duration_ms,
                a.entry.delay_ms,
            )
        })
        .collect();
    let float = (
        Some(EffectKind::FloatIn),
        Some(EffectOption::FromTop),
        1500,
        250,
    );
    let spin = (Some(EffectKind::Spin), None, 2000, 0);
    let row = |n: u32, e: (Option<EffectKind>, Option<EffectOption>, u32, u32)| {
        (Some(n), e.0, e.1, e.2, e.3)
    };
    assert_eq!(
        read,
        vec![
            row(text, float),
            row(text, spin),
            row(rect, float),
            row(rect, spin)
        ]
    );
    assert_eq!(again.save().unwrap(), saved, "not a fixed point");
    while v.can_undo_redo().0 {
        v.undo().unwrap();
    }
    assert_eq!(v.save().unwrap(), source, "undone");
    let Some((code, report)) = validate("tools/f4/validate.py", &saved, "f4-ux006", "F4_REQUIRED")
    else {
        return;
    };
    assert!(code == 0 || code == 2, "F4 rejected it:\n{report}");
}

#[test]
fn a_slide_drawn_for_a_step_hides_the_shape_and_leaves_the_model_alone() {
    let (source, _, rect) = base();
    let mut v = Viewer::open(source.clone()).unwrap();
    let whole = v.render_png(0, 320).unwrap();
    let hiding = v.render_png_hiding(0, 320, &[rect]).unwrap();
    assert_ne!(whole, hiding);
    assert_eq!(v.render_png(0, 320).unwrap(), whole);
    assert_eq!(v.save().unwrap(), source);
}

#[test]
fn an_animated_deck_from_a_file_lists_its_effects() {
    let v = Viewer::open(fixture("rr-animated.pptx")).unwrap();
    let list = v.animations(0).unwrap();
    assert!(!list.is_empty());
    assert!(list.iter().all(|a| a.entry.spid.is_some()));
}

fn validate(tool: &str, saved: &[u8], name: &str, required: &str) -> Option<(i32, String)> {
    let python = ["python3", "python"]
        .into_iter()
        .find(|p| Command::new(p).arg("--version").output().is_ok());
    let Some(python) = python else {
        assert!(
            std::env::var_os(required).is_none(),
            "{required} set, no python"
        );
        return None;
    };
    let dir = std::env::temp_dir().join("openpresent-vs10");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{name}-{}.pptx", std::process::id()));
    std::fs::write(&file, saved).unwrap();
    let tool = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(tool);
    let out = Command::new(python)
        .arg("-I")
        .arg(tool)
        .arg(&file)
        .output()
        .unwrap();
    std::fs::remove_file(&file).unwrap();
    let report =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    Some((out.status.code().unwrap_or(-1), report))
}

fn animated_deck() -> Vec<u8> {
    let (source, text, rect) = base();
    let mut v = Viewer::open(source).unwrap();
    animate(&mut v, text, rect);
    v.save().unwrap()
}

/// F4 (`tools/f4/validate.py`, xmllint and the schemas) on the animated deck. Skips
/// where python or xmllint is missing, unless `F4_REQUIRED` is set.
#[test]
fn an_animated_deck_passes_f4() {
    let Some((code, report)) = validate(
        "tools/f4/validate.py",
        &animated_deck(),
        "f4",
        "F4_REQUIRED",
    ) else {
        return;
    };
    match code {
        0 => {}
        2 if std::env::var_os("F4_REQUIRED").is_none() => eprintln!("F4 could not run: {report}"),
        _ => panic!("F4 rejected the animated deck:\n{report}"),
    }
}

/// Microsoft's Open XML SDK validator on the animated deck; skips without `dotnet`
/// unless `OOXML_SDK_REQUIRED` is set.
#[test]
fn an_animated_deck_passes_the_sdk_validator() {
    let Some((code, report)) = validate(
        "tools/ooxml-sdk/validate.py",
        &animated_deck(),
        "sdk",
        "OOXML_SDK_REQUIRED",
    ) else {
        return;
    };
    match code {
        0 => {}
        2 if std::env::var_os("OOXML_SDK_REQUIRED").is_none() => {
            eprintln!("SDK validator could not run: {report}")
        }
        _ => panic!("the SDK validator rejected the animated deck:\n{report}"),
    }
}
