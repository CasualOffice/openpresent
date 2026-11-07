//! Geometry as plain typed values (`docs/spec/GEOM.md`): `prstGeom`, `custGeom`,
//! `prstTxWarp`, and their guides, handles, sites and paths.
//!
//! Custom geometry keeps its coordinates and formulas as the file wrote them, since
//! `ST_AdjCoordinate` is a number or a guide name and a formula is a small program:
//! `casual-slide-geometry` is what gives them meaning. `Option` means the file did
//! not state a value, or stated one that could not be used.

use crate::presentation::Coordinate;
use crate::properties::token_enum;

token_enum!(
    /// `a:ST_ShapeType`.
    ShapeType {
        Line = "line",
        LineInv = "lineInv",
        Triangle = "triangle",
        RtTriangle = "rtTriangle",
        Rect = "rect",
        Diamond = "diamond",
        Parallelogram = "parallelogram",
        Trapezoid = "trapezoid",
        NonIsoscelesTrapezoid = "nonIsoscelesTrapezoid",
        Pentagon = "pentagon",
        Hexagon = "hexagon",
        Heptagon = "heptagon",
        Octagon = "octagon",
        Decagon = "decagon",
        Dodecagon = "dodecagon",
        Star4 = "star4",
        Star5 = "star5",
        Star6 = "star6",
        Star7 = "star7",
        Star8 = "star8",
        Star10 = "star10",
        Star12 = "star12",
        Star16 = "star16",
        Star24 = "star24",
        Star32 = "star32",
        RoundRect = "roundRect",
        Round1Rect = "round1Rect",
        Round2SameRect = "round2SameRect",
        Round2DiagRect = "round2DiagRect",
        SnipRoundRect = "snipRoundRect",
        Snip1Rect = "snip1Rect",
        Snip2SameRect = "snip2SameRect",
        Snip2DiagRect = "snip2DiagRect",
        Plaque = "plaque",
        Ellipse = "ellipse",
        Teardrop = "teardrop",
        HomePlate = "homePlate",
        Chevron = "chevron",
        PieWedge = "pieWedge",
        Pie = "pie",
        BlockArc = "blockArc",
        Donut = "donut",
        NoSmoking = "noSmoking",
        RightArrow = "rightArrow",
        LeftArrow = "leftArrow",
        UpArrow = "upArrow",
        DownArrow = "downArrow",
        StripedRightArrow = "stripedRightArrow",
        NotchedRightArrow = "notchedRightArrow",
        BentUpArrow = "bentUpArrow",
        LeftRightArrow = "leftRightArrow",
        UpDownArrow = "upDownArrow",
        LeftUpArrow = "leftUpArrow",
        LeftRightUpArrow = "leftRightUpArrow",
        QuadArrow = "quadArrow",
        LeftArrowCallout = "leftArrowCallout",
        RightArrowCallout = "rightArrowCallout",
        UpArrowCallout = "upArrowCallout",
        DownArrowCallout = "downArrowCallout",
        LeftRightArrowCallout = "leftRightArrowCallout",
        UpDownArrowCallout = "upDownArrowCallout",
        QuadArrowCallout = "quadArrowCallout",
        BentArrow = "bentArrow",
        UturnArrow = "uturnArrow",
        CircularArrow = "circularArrow",
        LeftCircularArrow = "leftCircularArrow",
        LeftRightCircularArrow = "leftRightCircularArrow",
        CurvedRightArrow = "curvedRightArrow",
        CurvedLeftArrow = "curvedLeftArrow",
        CurvedUpArrow = "curvedUpArrow",
        CurvedDownArrow = "curvedDownArrow",
        SwooshArrow = "swooshArrow",
        Cube = "cube",
        Can = "can",
        LightningBolt = "lightningBolt",
        Heart = "heart",
        Sun = "sun",
        Moon = "moon",
        SmileyFace = "smileyFace",
        IrregularSeal1 = "irregularSeal1",
        IrregularSeal2 = "irregularSeal2",
        FoldedCorner = "foldedCorner",
        Bevel = "bevel",
        Frame = "frame",
        HalfFrame = "halfFrame",
        Corner = "corner",
        DiagStripe = "diagStripe",
        Chord = "chord",
        Arc = "arc",
        LeftBracket = "leftBracket",
        RightBracket = "rightBracket",
        LeftBrace = "leftBrace",
        RightBrace = "rightBrace",
        BracketPair = "bracketPair",
        BracePair = "bracePair",
        StraightConnector1 = "straightConnector1",
        BentConnector2 = "bentConnector2",
        BentConnector3 = "bentConnector3",
        BentConnector4 = "bentConnector4",
        BentConnector5 = "bentConnector5",
        CurvedConnector2 = "curvedConnector2",
        CurvedConnector3 = "curvedConnector3",
        CurvedConnector4 = "curvedConnector4",
        CurvedConnector5 = "curvedConnector5",
        Callout1 = "callout1",
        Callout2 = "callout2",
        Callout3 = "callout3",
        AccentCallout1 = "accentCallout1",
        AccentCallout2 = "accentCallout2",
        AccentCallout3 = "accentCallout3",
        BorderCallout1 = "borderCallout1",
        BorderCallout2 = "borderCallout2",
        BorderCallout3 = "borderCallout3",
        AccentBorderCallout1 = "accentBorderCallout1",
        AccentBorderCallout2 = "accentBorderCallout2",
        AccentBorderCallout3 = "accentBorderCallout3",
        WedgeRectCallout = "wedgeRectCallout",
        WedgeRoundRectCallout = "wedgeRoundRectCallout",
        WedgeEllipseCallout = "wedgeEllipseCallout",
        CloudCallout = "cloudCallout",
        Cloud = "cloud",
        Ribbon = "ribbon",
        Ribbon2 = "ribbon2",
        EllipseRibbon = "ellipseRibbon",
        EllipseRibbon2 = "ellipseRibbon2",
        LeftRightRibbon = "leftRightRibbon",
        VerticalScroll = "verticalScroll",
        HorizontalScroll = "horizontalScroll",
        Wave = "wave",
        DoubleWave = "doubleWave",
        Plus = "plus",
        FlowChartProcess = "flowChartProcess",
        FlowChartDecision = "flowChartDecision",
        FlowChartInputOutput = "flowChartInputOutput",
        FlowChartPredefinedProcess = "flowChartPredefinedProcess",
        FlowChartInternalStorage = "flowChartInternalStorage",
        FlowChartDocument = "flowChartDocument",
        FlowChartMultidocument = "flowChartMultidocument",
        FlowChartTerminator = "flowChartTerminator",
        FlowChartPreparation = "flowChartPreparation",
        FlowChartManualInput = "flowChartManualInput",
        FlowChartManualOperation = "flowChartManualOperation",
        FlowChartConnector = "flowChartConnector",
        FlowChartPunchedCard = "flowChartPunchedCard",
        FlowChartPunchedTape = "flowChartPunchedTape",
        FlowChartSummingJunction = "flowChartSummingJunction",
        FlowChartOr = "flowChartOr",
        FlowChartCollate = "flowChartCollate",
        FlowChartSort = "flowChartSort",
        FlowChartExtract = "flowChartExtract",
        FlowChartMerge = "flowChartMerge",
        FlowChartOfflineStorage = "flowChartOfflineStorage",
        FlowChartOnlineStorage = "flowChartOnlineStorage",
        FlowChartMagneticTape = "flowChartMagneticTape",
        FlowChartMagneticDisk = "flowChartMagneticDisk",
        FlowChartMagneticDrum = "flowChartMagneticDrum",
        FlowChartDisplay = "flowChartDisplay",
        FlowChartDelay = "flowChartDelay",
        FlowChartAlternateProcess = "flowChartAlternateProcess",
        FlowChartOffpageConnector = "flowChartOffpageConnector",
        ActionButtonBlank = "actionButtonBlank",
        ActionButtonHome = "actionButtonHome",
        ActionButtonHelp = "actionButtonHelp",
        ActionButtonInformation = "actionButtonInformation",
        ActionButtonForwardNext = "actionButtonForwardNext",
        ActionButtonBackPrevious = "actionButtonBackPrevious",
        ActionButtonEnd = "actionButtonEnd",
        ActionButtonBeginning = "actionButtonBeginning",
        ActionButtonReturn = "actionButtonReturn",
        ActionButtonDocument = "actionButtonDocument",
        ActionButtonSound = "actionButtonSound",
        ActionButtonMovie = "actionButtonMovie",
        Gear6 = "gear6",
        Gear9 = "gear9",
        Funnel = "funnel",
        MathPlus = "mathPlus",
        MathMinus = "mathMinus",
        MathMultiply = "mathMultiply",
        MathDivide = "mathDivide",
        MathEqual = "mathEqual",
        MathNotEqual = "mathNotEqual",
        CornerTabs = "cornerTabs",
        SquareTabs = "squareTabs",
        PlaqueTabs = "plaqueTabs",
        ChartX = "chartX",
        ChartStar = "chartStar",
        ChartPlus = "chartPlus",
    }
);

