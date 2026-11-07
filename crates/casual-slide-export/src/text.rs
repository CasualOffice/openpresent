//! Patch text bodies and list styles in place (ADR-013, Tier 1F).
//!
//! Each element is compared with the model as read, and only what differs is touched.
//! Paragraphs, runs and tab stops match by position, and the tail may grow or shrink.
//! So what a run holds beyond the model stays with it, and a retained element, such as
//! an equation's `mc:AlternateContent`, stays where it is.
//!
//! A run's fill, line, effects and underline, and a body's warp and 3D, go through Tier
//! 1E's writer. A picture bullet can be kept or removed, and not made from here: asking for one is refused as
//! `CS-EXP-0009`, and nothing is saved.
//! A change that would leave required content out (`bodyPr`, the last `p`, `a:t`) is
//! refused the same way.

use casual_slide_import::properties::{Elem, ns};
use casual_slide_model::color::Color;
use casual_slide_model::d3::{FlatText, Shape3D};
use casual_slide_model::effect::EffectProperties;
use casual_slide_model::fill::Fill;
use casual_slide_model::line::Line;
use casual_slide_model::presentation::Percentage;
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::Hyperlink;
use casual_slide_model::text::{
    Autofit, BodyProperties, BooleanElement, Bullet, BulletColor, BulletFont, BulletSize,
    CharacterProperties, LineBreak, Paragraph, ParagraphProperties, TabStop, Text3D, TextBody,
    TextField, TextItem, TextListStyle, TextRun, TextSpacing, UnderlineFill, UnderlineLine,
    Written,
};

use crate::d3::{flat_text_node, scene3d_node, sp3d_node};
use crate::drawing::{Space, patch};
use crate::effect::effect_properties_node;
use crate::fill::fill_node;
use crate::geometry::warp_node;
use crate::line::line_node;
use crate::presentation::patch_text_font;
use crate::properties::{ExportError, Writer};
use crate::slide::{patch_hyperlink, sync_ordered};

type Result<T = ()> = std::result::Result<T, ExportError>;

const BODY_ORDER: &[&str] = &["bodyPr", "lstStyle", "p"];
const BODY_PROPERTIES_ORDER: &[&str] = &[
    "prstTxWarp",
    "noAutofit",
    "normAutofit",
    "spAutoFit",
    "scene3d",
    "sp3d",
    "flatTx",
    "extLst",
];
const LIST_ORDER: &[&str] = &[
    "defPPr", "lvl1pPr", "lvl2pPr", "lvl3pPr", "lvl4pPr", "lvl5pPr", "lvl6pPr", "lvl7pPr",
    "lvl8pPr", "lvl9pPr", "extLst",
];
const PARAGRAPH_PROPERTIES_ORDER: &[&str] = &[
    "lnSpc",
    "spcBef",
    "spcAft",
    "buClrTx",
    "buClr",
    "buSzTx",
    "buSzPct",
    "buSzPts",
    "buFontTx",
    "buFont",
    "buNone",
    "buAutoNum",
    "buChar",
    "buBlip",
    "tabLst",
    "defRPr",
    "extLst",
];
const CHARACTER_ORDER: &[&str] = &[
    "ln",
    "noFill",
    "solidFill",
    "gradFill",
    "blipFill",
    "pattFill",
    "grpFill",
    "effectLst",
    "effectDag",
    "highlight",
    "uLnTx",
    "uLn",
    "uFillTx",
    "uFill",
    "latin",
    "ea",
    "cs",
    "sym",
    "hlinkClick",
    "hlinkMouseOver",
    "rtl",
    "extLst",
];
const RUN_ORDER: &[&str] = &["rPr", "t"];
const FIELD_ORDER: &[&str] = &["rPr", "pPr", "t"];
const PARAGRAPH_ORDER: &[&str] = &["pPr", "r", "br", "fld", "endParaRPr"];
const FILLS: &[&str] = &[
    "noFill",
    "solidFill",
    "gradFill",
    "blipFill",
    "pattFill",
    "grpFill",
];

fn unsupported(w: &Writer<'_>, what: &'static str) -> ExportError {
    ExportError::UnsupportedEdit {
        what,
        part: w.part.to_owned(),
    }
}

/// A child in this choice group holding `wanted`, made if the group has none of that
/// name. Every other member goes.
fn swap_choice(
    w: &mut Writer<'_>,
    parent: &Elem,
    group: &[&str],
    order: &[&str],
    wanted: &str,
) -> Result<Elem> {
    for child in parent.children(w.tree, w.part)? {
        if child.in_namespace(ns::DML) && child.local != wanted && group.contains(&&*child.local) {
            w.tree.remove_child(parent.id, child.id);
            w.changes += 1;
        }
    }
    if let Some(found) = parent.first_child(w.tree, w.part, ns::DML, wanted)? {
        return Ok(found);
    }
    let el = w.new_child(parent, ns::DML, "a", wanted)?;
    let rank = |name: &str| order.iter().position(|o| *o == name);
    let mine = rank(wanted);
    let landmark = parent
        .children(w.tree, w.part)?
        .into_iter()
        .find(|c| {
            c.in_namespace(ns::DML)
                && matches!((rank(&c.local), mine), (Some(theirs), Some(mine)) if theirs > mine)
        })
        .map(|c| c.id);
    w.tree.insert_child(parent.id, landmark, el.id);
    Ok(el)
}

