//! A slide's animation timing tree as plain typed values (`docs/spec/TIMING.md`, Tier
//! 1I): `p:timing`, every time node and behaviour, conditions, targets and build lists.
//!
//! `Option` means the file did not state a value, or stated one that could not be used
//! (the import report says which). A schema default is never filled in, nor one
//! `MS-OI29500` gives for PowerPoint: those are recorded in `docs/22` for Phase 8. What
//! the tree means when it runs (`BEH-23`) is Phase 8's. Nothing here plays anything.
//!
//! A target names a shape by `spid`, the file's `cNvPr/@id`, never by [`ObjectId`],
//! which lasts a session and is not written. [`SlidePart::animated_node`] answers which
//! node a target means, and [`SlidePart::retarget_shape`] follows a change of `spid`.
//!
//! `mc:AlternateContent` keeps every branch as typed values. Which one applies is what
//! `casual-slide-ooxml`'s resolver says, and that is stored beside them.

use crate::color::{Angle, Color};
use crate::presentation::Percentage;
use crate::properties::{Float32, Lexical, token_enum, trim_xml};
use crate::shape::{ObjectId, ShapeNode};
use crate::slide::SlidePart;

/// One `mc:AlternateContent`, with every branch. `T` is what a branch holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alternates<T> {
    /// `mc:Choice` and `mc:Fallback`, in file order.
    pub branches: Vec<AlternateBranch<T>>,
    /// The place in `branches` the resolver chose. `None` when nothing applies, and the
    /// construct reads as absent.
    pub selected: Option<usize>,
}

impl<T> Alternates<T> {
    /// What applies, by the resolver's answer.
    pub fn resolved(&self) -> Option<&T> {
        self.branches.get(self.selected?).map(|b| &b.value)
    }
}

/// One branch of an `mc:AlternateContent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlternateBranch<T> {
    /// `mc:Choice/@Requires`, as written. `None` for `mc:Fallback`.
    pub requires: Option<String>,
    /// Whether this is a `mc:Choice` and not a `mc:Fallback`. A `Choice` with no
    /// `Requires` is never selected, and is still kept.
    pub is_choice: bool,
    /// What it holds.
    pub value: T,
}

/// A part's `p:transition` or `p:timing`, however the file holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Slot<T> {
    /// A direct child of the part's root.
    Plain(Box<T>),
    /// Inside `mc:AlternateContent`. A branch without one holds `None`.
    Alternate(Alternates<Option<Box<T>>>),
    /// It could not be read, or crossed a bound. It stays whole in the part's tree and
    /// the report says so (`CS-TMG-0001`, `CS-TMG-0002`), and the deck opens. A later
    /// edit can replace it.
    Unread,
}

impl<T> Slot<T> {
    /// What applies: the element itself, or the branch the resolver chose.
    pub fn resolved(&self) -> Option<&T> {
        match self {
            Self::Plain(value) => Some(value),
            Self::Alternate(alternates) => alternates.resolved()?.as_deref(),
            Self::Unread => None,
        }
    }
}

token_enum!(
    /// `p:ST_TLTimeIndefinite`.
    TimeIndefinite { Indefinite = "indefinite" }
);

/// `p:ST_TLTime`: milliseconds, or `indefinite`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TimeValue {
    /// A count of milliseconds.
    Milliseconds(u32),
    /// `indefinite`.
    Indefinite,
}

impl Lexical for TimeValue {
    fn parse_lexical(s: &str) -> Option<Self> {
        if let Some(TimeIndefinite::Indefinite) = TimeIndefinite::parse_lexical(s) {
            return Some(Self::Indefinite);
        }
        u32::parse_lexical(s).map(Self::Milliseconds)
    }
    fn to_lexical(&self) -> String {
        match self {
            Self::Milliseconds(value) => value.to_string(),
            Self::Indefinite => TimeIndefinite::Indefinite.to_lexical(),
        }
    }
}

/// `p:ST_TLTimeAnimateValueTime`: a place in the animation, or `indefinite`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimateValueTime {
    /// `ST_PositiveFixedPercentage`.
    At(Percentage),
    /// `indefinite`.
    Indefinite,
}

impl Lexical for AnimateValueTime {
    fn parse_lexical(s: &str) -> Option<Self> {
        if let Some(TimeIndefinite::Indefinite) = TimeIndefinite::parse_lexical(s) {
            return Some(Self::Indefinite);
        }
        Percentage::parse_lexical(s).map(Self::At)
    }
    fn to_lexical(&self) -> String {
        match self {
            Self::At(value) => value.to_lexical(),
            Self::Indefinite => TimeIndefinite::Indefinite.to_lexical(),
        }
    }
}

