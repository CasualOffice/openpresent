//! 3D properties as plain typed values (`docs/spec/D3.md`): `scene3d` and `sp3d`.
//!
//! BEH-27: these are held and written back, never flattened. The defaults `MS-OI29500`
//! gives for a renderer (2.1.1272 extrusion colour is the front face's, 2.1.1276 contour
//! colour is black, 2.1.1274 `rot` overrides a light rig's `dir`, 2.1.1270 `fov` applies
//! only to perspective cameras) are the resolver's to apply, not values to store.

use crate::color::{Angle, Color, Percentage};
use crate::presentation::Coordinate;
use crate::properties::token_enum;

token_enum!(
    /// `a:ST_PresetCameraType`.
    PresetCamera {
        LegacyObliqueTopLeft = "legacyObliqueTopLeft",
        LegacyObliqueTop = "legacyObliqueTop",
        LegacyObliqueTopRight = "legacyObliqueTopRight",
        LegacyObliqueLeft = "legacyObliqueLeft",
        LegacyObliqueFront = "legacyObliqueFront",
        LegacyObliqueRight = "legacyObliqueRight",
        LegacyObliqueBottomLeft = "legacyObliqueBottomLeft",
        LegacyObliqueBottom = "legacyObliqueBottom",
        LegacyObliqueBottomRight = "legacyObliqueBottomRight",
        LegacyPerspectiveTopLeft = "legacyPerspectiveTopLeft",
        LegacyPerspectiveTop = "legacyPerspectiveTop",
        LegacyPerspectiveTopRight = "legacyPerspectiveTopRight",
        LegacyPerspectiveLeft = "legacyPerspectiveLeft",
        LegacyPerspectiveFront = "legacyPerspectiveFront",
        LegacyPerspectiveRight = "legacyPerspectiveRight",
        LegacyPerspectiveBottomLeft = "legacyPerspectiveBottomLeft",
        LegacyPerspectiveBottom = "legacyPerspectiveBottom",
        LegacyPerspectiveBottomRight = "legacyPerspectiveBottomRight",
        OrthographicFront = "orthographicFront",
        IsometricTopUp = "isometricTopUp",
        IsometricTopDown = "isometricTopDown",
        IsometricBottomUp = "isometricBottomUp",
        IsometricBottomDown = "isometricBottomDown",
        IsometricLeftUp = "isometricLeftUp",
        IsometricLeftDown = "isometricLeftDown",
        IsometricRightUp = "isometricRightUp",
        IsometricRightDown = "isometricRightDown",
        IsometricOffAxis1Left = "isometricOffAxis1Left",
        IsometricOffAxis1Right = "isometricOffAxis1Right",
        IsometricOffAxis1Top = "isometricOffAxis1Top",
        IsometricOffAxis2Left = "isometricOffAxis2Left",
        IsometricOffAxis2Right = "isometricOffAxis2Right",
        IsometricOffAxis2Top = "isometricOffAxis2Top",
        IsometricOffAxis3Left = "isometricOffAxis3Left",
        IsometricOffAxis3Right = "isometricOffAxis3Right",
        IsometricOffAxis3Bottom = "isometricOffAxis3Bottom",
        IsometricOffAxis4Left = "isometricOffAxis4Left",
        IsometricOffAxis4Right = "isometricOffAxis4Right",
        IsometricOffAxis4Bottom = "isometricOffAxis4Bottom",
        ObliqueTopLeft = "obliqueTopLeft",
        ObliqueTop = "obliqueTop",
        ObliqueTopRight = "obliqueTopRight",
        ObliqueLeft = "obliqueLeft",
        ObliqueRight = "obliqueRight",
        ObliqueBottomLeft = "obliqueBottomLeft",
        ObliqueBottom = "obliqueBottom",
        ObliqueBottomRight = "obliqueBottomRight",
        PerspectiveFront = "perspectiveFront",
        PerspectiveLeft = "perspectiveLeft",
        PerspectiveRight = "perspectiveRight",
        PerspectiveAbove = "perspectiveAbove",
        PerspectiveBelow = "perspectiveBelow",
        PerspectiveAboveLeftFacing = "perspectiveAboveLeftFacing",
        PerspectiveAboveRightFacing = "perspectiveAboveRightFacing",
        PerspectiveContrastingLeftFacing = "perspectiveContrastingLeftFacing",
        PerspectiveContrastingRightFacing = "perspectiveContrastingRightFacing",
        PerspectiveHeroicLeftFacing = "perspectiveHeroicLeftFacing",
        PerspectiveHeroicRightFacing = "perspectiveHeroicRightFacing",
        PerspectiveHeroicExtremeLeftFacing = "perspectiveHeroicExtremeLeftFacing",
        PerspectiveHeroicExtremeRightFacing = "perspectiveHeroicExtremeRightFacing",
        PerspectiveRelaxed = "perspectiveRelaxed",
        PerspectiveRelaxedModerately = "perspectiveRelaxedModerately",
    }
);

token_enum!(
    /// `a:ST_LightRigDirection`.
    LightRigDirection {
        Tl = "tl",
        T = "t",
        Tr = "tr",
        L = "l",
        R = "r",
        Bl = "bl",
        B = "b",
        Br = "br",
    }
);