fn remove_group(w: &mut Writer<'_>, parent: &Elem, group: &[&str]) -> Result {
    for name in group {
        w.remove_all(parent, ns::DML, name)?;
    }
    Ok(())
}

fn set_color(w: &mut Writer<'_>, holder: &Elem, color: &Color) -> Result {
    let changed = crate::color::set_color(w.tree, holder.id, color).map_err(|_| {
        ExportError::MalformedElement {
            part: w.part.to_owned(),
        }
    })?;
    w.changes += usize::from(changed);
    Ok(())
}

/// A percentage attribute: as the value was written if it was, else in the form the
/// element already uses, or `N%` in Strict.
fn percent_attr(
    w: &mut Writer<'_>,
    el: &Elem,
    name: &str,
    before: Option<&Written<Percentage>>,
    after: Option<&Written<Percentage>>,
) -> Result {
    if before == after {
        return Ok(());
    }
    let Some(value) = after else {
        return w.attr::<Written<Percentage>>(el, name, before, None);
    };
    let percent_form =
        value.written().is_none() && (w.strict || el.get(name).is_some_and(|v| v.contains('%')));
    let text = if percent_form {
        value.value().to_percent_form()
    } else {
        value.to_lexical()
    };
    // `before` differs, so a stated "changed" is what is being said.
    w.attr(el, name, Some(&String::new()), Some(&text))
}

/// `buSzPct/@val`. The schema has only the `N%` form, in Transitional too, whatever
/// else a file may hold; a value with no written form yet is spelled that way.
fn bullet_percent(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&Written<Percentage>>,
    after: &Written<Percentage>,
) -> Result {
    if before == Some(after) {
        return Ok(());
    }
    let text = after
        .written()
        .map_or_else(|| after.value().to_percent_form(), str::to_owned);
    w.attr(el, "val", Some(&String::new()), Some(&text))
}

/// `a:CT_TextBody`, under whichever element holds it.
pub(crate) fn patch_text_body(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&TextBody>,
    a: &TextBody,
) -> Result {
    if a.properties.is_none() && (b.is_none() || b.is_some_and(|b| b.properties.is_some())) {
        return Err(unsupported(w, "a text body with no bodyPr"));
    }
    if a.paragraphs.is_empty() && !b.is_some_and(|b| b.paragraphs.is_empty()) {
        return Err(unsupported(w, "a text body with no paragraph"));
    }
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        BODY_ORDER,
        "bodyPr",
        b.and_then(|b| b.properties.as_ref()),
        a.properties.as_ref(),
        patch_body_properties,
    )?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        BODY_ORDER,
        "lstStyle",
        b.and_then(|b| b.list_style.as_ref()),
        a.list_style.as_ref(),
        patch_list_style,
    )?;
    let before: &[Paragraph] = b.map_or(&[], |b| &b.paragraphs);
    if before == a.paragraphs.as_slice() {
        return Ok(());
    }
    let elements: Vec<Elem> = el
        .children(w.tree, w.part)?
        .into_iter()
        .filter(|c| c.is(ns::DML, "p"))
        .collect();
    if elements.len() != before.len() {
        return Err(ExportError::MalformedElement {
            part: w.part.to_owned(),
        });
    }
    for (at, wanted) in a.paragraphs.iter().enumerate() {
        match (elements.get(at), before.get(at)) {
            (Some(p), Some(old)) => {
                if old != wanted {
                    patch_paragraph(w, p, Some(old), wanted)?;
                }
            }
            _ => {
                let p = w.new_child(el, ns::DML, "a", "p")?;
                w.tree.insert_child(el.id, None, p.id);
                patch_paragraph(w, &p, None, wanted)?;
            }
        }
    }
    for extra in elements.iter().skip(a.paragraphs.len()) {
        w.tree.remove_child(el.id, extra.id);
        w.changes += 1;
    }
    Ok(())
}

