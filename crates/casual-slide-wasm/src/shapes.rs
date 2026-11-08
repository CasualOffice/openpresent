//! PowerPoint's shape experience (`SHAPE-01`): every preset of the Shapes gallery,
//! drawn as an icon by the geometry engine; inserting one at a drawn box; the yellow
//! adjust handles, which edit `a:avLst`; and the Shape Format tab's fill, outline and
//! theme styles. Every change is one [`Operation::SetLook`] or `InsertNode`, with its
//! inverse, in the one history.
//!
//! What crosses the boundary is numbers and short tokens the host built: a preset's
//! name, a colour as `scheme accent1 lumMod 75000` or `srgb 4472C4`. Each is checked
//! against a closed list here, never trusted.

use casual_slide_geometry::{Adjust, AdjustHandle, Extents, GeometryDef, Segment, preset_shape};
use casual_slide_model::color::{
    Color, ColorTransform, ColorValue, ResolveContext, SchemeColor, TransformKind, resolve,
};
use casual_slide_model::drawing::ShapeProperties;
use casual_slide_model::fill::Fill;
use casual_slide_model::geometry::{GeomGuide, Geometry};
use casual_slide_model::line::{Line, LineDash, PresetLineDash};
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode};
use casual_slide_model::theme::{FontCollectionIndex, FontRef, ShapeStyle, StyleMatrixRef};
use casual_slide_transaction::Operation;

use crate::build::{Ends, Insert};
use crate::{BridgeError, Viewer};

/// The widest line Shape Outline sets, in EMU: PowerPoint's 1584 points.
pub const MAX_LINE_WIDTH: i32 = 20_116_800;

/// Points a 1-D handle search samples before it refines.
const SAMPLES: usize = 48;
/// Refinement steps after sampling.
const REFINE: usize = 24;

/// A preset's outline as SVG path data at `w` by `h` units, for the gallery's icons:
/// one line a path, two flag characters and then the path. The first is the fill
/// mode: `n` none, `f` normal, `l`/`L` lighten (less), `d`/`D` darken (less); the
/// second is `s` for a stroked path, `-` for one that is not. `None` for a name the
/// preset table lacks.
pub fn preset_outline(name: &str, w: f64, h: f64) -> Option<String> {
    use casual_slide_geometry::FillMode;
    let def = preset_shape(name)?;
    let w = if w.is_finite() {
        w.clamp(1.0, 4096.0)
    } else {
        32.0
    };
    let h = if h.is_finite() {
        h.clamp(1.0, 4096.0)
    } else {
        32.0
    };
    // Evaluate at a size in EMU-like units so the integer guides keep precision.
    let k = 12_700.0;
    let extents = Extents::new((w * k) as i64, (h * k) as i64);
    let resolved = def.evaluate(extents, &[]);
    let mut out = String::new();
    for path in &resolved.paths {
        let path = path.scaled_to(extents).with_arcs_as_cubics();
        let fill = match path.fill {
            FillMode::None => 'n',
            FillMode::Norm => 'f',
            FillMode::Lighten => 'l',
            FillMode::LightenLess => 'L',
            FillMode::Darken => 'd',
            FillMode::DarkenLess => 'D',
        };
        out.push(fill);
        out.push(if path.stroke { 's' } else { '-' });
        let p = |pt: casual_slide_geometry::Point| format!("{:.2} {:.2}", pt.x / k, pt.y / k);
        for seg in &path.segments {
            let text = match *seg {
                Segment::MoveTo(a) => format!(" M{}", p(a)),
                Segment::LineTo(a) => format!(" L{}", p(a)),
                Segment::QuadTo { ctrl, end } => format!(" Q{} {}", p(ctrl), p(end)),
                Segment::CubicTo { c1, c2, end } => format!(" C{} {} {}", p(c1), p(c2), p(end)),
                // `with_arcs_as_cubics` leaves none; draw a chord if one stayed.
                Segment::ArcTo(a) => format!(" L{}", p(a.end)),
                Segment::Close => " Z".to_owned(),
            };
            out.push_str(&text);
        }
        out.push('\n');
    }
    Some(out)
}

