//! Outer shadow and glow, the effects real decks use most, as display items behind
//! the shape.
//!
//! The display list has no blur, so a blur is stepped: the shape's outline filled
//! once faintly, then stroked in widening rings of falling opacity out to the blur
//! radius, which reads as a soft edge at slide sizes. That is an approximation, and
//! the report says so. Every other effect is reported by name and not drawn.

use casual_slide_model::color::Color;
use casual_slide_model::effect::{Effect, EffectProperties, OuterShadow};
use casual_slide_model::shape::ShapeNode;

use super::Builder;
use crate::display::{Affine, Cap, Join, Layer, Paint, PathCmd, Rgba, Stroke};

/// Rings a blur or glow is stepped into.
const BLUR_STEPS: usize = 6;

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
                            self.soft(node, layer, transform, p, c, r, false);
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
                self.soft(node, layer, at, p, color, blur / 2.0, true);
            } else {
                self.push(node, layer, at, p.to_vec(), Some(Paint::Solid(color)), None);
            }
        }
        if blur > 0.0 {
            self.report.note("shadow or glow blur (stepped)");
        }
    }

    /// A soft halo of `color` around `path`, reaching `reach` EMU out; `filled` also
    /// fills the inside, as a shadow does and a glow does not.
    #[allow(clippy::too_many_arguments)] // An outline, its place, look, reach and owner.
    fn soft(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        at: Affine,
        path: &[PathCmd],
        color: Rgba,
        reach: f64,
        filled: bool,
    ) {
        let alpha = f64::from(color.a);
        let tint = |a: f64| {
            Paint::Solid(Rgba {
                a: a.round().clamp(0.0, 255.0) as u8,
                ..color
            })
        };
        if filled {
            // The inside, less a step so the rings fade its edge.
            self.push(
                node,
                layer,
                at,
                path.to_vec(),
                Some(tint(alpha * 0.5)),
                None,
            );
        }
        // Rings from widest to narrowest: each adds a little, so the colour builds
        // towards the outline and fades outwards.
        for k in (1..=BLUR_STEPS).rev() {
            let width = 2.0 * reach * k as f64 / BLUR_STEPS as f64;
            let stroke = Stroke {
                paint: tint(alpha / (BLUR_STEPS as f64 * 0.9)),
                width,
                dash: Vec::new(),
                cap: Cap::Round,
                join: Join::Round,
            };
            self.push(node, layer, at, path.to_vec(), None, Some(stroke));
        }
    }
}
