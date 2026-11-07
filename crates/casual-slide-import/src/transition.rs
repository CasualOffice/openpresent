//! Read `p:transition` (Tier 1I, `T1I-02`): the 21 base effects, the 19 from the 2010
//! namespace, `p15:prstTrans` and Morph, and what a transition holds around them. Timing
//! and transitions share one reader, so a bound, a fault and a branch of
//! `mc:AlternateContent` go the same way: see [`crate::timing`].
//!
//! Which branch applies is the resolver's: its list of what it understands is ECMA-376
//! alone, so a `Choice` requiring `p14` gives way to its `Fallback`. Both are read.

use casual_slide_model::transition::*;

use crate::properties::{Elem, ImportFindingKind as K, ns};
use crate::timing::{MotionReader, Read, once, p14_name};

/// `p14:`, then the namespace Office 2010's beta wrote for the same attribute:
/// `fixtures/real/sdk-animation.pptx` holds `dur` in it. The first is listed twice, as
/// `Writer::uri` takes the second for a Strict deck, and only the first is written new.
pub const P14: &[&str] = &[
    "http://schemas.microsoft.com/office/powerpoint/2010/main",
    "http://schemas.microsoft.com/office/powerpoint/2010/main",
    "http://schemas.microsoft.com/office/powerpoint/2007/7/12/main",
];
/// `p15:`.
pub const P15: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2012/main"];
/// `p159:`.
pub const P159: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2015/09/main"];

impl MotionReader<'_, '_> {
    /// `p:CT_SlideTransition`.
    // spec-audit: owner=transition
    pub(crate) fn transition(&mut self, el: &Elem) -> Read<SlideTransition> {
        let mut e = el.clone();
        let mut transition = SlideTransition {
            spd: self.r.attr(&mut e, "spd")?,
            adv_click: self.r.attr(&mut e, "advClick")?,
            adv_tm: self.r.attr(&mut e, "advTm")?,
            p14_dur: match p14_name(&e, "dur") {
                Some(written) => self.r.attr(&mut e, &written)?,
                None => None,
            },
            ..SlideTransition::default()
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            let effect = self.effect(&child)?;
            if let Some(effect) = effect {
                once!(self, &child, transition.effect, effect);
            } else if child.is(ns::PML, "sndAc") {
                once!(self, &child, transition.snd_ac, self.sound_action(&child)?);
            } else {
                // `extLst` stays.
                self.r.retained(&child);
            }
        }
        Ok(transition)
    }

    /// The effect this child is, if it is one.
    // spec-audit: owner=transition
    fn effect(&mut self, c: &Elem) -> Read<Option<TransitionEffect>> {
        use TransitionEffect as E;
        if c.in_namespace(ns::PML) {
            return Ok(Some(match c.local.as_str() {
                "blinds" => E::Blinds(self.orientation(c)?),
                "checker" => E::Checker(self.orientation(c)?),
                "circle" => self.unit(c, E::Circle)?,
                "dissolve" => self.unit(c, E::Dissolve)?,
                "comb" => E::Comb(self.orientation(c)?),
                "cover" => E::Cover(self.eight(c)?),
                "cut" => E::Cut(self.black(c)?),
                "diamond" => self.unit(c, E::Diamond)?,
                "fade" => E::Fade(self.black(c)?),
                "newsflash" => self.unit(c, E::Newsflash)?,
                "plus" => self.unit(c, E::Plus)?,
                "pull" => E::Pull(self.eight(c)?),
                "push" => E::Push(self.side(c)?),
                "random" => self.unit(c, E::Random)?,
                "randomBar" => E::RandomBar(self.orientation(c)?),
                "split" => E::Split(self.split(c)?),
                "strips" => E::Strips(self.corner(c)?),
                "wedge" => self.unit(c, E::Wedge)?,
                "wheel" => E::Wheel(self.wheel(c)?),
                "wipe" => E::Wipe(self.side(c)?),
                "zoom" => E::Zoom(self.in_out(c)?),
                _ => return Ok(None),
            }));
        }
        if c.in_namespace(P14) {
            return Ok(Some(match c.local.as_str() {
                "vortex" => E::Vortex(self.side(c)?),
                "switch" => E::Switch(self.left_right(c)?),
                "flip" => E::Flip(self.left_right(c)?),
                "ripple" => E::Ripple(self.ripple(c)?),
                "honeycomb" => self.unit(c, E::Honeycomb)?,
                "prism" => E::Prism(self.prism(c)?),
                "doors" => E::Doors(self.orientation(c)?),
                "window" => E::Window(self.orientation(c)?),
                "ferris" => E::Ferris(self.left_right(c)?),
                "gallery" => E::Gallery(self.left_right(c)?),
                "conveyor" => E::Conveyor(self.left_right(c)?),
                "pan" => E::Pan(self.side(c)?),
                "glitter" => E::Glitter(self.glitter(c)?),
                "warp" => E::Warp(self.in_out(c)?),
                "flythrough" => E::Flythrough(self.flythrough(c)?),
                "flash" => self.unit(c, E::Flash)?,
                "shred" => E::Shred(self.shred(c)?),
                "reveal" => E::Reveal(self.reveal(c)?),
                "wheelReverse" => E::WheelReverse(self.wheel(c)?),
                _ => return Ok(None),
            }));
        }
        if c.is(P15, "prstTrans") {
            return Ok(Some(E::Preset(self.preset(c)?)));
        }
        // spec-audit: x:powerpoint/2015/09/main:<morph>
        if c.is(P159, "morph") {
            return Ok(Some(E::Morph(self.morph(c)?)));
        }
        Ok(None)
    }

