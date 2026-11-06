//! Custom geometry: file text to the same compiled form as the preset table.
//!
//! The caller, with its bounded XML walker, supplies attribute text. Size limits
//! fail the compile; anything else malformed is repaired to a defined value and
//! reported, so that one bad guide does not lose the shape.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::def::{
    Arg, AxisDef, Cmd, FillMode, GeometryDef, GuideDef, HandleDef, Op, PathDef, PtDef, RectDef,
    SiteDef,
};
use crate::diagnostic::{Diagnostic, Diagnostics, clip};
use crate::generated::BUILTIN_ORDER;
use crate::limits;

/// A compile that cannot proceed: the definition is over a size limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryError {
    /// More `av` and `gd` guides than [`limits::MAX_GUIDES`].
    TooManyGuides,
    /// More paths than [`limits::MAX_PATHS`].
    TooManyPaths,
    /// More path points than [`limits::MAX_PATH_POINTS`].
    TooManyPathPoints,
    /// More handles than [`limits::MAX_HANDLES`].
    TooManyHandles,
    /// More connection sites than [`limits::MAX_SITES`].
    TooManySites,
}

impl std::fmt::Display for GeometryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GeometryError::TooManyGuides => "custom geometry has too many guides",
            GeometryError::TooManyPaths => "custom geometry has too many paths",
            GeometryError::TooManyPathPoints => "custom geometry has too many path points",
            GeometryError::TooManyHandles => "custom geometry has too many adjust handles",
            GeometryError::TooManySites => "custom geometry has too many connection sites",
        })
    }
}

impl std::error::Error for GeometryError {}

/// `a:gd`, as written.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RawGuide {
    /// `@name`
    pub name: String,
    /// `@fmla`
    pub fmla: String,
}

/// `a:pt` or `a:pos`, as written.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RawPoint {
    /// `@x`
    pub x: String,
    /// `@y`
    pub y: String,
}

/// A path command, as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RawCommand {
    /// `a:moveTo`
    MoveTo(RawPoint),
    /// `a:lnTo`
    LineTo(RawPoint),
    /// `a:arcTo`
    ArcTo {
        /// `@wR`
        wr: String,
        /// `@hR`
        hr: String,
        /// `@stAng`
        st_ang: String,
        /// `@swAng`
        sw_ang: String,
    },
    /// `a:quadBezTo`
    QuadBezTo(RawPoint, RawPoint),
    /// `a:cubicBezTo`
    CubicBezTo(RawPoint, RawPoint, RawPoint),
    /// `a:close`
    Close,
}

/// `a:path`, with attributes already decoded to the schema's types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawPath {
    /// `@w`. Negative values are treated as absent.
    pub w: i64,
    /// `@h`. Negative values are treated as absent.
    pub h: i64,
    /// `@fill`
    pub fill: FillMode,
    /// `@stroke`
    pub stroke: bool,
    /// `@extrusionOk`
    pub extrusion_ok: bool,
    /// Children.
    pub commands: Vec<RawCommand>,
}

impl Default for RawPath {
    /// The schema's defaults.
    fn default() -> RawPath {
        RawPath {
            w: 0,
            h: 0,
            fill: FillMode::Norm,
            stroke: true,
            extrusion_ok: true,
            commands: Vec::new(),
        }
    }
}

/// One handle axis, as written.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RawAxis {
    /// `@gdRef*`
    pub guide: Option<String>,
    /// `@min*`
    pub min: Option<String>,
    /// `@max*`
    pub max: Option<String>,
}

/// An adjust handle, as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RawHandle {
    /// `a:ahXY`
    Xy {
        /// `@gdRefX`, `@minX`, `@maxX`
        x: RawAxis,
        /// `@gdRefY`, `@minY`, `@maxY`
        y: RawAxis,
        /// `a:pos`
        pos: RawPoint,
    },
    /// `a:ahPolar`
    Polar {
        /// `@gdRefR`, `@minR`, `@maxR`
        r: RawAxis,
        /// `@gdRefAng`, `@minAng`, `@maxAng`
        ang: RawAxis,
        /// `a:pos`
        pos: RawPoint,
    },
}

