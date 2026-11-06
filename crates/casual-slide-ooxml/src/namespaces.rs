//! The namespaces this engine knows: the Strict and Transitional spellings of
//! each, and the fixed list markup compatibility treats as understood.
//!
//! ECMA-376 spells one vocabulary with two sets of URIs. A file is read through
//! [`to_transitional`], so code above this layer sees one set, and it is written
//! back in the form it arrived in: `XmlTree` never rewrites a declaration, and
//! anything newly written uses [`Conformance::spell`](crate::Conformance::spell).

use std::borrow::Cow;

/// The markup compatibility namespace (ECMA-376 Part 3, 7.1). Strict uses it too.
pub const MARKUP_COMPATIBILITY: &str =
    "http://schemas.openxmlformats.org/markup-compatibility/2006";

/// `(Strict, Transitional)` for every namespace that differs. The vendored schemas
/// in `schemas/ooxml-strict` and `schemas/ooxml` hold this table to account.
pub const STRICT_TO_TRANSITIONAL: &[(&str, &str)] = &[
    (
        "http://purl.oclc.org/ooxml/presentationml/main",
        "http://schemas.openxmlformats.org/presentationml/2006/main",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/main",
        "http://schemas.openxmlformats.org/drawingml/2006/main",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/chart",
        "http://schemas.openxmlformats.org/drawingml/2006/chart",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/chartDrawing",
        "http://schemas.openxmlformats.org/drawingml/2006/chartDrawing",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/diagram",
        "http://schemas.openxmlformats.org/drawingml/2006/diagram",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/lockedCanvas",
        "http://schemas.openxmlformats.org/drawingml/2006/lockedCanvas",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/picture",
        "http://schemas.openxmlformats.org/drawingml/2006/picture",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/spreadsheetDrawing",
        "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/wordprocessingDrawing",
        "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/sharedTypes",
        "http://schemas.openxmlformats.org/officeDocument/2006/sharedTypes",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/math",
        "http://schemas.openxmlformats.org/officeDocument/2006/math",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/customXml",
        "http://schemas.openxmlformats.org/officeDocument/2006/customXml",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/customProperties",
        "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/extendedProperties",
        "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/docPropsVTypes",
        "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/bibliography",
        "http://schemas.openxmlformats.org/officeDocument/2006/bibliography",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/characteristics",
        "http://schemas.openxmlformats.org/officeDocument/2006/characteristics",
    ),
    (
        "http://purl.oclc.org/ooxml/schemaLibrary/main",
        "http://schemas.openxmlformats.org/schemaLibrary/2006/main",
    ),
    (
        "http://purl.oclc.org/ooxml/spreadsheetml/main",
        "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
    ),
    (
        "http://purl.oclc.org/ooxml/wordprocessingml/main",
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
    ),
];

/// Relationship-type prefixes, Strict then Transitional.
const STRICT_REL_PREFIX: &str = "http://purl.oclc.org/ooxml/officeDocument/relationships/";
const TRANSITIONAL_REL_PREFIX: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";

/// Relationship-type names spelled differently once the prefix is mapped.
const REL_NAMES: &[(&str, &str)] = &[
    ("extendedProperties", "extended-properties"),
    ("customProperties", "custom-properties"),
];

const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";

/// Namespaces in neither of the tables above that ECMA-376 defines for a package.
const OTHER_UNDERSTOOD: &[&str] = &[
    "http://schemas.openxmlformats.org/package/2006/relationships",
    "http://schemas.openxmlformats.org/package/2006/content-types",
    "http://schemas.openxmlformats.org/package/2006/metadata/core-properties",
    "http://purl.org/dc/elements/1.1/",
    "http://purl.org/dc/terms/",
    "http://purl.org/dc/dcmitype/",
    "http://www.w3.org/2001/XMLSchema-instance",
    "urn:schemas-microsoft-com:vml",
    "urn:schemas-microsoft-com:office:office",
    "urn:schemas-microsoft-com:office:powerpoint",
];

/// Whether markup compatibility treats `uri` as understood.
///
/// The list is fixed and deliberately ECMA-376 alone: Microsoft's extension
/// namespaces such as `p14` are not on it, so a `Choice` that requires one gives
/// way to the `Fallback` every such file carries. Both branches stay in the bytes.
/// A namespace joins this list when the importer maps it, not before.
pub fn is_understood(uri: &str) -> bool {
    uri == MARKUP_COMPATIBILITY
        || uri == XML_NAMESPACE
        || OTHER_UNDERSTOOD.contains(&uri)
        || STRICT_TO_TRANSITIONAL
            .iter()
            .any(|&(s, t)| s == uri || t == uri)
}

