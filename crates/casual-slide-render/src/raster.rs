//! The CPU backend: a [`DisplayList`] onto a `tiny-skia` pixmap, then a PNG.
//!
//! The executor pattern is OpenCalc's `casual-calc-render` (`render_pixmap_with_images`):
//! a white ground, the list in painter's order, pictures from an [`ImageSource`], and a
//! report of everything that did not reach the surface. What changed for slides is the
//! item set: every item is a path with a transform, so one routine draws shapes,
//! connectors, pictures and picture fills alike.

use std::collections::BTreeMap;

use tiny_skia::{
    Color, FillRule, FilterQuality, GradientStop, LineCap, LineJoin, LinearGradient,
    Paint as SkPaint, PathBuilder, Pattern, Pixmap, Point, RadialGradient, Shader, SpreadMode,
    Stroke as SkStroke, StrokeDash, Transform,
};

use crate::display::{
    Affine, Cap, DisplayItem, DisplayList, Join, PATTERN_CELL, Paint, PathCmd, Rgba, Stroke, Tile,
};
use crate::images::{self, ImageReport, ImageSource};
use crate::{RenderError, RenderReport};

/// EMU in an inch.
pub const EMU_PER_INCH: f64 = 914_400.0;

/// The longest side a render may have, in pixels. A slide is at most 56 inches a
/// side, so this allows about 290 DPI on the largest and far more on a usual one.
pub const MAX_SIDE_PX: u32 = 16_384;

/// Outlines for the glyphs a [`DisplayItem::GlyphRun`] names. The text crate (lane B)
/// implements this; a render given none reports every glyph run as undrawn.
pub trait GlyphOutlines {
    /// Append glyph `id` of `face` at em size `size` EMU, origin at 0,0, y down, in
    /// EMU. `false` if the face or glyph is unknown.
    fn outline(&self, face: &str, id: u32, size: f64, out: &mut Vec<PathCmd>) -> bool;
}

/// Glyph outlines from `casual-slide-text`'s bundled and registered faces, by the
/// face key its glyph runs carry.
#[derive(Debug, Clone, Copy, Default)]
pub struct BundledOutlines;

impl GlyphOutlines for BundledOutlines {
    fn outline(&self, face: &str, id: u32, size: f64, out: &mut Vec<PathCmd>) -> bool {
        use casual_slide_text::OutlineOp as O;
        let mut ops = Vec::new();
        if !casual_slide_text::outline_by_key(face, id, size as f32, &mut ops) {
            return false;
        }
        let f = f64::from;
        out.extend(ops.into_iter().map(|op| match op {
            O::MoveTo(x, y) => PathCmd::MoveTo(f(x), f(y)),
            O::LineTo(x, y) => PathCmd::LineTo(f(x), f(y)),
            O::QuadTo(a, b, x, y) => PathCmd::QuadTo(f(a), f(b), f(x), f(y)),
            O::CubicTo(a, b, c, d, x, y) => PathCmd::CubicTo(f(a), f(b), f(c), f(d), f(x), f(y)),
            O::Close => PathCmd::Close,
        }));
        true
    }
}

/// The pixel size a slide renders to at `dpi`.
pub fn pixel_size(list: &DisplayList, dpi: f64) -> (u32, u32) {
    let px = |emu: f64| {
        let v = (emu * dpi / EMU_PER_INCH).round();
        if v.is_finite() && v >= 1.0 {
            v.min(f64::from(MAX_SIDE_PX)) as u32
        } else {
            0
        }
    };
    (px(list.width), px(list.height))
}

fn color(c: Rgba) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

fn sk_transform(t: &Affine) -> Transform {
    Transform::from_row(
        t.a as f32, t.b as f32, t.c as f32, t.d as f32, t.e as f32, t.f as f32,
    )
}

fn build_path(cmds: &[PathCmd]) -> Option<tiny_skia::Path> {
    let mut pb = PathBuilder::new();
    for cmd in cmds {
        match *cmd {
            PathCmd::MoveTo(x, y) => pb.move_to(x as f32, y as f32),
            PathCmd::LineTo(x, y) => pb.line_to(x as f32, y as f32),
            PathCmd::QuadTo(x1, y1, x, y) => pb.quad_to(x1 as f32, y1 as f32, x as f32, y as f32),
            PathCmd::CubicTo(x1, y1, x2, y2, x, y) => pb.cubic_to(
                x1 as f32, y1 as f32, x2 as f32, y2 as f32, x as f32, y as f32,
            ),
            PathCmd::Close => pb.close(),
        }
    }
    pb.finish()
}

