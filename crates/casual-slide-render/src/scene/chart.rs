//! A classic chart's graphic frame (`c:chartSpace`) to display items, from the
//! values cached in the chart part. The embedded workbook is never opened
//! (`SEC-P09`).
//!
//! **Ported from OpenCalc** (`casual-calc-layout/src/chart.rs` at `b23c67c`, owner
//! instruction 42; PROVENANCE.md): the frame, then the title, then the legend taken
//! off the plot's side before anything is drawn, the value extent that always holds
//! zero and sums stacks positive and negative apart, bars measured from the zero
//! line, the pie from twelve o'clock clockwise, the accent palette varied past the
//! sixth series, and category labels thinned to what fits. OpenCalc's constants are
//! CSS pixels in twips; here [`PX`] is one pixel at 96 dpi in EMU.
//!
//! Changed for slides: a chart's own colours (a series' `spPr`, a point's `dPt`)
//! win over the palette, as PowerPoint draws them; bar width follows `gapWidth`;
//! horizontal bars (`barDir="bar"`) are drawn; the value axis gets PowerPoint's
//! round tick steps and major gridlines, where OpenCalc labels its two extremes;
//! line and area series are drawn; and the frame is drawn only as the chart's
//! `spPr` states, since a slide chart sits on the slide rather than on a grid.
//!
//! Not drawn, and reported: 3D (drawn flat), radar, surface and bubble families,
//! `cx:chartSpace` (2016+ charts), and, by name (`SKEL-04`), data labels, line
//! and scatter markers, trendlines, error bars, drop, high-low and series lines,
//! up-down bars, pie explosion and the data table.

use casual_slide_import::chart::chart_of;
use casual_slide_model::chart::c;
use casual_slide_model::color::{Color, ColorValue, SchemeColor};
use casual_slide_model::drawing::ShapeProperties;
use casual_slide_model::fill::Fill;
use casual_slide_model::shape::ShapeNode;
use casual_slide_model::text::{TextBody, TextItem};
use casual_slide_text::Frame;
use casual_slide_text::layout::{
    Align, BodyLayout, ParagraphLayout, Rgba as TextRgba, RunStyle, StyledText, layout_body,
};

use super::{Builder, EMU_PER_POINT, placement, rect_path};
use crate::display::{Affine, Cap, DisplayItem, Glyph, Join, Layer, Paint, PathCmd, Rgba, Stroke};

/// One CSS pixel at 96 dpi, in EMU: OpenCalc's `PX`, in this crate's unit.
const PX: f64 = 9_525.0;

/// The most points one series plots, and series one chart plots. Import bounds
/// both; this keeps the drawing bounded on its own terms.
const MAX_CHART_POINTS: usize = 4_096;

/// The most series drawn of one chart.
const MAX_CHART_SERIES: usize = 256;

/// Text PowerPoint draws a chart's labels in when the chart states no colour.
const CHART_TEXT: Rgba = Rgba {
    r: 0x59,
    g: 0x59,
    b: 0x59,
    a: 255,
};
/// Gridlines and axis lines.
const CHART_GRID: Rgba = Rgba {
    r: 0xD9,
    g: 0xD9,
    b: 0xD9,
    a: 255,
};

/// How the seventh series on varies the accent it shares: OpenCalc's table.
const ACCENT_VARIANTS: [f64; 5] = [-0.25, 0.40, -0.50, 0.60, 0.80];

