//! A transition as a description of its element, for the patcher in `crate::drawing`
//! (ADR-013, Tier 1I, `T1I-02`): see `crate::timing`, which places it in the part.

use casual_slide_model::transition::*;

use crate::drawing::{Node, Space};
use crate::timing::{p_node, sound_node};

/// The `mc:Choice/@Requires` token a file needs to hold this effect, if it needs one.
pub(crate) fn effect_token(effect: &TransitionEffect) -> Option<&'static str> {
    match effect {
        TransitionEffect::Preset(_) => Some("p15"),
        TransitionEffect::Morph(_) => Some("p159"),
        other if other.is_extension() => Some("p14"),
        _ => None,
    }
}

/// `p:CT_SlideTransition`.
// spec-audit: owner=transition
pub(crate) fn transition_node(t: &SlideTransition) -> Node {
    p_node("transition")
        .lex("spd", t.spd.as_ref())
        .lex("advClick", t.adv_click.as_ref())
        .lex("advTm", t.adv_tm.as_ref())
        .p14("dur", t.p14_dur.as_ref())
        .opt(t.effect.as_ref().map(effect_node))
        .opt(t.snd_ac.as_ref().map(sound_action_node))
}

// spec-audit: owner=blinds,checker,comb,randomBar
fn orientation(name: &'static str, t: &OrientationTransition) -> Node {
    p_node(name).lex("dir", t.dir.as_ref())
}

// spec-audit: owner=doors,window
fn p14_orientation(name: &'static str, t: &OrientationTransition) -> Node {
    Node::new(Space::P14, name).lex("dir", t.dir.as_ref())
}

// spec-audit: owner=cover,pull
fn eight(name: &'static str, t: &EightDirectionTransition) -> Node {
    p_node(name).lex("dir", t.dir.as_ref())
}

// spec-audit: owner=cut,fade
fn black(name: &'static str, t: &OptionalBlackTransition) -> Node {
    p_node(name).lex("thruBlk", t.thru_blk.as_ref())
}

// spec-audit: owner=push,wipe
fn side(name: &'static str, t: &SideDirectionTransition) -> Node {
    p_node(name).lex("dir", t.dir.as_ref())
}

fn p14_side(name: &'static str, t: &SideDirectionTransition) -> Node {
    Node::new(Space::P14, name).lex("dir", t.dir.as_ref())
}

fn left_right(name: &'static str, t: &LeftRightDirectionTransition) -> Node {
    Node::new(Space::P14, name).lex("dir", t.dir.as_ref())
}

// spec-audit: owner=transition
fn effect_node(effect: &TransitionEffect) -> Node {
    use TransitionEffect as E;
    match effect {
        E::Blinds(t) => orientation("blinds", t),
        E::Checker(t) => orientation("checker", t),
        E::Circle => p_node("circle"),
        E::Dissolve => p_node("dissolve"),
        E::Comb(t) => orientation("comb", t),
        E::Cover(t) => eight("cover", t),
        E::Cut(t) => black("cut", t),
        E::Diamond => p_node("diamond"),
        E::Fade(t) => black("fade", t),
        E::Newsflash => p_node("newsflash"),
        E::Plus => p_node("plus"),
        E::Pull(t) => eight("pull", t),
        E::Push(t) => side("push", t),
        E::Random => p_node("random"),
        E::RandomBar(t) => orientation("randomBar", t),
        E::Split(t) => p_node("split")
            .lex("orient", t.orient.as_ref())
            .lex("dir", t.dir.as_ref()),
        E::Strips(t) => p_node("strips").lex("dir", t.dir.as_ref()),
        E::Wedge => p_node("wedge"),
        E::Wheel(t) => p_node("wheel").lex("spokes", t.spokes.as_ref()),
        E::Wipe(t) => side("wipe", t),
        E::Zoom(t) => p_node("zoom").lex("dir", t.dir.as_ref()),
        E::Vortex(t) => p14_side("vortex", t),
        E::Switch(t) => left_right("switch", t),
        E::Flip(t) => left_right("flip", t),
        E::Ripple(t) => Node::new(Space::P14, "ripple").lex("dir", t.dir.as_ref()),
        E::Honeycomb => Node::new(Space::P14, "honeycomb"),
        E::Prism(t) => Node::new(Space::P14, "prism")
            .lex("dir", t.dir.as_ref())
            .lex("isContent", t.is_content.as_ref())
            .lex("isInverted", t.is_inverted.as_ref()),
        E::Doors(t) => p14_orientation("doors", t),
        E::Window(t) => p14_orientation("window", t),
        E::Ferris(t) => left_right("ferris", t),
        E::Gallery(t) => left_right("gallery", t),
        E::Conveyor(t) => left_right("conveyor", t),
        E::Pan(t) => p14_side("pan", t),
        E::Glitter(t) => Node::new(Space::P14, "glitter")
            .lex("dir", t.dir.as_ref())
            .lex("pattern", t.pattern.as_ref()),
        E::Warp(t) => Node::new(Space::P14, "warp").lex("dir", t.dir.as_ref()),
        E::Flythrough(t) => Node::new(Space::P14, "flythrough")
            .lex("dir", t.dir.as_ref())
            .lex("hasBounce", t.has_bounce.as_ref()),
        E::Flash => Node::new(Space::P14, "flash"),
        E::Shred(t) => Node::new(Space::P14, "shred")
            .lex("pattern", t.pattern.as_ref())
            .lex("dir", t.dir.as_ref()),
        E::Reveal(t) => Node::new(Space::P14, "reveal")
            .lex("thruBlk", t.thru_blk.as_ref())
            .lex("dir", t.dir.as_ref()),
        E::WheelReverse(t) => {
            Node::new(Space::P14, "wheelReverse").lex("spokes", t.spokes.as_ref())
        }
        E::Preset(t) => Node::new(Space::P15, "prstTrans")
            .lex("prst", t.prst.as_ref())
            .lex("invX", t.inv_x.as_ref())
            .lex("invY", t.inv_y.as_ref()),
        E::Morph(t) => Node::new(Space::P159, "morph").lex("option", t.option.as_ref()),
    }
}

// spec-audit: owner=sndAc,transition,stSnd,endSnd
fn sound_action_node(action: &TransitionSoundAction) -> Node {
    let holder = p_node("sndAc");
    match action {
        TransitionSoundAction::Empty => holder,
        TransitionSoundAction::End => holder.kid(p_node("endSnd")),
        TransitionSoundAction::Start(start) => holder.kid(start_sound_node(start)),
    }
}

// spec-audit: owner=stSnd,sndAc
fn start_sound_node(start: &TransitionStartSound) -> Node {
    p_node("stSnd")
        .lex("loop", start.looping.as_ref())
        .opt(start.snd.as_ref().map(|s| sound_node("snd", s)))
}
