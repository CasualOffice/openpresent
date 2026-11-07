//! Edits a theme part's tree, and colour maps wherever they live. Each function
//! names the node it changes and leaves all else as read. Each returns whether
//! anything changed, so a no-op edit can leave the part out of the save.

use casual_slide_import::drawing::is_fill_name;
use casual_slide_import::properties::{Elem, ns};
use casual_slide_import::theme::{
    ThemeImportError, import_theme_override_with_rels, import_theme_with_rels,
};
use casual_slide_model::color::Color;
use casual_slide_model::theme::{
    ColorMapOverride, ColorMapSlot, ColorSchemeIndex, FontCollectionIndex, FormatScheme,
    ObjectDefaults, ShapeDefinition, StyleEffect,
};
use casual_slide_ooxml::{ElementId, OoxmlLimits, Relationship, XmlTree};
use casual_slide_package::PartEdits;

use crate::color::{ExportError, dml_child, dml_name, is_dml, set_color, validate};
use crate::d3::{scene3d_node, sp3d_node};
use crate::drawing::{Node, Space, check_svg_fallback, patch, patch_style, properties_node};
use crate::effect::effect_properties_node;
use crate::fill::fill_node;
use crate::line::line_node;
use crate::properties::{ExportError as PartError, Writer};
use crate::text::{patch_body_properties, patch_list_style};

/// Which font collection of a font scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontRole {
    /// `majorFont`.
    Major,
    /// `minorFont`.
    Minor,
}

impl FontRole {
    fn element(self) -> &'static str {
        match self {
            Self::Major => "majorFont",
            Self::Minor => "minorFont",
        }
    }
}

/// Which script slot of a font collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSlot {
    /// `latin`.
    Latin,
    /// `ea`.
    Ea,
    /// `cs`.
    Cs,
}

impl FontSlot {
    fn element(self) -> &'static str {
        match self {
            Self::Latin => "latin",
            Self::Ea => "ea",
            Self::Cs => "cs",
        }
    }
}

fn require(found: Option<ElementId>, what: &'static str) -> Result<ElementId, ExportError> {
    found.ok_or(ExportError::NodeMissing(what))
}

fn step(tree: &XmlTree, parent: ElementId, local: &'static str) -> Result<ElementId, ExportError> {
    require(dml_child(tree, parent, local), local)
}

fn theme_elements(tree: &XmlTree) -> Result<ElementId, ExportError> {
    step(tree, tree.root(), "themeElements")
}

/// Whether text can be written into XML at all.
fn check_text(value: &str, what: &'static str) -> Result<(), ExportError> {
    let legal = |c: char| {
        matches!(c, '\t' | '\n' | '\r')
            || ('\u{20}'..='\u{D7FF}').contains(&c)
            || ('\u{E000}'..='\u{FFFD}').contains(&c)
            || c >= '\u{10000}'
    };
    if value.chars().all(legal) {
        Ok(())
    } else {
        Err(ExportError::InvalidValue(what))
    }
}

fn patch_text_attribute(
    tree: &mut XmlTree,
    el: ElementId,
    name: &'static str,
    value: &str,
) -> Result<bool, ExportError> {
    check_text(value, name)?;
    if !tree.attributes_valid(el) {
        return Err(ExportError::MalformedNode(name));
    }
    if tree.plain_attribute(el, name).as_deref() == Some(value) {
        return Ok(false);
    }
    if !tree.set_attribute(el, name, value) {
        return Err(ExportError::MalformedNode(name));
    }
    Ok(true)
}

/// Set one slot of the theme's colour scheme. This is the "edit a theme colour"
/// the fidelity contract is written against.
// spec-audit: owner=themeElements,clrScheme
pub fn set_scheme_color(
    tree: &mut XmlTree,
    slot: ColorSchemeIndex,
    color: &Color,
) -> Result<bool, ExportError> {
    let scheme = step(tree, theme_elements(tree)?, "clrScheme")?;
    set_slot(tree, scheme, slot, color)
}

fn set_slot(
    tree: &mut XmlTree,
    scheme: ElementId,
    slot: ColorSchemeIndex,
    color: &Color,
) -> Result<bool, ExportError> {
    let holder = require(dml_child(tree, scheme, slot.as_xml()), "colour scheme slot")?;
    set_color(tree, holder, color)
}