token_enum!(
    /// `p:ST_IterateType`.
    IterateType { Element = "el", Word = "wd", Letter = "lt" }
);
token_enum!(
    /// `p:ST_TLChartSubelementType`.
    ChartSubelementType {
        GridLegend = "gridLegend",
        Series = "series",
        Category = "category",
        PointInSeries = "ptInSeries",
        PointInCategory = "ptInCategory",
    }
);
token_enum!(
    /// `p:ST_TLTriggerRuntimeNode`.
    TriggerRuntimeNode { First = "first", Last = "last", All = "all" }
);
token_enum!(
    /// `p:ST_TLTriggerEvent`.
    TriggerEvent {
        OnBegin = "onBegin",
        OnEnd = "onEnd",
        Begin = "begin",
        End = "end",
        OnClick = "onClick",
        OnDoubleClick = "onDblClick",
        OnMouseOver = "onMouseOver",
        OnMouseOut = "onMouseOut",
        OnNext = "onNext",
        OnPrevious = "onPrev",
        OnStopAudio = "onStopAudio",
    }
);
token_enum!(
    /// `p:ST_TLTimeNodePresetClassType`.
    TimeNodePresetClass {
        Entrance = "entr",
        Exit = "exit",
        Emphasis = "emph",
        Path = "path",
        Verb = "verb",
        MediaCall = "mediacall",
    }
);
token_enum!(
    /// `p:ST_TLTimeNodeRestartType`.
    TimeNodeRestart {
        Always = "always",
        WhenNotActive = "whenNotActive",
        Never = "never",
    }
);
token_enum!(
    /// `p:ST_TLTimeNodeFillType`.
    TimeNodeFill {
        Remove = "remove",
        Freeze = "freeze",
        Hold = "hold",
        Transition = "transition",
    }
);
token_enum!(
    /// `p:ST_TLTimeNodeSyncType`.
    TimeNodeSync { CanSlip = "canSlip", Locked = "locked" }
);
token_enum!(
    /// `p:ST_TLTimeNodeMasterRelation`.
    TimeNodeMasterRelation {
        SameClick = "sameClick",
        LastClick = "lastClick",
        NextClick = "nextClick",
    }
);
token_enum!(
    /// `p:ST_TLTimeNodeType`.
    TimeNodeType {
        ClickEffect = "clickEffect",
        WithEffect = "withEffect",
        AfterEffect = "afterEffect",
        MainSequence = "mainSeq",
        InteractiveSequence = "interactiveSeq",
        ClickParallel = "clickPar",
        WithGroup = "withGroup",
        AfterGroup = "afterGroup",
        Root = "tmRoot",
    }
);
token_enum!(
    /// `p:ST_TLNextActionType`.
    NextAction { None = "none", Seek = "seek" }
);
token_enum!(
    /// `p:ST_TLPreviousActionType`.
    PreviousAction { None = "none", SkipTimed = "skipTimed" }
);
token_enum!(
    /// `p:ST_TLBehaviorAdditiveType`.
    BehaviorAdditive {
        Base = "base",
        Sum = "sum",
        Replace = "repl",
        Multiply = "mult",
        None = "none",
    }
);
token_enum!(
    /// `p:ST_TLBehaviorAccumulateType`.
    BehaviorAccumulate { None = "none", Always = "always" }
);
token_enum!(
    /// `p:ST_TLBehaviorTransformType`.
    BehaviorTransform { Point = "pt", Image = "img" }
);
token_enum!(
    /// `p:ST_TLBehaviorOverrideType`.
    BehaviorOverride { Normal = "normal", ChildStyle = "childStyle" }
);
token_enum!(
    /// `p:ST_TLAnimateBehaviorCalcMode`.
    AnimateCalcMode { Discrete = "discrete", Linear = "lin", Formula = "fmla" }
);
token_enum!(
    /// `p:ST_TLAnimateBehaviorValueType`.
    AnimateValueType { String = "str", Number = "num", Color = "clr" }
);
token_enum!(
    /// `p:ST_TLAnimateColorSpace`.
    AnimateColorSpace { Rgb = "rgb", Hsl = "hsl" }
);
token_enum!(
    /// `p:ST_TLAnimateColorDirection`.
    AnimateColorDirection { Clockwise = "cw", CounterClockwise = "ccw" }
);
token_enum!(
    /// `p:ST_TLAnimateEffectTransition`.
    AnimateEffectTransition { In = "in", Out = "out", None = "none" }
);
token_enum!(
    /// `p:ST_TLAnimateMotionBehaviorOrigin`.
    AnimateMotionOrigin { Parent = "parent", Layout = "layout" }
);
token_enum!(
    /// `p:ST_TLAnimateMotionPathEditMode`.
    AnimateMotionPathEditMode { Relative = "relative", Fixed = "fixed" }
);
token_enum!(
    /// `p:ST_TLCommandType`.
    CommandType { Event = "evt", Call = "call", Verb = "verb" }
);
token_enum!(
    /// `p:ST_TLParaBuildType`.
    ParagraphBuildType {
        AllAtOnce = "allAtOnce",
        Paragraph = "p",
        Custom = "cust",
        Whole = "whole",
    }
);
token_enum!(
    /// `p:ST_TLDiagramBuildType`.
    DiagramBuildType {
        Whole = "whole",
        DepthByNode = "depthByNode",
        DepthByBranch = "depthByBranch",
        BreadthByNode = "breadthByNode",
        BreadthByLevel = "breadthByLvl",
        Clockwise = "cw",
        ClockwiseIn = "cwIn",
        ClockwiseOut = "cwOut",
        CounterClockwise = "ccw",
        CounterClockwiseIn = "ccwIn",
        CounterClockwiseOut = "ccwOut",
        InByRing = "inByRing",
        OutByRing = "outByRing",
        Up = "up",
        Down = "down",
        AllAtOnce = "allAtOnce",
        Custom = "cust",
    }
);
token_enum!(
    /// `p:ST_TLOleChartBuildType`.
    OleChartBuildType {
        AllAtOnce = "allAtOnce",
        Series = "series",
        Category = "category",
        SeriesElement = "seriesEl",
        CategoryElement = "categoryEl",
    }
);
token_enum!(
    /// `a:ST_ChartBuildStep`.
    ChartBuildStep {
        Category = "category",
        PointInCategory = "ptInCategory",
        Series = "series",
        PointInSeries = "ptInSeries",
        AllPoints = "allPts",
        GridLegend = "gridLegend",
    }
);
token_enum!(
    /// `a:ST_DgmBuildStep`.
    DiagramBuildStep { Shape = "sp", Background = "bg" }
);
token_enum!(
    /// `a:ST_AnimationBuildType`.
    AnimationBuildType { AllAtOnce = "allAtOnce" }
);
token_enum!(
    /// `a:ST_AnimationDgmOnlyBuildType`.
    AnimationDiagramOnlyBuildType {
        One = "one",
        LevelOne = "lvlOne",
        LevelAtOnce = "lvlAtOnce",
    }
);
token_enum!(
    /// `a:ST_AnimationChartOnlyBuildType`.
    AnimationChartOnlyBuildType {
        Series = "series",
        Category = "category",
        SeriesElement = "seriesEl",
        CategoryElement = "categoryEl",
    }
);

