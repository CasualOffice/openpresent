//! The state of a plan being applied: every change held beside the input, which stays
//! untouched until [`Draft::emit`]. Dropping a draft is how a refused plan leaves
//! nothing behind.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use casual_slide_ooxml::{
    Conformance, ElementId, FindingKind, OoxmlLimits, PresentationPackage, Relationship,
    TargetMode, XmlTree, part_name_key, relationship_type_to_strict, resolve_internal,
};
use casual_slide_package::PartEdits;

use super::error::WriterError;
use super::names::{Families, is_rels_part, key, next_rel_id, rels_part_for, target_for};
use super::plan::WriterLimits;
use super::preflight::{self, Preflight};
use super::view::{Parts, View};
use super::zipcheck;
use crate::properties::ExportError;

pub(crate) const CONTENT_TYPES_PART: &str = "[Content_Types].xml";
pub(crate) const RELS_NAMESPACE: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships";
pub(crate) const DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";
const TRANSITIONAL_RELS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
const STRICT_RELS: &str = "http://purl.oclc.org/ooxml/officeDocument/relationships/";

/// The kind of a relationship: its type without the standard's prefix in either form,
/// so `slide` in Transitional and Strict alike. Any other type is itself.
pub(crate) fn rel_kind(rel_type: &str) -> &str {
    rel_type
        .strip_prefix(TRANSITIONAL_RELS)
        .or_else(|| rel_type.strip_prefix(STRICT_RELS))
        .unwrap_or(rel_type)
}

/// What became of a part, in OpenCalc's vocabulary (`docs/36`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Disposition {
    /// Copied with its compressed bytes.
    UnchangedCopy,
    /// New in this save.
    Regenerate,
    /// Existing, and patched.
    Merge,
    /// Removed by this save.
    Omit,
}

/// One line of what a save did, or of what it found and left alone.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReportEntry {
    /// The stable code.
    pub code: &'static str,
    /// The invariant, where one is involved.
    pub rule: &'static str,
    /// The part concerned.
    pub part: String,
    /// A fixed phrase, with ids and counts only.
    pub detail: String,
}

/// What a plan's save did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriterOutcome {
    /// The package.
    pub bytes: Vec<u8>,
    /// Parts whose bytes differ, or are new or gone, in order.
    pub changed_parts: Vec<String>,
    /// What became of every part.
    pub dispositions: BTreeMap<String, Disposition>,
    /// What the save did beyond the plain edits, and what it found.
    pub report: Vec<ReportEntry>,
    /// The deck carried a digital signature, and an edit has invalidated it.
    pub signature_invalidated: bool,
    /// Each cloned part, by the part it was copied from: clone to source.
    pub clones: BTreeMap<String, String>,
    /// The slide parts each `Add` or `Duplicate` created, in plan order.
    pub new_slides: Vec<String>,
}

/// One id a copied shape was given.
#[derive(Debug, Clone)]
pub(crate) struct CopiedShape {
    pub part: String,
    pub old: String,
    pub new: String,
    pub animated: bool,
}

/// What has been done to the package, which preflight judges against the input.
#[derive(Debug, Default, Clone)]
pub(crate) struct Changes {
    /// Parts new in this save, by actual name.
    pub added: BTreeSet<String>,
    /// Input parts gone, by actual name. A name re-added stays here too.
    pub removed: BTreeSet<String>,
    /// Each clone's source.
    pub clone_of: BTreeMap<String, String>,
    /// The content type each new part was declared with.
    pub expected_types: BTreeMap<String, String>,
    /// Relationships an operation removed, as (source, id).
    pub removed_rels: BTreeSet<(String, String)>,
    /// Relationships an operation pointed somewhere else, as (source, id).
    pub retargeted: BTreeSet<(String, String)>,
    /// An operation asked for sections to exist.
    pub section_list_requested: bool,
    /// Sections an operation removed, by id.
    pub removed_sections: BTreeSet<String>,
    /// Slide ids moved between sections by an operation that names the section.
    pub placed_slides: BTreeSet<String>,
    /// Slide ids given up in this save, not reused.
    pub retired_slide_ids: BTreeSet<u64>,
    /// Master or layout ids given up in this save, not reused.
    pub retired_layout_ids: BTreeSet<u64>,
    /// The notes master this save created.
    pub created_notes_master: Option<String>,
    /// The file type asked for, by extension.
    pub requested_extension: Option<String>,
    /// A VBA project was dropped on confirmation.
    pub vba_dropped: bool,
    /// Shapes copied, for the timing they should carry.
    pub copied_shapes: Vec<CopiedShape>,
    /// A model patch rewrote the main part.
    pub model_edited_main: bool,
    /// New parts an operation authored, whose bytes may equal a removed part's.
    pub authored: BTreeSet<String>,
    /// Slides new in this save, by id.
    pub new_slide_ids: BTreeSet<String>,
}

