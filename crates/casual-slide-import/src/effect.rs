//! Read effect lists, effect DAGs and blip effects (`docs/spec/EFFECT.md`), and the
//! `a14` picture effects.
//!
//! A reference in a DAG is held by name and never followed, so a cycle written in a
//! file costs nothing here. Names are checked once a top-level container is read
//! (`MS-OI29500` 2.1.1291, 2.1.1294).

use std::collections::HashSet;

use casual_slide_model::color::{Color, ValueDomain};
use casual_slide_model::effect::{
    ArtisticEffect, BackgroundRemoval, Effect, EffectContainer, EffectProperties, ImageLayer,
    ImageProperties, OuterShadow, PictureEffect, PictureEffectKind, Reflection, RemovalMark,
    TransformEffect,
};
use casual_slide_model::fill::Fill;
use casual_slide_model::properties::Lexical;

use crate::color::{is_color_element, read_color};
use crate::drawing::{A14, Ctx, DrawingFinding, Res, is_fill_element};
use crate::properties::{Elem, ImportFindingKind as K, ns};

/// Which effects a place allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectScope {
    /// `EG_Effect`: all of them.
    Dag,
    /// `CT_EffectList`.
    List,
    /// `CT_Blip`.
    Blip,
}

impl EffectScope {
    fn allows(self, local: &str) -> bool {
        match self {
            Self::Dag => true,
            Self::List => matches!(
                local,
                "blur"
                    | "fillOverlay"
                    | "glow"
                    | "innerShdw"
                    | "outerShdw"
                    | "prstShdw"
                    | "reflection"
                    | "softEdge"
            ),
            Self::Blip => matches!(
                local,
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
            ),
        }
    }
}

impl Ctx<'_, '_> {
    /// `effectLst` or `effectDag`, which the caller has checked it is.
    pub(crate) fn effect_properties(&mut self, el: &Elem) -> Res<EffectProperties> {
        if el.local == "effectLst" {
            self.r.finish(el);
            let mut list = Vec::new();
            let mut seen = HashSet::new();
            for kid in self.kids(el, self.limits.max_effects)? {
                match self.effect(&kid, EffectScope::List)? {
                    Some(_) if !seen.insert(kid.local.clone()) => {
                        self.r.note(K::DuplicateElement, &kid, None);
                    }
                    Some(effect) => list.push(effect),
                    None => self.r.retained(&kid),
                }
            }
            return Ok(EffectProperties::List(list));
        }
        let dag = self.container(el)?;
        self.check_dag(el, &dag);
        Ok(EffectProperties::Dag(dag))
    }

    fn container(&mut self, el: &Elem) -> Res<EffectContainer> {
        self.enter()?;
        let mut e = el.clone();
        let mut c = EffectContainer {
            container_type: self.attr(&mut e, "type")?,
            name: self.attr(&mut e, "name")?,
            effects: Vec::new(),
        };
        self.r.finish(&e);
        for kid in self.kids(el, self.limits.max_effects)? {
            match self.effect(&kid, EffectScope::Dag)? {
                Some(effect) => c.effects.push(effect),
                None => self.r.retained(&kid),
            }
        }
        self.leave();
        Ok(c)
    }

    /// Names repeat, and references that name nothing. The depth is bounded by reading.
    fn check_dag(&mut self, el: &Elem, top: &EffectContainer) {
        const SPECIAL: [&str; 4] = ["fill", "fillLine", "line", "text"];
        let mut names = HashSet::new();
        let mut refs = Vec::new();
        let mut repeated = false;
        let mut stack = vec![top];
        while let Some(c) = stack.pop() {
            if let Some(name) = &c.name {
                repeated |= !names.insert(name.as_str());
            }
            for effect in &c.effects {
                match effect {
                    Effect::Container(inner) => stack.push(inner),
                    Effect::AlphaModulate(Some(inner))
                    | Effect::Blend {
                        container: Some(inner),
                        ..
                    } => stack.push(inner),
                    Effect::Reference(Some(name)) => refs.push(name.as_str()),
                    _ => {}
                }
            }
        }
        let dangling = refs
            .iter()
            .any(|name| !SPECIAL.contains(name) && !names.contains(name));
        if repeated || dangling {
            self.r
                .note(K::Drawing(DrawingFinding::EffectReference), el, None);
        }
    }