fn stops(list: &[crate::display::Stop]) -> Vec<GradientStop> {
    list.iter()
        .map(|s| GradientStop::new(s.offset.clamp(0.0, 1.0) as f32, color(s.color)))
        .collect()
}

/// Pictures decoded once per render, by part.
struct Pictures<'a> {
    source: &'a dyn ImageSource,
    decoded: BTreeMap<String, Option<Pixmap>>,
}

impl Pictures<'_> {
    fn get(&mut self, part: &str, report: &mut ImageReport) -> Option<&Pixmap> {
        if !self.decoded.contains_key(part) {
            let decoded = match images::decode(part, self.source) {
                Ok(p) => Some(p),
                Err(reason) => {
                    report.missed(part, reason);
                    None
                }
            };
            self.decoded.insert(part.to_owned(), decoded);
        }
        self.decoded.get(part).and_then(Option::as_ref)
    }
}

/// Paint `paint` over `path`; `stroke` makes it a stroke rather than a fill.
fn draw(
    pixmap: &mut Pixmap,
    path: &tiny_skia::Path,
    paint: &Paint,
    stroke: Option<&SkStroke>,
    transform: Transform,
    pictures: &mut Pictures<'_>,
    report: &mut ImageReport,
) {
    let shader = match paint {
        Paint::Solid(c) => Some(Shader::SolidColor(color(*c))),
        Paint::Linear {
            start,
            end,
            stops: s,
        } => LinearGradient::new(
            Point::from_xy(start.0 as f32, start.1 as f32),
            Point::from_xy(end.0 as f32, end.1 as f32),
            stops(s),
            SpreadMode::Pad,
            Transform::identity(),
        ),
        Paint::Radial {
            center,
            radius,
            stops: s,
        } => {
            let c = Point::from_xy(center.0 as f32, center.1 as f32);
            RadialGradient::new(
                c,
                c,
                *radius as f32,
                stops(s),
                SpreadMode::Pad,
                Transform::identity(),
            )
            // A gradient of no size is its last colour.
            .or_else(|| s.last().map(|l| Shader::SolidColor(color(l.color))))
        }
        Paint::Picture {
            part,
            frame,
            alpha,
            tile,
        } => {
            let Some(image) = pictures.get(part, report) else {
                return;
            };
            let alpha = alpha.clamp(0.0, 1.0) as f32;
            match tile {
                None => {
                    let (iw, ih) = (image.width() as f32, image.height() as f32);
                    let (x, y, w, h) = (
                        frame.0 as f32,
                        frame.1 as f32,
                        frame.2 as f32,
                        frame.3 as f32,
                    );
                    let place = Transform::from_row(w / iw, 0.0, 0.0, h / ih, x, y);
                    let shader = Pattern::new(
                        image.as_ref(),
                        SpreadMode::Pad,
                        FilterQuality::Bicubic,
                        alpha,
                        place,
                    );
                    paint_with(pixmap, path, shader, stroke, transform);
                }
                Some(tile) => {
                    let Some((cell, place)) = tile_cell(image, *frame, tile) else {
                        return;
                    };
                    let shader = Pattern::new(
                        cell.as_ref(),
                        SpreadMode::Repeat,
                        FilterQuality::Bicubic,
                        alpha,
                        place,
                    );
                    paint_with(pixmap, path, shader, stroke, transform);
                }
            }
            report.drew();
            return;
        }
        Paint::Pattern { fg, bg, bits } => {
            let Some(cell) = pattern_cell(*fg, *bg, bits) else {
                return;
            };
            let k = PATTERN_CELL as f32;
            let shader = Pattern::new(
                cell.as_ref(),
                SpreadMode::Repeat,
                FilterQuality::Nearest,
                1.0,
                Transform::from_scale(k, k),
            );
            paint_with(pixmap, path, shader, stroke, transform);
            return;
        }
    };
    let Some(shader) = shader else {
        return;
    };
    let sk = SkPaint {
        shader,
        anti_alias: true,
        ..SkPaint::default()
    };
    match stroke {
        Some(s) => pixmap.stroke_path(path, &sk, s, transform, None),
        None => pixmap.fill_path(path, &sk, FillRule::Winding, transform, None),
    }
}