/// A plan being applied. Reached through [`super::SavePlan::stage`].
///
/// The `*_part` methods below are the hook `docs/36` asks for: a test injects a seeded
/// break into the draft, and [`Draft::preflight`] must refuse it.
#[derive(Debug)]
pub struct Draft<'p> {
    pub(crate) package: &'p mut PresentationPackage,
    pub(crate) limits: WriterLimits,
    pub(crate) xml: OoxmlLimits,
    pub(crate) conformance: Conformance,
    pub(crate) main: String,
    /// By equivalence key, the actual names now in the package.
    pub(crate) names: BTreeMap<String, String>,
    input_names: BTreeSet<String>,
    overlay: BTreeMap<String, Vec<u8>>,
    trees: BTreeMap<String, XmlTree>,
    pub(crate) families: Families,
    pub(crate) log: Changes,
    pub(crate) report: Vec<ReportEntry>,
    pub(crate) new_slides: Vec<String>,
    /// The parts reachable from the package root in the input, by key.
    input_reach: BTreeSet<String>,
}

fn malformed(part: &str) -> WriterError {
    WriterError::Export(ExportError::MalformedElement {
        part: part.to_owned(),
    })
}

/// Set an attribute, refusing an element whose text is not well formed.
pub(crate) fn set_attr(
    tree: &mut XmlTree,
    el: ElementId,
    name: &str,
    value: &str,
    part: &str,
) -> Result<(), WriterError> {
    if tree.set_attribute(el, name, value) {
        Ok(())
    } else {
        Err(malformed(part))
    }
}

/// `prefix:local`, with the prefix `el` is written under.
pub(crate) fn sibling_name(tree: &XmlTree, el: ElementId, local: &str) -> String {
    match tree.qualified_name(el).split_once(':') {
        Some((prefix, _)) => format!("{prefix}:{local}"),
        None => local.to_owned(),
    }
}

/// What the rels of `tree` say, in order.
pub(crate) fn read_relationships(tree: &XmlTree) -> Vec<Relationship> {
    tree.child_elements(tree.root())
        .into_iter()
        .filter(|&el| tree.local_name(el) == "Relationship")
        .filter_map(|el| {
            let mode = match tree.plain_attribute(el, "TargetMode").as_deref() {
                Some("External") => TargetMode::External,
                _ => TargetMode::Internal,
            };
            Some(Relationship {
                id: tree.plain_attribute(el, "Id")?,
                rel_type: tree.plain_attribute(el, "Type")?,
                target: tree.plain_attribute(el, "Target")?,
                mode,
            })
        })
        .collect()
}

/// Extensions binary media may claim with a `Default` (`INV-CT-04`, Inferred). Anything
/// else keeps its name and gets an `Override`.
fn is_usual_binary_extension(ext: &str) -> bool {
    const USUAL: &[&str] = &[
        "png", "jpg", "jpeg", "gif", "bmp", "tif", "tiff", "emf", "wmf", "svg", "wdp", "mp4",
        "m4v", "mp3", "m4a", "wav", "wma", "wmv", "avi", "mov", "mpg", "mpeg", "bin", "xlsx",
        "docx", "pptx", "vml",
    ];
    USUAL.contains(&ext.to_ascii_lowercase().as_str())
}

