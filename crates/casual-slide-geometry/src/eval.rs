//! The evaluator: a single forward pass over the guides, then paths and the rest.
//! No recursion and no reference cycles; time and memory are linear in the size of
//! the definition, which [`crate::compile`] caps for custom geometry.

use std::collections::BTreeMap;

use crate::def::{
    Arg, AxisDef, Cmd, FillMode, GeometryDef, GuideDef, HandleDef, Op, PathDef, PtDef,
};
use crate::diagnostic::{Diagnostic, Diagnostics, clip};
use crate::generated::BUILTIN_ORDER;
use crate::math::{COORD_LIMIT, TURN, atan2, coordinate, saturate, sin_cos, unit_vector};
use crate::{compile, limits};

/// Largest legal shape extent, from `ST_PositiveCoordinate`.
const MAX_EXTENT: i64 = 27_273_042_316_900;

/// Shape extents in EMU. Out-of-range values are clamped to `ST_PositiveCoordinate`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Extents {
    /// Width.
    pub w: i64,
    /// Height.
    pub h: i64,
}

impl Extents {
    /// Clamps both values to the legal range.
    pub fn new(w: i64, h: i64) -> Extents {
        Extents {
            w: w.clamp(0, MAX_EXTENT),
            h: h.clamp(0, MAX_EXTENT),
        }
    }
}

/// Computes the built-in guides for a shape, in slot order.
fn builtins(extents: Extents) -> [f64; BUILTIN_ORDER.len()] {
    let (w, h) = (extents.w as f64, extents.h as f64);
    let (ss, ls) = (w.min(h), w.max(h));
    // Order is `BUILTIN_ORDER`; a test checks each by name.
    [
        16_200_000.0, // 3cd4
        8_100_000.0,  // 3cd8
        13_500_000.0, // 5cd8
        18_900_000.0, // 7cd8
        h,            // b
        10_800_000.0, // cd2
        5_400_000.0,  // cd4
        2_700_000.0,  // cd8
        h,            // h
        w / 2.0,      // hc
        h / 10.0,     // hd10
        h / 2.0,      // hd2
        h / 3.0,      // hd3
        h / 4.0,      // hd4
        h / 5.0,      // hd5
        h / 6.0,      // hd6
        h / 8.0,      // hd8
        0.0,          // l
        ls,           // ls
        w,            // r
        ss,           // ss
        ss / 16.0,    // ssd16
        ss / 2.0,     // ssd2
        ss / 32.0,    // ssd32
        ss / 4.0,     // ssd4
        ss / 6.0,     // ssd6
        ss / 8.0,     // ssd8
        0.0,          // t
        h / 2.0,      // vc
        w,            // w
        w / 10.0,     // wd10
        w / 12.0,     // wd12
        w / 16.0,     // wd16
        w / 2.0,      // wd2
        w / 3.0,      // wd3
        w / 32.0,     // wd32
        w / 4.0,      // wd4
        w / 5.0,      // wd5
        w / 6.0,      // wd6
        w / 8.0,      // wd8
    ]
}

/// Applies one operator. Total: every input gives a finite, in-range value.
pub fn apply(op: Op, x: f64, y: f64, z: f64) -> f64 {
    let divide = |numerator: f64, denominator: f64| {
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    };
    saturate(match op {
        Op::Val => x,
        Op::Abs => x.abs(),
        Op::Sqrt => {
            if x > 0.0 {
                x.sqrt()
            } else {
                0.0
            }
        }
        // ECMA writes `at2 x y` as arctan(y / x).
        Op::At2 => atan2(y, x),
        Op::Cos => x * sin_cos(y).1,
        Op::Sin => x * sin_cos(y).0,
        Op::Tan => {
            let (s, c) = sin_cos(y);
            x * divide(s, c)
        }
        Op::Max => x.max(y),
        Op::Min => x.min(y),
        Op::MulDiv => divide(x * y, z),
        Op::AddSub => x + y - z,
        Op::AddDiv => divide(x + y, z),
        Op::If => {
            if x > 0.0 {
                y
            } else {
                z
            }
        }
        Op::Cat2 => x * unit_vector(y, z).0,
        Op::Sat2 => x * unit_vector(y, z).1,
        Op::Mod => (x * x + y * y + z * z).sqrt(),
        Op::Pin => {
            if y < x {
                x
            } else if y > z {
                z
            } else {
                y
            }
        }
    })
}