    fn unit(&mut self, c: &Elem, effect: TransitionEffect) -> Read<TransitionEffect> {
        self.r.finish(c);
        self.nothing_inside(c)?;
        Ok(effect)
    }

    fn finish_effect(&mut self, e: &Elem) -> Read<()> {
        self.r.finish(e);
        self.nothing_inside(e)
    }

    /// `p:CT_OrientationTransition`.
    // spec-audit: owner=blinds,checker,comb,randomBar
    fn orientation(&mut self, c: &Elem) -> Read<OrientationTransition> {
        let mut e = c.clone();
        let t = OrientationTransition {
            dir: self.r.attr(&mut e, "dir")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    /// `p:CT_EightDirectionTransition`.
    // spec-audit: owner=cover,pull
    fn eight(&mut self, c: &Elem) -> Read<EightDirectionTransition> {
        let mut e = c.clone();
        let t = EightDirectionTransition {
            dir: self.r.attr(&mut e, "dir")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    /// `p:CT_OptionalBlackTransition`.
    // spec-audit: owner=cut,fade
    fn black(&mut self, c: &Elem) -> Read<OptionalBlackTransition> {
        let mut e = c.clone();
        let t = OptionalBlackTransition {
            thru_blk: self.r.attr(&mut e, "thruBlk")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    /// `p:CT_SideDirectionTransition`.
    // spec-audit: owner=push,wipe
    fn side(&mut self, c: &Elem) -> Read<SideDirectionTransition> {
        let mut e = c.clone();
        let t = SideDirectionTransition {
            dir: self.r.attr(&mut e, "dir")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    /// `p:CT_SplitTransition`.
    // spec-audit: owner=split
    fn split(&mut self, c: &Elem) -> Read<SplitTransition> {
        let mut e = c.clone();
        let t = SplitTransition {
            orient: self.r.attr(&mut e, "orient")?,
            dir: self.r.attr(&mut e, "dir")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    /// `p:CT_CornerDirectionTransition`.
    // spec-audit: owner=strips
    fn corner(&mut self, c: &Elem) -> Read<CornerDirectionTransition> {
        let mut e = c.clone();
        let t = CornerDirectionTransition {
            dir: self.r.attr(&mut e, "dir")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    /// `p:CT_WheelTransition`.
    // spec-audit: owner=wheel
    fn wheel(&mut self, c: &Elem) -> Read<WheelTransition> {
        let mut e = c.clone();
        let t = WheelTransition {
            spokes: self.r.attr(&mut e, "spokes")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    /// `p:CT_InOutTransition`.
    // spec-audit: owner=zoom
    fn in_out(&mut self, c: &Elem) -> Read<InOutTransition> {
        let mut e = c.clone();
        let t = InOutTransition {
            dir: self.r.attr(&mut e, "dir")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    fn left_right(&mut self, c: &Elem) -> Read<LeftRightDirectionTransition> {
        let mut e = c.clone();
        let t = LeftRightDirectionTransition {
            dir: self.r.attr(&mut e, "dir")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    fn ripple(&mut self, c: &Elem) -> Read<RippleTransition> {
        let mut e = c.clone();
        let t = RippleTransition {
            dir: self.r.attr(&mut e, "dir")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    fn prism(&mut self, c: &Elem) -> Read<PrismTransition> {
        let mut e = c.clone();
        let t = PrismTransition {
            dir: self.r.attr(&mut e, "dir")?,
            is_content: self.r.attr(&mut e, "isContent")?,
            is_inverted: self.r.attr(&mut e, "isInverted")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    fn glitter(&mut self, c: &Elem) -> Read<GlitterTransition> {
        let mut e = c.clone();
        let t = GlitterTransition {
            dir: self.r.attr(&mut e, "dir")?,
            pattern: self.r.attr(&mut e, "pattern")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    fn flythrough(&mut self, c: &Elem) -> Read<FlyThroughTransition> {
        let mut e = c.clone();
        let t = FlyThroughTransition {
            dir: self.r.attr(&mut e, "dir")?,
            has_bounce: self.r.attr(&mut e, "hasBounce")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    fn shred(&mut self, c: &Elem) -> Read<ShredTransition> {
        let mut e = c.clone();
        let t = ShredTransition {
            pattern: self.r.attr(&mut e, "pattern")?,
            dir: self.r.attr(&mut e, "dir")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    fn reveal(&mut self, c: &Elem) -> Read<RevealTransition> {
        let mut e = c.clone();
        let t = RevealTransition {
            thru_blk: self.r.attr(&mut e, "thruBlk")?,
            dir: self.r.attr(&mut e, "dir")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    fn preset(&mut self, c: &Elem) -> Read<PresetTransition> {
        let mut e = c.clone();
        let t = PresetTransition {
            prst: self.r.attr(&mut e, "prst")?,
            inv_x: self.r.attr(&mut e, "invX")?,
            inv_y: self.r.attr(&mut e, "invY")?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    /// `CT_MorphTransition`.
    // spec-audit: owner=morph
    fn morph(&mut self, c: &Elem) -> Read<MorphTransition> {
        let mut e = c.clone();
        let t = MorphTransition {
            option: self.r.required(&mut e, "option", |_: &MorphOption| true)?,
        };
        self.finish_effect(&e)?;
        Ok(t)
    }

    /// `p:CT_TransitionSoundAction`.
    // spec-audit: owner=sndAc
    fn sound_action(&mut self, el: &Elem) -> Read<TransitionSoundAction> {
        self.r.finish(el);
        let mut action = None;
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::PML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "stSnd" => {
                    let start = self.start_sound(&child)?;
                    once!(self, &child, action, TransitionSoundAction::Start(start));
                }
                "endSnd" => {
                    self.r.finish(&child);
                    self.nothing_inside(&child)?;
                    once!(self, &child, action, TransitionSoundAction::End);
                }
                _ => self.r.retained(&child),
            }
        }
        Ok(action.unwrap_or(TransitionSoundAction::Empty))
    }

    /// `p:CT_TransitionStartSoundAction`.
    // spec-audit: owner=stSnd
    fn start_sound(&mut self, el: &Elem) -> Read<TransitionStartSound> {
        let mut e = el.clone();
        let mut start = TransitionStartSound {
            looping: self.r.attr(&mut e, "loop")?,
            snd: None,
        };
        self.r.finish(&e);
        for child in self.r.children(el)? {
            if child.is(ns::PML, "snd") {
                once!(self, &child, start.snd, self.sound(&child)?);
            } else {
                self.r.retained(&child);
            }
        }
        if start.snd.is_none() {
            self.r.note(K::MissingRequired, el, Some("snd"));
        }
        Ok(start)
    }
}