/// `a:CT_TextBodyProperties`.
fn patch_body_properties(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&BodyProperties>,
    a: &BodyProperties,
) -> Result {
    macro_rules! attrs {
        ($($name:literal => $field:ident),+ $(,)?) => {$(
            w.attr(el, $name, b.and_then(|b| b.$field.as_ref()), a.$field.as_ref())?;
        )+};
    }
    attrs!(
        "rot" => rotation,
        "spcFirstLastPara" => space_first_last_paragraph,
        "vertOverflow" => vertical_overflow,
        "horzOverflow" => horizontal_overflow,
        "vert" => vertical,
        "wrap" => wrap,
        "lIns" => inset_left,
        "tIns" => inset_top,
        "rIns" => inset_right,
        "bIns" => inset_bottom,
        "numCol" => columns,
        "spcCol" => column_spacing,
        "rtlCol" => rtl_columns,
        "fromWordArt" => from_word_art,
        "anchor" => anchor,
        "anchorCtr" => anchor_center,
        "forceAA" => force_antialias,
        "upright" => upright,
        "compatLnSpc" => compat_line_spacing,
    );
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        BODY_PROPERTIES_ORDER,
        "prstTxWarp",
        b.and_then(|b| b.warp.as_ref()),
        a.warp.as_ref(),
        |w, el, before, after| patch(w, el, before.map(warp_node).as_ref(), &warp_node(after)),
    )?;
    let before = b.and_then(|b| b.autofit.as_ref());
    if before != a.autofit.as_ref() {
        const GROUP: &[&str] = &["noAutofit", "normAutofit", "spAutoFit"];
        match &a.autofit {
            None => remove_group(w, el, GROUP)?,
            Some(Autofit::None) => {
                swap_choice(w, el, GROUP, BODY_PROPERTIES_ORDER, "noAutofit")?;
            }
            Some(Autofit::Shape) => {
                swap_choice(w, el, GROUP, BODY_PROPERTIES_ORDER, "spAutoFit")?;
            }
            Some(Autofit::Normal {
                font_scale,
                line_spacing_reduction,
            }) => {
                let fit = swap_choice(w, el, GROUP, BODY_PROPERTIES_ORDER, "normAutofit")?;
                let (old_scale, old_reduction) = match before {
                    Some(Autofit::Normal {
                        font_scale,
                        line_spacing_reduction,
                    }) => (font_scale.as_ref(), line_spacing_reduction.as_ref()),
                    _ => (None, None),
                };
                percent_attr(w, &fit, "fontScale", old_scale, font_scale.as_ref())?;
                percent_attr(
                    w,
                    &fit,
                    "lnSpcReduction",
                    old_reduction,
                    line_spacing_reduction.as_ref(),
                )?;
            }
        }
    }
    patch_body_3d(w, el, b, a)
}

fn text_3d_shape(t: Option<&Text3D>) -> Option<&Shape3D> {
    match t {
        Some(Text3D::Shape(s)) => Some(s),
        _ => None,
    }
}

fn text_3d_flat(t: Option<&Text3D>) -> Option<FlatText> {
    match t {
        Some(Text3D::Flat(f)) => Some(*f),
        _ => None,
    }
}

/// `scene3d` and the `EG_Text3D` choice. One the model could not hold (over a bound)
/// goes when the choice is made the other way, or the reader would find both.
fn patch_body_3d(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&BodyProperties>,
    a: &BodyProperties,
) -> Result {
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        BODY_PROPERTIES_ORDER,
        "scene3d",
        b.and_then(|b| b.scene3d.as_deref()),
        a.scene3d.as_deref(),
        |w, el, before, after| {
            let node = |s| scene3d_node(s, Space::Dml, "scene3d");
            patch(w, el, before.map(node).as_ref(), &node(after))
        },
    )?;
    let before = b.and_then(|b| b.text_3d.as_ref());
    if before == a.text_3d.as_ref() {
        return Ok(());
    }
    match a.text_3d {
        Some(Text3D::Shape(_)) => w.remove_all(el, ns::DML, "flatTx")?,
        Some(Text3D::Flat(_)) => w.remove_all(el, ns::DML, "sp3d")?,
        None => {}
    }
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        BODY_PROPERTIES_ORDER,
        "sp3d",
        text_3d_shape(before),
        text_3d_shape(a.text_3d.as_ref()),
        |w, el, before, after| {
            let node = |s| sp3d_node(s, Space::Dml, "sp3d");
            patch(w, el, before.map(node).as_ref(), &node(after))
        },
    )?;
    let (old, new) = (text_3d_flat(before), text_3d_flat(a.text_3d.as_ref()));
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        BODY_PROPERTIES_ORDER,
        "flatTx",
        old.as_ref(),
        new.as_ref(),
        |w, el, before, after| {
            patch(
                w,
                el,
                before.map(flat_text_node).as_ref(),
                &flat_text_node(after),
            )
        },
    )
}

/// `a:CT_TextListStyle`, under whichever element holds it. Its children are in
/// DrawingML whatever namespace the holder is in.
pub(crate) fn patch_list_style(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&TextListStyle>,
    a: &TextListStyle,
) -> Result {
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        LIST_ORDER,
        "defPPr",
        b.and_then(|b| b.default.as_ref()),
        a.default.as_ref(),
        boxed_paragraph_properties,
    )?;
    for (at, wanted) in a.levels.iter().enumerate() {
        let Some(name) = TextListStyle::level_element(at) else {
            continue;
        };
        sync_ordered(
            w,
            el,
            ns::DML,
            "a",
            LIST_ORDER,
            name,
            b.and_then(|b| b.levels.get(at)).and_then(Option::as_ref),
            wanted.as_ref(),
            boxed_paragraph_properties,
        )?;
    }
    Ok(())
}

