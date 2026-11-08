//! A slide's resolved shapes to a [`DisplayList`]: background, the master's and the
//! layout's visible shapes, then the slide's own, back to front.
//!
//! Effective values come through `casual-slide-style`'s chain (placement, colour map,
//! background, which inherited shapes show) and, for the families `-style` does not
//! resolve yet, by the same rule here: the shape's own value, then its layout and
//! master placeholders', then its style matrix reference into the theme. Geometry is
//! `casual-slide-geometry`'s, evaluated at the shape's size.
//!
//! What this cannot draw yet is counted in [`SceneReport`], never dropped silently:
//! text above all, which lane B's text crate fills as [`DisplayItem::GlyphRun`].

use std::collections::BTreeMap;

use casual_slide_geometry::{Adjust, Extents, FillMode, GeometryDef, Segment, preset_shape};
use casual_slide_model::color::{Color, ResolveContext, resolve};
use casual_slide_model::effect::EffectProperties;
use casual_slide_model::fill::{
    BlipFill, BlipFillMode, Fill, GradientFill, GradientShade, PathShadeType, PatternFill,
};
use casual_slide_model::geometry::Geometry;
use casual_slide_model::line::{Line, LineCap, LineDash, LineJoin, PresetLineDash};
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::{NodeKind, ShapeNode, Transform};
use casual_slide_model::slide::{BackgroundFill, SlideParts};
use casual_slide_model::theme::{ColorScheme, FormatScheme, StyleMatrixRef, Theme};
use casual_slide_style::Chain;

use crate::display::{
    Affine, Cap, DisplayItem, DisplayList, Join, Layer, Paint, PathCmd, Rgba, Stop, Stroke,
};

/// Turns a relationship id on a part into the media part it names.
pub trait MediaResolver {
    /// The package path of the internal target of `rel_id` on part `ordinal`.
    fn media_part(&self, ordinal: u32, rel_id: &str) -> Option<String>;
}

/// A resolver that knows no relationships: every picture is reported.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoMedia;

impl MediaResolver for NoMedia {
    fn media_part(&self, _ordinal: u32, _rel_id: &str) -> Option<String> {
        None
    }
}

/// What building a slide's display list left out or approximated, counted by a
/// stable feature key. Keys name constructs; no document content.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneReport {
    /// Shapes whose text body holds text: drawn as nothing until the text crate
    /// (lane B) supplies glyph runs.
    pub text_bodies_undrawn: u32,
    /// Everything else not drawn, or drawn approximately, by feature.
    pub features: BTreeMap<&'static str, u32>,
}

impl SceneReport {
    fn note(&mut self, feature: &'static str) {
        let n = self.features.entry(feature).or_insert(0);
        *n = n.saturating_add(1);
    }
}

/// Everything a slide's scene is built from.
#[derive(Clone, Copy)]
pub struct SlideScene<'a> {
    /// The deck's slide-like parts.
    pub parts: &'a SlideParts,
    /// The theme of the slide's master, if it has one that could be read.
    pub theme: Option<&'a Theme>,
    /// The slide size, EMU.
    pub size: (i64, i64),
    /// Where pictures' bytes live.
    pub media: &'a dyn MediaResolver,
}

impl core::fmt::Debug for SlideScene<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SlideScene")
            .field("size", &self.size)
            .field("theme", &self.theme.is_some())
            .finish_non_exhaustive()
    }
}

/// The deepest group nesting walked. Import bounds nesting already; this keeps the
/// renderer's recursion bounded on its own terms too.
const MAX_GROUP_DEPTH: usize = 64;

/// Paths and points past this are dropped from one shape and reported.
const MAX_SEGMENTS_PER_SHAPE: usize = 200_000;

/// The schema's default line width, `a:ln/@w` absent, is 0: the thinnest line.
const DEFAULT_LINE_WIDTH: f64 = 0.0;

struct Builder<'a> {
    scene: SlideScene<'a>,
    chain: Chain<'a>,
    items: Vec<DisplayItem>,
    report: SceneReport,
}