/// A point, in EMU or path units.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Point {
    /// Horizontal.
    pub x: f64,
    /// Vertical.
    pub y: f64,
}

/// A resolved `a:arcTo`: the authored values and the ellipse they describe.
///
/// `st_ang` and `sw_ang` are as authored, and are what DrawingML calls angles: the
/// direction of a ray from the centre, so on a non-circular ellipse they are not
/// the parameter of the ellipse equation. `start_param` and `sweep_param` are that
/// parameter, which is what drawing needs; the presets' own `cat2`/`sat2` guides
/// make the same conversion, and arcs meet those guides' endpoints because of it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arc {
    /// Horizontal radius.
    pub wr: f64,
    /// Vertical radius.
    pub hr: f64,
    /// Authored start angle.
    pub st_ang: f64,
    /// Authored sweep, limited to one turn either way.
    pub sw_ang: f64,
    /// Ellipse centre.
    pub center: Point,
    /// Ellipse parameter at the start, 60000ths of a degree.
    pub start_param: f64,
    /// Ellipse parameter sweep, 60000ths of a degree.
    pub sweep_param: f64,
    /// Where the arc starts: the current point when it was issued.
    pub start: Point,
    /// Where the arc ends.
    pub end: Point,
}

/// A resolved path command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    /// Start a subpath.
    MoveTo(Point),
    /// Straight line.
    LineTo(Point),
    /// Elliptical arc.
    ArcTo(Arc),
    /// Quadratic Bezier.
    QuadTo {
        /// Control point.
        ctrl: Point,
        /// End point.
        end: Point,
    },
    /// Cubic Bezier.
    CubicTo {
        /// First control point.
        c1: Point,
        /// Second control point.
        c2: Point,
        /// End point.
        end: Point,
    },
    /// Close the subpath. The current point returns to the subpath's start, and a
    /// following line draws from there: the accent callouts rely on this.
    Close,
}

/// A resolved path, in its own coordinate space (see [`ResolvedPath::scaled_to`]).
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedPath {
    /// Coordinate-space width; 0 means the shape's own.
    pub w: i64,
    /// Coordinate-space height; 0 means the shape's own.
    pub h: i64,
    /// Fill mode.
    pub fill: FillMode,
    /// Whether the path is stroked.
    pub stroke: bool,
    /// Whether the path may be extruded.
    pub extrusion_ok: bool,
    /// Commands. A non-empty list always starts with `MoveTo`.
    pub segments: Vec<Segment>,
}

impl ResolvedPath {
    /// Horizontal and vertical scale from path units to shape EMU.
    pub fn scale(&self, extents: Extents) -> (f64, f64) {
        let axis = |size: i64, extent: i64| {
            if size > 0 {
                extent as f64 / size as f64
            } else {
                1.0
            }
        };
        let e = Extents::new(extents.w, extents.h);
        (axis(self.w, e.w), axis(self.h, e.h))
    }

    /// The path in shape EMU. Radii and points scale per axis; the ellipse
    /// parameter is unchanged by that, so arcs stay exact. `st_ang`/`sw_ang` stay as
    /// authored, in the path's space. `w` and `h` become 0 to say so.
    pub fn scaled_to(&self, extents: Extents) -> ResolvedPath {
        let (sx, sy) = self.scale(extents);
        let pt = |p: Point| Point {
            x: coordinate(p.x * sx),
            y: coordinate(p.y * sy),
        };
        let segments = self
            .segments
            .iter()
            .map(|segment| match *segment {
                Segment::MoveTo(p) => Segment::MoveTo(pt(p)),
                Segment::LineTo(p) => Segment::LineTo(pt(p)),
                Segment::QuadTo { ctrl, end } => Segment::QuadTo {
                    ctrl: pt(ctrl),
                    end: pt(end),
                },
                Segment::CubicTo { c1, c2, end } => Segment::CubicTo {
                    c1: pt(c1),
                    c2: pt(c2),
                    end: pt(end),
                },
                Segment::ArcTo(arc) => Segment::ArcTo(Arc {
                    wr: coordinate(arc.wr * sx),
                    hr: coordinate(arc.hr * sy),
                    center: pt(arc.center),
                    start: pt(arc.start),
                    end: pt(arc.end),
                    ..arc
                }),
                Segment::Close => Segment::Close,
            })
            .collect();
        ResolvedPath {
            w: 0,
            h: 0,
            segments,
            ..*self
        }
    }

