//! `scene3d` and `sp3d` as what the tree should say (`docs/spec/D3.md`).

use casual_slide_model::d3::{Bevel, FlatText, LightRig, Scene3D, Shape3D, SphereCoords, Vector3D};

use crate::drawing::{Node, Space};

// spec-audit: owner=rot,camera,lightRig
fn sphere(s: &SphereCoords) -> Node {
    Node::a("rot")
        .ang("lat", s.latitude)
        .ang("lon", s.longitude)
        .ang("rev", s.revolution)
}

// spec-audit: owner=norm,up
fn vector(local: &'static str, v: &Vector3D) -> Node {
    Node::a(local)
        .lex("dx", v.dx.as_ref())
        .lex("dy", v.dy.as_ref())
        .lex("dz", v.dz.as_ref())
}

/// `a:CT_LightRig`.
// spec-audit: owner=scene3d,hiddenScene3d,cell3D
pub(crate) fn light_rig(r: &LightRig) -> Node {
    Node::a("lightRig")
        .lex("rig", r.rig.as_ref())
        .lex("dir", r.direction.as_ref())
        .opt(r.rotation.as_ref().map(sphere))
}

/// `a:CT_Scene3D`, under the name it has here.
// spec-audit: owner=scene3d,hiddenScene3d
pub(crate) fn scene3d_node(s: &Scene3D, space: Space, local: &'static str) -> Node {
    let camera = s.camera.as_ref().map(|c| {
        Node::a("camera")
            .lex("prst", c.preset.as_ref())
            .ang("fov", c.field_of_view)
            .pct("zoom", c.zoom)
            .opt(c.rotation.as_ref().map(sphere))
    });
    let rig = s.light_rig.as_ref().map(light_rig);
    let backdrop = s.backdrop.as_ref().map(|b| {
        let anchor = b.anchor.as_ref().map(|p| {
            Node::a("anchor")
                .lex("x", p.x.as_ref())
                .lex("y", p.y.as_ref())
                .lex("z", p.z.as_ref())
        });
        Node::a("backdrop")
            .opt(anchor)
            .opt(b.normal.as_ref().map(|v| vector("norm", v)))
            .opt(b.up.as_ref().map(|v| vector("up", v)))
    });
    Node::new(space, local).opt(camera).opt(rig).opt(backdrop)
}

// spec-audit: owner=bevelT,bevelB,bevel
pub(crate) fn bevel(local: &'static str, b: &Bevel) -> Node {
    Node::a(local)
        .pos("w", b.width.as_ref())
        .pos("h", b.height.as_ref())
        .lex("prst", b.preset.as_ref())
}

/// `a:CT_Shape3D`, under the name it has here.
// spec-audit: owner=sp3d,hiddenSp3d
pub(crate) fn sp3d_node(s: &Shape3D, space: Space, local: &'static str) -> Node {
    Node::new(space, local)
        .lex("z", s.z.as_ref())
        .pos("extrusionH", s.extrusion_height.as_ref())
        .pos("contourW", s.contour_width.as_ref())
        .lex("prstMaterial", s.material.as_ref())
        .opt(s.bevel_top.as_ref().map(|b| bevel("bevelT", b)))
        .opt(s.bevel_bottom.as_ref().map(|b| bevel("bevelB", b)))
        .opt(
            s.extrusion_color
                .as_ref()
                .map(|c| Node::a("extrusionClr").color(Some(c))),
        )
        .opt(
            s.contour_color
                .as_ref()
                .map(|c| Node::a("contourClr").color(Some(c))),
        )
}

/// `a:CT_FlatText`, a text body's `flatTx`.
// spec-audit: owner=flatTx
pub(crate) fn flat_text_node(f: &FlatText) -> Node {
    Node::a("flatTx").lex("z", f.z.as_ref())
}
