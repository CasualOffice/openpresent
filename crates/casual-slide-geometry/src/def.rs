//! Compiled geometry: guide programs and paths with names already resolved to
//! slots. Generated preset tables and compiled custom geometry share this form,
//! so one evaluator serves both.

use std::borrow::Cow;

/// A guide operand: a literal, or the slot of an earlier guide or built-in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arg {
    /// Literal value.
    Lit(i64),
    /// Evaluation slot.
    Slot(u16),
}

/// Guide operators (ECMA-376 Part 1, 20.1.9.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// `val x`: x
    Val,
    /// `abs x`: |x|
    Abs,
    /// `sqrt x`: square root of x; 0 when x is negative
    Sqrt,
    /// `at2 x y`: arctangent of y/x, as an angle
    At2,
    /// `cos x y`: x * cos(y)
    Cos,
    /// `sin x y`: x * sin(y)
    Sin,
    /// `tan x y`: x * tan(y); 0 at a pole
    Tan,
    /// `max x y`
    Max,
    /// `min x y`
    Min,
    /// `*/ x y z`: x * y / z; 0 when z is 0
    MulDiv,
    /// `+- x y z`: x + y - z
    AddSub,
    /// `+/ x y z`: (x + y) / z; 0 when z is 0
    AddDiv,
    /// `?: x y z`: y if x > 0, else z
    If,
    /// `cat2 x y z`: x * cos(arctan(z / y))
    Cat2,
    /// `sat2 x y z`: x * sin(arctan(z / y))
    Sat2,
    /// `mod x y z`: sqrt(x^2 + y^2 + z^2)
    Mod,
    /// `pin x y z`: y clamped between x and z, testing x first
    Pin,
}

/// One named guide.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuideDef {
    /// Name, as written. Repeats are legal: later uses see the latest definition.
    pub name: Cow<'static, str>,
    /// Operator.
    pub op: Op,
    /// First operand.
    pub a: Arg,
    /// Second operand; unused operands are `Lit(0)`.
    pub b: Arg,
    /// Third operand.
    pub c: Arg,
}

/// A point of guide references.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PtDef {
    /// Horizontal coordinate.
    pub x: Arg,
    /// Vertical coordinate.
    pub y: Arg,
}

/// One path command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cmd {
    /// `a:moveTo`
    MoveTo(PtDef),
    /// `a:lnTo`
    LineTo(PtDef),
    /// `a:arcTo`
    ArcTo {
        /// Horizontal radius.
        wr: Arg,
        /// Vertical radius.
        hr: Arg,
        /// Start angle.
        st: Arg,
        /// Sweep angle.
        sw: Arg,
    },
    /// `a:quadBezTo`
    QuadTo(PtDef, PtDef),
    /// `a:cubicBezTo`
    CubicTo(PtDef, PtDef, PtDef),
    /// `a:close`
    Close,
}

/// `ST_PathFillMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FillMode {
    /// `none`
    None,
    /// `norm`
    #[default]
    Norm,
    /// `lighten`
    Lighten,
    /// `lightenLess`
    LightenLess,
    /// `darken`
    Darken,
    /// `darkenLess`
    DarkenLess,
}

impl FillMode {
    /// Parses the schema token.
    pub fn parse(token: &str) -> Option<FillMode> {
        Some(match token {
            "none" => FillMode::None,
            "norm" => FillMode::Norm,
            "lighten" => FillMode::Lighten,
            "lightenLess" => FillMode::LightenLess,
            "darken" => FillMode::Darken,
            "darkenLess" => FillMode::DarkenLess,
            _ => return None,
        })
    }
}

/// One `a:path`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathDef {
    /// Path coordinate width; 0 means the shape's own.
    pub w: i64,
    /// Path coordinate height; 0 means the shape's own.
    pub h: i64,
    /// Fill mode.
    pub fill: FillMode,
    /// Whether the path is stroked.
    pub stroke: bool,
    /// Whether the path may be extruded.
    pub extrusion_ok: bool,
    /// Commands in document order.
    pub cmds: Cow<'static, [Cmd]>,
}

/// One adjust handle axis: the guide it edits and its limits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxisDef {
    /// Referenced guide, by name.
    pub guide: Option<Cow<'static, str>>,
    /// Lower limit.
    pub min: Option<Arg>,
    /// Upper limit.
    pub max: Option<Arg>,
}

/// One adjust handle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HandleDef {
    /// `a:ahXY`
    Xy {
        /// Horizontal axis.
        x: AxisDef,
        /// Vertical axis.
        y: AxisDef,
        /// Handle position.
        pos: PtDef,
    },
    /// `a:ahPolar`
    Polar {
        /// Radius axis.
        r: AxisDef,
        /// Angle axis.
        ang: AxisDef,
        /// Handle position.
        pos: PtDef,
    },
}