/// A colour from the host's short form: `srgb RRGGBB`, or `scheme <token>` followed
/// by up to four `lumMod`/`lumOff`/`shade`/`tint` transforms with a value from 0 to
/// 100000.
pub fn parse_color(spec: &str) -> Option<Color> {
    let mut words = spec.split_ascii_whitespace();
    let value = match words.next()? {
        "srgb" => {
            let hex = words.next()?;
            if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
                return None;
            }
            let n = u32::from_str_radix(hex, 16).ok()?;
            let [_, r, g, b] = n.to_be_bytes();
            return words.next().is_none().then(|| Color::srgb(r, g, b));
        }
        "scheme" => ColorValue::Scheme(SchemeColor::from_xml(words.next()?)?),
        _ => return None,
    };
    let mut color = Color::new(value);
    while let Some(kind) = words.next() {
        let kind = match kind {
            "lumMod" => TransformKind::LumMod,
            "lumOff" => TransformKind::LumOff,
            "shade" => TransformKind::Shade,
            "tint" => TransformKind::Tint,
            _ => return None,
        };
        let v: i32 = words.next()?.parse().ok()?;
        if !(0..=100_000).contains(&v) || color.transforms.len() >= 4 {
            return None;
        }
        color.transforms.push(ColorTransform::new(kind, v));
    }
    Some(color)
}

/// One of the Shape Styles gallery's theme styles: `row` 0 Colored Outline, 1
/// Colored Fill, 2 Light 1 Outline, Colored Fill, 3 Subtle Effect, 4 Moderate Effect,
/// 5 Intense Effect; `column` 0 Dark 1, 1 to 6 the accents.
pub fn theme_style(row: u32, column: u32) -> Option<ShapeStyle> {
    let c = match column {
        0 => SchemeColor::Dk1,
        1 => SchemeColor::Accent1,
        2 => SchemeColor::Accent2,
        3 => SchemeColor::Accent3,
        4 => SchemeColor::Accent4,
        5 => SchemeColor::Accent5,
        6 => SchemeColor::Accent6,
        _ => return None,
    };
    let scheme = |s: SchemeColor| Color::new(ColorValue::Scheme(s));
    let shaded = || {
        let mut col = scheme(c);
        col.transforms
            .push(ColorTransform::new(TransformKind::Shade, 50_000));
        col
    };
    let r = |idx: u32, color: Color| {
        Some(StyleMatrixRef {
            idx: Some(idx),
            color: Some(color),
        })
    };
    let (line, fill, effect, font) = match row {
        0 => (
            r(2, scheme(c)),
            r(1, scheme(SchemeColor::Lt1)),
            r(0, scheme(c)),
            SchemeColor::Dk1,
        ),
        1 => (
            r(2, shaded()),
            r(1, scheme(c)),
            r(0, scheme(c)),
            SchemeColor::Lt1,
        ),
        2 => (
            r(3, scheme(SchemeColor::Lt1)),
            r(1, scheme(c)),
            r(1, scheme(c)),
            SchemeColor::Lt1,
        ),
        3 => (
            r(1, scheme(c)),
            r(2, scheme(c)),
            r(1, scheme(c)),
            SchemeColor::Dk1,
        ),
        4 => (
            r(0, scheme(c)),
            r(3, scheme(c)),
            r(2, scheme(c)),
            SchemeColor::Lt1,
        ),
        5 => (
            r(0, scheme(c)),
            r(3, scheme(c)),
            r(3, scheme(c)),
            SchemeColor::Lt1,
        ),
        _ => return None,
    };
    Some(ShapeStyle {
        line,
        fill,
        effect,
        font: Some(FontRef {
            idx: Some(FontCollectionIndex::Minor),
            color: Some(scheme(font)),
        }),
    })
}