/// Fill or stroke `path` with a picture shader, smoothing edges.
fn paint_with(
    pixmap: &mut Pixmap,
    path: &tiny_skia::Path,
    shader: Shader<'_>,
    stroke: Option<&SkStroke>,
    transform: Transform,
) {
    let sk = SkPaint {
        shader,
        anti_alias: true,
        force_hq_pipeline: true,
        ..SkPaint::default()
    };
    match stroke {
        Some(s) => pixmap.stroke_path(path, &sk, s, transform, None),
        None => pixmap.fill_path(path, &sk, FillRule::Winding, transform, None),
    }
}

/// A pattern's 8 by 8 cell: `fg` where a bit is set, `bg` where clear.
fn pattern_cell(fg: Rgba, bg: Rgba, bits: &[u8; 8]) -> Option<Pixmap> {
    let mut cell = Pixmap::new(8, 8)?;
    let (fg, bg) = (color(fg).premultiply(), color(bg).premultiply());
    for (i, px) in cell.pixels_mut().iter_mut().enumerate() {
        let (row, col) = (i / 8, i % 8);
        let set = bits.get(row).is_some_and(|b| b & (0x80 >> col) != 0);
        *px = if set { fg } else { bg }.to_color_u8();
    }
    Some(cell)
}

/// One repeat of a tiled picture, mirrored copies included when `flip` asks, and
/// where the repeat starts in the item's space: the picture at its own size times
/// the scale, aligned in `frame` as `align` says, then moved by the offset.
fn tile_cell(
    image: &Pixmap,
    frame: (f64, f64, f64, f64),
    tile: &Tile,
) -> Option<(Pixmap, Transform)> {
    let (iw, ih) = (f64::from(image.width()), f64::from(image.height()));
    let emu_per_px = EMU_PER_INCH / tile.dpi;
    let (tw, th) = (
        iw * emu_per_px * tile.scale.0,
        ih * emu_per_px * tile.scale.1,
    );
    if !(tw.is_finite() && th.is_finite()) || tw.abs() < 1.0 || th.abs() < 1.0 {
        return None;
    }
    let (fx, fy) = (u32::from(tile.flip.0) + 1, u32::from(tile.flip.1) + 1);
    let mut cell = Pixmap::new(
        image.width().checked_mul(fx)?,
        image.height().checked_mul(fy)?,
    )?;
    let (w, h) = (image.width() as f32, image.height() as f32);
    for (mx, my) in [(false, false), (true, false), (false, true), (true, true)] {
        if (mx && fx == 1) || (my && fy == 1) {
            continue;
        }
        // A mirrored copy: reflected within its own cell, beside the original.
        let t = Transform::from_row(
            if mx { -1.0 } else { 1.0 },
            0.0,
            0.0,
            if my { -1.0 } else { 1.0 },
            if mx { 2.0 * w } else { 0.0 },
            if my { 2.0 * h } else { 0.0 },
        );
        cell.draw_pixmap(
            0,
            0,
            image.as_ref(),
            &tiny_skia::PixmapPaint::default(),
            t,
            None,
        );
    }
    let (x, y, bw, bh) = frame;
    let x0 = x + tile.align.0 * (bw - tw) + tile.offset.0;
    let y0 = y + tile.align.1 * (bh - th) + tile.offset.1;
    let place = Transform::from_row(
        (tw / iw) as f32,
        0.0,
        0.0,
        (th / ih) as f32,
        x0 as f32,
        y0 as f32,
    );
    Some((cell, place))
}

