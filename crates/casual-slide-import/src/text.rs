//! Read text bodies and list styles from an [`XmlTree`](casual_slide_ooxml::XmlTree)
//! (Tier 1F): `bodyPr`, `lstStyle`, paragraphs, runs, breaks, fields, and every run
//! and paragraph property.
//!
//! A value that can be used goes in the model, with its written form where the type
//! has several. One that cannot is reported and left out, and stays in the tree. What
//! other tiers still own (a picture bullet's content, `extLst`) is reported and stays in
//! the tree. A run's fill, line and effects, and a body's warp and 3D, go through Tier
//! 1E's readers, with their own bounds. A count over a bound in
//! [`TextLimits`] refuses the part. Nothing here recurses on input: a text body has a
//! fixed depth, and colours have their own bound.
//!
//! Text content never reaches a finding or an error: both carry element and attribute
//! names only, so `a:t`, `@typeface`, `@char` and the rest stay out of every report.

use std::collections::HashSet;

use casual_slide_model::presentation::{Coordinate, Percentage};
use casual_slide_model::properties::trim_xml;
use casual_slide_model::text::{
    Autofit, BULLET_SIZE_MAX, BULLET_SIZE_MIN, BULLET_START_MAX, BodyProperties, BooleanElement,
    Bullet, BulletColor, BulletFont, BulletSize, CharacterProperties, FONT_SCALE_MAX,
    FONT_SCALE_MIN, FONT_SIZE_MIN, LineBreak, Paragraph, ParagraphProperties, SPACING_PERCENT_MAX,
    SPACING_POINT_MAX, TEXT_COLUMNS_MAX, TEXT_LEVEL_MAX, TEXT_MARGIN_MAX, TEXT_POINT_MAX, TabStop,
    Text3D, TextBody, TextField, TextItem, TextListStyle, TextPoint, TextRun, TextSpacing,
    UnderlineFill, UnderlineLine, Written,
};
use casual_slide_ooxml::Relationship;

use crate::color::{Findings as ColorFindings, read_color_choice};
use crate::drawing::{Ctx, DrawingLimits, is_fill_element, read_effects, read_fill, read_line};
use crate::geometry::read_preset_text_shape;
use crate::presentation::read_text_font;
use crate::properties::{
    Elem, ImportError, ImportFindingKind as K, Reader, TextBound, is_guid, ns,
};
use crate::shape::read_hyperlink;

/// Bounds on what text can make this reader hold, on top of the XML bounds. Over a
/// bound, the part is refused. Each is far beyond a real deck: the largest committed
/// real deck is reported by `tools/real-decks`, and `docs/21` says what they cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextLimits {
    /// Paragraphs in one text body. `CS-TXT-0001`.
    pub max_paragraphs_per_body: usize,
    /// Runs, breaks and fields in one paragraph. `CS-TXT-0002`.
    pub max_runs_per_paragraph: usize,
    /// Paragraphs across one part. `CS-TXT-0003`.
    pub max_paragraphs_per_part: usize,
    /// Runs, breaks and fields across one part. `CS-TXT-0004`.
    pub max_runs_per_part: usize,
    /// Tab stops in one list. The schema allows 32. `CS-TXT-0005`.
    pub max_tab_stops: usize,
}

impl Default for TextLimits {
    fn default() -> Self {
        Self {
            max_paragraphs_per_body: 100_000,
            max_runs_per_paragraph: 100_000,
            max_paragraphs_per_part: 250_000,
            max_runs_per_part: 1_000_000,
            max_tab_stops: 1_024,
        }
    }
}

/// What a part has spent of the part-wide bounds so far.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TextCounts {
    pub paragraphs: usize,
    pub runs: usize,
}

/// One text body, list style or property being read.
pub(crate) struct TextReader<'a, 'r> {
    pub r: &'a mut Reader<'r>,
    pub rels: &'a [Relationship],
    pub color: &'a mut ColorFindings,
    pub counts: &'a mut TextCounts,
    pub drawing: DrawingLimits,
}

fn coordinate32(c: &Written<Coordinate>) -> bool {
    (i64::from(i32::MIN)..=i64::from(i32::MAX)).contains(&c.value().0)
}

