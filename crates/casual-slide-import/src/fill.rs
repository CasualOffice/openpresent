//! Read `EG_FillProperties` (`docs/spec/FILL.md`) into the model.

use casual_slide_model::color::ValueDomain;
use casual_slide_model::fill::{
    Blip, BlipFill, BlipFillMode, Fill, GradientFill, GradientShade, GradientStop, PatternFill,
    RelativeRect, TileInfo,
};

use crate::drawing::{A14, Ctx, Res};
use crate::effect::EffectScope;
use crate::properties::{Elem, ImportFindingKind as K, ns};

impl Ctx<'_, '_> {
    /// One element of `EG_FillProperties`, which the caller has checked it is.
    pub(crate) fn fill(&mut self, el: &Elem) -> Res<Fill> {
        self.enter()?;
        let fill = match el.local.as_str() {
            "noFill" => {
                self.r.finish(el);
                self.no_kids(el)?;
                Fill::None
            }
            "grpFill" => {
                self.r.finish(el);
                self.no_kids(el)?;
                Fill::Group
            }
            "solidFill" => {
                self.r.finish(el);
                Fill::Solid(self.holder_color(el, false)?)
            }
            "gradFill" => Fill::Gradient(Box::new(self.gradient(el)?)),
            "blipFill" => Fill::Blip(Box::new(self.blip_fill(el)?)),
            _ => Fill::Pattern(Box::new(self.pattern(el)?)),
        };
        self.leave();
        Ok(fill)
    }

    fn gradient(&mut self, el: &Elem) -> Res<GradientFill> {
        let mut e = el.clone();
        let mut g = GradientFill {
            flip: self.attr(&mut e, "flip")?,
            rotate_with_shape: self.attr(&mut e, "rotWithShape")?,
            ..GradientFill::default()
        };
        self.r.finish(&e);
        self.sequence(el, |c, kid| {
            if !kid.in_namespace(ns::DML) {
                return Ok(false);
            }
            match kid.local.as_str() {
                "gsLst" => g.stops = Some(c.stops(kid)?),
                "lin" | "path" if g.shade.is_some() => {
                    c.r.note(K::DuplicateElement, kid, None);
                }
                "lin" => {
                    let mut k = kid.clone();
                    g.shade = Some(GradientShade::Linear {
                        angle: c.pos_angle(&mut k, "ang")?,
                        scaled: c.attr(&mut k, "scaled")?,
                    });
                    c.r.finish(&k);
                    c.no_kids(kid)?;
                }
                "path" => {
                    let mut k = kid.clone();
                    let path = c.attr(&mut k, "path")?;
                    c.r.finish(&k);
                    let mut fill_to_rect = None;
                    c.sequence(kid, |c, r| {
                        if r.is(ns::DML, "fillToRect") {
                            fill_to_rect = Some(c.relative_rect(r)?);
                            Ok(true)
                        } else {
                            Ok(false)
                        }
                    })?;
                    g.shade = Some(GradientShade::Path { path, fill_to_rect });
                }
                "tileRect" => g.tile_rect = Some(c.relative_rect(kid)?),
                _ => return Ok(false),
            }
            Ok(true)
        })?;
        Ok(g)
    }

    fn stops(&mut self, el: &Elem) -> Res<Vec<GradientStop>> {
        self.r.finish(el);
        let kids = self.kids(el, self.limits.max_gradient_stops)?;
        let mut stops = Vec::new();
        for kid in kids {
            if !kid.is(ns::DML, "gs") {
                self.r.retained(&kid);
                continue;
            }
            let mut k = kid.clone();
            let position = self.req_pct(&mut k, "pos", ValueDomain::PositiveFixedPercentage)?;
            self.r.finish(&k);
            stops.push(GradientStop {
                position,
                color: self.holder_color(&kid, true)?,
            });
        }
        Ok(stops)
    }

    pub(crate) fn relative_rect(&mut self, el: &Elem) -> Res<RelativeRect> {
        let mut e = el.clone();
        let rect = RelativeRect {
            l: self.pct(&mut e, "l", ValueDomain::Percentage)?,
            t: self.pct(&mut e, "t", ValueDomain::Percentage)?,
            r: self.pct(&mut e, "r", ValueDomain::Percentage)?,
            b: self.pct(&mut e, "b", ValueDomain::Percentage)?,
        };
        self.r.finish(&e);
        self.no_kids(el)?;
        Ok(rect)
    }

