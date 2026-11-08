//! The Home tab's Font and Paragraph groups (`UX-003`): what the Font and Size boxes
//! show for the character at the caret, and the edits they and their neighbours make
//! (font, size, Increase and Decrease Font Size, Font Color, Highlight, alignment and
//! bullets), each one operation through the history so it undoes, and saved by the
//! writer, which patches only the run and paragraph properties that differ.
//!
//! The font list is OpenCalc's (`casual-calc-wasm/src/style.rs` `font_families` at
//! `b23c67c`, owner instruction 54): the picker offers the shared substitution table's
//! families, each named with the bundled face it renders as.

use casual_slide_model::fill::Fill;
use casual_slide_model::presentation::Percentage;
use casual_slide_model::presentation::TextFont;
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::ObjectId;
use casual_slide_model::text::CharacterProperties;
use casual_slide_model::text::{
    AutonumberScheme, Bullet, BulletFont, ParagraphProperties, TextAlign, TextBody, TextSpacing,
    Written,
};
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

/// Where a numbered paragraph's text starts and its number hangs, EMU: what PowerPoint
/// writes when Numbering is turned on in a text box (0.375 inch).
const NUMBER_MARGIN: i32 = 342_900;

/// How far each list level steps a paragraph that states its own left margin, EMU:
/// half an inch, as PowerPoint's Increase List Level moves a text box's paragraph.
const LEVEL_STEP: i32 = 457_200;

/// The deepest list level, from 0: `ST_TextIndentLevelType`'s 8.
const MAX_LEVEL: u8 = 8;

/// The Bullets gallery: each choice's name, character, font, panose, pitch family and
/// charset, as PowerPoint writes them (Filled Round, Hollow Round, Filled Square,
/// Hollow Square, Star, Arrow and Checkmark Bullets).
const BULLET_STYLES: &[(&str, &str, &str, &str, i8, i8)] = &[
    ("round", "\u{2022}", "Arial", "020B0604020202020204", 34, 0),
    ("hollow", "o", "Courier New", "02070309020205020404", 49, 0),
    (
        "square",
        "\u{a7}",
        "Wingdings",
        "05000000000000000000",
        2,
        2,
    ),
    (
        "hollow-square",
        "q",
        "Wingdings",
        "05000000000000000000",
        2,
        2,
    ),
    ("star", "v", "Wingdings", "05000000000000000000", 2, 2),
    ("arrow", "\u{d8}", "Wingdings", "05000000000000000000", 2, 2),
    ("check", "\u{fc}", "Wingdings", "05000000000000000000", 2, 2),
];

/// The Numbering gallery's schemes, as PowerPoint lists them.
pub const NUMBER_SCHEMES: &[&str] = &[
    "arabicPeriod",
    "arabicParenR",
    "romanUcPeriod",
    "alphaUcPeriod",
    "alphaLcParenR",
    "alphaLcPeriod",
    "romanLcPeriod",
];

/// The bounds a Line Spacing multiple keeps to, in lines.
const MIN_LINE_SPACING: f64 = 0.1;
const MAX_LINE_SPACING: f64 = 9.99;

/// The most characters one paste from another application inserts: a bound on what
/// the clipboard's HTML can make the engine do, far past any slide's text.
pub const MAX_PASTE_CHARS: usize = 262_144;

/// The most runs one paste carries.
pub const MAX_PASTE_PIECES: usize = 8_192;

/// One run of text pasted from another application (`PAR-11`): its text, with line
/// feeds between paragraphs and vertical tabs for line breaks, and the look the
/// clipboard's HTML gave it. `None` leaves an attribute to the text it lands in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RichPiece {
    /// The text. Control characters other than line feed, vertical tab and tab are
    /// dropped.
    pub text: String,
    /// Bold.
    pub bold: Option<bool>,
    /// Italic.
    pub italic: Option<bool>,
    /// Single underline, or none.
    pub underline: Option<bool>,
    /// Size, hundredths of a point; kept within `ST_TextFontSize`.
    pub size: Option<i32>,
    /// Colour, `0xRRGGBB`.
    pub color: Option<u32>,
}

