//! The built-in blank presentation, for File > New (`VS-08`): one 16:9 master with
//! title, body, date, footer and slide-number placeholders, four layouts (Title Slide,
//! Title and Content, Title Only, Blank), an Office-coloured theme, and one Title Slide.
//! Its parts are plain XML in `src/blank/`, readable and diffable; they are zipped
//! here, so the same build always gives the same bytes.

use casual_slide_package::{PackageError, build_package};

const PARTS: [(&str, &str); 15] = [
    (
        "[Content_Types].xml",
        include_str!("blank/content-types.xml"),
    ),
    ("_rels/.rels", include_str!("blank/root.rels")),
    (
        "ppt/presentation.xml",
        include_str!("blank/presentation.xml"),
    ),
    (
        "ppt/_rels/presentation.xml.rels",
        include_str!("blank/presentation.xml.rels"),
    ),
    ("ppt/presProps.xml", include_str!("blank/presProps.xml")),
    ("ppt/viewProps.xml", include_str!("blank/viewProps.xml")),
    (
        "ppt/slideMasters/slideMaster1.xml",
        include_str!("blank/slideMaster1.xml"),
    ),
    (
        "ppt/slideMasters/_rels/slideMaster1.xml.rels",
        include_str!("blank/slideMaster1.xml.rels"),
    ),
    (
        "ppt/slideLayouts/slideLayout1.xml",
        include_str!("blank/slideLayout1.xml"),
    ),
    (
        "ppt/slideLayouts/slideLayout2.xml",
        include_str!("blank/slideLayout2.xml"),
    ),
    (
        "ppt/slideLayouts/slideLayout3.xml",
        include_str!("blank/slideLayout3.xml"),
    ),
    (
        "ppt/slideLayouts/slideLayout4.xml",
        include_str!("blank/slideLayout4.xml"),
    ),
    ("ppt/theme/theme1.xml", include_str!("blank/theme1.xml")),
    ("ppt/slides/slide1.xml", include_str!("blank/slide1.xml")),
    (
        "ppt/slides/_rels/slide1.xml.rels",
        include_str!("blank/slide1.xml.rels"),
    ),
];

/// Every layout's relationships: its master.
const LAYOUT_RELS: &str = include_str!("blank/slideLayout.xml.rels");

/// The blank presentation's bytes.
///
/// # Errors
///
/// Only if the archive cannot be written, which a fixed set of safe names never causes.
pub fn blank_deck() -> Result<Vec<u8>, PackageError> {
    let mut parts: Vec<(String, &[u8])> = PARTS
        .iter()
        .map(|(name, xml)| ((*name).to_owned(), xml.as_bytes()))
        .collect();
    for n in 1..=4 {
        parts.push((
            format!("ppt/slideLayouts/_rels/slideLayout{n}.xml.rels"),
            LAYOUT_RELS.as_bytes(),
        ));
    }
    let refs: Vec<(&str, &[u8])> = parts.iter().map(|(n, b)| (n.as_str(), *b)).collect();
    build_package(&refs)
}