/// A preset's adjust values for `evaluate`: the shape's own `avLst`, as written.
fn own_adjusts(
    geometry: Option<&Geometry>,
) -> Option<(&'static GeometryDef, Vec<(String, String)>)> {
    let Some(Geometry::Preset(p)) = geometry else {
        return None;
    };
    let def = preset_shape(&p.preset?.to_lexical())?;
    let adjusts = p
        .adjust
        .iter()
        .flatten()
        .filter_map(|g| Some((g.name.clone()?, g.formula.clone()?)))
        .collect();
    Some((def, adjusts))
}

fn as_adjusts(list: &[(String, String)]) -> Vec<Adjust<'_>> {
    list.iter().map(|(n, f)| Adjust::formula(n, f)).collect()
}

/// A shape's placement: its box and how it is turned, to carry points between
/// the slide and the shape's own frame.
#[derive(Debug, Clone, Copy)]
struct Frame {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    rot: f64,
    flip_h: bool,
    flip_v: bool,
}

impl Frame {
    fn to_slide(self, px: f64, py: f64) -> (f64, f64) {
        let (mut lx, mut ly) = (px - self.w / 2.0, py - self.h / 2.0);
        if self.flip_h {
            lx = -lx;
        }
        if self.flip_v {
            ly = -ly;
        }
        let (s, c) = self.rot.sin_cos();
        (
            self.x + self.w / 2.0 + lx * c - ly * s,
            self.y + self.h / 2.0 + lx * s + ly * c,
        )
    }

    fn to_shape(self, sx: f64, sy: f64) -> (f64, f64) {
        let (dx, dy) = (sx - self.x - self.w / 2.0, sy - self.y - self.h / 2.0);
        let (s, c) = self.rot.sin_cos();
        let (mut lx, mut ly) = (dx * c + dy * s, -dx * s + dy * c);
        if self.flip_h {
            lx = -lx;
        }
        if self.flip_v {
            ly = -ly;
        }
        (lx + self.w / 2.0, ly + self.h / 2.0)
    }
}

fn handle_pos(h: &AdjustHandle<'_>) -> (f64, f64) {
    match h {
        AdjustHandle::Xy { pos, .. } | AdjustHandle::Polar { pos, .. } => (pos.x, pos.y),
    }
}