/// Set one slot of a `themeOverride` part's colour scheme.
// spec-audit: owner=themeOverride
pub fn set_override_scheme_color(
    tree: &mut XmlTree,
    slot: ColorSchemeIndex,
    color: &Color,
) -> Result<bool, ExportError> {
    let scheme = step(tree, tree.root(), "clrScheme")?;
    set_slot(tree, scheme, slot, color)
}

/// Set one slot of an extra colour scheme, by its place in the list.
// spec-audit: owner=extraClrSchemeLst,extraClrScheme
pub fn set_extra_scheme_color(
    tree: &mut XmlTree,
    index: usize,
    slot: ColorSchemeIndex,
    color: &Color,
) -> Result<bool, ExportError> {
    let list = step(tree, tree.root(), "extraClrSchemeLst")?;
    let extra = nth_named(tree, list, "extraClrScheme", index)?;
    let scheme = step(tree, extra, "clrScheme")?;
    set_slot(tree, scheme, slot, color)
}

fn nth_named(
    tree: &XmlTree,
    parent: ElementId,
    local: &'static str,
    index: usize,
) -> Result<ElementId, ExportError> {
    require(
        tree.child_elements(parent)
            .into_iter()
            .filter(|&c| tree.local_name(c) == local && is_dml(tree, c))
            .nth(index),
        local,
    )
}

/// Set a custom colour's value, by its place in the list.
// spec-audit: owner=custClrLst,custClr
pub fn set_custom_color(
    tree: &mut XmlTree,
    index: usize,
    color: &Color,
) -> Result<bool, ExportError> {
    let list = step(tree, tree.root(), "custClrLst")?;
    let holder = nth_named(tree, list, "custClr", index)?;
    set_color(tree, holder, color)
}

/// Add a custom colour at the end of the list, creating the list in its schema
/// position if the theme has none.
// spec-audit: owner=custClrLst,custClr
pub fn add_custom_color(
    tree: &mut XmlTree,
    name: Option<&str>,
    color: &Color,
) -> Result<(), ExportError> {
    validate(color)?;
    if let Some(name) = name {
        check_text(name, "name")?;
    }
    let root = tree.root();
    let list = match dml_child(tree, root, "custClrLst") {
        Some(list) => list,
        None => {
            let list_name = dml_name(tree, root, "custClrLst")?;
            let before = dml_child(tree, root, "extLst");
            tree.insert_element(root, before, &list_name, &[])
        }
    };
    let qualified = dml_name(tree, list, "custClr")?;
    let attrs: Vec<(&str, &str)> = name.iter().map(|n| ("name", *n)).collect();
    let holder = tree.insert_element(list, None, &qualified, &attrs);
    set_color(tree, holder, color)?;
    Ok(())
}

/// Set the theme's name.
// spec-audit: owner=theme
pub fn set_theme_name(tree: &mut XmlTree, name: &str) -> Result<bool, ExportError> {
    let root = tree.root();
    patch_text_attribute(tree, root, "name", name)
}

/// Set the colour scheme's name.
// spec-audit: owner=clrScheme
pub fn set_color_scheme_name(tree: &mut XmlTree, name: &str) -> Result<bool, ExportError> {
    let el = step(tree, theme_elements(tree)?, "clrScheme")?;
    patch_text_attribute(tree, el, "name", name)
}

/// Set the font scheme's name.
// spec-audit: owner=fontScheme
pub fn set_font_scheme_name(tree: &mut XmlTree, name: &str) -> Result<bool, ExportError> {
    let el = step(tree, theme_elements(tree)?, "fontScheme")?;
    patch_text_attribute(tree, el, "name", name)
}

/// Set the format scheme's name.
// spec-audit: owner=fmtScheme
pub fn set_format_scheme_name(tree: &mut XmlTree, name: &str) -> Result<bool, ExportError> {
    let el = step(tree, theme_elements(tree)?, "fmtScheme")?;
    patch_text_attribute(tree, el, "name", name)
}

/// Set the typeface of a font scheme's `latin`, `ea` or `cs`.
// spec-audit: owner=majorFont,minorFont,latin,ea,cs
pub fn set_typeface(
    tree: &mut XmlTree,
    role: FontRole,
    slot: FontSlot,
    typeface: &str,
) -> Result<bool, ExportError> {
    let scheme = step(tree, theme_elements(tree)?, "fontScheme")?;
    let collection = step(tree, scheme, role.element())?;
    let font = step(tree, collection, slot.element())?;
    patch_text_attribute(tree, font, "typeface", typeface)
}

