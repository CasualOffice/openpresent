//! Laying out a text body whose values are already resolved: runs to shaped
//! items, items to lines, lines to positioned glyph runs.
//!
//! New here (OpenCalc does not wrap). Pure: the same body and frame give the same
//! runs on every host, because every face is bundled or supplied and nothing is
//! asked of the system.
//!
//! # Rules, and what they rest on
//!
//! | Rule | Evidence |
//! | --- | --- |
//! | A line's height is the face's `usWinAscent` plus `usWinDescent` at its largest run, times the line spacing percentage; points replace it | **Inferred**: PowerPoint's single spacing matches Windows' line metrics; to be measured against `DEC-17`'s PDFs |
//! | Extra height from spacing goes above the baseline | **Inferred** |
//! | `spcBef` and `spcAft` as a percentage are of the first line's single height | **Inferred**: ECMA-376 21.1.2.2.11 says "of the text size" without saying which |
//! | `spcBef` of the first paragraph and `spcAft` of the last apply only with `spcFirstLastPara` | ECMA-376 21.1.2.1.1 |
//! | The first line starts at `marL + indent`, later lines at `marL`; a bullet sits at `marL + indent` and pushes the text to `marL`, or past itself if wider | ECMA-376 21.1.2.2.7; the push is **Inferred** |
//! | Trailing spaces do not count towards fitting or alignment | UAX #14 practice; **Inferred** for PowerPoint |
//! | A word wider than the line breaks between characters | **Inferred** |
//! | `just` and `dist` anchoring are laid out as top | Not yet modelled: `T2D4-01` |
//! | An underline runs under each glyph run, trailing spaces of the line excluded, at the face's `post` underline position and thickness, in the run's colour | ECMA-376 21.1.2.3.9 names the kinds; the metrics are **Inferred** from how PowerPoint draws (`VS-18`) |
//! | A strikethrough runs through each glyph run at the face's `OS/2` strikeout position and thickness, in the run's colour; a double one draws two lines a thickness either side of it | ECMA-376 21.1.2.3.12 names the kinds; the metrics and the double's gap are **Inferred** from how PowerPoint draws (`PAR-67`) |
//! | A run with a `baseline` is raised (positive) or lowered by that share of its size, and drawn and advanced at two thirds of its size | ECMA-376 21.1.2.3.9 gives the offset as a percentage of the font size; the two-thirds size is **Inferred** from how PowerPoint draws superscript, not measured (`PAR-67`) |
//! | `spc` is added to the advance after every character of the run, the last included | ECMA-376 21.1.2.3.9; after the last is **Inferred** (`PAR-67`) |
//! | A highlight fills each glyph run's advance, from the face's ascent above the baseline to its descent below, behind every glyph of the body | ECMA-376 21.1.2.3.4 names `highlight`; the extent is **Inferred** from how PowerPoint draws, not measured (`UX-005`) |
//!
//! Not yet here: tabs, columns, vertical and rotated text, autofit's scale,
//! kinsoku, bidi reordering across runs. Each is a row in `docs/14`.

use skrifa::raw::TableProvider;
use skrifa::{FontRef, MetadataProvider, instance::LocationRef, prelude::Size};

pub use casual_slide_model::text::UnderlineType;

use crate::breaks::{Break, opportunities};
use crate::faces::TextFaces;

/// A rectangle, in points, in whatever space the caller lays out in.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Frame {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

/// 8-bit sRGB with alpha, 255 opaque.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rgba {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// Alpha.
    pub a: u8,
}

impl Rgba {
    /// Opaque black, what text is drawn in when nothing says otherwise.
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
}

/// How one run is drawn.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunStyle {
    /// The requested family, before substitution. `None` is the default family.
    pub family: Option<String>,
    /// Size in points.
    pub size: f32,
    /// Bold.
    pub bold: bool,
    /// Italic.
    pub italic: bool,
    /// Colour.
    pub color: Rgba,
    /// `u`: the underline kind; `None` draws none. [`UnderlineType::None`] is never
    /// stored here: the caller maps it to `None`.
    pub underline: Option<UnderlineType>,
    /// `highlight`: the colour drawn behind the glyphs; `None` draws none.
    pub highlight: Option<Rgba>,
    /// `strike`: the strikethrough; `None` draws none.
    pub strike: Option<StrikeKind>,
    /// `baseline`, as a share of the size: `0.3` raises the run by 30% of its size,
    /// a negative value lowers it; `0.0` is the line's own baseline.
    pub baseline: f32,
    /// `spc`: extra advance after each character, points; negative tightens.
    pub spacing: f32,
    /// The run's East Asian typeface (`ea`, theme references followed) when it draws
    /// in another: its height still counts toward the line's, as PowerPoint makes an
    /// East Asian font taller than the Latin one open the lines of Latin text too.
    /// `None` when the run states none or draws in it.
    pub east_asian_family: Option<String>,
}