impl<'p> Draft<'p> {
    pub(crate) fn new(package: &'p mut PresentationPackage, limits: WriterLimits) -> Self {
        let xml = *package.limits();
        let conformance = package.conformance();
        let main = package.main_part().to_owned();
        let mut names = BTreeMap::new();
        let mut families = Families::default();
        let mut input_names = BTreeSet::new();
        for name in package.part_names() {
            if name.ends_with('/') {
                continue;
            }
            families.note(&name);
            input_names.insert(name.clone());
            names.insert(key(&name), name);
        }
        let mut draft = Self {
            package,
            limits,
            xml,
            conformance,
            main,
            names,
            input_names,
            overlay: BTreeMap::new(),
            trees: BTreeMap::new(),
            families,
            log: Changes::default(),
            report: Vec::new(),
            new_slides: Vec::new(),
            input_reach: BTreeSet::new(),
        };
        draft.input_reach = draft.reach_from(&[String::new()]);
        draft
    }

    /// The actual name of a part, in any equivalent spelling.
    pub(crate) fn actual(&self, name: &str) -> Option<String> {
        self.names.get(&key(name)).cloned()
    }

    /// Whether the package holds this part now.
    pub fn has_part(&self, name: &str) -> bool {
        self.names.contains_key(&key(name))
    }

    /// The bytes of a part as the draft has it.
    pub fn part_bytes(&mut self, name: &str) -> Result<Vec<u8>, WriterError> {
        let actual = self.actual(name).ok_or_else(|| WriterError::BadOperation {
            what: "no such part",
            subject: name.to_owned(),
        })?;
        if let Some(tree) = self.trees.get(&actual) {
            return Ok(tree.to_bytes());
        }
        if let Some(bytes) = self.overlay.get(&actual) {
            return Ok(bytes.clone());
        }
        Ok(self.package.read_part(&actual)?)
    }

    /// The bytes of a part, for XML: an input part declaring more than the XML cap is
    /// refused before any of it is decompressed.
    pub(crate) fn xml_bytes(&mut self, name: &str) -> Result<Vec<u8>, WriterError> {
        let actual = self.actual(name).ok_or_else(|| WriterError::BadOperation {
            what: "no such part",
            subject: name.to_owned(),
        })?;
        if let Some(tree) = self.trees.get(&actual) {
            return Ok(tree.to_bytes());
        }
        if let Some(bytes) = self.overlay.get(&actual) {
            return Ok(bytes.clone());
        }
        Ok(self.package.read_xml_part(&actual)?)
    }

    /// Set a part's bytes, adding the entry if it is not there. No relationship or
    /// content type follows, which is what makes this a way to seed a break.
    pub fn set_part_bytes(&mut self, name: &str, bytes: Vec<u8>) {
        let actual = match self.actual(name) {
            Some(actual) => actual,
            None => {
                self.names.insert(key(name), name.to_owned());
                self.families.note(name);
                self.log.added.insert(name.to_owned());
                name.to_owned()
            }
        };
        self.trees.remove(&actual);
        self.overlay.insert(actual, bytes);
    }

    /// Take an entry out, and nothing else: no `.rels`, no `Override`, no incoming
    /// relationship. The cascade is what [`Draft::remove_part`] adds.
    pub fn drop_entry(&mut self, name: &str) {
        if let Some(actual) = self.names.remove(&key(name)) {
            self.overlay.remove(&actual);
            self.trees.remove(&actual);
            self.log.added.remove(&actual);
            if self.input_names.contains(&actual) {
                self.log.removed.insert(actual);
            }
        }
    }

    /// Edit a part's tree in place.
    pub fn edit_part<T>(
        &mut self,
        name: &str,
        edit: impl FnOnce(&mut XmlTree) -> T,
    ) -> Result<T, WriterError> {
        Ok(edit(self.tree(name)?))
    }