const ACCENTS: [SchemeColor; 6] = [
    SchemeColor::Accent1,
    SchemeColor::Accent2,
    SchemeColor::Accent3,
    SchemeColor::Accent4,
    SchemeColor::Accent5,
    SchemeColor::Accent6,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Column,
    Bar,
    Line,
    Area,
    Pie,
    Scatter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Grouping {
    Clustered,
    Stacked,
    Percent,
}

/// One series, resolved to the numbers it plots and the paint it is drawn in.
struct Series {
    name: String,
    values: Vec<Option<f64>>,
    xs: Option<Vec<Option<f64>>>,
    cats: Vec<String>,
    fill: Option<Paint>,
    line: Option<Paint>,
    points: Vec<(usize, Paint)>,
    kind: Kind,
    grouping: Grouping,
    /// The family it came from, so a family's series stack together.
    family: usize,
    gap: f64,
    hole: f64,
    first_angle: f64,
    smooth_or_line: bool,
}

/// A plot rectangle, whittled down by the title and the legend.
#[derive(Debug, Clone, Copy)]
struct Plot {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl<'a> Builder<'a> {
    /// Draw a chart frame.
    pub(super) fn chart(&mut self, node: &'a ShapeNode, layer: Layer, parent: &Affine) {
        let Some(reference) = node.chart.as_ref() else {
            return;
        };
        if reference.extended {
            self.report.note("chart (2016+ family, not drawn)");
            return;
        }
        let Some(part) = reference
            .rel_id
            .as_deref()
            .and_then(|r| self.scene.media.media_part(node.id.part, r))
        else {
            self.report.note("chart (relationship unresolved)");
            return;
        };
        let space = self
            .scene
            .charts
            .and_then(|c| c.get(&part))
            .and_then(|c| match &c.status {
                casual_slide_import::chart::ChartStatus::Read(p) => chart_of(p),
                casual_slide_import::chart::ChartStatus::SetAside { .. } => None,
            });
        let Some(space) = space else {
            self.report.note("chart (part not read)");
            return;
        };
        let Some(t) = self.chain.transform(node).map(|t| t.value) else {
            self.report.note("chart (no placement)");
            return;
        };
        let (w, h) = (t.extent.cx as f64, t.extent.cy as f64);
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let transform = parent.then_after(&placement(
            t.offset.x.0 as f64,
            t.offset.y.0 as f64,
            w,
            h,
            t.rotation,
            t.flip_h,
            t.flip_v,
        ));
        let ordinal = node.id.part;
        // The frame: as the chart's `spPr` says, else nothing.
        self.chart_box(
            node,
            layer,
            transform,
            space.sp_pr.as_deref(),
            (0.0, 0.0, w, h),
            ordinal,
        );

        let Some(chart) = space.chart.as_deref() else {
            return;
        };
        let series = self.chart_series(space);
        let text_color = |b: Option<&TextBody>| b.and_then(body_color);
        let default_text = text_color(space.tx_pr.as_deref());
        let text = |c: Option<Color>, me: &mut Self| {
            c.or(default_text.clone())
                .and_then(|c| me.color(&c, None))
                .unwrap_or(CHART_TEXT)
        };

        let mut top = 6.0 * PX;
        let title = chart_title(chart, &series);
        if let Some(title) = title {
            let tc = chart.title.as_deref();
            let color = text(
                tc.and_then(|t| t.tx_pr.as_deref()).and_then(body_color),
                self,
            );
            let size = tc
                .and_then(|t| t.tx.as_deref())
                .and_then(|t| t.rich.as_deref())
                .and_then(body_size)
                .or_else(|| tc.and_then(|t| t.tx_pr.as_deref()).and_then(body_size))
                .unwrap_or(14.0);
            self.label(
                node,
                layer,
                transform,
                &title,
                (0.0, top, w),
                size,
                Align::Center,
                color,
                false,
            );
            top += f64::from(size) * EMU_PER_POINT * 1.4;
        }
        if series.is_empty() {
            return;
        }
        let mut plot = Plot {
            x: 10.0 * PX,
            y: top,
            w: w - 20.0 * PX,
            h: h - top - 8.0 * PX,
        };
        let pie = series.iter().all(|s| s.kind == Kind::Pie);
        // The legend comes off the frame before the axes do, as in OpenCalc.
        if let Some(legend) = chart.legend.as_deref() {
            let pos = legend
                .legend_pos
                .as_deref()
                .and_then(|p| p.val)
                .unwrap_or(c::LegendPosValue::R);
            let entries: Vec<(String, Paint)> = if pie {
                let s = &series[0];
                (0..s.values.len())
                    .map(|i| {
                        (
                            s.cats.get(i).cloned().unwrap_or_default(),
                            self.point_paint(s, i),
                        )
                    })
                    .collect()
            } else {
                series
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (s.name.clone(), self.series_paint(s, i)))
                    .collect()
            };
            let color = text(legend.tx_pr.as_deref().and_then(body_color), self);
            let size = legend.tx_pr.as_deref().and_then(body_size).unwrap_or(9.0);
            self.legend(
                node, layer, transform, &entries, pos, &mut plot, size, color,
            );
        }
        if plot.w < 20.0 * PX || plot.h < 20.0 * PX {
            return;
        }
        if pie {
            self.pie(node, layer, transform, &series[0], plot);
            return;
        }
        let axis_size = 9.0;
        let axis_color = text(None, self);
        self.xy(
            node, layer, transform, chart, &series, plot, axis_size, axis_color,
        );
    }

    /// A chart element's box: its fill and outline, where its `spPr` states them.
    fn chart_box(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        transform: Affine,
        sp: Option<&ShapeProperties>,
        (x, y, w, h): (f64, f64, f64, f64),
        ordinal: u32,
    ) {
        let Some(sp) = sp else { return };
        let fill = sp
            .fill
            .as_ref()
            .and_then(|f| self.fill_paint(f, None, w, h, ordinal, None));
        let stroke = sp
            .line
            .as_deref()
            .and_then(|l| self.line_stroke(l, None, w, h, ordinal, None))
            .map(|(s, _)| s);
        if fill.is_some() || stroke.is_some() {
            let at = transform.then_after(&Affine::translate(x, y));
            self.push(node, layer, at, rect_path(w, h), fill, stroke);
        }
    }

    /// Every series of every family the plot area holds, in drawing order.
    fn chart_series(&mut self, space: &c::ChartSpace) -> Vec<Series> {
        let mut out = Vec::new();
        for part in undrawn_parts(space) {
            self.report.note(part);
        }
        for (family, chart) in space.charts().iter().enumerate() {
            use c::PlotAreaCharts as P;
            let three_d = matches!(
                chart,
                P::Bar3DChart(_) | P::Line3DChart(_) | P::Area3DChart(_) | P::Pie3DChart(_)
            );
            if three_d {
                self.report.note("3D chart (drawn flat)");
            }
            let line_group = |g: &Option<Box<c::Grouping>>| match g.as_deref().and_then(|g| g.val) {
                Some(c::GroupingValue::Stacked) => Grouping::Stacked,
                Some(c::GroupingValue::PercentStacked) => Grouping::Percent,
                _ => Grouping::Clustered,
            };
            let bar_group = |g: &Option<Box<c::BarGrouping>>| match g.as_deref().and_then(|g| g.val)
            {
                Some(c::BarGroupingValue::Stacked) => Grouping::Stacked,
                Some(c::BarGroupingValue::PercentStacked) => Grouping::Percent,
                _ => Grouping::Clustered,
            };
            let bar_kind = |d: &Option<Box<c::BarDir>>| {
                if d.as_deref().and_then(|d| d.val) == Some(c::BarDirValue::Bar) {
                    Kind::Bar
                } else {
                    Kind::Column
                }
            };
            let gap = |g: &Option<Box<c::GapAmount>>| {
                g.as_deref()
                    .and_then(|g| g.val.as_deref())
                    .and_then(|v| v.trim().trim_end_matches('%').parse::<f64>().ok())
                    .map_or(1.5, |v| (v / 100.0).clamp(0.0, 5.0))
            };
            let mut push = |s: Series| {
                if out.len() < MAX_CHART_SERIES {
                    out.push(s);
                }
            };
            let base = |kind, grouping| Series {
                name: String::new(),
                values: Vec::new(),
                xs: None,
                cats: Vec::new(),
                fill: None,
                line: None,
                points: Vec::new(),
                kind,
                grouping,
                family,
                gap: 1.5,
                hole: 0.0,
                first_angle: 0.0,
                smooth_or_line: true,
            };
            match chart {
                P::BarChart(b) => {
                    for s in &b.ser {
                        let mut ser = base(bar_kind(&b.bar_dir), bar_group(&b.grouping));
                        ser.gap = gap(&b.gap_width);
                        self.fill_series(&mut ser, s.tx.as_deref(), s.sp_pr.as_deref(), &s.d_pt);
                        ser.values = num_values(s.val.as_deref());
                        ser.cats = cat_values(s.cat.as_deref());
                        push(ser);
                    }
                }
                P::Bar3DChart(b) => {
                    for s in &b.ser {
                        let mut ser = base(bar_kind(&b.bar_dir), bar_group(&b.grouping));
                        ser.gap = gap(&b.gap_width);
                        self.fill_series(&mut ser, s.tx.as_deref(), s.sp_pr.as_deref(), &s.d_pt);
                        ser.values = num_values(s.val.as_deref());
                        ser.cats = cat_values(s.cat.as_deref());
                        push(ser);
                    }
                }
                P::LineChart(l) => {
                    for s in &l.ser {
                        let mut ser = base(Kind::Line, line_group(&l.grouping));
                        self.fill_series(&mut ser, s.tx.as_deref(), s.sp_pr.as_deref(), &s.d_pt);
                        ser.values = num_values(s.val.as_deref());
                        ser.cats = cat_values(s.cat.as_deref());
                        push(ser);
                    }
                }
                P::Line3DChart(l) => {
                    for s in &l.ser {
                        let mut ser = base(Kind::Line, line_group(&l.grouping));
                        self.fill_series(&mut ser, s.tx.as_deref(), s.sp_pr.as_deref(), &s.d_pt);
                        ser.values = num_values(s.val.as_deref());
                        ser.cats = cat_values(s.cat.as_deref());
                        push(ser);
                    }
                }
                P::StockChart(l) => {
                    for s in &l.ser {
                        let mut ser = base(Kind::Line, Grouping::Clustered);
                        self.fill_series(&mut ser, s.tx.as_deref(), s.sp_pr.as_deref(), &s.d_pt);
                        ser.values = num_values(s.val.as_deref());
                        ser.cats = cat_values(s.cat.as_deref());
                        push(ser);
                    }
                }
                P::AreaChart(a) => {
                    for s in &a.ser {
                        let mut ser = base(Kind::Area, line_group(&a.grouping));
                        self.fill_series(&mut ser, s.tx.as_deref(), s.sp_pr.as_deref(), &s.d_pt);
                        ser.values = num_values(s.val.as_deref());
                        ser.cats = cat_values(s.cat.as_deref());
                        push(ser);
                    }
                }
                P::Area3DChart(a) => {
                    for s in &a.ser {
                        let mut ser = base(Kind::Area, line_group(&a.grouping));
                        self.fill_series(&mut ser, s.tx.as_deref(), s.sp_pr.as_deref(), &s.d_pt);
                        ser.values = num_values(s.val.as_deref());
                        ser.cats = cat_values(s.cat.as_deref());
                        push(ser);
                    }
                }
                P::PieChart(_) | P::Pie3DChart(_) | P::DoughnutChart(_) | P::OfPieChart(_) => {
                    let (sers, hole, angle) = match chart {
                        P::PieChart(p) => (&p.ser, 0.0, first_angle(&p.first_slice_ang)),
                        P::Pie3DChart(p) => (&p.ser, 0.0, 0.0),
                        P::DoughnutChart(p) => (
                            &p.ser,
                            p.hole_size
                                .as_deref()
                                .and_then(|h| h.val.as_deref())
                                .and_then(|v| v.trim().trim_end_matches('%').parse::<f64>().ok())
                                .map_or(0.5, |v| (v / 100.0).clamp(0.1, 0.9)),
                            first_angle(&p.first_slice_ang),
                        ),
                        P::OfPieChart(p) => {
                            self.report.note("pie of pie (drawn as one pie)");
                            (&p.ser, 0.0, 0.0)
                        }
                        _ => continue,
                    };
                    for s in sers {
                        let mut ser = base(Kind::Pie, Grouping::Clustered);
                        ser.hole = hole;
                        ser.first_angle = angle;
                        self.fill_series(&mut ser, s.tx.as_deref(), s.sp_pr.as_deref(), &s.d_pt);
                        ser.values = num_values(s.val.as_deref());
                        ser.cats = cat_values(s.cat.as_deref());
                        push(ser);
                    }
                }
                P::ScatterChart(sc) => {
                    let style = sc.scatter_style.as_deref().and_then(|s| s.val);
                    for s in &sc.ser {
                        let mut ser = base(Kind::Scatter, Grouping::Clustered);
                        ser.smooth_or_line = !matches!(
                            style,
                            Some(c::ScatterStyleValue::Marker | c::ScatterStyleValue::None)
                        ) && s
                            .sp_pr
                            .as_deref()
                            .and_then(|p| p.line.as_deref())
                            .and_then(|l| l.fill.as_ref())
                            .is_none_or(|f| !matches!(f, Fill::None));
                        self.fill_series(&mut ser, s.tx.as_deref(), s.sp_pr.as_deref(), &s.d_pt);
                        ser.values = num_values(s.y_val.as_deref());
                        let xs = cat_values(s.x_val.as_deref());
                        ser.xs = Some(xs.iter().map(|v| v.trim().parse::<f64>().ok()).collect());
                        push(ser);
                    }
                }
                P::RadarChart(_)
                | P::SurfaceChart(_)
                | P::Surface3DChart(_)
                | P::BubbleChart(_) => {
                    self.report
                        .note("chart family (radar, surface or bubble, not drawn)");
                }
            }
        }
        out
    }

    /// A series' name and its own paints.
    fn fill_series(
        &mut self,
        ser: &mut Series,
        tx: Option<&c::SerTx>,
        sp: Option<&ShapeProperties>,
        points: &[c::DPt],
    ) {
        ser.name = tx
            .and_then(|t| {
                t.v.clone().or_else(|| {
                    t.str_ref
                        .as_deref()?
                        .str_cache
                        .as_deref()?
                        .pt
                        .first()?
                        .v
                        .clone()
                })
            })
            .unwrap_or_default();
        if let Some(sp) = sp {
            ser.fill = sp
                .fill
                .as_ref()
                .and_then(|f| self.fill_paint(f, None, 1.0, 1.0, 0, None));
            if sp
                .line
                .as_deref()
                .is_some_and(|l| matches!(l.fill, Some(Fill::None)))
            {
                ser.smooth_or_line = false;
            }
            ser.line = sp
                .line
                .as_deref()
                .and_then(|l| l.fill.as_ref())
                .and_then(|f| self.fill_paint(f, None, 1.0, 1.0, 0, None));
        }
        for p in points.iter().take(MAX_CHART_POINTS) {
            let Some(idx) = p.idx.as_deref().and_then(|i| i.val) else {
                continue;
            };
            if let Some(paint) = p
                .sp_pr
                .as_deref()
                .and_then(|s| s.fill.as_ref())
                .and_then(|f| self.fill_paint(f, None, 1.0, 1.0, 0, None))
            {
                ser.points.push((idx as usize, paint));
            }
        }
    }

    /// The accent palette, as OpenCalc's `series_color`: accent `i mod 6`, varied
    /// past the sixth.
    fn palette(&mut self, i: usize) -> Paint {
        let accent = ACCENTS[i % ACCENTS.len()];
        let base = self
            .color(&Color::new(ColorValue::Scheme(accent)), None)
            .unwrap_or(CHART_TEXT);
        let c = match (i / ACCENTS.len()).checked_sub(1) {
            None => base,
            Some(round) => vary(base, ACCENT_VARIANTS[round % ACCENT_VARIANTS.len()]),
        };
        Paint::Solid(c)
    }

    /// The paint a series' body is drawn in: its own fill, else its line for a line
    /// series, else the palette.
    fn series_paint(&mut self, s: &Series, i: usize) -> Paint {
        let own = if matches!(s.kind, Kind::Line | Kind::Scatter) {
            s.line.clone().or_else(|| s.fill.clone())
        } else {
            s.fill.clone()
        };
        own.unwrap_or_else(|| self.palette(i))
    }

    /// The paint of point `i` of a pie: its `dPt`, else the palette by point.
    fn point_paint(&mut self, s: &Series, i: usize) -> Paint {
        s.points
            .iter()
            .find(|(idx, _)| *idx == i)
            .map(|(_, p)| p.clone())
            .unwrap_or_else(|| self.palette(i))
    }

    /// One line of chart text, `size` points, in the box `(x, y, w)` of the frame.
    #[allow(clippy::too_many_arguments)] // A text item's place, size, look and owner.
    fn label(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        transform: Affine,
        text: &str,
        (x, y, w): (f64, f64, f64),
        size: f32,
        align: Align,
        color: Rgba,
        bold: bool,
    ) {
        if text.is_empty() || w <= 0.0 {
            return;
        }
        let style = RunStyle {
            family: None,
            size,
            bold,
            italic: false,
            color: TextRgba {
                r: color.r,
                g: color.g,
                b: color.b,
                a: color.a,
            },
            underline: None,
        };
        let body = BodyLayout {
            wrap: false,
            paragraphs: vec![ParagraphLayout {
                align,
                runs: vec![StyledText {
                    text: text.to_owned(),
                    style: style.clone(),
                }],
                end: style,
                ..ParagraphLayout::default()
            }],
            ..BodyLayout::default()
        };
        let frame = Frame {
            x: (x / EMU_PER_POINT) as f32,
            y: (y / EMU_PER_POINT) as f32,
            width: (w / EMU_PER_POINT) as f32,
            height: size * 2.0,
        };
        let layout = layout_body(&body, frame);
        let pt = |v: f32| f64::from(v) * EMU_PER_POINT;
        for run in layout.runs {
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
        }
    }

    /// The legend, taken off the plot's side first: OpenCalc's `legend_box` and
    /// `draw_legend`, with the label widths measured in the face drawn.
    #[allow(clippy::too_many_arguments)] // The legend's entries, side, plot and look.
    fn legend(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        transform: Affine,
        entries: &[(String, Paint)],
        pos: c::LegendPosValue,
        plot: &mut Plot,
        size: f32,
        color: Rgba,
    ) {
        if entries.is_empty() {
            return;
        }
        let em = f64::from(size) * EMU_PER_POINT;
        let width_of = |s: &str| {
            f64::from(casual_slide_text::advance_width(
                s, size, false, false, None,
            )) * EMU_PER_POINT
        };
        let swatch = em * 0.7;
        let row = em * 1.5;
        let side = matches!(
            pos,
            c::LegendPosValue::R | c::LegendPosValue::L | c::LegendPosValue::Tr
        );
        if side {
            let widest = entries
                .iter()
                .map(|(n, _)| width_of(n))
                .fold(24.0 * PX, f64::max);
            let width = (widest + swatch + 12.0 * PX).min(plot.w * 0.4);
            if plot.w - width < 40.0 * PX {
                return;
            }
            plot.w -= width;
            let left = if pos == c::LegendPosValue::L {
                let l = plot.x;
                plot.x += width;
                l
            } else {
                plot.x + plot.w + 6.0 * PX
            };
            let total = row * entries.len() as f64;
            let mut y = plot.y + ((plot.h - total) / 2.0).max(0.0);
            for (name, paint) in entries {
                if y + row > plot.y + plot.h + row {
                    break;
                }
                self.swatch(
                    node,
                    layer,
                    transform,
                    left,
                    y + (row - swatch) / 2.0,
                    swatch,
                    paint,
                );
                let tx = left + swatch + 4.0 * PX;
                self.label(
                    node,
                    layer,
                    transform,
                    name,
                    (tx, y + (row - em * 1.2) / 2.0, width - swatch - 4.0 * PX),
                    size,
                    Align::Left,
                    color,
                    false,
                );
                y += row;
            }
            return;
        }
        if plot.h - row < 40.0 * PX {
            return;
        }
        plot.h -= row;
        let y = if pos == c::LegendPosValue::T {
            let t = plot.y;
            plot.y += row;
            t
        } else {
            plot.y + plot.h + 2.0 * PX
        };
        let total: f64 = entries
            .iter()
            .map(|(n, _)| swatch + 4.0 * PX + width_of(n) + 10.0 * PX)
            .sum();
        let mut x = plot.x + ((plot.w - total) / 2.0).max(0.0);
        for (name, paint) in entries {
            let wn = width_of(name);
            if x + swatch > plot.x + plot.w {
                break;
            }
            self.swatch(
                node,
                layer,
                transform,
                x,
                y + (row - swatch) / 2.0,
                swatch,
                paint,
            );
            let tx = x + swatch + 4.0 * PX;
            self.label(
                node,
                layer,
                transform,
                name,
                (tx, y + (row - em * 1.2) / 2.0, wn + 2.0 * PX),
                size,
                Align::Left,
                color,
                false,
            );
            x = tx + wn + 10.0 * PX;
        }
    }

    #[allow(clippy::too_many_arguments)] // A square's place, size, paint and owner.
    fn swatch(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        transform: Affine,
        x: f64,
        y: f64,
        size: f64,
        paint: &Paint,
    ) {
        let at = transform.then_after(&Affine::translate(x, y));
        self.push(
            node,
            layer,
            at,
            rect_path(size, size),
            Some(paint.clone()),
            None,
        );
    }

    /// A pie or doughnut: one series, its positive values, from the first slice
    /// angle clockwise from twelve o'clock.
    fn pie(&mut self, node: &ShapeNode, layer: Layer, transform: Affine, s: &Series, plot: Plot) {
        let total: f64 = s.values.iter().flatten().filter(|v| **v > 0.0).sum();
        if total <= 0.0 || !total.is_finite() {
            return;
        }
        let cx = plot.x + plot.w / 2.0;
        let cy = plot.y + plot.h / 2.0;
        let r = (plot.w.min(plot.h) / 2.0 - 4.0 * PX).max(6.0 * PX);
        let inner = r * s.hole;
        let mut from = s.first_angle;
        let edge = Stroke {
            paint: Paint::Solid(Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            }),
            width: 9_525.0,
            dash: Vec::new(),
            cap: Cap::Butt,
            join: Join::Round,
        };
        for (i, v) in s.values.iter().enumerate() {
            let Some(v) = v.filter(|v| *v > 0.0) else {
                continue;
            };
            let sweep = v / total * 360.0;
            let path = wedge(cx, cy, r, inner, from, from + sweep);
            let paint = self.point_paint(s, i);
            self.push(
                node,
                layer,
                transform,
                path,
                Some(paint),
                Some(edge.clone()),
            );
            from += sweep;
        }
    }

    /// Everything with a category axis and a value axis.
    #[allow(clippy::too_many_arguments)] // The chart, its series, the plot, and the label look.
    fn xy(
        &mut self,
        node: &ShapeNode,
        layer: Layer,
        transform: Affine,
        chart: &c::Chart,
        series: &[Series],
        mut plot: Plot,
        size: f32,
        color: Rgba,
    ) {
        let series: Vec<&Series> = series.iter().filter(|s| s.kind != Kind::Pie).collect();
        if series.is_empty() {
            return;
        }
        let horizontal = series.iter().all(|s| s.kind == Kind::Bar);
        let scatter = series.iter().all(|s| s.kind == Kind::Scatter);
        let points = series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0)
            .min(MAX_CHART_POINTS);
        if points == 0 {
            return;
        }
        let val_ax = chart.plot_area.as_deref().and_then(|p| {
            p.axes.iter().find_map(|a| match a {
                c::PlotAreaAxes::ValAx(v) => Some(&**v),
                _ => None,
            })
        });
        let cat_deleted = chart
            .plot_area
            .as_deref()
            .and_then(|p| {
                p.axes.iter().find_map(|a| match a {
                    c::PlotAreaAxes::CatAx(v) => v.delete.as_deref().and_then(|d| d.val),
                    c::PlotAreaAxes::DateAx(v) => v.delete.as_deref().and_then(|d| d.val),
                    _ => None,
                })
            })
            .unwrap_or(false);
        let val_deleted = val_ax
            .and_then(|v| v.delete.as_deref())
            .and_then(|d| d.val)
            .unwrap_or(false);
        let percent = series.iter().any(|s| s.grouping == Grouping::Percent);

        // The value extent, holding zero, stacks summed apart by sign (OpenCalc).
        let (mut lo, mut hi) = extent(&series, points);
        if let Some(scaling) = val_ax.and_then(|v| v.scaling.as_deref()) {
            if let Some(m) = scaling.min.as_deref().and_then(|d| d.val) {
                lo = m.get();
            }
            if let Some(m) = scaling.max.as_deref().and_then(|d| d.val) {
                hi = m.get();
            }
        }
        if !(lo.is_finite() && hi.is_finite()) || hi <= lo {
            hi = lo + 1.0;
        }
        let unit = val_ax
            .and_then(|v| v.major_unit.as_deref())
            .and_then(|u| u.val)
            .map(|u| u.get())
            .filter(|u| u.is_finite() && *u > 0.0);
        let step = unit.unwrap_or_else(|| nice_step(hi - lo));
        let lo = (lo / step).floor() * step;
        let hi = (hi / step).ceil() * step;
        let ticks = (((hi - lo) / step).round() as usize).min(50);

        // Room for the value labels and the category labels.
        let em = f64::from(size) * EMU_PER_POINT;
        let label_of = |v: f64| axis_label(v, step, percent);
        let widest = (0..=ticks)
            .map(|i| label_of(lo + step * i as f64))
            .map(|t| {
                f64::from(casual_slide_text::advance_width(
                    &t, size, false, false, None,
                )) * EMU_PER_POINT
            })
            .fold(0.0, f64::max);
        if horizontal {
            if !val_deleted {
                plot.h -= em * 1.6;
            }
            if !cat_deleted {
                let cats = series[0].cats.iter().map(|t| {
                    f64::from(casual_slide_text::advance_width(
                        t, size, false, false, None,
                    )) * EMU_PER_POINT
                });
                let w = cats.fold(0.0, f64::max).min(plot.w * 0.3);
                plot.x += w + 6.0 * PX;
                plot.w -= w + 6.0 * PX;
            }
        } else {
            if !val_deleted {
                plot.x += widest + 6.0 * PX;
                plot.w -= widest + 6.0 * PX;
            }
            if !cat_deleted {
                plot.h -= em * 1.6;
            }
        }
        if plot.w < 20.0 * PX || plot.h < 20.0 * PX {
            return;
        }
        // Where a value lands along the value axis, and a category position
        // (0 to 1 across the plot) along the other.
        let val = |v: f64| (v - lo) / (hi - lo);
        let to_frame = |cat: f64, v: f64| -> (f64, f64) {
            if horizontal {
                (plot.x + plot.w * v, plot.y + plot.h * (1.0 - cat))
            } else {
                (plot.x + plot.w * cat, plot.y + plot.h * (1.0 - v))
            }
        };
        let grid = Stroke {
            paint: Paint::Solid(CHART_GRID),
            width: 9_525.0,
            dash: Vec::new(),
            cap: Cap::Butt,
            join: Join::Round,
        };
        let gridlines = val_ax.is_some_and(|v| v.major_gridlines.is_some());
        for i in 0..=ticks {
            let v = lo + step * i as f64;
            let (a, b) = (to_frame(0.0, val(v)), to_frame(1.0, val(v)));
            if gridlines || (v == 0.0 || i == 0) {
                let path = vec![PathCmd::MoveTo(a.0, a.1), PathCmd::LineTo(b.0, b.1)];
                self.push(node, layer, transform, path, None, Some(grid.clone()));
            }
            if !val_deleted {
                let text = label_of(v);
                if horizontal {
                    self.label(
                        node,
                        layer,
                        transform,
                        &text,
                        (a.0 - 30.0 * PX, plot.y + plot.h + 3.0 * PX, 60.0 * PX),
                        size,
                        Align::Center,
                        color,
                        false,
                    );
                } else {
                    self.label(
                        node,
                        layer,
                        transform,
                        &text,
                        (
                            plot.x - widest - 6.0 * PX,
                            a.1 - em * 0.6,
                            widest + 2.0 * PX,
                        ),
                        size,
                        Align::Right,
                        color,
                        false,
                    );
                }
            }
        }
        // Category labels, thinned to what fits (OpenCalc's rule).
        if !cat_deleted && !scatter {
            let cats = &series[0].cats;
            let along = if horizontal { plot.h } else { plot.w };
            let every = (((points as f64 * 34.0 * PX) / along).ceil() as usize).max(1);
            let area_only = series.iter().all(|s| s.kind == Kind::Area);
            for (i, label) in cats.iter().enumerate().take(points).step_by(every) {
                let cat = if area_only && points > 1 {
                    i as f64 / (points - 1) as f64
                } else {
                    (i as f64 + 0.5) / points as f64
                };
                let (x, y) = to_frame(cat, 0.0);
                if horizontal {
                    self.label(
                        node,
                        layer,
                        transform,
                        label,
                        (plot.x - plot.w * 0.3 - 6.0 * PX, y - em * 0.6, plot.w * 0.3),
                        size,
                        Align::Right,
                        color,
                        false,
                    );
                } else {
                    let bw = (plot.w / points as f64 * every as f64).max(34.0 * PX);
                    self.label(
                        node,
                        layer,
                        transform,
                        label,
                        (x - bw / 2.0, plot.y + plot.h + 3.0 * PX, bw),
                        size,
                        Align::Center,
                        color,
                        false,
                    );
                }
            }
        }

        // Bars: lanes side by side, or one lane stacked; width from `gapWidth`.
        let bars: Vec<(usize, &Series)> = series
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s.kind, Kind::Column | Kind::Bar))
            .map(|(i, s)| (i, *s))
            .collect();
        if let Some((_, first)) = bars.first() {
            let stacked = first.grouping != Grouping::Clustered;
            let lanes = if stacked { 1 } else { bars.len() };
            let group = 1.0 / points as f64;
            let bw = group / (lanes as f64 + first.gap);
            for p in 0..points {
                let (mut up, mut down) = (0.0f64, 0.0f64);
                let total: f64 = bars
                    .iter()
                    .filter_map(|(_, s)| s.values.get(p).copied().flatten())
                    .map(f64::abs)
                    .sum();
                for (lane, (i, s)) in bars.iter().enumerate() {
                    let Some(mut v) = s.values.get(p).copied().flatten() else {
                        continue;
                    };
                    if first.grouping == Grouping::Percent {
                        if total <= 0.0 {
                            continue;
                        }
                        v /= total;
                    }
                    let (v0, v1) = if stacked {
                        let base = if v >= 0.0 { &mut up } else { &mut down };
                        let from = *base;
                        *base += v;
                        (from, *base)
                    } else {
                        (0.0, v)
                    };
                    let lane = if stacked { 0 } else { lane };
                    let c0 = p as f64 * group + first.gap * bw / 2.0 + lane as f64 * bw;
                    let a = to_frame(c0, val(v0.clamp(lo, hi)));
                    let b = to_frame(c0 + bw, val(v1.clamp(lo, hi)));
                    let (x, y) = (a.0.min(b.0), a.1.min(b.1));
                    let (w, h) = ((a.0 - b.0).abs(), (a.1 - b.1).abs());
                    let paint = s
                        .points
                        .iter()
                        .find(|(idx, _)| *idx == p)
                        .map(|(_, pt)| pt.clone())
                        .unwrap_or_else(|| self.series_paint(s, *i));
                    let at = transform.then_after(&Affine::translate(x, y));
                    self.push(node, layer, at, rect_path(w, h), Some(paint), None);
                }
            }
        }

        // Lines and areas: one path per series, stacked by family where asked.
        let mut stack: Vec<f64> = vec![0.0; points];
        let mut stack_family = None;
        for (i, s) in series.iter().enumerate() {
            if !matches!(s.kind, Kind::Line | Kind::Area | Kind::Scatter) {
                continue;
            }
            if stack_family != Some(s.family) {
                stack.iter_mut().for_each(|v| *v = 0.0);
                stack_family = Some(s.family);
            }
            let totals: Vec<f64> = (0..points)
                .map(|p| {
                    series
                        .iter()
                        .filter(|o| o.family == s.family)
                        .filter_map(|o| o.values.get(p).copied().flatten())
                        .map(f64::abs)
                        .sum()
                })
                .collect();
            let (xlo, xhi) =
                s.xs.as_ref()
                    .map(|xs| {
                        xs.iter()
                            .flatten()
                            .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| {
                                (a.min(*v), b.max(*v))
                            })
                    })
                    .filter(|(a, b)| a.is_finite() && b > a)
                    .unwrap_or((0.0, 0.0));
            let mut pts: Vec<Option<(f64, f64, f64)>> = Vec::with_capacity(points);
            for (p, slot) in stack.iter_mut().enumerate().take(points) {
                let Some(mut v) = s.values.get(p).copied().flatten() else {
                    pts.push(None);
                    continue;
                };
                if s.grouping == Grouping::Percent {
                    let t = totals.get(p).copied().unwrap_or(0.0);
                    v = if t > 0.0 { v / t } else { 0.0 };
                }
                let base = if s.grouping == Grouping::Clustered {
                    0.0
                } else {
                    *slot
                };
                let top = base + v;
                if s.grouping != Grouping::Clustered {
                    *slot = top;
                }
                let cat = match s.xs.as_ref().and_then(|xs| xs.get(p).copied().flatten()) {
                    Some(x) if xhi > xlo => (x - xlo) / (xhi - xlo),
                    _ if s.kind == Kind::Area && points > 1 => p as f64 / (points - 1) as f64,
                    _ if s.kind == Kind::Scatter && points > 1 => p as f64 / (points - 1) as f64,
                    _ => (p as f64 + 0.5) / points as f64,
                };
                pts.push(Some((cat, val(base.clamp(lo, hi)), val(top.clamp(lo, hi)))));
            }
            let paint = self.series_paint(s, i);
            if s.kind == Kind::Area {
                let known: Vec<(f64, f64, f64)> = pts.iter().flatten().copied().collect();
                if known.len() >= 2 {
                    let mut path = Vec::new();
                    for (k, (c0, _, top)) in known.iter().enumerate() {
                        let (x, y) = to_frame(*c0, *top);
                        path.push(if k == 0 {
                            PathCmd::MoveTo(x, y)
                        } else {
                            PathCmd::LineTo(x, y)
                        });
                    }
                    for (c0, base, _) in known.iter().rev() {
                        let (x, y) = to_frame(*c0, *base);
                        path.push(PathCmd::LineTo(x, y));
                    }
                    path.push(PathCmd::Close);
                    self.push(node, layer, transform, path, Some(paint), None);
                }
                continue;
            }
            if s.smooth_or_line {
                let stroke = Stroke {
                    paint: paint.clone(),
                    width: 28_575.0,
                    dash: Vec::new(),
                    cap: Cap::Round,
                    join: Join::Round,
                };
                // A gap in the values breaks the line, as `dispBlanksAs="gap"`.
                let mut path = Vec::new();
                let mut open = false;
                for p in &pts {
                    match p {
                        Some((c0, _, top)) => {
                            let (x, y) = to_frame(*c0, *top);
                            path.push(if open {
                                PathCmd::LineTo(x, y)
                            } else {
                                PathCmd::MoveTo(x, y)
                            });
                            open = true;
                        }
                        None => open = false,
                    }
                }
                if !path.is_empty() {
                    self.push(node, layer, transform, path, None, Some(stroke));
                }
            }
            if s.kind == Kind::Scatter {
                let r = 3.5 * PX;
                for (c0, _, top) in pts.iter().flatten() {
                    let (x, y) = to_frame(*c0, *top);
                    let path = wedge(x, y, r, 0.0, 0.0, 360.0);
                    self.push(node, layer, transform, path, Some(paint.clone()), None);
                }
            }
        }
    }
}

