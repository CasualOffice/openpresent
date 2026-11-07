//! Read `scene3d` and `sp3d` (`docs/spec/D3.md`). BEH-27: held, never flattened.

use casual_slide_model::color::ValueDomain;
use casual_slide_model::d3::{
    Backdrop, Bevel, Camera, FlatText, LightRig, Point3D, Scene3D, Shape3D, SphereCoords, Vector3D,
};

use crate::drawing::{Ctx, Res};
use crate::properties::{Elem, ImportFindingKind as K, ns};

impl Ctx<'_, '_> {
    /// `a:CT_Scene3D`, under any name.
    pub(crate) fn scene3d(&mut self, el: &Elem) -> Res<Scene3D> {
        self.r.finish(el);
        let mut s = Scene3D::default();
        self.sequence(el, |c, kid| {
            if !kid.in_namespace(ns::DML) {
                return Ok(false);
            }
            match kid.local.as_str() {
                "camera" => s.camera = Some(c.camera(kid)?),
                "lightRig" => s.light_rig = Some(c.light_rig(kid)?),
                "backdrop" => s.backdrop = Some(c.backdrop(kid)?),
                // `extLst` stays in the tree.
                _ => return Ok(false),
            }
            Ok(true)
        })?;
        Ok(s)
    }

    fn sphere(&mut self, el: &Elem) -> Res<SphereCoords> {
        let mut e = el.clone();
        let has = |e: &Elem, n: &str| e.get(n).is_some();
        for name in ["lat", "lon", "rev"] {
            if !has(&e, name) {
                self.r.note(K::MissingRequired, &e, Some(name));
            }
        }
        let s = SphereCoords {
            latitude: self.pos_angle(&mut e, "lat")?,
            longitude: self.pos_angle(&mut e, "lon")?,
            revolution: self.pos_angle(&mut e, "rev")?,
        };
        self.r.finish(&e);
        self.no_kids(el)?;
        Ok(s)
    }

    fn camera(&mut self, el: &Elem) -> Res<Camera> {
        let mut e = el.clone();
        let mut cam = Camera {
            preset: self.req(&mut e, "prst")?,
            field_of_view: self.fov_angle(&mut e, "fov")?,
            zoom: self.pct(&mut e, "zoom", ValueDomain::PositivePercentage)?,
            rotation: None,
        };
        self.r.finish(&e);
        self.sequence(el, |c, kid| {
            if kid.is(ns::DML, "rot") {
                cam.rotation = Some(c.sphere(kid)?);
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        Ok(cam)
    }

    fn light_rig(&mut self, el: &Elem) -> Res<LightRig> {
        let mut e = el.clone();
        let mut rig = LightRig {
            rig: self.req(&mut e, "rig")?,
            direction: self.req(&mut e, "dir")?,
            rotation: None,
        };
        self.r.finish(&e);
        self.sequence(el, |c, kid| {
            if kid.is(ns::DML, "rot") {
                rig.rotation = Some(c.sphere(kid)?);
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        Ok(rig)
    }

    fn backdrop(&mut self, el: &Elem) -> Res<Backdrop> {
        self.r.finish(el);
        let mut b = Backdrop::default();
        self.sequence(el, |c, kid| {
            if !kid.in_namespace(ns::DML) {
                return Ok(false);
            }
            match kid.local.as_str() {
                "anchor" => {
                    let mut k = kid.clone();
                    b.anchor = Some(Point3D {
                        x: c.req_coord(&mut k, "x")?,
                        y: c.req_coord(&mut k, "y")?,
                        z: c.req_coord(&mut k, "z")?,
                    });
                    c.r.finish(&k);
                    c.no_kids(kid)?;
                }
                "norm" => b.normal = Some(c.vector(kid)?),
                "up" => b.up = Some(c.vector(kid)?),
                _ => return Ok(false),
            }
            Ok(true)
        })?;
        Ok(b)
    }

    fn vector(&mut self, el: &Elem) -> Res<Vector3D> {
        let mut e = el.clone();
        let v = Vector3D {
            dx: self.req_coord(&mut e, "dx")?,
            dy: self.req_coord(&mut e, "dy")?,
            dz: self.req_coord(&mut e, "dz")?,
        };
        self.r.finish(&e);
        self.no_kids(el)?;
        Ok(v)
    }

    fn bevel(&mut self, el: &Elem) -> Res<Bevel> {
        let mut e = el.clone();
        let b = Bevel {
            width: self.pos_coord(&mut e, "w")?,
            height: self.pos_coord(&mut e, "h")?,
            preset: self.attr(&mut e, "prst")?,
        };
        self.r.finish(&e);
        self.no_kids(el)?;
        Ok(b)
    }

    /// `a:CT_Shape3D`, under any name.
    /// `a:CT_FlatText`, a text body's `flatTx`.
    pub(crate) fn flat_text(&mut self, el: &Elem) -> Res<FlatText> {
        let mut e = el.clone();
        let z = self.coord(&mut e, "z")?;
        self.r.finish(&e);
        self.no_kids(el)?;
        Ok(FlatText { z })
    }

    pub(crate) fn sp3d(&mut self, el: &Elem) -> Res<Shape3D> {
        let mut e = el.clone();
        let mut s = Shape3D {
            z: self.coord(&mut e, "z")?,
            extrusion_height: self.pos_coord(&mut e, "extrusionH")?,
            contour_width: self.pos_coord(&mut e, "contourW")?,
            material: self.attr(&mut e, "prstMaterial")?,
            ..Shape3D::default()
        };
        self.r.finish(&e);
        self.sequence(el, |c, kid| {
            if !kid.in_namespace(ns::DML) {
                return Ok(false);
            }
            match kid.local.as_str() {
                "bevelT" => s.bevel_top = Some(c.bevel(kid)?),
                "bevelB" => s.bevel_bottom = Some(c.bevel(kid)?),
                "extrusionClr" => {
                    c.r.finish(kid);
                    s.extrusion_color = c.holder_color(kid, true)?;
                }
                "contourClr" => {
                    c.r.finish(kid);
                    s.contour_color = c.holder_color(kid, true)?;
                }
                // `extLst` stays in the tree.
                _ => return Ok(false),
            }
            Ok(true)
        })?;
        Ok(s)
    }
}