// `sync_ordered`'s value type is the field's own, `Box` included.
#[allow(clippy::borrowed_box)]
fn boxed_paragraph_properties(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&Box<ParagraphProperties>>,
    a: &Box<ParagraphProperties>,
) -> Result {
    patch_paragraph_properties(w, el, b.map(|b| &**b), a)
}

// `sync_ordered`'s value type is the field's own, `Box` included.
#[allow(clippy::borrowed_box)]
fn boxed_character(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&Box<CharacterProperties>>,
    a: &Box<CharacterProperties>,
) -> Result {
    patch_character(w, el, b.map(|b| &**b), a)
}

/// `a:CT_TextParagraphProperties`.
fn patch_paragraph_properties(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&ParagraphProperties>,
    a: &ParagraphProperties,
) -> Result {
    macro_rules! attrs {
        ($($name:literal => $field:ident),+ $(,)?) => {$(
            w.attr(el, $name, b.and_then(|b| b.$field.as_ref()), a.$field.as_ref())?;
        )+};
    }
    attrs!(
        "marL" => margin_left,
        "marR" => margin_right,
        "lvl" => level,
        "indent" => indent,
        "algn" => align,
        "defTabSz" => default_tab_size,
        "rtl" => rtl,
        "eaLnBrk" => east_asian_line_break,
        "fontAlgn" => font_align,
        "latinLnBrk" => latin_line_break,
        "hangingPunct" => hanging_punctuation,
    );
    for (name, before, after) in [
        (
            "lnSpc",
            b.and_then(|b| b.line_spacing.as_ref()),
            a.line_spacing.as_ref(),
        ),
        (
            "spcBef",
            b.and_then(|b| b.space_before.as_ref()),
            a.space_before.as_ref(),
        ),
        (
            "spcAft",
            b.and_then(|b| b.space_after.as_ref()),
            a.space_after.as_ref(),
        ),
    ] {
        sync_ordered(
            w,
            el,
            ns::DML,
            "a",
            PARAGRAPH_PROPERTIES_ORDER,
            name,
            before,
            after,
            patch_spacing,
        )?;
    }
    patch_bullet_color(
        w,
        el,
        b.and_then(|b| b.bullet_color.as_ref()),
        a.bullet_color.as_ref(),
    )?;
    patch_bullet_size(
        w,
        el,
        b.and_then(|b| b.bullet_size.as_ref()),
        a.bullet_size.as_ref(),
    )?;
    patch_bullet_font(
        w,
        el,
        b.and_then(|b| b.bullet_font.as_ref()),
        a.bullet_font.as_ref(),
    )?;
    patch_bullet(w, el, b.and_then(|b| b.bullet.as_ref()), a.bullet.as_ref())?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        PARAGRAPH_PROPERTIES_ORDER,
        "tabLst",
        b.and_then(|b| b.tabs.as_ref()),
        a.tabs.as_ref(),
        patch_tabs,
    )?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        PARAGRAPH_PROPERTIES_ORDER,
        "defRPr",
        b.and_then(|b| b.default_run.as_ref()),
        a.default_run.as_ref(),
        boxed_character,
    )
}

fn patch_spacing(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&TextSpacing>,
    a: &TextSpacing,
) -> Result {
    const GROUP: &[&str] = &["spcPct", "spcPts"];
    match a {
        TextSpacing::Percent(value) => {
            let holder = swap_choice(w, el, GROUP, GROUP, "spcPct")?;
            let old = match b {
                Some(TextSpacing::Percent(old)) => Some(old),
                _ => None,
            };
            percent_attr(w, &holder, "val", old, Some(value))
        }
        TextSpacing::Points(value) => {
            let holder = swap_choice(w, el, GROUP, GROUP, "spcPts")?;
            let old = match b {
                Some(TextSpacing::Points(old)) => Some(old),
                _ => None,
            };
            w.attr(&holder, "val", old, Some(value))
        }
    }
}

fn patch_bullet_color(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&BulletColor>,
    a: Option<&BulletColor>,
) -> Result {
    const GROUP: &[&str] = &["buClrTx", "buClr"];
    if b == a {
        return Ok(());
    }
    match a {
        None => remove_group(w, el, GROUP),
        Some(BulletColor::FollowText) => {
            swap_choice(w, el, GROUP, PARAGRAPH_PROPERTIES_ORDER, "buClrTx").map(|_| ())
        }
        Some(BulletColor::Color(Some(color))) => {
            let holder = swap_choice(w, el, GROUP, PARAGRAPH_PROPERTIES_ORDER, "buClr")?;
            set_color(w, &holder, color)
        }
        Some(BulletColor::Color(None)) => Err(unsupported(w, "a bullet colour with no colour")),
    }
}