/// The values a numeric source caches, by `idx`.
fn num_values(src: Option<&c::NumDataSource>) -> Vec<Option<f64>> {
    let Some(src) = src else {
        return Vec::new();
    };
    let Some(data) = src
        .num_ref
        .as_deref()
        .and_then(|r| r.num_cache.as_deref())
        .or(src.num_lit.as_deref())
    else {
        return Vec::new();
    };
    let count = data
        .pt_count
        .as_deref()
        .and_then(|c| c.val)
        .map(|c| c as usize)
        .or_else(|| {
            data.pt
                .iter()
                .filter_map(|p| p.idx)
                .max()
                .map(|m| m as usize + 1)
        })
        .unwrap_or(data.pt.len())
        .min(MAX_CHART_POINTS);
    let mut out = vec![None; count];
    for (k, p) in data.pt.iter().enumerate() {
        let idx = p.idx.map_or(k, |i| i as usize);
        if let Some(slot) = out.get_mut(idx) {
            *slot =
                p.v.as_deref()
                    .and_then(|v| v.trim().parse::<f64>().ok())
                    .filter(|v| v.is_finite());
        }
    }
    out
}

/// The text a category source caches, by `idx`.
fn cat_values(src: Option<&c::AxDataSource>) -> Vec<String> {
    let Some(src) = src else {
        return Vec::new();
    };
    let pts: Vec<(Option<u32>, Option<&str>)> = if let Some(d) = src
        .str_ref
        .as_deref()
        .and_then(|r| r.str_cache.as_deref())
        .or(src.str_lit.as_deref())
    {
        d.pt.iter().map(|p| (p.idx, p.v.as_deref())).collect()
    } else if let Some(d) = src
        .num_ref
        .as_deref()
        .and_then(|r| r.num_cache.as_deref())
        .or(src.num_lit.as_deref())
    {
        d.pt.iter().map(|p| (p.idx, p.v.as_deref())).collect()
    } else if let Some(l) = src
        .multi_lvl_str_ref
        .as_deref()
        .and_then(|r| r.multi_lvl_str_cache.as_deref())
        .and_then(|c| c.lvl.first())
    {
        l.pt.iter().map(|p| (p.idx, p.v.as_deref())).collect()
    } else {
        return Vec::new();
    };
    let count = pts
        .iter()
        .enumerate()
        .map(|(k, (i, _))| i.map_or(k, |i| i as usize) + 1)
        .max()
        .unwrap_or(0)
        .min(MAX_CHART_POINTS);
    let mut out = vec![String::new(); count];
    for (k, (idx, v)) in pts.into_iter().enumerate() {
        if let Some(slot) = out.get_mut(idx.map_or(k, |i| i as usize)) {
            *slot = v.unwrap_or("").to_owned();
        }
    }
    out
}

