//! Reads colours from an [`XmlTree`] into the model.
//!
//! Nothing here refuses a part. A colour that cannot be believed is left absent
//! and reported, and the tree keeps its bytes, so saving loses nothing. A finding
//! names an element and an attribute, never a value from the file.

use casual_slide_model::color::{
    Angle, Color, ColorChange, ColorTransform, ColorValue, Percentage, PresetColor, Rgb,
    SchemeColor, SystemColor, TransformKind, ValueDomain, parse_int, parse_percentage,
};
use casual_slide_ooxml::{ElementId, OoxmlLimits, XmlTree};

use crate::properties::reportable_name;

/// DrawingML, Transitional.
pub const DML_TRANSITIONAL: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
/// DrawingML, Strict.
pub const DML_STRICT: &str = "http://purl.oclc.org/ooxml/drawingml/main";

/// The default for [`OoxmlLimits::max_theme_list_entries`]. Far beyond real use.
pub const MAX_LIST_ENTRIES: usize = 4096;

/// What was noticed. Codes are stable; none of these stops a deck opening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FindingKind {
    /// A colour's required attribute is absent or unreadable; the colour is absent.
    BadColorValue,
    /// A colour has more than [`OoxmlLimits::max_color_transforms`]; the colour is absent.
    TooManyTransforms,
    /// A transform's value is absent or outside its type; that transform is
    /// dropped and the colour kept.
    BadTransform,
    /// An element's attributes are malformed or repeated; nothing is read from it.
    MalformedAttributes,
    /// A value is not one the schema allows; the field is absent.
    BadValue,
    /// A percentage had more precision than the model holds and was rounded.
    PercentRounded,
    /// A required element or attribute is absent.
    MissingRequired,
    /// An element repeats where one is allowed; the first was used.
    Duplicate,
    /// An element this reader does not model; preserved but not modelled.
    Unmodelled,
    /// A list passed [`OoxmlLimits::max_theme_list_entries`]; the rest is preserved but
    /// not modelled.
    ListCapped,
}

impl FindingKind {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::BadColorValue => "CS-CLR-0001",
            Self::TooManyTransforms => "CS-CLR-0002",
            Self::BadTransform => "CS-CLR-0003",
            Self::MalformedAttributes => "CS-CLR-0004",
            Self::BadValue => "CS-CLR-0005",
            Self::PercentRounded => "CS-CLR-0006",
            Self::MissingRequired => "CS-CLR-0007",
            Self::Duplicate => "CS-CLR-0008",
            Self::Unmodelled => "CS-CLR-0009",
            Self::ListCapped => "CS-CLR-0010",
        }
    }
}

/// One thing noticed while reading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// What.
    pub kind: FindingKind,
    /// The element's qualified name if the schemas have it, else
    /// [`UNKNOWN_NAME`](crate::properties::UNKNOWN_NAME). A hostile part chooses it.
    pub element: String,
    /// The attribute or child concerned, if any; from the schema, not the file.
    pub detail: &'static str,
}

/// The caps a read of colours and themes works under, from [`OoxmlLimits`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorBounds {
    /// [`OoxmlLimits::max_color_transforms`].
    pub transforms: usize,
    /// [`OoxmlLimits::max_theme_list_entries`].
    pub list_entries: usize,
    /// [`OoxmlLimits::max_color_findings`].
    pub findings: usize,
}

impl Default for ColorBounds {
    fn default() -> Self {
        Self::from(&OoxmlLimits::default())
    }
}

impl From<&OoxmlLimits> for ColorBounds {
    fn from(limits: &OoxmlLimits) -> Self {
        Self {
            transforms: limits.max_color_transforms,
            list_entries: limits.max_theme_list_entries,
            findings: limits.max_color_findings,
        }
    }
}

/// Findings, in document order and bounded: past their bound only the count grows, so a
/// hostile part cannot make the report the cost. These travel with every read of a colour,
/// so they carry the bounds that read is under.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Findings {
    /// The findings kept.
    pub items: Vec<Finding>,
    /// How many there were, kept or not.
    pub total: usize,
    /// The caps this read is under.
    pub bounds: ColorBounds,
}

/// The default for [`OoxmlLimits::max_color_findings`].
pub const MAX_FINDINGS: usize = 1024;

impl Findings {
    /// An empty report for a read under these limits.
    pub fn with_limits(limits: &OoxmlLimits) -> Self {
        Self {
            bounds: limits.into(),
            ..Self::default()
        }
    }

    /// Note one.
    pub fn push(&mut self, kind: FindingKind, tree: &XmlTree, at: ElementId, detail: &'static str) {
        self.total = self.total.saturating_add(1);
        if self.items.len() < self.bounds.findings {
            let element = reportable_name(tree.qualified_name(at)).to_owned();
            self.items.push(Finding {
                kind,
                element,
                detail,
            });
        }
    }

