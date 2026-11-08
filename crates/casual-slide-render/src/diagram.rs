//! A SmartArt diagram's stored drawing (`dsp:drawing`, the last layout PowerPoint
//! made) as shape nodes the scene draws like any other. No layout definition is run
//! (`SEC-C12`): what is drawn is what PowerPoint stored.
//!
//! A drawing shape's `txXfrm`, the box its text sits in when that differs from the
//! shape, becomes a second, unfilled and unstroked node holding the text, so the
//! scene's one text path lays it out.

use casual_slide_model::diagram::schema::{DspDrawing, DspGroupShape, DspGroupShapeItem, DspShape};
use casual_slide_model::fill::Fill;
use casual_slide_model::line::Line;
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode};

/// The deepest group nesting converted.
const MAX_DIAGRAM_DEPTH: usize = 32;

/// The most drawing shapes converted from one diagram.
const MAX_DIAGRAM_SHAPES: usize = 10_000;

/// A diagram's drawing as nodes in the frame's own space, each carrying the frame's
/// id, so an editor invalidates the frame.
pub(crate) fn drawing_nodes(frame: ObjectId, drawing: &DspDrawing) -> Vec<ShapeNode> {
    let mut out = Vec::new();
    let mut budget = MAX_DIAGRAM_SHAPES;
    if let Some(tree) = drawing.sp_tree.as_deref() {
        for item in &tree.items {
            convert(frame, item, 0, &mut budget, &mut out);
        }
    }
    out
}

fn convert(
    frame: ObjectId,
    item: &DspGroupShapeItem,
    depth: usize,
    budget: &mut usize,
    out: &mut Vec<ShapeNode>,
) {
    if *budget == 0 || depth > MAX_DIAGRAM_DEPTH {
        return;
    }
    *budget -= 1;
    match item {
        DspGroupShapeItem::Sp(sp) => shape(frame, sp, out),
        DspGroupShapeItem::GrpSp(g) => out.push(group(frame, g, depth, budget)),
    }
}

fn group(frame: ObjectId, g: &DspGroupShape, depth: usize, budget: &mut usize) -> ShapeNode {
    let mut node = ShapeNode::new(frame, NodeKind::Group);
    if let Some(p) = g.grp_sp_pr.as_deref() {
        node.transform = p.transform;
        node.properties = p.properties.clone();
    }
    for item in &g.items {
        convert(frame, item, depth + 1, budget, &mut node.children);
    }
    node
}

fn shape(frame: ObjectId, sp: &DspShape, out: &mut Vec<ShapeNode>) {
    let mut node = ShapeNode::new(frame, NodeKind::Shape);
    if let Some(p) = sp.sp_pr.as_deref() {
        node.transform = p.transform;
        node.properties = p.properties.clone();
    }
    node.style = sp.style.clone();
    let text = sp.tx_body.clone();
    match (sp.tx_xfrm.as_deref(), text) {
        (Some(t), Some(body)) => {
            let mut holder = ShapeNode::new(frame, NodeKind::Shape);
            holder.transform = Some(*t);
            holder.style = sp.style.clone();
            holder.properties.fill = Some(Fill::None);
            holder.properties.line = Some(Box::new(Line {
                fill: Some(Fill::None),
                ..Line::default()
            }));
            holder.text_body = Some(body);
            out.push(node);
            out.push(holder);
        }
        (None, body) => {
            node.text_body = body;
            out.push(node);
        }
        (Some(_), None) => out.push(node),
    }
}