/// The share of its size a raised or lowered run is drawn at: **Inferred** from how
/// PowerPoint draws superscript and subscript (`PAR-67`).
pub const SCRIPT_SCALE: f32 = 2.0 / 3.0;

impl RunStyle {
    /// The size the glyphs are drawn and advanced at: smaller when raised or lowered.
    #[must_use]
    pub fn drawn_size(&self) -> f32 {
        if self.baseline == 0.0 {
            self.size
        } else {
            self.size * SCRIPT_SCALE
        }
    }

    /// How far the run's baseline is raised above the line's, points, `y` up.
    #[must_use]
    pub fn rise(&self) -> f32 {
        self.baseline * self.size
    }
}

/// `a:ST_TextStrikeType`, less `noStrike`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrikeKind {
    /// `sngStrike`: one line.
    Single,
    /// `dblStrike`: two lines.
    Double,
}

/// Text with one style. A line break inside a paragraph is a `\n` in `text`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StyledText {
    /// The characters.
    pub text: String,
    /// How they are drawn.
    pub style: RunStyle,
}

/// Horizontal alignment of a paragraph's lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    /// `l`.
    #[default]
    Left,
    /// `ctr`.
    Center,
    /// `r`.
    Right,
    /// `just`, `justLow`, `dist`, `thaiDist`: space is spread over every line but the last.
    Justified,
}

/// Line or paragraph spacing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spacing {
    /// A multiple of single spacing, `1.0` for 100%.
    Percent(f32),
    /// An absolute amount, in points.
    Points(f32),
}

impl Default for Spacing {
    fn default() -> Self {
        Self::Percent(1.0)
    }
}

/// One paragraph, resolved.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParagraphLayout {
    /// Alignment.
    pub align: Align,
    /// `marL`, points.
    pub margin_left: f32,
    /// `marR`, points.
    pub margin_right: f32,
    /// `indent`, points; negative hangs the first line.
    pub indent: f32,
    /// `lnSpc`.
    pub line_spacing: Spacing,
    /// `spcBef`.
    pub space_before: Spacing,
    /// `spcAft`.
    pub space_after: Spacing,
    /// The bullet, as the text it draws.
    pub bullet: Option<StyledText>,
    /// The runs, in order.
    pub runs: Vec<StyledText>,
    /// The paragraph mark's style, which sizes an empty paragraph.
    pub end: RunStyle,
}

/// Vertical anchoring of the text in its frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    /// `t`.
    #[default]
    Top,
    /// `ctr`.
    Middle,
    /// `b`.
    Bottom,
}

/// Distances from the frame's edges to the text, in points.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Insets {
    /// `lIns`.
    pub left: f32,
    /// `tIns`.
    pub top: f32,
    /// `rIns`.
    pub right: f32,
    /// `bIns`.
    pub bottom: f32,
}

/// A whole body, resolved.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BodyLayout {
    /// Insets.
    pub insets: Insets,
    /// `wrap="square"`. False: lines end only where the text says.
    pub wrap: bool,
    /// Vertical anchoring.
    pub anchor: Anchor,
    /// `spcFirstLastPara`.
    pub space_first_last: bool,
    /// The paragraphs.
    pub paragraphs: Vec<ParagraphLayout>,
}

/// One glyph, at its origin on the baseline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glyph {
    /// Glyph id in the run's face.
    pub id: u16,
    /// Pen position, points.
    pub x: f32,
    /// Baseline, points, `y` down.
    pub y: f32,
}

