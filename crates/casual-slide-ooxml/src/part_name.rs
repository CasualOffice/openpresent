//! OPC part-name equivalence, per ECMA-376 Part 2.
//!
//! A relationship target and a ZIP item name for one part can be spelled
//! differently: `../media/my image.png` against `ppt/media/my%20image.png`, or
//! `é.png` against `%C3%A9.png`. Part 2 maps ZIP item names to Unicode part names
//! by un-percent-encoding (7.3.5), and compares them by ASCII case-insensitive
//! matching over code points (6.2.2.3). So `É` and `é` are different parts, and so
//! are the NFC and NFD spellings of one letter, which the standard only recommends
//! a package avoid.
//!
//! Decoding is exactly the step where a disguised path turns into a real one, so
//! everything that decodes to traversal is refused rather than normalized, at
//! every layer of encoding, for whichever reader decodes once more than we do.

use std::collections::BTreeMap;

use casual_slide_package::PackageError;

use crate::error::OoxmlError;

/// More layers of percent-encoding than this is not a name anyone wrote.
const MAX_ENCODING_LAYERS: usize = 8;

/// Marks an entry with no valid part name, so no target can ever reach it.
const UNREACHABLE: char = '\0';

/// Characters a later normalizer or a file system may turn into a separator.
const SLASH_LOOKALIKES: [char; 6] = [
    '\u{2044}', '\u{2215}', '\u{29F8}', '\u{FF0F}', '\u{FE68}', '\u{FF3C}',
];

/// Characters compatibility normalization turns into one or more dots.
const DOT_LOOKALIKES: [char; 5] = ['\u{2024}', '\u{2025}', '\u{2026}', '\u{FE52}', '\u{FF0E}'];

/// The form two equivalent part names share, or `None` for a name to be refused.
///
/// Takes a ZIP item name, a part name with its leading slash, or a target already
/// resolved against its source. Percent-encoding is removed once, as the standard
/// has it, then ASCII letters are lowered. Refused: percent-encoded dots and
/// separators in any layer (`%2e`, `%252e`), bytes that are not UTF-8, which covers
/// overlong forms such as `%c0%ae`, empty segments, segments ending in a dot,
/// backslashes, control characters, and look-alikes of `/` and `..`.
pub fn part_name_key(name: &str) -> Option<String> {
    let name = name.strip_prefix('/').unwrap_or(name);
    if has_encoded_structure(name) {
        return None;
    }
    let decoded = percent_decode(name)?;
    if !segments_are_safe(&decoded) {
        return None;
    }
    let mut layer = decoded.clone();
    let mut layers = 0;
    while layer.contains('%') {
        if has_encoded_structure(&layer) {
            return None;
        }
        let next = percent_decode(&layer)?;
        if next == layer {
            break;
        }
        if !segments_are_safe(&next) {
            return None;
        }
        layers += 1;
        if layers >= MAX_ENCODING_LAYERS {
            return None;
        }
        layer = next;
    }
    Some(decoded.to_ascii_lowercase())
}