token_enum!(
    /// `a:ST_LightRigType`.
    LightRigType {
        LegacyFlat1 = "legacyFlat1",
        LegacyFlat2 = "legacyFlat2",
        LegacyFlat3 = "legacyFlat3",
        LegacyFlat4 = "legacyFlat4",
        LegacyNormal1 = "legacyNormal1",
        LegacyNormal2 = "legacyNormal2",
        LegacyNormal3 = "legacyNormal3",
        LegacyNormal4 = "legacyNormal4",
        LegacyHarsh1 = "legacyHarsh1",
        LegacyHarsh2 = "legacyHarsh2",
        LegacyHarsh3 = "legacyHarsh3",
        LegacyHarsh4 = "legacyHarsh4",
        ThreePt = "threePt",
        Balanced = "balanced",
        Soft = "soft",
        Harsh = "harsh",
        Flood = "flood",
        Contrasting = "contrasting",
        Morning = "morning",
        Sunrise = "sunrise",
        Sunset = "sunset",
        Chilly = "chilly",
        Freezing = "freezing",
        Flat = "flat",
        TwoPt = "twoPt",
        Glow = "glow",
        BrightRoom = "brightRoom",
    }
);

token_enum!(
    /// `a:ST_BevelPresetType`.
    BevelPreset {
        RelaxedInset = "relaxedInset",
        Circle = "circle",
        Slope = "slope",
        Cross = "cross",
        Angle = "angle",
        SoftRound = "softRound",
        Convex = "convex",
        CoolSlant = "coolSlant",
        Divot = "divot",
        Riblet = "riblet",
        HardEdge = "hardEdge",
        ArtDeco = "artDeco",
    }
);

token_enum!(
    /// `a:ST_PresetMaterialType`.
    PresetMaterial {
        LegacyMatte = "legacyMatte",
        LegacyPlastic = "legacyPlastic",
        LegacyMetal = "legacyMetal",
        LegacyWireframe = "legacyWireframe",
        Matte = "matte",
        Plastic = "plastic",
        Metal = "metal",
        WarmMatte = "warmMatte",
        TranslucentPowder = "translucentPowder",
        Powder = "powder",
        DkEdge = "dkEdge",
        SoftEdge = "softEdge",
        Clear = "clear",
        Flat = "flat",
        Softmetal = "softmetal",
    }
);

/// `a:CT_SphereCoords`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SphereCoords {
    /// `@lat`.
    pub latitude: Option<Angle>,
    /// `@lon`.
    pub longitude: Option<Angle>,
    /// `@rev`.
    pub revolution: Option<Angle>,
}

/// `a:CT_Point3D`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point3D {
    /// `@x`.
    pub x: Option<Coordinate>,
    /// `@y`.
    pub y: Option<Coordinate>,
    /// `@z`.
    pub z: Option<Coordinate>,
}

/// `a:CT_Vector3D`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Vector3D {
    /// `@dx`.
    pub dx: Option<Coordinate>,
    /// `@dy`.
    pub dy: Option<Coordinate>,
    /// `@dz`.
    pub dz: Option<Coordinate>,
}

/// `a:CT_Camera`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Camera {
    /// `@prst`.
    pub preset: Option<PresetCamera>,
    /// `@fov`.
    pub field_of_view: Option<Angle>,
    /// `@zoom`.
    pub zoom: Option<Percentage>,
    /// `rot`.
    pub rotation: Option<SphereCoords>,
}

/// `a:CT_LightRig`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LightRig {
    /// `@rig`.
    pub rig: Option<LightRigType>,
    /// `@dir`.
    pub direction: Option<LightRigDirection>,
    /// `rot`.
    pub rotation: Option<SphereCoords>,
}

/// `a:CT_Backdrop`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Backdrop {
    /// `anchor`.
    pub anchor: Option<Point3D>,
    /// `norm`.
    pub normal: Option<Vector3D>,
    /// `up`.
    pub up: Option<Vector3D>,
}

/// `a:CT_Scene3D`. `extLst` stays in the tree.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Scene3D {
    /// `camera`.
    pub camera: Option<Camera>,
    /// `lightRig`.
    pub light_rig: Option<LightRig>,
    /// `backdrop`.
    pub backdrop: Option<Backdrop>,
}

/// `a:CT_Bevel`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bevel {
    /// `@w`.
    pub width: Option<Coordinate>,
    /// `@h`.
    pub height: Option<Coordinate>,
    /// `@prst`.
    pub preset: Option<BevelPreset>,
}

/// `a:CT_Shape3D`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Shape3D {
    /// `@z`.
    pub z: Option<Coordinate>,
    /// `@extrusionH`.
    pub extrusion_height: Option<Coordinate>,
    /// `@contourW`.
    pub contour_width: Option<Coordinate>,
    /// `@prstMaterial`.
    pub material: Option<PresetMaterial>,
    /// `bevelT`.
    pub bevel_top: Option<Bevel>,
    /// `bevelB`.
    pub bevel_bottom: Option<Bevel>,
    /// `extrusionClr`.
    pub extrusion_color: Option<Color>,
    /// `contourClr`.
    pub contour_color: Option<Color>,
}

/// `a:CT_FlatText`, for a text body's `flatTx`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FlatText {
    /// `@z`.
    pub z: Option<Coordinate>,
}