/// One `a:cxn`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SiteDef {
    /// Incoming connector angle.
    pub ang: Arg,
    /// Site position.
    pub pos: PtDef,
}

/// `a:rect`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RectDef {
    /// Left.
    pub l: Arg,
    /// Top.
    pub t: Arg,
    /// Right.
    pub r: Arg,
    /// Bottom.
    pub b: Arg,
}

/// A compiled geometry: a preset from the generated table, or custom geometry
/// from [`crate::RawGeometry::compile`].
///
/// Slots run: built-ins, then `av`, then `gd`, in order. The evaluator is a single
/// forward pass over them. An operand can only name an earlier slot, because names
/// resolve while compiling, so reference cycles cannot be expressed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeometryDef {
    /// Preset name; empty for custom geometry.
    pub name: Cow<'static, str>,
    /// Adjust guides.
    pub av: Cow<'static, [GuideDef]>,
    /// Geometry guides.
    pub gd: Cow<'static, [GuideDef]>,
    /// Adjust handles.
    pub handles: Cow<'static, [HandleDef]>,
    /// Connection sites; the index is persisted by connectors.
    pub sites: Cow<'static, [SiteDef]>,
    /// Text rectangle; `None` means the shape's bounds.
    pub rect: Option<RectDef>,
    /// Paths.
    pub paths: Cow<'static, [PathDef]>,
}

/// Terse constructors for the generated table.
pub(crate) mod ctor {
    use super::{Arg, AxisDef, Cmd, GuideDef, HandleDef, Op, PtDef, RectDef, SiteDef};
    use std::borrow::Cow::{self, Borrowed};

    pub(crate) const fn g(name: &'static str, op: Op, a: Arg, b: Arg, c: Arg) -> GuideDef {
        GuideDef {
            name: Borrowed(name),
            op,
            a,
            b,
            c,
        }
    }
    const fn pt(x: Arg, y: Arg) -> PtDef {
        PtDef { x, y }
    }
    pub(crate) const fn m(x: Arg, y: Arg) -> Cmd {
        Cmd::MoveTo(pt(x, y))
    }
    pub(crate) const fn l(x: Arg, y: Arg) -> Cmd {
        Cmd::LineTo(pt(x, y))
    }
    pub(crate) const fn a(wr: Arg, hr: Arg, st: Arg, sw: Arg) -> Cmd {
        Cmd::ArcTo { wr, hr, st, sw }
    }
    pub(crate) const fn q(x1: Arg, y1: Arg, x2: Arg, y2: Arg) -> Cmd {
        Cmd::QuadTo(pt(x1, y1), pt(x2, y2))
    }
    pub(crate) const fn c(x1: Arg, y1: Arg, x2: Arg, y2: Arg, x3: Arg, y3: Arg) -> Cmd {
        Cmd::CubicTo(pt(x1, y1), pt(x2, y2), pt(x3, y3))
    }
    pub(crate) const fn s(ang: Arg, x: Arg, y: Arg) -> SiteDef {
        SiteDef { ang, pos: pt(x, y) }
    }
    pub(crate) const fn r(l: Arg, t: Arg, r: Arg, b: Arg) -> RectDef {
        RectDef { l, t, r, b }
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn hxy(
        x: Option<Cow<'static, str>>,
        min_x: Option<Arg>,
        max_x: Option<Arg>,
        y: Option<Cow<'static, str>>,
        min_y: Option<Arg>,
        max_y: Option<Arg>,
        px: Arg,
        py: Arg,
    ) -> HandleDef {
        HandleDef::Xy {
            x: AxisDef {
                guide: x,
                min: min_x,
                max: max_x,
            },
            y: AxisDef {
                guide: y,
                min: min_y,
                max: max_y,
            },
            pos: pt(px, py),
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn hpolar(
        r: Option<Cow<'static, str>>,
        min_r: Option<Arg>,
        max_r: Option<Arg>,
        ang: Option<Cow<'static, str>>,
        min_ang: Option<Arg>,
        max_ang: Option<Arg>,
        px: Arg,
        py: Arg,
    ) -> HandleDef {
        HandleDef::Polar {
            r: AxisDef {
                guide: r,
                min: min_r,
                max: max_r,
            },
            ang: AxisDef {
                guide: ang,
                min: min_ang,
                max: max_ang,
            },
            pos: pt(px, py),
        }
    }
}
