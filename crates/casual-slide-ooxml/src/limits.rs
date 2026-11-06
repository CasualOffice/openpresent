//! Per-part XML bounds and graph bounds, on top of the package's.

use casual_slide_package::PackageLimits;

/// Bounds applied while inspecting a presentation package.
#[derive(Debug, Clone, Copy)]
pub struct OoxmlLimits {
    /// Limits for the underlying ZIP/OPC package.
    pub package: PackageLimits,
    /// Maximum size of one XML part handed to the walker, in bytes.
    pub max_xml_part_bytes: usize,
    /// Maximum XML elements per part.
    pub max_xml_elements: usize,
    /// Maximum XML nesting depth per part.
    pub max_xml_depth: usize,
    /// Maximum attributes on one element.
    pub max_xml_attributes: usize,
    /// Maximum relationships declared by one part.
    pub max_part_relationships: usize,
    /// Maximum relationships across the whole package.
    pub max_total_relationships: usize,
    /// Maximum entries in `[Content_Types].xml`.
    pub max_content_types: usize,
}

impl Default for OoxmlLimits {
    fn default() -> Self {
        Self {
            package: PackageLimits::default(),
            max_xml_part_bytes: 256 << 20,
            max_xml_elements: 20_000_000,
            max_xml_depth: 256,
            max_xml_attributes: 1024,
            max_part_relationships: 200_000,
            max_total_relationships: 2_000_000,
            max_content_types: 200_000,
        }
    }
}
