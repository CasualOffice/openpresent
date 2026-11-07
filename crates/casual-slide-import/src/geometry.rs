//! Read `prstGeom`, `custGeom` and `prstTxWarp` (`docs/spec/GEOM.md`) into the model,
//! and hand them to `casual-slide-geometry` (`GEO-04`).
//!
//! A preset's token and adjust names are checked against the engine's tables, and a
//! custom shape's guides, handles and paths compile or are reported. Nothing is
//! evaluated at a size here: no paths are produced, and how a shape looks is not this
//! tier's (owner: no painting or rendering until Phase 1 closes). The model keeps the
//! file's text.
//! `MS-OI29500` 2.1.1309 to 2.1.1314 cover what the engine does with these.

use casual_slide_geometry::{
    CompiledGeometry, FillMode, GeometryError, RawAxis, RawCommand, RawGeometry, RawGuide,
    RawHandle, RawPath, RawPoint, RawRect, RawSite, preset_shape, text_warp,
};
use casual_slide_model::geometry::{
    AdjPoint, AdjustHandle, ConnectionSite, CustomGeometry, GeomGuide, GeomRect, Geometry, Path2D,
    PathCommand, PathFillMode, PolarHandle, PresetGeometry, PresetTextShape, XyHandle,
};
use casual_slide_model::properties::Lexical;

use crate::drawing::{Ctx, DrawingFinding, DrawingLimits, Res, Stop};
use crate::properties::{Elem, ImportError, ImportFindingKind as K, Reader, ns};

impl Ctx<'_, '_> {
    /// `custGeom` or `prstGeom`, which the caller has checked it is.
    pub(crate) fn geometry(&mut self, el: &Elem) -> Res<Geometry> {
        if el.local == "prstGeom" {
            let g = self.preset_geometry(el)?;
            return Ok(Geometry::Preset(g));
        }
        let g = self.custom_geometry(el)?;
        match compile_custom(&g) {
            Ok(compiled) if !compiled.diagnostics.is_empty() => {
                self.r
                    .note(K::Drawing(DrawingFinding::GeometryRepaired), el, None);
            }
            Ok(_) => {}
            Err(_) => return Err(Stop::Over),
        }
        Ok(Geometry::Custom(Box::new(g)))
    }

    fn preset_geometry(&mut self, el: &Elem) -> Res<PresetGeometry> {
        let mut e = el.clone();
        let preset: Option<casual_slide_model::geometry::ShapeType> = self.req(&mut e, "prst")?;
        self.r.finish(&e);
        let adjust = self.adjust_child(el)?;
        let g = PresetGeometry { preset, adjust };
        if let Some(token) = g.preset.map(|p| p.to_lexical()) {
            match preset_shape(&token) {
                None => self.r.note(K::UnresolvedReference, el, Some("prst")),
                Some(def) => {
                    // By name against the preset's own adjusts. Nothing is evaluated:
                    // no size, no path, nothing about how the shape looks.
                    let unknown = g.adjust.iter().flatten().any(|gd| {
                        gd.name
                            .as_deref()
                            .is_some_and(|name| !def.av.iter().any(|a| a.name == name))
                    });
                    if unknown {
                        self.r
                            .note(K::Drawing(DrawingFinding::GeometryRepaired), el, None);
                    }
                }
            }
        }
        Ok(g)
    }