fn patch_bullet_size(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&BulletSize>,
    a: Option<&BulletSize>,
) -> Result {
    const GROUP: &[&str] = &["buSzTx", "buSzPct", "buSzPts"];
    if b == a {
        return Ok(());
    }
    match a {
        None => remove_group(w, el, GROUP),
        Some(BulletSize::FollowText) => {
            swap_choice(w, el, GROUP, PARAGRAPH_PROPERTIES_ORDER, "buSzTx").map(|_| ())
        }
        Some(BulletSize::Percent(value)) => {
            let holder = swap_choice(w, el, GROUP, PARAGRAPH_PROPERTIES_ORDER, "buSzPct")?;
            let old = match b {
                Some(BulletSize::Percent(old)) => Some(old),
                _ => None,
            };
            bullet_percent(w, &holder, old, value)
        }
        Some(BulletSize::Points(value)) => {
            let holder = swap_choice(w, el, GROUP, PARAGRAPH_PROPERTIES_ORDER, "buSzPts")?;
            let old = match b {
                Some(BulletSize::Points(old)) => Some(old),
                _ => None,
            };
            w.attr(&holder, "val", old, Some(value))
        }
    }
}

fn patch_bullet_font(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&BulletFont>,
    a: Option<&BulletFont>,
) -> Result {
    const GROUP: &[&str] = &["buFontTx", "buFont"];
    if b == a {
        return Ok(());
    }
    match a {
        None => remove_group(w, el, GROUP),
        Some(BulletFont::FollowText) => {
            swap_choice(w, el, GROUP, PARAGRAPH_PROPERTIES_ORDER, "buFontTx").map(|_| ())
        }
        Some(BulletFont::Font(font)) => {
            let holder = swap_choice(w, el, GROUP, PARAGRAPH_PROPERTIES_ORDER, "buFont")?;
            let old = match b {
                Some(BulletFont::Font(old)) => Some(old),
                _ => None,
            };
            patch_text_font(w, &holder, old, font)
        }
    }
}

fn patch_bullet(w: &mut Writer<'_>, el: &Elem, b: Option<&Bullet>, a: Option<&Bullet>) -> Result {
    const GROUP: &[&str] = &["buNone", "buAutoNum", "buChar", "buBlip"];
    if b == a {
        return Ok(());
    }
    match a {
        None => remove_group(w, el, GROUP),
        Some(Bullet::None) => {
            swap_choice(w, el, GROUP, PARAGRAPH_PROPERTIES_ORDER, "buNone").map(|_| ())
        }
        Some(Bullet::AutoNumber { scheme, start_at }) => {
            let holder = swap_choice(w, el, GROUP, PARAGRAPH_PROPERTIES_ORDER, "buAutoNum")?;
            let (old_scheme, old_start) = match b {
                Some(Bullet::AutoNumber { scheme, start_at }) => {
                    (scheme.as_ref(), start_at.as_ref())
                }
                _ => (None, None),
            };
            if scheme.is_none() {
                return Err(unsupported(w, "an automatic number with no scheme"));
            }
            w.attr(&holder, "type", old_scheme, scheme.as_ref())?;
            w.attr(&holder, "startAt", old_start, start_at.as_ref())
        }
        Some(Bullet::Char(value)) => {
            let holder = swap_choice(w, el, GROUP, PARAGRAPH_PROPERTIES_ORDER, "buChar")?;
            if value.is_none() {
                return Err(unsupported(w, "a character bullet with no character"));
            }
            let old = match b {
                Some(Bullet::Char(old)) => old.as_ref(),
                _ => None,
            };
            w.attr(&holder, "char", old, value.as_ref())
        }
        // Same picture is caught by `b == a`; another means writing Tier 1H's element.
        Some(Bullet::Picture { .. }) => Err(unsupported(w, "a picture bullet")),
    }
}

// `sync_ordered`'s value type is the field's own, `Vec` included.
#[allow(clippy::ptr_arg)]
fn patch_tabs(w: &mut Writer<'_>, el: &Elem, b: Option<&Vec<TabStop>>, a: &Vec<TabStop>) -> Result {
    let elements: Vec<Elem> = el
        .children(w.tree, w.part)?
        .into_iter()
        .filter(|c| c.is(ns::DML, "tab"))
        .collect();
    let before: &[TabStop] = b.map_or(&[], Vec::as_slice);
    for (at, wanted) in a.iter().enumerate() {
        let (tab, old) = match (elements.get(at), before.get(at)) {
            (Some(tab), old) => (tab.clone(), old),
            (None, _) => {
                let tab = w.new_child(el, ns::DML, "a", "tab")?;
                w.tree.insert_child(el.id, None, tab.id);
                (tab, None)
            }
        };
        w.attr(
            &tab,
            "pos",
            old.and_then(|o| o.position.as_ref()),
            wanted.position.as_ref(),
        )?;
        w.attr(
            &tab,
            "algn",
            old.and_then(|o| o.align.as_ref()),
            wanted.align.as_ref(),
        )?;
    }
    for extra in elements.iter().skip(a.len()) {
        w.tree.remove_child(el.id, extra.id);
        w.changes += 1;
    }
    Ok(())
}