    /// The same outline with every arc as cubic Beziers, for renderers with no
    /// elliptical arc primitive. At most four curves per arc.
    pub fn with_arcs_as_cubics(&self) -> ResolvedPath {
        let mut segments = Vec::with_capacity(self.segments.len());
        for segment in &self.segments {
            match segment {
                Segment::ArcTo(arc) => arc.push_cubics(&mut segments),
                other => segments.push(*other),
            }
        }
        ResolvedPath {
            w: self.w,
            h: self.h,
            segments,
            ..*self
        }
    }
}

impl Arc {
    fn push_cubics(&self, out: &mut Vec<Segment>) {
        // Never over a quarter turn per curve; the sweep is capped at one turn.
        let count = (self.sweep_param.abs() / (TURN / 4.0))
            .ceil()
            .clamp(1.0, 4.0);
        let step = self.sweep_param / count;
        let (quarter_sin, quarter_cos) = sin_cos(step / 4.0);
        // 4/3 tan(step / 4), the usual control-point distance.
        let k = if quarter_cos == 0.0 {
            0.0
        } else {
            4.0 / 3.0 * (quarter_sin / quarter_cos)
        };
        let at = |param: f64| {
            let (s, c) = sin_cos(param);
            (
                Point {
                    x: coordinate(self.center.x + self.wr * c),
                    y: coordinate(self.center.y + self.hr * s),
                },
                s,
                c,
            )
        };
        let mut param = self.start_param;
        let (first_sin, first_cos) = sin_cos(param);
        let (mut from, mut s0, mut c0) = (self.start, first_sin, first_cos);
        for index in 0..count as usize {
            param += step;
            let last = index + 1 == count as usize;
            let (mut to, s1, c1) = at(param);
            if last {
                to = self.end;
            }
            out.push(Segment::CubicTo {
                c1: Point {
                    x: coordinate(from.x - k * self.wr * s0),
                    y: coordinate(from.y + k * self.hr * c0),
                },
                c2: Point {
                    x: coordinate(to.x + k * self.wr * s1),
                    y: coordinate(to.y - k * self.hr * c1),
                },
                end: to,
            });
            (from, s0, c0) = (to, s1, c1);
        }
    }
}

/// The text rectangle, in shape EMU.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TextRect {
    /// Left.
    pub l: f64,
    /// Top.
    pub t: f64,
    /// Right.
    pub r: f64,
    /// Bottom.
    pub b: f64,
}

/// A resolved connection site, in shape EMU.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConnectionSite {
    /// Position.
    pub pos: Point,
    /// Incoming connector angle, 60000ths of a degree.
    pub ang: f64,
}

/// One resolved handle axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HandleAxis<'d> {
    /// Guide the handle edits, if this axis moves.
    pub guide: Option<&'d str>,
    /// Lower limit.
    pub min: Option<f64>,
    /// Upper limit.
    pub max: Option<f64>,
}

/// A resolved adjust handle, in shape EMU.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AdjustHandle<'d> {
    /// Moves along x and y.
    Xy {
        /// Horizontal axis.
        x: HandleAxis<'d>,
        /// Vertical axis.
        y: HandleAxis<'d>,
        /// Position.
        pos: Point,
    },
    /// Moves in radius and angle.
    Polar {
        /// Radius axis.
        r: HandleAxis<'d>,
        /// Angle axis.
        ang: HandleAxis<'d>,
        /// Position.
        pos: Point,
    },
}

/// An adjust value supplied by the file for this shape (`a:avLst` on the instance).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adjust<'a> {
    /// Guide name.
    pub name: &'a str,
    /// Value.
    pub value: AdjustValue<'a>,
}

/// The value part of an [`Adjust`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdjustValue<'a> {
    /// A number: `val n`.
    Number(i64),
    /// A formula, as written in the file.
    Formula(&'a str),
}