    /// The tree of a part, opened on first use. It writes back the bytes it read, so
    /// a part opened and left alone is unchanged.
    pub(crate) fn tree(&mut self, name: &str) -> Result<&mut XmlTree, WriterError> {
        let actual = self.actual(name).ok_or_else(|| WriterError::BadOperation {
            what: "no such part",
            subject: name.to_owned(),
        })?;
        if !self.trees.contains_key(&actual) {
            let bytes = self.xml_bytes(&actual)?;
            let tree = XmlTree::parse(&bytes, &actual, &self.xml)?;
            self.overlay.remove(&actual);
            self.trees.insert(actual.clone(), tree);
        }
        self.trees
            .get_mut(&actual)
            .ok_or_else(|| malformed(&actual))
    }

    /// Add a part with bytes and a declared content type, in one step.
    pub(crate) fn add_part(
        &mut self,
        name: &str,
        content_type: Option<&str>,
        bytes: Vec<u8>,
    ) -> Result<(), WriterError> {
        self.set_part_bytes(name, bytes);
        self.log.added.insert(name.to_owned());
        self.log.authored.insert(name.to_owned());
        if let Some(ct) = content_type {
            self.declare(name, ct)?;
        }
        Ok(())
    }

    /// Remove a part with its `.rels` and its `Override` (`INV-PN-07`, `INV-CT-05`).
    /// Relationships to it are the caller's, or the sweep's.
    pub fn remove_part(&mut self, name: &str) -> Result<(), WriterError> {
        let Some(actual) = self.actual(name) else {
            return Ok(());
        };
        if let Some(rels) = self.actual(&rels_part_for(&actual)) {
            self.drop_entry(&rels);
        }
        self.undeclare(&actual)?;
        self.drop_entry(&actual);
        Ok(())
    }

    /// The relationship type in the form this file is written in (`INV-RI-09`).
    pub(crate) fn rel_type(&self, kind: &str) -> String {
        let transitional = format!("{TRANSITIONAL_RELS}{kind}");
        match self.conformance {
            Conformance::Transitional => transitional,
            Conformance::Strict => relationship_type_to_strict(&transitional).into_owned(),
        }
    }

    /// The relationships `source` declares now.
    pub(crate) fn relationships(&mut self, source: &str) -> Result<Vec<Relationship>, WriterError> {
        let rels_name = rels_part_for(source);
        let Some(actual) = self.actual(&rels_name) else {
            return Ok(Vec::new());
        };
        if let Some(tree) = self.trees.get(&actual) {
            return Ok(read_relationships(tree));
        }
        if !self.overlay.contains_key(&actual) && self.input_names.contains(&actual) {
            let known = self.package.relationships_of(source);
            if !known.is_empty() {
                return Ok(known.to_vec());
            }
        }
        let bytes = self.xml_bytes(&actual)?;
        let tree = XmlTree::parse(&bytes, &actual, &self.xml)?;
        Ok(read_relationships(&tree))
    }

    /// The part a relationship of `source` lands on, if it is in the package now.
    pub(crate) fn resolve(&self, source: &str, rel: &Relationship) -> Option<String> {
        if rel.mode != TargetMode::Internal || rel.target.starts_with('#') {
            return None;
        }
        let target = resolve_internal(source, &rel.target)?;
        self.names.get(&part_name_key(&target)?).cloned()
    }

    fn rels_tree(&mut self, source: &str) -> Result<(String, &mut XmlTree), WriterError> {
        let rels_name = rels_part_for(source);
        if self.actual(&rels_name).is_none() {
            let body = format!("{DECL}<Relationships xmlns=\"{RELS_NAMESPACE}\"/>");
            self.set_part_bytes(&rels_name, body.into_bytes());
        }
        let actual = self.actual(&rels_name).unwrap_or(rels_name);
        let tree = self.tree(&actual)?;
        Ok((actual, tree))
    }

    /// Add a relationship to a part, in the form its siblings are written.
    pub(crate) fn add_relationship(
        &mut self,
        source: &str,
        kind: &str,
        target: &str,
    ) -> Result<String, WriterError> {
        let rel_type = self.rel_type(kind);
        self.add_relationship_typed(source, &rel_type, target)
    }