/// Set the typeface of a supplemental font, adding the `font` entry in its schema
/// position if this script has none.
// spec-audit: owner=font
pub fn set_supplemental_font(
    tree: &mut XmlTree,
    role: FontRole,
    script: &str,
    typeface: &str,
) -> Result<bool, ExportError> {
    check_text(script, "script")?;
    check_text(typeface, "typeface")?;
    let scheme = step(tree, theme_elements(tree)?, "fontScheme")?;
    let collection = step(tree, scheme, role.element())?;
    let existing = tree.child_elements(collection).into_iter().find(|&c| {
        tree.local_name(c) == "font"
            && is_dml(tree, c)
            && tree.plain_attribute(c, "script").as_deref() == Some(script)
    });
    if let Some(font) = existing {
        return patch_text_attribute(tree, font, "typeface", typeface);
    }
    let name = dml_name(tree, collection, "font")?;
    let before = dml_child(tree, collection, "extLst");
    tree.insert_element(
        collection,
        before,
        &name,
        &[("script", script), ("typeface", typeface)],
    );
    Ok(true)
}

/// A `CT_TextFont` of a font scheme, by its place.
fn font_slot(tree: &XmlTree, role: FontRole, slot: FontSlot) -> Result<ElementId, ExportError> {
    let scheme = step(tree, theme_elements(tree)?, "fontScheme")?;
    let collection = step(tree, scheme, role.element())?;
    step(tree, collection, slot.element())
}

/// Set an attribute to this text, or take it away. Taken away it is absent: a schema
/// default is never written.
fn patch_optional_attribute(
    tree: &mut XmlTree,
    el: ElementId,
    name: &'static str,
    value: Option<&str>,
) -> Result<bool, ExportError> {
    match value {
        Some(value) => patch_text_attribute(tree, el, name, value),
        None if tree.plain_attribute(el, name).is_none() => Ok(false),
        None if tree.attributes_valid(el) && tree.remove_attribute(el, name) => Ok(true),
        None => Err(ExportError::MalformedNode(name)),
    }
}

/// Set a font's `@panose`, ten bytes as hex text, or take it away.
// spec-audit: owner=latin,ea,cs
pub fn set_font_panose(
    tree: &mut XmlTree,
    role: FontRole,
    slot: FontSlot,
    panose: Option<&str>,
) -> Result<bool, ExportError> {
    let font = font_slot(tree, role, slot)?;
    if panose.is_some_and(|p| p.len() != 20 || !p.bytes().all(|b| b.is_ascii_hexdigit())) {
        return Err(ExportError::InvalidValue("panose"));
    }
    patch_optional_attribute(tree, font, "panose", panose)
}

/// Set a font's `@pitchFamily`, or take it away. `ST_PitchFamily` lists its values: a
/// pitch from 0 to 2, with a family from 0 to 5 in the high nibble, spelled in decimal.
// spec-audit: owner=latin,ea,cs
pub fn set_font_pitch_family(
    tree: &mut XmlTree,
    role: FontRole,
    slot: FontSlot,
    pitch_family: Option<i8>,
) -> Result<bool, ExportError> {
    let font = font_slot(tree, role, slot)?;
    if pitch_family.is_some_and(|v| !(0..=0x52).contains(&v) || v & 0x0F > 2) {
        return Err(ExportError::InvalidValue("pitchFamily"));
    }
    patch_optional_attribute(
        tree,
        font,
        "pitchFamily",
        pitch_family.map(|v| v.to_string()).as_deref(),
    )
}

/// Set a font's `@charset`, an `xsd:byte`, or take it away.
// spec-audit: owner=latin,ea,cs
pub fn set_font_charset(
    tree: &mut XmlTree,
    role: FontRole,
    slot: FontSlot,
    charset: Option<i8>,
) -> Result<bool, ExportError> {
    let font = font_slot(tree, role, slot)?;
    patch_optional_attribute(
        tree,
        font,
        "charset",
        charset.map(|v| v.to_string()).as_deref(),
    )
}

/// Set `@idx` of a `CT_StyleMatrixReference`: `lnRef`, `fillRef`, `effectRef`.
// spec-audit: owner=lnRef,fillRef,effectRef
pub fn set_style_matrix_index(
    tree: &mut XmlTree,
    reference: ElementId,
    idx: u32,
) -> Result<bool, ExportError> {
    patch_text_attribute(tree, reference, "idx", &idx.to_string())
}

