//! Text editing in the viewer (`VS-05`): where a click lands in a text box, where the
//! caret and the selection are drawn, and the edits themselves, applied to the deck's
//! model through `casual-slide-transaction` so each can be undone.
//!
//! Positions cross the boundary as a shape (`part`, `node`) and a character offset in
//! its body, as `docs/24` defines them. Geometry crosses in slide EMU. Nothing here
//! carries document text out, except to the host that asked for its own edit.

use casual_slide_model::shape::ObjectId;
use casual_slide_model::text::{
    CharacterProperties, ParagraphProperties, TextBody, TextItem, UnderlineType,
};
use casual_slide_render::display::Affine;
use casual_slide_style::text::TextChain;
use casual_slide_text::CaretLine;
use casual_slide_transaction::{Attribute, Operation, body_length, locate, paragraph_start};

use crate::{BridgeError, Viewer};

/// EMU in a point.
const EMU_PER_POINT: f64 = 12_700.0;

/// Which character attribute a toggle changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Toggle {
    /// Bold, Ctrl+B.
    Bold,
    /// Italic, Ctrl+I.
    Italic,
    /// Underline, Ctrl+U.
    Underline,
}

impl Toggle {
    /// From the bridge's number: 0 bold, 1 italic, 2 underline.
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::Bold),
            1 => Some(Self::Italic),
            2 => Some(Self::Underline),
            _ => None,
        }
    }

    /// Whether the run with own properties `run`, in a paragraph with `paragraph`, has
    /// this attribute as drawn: its own value, else what its levels give it.
    fn is_on(
        self,
        chain: Option<&TextChain<'_>>,
        run: Option<&CharacterProperties>,
        paragraph: Option<&ParagraphProperties>,
    ) -> bool {
        let own = run.and_then(|p| self.own(p));
        if let Some(v) = own {
            return v;
        }
        chain
            .and_then(|c| c.run(None, paragraph, |p| self.own(p)))
            .is_some_and(|r| r.value)
    }

    /// What `p` states of this attribute, if anything.
    fn own(self, p: &CharacterProperties) -> Option<bool> {
        match self {
            Self::Bold => p.bold,
            Self::Italic => p.italic,
            Self::Underline => p.underline.map(|u| u != UnderlineType::None),
        }
    }

    fn attribute(self, on: bool) -> Attribute {
        match self {
            Self::Bold => Attribute::Bold(Some(on)),
            Self::Italic => Attribute::Italic(Some(on)),
            Self::Underline => Attribute::Underline(Some(if on {
                UnderlineType::Sng
            } else {
                UnderlineType::None
            })),
        }
    }
}

/// A caret move that needs the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    /// To the start of the line.
    Home,
    /// To the end of the line.
    End,
    /// To the line above, at the nearest x.
    Up,
    /// To the line below.
    Down,
}

impl Move {
    /// From the bridge's number: 0 home, 1 end, 2 up, 3 down.
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::Home),
            1 => Some(Self::End),
            2 => Some(Self::Up),
            3 => Some(Self::Down),
            _ => None,
        }
    }
}

fn invert(m: &Affine, x: f64, y: f64) -> Option<(f64, f64)> {
    let det = m.a * m.d - m.b * m.c;
    if !det.is_finite() || det.abs() < 1e-12 {
        return None;
    }
    let (x, y) = (x - m.e, y - m.f);
    Some(((m.d * x - m.c * y) / det, (m.a * y - m.b * x) / det))
}

/// A caret line's stops as body offsets.
fn body_stops(body: &TextBody, line: &CaretLine) -> Vec<(usize, f64)> {
    let base = paragraph_start(body, line.paragraph);
    line.stops
        .iter()
        .map(|&(o, x)| (base.saturating_add(o), f64::from(x)))
        .collect()
}

/// The line a body offset is on: the last whose first stop is at or before it, so a
/// caret at a soft wrap stands at the start of the next line, as PowerPoint's does.
fn line_of(body: &TextBody, lines: &[CaretLine], offset: usize) -> Option<usize> {
    let (p, _) = locate(body, offset).ok()?;
    let mut found = None;
    for (index, line) in lines.iter().enumerate() {
        if line.paragraph != p {
            continue;
        }
        let stops = body_stops(body, line);
        if stops.first().is_some_and(|s| s.0 <= offset) {
            found = Some(index);
        }
    }
    found
}

fn x_at(stops: &[(usize, f64)], offset: usize) -> f64 {
    stops
        .iter()
        .find(|s| s.0 >= offset)
        .or(stops.last())
        .map_or(0.0, |s| s.1)
}

fn nearest(stops: &[(usize, f64)], x: f64) -> Option<usize> {
    stops
        .iter()
        .min_by(|a, b| (a.1 - x).abs().total_cmp(&(b.1 - x).abs()))
        .map(|s| s.0)
}

