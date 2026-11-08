//! Transitions and animations (`VS-10`): the Transitions tab's choice and duration,
//! Apply To All, and the Animations tab's effects on the selected shape with the
//! Animation Pane's order, each one undoable edit through `casual-slide-transaction`.
//! For the slide show, a slide drawn with some shapes hidden, so an entrance can start
//! from the slide without its shape.
//!
//! Shapes cross the boundary by node, as the editor selects them; the file names them
//! by `cNvPr/@id` (`spid`), which this module looks up.

use casual_slide_model::shape::ObjectId;
use casual_slide_model::timing::TimeNodePresetClass;
use casual_slide_transaction::animate::{
    EffectEntry, build_transition_with, effects, tab_view, with_advance,
};
use casual_slide_transaction::{
    EffectKind, EffectStart, NewEffect, Operation, TransitionKind, TransitionOption,
};

use crate::{BridgeError, Viewer};

/// A slide's transition as the Transitions tab shows it (`UX-004`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionChoice {
    /// Which of the tab's choices; `None` for an effect it does not offer.
    pub kind: Option<TransitionKind>,
    /// Its Effect Options choice, where it has one.
    pub option: Option<TransitionOption>,
    /// How long it runs, in milliseconds.
    pub duration_ms: u32,
    /// Advance Slide On Mouse Click (`@advClick`, on unless stated off).
    pub on_click: bool,
    /// Advance Slide After, in milliseconds (`@advTm`).
    pub after_ms: Option<u32>,
}

/// One effect as the Animation Pane and the show need it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimationItem {
    /// The node of the shape it animates, if one on the slide has its `spid`.
    pub node: Option<u32>,
    /// What the effect is, as read.
    pub entry: EffectEntry,
}

impl Viewer {
    fn ordinal_at(&self, index: usize) -> Result<u32, BridgeError> {
        self.deck
            .parts()
            .structure
            .slide(index)
            .ok_or(BridgeError::Render(
                casual_slide_render::RenderError::NoSuchSlide(index),
            ))
    }

    fn motion(&mut self, op: Operation) -> Result<bool, BridgeError> {
        self.typing = None;
        let changed = self.history.apply(self.deck.parts_mut(), op)?;
        if changed {
            self.reports.clear();
        }
        Ok(changed)
    }