    fn pattern(&mut self, el: &Elem) -> Res<PatternFill> {
        let mut e = el.clone();
        let mut p = PatternFill {
            preset: self.attr(&mut e, "prst")?,
            ..PatternFill::default()
        };
        self.r.finish(&e);
        self.sequence(el, |c, kid| {
            if !kid.in_namespace(ns::DML) {
                return Ok(false);
            }
            match kid.local.as_str() {
                "fgClr" => {
                    c.r.finish(kid);
                    p.foreground = c.holder_color(kid, true)?;
                }
                "bgClr" => {
                    c.r.finish(kid);
                    p.background = c.holder_color(kid, true)?;
                }
                _ => return Ok(false),
            }
            Ok(true)
        })?;
        Ok(p)
    }

    /// `a:CT_BlipFillProperties`, under `a:blipFill` or `p:blipFill`.
    pub(crate) fn blip_fill(&mut self, el: &Elem) -> Res<BlipFill> {
        let mut e = el.clone();
        let mut b = BlipFill {
            dpi: self.attr(&mut e, "dpi")?,
            rotate_with_shape: self.attr(&mut e, "rotWithShape")?,
            ..BlipFill::default()
        };
        self.r.finish(&e);
        self.sequence(el, |c, kid| {
            if !kid.in_namespace(ns::DML) {
                return Ok(false);
            }
            match kid.local.as_str() {
                "blip" => b.blip = Some(c.blip(kid)?),
                "srcRect" => b.source_rect = Some(c.relative_rect(kid)?),
                "tile" | "stretch" if b.mode.is_some() => {
                    c.r.note(K::DuplicateElement, kid, None);
                }
                "tile" => {
                    let mut k = kid.clone();
                    b.mode = Some(BlipFillMode::Tile(TileInfo {
                        tx: c.coord(&mut k, "tx")?,
                        ty: c.coord(&mut k, "ty")?,
                        sx: c.pct(&mut k, "sx", ValueDomain::Percentage)?,
                        sy: c.pct(&mut k, "sy", ValueDomain::Percentage)?,
                        flip: c.attr(&mut k, "flip")?,
                        align: c.attr(&mut k, "algn")?,
                    }));
                    c.r.finish(&k);
                    c.no_kids(kid)?;
                }
                "stretch" => {
                    c.r.finish(kid);
                    let mut rect = None;
                    c.sequence(kid, |c, r| {
                        if r.is(ns::DML, "fillRect") {
                            rect = Some(c.relative_rect(r)?);
                            Ok(true)
                        } else {
                            Ok(false)
                        }
                    })?;
                    b.mode = Some(BlipFillMode::Stretch(rect));
                }
                _ => return Ok(false),
            }
            Ok(true)
        })?;
        Ok(b)
    }

    fn blip(&mut self, el: &Elem) -> Res<Blip> {
        let mut e = el.clone();
        let mut b = Blip {
            embed: self.rel(&mut e, "embed"),
            link: self.rel(&mut e, "link"),
            compression: self.attr(&mut e, "cstate")?,
            ..Blip::default()
        };
        self.r.finish(&e);
        let kids = self.kids(el, self.limits.max_effects)?;
        let mut seen_ext = false;
        for kid in kids {
            if kid.is(ns::DML, "extLst") {
                if seen_ext {
                    self.r.note(K::DuplicateElement, &kid, None);
                    continue;
                }
                seen_ext = true;
                self.blip_extensions(&kid, &mut b)?;
            } else if let Some(effect) = self.effect(&kid, EffectScope::Blip)? {
                b.effects.push(effect);
            } else {
                self.r.retained(&kid);
            }
        }
        Ok(b)
    }

    fn blip_extensions(&mut self, el: &Elem, b: &mut Blip) -> Res<()> {
        for item in self.extension_items(el)? {
            if item.is(A14, "imgProps") && b.image_properties.is_none() {
                b.image_properties = Some(self.image_properties(&item)?);
            } else if item.is(A14, "useLocalDpi") && b.use_local_dpi.is_none() {
                let mut k = item.clone();
                b.use_local_dpi = self.attr(&mut k, "val")?;
                self.r.finish(&k);
            } else {
                // `asvg:svgBlip` and the rest: Tier 1H, or nobody's.
                self.r.retained(&item);
            }
        }
        Ok(())
    }
}
