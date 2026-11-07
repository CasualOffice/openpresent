//! Effects as what the tree should say (`docs/spec/EFFECT.md`).

use casual_slide_model::effect::{
    ArtisticEffect, Effect, EffectContainer, EffectProperties, ImageProperties, PictureEffect,
    PictureEffectKind, RemovalMark,
};

use crate::drawing::{Node, Space, color_node};
use crate::fill::fill_node;

// spec-audit: owner=cont,effectDag,alphaMod,blend
fn container(local: &'static str, c: &EffectContainer) -> Node {
    Node::a(local)
        .lex("type", c.container_type.as_ref())
        .lex("name", c.name.as_ref())
        .list(c.effects.iter().map(effect_node))
}

/// One effect.
// spec-audit: owner=effectLst,effectDag,cont,blip,alphaMod,blend
pub(crate) fn effect_node(e: &Effect) -> Node {
    match e {
        Effect::Container(c) => container("cont", c),
        Effect::Reference(r) => Node::a("effect").lex("ref", r.as_ref()),
        Effect::AlphaBiLevel(v) => Node::a("alphaBiLevel").pct("thresh", *v),
        Effect::AlphaCeiling => Node::a("alphaCeiling"),
        Effect::AlphaFloor => Node::a("alphaFloor"),
        Effect::AlphaInverse(c) => Node::a("alphaInv").color(c.as_ref()),
        Effect::AlphaModulate(c) => {
            Node::a("alphaMod").opt(c.as_ref().map(|c| container("cont", c)))
        }
        Effect::AlphaModulateFixed(v) => Node::a("alphaModFix").pct("amt", *v),
        Effect::AlphaOutset(v) => Node::a("alphaOutset").lex("rad", v.as_ref()),
        Effect::AlphaReplace(v) => Node::a("alphaRepl").pct("a", *v),
        Effect::BiLevel(v) => Node::a("biLevel").pct("thresh", *v),
        Effect::Blend {
            blend,
            container: c,
        } => Node::a("blend")
            .lex("blend", blend.as_ref())
            .opt(c.as_ref().map(|c| container("cont", c))),
        Effect::Blur { radius, grow } => Node::a("blur")
            .pos("rad", radius.as_ref())
            .lex("grow", grow.as_ref()),
        Effect::ColorChange {
            use_alpha,
            from,
            to,
        } => Node::a("clrChange")
            .lex("useA", use_alpha.as_ref())
            .opt(from.as_ref().map(|c| Node::a("clrFrom").color(Some(c))))
            .opt(to.as_ref().map(|c| Node::a("clrTo").color(Some(c)))),
        Effect::ColorReplace(c) => Node::a("clrRepl").color(c.as_ref()),
        Effect::Duotone(colors) => Node::a("duotone").list(colors.iter().map(color_node)),
        Effect::Fill(f) => Node::a("fill").opt(f.as_ref().map(fill_node)),
        Effect::FillOverlay { blend, fill } => Node::a("fillOverlay")
            .lex("blend", blend.as_ref())
            .opt(fill.as_ref().map(fill_node)),
        Effect::Glow { radius, color } => Node::a("glow")
            .pos("rad", radius.as_ref())
            .color(color.as_ref()),
        Effect::Grayscale => Node::a("grayscl"),
        Effect::Hsl { hue, sat, lum } => Node::a("hsl")
            .ang("hue", *hue)
            .pct("sat", *sat)
            .pct("lum", *lum),
        Effect::InnerShadow {
            blur_radius,
            distance,
            direction,
            color,
        } => Node::a("innerShdw")
            .pos("blurRad", blur_radius.as_ref())
            .pos("dist", distance.as_ref())
            .ang("dir", *direction)
            .color(color.as_ref()),
        Effect::Luminance { bright, contrast } => Node::a("lum")
            .pct("bright", *bright)
            .pct("contrast", *contrast),
        Effect::OuterShadow(s) => Node::a("outerShdw")
            .pos("blurRad", s.blur_radius.as_ref())
            .pos("dist", s.distance.as_ref())
            .ang("dir", s.direction)
            .pct("sx", s.sx)
            .pct("sy", s.sy)
            .ang("kx", s.kx)
            .ang("ky", s.ky)
            .lex("algn", s.align.as_ref())
            .lex("rotWithShape", s.rotate_with_shape.as_ref())
            .color(s.color.as_ref()),
        Effect::PresetShadow {
            preset,
            distance,
            direction,
            color,
        } => Node::a("prstShdw")
            .lex("prst", preset.as_ref())
            .pos("dist", distance.as_ref())
            .ang("dir", *direction)
            .color(color.as_ref()),
        Effect::Reflection(r) => Node::a("reflection")
            .pos("blurRad", r.blur_radius.as_ref())
            .pct("stA", r.start_alpha)
            .pct("stPos", r.start_position)
            .pct("endA", r.end_alpha)
            .pct("endPos", r.end_position)
            .pos("dist", r.distance.as_ref())
            .ang("dir", r.direction)
            .ang("fadeDir", r.fade_direction)
            .pct("sx", r.sx)
            .pct("sy", r.sy)
            .ang("kx", r.kx)
            .ang("ky", r.ky)
            .lex("algn", r.align.as_ref())
            .lex("rotWithShape", r.rotate_with_shape.as_ref()),
        Effect::RelativeOffset { tx, ty } => Node::a("relOff").pct("tx", *tx).pct("ty", *ty),
        Effect::SoftEdge(v) => Node::a("softEdge").pos("rad", v.as_ref()),
        Effect::Tint { hue, amount } => Node::a("tint").ang("hue", *hue).pct("amt", *amount),
        Effect::Transform(t) => Node::a("xfrm")
            .pct("sx", t.sx)
            .pct("sy", t.sy)
            .ang("kx", t.kx)
            .ang("ky", t.ky)
            .lex("tx", t.tx.as_ref())
            .lex("ty", t.ty.as_ref()),
    }
}