    /// The distinct codes, sorted.
    pub fn codes(&self) -> Vec<&'static str> {
        let mut codes: Vec<_> = self.items.iter().map(|f| f.kind.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        codes
    }
}

/// An element's children up to a bound, so no loop here is sized by the file.
/// Each one costs a namespace lookup through its ancestors. Past the bound they
/// are preserved but not modelled, and that is reported.
pub(crate) fn bounded_children(
    tree: &XmlTree,
    el: ElementId,
    findings: &mut Findings,
) -> Vec<ElementId> {
    let mut children = tree.child_elements(el);
    let bound = findings.bounds.list_entries.saturating_add(8);
    if children.len() > bound {
        findings.push(FindingKind::ListCapped, tree, el, "");
        children.truncate(bound);
    }
    children
}

/// Whether an element is in DrawingML, Transitional or Strict.
pub fn is_dml(tree: &XmlTree, id: ElementId) -> bool {
    tree.namespace_uri(id)
        .is_some_and(|uri| uri == DML_TRANSITIONAL || uri == DML_STRICT)
}

/// The local name of an element if it is in DrawingML.
pub(crate) fn dml_name(tree: &XmlTree, id: ElementId) -> Option<&str> {
    is_dml(tree, id).then(|| tree.local_name(id))
}

/// Whether this element is one of `EG_ColorChoice`.
pub fn is_color_element(tree: &XmlTree, id: ElementId) -> bool {
    matches!(
        dml_name(tree, id),
        Some("scrgbClr" | "srgbClr" | "hslClr" | "sysClr" | "schemeClr" | "prstClr")
    )
}

/// Read `EG_ColorChoice` from a holder that has exactly one colour as a child.
/// `None`, reported, if it has none or the colour cannot be believed.
pub fn read_color_choice(
    tree: &XmlTree,
    holder: ElementId,
    findings: &mut Findings,
) -> Option<Color> {
    let mut found = None;
    let mut any = false;
    for child in bounded_children(tree, holder, findings) {
        if !is_color_element(tree, child) {
            continue;
        }
        if any {
            findings.push(FindingKind::Duplicate, tree, child, "colour");
        } else {
            any = true;
            found = read_color(tree, child, findings);
        }
    }
    if !any {
        findings.push(FindingKind::MissingRequired, tree, holder, "colour");
    }
    found
}

/// Read an optional `EG_ColorChoice` among other children: `None` silently if
/// there is none.
pub fn read_optional_color(
    tree: &XmlTree,
    holder: ElementId,
    findings: &mut Findings,
) -> Option<Color> {
    let has = bounded_children(tree, holder, findings)
        .into_iter()
        .any(|c| is_color_element(tree, c));
    if has {
        read_color_choice(tree, holder, findings)
    } else {
        None
    }
}

/// Read one colour element and its transforms, in document order.
pub fn read_color(tree: &XmlTree, el: ElementId, findings: &mut Findings) -> Option<Color> {
    if !tree.attributes_valid(el) {
        findings.push(FindingKind::MalformedAttributes, tree, el, "");
        return None;
    }
    let value = read_value(tree, el, findings)?;
    let mut transforms = Vec::new();
    let mut seen = 0usize;
    for child in bounded_children(tree, el, findings) {
        let kind = dml_name(tree, child).and_then(TransformKind::from_xml);
        let Some(kind) = kind else {
            findings.push(FindingKind::Unmodelled, tree, child, "");
            continue;
        };
        seen += 1;
        if seen > findings.bounds.transforms {
            findings.push(FindingKind::TooManyTransforms, tree, el, "");
            return None;
        }
        if let Some(t) = read_transform(tree, child, kind, findings) {
            transforms.push(t);
        }
    }
    Some(Color { value, transforms })
}

