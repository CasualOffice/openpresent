//! Writes a colour by patching the tree: only attributes and children that differ.
//!
//! The same colour written twice changes nothing, so an edit that lands where the
//! value already is leaves the bytes exactly as read, spelling included.

use casual_slide_model::color::{
    Color, ColorTransform, ColorValue, MAX_COLOR_TRANSFORMS, Rgb, TransformKind, ValueDomain,
    format_percentage, parse_int, parse_percentage,
};
use casual_slide_ooxml::{ElementId, XmlTree};

/// DrawingML, Transitional.
pub const DML_TRANSITIONAL: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
/// DrawingML, Strict.
pub const DML_STRICT: &str = "http://purl.oclc.org/ooxml/drawingml/main";

/// Why a tree was not changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportError {
    /// The value is outside what the schema allows, or holds characters XML
    /// cannot carry.
    InvalidValue(&'static str),
    /// More than [`MAX_COLOR_TRANSFORMS`] transforms.
    TooManyTransforms,
    /// The element the edit names is not in this part.
    NodeMissing(&'static str),
    /// The element's own attributes are malformed, so it cannot be patched.
    MalformedNode(&'static str),
    /// No prefix in scope names DrawingML here.
    NoNamespace,
}

impl core::fmt::Display for ExportError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidValue(what) => write!(f, "invalid value for {what}"),
            Self::TooManyTransforms => write!(f, "too many colour transforms"),
            Self::NodeMissing(what) => write!(f, "{what} is not in this part"),
            Self::MalformedNode(what) => write!(f, "{what} has malformed attributes"),
            Self::NoNamespace => write!(f, "no prefix in scope names DrawingML"),
        }
    }
}

impl std::error::Error for ExportError {}

/// Whether an element is in DrawingML, Transitional or Strict.
pub(crate) fn is_dml(tree: &XmlTree, id: ElementId) -> bool {
    tree.namespace_uri(id)
        .is_some_and(|uri| uri == DML_TRANSITIONAL || uri == DML_STRICT)
}

/// The first child in DrawingML with this local name.
pub(crate) fn dml_child(tree: &XmlTree, parent: ElementId, local: &str) -> Option<ElementId> {
    tree.child_elements(parent)
        .into_iter()
        .find(|&c| tree.local_name(c) == local && is_dml(tree, c))
}

fn is_strict(tree: &XmlTree, at: ElementId) -> bool {
    tree.namespace_uri(at).as_deref() == Some(DML_STRICT)
}

/// The name for a new DrawingML element here, with the prefix that is in scope.
pub(crate) fn dml_name(tree: &XmlTree, at: ElementId, local: &str) -> Result<String, ExportError> {
    let prefix = tree
        .prefix_for_namespace(at, DML_TRANSITIONAL)
        .or_else(|| tree.prefix_for_namespace(at, DML_STRICT))
        .ok_or(ExportError::NoNamespace)?;
    Ok(if prefix.is_empty() {
        local.to_owned()
    } else {
        format!("{prefix}:{local}")
    })
}

fn is_color_name(name: &str) -> bool {
    matches!(
        name,
        "scrgbClr" | "srgbClr" | "hslClr" | "sysClr" | "schemeClr" | "prstClr"
    )
}

/// Refuse a colour the schema does not allow, before anything is touched.
pub(crate) fn validate(color: &Color) -> Result<(), ExportError> {
    if color.transforms.len() > MAX_COLOR_TRANSFORMS {
        return Err(ExportError::TooManyTransforms);
    }
    color
        .validate()
        .map_err(|_| ExportError::InvalidValue("colour"))
}

/// One attribute a colour value needs, in the model's terms.
enum AttrValue {
    Percent(i32),
    Int(i32),
    Hex(Rgb),
    Token(&'static str),
}

impl AttrValue {
    fn same_as(&self, text: &str) -> bool {
        match self {
            Self::Percent(v) => parse_percentage(text).is_some_and(|p| p.value == *v),
            Self::Int(v) => parse_int(text) == Some(*v),
            Self::Hex(v) => Rgb::from_hex(text) == Some(*v),
            Self::Token(v) => text == *v,
        }
    }