token_enum!(
    /// `a:ST_TextShapeType`.
    TextShapeType {
        TextNoShape = "textNoShape",
        TextPlain = "textPlain",
        TextStop = "textStop",
        TextTriangle = "textTriangle",
        TextTriangleInverted = "textTriangleInverted",
        TextChevron = "textChevron",
        TextChevronInverted = "textChevronInverted",
        TextRingInside = "textRingInside",
        TextRingOutside = "textRingOutside",
        TextArchUp = "textArchUp",
        TextArchDown = "textArchDown",
        TextCircle = "textCircle",
        TextButton = "textButton",
        TextArchUpPour = "textArchUpPour",
        TextArchDownPour = "textArchDownPour",
        TextCirclePour = "textCirclePour",
        TextButtonPour = "textButtonPour",
        TextCurveUp = "textCurveUp",
        TextCurveDown = "textCurveDown",
        TextCanUp = "textCanUp",
        TextCanDown = "textCanDown",
        TextWave1 = "textWave1",
        TextWave2 = "textWave2",
        TextDoubleWave1 = "textDoubleWave1",
        TextWave4 = "textWave4",
        TextInflate = "textInflate",
        TextDeflate = "textDeflate",
        TextInflateBottom = "textInflateBottom",
        TextDeflateBottom = "textDeflateBottom",
        TextInflateTop = "textInflateTop",
        TextDeflateTop = "textDeflateTop",
        TextDeflateInflate = "textDeflateInflate",
        TextDeflateInflateDeflate = "textDeflateInflateDeflate",
        TextFadeRight = "textFadeRight",
        TextFadeLeft = "textFadeLeft",
        TextFadeUp = "textFadeUp",
        TextFadeDown = "textFadeDown",
        TextSlantUp = "textSlantUp",
        TextSlantDown = "textSlantDown",
        TextCascadeUp = "textCascadeUp",
        TextCascadeDown = "textCascadeDown",
    }
);

