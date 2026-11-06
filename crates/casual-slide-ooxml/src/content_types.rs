//! `[Content_Types].xml`.

use std::collections::BTreeMap;

use crate::error::OoxmlError;
use crate::limits::OoxmlLimits;
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
                    Err(wrong_root(part, "Types"))
                };
            }
            let (map, key) = match name {
                b"Override" => (
                    &mut out.overrides,
                    required(e, b"PartName", part)?
                        .trim_start_matches('/')
                        .to_lowercase(),
                ),
                b"Default" => (
                    &mut out.defaults,
                    required(e, b"Extension", part)?.to_lowercase(),
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
                    reason: format!("{key:?} declared with two types"),
                });
            }
            map.insert(key, value);
            Ok(())
        })?;
        Ok(out)
    }

    /// The content type of `part`, by OPC's rules. Case-insensitive.
    pub fn resolve(&self, part: &str) -> Option<&str> {
        let key = part.trim_start_matches('/').to_lowercase();
        if let Some(ct) = self.overrides.get(&key) {
            return Some(ct);
        }
        let last = key.rsplit('/').next()?;
        let (_, ext) = last.rsplit_once('.')?;
        self.defaults.get(ext).map(String::as_str)
    }
}