/// The guide one axis of a handle moves, with its range.
fn axes<'d>(h: &AdjustHandle<'d>) -> Vec<(&'d str, f64, f64)> {
    let list = match h {
        AdjustHandle::Xy { x, y, .. } => [x, y],
        AdjustHandle::Polar { r, ang, .. } => [r, ang],
    };
    let mut out: Vec<(&'d str, f64, f64)> = Vec::new();
    for a in list {
        let Some(g) = a.guide else { continue };
        if out.iter().any(|o| o.0 == g) {
            continue;
        }
        let (lo, hi) = match (a.min, a.max) {
            (Some(lo), Some(hi)) if lo <= hi => (lo, hi),
            (Some(hi), Some(lo)) => (lo, hi),
            _ => (-100_000.0, 200_000.0),
        };
        out.push((g, lo, hi));
    }
    out
}

/// Set `name` to `value` in an adjust list, adding it if absent.
fn put(list: &mut Vec<(String, String)>, name: &str, value: i64) {
    let fmla = format!("val {value}");
    match list.iter_mut().find(|(n, _)| n == name) {
        Some(entry) => entry.1 = fmla,
        None => list.push((name.to_owned(), fmla)),
    }
}

/// The adjust values that bring handle `index` nearest `target`, in shape EMU:
/// each guide the handle moves is searched over its range, holding the others,
/// twice round. Pure, so a test can drive it without a deck.
pub(crate) fn solve_handle(
    def: &GeometryDef,
    extents: Extents,
    start: &[(String, String)],
    index: usize,
    target: (f64, f64),
) -> Option<Vec<(String, String)>> {
    let first = def.evaluate(extents, &as_adjusts(start));
    let handle = first.adjust_handles.get(index)?;
    let guides: Vec<(String, f64, f64)> = axes(handle)
        .into_iter()
        .map(|(g, lo, hi)| (g.to_owned(), lo, hi))
        .collect();
    if guides.is_empty() {
        return None;
    }
    let mut list = start.to_vec();
    let distance = |list: &[(String, String)]| -> f64 {
        let r = def.evaluate(extents, &as_adjusts(list));
        r.adjust_handles.get(index).map_or(f64::INFINITY, |h| {
            let (x, y) = handle_pos(h);
            (x - target.0).powi(2) + (y - target.1).powi(2)
        })
    };
    for _ in 0..2 {
        for (name, lo, hi) in &guides {
            let mut probe = list.clone();
            let at = |v: f64, probe: &mut Vec<(String, String)>| {
                put(probe, name, v.round() as i64);
                distance(probe)
            };
            let step = (hi - lo) / SAMPLES as f64;
            let (mut best, mut best_d) = (*lo, f64::INFINITY);
            for i in 0..=SAMPLES {
                let v = lo + step * i as f64;
                let d = at(v, &mut probe);
                if d < best_d {
                    (best, best_d) = (v, d);
                }
            }
            let (mut a, mut b) = ((best - step).max(*lo), (best + step).min(*hi));
            for _ in 0..REFINE {
                let m1 = a + (b - a) / 3.0;
                let m2 = b - (b - a) / 3.0;
                if at(m1, &mut probe) <= at(m2, &mut probe) {
                    b = m2;
                } else {
                    a = m1;
                }
            }
            let v = (a + b) / 2.0;
            if at(v, &mut probe) <= best_d {
                best = v;
            }
            put(&mut list, name, best.round() as i64);
        }
    }
    Some(list)
}

impl Viewer {
    fn top_shape(
        &self,
        index: usize,
        node: u32,
    ) -> Result<(ObjectId, &ShapeNode, Frame), BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let id = ObjectId {
            part: ordinal,
            node,
        };
        let parts = self.deck.parts();
        let shape = parts
            .get(ordinal)
            .and_then(|p| p.node(id))
            .ok_or(BridgeError::BadArgument)?;
        let t = casual_slide_style::Chain::of(parts, ordinal)
            .transform(shape)
            .ok_or(BridgeError::BadArgument)?
            .value;
        let frame = Frame {
            x: t.offset.x.0 as f64,
            y: t.offset.y.0 as f64,
            w: t.extent.cx as f64,
            h: t.extent.cy as f64,
            rot: f64::from(t.rotation) / 60_000.0 * core::f64::consts::PI / 180.0,
            flip_h: t.flip_h,
            flip_v: t.flip_v,
        };
        Ok((id, shape, frame))
    }

    /// Insert a gallery preset on slide `index` at a box in EMU, flipped as the drag
    /// went, named `label` and a number as PowerPoint names it. Returns its node.
    pub fn insert_preset(
        &mut self,
        index: usize,
        preset: &str,
        ends: Ends,
        label: &str,
        b: [i64; 4],
        flip: (bool, bool),
    ) -> Result<u32, BridgeError> {
        let def = preset_shape(preset).ok_or(BridgeError::BadArgument)?;
        let name: &'static str = match &def.name {
            std::borrow::Cow::Borrowed(s) => s,
            std::borrow::Cow::Owned(_) => return Err(BridgeError::BadArgument),
        };
        let label = (!label.trim().is_empty()).then_some(label);
        self.insert_node(index, Insert::Preset(name, ends), None, b, flip, label)
    }

    /// The yellow handles of shape `node` on slide `index`: two numbers each, its
    /// place on the slide in EMU. Empty for a shape whose geometry has none, or is
    /// not a preset.
    pub fn handles(&self, index: usize, node: u32) -> Result<Vec<f64>, BridgeError> {
        let (_, shape, frame) = self.top_shape(index, node)?;
        let Some((def, adjusts)) = own_adjusts(shape.properties.geometry.as_ref()) else {
            return Ok(Vec::new());
        };
        let extents = Extents::new(frame.w as i64, frame.h as i64);
        let resolved = def.evaluate(extents, &as_adjusts(&adjusts));
        let mut out = Vec::new();
        for h in &resolved.adjust_handles {
            let (px, py) = handle_pos(h);
            let (sx, sy) = frame.to_slide(px, py);
            out.extend([sx, sy]);
        }
        Ok(out)
    }

    /// Drag handle `handle` of shape `node` to a point on the slide, in EMU: its
    /// adjust values change, and `a:avLst` with them. `first` starts a gesture; the
    /// moves after it fold into the same undo step until the host ends it with
    /// `break_typing`.
    pub fn drag_handle(
        &mut self,
        index: usize,
        node: u32,
        handle: usize,
        (x, y): (f64, f64),
        first: bool,
    ) -> Result<bool, BridgeError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(BridgeError::BadArgument);
        }
        let (id, shape, frame) = self.top_shape(index, node)?;
        let Some(Geometry::Preset(preset)) = shape.properties.geometry.as_ref() else {
            return Err(BridgeError::BadArgument);
        };
        let (def, adjusts) =
            own_adjusts(shape.properties.geometry.as_ref()).ok_or(BridgeError::BadArgument)?;
        let extents = Extents::new(frame.w as i64, frame.h as i64);
        let target = frame.to_shape(x, y);
        let list =
            solve_handle(def, extents, &adjusts, handle, target).ok_or(BridgeError::BadArgument)?;
        // The list as PowerPoint writes it: the preset's adjust guides, in its order.
        let mut av: Vec<GeomGuide> = Vec::new();
        for g in def.av.iter() {
            let formula = match list.iter().find(|(n, _)| n.as_str() == g.name.as_ref()) {
                Some((_, f)) => f.clone(),
                // Not set on the shape: the preset's own default, as PowerPoint writes it.
                None => match (g.op, g.a) {
                    (casual_slide_geometry::Op::Val, casual_slide_geometry::Arg::Lit(n)) => {
                        format!("val {n}")
                    }
                    _ => continue,
                },
            };
            av.push(GeomGuide {
                name: Some(g.name.to_string()),
                formula: Some(formula),
            });
        }
        let mut geometry = preset.clone();
        geometry.adjust = Some(av);
        let mut properties = shape.properties.clone();
        properties.geometry = Some(Geometry::Preset(geometry));
        let style = shape.style.clone();
        if first {
            self.typing = None;
        }
        self.edit(
            Operation::SetLook {
                node: id,
                properties: Box::new(properties),
                style,
            },
            true,
        )
    }

    fn change_look(
        &mut self,
        index: usize,
        node: u32,
        f: impl FnOnce(&mut ShapeProperties, &mut Option<Box<ShapeStyle>>, NodeKind) -> Option<()>,
    ) -> Result<bool, BridgeError> {
        let (id, shape, _) = self.top_shape(index, node)?;
        if !matches!(shape.kind, NodeKind::Shape | NodeKind::Connector) {
            return Err(BridgeError::BadArgument);
        }
        let mut properties = shape.properties.clone();
        let mut style = shape.style.clone();
        f(&mut properties, &mut style, shape.kind).ok_or(BridgeError::BadArgument)?;
        self.edit(
            Operation::SetLook {
                node: id,
                properties: Box::new(properties),
                style,
            },
            false,
        )
    }

    /// Shape Fill: `none`, or a colour in [`parse_color`]'s form.
    pub fn set_fill(&mut self, index: usize, node: u32, spec: &str) -> Result<bool, BridgeError> {
        let fill = match spec {
            "none" => Fill::None,
            _ => Fill::Solid(Some(parse_color(spec).ok_or(BridgeError::BadArgument)?)),
        };
        self.change_look(index, node, |p, _, _| {
            p.fill = Some(fill);
            Some(())
        })
    }

    /// Shape Outline: `what` 0 sets the colour (`none` or a colour), 1 the weight in
    /// EMU, 2 the dash (an `ST_PresetLineDash` token), 3 the arrowheads (`none`,
    /// `tail`, `head`, `both`).
    pub fn set_outline(
        &mut self,
        index: usize,
        node: u32,
        what: u32,
        value: &str,
    ) -> Result<bool, BridgeError> {
        self.change_look(index, node, |p, _, _| {
            let line = p.line.get_or_insert_with(Box::default);
            match what {
                0 => {
                    line.fill = Some(match value {
                        "none" => Fill::None,
                        spec => Fill::Solid(Some(parse_color(spec)?)),
                    });
                }
                1 => {
                    let w: i32 = value.parse().ok()?;
                    line.width = Some(w.clamp(0, MAX_LINE_WIDTH));
                }
                2 => {
                    line.dash = Some(LineDash::Preset(Some(PresetLineDash::parse_lexical(
                        value,
                    )?)))
                }
                3 => {
                    let end = || {
                        Some(casual_slide_model::line::LineEnd {
                            end_type: Some(casual_slide_model::line::LineEndType::Triangle),
                            ..Default::default()
                        })
                    };
                    let (head, tail) = match value {
                        "none" => (None, None),
                        "tail" => (None, end()),
                        "head" => (end(), None),
                        "both" => (end(), end()),
                        _ => return None,
                    };
                    line.head_end = head;
                    line.tail_end = tail;
                }
                _ => return None,
            }
            Some(())
        })
    }

    /// Shape Styles: a theme style from the gallery ([`theme_style`]). As PowerPoint,
    /// it takes away the shape's own fill and line, keeping a line's arrowheads.
    pub fn set_theme_style(
        &mut self,
        index: usize,
        node: u32,
        row: u32,
        column: u32,
    ) -> Result<bool, BridgeError> {
        let style = theme_style(row, column).ok_or(BridgeError::BadArgument)?;
        self.change_look(index, node, |p, s, _| {
            p.fill = None;
            p.line = p.line.take().and_then(|l| {
                (l.head_end.is_some() || l.tail_end.is_some()).then(|| {
                    Box::new(Line {
                        head_end: l.head_end,
                        tail_end: l.tail_end,
                        ..Line::default()
                    })
                })
            });
            *s = Some(Box::new(style));
            Some(())
        })
    }

    /// The ten colours of slide `index`'s theme as the colour pickers show them, as
    /// `RRGGBB`: Background 1, Text 1, Background 2, Text 2, Accent 1 to 6, through
    /// the slide's colour map. One the theme lacks is empty.
    pub fn theme_colors(&self, index: usize) -> Result<Vec<String>, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let theme = self.deck.theme_of_slide(index);
        let chain = casual_slide_style::Chain::of(self.deck.parts(), ordinal);
        let ctx = ResolveContext {
            scheme: theme
                .and_then(|t| t.elements.as_ref())
                .and_then(|e| e.color_scheme.as_ref()),
            map: chain.color_map().map(|m| m.value),
            placeholder: None,
            system_palette: None,
        };
        Ok([
            SchemeColor::Bg1,
            SchemeColor::Tx1,
            SchemeColor::Bg2,
            SchemeColor::Tx2,
            SchemeColor::Accent1,
            SchemeColor::Accent2,
            SchemeColor::Accent3,
            SchemeColor::Accent4,
            SchemeColor::Accent5,
            SchemeColor::Accent6,
        ]
        .into_iter()
        .map(|s| {
            resolve(&Color::new(ColorValue::Scheme(s)), &ctx).map_or(String::new(), |c| {
                format!("{:02X}{:02X}{:02X}", c.rgb.r, c.rgb.g, c.rgb.b)
            })
        })
        .collect())
    }
}

#[cfg(test)]
mod tests;