token_enum!(
    /// `a:ST_PathFillMode`.
    PathFillMode {
        None = "none",
        Norm = "norm",
        Lighten = "lighten",
        LightenLess = "lightenLess",
        Darken = "darken",
        DarkenLess = "darkenLess",
    }
);

/// `a:CT_GeomGuide`: `gd`, in `avLst` and `gdLst`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeomGuide {
    /// `@name`, as written.
    pub name: Option<String>,
    /// `@fmla`, as written.
    pub formula: Option<String>,
}

/// `a:CT_AdjPoint2D`. Each is a coordinate or a guide name, as written.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdjPoint {
    /// `@x`.
    pub x: Option<String>,
    /// `@y`.
    pub y: Option<String>,
}

/// `a:CT_GeomRect`, as written.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeomRect {
    /// `@l`.
    pub l: Option<String>,
    /// `@t`.
    pub t: Option<String>,
    /// `@r`.
    pub r: Option<String>,
    /// `@b`.
    pub b: Option<String>,
}

/// `a:CT_XYAdjustHandle`, as written.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XyHandle {
    /// `@gdRefX`.
    pub gd_ref_x: Option<String>,
    /// `@minX`.
    pub min_x: Option<String>,
    /// `@maxX`.
    pub max_x: Option<String>,
    /// `@gdRefY`.
    pub gd_ref_y: Option<String>,
    /// `@minY`.
    pub min_y: Option<String>,
    /// `@maxY`.
    pub max_y: Option<String>,
    /// `pos`.
    pub pos: Option<AdjPoint>,
}