fn fill_element(fill: &Fill) -> &'static str {
    match fill {
        Fill::None => "noFill",
        Fill::Solid(_) => "solidFill",
        Fill::Gradient(_) => "gradFill",
        Fill::Blip(_) => "blipFill",
        Fill::Pattern(_) => "pattFill",
        Fill::Group => "grpFill",
    }
}

/// A fill among `order`'s children, by Tier 1E's writer. A fill of another kind replaces
/// it; one of the same kind is patched, so what it holds beyond the model stays.
fn patch_fill(
    w: &mut Writer<'_>,
    el: &Elem,
    order: &[&str],
    b: Option<&Fill>,
    a: Option<&Fill>,
) -> Result {
    if b == a {
        return Ok(());
    }
    let Some(wanted) = a else {
        return remove_group(w, el, FILLS);
    };
    let local = fill_element(wanted);
    let holder = swap_choice(w, el, FILLS, order, local)?;
    let before = b.filter(|old| fill_element(old) == local).map(fill_node);
    patch(w, &holder, before.as_ref(), &fill_node(wanted))
}

fn patch_line(
    w: &mut Writer<'_>,
    el: &Elem,
    local: &'static str,
    b: Option<&Line>,
    a: &Line,
) -> Result {
    let before = b.map(|old| line_node(old, Space::Dml, local));
    patch(w, el, before.as_ref(), &line_node(a, Space::Dml, local))
}

fn patch_effects(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&EffectProperties>,
    a: Option<&EffectProperties>,
) -> Result {
    const GROUP: &[&str] = &["effectLst", "effectDag"];
    if b == a {
        return Ok(());
    }
    let Some(wanted) = a else {
        return remove_group(w, el, GROUP);
    };
    let element = |e: &EffectProperties| match e {
        EffectProperties::List(_) => "effectLst",
        EffectProperties::Dag(_) => "effectDag",
    };
    let local = element(wanted);
    let holder = swap_choice(w, el, GROUP, CHARACTER_ORDER, local)?;
    let before = b
        .filter(|old| element(old) == local)
        .map(effect_properties_node);
    patch(w, &holder, before.as_ref(), &effect_properties_node(wanted))
}

/// `a:CT_TextCharacterProperties`.
fn patch_character(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&CharacterProperties>,
    a: &CharacterProperties,
) -> Result {
    macro_rules! attrs {
        ($($name:literal => $field:ident),+ $(,)?) => {$(
            w.attr(el, $name, b.and_then(|b| b.$field.as_ref()), a.$field.as_ref())?;
        )+};
    }
    attrs!(
        "kumimoji" => kumimoji,
        "lang" => lang,
        "altLang" => alt_lang,
        "sz" => size,
        "b" => bold,
        "i" => italic,
        "u" => underline,
        "strike" => strike,
        "kern" => kerning,
        "cap" => caps,
        "spc" => spacing,
        "normalizeH" => normalize_height,
        "noProof" => no_proof,
        "dirty" => dirty,
        "err" => error,
        "smtClean" => smart_tag_clean,
        "smtId" => smart_tag_id,
        "bmk" => bookmark,
    );
    percent_attr(
        w,
        el,
        "baseline",
        b.and_then(|b| b.baseline.as_ref()),
        a.baseline.as_ref(),
    )?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        CHARACTER_ORDER,
        "ln",
        b.and_then(|b| b.line.as_ref()),
        a.line.as_ref(),
        |w, el, before, after| patch_line(w, el, "ln", before.map(|b| &**b), after),
    )?;
    patch_fill(
        w,
        el,
        CHARACTER_ORDER,
        b.and_then(|b| b.fill.as_ref()),
        a.fill.as_ref(),
    )?;
    patch_effects(
        w,
        el,
        b.and_then(|b| b.effects.as_deref()),
        a.effects.as_deref(),
    )?;
    patch_highlight(
        w,
        el,
        b.and_then(|b| b.highlight.as_ref()),
        a.highlight.as_ref(),
    )?;
    patch_underline(w, el, b, a)?;
    for (name, before, after) in [
        ("latin", b.and_then(|b| b.latin.as_ref()), a.latin.as_ref()),
        (
            "ea",
            b.and_then(|b| b.east_asian.as_ref()),
            a.east_asian.as_ref(),
        ),
        (
            "cs",
            b.and_then(|b| b.complex_script.as_ref()),
            a.complex_script.as_ref(),
        ),
        ("sym", b.and_then(|b| b.symbol.as_ref()), a.symbol.as_ref()),
    ] {
        sync_ordered(
            w,
            el,
            ns::DML,
            "a",
            CHARACTER_ORDER,
            name,
            before,
            after,
            |w, el, before, after| patch_text_font(w, el, before.map(|b| &**b), after),
        )?;
    }
    for (name, before, after) in [
        (
            "hlinkClick",
            b.and_then(|b| b.hyperlink_click.as_ref()),
            a.hyperlink_click.as_ref(),
        ),
        (
            "hlinkMouseOver",
            b.and_then(|b| b.hyperlink_mouse_over.as_ref()),
            a.hyperlink_mouse_over.as_ref(),
        ),
    ] {
        sync_ordered(
            w,
            el,
            ns::DML,
            "a",
            CHARACTER_ORDER,
            name,
            before,
            after,
            patch_run_hyperlink,
        )?;
    }
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        CHARACTER_ORDER,
        "rtl",
        b.and_then(|b| b.rtl.as_ref()),
        a.rtl.as_ref(),
        |w, el, before: Option<&BooleanElement>, after| {
            w.attr(
                el,
                "val",
                before.and_then(|b| b.value.as_ref()),
                after.value.as_ref(),
            )
        },
    )
}

