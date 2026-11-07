//! A slide's transition as plain typed values (`docs/spec/TRANS.md`, Tier 1I): the 21
//! base effects, the 19 from the 2010 namespace, `p15:prstTrans` and Morph.
//!
//! The newer ones live in `mc:AlternateContent`, with an older effect as the fallback.
//! Both branches are values here, in [`crate::timing::Slot::Alternate`], and both stay in
//! the tree. What applies is what the resolver says. Nothing here draws a transition.

use crate::properties::{Lexical, token_enum};
use crate::timing::EmbeddedSound;

token_enum!(
    /// `p:ST_TransitionSideDirectionType`.
    SideDirection { Left = "l", Up = "u", Right = "r", Down = "d" }
);
token_enum!(
    /// `p:ST_TransitionCornerDirectionType`.
    CornerDirection {
        LeftUp = "lu",
        RightUp = "ru",
        LeftDown = "ld",
        RightDown = "rd",
    }
);
token_enum!(
    /// `p:ST_TransitionInOutDirectionType`.
    InOutDirection { Out = "out", In = "in" }
);
token_enum!(
    /// `p:ST_Direction`.
    OrientationDirection { Horizontal = "horz", Vertical = "vert" }
);
token_enum!(
    /// `p:ST_TransitionSpeed`.
    TransitionSpeed { Slow = "slow", Medium = "med", Fast = "fast" }
);
token_enum!(
    /// `ST_TransitionMorphOption`, in the 2015 namespace.
    MorphOption { ByObject = "byObject", ByWord = "byWord", ByChar = "byChar" }
);
token_enum!(
    /// `ST_TransitionPattern`, in the 2010 namespace.
    TransitionPattern { Diamond = "diamond", Hexagon = "hexagon" }
);
token_enum!(
    /// `ST_TransitionCenterDirectionType`, in the 2010 namespace.
    CenterDirection { Center = "center" }
);
token_enum!(
    /// `ST_TransitionShredPattern`, in the 2010 namespace.
    ShredPattern { Strip = "strip", Rectangle = "rectangle" }
);
token_enum!(
    /// `ST_TransitionLeftRightDirectionType`, in the 2010 namespace.
    LeftRightDirection { Left = "l", Right = "r" }
);

/// `p:ST_TransitionEightDirectionType`, a union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EightDirection {
    /// `p:ST_TransitionSideDirectionType`.
    Side(SideDirection),
    /// `p:ST_TransitionCornerDirectionType`.
    Corner(CornerDirection),
}

impl Lexical for EightDirection {
    fn parse_lexical(s: &str) -> Option<Self> {
        SideDirection::parse_lexical(s)
            .map(Self::Side)
            .or_else(|| CornerDirection::parse_lexical(s).map(Self::Corner))
    }
    fn to_lexical(&self) -> String {
        match self {
            Self::Side(value) => value.to_lexical(),
            Self::Corner(value) => value.to_lexical(),
        }
    }
}

/// `ST_TransitionCornerAndCenterDirectionType`, a union, in the 2010 namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerOrCenter {
    /// `p:ST_TransitionCornerDirectionType`.
    Corner(CornerDirection),
    /// `center`.
    Center(CenterDirection),
}

impl Lexical for CornerOrCenter {
    fn parse_lexical(s: &str) -> Option<Self> {
        CornerDirection::parse_lexical(s)
            .map(Self::Corner)
            .or_else(|| CenterDirection::parse_lexical(s).map(Self::Center))
    }
    fn to_lexical(&self) -> String {
        match self {
            Self::Corner(value) => value.to_lexical(),
            Self::Center(value) => value.to_lexical(),
        }
    }
}

/// `p:CT_SideDirectionTransition`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SideDirectionTransition {
    /// `@dir`.
    pub dir: Option<SideDirection>,
}

/// `p:CT_CornerDirectionTransition`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CornerDirectionTransition {
    /// `@dir`.
    pub dir: Option<CornerDirection>,
}