    fn adjust_child(&mut self, el: &Elem) -> Res<Option<Vec<GeomGuide>>> {
        let mut adjust = None;
        self.sequence(el, |c, kid| {
            if kid.is(ns::DML, "avLst") {
                adjust = Some(c.guides(kid)?);
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        Ok(adjust)
    }

    /// `a:CT_PresetTextShape`, for a text body's reader.
    pub(crate) fn preset_text_shape(&mut self, el: &Elem) -> Res<PresetTextShape> {
        let mut e = el.clone();
        let preset: Option<casual_slide_model::geometry::TextShapeType> =
            self.req(&mut e, "prst")?;
        self.r.finish(&e);
        if let Some(token) = preset.map(|p| p.to_lexical())
            && text_warp(&token).is_none()
        {
            self.r.note(K::UnresolvedReference, el, Some("prst"));
        }
        Ok(PresetTextShape {
            preset,
            adjust: self.adjust_child(el)?,
        })
    }

    fn guides(&mut self, el: &Elem) -> Res<Vec<GeomGuide>> {
        self.r.finish(el);
        let mut out = Vec::new();
        for kid in self.kids(el, self.limits.max_guides)? {
            if !kid.is(ns::DML, "gd") {
                self.r.retained(&kid);
                continue;
            }
            let mut k = kid.clone();
            out.push(GeomGuide {
                name: self.req(&mut k, "name")?,
                formula: self.req(&mut k, "fmla")?,
            });
            self.r.finish(&k);
            self.no_kids(&kid)?;
        }
        Ok(out)
    }

    fn adj_point(&mut self, el: &Elem) -> Res<AdjPoint> {
        let mut e = el.clone();
        let p = AdjPoint {
            x: self.req(&mut e, "x")?,
            y: self.req(&mut e, "y")?,
        };
        self.r.finish(&e);
        self.no_kids(el)?;
        Ok(p)
    }

    fn pos_child(&mut self, el: &Elem) -> Res<Option<AdjPoint>> {
        let mut pos = None;
        self.sequence(el, |c, kid| {
            if kid.is(ns::DML, "pos") {
                pos = Some(c.adj_point(kid)?);
                Ok(true)
            } else {
                Ok(false)
            }
        })?;
        if pos.is_none() {
            self.r.note(K::MissingRequired, el, Some("pos"));
        }
        Ok(pos)
    }

    fn custom_geometry(&mut self, el: &Elem) -> Res<CustomGeometry> {
        self.r.finish(el);
        let mut g = CustomGeometry::default();
        self.sequence(el, |c, kid| {
            if !kid.in_namespace(ns::DML) {
                return Ok(false);
            }
            match kid.local.as_str() {
                "avLst" => g.adjust = Some(c.guides(kid)?),
                "gdLst" => g.guides = Some(c.guides(kid)?),
                "ahLst" => g.handles = Some(c.handles(kid)?),
                "cxnLst" => g.sites = Some(c.sites(kid)?),
                "rect" => {
                    let mut k = kid.clone();
                    g.rect = Some(GeomRect {
                        l: c.req(&mut k, "l")?,
                        t: c.req(&mut k, "t")?,
                        r: c.req(&mut k, "r")?,
                        b: c.req(&mut k, "b")?,
                    });
                    c.r.finish(&k);
                    c.no_kids(kid)?;
                }
                "pathLst" => g.paths = Some(c.paths(kid)?),
                _ => return Ok(false),
            }
            Ok(true)
        })?;
        if g.paths.is_none() {
            self.r.note(K::MissingRequired, el, Some("pathLst"));
        }
        Ok(g)
    }

    fn handles(&mut self, el: &Elem) -> Res<Vec<AdjustHandle>> {
        self.r.finish(el);
        let mut out = Vec::new();
        for kid in self.kids(el, self.limits.max_handles)? {
            let mut k = kid.clone();
            if kid.is(ns::DML, "ahXY") {
                let mut h = XyHandle {
                    gd_ref_x: self.attr(&mut k, "gdRefX")?,
                    min_x: self.attr(&mut k, "minX")?,
                    max_x: self.attr(&mut k, "maxX")?,
                    gd_ref_y: self.attr(&mut k, "gdRefY")?,
                    min_y: self.attr(&mut k, "minY")?,
                    max_y: self.attr(&mut k, "maxY")?,
                    pos: None,
                };
                self.r.finish(&k);
                h.pos = self.pos_child(&kid)?;
                out.push(AdjustHandle::Xy(h));
            } else if kid.is(ns::DML, "ahPolar") {
                let mut h = PolarHandle {
                    gd_ref_r: self.attr(&mut k, "gdRefR")?,
                    min_r: self.attr(&mut k, "minR")?,
                    max_r: self.attr(&mut k, "maxR")?,
                    gd_ref_ang: self.attr(&mut k, "gdRefAng")?,
                    min_ang: self.attr(&mut k, "minAng")?,
                    max_ang: self.attr(&mut k, "maxAng")?,
                    pos: None,
                };
                self.r.finish(&k);
                h.pos = self.pos_child(&kid)?;
                out.push(AdjustHandle::Polar(h));
            } else {
                self.r.retained(&kid);
            }
        }
        Ok(out)
    }

    fn sites(&mut self, el: &Elem) -> Res<Vec<ConnectionSite>> {
        self.r.finish(el);
        let mut out = Vec::new();
        for kid in self.kids(el, self.limits.max_sites)? {
            if !kid.is(ns::DML, "cxn") {
                self.r.retained(&kid);
                continue;
            }
            let mut k = kid.clone();
            let angle = self.req(&mut k, "ang")?;
            self.r.finish(&k);
            out.push(ConnectionSite {
                angle,
                pos: self.pos_child(&kid)?,
            });
        }
        Ok(out)
    }

    fn paths(&mut self, el: &Elem) -> Res<Vec<Path2D>> {
        self.r.finish(el);
        let mut out = Vec::new();
        for kid in self.kids(el, self.limits.max_paths)? {
            if !kid.is(ns::DML, "path") {
                self.r.retained(&kid);
                continue;
            }
            let mut k = kid.clone();
            let mut p = Path2D {
                w: self.pos_coord(&mut k, "w")?,
                h: self.pos_coord(&mut k, "h")?,
                fill: self.attr(&mut k, "fill")?,
                stroke: self.attr(&mut k, "stroke")?,
                extrusion_ok: self.attr(&mut k, "extrusionOk")?,
                commands: Vec::new(),
            };
            self.r.finish(&k);
            for cmd in self.kids(&kid, self.limits.max_path_commands)? {
                match self.command(&cmd)? {
                    Some(command) => p.commands.push(command),
                    None => self.r.retained(&cmd),
                }
            }
            out.push(p);
        }
        Ok(out)
    }

    fn points(&mut self, el: &Elem, want: usize) -> Res<Vec<AdjPoint>> {
        self.r.finish(el);
        let mut pts = Vec::new();
        for kid in self.all_kids(el)? {
            if kid.is(ns::DML, "pt") {
                pts.push(self.adj_point(&kid)?);
            } else {
                self.r.retained(&kid);
            }
        }
        if pts.len() != want {
            self.r.note(K::CountMismatch, el, Some("pt"));
        }
        Ok(pts)
    }

    fn command(&mut self, el: &Elem) -> Res<Option<PathCommand>> {
        if !el.in_namespace(ns::DML) {
            return Ok(None);
        }
        Ok(Some(match el.local.as_str() {
            "moveTo" => PathCommand::MoveTo(self.points(el, 1)?.into_iter().next()),
            "lnTo" => PathCommand::LineTo(self.points(el, 1)?.into_iter().next()),
            "quadBezTo" => PathCommand::QuadBezTo(self.points(el, 2)?),
            "cubicBezTo" => PathCommand::CubicBezTo(self.points(el, 3)?),
            "close" => {
                self.r.finish(el);
                self.no_kids(el)?;
                PathCommand::Close
            }
            "arcTo" => {
                let mut e = el.clone();
                let arc = PathCommand::ArcTo {
                    width_radius: self.req(&mut e, "wR")?,
                    height_radius: self.req(&mut e, "hR")?,
                    start_angle: self.req(&mut e, "stAng")?,
                    swing_angle: self.req(&mut e, "swAng")?,
                };
                self.r.finish(&e);
                self.no_kids(el)?;
                arc
            }
            _ => return Ok(None),
        }))
    }
}

/// Read `prstTxWarp`, for a text body's reader. `None`, reported, over a bound.
pub fn read_preset_text_shape(
    r: &mut Reader<'_>,
    limits: DrawingLimits,
    el: &Elem,
) -> Result<Option<PresetTextShape>, ImportError> {
    Ctx::new(r, &[], limits).guard(el, |c| c.preset_text_shape(el))
}

fn text(s: &Option<String>) -> String {
    s.clone().unwrap_or_default()
}

fn raw_point(p: &AdjPoint) -> RawPoint {
    RawPoint {
        x: text(&p.x),
        y: text(&p.y),
    }
}

fn raw_guides(list: &Option<Vec<GeomGuide>>) -> Vec<RawGuide> {
    list.iter()
        .flatten()
        .map(|g| RawGuide {
            name: text(&g.name),
            fmla: text(&g.formula),
        })
        .collect()
}

fn raw_axis(guide: &Option<String>, min: &Option<String>, max: &Option<String>) -> RawAxis {
    RawAxis {
        guide: guide.clone(),
        min: min.clone(),
        max: max.clone(),
    }
}

/// The geometry engine's input for a custom geometry, with what is absent as the
/// engine's own "absent". The schema's defaults come from [`RawPath::default`].
pub fn custom_to_raw(g: &CustomGeometry) -> RawGeometry {
    let point = |p: &Option<AdjPoint>| p.as_ref().map(raw_point).unwrap_or_default();
    let nth = |pts: &[AdjPoint], i: usize| pts.get(i).map(raw_point).unwrap_or_default();
    RawGeometry {
        av: raw_guides(&g.adjust),
        gd: raw_guides(&g.guides),
        handles: g
            .handles
            .iter()
            .flatten()
            .map(|h| match h {
                AdjustHandle::Xy(h) => RawHandle::Xy {
                    x: raw_axis(&h.gd_ref_x, &h.min_x, &h.max_x),
                    y: raw_axis(&h.gd_ref_y, &h.min_y, &h.max_y),
                    pos: point(&h.pos),
                },
                AdjustHandle::Polar(h) => RawHandle::Polar {
                    r: raw_axis(&h.gd_ref_r, &h.min_r, &h.max_r),
                    ang: raw_axis(&h.gd_ref_ang, &h.min_ang, &h.max_ang),
                    pos: point(&h.pos),
                },
            })
            .collect(),
        sites: g
            .sites
            .iter()
            .flatten()
            .map(|s| RawSite {
                ang: text(&s.angle),
                pos: point(&s.pos),
            })
            .collect(),
        rect: g.rect.as_ref().map(|r| RawRect {
            l: text(&r.l),
            t: text(&r.t),
            r: text(&r.r),
            b: text(&r.b),
        }),
        paths: g
            .paths
            .iter()
            .flatten()
            .map(|p| {
                let default = RawPath::default();
                RawPath {
                    w: p.w.map_or(default.w, |c| c.0),
                    h: p.h.map_or(default.h, |c| c.0),
                    fill: p.fill.map_or(default.fill, path_fill),
                    stroke: p.stroke.unwrap_or(default.stroke),
                    extrusion_ok: p.extrusion_ok.unwrap_or(default.extrusion_ok),
                    commands: p
                        .commands
                        .iter()
                        .map(|c| match c {
                            PathCommand::MoveTo(pt) => RawCommand::MoveTo(point(pt)),
                            PathCommand::LineTo(pt) => RawCommand::LineTo(point(pt)),
                            PathCommand::ArcTo {
                                width_radius,
                                height_radius,
                                start_angle,
                                swing_angle,
                            } => RawCommand::ArcTo {
                                wr: text(width_radius),
                                hr: text(height_radius),
                                st_ang: text(start_angle),
                                sw_ang: text(swing_angle),
                            },
                            PathCommand::QuadBezTo(pts) => {
                                RawCommand::QuadBezTo(nth(pts, 0), nth(pts, 1))
                            }
                            PathCommand::CubicBezTo(pts) => {
                                RawCommand::CubicBezTo(nth(pts, 0), nth(pts, 1), nth(pts, 2))
                            }
                            PathCommand::Close => RawCommand::Close,
                        })
                        .collect(),
                }
            })
            .collect(),
    }
}

fn path_fill(mode: PathFillMode) -> FillMode {
    match mode {
        PathFillMode::None => FillMode::None,
        PathFillMode::Norm => FillMode::Norm,
        PathFillMode::Lighten => FillMode::Lighten,
        PathFillMode::LightenLess => FillMode::LightenLess,
        PathFillMode::Darken => FillMode::Darken,
        PathFillMode::DarkenLess => FillMode::DarkenLess,
    }
}

/// Compile a custom geometry with `casual_slide_geometry`, which fails only on its caps.
pub fn compile_custom(g: &CustomGeometry) -> Result<CompiledGeometry, GeometryError> {
    custom_to_raw(g).compile()
}
