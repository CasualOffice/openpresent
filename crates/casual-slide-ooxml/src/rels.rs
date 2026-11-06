//! Relationships, target resolution, and what an external target really is.

use std::collections::BTreeSet;

use crate::error::OoxmlError;
use crate::limits::OoxmlLimits;
use crate::xml::{attr, required, walk, wrong_root};

/// Whether a target is inside the package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetMode {
    /// A part in this package.
    Internal,
    /// A URI or path outside it. Never followed by the engine.
    External,
}

/// One `<Relationship>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    /// `Id`, unique within the part, referenced as `r:id`.
    pub id: String,
    /// `Type`, a URI.
    pub rel_type: String,
    /// `Target`, exactly as written.
    pub target: String,
    /// `TargetMode`.
    pub mode: TargetMode,
}

/// Parse a `.rels` part. Missing required attributes, a repeated id and an
/// unknown target mode are refused.
pub(crate) fn parse_relationships(
    bytes: &[u8],
    part: &str,
    limits: &OoxmlLimits,
) -> Result<Vec<Relationship>, OoxmlError> {
    let mut rels = Vec::new();
    let mut ids = BTreeSet::new();
    walk(bytes, part, limits, |e, depth| {
        let name = e.local_name();
        if depth == 1 {
            return if name.as_ref() == b"Relationships" {
                Ok(())
            } else {
                Err(wrong_root(part, "Relationships"))
            };
        }
        if name.as_ref() != b"Relationship" {
            return Ok(());
        }
        if rels.len() >= limits.max_part_relationships {
            return Err(OoxmlError::TooManyRelationships {
                limit: limits.max_part_relationships,
            });
        }
        let id = required(e, b"Id", part)?;
        if !ids.insert(id.clone()) {
            return Err(OoxmlError::MalformedXml {
                part: part.to_owned(),
                reason: format!("relationship id {id:?} repeated"),
            });
        }
        let mode = match attr(e, b"TargetMode", part)?.as_deref() {
            None | Some("Internal") => TargetMode::Internal,
            Some("External") => TargetMode::External,
            Some(other) => {
                return Err(OoxmlError::MalformedXml {
                    part: part.to_owned(),
                    reason: format!("unknown target mode {other:?}"),
                });
            }
        };
        rels.push(Relationship {
            id,
            rel_type: required(e, b"Type", part)?,
            target: required(e, b"Target", part)?,
            mode,
        });
        Ok(())
    })?;
    Ok(rels)
}

/// The part a target resolves to from `source`. `None` when it climbs above the
/// package root, which a tolerant normalizer would silently turn into a
/// different part.
pub(crate) fn resolve_internal(source: &str, target: &str) -> Option<String> {
    let target = target.split('#').next().unwrap_or_default();
    let joined;
    let path = if let Some(absolute) = target.strip_prefix('/') {
        absolute
    } else {
        let dir = source.rfind('/').map_or("", |i| &source[..i]);
        joined = if dir.is_empty() {
            target.to_owned()
        } else {
            format!("{dir}/{target}")
        };
        &joined
    };
    let mut out: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                out.pop()?;
            }
            other => out.push(other),
        }
    }
    (!out.is_empty()).then(|| out.join("/"))
}

/// The `.rels` part carrying relationships for `part`. The empty string is the
/// package itself.
pub(crate) fn rels_part_for(part: &str) -> String {
    match part.rfind('/') {
        Some(i) => format!("{}/_rels/{}.rels", &part[..i], &part[i + 1..]),
        None => format!("_rels/{part}.rels"),
    }
}

/// The source part a `.rels` part describes, or `None` if `name` is not one.
pub(crate) fn source_of_rels(name: &str) -> Option<String> {
    let stem = name.strip_suffix(".rels")?;
    let (dir, file) = stem.rsplit_once("_rels/")?;
    if !dir.is_empty() && !dir.ends_with('/') {
        return None;
    }
    Some(format!("{dir}{file}"))
}

/// What an external target is, once the disguises are removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExternalKind {
    /// `http` or `https`.
    Web,
    /// `mailto` or `tel`.
    Contact,
    /// A PowerPoint action such as `ppaction://hlinkshowjump`. Retained, and a
    /// program or macro action is never run.
    PowerPointAction,
    /// A UNC path or `file:` URI. **Touching one can send the user's NTLM
    /// credentials to the host named in the file.**
    LocalOrUnc,
    /// `javascript:`, `vbscript:`, `data:`: code or content in the URI.
    Script,
    /// A relative path with no scheme, resolved by desktop PowerPoint against
    /// the deck's folder.
    RelativePath,
    /// Any other scheme.
    Other,
}