fn read_value(tree: &XmlTree, el: ElementId, findings: &mut Findings) -> Option<ColorValue> {
    let bad = |findings: &mut Findings, attr| {
        findings.push(FindingKind::BadColorValue, tree, el, attr);
    };
    let attr = |name: &str| tree.plain_attribute(el, name);
    let percent = |findings: &mut Findings, name: &'static str| {
        let Some(read) = attr(name).as_deref().and_then(parse_percentage) else {
            bad(findings, name);
            return None;
        };
        if read.rounded {
            findings.push(FindingKind::PercentRounded, tree, el, name);
        }
        Some(Percentage(read.value))
    };
    match tree.local_name(el) {
        "srgbClr" => {
            let rgb = attr("val").as_deref().and_then(Rgb::from_hex);
            if rgb.is_none() {
                bad(findings, "val");
            }
            Some(ColorValue::Srgb(rgb?))
        }
        "scrgbClr" => {
            let (r, g, b) = (
                percent(findings, "r"),
                percent(findings, "g"),
                percent(findings, "b"),
            );
            Some(ColorValue::ScRgb {
                r: r?,
                g: g?,
                b: b?,
            })
        }
        "hslClr" => {
            let hue = attr("hue")
                .as_deref()
                .and_then(parse_int)
                .filter(|v| ValueDomain::PositiveFixedAngle.contains(*v));
            if hue.is_none() {
                bad(findings, "hue");
            }
            let (sat, lum) = (percent(findings, "sat"), percent(findings, "lum"));
            Some(ColorValue::Hsl {
                hue: Angle(hue?),
                sat: sat?,
                lum: lum?,
            })
        }
        "sysClr" => {
            let val = attr("val").as_deref().and_then(SystemColor::from_xml);
            if val.is_none() {
                bad(findings, "val");
            }
            let last = match attr("lastClr") {
                None => None,
                Some(text) => {
                    let parsed = Rgb::from_hex(&text);
                    if parsed.is_none() {
                        // lastClr is a cache. The colour stands without it.
                        findings.push(FindingKind::BadValue, tree, el, "lastClr");
                    }
                    parsed
                }
            };
            Some(ColorValue::System { val: val?, last })
        }
        "schemeClr" => {
            let val = attr("val").as_deref().and_then(SchemeColor::from_xml);
            if val.is_none() {
                bad(findings, "val");
            }
            Some(ColorValue::Scheme(val?))
        }
        "prstClr" => {
            let val = attr("val").as_deref().and_then(PresetColor::from_xml);
            if val.is_none() {
                bad(findings, "val");
            }
            Some(ColorValue::Preset(val?))
        }
        _ => {
            findings.push(FindingKind::Unmodelled, tree, el, "");
            None
        }
    }
}

/// Read one transform element's value; `None`, reported, if unbelievable.
fn read_transform(
    tree: &XmlTree,
    el: ElementId,
    kind: TransformKind,
    findings: &mut Findings,
) -> Option<ColorTransform> {
    let domain = kind.domain();
    if domain == ValueDomain::None {
        return Some(ColorTransform::new(kind, 0));
    }
    let bad = |findings: &mut Findings| {
        findings.push(FindingKind::BadTransform, tree, el, "val");
    };
    if !tree.attributes_valid(el) {
        findings.push(FindingKind::MalformedAttributes, tree, el, "");
        return None;
    }
    let text = tree.plain_attribute(el, "val");
    let parsed = if domain.is_percentage() {
        text.as_deref().and_then(parse_percentage).map(|read| {
            if read.rounded {
                findings.push(FindingKind::PercentRounded, tree, el, "val");
            }
            read.value
        })
    } else {
        text.as_deref().and_then(parse_int)
    };
    match parsed.filter(|v| domain.contains(*v)) {
        Some(value) => Some(ColorTransform::new(kind, value)),
        None => {
            bad(findings);
            None
        }
    }
}

/// `CT_ColorMRU`: every colour under this element, bounded.
pub fn read_color_mru(tree: &XmlTree, el: ElementId, findings: &mut Findings) -> Vec<Color> {
    let mut out = Vec::new();
    for child in bounded_children(tree, el, findings) {
        if out.len() >= findings.bounds.list_entries {
            findings.push(FindingKind::ListCapped, tree, el, "");
            break;
        }
        if is_color_element(tree, child) {
            out.extend(read_color(tree, child, findings));
        } else {
            findings.push(FindingKind::Unmodelled, tree, child, "");
        }
    }
    out
}

/// `CT_ColorChangeEffect`.
pub fn read_color_change(tree: &XmlTree, el: ElementId, findings: &mut Findings) -> ColorChange {
    let mut change = ColorChange {
        from: None,
        to: None,
        use_alpha: None,
    };
    if !tree.attributes_valid(el) {
        findings.push(FindingKind::MalformedAttributes, tree, el, "");
        return change;
    }
    change.use_alpha = match tree.plain_attribute(el, "useA").as_deref() {
        None => None,
        Some("1" | "true" | "on") => Some(true),
        Some("0" | "false" | "off") => Some(false),
        Some(_) => {
            findings.push(FindingKind::BadValue, tree, el, "useA");
            None
        }
    };
    for child in bounded_children(tree, el, findings) {
        match dml_name(tree, child) {
            Some("clrFrom") => change.from = read_color_choice(tree, child, findings),
            Some("clrTo") => change.to = read_color_choice(tree, child, findings),
            _ => findings.push(FindingKind::Unmodelled, tree, child, ""),
        }
    }
    change
}

/// `CT_ColorReplaceEffect`.
pub fn read_color_replace(tree: &XmlTree, el: ElementId, findings: &mut Findings) -> Option<Color> {
    read_color_choice(tree, el, findings)
}
