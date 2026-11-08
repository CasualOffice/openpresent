//! Presenting (`VS-07`): what a slide show needs from the open deck beyond its
//! pictures. Which slides are hidden (`p:sld/@show`), whether each has a
//! transition and how long it runs, and the speaker notes' text for Presenter View.
//!
//! The show itself, its keys and its windows, is the webapp's
//! (`webapp/editor.show.js`). This build plays every transition as a simple fade
//! and reports the effect it stands in for; nothing here decides how one looks.

use casual_slide_model::shape::PlaceholderType;
use casual_slide_model::text::{TextBody, TextItem};
use casual_slide_model::transition::{SlideTransition, TransitionEffect, TransitionSpeed};

use crate::{BridgeError, Viewer};

/// The longest transition played, in milliseconds: PowerPoint's own limit of a
/// minute. A longer `p14:dur` from a file is held to it.
pub const MAX_TRANSITION_MS: u32 = 60_000;

/// The most characters of notes handed to the page for one slide. Notes past it
/// are cut, so a hostile notes page cannot make the presenter window hold an
/// unbounded string; the cut is marked with an ellipsis.
pub const MAX_NOTES_CHARS: usize = 100_000;

/// How one slide takes part in a slide show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowSlide {
    /// `@show="0"`: skipped when presenting, as PowerPoint does.
    pub hidden: bool,
    /// The transition into this slide, if it has one.
    pub transition: Option<ShowTransition>,
}

/// A slide's transition, as this build plays it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShowTransition {
    /// The effect's element name (`fade`, `push`, `morph`), or `unread` when the
    /// transition could not be read and stays in the file as it was.
    pub effect: &'static str,
    /// How long it runs: `p14:dur` where given, otherwise `@spd`'s 500, 750 or
    /// 1000 ms (fast, medium, slow; slow is the schema's default). Zero for a cut
    /// or a transition with no effect, which change the slide at once.
    pub duration_ms: u32,
    /// Whether this build plays the effect as it is: `fade`, or nothing to see
    /// (`cut`, or no effect stated). Anything else is played as a fade and
    /// reported.
    pub played_as_is: bool,
}

impl Viewer {
    /// How slide `index` takes part in a slide show.
    ///
    /// # Errors
    ///
    /// `CS-RND-0003` for a slide that does not exist.
    pub fn show_slide(&self, index: usize) -> Result<ShowSlide, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let Some(part) = self.deck.parts().get(ordinal) else {
            // A slide that could not be read is drawn as a placeholder and shown.
            return Ok(ShowSlide {
                hidden: false,
                transition: None,
            });
        };
        let transition = part.transition.as_ref().map(|slot| {
            if matches!(slot, casual_slide_model::timing::Slot::Unread) {
                return ShowTransition {
                    effect: "unread",
                    duration_ms: speed_ms(None),
                    played_as_is: false,
                };
            }
            transition_of(slot.resolved())
        });
        Ok(ShowSlide {
            hidden: part.show == Some(false),
            transition,
        })
    }

    /// The speaker notes of slide `index`, as plain text: the notes page's body
    /// placeholders, a line per paragraph, with line breaks kept. Empty when the
    /// slide has no notes page. Document content: for the presenter's own screen,
    /// never a diagnostic.
    ///
    /// # Errors
    ///
    /// `CS-RND-0003` for a slide that does not exist.
    pub fn notes_text(&self, index: usize) -> Result<String, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let parts = self.deck.parts();
        let Some(notes) = parts
            .structure
            .entry(ordinal)
            .and_then(|e| e.notes)
            .and_then(|n| parts.get(n))
        else {
            return Ok(String::new());
        };
        let Some(tree) = notes.shape_tree() else {
            return Ok(String::new());
        };
        let mut out = String::new();
        for node in tree.descendants() {
            let is_body = node
                .placeholder()
                .is_some_and(|p| p.placeholder_type == Some(PlaceholderType::Body));
            let Some(body) = node.text_body.as_deref().filter(|_| is_body) else {
                continue;
            };
            if !out.is_empty() {
                out.push('\n');
            }
            push_body(&mut out, body);
            if out.chars().count() > MAX_NOTES_CHARS {
                break;
            }
        }
        if out.chars().count() > MAX_NOTES_CHARS {
            out = out.chars().take(MAX_NOTES_CHARS).collect();
            out.push('\u{2026}');
        }
        Ok(out.trim_end().to_string())
    }
}