/// Glyphs in one face, size and colour, ready to outline and fill.
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRun {
    /// The face's bytes, for `skrifa` to outline. Bundled or supplied, so `'static`.
    pub face: &'static [u8],
    /// The face's stable name, for a display list: [`crate::outline::face_by_key`]
    /// finds the bytes again.
    pub face_key: String,
    /// Size in points.
    pub size: f32,
    /// Fill colour.
    pub color: Rgba,
    /// Bold was asked for and the face has none: embolden when drawing.
    pub synthetic_bold: bool,
    /// The glyphs, in drawing order.
    pub glyphs: Vec<Glyph>,
    /// The characters these glyphs draw, for text extraction. Document data.
    pub text: String,
    /// The run's underline, if it has one.
    pub underline: Option<Underline>,
    /// The run's highlight, if it has one.
    pub highlight: Option<Highlight>,
    /// The run's strikethrough, if it has one.
    pub strike: Option<Strikethrough>,
}

/// Where a strikethrough is drawn through one [`GlyphRun`], points, in the layout's
/// space (`PAR-67`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strikethrough {
    /// One line or two.
    pub kind: StrikeKind,
    /// Start, at the first glyph's pen position.
    pub x0: f32,
    /// End, after the last glyph's advance.
    pub x1: f32,
    /// The centre of the single line, `y` down: above the run's baseline by the
    /// face's strikeout position. A double one draws a thickness either side.
    pub y: f32,
    /// The face's strikeout thickness at the run's drawn size.
    pub thickness: f32,
}

/// The box a run's highlight fills, behind its glyphs, points, in the layout's space:
/// from the first glyph's pen position to the last one's advance, and from the face's
/// ascent above the baseline to its descent below (`UX-005`). PowerPoint's own extent
/// is **Inferred** from how it draws, not measured.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Highlight {
    /// The colour.
    pub color: Rgba,
    /// Start.
    pub x0: f32,
    /// End.
    pub x1: f32,
    /// Top, `y` down.
    pub top: f32,
    /// Bottom.
    pub bottom: f32,
}

/// Where an underline is drawn under one [`GlyphRun`], points, in the layout's space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Underline {
    /// The kind, as the run asks; never [`UnderlineType::None`].
    pub kind: UnderlineType,
    /// Start, at the first glyph's pen position.
    pub x0: f32,
    /// End, after the last glyph's advance.
    pub x1: f32,
    /// The centre of the line, `y` down: below the baseline by the face's
    /// underline position.
    pub y: f32,
    /// The face's underline thickness at the run's size.
    pub thickness: f32,
}

/// What laying out a body produced.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextLayout {
    /// The glyph runs, in reading order.
    pub runs: Vec<GlyphRun>,
    /// Height of the laid-out text, points, without insets.
    pub content_height: f32,
    /// Lines laid out.
    pub lines: usize,
    /// Each requested family that was drawn with another, once, in order of first use.
    pub substitutions: Vec<FontSubstitution>,
    /// Scripts in the text that no available face can draw.
    pub missing_scripts: Vec<crate::faces::MissingScript>,
    /// Where a caret can stand on each line, in reading order: what an editor
    /// places a caret and hit-tests with (`VS-05`).
    pub carets: Vec<CaretLine>,
}

/// One laid-out line's caret stops, in the frame's space, points.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CaretLine {
    /// The paragraph the line is in, from 0.
    pub paragraph: usize,
    /// The top of the line: its baseline less its ascent.
    pub top: f32,
    /// The bottom: its baseline plus its descent.
    pub bottom: f32,
    /// `(offset, x)`: each place a caret can stand, as a character offset into the
    /// paragraph's own text (a line break one character, a bullet none), and the x of
    /// the caret there. In offset order; the first is where the line starts.
    pub stops: Vec<(usize, f32)>,
}

/// A requested family drawn with a bundled one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontSubstitution {
    /// The family the file asked for.
    pub requested: String,
    /// The bundled family drawn instead.
    pub used: &'static str,
    /// How close the match is.
    pub kind: crate::substitution::SubstituteKind,
}

/// Line metrics of a face at a size, in points.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Metrics {
    ascent: f32,
    descent: f32,
}