fn first_angle(a: &Option<Box<c::FirstSliceAng>>) -> f64 {
    a.as_deref()
        .and_then(|a| a.val)
        .map_or(0.0, |v| f64::from(v.min(360)))
}

/// The chart's title, if it shows one: its own text, else the one series' name, as
/// PowerPoint titles a one-series chart.
fn chart_title(chart: &c::Chart, series: &[Series]) -> Option<String> {
    if chart.auto_title_deleted.as_deref().and_then(|b| b.val) == Some(true)
        && chart.title.is_none()
    {
        return None;
    }
    let title = chart.title.as_deref()?;
    let tx = title.tx.as_deref();
    if let Some(rich) = tx.and_then(|t| t.rich.as_deref()) {
        let text = body_text(rich);
        if !text.is_empty() {
            return Some(text);
        }
    }
    if let Some(v) = tx
        .and_then(|t| t.str_ref.as_deref())
        .and_then(|r| r.str_cache.as_deref())
        .and_then(|c| c.pt.first())
        .and_then(|p| p.v.clone())
    {
        return Some(v);
    }
    match series {
        [one] if !one.name.is_empty() => Some(one.name.clone()),
        _ => Some(if series.len() == 1 {
            String::new()
        } else {
            "Chart Title".to_owned()
        }),
    }
    .filter(|t| !t.is_empty())
}

