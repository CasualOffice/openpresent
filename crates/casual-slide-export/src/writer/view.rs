//! What preflight reads: the structure of a package, the same way for the input and for
//! the draft, so a rule is one function and a violation is compared by what it names.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use casual_slide_ooxml::{
    ElementId, OoxmlError, OoxmlLimits, Relationship, TargetMode, XmlTree, part_name_key,
    resolve_internal, to_transitional,
};

use super::draft::{CONTENT_TYPES_PART, read_relationships, rel_kind};
use super::error::WriterError;
use super::names::{is_rels_part, key, rels_part_for, source_of_rels};

pub(crate) const SECTION_EXT_URI: &str = "{521415D9-36F7-43E2-AB2F-B90AF26B5E84}";
const TRANSITIONAL_PML: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const TRANSITIONAL_R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Where a view reads parts from.
pub(crate) trait Parts {
    /// Every entry, as it is named.
    fn names(&self) -> Vec<String>;
    /// The bytes of a part to be parsed.
    fn read_xml(&mut self, name: &str) -> Result<Vec<u8>, WriterError>;
    /// The bounds parts are parsed under.
    fn limits(&self) -> OoxmlLimits;
}

/// One relationships part.
#[derive(Debug, Clone, Default)]
pub(crate) struct RelsFacts {
    pub part: String,
    pub items: Vec<Relationship>,
    pub has_xml_base: bool,
    pub duplicate_ids: Vec<String>,
}

/// `[Content_Types].xml`.
#[derive(Debug, Clone, Default)]
pub(crate) struct TypesFacts {
    pub defaults: Vec<(String, String)>,
    pub overrides: Vec<(String, String)>,
    /// By key, for what `overrides` holds first.
    pub override_of: BTreeMap<String, String>,
    pub root_attributes: Vec<String>,
}

/// One entry of a list of ids.
#[derive(Debug, Clone)]
pub(crate) struct IdEntry {
    pub raw: String,
    pub id: Option<u64>,
    pub rel_id: String,
}

/// One section.
#[derive(Debug, Clone)]
pub(crate) struct SectionFacts {
    pub id: String,
    pub slides: Vec<String>,
}

/// One custom show.
#[derive(Debug, Clone)]
pub(crate) struct CustomShowFacts {
    pub id: String,
}

/// The main part.
#[derive(Debug, Clone, Default)]
pub(crate) struct PresentationFacts {
    pub slides: Vec<IdEntry>,
    pub masters: Vec<IdEntry>,
    pub notes_masters: Vec<String>,
    pub handout_masters: Vec<String>,
    pub child_order: Vec<String>,
    pub first_slide_num: Option<String>,
    /// `None` when the deck has no section list.
    pub sections: Option<Vec<SectionFacts>>,
    pub custom_shows: Vec<CustomShowFacts>,
}

/// One slide master, and what its list says.
#[derive(Debug, Clone, Default)]
pub(crate) struct MasterFacts {
    pub part: String,
    pub layouts: Vec<IdEntry>,
}

