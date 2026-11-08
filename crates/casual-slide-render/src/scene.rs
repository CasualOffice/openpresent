//! A slide's resolved shapes to a [`DisplayList`]: background, the master's and the
//! layout's visible shapes, then the slide's own, back to front.
//!
//! Effective values come through `casual-slide-style`'s chain (placement, colour map,
//! background, which inherited shapes show) and, for the families `-style` does not
//! resolve yet, by the same rule here: the shape's own value, then its layout and
//! master placeholders', then its style matrix reference into the theme. Geometry is
//! `casual-slide-geometry`'s, evaluated at the shape's size.
//!
//! Text is laid out by `casual-slide-text` in the geometry's text rectangle and
//! carried as [`DisplayItem::GlyphRun`]. What this cannot draw yet is counted in
//! [`SceneReport`], never dropped silently.

use std::collections::BTreeMap;

use casual_slide_geometry::{
    Adjust, Extents, FillMode, GeometryDef, Segment, TextRect, preset_shape,
};
use casual_slide_model::color::{Color, ResolveContext, resolve};
use casual_slide_model::effect::EffectProperties;
use casual_slide_model::fill::{
    BlipFill, BlipFillMode, Fill, GradientFill, GradientShade, PathShadeType, PatternFill,
};
use casual_slide_model::geometry::Geometry;
use casual_slide_model::line::{
    Line, LineCap, LineDash, LineEnd, LineEndType, LineJoin, PresetLineDash,
};
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode, Transform};
use casual_slide_model::slide::{BackgroundFill, SlideParts};
use casual_slide_model::table::TableStyleList;
use casual_slide_model::text::{Autofit, TextBody, TextListStyle, TextVertical};
use casual_slide_model::theme::{ColorScheme, FormatScheme, ShapeStyle, StyleMatrixRef, Theme};
use casual_slide_style::Chain;
use casual_slide_text::{CaretLine, Frame, TextEnv, layout_text_body};

use crate::display::{
    Affine, Cap, DisplayItem, DisplayList, Glyph, Join, Layer, Paint, PathCmd, Rgba, Stop, Stroke,
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
    /// Text bodies laid out and drawn as glyph runs.
    pub text_bodies_drawn: u32,
    /// Text bodies holding items that laid out to no glyphs.
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
    /// The presentation's `defaultTextStyle`, the last level of every text chain.
    pub default_text_style: Option<&'a TextListStyle>,
    /// The deck's `tableStyles` part, which a table's `tableStyleId` names into.
    pub table_styles: Option<&'a TableStyleList>,
    /// The deck's chart parts, which a chart frame's relationship names.
    pub charts: Option<&'a casual_slide_import::chart::DeckCharts>,
    /// Each SmartArt frame's stored drawing, as nodes in the frame's space.
    pub diagrams: Option<&'a BTreeMap<ObjectId, Vec<ShapeNode>>>,
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

/// EMU in a point.
const EMU_PER_POINT: f64 = 12_700.0;

mod chart;
mod effects;
mod table;

struct Builder<'a> {
    scene: SlideScene<'a>,
    chain: Chain<'a>,
    items: Vec<DisplayItem>,
    report: SceneReport,
    capture: Option<ObjectId>,
    text: SceneText,
}

/// A text body on the slide's own layer, where an editor can click into it.
#[derive(Debug, Clone, PartialEq)]
pub struct TextBox {
    /// The shape.
    pub object: ObjectId,
    /// The shape's space onto the slide's, EMU.
    pub transform: Affine,
    /// The text rectangle in the shape's space, EMU: left, top, right, bottom.
    pub rect: (f64, f64, f64, f64),
}

/// What an editor needs of a slide's text (`VS-05`): its text boxes, top last, and the
/// caret lines of the one asked for.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SceneText {
    /// Every text body on the slide's own layer, in painting order.
    pub boxes: Vec<TextBox>,
    /// The asked-for shape's transform and caret lines, in points in its space.
    pub captured: Option<(Affine, Vec<CaretLine>)>,
}

