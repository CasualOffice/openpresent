//! SPK-01 spike text pipeline: not product code.
//!
//! runs -> font fallback -> itemize (bidi, script) -> shape -> line break -> place.
//! Features exist so the size table can price each stage; `layout` is all of them.
//!
//! Known simplifications, stated in docs/64: each paragraph is shaped once and broken
//! on cluster advances, with no re-shape at an `unsafe_to_break` line end; no tabs,
//! bullets, columns, autofit or letter spacing; left or right alignment only.
#![forbid(unsafe_code)]

pub mod corpus;

#[cfg(feature = "outline")]
pub use outline::*;

#[cfg(feature = "outline")]
mod outline {
    use skrifa::instance::{LocationRef, Size};
    use skrifa::outline::{DrawSettings, OutlinePen};
    use skrifa::raw::TableProvider;
    use skrifa::{FontRef, GlyphId, MetadataProvider};
    use std::fmt::Write;

    /// One registered face. Bytes are leaked: a spike registers a handful for the
    /// life of the page, and it keeps `forbid(unsafe_code)` honest.
    pub struct Face {
        pub font: FontRef<'static>,
        pub charmap: skrifa::charmap::Charmap<'static>,
        pub upem: f32,
        pub ascent: f32,
        pub descent: f32,
        pub leading: f32,
        pub outlines: skrifa::outline::OutlineGlyphCollection<'static>,
        #[cfg(feature = "shape")]
        pub shaper: harfrust::Shaper<'static>,
    }

    impl Face {
        /// `None` when the bytes are not a readable face.
        pub fn parse(bytes: Vec<u8>) -> Option<Face> {
            let data: &'static [u8] = Box::leak(bytes.into_boxed_slice());
            let font = FontRef::from_index(data, 0).ok()?;
            let upem = f32::from(font.head().ok()?.units_per_em());
            let m = font.metrics(Size::unscaled(), LocationRef::default());
            Some(Face {
                charmap: font.charmap(),
                outlines: font.outline_glyphs(),
                upem,
                ascent: m.ascent,
                descent: m.descent,
                leading: m.leading,
                #[cfg(feature = "shape")]
                shaper: {
                    let sd: &'static harfrust::ShaperData =
                        Box::leak(Box::new(harfrust::ShaperData::new(&font)));
                    sd.shaper(&font).build()
                },
                font,
            })
        }

        /// SVG path data in font units, y up. The painter flips and scales by transform.
        #[must_use]
        pub fn glyph_svg(&self, gid: u32) -> String {
            let mut pen = SvgPen(String::with_capacity(128));
            if let Some(g) = self.outlines.get(GlyphId::new(gid)) {
                let settings = DrawSettings::unhinted(Size::unscaled(), LocationRef::default());
                let _ = g.draw(settings, &mut pen);
            }
            pen.0
        }
    }

    struct SvgPen(String);
    impl OutlinePen for SvgPen {
        fn move_to(&mut self, x: f32, y: f32) {
            let _ = write!(self.0, "M{x} {y}");
        }
        fn line_to(&mut self, x: f32, y: f32) {
            let _ = write!(self.0, "L{x} {y}");
        }
        fn quad_to(&mut self, a: f32, b: f32, x: f32, y: f32) {
            let _ = write!(self.0, "Q{a} {b} {x} {y}");
        }
        fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
            let _ = write!(self.0, "C{a} {b} {c} {d} {x} {y}");
        }
        fn close(&mut self) {
            self.0.push('Z');
        }
    }
}

/// Plain shaping with no plan cache: the shaper's price alone, for the size ladder.
#[cfg(feature = "shape")]
#[must_use]
pub fn ladder_shape(face: &Face, text: &str) -> Vec<u32> {
    let mut buf = harfrust::UnicodeBuffer::new();
    buf.push_str(text);
    buf.guess_segment_properties();
    let out = face.shaper.shape(buf, harfrust::ShapeOptions::new());
    out.glyph_infos().iter().map(|g| g.glyph_id).collect()
}

/// Bidi levels and script runs alone, for the size ladder.
#[cfg(feature = "itemize")]
#[must_use]
pub fn ladder_itemize(text: &str) -> usize {
    use unicode_script::UnicodeScript;
    let info = unicode_bidi::BidiInfo::new(text, None);
    let scripts = text.chars().map(|c| c.script()).collect::<Vec<_>>();
    info.levels.len() + scripts.windows(2).filter(|w| w[0] != w[1]).count()
}

/// (byte offset after which a break is allowed, mandatory?)
#[cfg(feature = "lb-uax14")]
#[must_use]
pub fn line_breaks(text: &str) -> Vec<(usize, bool)> {
    unicode_linebreak::linebreaks(text)
        .map(|(i, o)| (i, o == unicode_linebreak::BreakOpportunity::Mandatory))
        .collect()
}