    /// How to write it. A percentage follows the spelling it replaces, else the
    /// part's conformance class.
    fn write(&self, percent_form: bool) -> String {
        match self {
            Self::Percent(v) if percent_form => format_percentage(*v),
            Self::Percent(v) | Self::Int(v) => v.to_string(),
            Self::Hex(v) => v.to_hex(),
            Self::Token(v) => (*v).to_owned(),
        }
    }
}

fn value_attributes(value: &ColorValue) -> Vec<(&'static str, Option<AttrValue>)> {
    use AttrValue::{Hex, Int, Percent, Token};
    match *value {
        ColorValue::ScRgb { r, g, b } => vec![
            ("r", Some(Percent(r.0))),
            ("g", Some(Percent(g.0))),
            ("b", Some(Percent(b.0))),
        ],
        ColorValue::Srgb(rgb) => vec![("val", Some(Hex(rgb)))],
        ColorValue::Hsl { hue, sat, lum } => vec![
            ("hue", Some(Int(hue.0))),
            ("sat", Some(Percent(sat.0))),
            ("lum", Some(Percent(lum.0))),
        ],
        ColorValue::System { val, last } => vec![
            ("val", Some(Token(val.as_xml()))),
            ("lastClr", last.map(Hex)),
        ],
        ColorValue::Scheme(s) => vec![("val", Some(Token(s.as_xml())))],
        ColorValue::Preset(p) => vec![("val", Some(Token(p.as_xml())))],
    }
}

/// Make the colour under `holder` be `color`, patching what differs. Returns
/// whether anything changed. Elements foreign to a colour are kept.
///
/// Changing the colour's kind replaces the colour element, so whatever unknown
/// attributes it had go with it: the edit named that node.
pub fn set_color(
    tree: &mut XmlTree,
    holder: ElementId,
    color: &Color,
) -> Result<bool, ExportError> {
    validate(color)?;
    let local = color.value.element_name();
    let name = dml_name(tree, holder, local)?;
    let strict = is_strict(tree, holder);
    let existing = tree
        .child_elements(holder)
        .into_iter()
        .find(|&c| is_dml(tree, c) && is_color_name(tree.local_name(c)));

    let (el, mut changed) = match existing {
        Some(el) if tree.local_name(el) == local => {
            let changed = patch_attributes(tree, el, &color.value, strict)?;
            (el, changed)
        }
        Some(el) => {
            let attrs = new_attributes(&color.value, strict);
            let borrowed: Vec<(&str, &str)> = attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();
            tree.replace_element(el, &name, &borrowed);
            (el, true)
        }
        None => {
            let attrs = new_attributes(&color.value, strict);
            let borrowed: Vec<(&str, &str)> = attrs.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let first = tree.child_elements(holder).first().copied();
            (tree.insert_element(holder, first, &name, &borrowed), true)
        }
    };
    changed |= patch_transforms(tree, el, &color.transforms, strict)?;
    Ok(changed)
}

fn new_attributes(value: &ColorValue, strict: bool) -> Vec<(&'static str, String)> {
    value_attributes(value)
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k, v.write(strict))))
        .collect()
}

fn patch_attributes(
    tree: &mut XmlTree,
    el: ElementId,
    value: &ColorValue,
    strict: bool,
) -> Result<bool, ExportError> {
    if !tree.attributes_valid(el) {
        return Err(ExportError::MalformedNode("colour"));
    }
    let mut changed = false;
    for (name, wanted) in value_attributes(value) {
        let current = tree.plain_attribute(el, name);
        match (wanted, current) {
            (Some(want), Some(text)) => {
                if !want.same_as(&text) {
                    tree.set_attribute(el, name, &want.write(strict || text.contains('%')));
                    changed = true;
                }
            }
            (Some(want), None) => {
                tree.set_attribute(el, name, &want.write(strict));
                changed = true;
            }
            (None, Some(_)) => {
                changed |= tree.remove_attribute(el, name);
            }
            (None, None) => {}
        }
    }
    Ok(changed)
}

fn read_existing(tree: &XmlTree, el: ElementId) -> Option<ColorTransform> {
    let kind = TransformKind::from_xml(tree.local_name(el))?;
    let domain = kind.domain();
    if domain == ValueDomain::None {
        return Some(ColorTransform::new(kind, 0));
    }
    let text = tree.plain_attribute(el, "val")?;
    let value = if domain.is_percentage() {
        parse_percentage(&text)?.value
    } else {
        parse_int(&text)?
    };
    domain
        .contains(value)
        .then_some(ColorTransform::new(kind, value))
}

fn patch_transforms(
    tree: &mut XmlTree,
    el: ElementId,
    wanted: &[ColorTransform],
    strict: bool,
) -> Result<bool, ExportError> {
    // Only transforms the importer believes are ours to compare and replace. One it
    // dropped is not in the model, so writing the model back must not delete it.
    let old: Vec<(ElementId, ColorTransform)> = tree
        .child_elements(el)
        .into_iter()
        .filter(|&c| is_dml(tree, c))
        .filter_map(|c| read_existing(tree, c).map(|t| (c, t)))
        .collect();
    if old.len() == wanted.len() && old.iter().zip(wanted).all(|((_, t), w)| t == w) {
        return Ok(false);
    }
    // Rebuilding the list loses the old spelling of values that did not change,
    // which is within the node the edit named.
    for &(c, _) in &old {
        tree.remove_element(el, c);
    }
    let before = tree.child_elements(el).first().copied();
    for t in wanted {
        let name = dml_name(tree, el, t.kind.as_xml())?;
        let domain = t.kind.domain();
        let text;
        let attrs: &[(&str, &str)] = if domain == ValueDomain::None {
            &[]
        } else {
            text = if domain.is_percentage() && strict {
                format_percentage(t.value)
            } else {
                t.value.to_string()
            };
            &[("val", text.as_str())]
        };
        tree.insert_element(el, before, &name, attrs);
    }
    Ok(true)
}