/// One package, as the rules read it.
#[derive(Debug, Clone, Default)]
pub(crate) struct View {
    /// Key to actual name, files only.
    pub names: BTreeMap<String, String>,
    /// Relationships by source, under the source's actual name; `""` is the package.
    pub rels: BTreeMap<String, RelsFacts>,
    /// `.rels` entries with no source.
    pub orphan_rels: Vec<String>,
    pub types: TypesFacts,
    pub main: String,
    pub presentation: PresentationFacts,
    pub masters: Vec<MasterFacts>,
    /// Keys of what the package root reaches.
    pub reach: BTreeSet<String>,
    /// What a `custShow` element elsewhere refers to.
    pub custom_show_refs: Vec<(String, String)>,
    /// What each notes master lacks.
    pub notes_master_gaps: Vec<(String, &'static str)>,
}

/// A child in the PresentationML namespace, whichever form the file uses, so a foreign
/// element of the same local name is not taken for it.
pub(crate) fn pml_children(tree: &XmlTree, parent: ElementId, local: &str) -> Vec<ElementId> {
    tree.child_elements(parent)
        .into_iter()
        .filter(|&c| {
            tree.local_name(c) == local
                && tree.normalized_namespace(c).as_deref() == Some(TRANSITIONAL_PML)
        })
        .collect()
}

/// The first such child.
pub(crate) fn pml_child(tree: &XmlTree, parent: ElementId, local: &str) -> Option<ElementId> {
    pml_children(tree, parent, local).into_iter().next()
}

/// The value of the attribute in the relationships namespace with this local name, on
/// `el`. Prefixes resolve outward, so any spelling of the prefix counts.
pub(crate) fn rel_attribute(tree: &XmlTree, el: ElementId, local: &str) -> Option<String> {
    tree.declared_attributes(el)
        .into_iter()
        .find_map(|(name, value)| {
            let (prefix, name) = name.split_once(':')?;
            (name == local && prefix_is_relationships(tree, el, prefix)).then_some(value)
        })
}

/// Whether `prefix` is bound to the relationships namespace at `el`.
pub(crate) fn prefix_is_relationships(tree: &XmlTree, el: ElementId, prefix: &str) -> bool {
    if prefix == "xmlns" {
        return false;
    }
    let wanted = format!("xmlns:{prefix}");
    let mut at = Some(el);
    while let Some(here) = at {
        if let Some((_, uri)) = tree
            .declared_attributes(here)
            .into_iter()
            .find(|(name, _)| *name == wanted)
        {
            return to_transitional(&uri) == TRANSITIONAL_R;
        }
        at = tree.parent_element(here);
    }
    false
}

fn id_entries(tree: &XmlTree, list: Option<ElementId>, child: &str) -> Vec<IdEntry> {
    list.map(|list| {
        pml_children(tree, list, child)
            .into_iter()
            .map(|el| {
                let raw = tree.plain_attribute(el, "id").unwrap_or_default();
                IdEntry {
                    id: raw.parse().ok(),
                    raw,
                    rel_id: rel_attribute(tree, el, "id").unwrap_or_default(),
                }
            })
            .collect()
    })
    .unwrap_or_default()
}

/// The elements of the section list, if the deck has one.
pub(crate) fn section_list(
    tree: &XmlTree,
    root: ElementId,
) -> Option<(ElementId, ElementId, ElementId)> {
    let ext_list = pml_child(tree, root, "extLst")?;
    let ext = tree
        .child_elements(ext_list)
        .into_iter()
        .find(|&e| tree.plain_attribute(e, "uri").as_deref() == Some(SECTION_EXT_URI))?;
    let list = tree.child(ext, "sectionLst")?;
    Some((ext_list, ext, list))
}

/// The sections of the part `tree` holds.
pub(crate) fn read_sections(tree: &XmlTree, list: ElementId) -> Vec<SectionFacts> {
    tree.child_elements(list)
        .into_iter()
        .filter(|&s| tree.local_name(s) == "section")
        .map(|section| SectionFacts {
            id: tree.plain_attribute(section, "id").unwrap_or_default(),
            slides: tree
                .child(section, "sldIdLst")
                .map(|l| {
                    tree.child_elements(l)
                        .into_iter()
                        .filter_map(|e| tree.plain_attribute(e, "id"))
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect()
}

fn presentation_facts(tree: &XmlTree) -> PresentationFacts {
    let root = tree.root();
    let slides = id_entries(tree, pml_child(tree, root, "sldIdLst"), "sldId");
    let masters = id_entries(tree, pml_child(tree, root, "sldMasterIdLst"), "sldMasterId");
    let rel_only = |list: &str, child: &str| -> Vec<String> {
        pml_child(tree, root, list)
            .map(|l| {
                pml_children(tree, l, child)
                    .into_iter()
                    .filter_map(|el| rel_attribute(tree, el, "id"))
                    .collect()
            })
            .unwrap_or_default()
    };
    let custom_shows = pml_child(tree, root, "custShowLst")
        .map(|l| {
            pml_children(tree, l, "custShow")
                .into_iter()
                .map(|show| CustomShowFacts {
                    id: tree.plain_attribute(show, "id").unwrap_or_default(),
                })
                .collect()
        })
        .unwrap_or_default();
    PresentationFacts {
        slides,
        masters,
        notes_masters: rel_only("notesMasterIdLst", "notesMasterId"),
        handout_masters: rel_only("handoutMasterIdLst", "handoutMasterId"),
        child_order: tree
            .child_elements(root)
            .into_iter()
            .map(|c| tree.local_name(c).to_owned())
            .collect(),
        first_slide_num: tree.plain_attribute(root, "firstSlideNum"),
        sections: section_list(tree, root).map(|(_, _, list)| read_sections(tree, list)),
        custom_shows,
    }
}

fn notes_master_gaps(tree: &XmlTree) -> Vec<&'static str> {
    let root = tree.root();
    let mut gaps = Vec::new();
    let c_sld = pml_child(tree, root, "cSld");
    if c_sld.is_none() {
        gaps.push("no cSld");
    }
    if pml_child(tree, root, "clrMap").is_none() {
        gaps.push("no clrMap");
    }
    let mut kinds = BTreeSet::new();
    if let Some(sp_tree) = c_sld.and_then(|c| pml_child(tree, c, "spTree")) {
        for sp in pml_children(tree, sp_tree, "sp") {
            let kind = pml_child(tree, sp, "nvSpPr")
                .and_then(|nv| pml_child(tree, nv, "nvPr"))
                .and_then(|nv| pml_child(tree, nv, "ph"))
                .and_then(|ph| tree.plain_attribute(ph, "type"));
            if let Some(kind) = kind {
                kinds.insert(kind);
            }
        }
    }
    for (kind, gap) in [
        ("hdr", "no hdr placeholder"),
        ("dt", "no dt placeholder"),
        ("sldImg", "no sldImg placeholder"),
        ("body", "no body placeholder"),
        ("ftr", "no ftr placeholder"),
        ("sldNum", "no sldNum placeholder"),
    ] {
        if !kinds.contains(kind) {
            gaps.push(gap);
        }
    }
    gaps
}

impl View {
    /// Read the structure from `parts`. A part that will not parse holds no facts: the
    /// save is not the place a retained part's fault becomes a refusal.
    pub(crate) fn build(parts: &mut dyn Parts, main: &str) -> Result<Self, WriterError> {
        let limits = parts.limits();
        let parse = |bytes: &[u8], name: &str| -> Result<XmlTree, OoxmlError> {
            XmlTree::parse(bytes, name, &limits)
        };
        let mut view = Self {
            main: main.to_owned(),
            ..Self::default()
        };
        let names = parts.names();
        for name in &names {
            view.names.insert(key(name), name.clone());
        }
        for name in &names {
            let Some(source_key) = source_of_rels(&key(name)) else {
                continue;
            };
            let source = if source_key.is_empty() {
                Some(String::new())
            } else {
                view.names.get(&source_key).cloned()
            };
            let Some(source) = source else {
                view.orphan_rels.push(name.clone());
                continue;
            };
            let Some(tree) = parts.read_xml(name).ok().and_then(|b| parse(&b, name).ok()) else {
                continue;
            };
            let items = read_relationships(&tree);
            let mut seen = BTreeSet::new();
            let duplicate_ids = items
                .iter()
                .filter(|r| !seen.insert(r.id.clone()))
                .map(|r| r.id.clone())
                .collect();
            let root = tree.root();
            let has_xml_base = std::iter::once(root)
                .chain(tree.child_elements(root))
                .any(|el| tree.plain_attribute(el, "xml:base").is_some());
            view.rels.insert(
                source,
                RelsFacts {
                    part: name.clone(),
                    items,
                    has_xml_base,
                    duplicate_ids,
                },
            );
        }
        if let Some(actual) = view.names.get(&key(CONTENT_TYPES_PART)).cloned()
            && let Ok(bytes) = parts.read_xml(&actual)
            && let Ok(tree) = parse(&bytes, &actual)
        {
            let root = tree.root();
            view.types.root_attributes = tree
                .declared_attributes(root)
                .into_iter()
                .map(|(n, _)| n)
                .filter(|n| n != "xmlns" && !n.starts_with("xmlns:"))
                .collect();
            for el in tree.child_elements(root) {
                match tree.local_name(el) {
                    "Default" => view.types.defaults.push((
                        tree.plain_attribute(el, "Extension").unwrap_or_default(),
                        tree.plain_attribute(el, "ContentType").unwrap_or_default(),
                    )),
                    "Override" => {
                        let name = tree.plain_attribute(el, "PartName").unwrap_or_default();
                        let ct = tree.plain_attribute(el, "ContentType").unwrap_or_default();
                        view.types
                            .override_of
                            .entry(key(&name))
                            .or_insert_with(|| ct.clone());
                        view.types.overrides.push((name, ct));
                    }
                    _ => {}
                }
            }
        }
        if let Ok(bytes) = parts.read_xml(main)
            && let Ok(tree) = parse(&bytes, main)
        {
            view.presentation = presentation_facts(&tree);
        }
        let master_rels: Vec<String> = view
            .presentation
            .masters
            .iter()
            .filter_map(|m| view.target_of(main, &m.rel_id))
            .collect();
        for part in master_rels {
            let layouts = parts
                .read_xml(&part)
                .ok()
                .and_then(|b| parse(&b, &part).ok())
                .map(|tree| {
                    id_entries(
                        &tree,
                        pml_child(&tree, tree.root(), "sldLayoutIdLst"),
                        "sldLayoutId",
                    )
                })
                .unwrap_or_default();
            view.masters.push(MasterFacts { part, layouts });
        }
        if let Some(props) = view.first_target(main, "presProps")
            && let Ok(bytes) = parts.read_xml(&props)
            && let Ok(tree) = parse(&bytes, &props)
        {
            let mut stack = vec![tree.root()];
            let mut budget = 10_000usize;
            while let Some(el) = stack.pop() {
                budget = budget.saturating_sub(1);
                if budget == 0 {
                    break;
                }
                if tree.local_name(el) == "custShow"
                    && let Some(id) = tree.plain_attribute(el, "id")
                {
                    view.custom_show_refs.push((props.clone(), id));
                }
                stack.extend(tree.child_elements(el));
            }
        }
        let notes_masters: Vec<String> = view.parts_of_type(".notesmaster+xml");
        for master in notes_masters {
            if let Ok(bytes) = parts.read_xml(&master)
                && let Ok(tree) = parse(&bytes, &master)
            {
                for gap in notes_master_gaps(&tree) {
                    view.notes_master_gaps.push((master.clone(), gap));
                }
            }
        }
        view.reach = view.reach_of(&[String::new()]);
        Ok(view)
    }

    /// The part a relationship of `source` lands on.
    pub(crate) fn resolved(&self, source: &str, rel: &Relationship) -> Option<String> {
        if rel.mode != TargetMode::Internal || rel.target.starts_with('#') {
            return None;
        }
        let target = resolve_internal(source, &rel.target)?;
        self.names.get(&part_name_key(&target)?).cloned()
    }

    /// The part `source`'s relationship `id` lands on.
    pub(crate) fn target_of(&self, source: &str, id: &str) -> Option<String> {
        let rel = self.rels.get(source)?.items.iter().find(|r| r.id == id)?;
        self.resolved(source, rel)
    }

    /// The first target of this kind from `source`.
    pub(crate) fn first_target(&self, source: &str, kind: &str) -> Option<String> {
        self.rels
            .get(source)?
            .items
            .iter()
            .find(|r| rel_kind(&r.rel_type) == kind)
            .and_then(|r| self.resolved(source, r))
    }

    /// All relationships of this kind from `source`.
    pub(crate) fn of_kind<'v>(&'v self, source: &str, kind: &str) -> Vec<&'v Relationship> {
        self.rels
            .get(source)
            .map(|r| {
                r.items
                    .iter()
                    .filter(|r| rel_kind(&r.rel_type) == kind)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// What is reachable from `from`, by key.
    pub(crate) fn reach_of(&self, from: &[String]) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut queue: VecDeque<String> = from.iter().cloned().collect();
        while let Some(part) = queue.pop_front() {
            if !seen.insert(key(&part)) {
                continue;
            }
            if let Some(rels) = self.rels.get(&part) {
                for rel in &rels.items {
                    if let Some(target) = self.resolved(&part, rel) {
                        queue.push_back(target);
                    }
                }
            }
        }
        seen
    }

    /// The type a part resolves to.
    pub(crate) fn content_type(&self, part: &str) -> Option<&str> {
        let wanted = key(part);
        if let Some(ct) = self.types.override_of.get(&wanted) {
            return Some(ct);
        }
        let (_, ext) = wanted.rsplit('/').next()?.rsplit_once('.')?;
        self.types
            .defaults
            .iter()
            .find(|(e, _)| e.eq_ignore_ascii_case(ext))
            .map(|(_, ct)| ct.as_str())
    }

    /// Whether this entry is a relationships part or the content types stream.
    pub(crate) fn is_structural(&self, name: &str) -> bool {
        is_rels_part(name) || key(name) == key(CONTENT_TYPES_PART)
    }

    /// The name of a part's `.rels`, if there is one.
    pub(crate) fn rels_entry(&self, part: &str) -> Option<&String> {
        self.names.get(&key(&rels_part_for(part)))
    }

    /// The parts of a kind, by the presentation's content type suffix.
    pub(crate) fn parts_of_type(&self, suffix: &str) -> Vec<String> {
        self.names
            .values()
            .filter(|n| {
                self.content_type(n)
                    .is_some_and(|ct| ct.to_ascii_lowercase().ends_with(suffix))
            })
            .cloned()
            .collect()
    }
}
