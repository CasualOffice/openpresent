//! A model text body, through the style chain, to [`crate::layout`].
//!
//! The one entry point a renderer needs is [`layout_text_body`]. Every value is
//! read through `casual-slide-style`'s [`TextChain`], so what is drawn is what the
//! chain says, level by level; nothing is filled into the model.
//!
//! # Defaults used when no level says
//!
//! | Value | Default | Evidence |
//! | --- | --- | --- |
//! | `lIns`, `rIns` | 91440 EMU | ECMA-376 21.1.2.1.1 |
//! | `tIns`, `bIns` | 45720 EMU | ECMA-376 21.1.2.1.1 |
//! | `wrap` | `square` | ECMA-376 21.1.2.1.1 |
//! | `anchor` | `t` | ECMA-376 21.1.2.1.1 |
//! | `algn` | `l` | ECMA-376 21.1.2.2.7 |
//! | `lnSpc` | 100% | **Inferred**: single spacing |
//! | Run colour | black | **Inferred**: what is left when no level and no `fontRef` says |
//! | A symbol font's bullet character | its Unicode look-alike, else `•` | **Inferred**: Wingdings and Symbol are not shipped (`DEC-16`) |
//!
//! Autofit is not applied yet: a body is laid out at its stated sizes (`T2D6-01`).

use casual_slide_model::color::{ResolveContext, resolve};
use casual_slide_model::fill::Fill;
use casual_slide_model::text::{
    AutonumberScheme, Bullet, BulletColor, BulletFont, BulletSize, CapsType, CharacterProperties,
    ParagraphProperties, TextAlign, TextAnchor, TextBody, TextItem, TextSpacing, TextWrap,
    UnderlineType,
};
use casual_slide_model::theme::{FontCollectionIndex, FontScheme};
use casual_slide_style::text::{FontScript, RunFont, TextChain, paragraph_level};

use crate::layout::{
    Align, Anchor, BodyLayout, FontSubstitution, Frame, Insets, ParagraphLayout, Rgba, RunStyle,
    Spacing, StyledText, TextLayout, layout_body,
};
use crate::substitution::{SubstituteKind, substitute};

/// EMU in a point.
const EMU_PER_POINT: f32 = 12_700.0;

/// What a text body resolves against, besides its own chain.
#[derive(Debug, Clone, Copy, Default)]
pub struct TextEnv<'a> {
    /// Colour scheme, colour map and placeholder colour in force for the shape.
    pub colors: ResolveContext<'a>,
    /// The theme's font scheme, for `+mj-lt`, `+mn-ea` and a `fontRef`.
    pub fonts: Option<&'a FontScheme>,
}

/// Lay out a shape's text body in `frame`, in points.
///
/// `chain` is the shape's [`TextChain`], from `Chain::text_chain`. `frame` is the
/// shape's text rectangle, before insets; this applies them.
#[must_use]
pub fn layout_text_body<'a>(
    body: &'a TextBody,
    chain: &TextChain<'a>,
    env: &TextEnv<'_>,
    frame: Frame,
) -> TextLayout {
    let (resolved, substitutions) = resolve_body(body, chain, env);
    let mut layout = layout_body(&resolved, frame);
    layout.substitutions = substitutions;
    let all: String = resolved
        .paragraphs
        .iter()
        .flat_map(|p| p.bullet.iter().chain(p.runs.iter()))
        .map(|r| r.text.as_str())
        .collect();
    layout.missing_scripts = crate::faces::missing_scripts(&all);
    layout
}

fn emu(value: i64) -> f32 {
    value as f32 / EMU_PER_POINT
}

fn spacing(value: &TextSpacing) -> Spacing {
    match value {
        TextSpacing::Percent(p) => Spacing::Percent(p.value().0 as f32 / 100_000.0),
        TextSpacing::Points(p) => Spacing::Points(*p as f32 / 100.0),
    }
}

