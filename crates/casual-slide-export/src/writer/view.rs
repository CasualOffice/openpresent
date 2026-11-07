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
    /// `docProps/app.xml`, as the package root names it.
    pub app: Option<AppFacts>,
    /// `docProps/core.xml`, as the package root names it.
    pub core: Option<CoreFacts>,
    /// `sldIdLst`'s slides, read only when `app` has something to hold them to.
    pub slides: Vec<SlideFacts>,
}

/// What `app.xml` says, as written.
#[derive(Debug, Clone, Default)]
pub(crate) struct AppFacts {
    pub part: String,
    pub slides: Option<String>,
    pub hidden_slides: Option<String>,
    pub notes: Option<String>,
    pub application: Option<String>,
    pub app_version: Option<String>,
    pub dig_sig: bool,
    /// `HeadingPairs`: each group's name and count.
    pub groups: Option<Vec<(String, Option<usize>)>>,
    pub heading_size: Option<String>,
    pub heading_items: usize,
    pub titles: Option<Vec<String>>,
    pub titles_size: Option<String>,
}

/// What `core.xml` says, as written.
#[derive(Debug, Clone, Default)]
pub(crate) struct CoreFacts {
    pub part: String,
    pub modified: Option<String>,
    pub modified_type: Option<String>,
    pub created: Option<String>,
    pub revision: Option<String>,
    pub last_modified_by: Option<String>,
    /// The keywords element, attributes and text, for comparison only.
    pub keywords: Option<String>,
}

/// One slide, as the counts and titles read it.
#[derive(Debug, Clone, Default)]
pub(crate) struct SlideFacts {
    pub key: String,
    pub hidden: bool,
    pub has_notes: bool,
    pub title: String,
}

/// Slides, hidden slides, and slides with notes.
pub(crate) fn counts_of(slides: &[SlideFacts]) -> (usize, usize, usize) {
    (
        slides.len(),
        slides.iter().filter(|s| s.hidden).count(),
        slides.iter().filter(|s| s.has_notes).count(),
    )
}

fn local_child(tree: &XmlTree, parent: ElementId, local: &str) -> Option<ElementId> {
    tree.child_elements(parent)
        .into_iter()
        .find(|&el| tree.local_name(el) == local)
}

fn app_facts(tree: &XmlTree, part: &str) -> AppFacts {
    let root = tree.root();
    let text = |local: &str| local_child(tree, root, local).and_then(|el| tree.text(el));
    let vector =
        |local: &str| local_child(tree, root, local).and_then(|el| local_child(tree, el, "vector"));
    let heading = vector("HeadingPairs");
    let variants = heading.map(|v| tree.child_elements(v)).unwrap_or_default();
    let inner = |el: &ElementId| tree.child_elements(*el).first().and_then(|&c| tree.text(c));
    let groups = heading.map(|_| {
        variants
            .chunks(2)
            .map(|pair| {
                let name = pair.first().and_then(inner).unwrap_or_default();
                let count = pair
                    .get(1)
                    .and_then(inner)
                    .and_then(|t| t.trim().parse::<usize>().ok());
                (name, count)
            })
            .collect()
    });
    let list = vector("TitlesOfParts");
    AppFacts {
        part: part.to_owned(),
        slides: text("Slides"),
        hidden_slides: text("HiddenSlides"),
        notes: text("Notes"),
        application: text("Application"),
        app_version: text("AppVersion"),
        dig_sig: local_child(tree, root, "DigSig").is_some(),
        groups,
        heading_size: heading.and_then(|v| tree.plain_attribute(v, "size")),
        heading_items: variants.len(),
        titles: list.map(|v| {
            tree.child_elements(v)
                .iter()
                .map(|&el| tree.text(el).unwrap_or_default())
                .collect()
        }),
        titles_size: list.and_then(|v| tree.plain_attribute(v, "size")),
    }
}

fn core_facts(tree: &XmlTree, part: &str) -> CoreFacts {
    let root = tree.root();
    let text = |local: &str| local_child(tree, root, local).and_then(|el| tree.text(el));
    let modified = local_child(tree, root, "modified");
    CoreFacts {
        part: part.to_owned(),
        modified: modified.and_then(|el| tree.text(el)),
        modified_type: modified.and_then(|el| tree.plain_attribute(el, "xsi:type")),
        created: text("created"),
        revision: text("revision"),
        last_modified_by: text("lastModifiedBy"),
        keywords: local_child(tree, root, "keywords").map(|el| {
            let mut out = String::new();
            let mut stack = vec![el];
            while let Some(here) = stack.pop() {
                for (name, value) in tree.declared_attributes(here) {
                    out.push_str(&name);
                    out.push('=');
                    out.push_str(&value);
                    out.push(' ');
                }
                out.push_str(&tree.own_text(here).unwrap_or_default());
                stack.extend(tree.child_elements(here));
            }
            out
        }),
    }
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

fn slide_title_of(tree: Option<&XmlTree>) -> String {
    tree.map_or_else(
        || super::bookkeeping::UNTITLED.to_owned(),
        super::bookkeeping::slide_title,
    )
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
        view.read_bookkeeping(parts)?;
        Ok(view)
    }

    /// `app.xml`, `core.xml`, and each slide only if `app.xml` is there to be judged. A
    /// part that will not parse holds no facts.
    fn read_bookkeeping(&mut self, parts: &mut dyn Parts) -> Result<(), WriterError> {
        let limits = parts.limits();
        let root: Vec<Relationship> = self
            .rels
            .get("")
            .map(|r| r.items.clone())
            .unwrap_or_default();
        let target = |view: &Self, suffix: &str| {
            root.iter()
                .find(|r| r.rel_type.ends_with(suffix))
                .and_then(|r| view.resolved("", r))
        };
        let tree_of = |parts: &mut dyn Parts, name: &str| {
            parts
                .read_xml(name)
                .ok()
                .and_then(|b| XmlTree::parse(&b, name, &limits).ok())
        };
        if let Some(part) = target(self, "/metadata/core-properties")
            && let Some(tree) = tree_of(parts, &part)
        {
            self.core = Some(core_facts(&tree, &part));
        }
        let Some(part) = target(self, "/extended-properties") else {
            return Ok(());
        };
        let Some(tree) = tree_of(parts, &part) else {
            return Ok(());
        };
        self.app = Some(app_facts(&tree, &part));
        let entries = self.presentation.slides.clone();
        for entry in entries {
            let target = self.target_of(&self.main, &entry.rel_id);
            let mut slide = SlideFacts {
                title: slide_title_of(None),
                ..SlideFacts::default()
            };
            if let Some(name) = target {
                slide.key = key(&name);
                slide.has_notes = !self.of_kind(&name, "notesSlide").is_empty();
                if let Some(tree) = tree_of(parts, &name) {
                    slide.hidden = tree
                        .plain_attribute(tree.root(), "show")
                        .is_some_and(|v| v == "0" || v == "false");
                    slide.title = slide_title_of(Some(&tree));
                }
            }
            self.slides.push(slide);
        }
        Ok(())
    }

    /// Slides, hidden slides, and slides with notes.
    pub(crate) fn counts(&self) -> (usize, usize, usize) {
        counts_of(&self.slides)
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