impl<'a> Adjust<'a> {
    /// `name = val n`.
    pub fn val(name: &'a str, value: i64) -> Adjust<'a> {
        Adjust {
            name,
            value: AdjustValue::Number(value),
        }
    }

    /// `name = fmla`.
    pub fn formula(name: &'a str, formula: &'a str) -> Adjust<'a> {
        Adjust {
            name,
            value: AdjustValue::Formula(formula),
        }
    }
}

/// The result of evaluating a geometry at one size.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedGeometry<'d> {
    /// The shape extents used, after clamping.
    pub extents: Extents,
    /// Paths, in their own coordinate space.
    pub paths: Vec<ResolvedPath>,
    /// Text rectangle.
    pub text_rect: TextRect,
    /// Connection sites, in index order.
    pub connection_sites: Vec<ConnectionSite>,
    /// Adjust handles.
    pub adjust_handles: Vec<AdjustHandle<'d>>,
    /// Anything ignored or repaired while evaluating. Nothing is dropped silently.
    pub diagnostics: Diagnostics,
    def: &'d GeometryDef,
    slots: Vec<f64>,
}

impl ResolvedGeometry<'_> {
    /// FNV-1a over the exact bits of the result. Two targets agree on this only if
    /// they agree on every coordinate to the last bit.
    pub fn fingerprint(&self) -> u64 {
        let mut hash = Fingerprint::default();
        for path in &self.paths {
            hash.mix(path.w as u64);
            hash.mix(path.h as u64);
            hash.mix(path.fill as u64);
            hash.mix(u64::from(path.stroke));
            hash.mix(u64::from(path.extrusion_ok));
            for segment in &path.segments {
                match segment {
                    Segment::MoveTo(p) => hash.points(0, &[*p]),
                    Segment::LineTo(p) => hash.points(1, &[*p]),
                    Segment::QuadTo { ctrl, end } => hash.points(2, &[*ctrl, *end]),
                    Segment::CubicTo { c1, c2, end } => hash.points(3, &[*c1, *c2, *end]),
                    Segment::Close => hash.mix(4),
                    Segment::ArcTo(arc) => {
                        hash.points(5, &[arc.center, arc.start, arc.end]);
                        hash.floats(&[
                            arc.wr,
                            arc.hr,
                            arc.st_ang,
                            arc.sw_ang,
                            arc.start_param,
                            arc.sweep_param,
                        ]);
                    }
                }
            }
        }
        let t = self.text_rect;
        hash.floats(&[t.l, t.t, t.r, t.b]);
        for site in &self.connection_sites {
            hash.points(6, &[site.pos]);
            hash.floats(&[site.ang]);
        }
        for handle in &self.adjust_handles {
            match handle {
                AdjustHandle::Xy { pos, .. } | AdjustHandle::Polar { pos, .. } => {
                    hash.points(7, &[*pos]);
                }
            }
        }
        hash.0
    }

    /// The value of a guide or built-in, after evaluation. Where a name is defined
    /// more than once, the last definition.
    pub fn guide(&self, name: &str) -> Option<f64> {
        let base = BUILTIN_ORDER.len();
        let (av, gd) = (&self.def.av, &self.def.gd);
        if let Some(i) = gd.iter().rposition(|g| g.name == name) {
            return self.slots.get(base + av.len() + i).copied();
        }
        if let Some(i) = av.iter().rposition(|g| g.name == name) {
            return self.slots.get(base + i).copied();
        }
        let i = BUILTIN_ORDER.binary_search(&name).ok()?;
        self.slots.get(i).copied()
    }
}

struct Fingerprint(u64);

impl Default for Fingerprint {
    fn default() -> Fingerprint {
        Fingerprint(0xcbf2_9ce4_8422_2325)
    }
}

impl Fingerprint {
    fn mix(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.0 = (self.0 ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }

    fn floats(&mut self, values: &[f64]) {
        for value in values {
            self.mix(value.to_bits());
        }
    }

    fn points(&mut self, tag: u64, points: &[Point]) {
        self.mix(tag);
        for point in points {
            self.floats(&[point.x, point.y]);
        }
    }
}

/// Ellipse parameter for a DrawingML angle: monotonic in the angle, with
/// `f(a + turn) = f(a) + turn`, so a sweep keeps its direction and whole turns.
fn ellipse_param(angle: f64, wr: f64, hr: f64) -> f64 {
    if wr == hr {
        return angle;
    }
    let turns = (angle / TURN).floor();
    let within = angle - turns * TURN;
    let (s, c) = sin_cos(within);
    let mut param = atan2(wr * s, hr * c);
    if param < 0.0 {
        param += TURN;
    }
    param + turns * TURN
}

struct Machine {
    slots: Vec<f64>,
}

impl Machine {
    fn get(&self, arg: Arg) -> f64 {
        match arg {
            Arg::Lit(value) => saturate(value as f64),
            // A compiled operand always names an earlier slot; 0 keeps a bad table
            // from panicking all the same.
            Arg::Slot(slot) => self.slots.get(usize::from(slot)).copied().unwrap_or(0.0),
        }
    }