/// `a:cxn`, as written.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RawSite {
    /// `@ang`
    pub ang: String,
    /// `a:pos`
    pub pos: RawPoint,
}

/// `a:rect`, as written.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RawRect {
    /// `@l`
    pub l: String,
    /// `@t`
    pub t: String,
    /// `@r`
    pub r: String,
    /// `@b`
    pub b: String,
}

/// `a:custGeom`, as written.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RawGeometry {
    /// `a:avLst`
    pub av: Vec<RawGuide>,
    /// `a:gdLst`
    pub gd: Vec<RawGuide>,
    /// `a:ahLst`
    pub handles: Vec<RawHandle>,
    /// `a:cxnLst`
    pub sites: Vec<RawSite>,
    /// `a:rect`
    pub rect: Option<RawRect>,
    /// `a:pathLst`
    pub paths: Vec<RawPath>,
}

/// A compiled custom geometry and what was repaired on the way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledGeometry {
    /// The definition, ready to evaluate.
    pub def: GeometryDef,
    /// Repairs made.
    pub diagnostics: Diagnostics,
}

pub(crate) struct ParsedFormula {
    pub op: Op,
    pub a: Arg,
    pub b: Arg,
    pub c: Arg,
    pub problems: Vec<Diagnostic>,
}

fn operator(token: &str) -> Option<(Op, usize)> {
    Some(match token {
        "val" => (Op::Val, 1),
        "abs" => (Op::Abs, 1),
        "sqrt" => (Op::Sqrt, 1),
        "at2" => (Op::At2, 2),
        "cos" => (Op::Cos, 2),
        "sin" => (Op::Sin, 2),
        "tan" => (Op::Tan, 2),
        "max" => (Op::Max, 2),
        "min" => (Op::Min, 2),
        "*/" => (Op::MulDiv, 3),
        "+-" => (Op::AddSub, 3),
        "+/" => (Op::AddDiv, 3),
        "?:" => (Op::If, 3),
        "cat2" => (Op::Cat2, 3),
        "sat2" => (Op::Sat2, 3),
        "mod" => (Op::Mod, 3),
        "pin" => (Op::Pin, 3),
        _ => return None,
    })
}

/// Parses an `xsd:long` lexically, saturating at the ends of `i64`.
fn integer(token: &str) -> Option<i64> {
    let (negative, digits) = match token.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, token.strip_prefix('+').unwrap_or(token)),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(
        token
            .parse::<i64>()
            .unwrap_or(if negative { i64::MIN } else { i64::MAX }),
    )
}

/// `ST_UniversalMeasure`: `-?[0-9]+(\.[0-9]+)?(mm|cm|in|pt|pc|pi)`, to whole EMU.
fn universal_measure(token: &str) -> Option<i64> {
    let split = token.len().checked_sub(2)?;
    if !token.is_char_boundary(split) {
        return None;
    }
    let (number, unit) = token.split_at(split);
    let emu_per_unit = match unit {
        "mm" => 36_000.0,
        "cm" => 360_000.0,
        "in" => 914_400.0,
        "pt" => 12_700.0,
        "pc" | "pi" => 152_400.0,
        _ => return None,
    };
    let unsigned = number.strip_prefix('-').unwrap_or(number);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, "0"));
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(whole) || !digits(fraction) {
        return None;
    }
    let value: f64 = number.parse().ok()?;
    // `as` saturates, and parsing rounds correctly on every target.
    Some((value * emu_per_unit).round() as i64)
}

fn builtin_slot(name: &str) -> Option<u16> {
    BUILTIN_ORDER.binary_search(&name).ok().map(|i| i as u16)
}