/// `p:CT_EightDirectionTransition`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EightDirectionTransition {
    /// `@dir`.
    pub dir: Option<EightDirection>,
}

/// `p:CT_OrientationTransition`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OrientationTransition {
    /// `@dir`.
    pub dir: Option<OrientationDirection>,
}

/// `p:CT_InOutTransition`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InOutTransition {
    /// `@dir`.
    pub dir: Option<InOutDirection>,
}

/// `p:CT_OptionalBlackTransition`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OptionalBlackTransition {
    /// `@thruBlk`.
    pub thru_blk: Option<bool>,
}

/// `p:CT_SplitTransition`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SplitTransition {
    /// `@orient`.
    pub orient: Option<OrientationDirection>,
    /// `@dir`.
    pub dir: Option<InOutDirection>,
}

/// `p:CT_WheelTransition`. `MS-OI29500` 2.1.1186 a: PowerPoint maps spokes other than
/// 1, 2, 3, 4 and 8 to the nearest it supports. The file's number is what is held.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WheelTransition {
    /// `@spokes`.
    pub spokes: Option<u32>,
}

/// `CT_LeftRightDirectionTransition`, in the 2010 namespace.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LeftRightDirectionTransition {
    /// `@dir`.
    pub dir: Option<LeftRightDirection>,
}

/// `CT_RippleTransition`, in the 2010 namespace.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RippleTransition {
    /// `@dir`.
    pub dir: Option<CornerOrCenter>,
}

/// `CT_PrismTransition`, in the 2010 namespace.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PrismTransition {
    /// `@dir`.
    pub dir: Option<SideDirection>,
    /// `@isContent`.
    pub is_content: Option<bool>,
    /// `@isInverted`.
    pub is_inverted: Option<bool>,
}

/// `CT_GlitterTransition`, in the 2010 namespace.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GlitterTransition {
    /// `@dir`.
    pub dir: Option<SideDirection>,
    /// `@pattern`.
    pub pattern: Option<TransitionPattern>,
}

/// `CT_FlyThroughTransition`, in the 2010 namespace.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlyThroughTransition {
    /// `@dir`.
    pub dir: Option<InOutDirection>,
    /// `@hasBounce`.
    pub has_bounce: Option<bool>,
}

/// `CT_ShredTransition`, in the 2010 namespace.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ShredTransition {
    /// `@pattern`.
    pub pattern: Option<ShredPattern>,
    /// `@dir`.
    pub dir: Option<InOutDirection>,
}

/// `CT_RevealTransition`, in the 2010 namespace.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RevealTransition {
    /// `@thruBlk`.
    pub thru_blk: Option<bool>,
    /// `@dir`.
    pub dir: Option<LeftRightDirection>,
}

/// `CT_PresetTransition`, `p15:prstTrans`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PresetTransition {
    /// `@prst`, as written: the schema says only string, and no list of names is
    /// documented here.
    pub prst: Option<String>,
    /// `@invX`.
    pub inv_x: Option<bool>,
    /// `@invY`.
    pub inv_y: Option<bool>,
}

/// `CT_MorphTransition`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MorphTransition {
    /// `@option`.
    pub option: Option<MorphOption>,
}