fn item_props(item: &TextItem) -> Option<&CharacterProperties> {
    match item {
        TextItem::Run(r) => r.properties.as_deref(),
        TextItem::Break(b) => b.properties.as_deref(),
        TextItem::Field(f) => f.properties.as_deref(),
        TextItem::Math(_) | TextItem::Retained(_) => None,
    }
}

/// The run's own properties at a body offset: the character after it, or before it at
/// the end of a paragraph, or the paragraph's end properties when it is empty.
/// The properties of the paragraph holding `offset`.
pub(crate) fn paragraph_at(body: &TextBody, offset: usize) -> Option<&ParagraphProperties> {
    let (p, _) = locate(body, offset).ok()?;
    body.paragraphs.get(p)?.properties.as_deref()
}

pub(crate) fn props_at(body: &TextBody, offset: usize) -> Option<&CharacterProperties> {
    let (p, c) = locate(body, offset).ok()?;
    let paragraph = body.paragraphs.get(p)?;
    let mut acc = 0usize;
    let mut last = None;
    for item in &paragraph.items {
        let len = casual_slide_transaction::item_length(item);
        if len > 0 {
            if c < acc + len {
                return item_props(item);
            }
            last = Some(item);
        }
        acc += len;
    }
    match last {
        Some(item) => item_props(item),
        None => paragraph.end_properties.as_deref(),
    }
}

impl Viewer {
    pub(crate) fn body(&self, object: ObjectId) -> Result<&TextBody, BridgeError> {
        self.deck
            .parts()
            .get(object.part)
            .and_then(|p| p.node(object))
            .and_then(|n| n.text_body.as_deref())
            .ok_or(BridgeError::NoText)
    }

    fn caret_lines(
        &self,
        index: usize,
        object: ObjectId,
    ) -> Result<(Affine, Vec<CaretLine>), BridgeError> {
        self.deck
            .slide_text(index, Some(object))?
            .captured
            .ok_or(BridgeError::NoText)
    }

    /// The body's length in characters, a paragraph break counting one.
    pub fn text_length(&self, object: ObjectId) -> Result<usize, BridgeError> {
        Ok(body_length(self.body(object)?))
    }

    /// The text box under a point of slide `index`, in slide EMU, and the offset nearest
    /// it; the topmost box wins. `None` when no text box is there.
    pub fn hit_text(
        &self,
        index: usize,
        x: f64,
        y: f64,
    ) -> Result<Option<(ObjectId, usize)>, BridgeError> {
        let text = self.deck.slide_text(index, None)?;
        for b in text.boxes.iter().rev() {
            let Some((lx, ly)) = invert(&b.transform, x, y) else {
                continue;
            };
            let (l, t, r, bottom) = b.rect;
            if lx < l.min(r) || lx > l.max(r) || ly < t.min(bottom) || ly > t.max(bottom) {
                continue;
            }
            return Ok(Some((b.object, self.offset_near(index, b.object, x, y)?)));
        }
        Ok(None)
    }

    /// The offset in `object`'s body nearest a point in slide EMU: the line nearest in
    /// y, then the stop nearest in x. Used for a click and for extending a selection.
    pub fn offset_near(
        &self,
        index: usize,
        object: ObjectId,
        x: f64,
        y: f64,
    ) -> Result<usize, BridgeError> {
        let (m, lines) = self.caret_lines(index, object)?;
        let body = self.body(object)?;
        let Some((lx, ly)) = invert(&m, x, y) else {
            return Ok(0);
        };
        let (px, py) = (lx / EMU_PER_POINT, ly / EMU_PER_POINT);
        let distance = |l: &CaretLine| {
            let (t, b) = (f64::from(l.top), f64::from(l.bottom));
            if py < t {
                t - py
            } else if py > b {
                py - b
            } else {
                0.0
            }
        };
        let line = lines
            .iter()
            .min_by(|a, b| distance(a).total_cmp(&distance(b)));
        Ok(line
            .and_then(|l| nearest(&body_stops(body, l), px))
            .unwrap_or(0))
    }

    /// The caret at `offset`, as a line from its top to its bottom in slide EMU:
    /// `[x1, y1, x2, y2]`.
    pub fn caret_rect(
        &self,
        index: usize,
        object: ObjectId,
        offset: usize,
    ) -> Result<[f64; 4], BridgeError> {
        let (m, lines) = self.caret_lines(index, object)?;
        let body = self.body(object)?;
        let line = line_of(body, &lines, offset)
            .and_then(|i| lines.get(i))
            .ok_or(BridgeError::NoText)?;
        let x = x_at(&body_stops(body, line), offset) * EMU_PER_POINT;
        let (x1, y1) = m.apply(x, f64::from(line.top) * EMU_PER_POINT);
        let (x2, y2) = m.apply(x, f64::from(line.bottom) * EMU_PER_POINT);
        Ok([x1, y1, x2, y2])
    }