#[cfg(all(feature = "lb-icu", not(feature = "lb-uax14")))]
#[must_use]
pub fn line_breaks(text: &str) -> Vec<(usize, bool)> {
    use icu_segmenter::LineSegmenter;
    #[cfg(feature = "lb-icu-auto")]
    let seg = LineSegmenter::new_auto(Default::default());
    #[cfg(not(feature = "lb-icu-auto"))]
    let seg = LineSegmenter::new_for_non_complex_scripts(Default::default());
    seg.segment_str(text)
        .filter(|&i| i > 0)
        .map(|i| {
            let mand = i == text.len()
                || matches!(
                    text[..i].chars().next_back(),
                    Some('\n' | '\r' | '\u{2028}' | '\u{2029}')
                );
            (i, mand)
        })
        .collect()
}

#[cfg(all(feature = "layout", any(feature = "lb-uax14", feature = "lb-icu")))]
pub use pipeline::*;

#[cfg(all(feature = "layout", any(feature = "lb-uax14", feature = "lb-icu")))]
mod pipeline {
    use super::{Face, line_breaks};
    use harfrust::{Direction, ShapeOptions, ShapePlan, Tag, UnicodeBuffer};
    use std::collections::HashMap;
    use std::ops::Range;
    use unicode_bidi::BidiInfo;
    use unicode_script::{Script as Us, UnicodeScript};

    #[derive(Debug, Clone)]
    pub struct Run {
        pub start: usize,
        pub end: usize,
        pub level: u8,
        pub script: Us,
        pub face: usize,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Glyph {
        pub gid: u32,
        pub adv: f32,
        pub xo: f32,
        pub yo: f32,
        pub cluster: u32,
    }

    /// Flat, ready for a display list. Glyphs and clusters are in visual order.
    #[derive(Debug, Default, Clone)]
    pub struct Layout {
        pub gids: Vec<u32>,
        pub faces: Vec<u32>,
        pub xs: Vec<f32>,
        pub ys: Vec<f32>,
        pub cl_start: Vec<u32>,
        pub cl_end: Vec<u32>,
        pub cl_x: Vec<f32>,
        pub cl_y: Vec<f32>,
        pub cl_adv: Vec<f32>,
        pub cl_face: Vec<u32>,
        pub cl_rtl: Vec<u8>,
        pub cl_n: Vec<u32>,
        pub line_start: Vec<u32>,
        pub line_end: Vec<u32>,
        pub line_x: Vec<f32>,
        pub line_y: Vec<f32>,
        pub line_w: Vec<f32>,
        pub line_rtl: Vec<u8>,
    }

    pub struct Engine {
        pub faces: Vec<Face>,
        plans: HashMap<(usize, u32, bool), ShapePlan>,
        buf: Option<UnicodeBuffer>,
    }

    impl Default for Engine {
        fn default() -> Self {
            Self::new()
        }
    }

    fn hb_script(s: Us) -> Option<harfrust::Script> {
        let name = s.short_name().as_bytes();
        let tag: [u8; 4] = name.try_into().ok()?;
        harfrust::Script::from_iso15924_tag(Tag::new(&tag))
    }

    fn is_join_control(c: char) -> bool {
        matches!(c, '\u{200C}' | '\u{200D}' | '\u{FE00}'..='\u{FE0F}')
    }

    impl Engine {
        #[must_use]
        pub fn new() -> Self {
            Engine {
                faces: Vec::new(),
                plans: HashMap::new(),
                buf: Some(UnicodeBuffer::new()),
            }
        }

        pub fn register(&mut self, bytes: Vec<u8>) -> Option<usize> {
            self.faces.push(Face::parse(bytes)?);
            Some(self.faces.len() - 1)
        }

        fn has(&self, face: usize, c: char) -> bool {
            self.faces[face].charmap.map(c).is_some()
        }

        /// One paragraph into runs of one level, script and face.
        pub fn itemize(
            &self,
            text: &str,
            para: Range<usize>,
            levels: &[unicode_bidi::Level],
            stack: &[usize],
            out: &mut Vec<Run>,
        ) {
            let mut cur: Option<Run> = None;
            for (i, c) in text[para.clone()].char_indices() {
                let at = para.start + i;
                let level = levels[at].number();
                let sc = c.script();
                let neutral = matches!(sc, Us::Common | Us::Inherited);
                let (script, face) = match &cur {
                    Some(r) if sc == Us::Inherited || is_join_control(c) => (r.script, r.face),
                    Some(r)
                        if (neutral || sc == r.script || r.script == Us::Common)
                            && self.has(r.face, c) =>
                    {
                        (if neutral { r.script } else { sc }, r.face)
                    }
                    _ => {
                        let f = stack
                            .iter()
                            .copied()
                            .find(|&f| self.has(f, c))
                            .unwrap_or(stack[0]);
                        let keep = cur.as_ref().map_or(Us::Common, |r| r.script);
                        (if neutral { keep } else { sc }, f)
                    }
                };
                match &mut cur {
                    Some(r)
                        if r.level == level
                            && r.face == face
                            && (r.script == script || r.script == Us::Common) =>
                    {
                        r.script = script;
                        r.end = at + c.len_utf8();
                    }
                    _ => {
                        if let Some(r) = cur.take() {
                            out.push(r);
                        }
                        cur = Some(Run {
                            start: at,
                            end: at + c.len_utf8(),
                            level,
                            script,
                            face,
                        });
                    }
                }
            }
            if let Some(r) = cur {
                out.push(r);
            }
        }