/// `effectLst` or `effectDag`.
// spec-audit: owner=effectLst,effectDag
pub(crate) fn effect_properties_node(p: &EffectProperties) -> Node {
    match p {
        EffectProperties::List(list) => Node::a("effectLst").list(list.iter().map(effect_node)),
        EffectProperties::Dag(c) => container("effectDag", c),
    }
}

// spec-audit: owner=foregroundMark,backgroundMark
fn mark(local: &'static str, m: &RemovalMark) -> Node {
    Node::new(Space::A14, local)
        .pct("x1", m.x1)
        .pct("y1", m.y1)
        .pct("x2", m.x2)
        .pct("y2", m.y2)
}

/// The element an artistic effect is written as. The closed set the reader takes.
fn artistic_name(effect: ArtisticEffect) -> &'static str {
    use ArtisticEffect as A;
    match effect {
        A::Blur => "artisticBlur",
        A::Cement => "artisticCement",
        A::ChalkSketch => "artisticChalkSketch",
        A::CrisscrossEtching => "artisticCrisscrossEtching",
        A::Cutout => "artisticCutout",
        A::FilmGrain => "artisticFilmGrain",
        A::Glass => "artisticGlass",
        A::GlowDiffused => "artisticGlowDiffused",
        A::GlowEdges => "artisticGlowEdges",
        A::LightScreen => "artisticLightScreen",
        A::LineDrawing => "artisticLineDrawing",
        A::Marker => "artisticMarker",
        A::MosiaicBubbles => "artisticMosiaicBubbles",
        A::PaintStrokes => "artisticPaintStrokes",
        A::PaintBrush => "artisticPaintBrush",
        A::PastelsSmooth => "artisticPastelsSmooth",
        A::PencilGrayscale => "artisticPencilGrayscale",
        A::PencilSketch => "artisticPencilSketch",
        A::Photocopy => "artisticPhotocopy",
        A::PlasticWrap => "artisticPlasticWrap",
        A::Texturizer => "artisticTexturizer",
        A::WatercolorSponge => "artisticWatercolorSponge",
    }
}