/// Set `@idx` of a `CT_FontReference`.
// spec-audit: owner=fontRef
pub fn set_font_reference_index(
    tree: &mut XmlTree,
    reference: ElementId,
    idx: FontCollectionIndex,
) -> Result<bool, ExportError> {
    patch_text_attribute(tree, reference, "idx", idx.as_xml())
}

/// Set `@ref` of a `CT_EffectReference`. An `xsd:token`: no empty value, and no leading,
/// trailing or doubled space, or other white space.
pub fn set_effect_reference(
    tree: &mut XmlTree,
    effect: ElementId,
    reference: &str,
) -> Result<bool, ExportError> {
    let token = !reference.is_empty()
        && !reference.starts_with(' ')
        && !reference.ends_with(' ')
        && !reference.contains("  ")
        && !reference.contains(['\t', '\n', '\r']);
    if !token {
        return Err(ExportError::InvalidValue("ref"));
    }
    patch_text_attribute(tree, effect, "ref", reference)
}

/// Set one attribute of a `CT_ColorMapping` element: `p:clrMap` on a master,
/// `a:overrideClrMapping`.
// spec-audit: owner=clrMap
pub fn set_color_map_slot(
    tree: &mut XmlTree,
    map: ElementId,
    slot: ColorMapSlot,
    to: ColorSchemeIndex,
) -> Result<bool, ExportError> {
    patch_text_attribute(tree, map, slot.as_xml(), to.as_xml())
}

/// Make a `clrMapOvr` hold this choice, replacing the other. An override with the
/// same values as the one there changes nothing; a colour map's `extLst` is kept
/// when the choice stays an override. A map with slots absent keeps those absent.
// spec-audit: owner=clrMapOvr,overrideClrMapping
pub fn set_color_map_override(
    tree: &mut XmlTree,
    holder: ElementId,
    value: &ColorMapOverride,
) -> Result<bool, ExportError> {
    let existing = tree.child_elements(holder).into_iter().find(|&c| {
        matches!(
            tree.local_name(c),
            "masterClrMapping" | "overrideClrMapping"
        )
    });
    match value {
        ColorMapOverride::Master => {
            let name = dml_name(tree, holder, "masterClrMapping")?;
            match existing {
                Some(el) if tree.local_name(el) == "masterClrMapping" => Ok(false),
                Some(el) => {
                    tree.replace_element(el, &name, &[]);
                    Ok(true)
                }
                None => {
                    tree.insert_element(holder, None, &name, &[]);
                    Ok(true)
                }
            }
        }
        ColorMapOverride::Override(map) => {
            let name = dml_name(tree, holder, "overrideClrMapping")?;
            let (el, mut changed) = match existing {
                Some(el) if tree.local_name(el) == "overrideClrMapping" => (el, false),
                Some(el) => {
                    tree.replace_element(el, &name, &[]);
                    (el, true)
                }
                None => (tree.insert_element(holder, None, &name, &[]), true),
            };
            for slot in ColorMapSlot::ALL {
                if let Some(to) = map.get(*slot) {
                    changed |= set_color_map_slot(tree, el, *slot, to)?;
                }
            }
            Ok(changed)
        }
    }
}

fn effect_style_node(e: &StyleEffect) -> Node {
    Node::a("effectStyle")
        .opt(e.effects.as_ref().map(effect_properties_node))
        .opt(
            e.scene3d
                .as_ref()
                .map(|s| scene3d_node(s, Space::Dml, "scene3d")),
        )
        .opt(e.sp3d.as_ref().map(|s| sp3d_node(s, Space::Dml, "sp3d")))
}

// spec-audit: owner=fmtScheme,fillStyleLst,lnStyleLst,effectStyleLst,bgFillStyleLst
fn format_scheme_node(f: &FormatScheme) -> Node {
    let fills = |local, list: &Vec<_>| Node::a(local).list(list.iter().map(fill_node));
    Node::a("fmtScheme")
        .lex("name", f.name.as_ref())
        .opt(f.fills.as_ref().map(|l| fills("fillStyleLst", l)))
        .opt(f.lines.as_ref().map(|l| {
            Node::a("lnStyleLst").list(l.iter().map(|line| line_node(line, Space::Dml, "ln")))
        }))
        .opt(
            f.effects
                .as_ref()
                .map(|l| Node::a("effectStyleLst").list(l.iter().map(effect_style_node))),
        )
        .opt(
            f.background_fills
                .as_ref()
                .map(|l| fills("bgFillStyleLst", l)),
        )
}