/// A run's line metrics: its own substituted face's, and its East Asian face's where
/// that is taller, whichever is greater on each side.
fn style_metrics(style: &RunStyle) -> Metrics {
    let face = crate::faces::face_bytes_for(style.family.as_deref(), style.bold, style.italic);
    let own = metrics(face, style.size);
    let Some(ea) = style.east_asian_family.as_deref() else {
        return own;
    };
    let other = metrics(
        crate::faces::face_bytes_for(Some(ea), style.bold, style.italic),
        style.size,
    );
    Metrics {
        ascent: own.ascent.max(other.ascent),
        descent: own.descent.max(other.descent),
    }
}

fn metrics(face: &[u8], size: f32) -> Metrics {
    let fallback = Metrics {
        ascent: size * 0.9,
        descent: size * 0.25,
    };
    let Ok(font) = FontRef::new(face) else {
        return fallback;
    };
    let upem = font
        .head()
        .map(|h| f32::from(h.units_per_em()))
        .unwrap_or(0.0);
    if let (Ok(os2), true) = (font.os2(), upem > 0.0) {
        let ascent = f32::from(os2.us_win_ascent());
        let descent = f32::from(os2.us_win_descent());
        if ascent + descent > 0.0 {
            return Metrics {
                ascent: ascent * size / upem,
                descent: descent * size / upem,
            };
        }
    }
    let m = font.metrics(Size::new(size), LocationRef::default());
    if m.ascent - m.descent > 0.0 {
        Metrics {
            ascent: m.ascent,
            descent: -m.descent,
        }
    } else {
        fallback
    }
}

/// The face's underline offset below the baseline (positive down) and its thickness,
/// in points at `size`, from `post`; a tenth and a twentieth of the size when the face
/// does not say.
fn underline_metrics(face: &[u8], size: f32) -> (f32, f32) {
    let fallback = (size * 0.1, size * 0.05);
    let Ok(font) = FontRef::new(face) else {
        return fallback;
    };
    match font
        .metrics(Size::new(size), LocationRef::default())
        .underline
    {
        Some(d) if d.thickness > 0.0 && d.thickness.is_finite() && d.offset.is_finite() => {
            // `post` gives the top of the line, y up; the centre is half a thickness below.
            (-d.offset + d.thickness / 2.0, d.thickness)
        }
        _ => fallback,
    }
}

/// The face's strikeout centre above the baseline (positive up) and its thickness, in
/// points at `size`, from `OS/2`; 0.3 and a twentieth of the size when the face does
/// not say.
fn strike_metrics(face: &[u8], size: f32) -> (f32, f32) {
    let fallback = (size * 0.3, size * 0.05);
    let Ok(font) = FontRef::new(face) else {
        return fallback;
    };
    match font
        .metrics(Size::new(size), LocationRef::default())
        .strikeout
    {
        Some(d) if d.thickness > 0.0 && d.thickness.is_finite() && d.offset.is_finite() => {
            // `OS/2` gives the top of the line, y up; the centre is half a thickness below.
            (d.offset - d.thickness / 2.0, d.thickness)
        }
        _ => fallback,
    }
}

/// One shaped glyph, with its cluster as an offset into the paragraph's text.
#[derive(Debug, Clone, Copy)]
struct Shaped {
    id: u16,
    cluster: usize,
    advance: f32,
    x_offset: f32,
    y_offset: f32,
}

/// Text in one face and style.
#[derive(Debug, Clone)]
struct Item {
    start: usize,
    end: usize,
    face: Option<&'static [u8]>,
    style: usize,
    synthetic_bold: bool,
    glyphs: Vec<Shaped>,
}

/// A paragraph's text, its styles, and its items, the bullet first if any.
struct Prepared<'a> {
    text: String,
    styles: Vec<&'a RunStyle>,
    items: Vec<Item>,
    /// Byte length of the bullet's text at the start of `text`; zero without one.
    bullet_len: usize,
}