// `sync_ordered`'s value type is the field's own, `Box` included.
#[allow(clippy::borrowed_box)]
fn patch_run_hyperlink(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&Box<Hyperlink>>,
    a: &Box<Hyperlink>,
) -> Result {
    if let Some(before) = b {
        return patch_hyperlink(w, el, before, a);
    }
    // New: the schema requires `r:id`, and an action with no target states it empty.
    let mut stated = Hyperlink::default();
    match a.rel_id.as_deref() {
        None => return Err(unsupported(w, "a hyperlink with no r:id")),
        Some("") => {
            let prefix = w.declared_prefix(el, ns::REL, "r")?;
            w.attr(el, &format!("{prefix}:id"), None, Some(&String::new()))?;
            stated.rel_id = Some(String::new());
        }
        Some(_) => {}
    }
    patch_hyperlink(w, el, &stated, a)
}

fn patch_highlight(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&Option<Color>>,
    a: Option<&Option<Color>>,
) -> Result {
    if b == a {
        return Ok(());
    }
    match a {
        None => w.remove_all(el, ns::DML, "highlight"),
        Some(None) => Err(unsupported(w, "a highlight with no colour")),
        Some(Some(color)) => {
            let holder = swap_choice(w, el, &["highlight"], CHARACTER_ORDER, "highlight")?;
            set_color(w, &holder, color)
        }
    }
}

fn patch_underline(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&CharacterProperties>,
    a: &CharacterProperties,
) -> Result {
    const LINE: &[&str] = &["uLnTx", "uLn"];
    const FILL: &[&str] = &["uFillTx", "uFill"];
    let line = b.and_then(|b| b.underline_line.as_ref());
    if line != a.underline_line.as_ref() {
        match &a.underline_line {
            None => remove_group(w, el, LINE)?,
            Some(UnderlineLine::FollowText) => {
                swap_choice(w, el, LINE, CHARACTER_ORDER, "uLnTx")?;
            }
            Some(UnderlineLine::Line(wanted)) => {
                let holder = swap_choice(w, el, LINE, CHARACTER_ORDER, "uLn")?;
                let old = match line {
                    Some(UnderlineLine::Line(old)) => Some(&**old),
                    _ => None,
                };
                patch_line(w, &holder, "uLn", old, wanted)?;
            }
        }
    }
    let fill = b.and_then(|b| b.underline_fill.as_ref());
    if fill != a.underline_fill.as_ref() {
        match &a.underline_fill {
            None => remove_group(w, el, FILL)?,
            Some(UnderlineFill::FollowText) => {
                swap_choice(w, el, FILL, CHARACTER_ORDER, "uFillTx")?;
            }
            Some(UnderlineFill::Fill(wanted)) => {
                let holder = swap_choice(w, el, FILL, CHARACTER_ORDER, "uFill")?;
                let old = match fill {
                    Some(UnderlineFill::Fill(old)) => old.as_ref(),
                    _ => None,
                };
                patch_fill(w, &holder, FILLS, old, wanted.as_ref())?;
            }
        }
    }
    Ok(())
}

/// `a:CT_TextParagraph`.
fn patch_paragraph(w: &mut Writer<'_>, el: &Elem, b: Option<&Paragraph>, a: &Paragraph) -> Result {
    let old_properties = b.and_then(|b| b.properties.as_ref());
    if old_properties != a.properties.as_ref() {
        match &a.properties {
            None => w.remove_all(el, ns::DML, "pPr")?,
            Some(wanted) => {
                // First, whatever the first child is: an `mc:AlternateContent` is no
                // landmark for a list of DrawingML names.
                let holder = match el.first_child(w.tree, w.part, ns::DML, "pPr")? {
                    Some(holder) => holder,
                    None => {
                        let holder = w.new_child(el, ns::DML, "a", "pPr")?;
                        let first = w.tree.child_elements(el.id).first().copied();
                        w.tree.insert_child(el.id, first, holder.id);
                        holder
                    }
                };
                patch_paragraph_properties(w, &holder, old_properties.map(|b| &**b), wanted)?;
            }
        }
    }
    let before: &[TextItem] = b.map_or(&[], |b| &b.items);
    if before != a.items.as_slice() {
        patch_items(w, el, before, &a.items)?;
    }
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        PARAGRAPH_ORDER,
        "endParaRPr",
        b.and_then(|b| b.end_properties.as_ref()),
        a.end_properties.as_ref(),
        boxed_character,
    )
}