/// A body's values, resolved, and the font substitutions they needed.
#[must_use]
pub fn resolve_body<'a>(
    body: &'a TextBody,
    chain: &TextChain<'a>,
    env: &TextEnv<'_>,
) -> (BodyLayout, Vec<FontSubstitution>) {
    let own = body.properties.as_ref();
    let inset = |get: fn(&casual_slide_model::text::BodyProperties) -> Option<i64>,
                 default: i64| {
        emu(chain.body(own, get).map_or(default, |r| r.value))
    };
    let mut out = BodyLayout {
        insets: Insets {
            left: inset(|b| b.inset_left.as_ref().map(|w| w.value().0), 91_440),
            top: inset(|b| b.inset_top.as_ref().map(|w| w.value().0), 45_720),
            right: inset(|b| b.inset_right.as_ref().map(|w| w.value().0), 91_440),
            bottom: inset(|b| b.inset_bottom.as_ref().map(|w| w.value().0), 45_720),
        },
        wrap: chain
            .body(own, |b| b.wrap)
            .is_none_or(|w| w.value != TextWrap::None),
        anchor: match chain.body(own, |b| b.anchor).map(|a| a.value) {
            Some(TextAnchor::Center) => Anchor::Middle,
            Some(TextAnchor::Bottom) => Anchor::Bottom,
            _ => Anchor::Top,
        },
        space_first_last: chain
            .body(own, |b| b.space_first_last_paragraph)
            .is_some_and(|v| v.value),
        paragraphs: Vec::with_capacity(body.paragraphs.len()),
    };
    let mut fonts = Fonts {
        env,
        substitutions: Vec::new(),
        script: None,
    };
    let mut numbering = Numbering::default();
    for paragraph in &body.paragraphs {
        let p = paragraph.properties.as_deref();
        let mut runs = Vec::with_capacity(paragraph.items.len());
        for item in &paragraph.items {
            let (text, props) = match item {
                TextItem::Run(r) => (r.text.as_deref().unwrap_or(""), r.properties.as_deref()),
                TextItem::Break(b) => ("\n", b.properties.as_deref()),
                TextItem::Field(f) => (f.text.as_deref().unwrap_or(""), f.properties.as_deref()),
                // Equations draw from their fallback (`T2K-01`); anything retained is
                // reported by import and has no text here.
                TextItem::Math(_) | TextItem::Retained(_) => continue,
            };
            if text.is_empty() {
                continue;
            }
            let text = symbol_text(props, text);
            let text = text.as_ref();
            let style = run_style(chain, props, p, text, &mut fonts);
            let caps = chain
                .run(props, p, |c| c.caps)
                .is_some_and(|c| c.value == CapsType::All);
            runs.push(StyledText {
                text: if caps {
                    text.to_uppercase()
                } else {
                    text.to_owned()
                },
                style,
            });
        }
        let end = run_style(
            chain,
            paragraph.end_properties.as_deref(),
            p,
            "",
            &mut fonts,
        );
        let bullet = bullet(chain, p, runs.first(), &end, &mut numbering, &mut fonts);
        let margin_right = chain
            .paragraph(p, |pp| pp.margin_right)
            .map_or(0, |r| r.value);
        out.paragraphs.push(ParagraphLayout {
            align: match chain.paragraph(p, |pp| pp.align).map(|a| a.value) {
                Some(TextAlign::Center) => Align::Center,
                Some(TextAlign::Right) => Align::Right,
                Some(
                    TextAlign::Justified
                    | TextAlign::JustifiedLow
                    | TextAlign::Distributed
                    | TextAlign::ThaiDistributed,
                ) => Align::Justified,
                Some(TextAlign::Left) | None => Align::Left,
            },
            margin_left: emu(i64::from(chain.margin_left(p).value)),
            margin_right: emu(i64::from(margin_right)),
            indent: emu(i64::from(chain.indent(p).value)),
            line_spacing: chain
                .paragraph(p, |pp| pp.line_spacing.as_ref().map(spacing))
                .map_or(Spacing::Percent(1.0), |r| r.value),
            space_before: chain
                .paragraph(p, |pp| pp.space_before.as_ref().map(spacing))
                .map_or(Spacing::Points(0.0), |r| r.value),
            space_after: chain
                .paragraph(p, |pp| pp.space_after.as_ref().map(spacing))
                .map_or(Spacing::Points(0.0), |r| r.value),
            bullet,
            runs,
            end,
        });
    }
    (out, fonts.substitutions)
}

/// Font names to families, recording each substitution once.
struct Fonts<'e> {
    env: &'e TextEnv<'e>,
    substitutions: Vec<FontSubstitution>,
    /// The theme's script tag (`Jpan`, `Hang`, ...) for the run being styled, from
    /// its language; picks a supplemental font when `ea` or `cs` is empty.
    script: Option<&'static str>,
}

/// A language tag's script, as theme supplemental fonts name it. Only the East
/// Asian scripts, whose `ea` font Office themes leave empty.
fn script_tag(lang: Option<&str>) -> Option<&'static str> {
    let lang = lang?.to_ascii_lowercase();
    let mut parts = lang.split(['-', '_']);
    let primary = parts.next()?;
    let rest: Vec<&str> = parts.collect();
    match primary {
        "ja" => Some("Jpan"),
        "ko" => Some("Hang"),
        "zh" => {
            let traditional = rest
                .iter()
                .any(|p| matches!(*p, "hant" | "tw" | "hk" | "mo"));
            Some(if traditional { "Hant" } else { "Hans" })
        }
        _ => None,
    }
}

impl Fonts<'_> {
    /// A theme font reference, `+mj-lt` and the like, to its typeface.
    fn theme(&self, major: bool, script: FontScript) -> Option<String> {
        let scheme = self.env.fonts?;
        let collection = if major {
            scheme.major.as_ref()
        } else {
            scheme.minor.as_ref()
        }?;
        let font = match script {
            FontScript::Latin => collection.latin.as_ref(),
            FontScript::EastAsian => collection.ea.as_ref(),
            FontScript::ComplexScript => collection.cs.as_ref(),
        };
        let stated = font
            .and_then(|f| f.typeface.clone())
            .filter(|t| !t.trim().is_empty());
        if stated.is_some() || script == FontScript::Latin {
            return stated;
        }
        // An empty `ea` or `cs` gives way to the collection's font for the run's
        // script, as Office themes write it (`<a:ea typeface=""/>` beside
        // `<a:font script="Jpan" .../>`): ISO 29500 20.1.4.1.16.
        let tag = self.script?;
        collection
            .fonts
            .iter()
            .find(|f| f.script.as_deref() == Some(tag))
            .and_then(|f| f.typeface.clone())
            .filter(|t| !t.trim().is_empty())
    }

    /// A typeface as written, with `+mj-` and `+mn-` references followed.
    fn typeface(&self, written: &str) -> Option<String> {
        let script = |s: &str| match s {
            "ea" => Some(FontScript::EastAsian),
            "cs" => Some(FontScript::ComplexScript),
            "lt" => Some(FontScript::Latin),
            _ => None,
        };
        if let Some(rest) = written.strip_prefix("+mj-") {
            return self.theme(true, script(rest)?);
        }
        if let Some(rest) = written.strip_prefix("+mn-") {
            return self.theme(false, script(rest)?);
        }
        Some(written.to_owned()).filter(|t| !t.trim().is_empty())
    }

    fn record(&mut self, family: &str) {
        let Some(found) = substitute(family) else {
            return;
        };
        if found.kind == SubstituteKind::Bundled
            || self.substitutions.iter().any(|s| s.requested == family)
        {
            return;
        }
        self.substitutions.push(FontSubstitution {
            requested: family.to_owned(),
            used: found.family.name,
            kind: found.kind,
        });
    }
}