fn same_face(a: Option<&'static [u8]>, b: Option<&'static [u8]>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => core::ptr::eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

fn prepare(paragraph: &ParagraphLayout) -> Prepared<'_> {
    let mut prepared = Prepared {
        text: String::new(),
        styles: Vec::new(),
        items: Vec::new(),
        bullet_len: 0,
    };
    let bullet = paragraph.bullet.iter().filter(|b| !b.text.is_empty());
    let has_text = paragraph.runs.iter().any(|r| !r.text.is_empty());
    let runs = bullet
        .clone()
        .filter(|_| has_text)
        .chain(paragraph.runs.iter());
    for (index, run) in runs.enumerate() {
        let base = prepared.text.len();
        prepared.text.push_str(&run.text);
        if index == 0
            && has_text
            && paragraph
                .bullet
                .as_ref()
                .is_some_and(|b| !b.text.is_empty())
        {
            prepared.bullet_len = run.text.len();
        }
        let style = prepared.styles.len();
        prepared.styles.push(&run.style);
        let faces = TextFaces::new(
            run.style.family.as_deref(),
            run.style.bold,
            run.style.italic,
            &run.text,
        );
        // Split the run where the face drawing it changes.
        let mut segment: Option<(usize, Option<&'static [u8]>, bool)> = None;
        let mut close = |from: usize, to: usize, face: Option<&'static [u8]>, synth: bool| {
            if to > from {
                prepared.items.push(Item {
                    start: base + from,
                    end: base + to,
                    face,
                    style,
                    synthetic_bold: synth,
                    glyphs: Vec::new(),
                });
            }
        };
        for (offset, ch) in run.text.char_indices() {
            let resolved = faces.resolve(ch);
            // A line feed is never drawn; it stands alone so it shapes to nothing.
            let face = if ch == '\n' { None } else { resolved.face };
            let breaks_segment = ch == '\n' || segment.is_some_and(|(_, f, _)| !same_face(f, face));
            if breaks_segment && let Some((from, f, s)) = segment.take() {
                close(from, offset, f, s);
            }
            if segment.is_none() {
                segment = Some((offset, face, resolved.synthetic_bold));
            }
            if ch == '\n'
                && let Some((from, f, s)) = segment.take()
            {
                close(from, offset + ch.len_utf8(), f, s);
            }
        }
        if let Some((from, f, s)) = segment.take() {
            close(from, run.text.len(), f, s);
        }
    }
    for item in &mut prepared.items {
        let Some(style) = prepared.styles.get(item.style) else {
            continue;
        };
        let Some(text) = prepared.text.get(item.start..item.end) else {
            continue;
        };
        let size = style.drawn_size();
        item.glyphs = match item.face {
            Some(face) => crate::shape::run(face, text, size)
                .unwrap_or_default()
                .into_iter()
                .map(|g| Shaped {
                    id: g.id,
                    cluster: item.start.saturating_add(g.cluster as usize),
                    advance: g.advance,
                    x_offset: g.x_offset,
                    y_offset: g.y_offset,
                })
                .collect(),
            // Nothing can draw it: it advances by the width it would have, and
            // draws nothing, so the rest of the line is where it should be.
            None => text
                .char_indices()
                .filter(|&(_, ch)| ch != '\n')
                .map(|(offset, ch)| Shaped {
                    id: 0,
                    cluster: item.start.saturating_add(offset),
                    advance: crate::char_advance(
                        &TextFaces::new(style.family.as_deref(), style.bold, style.italic, text)
                            .resolve(ch),
                        ch,
                        size,
                    ),
                    x_offset: 0.0,
                    y_offset: 0.0,
                })
                .collect(),
        };
        // `spc`: extra advance after each character, on the last glyph of its cluster
        // so a mark stays over its base.
        if style.spacing != 0.0 {
            let clusters: Vec<usize> = item.glyphs.iter().map(|g| g.cluster).collect();
            for (i, glyph) in item.glyphs.iter_mut().enumerate() {
                if clusters.get(i + 1) != Some(&glyph.cluster) {
                    glyph.advance += style.spacing;
                }
            }
        }
    }
    prepared
}

/// Advance of the text before each byte offset: `prefix[i]` is the width of the
/// glyphs whose cluster is before `i`.
fn prefix_widths(prepared: &Prepared<'_>) -> Vec<f32> {
    let len = prepared.text.len();
    let mut at = vec![0.0f32; len.saturating_add(1)];
    for glyph in prepared.items.iter().flat_map(|i| i.glyphs.iter()) {
        if let Some(slot) = at.get_mut(glyph.cluster.saturating_add(1)) {
            *slot += glyph.advance;
        }
    }
    let mut total = 0.0;
    for slot in &mut at {
        total += *slot;
        *slot = total;
    }
    at
}