/// Build the display list of the part at `ordinal`, which is usually a slide but may
/// be a layout or master, drawn as a slide of its own.
pub fn build_slide(scene: SlideScene<'_>, ordinal: u32) -> (DisplayList, SceneReport) {
    let (list, report, _) = build_slide_text(scene, ordinal, None);
    (list, report)
}

/// As [`build_slide`], and the slide's text boxes, with the caret lines of `capture`
/// laid out even when its body is empty.
pub fn build_slide_text(
    scene: SlideScene<'_>,
    ordinal: u32,
    capture: Option<ObjectId>,
) -> (DisplayList, SceneReport, SceneText) {
    let chain = Chain::of(scene.parts, ordinal);
    let mut b = Builder {
        scene,
        chain,
        items: Vec::new(),
        report: SceneReport::default(),
        capture,
        text: SceneText::default(),
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
        b.text,
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

    /// The colour context text resolves in; `placeholder` stands for `phClr`.
    fn text_colors<'c>(&self, placeholder: Option<&'c Color>) -> ResolveContext<'c>
    where
        'a: 'c,
    {
        ResolveContext {
            scheme: self.scheme(),
            map: self.chain.color_map().map(|m| m.value),
            placeholder,
            system_palette: None,
        }
    }

    /// The theme's font scheme.
    fn font_scheme(&self) -> Option<&'a casual_slide_model::theme::FontScheme> {
        self.scene
            .theme
            .and_then(|t| t.elements.as_ref())
            .and_then(|e| e.font_scheme.as_ref())
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
                // A group whose own fill is `grpFill`, or none, passes its parent's on.
                let fill = match node.properties.fill.as_ref() {
                    Some(Fill::Group) | None => group_fill,
                    own => own,
                };
                for child in &node.children {
                    self.node(child, layer, &mapping, fill, depth + 1);
                }
            }
            NodeKind::Shape | NodeKind::Connector | NodeKind::Picture => {
                self.shape(node, layer, parent, group_fill);
            }
            NodeKind::GraphicFrame => {
                if let Some(table) = node.table.as_deref() {
                    self.table(node, table, layer, parent);
                } else if node.chart.is_some() {
                    self.chart(node, layer, parent);
                } else if node.diagram.is_some() {
                    self.diagram(node, layer, parent, depth);
                } else if node.retained.is_some() {
                    self.retained(node, layer, parent);
                } else {
                    self.report.note("graphic frame");
                }
            }
            NodeKind::ContentPart => self.report.note("content part"),
        }
    }

    /// An OLE object, control or 3D model: never opened (`SEC-C07`), drawn from the
    /// picture stored in its place.
    fn retained(&mut self, node: &'a ShapeNode, layer: Layer, parent: &Affine) {
        let fallback = node
            .retained
            .as_deref()
            .and_then(|r| r.fallback_rel_id.as_deref());
        let Some(rel) = fallback else {
            self.report.note("embedded object (no fallback picture)");
            return;
        };
        let Some(part) = self.scene.media.media_part(node.id.part, rel) else {
            self.report.note("picture (relationship unresolved)");
            return;
        };
        let Some(t) = self.chain.transform(node).map(|t| t.value) else {
            self.report.note("embedded object (no placement)");
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
        let paint = Paint::Picture {
            part,
            frame: (0.0, 0.0, w, h),
            alpha: 1.0,
        };
        self.push(node, layer, transform, rect_path(w, h), Some(paint), None);
    }

    /// A SmartArt frame, from the drawing PowerPoint stored for it.
    fn diagram(&mut self, node: &'a ShapeNode, layer: Layer, parent: &Affine, depth: usize) {
        let Some(nodes) = self.scene.diagrams.and_then(|d| d.get(&node.id)) else {
            self.report.note("diagram (no stored drawing)");
            return;
        };
        let Some(t) = self.chain.transform(node).map(|t| t.value) else {
            self.report.note("diagram (no placement)");
            return;
        };
        let mapping = parent.then_after(&placement(
            t.offset.x.0 as f64,
            t.offset.y.0 as f64,
            t.extent.cx as f64,
            t.extent.cy as f64,
            t.rotation,
            t.flip_h,
            t.flip_v,
        ));
        for child in nodes {
            self.node(child, layer, &mapping, None, depth + 1);
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
        let (stroke, ends) = self
            .stroke(
                &levels,
                style.and_then(|s| s.line.as_ref()),
                w,
                h,
                group_fill,
            )
            .unzip();
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
        let effects = own_effects.or(style_effects).filter(|e| has_effect(e));
        // An effect reference's colour stands for `phClr` in a theme effect.
        let effect_ph = if own_effects.is_some() {
            None
        } else {
            style
                .and_then(|s| s.effect.as_ref())
                .and_then(|e| e.color.as_ref())
        };
        if node.properties.scene3d.is_some() || node.properties.sp3d.is_some() {
            self.report.note("3D");
        }
        let geometry = levels.iter().find_map(|n| n.properties.geometry.as_ref());
        let (paths, text_rect) = self.paths(geometry, w, h, node.kind);
        if let Some(effects) = effects {
            // Shadows and glows sit behind what the shape draws.
            let drawn: Vec<&[PathCmd]> = paths
                .iter()
                .filter(|(_, mode, stroked)| {
                    picture.is_some()
                        || (fill.is_some() && *mode != FillMode::None)
                        || (stroke.is_some() && *stroked)
                })
                .map(|(p, _, _)| p.as_slice())
                .collect();
            self.effects(node, layer, transform, effects, effect_ph, &drawn);
        }
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
                let heads = match (&stroke, ends) {
                    (Some(s), Some(ends)) => arrowheads(&path, s, ends),
                    _ => Vec::new(),
                };
                self.push(node, layer, transform, path, fill, stroke);
                for (head, fill, stroke) in heads {
                    self.push(node, layer, transform, head, fill, stroke);
                }
            }
        }
        let rect = text_rect.unwrap_or(TextRect {
            l: 0.0,
            t: 0.0,
            r: w,
            b: h,
        });
        self.text(node, layer, transform, rect, style);
    }

    /// A shape's text body, laid out by `casual-slide-text` in the geometry's text
    /// rectangle, as glyph runs in the shape's space.
    fn text(
        &mut self,
        node: &'a ShapeNode,
        layer: Layer,
        transform: Affine,
        rect: TextRect,
        style: Option<&'a ShapeStyle>,
    ) {
        let Some(body) = node.text_body.as_deref() else {
            return;
        };
        if layer == Layer::Slide {
            self.text.boxes.push(TextBox {
                object: node.id,
                transform,
                rect: (rect.l, rect.t, rect.r, rect.b),
            });
        }
        self.text_in(node, body, layer, transform, rect, style);
    }

    /// Lay out `body` in `rect` of `node`'s space, with `node`'s text chain, and push
    /// its glyph runs.
    fn text_in(
        &mut self,
        node: &'a ShapeNode,
        body: &TextBody,
        layer: Layer,
        transform: Affine,
        rect: TextRect,
        style: Option<&'a ShapeStyle>,
    ) {
        let captured = self.capture == Some(node.id);
        if !captured && !body.paragraphs.iter().any(|p| !p.items.is_empty()) {
            return;
        }
        let object_defaults = self.scene.theme.and_then(|t| t.object_defaults.as_ref());
        let chain = self
            .chain
            .text_chain(node, self.scene.default_text_style, object_defaults);
        let env = TextEnv {
            colors: self.text_colors(
                style
                    .and_then(|s| s.font.as_ref())
                    .and_then(|f| f.color.as_ref()),
            ),
            fonts: self.font_scheme(),
        };
        let frame = Frame {
            x: (rect.l / EMU_PER_POINT) as f32,
            y: (rect.t / EMU_PER_POINT) as f32,
            width: ((rect.r - rect.l).max(0.0) / EMU_PER_POINT) as f32,
            height: ((rect.b - rect.t).max(0.0) / EMU_PER_POINT) as f32,
        };
        let autofit = chain
            .body(body.properties.as_ref(), |b| b.autofit.as_ref())
            .map(|r| r.value);
        let layout = match autofit {
            Some(Autofit::Normal {
                font_scale,
                line_spacing_reduction,
            }) if font_scale.is_some() || line_spacing_reduction.is_some() => {
                let scale = font_scale
                    .as_ref()
                    .map_or(1.0, |p| f64::from(p.value().0) / 100_000.0);
                let reduce = line_spacing_reduction
                    .as_ref()
                    .map_or(0.0, |p| f64::from(p.value().0) / 100_000.0);
                autofit_layout(body, &chain, &env, frame, scale, reduce)
            }
            _ => layout_text_body(body, &chain, &env, frame),
        };
        if captured {
            self.text.captured = Some((transform, layout.carets.clone()));
        }
        if layout.runs.is_empty() {
            self.report.text_bodies_undrawn = self.report.text_bodies_undrawn.saturating_add(1);
            return;
        }
        self.report.text_bodies_drawn = self.report.text_bodies_drawn.saturating_add(1);
        for _ in &layout.substitutions {
            self.report.note("font substituted");
        }
        for _ in &layout.missing_scripts {
            self.report.note("script with no face");
        }
        if body.properties.as_ref().is_some_and(|b| {
            b.rotation.is_some_and(|r| r != 0)
                || b.vertical.is_some_and(|v| v != TextVertical::Horz)
        }) {
            self.report
                .note("rotated or vertical text (drawn horizontal)");
        }
        for run in layout.runs {
            if run.synthetic_bold {
                self.report.note("synthetic bold (drawn regular)");
            }
            let pt = |v: f32| f64::from(v) * EMU_PER_POINT;
            self.items.push(DisplayItem::GlyphRun {
                object: Some(node.id),
                layer,
                transform,
                face: run.face_key,
                size: pt(run.size),
                glyphs: run
                    .glyphs
                    .iter()
                    .map(|g| Glyph {
                        id: u32::from(g.id),
                        x: pt(g.x),
                        y: pt(g.y),
                    })
                    .collect(),
                fill: Paint::Solid(Rgba {
                    r: run.color.r,
                    g: run.color.g,
                    b: run.color.b,
                    a: run.color.a,
                }),
                text: run.text,
            });
            if let Some(line) = run.underline {
                let (strokes, note) = underline_strokes(&line);
                if let Some(note) = note {
                    self.report.note(note);
                }
                let paint = Paint::Solid(Rgba {
                    r: run.color.r,
                    g: run.color.g,
                    b: run.color.b,
                    a: run.color.a,
                });
                for s in strokes {
                    self.items.push(DisplayItem::Path {
                        object: Some(node.id),
                        layer,
                        transform,
                        path: vec![
                            PathCmd::MoveTo(pt(line.x0), pt(s.y)),
                            PathCmd::LineTo(pt(line.x1), pt(s.y)),
                        ],
                        fill: None,
                        stroke: Some(Stroke {
                            paint: paint.clone(),
                            width: pt(s.width),
                            dash: s.dash.iter().map(|d| pt(d * s.width)).collect(),
                            cap: Cap::Butt,
                            join: Join::Bevel,
                        }),
                    });
                }
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
    ) -> Option<(Stroke, Ends)> {
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
        self.line_stroke(&merged, ph, w, h, ordinal, group_fill)
    }

    /// A merged line as a stroke over a `w` by `h` box, and its ends. `ph` stands
    /// for `phClr`.
    fn line_stroke(
        &mut self,
        merged: &Line,
        ph: Option<&Color>,
        w: f64,
        h: f64,
        ordinal: u32,
        group_fill: Option<&Fill>,
    ) -> Option<(Stroke, Ends)> {
        let paint = self.fill_paint(merged.fill.as_ref()?, ph, w, h, ordinal, group_fill)?;
        let width = merged.width.map_or(DEFAULT_LINE_WIDTH, f64::from);
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
        Some((
            Stroke {
                paint,
                width,
                dash,
                cap,
                join,
            },
            Ends {
                head: merged.head_end,
                tail: merged.tail_end,
            },
        ))
    }

    /// The shape's outlines in its own space, each with its fill mode and whether it
    /// is stroked.
    fn paths(
        &mut self,
        geometry: Option<&Geometry>,
        w: f64,
        h: f64,
        kind: NodeKind,
    ) -> (Vec<OutlinePath>, Option<TextRect>) {
        let extents = Extents::new(w.round() as i64, h.round() as i64);
        let compiled;
        let (def, adjusts): (&GeometryDef, Vec<(String, String)>) = match geometry {
            Some(Geometry::Preset(p)) => {
                let name = p.preset.map(|s| s.to_lexical());
                let Some(def) = name.as_deref().and_then(preset_shape) else {
                    self.report
                        .note("preset geometry (unknown, drawn as a rectangle)");
                    return (vec![(rect_path(w, h), FillMode::Norm, true)], None);
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
                    return (Vec::new(), None);
                }
            },
            None => {
                // A shape with no geometry anywhere in its chain: a rectangle, as a
                // picture or placeholder box is drawn. A connector without one has
                // nothing to follow.
                if kind == NodeKind::Connector {
                    return (Vec::new(), None);
                }
                return (vec![(rect_path(w, h), FillMode::Norm, true)], None);
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
        (out, Some(resolved.text_rect))
    }
}

/// `normAutofit`'s stored result applied: every run, bullet and paragraph mark
/// scaled by `fontScale`, and percentage line spacing lowered by
/// `lnSpcReduction`, as PowerPoint draws a body it last fitted. The scale is the
/// file's, not recomputed here.
fn autofit_layout(
    body: &TextBody,
    chain: &casual_slide_style::text::TextChain<'_>,
    env: &TextEnv<'_>,
    frame: Frame,
    scale: f64,
    reduce: f64,
) -> casual_slide_text::TextLayout {
    use casual_slide_text::layout::{Spacing, layout_body};
    let (mut resolved, substitutions) = casual_slide_text::body::resolve_body(body, chain, env);
    let scale = scale.clamp(0.01, 1.0) as f32;
    let reduce = reduce.clamp(0.0, 0.9) as f32;
    let mut text = String::new();
    for p in &mut resolved.paragraphs {
        for run in p.runs.iter_mut().chain(p.bullet.iter_mut()) {
            run.style.size *= scale;
            text.push_str(&run.text);
        }
        p.end.size *= scale;
        if let Spacing::Percent(v) = &mut p.line_spacing {
            *v = (*v - reduce).max(0.1);
        }
    }
    let mut layout = layout_body(&resolved, frame);
    layout.substitutions = substitutions;
    layout.missing_scripts = casual_slide_text::faces::missing_scripts(&text);
    layout
}

/// A line's two ends: `headEnd` at the path's start, `tailEnd` at its end.
#[derive(Debug, Clone, Copy)]
struct Ends {
    head: Option<LineEnd>,
    tail: Option<LineEnd>,
}

/// One outline of a shape's geometry: its commands, its fill mode, and whether it
/// is stroked.
type OutlinePath = (Vec<PathCmd>, FillMode, bool);

/// One drawn piece: a path, its fill, its stroke.
type Piece = (Vec<PathCmd>, Option<Paint>, Option<Stroke>);

/// The points a path passes through and is pulled towards, in order.
fn path_points(path: &[PathCmd]) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for cmd in path {
        match *cmd {
            PathCmd::MoveTo(x, y) | PathCmd::LineTo(x, y) => out.push((x, y)),
            PathCmd::QuadTo(a, b, x, y) => out.extend([(a, b), (x, y)]),
            PathCmd::CubicTo(a, b, c, d, x, y) => out.extend([(a, b), (c, d), (x, y)]),
            PathCmd::Close => {}
        }
    }
    out
}

/// The tip of one end and the unit direction pointing out of the line there.
fn end_of(points: &[(f64, f64)]) -> Option<((f64, f64), (f64, f64))> {
    let (&tip, rest) = points.split_first()?;
    let from = rest.iter().find(|p| **p != tip)?;
    let (dx, dy) = (tip.0 - from.0, tip.1 - from.1);
    let len = (dx * dx + dy * dy).sqrt();
    (len > 0.0 && len.is_finite()).then_some((tip, (dx / len, dy / len)))
}

/// Size of a line end in line widths, as PowerPoint draws `sm`, `med` and `lg`.
fn end_scale(token: Option<String>) -> f64 {
    match token.as_deref() {
        Some("sm") => 2.0,
        Some("lg") => 5.0,
        _ => 3.0,
    }
}

fn polyline(points: &[(f64, f64)], close: bool) -> Vec<PathCmd> {
    let mut cmds: Vec<PathCmd> = points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i == 0 {
                PathCmd::MoveTo(p.0, p.1)
            } else {
                PathCmd::LineTo(p.0, p.1)
            }
        })
        .collect();
    if close {
        cmds.push(PathCmd::Close);
    }
    cmds
}