fn percent_written(p: &Written<Percentage>) -> bool {
    p.written().is_some_and(|w| w.contains('%'))
}

/// `ST_TextFontScalePercentOrPercentString`: the range binds the decimal form, and
/// the percent form has no range in the schema.
fn font_scale(p: &Written<Percentage>) -> bool {
    percent_written(p) || (FONT_SCALE_MIN..=FONT_SCALE_MAX).contains(&p.value().0)
}

/// `ST_TextSpacingPercentOrPercentString`, the same way.
fn spacing_percent(p: &Written<Percentage>) -> bool {
    percent_written(p) || (0..=SPACING_PERCENT_MAX).contains(&p.value().0)
}

impl TextReader<'_, '_> {
    fn over(&self, bound: TextBound, limit: usize) -> ImportError {
        ImportError::TextBound {
            bound,
            part: self.r.part.to_owned(),
            limit,
        }
    }

    /// `a:CT_TextBody`, under whichever element holds it.
    pub fn body(&mut self, el: &Elem) -> Result<TextBody, ImportError> {
        self.r.finish(el);
        let limits = self.r.limits.text;
        // `bodyPr`, `lstStyle` and a little damage beside the paragraphs. The exact
        // count is below; this stops a hostile body costing a read of every child.
        let slack = limits.max_paragraphs_per_body.saturating_add(8);
        if self.r.tree.child_elements(el.id).len() > slack {
            return Err(self.over(TextBound::ParagraphsInBody, limits.max_paragraphs_per_body));
        }
        let mut body = TextBody::default();
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "p" => {
                    if body.paragraphs.len() >= limits.max_paragraphs_per_body {
                        return Err(
                            self.over(TextBound::ParagraphsInBody, limits.max_paragraphs_per_body)
                        );
                    }
                    self.counts.paragraphs = self.counts.paragraphs.saturating_add(1);
                    if self.counts.paragraphs > limits.max_paragraphs_per_part {
                        return Err(
                            self.over(TextBound::ParagraphsInPart, limits.max_paragraphs_per_part)
                        );
                    }
                    body.paragraphs.push(self.paragraph(&child)?);
                }
                name @ ("bodyPr" | "lstStyle") => {
                    if !seen.insert(name.to_owned()) {
                        self.r.note(K::DuplicateElement, &child, None);
                    } else if name == "bodyPr" {
                        body.properties = Some(self.body_properties(&child)?);
                    } else {
                        body.list_style = Some(self.list_style(&child)?);
                    }
                }
                _ => self.r.retained(&child),
            }
        }
        if body.properties.is_none() {
            self.r.note(K::MissingRequired, el, Some("bodyPr"));
        }
        if body.paragraphs.is_empty() {
            self.r.note(K::MissingRequired, el, Some("p"));
        }
        Ok(body)
    }

    /// `a:CT_TextBodyProperties`.
    pub fn body_properties(&mut self, el: &Elem) -> Result<BodyProperties, ImportError> {
        let mut e = el.clone();
        let drawing = self.drawing;
        let rels = self.rels;
        let r = &mut *self.r;
        let mut b = BodyProperties {
            rotation: r.attr(&mut e, "rot")?,
            space_first_last_paragraph: r.attr(&mut e, "spcFirstLastPara")?,
            vertical_overflow: r.attr(&mut e, "vertOverflow")?,
            horizontal_overflow: r.attr(&mut e, "horzOverflow")?,
            vertical: r.attr(&mut e, "vert")?,
            wrap: r.attr(&mut e, "wrap")?,
            inset_left: r.attr_in(&mut e, "lIns", coordinate32)?,
            inset_top: r.attr_in(&mut e, "tIns", coordinate32)?,
            inset_right: r.attr_in(&mut e, "rIns", coordinate32)?,
            inset_bottom: r.attr_in(&mut e, "bIns", coordinate32)?,
            columns: r.attr_in(&mut e, "numCol", |v: &u8| {
                (1..=TEXT_COLUMNS_MAX).contains(v)
            })?,
            column_spacing: r.attr_in(&mut e, "spcCol", |v: &i32| *v >= 0)?,
            rtl_columns: r.attr(&mut e, "rtlCol")?,
            from_word_art: r.attr(&mut e, "fromWordArt")?,
            anchor: r.attr(&mut e, "anchor")?,
            anchor_center: r.attr(&mut e, "anchorCtr")?,
            force_antialias: r.attr(&mut e, "forceAA")?,
            upright: r.attr(&mut e, "upright")?,
            compat_line_spacing: r.attr(&mut e, "compatLnSpc")?,
            warp: None,
            autofit: None,
            scene3d: None,
            text_3d: None,
        };
        r.finish(&e);
        // A property over a bound is left out, and is still one seen.
        let mut seen = HashSet::new();
        for child in r.children(el)? {
            if !child.in_namespace(ns::DML) {
                r.retained(&child);
                continue;
            }
            let slot = match child.local.as_str() {
                "sp3d" | "flatTx" => Some("text3d"),
                "prstTxWarp" => Some("warp"),
                "scene3d" => Some("scene3d"),
                _ => None,
            };
            if slot.is_some_and(|slot| !seen.insert(slot)) {
                r.note(K::DuplicateElement, &child, None);
                continue;
            }
            match child.local.as_str() {
                "prstTxWarp" => b.warp = read_preset_text_shape(r, drawing, &child)?,
                "scene3d" => {
                    let mut ctx = Ctx::new(&mut *r, rels, drawing);
                    b.scene3d = ctx.guard(&child, |c| c.scene3d(&child))?.map(Box::new);
                }
                "sp3d" => {
                    let mut ctx = Ctx::new(&mut *r, rels, drawing);
                    b.text_3d = ctx
                        .guard(&child, |c| c.sp3d(&child))?
                        .map(|s| Text3D::Shape(Box::new(s)));
                }
                "flatTx" => {
                    let mut ctx = Ctx::new(&mut *r, rels, drawing);
                    b.text_3d = ctx
                        .guard(&child, |c| c.flat_text(&child))?
                        .map(Text3D::Flat);
                }
                "noAutofit" | "normAutofit" | "spAutoFit" if b.autofit.is_some() => {
                    r.note(K::DuplicateElement, &child, None);
                }
                "noAutofit" => {
                    b.autofit = Some(Autofit::None);
                    finish_empty(r, &child)?;
                }
                "spAutoFit" => {
                    b.autofit = Some(Autofit::Shape);
                    finish_empty(r, &child)?;
                }
                "normAutofit" => {
                    let mut fit = child.clone();
                    b.autofit = Some(Autofit::Normal {
                        font_scale: r.attr_in(&mut fit, "fontScale", font_scale)?,
                        line_spacing_reduction: r.attr_in(
                            &mut fit,
                            "lnSpcReduction",
                            spacing_percent,
                        )?,
                    });
                    r.finish(&fit);
                    for inner in r.children(&child)? {
                        r.retained(&inner);
                    }
                }
                // `extLst` stays.
                _ => r.retained(&child),
            }
        }
        Ok(b)
    }

    /// `a:CT_TextListStyle`, under whichever element holds it.
    pub fn list_style(&mut self, el: &Elem) -> Result<TextListStyle, ImportError> {
        self.r.finish(el);
        let mut style = TextListStyle::default();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            let name = child.local.as_str();
            if name == "defPPr" {
                if style.default.is_some() {
                    self.r.note(K::DuplicateElement, &child, None);
                } else {
                    style.default = Some(Box::new(self.paragraph_properties(&child)?));
                }
                continue;
            }
            let level = (0..style.levels.len())
                .find(|l| TextListStyle::level_element(*l) == Some(name))
                .and_then(|l| style.levels.get_mut(l));
            match level {
                Some(slot) if slot.is_some() => self.r.note(K::DuplicateElement, &child, None),
                Some(slot) => *slot = Some(Box::new(Self::level_props(self, &child)?)),
                None => self.r.retained(&child),
            }
        }
        Ok(style)
    }

    fn level_props(&mut self, el: &Elem) -> Result<ParagraphProperties, ImportError> {
        self.paragraph_properties(el)
    }

    fn paragraph(&mut self, el: &Elem) -> Result<Paragraph, ImportError> {
        self.r.finish(el);
        let limits = self.r.limits.text;
        let slack = limits.max_runs_per_paragraph.saturating_add(8);
        if self.r.tree.child_elements(el.id).len() > slack {
            return Err(self.over(TextBound::RunsInParagraph, limits.max_runs_per_paragraph));
        }
        let mut p = Paragraph::default();
        let mut seen_properties = false;
        let mut seen_end = false;
        for child in self.r.children(el)? {
            let dml = child.in_namespace(ns::DML);
            match child.local.as_str() {
                "pPr" if dml => {
                    if seen_properties {
                        self.r.note(K::DuplicateElement, &child, None);
                    } else {
                        seen_properties = true;
                        p.properties = Some(Box::new(self.paragraph_properties(&child)?));
                    }
                    continue;
                }
                "endParaRPr" if dml => {
                    if seen_end {
                        self.r.note(K::DuplicateElement, &child, None);
                    } else {
                        seen_end = true;
                        p.end_properties = Some(Box::new(self.character(&child)?));
                    }
                    continue;
                }
                _ => {}
            }
            if p.items.len() >= limits.max_runs_per_paragraph {
                return Err(self.over(TextBound::RunsInParagraph, limits.max_runs_per_paragraph));
            }
            self.counts.runs = self.counts.runs.saturating_add(1);
            if self.counts.runs > limits.max_runs_per_part {
                return Err(self.over(TextBound::RunsInPart, limits.max_runs_per_part));
            }
            let item = match child.local.as_str() {
                "r" if dml => TextItem::Run(self.run(&child)?),
                "br" if dml => TextItem::Break(self.line_break(&child)?),
                "fld" if dml => TextItem::Field(self.field(&child)?),
                other => {
                    // Such as `mc:AlternateContent` around an equation. It is left in
                    // its place, and the model says that something is there.
                    self.r.retained(&child);
                    TextItem::Retained(other.to_owned())
                }
            };
            p.items.push(item);
        }
        Ok(p)
    }

    fn run(&mut self, el: &Elem) -> Result<TextRun, ImportError> {
        self.r.finish(el);
        let mut run = TextRun::default();
        let mut seen = HashSet::new();
        let mut has_text = false;
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) || !matches!(child.local.as_str(), "rPr" | "t") {
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(child.local.clone()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            if child.local == "rPr" {
                run.properties = Some(Box::new(self.character(&child)?));
            } else {
                has_text = true;
                self.r.finish(&child);
                run.text = self.r.text(&child)?;
            }
        }
        if !has_text {
            self.r.note(K::MissingRequired, el, Some("t"));
        }
        Ok(run)
    }

    fn line_break(&mut self, el: &Elem) -> Result<LineBreak, ImportError> {
        self.r.finish(el);
        let mut br = LineBreak::default();
        for child in self.r.children(el)? {
            if !child.is(ns::DML, "rPr") {
                self.r.retained(&child);
            } else if br.properties.is_some() {
                self.r.note(K::DuplicateElement, &child, None);
            } else {
                br.properties = Some(Box::new(self.character(&child)?));
            }
        }
        Ok(br)
    }

    fn field(&mut self, el: &Elem) -> Result<TextField, ImportError> {
        let mut e = el.clone();
        let mut field = TextField {
            id: self
                .r
                .required(&mut e, "id", |s: &String| is_guid(trim_xml(s)))?,
            field_type: self.r.attr(&mut e, "type")?,
            ..TextField::default()
        };
        self.r.finish(&e);
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) || !matches!(child.local.as_str(), "rPr" | "pPr" | "t")
            {
                self.r.retained(&child);
                continue;
            }
            if !seen.insert(child.local.clone()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            match child.local.as_str() {
                "rPr" => field.properties = Some(Box::new(self.character(&child)?)),
                "pPr" => {
                    field.paragraph_properties = Some(Box::new(self.paragraph_properties(&child)?));
                }
                _ => {
                    self.r.finish(&child);
                    field.text = self.r.text(&child)?;
                }
            }
        }
        Ok(field)
    }

    /// `a:CT_TextParagraphProperties`: `pPr`, or a list style level.
    pub fn paragraph_properties(&mut self, el: &Elem) -> Result<ParagraphProperties, ImportError> {
        let mut e = el.clone();
        let r = &mut *self.r;
        let margin = |v: &i32| (0..=TEXT_MARGIN_MAX).contains(v);
        let mut p = ParagraphProperties {
            margin_left: r.attr_in(&mut e, "marL", margin)?,
            margin_right: r.attr_in(&mut e, "marR", margin)?,
            level: r.attr_in(&mut e, "lvl", |v: &u8| *v <= TEXT_LEVEL_MAX)?,
            indent: r.attr_in(&mut e, "indent", |v: &i32| {
                (-TEXT_MARGIN_MAX..=TEXT_MARGIN_MAX).contains(v)
            })?,
            align: r.attr(&mut e, "algn")?,
            default_tab_size: r.attr_in(&mut e, "defTabSz", coordinate32)?,
            rtl: r.attr(&mut e, "rtl")?,
            east_asian_line_break: r.attr(&mut e, "eaLnBrk")?,
            font_align: r.attr(&mut e, "fontAlgn")?,
            latin_line_break: r.attr(&mut e, "latinLnBrk")?,
            hanging_punctuation: r.attr(&mut e, "hangingPunct")?,
            ..ParagraphProperties::default()
        };
        r.finish(&e);
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            let name = child.local.as_str();
            let group = match name {
                "lnSpc" | "spcBef" | "spcAft" | "tabLst" | "defRPr" => name,
                "buClrTx" | "buClr" => "colour",
                "buSzTx" | "buSzPct" | "buSzPts" => "size",
                "buFontTx" | "buFont" => "font",
                "buNone" | "buAutoNum" | "buChar" | "buBlip" => "bullet",
                _ => {
                    self.r.retained(&child);
                    continue;
                }
            };
            if !seen.insert(group.to_owned()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            match name {
                "lnSpc" => p.line_spacing = self.spacing(&child)?,
                "spcBef" => p.space_before = self.spacing(&child)?,
                "spcAft" => p.space_after = self.spacing(&child)?,
                "buClrTx" => {
                    finish_empty(self.r, &child)?;
                    p.bullet_color = Some(BulletColor::FollowText);
                }
                "buClr" => {
                    self.r.finish(&child);
                    p.bullet_color = Some(BulletColor::Color(read_color_choice(
                        self.r.tree,
                        child.id,
                        self.color,
                    )));
                }
                "buSzTx" => {
                    finish_empty(self.r, &child)?;
                    p.bullet_size = Some(BulletSize::FollowText);
                }
                "buSzPct" => {
                    let mut size = child.clone();
                    let value = self
                        .r
                        .required(&mut size, "val", |v: &Written<Percentage>| {
                            (BULLET_SIZE_MIN..=BULLET_SIZE_MAX).contains(&v.value().0)
                        })?;
                    self.r.finish(&size);
                    p.bullet_size = value.map(BulletSize::Percent);
                }
                "buSzPts" => {
                    let mut size = child.clone();
                    let value = self.r.required(&mut size, "val", |v: &i32| {
                        (FONT_SIZE_MIN..=TEXT_POINT_MAX).contains(v)
                    })?;
                    self.r.finish(&size);
                    p.bullet_size = value.map(BulletSize::Points);
                }
                "buFontTx" => {
                    finish_empty(self.r, &child)?;
                    p.bullet_font = Some(BulletFont::FollowText);
                }
                "buFont" => {
                    p.bullet_font = read_text_font(self.r, &child)?.map(BulletFont::Font);
                }
                "buNone" => {
                    finish_empty(self.r, &child)?;
                    p.bullet = Some(Bullet::None);
                }
                "buAutoNum" => {
                    let mut num = child.clone();
                    p.bullet = Some(Bullet::AutoNumber {
                        scheme: self.r.required(&mut num, "type", |_| true)?,
                        start_at: self.r.attr_in(&mut num, "startAt", |v: &i32| {
                            (1..=BULLET_START_MAX).contains(v)
                        })?,
                    });
                    self.r.finish(&num);
                }
                "buChar" => {
                    let mut ch = child.clone();
                    p.bullet = Some(Bullet::Char(self.r.required(&mut ch, "char", |_| true)?));
                    self.r.finish(&ch);
                }
                "buBlip" => p.bullet = Some(self.bullet_picture(&child)?),
                "tabLst" => p.tabs = Some(self.tabs(&child)?),
                _ => p.default_run = Some(Box::new(self.character(&child)?)),
            }
        }
        Ok(p)
    }

    fn bullet_picture(&mut self, el: &Elem) -> Result<Bullet, ImportError> {
        self.r.finish(el);
        let mut embed = None;
        let mut seen = false;
        for child in self.r.children(el)? {
            if !child.is(ns::DML, "blip") || seen {
                self.r.retained(&child);
                continue;
            }
            seen = true;
            let mut blip = child.clone();
            embed = blip.take_ns(ns::REL, "embed");
            if let Some(id) = &embed
                && !self.rels.iter().any(|r| r.id == *id)
            {
                self.r.note(K::DanglingRelationship, &blip, Some("r:embed"));
            }
            self.r.finish(&blip);
            for inner in self.r.children(&child)? {
                // Tier 1H: nothing here decodes a picture.
                self.r.retained(&inner);
            }
        }
        if !seen {
            self.r.note(K::MissingRequired, el, Some("blip"));
        }
        Ok(Bullet::Picture { embed })
    }

    /// `a:CT_TextSpacing`. `None`, reported, if it holds nothing usable.
    fn spacing(&mut self, el: &Elem) -> Result<Option<TextSpacing>, ImportError> {
        self.r.finish(el);
        let mut value = None;
        let mut seen = false;
        for child in self.r.children(el)? {
            let name = child.local.as_str();
            if !child.in_namespace(ns::DML) || !matches!(name, "spcPct" | "spcPts") {
                self.r.retained(&child);
                continue;
            }
            if seen {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            seen = true;
            let mut holder = child.clone();
            value = if name == "spcPct" {
                self.r
                    .required(&mut holder, "val", spacing_percent)?
                    .map(TextSpacing::Percent)
            } else {
                self.r
                    .required(&mut holder, "val", |v: &i32| {
                        (0..=SPACING_POINT_MAX).contains(v)
                    })?
                    .map(TextSpacing::Points)
            };
            self.r.finish(&holder);
        }
        if !seen {
            self.r.note(K::MissingRequired, el, None);
        }
        Ok(value)
    }

    fn tabs(&mut self, el: &Elem) -> Result<Vec<TabStop>, ImportError> {
        self.r.finish(el);
        let limit = self.r.limits.text.max_tab_stops;
        if self.r.tree.child_elements(el.id).len() > limit {
            return Err(self.over(TextBound::TabStops, limit));
        }
        let mut tabs = Vec::new();
        for child in self.r.children(el)? {
            if !child.is(ns::DML, "tab") {
                self.r.retained(&child);
                continue;
            }
            let mut tab = child.clone();
            tabs.push(TabStop {
                position: self.r.attr_in(&mut tab, "pos", coordinate32)?,
                align: self.r.attr(&mut tab, "algn")?,
            });
            self.r.finish(&tab);
        }
        if tabs.len() > 32 {
            // Over the schema's 32, under this reader's bound. Kept.
            self.r.note(K::OutOfRange, el, None);
        }
        Ok(tabs)
    }

    /// `a:CT_TextCharacterProperties`: `rPr`, `defRPr` or `endParaRPr`.
    pub fn character(&mut self, el: &Elem) -> Result<CharacterProperties, ImportError> {
        let mut e = el.clone();
        let r = &mut *self.r;
        let mut c = CharacterProperties {
            kumimoji: r.attr(&mut e, "kumimoji")?,
            lang: r.attr(&mut e, "lang")?,
            alt_lang: r.attr(&mut e, "altLang")?,
            size: r.attr_in(&mut e, "sz", |v: &i32| {
                (FONT_SIZE_MIN..=TEXT_POINT_MAX).contains(v)
            })?,
            bold: r.attr(&mut e, "b")?,
            italic: r.attr(&mut e, "i")?,
            underline: r.attr(&mut e, "u")?,
            strike: r.attr(&mut e, "strike")?,
            kerning: r.attr_in(&mut e, "kern", |v: &i32| (0..=TEXT_POINT_MAX).contains(v))?,
            caps: r.attr(&mut e, "cap")?,
            spacing: r.attr_in(&mut e, "spc", |v: &Written<TextPoint>| {
                (-TEXT_POINT_MAX..=TEXT_POINT_MAX).contains(&v.value().0)
            })?,
            normalize_height: r.attr(&mut e, "normalizeH")?,
            baseline: r.attr(&mut e, "baseline")?,
            no_proof: r.attr(&mut e, "noProof")?,
            dirty: r.attr(&mut e, "dirty")?,
            error: r.attr(&mut e, "err")?,
            smart_tag_clean: r.attr(&mut e, "smtClean")?,
            smart_tag_id: r.attr(&mut e, "smtId")?,
            bookmark: r.attr(&mut e, "bmk")?,
            ..CharacterProperties::default()
        };
        r.finish(&e);
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if !child.in_namespace(ns::DML) {
                self.r.retained(&child);
                continue;
            }
            let name = child.local.as_str();
            let group = match name {
                "ln" | "highlight" | "latin" | "ea" | "cs" | "sym" | "hlinkClick"
                | "hlinkMouseOver" | "rtl" => name,
                "noFill" | "solidFill" | "gradFill" | "blipFill" | "pattFill" | "grpFill" => "fill",
                "effectLst" | "effectDag" => "effects",
                "uLnTx" | "uLn" => "underline line",
                "uFillTx" | "uFill" => "underline fill",
                _ => {
                    self.r.retained(&child);
                    continue;
                }
            };
            if !seen.insert(group.to_owned()) {
                self.r.note(K::DuplicateElement, &child, None);
                continue;
            }
            match name {
                "ln" => {
                    c.line = read_line(self.r, self.rels, self.drawing, &child)?.map(Box::new);
                }
                "noFill" | "solidFill" | "gradFill" | "blipFill" | "pattFill" | "grpFill" => {
                    c.fill = read_fill(self.r, self.rels, self.drawing, &child)?;
                }
                "effectLst" | "effectDag" => {
                    c.effects =
                        read_effects(self.r, self.rels, self.drawing, &child)?.map(Box::new);
                }
                "highlight" => {
                    self.r.finish(&child);
                    c.highlight = Some(read_color_choice(self.r.tree, child.id, self.color));
                }
                "uLnTx" => {
                    finish_empty(self.r, &child)?;
                    c.underline_line = Some(UnderlineLine::FollowText);
                }
                "uLn" => {
                    c.underline_line = read_line(self.r, self.rels, self.drawing, &child)?
                        .map(|line| UnderlineLine::Line(Box::new(line)));
                }
                "uFillTx" => {
                    finish_empty(self.r, &child)?;
                    c.underline_fill = Some(UnderlineFill::FollowText);
                }
                "uFill" => {
                    self.r.finish(&child);
                    let mut fill = None;
                    for inner in self.r.children(&child)? {
                        if !is_fill_element(&inner) {
                            self.r.retained(&inner);
                        } else if fill.is_some() {
                            self.r.note(K::DuplicateElement, &inner, None);
                        } else {
                            fill = read_fill(self.r, self.rels, self.drawing, &inner)?;
                        }
                    }
                    c.underline_fill = Some(UnderlineFill::Fill(fill));
                }
                "latin" => c.latin = read_text_font(self.r, &child)?.map(Box::new),
                "ea" => c.east_asian = read_text_font(self.r, &child)?.map(Box::new),
                "cs" => c.complex_script = read_text_font(self.r, &child)?.map(Box::new),
                "sym" => c.symbol = read_text_font(self.r, &child)?.map(Box::new),
                "hlinkClick" => {
                    c.hyperlink_click = Some(Box::new(read_hyperlink(self.r, self.rels, &child)?));
                }
                "hlinkMouseOver" => {
                    c.hyperlink_mouse_over =
                        Some(Box::new(read_hyperlink(self.r, self.rels, &child)?));
                }
                _ => {
                    let mut flag = child.clone();
                    c.rtl = Some(BooleanElement {
                        value: self.r.attr(&mut flag, "val")?,
                    });
                    self.r.finish(&flag);
                }
            }
        }
        Ok(c)
    }
}

/// An element the schema leaves empty. Whatever it holds is reported.
fn finish_empty(r: &mut Reader<'_>, el: &Elem) -> Result<(), ImportError> {
    r.finish(el);
    for child in r.children(el)? {
        r.retained(&child);
    }
    Ok(())
}