    /// The selection `[start, end)` as one quadrilateral a line, four corners each in
    /// slide EMU, eight numbers a line.
    pub fn selection_rects(
        &self,
        index: usize,
        object: ObjectId,
        start: usize,
        end: usize,
    ) -> Result<Vec<f64>, BridgeError> {
        let (m, lines) = self.caret_lines(index, object)?;
        let body = self.body(object)?;
        let (start, end) = (start.min(end), start.max(end));
        let mut out = Vec::new();
        for line in &lines {
            let stops = body_stops(body, line);
            let (Some(first), Some(last)) = (stops.first(), stops.last()) else {
                continue;
            };
            let lo = start.max(first.0);
            let hi = end.min(last.0);
            // A whole selected paragraph break shows as a sliver at the line's end.
            let takes_break = end > last.0 && start <= last.0;
            if lo > hi || (lo == hi && !takes_break) {
                continue;
            }
            let x1 = x_at(&stops, lo) * EMU_PER_POINT;
            let mut x2 = x_at(&stops, hi) * EMU_PER_POINT;
            if takes_break {
                x2 += 4.0 * EMU_PER_POINT;
            }
            let (t, b) = (
                f64::from(line.top) * EMU_PER_POINT,
                f64::from(line.bottom) * EMU_PER_POINT,
            );
            for (x, y) in [(x1, t), (x2, t), (x2, b), (x1, b)] {
                let (sx, sy) = m.apply(x, y);
                out.extend([sx, sy]);
            }
        }
        Ok(out)
    }

    /// Where the caret goes from `offset` on a move that needs the layout.
    pub fn navigate(
        &self,
        index: usize,
        object: ObjectId,
        offset: usize,
        how: Move,
    ) -> Result<usize, BridgeError> {
        let (_, lines) = self.caret_lines(index, object)?;
        let body = self.body(object)?;
        let Some(at) = line_of(body, &lines, offset) else {
            return Ok(offset);
        };
        let stops = lines
            .get(at)
            .map(|l| body_stops(body, l))
            .unwrap_or_default();
        let target = match how {
            Move::Home => return Ok(stops.first().map_or(offset, |s| s.0)),
            Move::End => return Ok(stops.last().map_or(offset, |s| s.0)),
            Move::Up => at.checked_sub(1),
            Move::Down => Some(at + 1).filter(|&n| n < lines.len()),
        };
        let Some(line) = target.and_then(|t| lines.get(t)) else {
            return Ok(match how {
                Move::Up => 0,
                _ => body_length(body),
            });
        };
        let x = x_at(&stops, offset);
        Ok(nearest(&body_stops(body, line), x).unwrap_or(offset))
    }

    /// Apply an edit through the history. `typing` folds it into the typing burst it
    /// continues, so one undo takes back a run of keystrokes.
    pub fn edit(&mut self, op: Operation, typing: bool) -> Result<bool, BridgeError> {
        let mark = match self.typing {
            Some(mark) if typing => mark,
            _ => self.history.undo_mark(),
        };
        let changed = self
            .history
            .apply(self.deck.parts_mut(), op)
            .map_err(BridgeError::Edit)?;
        if changed {
            self.reports.clear();
            if typing {
                self.history.group_since(mark);
                self.typing = Some(mark);
            } else {
                self.typing = None;
            }
        }
        Ok(changed)
    }

    /// Bold, italic or underline over `[start, end)`: on unless the first character
    /// already has it, its own or inherited, as PowerPoint toggles. An empty range changes nothing.
    pub fn toggle(
        &mut self,
        object: ObjectId,
        start: usize,
        end: usize,
        which: Toggle,
    ) -> Result<bool, BridgeError> {
        let (start, end) = (start.min(end), start.max(end));
        if start == end {
            return Ok(false);
        }
        let body = self.body(object)?;
        let chain = self.deck.text_chain(object);
        let on = !which.is_on(
            chain.as_ref(),
            props_at(body, start),
            paragraph_at(body, start),
        );
        self.edit(
            Operation::FormatRange {
                object,
                start,
                end,
                attribute: which.attribute(on),
            },
            false,
        )
    }

    /// Whether the character at `offset` is drawn bold, italic and underlined, its own
    /// or inherited: bits 1, 2 and 4, for the ribbon's pressed state.
    pub fn format_state(&self, object: ObjectId, offset: usize) -> Result<u32, BridgeError> {
        let body = self.body(object)?;
        // The character before the caret is the one typing continues.
        let at = offset.saturating_sub(1);
        let (p, para) = (props_at(body, at), paragraph_at(body, at));
        let chain = self.deck.text_chain(object);
        let on = |t: Toggle| u32::from(t.is_on(chain.as_ref(), p, para));
        Ok(on(Toggle::Bold) | on(Toggle::Italic) << 1 | on(Toggle::Underline) << 2)
    }

    /// End the typing burst, so the next keystroke starts a new undo step: the caret
    /// moved, or the user clicked elsewhere.
    pub fn break_typing(&mut self) {
        self.typing = None;
    }
}

#[cfg(test)]
mod tests;