/// `a:ST_AnimationDgmBuildType`, a union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationDiagramBuild {
    /// `a:ST_AnimationBuildType`.
    Common(AnimationBuildType),
    /// `a:ST_AnimationDgmOnlyBuildType`.
    Diagram(AnimationDiagramOnlyBuildType),
}

impl Lexical for AnimationDiagramBuild {
    fn parse_lexical(s: &str) -> Option<Self> {
        AnimationBuildType::parse_lexical(s)
            .map(Self::Common)
            .or_else(|| AnimationDiagramOnlyBuildType::parse_lexical(s).map(Self::Diagram))
    }
    fn to_lexical(&self) -> String {
        match self {
            Self::Common(value) => value.to_lexical(),
            Self::Diagram(value) => value.to_lexical(),
        }
    }
}

/// `a:ST_AnimationChartBuildType`, a union.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationChartBuild {
    /// `a:ST_AnimationBuildType`.
    Common(AnimationBuildType),
    /// `a:ST_AnimationChartOnlyBuildType`.
    Chart(AnimationChartOnlyBuildType),
}

impl Lexical for AnimationChartBuild {
    fn parse_lexical(s: &str) -> Option<Self> {
        AnimationBuildType::parse_lexical(s)
            .map(Self::Common)
            .or_else(|| AnimationChartOnlyBuildType::parse_lexical(s).map(Self::Chart))
    }
    fn to_lexical(&self) -> String {
        match self {
            Self::Common(value) => value.to_lexical(),
            Self::Chart(value) => value.to_lexical(),
        }
    }
}

/// `p:CT_EmbeddedWAVAudioFile`, a sound by relationship. Nothing here reads the sound.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EmbeddedSound {
    /// `@r:embed`.
    pub rel_id: Option<String>,
    /// `@name`.
    pub name: Option<String>,
    /// `@builtIn`.
    pub built_in: Option<bool>,
}

/// `p:CT_IndexRange`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IndexRange {
    /// `@st`.
    pub start: Option<u32>,
    /// `@end`.
    pub end: Option<u32>,
}

/// `p:CT_TLTextTargetElement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextTarget {
    /// `txEl` with neither child.
    Whole,
    /// `charRg`.
    Characters(IndexRange),
    /// `pRg`.
    Paragraphs(IndexRange),
}

/// `a:CT_AnimationDgmElement`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimationDiagramElement {
    /// `@id`, an `ST_Guid`, as written.
    pub id: Option<String>,
    /// `@bldStep`.
    pub build_step: Option<DiagramBuildStep>,
}

/// `a:CT_AnimationChartElement`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AnimationChartElement {
    /// `@seriesIdx`.
    pub series_idx: Option<i32>,
    /// `@categoryIdx`.
    pub category_idx: Option<i32>,
    /// `@bldStep`.
    pub build_step: Option<ChartBuildStep>,
}

/// `a:CT_AnimationElementChoice`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimationElementChoice {
    /// `graphicEl` with neither child.
    Empty,
    /// `dgm`.
    Diagram(AnimationDiagramElement),
    /// `chart`.
    Chart(AnimationChartElement),
}

/// `p:CT_TLOleChartTargetElement`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OleChartTarget {
    /// `@type`.
    pub element_type: Option<ChartSubelementType>,
    /// `@lvl`.
    pub lvl: Option<u32>,
}

/// What of a shape a target means. The children of `p:CT_TLShapeTargetElement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeSubTarget {
    /// `bg`.
    Background,
    /// `subSp/@spid`, an `ST_ShapeID`, as written.
    SubShape(Option<String>),
    /// `oleChartEl`.
    OleChart(OleChartTarget),
    /// `txEl`.
    Text(TextTarget),
    /// `graphicEl`.
    Graphic(AnimationElementChoice),
}

/// `p:CT_TLShapeTargetElement`, `spTgt`. `@spid` is the file's `cNvPr/@id`:
/// [`SlidePart::animated_node`] finds the node it means.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShapeTarget {
    /// `@spid`.
    pub spid: Option<u32>,
    /// `bg`, `subSp`, `oleChartEl`, `txEl` or `graphicEl`.
    pub sub: Option<ShapeSubTarget>,
}

/// `p:CT_TLTimeTargetElement`, `tgtEl`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeTarget {
    /// `tgtEl` with no child.
    Empty,
    /// `sldTgt`.
    Slide,
    /// `sndTgt`.
    Sound(EmbeddedSound),
    /// `spTgt`.
    Shape(ShapeTarget),
    /// `inkTgt/@spid`, an `ST_ShapeID`, as written.
    Ink(Option<String>),
}

/// What a `p:CT_TLTimeCondition` listens to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionTrigger {
    /// `tgtEl`.
    Target(TimeTarget),
    /// `tn/@val`, a time node.
    Node(Option<u32>),
    /// `rtn/@val`.
    Runtime(Option<TriggerRuntimeNode>),
}

/// `p:CT_TLTimeCondition`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TimeCondition {
    /// `@evt`.
    pub evt: Option<TriggerEvent>,
    /// `@delay`.
    pub delay: Option<TimeValue>,
    /// `tgtEl`, `tn` or `rtn`.
    pub trigger: Option<ConditionTrigger>,
}

/// The interval in a `p:CT_TLIterateData`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IterateInterval {
    /// `tmAbs/@val`, `p:CT_TLIterateIntervalTime`.
    Time(Option<TimeValue>),
    /// `tmPct/@val`, `p:CT_TLIterateIntervalPercentage`.
    Percent(Option<Percentage>),
}