/// What a pasted paragraph is: `list` 0 plain, 1 bulleted, 2 numbered; `level` from 0.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RichParagraph {
    /// 0 plain, 1 bulleted, 2 numbered.
    pub list: u8,
    /// The list level, from 0; past 8 it is 8.
    pub level: u8,
}

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
    /// Character Spacing: points, as text, from -1584 to 1584 (`ST_TextPoint`), or
    /// empty for normal (`PAR-20`).
    Spacing,
    /// Clear All Formatting; the value is ignored.
    Clear,
}

impl RunSetting {
    /// From the bridge's number: 0 font, 1 size, 2 colour, 3 highlight, 4 character
    /// spacing, 5 clear all formatting.
    pub fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::Font),
            1 => Some(Self::Size),
            2 => Some(Self::Color),
            3 => Some(Self::Highlight),
            4 => Some(Self::Spacing),
            5 => Some(Self::Clear),
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

/// Hundredths of a point from Character Spacing's points, within `ST_TextPoint`'s
/// -400000 to 400000, rounded to the tenth of a point PowerPoint's box keeps.
fn spacing_from_text(value: &str) -> Option<i32> {
    let points: f64 = value.trim().parse().ok()?;
    if !points.is_finite() || points.abs() > 1584.0 {
        return None;
    }
    // Within 1584 points, so the product fits an `i32`.
    #[allow(clippy::cast_possible_truncation)]
    Some((points * 10.0).round() as i32 * 10)
}