        /// Appends glyphs in the run's visual order, with `px` positions.
        pub fn shape_run(&mut self, text: &str, run: &Run, size: f32, out: &mut Vec<Glyph>) {
            let rtl = run.level % 2 == 1;
            let dir = if rtl {
                Direction::RightToLeft
            } else {
                Direction::LeftToRight
            };
            let script = hb_script(run.script).unwrap_or(harfrust::script::LATIN);
            let mut buf = self.buf.take().unwrap_or_default();
            for (i, c) in text[run.start..run.end].char_indices() {
                buf.add(c, (run.start + i) as u32);
            }
            buf.set_direction(dir);
            buf.set_script(script);
            let face = &self.faces[run.face];
            let key = (
                run.face,
                u32::from_be_bytes(script.tag().to_be_bytes()),
                rtl,
            );
            let plan = self
                .plans
                .entry(key)
                .or_insert_with(|| ShapePlan::new(&face.shaper, dir, Some(script), None, &[]));
            let shaped = face
                .shaper
                .shape(buf, ShapeOptions::new().plan(Some(&*plan)));
            let k = size / face.upem;
            for (g, p) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
                out.push(Glyph {
                    gid: g.glyph_id,
                    adv: p.x_advance as f32 * k,
                    xo: p.x_offset as f32 * k,
                    yo: p.y_offset as f32 * k,
                    cluster: g.cluster,
                });
            }
            self.buf = Some(shaped.clear());
        }

        /// Stage bench: bidi plus itemization, returns the run count.
        pub fn bench_itemize(&self, text: &str, stack: &[usize]) -> usize {
            let info = BidiInfo::new(text, None);
            let mut runs = Vec::new();
            for p in &info.paragraphs {
                self.itemize(text, p.range.clone(), &info.levels, stack, &mut runs);
            }
            runs.len()
        }

        /// Stage bench: bidi, itemization and shaping, returns the glyph count.
        pub fn bench_shape(&mut self, text: &str, stack: &[usize], size: f32) -> usize {
            let info = BidiInfo::new(text, None);
            let mut runs = Vec::new();
            for p in &info.paragraphs {
                self.itemize(text, p.range.clone(), &info.levels, stack, &mut runs);
            }
            let mut glyphs = Vec::with_capacity(text.len());
            for r in &runs {
                self.shape_run(text, r, size, &mut glyphs);
            }
            glyphs.len()
        }