/// Arrowheads for an open stroked path, in its space, filled or stroked with the
/// line's own paint.
fn arrowheads(path: &[PathCmd], stroke: &Stroke, ends: Ends) -> Vec<Piece> {
    if path.iter().any(|c| matches!(c, PathCmd::Close)) {
        return Vec::new();
    }
    let points = path_points(path);
    let reversed: Vec<(f64, f64)> = points.iter().rev().copied().collect();
    let unit = stroke.width.max(9_525.0);
    let mut out = Vec::new();
    for (end, pts) in [(ends.head, &points), (ends.tail, &reversed)] {
        let Some(kind) = end.and_then(|e| e.end_type) else {
            continue;
        };
        let Some((tip, d)) = end_of(pts) else {
            continue;
        };
        let len = unit * end_scale(end.and_then(|e| e.length).map(|l| l.to_lexical()));
        let wid = unit * end_scale(end.and_then(|e| e.width).map(|w| w.to_lexical()));
        let n = (-d.1, d.0);
        let at = |along: f64, across: f64| {
            (
                tip.0 + d.0 * along + n.0 * across,
                tip.1 + d.1 * along + n.1 * across,
            )
        };
        let filled = |cmds: Vec<PathCmd>| (cmds, Some(stroke.paint.clone()), None);
        let piece = match kind {
            LineEndType::None => continue,
            LineEndType::Triangle => filled(polyline(
                &[tip, at(-len, wid / 2.0), at(-len, -wid / 2.0)],
                true,
            )),
            LineEndType::Stealth => filled(polyline(
                &[
                    tip,
                    at(-len, wid / 2.0),
                    at(-len * 0.6, 0.0),
                    at(-len, -wid / 2.0),
                ],
                true,
            )),
            LineEndType::Diamond => filled(polyline(
                &[
                    at(len / 2.0, 0.0),
                    at(0.0, wid / 2.0),
                    at(-len / 2.0, 0.0),
                    at(0.0, -wid / 2.0),
                ],
                true,
            )),
            LineEndType::Oval => {
                // An ellipse as four cubics, in the end's own frame.
                const K: f64 = 0.552_284_749_830_793_6;
                let (a, b) = (len / 2.0, wid / 2.0);
                let start = at(a, 0.0);
                let mut cmds = vec![PathCmd::MoveTo(start.0, start.1)];
                for [c1, c2, e] in [
                    [at(a, b * K), at(a * K, b), at(0.0, b)],
                    [at(-a * K, b), at(-a, b * K), at(-a, 0.0)],
                    [at(-a, -b * K), at(-a * K, -b), at(0.0, -b)],
                    [at(a * K, -b), at(a, -b * K), at(a, 0.0)],
                ] {
                    cmds.push(PathCmd::CubicTo(c1.0, c1.1, c2.0, c2.1, e.0, e.1));
                }
                cmds.push(PathCmd::Close);
                filled(cmds)
            }
            LineEndType::Arrow => (
                polyline(&[at(-len, wid / 2.0), tip, at(-len, -wid / 2.0)], false),
                None,
                Some(Stroke {
                    dash: Vec::new(),
                    ..stroke.clone()
                }),
            ),
        };
        out.push(piece);
    }
    out
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

/// One line of an underline, points: its centre, width, and dash and gap lengths in
/// widths (empty is solid).
#[derive(Debug, Clone, PartialEq)]
struct UnderlineStroke {
    y: f32,
    width: f32,
    dash: &'static [f32],
}

/// The lines that draw `u`, and what is drawn differently from PowerPoint, if
/// anything. Single, double, heavy, dotted and dashed kinds are drawn as their name
/// says; `words` underlines the spaces too, and a wavy kind is drawn straight.
fn underline_strokes(
    u: &casual_slide_text::Underline,
) -> (Vec<UnderlineStroke>, Option<&'static str>) {
    use casual_slide_text::UnderlineType as U;
    let t = u.thickness;
    let line = |width: f32, dash: &'static [f32]| UnderlineStroke {
        y: u.y,
        width,
        dash,
    };
    const DOT: &[f32] = &[1.0, 1.0];
    const DASH: &[f32] = &[4.0, 3.0];
    const LONG: &[f32] = &[8.0, 3.0];
    const DOT_DASH: &[f32] = &[4.0, 3.0, 1.0, 3.0];
    const DOT_DOT_DASH: &[f32] = &[4.0, 3.0, 1.0, 3.0, 1.0, 3.0];
    let double = || {
        vec![
            UnderlineStroke {
                y: u.y,
                width: t,
                dash: &[],
            },
            UnderlineStroke {
                y: u.y + 2.0 * t,
                width: t,
                dash: &[],
            },
        ]
    };
    match u.kind {
        U::None => (Vec::new(), None),
        U::Sng => (vec![line(t, &[])], None),
        U::Words => (
            vec![line(t, &[])],
            Some("underline words (spaces underlined)"),
        ),
        U::Dbl => (double(), None),
        U::Heavy => (vec![line(2.0 * t, &[])], None),
        U::Dotted => (vec![line(t, DOT)], None),
        U::DottedHeavy => (vec![line(2.0 * t, DOT)], None),
        U::Dash => (vec![line(t, DASH)], None),
        U::DashHeavy => (vec![line(2.0 * t, DASH)], None),
        U::DashLong => (vec![line(t, LONG)], None),
        U::DashLongHeavy => (vec![line(2.0 * t, LONG)], None),
        U::DotDash => (vec![line(t, DOT_DASH)], None),
        U::DotDashHeavy => (vec![line(2.0 * t, DOT_DASH)], None),
        U::DotDotDash => (vec![line(t, DOT_DOT_DASH)], None),
        U::DotDotDashHeavy => (vec![line(2.0 * t, DOT_DOT_DASH)], None),
        U::Wavy => (vec![line(t, &[])], Some("wavy underline (drawn straight)")),
        U::WavyHeavy => (
            vec![line(2.0 * t, &[])],
            Some("wavy underline (drawn straight)"),
        ),
        U::WavyDbl => (double(), Some("wavy underline (drawn straight)")),
    }
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

    fn solid_stroke(width: f64) -> Stroke {
        Stroke {
            paint: Paint::Solid(Rgba {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }),
            width,
            dash: vec![10.0, 10.0],
            cap: Cap::Butt,
            join: Join::Round,
        }
    }

    fn end(kind: LineEndType) -> Option<LineEnd> {
        Some(LineEnd {
            end_type: Some(kind),
            width: None,
            length: None,
        })
    }

    #[test]
    fn a_tail_triangle_points_out_of_the_end_of_an_open_line() {
        let path = vec![PathCmd::MoveTo(0.0, 0.0), PathCmd::LineTo(100_000.0, 0.0)];
        let ends = Ends {
            head: None,
            tail: end(LineEndType::Triangle),
        };
        let heads = arrowheads(&path, &solid_stroke(10_000.0), ends);
        assert_eq!(heads.len(), 1);
        let (cmds, fill, stroke) = &heads[0];
        assert!(fill.is_some() && stroke.is_none());
        // Tip at the end; the base three widths back, three widths across.
        assert_eq!(cmds[0], PathCmd::MoveTo(100_000.0, 0.0));
        assert_eq!(cmds[1], PathCmd::LineTo(70_000.0, 15_000.0));
        assert_eq!(cmds[2], PathCmd::LineTo(70_000.0, -15_000.0));
    }

    #[test]
    fn a_closed_path_and_an_end_of_none_draw_no_heads() {
        let open = vec![PathCmd::MoveTo(0.0, 0.0), PathCmd::LineTo(1.0, 0.0)];
        let none = Ends {
            head: end(LineEndType::None),
            tail: None,
        };
        assert!(arrowheads(&open, &solid_stroke(1.0), none).is_empty());
        let mut closed = open.clone();
        closed.push(PathCmd::Close);
        let arrow = Ends {
            head: end(LineEndType::Arrow),
            tail: None,
        };
        assert!(arrowheads(&closed, &solid_stroke(1.0), arrow).is_empty());
        // An open arrow is stroked, undashed.
        let heads = arrowheads(&open, &solid_stroke(1.0), arrow);
        assert!(heads[0].2.as_ref().is_some_and(|s| s.dash.is_empty()));
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

#[cfg(test)]
mod underline_tests {
    use super::underline_strokes;
    use casual_slide_text::{Underline, UnderlineType};

    fn under(kind: UnderlineType) -> Underline {
        Underline {
            kind,
            x0: 10.0,
            x1: 50.0,
            y: 30.0,
            thickness: 2.0,
        }
    }

    #[test]
    fn single_double_heavy_and_dash_underlines_draw_as_powerpoint_names_them() {
        let (sng, note) = underline_strokes(&under(UnderlineType::Sng));
        assert_eq!(sng.len(), 1);
        assert_eq!((sng[0].y, sng[0].width, sng[0].dash.len()), (30.0, 2.0, 0));
        assert!(note.is_none());
        let (dbl, _) = underline_strokes(&under(UnderlineType::Dbl));
        assert_eq!(dbl.len(), 2, "double is two lines");
        assert!(dbl[1].y - dbl[0].y >= 2.0, "with a gap: {dbl:?}");
        let (heavy, _) = underline_strokes(&under(UnderlineType::Heavy));
        assert_eq!(heavy[0].width, 4.0, "heavy is twice as thick");
        let (dash, _) = underline_strokes(&under(UnderlineType::Dash));
        assert!(!dash[0].dash.is_empty(), "dash is dashed");
        let (_, wavy) = underline_strokes(&under(UnderlineType::Wavy));
        assert!(wavy.is_some(), "a kind not drawn as named is reported");
    }
}