    /// The transition into slide `index` as the Transitions tab shows it.
    pub fn transition_choice(&self, index: usize) -> Result<TransitionChoice, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        let t = self
            .deck
            .parts()
            .get(ordinal)
            .and_then(|p| p.transition.as_ref())
            .and_then(tab_view);
        let Some(t) = t else {
            return Ok(TransitionChoice {
                kind: Some(TransitionKind::None),
                option: None,
                duration_ms: 0,
                on_click: true,
                after_ms: None,
            });
        };
        let ms = self
            .show_slide(index)?
            .transition
            .map_or(0, |s| s.duration_ms);
        Ok(TransitionChoice {
            kind: TransitionKind::of(t),
            option: TransitionKind::option_of(t),
            duration_ms: ms,
            on_click: t.adv_click != Some(false),
            after_ms: t.adv_tm,
        })
    }

    /// Give slide `index` a transition with an Effect Options choice: one undo step.
    pub fn set_transition(
        &mut self,
        index: usize,
        kind: TransitionKind,
        option: Option<TransitionOption>,
        duration_ms: u32,
    ) -> Result<bool, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        let existing = self
            .deck
            .parts()
            .get(ordinal)
            .and_then(|p| p.transition.clone());
        let transition = build_transition_with(kind, option, duration_ms, existing.as_ref());
        self.motion(Operation::SetTransition {
            ordinal,
            transition,
        })
    }

    /// The Timing group's Advance Slide on slide `index` (`UX-004`): on a click, and
    /// after `after_ms` where given. One undo step.
    pub fn set_transition_advance(
        &mut self,
        index: usize,
        on_click: bool,
        after_ms: Option<u32>,
    ) -> Result<bool, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        let existing = self
            .deck
            .parts()
            .get(ordinal)
            .and_then(|p| p.transition.clone());
        let transition = with_advance(existing.as_ref(), on_click, after_ms);
        self.motion(Operation::SetTransition {
            ordinal,
            transition,
        })
    }

    /// Apply To All: every slide gets slide `index`'s transition, as one undo step.
    pub fn transition_to_all(&mut self, index: usize) -> Result<bool, BridgeError> {
        let from = self.ordinal_at(index)?;
        let transition = self
            .deck
            .parts()
            .get(from)
            .and_then(|p| p.transition.clone());
        let ops = self
            .deck
            .parts()
            .structure
            .slides
            .iter()
            .filter(|&&o| o != from)
            .map(|&ordinal| Operation::SetTransition {
                ordinal,
                transition: transition.clone(),
            })
            .collect();
        self.motion(Operation::Batch(ops))
    }

    /// The effects of slide `index`'s main sequence, in order.
    ///
    /// # Errors
    ///
    /// `CS-TXN-0008` for a timing this build does not edit.
    pub fn animations(&self, index: usize) -> Result<Vec<AnimationItem>, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        let Some(part) = self.deck.parts().get(ordinal) else {
            return Ok(Vec::new());
        };
        Ok(effects(ordinal, part.timing.as_ref())?
            .into_iter()
            .map(|entry| AnimationItem {
                node: entry
                    .spid
                    .and_then(|s| part.animated_node(s))
                    .map(|id| id.node),
                entry,
            })
            .collect())
    }

    /// The shape's name (`cNvPr/@name`), document text for the Animation Pane.
    pub fn shape_name(&self, index: usize, node: u32) -> Result<String, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        Ok(self
            .deck
            .parts()
            .get(ordinal)
            .and_then(|p| {
                p.node(ObjectId {
                    part: ordinal,
                    node,
                })
            })
            .and_then(|n| n.non_visual.as_ref()?.name.clone())
            .unwrap_or_default())
    }

    fn spid_of(&self, ordinal: u32, node: u32) -> Result<u32, BridgeError> {
        self.deck
            .parts()
            .get(ordinal)
            .and_then(|p| {
                p.node(ObjectId {
                    part: ordinal,
                    node,
                })
            })
            .and_then(|n| n.non_visual.as_ref()?.spid)
            .ok_or(BridgeError::BadArgument)
    }

    /// Add Animation: `kind` on shape `node`, last in the sequence.
    pub fn add_animation(
        &mut self,
        index: usize,
        node: u32,
        kind: EffectKind,
        start: EffectStart,
    ) -> Result<bool, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        let spid = self.spid_of(ordinal, node)?;
        let at = self.animations(index)?.len();
        self.motion(Operation::AddEffect {
            ordinal,
            index: at,
            effect: NewEffect {
                spid,
                kind,
                start,
                duration_ms: kind.default_ms(),
            },
        })
    }

    /// The Animation Styles gallery: the shape's effects become this one, in the
    /// place of its first, as one undo step. `None` takes them all away.
    pub fn replace_animation(
        &mut self,
        index: usize,
        node: u32,
        kind: Option<EffectKind>,
    ) -> Result<bool, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        let spid = self.spid_of(ordinal, node)?;
        let list = self.animations(index)?;
        let mine: Vec<usize> = list
            .iter()
            .enumerate()
            .filter(|(_, a)| a.entry.spid == Some(spid))
            .map(|(i, _)| i)
            .collect();
        let at = mine.first().copied().unwrap_or(list.len());
        let start = mine
            .first()
            .and_then(|&i| list.get(i))
            .map_or(EffectStart::OnClick, |a| a.entry.start);
        let mut ops: Vec<Operation> = mine
            .iter()
            .rev()
            .map(|&i| Operation::RemoveEffect { ordinal, index: i })
            .collect();
        if let Some(kind) = kind {
            ops.push(Operation::AddEffect {
                ordinal,
                index: at,
                effect: NewEffect {
                    spid,
                    kind,
                    start,
                    duration_ms: kind.default_ms(),
                },
            });
        }
        self.motion(Operation::Batch(ops))
    }

    /// Delete the effect at `at`.
    pub fn remove_animation(&mut self, index: usize, at: usize) -> Result<bool, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        self.motion(Operation::RemoveEffect { ordinal, index: at })
    }

    /// Move the effect at `from` to `to`.
    pub fn move_animation(
        &mut self,
        index: usize,
        from: usize,
        to: usize,
    ) -> Result<bool, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        self.motion(Operation::MoveEffect { ordinal, from, to })
    }

    /// Set when the effect at `at` starts.
    pub fn set_animation_start(
        &mut self,
        index: usize,
        at: usize,
        start: EffectStart,
    ) -> Result<bool, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        self.motion(Operation::SetEffectStart {
            ordinal,
            index: at,
            start,
        })
    }

    /// Draw slide `index` as PNG with the shapes `nodes` hidden, for the slide show's
    /// steps. The model is as it was afterwards: nothing here is an edit.
    pub fn render_png_hiding(
        &mut self,
        index: usize,
        width: u32,
        nodes: &[u32],
    ) -> Result<Vec<u8>, BridgeError> {
        let ordinal = self.ordinal_at(index)?;
        let saved = self.deck.parts().get(ordinal).cloned();
        if let Some(part) = self.deck.parts_mut().get_mut(ordinal) {
            for &node in nodes {
                if let Some(n) = part.node_mut(ObjectId {
                    part: ordinal,
                    node,
                }) {
                    n.non_visual.get_or_insert_with(Default::default).hidden = Some(true);
                }
            }
        }
        let drawn = self.render_png(index, width);
        if let (Some(saved), Some(part)) = (saved, self.deck.parts_mut().get_mut(ordinal)) {
            *part = saved;
        }
        drawn
    }
}

/// The class as the bridge's number: 1 entrance, 2 emphasis, 3 exit, 0 other.
pub(crate) fn class_code(class: Option<TimeNodePresetClass>) -> u32 {
    match class {
        Some(TimeNodePresetClass::Entrance) => 1,
        Some(TimeNodePresetClass::Emphasis) => 2,
        Some(TimeNodePresetClass::Exit) => 3,
        _ => 0,
    }
}
