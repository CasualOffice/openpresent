//! The Home tab's Font and Paragraph groups (`UX-003`): what the Font and Size boxes
//! show for the character at the caret, and the edits they and their neighbours make
//! (font, size, Increase and Decrease Font Size, Font Color, Highlight, alignment and
//! bullets), each one operation through the history so it undoes, and saved by the
//! writer, which patches only the run and paragraph properties that differ.
//!
//! The font list is OpenCalc's (`casual-calc-wasm/src/style.rs` `font_families` at
//! `b23c67c`, owner instruction 53): the picker offers the shared substitution table's
//! families, each named with the bundled face it renders as.

use casual_slide_model::fill::Fill;
use casual_slide_model::presentation::TextFont;
use casual_slide_model::shape::ObjectId;
use casual_slide_model::text::{Bullet, BulletFont, TextAlign, TextBody};
use casual_slide_model::theme::FontCollectionIndex;
use casual_slide_style::text::{FontScript, RunFont, TextChain};
use casual_slide_transaction::{
    Attribute, Operation, ParagraphChange, body_length, item_length, locate,
};

use crate::edit::{paragraph_at, props_at};
use crate::shapes::parse_color;
use crate::{BridgeError, Viewer};

/// PowerPoint's Font Size list, in points; Increase and Decrease step along it.
pub const SIZE_LADDER: &[f64] = &[
    8.0, 9.0, 10.0, 10.5, 11.0, 12.0, 14.0, 16.0, 18.0, 20.0, 24.0, 28.0, 32.0, 36.0, 40.0, 44.0,
    48.0, 54.0, 60.0, 66.0, 72.0, 80.0, 88.0, 96.0,
];

/// The largest size the Font Size box takes, in points: `ST_TextFontSize`'s 4000.
pub const MAX_FONT_POINTS: f64 = 4000.0;
/// The smallest, in points: `ST_TextFontSize`'s 1.
pub const MIN_FONT_POINTS: f64 = 1.0;

/// Where a bulleted paragraph's text starts and its bullet hangs, EMU: what PowerPoint
/// writes when Bullets is turned on in a text box (0.3125 inch).
const BULLET_MARGIN: i32 = 285_750;

/// The longest font name the Font box takes, in characters.
const MAX_FONT_NAME_CHARS: usize = 64;

/// What a Font group control sets over a range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunSetting {
    /// The Font box: a family name.
    Font,
    /// The Font Size box: points, as text.
    Size,
    /// Font Color: a colour in [`parse_color`]'s form, or empty for automatic.
    Color,
    /// Text Highlight Color: a colour, or empty for none.
    Highlight,
}

impl RunSetting {
    /// From the bridge's number: 0 font, 1 size, 2 colour, 3 highlight.
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::Font),
            1 => Some(Self::Size),
            2 => Some(Self::Color),
            3 => Some(Self::Highlight),
            _ => None,
        }
    }
}

/// Hundredths of a point from the host's text, within `ST_TextFontSize`, rounded to
/// the half point PowerPoint's box keeps.
fn size_from_text(value: &str) -> Option<i32> {
    let points: f64 = value.trim().parse().ok()?;
    if !points.is_finite() {
        return None;
    }
    let points = (points * 2.0).round() / 2.0;
    if !(MIN_FONT_POINTS..=MAX_FONT_POINTS).contains(&points) {
        return None;
    }
    // In range, so the product fits an `i32`.
    #[allow(clippy::cast_possible_truncation)]
    Some((points * 100.0).round() as i32)
}

/// The next size up or down the ladder from `hundredths`; past its ends, steps of ten
/// points up and one down, within `ST_TextFontSize`. Not measured against PowerPoint.
pub fn step_size(hundredths: i32, up: bool) -> i32 {
    let points = f64::from(hundredths) / 100.0;
    let next = if up {
        SIZE_LADDER
            .iter()
            .copied()
            .find(|&s| s > points + 1e-9)
            .unwrap_or(((points / 10.0).floor() + 1.0) * 10.0)
    } else {
        SIZE_LADDER
            .iter()
            .rev()
            .copied()
            .find(|&s| s < points - 1e-9)
            .unwrap_or((points.ceil() - 1.0).max(MIN_FONT_POINTS))
    };
    let next = next.clamp(MIN_FONT_POINTS, MAX_FONT_POINTS);
    #[allow(clippy::cast_possible_truncation)]
    let out = (next * 100.0).round() as i32;
    out
}

/// A size in hundredths of a point as the box shows it: `18`, `10.5`.
fn size_text(hundredths: i32) -> String {
    if hundredths % 100 == 0 {
        format!("{}", hundredths / 100)
    } else {
        let s = format!("{:.2}", f64::from(hundredths) / 100.0);
        s.trim_end_matches('0').trim_end_matches('.').to_owned()
    }
}

