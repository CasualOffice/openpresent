//! Read `EG_FillProperties` (`docs/spec/FILL.md`) into the model.

use casual_slide_model::color::ValueDomain;
use casual_slide_model::fill::{
    Blip, BlipFill, BlipFillMode, Fill, GradientFill, GradientShade, GradientStop, PatternFill,
    RelativeRect, TileInfo,
};

use casual_slide_model::picture::{ImageFormula, OEmbed, SvgBlip};

use crate::drawing::{A14, Ctx, Res};
use crate::effect::EffectScope;
use crate::media::{Expect, MediaFinding, is_extension, read_rel};
use crate::properties::{Elem, ImportFindingKind as K, ns};

impl Ctx<'_, '_> {
    /// One element of `EG_FillProperties`, which the caller has checked it is.
    // spec-audit: owner=ln,spPr,grpSpPr,fillOverlay,rPr,defRPr,endParaRPr
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

    // spec-audit: owner=gradFill,lin,path
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

    // spec-audit: owner=gsLst,gs
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

    // spec-audit: owner=tileRect,fillToRect,fillRect,srcRect
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

    // spec-audit: owner=pattFill
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
    // spec-audit: owner=blipFill,stretch,tile
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

    // spec-audit: owner=blip
    pub(crate) fn blip(&mut self, el: &Elem) -> Res<Blip> {
        let mut e = el.clone();
        let mut b = Blip {
            embed: read_rel(self.r, self.rels, &mut e, "embed", Expect::Image),
            link: read_rel(self.r, self.rels, &mut e, "link", Expect::Image),
            cstate: self.attr(&mut e, "cstate")?,
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
            } else if matches!(
                kid.local.as_str(),
                "alphaBiLevel"
                    | "alphaCeiling"
                    | "alphaFloor"
                    | "alphaInv"
                    | "alphaMod"
                    | "alphaModFix"
                    | "alphaRepl"
                    | "biLevel"
                    | "blur"
                    | "clrChange"
                    | "clrRepl"
                    | "duotone"
                    | "fillOverlay"
                    | "grayscl"
                    | "hsl"
                    | "lum"
                    | "tint"
            ) && let Some(effect) = self.effect(&kid, EffectScope::Blip)?
            {
                b.effects.push(effect);
            } else {
                self.r.retained(&kid);
            }
        }
        Ok(b)
    }

    /// A reference. Nothing here, or anywhere, parses what it reaches.
    fn svg_blip(&mut self, item: &Elem, no_fallback: bool) -> Res<SvgBlip> {
        let mut k = item.clone();
        let svg = SvgBlip {
            embed: read_rel(self.r, self.rels, &mut k, "embed", Expect::Image),
            link: read_rel(self.r, self.rels, &mut k, "link", Expect::Image),
        };
        self.r.finish(&k);
        self.no_kids(item)?;
        if no_fallback {
            self.r
                .note(K::Media(MediaFinding::SvgWithoutFallback), item, None);
        }
        Ok(svg)
    }

    fn image_formula(&mut self, item: &Elem) -> Res<ImageFormula> {
        let mut k = item.clone();
        let formula = ImageFormula {
            formula: self.attr(&mut k, "formula")?,
            ext_uri: self.holder_uri(item),
        };
        self.r.finish(&k);
        self.no_kids(item)?;
        Ok(formula)
    }

    fn oembed(&mut self, item: &Elem) -> Res<OEmbed> {
        let mut k = item.clone();
        let oembed = OEmbed {
            o_embed_url: self.r.required(&mut k, "oEmbedUrl", |_: &String| true)?,
            media_type: self.r.required(&mut k, "mediaType", |_: &String| true)?,
            pic_locks_auto_for_o_embed: self.attr(&mut k, "picLocksAutoForOEmbed")?,
            ext_uri: self.holder_uri(item),
        };
        self.r.finish(&k);
        self.no_kids(item)?;
        if oembed.o_embed_url.is_some() {
            self.r.note(
                K::Media(MediaFinding::ExternalTarget),
                item,
                Some("oEmbedUrl"),
            );
        }
        Ok(oembed)
    }

    /// `@uri` of the `a:ext` an extension's item sits in.
    fn holder_uri(&self, item: &Elem) -> Option<String> {
        let ext = self.r.tree.parent_element(item.id)?;
        self.r.tree.plain_attribute(ext, "uri")
    }

    // spec-audit: owner=useLocalDpi
    fn blip_extensions(&mut self, el: &Elem, b: &mut Blip) -> Res<()> {
        for item in self.extension_items(el)? {
            if item.is(A14, "imgProps") && b.image_properties.is_none() {
                b.image_properties = Some(self.image_properties(&item)?);
            } else if item.is(A14, "useLocalDpi") && b.use_local_dpi.is_none() {
                let mut k = item.clone();
                b.use_local_dpi = self.attr(&mut k, "val")?;
                self.r.finish(&k);
            } else if is_extension(&item, "asvg:svgBlip") && b.svg.is_none() {
                b.svg = Some(self.svg_blip(&item, b.embed.is_none() && b.link.is_none())?);
            } else if is_extension(&item, "aif:imageFormula") && b.image_formula.is_none() {
                b.image_formula = Some(self.image_formula(&item)?);
            } else if is_extension(&item, "woe:oembed") && b.oembed.is_none() {
                b.oembed = Some(self.oembed(&item)?);
            } else {
                // Nobody's: it stays in the tree.
                self.r.retained(&item);
            }
        }
        Ok(())
    }
}