/// Every namespace [`is_understood`] accepts, in a fixed order.
pub fn understood_namespaces() -> Vec<&'static str> {
    let mut all = vec![MARKUP_COMPATIBILITY, XML_NAMESPACE];
    all.extend_from_slice(OTHER_UNDERSTOOD);
    for &(strict, transitional) in STRICT_TO_TRANSITIONAL {
        all.push(strict);
        all.push(transitional);
    }
    all
}

/// The Transitional spelling of a namespace. Any other URI comes back unchanged.
pub fn to_transitional(uri: &str) -> &str {
    STRICT_TO_TRANSITIONAL
        .iter()
        .find_map(|&(strict, transitional)| (strict == uri).then_some(transitional))
        .unwrap_or(uri)
}

/// The Strict spelling of a namespace. Any other URI comes back unchanged.
pub fn to_strict(uri: &str) -> &str {
    STRICT_TO_TRANSITIONAL
        .iter()
        .find_map(|&(strict, transitional)| (transitional == uri).then_some(strict))
        .unwrap_or(uri)
}

/// The Transitional spelling of a relationship type, which is a namespace-like
/// prefix and a name, two of which Strict spells differently.
pub fn relationship_type_to_transitional(rel_type: &str) -> Cow<'_, str> {
    match rel_type.strip_prefix(STRICT_REL_PREFIX) {
        None => Cow::Borrowed(rel_type),
        Some(name) => {
            let name = REL_NAMES
                .iter()
                .find_map(|&(strict, transitional)| (strict == name).then_some(transitional))
                .unwrap_or(name);
            Cow::Owned(format!("{TRANSITIONAL_REL_PREFIX}{name}"))
        }
    }
}

/// The Strict spelling of a relationship type.
pub fn relationship_type_to_strict(rel_type: &str) -> Cow<'_, str> {
    match rel_type.strip_prefix(TRANSITIONAL_REL_PREFIX) {
        None => Cow::Borrowed(rel_type),
        Some(name) => {
            let name = REL_NAMES
                .iter()
                .find_map(|&(strict, transitional)| (transitional == name).then_some(strict))
                .unwrap_or(name);
            Cow::Owned(format!("{STRICT_REL_PREFIX}{name}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PML_STRICT: &str = "http://purl.oclc.org/ooxml/presentationml/main";
    const PML: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";

    #[test]
    fn strict_and_transitional_map_both_ways() {
        assert_eq!(to_transitional(PML_STRICT), PML);
        assert_eq!(to_strict(PML), PML_STRICT);
        for &(strict, transitional) in STRICT_TO_TRANSITIONAL {
            assert_eq!(to_transitional(strict), transitional);
            assert_eq!(to_strict(transitional), strict);
            assert_eq!(to_transitional(transitional), transitional);
            assert_eq!(to_strict(strict), strict);
        }
    }

    #[test]
    fn a_namespace_in_neither_form_passes_through() {
        for uri in [
            MARKUP_COMPATIBILITY,
            "http://schemas.microsoft.com/office/powerpoint/2010/main",
            "http://purl.oclc.org/ooxml/presentationml/main/",
            "http://example.com/evil",
            "",
        ] {
            assert_eq!(to_transitional(uri), uri);
            assert_eq!(to_strict(uri), uri);
        }
    }

    #[test]
    fn relationship_types_map_with_their_two_renamed_members() {
        let strict = "http://purl.oclc.org/ooxml/officeDocument/relationships/slide";
        let transitional =
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide";
        assert_eq!(relationship_type_to_transitional(strict), transitional);
        assert_eq!(relationship_type_to_strict(transitional), strict);
        assert_eq!(
            relationship_type_to_transitional(
                "http://purl.oclc.org/ooxml/officeDocument/relationships/extendedProperties"
            ),
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties"
        );
        assert_eq!(
            relationship_type_to_strict(
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties"
            ),
            "http://purl.oclc.org/ooxml/officeDocument/relationships/customProperties"
        );
        let other = "http://schemas.microsoft.com/office/2006/relationships/vbaProject";
        assert_eq!(relationship_type_to_transitional(other), other);
        assert_eq!(relationship_type_to_strict(other), other);
    }

    #[test]
    fn microsoft_extensions_are_not_understood() {
        assert!(is_understood(PML));
        assert!(is_understood(PML_STRICT));
        assert!(is_understood(MARKUP_COMPATIBILITY));
        assert!(!is_understood(
            "http://schemas.microsoft.com/office/powerpoint/2010/main"
        ));
        assert!(!is_understood("http://example.com/evil"));
    }
}