/// The effect, the choice in `p:CT_SlideTransition`, with what later Office versions add.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionEffect {
    /// `blinds`.
    Blinds(OrientationTransition),
    /// `checker`.
    Checker(OrientationTransition),
    /// `circle`.
    Circle,
    /// `dissolve`.
    Dissolve,
    /// `comb`.
    Comb(OrientationTransition),
    /// `cover`.
    Cover(EightDirectionTransition),
    /// `cut`.
    Cut(OptionalBlackTransition),
    /// `diamond`.
    Diamond,
    /// `fade`.
    Fade(OptionalBlackTransition),
    /// `newsflash`.
    Newsflash,
    /// `plus`.
    Plus,
    /// `pull`.
    Pull(EightDirectionTransition),
    /// `push`.
    Push(SideDirectionTransition),
    /// `random`.
    Random,
    /// `randomBar`.
    RandomBar(OrientationTransition),
    /// `split`.
    Split(SplitTransition),
    /// `strips`.
    Strips(CornerDirectionTransition),
    /// `wedge`.
    Wedge,
    /// `wheel`.
    Wheel(WheelTransition),
    /// `wipe`.
    Wipe(SideDirectionTransition),
    /// `zoom`.
    Zoom(InOutTransition),
    /// `p14:vortex`.
    Vortex(SideDirectionTransition),
    /// `p14:switch`.
    Switch(LeftRightDirectionTransition),
    /// `p14:flip`.
    Flip(LeftRightDirectionTransition),
    /// `p14:ripple`.
    Ripple(RippleTransition),
    /// `p14:honeycomb`.
    Honeycomb,
    /// `p14:prism`.
    Prism(PrismTransition),
    /// `p14:doors`.
    Doors(OrientationTransition),
    /// `p14:window`.
    Window(OrientationTransition),
    /// `p14:ferris`.
    Ferris(LeftRightDirectionTransition),
    /// `p14:gallery`.
    Gallery(LeftRightDirectionTransition),
    /// `p14:conveyor`.
    Conveyor(LeftRightDirectionTransition),
    /// `p14:pan`.
    Pan(SideDirectionTransition),
    /// `p14:glitter`.
    Glitter(GlitterTransition),
    /// `p14:warp`.
    Warp(InOutTransition),
    /// `p14:flythrough`.
    Flythrough(FlyThroughTransition),
    /// `p14:flash`.
    Flash,
    /// `p14:shred`.
    Shred(ShredTransition),
    /// `p14:reveal`.
    Reveal(RevealTransition),
    /// `p14:wheelReverse`.
    WheelReverse(WheelTransition),
    /// `p15:prstTrans`.
    Preset(PresetTransition),
    /// `morph`, in the 2015 namespace.
    Morph(MorphTransition),
}

impl TransitionEffect {
    /// Whether the schema outside `p:CT_SlideTransition` holds this one, so a file may
    /// carry it only where it is ignorable or in a branch that requires it.
    pub fn is_extension(&self) -> bool {
        matches!(
            self,
            Self::Vortex(_)
                | Self::Switch(_)
                | Self::Flip(_)
                | Self::Ripple(_)
                | Self::Honeycomb
                | Self::Prism(_)
                | Self::Doors(_)
                | Self::Window(_)
                | Self::Ferris(_)
                | Self::Gallery(_)
                | Self::Conveyor(_)
                | Self::Pan(_)
                | Self::Glitter(_)
                | Self::Warp(_)
                | Self::Flythrough(_)
                | Self::Flash
                | Self::Shred(_)
                | Self::Reveal(_)
                | Self::WheelReverse(_)
                | Self::Preset(_)
                | Self::Morph(_)
        )
    }
}

/// `p:CT_TransitionStartSoundAction`, `stSnd`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransitionStartSound {
    /// `@loop`.
    pub looping: Option<bool>,
    /// `snd`.
    pub snd: Option<EmbeddedSound>,
}

/// `p:CT_TransitionSoundAction`, `sndAc`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionSoundAction {
    /// `sndAc` with neither child.
    Empty,
    /// `stSnd`.
    Start(TransitionStartSound),
    /// `endSnd`.
    End,
}

/// `p:CT_SlideTransition`, `transition`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SlideTransition {
    /// `@spd`.
    pub spd: Option<TransitionSpeed>,
    /// `@advClick`.
    pub adv_click: Option<bool>,
    /// `@advTm`.
    pub adv_tm: Option<u32>,
    /// `@p14:dur`, an `ST_UniversalTimeOffset`, as written. Read from the 2010 namespace
    /// and from the older one Office 2010's beta wrote (`fixtures/real/sdk-animation.pptx`).
    pub p14_dur: Option<String>,
    /// The effect.
    pub effect: Option<TransitionEffect>,
    /// `sndAc`.
    pub snd_ac: Option<TransitionSoundAction>,
}