/// Build the display list of the part at `ordinal`, which is usually a slide but may
/// be a layout or master, drawn as a slide of its own.
pub fn build_slide(scene: SlideScene<'_>, ordinal: u32) -> (DisplayList, SceneReport) {
    let chain = Chain::of(scene.parts, ordinal);
    let mut b = Builder {
        scene,
        chain,
        items: Vec::new(),
        report: SceneReport::default(),
    };
    b.background();
    let master = chain.master.map(|m| m.id.part);
    for node in chain.inherited_shapes() {
        let layer = if Some(node.id.part) == master {
            Layer::Master
        } else {
            Layer::Layout
        };
        b.node(node, layer, &Affine::IDENTITY, None, 0);
    }
    let own = chain.slide.or(chain.layout).or(chain.master);
    if let Some(tree) = own.and_then(|p| p.shape_tree()) {
        let layer = if chain.slide.is_some() {
            Layer::Slide
        } else if chain.layout.is_some() {
            Layer::Layout
        } else {
            Layer::Master
        };
        for child in &tree.children {
            b.node(child, layer, &Affine::IDENTITY, None, 0);
        }
    }
    let (w, h) = scene.size;
    (
        DisplayList {
            width: w as f64,
            height: h as f64,
            items: b.items,
        },
        b.report,
    )
}

fn rect_path(w: f64, h: f64) -> Vec<PathCmd> {
    vec![
        PathCmd::MoveTo(0.0, 0.0),
        PathCmd::LineTo(w, 0.0),
        PathCmd::LineTo(w, h),
        PathCmd::LineTo(0.0, h),
        PathCmd::Close,
    ]
}

/// A shape's box onto its parent's space: rotation and flips about the box centre.
fn placement(x: f64, y: f64, w: f64, h: f64, rot: i32, flip_h: bool, flip_v: bool) -> Affine {
    let to_centre = Affine::translate(-w / 2.0, -h / 2.0);
    let flip = Affine::scale(
        if flip_h { -1.0 } else { 1.0 },
        if flip_v { -1.0 } else { 1.0 },
    );
    let rotate = Affine::rotate_degrees(f64::from(rot) / 60_000.0);
    let back = Affine::translate(x + w / 2.0, y + h / 2.0);
    back.then_after(&rotate.then_after(&flip.then_after(&to_centre)))
}

fn percent(p: Option<casual_slide_model::color::Percentage>) -> f64 {
    p.map_or(0.0, |p| f64::from(p.0) / 100_000.0)
}

fn rgba(c: casual_slide_model::color::ResolvedColor) -> Rgba {
    let a = (f64::from(c.alpha.min(100_000)) * 255.0 / 100_000.0).round() as u8;
    Rgba {
        r: c.rgb.r,
        g: c.rgb.g,
        b: c.rgb.b,
        a,
    }
}