    fn guide(&self, guide: &GuideDef) -> f64 {
        apply(
            guide.op,
            self.get(guide.a),
            self.get(guide.b),
            self.get(guide.c),
        )
    }

    fn point(&self, p: PtDef) -> Point {
        Point {
            x: coordinate(self.get(p.x)),
            y: coordinate(self.get(p.y)),
        }
    }

    fn axis<'d>(&self, axis: &'d AxisDef) -> HandleAxis<'d> {
        HandleAxis {
            guide: axis.guide.as_deref(),
            min: axis.min.map(|a| coordinate(self.get(a))),
            max: axis.max.map(|a| coordinate(self.get(a))),
        }
    }

    fn path(&self, path: &PathDef, diagnostics: &mut Diagnostics) -> ResolvedPath {
        let mut segments = Vec::with_capacity(path.cmds.len());
        let mut current = Point::default();
        let mut subpath_start = Point::default();
        for (index, cmd) in path.cmds.iter().enumerate() {
            if index == 0 && !matches!(cmd, Cmd::MoveTo(_)) {
                diagnostics.push(Diagnostic::PathWithoutMoveTo);
                segments.push(Segment::MoveTo(current));
            }
            match *cmd {
                Cmd::MoveTo(p) => {
                    current = self.point(p);
                    subpath_start = current;
                    segments.push(Segment::MoveTo(current));
                }
                Cmd::LineTo(p) => {
                    current = self.point(p);
                    segments.push(Segment::LineTo(current));
                }
                Cmd::QuadTo(ctrl, end) => {
                    current = self.point(end);
                    segments.push(Segment::QuadTo {
                        ctrl: self.point(ctrl),
                        end: current,
                    });
                }
                Cmd::CubicTo(c1, c2, end) => {
                    current = self.point(end);
                    segments.push(Segment::CubicTo {
                        c1: self.point(c1),
                        c2: self.point(c2),
                        end: current,
                    });
                }
                Cmd::ArcTo { wr, hr, st, sw } => {
                    let arc = resolve_arc(
                        current,
                        coordinate(self.get(wr)),
                        coordinate(self.get(hr)),
                        coordinate(self.get(st)),
                        coordinate(self.get(sw)),
                    );
                    current = arc.end;
                    segments.push(Segment::ArcTo(arc));
                }
                Cmd::Close => {
                    current = subpath_start;
                    segments.push(Segment::Close);
                }
            }
        }
        ResolvedPath {
            w: path.w,
            h: path.h,
            fill: path.fill,
            stroke: path.stroke,
            extrusion_ok: path.extrusion_ok,
            segments,
        }
    }
}

fn resolve_arc(start: Point, wr: f64, hr: f64, st_ang: f64, sw_ang: f64) -> Arc {
    // A negative radius has no meaning; a sweep past one turn only retraces.
    let (wr, hr) = (wr.abs(), hr.abs());
    let sw_ang = sw_ang.clamp(-TURN, TURN);
    let start_param = ellipse_param(st_ang, wr, hr);
    let sweep_param = saturate(ellipse_param(st_ang + sw_ang, wr, hr) - start_param);
    let (s0, c0) = sin_cos(start_param);
    let (s1, c1) = sin_cos(start_param + sweep_param);
    let center = Point {
        x: coordinate(start.x - wr * c0),
        y: coordinate(start.y - hr * s0),
    };
    let end = if sweep_param == 0.0 {
        start
    } else {
        Point {
            x: coordinate(center.x + wr * c1),
            y: coordinate(center.y + hr * s1),
        }
    };
    Arc {
        wr,
        hr,
        st_ang,
        sw_ang,
        center,
        start_param,
        sweep_param,
        start,
        end,
    }
}