    fn container_child(&mut self, el: &Elem) -> Res<Option<EffectContainer>> {
        let mut found = None;
        self.sequence(el, |c, kid| {
            if kid.is(ns::DML, "cont") {
                found = Some(c.container(kid)?);
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        Ok(found)
    }

    fn fill_child(&mut self, el: &Elem) -> Res<Option<Fill>> {
        let mut found = None;
        for kid in self.all_kids(el)? {
            if !is_fill_element(&kid) {
                self.r.retained(&kid);
            } else if found.is_some() {
                self.r.note(K::DuplicateElement, &kid, None);
            } else {
                found = Some(self.fill(&kid)?);
            }
        }
        Ok(found)
    }

    /// One effect. `None` if it is not one, or not one this place allows.
    pub(crate) fn effect(&mut self, el: &Elem, scope: EffectScope) -> Res<Option<Effect>> {
        if !el.in_namespace(ns::DML) || !scope.allows(&el.local) {
            return Ok(None);
        }
        use ValueDomain as D;
        let mut e = el.clone();
        let effect = match el.local.as_str() {
            "cont" => Effect::Container(self.container(el)?),
            "effect" => {
                let v = Effect::Reference(self.req(&mut e, "ref")?);
                self.done(&e, el)?;
                v
            }
            "alphaBiLevel" => {
                let v = self.req_pct(&mut e, "thresh", D::PositiveFixedPercentage)?;
                self.done(&e, el)?;
                Effect::AlphaBiLevel(v)
            }
            "alphaCeiling" => {
                self.done(&e, el)?;
                Effect::AlphaCeiling
            }
            "alphaFloor" => {
                self.done(&e, el)?;
                Effect::AlphaFloor
            }
            "alphaInv" => {
                self.r.finish(&e);
                Effect::AlphaInverse(self.holder_color(el, false)?)
            }
            "alphaMod" => {
                self.r.finish(&e);
                Effect::AlphaModulate(self.container_child(el)?)
            }
            "alphaModFix" => {
                // MS-OI29500 2.1.1287: Office takes this modulo 100, not clamped. The
                // schema's domain is what is checked here; the renderer does the rest.
                let v = self.pct(&mut e, "amt", D::PositivePercentage)?;
                self.done(&e, el)?;
                Effect::AlphaModulateFixed(v)
            }
            "alphaOutset" => {
                let v = self.coord(&mut e, "rad")?;
                self.done(&e, el)?;
                Effect::AlphaOutset(v)
            }
            "alphaRepl" => {
                let v = self.req_pct(&mut e, "a", D::PositiveFixedPercentage)?;
                self.done(&e, el)?;
                Effect::AlphaReplace(v)
            }
            "biLevel" => {
                let v = self.req_pct(&mut e, "thresh", D::PositiveFixedPercentage)?;
                self.done(&e, el)?;
                Effect::BiLevel(v)
            }
            "blend" => {
                let blend = self.req(&mut e, "blend")?;
                self.r.finish(&e);
                Effect::Blend {
                    blend,
                    container: self.container_child(el)?,
                }
            }
            "blur" => {
                let radius = self.pos_coord(&mut e, "rad")?;
                let grow = self.attr(&mut e, "grow")?;
                self.done(&e, el)?;
                Effect::Blur { radius, grow }
            }
            "clrChange" => {
                let use_alpha = self.attr(&mut e, "useA")?;
                self.r.finish(&e);
                let (mut from, mut to) = (None, None);
                self.sequence(el, |c, kid| {
                    if !kid.in_namespace(ns::DML) {
                        return Ok(false);
                    }
                    match kid.local.as_str() {
                        "clrFrom" => from = c.holder_color(kid, true)?,
                        "clrTo" => to = c.holder_color(kid, true)?,
                        _ => return Ok(false),
                    }
                    c.r.finish(kid);
                    Ok(true)
                })?;
                Effect::ColorChange {
                    use_alpha,
                    from,
                    to,
                }
            }
            "clrRepl" => {
                self.r.finish(&e);
                Effect::ColorReplace(self.holder_color(el, true)?)
            }
            "duotone" => {
                self.r.finish(&e);
                Effect::Duotone(self.colors(el)?)
            }
            "fill" => {
                self.r.finish(&e);
                Effect::Fill(self.fill_child(el)?)
            }
            "fillOverlay" => {
                let blend = self.req(&mut e, "blend")?;
                self.r.finish(&e);
                Effect::FillOverlay {
                    blend,
                    fill: self.fill_child(el)?,
                }
            }
            "glow" => {
                let radius = self.pos_coord(&mut e, "rad")?;
                self.r.finish(&e);
                Effect::Glow {
                    radius,
                    color: self.holder_color(el, true)?,
                }
            }
            "grayscl" => {
                self.done(&e, el)?;
                Effect::Grayscale
            }
            "hsl" => {
                let hue = self.pos_angle(&mut e, "hue")?;
                let sat = self.pct(&mut e, "sat", D::FixedPercentage)?;
                let lum = self.pct(&mut e, "lum", D::FixedPercentage)?;
                self.done(&e, el)?;
                Effect::Hsl { hue, sat, lum }
            }
            "innerShdw" => {
                let blur_radius = self.pos_coord(&mut e, "blurRad")?;
                let distance = self.pos_coord(&mut e, "dist")?;
                let direction = self.pos_angle(&mut e, "dir")?;
                self.r.finish(&e);
                Effect::InnerShadow {
                    blur_radius,
                    distance,
                    direction,
                    color: self.holder_color(el, true)?,
                }
            }
            "lum" => {
                let bright = self.pct(&mut e, "bright", D::FixedPercentage)?;
                let contrast = self.pct(&mut e, "contrast", D::FixedPercentage)?;
                self.done(&e, el)?;
                Effect::Luminance { bright, contrast }
            }
            "outerShdw" => {
                let s = OuterShadow {
                    blur_radius: self.pos_coord(&mut e, "blurRad")?,
                    distance: self.pos_coord(&mut e, "dist")?,
                    direction: self.pos_angle(&mut e, "dir")?,
                    sx: self.pct(&mut e, "sx", D::Percentage)?,
                    sy: self.pct(&mut e, "sy", D::Percentage)?,
                    kx: self.fixed_angle(&mut e, "kx")?,
                    ky: self.fixed_angle(&mut e, "ky")?,
                    align: self.attr(&mut e, "algn")?,
                    rotate_with_shape: self.attr(&mut e, "rotWithShape")?,
                    color: None,
                };
                self.r.finish(&e);
                let color = self.holder_color(el, true)?;
                Effect::OuterShadow(Box::new(OuterShadow { color, ..s }))
            }
            "prstShdw" => {
                let preset = self.req(&mut e, "prst")?;
                let distance = self.pos_coord(&mut e, "dist")?;
                let direction = self.pos_angle(&mut e, "dir")?;
                self.r.finish(&e);
                Effect::PresetShadow {
                    preset,
                    distance,
                    direction,
                    color: self.holder_color(el, true)?,
                }
            }
            "reflection" => {
                let v = Reflection {
                    blur_radius: self.pos_coord(&mut e, "blurRad")?,
                    start_alpha: self.pct(&mut e, "stA", D::PositiveFixedPercentage)?,
                    start_position: self.pct(&mut e, "stPos", D::PositiveFixedPercentage)?,
                    end_alpha: self.pct(&mut e, "endA", D::PositiveFixedPercentage)?,
                    end_position: self.pct(&mut e, "endPos", D::PositiveFixedPercentage)?,
                    distance: self.pos_coord(&mut e, "dist")?,
                    direction: self.pos_angle(&mut e, "dir")?,
                    fade_direction: self.pos_angle(&mut e, "fadeDir")?,
                    sx: self.pct(&mut e, "sx", D::Percentage)?,
                    sy: self.pct(&mut e, "sy", D::Percentage)?,
                    kx: self.fixed_angle(&mut e, "kx")?,
                    ky: self.fixed_angle(&mut e, "ky")?,
                    align: self.attr(&mut e, "algn")?,
                    rotate_with_shape: self.attr(&mut e, "rotWithShape")?,
                };
                self.done(&e, el)?;
                Effect::Reflection(Box::new(v))
            }
            "relOff" => {
                let tx = self.pct(&mut e, "tx", D::Percentage)?;
                let ty = self.pct(&mut e, "ty", D::Percentage)?;
                self.done(&e, el)?;
                Effect::RelativeOffset { tx, ty }
            }
            "softEdge" => {
                if e.get("rad").is_none() {
                    self.r.note(K::MissingRequired, &e, Some("rad"));
                }
                let v = self.pos_coord(&mut e, "rad")?;
                self.done(&e, el)?;
                Effect::SoftEdge(v)
            }
            "tint" => {
                let hue = self.pos_angle(&mut e, "hue")?;
                let amount = self.pct(&mut e, "amt", D::FixedPercentage)?;
                self.done(&e, el)?;
                Effect::Tint { hue, amount }
            }
            "xfrm" => {
                let v = TransformEffect {
                    sx: self.pct(&mut e, "sx", D::Percentage)?,
                    sy: self.pct(&mut e, "sy", D::Percentage)?,
                    kx: self.fixed_angle(&mut e, "kx")?,
                    ky: self.fixed_angle(&mut e, "ky")?,
                    tx: self.coord(&mut e, "tx")?,
                    ty: self.coord(&mut e, "ty")?,
                };
                self.done(&e, el)?;
                Effect::Transform(Box::new(v))
            }
            _ => return Ok(None),
        };
        Ok(Some(effect))
    }

    /// Report what is left of an effect with no children.
    fn done(&mut self, e: &Elem, el: &Elem) -> Res<()> {
        self.r.finish(e);
        self.no_kids(el)
    }

    /// Each colour child, as `duotone` holds several.
    fn colors(&mut self, el: &Elem) -> Res<Vec<Color>> {
        let tree = self.r.tree;
        let mut out = Vec::new();
        for kid in self.all_kids(el)? {
            if is_color_element(tree, kid.id) {
                if let Some(color) = read_color(tree, kid.id, &mut self.color) {
                    out.push(color);
                }
            } else {
                self.r.retained(&kid);
            }
        }
        Ok(out)
    }

    /// `a14:imgProps`.
    pub(crate) fn image_properties(&mut self, el: &Elem) -> Res<ImageProperties> {
        self.r.finish(el);
        let mut p = ImageProperties::default();
        for kid in self.all_kids(el)? {
            if !kid.is(A14, "imgLayer") {
                self.r.retained(&kid);
            } else if p.layer.is_some() {
                self.r.note(K::DuplicateElement, &kid, None);
            } else {
                p.layer = Some(self.image_layer(&kid)?);
            }
        }
        Ok(p)
    }

    fn image_layer(&mut self, el: &Elem) -> Res<ImageLayer> {
        let mut e = el.clone();
        let mut layer = ImageLayer {
            embed: self.rel(&mut e, "embed"),
            effects: Vec::new(),
        };
        self.r.finish(&e);
        for kid in self.kids(el, self.limits.max_picture_effects)? {
            if !kid.is(A14, "imgEffect") {
                self.r.retained(&kid);
                continue;
            }
            let mut k = kid.clone();
            let visible = self.attr(&mut k, "visible")?;
            self.r.finish(&k);
            let mut kind = None;
            for choice in self.all_kids(&kid)? {
                if !choice.in_namespace(A14) {
                    self.r.retained(&choice);
                    continue;
                }
                match self.picture_effect(&choice)? {
                    Some(_) if kind.is_some() => {
                        self.r.note(K::DuplicateElement, &choice, None);
                    }
                    Some(found) => kind = Some(found),
                    None => self.r.retained(&choice),
                }
            }
            layer.effects.push(PictureEffect { visible, kind });
        }
        Ok(layer)
    }

    fn picture_effect(&mut self, el: &Elem) -> Res<Option<PictureEffectKind>> {
        use ValueDomain as D;
        let mut e = el.clone();
        let local = el.local.as_str();
        let kind = if let Some(effect) = ArtisticEffect::parse_lexical(local) {
            let transparency = if effect.has_transparency() {
                self.pct(&mut e, "trans", D::PositiveFixedPercentage)?
            } else {
                None
            };
            let (name, max) = effect.parameter();
            let parameter = self
                .r
                .attr_in::<i32>(&mut e, name, |v| (0..=max).contains(v))?;
            self.done(&e, el)?;
            PictureEffectKind::Artistic {
                effect,
                transparency,
                parameter,
            }
        } else {
            match local {
                "backgroundRemoval" => {
                    let mut b = BackgroundRemoval {
                        t: self.req_pct(&mut e, "t", D::PositiveFixedPercentage)?,
                        b: self.req_pct(&mut e, "b", D::PositiveFixedPercentage)?,
                        l: self.req_pct(&mut e, "l", D::PositiveFixedPercentage)?,
                        r: self.req_pct(&mut e, "r", D::PositiveFixedPercentage)?,
                        ..BackgroundRemoval::default()
                    };
                    self.r.finish(&e);
                    for mark in self.kids(el, self.limits.max_picture_effects)? {
                        let is_fg = mark.is(A14, "foregroundMark");
                        if !is_fg && !mark.is(A14, "backgroundMark") {
                            self.r.retained(&mark);
                            continue;
                        }
                        let mut m = mark.clone();
                        let value = RemovalMark {
                            x1: self.req_pct(&mut m, "x1", D::PositiveFixedPercentage)?,
                            y1: self.req_pct(&mut m, "y1", D::PositiveFixedPercentage)?,
                            x2: self.req_pct(&mut m, "x2", D::PositiveFixedPercentage)?,
                            y2: self.req_pct(&mut m, "y2", D::PositiveFixedPercentage)?,
                        };
                        self.done(&m, &mark)?;
                        if is_fg {
                            b.foreground.push(value);
                        } else {
                            b.background.push(value);
                        }
                    }
                    PictureEffectKind::BackgroundRemoval(b)
                }
                "brightnessContrast" => {
                    let bright = self.pct(&mut e, "bright", D::FixedPercentage)?;
                    let contrast = self.pct(&mut e, "contrast", D::FixedPercentage)?;
                    self.done(&e, el)?;
                    PictureEffectKind::BrightnessContrast { bright, contrast }
                }
                "colorTemperature" => {
                    let v = self
                        .r
                        .attr_in::<i32>(&mut e, "colorTemp", |v| (1_500..=11_500).contains(v))?;
                    self.done(&e, el)?;
                    PictureEffectKind::ColorTemperature(v)
                }
                "saturation" => {
                    let v = self.pct_range(&mut e, "sat", 0..=400_000)?;
                    self.done(&e, el)?;
                    PictureEffectKind::Saturation(v)
                }
                "sharpenSoften" => {
                    let v = self.pct(&mut e, "amount", D::FixedPercentage)?;
                    self.done(&e, el)?;
                    PictureEffectKind::SharpenSoften(v)
                }
                _ => return Ok(None),
            }
        };
        Ok(Some(kind))
    }
}