fn from_import(err: ThemeImportError, part: &str) -> PartError {
    match err {
        ThemeImportError::Xml(err) => err.into(),
        ThemeImportError::WrongRoot { expected } => PartError::UnsupportedEdit {
            what: expected,
            part: part.to_owned(),
        },
    }
}

fn parsed_again(tree: &XmlTree, part: &str, limits: &OoxmlLimits) -> Result<XmlTree, PartError> {
    Ok(XmlTree::parse(&tree.to_bytes(), part, limits)?)
}

/// A list entry past a bound is reported and not in the model, so the model's entries no
/// longer pair with the part's by place. Nothing is patched on that footing.
fn check_lists_pair(w: &Writer<'_>, el: &Elem, before: &FormatScheme) -> Result<(), PartError> {
    for list in el.children(w.tree, w.part)? {
        if !list.in_namespace(ns::DML) {
            continue;
        }
        let in_part = |pick: fn(&Elem) -> bool| -> Result<usize, PartError> {
            let kids = list.children(w.tree, w.part)?;
            Ok(kids.iter().filter(|k| pick(k)).count())
        };
        let (held, there) = match list.local.as_str() {
            "fillStyleLst" => (
                before.fills.as_ref().map(Vec::len),
                in_part(|k| k.in_namespace(ns::DML) && is_fill_name(&k.local))?,
            ),
            "bgFillStyleLst" => (
                before.background_fills.as_ref().map(Vec::len),
                in_part(|k| k.in_namespace(ns::DML) && is_fill_name(&k.local))?,
            ),
            "lnStyleLst" => (
                before.lines.as_ref().map(Vec::len),
                in_part(|k| k.is(ns::DML, "ln"))?,
            ),
            "effectStyleLst" => (
                before.effects.as_ref().map(Vec::len),
                in_part(|k| k.is(ns::DML, "effectStyle"))?,
            ),
            _ => continue,
        };
        if held != Some(there) {
            return Err(PartError::UnsupportedEdit {
                what: "a format scheme list with entries the model does not hold",
                part: w.part.to_owned(),
            });
        }
    }
    Ok(())
}

fn patch_format_scheme(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&FormatScheme>,
    after: &FormatScheme,
) -> Result<(), PartError> {
    if let Some(before) = before {
        check_lists_pair(w, el, before)?;
    }
    patch(
        w,
        el,
        before.map(format_scheme_node).as_ref(),
        &format_scheme_node(after),
    )
}

/// Bring a theme part's `fmtScheme` to this, its fills, lines and effects patched node by node.
/// Returns whether anything changed. What was asked is read back before returning.
// spec-audit: owner=themeElements
pub fn set_format_scheme(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &FormatScheme,
    limits: &OoxmlLimits,
) -> Result<bool, PartError> {
    let read = |tree: &XmlTree| {
        import_theme_with_rels(tree, rels, limits)
            .map(|i| i.value.elements.and_then(|e| e.format_scheme))
            .map_err(|e| from_import(e, part))
    };
    let before = read(tree)?;
    if before.as_ref() == Some(after) {
        return Ok(false);
    }
    let root = Elem::root(tree, part)?;
    let Some(holder) = root.first_child(tree, part, ns::DML, "themeElements")? else {
        return Err(PartError::UnsupportedEdit {
            what: "a format scheme where the theme has no themeElements",
            part: part.to_owned(),
        });
    };
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
        changes: 0,
    };
    w.sync_child(
        &holder,
        ns::DML,
        "a",
        &["clrScheme", "fontScheme", "fmtScheme", "extLst"],
        "fmtScheme",
        before.as_ref(),
        Some(after),
        patch_format_scheme,
    )?;
    if read(&parsed_again(tree, part, limits)?)?.as_ref() != Some(after) {
        return Err(PartError::RoundTrip {
            part: part.to_owned(),
        });
    }
    Ok(true)
}

/// Bring a `themeOverride` part's `fmtScheme` to this. As [`set_format_scheme`].
// spec-audit: owner=themeOverride
pub fn set_override_format_scheme(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &FormatScheme,
    limits: &OoxmlLimits,
) -> Result<bool, PartError> {
    let read = |tree: &XmlTree| {
        import_theme_override_with_rels(tree, rels)
            .map(|i| i.value.format_scheme)
            .map_err(|e| from_import(e, part))
    };
    let before = read(tree)?;
    if before.as_ref() == Some(after) {
        return Ok(false);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
        changes: 0,
    };
    w.sync_child(
        &root,
        ns::DML,
        "a",
        &["clrScheme", "fontScheme", "fmtScheme"],
        "fmtScheme",
        before.as_ref(),
        Some(after),
        patch_format_scheme,
    )?;
    if read(&parsed_again(tree, part, limits)?)?.as_ref() != Some(after) {
        return Err(PartError::RoundTrip {
            part: part.to_owned(),
        });
    }
    Ok(true)
}

