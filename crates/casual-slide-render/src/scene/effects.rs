//! Outer shadow and glow, the effects real decks use most, as display items behind
//! the shape.
//!
//! A blurred shadow or glow is a [`DisplayItem::Blurred`]: the silhouette in one
//! colour and a Gaussian's deviation, which the raster backend blurs and a backend
//! with no blur steps into rings. The deviation is Inferred: a quarter of `blurRad`
//! for a shadow, so its soft edge spans the radius across the outline; a glow grows
//! the outline by half `rad` and blurs by a quarter, so it fades out to `rad`.
//! Every other effect is reported by name and not drawn.

use casual_slide_model::color::Color;
use casual_slide_model::effect::{Effect, EffectProperties, OuterShadow};
use casual_slide_model::shape::ShapeNode;

use super::Builder;
use crate::display::{Affine, DisplayItem, Layer, Paint, PathCmd, Rgba};

impl<'a> Builder<'a> {
    /// Draw the shadows and glows of `effects` for the outlines `paths` draw, and
    /// report the rest.
    pub(super) fn effects(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        transform: Affine,
        effects: &EffectProperties,
        ph: Option<&Color>,
        paths: &[&[PathCmd]],
    ) {
        let EffectProperties::List(list) = effects else {
            self.report.note("effect DAG");
            return;
        };
        for effect in list {
            match effect {
                Effect::OuterShadow(s) if !paths.is_empty() => {
                    self.outer_shadow(node, layer, transform, s, ph, paths);
                }
                Effect::Glow { radius, color } if !paths.is_empty() => {
                    let r = radius.map_or(0.0, |r| r.0.max(0) as f64);
                    let c = color
                        .as_ref()
                        .and_then(|c| self.color(c, ph))
                        .unwrap_or(Rgba {
                            r: 0,
                            g: 0,
                            b: 0,
                            a: 255,
                        });
                    if r > 0.0 {
                        for p in paths {
                            self.blurred(node, layer, transform, p, c, false, r / 2.0, r / 4.0);
                        }
                    }
                }
                Effect::OuterShadow(_) | Effect::Glow { .. } => {
                    self.report.note("effect on text (not drawn)");
                }
                Effect::InnerShadow { .. } => self.report.note("inner shadow"),
                Effect::PresetShadow { .. } => self.report.note("preset shadow"),
                Effect::Reflection(_) => self.report.note("reflection"),
                Effect::SoftEdge(_) => self.report.note("soft edge"),
                Effect::Blur { .. } => self.report.note("blur"),
                _ => self.report.note("effects"),
            }
        }
    }

    fn outer_shadow(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        transform: Affine,
        s: &OuterShadow,
        ph: Option<&Color>,
        paths: &[&[PathCmd]],
    ) {
        // The schema's defaults: black, no blur, no distance.
        let color = s
            .color
            .as_ref()
            .and_then(|c| self.color(c, ph))
            .unwrap_or(Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            });
        let blur = s.blur_radius.map_or(0.0, |b| b.0.max(0) as f64);
        let dist = s.distance.map_or(0.0, |d| d.0 as f64);
        let dir = s.direction.map_or(0.0, |a| f64::from(a.0) / 60_000.0);
        let (dy, dx) = dir.to_radians().sin_cos();
        let scaled = [s.sx, s.sy]
            .iter()
            .any(|p| p.is_some_and(|p| p.0 != 100_000))
            || [s.kx, s.ky].iter().any(|a| a.is_some_and(|a| a.0 != 0));
        if scaled {
            self.report.note("shadow scale or skew (drawn unscaled)");
        }
        // The offset is in slide space: a shadow falls the same way however the
        // shape is turned, unless `rotWithShape`, which this does not distinguish.
        let at = Affine::translate(dx * dist, dy * dist).then_after(&transform);
        for p in paths {
            if blur > 0.0 {
                self.blurred(node, layer, at, p, color, true, 0.0, blur / 4.0);
            } else {
                self.push(node, layer, at, p.to_vec(), Some(Paint::Solid(color)), None);
            }
        }
    }

    /// The silhouette of `path` in `color`, grown by `grow` and blurred by a
    /// Gaussian of deviation `sigma`, both EMU; `filled` keeps the inside, as a
    /// shadow does and a glow does not.
    #[allow(clippy::too_many_arguments)] // An outline, its place, look, blur and owner.
    fn blurred(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        at: Affine,
        path: &[PathCmd],
        color: Rgba,
        filled: bool,
        grow: f64,
        sigma: f64,
    ) {
        self.items.push(DisplayItem::Blurred {
            object: Some(node.id),
            layer,
            transform: at,
            path: path.to_vec(),
            color,
            filled,
            grow,
            sigma,
        });
    }
}