fn align_token(a: TextAlign) -> &'static str {
    match a {
        TextAlign::Left => "l",
        TextAlign::Center => "ctr",
        TextAlign::Right => "r",
        TextAlign::Justified | TextAlign::JustifiedLow => "just",
        TextAlign::Distributed | TextAlign::ThaiDistributed => "dist",
    }
}

fn is_bulleted(b: Option<&Bullet>) -> bool {
    matches!(
        b,
        Some(Bullet::Char(_) | Bullet::AutoNumber { .. } | Bullet::Picture(_))
    )
}

/// The paragraphs `[start, end)` touches: the first and how many.
fn paragraph_span(
    body: &TextBody,
    start: usize,
    end: usize,
) -> Result<(usize, usize), BridgeError> {
    let (p0, _) = locate(body, start)?;
    let (p1, _) = locate(body, end)?;
    Ok((p0, p1 - p0 + 1))
}

impl Viewer {
    fn chain_of(&self, object: ObjectId) -> Option<TextChain<'_>> {
        self.deck.text_chain(object)
    }

    /// The theme's major or minor Latin typeface for the part `object` is on.
    fn theme_font(&self, part: u32, major: bool) -> Option<String> {
        let master = casual_slide_style::Chain::of(self.deck.parts(), part).master?;
        let scheme = self
            .deck
            .themes()
            .get(&master.id.part)?
            .elements
            .as_ref()?
            .font_scheme
            .as_ref()?;
        let collection = if major {
            scheme.major.as_ref()
        } else {
            scheme.minor.as_ref()
        }?;
        collection.latin.as_ref()?.typeface.clone()
    }

    /// A typeface as written, with `+mj-lt` and `+mn-lt` followed to the theme.
    fn follow_typeface(&self, part: u32, written: &str) -> Option<String> {
        match written {
            w if w.starts_with("+mj-") => self.theme_font(part, true),
            w if w.starts_with("+mn-") => self.theme_font(part, false),
            w => Some(w.to_owned()),
        }
    }

    /// What the Home tab shows for the character before `offset` in `object`, as
    /// drawn: `[font, size in points, alignment token, "1" if bulleted]`. A value no
    /// level states is empty. The font name is the document's: the host shows it
    /// through `textContent` only.
    pub fn run_state(&self, object: ObjectId, offset: usize) -> Result<Vec<String>, BridgeError> {
        let body = self.body(object)?;
        let at = offset.saturating_sub(1).min(body_length(body));
        let (run, para) = (props_at(body, at), paragraph_at(body, at));
        let chain = self.chain_of(object);
        let font = chain
            .as_ref()
            .and_then(|c| c.run_font(run, para, FontScript::Latin))
            .and_then(|r| match r.value {
                RunFont::Stated(f) => self.follow_typeface(object.part, &f.typeface),
                RunFont::Theme(FontCollectionIndex::Major) => self.theme_font(object.part, true),
                RunFont::Theme(_) => self.theme_font(object.part, false),
            })
            .unwrap_or_default();
        let size = chain
            .as_ref()
            .map_or(1800, |c| c.font_size(run, para).value);
        let align = chain
            .as_ref()
            .and_then(|c| c.paragraph(para, |p| p.align))
            .map_or("l", |r| align_token(r.value));
        let bulleted = chain
            .as_ref()
            .and_then(|c| c.paragraph(para, |p| p.bullet.as_ref()))
            .is_some_and(|r| is_bulleted(Some(r.value)));
        Ok(vec![
            font,
            size_text(size),
            align.to_owned(),
            if bulleted { "1" } else { "0" }.to_owned(),
        ])
    }

    /// Give `[start, end)` of `object` a Font group setting: one undo step. An empty
    /// range changes nothing.
    pub fn set_run(
        &mut self,
        object: ObjectId,
        (start, end): (usize, usize),
        what: RunSetting,
        value: &str,
    ) -> Result<bool, BridgeError> {
        let (start, end) = (start.min(end), start.max(end));
        if start == end {
            return Ok(false);
        }
        let attribute = match what {
            RunSetting::Font => {
                let name: String = value
                    .trim()
                    .chars()
                    .filter(|c| !c.is_control())
                    .take(MAX_FONT_NAME_CHARS)
                    .collect();
                Attribute::Latin((!name.is_empty()).then(|| {
                    Box::new(TextFont {
                        typeface: name,
                        panose: None,
                        pitch_family: None,
                        charset: None,
                    })
                }))
            }
            RunSetting::Size => {
                Attribute::Size(Some(size_from_text(value).ok_or(BridgeError::BadArgument)?))
            }
            RunSetting::Color if value.is_empty() => Attribute::Fill(None),
            RunSetting::Color => Attribute::Fill(Some(Fill::Solid(Some(
                parse_color(value).ok_or(BridgeError::BadArgument)?,
            )))),
            RunSetting::Highlight if value.is_empty() => Attribute::Highlight(None),
            RunSetting::Highlight => {
                Attribute::Highlight(Some(parse_color(value).ok_or(BridgeError::BadArgument)?))
            }
        };
        self.edit(
            Operation::FormatRange {
                object,
                start,
                end,
                attribute,
            },
            false,
        )
    }

    /// Increase or Decrease Font Size over `[start, end)`: each run steps from its own
    /// drawn size, as PowerPoint's buttons do, in one undo step.
    pub fn grow_text(
        &mut self,
        object: ObjectId,
        (start, end): (usize, usize),
        up: bool,
    ) -> Result<bool, BridgeError> {
        let (start, end) = (start.min(end), start.max(end));
        if start == end {
            return Ok(false);
        }
        let body = self.body(object)?;
        if end > body_length(body) {
            return Err(BridgeError::BadArgument);
        }
        let chain = self.chain_of(object);
        // Each stretch of the range in one run, with the size it steps to.
        let mut ops = Vec::new();
        let mut base = 0usize;
        for p in &body.paragraphs {
            let mut at = base;
            for item in &p.items {
                let len = item_length(item);
                let (s, e) = (at.max(start), (at + len).min(end));
                if s < e {
                    let run = props_at(body, s);
                    let para = p.properties.as_deref();
                    let size = chain
                        .as_ref()
                        .map_or(1800, |c| c.font_size(run, para).value);
                    ops.push(Operation::FormatRange {
                        object,
                        start: s,
                        end: e,
                        attribute: Attribute::Size(Some(step_size(size, up))),
                    });
                }
                at += len;
            }
            // The paragraph's end counts one.
            base = at + 1;
        }
        if ops.is_empty() {
            return Ok(false);
        }
        self.edit(Operation::Batch(ops), false)
    }

    /// Paragraph alignment (`what` 0: `l`, `ctr`, `r`, `just`) or Bullets (`what` 1:
    /// `on`, `off`, `toggle`) for the paragraphs `[start, end)` touches. One undo step.
    pub fn set_paragraphs(
        &mut self,
        object: ObjectId,
        (start, end): (usize, usize),
        what: u32,
        value: &str,
    ) -> Result<bool, BridgeError> {
        let (start, end) = (start.min(end), start.max(end));
        let body = self.body(object)?;
        if end > body_length(body) {
            return Err(BridgeError::BadArgument);
        }
        let (first, count) = paragraph_span(body, start, end)?;
        let ops = match what {
            0 => {
                let align = match value {
                    "l" => TextAlign::Left,
                    "ctr" => TextAlign::Center,
                    "r" => TextAlign::Right,
                    "just" => TextAlign::Justified,
                    _ => return Err(BridgeError::BadArgument),
                };
                vec![Operation::FormatParagraphs {
                    object,
                    first,
                    count,
                    change: ParagraphChange::Align(Some(align)),
                }]
            }
            1 => self.bullet_ops(object, body, first, count, value)?,
            _ => return Err(BridgeError::BadArgument),
        };
        self.edit(Operation::Batch(ops), false)
    }

    fn bullet_ops(
        &self,
        object: ObjectId,
        body: &TextBody,
        first: usize,
        count: usize,
        value: &str,
    ) -> Result<Vec<Operation>, BridgeError> {
        let chain = self.chain_of(object);
        let props = |i: usize| body.paragraphs.get(i).and_then(|p| p.properties.as_deref());
        let bulleted = |i: usize| {
            chain
                .as_ref()
                .and_then(|c| c.paragraph(props(i), |p| p.bullet.as_ref()))
                .is_some_and(|r| is_bulleted(Some(r.value)))
        };
        let on = match value {
            "on" => true,
            "off" => false,
            // On unless the first paragraph already has bullets, as PowerPoint toggles.
            "toggle" => !bulleted(first),
            _ => return Err(BridgeError::BadArgument),
        };
        let mut ops = Vec::new();
        for i in first..first + count {
            let margin = chain
                .as_ref()
                .and_then(|c| c.paragraph(props(i), |p| p.margin_left))
                .map_or(0, |r| r.value);
            let one = |change| Operation::FormatParagraphs {
                object,
                first: i,
                count: 1,
                change,
            };
            if on {
                if bulleted(i) {
                    continue;
                }
                ops.push(one(ParagraphChange::Bullet(
                    Some(Bullet::Char(Some("\u{2022}".to_owned()))),
                    Some(BulletFont::Font(TextFont {
                        typeface: "Arial".to_owned(),
                        panose: Some("020B0604020202020204".to_owned()),
                        pitch_family: Some(34),
                        charset: Some(0),
                    })),
                )));
                // A paragraph at the left edge gets PowerPoint's hanging indent.
                if margin == 0 {
                    ops.push(one(ParagraphChange::Indent {
                        margin_left: Some(BULLET_MARGIN),
                        indent: Some(-BULLET_MARGIN),
                    }));
                }
            } else {
                if !bulleted(i) {
                    continue;
                }
                ops.push(one(ParagraphChange::Bullet(Some(Bullet::None), None)));
                if margin != 0 {
                    ops.push(one(ParagraphChange::Indent {
                        margin_left: Some(0),
                        indent: Some(0),
                    }));
                }
            }
        }
        Ok(ops)
    }

    /// The plain text of `[start, end)`, paragraphs joined by line feeds and line breaks
    /// as vertical tabs, for Copy and Cut. Document text, handed only to the host that
    /// asked, for the clipboard.
    pub fn text_slice(
        &self,
        object: ObjectId,
        start: usize,
        end: usize,
    ) -> Result<String, BridgeError> {
        use casual_slide_model::text::TextItem;
        let (start, end) = (start.min(end), start.max(end));
        let body = self.body(object)?;
        if end > body_length(body) {
            return Err(BridgeError::BadArgument);
        }
        let mut out = String::new();
        let mut at = 0usize;
        for (n, p) in body.paragraphs.iter().enumerate() {
            if n > 0 {
                if at >= start && at < end {
                    out.push('\n');
                }
                at += 1;
            }
            for item in &p.items {
                let text: String = match item {
                    TextItem::Run(r) => r.text.clone().unwrap_or_default(),
                    TextItem::Field(f) => f.text.clone().unwrap_or_default(),
                    TextItem::Break(_) => "\u{b}".to_owned(),
                    TextItem::Math(_) | TextItem::Retained(_) => " ".repeat(item_length(item)),
                };
                for ch in text.chars() {
                    if at >= start && at < end {
                        out.push(ch);
                    }
                    at += 1;
                }
            }
        }
        Ok(out)
    }
}