impl GeometryDef {
    /// Evaluates at one size with the file's adjust values, which replace the
    /// defaults of the same name. Never panics, on any input.
    pub fn evaluate(&self, extents: Extents, adjusts: &[Adjust<'_>]) -> ResolvedGeometry<'_> {
        let extents = Extents::new(extents.w, extents.h);
        let mut diagnostics = Diagnostics::default();
        let mut machine = Machine {
            slots: Vec::with_capacity(BUILTIN_ORDER.len() + self.av.len() + self.gd.len()),
        };
        machine.slots.extend_from_slice(&builtins(extents));

        let overrides = self.collect_overrides(adjusts, &mut diagnostics);
        // What a file's adjust formula can name: built-ins and earlier adjusts.
        let mut scope: BTreeMap<&str, u16> = BTreeMap::new();
        for guide in self.av.iter() {
            let value = match overrides.get(guide.name.as_ref()) {
                None => machine.guide(guide),
                Some(AdjustValue::Number(number)) => saturate(*number as f64),
                Some(AdjustValue::Formula(formula)) => {
                    match compile::parse_formula(formula, |name| scope.get(name).copied()) {
                        Ok(parsed) => {
                            for problem in parsed.problems {
                                diagnostics.push(problem);
                            }
                            apply(
                                parsed.op,
                                machine.get(parsed.a),
                                machine.get(parsed.b),
                                machine.get(parsed.c),
                            )
                        }
                        Err(problem) => {
                            diagnostics.push(problem);
                            machine.guide(guide)
                        }
                    }
                }
            };
            if !overrides.is_empty() {
                scope.insert(guide.name.as_ref(), machine.slots.len() as u16);
            }
            machine.slots.push(value);
        }
        for guide in self.gd.iter() {
            let value = machine.guide(guide);
            machine.slots.push(value);
        }

        let paths = self
            .paths
            .iter()
            .map(|path| machine.path(path, &mut diagnostics))
            .collect();
        let text_rect = match self.rect {
            Some(rect) => TextRect {
                l: coordinate(machine.get(rect.l)),
                t: coordinate(machine.get(rect.t)),
                r: coordinate(machine.get(rect.r)),
                b: coordinate(machine.get(rect.b)),
            },
            None => TextRect {
                l: 0.0,
                t: 0.0,
                r: extents.w as f64,
                b: extents.h as f64,
            },
        };
        let connection_sites = self
            .sites
            .iter()
            .map(|site| ConnectionSite {
                pos: machine.point(site.pos),
                ang: coordinate(machine.get(site.ang)),
            })
            .collect();
        let adjust_handles = self
            .handles
            .iter()
            .map(|handle| match handle {
                HandleDef::Xy { x, y, pos } => AdjustHandle::Xy {
                    x: machine.axis(x),
                    y: machine.axis(y),
                    pos: machine.point(*pos),
                },
                HandleDef::Polar { r, ang, pos } => AdjustHandle::Polar {
                    r: machine.axis(r),
                    ang: machine.axis(ang),
                    pos: machine.point(*pos),
                },
            })
            .collect();
        ResolvedGeometry {
            extents,
            paths,
            text_rect,
            connection_sites,
            adjust_handles,
            diagnostics,
            def: self,
            slots: machine.slots,
        }
    }

    /// Last value wins per name. A name this geometry has no adjust for is reported.
    fn collect_overrides<'a>(
        &self,
        adjusts: &[Adjust<'a>],
        diagnostics: &mut Diagnostics,
    ) -> BTreeMap<&'a str, AdjustValue<'a>> {
        let mut overrides = BTreeMap::new();
        if adjusts.len() > limits::MAX_GUIDES {
            diagnostics.push(Diagnostic::TooManyAdjusts);
        }
        for adjust in adjusts.iter().take(limits::MAX_GUIDES) {
            if self.av.iter().any(|g| g.name == adjust.name) {
                overrides.insert(adjust.name, adjust.value);
            } else {
                diagnostics.push(Diagnostic::UnknownAdjust(clip(adjust.name)));
            }
        }
        overrides
    }
}

const _: () = assert!(COORD_LIMIT > MAX_EXTENT as f64);