/// `p:CT_TLIterateData`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IterateData {
    /// `@type`.
    pub iterate_type: Option<IterateType>,
    /// `@backwards`.
    pub backwards: Option<bool>,
    /// `tmAbs` or `tmPct`.
    pub interval: Option<IterateInterval>,
}

/// `p:CT_TLCommonTimeNodeData`, `cTn`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommonTimeNode {
    /// `@id`.
    pub node_id: Option<u32>,
    /// `@presetID`. `MS-OI29500` 2.1.1157 a: PowerPoint reads 0 as custom.
    pub preset_id: Option<i32>,
    /// `@presetClass`.
    pub preset_class: Option<TimeNodePresetClass>,
    /// `@presetSubtype`.
    pub preset_subtype: Option<i32>,
    /// `@dur`.
    pub dur: Option<TimeValue>,
    /// `@repeatCount`.
    pub repeat_count: Option<TimeValue>,
    /// `@repeatDur`.
    pub repeat_dur: Option<TimeValue>,
    /// `@spd`.
    pub spd: Option<Percentage>,
    /// `@accel`.
    pub accel: Option<Percentage>,
    /// `@decel`.
    pub decel: Option<Percentage>,
    /// `@autoRev`.
    pub auto_rev: Option<bool>,
    /// `@restart`.
    pub restart: Option<TimeNodeRestart>,
    /// `@fill`.
    pub fill: Option<TimeNodeFill>,
    /// `@syncBehavior`. `MS-OI29500` 2.1.1157 n: PowerPoint does not support it.
    pub sync_behavior: Option<TimeNodeSync>,
    /// `@tmFilter`, as written.
    pub tm_filter: Option<String>,
    /// `@evtFilter`, as written.
    pub evt_filter: Option<String>,
    /// `@display`.
    pub display: Option<bool>,
    /// `@masterRel`.
    pub master_rel: Option<TimeNodeMasterRelation>,
    /// `@bldLvl`.
    pub bld_lvl: Option<i32>,
    /// `@grpId`.
    pub grp_id: Option<u32>,
    /// `@afterEffect`.
    pub after_effect: Option<bool>,
    /// `@nodeType`.
    pub node_type: Option<TimeNodeType>,
    /// `@nodePh`.
    pub node_ph: Option<bool>,
    /// `stCondLst`, each `stCondLst/cond`.
    pub st_cond_lst: Option<Vec<TimeCondition>>,
    /// `endCondLst`.
    pub end_cond_lst: Option<Vec<TimeCondition>>,
    /// `endSync`.
    pub end_sync: Option<TimeCondition>,
    /// `iterate`.
    pub iterate: Option<IterateData>,
    /// `childTnLst`.
    pub child_tn_lst: Option<Vec<TimeNode>>,
    /// `subTnLst`.
    pub sub_tn_lst: Option<Vec<TimeNode>>,
}

/// `p:CT_TLTimeNodeParallel` and `p:CT_TLTimeNodeExclusive`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContainerTimeNode {
    /// `cTn`.
    pub common: Option<CommonTimeNode>,
}

/// `p:CT_TLTimeNodeSequence`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SequenceTimeNode {
    /// `@concurrent`.
    pub concurrent: Option<bool>,
    /// `@prevAc`.
    pub prev_ac: Option<PreviousAction>,
    /// `@nextAc`.
    pub next_ac: Option<NextAction>,
    /// `cTn`.
    pub common: Option<CommonTimeNode>,
    /// `prevCondLst`.
    pub prev_cond_lst: Option<Vec<TimeCondition>>,
    /// `nextCondLst`.
    pub next_cond_lst: Option<Vec<TimeCondition>>,
}

/// `p:CT_TLCommonBehaviorData`, `cBhvr`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommonBehavior {
    /// `@additive`.
    pub additive: Option<BehaviorAdditive>,
    /// `@accumulate`.
    pub accumulate: Option<BehaviorAccumulate>,
    /// `@xfrmType`.
    pub xfrm_type: Option<BehaviorTransform>,
    /// `@from`, as written.
    pub from: Option<String>,
    /// `@to`, as written.
    pub to: Option<String>,
    /// `@by`, as written.
    pub by: Option<String>,
    /// `@rctx`, as written. `MS-OI29500` 2.1.1152 g gives a grammar nothing here applies.
    pub rctx: Option<String>,
    /// `@override`.
    pub override_style: Option<BehaviorOverride>,
    /// `cTn`.
    pub common: Option<CommonTimeNode>,
    /// `tgtEl`.
    pub target: Option<TimeTarget>,
    /// `attrNameLst`, each `attrNameLst/attrName`'s text. One that could not be read is empty, so
    /// each element keeps its place.
    pub attr_name_lst: Option<Vec<String>>,
}

/// `p:CT_TLAnimVariant`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnimVariant {
    /// The element with no child.
    Empty,
    /// `boolVal/@val`, `p:CT_TLAnimVariantBooleanVal`.
    Bool(Option<bool>),
    /// `intVal/@val`, `p:CT_TLAnimVariantIntegerVal`.
    Int(Option<i32>),
    /// `fltVal/@val`, `p:CT_TLAnimVariantFloatVal`.
    Float(Option<Float32>),
    /// `strVal/@val`, `p:CT_TLAnimVariantStringVal`.
    Str(Option<String>),
    /// `clrVal`.
    Color(Option<Color>),
}

/// `p:CT_TLTimeAnimateValue`, `tav`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TimeAnimateValue {
    /// `@tm`.
    pub tm: Option<AnimateValueTime>,
    /// `@fmla`, as written. Never evaluated here.
    pub fmla: Option<String>,
    /// `val`.
    pub val: Option<AnimVariant>,
}

