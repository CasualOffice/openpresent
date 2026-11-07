//! `prstGeom` and `custGeom` as what the tree should say (`docs/spec/GEOM.md`).

use casual_slide_model::geometry::{
    AdjPoint, AdjustHandle, CustomGeometry, GeomGuide, Geometry, Path2D, PathCommand,
    PresetTextShape,
};

use crate::drawing::Node;

// spec-audit: owner=avLst,gdLst,gd
fn guides(local: &'static str, list: &[GeomGuide]) -> Node {
    Node::a(local).list(list.iter().map(|g| {
        Node::a("gd")
            .lex("name", g.name.as_ref())
            .lex("fmla", g.formula.as_ref())
    }))
}

// spec-audit: owner=pt,pos
fn point(local: &'static str, p: &AdjPoint) -> Node {
    Node::a(local).lex("x", p.x.as_ref()).lex("y", p.y.as_ref())
}

// spec-audit: owner=path,pathLst
fn command(c: &PathCommand) -> Node {
    match c {
        PathCommand::MoveTo(p) => Node::a("moveTo").opt(p.as_ref().map(|p| point("pt", p))),
        PathCommand::LineTo(p) => Node::a("lnTo").opt(p.as_ref().map(|p| point("pt", p))),
        PathCommand::ArcTo {
            width_radius,
            height_radius,
            start_angle,
            swing_angle,
        } => Node::a("arcTo")
            .lex("wR", width_radius.as_ref())
            .lex("hR", height_radius.as_ref())
            .lex("stAng", start_angle.as_ref())
            .lex("swAng", swing_angle.as_ref()),
        PathCommand::QuadBezTo(pts) => {
            Node::a("quadBezTo").list(pts.iter().map(|p| point("pt", p)))
        }
        PathCommand::CubicBezTo(pts) => {
            Node::a("cubicBezTo").list(pts.iter().map(|p| point("pt", p)))
        }
        PathCommand::Close => Node::a("close"),
    }
}

// spec-audit: owner=path,pathLst
fn path(p: &Path2D) -> Node {
    Node::a("path")
        .pos("w", p.w.as_ref())
        .pos("h", p.h.as_ref())
        .lex("fill", p.fill.as_ref())
        .lex("stroke", p.stroke.as_ref())
        .lex("extrusionOk", p.extrusion_ok.as_ref())
        .list(p.commands.iter().map(command))
}

// spec-audit: owner=custGeom,ahLst,cxnLst,ahXY,ahPolar,cxn,rect,pathLst
fn custom(g: &CustomGeometry) -> Node {
    let handles = g.handles.as_ref().map(|list| {
        Node::a("ahLst").list(list.iter().map(|h| {
            match h {
                AdjustHandle::Xy(h) => Node::a("ahXY")
                    .lex("gdRefX", h.gd_ref_x.as_ref())
                    .lex("minX", h.min_x.as_ref())
                    .lex("maxX", h.max_x.as_ref())
                    .lex("gdRefY", h.gd_ref_y.as_ref())
                    .lex("minY", h.min_y.as_ref())
                    .lex("maxY", h.max_y.as_ref())
                    .opt(h.pos.as_ref().map(|p| point("pos", p))),
                AdjustHandle::Polar(h) => Node::a("ahPolar")
                    .lex("gdRefR", h.gd_ref_r.as_ref())
                    .lex("minR", h.min_r.as_ref())
                    .lex("maxR", h.max_r.as_ref())
                    .lex("gdRefAng", h.gd_ref_ang.as_ref())
                    .lex("minAng", h.min_ang.as_ref())
                    .lex("maxAng", h.max_ang.as_ref())
                    .opt(h.pos.as_ref().map(|p| point("pos", p))),
            }
        }))
    });
    let sites = g.sites.as_ref().map(|list| {
        Node::a("cxnLst").list(list.iter().map(|s| {
            Node::a("cxn")
                .lex("ang", s.angle.as_ref())
                .opt(s.pos.as_ref().map(|p| point("pos", p)))
        }))
    });
    let rect = g.rect.as_ref().map(|r| {
        Node::a("rect")
            .lex("l", r.l.as_ref())
            .lex("t", r.t.as_ref())
            .lex("r", r.r.as_ref())
            .lex("b", r.b.as_ref())
    });
    Node::a("custGeom")
        .opt(g.adjust.as_ref().map(|l| guides("avLst", l)))
        .opt(g.guides.as_ref().map(|l| guides("gdLst", l)))
        .opt(handles)
        .opt(sites)
        .opt(rect)
        .opt(
            g.paths
                .as_ref()
                .map(|l| Node::a("pathLst").list(l.iter().map(path))),
        )
}

/// `EG_Geometry`.
// spec-audit: owner=prstGeom,custGeom,spPr
pub(crate) fn geometry_node(g: &Geometry) -> Node {
    match g {
        Geometry::Preset(p) => Node::a("prstGeom")
            .lex("prst", p.preset.as_ref())
            .opt(p.adjust.as_ref().map(|l| guides("avLst", l))),
        Geometry::Custom(c) => custom(c),
    }
}

/// `prstTxWarp`.
// spec-audit: owner=prstTxWarp
pub(crate) fn warp_node(w: &PresetTextShape) -> Node {
    Node::a("prstTxWarp")
        .lex("prst", w.preset.as_ref())
        .opt(w.adjust.as_ref().map(|list| guides("avLst", list)))
}
