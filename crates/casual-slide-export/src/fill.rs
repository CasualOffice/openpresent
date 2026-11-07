//! `EG_FillProperties` as what the tree should say (`docs/spec/FILL.md`).

use casual_slide_model::fill::{
    Blip, BlipFill, BlipFillMode, Fill, GradientFill, GradientShade, PatternFill, RelativeRect,
};

use casual_slide_model::picture::{ImageFormula, OEmbed, SvgBlip};

use crate::drawing::{Node, Space};
use crate::effect::{effect_node, image_properties_node};

// spec-audit: owner=tileRect,fillToRect,fillRect,srcRect
fn rect(local: &'static str, r: &RelativeRect) -> Node {
    Node::a(local)
        .pct("l", r.l)
        .pct("t", r.t)
        .pct("r", r.r)
        .pct("b", r.b)
}

// spec-audit: owner=gradFill,gs,gsLst,lin,path,ln,spPr
fn gradient(g: &GradientFill) -> Node {
    let shade = g.shade.as_ref().map(|shade| match shade {
        GradientShade::Linear { angle, scaled } => Node::a("lin")
            .ang("ang", *angle)
            .lex("scaled", scaled.as_ref()),
        GradientShade::Path { path, fill_to_rect } => Node::a("path")
            .lex("path", path.as_ref())
            .opt(fill_to_rect.as_ref().map(|r| rect("fillToRect", r))),
    });
    let stops = g.stops.as_ref().map(|stops| {
        Node::a("gsLst").list(
            stops
                .iter()
                .map(|s| Node::a("gs").pct("pos", s.position).color(s.color.as_ref())),
        )
    });
    Node::a("gradFill")
        .lex("flip", g.flip.as_ref())
        .lex("rotWithShape", g.rotate_with_shape.as_ref())
        .opt(stops)
        .opt(shade)
        .opt(g.tile_rect.as_ref().map(|r| rect("tileRect", r)))
}

// spec-audit: owner=pattFill,ln,spPr
fn pattern(p: &PatternFill) -> Node {
    Node::a("pattFill")
        .lex("prst", p.preset.as_ref())
        .opt(
            p.foreground
                .as_ref()
                .map(|c| Node::a("fgClr").color(Some(c))),
        )
        .opt(
            p.background
                .as_ref()
                .map(|c| Node::a("bgClr").color(Some(c))),
        )
}

fn svg_blip(s: &SvgBlip) -> Node {
    Node::extension("asvg:svgBlip")
        .rel("embed", s.embed.as_ref())
        .rel("link", s.link.as_ref())
}

fn image_formula(v: &ImageFormula) -> Node {
    Node::extension("aif:imageFormula")
        .lex("formula", v.formula.as_ref())
        .ext_uri(v.ext_uri.as_ref())
}

fn oembed(o: &OEmbed) -> Node {
    Node::extension("woe:oembed")
        .lex("oEmbedUrl", o.o_embed_url.as_ref())
        .lex("mediaType", o.media_type.as_ref())
        .lex(
            "picLocksAutoForOEmbed",
            o.pic_locks_auto_for_o_embed.as_ref(),
        )
        .ext_uri(o.ext_uri.as_ref())
}

/// `a:CT_Blip`.
// spec-audit: owner=blip
pub(crate) fn blip(b: &Blip) -> Node {
    // spec-audit: x:drawing/2010/main:<useLocalDpi>
    let local_dpi = b
        .use_local_dpi
        .map(|v| Node::new(Space::A14, "useLocalDpi").lex("val", Some(&v)));
    // A blip holds only these. Anything else is left out here, and refused by the read-back.
    let effects = b.effects.iter().map(effect_node).filter(|n| {
        matches!(
            n.local(),
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
        )
    });
    Node::a("blip")
        .rel("embed", b.embed.as_ref())
        .rel("link", b.link.as_ref())
        .lex("cstate", b.cstate.as_ref())
        .list(effects)
        .ext(b.image_properties.as_ref().map(image_properties_node))
        .ext(local_dpi)
        .ext(b.svg.as_ref().map(svg_blip))
        .ext(b.image_formula.as_ref().map(image_formula))
        .ext(b.oembed.as_ref().map(oembed))
}

/// `a:CT_BlipFillProperties`: `a:blipFill`, or a picture's `p:blipFill`.
// spec-audit: owner=blipFill,stretch,tile
pub(crate) fn blip_fill_node(f: &BlipFill, space: Space) -> Node {
    let mode = f.mode.as_ref().map(|mode| match mode {
        BlipFillMode::Tile(t) => Node::a("tile")
            .lex("tx", t.tx.as_ref())
            .lex("ty", t.ty.as_ref())
            .pct("sx", t.sx)
            .pct("sy", t.sy)
            .lex("flip", t.flip.as_ref())
            .lex("algn", t.align.as_ref()),
        BlipFillMode::Stretch(r) => Node::a("stretch").opt(r.as_ref().map(|r| rect("fillRect", r))),
    });
    Node::new(space, "blipFill")
        .lex("dpi", f.dpi.as_ref())
        .lex("rotWithShape", f.rotate_with_shape.as_ref())
        .opt(f.blip.as_ref().map(blip))
        .opt(f.source_rect.as_ref().map(|r| rect("srcRect", r)))
        .opt(mode)
}

/// One element of `EG_FillProperties`.
// spec-audit: owner=ln,spPr,grpSpPr
pub(crate) fn fill_node(fill: &Fill) -> Node {
    match fill {
        Fill::None => Node::a("noFill"),
        Fill::Group => Node::a("grpFill"),
        Fill::Solid(color) => Node::a("solidFill").color(color.as_ref()),
        Fill::Gradient(g) => gradient(g),
        Fill::Blip(b) => blip_fill_node(b, Space::Dml),
        Fill::Pattern(p) => pattern(p),
    }
}
