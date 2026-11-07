//! Deep clone at file level (`INV-SP-10`, Inferred): a part and what belongs to it.
//!
//! A clone is a byte copy, so everything the model does not hold comes along, and its
//! `.rels` is a copy with **the same ids**. So nothing inside a cloned part needs
//! rewriting, including `dgm:dataModelExt/@relId`, which names a relationship of the
//! slide (`INV-RI-02`); only targets that moved change. Preflight checks that the clone's
//! `relId` lands on the clone's drawing.

use std::collections::{BTreeMap, VecDeque};

use casual_slide_ooxml::XmlTree;

use super::draft::{Draft, rel_kind, set_attr};
use super::error::WriterError;
use super::names::{key, rels_part_for, target_for};

/// What a clone shares rather than copies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClonePolicy {
    /// A slide: shares its layout, media, and other slides it links to. Everything else
    /// is copied, an unknown target included, and reported.
    Slide,
    /// A layout: shares its master and media.
    Layout,
    /// A master: copies its layouts and its theme (`INV-SP-07`), and shares media.
    Master,
}

fn is_media(kind: &str) -> bool {
    matches!(kind, "image" | "audio" | "video" | "movie")
        || kind.ends_with("/media")
        || kind.ends_with("/hdphoto")
}

impl ClonePolicy {
    fn shares(self, kind: &str) -> bool {
        if is_media(kind) || matches!(kind, "hyperlink" | "font" | "presProps" | "tableStyles") {
            return true;
        }
        match self {
            Self::Slide => matches!(
                kind,
                "slideLayout" | "slideMaster" | "notesMaster" | "handoutMaster" | "theme" | "slide"
            ),
            Self::Layout => matches!(kind, "slideMaster" | "theme" | "slide"),
            Self::Master => matches!(kind, "slide" | "notesMaster"),
        }
    }
}

/// Kinds a slide's clone is known to copy. Any other copied target is reported.
fn is_known_clone(kind: &str) -> bool {
    matches!(
        kind,
        "notesSlide"
            | "comments"
            | "chart"
            | "chartUserShapes"
            | "themeOverride"
            | "package"
            | "oleObject"
            | "diagramData"
            | "diagramLayout"
            | "diagramQuickStyle"
            | "diagramColors"
            | "tags"
            | "vmlDrawing"
            | "slideLayout"
            | "theme"
            | "customXml"
    ) || kind.ends_with("/diagramDrawing")
        || kind.ends_with("/chartStyle")
        || kind.ends_with("/chartColorStyle")
        || kind.ends_with("/ink")
        || kind.ends_with("/comments")
}

impl Draft<'_> {
    /// Copy `start` and everything `policy` says belongs to it. Returns the clone of each
    /// part copied, by source. New parts have their relationships and content types, and
    /// no relationship from outside the clone points to them yet.
    pub(crate) fn deep_clone(
        &mut self,
        start: &str,
        policy: ClonePolicy,
    ) -> Result<BTreeMap<String, String>, WriterError> {
        let mut clones: BTreeMap<String, String> = BTreeMap::new();
        let mut order = Vec::new();
        let mut queue = VecDeque::from([start.to_owned()]);
        let mut total: u64 = 0;
        while let Some(part) = queue.pop_front() {
            if clones.contains_key(&part) || key(&part) == key(&self.main) {
                continue;
            }
            if clones.len() >= self.limits.max_clone_parts {
                return Err(WriterError::TooLarge {
                    what: "parts in one clone",
                    limit: self.limits.max_clone_parts as u64,
                });
            }
            let (dir, file) = part.rsplit_once('/').unwrap_or(("ppt", part.as_str()));
            let (stem, ext) = file.rsplit_once('.').unwrap_or((file, ""));
            let name = self.families.mint(dir, stem, ext)?;
            clones.insert(part.clone(), name);
            order.push(part.clone());
            for rel in self.relationships(&part)? {
                let kind = rel_kind(&rel.rel_type);
                if !policy.shares(kind)
                    && let Some(target) = self.resolve(&part, &rel)
                {
                    if !is_known_clone(kind) {
                        self.note(
                            "CS-SAV-0022",
                            "INV-SP-10",
                            &target,
                            "a target of an unknown relationship type was copied with its slide",
                        );
                    }
                    queue.push_back(target);
                }
            }
        }
        for part in order {
            let clone = clones.get(&part).cloned().unwrap_or_default();
            let bytes = self.part_bytes(&part)?;
            total = total.saturating_add(bytes.len() as u64);
            if total > self.limits.max_clone_bytes {
                return Err(WriterError::TooLarge {
                    what: "bytes in one clone",
                    limit: self.limits.max_clone_bytes,
                });
            }
            let content_type = self.content_type_of(&part)?;
            self.add_part(&clone, content_type.as_deref(), bytes)?;
            self.log.clone_of.insert(clone.clone(), part.clone());
            self.clone_rels(&part, &clone, &clones)?;
        }
        Ok(clones)
    }

    /// The clone's `.rels`: the source's, with targets that were cloned moved. A target
    /// that did not move stays as written, relative to the same folder.
    fn clone_rels(
        &mut self,
        part: &str,
        clone: &str,
        clones: &BTreeMap<String, String>,
    ) -> Result<(), WriterError> {
        let Some(actual) = self.actual(&rels_part_for(part)) else {
            return Ok(());
        };
        let bytes = self.xml_bytes(&actual)?;
        let rels_name = rels_part_for(clone);
        let mut tree = XmlTree::parse(&bytes, &rels_name, &self.xml)?;
        for rel in self.relationships(part)? {
            let Some(target) = self.resolve(part, &rel) else {
                continue;
            };
            let moved = clones.get(&target);
            let same_folder =
                part.rsplit_once('/').map(|(d, _)| d) == clone.rsplit_once('/').map(|(d, _)| d);
            if moved.is_none() && same_folder {
                continue;
            }
            let goal = moved.cloned().unwrap_or(target);
            let found = tree
                .child_elements(tree.root())
                .into_iter()
                .find(|&el| tree.plain_attribute(el, "Id").as_deref() == Some(rel.id.as_str()));
            if let Some(el) = found {
                let written = target_for(clone, &goal, rel.target.starts_with('/'));
                set_attr(&mut tree, el, "Target", &written, &rels_name)?;
            }
        }
        self.set_part_bytes(&rels_name, tree.to_bytes());
        Ok(())
    }
}