fn width(prefix: &[f32], from: usize, to: usize) -> f32 {
    let a = prefix.get(from).copied().unwrap_or(0.0);
    let b = prefix.get(to).copied().unwrap_or(a);
    (b - a).max(0.0)
}

/// `to`, moved back over trailing white space and line feeds.
fn trim_end(text: &str, from: usize, to: usize) -> usize {
    text.get(from..to)
        .map_or(to, |s| from + s.trim_end_matches(char::is_whitespace).len())
}

/// Line spans `[start, end)` for a paragraph, greedy, first line `first` wide and
/// the rest `rest` wide. `None` for either is unbounded.
fn break_lines(
    text: &str,
    prefix: &[f32],
    skip: usize,
    first: Option<f32>,
    rest: Option<f32>,
) -> Vec<(usize, usize)> {
    const EPS: f32 = 0.01;
    let mut lines = Vec::new();
    let mut start = skip;
    let mut candidate: Option<usize> = None;
    let avail = |lines: &Vec<(usize, usize)>| if lines.is_empty() { first } else { rest };
    let fits = |lines: &Vec<(usize, usize)>, from: usize, to: usize| {
        avail(lines).is_none_or(|w| width(prefix, from, trim_end(text, from, to)) <= w + EPS)
    };
    for Break { at, mandatory } in opportunities(text) {
        if at <= start {
            continue;
        }
        if !fits(&lines, start, at) {
            if let Some(c) = candidate.take() {
                lines.push((start, c));
                start = c;
            }
            // One word wider than the line: break it between characters.
            while !fits(&lines, start, at) {
                let limit = avail(&lines).unwrap_or(f32::INFINITY);
                let mut end = start;
                for (offset, ch) in text.get(start..at).unwrap_or("").char_indices() {
                    let next = start + offset + ch.len_utf8();
                    if end > start
                        && width(prefix, start, trim_end(text, start, next)) > limit + EPS
                    {
                        break;
                    }
                    end = next;
                }
                if end >= at || end <= start {
                    break;
                }
                lines.push((start, end));
                start = end;
            }
        }
        if mandatory {
            lines.push((start, at));
            start = at;
            candidate = None;
        } else {
            candidate = Some(at);
        }
    }
    if lines.is_empty() {
        lines.push((skip, text.len()));
    } else if start == text.len() && text.ends_with('\n') {
        // A break ending the paragraph opens an empty last line, sized by the
        // paragraph mark, as PowerPoint draws it.
        lines.push((start, start));
    }
    lines
}