impl ExternalKind {
    /// Whether a user may be offered this as a link. Nothing here is ever
    /// fetched automatically, whatever the answer.
    pub fn is_openable_link(self) -> bool {
        matches!(self, ExternalKind::Web | ExternalKind::Contact)
    }
}

/// Classify an external target. Browsers ignore whitespace and control
/// characters inside a scheme, so `java\tscript:` is `javascript:`, and this has
/// to see what the browser would.
pub fn classify_external(target: &str) -> ExternalKind {
    let squeezed: String = target
        .chars()
        .filter(|c| !c.is_ascii_whitespace() && !c.is_control())
        .collect::<String>()
        .to_ascii_lowercase();
    if squeezed.starts_with("\\\\") || squeezed.starts_with("//") {
        return ExternalKind::LocalOrUnc;
    }
    let Some((scheme, _)) = squeezed.split_once(':') else {
        return ExternalKind::RelativePath;
    };
    let is_scheme = scheme.chars().enumerate().all(|(i, c)| {
        c.is_ascii_alphabetic() || (i > 0 && (c.is_ascii_digit() || matches!(c, '+' | '-' | '.')))
    });
    if scheme.is_empty() || !is_scheme {
        return ExternalKind::RelativePath;
    }
    match scheme {
        "http" | "https" => ExternalKind::Web,
        "mailto" | "tel" => ExternalKind::Contact,
        "ppaction" => ExternalKind::PowerPointAction,
        "file" => ExternalKind::LocalOrUnc,
        "javascript" | "vbscript" | "data" | "jscript" | "livescript" => ExternalKind::Script,
        // A drive letter, as in `C:\x`.
        s if s.len() == 1 => ExternalKind::LocalOrUnc,
        _ => ExternalKind::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative_absolute_and_fragment_targets() {
        let s = "ppt/slides/slide1.xml";
        assert_eq!(
            resolve_internal(s, "../media/a.png").unwrap(),
            "ppt/media/a.png"
        );
        assert_eq!(
            resolve_internal(s, "/ppt/theme/theme1.xml").unwrap(),
            "ppt/theme/theme1.xml"
        );
        assert_eq!(
            resolve_internal(s, "slide2.xml#x").unwrap(),
            "ppt/slides/slide2.xml"
        );
        assert_eq!(
            resolve_internal("", "ppt/presentation.xml").unwrap(),
            "ppt/presentation.xml"
        );
    }

    #[test]
    fn a_target_above_the_package_root_does_not_resolve() {
        assert_eq!(
            resolve_internal("ppt/slide.xml", "../../../etc/passwd"),
            None
        );
        assert_eq!(resolve_internal("a.xml", "../b.xml"), None);
    }

    #[test]
    fn rels_paths_map_both_ways() {
        assert_eq!(rels_part_for(""), "_rels/.rels");
        assert_eq!(
            rels_part_for("ppt/presentation.xml"),
            "ppt/_rels/presentation.xml.rels"
        );
        assert_eq!(
            source_of_rels("ppt/_rels/presentation.xml.rels").unwrap(),
            "ppt/presentation.xml"
        );
        assert_eq!(source_of_rels("_rels/.rels").unwrap(), "");
        assert_eq!(source_of_rels("ppt/slide.xml"), None);
        assert_eq!(source_of_rels("ppt/not_rels/a.xml.rels"), None);
    }

    #[test]
    fn classification_sees_through_disguises() {
        use ExternalKind::*;
        for (t, want) in [
            ("https://example.com/a", Web),
            ("HTTP://example.com", Web),
            ("mailto:a@example.com", Contact),
            ("ppaction://hlinkshowjump?jump=nextslide", PowerPointAction),
            ("\\\\attacker\\share\\a.png", LocalOrUnc),
            ("//attacker/share", LocalOrUnc),
            ("file:///C:/secret.txt", LocalOrUnc),
            ("C:\\secret.txt", LocalOrUnc),
            ("javascript:alert(1)", Script),
            ("  JaVa\tScRiPt:alert(1)", Script),
            ("java\u{1}script:alert(1)", Script),
            ("data:text/html,x", Script),
            ("vbscript:x", Script),
            ("movie.mp4", RelativePath),
            ("media/a:b.png", RelativePath),
            ("ftp://example.com", Other),
        ] {
            assert_eq!(classify_external(t), want, "{t:?}");
        }
    }

    #[test]
    fn only_web_and_contact_are_openable() {
        assert!(ExternalKind::Web.is_openable_link());
        assert!(ExternalKind::Contact.is_openable_link());
        for k in [
            ExternalKind::Script,
            ExternalKind::LocalOrUnc,
            ExternalKind::Other,
            ExternalKind::RelativePath,
            ExternalKind::PowerPointAction,
        ] {
            assert!(!k.is_openable_link());
        }
    }
}