/// `p:CT_TLAnimateBehavior`, `anim`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimateBehavior {
    /// `@by`, as written.
    pub by: Option<String>,
    /// `@from`, as written.
    pub from: Option<String>,
    /// `@to`, as written.
    pub to: Option<String>,
    /// `@calcmode`.
    pub calcmode: Option<AnimateCalcMode>,
    /// `@valueType`.
    pub value_type: Option<AnimateValueType>,
    /// `cBhvr`.
    pub behavior: Option<CommonBehavior>,
    /// `tavLst`, each `tavLst/tav`.
    pub tav_lst: Option<Vec<TimeAnimateValue>>,
}

/// `p:CT_TLByRgbColorTransform`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ByRgb {
    /// `@r`.
    pub red: Option<Percentage>,
    /// `@g`.
    pub green: Option<Percentage>,
    /// `@b`.
    pub blue: Option<Percentage>,
}

/// `p:CT_TLByHslColorTransform`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ByHsl {
    /// `@h`.
    pub hue: Option<Angle>,
    /// `@s`.
    pub saturation: Option<Percentage>,
    /// `@l`.
    pub lightness: Option<Percentage>,
}

/// `p:CT_TLByAnimateColorTransform`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByAnimateColor {
    /// `by` with neither child.
    Empty,
    /// `rgb`.
    Rgb(ByRgb),
    /// `hsl`.
    Hsl(ByHsl),
}

/// `p:CT_TLAnimateColorBehavior`, `animClr`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimateColorBehavior {
    /// `@clrSpc`.
    pub clr_spc: Option<AnimateColorSpace>,
    /// `@dir`.
    pub dir: Option<AnimateColorDirection>,
    /// `cBhvr`.
    pub behavior: Option<CommonBehavior>,
    /// `by`.
    pub by: Option<ByAnimateColor>,
    /// `from`. A holder with no colour reads as none, and is reported.
    pub from: Option<Option<Color>>,
    /// `to`. A holder with no colour reads as none, and is reported.
    pub to: Option<Option<Color>>,
}

/// `p:CT_TLAnimateEffectBehavior`, `animEffect`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimateEffectBehavior {
    /// `@transition`.
    pub transition: Option<AnimateEffectTransition>,
    /// `@filter`, as written.
    pub filter: Option<String>,
    /// `@prLst`, as written.
    pub pr_lst: Option<String>,
    /// `cBhvr`.
    pub behavior: Option<CommonBehavior>,
    /// `progress`.
    pub progress: Option<AnimVariant>,
}

/// `p:CT_TLPoint`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TimePoint {
    /// `@x`.
    pub x: Option<Percentage>,
    /// `@y`.
    pub y: Option<Percentage>,
}

/// `p:CT_TLAnimateMotionBehavior`, `animMotion`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimateMotionBehavior {
    /// `@origin`.
    pub origin: Option<AnimateMotionOrigin>,
    /// `@path`, as written. Never evaluated here.
    pub path: Option<String>,
    /// `@pathEditMode`.
    pub path_edit_mode: Option<AnimateMotionPathEditMode>,
    /// `@rAng`.
    pub r_ang: Option<Angle>,
    /// `@ptsTypes`, as written.
    pub pts_types: Option<String>,
    /// `cBhvr`.
    pub behavior: Option<CommonBehavior>,
    /// `by`.
    pub by: Option<TimePoint>,
    /// `from`.
    pub from: Option<TimePoint>,
    /// `to`.
    pub to: Option<TimePoint>,
    /// `rCtr`.
    pub r_ctr: Option<TimePoint>,
}

/// `p:CT_TLAnimateRotationBehavior`, `animRot`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimateRotationBehavior {
    /// `@by`.
    pub by: Option<Angle>,
    /// `@from`.
    pub from: Option<Angle>,
    /// `@to`.
    pub to: Option<Angle>,
    /// `cBhvr`.
    pub behavior: Option<CommonBehavior>,
}

/// `p:CT_TLAnimateScaleBehavior`, `animScale`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimateScaleBehavior {
    /// `@zoomContents`.
    pub zoom_contents: Option<bool>,
    /// `cBhvr`.
    pub behavior: Option<CommonBehavior>,
    /// `by`.
    pub by: Option<TimePoint>,
    /// `from`.
    pub from: Option<TimePoint>,
    /// `to`.
    pub to: Option<TimePoint>,
}

/// `p:CT_TLCommandBehavior`, `cmd`. The command is held and never run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandBehavior {
    /// `@type`.
    pub command_type: Option<CommandType>,
    /// `@cmd`, as written.
    pub cmd: Option<String>,
    /// `cBhvr`.
    pub behavior: Option<CommonBehavior>,
}

/// `p:CT_TLSetBehavior`, `set`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetBehavior {
    /// `cBhvr`.
    pub behavior: Option<CommonBehavior>,
    /// `to`.
    pub to: Option<AnimVariant>,
}

/// `p:CT_TLCommonMediaNodeData`, `cMediaNode`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommonMediaNode {
    /// `@vol`.
    pub vol: Option<Percentage>,
    /// `@mute`.
    pub mute: Option<bool>,
    /// `@numSld`.
    pub num_sld: Option<u32>,
    /// `@showWhenStopped`.
    pub show_when_stopped: Option<bool>,
    /// `cTn`.
    pub common: Option<CommonTimeNode>,
    /// `tgtEl`.
    pub target: Option<TimeTarget>,
}

/// `p:CT_TLMediaNodeAudio`, `audio`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AudioNode {
    /// `@isNarration`.
    pub is_narration: Option<bool>,
    /// `cMediaNode`.
    pub media: Option<CommonMediaNode>,
}

/// `p:CT_TLMediaNodeVideo`, `video`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VideoNode {
    /// `@fullScrn`.
    pub full_scrn: Option<bool>,
    /// `cMediaNode`.
    pub media: Option<CommonMediaNode>,
}