/// Lay out `body` in `frame`.
#[must_use]
pub fn layout_body(body: &BodyLayout, frame: Frame) -> TextLayout {
    let insets = body.insets;
    let left = frame.x + insets.left;
    let inner_width = (frame.width - insets.left - insets.right).max(0.0);
    let content_height = (frame.height - insets.top - insets.bottom).max(0.0);
    let top = frame.y + insets.top;

    let mut runs: Vec<GlyphRun> = Vec::new();
    let mut carets: Vec<CaretLine> = Vec::new();
    let mut y = 0.0f32;
    let mut lines_total = 0usize;
    let count = body.paragraphs.len();

    for (index, paragraph) in body.paragraphs.iter().enumerate() {
        let prepared = prepare(paragraph);
        let prefix = prefix_widths(&prepared);
        let bullet_width = width(&prefix, 0, prepared.bullet_len);
        let first_x = paragraph.margin_left + paragraph.indent;
        let text_first_x = if prepared.bullet_len > 0 {
            if paragraph.indent < 0.0 && first_x + bullet_width <= paragraph.margin_left {
                paragraph.margin_left
            } else {
                first_x + bullet_width
            }
        } else {
            first_x
        };
        let right = inner_width - paragraph.margin_right;
        let (first_w, rest_w) = if body.wrap {
            (
                Some((right - text_first_x).max(0.0)),
                Some((right - paragraph.margin_left).max(0.0)),
            )
        } else {
            (None, None)
        };
        let spans = break_lines(
            &prepared.text,
            &prefix,
            prepared.bullet_len,
            first_w,
            rest_w,
        );

        let end_metrics = style_metrics(&paragraph.end);
        let mut first_single = None;
        let mut lines_y = Vec::with_capacity(spans.len());
        let mut lines_m = Vec::with_capacity(spans.len());
        let mut height = 0.0f32;
        for (line_index, &(start, end)) in spans.iter().enumerate() {
            let from = if line_index == 0 { 0 } else { start };
            let mut m: Option<Metrics> = None;
            for item in &prepared.items {
                if item.end <= from || item.start >= end.max(from + 1) {
                    continue;
                }
                let Some(style) = prepared.styles.get(item.style) else {
                    continue;
                };
                // A line's metrics come from the run's own substituted face, not
                // the fallback that draws a character it lacks, as OpenCalc's
                // `FaceSelector::primary` gives a cell's (owner instruction 42).
                let im = style_metrics(style);
                m = Some(m.map_or(im, |m| Metrics {
                    ascent: m.ascent.max(im.ascent),
                    descent: m.descent.max(im.descent),
                }));
            }
            let m = m.unwrap_or(end_metrics);
            let single = m.ascent + m.descent;
            first_single.get_or_insert(single);
            let line_height = match paragraph.line_spacing {
                Spacing::Percent(p) => single * p,
                Spacing::Points(p) => p,
            };
            height += line_height;
            lines_y.push((height - m.descent, line_index));
            lines_m.push(m);
        }
        let single = first_single.unwrap_or(end_metrics.ascent + end_metrics.descent);
        let amount = |s: Spacing| match s {
            Spacing::Percent(p) => single * p,
            Spacing::Points(p) => p,
        };
        if index > 0 || body.space_first_last {
            y += amount(paragraph.space_before);
        }

        for ((&(start, end), &(baseline, line_index)), m) in
            spans.iter().zip(&lines_y).zip(&lines_m)
        {
            let baseline = y + baseline;
            let trimmed = trim_end(&prepared.text, start, end);
            let line_w = width(&prefix, start, trimmed);
            let (line_x, avail) = if line_index == 0 {
                (text_first_x, right - text_first_x)
            } else {
                (paragraph.margin_left, right - paragraph.margin_left)
            };
            let is_last = line_index + 1 == spans.len();
            let slack = avail - line_w;
            let (mut pen, gap_extra) = match paragraph.align {
                Align::Left => (line_x, 0.0),
                Align::Center => (line_x + slack / 2.0, 0.0),
                Align::Right => (line_x + slack, 0.0),
                Align::Justified => {
                    let gaps = prepared
                        .text
                        .get(start..trimmed)
                        .map_or(0, |s| s.chars().filter(|c| *c == ' ').count());
                    if is_last || gaps == 0 || !body.wrap || slack <= 0.0 {
                        (line_x, 0.0)
                    } else {
                        (line_x, slack / gaps as f32)
                    }
                }
            };
            let baseline = top + baseline;
            // The bullet, on the first line only, at the first line's indent.
            if line_index == 0 && prepared.bullet_len > 0 {
                let mut bullet_pen = first_x;
                let at = LinePlace {
                    origin_x: left,
                    baseline,
                    gap_extra: 0.0,
                };
                emit(
                    &prepared,
                    (0, prepared.bullet_len),
                    at,
                    &mut bullet_pen,
                    &mut runs,
                );
            }
            // Caret stops: every character boundary on the line, the trailing space
            // included and a line feed's own end not, since that is the next line's.
            let line_end =
                if prepared.text.get(..end).is_some_and(|t| t.ends_with('\n')) && end > start {
                    end - 1
                } else {
                    end
                };
            let caret_from = start.max(prepared.bullet_len);
            let text_tail = prepared.text.get(caret_from..line_end).unwrap_or("");
            let mut chars_before = prepared
                .text
                .get(prepared.bullet_len..caret_from)
                .map_or(0, |t| t.chars().count());
            let mut stops = Vec::with_capacity(text_tail.len().saturating_add(1));
            let pen_x = left + pen;
            stops.push((chars_before, pen_x));
            for (offset, ch) in text_tail.char_indices() {
                let after = caret_from + offset + ch.len_utf8();
                chars_before += 1;
                stops.push((chars_before, pen_x + width(&prefix, caret_from, after)));
            }
            carets.push(CaretLine {
                paragraph: index,
                top: baseline - m.ascent,
                bottom: baseline + m.descent,
                stops,
            });
            let at = LinePlace {
                origin_x: left,
                baseline,
                gap_extra,
            };
            emit(&prepared, (start, trimmed), at, &mut pen, &mut runs);
            lines_total += 1;
        }
        y += height;
        if index + 1 < count || body.space_first_last {
            y += amount(paragraph.space_after);
        }
    }

    let offset = match body.anchor {
        Anchor::Top => 0.0,
        Anchor::Middle => (content_height - y) / 2.0,
        Anchor::Bottom => content_height - y,
    };
    if offset != 0.0 {
        for glyph in runs.iter_mut().flat_map(|r| r.glyphs.iter_mut()) {
            glyph.y += offset;
        }
        for line in runs.iter_mut().filter_map(|r| r.underline.as_mut()) {
            line.y += offset;
        }
        for line in runs.iter_mut().filter_map(|r| r.strike.as_mut()) {
            line.y += offset;
        }
        for mark in runs.iter_mut().filter_map(|r| r.highlight.as_mut()) {
            mark.top += offset;
            mark.bottom += offset;
        }
        for line in &mut carets {
            line.top += offset;
            line.bottom += offset;
        }
    }

    TextLayout {
        runs,
        content_height: y,
        lines: lines_total,
        substitutions: Vec::new(),
        missing_scripts: Vec::new(),
        carets,
    }
}