    /// Add a relationship with a type as spelled.
    pub(crate) fn add_relationship_typed(
        &mut self,
        source: &str,
        rel_type: &str,
        target: &str,
    ) -> Result<String, WriterError> {
        let existing = self.relationships(source)?;
        let retired: Vec<&String> = self
            .log
            .removed_rels
            .iter()
            .filter(|(s, _)| s == source)
            .map(|(_, id)| id)
            .collect();
        let id = next_rel_id(
            existing
                .iter()
                .map(|r| r.id.as_str())
                .chain(retired.iter().map(|s| s.as_str())),
        );
        let internal: Vec<&Relationship> = existing
            .iter()
            .filter(|r| r.mode == TargetMode::Internal && !r.target.starts_with('#'))
            .collect();
        let absolute = !internal.is_empty() && internal.iter().all(|r| r.target.starts_with('/'));
        let written = target_for(source, target, absolute);
        let (rels_part, tree) = self.rels_tree(source)?;
        let root = tree.root();
        let name = sibling_name(tree, root, "Relationship");
        let new = tree.new_element(&name);
        set_attr(tree, new, "Id", &id, &rels_part)?;
        set_attr(tree, new, "Type", rel_type, &rels_part)?;
        set_attr(tree, new, "Target", &written, &rels_part)?;
        tree.insert_child(root, None, new);
        Ok(id)
    }

    /// The existing relationship of this kind to this part, or a new one (`INV-RI-10`).
    pub(crate) fn reuse_relationship(
        &mut self,
        source: &str,
        kind: &str,
        target: &str,
    ) -> Result<String, WriterError> {
        let target_key = key(target);
        for rel in self.relationships(source)? {
            if rel_kind(&rel.rel_type) == kind
                && self
                    .resolve(source, &rel)
                    .is_some_and(|t| key(&t) == target_key)
            {
                return Ok(rel.id);
            }
        }
        self.add_relationship(source, kind, target)
    }

    /// Remove a relationship by id.
    pub(crate) fn remove_relationship(
        &mut self,
        source: &str,
        id: &str,
    ) -> Result<(), WriterError> {
        let (rels_part, tree) = self.rels_tree(source)?;
        let root = tree.root();
        let found = tree
            .child_elements(root)
            .into_iter()
            .find(|&el| tree.plain_attribute(el, "Id").as_deref() == Some(id));
        let Some(el) = found else {
            return Err(WriterError::BadOperation {
                what: "no such relationship",
                subject: format!("{rels_part}#{id}"),
            });
        };
        tree.remove_child(root, el);
        self.log
            .removed_rels
            .insert((source.to_owned(), id.to_owned()));
        Ok(())
    }

    /// Point a relationship at another part, keeping its id (`INV-RI-02`).
    pub(crate) fn retarget_relationship(
        &mut self,
        source: &str,
        id: &str,
        target: &str,
    ) -> Result<(), WriterError> {
        let (rels_part, tree) = self.rels_tree(source)?;
        let found = tree
            .child_elements(tree.root())
            .into_iter()
            .find(|&el| tree.plain_attribute(el, "Id").as_deref() == Some(id));
        let Some(el) = found else {
            return Err(WriterError::BadOperation {
                what: "no such relationship",
                subject: format!("{rels_part}#{id}"),
            });
        };
        let absolute = tree
            .plain_attribute(el, "Target")
            .is_some_and(|t| t.starts_with('/'));
        let written = target_for(source, target, absolute);
        set_attr(tree, el, "Target", &written, &rels_part)?;
        self.log
            .retargeted
            .insert((source.to_owned(), id.to_owned()));
        Ok(())
    }

    fn content_types(&mut self) -> Result<&mut XmlTree, WriterError> {
        self.tree(CONTENT_TYPES_PART)
    }