fn patch_items(w: &mut Writer<'_>, el: &Elem, before: &[TextItem], after: &[TextItem]) -> Result {
    let elements: Vec<Elem> = el
        .children(w.tree, w.part)?
        .into_iter()
        .filter(|c| !(c.is(ns::DML, "pPr") || c.is(ns::DML, "endParaRPr")))
        .collect();
    if elements.len() != before.len() {
        return Err(ExportError::MalformedElement {
            part: w.part.to_owned(),
        });
    }
    for (at, wanted) in after.iter().enumerate() {
        match (before.get(at), elements.get(at)) {
            (Some(old), Some(holder)) => match (old, wanted) {
                (TextItem::Run(o), TextItem::Run(n)) => patch_run(w, holder, Some(o), n)?,
                (TextItem::Break(o), TextItem::Break(n)) => patch_break(w, holder, Some(o), n)?,
                (TextItem::Field(o), TextItem::Field(n)) => patch_field(w, holder, Some(o), n)?,
                (TextItem::Retained(o), TextItem::Retained(n)) if o == n => {}
                _ => return Err(unsupported(w, "reordering what a paragraph holds")),
            },
            _ => {
                let local = match wanted {
                    TextItem::Run(_) => "r",
                    TextItem::Break(_) => "br",
                    TextItem::Field(_) => "fld",
                    TextItem::Retained(_) => {
                        return Err(unsupported(w, "content this tier does not model"));
                    }
                };
                let holder = w.new_child(el, ns::DML, "a", local)?;
                // Before `endParaRPr`, which is last.
                let end = el.first_child(w.tree, w.part, ns::DML, "endParaRPr")?;
                w.tree.insert_child(el.id, end.map(|e| e.id), holder.id);
                match wanted {
                    TextItem::Run(n) => patch_run(w, &holder, None, n)?,
                    TextItem::Break(n) => patch_break(w, &holder, None, n)?,
                    TextItem::Field(n) => patch_field(w, &holder, None, n)?,
                    TextItem::Retained(_) => {}
                }
            }
        }
    }
    if before.len() > after.len() {
        for (old, holder) in before.iter().zip(&elements).skip(after.len()) {
            if matches!(old, TextItem::Retained(_)) {
                return Err(unsupported(w, "removing content this tier does not model"));
            }
            w.tree.remove_child(el.id, holder.id);
            w.changes += 1;
        }
    }
    Ok(())
}

fn patch_run(w: &mut Writer<'_>, el: &Elem, b: Option<&TextRun>, a: &TextRun) -> Result {
    if a.text.is_none() && b.is_none_or(|b| b.text.is_some()) {
        return Err(unsupported(w, "a run with no text element"));
    }
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        RUN_ORDER,
        "rPr",
        b.and_then(|b| b.properties.as_ref()),
        a.properties.as_ref(),
        boxed_character,
    )?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        RUN_ORDER,
        "t",
        b.and_then(|b| b.text.as_ref()),
        a.text.as_ref(),
        |w, t, _, value| w.set_text(t, value),
    )
}

fn patch_break(w: &mut Writer<'_>, el: &Elem, b: Option<&LineBreak>, a: &LineBreak) -> Result {
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        &["rPr"],
        "rPr",
        b.and_then(|b| b.properties.as_ref()),
        a.properties.as_ref(),
        boxed_character,
    )
}

fn patch_field(w: &mut Writer<'_>, el: &Elem, b: Option<&TextField>, a: &TextField) -> Result {
    if a.id.is_none() && b.is_none_or(|b| b.id.is_some()) {
        return Err(unsupported(w, "a field with no id"));
    }
    w.attr(el, "id", b.and_then(|b| b.id.as_ref()), a.id.as_ref())?;
    w.attr(
        el,
        "type",
        b.and_then(|b| b.field_type.as_ref()),
        a.field_type.as_ref(),
    )?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        FIELD_ORDER,
        "rPr",
        b.and_then(|b| b.properties.as_ref()),
        a.properties.as_ref(),
        boxed_character,
    )?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        FIELD_ORDER,
        "pPr",
        b.and_then(|b| b.paragraph_properties.as_ref()),
        a.paragraph_properties.as_ref(),
        boxed_paragraph_properties,
    )?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        FIELD_ORDER,
        "t",
        b.and_then(|b| b.text.as_ref()),
        a.text.as_ref(),
        |w, t, _, value| w.set_text(t, value),
    )
}