/// Where one line's glyphs go.
#[derive(Debug, Clone, Copy)]
struct LinePlace {
    /// The x every pen position is measured from.
    origin_x: f32,
    /// The line's baseline, absolute.
    baseline: f32,
    /// Extra advance after each space, for justified lines.
    gap_extra: f32,
}

/// Push the glyph runs for the text in `span`, advancing `pen`.
fn emit(
    prepared: &Prepared<'_>,
    span: (usize, usize),
    at: LinePlace,
    pen: &mut f32,
    out: &mut Vec<GlyphRun>,
) {
    let (from, to) = span;
    for item in &prepared.items {
        if item.end <= from || item.start >= to {
            continue;
        }
        let Some(style) = prepared.styles.get(item.style) else {
            continue;
        };
        let mut glyphs = Vec::new();
        let x_start = at.origin_x + *pen;
        // A raised or lowered run sits on its own baseline.
        let run_baseline = at.baseline - style.rise();
        let size = style.drawn_size();
        for g in item
            .glyphs
            .iter()
            .filter(|g| g.cluster >= from && g.cluster < to)
        {
            if item.face.is_some() {
                glyphs.push(Glyph {
                    id: g.id,
                    x: at.origin_x + *pen + g.x_offset,
                    y: run_baseline - g.y_offset,
                });
            }
            *pen += g.advance;
            if at.gap_extra != 0.0
                && prepared
                    .text
                    .get(g.cluster..)
                    .is_some_and(|s| s.starts_with(' '))
            {
                *pen += at.gap_extra;
            }
        }
        if let (Some(face), false) = (item.face, glyphs.is_empty()) {
            let underline = style.underline.map(|kind| {
                let (below, thickness) = underline_metrics(face, size);
                Underline {
                    kind,
                    x0: x_start,
                    x1: at.origin_x + *pen,
                    y: run_baseline + below,
                    thickness,
                }
            });
            let strike = style.strike.map(|kind| {
                let (above, thickness) = strike_metrics(face, size);
                Strikethrough {
                    kind,
                    x0: x_start,
                    x1: at.origin_x + *pen,
                    y: run_baseline - above,
                    thickness,
                }
            });
            let highlight = style.highlight.map(|color| {
                let m = metrics(face, style.size);
                Highlight {
                    color,
                    x0: x_start,
                    x1: at.origin_x + *pen,
                    top: at.baseline - m.ascent,
                    bottom: at.baseline + m.descent,
                }
            });
            out.push(GlyphRun {
                face,
                face_key: crate::outline::face_key(face),
                size,
                color: style.color,
                synthetic_bold: item.synthetic_bold,
                glyphs,
                text: prepared
                    .text
                    .get(item.start.max(from)..item.end.min(to))
                    .unwrap_or_default()
                    .to_owned(),
                underline,
                highlight,
                strike,
            });
        }
    }
}

#[cfg(test)]
mod tests;