fn run_family<'a>(
    chain: &TextChain<'a>,
    props: Option<&'a CharacterProperties>,
    paragraph: Option<&ParagraphProperties>,
    script: FontScript,
    fonts: &Fonts<'_>,
) -> Option<String> {
    match chain.run_font(props, paragraph, script)?.value {
        RunFont::Stated(font) => fonts.typeface(&font.typeface),
        RunFont::Theme(FontCollectionIndex::Major) => fonts.theme(true, script),
        RunFont::Theme(FontCollectionIndex::Minor) => fonts.theme(false, script),
        RunFont::Theme(FontCollectionIndex::None) => None,
    }
}

fn color_of(fill: &Fill, env: &TextEnv<'_>) -> Rgba {
    let color = match fill {
        Fill::Solid(Some(c)) => Some(c),
        // A gradient's first stop stands in until gradient text is drawn (`T2B-01`).
        Fill::Gradient(g) => g
            .stops
            .as_ref()
            .and_then(|s| s.first())
            .and_then(|s| s.color.as_ref()),
        Fill::None => {
            return Rgba {
                a: 0,
                ..Rgba::BLACK
            };
        }
        _ => None,
    };
    color
        .and_then(|c| resolve(c, &env.colors).ok())
        .map_or(Rgba::BLACK, |c| Rgba {
            r: c.rgb.r,
            g: c.rgb.g,
            b: c.rgb.b,
            a: u8::try_from((u64::from(c.alpha.min(100_000)) * 255 + 50_000) / 100_000)
                .unwrap_or(255),
        })
}

fn run_style<'a>(
    chain: &TextChain<'a>,
    props: Option<&'a CharacterProperties>,
    paragraph: Option<&'a ParagraphProperties>,
    text: &str,
    fonts: &mut Fonts<'_>,
) -> RunStyle {
    let east_asian = text.chars().any(crate::cjk::is_cjk);
    fonts.script = script_tag(
        chain
            .run(props, paragraph, |c| c.lang.as_deref())
            .map(|r| r.value),
    );
    let ea = run_family(chain, props, paragraph, FontScript::EastAsian, fonts);
    let family = east_asian
        .then(|| ea.clone())
        .flatten()
        .or_else(|| run_family(chain, props, paragraph, FontScript::Latin, fonts));
    let east_asian_family = ea.filter(|e| Some(e) != family.as_ref());
    if let Some(f) = &family {
        fonts.record(f);
    }
    // A hyperlinked run is drawn in the theme's `hlink` and underlined, whatever
    // its own fill, as PowerPoint and LibreOffice draw it, unless the 2019
    // extension asks for the text's own colour (`hlinkClr val="tx"`): then only
    // the underline is added.
    let click = props.and_then(|c| c.hyperlink_click.as_deref());
    let link = click.is_some();
    let link_colour =
        click.is_some_and(|h| h.color != Some(casual_slide_model::shape::HyperlinkColor::Text));
    RunStyle {
        family,
        size: chain.font_size(props, paragraph).value.max(1) as f32 / 100.0,
        bold: chain
            .run(props, paragraph, |c| c.bold)
            .is_some_and(|b| b.value),
        italic: chain
            .run(props, paragraph, |c| c.italic)
            .is_some_and(|b| b.value),
        color: link_colour
            .then(|| hyperlink_color(fonts.env))
            .flatten()
            .unwrap_or_else(|| {
                chain
                    .run_fill(props, paragraph)
                    .map_or(Rgba::BLACK, |f| color_of(&f.value, fonts.env))
            }),
        underline: chain
            .run(props, paragraph, |c| c.underline)
            .map(|u| u.value)
            .filter(|u| *u != UnderlineType::None)
            .or(link.then_some(UnderlineType::Sng)),
        highlight: chain
            .run(props, paragraph, |c| c.highlight.as_ref())
            .and_then(|h| h.value.clone())
            .map(|c| color_of(&Fill::Solid(Some(c)), fonts.env)),
        east_asian_family,
    }
}

/// The theme's `hlink`, through the colour map: the colour PowerPoint draws a
/// run with `hlinkClick` in, over the run's own fill. `None` when the theme has
/// no such slot, so the run keeps its own colour.
fn hyperlink_color(env: &TextEnv<'_>) -> Option<Rgba> {
    let hlink =
        casual_slide_model::color::Color::new(casual_slide_model::color::ColorValue::Scheme(
            casual_slide_model::color::SchemeColor::Hlink,
        ));
    resolve(&hlink, &env.colors).ok().map(|c| Rgba {
        r: c.rgb.r,
        g: c.rgb.g,
        b: c.rgb.b,
        a: 255,
    })
}

/// Autonumber counters by level: the scheme last seen there and its number.
#[derive(Debug, Default)]
struct Numbering {
    levels: [Option<(AutonumberScheme, i32)>; 9],
}

impl Numbering {
    /// The number for a paragraph at `level` numbered with `scheme`.
    fn next(&mut self, level: u8, scheme: AutonumberScheme, start: i32) -> i32 {
        let level = usize::from(level.min(8));
        for deeper in self.levels.iter_mut().skip(level + 1) {
            *deeper = None;
        }
        let Some(slot) = self.levels.get_mut(level) else {
            return start;
        };
        let n = match *slot {
            Some((s, n)) if s == scheme => n.saturating_add(1),
            _ => start,
        };
        *slot = Some((scheme, n));
        n
    }