fn blip_of(
    p: Option<&casual_slide_model::drawing::ShapeProperties>,
) -> Option<&casual_slide_model::fill::Blip> {
    match p?.fill.as_ref()? {
        casual_slide_model::fill::Fill::Blip(f) => f.blip.as_ref(),
        _ => None,
    }
}

// spec-audit: owner=spDef,lnDef,txDef,spPr,bodyPr,lstStyle
fn patch_definition(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&ShapeDefinition>,
    a: &ShapeDefinition,
) -> Result<(), PartError> {
    const ORDER: &[&str] = &["spPr", "bodyPr", "lstStyle", "style", "extLst"];
    let before = b.and_then(|b| b.shape_properties.as_ref());
    check_svg_fallback(w, blip_of(before), blip_of(a.shape_properties.as_ref()))?;
    w.sync_child(
        el,
        ns::DML,
        "a",
        ORDER,
        "spPr",
        before,
        a.shape_properties.as_ref(),
        |w, el, b, a| {
            patch(
                w,
                el,
                b.map(|b| properties_node(b, "spPr")).as_ref(),
                &properties_node(a, "spPr"),
            )
        },
    )?;
    w.sync_child(
        el,
        ns::DML,
        "a",
        ORDER,
        "bodyPr",
        b.and_then(|b| b.body_properties.as_ref()),
        a.body_properties.as_ref(),
        patch_body_properties,
    )?;
    w.sync_child(
        el,
        ns::DML,
        "a",
        ORDER,
        "lstStyle",
        b.and_then(|b| b.list_style.as_ref()),
        a.list_style.as_ref(),
        patch_list_style,
    )?;
    w.sync_child(
        el,
        ns::DML,
        "a",
        ORDER,
        "style",
        b.and_then(|b| b.style.as_ref()),
        a.style.as_ref(),
        patch_style,
    )
}

// spec-audit: owner=objectDefaults
fn patch_object_defaults(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&ObjectDefaults>,
    a: &ObjectDefaults,
) -> Result<(), PartError> {
    const ORDER: &[&str] = &["spDef", "lnDef", "txDef", "extLst"];
    let slots: [(&str, fn(&ObjectDefaults) -> Option<&ShapeDefinition>); 3] = [
        ("spDef", |d| d.shape.as_ref()),
        ("lnDef", |d| d.line.as_ref()),
        ("txDef", |d| d.text.as_ref()),
    ];
    for (local, slot) in slots {
        w.sync_child(
            el,
            ns::DML,
            "a",
            ORDER,
            local,
            b.and_then(slot),
            slot(a),
            patch_definition,
        )?;
    }
    Ok(())
}

/// Bring a theme part's `objectDefaults` to this, or take it away. Each default's `spPr`,
/// `bodyPr`, `lstStyle` and `style` are patched node by node; `extLst` stays in the part, and
/// a change to it does not read back. Returns whether anything changed.
pub fn set_object_defaults(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: Option<&ObjectDefaults>,
    limits: &OoxmlLimits,
) -> Result<bool, PartError> {
    let read = |tree: &XmlTree| {
        import_theme_with_rels(tree, rels, limits)
            .map(|i| i.value.object_defaults)
            .map_err(|e| from_import(e, part))
    };
    let before = read(tree)?;
    if before.as_ref() == after {
        return Ok(false);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
        changes: 0,
    };
    w.sync_child(
        &root,
        ns::DML,
        "a",
        &[
            "themeElements",
            "objectDefaults",
            "extraClrSchemeLst",
            "custClrLst",
            "extLst",
        ],
        "objectDefaults",
        before.as_ref(),
        after,
        patch_object_defaults,
    )?;
    if read(&parsed_again(tree, part, limits)?)?.as_ref() != after {
        return Err(PartError::RoundTrip {
            part: part.to_owned(),
        });
    }
    Ok(true)
}

/// The edit for saving a part, or none if its bytes did not change.
pub fn part_edit(part: &str, original: &[u8], tree: &XmlTree) -> PartEdits {
    let mut edits = PartEdits::default();
    let bytes = tree.to_bytes();
    if bytes != original {
        edits.replace(part.to_owned(), bytes);
    }
    edits
}