/// One entry of `p:CT_TimeNodeList`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeNode {
    /// `par`.
    Par(ContainerTimeNode),
    /// `seq`.
    Seq(SequenceTimeNode),
    /// `excl`.
    Excl(ContainerTimeNode),
    /// `anim`.
    Anim(AnimateBehavior),
    /// `animClr`.
    AnimClr(AnimateColorBehavior),
    /// `animEffect`.
    AnimEffect(AnimateEffectBehavior),
    /// `animMotion`.
    AnimMotion(AnimateMotionBehavior),
    /// `animRot`.
    AnimRot(AnimateRotationBehavior),
    /// `animScale`.
    AnimScale(AnimateScaleBehavior),
    /// `cmd`.
    Cmd(CommandBehavior),
    /// `set`.
    Set(SetBehavior),
    /// `audio`.
    Audio(AudioNode),
    /// `video`.
    Video(VideoNode),
    /// `mc:AlternateContent` in a list of time nodes, with every branch.
    Alternate(Alternates<Vec<TimeNode>>),
}

/// `p:AG_TLBuild`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildTarget {
    /// `@spid`, the file's `cNvPr/@id`.
    pub spid: Option<u32>,
    /// `@grpId`.
    pub grp_id: Option<u32>,
    /// `@uiExpand`.
    pub ui_expand: Option<bool>,
}

/// `p:CT_TLTemplate`, `tmpl`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildTemplate {
    /// `@lvl`.
    pub lvl: Option<u32>,
    /// `tnLst`.
    pub tn_lst: Option<Vec<TimeNode>>,
}

/// `p:CT_TLBuildParagraph`, `bldP`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildParagraph {
    /// `@spid`, `@grpId` and `@uiExpand`.
    pub target: BuildTarget,
    /// `@build`.
    pub build: Option<ParagraphBuildType>,
    /// `@bldLvl`. `MS-OI29500` 2.1.1149 g: PowerPoint honours it only with `build="p"`.
    pub bld_lvl: Option<u32>,
    /// `@animBg`.
    pub anim_bg: Option<bool>,
    /// `@autoUpdateAnimBg`. `MS-OI29500` 2.1.1149 e: PowerPoint does not support it.
    pub auto_update_anim_bg: Option<bool>,
    /// `@rev`.
    pub rev: Option<bool>,
    /// `@advAuto`. `MS-OI29500` 2.1.1149 d: PowerPoint reads `indefinite` as 0 here.
    pub adv_auto: Option<TimeValue>,
    /// `tmplLst`, each `tmplLst/tmpl`.
    pub tmpl_lst: Option<Vec<BuildTemplate>>,
}

/// `p:CT_TLBuildDiagram`, `bldDgm`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildDiagram {
    /// `@spid`, `@grpId` and `@uiExpand`.
    pub target: BuildTarget,
    /// `@bld`.
    pub bld: Option<DiagramBuildType>,
}

/// `p:CT_TLOleBuildChart`, `bldOleChart`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildOleChart {
    /// `@spid`, `@grpId` and `@uiExpand`.
    pub target: BuildTarget,
    /// `@bld`.
    pub bld: Option<OleChartBuildType>,
    /// `@animBg`.
    pub anim_bg: Option<bool>,
}

/// `a:CT_AnimationDgmBuildProperties`, `bldDgm` in `bldSub`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DiagramBuildProperties {
    /// `@bld`.
    pub bld: Option<AnimationDiagramBuild>,
    /// `@rev`.
    pub rev: Option<bool>,
}

/// `a:CT_AnimationChartBuildProperties`, `bldChart` in `bldSub`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChartBuildProperties {
    /// `@bld`.
    pub bld: Option<AnimationChartBuild>,
    /// `@animBg`.
    pub anim_bg: Option<bool>,
}

/// `a:CT_AnimationGraphicalObjectBuildProperties`, `bldSub`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicalObjectBuildProperties {
    /// `bldSub` with neither child.
    Empty,
    /// `bldDgm`.
    Diagram(DiagramBuildProperties),
    /// `bldChart`.
    Chart(ChartBuildProperties),
}

/// How a `p:CT_TLGraphicalObjectBuild` builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicBuildChoice {
    /// `bldAsOne`.
    AsOne,
    /// `bldSub`.
    Sub(GraphicalObjectBuildProperties),
}

/// `p:CT_TLGraphicalObjectBuild`, `bldGraphic`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildGraphic {
    /// `@spid`, `@grpId` and `@uiExpand`.
    pub target: BuildTarget,
    /// `bldAsOne` or `bldSub`.
    pub choice: Option<GraphicBuildChoice>,
}

/// One entry of `p:CT_BuildList`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildEntry {
    /// `bldP`.
    Paragraph(BuildParagraph),
    /// `bldDgm`.
    Diagram(BuildDiagram),
    /// `bldOleChart`.
    OleChart(BuildOleChart),
    /// `bldGraphic`.
    Graphic(BuildGraphic),
}

impl BuildEntry {
    /// `p:AG_TLBuild`, whichever kind this is.
    pub fn target(&self) -> &BuildTarget {
        match self {
            Self::Paragraph(entry) => &entry.target,
            Self::Diagram(entry) => &entry.target,
            Self::OleChart(entry) => &entry.target,
            Self::Graphic(entry) => &entry.target,
        }
    }

    fn target_mut(&mut self) -> &mut BuildTarget {
        match self {
            Self::Paragraph(entry) => &mut entry.target,
            Self::Diagram(entry) => &mut entry.target,
            Self::OleChart(entry) => &mut entry.target,
            Self::Graphic(entry) => &mut entry.target,
        }
    }
}

/// `p:CT_SlideTiming`, `timing`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SlideTiming {
    /// `tnLst`.
    pub tn_lst: Option<Vec<TimeNode>>,
    /// `bldLst`.
    pub bld_lst: Option<Vec<BuildEntry>>,
}