/// `@baseline`'s thousandths of a percent from the Font dialog's Offset, a percentage
/// from -100 to 100 in whole percents, as PowerPoint's box takes it (`PAR-20`).
fn baseline_from_text(value: &str) -> Option<i32> {
    let percent: f64 = value.trim().trim_end_matches('%').trim().parse().ok()?;
    if !percent.is_finite() || percent.abs() > 100.0 {
        return None;
    }
    // Within 100, so the product fits an `i32`.
    #[allow(clippy::cast_possible_truncation)]
    Some(percent.round() as i32 * 1000)
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
    /// drawn: `[font, size in points, alignment token, "1" if bulleted, bullet, list
    /// level, line spacing, bold, italic, underline token, strike (`none`, `single`,
    /// `double`), baseline percent, character spacing points]`. A value no level states
    /// is empty. The font name is the document's: the host shows it
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
        let bullet = chain
            .as_ref()
            .and_then(|c| c.paragraph(para, |p| p.bullet.as_ref()))
            .map_or(String::new(), |r| bullet_token(r.value));
        let level = casual_slide_style::text::paragraph_level(para);
        let spacing = chain
            .as_ref()
            .and_then(|c| c.paragraph(para, |p| p.line_spacing.as_ref()))
            .map_or(String::new(), |r| match r.value {
                TextSpacing::Percent(p) => lines_text(p.value().0),
                TextSpacing::Points(_) => String::new(),
            });
        // What the Font dialog opens with (`PAR-20`): each as drawn, own or inherited.
        let c = chain.as_ref();
        let flag = |on: bool| if on { "1" } else { "0" }.to_owned();
        let bold = c
            .and_then(|c| c.run(run, para, |p| p.bold))
            .is_some_and(|r| r.value);
        let italic = c
            .and_then(|c| c.run(run, para, |p| p.italic))
            .is_some_and(|r| r.value);
        let underline = c
            .and_then(|c| c.run(run, para, |p| p.underline))
            .map_or("none".to_owned(), |r| r.value.to_lexical());
        let strike = c
            .and_then(|c| c.run(run, para, |p| p.strike))
            .map_or("none", |r| match r.value {
                casual_slide_model::text::StrikeType::SngStrike => "single",
                casual_slide_model::text::StrikeType::DblStrike => "double",
                casual_slide_model::text::StrikeType::NoStrike => "none",
            });
        let baseline = c
            .and_then(|c| c.run(run, para, |p| p.baseline.as_ref()))
            .map_or(0, |r| r.value.value().0 / 1000);
        let char_spacing = c
            .and_then(|c| c.run(run, para, |p| p.spacing.as_ref()))
            .map_or(String::new(), |r| {
                format!("{}", f64::from(r.value.value().0) / 100.0)
            });
        Ok(vec![
            font,
            size_text(size),
            align.to_owned(),
            flag(bulleted),
            bullet,
            level.to_string(),
            spacing,
            flag(bold),
            flag(italic),
            underline,
            strike.to_owned(),
            baseline.to_string(),
            char_spacing,
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
            RunSetting::Spacing if value.trim().is_empty() => Attribute::Spacing(None),
            RunSetting::Spacing => Attribute::Spacing(Some(
                spacing_from_text(value).ok_or(BridgeError::BadArgument)?,
            )),
            RunSetting::Clear => Attribute::Clear,
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

    /// The Font dialog's OK (`PAR-20`): every field it states, over `[start, end)`, as
    /// one undo step. `fields` are name and value pairs, a field left out unchanged:
    /// `font` a family; `size` points; `bold`, `italic` `1` or `0`; `underline` an
    /// `ST_TextUnderlineType` token; `color` a colour in [`parse_color`]'s form, or empty
    /// for automatic; `strike` `none`, `single` or `double`; `baseline` a percentage of
    /// the size, positive raised, `0` for neither; `spacing` points, empty for normal.
    ///
    /// # Errors
    ///
    /// [`BridgeError::BadArgument`] for a field it does not know or a value it cannot
    /// read; nothing is changed then.
    pub fn set_font(
        &mut self,
        object: ObjectId,
        (start, end): (usize, usize),
        fields: &[(&str, &str)],
    ) -> Result<bool, BridgeError> {
        use casual_slide_model::properties::Lexical as _;
        use casual_slide_model::text::{StrikeType, UnderlineType};
        let (start, end) = (start.min(end), start.max(end));
        if start == end {
            return Ok(false);
        }
        let flag = |v: &str| match v {
            "1" => Ok(true),
            "0" => Ok(false),
            _ => Err(BridgeError::BadArgument),
        };
        let mut ops = Vec::with_capacity(fields.len());
        for &(name, value) in fields {
            let attribute = match name {
                // The Font group's own readings, so the dialog and the ribbon agree.
                "font" => {
                    let name: String = value
                        .trim()
                        .chars()
                        .filter(|c| !c.is_control())
                        .take(MAX_FONT_NAME_CHARS)
                        .collect();
                    if name.is_empty() {
                        return Err(BridgeError::BadArgument);
                    }
                    Attribute::Latin(Some(Box::new(TextFont {
                        typeface: name,
                        panose: None,
                        pitch_family: None,
                        charset: None,
                    })))
                }
                "size" => {
                    Attribute::Size(Some(size_from_text(value).ok_or(BridgeError::BadArgument)?))
                }
                "spacing" if value.trim().is_empty() => Attribute::Spacing(None),
                "spacing" => Attribute::Spacing(Some(
                    spacing_from_text(value).ok_or(BridgeError::BadArgument)?,
                )),
                "bold" => Attribute::Bold(Some(flag(value)?)),
                "italic" => Attribute::Italic(Some(flag(value)?)),
                "underline" => Attribute::Underline(Some(
                    UnderlineType::parse_lexical(value).ok_or(BridgeError::BadArgument)?,
                )),
                "color" if value.is_empty() => Attribute::Fill(None),
                "color" => Attribute::Fill(Some(Fill::Solid(Some(
                    parse_color(value).ok_or(BridgeError::BadArgument)?,
                )))),
                "strike" => Attribute::Strike(Some(match value {
                    "none" => StrikeType::NoStrike,
                    "single" => StrikeType::SngStrike,
                    "double" => StrikeType::DblStrike,
                    _ => return Err(BridgeError::BadArgument),
                })),
                "baseline" => Attribute::Baseline(Some(
                    baseline_from_text(value).ok_or(BridgeError::BadArgument)?,
                )),
                _ => return Err(BridgeError::BadArgument),
            };
            ops.push(Operation::FormatRange {
                object,
                start,
                end,
                attribute,
            });
        }
        if ops.is_empty() {
            return Ok(false);
        }
        self.edit(Operation::Batch(ops), false)
    }

    /// Change Case over `[start, end)` (`PAR-20`): `sentence`, `lower`, `upper`,
    /// `title` (each word capitalized) or `toggle`. Each run keeps its properties; one
    /// undo step.
    pub fn change_case(
        &mut self,
        object: ObjectId,
        (start, end): (usize, usize),
        mode: &str,
    ) -> Result<bool, BridgeError> {
        use casual_slide_model::text::TextItem;
        let (start, end) = (start.min(end), start.max(end));
        if !matches!(mode, "sentence" | "lower" | "upper" | "title" | "toggle") {
            return Err(BridgeError::BadArgument);
        }
        let body = self.body(object)?;
        if end > body_length(body) {
            return Err(BridgeError::BadArgument);
        }
        // Each stretch of the range inside one run: where it starts, its text, and the
        // run's own properties.
        let mut stretches: Vec<(usize, String, Option<Box<CharacterProperties>>)> = Vec::new();
        let mut base = 0usize;
        for p in &body.paragraphs {
            let mut at = base;
            for item in &p.items {
                let len = item_length(item);
                if let TextItem::Run(r) = item {
                    let (s, e) = (at.max(start), (at + len).min(end));
                    if s < e {
                        let text: String = r
                            .text
                            .as_deref()
                            .unwrap_or_default()
                            .chars()
                            .skip(s - at)
                            .take(e - s)
                            .collect();
                        stretches.push((s, text, r.properties.clone()));
                    }
                }
                at += len;
            }
            base = at + 1;
        }
        // The case of each character, decided over the whole range as one text: a word
        // starts after a space, a sentence after `.`, `!` or `?` and a space.
        let mut word_start = true;
        let mut sentence_start = true;
        let mut ops = Vec::new();
        for (at, text, props) in &stretches {
            let mut out = String::new();
            for c in text.chars() {
                let upper = match mode {
                    "upper" => true,
                    "lower" => false,
                    "toggle" => !c.is_uppercase(),
                    "title" => word_start,
                    _ => sentence_start,
                };
                if upper {
                    out.extend(c.to_uppercase());
                } else {
                    out.extend(c.to_lowercase());
                }
                if c.is_whitespace() {
                    word_start = true;
                } else {
                    word_start = false;
                    if c.is_alphanumeric() {
                        sentence_start = false;
                    } else if matches!(c, '.' | '!' | '?') {
                        sentence_start = true;
                    }
                }
            }
            if out != *text {
                ops.push((*at, text.chars().count(), out, props.clone()));
            }
        }
        if ops.is_empty() {
            return Ok(false);
        }
        // From the end back, so each stretch's offset still holds when it is replaced.
        let mut batch = Vec::new();
        for (at, len, text, properties) in ops.into_iter().rev() {
            batch.push(Operation::DeleteRange {
                object,
                start: at,
                end: at + len,
            });
            batch.push(Operation::InsertRun {
                object,
                at,
                text,
                properties,
            });
        }
        self.edit(Operation::Batch(batch), false)
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

    /// Paragraph alignment (`what` 0: `l`, `ctr`, `r`, `just`), Bullets (`what` 1: `on`,
    /// `off`, `toggle` or a gallery style), Numbering (`what` 2), List Level (`what` 3:
    /// `up`, `down`) or Line Spacing (`what` 4: lines, 0.1 to 9.99) for the paragraphs
    /// `[start, end)` touches. One undo step.
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
            2 => self.number_ops(object, body, first, count, value)?,
            3 => level_ops(object, body, first, count, value)?,
            4 => {
                let lines: f64 = value.trim().parse().map_err(|_| BridgeError::BadArgument)?;
                if !(MIN_LINE_SPACING..=MAX_LINE_SPACING).contains(&lines) {
                    return Err(BridgeError::BadArgument);
                }
                // In range, so the product fits an `i32`.
                #[allow(clippy::cast_possible_truncation)]
                let thousandths = (lines * 100_000.0).round() as i32;
                vec![Operation::FormatParagraphs {
                    object,
                    first,
                    count,
                    change: ParagraphChange::LineSpacing(Some(TextSpacing::Percent(Written::new(
                        Percentage(thousandths),
                    )))),
                }]
            }
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
        let style = BULLET_STYLES.iter().find(|s| s.0 == value);
        let on = match value {
            "on" => true,
            "off" => false,
            // On unless the first paragraph already has bullets, as PowerPoint toggles.
            "toggle" => !bulleted(first),
            _ if style.is_some() => true,
            _ => return Err(BridgeError::BadArgument),
        };
        let (_, ch, face, panose, pitch, charset) = style.copied().unwrap_or(BULLET_STYLES[0]);
        let char_bullet = ParagraphChange::Bullet(
            Some(Bullet::Char(Some(ch.to_owned()))),
            Some(BulletFont::Font(TextFont {
                typeface: face.to_owned(),
                panose: Some(panose.to_owned()),
                pitch_family: Some(pitch),
                charset: Some(charset),
            })),
        );
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
                // Bullets on leaves a bulleted paragraph be; a chosen style replaces
                // numbers and other bullets alike, unless the paragraph has it already.
                if style.is_none() && bulleted(i) {
                    continue;
                }
                if let ParagraphChange::Bullet(b, f) = &char_bullet
                    && props(i).is_some_and(|p| p.bullet == *b && p.bullet_font == *f)
                {
                    continue;
                }
                ops.push(one(char_bullet.clone()));
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

    /// Paste formatted text from another application (`PAR-11`) over `[start, end)`:
    /// each piece inserted with its look, then each pasted paragraph given its list,
    /// all one undo step. Where the caret lands, after the pasted text.
    pub fn paste_rich(
        &mut self,
        object: ObjectId,
        (start, end): (usize, usize),
        pieces: &[RichPiece],
        paragraphs: &[RichParagraph],
    ) -> Result<usize, BridgeError> {
        let (start, end) = (start.min(end), start.max(end));
        let body = self.body(object)?;
        if end > body_length(body) || pieces.len() > MAX_PASTE_PIECES {
            return Err(BridgeError::BadArgument);
        }
        // Each piece starts from the look of the text it lands in, as typing there
        // would, and sets what the clipboard said over it.
        let base = props_at(
            body,
            if start < end {
                start
            } else {
                start.saturating_sub(1)
            },
        )
        .cloned()
        .unwrap_or_default();
        let mut ops = Vec::new();
        if start < end {
            ops.push(Operation::DeleteRange { object, start, end });
        }
        let mut at = start;
        let mut total = 0usize;
        for piece in pieces {
            let text: String = piece
                .text
                .chars()
                .filter(|c| !c.is_control() || matches!(c, '\n' | '\t' | '\u{b}'))
                .collect();
            let n = text.chars().count();
            total = total.saturating_add(n);
            if total > MAX_PASTE_CHARS {
                return Err(BridgeError::BadArgument);
            }
            if n == 0 {
                continue;
            }
            let mut look = base.clone();
            if let Some(b) = piece.bold {
                look.bold = Some(b);
            }
            if let Some(i) = piece.italic {
                look.italic = Some(i);
            }
            if let Some(u) = piece.underline {
                look.underline = Some(if u {
                    casual_slide_model::text::UnderlineType::Sng
                } else {
                    casual_slide_model::text::UnderlineType::None
                });
            }
            if let Some(size) = piece.size {
                look.size = Some(size.clamp(100, 400_000));
            }
            if let Some(rgb) = piece.color.filter(|&c| c <= 0x00FF_FFFF) {
                let [_, r, g, b] = rgb.to_be_bytes();
                look.fill = Some(Fill::Solid(Some(casual_slide_model::color::Color::srgb(
                    r, g, b,
                ))));
            }
            // Absent stays absent: a look with nothing in it writes no `rPr`.
            let properties = (look != CharacterProperties::default()).then(|| Box::new(look));
            ops.push(Operation::InsertRun {
                object,
                at,
                text,
                properties,
            });
            at += n;
        }
        let mark = self.history.undo_mark();
        if !ops.is_empty() {
            self.edit(Operation::Batch(ops), false)?;
        }
        // The lists, on the body as the insert left it.
        let (first, _) = locate(self.body(object)?, start)?;
        let mut list_ops = Vec::new();
        for (k, para) in paragraphs.iter().enumerate() {
            let i = first + k;
            let body = self.body(object)?;
            if i >= body.paragraphs.len() {
                break;
            }
            let value = match para.list {
                1 => self.bullet_ops(object, body, i, 1, "round")?,
                2 => self.number_ops(object, body, i, 1, "arabicPeriod")?,
                _ => Vec::new(),
            };
            list_ops.extend(value);
            let level = para.level.min(MAX_LEVEL);
            if level > 0 {
                list_ops.push(Operation::FormatParagraphs {
                    object,
                    first: i,
                    count: 1,
                    change: ParagraphChange::Level(Some(level)),
                });
            }
        }
        if !list_ops.is_empty() {
            self.edit(Operation::Batch(list_ops), false)?;
        }
        self.history.group_since(mark);
        Ok(at)
    }

    /// Numbering (`what` 2): `toggle`, `off`, or a scheme from [`NUMBER_SCHEMES`] for
    /// the paragraphs `first..first + count`, with PowerPoint's hanging indent where a
    /// paragraph starts at the left edge.
    fn number_ops(
        &self,
        object: ObjectId,
        body: &TextBody,
        first: usize,
        count: usize,
        value: &str,
    ) -> Result<Vec<Operation>, BridgeError> {
        let chain = self.chain_of(object);
        let props = |i: usize| body.paragraphs.get(i).and_then(|p| p.properties.as_deref());
        let bullet = |i: usize| {
            chain
                .as_ref()
                .and_then(|c| c.paragraph(props(i), |p| p.bullet.as_ref()))
                .map(|r| r.value.clone())
        };
        let numbered = |i: usize| matches!(bullet(i), Some(Bullet::AutoNumber { .. }));
        let scheme = match value {
            "off" => None,
            "toggle" if numbered(first) => None,
            "toggle" => Some(AutonumberScheme::ArabicPeriod),
            v if NUMBER_SCHEMES.contains(&v) => {
                Some(AutonumberScheme::parse_lexical(v).ok_or(BridgeError::BadArgument)?)
            }
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
            if let Some(scheme) = scheme {
                let start_at = match bullet(i) {
                    Some(Bullet::AutoNumber { start_at, .. }) => start_at,
                    _ => None,
                };
                let want = Bullet::AutoNumber {
                    scheme: Some(scheme),
                    start_at,
                };
                if props(i).is_some_and(|p| p.bullet.as_ref() == Some(&want)) {
                    continue;
                }
                ops.push(one(ParagraphChange::Bullet(Some(want), None)));
                if margin == 0 {
                    ops.push(one(ParagraphChange::Indent {
                        margin_left: Some(NUMBER_MARGIN),
                        indent: Some(-NUMBER_MARGIN),
                    }));
                }
            } else {
                if !numbered(i) {
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

/// Increase (`up`) or Decrease (`down`) List Level for paragraphs `first..first +
/// count`: `lvl` steps within 0 to 8, written absent at 0 as PowerPoint leaves it, and a
/// paragraph that states its own left margin moves with it by [`LEVEL_STEP`].
fn level_ops(
    object: ObjectId,
    body: &TextBody,
    first: usize,
    count: usize,
    value: &str,
) -> Result<Vec<Operation>, BridgeError> {
    let up = match value {
        "up" => true,
        "down" => false,
        _ => return Err(BridgeError::BadArgument),
    };
    let mut ops = Vec::new();
    for i in first..first + count {
        let own: Option<&ParagraphProperties> =
            body.paragraphs.get(i).and_then(|p| p.properties.as_deref());
        let level = casual_slide_style::text::paragraph_level(own);
        let next = if up {
            level.saturating_add(1).min(MAX_LEVEL)
        } else {
            level.saturating_sub(1)
        };
        if next == level {
            continue;
        }
        let one = |change| Operation::FormatParagraphs {
            object,
            first: i,
            count: 1,
            change,
        };
        ops.push(one(ParagraphChange::Level((next > 0).then_some(next))));
        if let Some(margin) = own.and_then(|p| p.margin_left) {
            let moved = if up {
                margin.saturating_add(LEVEL_STEP)
            } else {
                margin.saturating_sub(LEVEL_STEP).max(0)
            };
            ops.push(one(ParagraphChange::MarginLeft(Some(moved))));
        }
    }
    Ok(ops)
}

/// What the Bullets and Numbering galleries show as chosen: `c:` and the character,
/// `n:` and the scheme, `p:` for a picture, or empty.
fn bullet_token(b: &Bullet) -> String {
    match b {
        Bullet::Char(Some(c)) => format!("c:{c}"),
        Bullet::AutoNumber { scheme, .. } => format!(
            "n:{}",
            scheme
                .unwrap_or(AutonumberScheme::ArabicPeriod)
                .to_lexical()
        ),
        Bullet::Picture(_) => "p:".to_owned(),
        Bullet::Char(None) | Bullet::None => String::new(),
    }
}

/// Line spacing in thousandths of a percent as the menu shows it: `1`, `1.5`.
fn lines_text(thousandths: i32) -> String {
    let s = format!("{:.2}", f64::from(thousandths) / 100_000.0);
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

pub(crate) fn json_string(s: &str) -> String {
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