fn body_text(body: &TextBody) -> String {
    let mut out = String::new();
    for (i, p) in body.paragraphs.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        for item in &p.items {
            match item {
                TextItem::Run(r) => out.push_str(r.text.as_deref().unwrap_or("")),
                TextItem::Field(f) => out.push_str(f.text.as_deref().unwrap_or("")),
                _ => {}
            }
        }
    }
    out
}

/// The first run size a body states, points.
fn body_size(body: &TextBody) -> Option<f32> {
    body.paragraphs.iter().find_map(|p| {
        p.properties
            .as_deref()
            .and_then(|pp| pp.default_run.as_deref())
            .and_then(|r| r.size)
            .or_else(|| {
                p.items.iter().find_map(|i| match i {
                    TextItem::Run(r) => r.properties.as_deref().and_then(|r| r.size),
                    _ => None,
                })
            })
            .map(|s| s as f32 / 100.0)
    })
}

/// The first solid text colour a body states.
fn body_color(body: &TextBody) -> Option<Color> {
    body.paragraphs.iter().find_map(|p| {
        let fill = p
            .properties
            .as_deref()
            .and_then(|pp| pp.default_run.as_deref())
            .and_then(|r| r.fill.as_ref());
        match fill {
            Some(Fill::Solid(Some(c))) => Some(c.clone()),
            _ => None,
        }
    })
}