fn sk_stroke(stroke: &Stroke) -> SkStroke {
    let (line_join, miter_limit) = match stroke.join {
        Join::Round => (LineJoin::Round, 4.0),
        Join::Bevel => (LineJoin::Bevel, 4.0),
        Join::Miter(limit) => (LineJoin::Miter, limit.clamp(1.0, 100.0) as f32),
    };
    SkStroke {
        width: stroke.width.max(0.0) as f32,
        miter_limit,
        line_cap: match stroke.cap {
            Cap::Butt => LineCap::Butt,
            Cap::Round => LineCap::Round,
            Cap::Square => LineCap::Square,
        },
        line_join,
        dash: StrokeDash::new(stroke.dash.iter().map(|d| *d as f32).collect(), 0.0),
    }
}

/// Execute a display list at `dpi` onto a pixmap, drawing the pictures `images`
/// supplies and the glyphs `glyphs` outlines.
pub fn render_pixmap(
    list: &DisplayList,
    dpi: f64,
    images: &dyn ImageSource,
    glyphs: Option<&dyn GlyphOutlines>,
) -> Result<(Pixmap, RenderReport), RenderError> {
    let (width, height) = pixel_size(list, dpi);
    let mut pixmap =
        Pixmap::new(width, height).ok_or(RenderError::InvalidSize { width, height })?;
    pixmap.fill(Color::WHITE);
    let scale = Affine::scale(dpi / EMU_PER_INCH, dpi / EMU_PER_INCH);
    let mut pictures = Pictures {
        source: images,
        decoded: BTreeMap::new(),
    };
    let mut report = RenderReport::default();
    for item in &list.items {
        match item {
            DisplayItem::Path {
                transform,
                path,
                fill,
                stroke,
                ..
            } => {
                let Some(p) = build_path(path) else {
                    continue;
                };
                let t = sk_transform(&scale.then_after(transform));
                if let Some(fill) = fill {
                    draw(
                        &mut pixmap,
                        &p,
                        fill,
                        None,
                        t,
                        &mut pictures,
                        &mut report.images,
                    );
                }
                if let Some(stroke) = stroke {
                    let s = sk_stroke(stroke);
                    draw(
                        &mut pixmap,
                        &p,
                        &stroke.paint,
                        Some(&s),
                        t,
                        &mut pictures,
                        &mut report.images,
                    );
                }
            }
            DisplayItem::GlyphRun {
                transform,
                face,
                size,
                glyphs: run,
                fill,
                ..
            } => {
                let Some(outlines) = glyphs else {
                    report.glyph_runs_undrawn = report.glyph_runs_undrawn.saturating_add(1);
                    continue;
                };
                let mut cmds = Vec::new();
                for g in run {
                    let mut one = Vec::new();
                    if !outlines.outline(face, g.id, *size, &mut one) {
                        report.glyphs_undrawn = report.glyphs_undrawn.saturating_add(1);
                        continue;
                    }
                    cmds.extend(one.into_iter().map(|c| offset(c, g.x, g.y)));
                }
                if let Some(p) = build_path(&cmds) {
                    let t = sk_transform(&scale.then_after(transform));
                    draw(
                        &mut pixmap,
                        &p,
                        fill,
                        None,
                        t,
                        &mut pictures,
                        &mut report.images,
                    );
                }
            }
        }
    }
    Ok((pixmap, report))
}

fn offset(cmd: PathCmd, dx: f64, dy: f64) -> PathCmd {
    match cmd {
        PathCmd::MoveTo(x, y) => PathCmd::MoveTo(x + dx, y + dy),
        PathCmd::LineTo(x, y) => PathCmd::LineTo(x + dx, y + dy),
        PathCmd::QuadTo(a, b, x, y) => PathCmd::QuadTo(a + dx, b + dy, x + dx, y + dy),
        PathCmd::CubicTo(a, b, c, d, x, y) => {
            PathCmd::CubicTo(a + dx, b + dy, c + dx, d + dy, x + dx, y + dy)
        }
        PathCmd::Close => PathCmd::Close,
    }
}