impl<'a> Builder<'a> {
    fn scheme(&self) -> Option<&'a ColorScheme> {
        self.scene
            .theme
            .and_then(|t| t.elements.as_ref())
            .and_then(|e| e.color_scheme.as_ref())
    }

    fn format(&self) -> Option<&'a FormatScheme> {
        self.scene
            .theme
            .and_then(|t| t.elements.as_ref())
            .and_then(|e| e.format_scheme.as_ref())
    }

    fn color(&mut self, c: &Color, ph: Option<&Color>) -> Option<Rgba> {
        let ctx = ResolveContext {
            scheme: self.scheme(),
            map: self.chain.color_map().map(|m| m.value),
            placeholder: ph,
            system_palette: None,
        };
        match resolve(c, &ctx) {
            Ok(r) => Some(rgba(r)),
            Err(_) => {
                self.report.note("colour (unresolved)");
                None
            }
        }
    }

    fn background(&mut self) {
        let Some(bg) = self.chain.background() else {
            return;
        };
        let (w, h) = (self.scene.size.0 as f64, self.scene.size.1 as f64);
        let paint = match &bg.value.fill {
            Some(BackgroundFill::Reference(r)) => self.style_fill(r, w, h, None),
            Some(BackgroundFill::Properties(p)) => match &p.fill {
                Some(fill) => {
                    let ordinal = match bg.from {
                        casual_slide_style::Level::Slide => self.chain.slide,
                        casual_slide_style::Level::Layout => self.chain.layout,
                        _ => self.chain.master,
                    }
                    .map_or(0, |p| p.id.part);
                    self.fill_paint(fill, None, w, h, ordinal, None)
                }
                None => None,
            },
            None => None,
        };
        if let Some(paint) = paint {
            self.items.push(DisplayItem::Path {
                object: None,
                layer: Layer::Background,
                transform: Affine::IDENTITY,
                path: rect_path(w, h),
                fill: Some(paint),
                stroke: None,
            });
        }
    }

    /// A style matrix fill reference, as a paint over a `w` by `h` box.
    fn style_fill(
        &mut self,
        r: &StyleMatrixRef,
        w: f64,
        h: f64,
        ordinal: Option<u32>,
    ) -> Option<Paint> {
        let idx = r.idx?;
        let Some(fill) = self.format().and_then(|f| f.fill(idx)) else {
            if !matches!(idx, 0 | 1000) {
                self.report.note("style fill (not in theme)");
            }
            return None;
        };
        // A picture in the theme is relative to the theme part, which this has no
        // resolver for yet.
        if matches!(fill, Fill::Blip(_)) {
            self.report.note("theme picture fill");
            return None;
        }
        self.fill_paint(fill, r.color.as_ref(), w, h, ordinal.unwrap_or(0), None)
    }

    /// A fill as a paint over a `w` by `h` box. `group` is the fill `grpFill` means.
    fn fill_paint(
        &mut self,
        fill: &Fill,
        ph: Option<&Color>,
        w: f64,
        h: f64,
        ordinal: u32,
        group: Option<&Fill>,
    ) -> Option<Paint> {
        match fill {
            Fill::None => None,
            Fill::Solid(c) => self.color(c.as_ref()?, ph).map(Paint::Solid),
            Fill::Gradient(g) => self.gradient(g, ph, w, h),
            Fill::Pattern(p) => self.pattern(p, ph),
            Fill::Blip(b) => self.picture(b, ordinal, w, h),
            Fill::Group => match group {
                Some(g) if !matches!(g, Fill::Group) => self.fill_paint(g, ph, w, h, ordinal, None),
                _ => None,
            },
        }
    }

    fn stops(&mut self, g: &GradientFill, ph: Option<&Color>) -> Vec<Stop> {
        let mut stops: Vec<Stop> = Vec::new();
        for s in g.stops.iter().flatten() {
            let Some(c) = s.color.as_ref().and_then(|c| self.color(c, ph)) else {
                continue;
            };
            stops.push(Stop {
                offset: percent(s.position).clamp(0.0, 1.0),
                color: c,
            });
        }
        stops.sort_by(|a, b| a.offset.total_cmp(&b.offset));
        stops
    }

    fn gradient(&mut self, g: &GradientFill, ph: Option<&Color>, w: f64, h: f64) -> Option<Paint> {
        let stops = self.stops(g, ph);
        if stops.is_empty() {
            return None;
        }
        if let [only] = stops.as_slice() {
            return Some(Paint::Solid(only.color));
        }
        match g.shade {
            Some(GradientShade::Path { path, fill_to_rect }) => {
                if !matches!(path, Some(PathShadeType::Circle)) {
                    self.report.note("path gradient (drawn radial)");
                }
                let r = fill_to_rect.unwrap_or_default();
                let (l, t, rr, b) = (percent(r.l), percent(r.t), percent(r.r), percent(r.b));
                let cx = w * (l + (1.0 - rr)) / 2.0;
                let cy = h * (t + (1.0 - b)) / 2.0;
                let radius = [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)]
                    .iter()
                    .map(|(x, y)| ((x - cx) * (x - cx) + (y - cy) * (y - cy)).sqrt())
                    .fold(0.0, f64::max);
                Some(Paint::Radial {
                    center: (cx, cy),
                    radius,
                    stops,
                })
            }
            Some(GradientShade::Linear { angle, scaled }) => Some(linear(
                angle.map_or(0, |a| a.0),
                scaled.unwrap_or(false),
                w,
                h,
                stops,
            )),
            // Neither `lin` nor `path`: drawn as `lin` at 0, Inferred.
            None => Some(linear(0, false, w, h, stops)),
        }
    }

    fn pattern(&mut self, p: &PatternFill, ph: Option<&Color>) -> Option<Paint> {
        self.report.note("pattern fill (drawn as a blend)");
        let fg = p.foreground.as_ref().and_then(|c| self.color(c, ph));
        let bg = p.background.as_ref().and_then(|c| self.color(c, ph));
        let weight = p.preset.map_or(0.5, |preset| {
            let token = preset.to_lexical();
            token
                .strip_prefix("pct")
                .and_then(|n| n.parse::<f64>().ok())
                .map_or(0.5, |n| n / 100.0)
        });
        match (fg, bg) {
            (Some(f), Some(b)) => {
                let mix = |x: u8, y: u8| {
                    (f64::from(x) * weight + f64::from(y) * (1.0 - weight)).round() as u8
                };
                Some(Paint::Solid(Rgba {
                    r: mix(f.r, b.r),
                    g: mix(f.g, b.g),
                    b: mix(f.b, b.b),
                    a: mix(f.a, b.a),
                }))
            }
            (Some(c), None) | (None, Some(c)) => Some(Paint::Solid(c)),
            (None, None) => None,
        }
    }

    fn picture(&mut self, b: &BlipFill, ordinal: u32, w: f64, h: f64) -> Option<Paint> {
        let blip = b.blip.as_ref()?;
        let Some(rel) = blip.embed.as_deref() else {
            if blip.link.is_some() {
                // Never fetched (SEC-N01).
                self.report.note("linked picture (not fetched)");
            }
            return None;
        };
        let Some(part) = self.scene.media.media_part(ordinal, rel) else {
            self.report.note("picture (relationship unresolved)");
            return None;
        };
        if !blip.effects.is_empty() {
            self.report.note("picture effect");
        }
        // The stretch rectangle, then the crop: the picture's own frame grows by what
        // the crop cuts off, so the kept part lands on the box.
        let mut frame = (0.0, 0.0, w, h);
        match b.mode {
            Some(BlipFillMode::Stretch(Some(r))) => {
                let (l, t, rr, bb) = (percent(r.l), percent(r.t), percent(r.r), percent(r.b));
                frame = (l * w, t * h, w * (1.0 - l - rr), h * (1.0 - t - bb));
            }
            Some(BlipFillMode::Tile(_)) => self.report.note("tiled picture fill (drawn stretched)"),
            _ => {}
        }
        if let Some(src) = b.source_rect {
            let (l, t, r, bb) = (
                percent(src.l),
                percent(src.t),
                percent(src.r),
                percent(src.b),
            );
            let kw = 1.0 - l - r;
            let kh = 1.0 - t - bb;
            if kw > 1e-6 && kh > 1e-6 {
                let fw = frame.2 / kw;
                let fh = frame.3 / kh;
                frame = (frame.0 - l * fw, frame.1 - t * fh, fw, fh);
            }
        }
        Some(Paint::Picture {
            part,
            frame,
            alpha: 1.0,
        })
    }

    /// The shape and the placeholders it inherits from, nearest first.
    fn levels(&self, node: &'a ShapeNode) -> Vec<&'a ShapeNode> {
        let inherited = self.chain.inherited(node);
        let mut out = vec![node];
        out.extend(inherited.layout);
        out.extend(inherited.master);
        out
    }

    fn node(
        &mut self,
        node: &'a ShapeNode,
        layer: Layer,
        parent: &Affine,
        group_fill: Option<&'a Fill>,
        depth: usize,
    ) {
        if node.non_visual.as_ref().and_then(|v| v.hidden) == Some(true) {
            return;
        }
        if depth > MAX_GROUP_DEPTH {
            self.report.note("group (nested too deep)");
            return;
        }
        match node.kind {
            NodeKind::Group => {
                let Some(t) = node.transform.as_ref() else {
                    // A group with no placement: its children's own coordinates.
                    for child in &node.children {
                        self.node(child, layer, parent, group_fill, depth + 1);
                    }
                    return;
                };
                let mapping = parent.then_after(&group_mapping(t));
                let fill = node.properties.fill.as_ref().or(group_fill);
                for child in &node.children {
                    self.node(child, layer, &mapping, fill, depth + 1);
                }
            }
            NodeKind::Shape | NodeKind::Connector | NodeKind::Picture => {
                self.shape(node, layer, parent, group_fill);
            }
            NodeKind::GraphicFrame => {
                if node.table.is_some() {
                    self.report.note("table");
                } else if node.chart.is_some() {
                    self.report.note("chart");
                } else if node.diagram.is_some() {
                    self.report.note("diagram");
                } else {
                    self.report.note("graphic frame");
                }
            }
            NodeKind::ContentPart => self.report.note("content part"),
        }
    }

    fn shape(
        &mut self,
        node: &'a ShapeNode,
        layer: Layer,
        parent: &Affine,
        group_fill: Option<&'a Fill>,
    ) {
        let Some(t) = self.chain.transform(node).map(|t| t.value) else {
            if node.text_body.is_some() || node.properties.geometry.is_some() {
                self.report.note("shape (no placement)");
            }
            return;
        };
        let (w, h) = (t.extent.cx as f64, t.extent.cy as f64);
        let transform = parent.then_after(&placement(
            t.offset.x.0 as f64,
            t.offset.y.0 as f64,
            w,
            h,
            t.rotation,
            t.flip_h,
            t.flip_v,
        ));
        let levels = self.levels(node);
        let ordinal = node.id.part;

        // The fill: own, inherited, then the style matrix.
        let style = levels.iter().find_map(|n| n.style.as_deref());
        let fill_source = levels
            .iter()
            .find_map(|n| n.properties.fill.as_ref().map(|f| (f, n.id.part)));
        let fill = match fill_source {
            Some((f, part)) => self.fill_paint(f, None, w, h, part, group_fill),
            None => match style.and_then(|s| s.fill.as_ref()) {
                Some(r) => self.style_fill(r, w, h, Some(ordinal)),
                None => None,
            },
        };
        let stroke = self.stroke(
            &levels,
            style.and_then(|s| s.line.as_ref()),
            w,
            h,
            group_fill,
        );
        let picture = if node.kind == NodeKind::Picture {
            node.properties
                .picture
                .as_deref()
                .and_then(|b| self.picture(b, ordinal, w, h))
        } else {
            None
        };
        // The nearest stated effect list wins, as for fills; an empty one is none.
        let own_effects = levels.iter().find_map(|n| n.properties.effects.as_ref());
        let style_effects = style
            .and_then(|s| s.effect.as_ref())
            .and_then(|e| e.idx)
            .and_then(|i| self.format().and_then(|f| f.effect(i)))
            .and_then(|e| e.effects.as_ref());
        if own_effects.or(style_effects).is_some_and(has_effect) {
            self.report.note("effects");
        }
        if node.properties.scene3d.is_some() || node.properties.sp3d.is_some() {
            self.report.note("3D");
        }
        if node
            .text_body
            .as_ref()
            .is_some_and(|b| b.paragraphs.iter().any(|p| !p.items.is_empty()))
        {
            self.report.text_bodies_undrawn = self.report.text_bodies_undrawn.saturating_add(1);
        }

        let geometry = levels.iter().find_map(|n| n.properties.geometry.as_ref());
        let paths = self.paths(geometry, w, h, node.kind);
        for (path, mode, stroked) in paths {
            let fill = match mode {
                FillMode::None => None,
                _ => fill.clone().map(|p| shade(p, mode)),
            };
            let stroke = if stroked { stroke.clone() } else { None };
            if let Some(pic) = &picture {
                // A picture's own fill sits under it; its line over it.
                if fill.is_some() {
                    self.push(node, layer, transform, path.clone(), fill, None);
                }
                self.push(node, layer, transform, path, Some(pic.clone()), stroke);
            } else if fill.is_some() || stroke.is_some() {
                self.push(node, layer, transform, path, fill, stroke);
            }
        }
    }

    fn push(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        transform: Affine,
        path: Vec<PathCmd>,
        fill: Option<Paint>,
        stroke: Option<Stroke>,
    ) {
        self.items.push(DisplayItem::Path {
            object: Some(node.id),
            layer,
            transform,
            path,
            fill,
            stroke,
        });
    }

    fn stroke(
        &mut self,
        levels: &[&'a ShapeNode],
        style: Option<&'a StyleMatrixRef>,
        w: f64,
        h: f64,
        group_fill: Option<&'a Fill>,
    ) -> Option<Stroke> {
        // Field by field: the shape's own line over its placeholders', over the theme
        // line its style names. The style's colour stands for `phClr`.
        let theme_line = style.and_then(|r| {
            let idx = r.idx?;
            self.format().and_then(|f| f.line(idx))
        });
        let mut merged = theme_line.cloned().unwrap_or_default();
        for n in levels.iter().rev() {
            if let Some(own) = n.properties.line.as_deref() {
                overlay(&mut merged, own);
            }
        }
        let ph = style.and_then(|r| r.color.as_ref());
        let ordinal = levels.first().map_or(0, |n| n.id.part);
        let paint = self.fill_paint(merged.fill.as_ref()?, ph, w, h, ordinal, group_fill)?;
        let width = merged.width.map_or(DEFAULT_LINE_WIDTH, f64::from);
        if merged
            .head_end
            .and_then(|e| e.end_type)
            .is_some_and(|t| t.to_lexical() != "none")
            || merged
                .tail_end
                .and_then(|e| e.end_type)
                .is_some_and(|t| t.to_lexical() != "none")
        {
            self.report.note("line end");
        }
        if merged.compound.is_some_and(|c| c.to_lexical() != "sng") {
            self.report.note("compound line (drawn single)");
        }
        let unit = width.max(9_525.0);
        let dash = match &merged.dash {
            Some(LineDash::Preset(Some(p))) => preset_dash(*p).iter().map(|d| d * unit).collect(),
            Some(LineDash::Custom(stops)) => stops
                .iter()
                .flat_map(|s| [percent(s.dash) * unit, percent(s.space) * unit])
                .collect(),
            _ => Vec::new(),
        };
        let cap = match merged.cap {
            Some(LineCap::Rnd) => Cap::Round,
            Some(LineCap::Sq) => Cap::Square,
            _ => Cap::Butt,
        };
        let join = match merged.join {
            Some(LineJoin::Round) => Join::Round,
            Some(LineJoin::Bevel) => Join::Bevel,
            Some(LineJoin::Miter(lim)) => {
                Join::Miter(lim.map_or(8.0, |l| f64::from(l.0) / 100_000.0))
            }
            // The schema has no default join; PowerPoint's own lines are round.
            None => Join::Round,
        };
        Some(Stroke {
            paint,
            width,
            dash,
            cap,
            join,
        })
    }

    /// The shape's outlines in its own space, each with its fill mode and whether it
    /// is stroked.
    fn paths(
        &mut self,
        geometry: Option<&Geometry>,
        w: f64,
        h: f64,
        kind: NodeKind,
    ) -> Vec<(Vec<PathCmd>, FillMode, bool)> {
        let extents = Extents::new(w.round() as i64, h.round() as i64);
        let compiled;
        let (def, adjusts): (&GeometryDef, Vec<(String, String)>) = match geometry {
            Some(Geometry::Preset(p)) => {
                let name = p.preset.map(|s| s.to_lexical());
                let Some(def) = name.as_deref().and_then(preset_shape) else {
                    self.report
                        .note("preset geometry (unknown, drawn as a rectangle)");
                    return vec![(rect_path(w, h), FillMode::Norm, true)];
                };
                let adjusts = p
                    .adjust
                    .iter()
                    .flatten()
                    .filter_map(|g| Some((g.name.clone()?, g.formula.clone()?)))
                    .collect();
                (def, adjusts)
            }
            Some(Geometry::Custom(c)) => match casual_slide_import::geometry::compile_custom(c) {
                Ok(c) => {
                    compiled = c;
                    (&compiled.def, Vec::new())
                }
                Err(_) => {
                    self.report.note("custom geometry (over a cap)");
                    return Vec::new();
                }
            },
            None => {
                // A shape with no geometry anywhere in its chain: a rectangle, as a
                // picture or placeholder box is drawn. A connector without one has
                // nothing to follow.
                if kind == NodeKind::Connector {
                    return Vec::new();
                }
                return vec![(rect_path(w, h), FillMode::Norm, true)];
            }
        };
        let adjusts: Vec<Adjust<'_>> = adjusts.iter().map(|(n, f)| Adjust::formula(n, f)).collect();
        let resolved = def.evaluate(extents, &adjusts);
        let mut out = Vec::new();
        let mut budget = MAX_SEGMENTS_PER_SHAPE;
        for path in &resolved.paths {
            let path = path.scaled_to(extents).with_arcs_as_cubics();
            if path.segments.len() > budget {
                self.report.note("geometry (over the segment budget)");
                break;
            }
            budget -= path.segments.len();
            let cmds = path
                .segments
                .iter()
                .filter_map(|s| match *s {
                    Segment::MoveTo(p) => Some(PathCmd::MoveTo(p.x, p.y)),
                    Segment::LineTo(p) => Some(PathCmd::LineTo(p.x, p.y)),
                    Segment::QuadTo { ctrl, end } => {
                        Some(PathCmd::QuadTo(ctrl.x, ctrl.y, end.x, end.y))
                    }
                    Segment::CubicTo { c1, c2, end } => {
                        Some(PathCmd::CubicTo(c1.x, c1.y, c2.x, c2.y, end.x, end.y))
                    }
                    Segment::Close => Some(PathCmd::Close),
                    // `with_arcs_as_cubics` leaves none.
                    Segment::ArcTo(_) => None,
                })
                .collect();
            out.push((cmds, path.fill, path.stroke));
        }
        out
    }
}

/// Whether an effect list draws anything.
fn has_effect(e: &EffectProperties) -> bool {
    match e {
        EffectProperties::List(list) => !list.is_empty(),
        EffectProperties::Dag(_) => true,
    }
}

/// A group's child coordinate space onto its parent's.
fn group_mapping(t: &Transform) -> Affine {
    let off = t
        .offset
        .map_or((0.0, 0.0), |p| (p.x.0 as f64, p.y.0 as f64));
    let ext = t.extent.map_or((0.0, 0.0), |s| (s.cx as f64, s.cy as f64));
    let ch_off = t.child_offset.map_or(off, |p| (p.x.0 as f64, p.y.0 as f64));
    let ch_ext = t.child_extent.map_or(ext, |s| (s.cx as f64, s.cy as f64));
    let sx = if ch_ext.0 > 0.0 {
        ext.0 / ch_ext.0
    } else {
        1.0
    };
    let sy = if ch_ext.1 > 0.0 {
        ext.1 / ch_ext.1
    } else {
        1.0
    };
    let place = placement(
        off.0,
        off.1,
        ext.0,
        ext.1,
        t.rotation.unwrap_or(0),
        t.flip_h.unwrap_or(false),
        t.flip_v.unwrap_or(false),
    );
    place.then_after(&Affine::scale(sx, sy).then_after(&Affine::translate(-ch_off.0, -ch_off.1)))
}

/// A linear gradient across a `w` by `h` box at `angle` (60000ths of a degree,
/// clockwise). `scaled` applies the angle in the unit square, then stretches.
fn linear(angle: i32, scaled: bool, w: f64, h: f64, stops: Vec<Stop>) -> Paint {
    let (s, c) = (f64::from(angle) / 60_000.0).to_radians().sin_cos();
    let (mut dx, mut dy) = (c, s);
    if scaled && w > 0.0 && h > 0.0 {
        dx /= w;
        dy /= h;
    }
    let len = (dx * dx + dy * dy).sqrt();
    if len > 0.0 {
        dx /= len;
        dy /= len;
    } else {
        dx = 1.0;
        dy = 0.0;
    }
    let half = ((w * dx).abs() + (h * dy).abs()) / 2.0;
    let (cx, cy) = (w / 2.0, h / 2.0);
    Paint::Linear {
        start: (cx - dx * half, cy - dy * half),
        end: (cx + dx * half, cy + dy * half),
        stops,
    }
}

/// `ST_PathFillMode` on a solid paint. Lighten and darken amounts are Inferred.
fn shade(p: Paint, mode: FillMode) -> Paint {
    let Paint::Solid(c) = p else {
        return p;
    };
    let towards = |c: Rgba, target: f64, amount: f64| {
        let f = |v: u8| {
            (f64::from(v) + (target - f64::from(v)) * amount)
                .round()
                .clamp(0.0, 255.0) as u8
        };
        Rgba {
            r: f(c.r),
            g: f(c.g),
            b: f(c.b),
            a: c.a,
        }
    };
    Paint::Solid(match mode {
        FillMode::Lighten => towards(c, 255.0, 0.4),
        FillMode::LightenLess => towards(c, 255.0, 0.2),
        FillMode::Darken => towards(c, 0.0, 0.4),
        FillMode::DarkenLess => towards(c, 0.0, 0.2),
        FillMode::Norm | FillMode::None => c,
    })
}

/// Dash and gap lengths in line widths, as PowerPoint draws each preset.
fn preset_dash(p: PresetLineDash) -> &'static [f64] {
    match p {
        PresetLineDash::Solid => &[],
        PresetLineDash::Dot => &[1.0, 1.0],
        PresetLineDash::Dash => &[4.0, 3.0],
        PresetLineDash::LgDash => &[8.0, 3.0],
        PresetLineDash::DashDot => &[4.0, 3.0, 1.0, 3.0],
        PresetLineDash::LgDashDot => &[8.0, 3.0, 1.0, 3.0],
        PresetLineDash::LgDashDotDot => &[8.0, 3.0, 1.0, 3.0, 1.0, 3.0],
        PresetLineDash::SysDash => &[3.0, 1.0],
        PresetLineDash::SysDot => &[1.0, 1.0],
        PresetLineDash::SysDashDot => &[3.0, 1.0, 1.0, 1.0],
        PresetLineDash::SysDashDotDot => &[3.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    }
}

/// Every field `top` states replaces the one in `base`.
fn overlay(base: &mut Line, top: &Line) {
    macro_rules! take {
        ($($f:ident),*) => { $( if top.$f.is_some() { base.$f = top.$f.clone(); } )* };
    }
    take!(
        width, cap, compound, align, fill, dash, join, head_end, tail_end, sketch
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_turns_about_the_box_centre() {
        let t = placement(100.0, 200.0, 40.0, 20.0, 90 * 60_000, false, false);
        let (cx, cy) = t.apply(20.0, 10.0);
        assert!((cx - 120.0).abs() < 1e-9 && (cy - 210.0).abs() < 1e-9);
        // The top-left corner swings to the top-right of the turned box.
        let (x, y) = t.apply(0.0, 0.0);
        assert!(
            (x - 130.0).abs() < 1e-9 && (y - 190.0).abs() < 1e-9,
            "{x},{y}"
        );
    }

    #[test]
    fn a_horizontal_flip_mirrors_within_the_box() {
        let t = placement(0.0, 0.0, 100.0, 50.0, 0, true, false);
        assert_eq!(t.apply(0.0, 0.0), (100.0, 0.0));
        assert_eq!(t.apply(100.0, 50.0), (0.0, 50.0));
    }

    #[test]
    fn a_group_maps_its_child_space_onto_its_box() {
        use casual_slide_model::presentation::{Coordinate, Point, PositiveSize};
        let t = Transform {
            offset: Some(Point {
                x: Coordinate(1000),
                y: Coordinate(2000),
            }),
            extent: Some(PositiveSize { cx: 200, cy: 100 }),
            child_offset: Some(Point {
                x: Coordinate(0),
                y: Coordinate(0),
            }),
            child_extent: Some(PositiveSize { cx: 100, cy: 100 }),
            ..Transform::default()
        };
        let m = group_mapping(&t);
        assert_eq!(m.apply(100.0, 100.0), (1200.0, 2100.0));
    }

    #[test]
    fn a_zero_degree_gradient_runs_left_to_right_across_the_box() {
        let stop = |o| Stop {
            offset: o,
            color: Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        };
        let Paint::Linear { start, end, .. } =
            linear(0, false, 100.0, 50.0, vec![stop(0.0), stop(1.0)])
        else {
            panic!("not linear");
        };
        assert_eq!((start, end), ((0.0, 25.0), (100.0, 25.0)));
        let Paint::Linear { start, end, .. } =
            linear(90 * 60_000, false, 100.0, 50.0, vec![stop(0.0), stop(1.0)])
        else {
            panic!("not linear");
        };
        assert!((start.1 - 0.0).abs() < 1e-9 && (end.1 - 50.0).abs() < 1e-9);
    }

    #[test]
    fn an_own_line_field_overrides_the_theme_line_and_keeps_the_rest() {
        let mut base = Line {
            width: Some(12_700),
            cap: Some(LineCap::Rnd),
            ..Line::default()
        };
        overlay(
            &mut base,
            &Line {
                width: Some(25_400),
                ..Line::default()
            },
        );
        assert_eq!(base.width, Some(25_400));
        assert_eq!(base.cap, Some(LineCap::Rnd));
    }
}