// spec-audit: owner=imgEffect,imgLayer
fn picture_effect(e: &PictureEffect) -> Node {
    let kind = e.kind.as_ref().map(|kind| match kind {
        PictureEffectKind::Artistic {
            effect,
            transparency,
            parameter,
            // The element comes from `artistic_name`, the parameter from `ArtisticEffect`.
            // spec-audit: x:drawing/2010/main:CT_PictureEffectBlur/@radius
            // spec-audit: x:drawing/2010/main:CT_PictureEffectCement/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectCement/@crackSpacing
            // spec-audit: x:drawing/2010/main:CT_PictureEffectChalkSketch/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectChalkSketch/@pressure
            // spec-audit: x:drawing/2010/main:CT_PictureEffectCrisscrossEtching/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectCrisscrossEtching/@pressure
            // spec-audit: x:drawing/2010/main:CT_PictureEffectCutout/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectCutout/@numberOfShades
            // spec-audit: x:drawing/2010/main:CT_PictureEffectFilmGrain/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectFilmGrain/@grainSize
            // spec-audit: x:drawing/2010/main:CT_PictureEffectGlass/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectGlass/@scaling
            // spec-audit: x:drawing/2010/main:CT_PictureEffectGlowDiffused/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectGlowDiffused/@intensity
            // spec-audit: x:drawing/2010/main:CT_PictureEffectGlowEdges/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectGlowEdges/@smoothness
            // spec-audit: x:drawing/2010/main:CT_PictureEffectLightScreen/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectLightScreen/@gridSize
            // spec-audit: x:drawing/2010/main:CT_PictureEffectLineDrawing/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectLineDrawing/@pencilSize
            // spec-audit: x:drawing/2010/main:CT_PictureEffectMarker/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectMarker/@size
            // spec-audit: x:drawing/2010/main:CT_PictureEffectMosiaicBubbles/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectMosiaicBubbles/@pressure
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPaintBrush/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPaintBrush/@brushSize
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPaintStrokes/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPaintStrokes/@intensity
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPastelsSmooth/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPastelsSmooth/@scaling
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPencilGrayscale/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPencilGrayscale/@pencilSize
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPencilSketch/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPencilSketch/@pressure
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPhotocopy/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPhotocopy/@detail
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPlasticWrap/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectPlasticWrap/@smoothness
            // spec-audit: x:drawing/2010/main:CT_PictureEffectTexturizer/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectTexturizer/@scaling
            // spec-audit: x:drawing/2010/main:CT_PictureEffectWatercolorSponge/@trans
            // spec-audit: x:drawing/2010/main:CT_PictureEffectWatercolorSponge/@brushSize
            // spec-audit: x:drawing/2010/main:CT_PictureEffectBrightnessContrast/@bright
            // spec-audit: x:drawing/2010/main:CT_PictureEffectBrightnessContrast/@contrast
            // spec-audit: x:drawing/2010/main:CT_PictureEffectColorTemperature/@colorTemp
            // spec-audit: x:drawing/2010/main:CT_PictureEffectSaturation/@sat
            // spec-audit: x:drawing/2010/main:CT_PictureEffectSharpenSoften/@amount
        } => Node::new(Space::A14, artistic_name(*effect))
            .pct("trans", *transparency)
            .lex(effect.parameter().0, parameter.as_ref()),
        PictureEffectKind::BackgroundRemoval(b) => Node::new(Space::A14, "backgroundRemoval")
            .pct("t", b.t)
            .pct("b", b.b)
            .pct("l", b.l)
            .pct("r", b.r)
            .list(b.foreground.iter().map(|m| mark("foregroundMark", m)))
            .list(b.background.iter().map(|m| mark("backgroundMark", m))),
        PictureEffectKind::BrightnessContrast { bright, contrast } => {
            Node::new(Space::A14, "brightnessContrast")
                .pct("bright", *bright)
                .pct("contrast", *contrast)
        }
        PictureEffectKind::ColorTemperature(v) => {
            Node::new(Space::A14, "colorTemperature").lex("colorTemp", v.as_ref())
        }
        PictureEffectKind::Saturation(v) => Node::new(Space::A14, "saturation").pct("sat", *v),
        PictureEffectKind::SharpenSoften(v) => {
            Node::new(Space::A14, "sharpenSoften").pct("amount", *v)
        }
    });
    Node::new(Space::A14, "imgEffect")
        .lex("visible", e.visible.as_ref())
        .opt(kind)
}

/// `a14:imgProps`.
// spec-audit: owner=imgProps,imgLayer
pub(crate) fn image_properties_node(p: &ImageProperties) -> Node {
    let layer = p.layer.as_ref().map(|l| {
        Node::new(Space::A14, "imgLayer")
            .rel("embed", l.embed.as_ref())
            .list(l.effects.iter().map(picture_effect))
    });
    // spec-audit: x:drawing/2010/main:<imgProps>
    Node::new(Space::A14, "imgProps").opt(layer)
}