fn push_body(out: &mut String, body: &TextBody) {
    for (i, p) in body.paragraphs.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        for item in &p.items {
            match item {
                TextItem::Run(r) => out.push_str(r.text.as_deref().unwrap_or("")),
                TextItem::Field(f) => out.push_str(f.text.as_deref().unwrap_or("")),
                TextItem::Break(_) => out.push('\n'),
                TextItem::Math(_) | TextItem::Retained(_) => {}
            }
        }
    }
}

fn speed_ms(speed: Option<TransitionSpeed>) -> u32 {
    match speed {
        Some(TransitionSpeed::Fast) => 500,
        Some(TransitionSpeed::Medium) => 750,
        // Slow, and the schema default when none is stated.
        _ => 1000,
    }
}

/// `p14:dur`: whole milliseconds as Office writes it. Anything else falls back
/// to the speed.
fn duration_ms(dur: Option<&str>) -> Option<u32> {
    let ms: u64 = dur?.trim().parse().ok()?;
    Some(u32::try_from(ms.min(u64::from(MAX_TRANSITION_MS))).unwrap_or(MAX_TRANSITION_MS))
}

fn transition_of(t: Option<&SlideTransition>) -> ShowTransition {
    let effect = t.and_then(|t| t.effect.as_ref());
    let name = effect.map_or("", effect_name);
    // A cut, or a transition stating no effect, changes the slide at once.
    let instant = matches!(effect, None | Some(TransitionEffect::Cut(_)));
    let duration_ms = if instant {
        0
    } else {
        t.and_then(|t| duration_ms(t.p14_dur.as_deref()))
            .unwrap_or_else(|| speed_ms(t.and_then(|t| t.spd)))
    };
    ShowTransition {
        effect: if name.is_empty() { "none" } else { name },
        duration_ms,
        played_as_is: instant || matches!(effect, Some(TransitionEffect::Fade(_))),
    }
}

/// The effect's element name, as the file spells it.
fn effect_name(e: &TransitionEffect) -> &'static str {
    use TransitionEffect as E;
    match e {
        E::Blinds(_) => "blinds",
        E::Checker(_) => "checker",
        E::Circle => "circle",
        E::Dissolve => "dissolve",
        E::Comb(_) => "comb",
        E::Cover(_) => "cover",
        E::Cut(_) => "cut",
        E::Diamond => "diamond",
        E::Fade(_) => "fade",
        E::Newsflash => "newsflash",
        E::Plus => "plus",
        E::Pull(_) => "pull",
        E::Push(_) => "push",
        E::Random => "random",
        E::RandomBar(_) => "randomBar",
        E::Split(_) => "split",
        E::Strips(_) => "strips",
        E::Wedge => "wedge",
        E::Wheel(_) => "wheel",
        E::Wipe(_) => "wipe",
        E::Zoom(_) => "zoom",
        E::Vortex(_) => "vortex",
        E::Switch(_) => "switch",
        E::Flip(_) => "flip",
        E::Ripple(_) => "ripple",
        E::Honeycomb => "honeycomb",
        E::Prism(_) => "prism",
        E::Doors(_) => "doors",
        E::Window(_) => "window",
        E::Ferris(_) => "ferris",
        E::Gallery(_) => "gallery",
        E::Conveyor(_) => "conveyor",
        E::Pan(_) => "pan",
        E::Glitter(_) => "glitter",
        E::Warp(_) => "warp",
        E::Flythrough(_) => "flythrough",
        E::Flash => "flash",
        E::Shred(_) => "shred",
        E::Reveal(_) => "reveal",
        E::WheelReverse(_) => "wheelReverse",
        E::Preset(_) => "prstTrans",
        E::Morph(_) => "morph",
    }
}