/// Every place a time-node tree names something, in file order.
pub trait TargetVisitor {
    /// A shape, by `spid`, with the element that names it.
    fn shape(&mut self, element: &'static str, spid: &mut u32);
    /// A time node's `cTn`.
    fn common(&mut self, _node: &mut CommonTimeNode) {}
    /// A condition's `tn/@val`.
    fn node_reference(&mut self, _val: u32) {}
}

fn visit_target(target: &mut TimeTarget, visitor: &mut dyn TargetVisitor) {
    if let TimeTarget::Shape(shape) = target
        && let Some(spid) = &mut shape.spid
    {
        visitor.shape("spTgt", spid);
    }
}

fn visit_conditions(list: &mut Option<Vec<TimeCondition>>, visitor: &mut dyn TargetVisitor) {
    for condition in list.iter_mut().flatten() {
        visit_condition(condition, visitor);
    }
}

fn visit_condition(condition: &mut TimeCondition, visitor: &mut dyn TargetVisitor) {
    match &mut condition.trigger {
        Some(ConditionTrigger::Target(target)) => visit_target(target, visitor),
        Some(ConditionTrigger::Node(Some(val))) => visitor.node_reference(*val),
        _ => {}
    }
}

fn visit_common(common: &mut Option<CommonTimeNode>, visitor: &mut dyn TargetVisitor) {
    let Some(common) = common else { return };
    visitor.common(common);
    visit_conditions(&mut common.st_cond_lst, visitor);
    visit_conditions(&mut common.end_cond_lst, visitor);
    if let Some(condition) = &mut common.end_sync {
        visit_condition(condition, visitor);
    }
    for node in common.child_tn_lst.iter_mut().flatten() {
        node.visit(visitor);
    }
    for node in common.sub_tn_lst.iter_mut().flatten() {
        node.visit(visitor);
    }
}

fn visit_behavior(behavior: &mut Option<CommonBehavior>, visitor: &mut dyn TargetVisitor) {
    let Some(behavior) = behavior else { return };
    visit_common(&mut behavior.common, visitor);
    if let Some(target) = &mut behavior.target {
        visit_target(target, visitor);
    }
}

fn visit_media(media: &mut Option<CommonMediaNode>, visitor: &mut dyn TargetVisitor) {
    let Some(media) = media else { return };
    visit_common(&mut media.common, visitor);
    if let Some(target) = &mut media.target {
        visit_target(target, visitor);
    }
}

impl TimeNode {
    /// Walk this node and what it holds. Depth follows the tree: a tree the reader
    /// produced is within `TimingLimits::max_node_depth`, and the writer checks one
    /// built by hand with [`Self::depth_within`] first.
    pub fn visit(&mut self, visitor: &mut dyn TargetVisitor) {
        match self {
            Self::Par(node) | Self::Excl(node) => visit_common(&mut node.common, visitor),
            Self::Seq(node) => {
                visit_common(&mut node.common, visitor);
                visit_conditions(&mut node.prev_cond_lst, visitor);
                visit_conditions(&mut node.next_cond_lst, visitor);
            }
            Self::Anim(node) => visit_behavior(&mut node.behavior, visitor),
            Self::AnimClr(node) => visit_behavior(&mut node.behavior, visitor),
            Self::AnimEffect(node) => visit_behavior(&mut node.behavior, visitor),
            Self::AnimMotion(node) => visit_behavior(&mut node.behavior, visitor),
            Self::AnimRot(node) => visit_behavior(&mut node.behavior, visitor),
            Self::AnimScale(node) => visit_behavior(&mut node.behavior, visitor),
            Self::Cmd(node) => visit_behavior(&mut node.behavior, visitor),
            Self::Set(node) => visit_behavior(&mut node.behavior, visitor),
            Self::Audio(node) => visit_media(&mut node.media, visitor),
            Self::Video(node) => visit_media(&mut node.media, visitor),
            Self::Alternate(alternates) => {
                for branch in &mut alternates.branches {
                    for node in &mut branch.value {
                        node.visit(visitor);
                    }
                }
            }
        }
    }

    /// Whether this node's depth, itself included, is at most `limit`. Stops at the
    /// limit, so a tree built by hand costs no more than a tree the reader would allow.
    pub fn depth_within(&self, limit: usize) -> bool {
        let Some(below) = limit.checked_sub(1) else {
            return false;
        };
        let lists = |common: &Option<CommonTimeNode>| {
            common.as_ref().is_none_or(|c| {
                [&c.child_tn_lst, &c.sub_tn_lst]
                    .into_iter()
                    .all(|list| list.iter().flatten().all(|node| node.depth_within(below)))
            })
        };
        let behavior = |b: &Option<CommonBehavior>| b.as_ref().is_none_or(|b| lists(&b.common));
        let media = |m: &Option<CommonMediaNode>| m.as_ref().is_none_or(|m| lists(&m.common));
        match self {
            Self::Par(node) | Self::Excl(node) => lists(&node.common),
            Self::Seq(node) => lists(&node.common),
            Self::Anim(node) => behavior(&node.behavior),
            Self::AnimClr(node) => behavior(&node.behavior),
            Self::AnimEffect(node) => behavior(&node.behavior),
            Self::AnimMotion(node) => behavior(&node.behavior),
            Self::AnimRot(node) => behavior(&node.behavior),
            Self::AnimScale(node) => behavior(&node.behavior),
            Self::Cmd(node) => behavior(&node.behavior),
            Self::Set(node) => behavior(&node.behavior),
            Self::Audio(node) => media(&node.media),
            Self::Video(node) => media(&node.media),
            Self::Alternate(alternates) => alternates
                .branches
                .iter()
                .all(|b| b.value.iter().all(|node| node.depth_within(below))),
        }
    }
}

impl SlideTiming {
    /// Walk the whole tree and the build list.
    pub fn visit(&mut self, visitor: &mut dyn TargetVisitor) {
        for node in self.tn_lst.iter_mut().flatten() {
            node.visit(visitor);
        }
        for entry in self.bld_lst.iter_mut().flatten() {
            if let BuildEntry::Paragraph(paragraph) = entry {
                for template in paragraph.tmpl_lst.iter_mut().flatten() {
                    for node in template.tn_lst.iter_mut().flatten() {
                        node.visit(visitor);
                    }
                }
            }
            let name = match entry {
                BuildEntry::Paragraph(_) => "bldP",
                BuildEntry::Diagram(_) => "bldDgm",
                BuildEntry::OleChart(_) => "bldOleChart",
                BuildEntry::Graphic(_) => "bldGraphic",
            };
            if let Some(spid) = &mut entry.target_mut().spid {
                visitor.shape(name, spid);
            }
        }
    }