    /// A paragraph at `level` with no number ends the sequence there.
    fn reset(&mut self, level: u8) {
        let level = usize::from(level.min(8));
        for slot in self.levels.iter_mut().skip(level) {
            *slot = None;
        }
    }
}

fn roman(mut n: i32, upper: bool) -> String {
    const TABLE: [(i32, &str); 13] = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut out = String::new();
    if n <= 0 {
        return n.to_string();
    }
    for (value, digits) in TABLE {
        while n >= value {
            out.push_str(digits);
            n -= value;
        }
    }
    if upper { out.to_uppercase() } else { out }
}

fn alpha(n: i32, upper: bool) -> String {
    if n <= 0 {
        return n.to_string();
    }
    // PowerPoint repeats the letter past z: aa, bb (Inferred).
    let index = u8::try_from((n - 1) % 26).unwrap_or(0);
    let count = usize::try_from((n - 1) / 26 + 1).unwrap_or(1).min(64);
    let base = if upper { b'A' } else { b'a' };
    char::from(base + index).to_string().repeat(count)
}

/// The text of an autonumber, for the schemes Latin decks use. The others are
/// drawn as `1.` until `T2D3-01` covers them.
fn autonumber(scheme: AutonumberScheme, n: i32) -> String {
    use AutonumberScheme as S;
    match scheme {
        S::ArabicPeriod => format!("{n}."),
        S::ArabicParenR => format!("{n})"),
        S::ArabicParenBoth => format!("({n})"),
        S::ArabicPlain => n.to_string(),
        S::AlphaLcPeriod => format!("{}.", alpha(n, false)),
        S::AlphaUcPeriod => format!("{}.", alpha(n, true)),
        S::AlphaLcParenR => format!("{})", alpha(n, false)),
        S::AlphaUcParenR => format!("{})", alpha(n, true)),
        S::AlphaLcParenBoth => format!("({})", alpha(n, false)),
        S::AlphaUcParenBoth => format!("({})", alpha(n, true)),
        S::RomanLcPeriod => format!("{}.", roman(n, false)),
        S::RomanUcPeriod => format!("{}.", roman(n, true)),
        S::RomanLcParenR => format!("{})", roman(n, false)),
        S::RomanUcParenR => format!("{})", roman(n, true)),
        S::RomanLcParenBoth => format!("({})", roman(n, false)),
        S::RomanUcParenBoth => format!("({})", roman(n, true)),
        _ => format!("{n}."),
    }
}

/// A symbol font's bullet character as the Unicode character it looks like.
/// `None` when the font is not a symbol font.
fn symbol_bullet(font: &str, ch: char) -> Option<char> {
    let key = font.trim().to_ascii_lowercase();
    if !(key.starts_with("wingdings") || key == "symbol") {
        return None;
    }
    // Symbol fonts are often written in the private use area, F020 to F0FF.
    let code = u32::from(ch);
    let ch = if (0xF020..=0xF0FF).contains(&code) {
        char::from_u32(code - 0xF000).unwrap_or(ch)
    } else {
        ch
    };
    let mapped = if key == "symbol" {
        match ch {
            '\u{b7}' => '\u{2022}',
            other => other,
        }
    } else {
        wingdings_char(ch).unwrap_or('\u{2022}')
    };
    let drawable = crate::faces::coverage_face_bytes(mapped, false, false).is_some();
    Some(if drawable { mapped } else { '\u{2022}' })
}

/// The Wingdings characters decks use most, as the Unicode characters they look like,
/// from the mapping in Unicode's Wingdings and Webdings proposal (L2/11-052): `n` is
/// the large black square and `§` the small one.
fn wingdings_char(ch: char) -> Option<char> {
    Some(match ch {
        'n' => '\u{25a0}',
        '\u{a7}' => '\u{25aa}',
        'l' => '\u{25cf}',
        // `q` is ❑ (U+2751), which no bundled face draws: the nearest one that does.
        'q' | 'o' => '\u{25a1}',
        'v' => '\u{2756}',
        '\u{d8}' => '\u{27a2}',
        '\u{fc}' => '\u{2714}',
        '\u{e0}' | '\u{e8}' => '\u{2192}',
        _ => return None,
    })
}

/// The Symbol font's encoding, as Unicode: Greek for the letters, and the
/// mathematical signs and bullet decks use. `None` for a code not listed.
fn symbol_char(ch: char) -> Option<char> {
    const LOWER: [char; 26] = [
        'α', 'β', 'χ', 'δ', 'ε', 'φ', 'γ', 'η', 'ι', 'ϕ', 'κ', 'λ', 'μ', 'ν', 'ο', 'π', 'θ', 'ρ',
        'σ', 'τ', 'υ', 'ϖ', 'ω', 'ξ', 'ψ', 'ζ',
    ];
    const UPPER: [char; 26] = [
        'Α', 'Β', 'Χ', 'Δ', 'Ε', 'Φ', 'Γ', 'Η', 'Ι', 'ϑ', 'Κ', 'Λ', 'Μ', 'Ν', 'Ο', 'Π', 'Θ', 'Ρ',
        'Σ', 'Τ', 'Υ', 'ς', 'Ω', 'Ξ', 'Ψ', 'Ζ',
    ];
    let code = u32::from(ch);
    Some(match ch {
        'a'..='z' => *LOWER.get(usize::try_from(code - u32::from('a')).ok()?)?,
        'A'..='Z' => *UPPER.get(usize::try_from(code - u32::from('A')).ok()?)?,
        '"' => '∀',
        '$' => '∃',
        '\'' => '∋',
        '*' => '∗',
        '-' => '−',
        ' ' | '!' | '#' | '%' | '&' | '('..=',' | '.'..='?' | '[' | ']' | '_' | '{'..='}' => ch,
        '\u{a3}' => '≤',
        '\u{a5}' => '∞',
        '\u{ac}' => '←',
        '\u{ad}' => '↑',
        '\u{ae}' => '→',
        '\u{af}' => '↓',
        '\u{b0}' => '°',
        '\u{b1}' => '±',
        '\u{b3}' => '≥',
        '\u{b4}' => '×',
        '\u{b7}' => '\u{2022}',
        '\u{b8}' => '÷',
        '\u{b9}' => '≠',
        '\u{ba}' => '≡',
        '\u{bb}' => '≈',
        '\u{d6}' => '√',
        '\u{d7}' => '⋅',
        _ => return None,
    })
}