/// Execute a display list and encode the surface as PNG.
pub fn render_png(
    list: &DisplayList,
    dpi: f64,
    images: &dyn ImageSource,
    glyphs: Option<&dyn GlyphOutlines>,
) -> Result<(Vec<u8>, RenderReport), RenderError> {
    let (pixmap, report) = render_pixmap(list, dpi, images, glyphs)?;
    let png = pixmap.encode_png().map_err(|_| RenderError::Encode)?;
    Ok((png, report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::{Glyph, Layer};
    use crate::images::NoImages;

    const RED: Rgba = Rgba {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };

    fn two_by_one_inch(items: Vec<DisplayItem>) -> DisplayList {
        DisplayList {
            width: EMU_PER_INCH * 2.0,
            height: EMU_PER_INCH,
            items,
        }
    }

    fn square(transform: Affine) -> DisplayItem {
        let s = EMU_PER_INCH / 2.0;
        DisplayItem::Path {
            object: None,
            layer: Layer::Slide,
            transform,
            path: vec![
                PathCmd::MoveTo(0.0, 0.0),
                PathCmd::LineTo(s, 0.0),
                PathCmd::LineTo(s, s),
                PathCmd::LineTo(0.0, s),
                PathCmd::Close,
            ],
            fill: Some(Paint::Solid(RED)),
            stroke: None,
        }
    }

    fn rgb(p: &Pixmap, x: u32, y: u32) -> (u8, u8, u8) {
        let c = p.pixel(x, y).map(|c| c.demultiply());
        c.map_or((0, 0, 0), |c| (c.red(), c.green(), c.blue()))
    }

    #[test]
    fn a_filled_path_lands_where_its_transform_puts_it() {
        let list = two_by_one_inch(vec![square(Affine::translate(EMU_PER_INCH, 0.0))]);
        let (p, _) = render_pixmap(&list, 100.0, &NoImages, None).unwrap();
        assert_eq!((p.width(), p.height()), (200, 100));
        assert_eq!(rgb(&p, 125, 25), (255, 0, 0));
        assert_eq!(rgb(&p, 25, 25), (255, 255, 255));
        assert_eq!(rgb(&p, 125, 75), (255, 255, 255));
    }

    #[test]
    fn a_glyph_run_with_no_outlines_is_counted_not_dropped() {
        let run = DisplayItem::GlyphRun {
            object: None,
            layer: Layer::Slide,
            transform: Affine::IDENTITY,
            face: "Carlito".to_owned(),
            size: 228_600.0,
            glyphs: vec![Glyph {
                id: 3,
                x: 0.0,
                y: 0.0,
            }],
            fill: Paint::Solid(RED),
            text: String::new(),
        };
        let (_, report) =
            render_pixmap(&two_by_one_inch(vec![run]), 10.0, &NoImages, None).unwrap();
        assert_eq!(report.glyph_runs_undrawn, 1);
    }

    struct OneSquareGlyph;

    impl GlyphOutlines for OneSquareGlyph {
        fn outline(&self, _face: &str, id: u32, size: f64, out: &mut Vec<PathCmd>) -> bool {
            if id != 1 {
                return false;
            }
            out.extend([
                PathCmd::MoveTo(0.0, -size),
                PathCmd::LineTo(size, -size),
                PathCmd::LineTo(size, 0.0),
                PathCmd::LineTo(0.0, 0.0),
                PathCmd::Close,
            ]);
            true
        }
    }

    #[test]
    fn glyph_outlines_are_placed_at_each_glyph_origin() {
        let run = DisplayItem::GlyphRun {
            object: None,
            layer: Layer::Slide,
            transform: Affine::IDENTITY,
            face: "Any".to_owned(),
            size: EMU_PER_INCH / 2.0,
            glyphs: vec![
                Glyph {
                    id: 1,
                    x: EMU_PER_INCH,
                    y: EMU_PER_INCH / 2.0,
                },
                Glyph {
                    id: 2,
                    x: 0.0,
                    y: 0.0,
                },
            ],
            fill: Paint::Solid(RED),
            text: String::new(),
        };
        let (p, report) = render_pixmap(
            &two_by_one_inch(vec![run]),
            100.0,
            &NoImages,
            Some(&OneSquareGlyph),
        )
        .unwrap();
        assert_eq!(rgb(&p, 125, 25), (255, 0, 0));
        assert_eq!(report.glyphs_undrawn, 1);
    }

    #[test]
    fn a_surface_of_no_size_is_an_error_not_a_panic() {
        let list = DisplayList::default();
        let err = render_pixmap(&list, 96.0, &NoImages, None).unwrap_err();
        assert_eq!(err.code(), "CS-RND-0001");
    }
}