    /// Whether every node is within this depth.
    pub fn depth_within(&self, limit: usize) -> bool {
        let list = |nodes: &Option<Vec<TimeNode>>| {
            nodes.iter().flatten().all(|node| node.depth_within(limit))
        };
        list(&self.tn_lst)
            && self.bld_lst.iter().flatten().all(|entry| match entry {
                BuildEntry::Paragraph(p) => p.tmpl_lst.iter().flatten().all(|t| list(&t.tn_lst)),
                _ => true,
            })
    }
}

impl<T> Slot<T> {
    /// Every value this slot holds, in every branch.
    pub fn values_mut(&mut self) -> Vec<&mut T> {
        match self {
            Self::Plain(value) => vec![&mut **value],
            Self::Alternate(alternates) => alternates
                .branches
                .iter_mut()
                .filter_map(|b| b.value.as_deref_mut())
                .collect(),
            Self::Unread => Vec::new(),
        }
    }
}

struct Spids<'a>(&'a mut Vec<u32>);

impl TargetVisitor for Spids<'_> {
    fn shape(&mut self, _element: &'static str, spid: &mut u32) {
        self.0.push(*spid);
    }
}

struct Retarget {
    from: u32,
    to: u32,
    changed: usize,
}

impl TargetVisitor for Retarget {
    fn shape(&mut self, _element: &'static str, spid: &mut u32) {
        if *spid == self.from {
            *spid = self.to;
            self.changed += 1;
        }
    }
}

impl SlidePart {
    /// Every `spid` this part's timing names, in every branch, repeats included.
    pub fn animated_spids(&self) -> Vec<u32> {
        let mut timing = self.timing.clone();
        let mut out = Vec::new();
        for value in timing.iter_mut().flat_map(Slot::values_mut) {
            value.visit(&mut Spids(&mut out));
        }
        out
    }

    /// The node an animation target means: `cNvPr/@id` in the shape tree, which holds
    /// the branch markup compatibility selects (Tier 1D). `None` for a `spid` on no node.
    pub fn animated_node(&self, spid: u32) -> Option<ObjectId> {
        self.shape_tree()?
            .descendants()
            .find(|node: &&ShapeNode| node.non_visual.as_ref().and_then(|v| v.spid) == Some(spid))
            .map(|node| node.id)
    }

    /// Give the node with `from` the `spid` `to`, and every animation target with it,
    /// so targets follow their shape. Returns how many targets moved. Without a node of
    /// that `spid`, nothing changes, and 0 comes back.
    pub fn retarget_shape(&mut self, from: u32, to: u32) -> usize {
        let Some(id) = self.animated_node(from) else {
            return 0;
        };
        if let Some(visual) = self.node_mut(id).and_then(|n| n.non_visual.as_mut()) {
            visual.spid = Some(to);
        }
        let mut visitor = Retarget {
            from,
            to,
            changed: 0,
        };
        for value in self.timing.iter_mut().flat_map(Slot::values_mut) {
            value.visit(&mut visitor);
        }
        visitor.changed
    }
}

/// A value of a list-valued token, split as XML does. Used for `mc:Choice/@Requires`.
pub fn requires_tokens(requires: &str) -> impl Iterator<Item = &str> {
    trim_xml(requires).split_ascii_whitespace()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_time_is_milliseconds_or_indefinite_and_nothing_else() {
        assert_eq!(
            TimeValue::parse_lexical(" 500 "),
            Some(TimeValue::Milliseconds(500))
        );
        assert_eq!(
            TimeValue::parse_lexical("indefinite"),
            Some(TimeValue::Indefinite)
        );
        assert_eq!(TimeValue::Indefinite.to_lexical(), "indefinite");
        assert_eq!(TimeValue::Milliseconds(7).to_lexical(), "7");
        for bad in ["", "-1", "5000000000", "INDEFINITE", "1.5"] {
            assert_eq!(TimeValue::parse_lexical(bad), None, "{bad}");
        }
    }

    #[test]
    fn a_tree_deeper_than_a_limit_is_said_to_be() {
        let mut node = TimeNode::Par(ContainerTimeNode::default());
        for _ in 0..4 {
            node = TimeNode::Par(ContainerTimeNode {
                common: Some(CommonTimeNode {
                    child_tn_lst: Some(vec![node]),
                    ..CommonTimeNode::default()
                }),
            });
        }
        assert!(node.depth_within(5));
        assert!(!node.depth_within(4));
        assert!(!node.depth_within(0));
    }

    #[test]
    fn the_unions_read_either_side() {
        assert_eq!(
            AnimationDiagramBuild::parse_lexical("lvlOne"),
            Some(AnimationDiagramBuild::Diagram(
                AnimationDiagramOnlyBuildType::LevelOne
            ))
        );
        assert_eq!(
            AnimationChartBuild::parse_lexical("allAtOnce"),
            Some(AnimationChartBuild::Common(AnimationBuildType::AllAtOnce))
        );
        assert_eq!(AnimationChartBuild::parse_lexical("one"), None);
        assert_eq!(
            AnimateValueTime::parse_lexical("50000"),
            Some(AnimateValueTime::At(Percentage(50000)))
        );
    }
}