/// The value range to cover, always holding zero; stacks sum positive and negative
/// apart, and a percent stack is the whole (OpenCalc's `extent_of`).
fn extent(series: &[&Series], points: usize) -> (f64, f64) {
    let (mut lo, mut hi) = (0.0f64, 0.0f64);
    let mut families: Vec<usize> = series.iter().map(|s| s.family).collect();
    families.dedup();
    for family in families {
        let members: Vec<&&Series> = series.iter().filter(|s| s.family == family).collect();
        let grouping = members.first().map_or(Grouping::Clustered, |s| s.grouping);
        match grouping {
            Grouping::Percent => hi = hi.max(1.0),
            Grouping::Stacked => {
                for p in 0..points {
                    let (mut up, mut down) = (0.0f64, 0.0f64);
                    for v in members
                        .iter()
                        .filter_map(|s| s.values.get(p).copied().flatten())
                    {
                        if v >= 0.0 {
                            up += v;
                        } else {
                            down += v;
                        }
                    }
                    lo = lo.min(down);
                    hi = hi.max(up);
                }
            }
            Grouping::Clustered => {
                for s in &members {
                    for v in s.values.iter().flatten() {
                        lo = lo.min(*v);
                        hi = hi.max(*v);
                    }
                }
            }
        }
    }
    if lo == hi {
        hi = lo + 1.0;
    }
    (lo, hi)
}