/// Resolves one formula. `lookup` sees the guides defined so far; built-ins are the
/// fallback, so a guide may shadow one, as textRingInside does with `r`.
pub(crate) fn parse_formula(
    formula: &str,
    lookup: impl Fn(&str) -> Option<u16>,
) -> Result<ParsedFormula, Diagnostic> {
    if formula.len() > limits::MAX_FORMULA_LEN {
        return Err(Diagnostic::TooLong);
    }
    let mut tokens = formula.split_ascii_whitespace();
    let first = tokens.next().unwrap_or("");
    let Some((op, arity)) = operator(first) else {
        return Err(Diagnostic::UnknownOperator(clip(formula)));
    };
    let mut args = [Arg::Lit(0); 3];
    let mut problems = Vec::new();
    let mut given = 0;
    for token in tokens {
        if let Some(arg) = args.get_mut(given).filter(|_| given < arity) {
            *arg = if let Some(value) = integer(token) {
                Arg::Lit(value)
            } else if let Some(slot) = lookup(token).or_else(|| builtin_slot(token)) {
                Arg::Slot(slot)
            } else {
                problems.push(Diagnostic::UnknownGuide(clip(token)));
                Arg::Lit(0)
            };
        }
        given += 1;
    }
    if given < arity {
        return Err(Diagnostic::WrongOperandCount(clip(formula)));
    }
    if given > arity {
        problems.push(Diagnostic::WrongOperandCount(clip(formula)));
    }
    let [a, b, c] = args;
    Ok(ParsedFormula {
        op,
        a,
        b,
        c,
        problems,
    })
}

struct Compiler {
    scope: BTreeMap<String, u16>,
    next_slot: usize,
    diagnostics: Diagnostics,
}

impl Compiler {
    fn guides(&mut self, list: &[RawGuide]) -> Vec<GuideDef> {
        let mut out = Vec::with_capacity(list.len());
        for raw in list {
            let name = if raw.name.len() > limits::MAX_NAME_LEN {
                // Unreachable by name, but it still takes its slot and its turn.
                self.diagnostics.push(Diagnostic::TooLong);
                String::new()
            } else {
                raw.name.clone()
            };
            let scope = &self.scope;
            let (op, a, b, c) = match parse_formula(&raw.fmla, |n| scope.get(n).copied()) {
                Ok(parsed) => {
                    for problem in parsed.problems {
                        self.diagnostics.push(problem);
                    }
                    (parsed.op, parsed.a, parsed.b, parsed.c)
                }
                Err(problem) => {
                    self.diagnostics.push(problem);
                    (Op::Val, Arg::Lit(0), Arg::Lit(0), Arg::Lit(0))
                }
            };
            if !name.is_empty() {
                self.scope.insert(name.clone(), self.next_slot as u16);
            }
            out.push(GuideDef {
                name: Cow::Owned(name),
                op,
                a,
                b,
                c,
            });
            self.next_slot += 1;
        }
        out
    }

    /// An operand attribute: a number, a guide name, or a measure.
    fn arg(&mut self, token: &str) -> Arg {
        let token = token.trim_ascii();
        if let Some(value) = integer(token) {
            return Arg::Lit(value);
        }
        let slot = self.scope.get(token).copied();
        if let Some(slot) = slot.or_else(|| builtin_slot(token)) {
            return Arg::Slot(slot);
        }
        if let Some(value) = universal_measure(token) {
            return Arg::Lit(value);
        }
        self.diagnostics.push(Diagnostic::UnknownGuide(clip(token)));
        Arg::Lit(0)
    }

    fn point(&mut self, point: &RawPoint) -> PtDef {
        PtDef {
            x: self.arg(&point.x),
            y: self.arg(&point.y),
        }
    }

    fn axis(&mut self, axis: &RawAxis) -> AxisDef {
        AxisDef {
            guide: axis.guide.as_ref().map(|name| Cow::Owned(clip(name))),
            min: axis.min.as_deref().map(|token| self.arg(token)),
            max: axis.max.as_deref().map(|token| self.arg(token)),
        }
    }

    fn command(&mut self, command: &RawCommand) -> Cmd {
        match command {
            RawCommand::MoveTo(p) => Cmd::MoveTo(self.point(p)),
            RawCommand::LineTo(p) => Cmd::LineTo(self.point(p)),
            RawCommand::ArcTo {
                wr,
                hr,
                st_ang,
                sw_ang,
            } => Cmd::ArcTo {
                wr: self.arg(wr),
                hr: self.arg(hr),
                st: self.arg(st_ang),
                sw: self.arg(sw_ang),
            },
            RawCommand::QuadBezTo(c, e) => Cmd::QuadTo(self.point(c), self.point(e)),
            RawCommand::CubicBezTo(a, b, e) => {
                Cmd::CubicTo(self.point(a), self.point(b), self.point(e))
            }
            RawCommand::Close => Cmd::Close,
        }
    }
}