/// `a:CT_PolarAdjustHandle`, as written.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PolarHandle {
    /// `@gdRefR`.
    pub gd_ref_r: Option<String>,
    /// `@minR`.
    pub min_r: Option<String>,
    /// `@maxR`.
    pub max_r: Option<String>,
    /// `@gdRefAng`.
    pub gd_ref_ang: Option<String>,
    /// `@minAng`.
    pub min_ang: Option<String>,
    /// `@maxAng`.
    pub max_ang: Option<String>,
    /// `pos`.
    pub pos: Option<AdjPoint>,
}

/// One child of `ahLst`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdjustHandle {
    /// `ahXY`.
    Xy(XyHandle),
    /// `ahPolar`.
    Polar(PolarHandle),
}

/// `a:CT_ConnectionSite`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectionSite {
    /// `@ang`, as written: an angle or a guide name.
    pub angle: Option<String>,
    /// `pos`.
    pub pos: Option<AdjPoint>,
}

/// One command of a path, in document order.
/// In `a:CT_Path2D`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathCommand {
    /// `moveTo/pt`.
    MoveTo(Option<AdjPoint>),
    /// `lnTo/pt`.
    LineTo(Option<AdjPoint>),
    /// `arcTo`, each as written.
    ArcTo {
        /// `@wR`.
        width_radius: Option<String>,
        /// `@hR`.
        height_radius: Option<String>,
        /// `@stAng`.
        start_angle: Option<String>,
        /// `@swAng`.
        swing_angle: Option<String>,
    },
    /// `quadBezTo/pt`: its children, of which the schema wants two.
    QuadBezTo(Vec<AdjPoint>),
    /// `cubicBezTo/pt`: its children, of which the schema wants three.
    CubicBezTo(Vec<AdjPoint>),
    /// `close`.
    Close,
}

/// `a:CT_Path2D`. The schema default for `extrusionOk` (true, as `MS-OI29500` 2.1.1312
/// confirms against the standard's prose) and the rest are not filled in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Path2D {
    /// `@w`.
    pub w: Option<Coordinate>,
    /// `@h`.
    pub h: Option<Coordinate>,
    /// `@fill`.
    pub fill: Option<PathFillMode>,
    /// `@stroke`.
    pub stroke: Option<bool>,
    /// `@extrusionOk`.
    pub extrusion_ok: Option<bool>,
    /// The commands, in order.
    pub commands: Vec<PathCommand>,
}

/// `a:CT_CustomGeometry2D`. An absent list and an empty one differ.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomGeometry {
    /// `avLst/gd`.
    pub adjust: Option<Vec<GeomGuide>>,
    /// `gdLst/gd`.
    pub guides: Option<Vec<GeomGuide>>,
    /// `ahLst/ahXY`, `ahLst/ahPolar`.
    pub handles: Option<Vec<AdjustHandle>>,
    /// `cxnLst/cxn`.
    pub sites: Option<Vec<ConnectionSite>>,
    /// `rect`.
    pub rect: Option<GeomRect>,
    /// `pathLst/path`.
    pub paths: Option<Vec<Path2D>>,
}

/// `a:CT_PresetGeometry2D`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PresetGeometry {
    /// `@prst`. `None` if it is absent or names no preset; the tree keeps it.
    pub preset: Option<ShapeType>,
    /// `avLst`. An absent list and an empty one differ.
    pub adjust: Option<Vec<GeomGuide>>,
}

/// `a:CT_PresetTextShape`, for a text body's `prstTxWarp`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PresetTextShape {
    /// `@prst`.
    pub preset: Option<TextShapeType>,
    /// `avLst/gd`.
    pub adjust: Option<Vec<GeomGuide>>,
}

/// `EG_Geometry`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Geometry {
    /// `prstGeom`.
    Preset(PresetGeometry),
    /// `custGeom`.
    Custom(Box<CustomGeometry>),
}
