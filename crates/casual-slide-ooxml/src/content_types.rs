//! `[Content_Types].xml`.

use std::collections::BTreeMap;

use crate::error::OoxmlError;
use crate::limits::OoxmlLimits;
use crate::part_name::entry_key;
use crate::xml::{required, walk, wrong_root};

pub(crate) const CONTENT_TYPES_PART: &str = "[Content_Types].xml";

/// What a package declares about its parts' types. An `<Override>` names one
/// part; a `<Default>` claims every part with an extension; the override wins.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentTypes {
    overrides: BTreeMap<String, String>,
    defaults: BTreeMap<String, String>,
}

impl ContentTypes {
    /// Parse the part. A part declared twice with different types is refused,
    /// since readers would disagree about what it is.
    pub fn parse(bytes: &[u8], limits: &OoxmlLimits) -> Result<Self, OoxmlError> {
        let part = CONTENT_TYPES_PART;
        let mut out = Self::default();
        let mut count = 0usize;
        walk(bytes, part, limits, |e, depth| {
            let name = e.local_name();
            let name = name.as_ref();
            if depth == 1 {
                return if name == b"Types" {
                    Ok(())
                } else {
                    Err(wrong_root(part, "the root element is not Types"))
                };
            }
            let (map, key) = match name {
                b"Override" => (
                    &mut out.overrides,
                    entry_key(required(e, b"PartName", part)?.trim_start_matches('/')),
                ),
                b"Default" => (
                    &mut out.defaults,
                    required(e, b"Extension", part)?.to_ascii_lowercase(),
                ),
                _ => return Ok(()),
            };
            count += 1;
            if count > limits.max_content_types {
                return Err(OoxmlError::TooManyContentTypes {
                    limit: limits.max_content_types,
                });
            }
            let value = required(e, b"ContentType", part)?;
            if map.get(&key).is_some_and(|v| *v != value) {
                return Err(OoxmlError::MalformedXml {
                    part: part.to_owned(),
                    reason: "a part or extension declared with two types",
                });
            }
            map.insert(key, value);
            Ok(())
        })?;
        Ok(out)
    }

    /// The content type of `part`, by OPC's rules. Any spelling equivalent under
    /// [`part_name_key`](crate::part_name_key) finds it.
    pub fn resolve(&self, part: &str) -> Option<&str> {
        let key = entry_key(part.trim_start_matches('/'));
        if let Some(ct) = self.overrides.get(&key) {
            return Some(ct);
        }
        let last = key.rsplit('/').next()?;
        let (_, ext) = last.rsplit_once('.')?;
        self.defaults.get(ext).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `XML-04`: a foreign attribute neither shadows the real one nor stands in for it.
    #[test]
    fn a_foreign_attribute_is_not_the_plain_one() {
        let parse = |body: &str| {
            let xml = format!("<Types xmlns:x=\"urn:x\">{body}</Types>");
            ContentTypes::parse(xml.as_bytes(), &OoxmlLimits::default())
        };
        let types = parse(
            "<Override x:PartName=\"/evil.xml\" PartName=\"/good.xml\" ContentType=\"a/b\" x:ContentType=\"evil/x\"/>\
             <Default x:Extension=\"evil\" Extension=\"png\" ContentType=\"image/png\"/>",
        )
        .unwrap();
        assert_eq!(types.resolve("good.xml"), Some("a/b"));
        assert_eq!(types.resolve("evil.xml"), None);
        assert_eq!(types.resolve("a.png"), Some("image/png"));
        assert_eq!(types.resolve("a.evil"), None);
        for body in [
            "<Override x:PartName=\"/a.xml\" ContentType=\"a/b\"/>",
            "<Override PartName=\"/a.xml\" x:ContentType=\"a/b\"/>",
            "<Default x:Extension=\"png\" ContentType=\"a/b\"/>",
        ] {
            assert_eq!(parse(body).unwrap_err().code(), "CS-XML-0004", "{body}");
        }
    }

    #[test]
    fn an_override_matches_any_equivalent_spelling_of_its_part() {
        let types = ContentTypes::parse(
            b"<Types><Override PartName=\"/ppt/my%20slide.xml\" ContentType=\"a/b\"/>\
              <Override PartName=\"/ppt/\xC3\xA9.xml\" ContentType=\"c/d\"/>\
              <Default Extension=\"PNG\" ContentType=\"image/png\"/></Types>",
            &OoxmlLimits::default(),
        )
        .unwrap();
        assert_eq!(types.resolve("ppt/my slide.xml"), Some("a/b"));
        assert_eq!(types.resolve("/PPT/MY%20SLIDE.XML"), Some("a/b"));
        assert_eq!(types.resolve("ppt/%C3%A9.xml"), Some("c/d"));
        assert_eq!(types.resolve("ppt/%c3%89.xml"), None);
        assert_eq!(types.resolve("ppt/my%20image.png"), Some("image/png"));
    }
}