/// A run's text with each private-use code of a symbol font (`F020` to `F0FF`,
/// as Office writes them) replaced by the character it shows, when the run's
/// `sym` or Latin font is Symbol or Wingdings. A code with no known character is
/// left, and is reported as having no face.
fn symbol_text<'t>(
    props: Option<&CharacterProperties>,
    text: &'t str,
) -> std::borrow::Cow<'t, str> {
    let pua = |c: char| (0xF020..=0xF0FF).contains(&u32::from(c));
    let font = props
        .and_then(|c| c.symbol.as_deref().or(c.latin.as_deref()))
        .map(|f| f.typeface.trim().to_ascii_lowercase());
    let map: fn(char) -> Option<char> = match font.as_deref() {
        Some("symbol") => symbol_char,
        Some(f) if f.starts_with("wingdings") => wingdings_char,
        _ => return text.into(),
    };
    if !text.chars().any(pua) {
        return text.into();
    }
    text.chars()
        .map(|c| {
            if pua(c) {
                char::from_u32(u32::from(c) - 0xF000)
                    .and_then(map)
                    .unwrap_or(c)
            } else {
                c
            }
        })
        .collect::<String>()
        .into()
}

fn bullet<'a>(
    chain: &TextChain<'a>,
    p: Option<&'a ParagraphProperties>,
    first: Option<&StyledText>,
    end: &RunStyle,
    numbering: &mut Numbering,
    fonts: &mut Fonts<'_>,
) -> Option<StyledText> {
    let level = paragraph_level(p);
    let has_text = first.is_some();
    let text_style = first.map_or(end, |r| &r.style);
    let kind = chain.paragraph(p, |pp| pp.bullet.clone()).map(|b| b.value);
    let font = match chain
        .paragraph(p, |pp| pp.bullet_font.clone())
        .map(|b| b.value)
    {
        Some(BulletFont::Font(f)) => fonts.typeface(&f.typeface),
        Some(BulletFont::FollowText) | None => text_style.family.clone(),
    };
    let (text, family) = match kind {
        Some(Bullet::Char(Some(c))) if has_text => {
            numbering.reset(level);
            let mut chars = c.chars();
            match (chars.next(), font.as_deref()) {
                (Some(ch), Some(f)) => match symbol_bullet(f, ch) {
                    Some(mapped) => (mapped.to_string(), text_style.family.clone()),
                    None => (c.clone(), font.clone()),
                },
                _ => (c.clone(), font.clone()),
            }
        }
        Some(Bullet::AutoNumber { scheme, start_at }) if has_text => {
            let scheme = scheme.unwrap_or(AutonumberScheme::ArabicPeriod);
            let n = numbering.next(level, scheme, start_at.unwrap_or(1));
            (autonumber(scheme, n), font.clone())
        }
        _ => {
            numbering.reset(level);
            return None;
        }
    };
    if let Some(f) = &family {
        fonts.record(f);
    }
    let size = match chain
        .paragraph(p, |pp| pp.bullet_size.clone())
        .map(|b| b.value)
    {
        Some(BulletSize::Percent(pct)) => text_style.size * pct.value().0 as f32 / 100_000.0,
        Some(BulletSize::Points(pts)) => pts as f32 / 100.0,
        Some(BulletSize::FollowText) | None => text_style.size,
    };
    let color = match chain
        .paragraph(p, |pp| pp.bullet_color.clone())
        .map(|b| b.value)
    {
        Some(BulletColor::Color(Some(c))) => color_of(&Fill::Solid(Some(c)), fonts.env),
        _ => text_style.color,
    };
    Some(StyledText {
        text,
        style: RunStyle {
            family,
            size: size.max(0.01),
            bold: text_style.bold,
            italic: false,
            color,
            // PowerPoint does not underline or highlight a bullet.
            underline: None,
            highlight: None,
            east_asian_family: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autonumbers_count_per_level_and_restart_after_a_gap() {
        let mut n = Numbering::default();
        assert_eq!(n.next(0, AutonumberScheme::ArabicPeriod, 1), 1);
        assert_eq!(n.next(0, AutonumberScheme::ArabicPeriod, 1), 2);
        assert_eq!(n.next(1, AutonumberScheme::AlphaLcParenR, 1), 1);
        assert_eq!(
            n.next(0, AutonumberScheme::ArabicPeriod, 1),
            3,
            "a deeper level does not restart the outer count"
        );
        assert_eq!(
            n.next(1, AutonumberScheme::AlphaLcParenR, 1),
            1,
            "but the outer paragraph restarts the deeper one"
        );
        n.reset(0);
        assert_eq!(n.next(0, AutonumberScheme::ArabicPeriod, 1), 1);
    }

    #[test]
    fn autonumber_text_follows_the_scheme() {
        assert_eq!(autonumber(AutonumberScheme::ArabicPeriod, 3), "3.");
        assert_eq!(autonumber(AutonumberScheme::AlphaUcParenR, 2), "B)");
        assert_eq!(autonumber(AutonumberScheme::RomanLcParenBoth, 4), "(iv)");
        assert_eq!(autonumber(AutonumberScheme::AlphaLcPeriod, 27), "aa.");
    }

    #[test]
    fn wingdings_bullets_become_characters_a_bundled_face_draws() {
        let ch = symbol_bullet("Wingdings", '\u{a7}').expect("a symbol font");
        assert!(crate::faces::coverage_face_bytes(ch, false, false).is_some());
        assert_eq!(symbol_bullet("Arial", '\u{2022}'), None);
        assert_eq!(symbol_bullet("Symbol", '\u{f0b7}'), Some('\u{2022}'));
    }

    #[test]
    fn wingdings_n_is_the_large_black_square_and_section_sign_the_small_one() {
        assert_eq!(symbol_bullet("Wingdings", 'n'), Some('\u{25a0}'));
        assert_eq!(symbol_bullet("Wingdings", '\u{f06e}'), Some('\u{25a0}'));
        assert_eq!(symbol_bullet("Wingdings", '\u{a7}'), Some('\u{25aa}'));
    }

    #[test]
    fn theme_font_references_follow_the_scheme_and_unknown_ones_give_nothing() {
        use casual_slide_model::theme::{FontCollection, TextFont};
        let face = |name: &str| TextFont {
            typeface: Some(name.to_owned()),
            ..TextFont::default()
        };
        let scheme = FontScheme {
            major: Some(FontCollection {
                latin: Some(face("Calibri Light")),
                ..FontCollection::default()
            }),
            minor: Some(FontCollection {
                latin: Some(face("Calibri")),
                ..FontCollection::default()
            }),
            ..FontScheme::default()
        };
        let env = TextEnv {
            fonts: Some(&scheme),
            ..TextEnv::default()
        };
        let fonts = Fonts {
            env: &env,
            substitutions: Vec::new(),
            script: None,
        };
        assert_eq!(fonts.typeface("+mj-lt").as_deref(), Some("Calibri Light"));
        assert_eq!(fonts.typeface("+mn-lt").as_deref(), Some("Calibri"));
        assert_eq!(fonts.typeface("+mn-ea"), None, "absent from the scheme");
        assert_eq!(fonts.typeface("+zz-lt"), Some("+zz-lt".to_owned()));
        assert_eq!(fonts.typeface("Arial").as_deref(), Some("Arial"));
    }

    #[test]
    fn an_empty_east_asian_theme_font_takes_the_script_font_for_the_runs_language() {
        use casual_slide_model::theme::{FontCollection, SupplementalFont, TextFont};
        let supplemental = |script: &str, face: &str| SupplementalFont {
            script: Some(script.to_owned()),
            typeface: Some(face.to_owned()),
        };
        let scheme = FontScheme {
            minor: Some(FontCollection {
                latin: Some(TextFont {
                    typeface: Some("Calibri".to_owned()),
                    ..TextFont::default()
                }),
                ea: Some(TextFont {
                    typeface: Some(String::new()),
                    ..TextFont::default()
                }),
                fonts: vec![
                    supplemental("Hang", "맑은 고딕"),
                    supplemental("Jpan", "ＭＳ Ｐゴシック"),
                ],
                ..FontCollection::default()
            }),
            ..FontScheme::default()
        };
        let env = TextEnv {
            fonts: Some(&scheme),
            ..TextEnv::default()
        };
        let mut fonts = Fonts {
            env: &env,
            substitutions: Vec::new(),
            script: None,
        };
        assert_eq!(
            fonts.typeface("+mn-ea"),
            None,
            "no language, no script font"
        );
        fonts.script = script_tag(Some("ja-JP"));
        assert_eq!(fonts.typeface("+mn-ea").as_deref(), Some("ＭＳ Ｐゴシック"));
        fonts.script = script_tag(Some("ko-KR"));
        assert_eq!(fonts.typeface("+mn-ea").as_deref(), Some("맑은 고딕"));
        assert_eq!(fonts.typeface("+mn-lt").as_deref(), Some("Calibri"));
        assert_eq!(script_tag(Some("zh-TW")), Some("Hant"));
        assert_eq!(script_tag(Some("zh-CN")), Some("Hans"));
        assert_eq!(script_tag(Some("en-US")), None);
    }

    #[test]
    fn a_substitution_is_reported_once_and_a_bundled_family_not_at_all() {
        let env = TextEnv::default();
        let mut fonts = Fonts {
            env: &env,
            substitutions: Vec::new(),
            script: None,
        };
        fonts.record("Calibri");
        fonts.record("Calibri");
        fonts.record("Carlito");
        assert_eq!(fonts.substitutions.len(), 1);
        assert_eq!(fonts.substitutions[0].used, "Carlito");
        assert_eq!(
            fonts.substitutions[0].kind,
            SubstituteKind::MetricCompatible
        );
    }

    mod from_the_model {
        use super::*;
        use casual_slide_model::color::Color;
        use casual_slide_model::presentation::{Coordinate, TextFont};
        use casual_slide_model::text::{BodyProperties, Paragraph, TextRun, Written};

        fn props(size: i32, bold: bool, font: &str, color: Option<Color>) -> CharacterProperties {
            CharacterProperties {
                size: Some(size),
                bold: Some(bold),
                latin: Some(Box::new(TextFont {
                    typeface: font.to_owned(),
                    panose: None,
                    pitch_family: None,
                    charset: None,
                })),
                fill: color.map(|c| Fill::Solid(Some(c))),
                ..CharacterProperties::default()
            }
        }

        fn run(text: &str, p: CharacterProperties) -> TextItem {
            TextItem::Run(TextRun {
                properties: Some(Box::new(p)),
                text: Some(text.to_owned()),
            })
        }

        fn deck_body() -> TextBody {
            TextBody {
                properties: Some(BodyProperties {
                    anchor: Some(TextAnchor::Bottom),
                    inset_left: Some(Written::new(Coordinate(0))),
                    ..BodyProperties::default()
                }),
                list_style: None,
                paragraphs: vec![
                    Paragraph {
                        properties: None,
                        items: vec![run(
                            "Title text",
                            props(2400, true, "Calibri", Some(Color::srgb(200, 0, 0))),
                        )],
                        end_properties: None,
                    },
                    Paragraph {
                        properties: Some(Box::new(ParagraphProperties {
                            margin_left: Some(342_900),
                            indent: Some(-342_900),
                            bullet: Some(Bullet::Char(Some("\u{2022}".to_owned()))),
                            ..ParagraphProperties::default()
                        })),
                        items: vec![
                            run("First ", props(1800, false, "Arial", None)),
                            TextItem::Break(Default::default()),
                            run("second", props(1800, false, "Arial", None)),
                        ],
                        end_properties: None,
                    },
                    Paragraph::default(),
                ],
            }
        }

        const FRAME: Frame = Frame {
            x: 0.0,
            y: 0.0,
            width: 400.0,
            height: 300.0,
        };

        #[test]
        fn a_body_lays_out_with_its_runs_sizes_colours_and_bullet() {
            let body = deck_body();
            let layout = layout_text_body(&body, &TextChain::default(), &TextEnv::default(), FRAME);
            let texts: Vec<&str> = layout.runs.iter().map(|r| r.text.as_str()).collect();
            assert_eq!(texts, ["Title text", "\u{2022}", "First", "second"]);
            assert_eq!(layout.lines, 4, "title, two lines, and the empty paragraph");
            let title = &layout.runs[0];
            assert_eq!(title.size, 24.0);
            assert_eq!(
                (title.color.r, title.color.g, title.color.b, title.color.a),
                (200, 0, 0, 255)
            );
            assert_eq!(title.face_key, "Carlito-Bold");
            assert_eq!(layout.runs[2].face_key, "LiberationSans-Regular");
            assert_eq!(layout.runs[2].color, Rgba::BLACK, "no fill stated");
            // lIns 0, so the title starts at the frame's edge; the bullet hangs at
            // marL + indent = 0 and the text sits at marL = 27 pt.
            assert!(title.glyphs[0].x.abs() < 0.01);
            assert!(layout.runs[1].glyphs[0].x.abs() < 0.01);
            assert!((layout.runs[2].glyphs[0].x - 27.0).abs() < 0.01);
            assert!(
                (layout.runs[3].glyphs[0].x - 27.0).abs() < 0.01,
                "the line after a break starts at marL"
            );
            // Anchored at the bottom, above the default bottom inset of 3.6 pt.
            let last_baseline = layout.runs[3].glyphs[0].y;
            assert!(last_baseline > 200.0 && last_baseline < 300.0 - 3.6);
        }

        #[test]
        fn every_substitution_is_reported_once_in_order_of_use() {
            let body = deck_body();
            let layout = layout_text_body(&body, &TextChain::default(), &TextEnv::default(), FRAME);
            let reported: Vec<(&str, &str)> = layout
                .substitutions
                .iter()
                .map(|s| (s.requested.as_str(), s.used))
                .collect();
            assert_eq!(
                reported,
                [("Calibri", "Carlito"), ("Arial", "Liberation Sans")]
            );
            assert!(layout.missing_scripts.is_empty());
        }

        #[test]
        fn absent_values_take_the_schema_defaults_and_nothing_is_filled_in() {
            let body = TextBody {
                properties: None,
                list_style: None,
                paragraphs: vec![Paragraph {
                    properties: None,
                    items: vec![TextItem::Run(TextRun {
                        properties: None,
                        text: Some("x".to_owned()),
                    })],
                    end_properties: None,
                }],
            };
            let before = body.clone();
            let (resolved, _) = resolve_body(&body, &TextChain::default(), &TextEnv::default());
            assert_eq!(body, before, "the model is not changed");
            assert_eq!(resolved.insets.left, 7.2);
            assert_eq!(resolved.insets.top, 3.6);
            assert!(resolved.wrap);
            assert_eq!(resolved.anchor, Anchor::Top);
            let p = &resolved.paragraphs[0];
            assert_eq!(p.align, Align::Left);
            assert_eq!(p.runs[0].style.size, 18.0, "MS-OI29500's 1800");
            assert_eq!(p.bullet, None);
        }

        #[test]
        fn autonumbered_paragraphs_count_up() {
            let numbered = |text: &str| Paragraph {
                properties: Some(Box::new(ParagraphProperties {
                    bullet: Some(Bullet::AutoNumber {
                        scheme: Some(AutonumberScheme::ArabicParenR),
                        start_at: Some(3),
                    }),
                    ..ParagraphProperties::default()
                })),
                items: vec![TextItem::Run(TextRun {
                    properties: None,
                    text: Some(text.to_owned()),
                })],
                end_properties: None,
            };
            let body = TextBody {
                properties: None,
                list_style: None,
                paragraphs: vec![numbered("a"), numbered("b")],
            };
            let (resolved, _) = resolve_body(&body, &TextChain::default(), &TextEnv::default());
            let bullets: Vec<&str> = resolved
                .paragraphs
                .iter()
                .map(|p| p.bullet.as_ref().map_or("", |b| b.text.as_str()))
                .collect();
            assert_eq!(bullets, ["3)", "4)"]);
        }

        #[test]
        fn a_symbol_font_character_in_a_run_draws_as_the_character_it_shows() {
            let mut with_sym = props(1800, false, "Times New Roman", None);
            with_sym.symbol = Some(Box::new(TextFont {
                typeface: "Symbol".to_owned(),
                panose: None,
                pitch_family: None,
                charset: None,
            }));
            let mut wingdings = props(1800, false, "Wingdings", None);
            wingdings.symbol = None;
            let body = TextBody {
                properties: None,
                list_style: None,
                paragraphs: vec![Paragraph {
                    properties: None,
                    items: vec![
                        run("\u{f0b7}", with_sym),
                        run("\u{f0fc}", wingdings),
                        run(" \u{f0b7} plain", props(1800, false, "Arial", None)),
                    ],
                    end_properties: None,
                }],
            };
            let (resolved, _) = resolve_body(&body, &TextChain::default(), &TextEnv::default());
            let texts: Vec<&str> = resolved.paragraphs[0]
                .runs
                .iter()
                .map(|r| r.text.as_str())
                .collect();
            assert_eq!(
                texts,
                ["\u{2022}", "\u{2714}", " \u{f0b7} plain"],
                "a symbol font's private-use code is its character; elsewhere it stays"
            );
        }

        #[test]
        fn a_break_ending_a_paragraph_opens_a_line_sized_by_the_paragraph_mark() {
            let body = |with_break: bool| {
                let mut items = vec![run("x", props(1800, false, "Arial", None))];
                if with_break {
                    items.push(TextItem::Break(Default::default()));
                }
                TextBody {
                    properties: Some(BodyProperties {
                        anchor: Some(TextAnchor::Bottom),
                        ..BodyProperties::default()
                    }),
                    list_style: None,
                    paragraphs: vec![Paragraph {
                        properties: None,
                        items,
                        end_properties: Some(Box::new(props(4400, false, "Arial", None))),
                    }],
                }
            };
            let env = TextEnv::default();
            let plain = layout_text_body(&body(false), &TextChain::default(), &env, FRAME);
            let broken = layout_text_body(&body(true), &TextChain::default(), &env, FRAME);
            assert_eq!((plain.lines, broken.lines), (1, 2));
            let lift = plain.runs[0].glyphs[0].y - broken.runs[0].glyphs[0].y;
            assert!(
                lift > 44.0,
                "the empty last line is 44 pt type high, not 18: lifted {lift}"
            );
        }

        #[test]
        fn a_hyperlinked_run_takes_the_hlink_colour_and_an_underline() {
            use casual_slide_model::shape::Hyperlink;
            use casual_slide_model::theme::{ColorMap, ColorScheme, ColorSchemeIndex};
            let mut scheme = ColorScheme::default();
            scheme.slots[ColorSchemeIndex::Hlink as usize] = Some(Color::srgb(5, 99, 193));
            let mut map = ColorMap::default();
            map.slots[casual_slide_model::theme::ColorMapSlot::Hlink as usize] =
                Some(ColorSchemeIndex::Hlink);
            let env = TextEnv {
                colors: ResolveContext {
                    scheme: Some(&scheme),
                    map: Some(&map),
                    ..ResolveContext::default()
                },
                fonts: None,
            };
            let mut linked = props(2100, false, "Arial", Some(Color::srgb(0, 0, 0)));
            linked.hyperlink_click = Some(Box::new(Hyperlink {
                rel_id: Some("rId2".to_owned()),
                ..Hyperlink::default()
            }));
            let body = TextBody {
                properties: None,
                list_style: None,
                paragraphs: vec![Paragraph {
                    properties: None,
                    items: vec![
                        run("link", linked),
                        run(" plain", props(2100, false, "Arial", None)),
                    ],
                    end_properties: None,
                }],
            };
            let (resolved, _) = resolve_body(&body, &TextChain::default(), &env);
            let runs = &resolved.paragraphs[0].runs;
            let c = runs[0].style.color;
            assert_eq!((c.r, c.g, c.b), (5, 99, 193), "the theme's hlink, not tx1");
            assert_eq!(runs[0].style.underline, Some(UnderlineType::Sng));
            assert_eq!(runs[1].style.color, Rgba::BLACK);
            assert_eq!(runs[1].style.underline, None);
        }

        #[test]
        fn a_hyperlink_asking_for_the_text_colour_keeps_the_runs_own_fill() {
            use casual_slide_model::shape::{Hyperlink, HyperlinkColor};
            use casual_slide_model::theme::{ColorMap, ColorScheme, ColorSchemeIndex};
            let mut scheme = ColorScheme::default();
            scheme.slots[ColorSchemeIndex::Hlink as usize] = Some(Color::srgb(5, 99, 193));
            let mut map = ColorMap::default();
            map.slots[casual_slide_model::theme::ColorMapSlot::Hlink as usize] =
                Some(ColorSchemeIndex::Hlink);
            let env = TextEnv {
                colors: ResolveContext {
                    scheme: Some(&scheme),
                    map: Some(&map),
                    ..ResolveContext::default()
                },
                fonts: None,
            };
            let mut linked = props(2100, false, "Arial", Some(Color::srgb(200, 10, 20)));
            linked.hyperlink_click = Some(Box::new(Hyperlink {
                rel_id: Some("rId2".to_owned()),
                color: Some(HyperlinkColor::Text),
                ..Hyperlink::default()
            }));
            let body = TextBody {
                properties: None,
                list_style: None,
                paragraphs: vec![Paragraph {
                    properties: None,
                    items: vec![run("link", linked)],
                    end_properties: None,
                }],
            };
            let (resolved, _) = resolve_body(&body, &TextChain::default(), &env);
            let style = &resolved.paragraphs[0].runs[0].style;
            let c = style.color;
            assert_eq!(
                (c.r, c.g, c.b),
                (200, 10, 20),
                "the run's own fill, not hlink"
            );
            assert_eq!(
                style.underline,
                Some(UnderlineType::Sng),
                "still underlined"
            );
        }
    }
}