/// A round step giving about five ticks over `range`: 1, 2 or 5 times a power of ten.
fn nice_step(range: f64) -> f64 {
    if !(range.is_finite() && range > 0.0) {
        return 1.0;
    }
    let raw = range / 5.0;
    let mag = 10f64.powf(raw.log10().floor());
    let norm = raw / mag;
    let k = if norm <= 1.0 {
        1.0
    } else if norm <= 2.0 {
        2.0
    } else if norm <= 5.0 {
        5.0
    } else {
        10.0
    };
    k * mag
}

/// A tick's label: as many decimals as the step needs, or a percentage.
fn axis_label(v: f64, step: f64, percent: bool) -> String {
    let (v, step) = if percent {
        (v * 100.0, step * 100.0)
    } else {
        (v, step)
    };
    let decimals = if step >= 1.0 || step <= 0.0 {
        0
    } else {
        (-step.log10().floor()) as usize
    }
    .min(6);
    let mut s = format!("{v:.decimals$}");
    if s == "-0" {
        s = "0".to_owned();
    }
    if percent {
        s.push('%');
    }
    s
}

/// OpenCalc's `vary`: towards white (`amount > 0`) or black (`amount < 0`).
fn vary(c: Rgba, amount: f64) -> Rgba {
    let f = |v: u8| {
        let v = f64::from(v);
        let moved = if amount >= 0.0 {
            v + (255.0 - v) * amount
        } else {
            v * (1.0 + amount)
        };
        moved.round().clamp(0.0, 255.0) as u8
    };
    Rgba {
        r: f(c.r),
        g: f(c.g),
        b: f(c.b),
        a: c.a,
    }
}

/// A pie wedge, or a ring segment when `inner > 0`, from `a0` to `a1` degrees
/// clockwise from twelve o'clock, as cubics of at most a quarter turn.
fn wedge(cx: f64, cy: f64, r: f64, inner: f64, a0: f64, a1: f64) -> Vec<PathCmd> {
    let at = |rad: f64, deg: f64| {
        let t = (deg - 90.0).to_radians();
        (cx + rad * t.cos(), cy + rad * t.sin())
    };
    let arc = |rad: f64, from: f64, to: f64, out: &mut Vec<PathCmd>| {
        let n = (((to - from).abs() / 90.0).ceil() as usize).max(1);
        let step = (to - from) / n as f64;
        for k in 0..n {
            let s = from + step * k as f64;
            let e = s + step;
            let alpha = 4.0 / 3.0 * (step.to_radians() / 4.0).tan();
            let (sx, sy) = at(rad, s);
            let (ex, ey) = at(rad, e);
            let (ts, te) = ((s - 90.0).to_radians(), (e - 90.0).to_radians());
            out.push(PathCmd::CubicTo(
                sx - alpha * rad * ts.sin(),
                sy + alpha * rad * ts.cos(),
                ex + alpha * rad * te.sin(),
                ey - alpha * rad * te.cos(),
                ex,
                ey,
            ));
        }
    };
    let mut out = Vec::new();
    let full = (a1 - a0).abs() >= 359.999;
    let start = at(r, a0);
    if inner > 0.0 {
        out.push(PathCmd::MoveTo(start.0, start.1));
        arc(r, a0, a1, &mut out);
        let back = at(inner, a1);
        if full {
            out.push(PathCmd::Close);
            out.push(PathCmd::MoveTo(back.0, back.1));
        } else {
            out.push(PathCmd::LineTo(back.0, back.1));
        }
        arc(inner, a1, a0, &mut out);
        out.push(PathCmd::Close);
    } else if full {
        out.push(PathCmd::MoveTo(start.0, start.1));
        arc(r, a0, a1, &mut out);
        out.push(PathCmd::Close);
    } else {
        out.push(PathCmd::MoveTo(cx, cy));
        out.push(PathCmd::LineTo(start.0, start.1));
        arc(r, a0, a1, &mut out);
        out.push(PathCmd::Close);
    }
    out
}

/// Whether a `c:CT_Boolean` is on; an element without `val` means true.
fn on(b: Option<&c::Boolean>) -> bool {
    b.is_some_and(|b| b.val.unwrap_or(true))
}

/// Whether any of a label's `show*` switches is on.
fn shows(v: [Option<&c::Boolean>; 5]) -> bool {
    v.into_iter().any(on)
}

/// Whether a series' or a family's data labels show anything (`SKEL-04`).
fn labels_shown(d: Option<&c::DLbls>) -> bool {
    let Some(d) = d else { return false };
    if on(d.delete.as_deref()) {
        return false;
    }
    shows([
        d.show_val.as_deref(),
        d.show_cat_name.as_deref(),
        d.show_ser_name.as_deref(),
        d.show_percent.as_deref(),
        d.show_bubble_size.as_deref(),
    ]) || d.d_lbl.iter().any(|l| {
        !on(l.delete.as_deref())
            && (l.tx.is_some()
                || shows([
                    l.show_val.as_deref(),
                    l.show_cat_name.as_deref(),
                    l.show_ser_name.as_deref(),
                    l.show_percent.as_deref(),
                    l.show_bubble_size.as_deref(),
                ]))
    })
}

/// Whether a series' marker is drawn: no symbol means the family's choice.
fn marker_shown(m: Option<&c::Marker>, family_default: bool) -> bool {
    match m.and_then(|m| m.symbol.as_deref()) {
        Some(s) => s.val != Some(c::MarkerStyleValue::None),
        None => family_default,
    }
}

/// What one series carries that is not drawn.
struct SerParts<'a> {
    labels: Option<&'a c::DLbls>,
    trendlines: bool,
    err_bars: bool,
}

