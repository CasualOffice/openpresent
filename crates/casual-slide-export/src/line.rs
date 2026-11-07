//! `a:CT_LineProperties` as what the tree should say (`docs/spec/LINE.md`).

use casual_slide_model::line::{Line, LineDash, LineEnd, LineJoin, LineSketch, LineSketchType};

use crate::drawing::{Node, Space};
use crate::fill::fill_node;
use crate::geometry::geometry_node;

// spec-audit: owner=headEnd,tailEnd
fn end(local: &'static str, e: &LineEnd) -> Node {
    Node::a(local)
        .lex("type", e.end_type.as_ref())
        .lex("w", e.width.as_ref())
        .lex("len", e.length.as_ref())
}

// spec-audit: owner=lineSketchStyleProps,type
fn sketch(s: &LineSketch) -> Node {
    let kind = s.sketch_type.as_ref().map(|choice| {
        let picked = choice.map(|c| {
            Node::new(
                Space::Ask,
                match c {
                    LineSketchType::None => "lineSketchNone",
                    LineSketchType::Curved => "lineSketchCurved",
                    LineSketchType::Freehand => "lineSketchFreehand",
                    LineSketchType::Scribble => "lineSketchScribble",
                },
            )
        });
        Node::new(Space::Ask, "type").opt(picked)
    });
    let seed = s
        .seed
        .map(|v| Node::new(Space::Ask, "seed").text(Some(v.to_string())));
    Node::new(Space::Ask, "lineSketchStyleProps")
        .lex("sd", s.seed_attribute.as_ref())
        .opt(s.geometry.as_ref().map(geometry_node))
        .opt(kind)
        .opt(seed)
}

/// `a:CT_LineProperties`, under the name it has here.
// spec-audit: owner=ln,hiddenLine,ds,custDash,prstDash,miter
pub(crate) fn line_node(l: &Line, space: Space, local: &'static str) -> Node {
    let dash = l.dash.as_ref().map(|d| match d {
        LineDash::Preset(v) => Node::a("prstDash").lex("val", v.as_ref()),
        LineDash::Custom(stops) => Node::a("custDash").list(
            stops
                .iter()
                .map(|s| Node::a("ds").dash("d", s.dash).dash("sp", s.space)),
        ),
    });
    let join = l.join.map(|j| match j {
        LineJoin::Round => Node::a("round"),
        LineJoin::Bevel => Node::a("bevel"),
        LineJoin::Miter(lim) => Node::a("miter").pct("lim", lim),
    });
    Node::new(space, local)
        .lex("w", l.width.as_ref())
        .lex("cap", l.cap.as_ref())
        .lex("cmpd", l.compound.as_ref())
        .lex("algn", l.align.as_ref())
        .opt(l.fill.as_ref().map(fill_node))
        .opt(dash)
        .opt(join)
        .opt(l.head_end.as_ref().map(|e| end("headEnd", e)))
        .opt(l.tail_end.as_ref().map(|e| end("tailEnd", e)))
        .ext(l.sketch.as_ref().map(sketch))
}
