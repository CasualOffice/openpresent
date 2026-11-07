//! OPC part-name equivalence, per ECMA-376 Part 2, and the index of a package's
//! entries by it. The rule itself is `casual-slide-package`'s, so the package and
//! the part graph cannot disagree about which names are one part (`PKG-02`).

use std::collections::BTreeMap;

use casual_slide_package::{PackageError, part_entry_key as entry_key_of};
pub use casual_slide_package::{part_name_key, part_names_equivalent};

use crate::error::OoxmlError;

/// More layers of percent-encoding than this is not a name anyone wrote.
#[cfg(test)]
const MAX_ENCODING_LAYERS: usize = 8;

/// The key an entry is stored under. See [`casual_slide_package::part_entry_key`].
pub(crate) fn entry_key(name: &str) -> String {
    entry_key_of(name)
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