        pub fn layout(&mut self, text: &str, stack: &[usize], size: f32, width: f32) -> Layout {
            let info = BidiInfo::new(text, None);
            let mut out = Layout::default();
            let first = &self.faces[stack[0]];
            let k = size / first.upem;
            let line_h = (first.ascent - first.descent + first.leading) * k;
            let mut y = first.ascent * k;
            let breaks = line_breaks(text);
            let mut bp = 0usize;
            let mut runs: Vec<Run> = Vec::new();
            let mut glyphs: Vec<Glyph> = Vec::with_capacity(text.len());
            let mut spans: Vec<Range<usize>> = Vec::new();
            let mut clusters: Vec<(u32, f32)> = Vec::new();
            for para in &info.paragraphs {
                runs.clear();
                glyphs.clear();
                spans.clear();
                clusters.clear();
                self.itemize(text, para.range.clone(), &info.levels, stack, &mut runs);
                for r in &runs {
                    let from = glyphs.len();
                    self.shape_run(text, r, size, &mut glyphs);
                    spans.push(from..glyphs.len());
                }
                // Logical cluster advances, for the greedy breaker.
                for (r, sp) in runs.iter().zip(&spans) {
                    let gs = &glyphs[sp.clone()];
                    let mut push = |g: &Glyph| match clusters.last_mut() {
                        Some(l) if l.0 == g.cluster => l.1 += g.adv,
                        _ => clusters.push((g.cluster, g.adv)),
                    };
                    if r.level % 2 == 1 {
                        gs.iter().rev().for_each(&mut push);
                    } else {
                        gs.iter().for_each(&mut push);
                    }
                }
                // Greedy line fill.
                let mut lines: Vec<Range<usize>> = Vec::new();
                let mut start = para.range.start;
                let mut w = 0.0f32;
                let mut last: Option<(usize, f32)> = None;
                for (i, &(cs, adv)) in clusters.iter().enumerate() {
                    let cs = cs as usize;
                    let ce = clusters.get(i + 1).map_or(para.range.end, |c| c.0 as usize);
                    let ws = text[cs..].chars().next().is_some_and(char::is_whitespace);
                    if w + adv > width && !ws {
                        if let Some((b, bw)) = last.take() {
                            lines.push(start..b);
                            start = b;
                            w -= bw;
                        } else if cs > start {
                            lines.push(start..cs);
                            start = cs;
                            w = 0.0;
                        }
                    }
                    w += adv;
                    while bp < breaks.len() && breaks[bp].0 <= ce {
                        if breaks[bp].0 == ce && ce > start {
                            if breaks[bp].1 && ce < para.range.end {
                                lines.push(start..ce);
                                start = ce;
                                w = 0.0;
                                last = None;
                            } else {
                                last = Some((ce, w));
                            }
                        }
                        bp += 1;
                    }
                }
                if start < para.range.end {
                    lines.push(start..para.range.end);
                }
                // Placement, per line, in visual order.
                let base_rtl = para.level.is_rtl();
                for line in lines {
                    let g0 = out.gids.len();
                    let c0 = out.cl_x.len();
                    let mut pen = 0.0f32;
                    let (_, vis) = info.visual_runs(para, line.clone());
                    for vr in vis {
                        let rtl = info.levels[vr.start].is_rtl();
                        let n = runs.len();
                        for o in 0..n {
                            let ri = if rtl { n - 1 - o } else { o };
                            let r = &runs[ri];
                            let s = r.start.max(vr.start);
                            let e = r.end.min(vr.end);
                            if s >= e {
                                continue;
                            }
                            let gs = &glyphs[spans[ri].clone()];
                            let run_rtl = r.level % 2 == 1;
                            // Clusters are monotonic in a run: rising LTR, falling RTL.
                            let (lo, hi) = if run_rtl {
                                (
                                    gs.partition_point(|g| g.cluster as usize >= e),
                                    gs.partition_point(|g| g.cluster as usize >= s),
                                )
                            } else {
                                (
                                    gs.partition_point(|g| (g.cluster as usize) < s),
                                    gs.partition_point(|g| (g.cluster as usize) < e),
                                )
                            };
                            let mut idx = lo;
                            let mut prev_end = e;
                            while idx < hi {
                                let cluster = gs[idx].cluster;
                                let cx = pen;
                                let mut adv = 0.0;
                                let mut j = idx;
                                while j < hi && gs[j].cluster == cluster {
                                    let g = &gs[j];
                                    out.gids.push(g.gid);
                                    out.faces.push(r.face as u32);
                                    out.xs.push(pen + g.xo);
                                    out.ys.push(y - g.yo);
                                    pen += g.adv;
                                    adv += g.adv;
                                    j += 1;
                                }
                                let cend = if run_rtl {
                                    prev_end
                                } else {
                                    gs[j..hi].first().map_or(e, |n| n.cluster as usize)
                                };
                                prev_end = cluster as usize;
                                out.cl_start.push(cluster);
                                out.cl_end.push(cend as u32);
                                out.cl_x.push(cx);
                                out.cl_y.push(y);
                                out.cl_adv.push(adv);
                                out.cl_face.push(r.face as u32);
                                out.cl_rtl.push(u8::from(run_rtl));
                                out.cl_n.push((j - idx) as u32);
                                idx = j;
                            }
                        }
                    }
                    let shift = if base_rtl { width - pen } else { 0.0 };
                    if shift != 0.0 {
                        out.xs[g0..].iter_mut().for_each(|x| *x += shift);
                        out.cl_x[c0..].iter_mut().for_each(|x| *x += shift);
                    }
                    out.line_start.push(line.start as u32);
                    out.line_end.push(line.end as u32);
                    out.line_x.push(shift);
                    out.line_y.push(y);
                    out.line_w.push(pen);
                    out.line_rtl.push(u8::from(base_rtl));
                    y += line_h;
                }
            }
            out
        }
    }

    /// Byte offset to UTF-16 offset, for handing clusters to JavaScript.
    #[must_use]
    pub fn utf16_table(text: &str) -> Vec<u32> {
        let mut t = vec![0u32; text.len() + 1];
        let mut u = 0u32;
        for (i, c) in text.char_indices() {
            t[i] = u;
            u += c.len_utf16() as u32;
        }
        t[text.len()] = u;
        t
    }
}