fn point_count(path: &RawPath) -> usize {
    path.commands
        .iter()
        .map(|command| match command {
            RawCommand::Close => 0usize,
            RawCommand::MoveTo(_) | RawCommand::LineTo(_) | RawCommand::ArcTo { .. } => 1,
            RawCommand::QuadBezTo(..) => 2,
            RawCommand::CubicBezTo(..) => 3,
        })
        .sum::<usize>()
        // A list of nothing but `close` still costs memory, so commands count too.
        .max(path.commands.len())
}

impl RawGeometry {
    /// Compiles to the form the evaluator runs. Guides resolve in document order
    /// against what is already defined, which is why cycles cannot exist: a guide
    /// naming itself or a later guide reads an earlier binding, or 0 and a
    /// diagnostic. Fails only on the caps in [`limits`], checked before any work.
    pub fn compile(&self) -> Result<CompiledGeometry, GeometryError> {
        let guides = self.av.len().saturating_add(self.gd.len());
        if guides > limits::MAX_GUIDES {
            return Err(GeometryError::TooManyGuides);
        }
        if self.paths.len() > limits::MAX_PATHS {
            return Err(GeometryError::TooManyPaths);
        }
        let points = self
            .paths
            .iter()
            .fold(0usize, |sum, path| sum.saturating_add(point_count(path)));
        if points > limits::MAX_PATH_POINTS {
            return Err(GeometryError::TooManyPathPoints);
        }
        if self.handles.len() > limits::MAX_HANDLES {
            return Err(GeometryError::TooManyHandles);
        }
        if self.sites.len() > limits::MAX_SITES {
            return Err(GeometryError::TooManySites);
        }

        let mut compiler = Compiler {
            scope: BTreeMap::new(),
            next_slot: BUILTIN_ORDER.len(),
            diagnostics: Diagnostics::default(),
        };
        let av = compiler.guides(&self.av);
        let gd = compiler.guides(&self.gd);
        let handles = self
            .handles
            .iter()
            .map(|handle| match handle {
                RawHandle::Xy { x, y, pos } => HandleDef::Xy {
                    x: compiler.axis(x),
                    y: compiler.axis(y),
                    pos: compiler.point(pos),
                },
                RawHandle::Polar { r, ang, pos } => HandleDef::Polar {
                    r: compiler.axis(r),
                    ang: compiler.axis(ang),
                    pos: compiler.point(pos),
                },
            })
            .collect::<Vec<_>>();
        let sites = self
            .sites
            .iter()
            .map(|site| SiteDef {
                ang: compiler.arg(&site.ang),
                pos: compiler.point(&site.pos),
            })
            .collect::<Vec<_>>();
        let rect = self.rect.as_ref().map(|rect| RectDef {
            l: compiler.arg(&rect.l),
            t: compiler.arg(&rect.t),
            r: compiler.arg(&rect.r),
            b: compiler.arg(&rect.b),
        });
        let paths = self
            .paths
            .iter()
            .map(|path| PathDef {
                w: path.w.max(0),
                h: path.h.max(0),
                fill: path.fill,
                stroke: path.stroke,
                extrusion_ok: path.extrusion_ok,
                cmds: Cow::Owned(path.commands.iter().map(|c| compiler.command(c)).collect()),
            })
            .collect::<Vec<_>>();
        Ok(CompiledGeometry {
            def: GeometryDef {
                name: Cow::Borrowed(""),
                av: Cow::Owned(av),
                gd: Cow::Owned(gd),
                handles: Cow::Owned(handles),
                sites: Cow::Owned(sites),
                rect,
                paths: Cow::Owned(paths),
            },
            diagnostics: compiler.diagnostics,
        })
    }
}