/// Every part of a chart the renderer does not draw, named for the render
/// report (`SKEL-04`, "nothing lost silently"), each name once per chart.
fn undrawn_parts(space: &c::ChartSpace) -> Vec<&'static str> {
    use c::PlotAreaCharts as P;
    let mut out: Vec<&'static str> = Vec::new();
    let mut add = |name: &'static str, present: bool| {
        if present && !out.contains(&name) {
            out.push(name);
        }
    };
    fn sers(
        add: &mut dyn FnMut(&'static str, bool),
        family: Option<&c::DLbls>,
        parts: &mut dyn Iterator<Item = SerParts<'_>>,
    ) {
        add("chart data labels", labels_shown(family));
        for s in parts {
            add("chart data labels", labels_shown(s.labels));
            add("chart trendlines", s.trendlines);
            add("chart error bars", s.err_bars);
        }
    }
    fn line(s: &c::LineSer) -> SerParts<'_> {
        SerParts {
            labels: s.d_lbls.as_deref(),
            trendlines: !s.trendline.is_empty(),
            err_bars: s.err_bars.is_some(),
        }
    }
    fn bar(s: &c::BarSer) -> SerParts<'_> {
        SerParts {
            labels: s.d_lbls.as_deref(),
            trendlines: !s.trendline.is_empty(),
            err_bars: s.err_bars.is_some(),
        }
    }
    fn area(s: &c::AreaSer) -> SerParts<'_> {
        SerParts {
            labels: s.d_lbls.as_deref(),
            trendlines: !s.trendline.is_empty(),
            err_bars: !s.err_bars.is_empty(),
        }
    }
    fn pie(s: &c::PieSer) -> SerParts<'_> {
        SerParts {
            labels: s.d_lbls.as_deref(),
            trendlines: false,
            err_bars: false,
        }
    }
    fn scatter(s: &c::ScatterSer) -> SerParts<'_> {
        SerParts {
            labels: s.d_lbls.as_deref(),
            trendlines: !s.trendline.is_empty(),
            err_bars: !s.err_bars.is_empty(),
        }
    }
    for chart in space.charts() {
        match chart {
            P::LineChart(l) => {
                sers(&mut add, l.d_lbls.as_deref(), &mut l.ser.iter().map(line));
                let dflt = on(l.marker.as_deref());
                for s in &l.ser {
                    add(
                        "chart line markers",
                        marker_shown(s.marker.as_deref(), dflt),
                    );
                }
                add("chart drop lines", l.drop_lines.is_some());
                add("chart high-low lines", l.hi_low_lines.is_some());
                add("chart up-down bars", l.up_down_bars.is_some());
            }
            P::Line3DChart(l) => {
                sers(&mut add, l.d_lbls.as_deref(), &mut l.ser.iter().map(line));
                add("chart drop lines", l.drop_lines.is_some());
            }
            P::StockChart(l) => {
                sers(&mut add, l.d_lbls.as_deref(), &mut l.ser.iter().map(line));
                for s in &l.ser {
                    add(
                        "chart line markers",
                        marker_shown(s.marker.as_deref(), false),
                    );
                }
                add("chart drop lines", l.drop_lines.is_some());
                add("chart high-low lines", l.hi_low_lines.is_some());
                add("chart up-down bars", l.up_down_bars.is_some());
            }
            P::ScatterChart(sc) => {
                sers(
                    &mut add,
                    sc.d_lbls.as_deref(),
                    &mut sc.ser.iter().map(scatter),
                );
                for s in &sc.ser {
                    add(
                        "chart scatter markers",
                        marker_shown(s.marker.as_deref(), true),
                    );
                }
            }
            P::BarChart(b) => {
                sers(&mut add, b.d_lbls.as_deref(), &mut b.ser.iter().map(bar));
                add("chart series lines", !b.ser_lines.is_empty());
            }
            P::Bar3DChart(b) => {
                sers(&mut add, b.d_lbls.as_deref(), &mut b.ser.iter().map(bar));
            }
            P::AreaChart(a) => {
                sers(&mut add, a.d_lbls.as_deref(), &mut a.ser.iter().map(area));
                add("chart drop lines", a.drop_lines.is_some());
            }
            P::Area3DChart(a) => {
                sers(&mut add, a.d_lbls.as_deref(), &mut a.ser.iter().map(area));
                add("chart drop lines", a.drop_lines.is_some());
            }
            P::PieChart(p) => {
                sers(&mut add, p.d_lbls.as_deref(), &mut p.ser.iter().map(pie));
                add(
                    "chart pie explosion",
                    p.ser.iter().any(|s| s.explosion.is_some()),
                );
            }
            P::Pie3DChart(p) => {
                sers(&mut add, p.d_lbls.as_deref(), &mut p.ser.iter().map(pie));
                add(
                    "chart pie explosion",
                    p.ser.iter().any(|s| s.explosion.is_some()),
                );
            }
            P::DoughnutChart(p) => {
                sers(&mut add, p.d_lbls.as_deref(), &mut p.ser.iter().map(pie));
                add(
                    "chart pie explosion",
                    p.ser.iter().any(|s| s.explosion.is_some()),
                );
            }
            P::OfPieChart(p) => {
                sers(&mut add, p.d_lbls.as_deref(), &mut p.ser.iter().map(pie));
                add(
                    "chart pie explosion",
                    p.ser.iter().any(|s| s.explosion.is_some()),
                );
                add("chart series lines", !p.ser_lines.is_empty());
            }
            // Whole families not drawn are reported where they are skipped.
            P::RadarChart(_) | P::SurfaceChart(_) | P::Surface3DChart(_) | P::BubbleChart(_) => {}
        }
    }
    let table = space
        .chart
        .as_deref()
        .and_then(|c| c.plot_area.as_deref())
        .is_some_and(|p| p.d_table.is_some());
    add("chart data table", table);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_space(ser: c::LineSer, marker: Option<Box<c::Boolean>>) -> c::ChartSpace {
        c::ChartSpace {
            chart: Some(Box::new(c::Chart {
                plot_area: Some(Box::new(c::PlotArea {
                    charts: vec![c::PlotAreaCharts::LineChart(Box::new(c::LineChart {
                        ser: vec![ser],
                        marker,
                        ..Default::default()
                    }))],
                    ..Default::default()
                })),
                ..Default::default()
            })),
            ..Default::default()
        }
    }

    #[test]
    fn undrawn_chart_parts_are_named_for_the_report() {
        let yes = || Some(Box::new(c::Boolean { val: Some(true) }));
        let line = c::LineSer {
            d_lbls: Some(Box::new(c::DLbls {
                show_val: yes(),
                ..Default::default()
            })),
            trendline: vec![c::Trendline::default()],
            err_bars: Some(Box::default()),
            ..Default::default()
        };
        // Names come in walk order: each family's series parts, then its markers.
        assert_eq!(
            undrawn_parts(&line_space(line, yes())),
            [
                "chart data labels",
                "chart trendlines",
                "chart error bars",
                "chart line markers"
            ]
        );
    }

    #[test]
    fn hidden_labels_and_no_markers_are_not_reported() {
        let line = c::LineSer {
            d_lbls: Some(Box::new(c::DLbls {
                show_val: Some(Box::new(c::Boolean { val: Some(false) })),
                ..Default::default()
            })),
            marker: Some(Box::new(c::Marker {
                symbol: Some(Box::new(c::MarkerStyle {
                    val: Some(c::MarkerStyleValue::None),
                })),
                ..Default::default()
            })),
            ..Default::default()
        };
        assert!(undrawn_parts(&line_space(line, None)).is_empty());
    }

    #[test]
    fn ticks_step_in_ones_twos_and_fives() {
        assert_eq!(nice_step(10.0), 2.0);
        assert_eq!(nice_step(100.0), 20.0);
        assert_eq!(nice_step(3.0), 1.0);
        assert!((nice_step(0.4) - 0.1).abs() < 1e-12);
    }

    #[test]
    fn a_stacked_extent_sums_each_sign_apart() {
        let s = |vals: &[f64]| Series {
            name: String::new(),
            values: vals.iter().map(|v| Some(*v)).collect(),
            xs: None,
            cats: Vec::new(),
            fill: None,
            line: None,
            points: Vec::new(),
            kind: Kind::Column,
            grouping: Grouping::Stacked,
            family: 0,
            gap: 1.5,
            hole: 0.0,
            first_angle: 0.0,
            smooth_or_line: true,
        };
        let (a, b) = (s(&[100.0, -10.0]), s(&[140.0, -20.0]));
        assert_eq!(extent(&[&a, &b], 2), (-30.0, 240.0));
    }

    #[test]
    fn a_quarter_wedge_ends_at_three_o_clock() {
        let path = wedge(0.0, 0.0, 10.0, 0.0, 0.0, 90.0);
        let Some(PathCmd::CubicTo(.., x, y)) = path.get(2) else {
            panic!("{path:?}");
        };
        assert!((x - 10.0).abs() < 1e-9 && y.abs() < 1e-9);
    }

    #[test]
    fn percent_ticks_are_labelled_as_percentages() {
        assert_eq!(axis_label(0.25, 0.25, true), "25%");
        assert_eq!(axis_label(2.5, 0.5, false), "2.5");
    }
}