    /// The content type a part resolves to now.
    pub(crate) fn content_type_of(&mut self, part: &str) -> Result<Option<String>, WriterError> {
        let wanted = key(part);
        let tree = self.content_types()?;
        let mut default = None;
        let ext = wanted
            .rsplit('/')
            .next()
            .and_then(|f| f.rsplit_once('.'))
            .map(|(_, e)| e.to_owned());
        for el in tree.child_elements(tree.root()) {
            match tree.local_name(el) {
                "Override" => {
                    if tree
                        .plain_attribute(el, "PartName")
                        .is_some_and(|n| key(&n) == wanted)
                    {
                        return Ok(tree.plain_attribute(el, "ContentType"));
                    }
                }
                "Default"
                    if tree
                        .plain_attribute(el, "Extension")
                        .is_some_and(|e| Some(e.to_ascii_lowercase()) == ext) =>
                {
                    default = tree.plain_attribute(el, "ContentType");
                }
                _ => {}
            }
        }
        Ok(default)
    }

    /// Declare a new part's type by the standard's steps (`INV-CT-03`, Documented), and
    /// where it allows either, XML by `Override` and binary by `Default` (`INV-CT-04`,
    /// Inferred). `PartName` is the name as stored (`INV-PE-05`).
    pub(crate) fn declare(&mut self, part: &str, content_type: &str) -> Result<(), WriterError> {
        self.log
            .expected_types
            .insert(part.to_owned(), content_type.to_owned());
        let file = part.rsplit('/').next().unwrap_or(part);
        let ext = file.rsplit_once('.').map(|(_, e)| e.to_owned());
        let main_xml = content_type.to_ascii_lowercase().ends_with("+xml");
        let tree = self.content_types()?;
        let root = tree.root();
        let children = tree.child_elements(root);
        let default = ext.as_deref().and_then(|ext| {
            children.iter().find_map(|&el| {
                (tree.local_name(el) == "Default"
                    && tree
                        .plain_attribute(el, "Extension")
                        .is_some_and(|e| e.eq_ignore_ascii_case(ext)))
                .then(|| tree.plain_attribute(el, "ContentType"))
                .flatten()
            })
        });
        let wants_override = match (&ext, &default) {
            (None, _) => true,
            (Some(_), Some(existing)) => main_xml || !existing.eq_ignore_ascii_case(content_type),
            (Some(ext), None) => main_xml || !is_usual_binary_extension(ext),
        };
        if wants_override {
            let name = sibling_name(tree, root, "Override");
            let new = tree.new_element(&name);
            set_attr(
                tree,
                new,
                "PartName",
                &format!("/{part}"),
                CONTENT_TYPES_PART,
            )?;
            set_attr(tree, new, "ContentType", content_type, CONTENT_TYPES_PART)?;
            tree.insert_child(root, None, new);
        } else if default.is_none() {
            let name = sibling_name(tree, root, "Default");
            let new = tree.new_element(&name);
            let ext = ext.unwrap_or_default();
            set_attr(tree, new, "Extension", &ext, CONTENT_TYPES_PART)?;
            set_attr(tree, new, "ContentType", content_type, CONTENT_TYPES_PART)?;
            // Defaults first, as every producer read writes them. The schema allows any order.
            let before = children
                .iter()
                .copied()
                .find(|&el| tree.local_name(el) == "Override");
            tree.insert_child(root, before, new);
        }
        Ok(())
    }

    /// Remove a part's `Override`, if it has one (`INV-CT-05`).
    pub(crate) fn undeclare(&mut self, part: &str) -> Result<(), WriterError> {
        let wanted = key(part);
        self.log.expected_types.remove(part);
        let tree = self.content_types()?;
        let root = tree.root();
        for el in tree.child_elements(root) {
            if tree.local_name(el) == "Override"
                && tree
                    .plain_attribute(el, "PartName")
                    .is_some_and(|n| key(&n) == wanted)
            {
                tree.remove_child(root, el);
            }
        }
        Ok(())
    }

