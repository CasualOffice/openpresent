//! Edits a theme part's tree, and colour maps wherever they live. Each function
//! names the node it changes and leaves all else as read. Each returns whether
//! anything changed, so a no-op edit can leave the part out of the save.

use casual_slide_model::color::Color;
use casual_slide_model::theme::{
    ColorMapOverride, ColorMapSlot, ColorSchemeIndex, FontCollectionIndex,
};
use casual_slide_ooxml::{ElementId, XmlTree};
use casual_slide_package::PartEdits;

use crate::color::{ExportError, dml_child, dml_name, is_dml, set_color, validate};

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
pub fn set_override_scheme_color(
    tree: &mut XmlTree,
    slot: ColorSchemeIndex,
    color: &Color,
) -> Result<bool, ExportError> {
    let scheme = step(tree, tree.root(), "clrScheme")?;
    set_slot(tree, scheme, slot, color)
}

/// Set one slot of an extra colour scheme, by its place in the list.
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
pub fn set_theme_name(tree: &mut XmlTree, name: &str) -> Result<bool, ExportError> {
    let root = tree.root();
    patch_text_attribute(tree, root, "name", name)
}

/// Set the colour scheme's name.
pub fn set_color_scheme_name(tree: &mut XmlTree, name: &str) -> Result<bool, ExportError> {
    let el = step(tree, theme_elements(tree)?, "clrScheme")?;
    patch_text_attribute(tree, el, "name", name)
}

/// Set the font scheme's name.
pub fn set_font_scheme_name(tree: &mut XmlTree, name: &str) -> Result<bool, ExportError> {
    let el = step(tree, theme_elements(tree)?, "fontScheme")?;
    patch_text_attribute(tree, el, "name", name)
}

/// Set the format scheme's name.
pub fn set_format_scheme_name(tree: &mut XmlTree, name: &str) -> Result<bool, ExportError> {
    let el = step(tree, theme_elements(tree)?, "fmtScheme")?;
    patch_text_attribute(tree, el, "name", name)
}

/// Set the typeface of a font scheme's `latin`, `ea` or `cs`.
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
pub fn set_style_matrix_index(
    tree: &mut XmlTree,
    reference: ElementId,
    idx: u32,
) -> Result<bool, ExportError> {
    patch_text_attribute(tree, reference, "idx", &idx.to_string())
}

/// Set `@idx` of a `CT_FontReference`.
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

/// The edit for saving a part, or none if its bytes did not change.
pub fn part_edit(part: &str, original: &[u8], tree: &XmlTree) -> PartEdits {
    let mut edits = PartEdits::default();
    let bytes = tree.to_bytes();
    if bytes != original {
        edits.replace(part.to_owned(), bytes);
    }
    edits
}
