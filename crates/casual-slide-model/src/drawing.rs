//! What a shape's properties hold beyond its transform: geometry, fill, line, effects
//! and 3D, attached to a [`ShapeNode`](crate::shape::ShapeNode).
//!
//! Each is `None` if the file did not state it. Inheritance from a placeholder or a
//! style matrix is the resolver's, and nothing here fills a default in.

use crate::d3::{Scene3D, Shape3D};
use crate::effect::EffectProperties;
use crate::fill::{BlipFill, Fill};
use crate::geometry::Geometry;
use crate::line::Line;

/// The `a14` extensions of `spPr`: properties a shape keeps but does not draw, so a
/// fill turned off in the UI can be turned on again.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HiddenProperties {
    /// `a14:hiddenFill`.
    pub fill: Option<Fill>,
    /// `a14:hiddenLine`.
    pub line: Option<Line>,
    /// `a14:hiddenEffects`.
    pub effects: Option<EffectProperties>,
    /// `a14:hiddenScene3d`.
    pub scene3d: Option<Scene3D>,
    /// `a14:hiddenSp3d`.
    pub sp3d: Option<Shape3D>,
    /// `a14:shadowObscured/@val`.
    pub shadow_obscured: Option<bool>,
}

/// `spPr` and `grpSpPr` beyond `xfrm` and `@bwMode`, and a picture's `p:blipFill`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShapeProperties {
    /// `prstGeom` or `custGeom`.
    pub geometry: Option<Geometry>,
    /// `EG_FillProperties`.
    pub fill: Option<Fill>,
    /// `ln`.
    pub line: Option<Box<Line>>,
    /// `effectLst` or `effectDag`.
    pub effects: Option<EffectProperties>,
    /// `scene3d`.
    pub scene3d: Option<Box<Scene3D>>,
    /// `sp3d`.
    pub sp3d: Option<Box<Shape3D>>,
    /// The `a14` extensions in `extLst`.
    pub hidden: Option<Box<HiddenProperties>>,
    /// A picture's `p:blipFill`, which sits beside `spPr` and not in it.
    pub picture: Option<Box<BlipFill>>,
}