    /// What is reachable from `from` over internal relationships, by key. Each part once.
    pub(crate) fn reach_from(&mut self, from: &[String]) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut queue: VecDeque<String> = from.iter().cloned().collect();
        while let Some(part) = queue.pop_front() {
            if !seen.insert(key(&part)) {
                continue;
            }
            let rels = self.relationships(&part).unwrap_or_default();
            for rel in rels {
                if let Some(target) = self.resolve(&part, &rel) {
                    queue.push_back(target);
                }
            }
        }
        seen
    }

    /// Remove what this save's edits made unreachable, and only that (`INV-SP-01`,
    /// `INV-SP-02`, Inferred). A part that arrived orphaned stays, with whatever it points
    /// to: a media part goes only with its last incoming relationship, from any source.
    pub(crate) fn sweep(&mut self) -> Result<(), WriterError> {
        let mut roots = vec![String::new()];
        for (k, name) in &self.names {
            if !self.input_reach.contains(k) && !is_rels_part(name) && k != &key(CONTENT_TYPES_PART)
            {
                roots.push(name.clone());
            }
        }
        let kept = self.reach_from(&roots);
        let gone: Vec<String> = self
            .names
            .iter()
            .filter(|(k, _)| self.input_reach.contains(*k) && !kept.contains(*k))
            .map(|(_, name)| name.clone())
            .collect();
        for name in gone {
            self.remove_part(&name)?;
        }
        Ok(())
    }

    fn flush(&mut self) {
        let trees = std::mem::take(&mut self.trees);
        for (name, tree) in trees {
            self.overlay.insert(name, tree.to_bytes());
        }
    }

    /// Entries whose bytes changed, with a part opened and left alone dropped. A part
    /// that went and came back under one name counts as changed.
    fn settle(&mut self) -> Result<(), WriterError> {
        self.flush();
        let names: Vec<String> = self.overlay.keys().cloned().collect();
        for name in names {
            if !self.input_names.contains(&name) || self.log.removed.contains(&name) {
                continue;
            }
            let same = {
                let current = self.overlay.get(&name);
                let original = self.package.read_part(&name)?;
                current.is_some_and(|c| *c == original)
            };
            if same {
                self.overlay.remove(&name);
            }
        }
        Ok(())
    }

    /// Whether anything is dirty: nothing means the input bytes come back (F1).
    pub fn is_dirty(&mut self) -> Result<bool, WriterError> {
        self.settle()?;
        Ok(!self.overlay.is_empty() || !self.log.removed.is_empty())
    }

    pub(crate) fn input_contains(&self, name: &str) -> bool {
        self.input_names.contains(name)
    }

    pub(crate) fn overlay_names(&self) -> impl Iterator<Item = &String> {
        self.overlay.keys()
    }

    pub(crate) fn input_reach(&self) -> &BTreeSet<String> {
        &self.input_reach
    }

    /// Run every implemented rule on the input and on this draft. Refuses a violation
    /// this save introduced, and reports one that arrived.
    pub fn preflight(&mut self) -> Result<Preflight, WriterError> {
        self.settle()?;
        let main = self.main.clone();
        let before = View::build(&mut InputParts { draft: self }, &main)?;
        let after = View::build(&mut DraftParts { draft: self }, &main)?;
        preflight::run(self, &before, &after)
    }

    /// The edits this draft stands for, with no check at all. The way a fixture writes a
    /// seeded deck; anything meant for a user goes through [`Draft::emit`].
    pub fn into_part_edits(mut self) -> Result<PartEdits, WriterError> {
        self.settle()?;
        let mut edits = PartEdits::default();
        for name in &self.log.removed {
            if !self.overlay.contains_key(name) {
                edits.remove(name);
            }
        }
        for (name, bytes) in std::mem::take(&mut self.overlay) {
            edits.replace(name, bytes);
        }
        Ok(edits)
    }

    /// Preflight, write, and check what was written. With nothing dirty, the input bytes.
    pub fn emit(mut self) -> Result<WriterOutcome, WriterError> {
        if !self.is_dirty()? {
            let bytes = self.package.save_unedited();
            let dispositions = self
                .input_names
                .iter()
                .map(|n| (n.clone(), Disposition::UnchangedCopy))
                .collect();
            return Ok(WriterOutcome {
                bytes,
                changed_parts: Vec::new(),
                dispositions,
                report: Vec::new(),
                signature_invalidated: false,
                clones: BTreeMap::new(),
                new_slides: Vec::new(),
            });
        }
        let verdict = self.preflight()?;
        let mut report = std::mem::take(&mut self.report);
        report.extend(verdict.arrived.iter().map(|v| ReportEntry {
            code: "CS-SAV-0021",
            rule: v.rule,
            part: v.part.clone(),
            detail: v.detail.clone(),
        }));
        report.sort();
        report.dedup();

        let mut changed: BTreeSet<String> = self.overlay.keys().cloned().collect();
        changed.extend(self.log.removed.iter().cloned());
        let mut dispositions = BTreeMap::new();
        for name in &self.input_names {
            let disposition = if self.log.removed.contains(name) && !self.overlay.contains_key(name)
            {
                Disposition::Omit
            } else if self.overlay.contains_key(name) {
                Disposition::Merge
            } else {
                Disposition::UnchangedCopy
            };
            dispositions.insert(name.clone(), disposition);
        }
        for name in self.overlay.keys() {
            if !self.input_names.contains(name) {
                dispositions.insert(name.clone(), Disposition::Regenerate);
            }
        }
        let signed = self
            .package
            .findings()
            .iter()
            .any(|f| f.kind == FindingKind::Signature);
        let new_slides = std::mem::take(&mut self.new_slides);
        let source = self.package.save_unedited();

        let mut edits = PartEdits::default();
        for name in &self.log.removed {
            if !self.overlay.contains_key(name) {
                edits.remove(name);
            }
        }
        for (name, bytes) in &self.overlay {
            edits.replace(name.clone(), bytes.clone());
        }
        let bytes = self.package.save_with(&edits)?;
        zipcheck::check_zip(&source, &bytes, &changed)?;
        // What was written must open, and say what was asked. The first reader of a new
        // file is this check, never the user.
        let mut reopened = PresentationPackage::open(bytes.clone(), self.xml)?;
        for (name, wanted) in &self.overlay {
            if reopened.read_part(name)? != *wanted {
                return Err(WriterError::Emission {
                    rule: "F2",
                    part: name.clone(),
                });
            }
        }
        Ok(WriterOutcome {
            bytes,
            changed_parts: changed.into_iter().collect(),
            dispositions,
            report,
            signature_invalidated: signed,
            clones: self.log.clone_of.clone(),
            new_slides,
        })
    }

    /// Add a report entry.
    pub(crate) fn note(
        &mut self,
        code: &'static str,
        rule: &'static str,
        part: &str,
        detail: &str,
    ) {
        self.report.push(ReportEntry {
            code,
            rule,
            part: part.to_owned(),
            detail: detail.to_owned(),
        });
    }

    /// Apply bytes a model patch produced: new names become added parts.
    pub(crate) fn apply_patched(&mut self, name: &str, bytes: Vec<u8>) {
        if key(name) == key(&self.main) {
            self.log.model_edited_main = true;
        }
        self.set_part_bytes(name, bytes);
    }
}

struct InputParts<'d, 'p> {
    draft: &'d mut Draft<'p>,
}

impl Parts for InputParts<'_, '_> {
    fn names(&self) -> Vec<String> {
        self.draft.input_names.iter().cloned().collect()
    }
    fn read_xml(&mut self, name: &str) -> Result<Vec<u8>, WriterError> {
        Ok(self.draft.package.read_xml_part(name)?)
    }
    fn limits(&self) -> OoxmlLimits {
        self.draft.xml
    }
}

struct DraftParts<'d, 'p> {
    draft: &'d mut Draft<'p>,
}

impl Parts for DraftParts<'_, '_> {
    fn names(&self) -> Vec<String> {
        self.draft.names.values().cloned().collect()
    }
    fn read_xml(&mut self, name: &str) -> Result<Vec<u8>, WriterError> {
        self.draft.xml_bytes(name)
    }
    fn limits(&self) -> OoxmlLimits {
        self.draft.xml
    }
}