/// Whether two names denote one part. A refused name is equivalent to nothing.
pub fn part_names_equivalent(a: &str, b: &str) -> bool {
    match (part_name_key(a), part_name_key(b)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// [`part_name_key`] for a name that has to be stored somewhere: an entry with no
/// valid name still has to be retained, read by its own name, and reported, while
/// no target resolves to it. The empty name is the package itself.
pub(crate) fn entry_key(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }
    part_name_key(name).unwrap_or_else(|| format!("{UNREACHABLE}{}", name.to_ascii_lowercase()))
}

fn has_encoded_structure(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    ["%2e", "%2f", "%5c"].iter().any(|e| lower.contains(e))
}

/// One layer of percent-decoding. A `%` not followed by two hex digits stands for
/// itself, as Part 2's Annex A has `/%XY.xml` and `/%25XY.xml` name one part.
fn percent_decode(name: &str) -> Option<String> {
    let bytes = name.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut rest = bytes;
    while let Some((&byte, tail)) = rest.split_first() {
        if byte == b'%'
            && let [high, low, after @ ..] = tail
            && let (Some(high), Some(low)) = (hex(*high), hex(*low))
        {
            out.push(high * 16 + low);
            rest = after;
        } else {
            out.push(byte);
            rest = tail;
        }
    }
    String::from_utf8(out).ok()
}

fn hex(byte: u8) -> Option<u8> {
    char::from(byte)
        .to_digit(16)
        .and_then(|d| u8::try_from(d).ok())
}

fn segments_are_safe(name: &str) -> bool {
    name.split('/').all(|segment| {
        !segment.is_empty()
            && !segment.ends_with('.')
            && !segment.contains(SLASH_LOOKALIKES)
            && !segment.chars().any(|c| c == '\\' || c.is_control())
            // Windows drops trailing spaces and dots, so `.. ` is `..` to it.
            && !segment
                .chars()
                .all(|c| matches!(c, '.' | ' ') || DOT_LOOKALIKES.contains(&c))
    })
}

/// Entry names by their equivalence key.
#[derive(Debug, Default)]
pub(crate) struct PartIndex {
    by_key: BTreeMap<String, String>,
}

impl PartIndex {
    /// Index every part. Two entries naming one part are refused as a duplicate:
    /// a scanner approves one, an editor opens the other (`SEC-P14`).
    pub(crate) fn build(names: Vec<String>) -> Result<Self, OoxmlError> {
        let mut by_key = BTreeMap::new();
        for name in names {
            if name.ends_with('/') {
                continue;
            }
            if by_key.insert(entry_key(&name), name.clone()).is_some() {
                return Err(PackageError::DuplicatePart { path: name }.into());
            }
        }
        Ok(Self { by_key })
    }

    /// The entry for a name in any equivalent spelling.
    pub(crate) fn find(&self, name: &str) -> Option<&str> {
        self.by_key.get(&entry_key(name)).map(String::as_str)
    }

    /// The entry for a key from [`part_name_key`].
    pub(crate) fn find_key(&self, key: &str) -> Option<&str> {
        self.by_key.get(key).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encoding_and_ascii_case_do_not_change_a_part() {
        for (a, b) in [
            ("ppt/media/my image.png", "ppt/media/my%20image.png"),
            ("ppt/media/é.png", "ppt/media/%C3%A9.png"),
            ("ppt/media/é.png", "ppt/media/%c3%a9.png"),
            ("/ppt/slides/slide1.xml", "PPT/Slides/SLIDE1.XML"),
            ("%41.xml", "a.xml"),
            ("[Content_Types].xml", "%5BContent_Types%5D.xml"),
            // Part 2, Annex A: a bare percent stands for itself.
            ("%XY.xml", "%25XY.xml"),
            ("100%.png", "100%25.png"),
        ] {
            assert!(part_names_equivalent(a, b), "{a} and {b}");
        }
    }

    #[test]
    fn equivalence_is_ascii_only_and_decodes_once() {
        for (a, b) in [
            // Part 2, 6.2.2.3: only A to Z fold.
            ("é.png", "É.png"),
            ("%C3%A9.png", "%C3%89.png"),
            // NFC and NFD are different code points, so different names.
            ("\u{e9}.png", "e\u{301}.png"),
            // Part 2, Annex A: `/%2541.xml` is not `/A.xml`.
            ("%2541.xml", "A.xml"),
            ("a.xml", "b.xml"),
        ] {
            assert!(!part_names_equivalent(a, b), "{a} and {b}");
        }
    }

    #[test]
    fn what_decodes_to_traversal_is_refused() {
        for name in [
            "ppt/%2e%2e/slide.xml",
            "ppt/%2E%2E/%2E%2E/etc/passwd",
            "%2e%2e%2fetc%2fpasswd",
            "ppt%2fslide.xml",
            "ppt%5cslide.xml",
            "ppt/a%2eb.xml",
            // Double and triple encoding, for a reader that decodes again.
            "ppt/%252e%252e/slide.xml",
            "ppt/%25252e%25252e/slide.xml",
            "ppt/%252fslide.xml",
            // Overlong UTF-8 for `.` and `/`, and bytes that are not UTF-8 at all.
            "ppt/%c0%ae%c0%ae/slide.xml",
            "ppt/%c0%afslide.xml",
            "ppt/%ff.png",
            // Separators and terminators after decoding.
            "ppt\\slide.xml",
            "..\\..\\evil.xml",
            "ppt/a%00.png",
            "ppt/a%0a.png",
            // Raw traversal, and what the next normalizer would make of it.
            "ppt/../slide.xml",
            "ppt/./slide.xml",
            "ppt/.../slide.xml",
            "ppt/.. /slide.xml",
            "ppt/slide.",
            "ppt//slide.xml",
            "ppt/\u{FF0E}\u{FF0E}/slide.xml",
            "ppt/%EF%BC%8E%EF%BC%8E/slide.xml",
            "ppt/\u{2025}/slide.xml",
            "ppt\u{FF0F}slide.xml",
            "ppt%E2%88%95slide.xml",
            "",
        ] {
            assert_eq!(part_name_key(name), None, "{name:?}");
        }
    }

    #[test]
    fn too_many_layers_of_encoding_are_refused() {
        let mut name = String::from("a%41.xml");
        for _ in 0..MAX_ENCODING_LAYERS + 1 {
            name = name.replace('%', "%25");
        }
        assert_eq!(part_name_key(&name), None);
    }

    #[test]
    fn ordinary_names_survive() {
        for name in [
            "_rels/.rels",
            "ppt/_rels/presentation.xml.rels",
            "[Content_Types].xml",
            "ppt/media/a…b.png",
            "ppt/media/a b.png",
            "ppt/media/d’été.png",
        ] {
            assert!(part_name_key(name).is_some(), "{name}");
        }
    }

    #[test]
    fn a_name_with_no_valid_form_is_kept_and_unreachable() {
        let index = PartIndex::build(vec!["ppt/%ff.png".to_owned(), "ppt/".to_owned()]).unwrap();
        assert_eq!(index.find("ppt/%FF.PNG"), Some("ppt/%ff.png"));
        assert_eq!(part_name_key("ppt/%ff.png"), None);
    }

    #[test]
    fn two_entries_naming_one_part_are_refused() {
        let err = PartIndex::build(vec!["a b.png".to_owned(), "a%20b.png".to_owned()]).unwrap_err();
        assert_eq!(err.code(), "CS-PKG-0007");
    }
}