/// The font families the Font box offers, as JSON `[{n,f,k}, …]`: the name, the bundled
/// family it renders as, and how close that is (`exact`, `metric`, `generic`). Copied
/// from OpenCalc's `font_families`.
pub fn font_families_json() -> String {
    use casual_slide_text::substitution::{PICKER_FAMILIES, SubstituteKind, substitute};
    let items: Vec<String> = PICKER_FAMILIES
        .iter()
        .filter_map(|name| {
            let sub = substitute(name)?;
            let kind = match sub.kind {
                SubstituteKind::Bundled => "exact",
                SubstituteKind::MetricCompatible => "metric",
                SubstituteKind::Generic => "generic",
            };
            Some(format!(
                "{{\"n\":{},\"f\":{},\"k\":\"{kind}\"}}",
                json_string(name),
                json_string(sub.family.name)
            ))
        })
        .collect();
    format!("[{}]", items.join(","))
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_step_along_powerpoints_list_and_past_its_ends() {
        assert_eq!(step_size(1800, true), 2000);
        assert_eq!(step_size(1800, false), 1600);
        assert_eq!(step_size(1050, true), 1100);
        assert_eq!(step_size(1100, false), 1050);
        assert_eq!(step_size(1700, true), 1800);
        assert_eq!(step_size(9600, true), 10_000);
        assert_eq!(step_size(800, false), 700);
        assert_eq!(step_size(100, false), 100);
        assert_eq!(step_size(400_000, true), 400_000);
    }

    #[test]
    fn the_size_box_reads_half_points_within_the_schema_range() {
        assert_eq!(size_from_text("18"), Some(1800));
        assert_eq!(size_from_text(" 10.5 "), Some(1050));
        assert_eq!(size_from_text("10.3"), Some(1050));
        assert_eq!(size_from_text("0"), None);
        assert_eq!(size_from_text("4001"), None);
        assert_eq!(size_from_text("NaN"), None);
        assert_eq!(size_from_text("big"), None);
        assert_eq!(size_text(1050), "10.5");
        assert_eq!(size_text(4400), "44");
    }

    #[test]
    fn the_font_list_is_the_substitution_tables_and_parses() {
        let json = font_families_json();
        assert!(json.starts_with("[{\"n\":\"Andale Mono\""));
        assert!(json.contains("{\"n\":\"Calibri\",\"f\":\"Carlito\",\"k\":\"metric\"}"));
        assert_eq!(
            json.matches("\"n\":").count(),
            casual_slide_text::substitution::PICKER_FAMILIES.len()
        );
    }
}
