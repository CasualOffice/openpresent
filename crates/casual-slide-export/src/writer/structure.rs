//! The lists in `presentation.xml` and the master parts: slide, master and layout ids,
//! the schema position of a new list, and the section list.

use std::collections::{BTreeMap, BTreeSet};

use casual_slide_model::presentation::{SLIDE_ID_MAX, SLIDE_ID_MIN, SLIDE_MASTER_ID_MIN};
use casual_slide_ooxml::{Conformance, ElementId, XmlTree};

use super::draft::{Draft, set_attr, sibling_name};
use super::error::WriterError;
use super::ops::SlideRef;
use super::view::{
    SECTION_EXT_URI, SectionFacts, pml_child, pml_children, read_sections, rel_attribute,
    section_list,
};
use crate::presentation::PRESENTATION_ORDER;
use crate::properties::ExportError;

const R_NAMESPACE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const P14_NAMESPACE: &str = "http://schemas.microsoft.com/office/powerpoint/2010/main";
/// `ST_SlideLayoutId` and `ST_SlideMasterId` run to the top of an unsigned 32-bit number.
const POOL_MAX: u64 = 4_294_967_295;

/// One entry of `sldIdLst`, with the part it lands on.
#[derive(Debug, Clone)]
pub(crate) struct SlideEntry {
    pub raw_id: String,
    pub id: Option<u64>,
    pub rel_id: String,
    pub part: Option<String>,
    pub element: ElementId,
}

fn no_such(what: &'static str, subject: impl Into<String>) -> WriterError {
    WriterError::BadOperation {
        what,
        subject: subject.into(),
    }
}

/// `INV-ID-02` (Inferred): one above the highest; if that is out of range, the lowest
/// free. `retired` keeps a number given up in this save from coming back.
fn next_slide_id(used: &BTreeSet<u64>) -> Result<u64, WriterError> {
    let min = u64::from(SLIDE_ID_MIN);
    let max = u64::from(SLIDE_ID_MAX);
    let next = used
        .iter()
        .next_back()
        .map_or(Some(min), |top| top.checked_add(1))
        .filter(|n| (min..=max).contains(n));
    match next {
        Some(next) => Ok(next.max(min)),
        None => (min..=max)
            .find(|n| !used.contains(n))
            .ok_or(WriterError::Exhausted { what: "slide id" }),
    }
}

/// `INV-ID-04` (Inferred): one above the highest in the shared pool, capped; on overflow
/// the lowest free.
fn next_pool_id(used: &BTreeSet<u64>) -> Result<u64, WriterError> {
    let min = u64::from(SLIDE_MASTER_ID_MIN);
    let next = used
        .iter()
        .next_back()
        .map_or(Some(min), |top| top.checked_add(1))
        .filter(|n| (min..=POOL_MAX).contains(n));
    match next {
        Some(next) => Ok(next.max(min)),
        None => (min..=POOL_MAX)
            .find(|n| !used.contains(n))
            .ok_or(WriterError::Exhausted {
                what: "master or layout id",
            }),
    }
}

/// `xmlns:r` for a new element holding `r:id`, if nothing in scope binds it.
fn r_attr_name(tree: &XmlTree, at: ElementId, uri: &str) -> (String, Option<String>) {
    match tree.prefix_for_namespace(at, uri) {
        Some(prefix) if !prefix.is_empty() => (format!("{prefix}:id"), None),
        _ => ("r:id".to_owned(), Some(uri.to_owned())),
    }
}

impl Draft<'_> {
    fn r_uri(&self) -> &'static str {
        match self.conformance {
            Conformance::Transitional => R_NAMESPACE,
            Conformance::Strict => "http://purl.oclc.org/ooxml/officeDocument/relationships",
        }
    }

    /// A new list element at its schema position (`INV-ID-08`, Documented), or the one
    /// that is there.
    pub(crate) fn ensure_list(&mut self, local: &str) -> Result<ElementId, WriterError> {
        let main = self.main.clone();
        let tree = self.tree(&main)?;
        let root = tree.root();
        if let Some(found) = pml_child(tree, root, local) {
            return Ok(found);
        }
        let at = PRESENTATION_ORDER.iter().position(|l| *l == local);
        let before = tree.child_elements(root).into_iter().find(|&c| {
            let order = PRESENTATION_ORDER
                .iter()
                .position(|l| *l == tree.local_name(c));
            matches!((order, at), (Some(o), Some(a)) if o > a)
        });
        let name = sibling_name(tree, root, local);
        let new = tree.new_element(&name);
        tree.insert_child(root, before, new);
        Ok(new)
    }

    /// `sldIdLst`, in order.
    pub(crate) fn slide_entries(&mut self) -> Result<Vec<SlideEntry>, WriterError> {
        let main = self.main.clone();
        let rows: Vec<(String, String, ElementId)> = {
            let tree = self.tree(&main)?;
            let root = tree.root();
            pml_child(tree, root, "sldIdLst")
                .map(|list| {
                    pml_children(tree, list, "sldId")
                        .into_iter()
                        .map(|el| {
                            (
                                tree.plain_attribute(el, "id").unwrap_or_default(),
                                rel_attribute(tree, el, "id").unwrap_or_default(),
                                el,
                            )
                        })
                        .collect()
                })
                .unwrap_or_default()
        };
        let rels = self.relationships(&main)?;
        Ok(rows
            .into_iter()
            .map(|(raw_id, rel_id, element)| {
                let part = rels
                    .iter()
                    .find(|r| r.id == rel_id)
                    .and_then(|r| self.resolve(&main, r));
                SlideEntry {
                    id: raw_id.parse().ok(),
                    raw_id,
                    rel_id,
                    part,
                    element,
                }
            })
            .collect())
    }

    /// The entry a reference names, with the part it lands on.
    pub(crate) fn slide(&mut self, wanted: &SlideRef) -> Result<(usize, SlideEntry), WriterError> {
        let entries = self.slide_entries()?;
        let found = entries.into_iter().enumerate().find(|(i, e)| match wanted {
            SlideRef::Index(n) => i == n,
            SlideRef::Id(id) => e.id == Some(*id),
            SlideRef::Part(name) => e
                .part
                .as_deref()
                .is_some_and(|p| super::names::key(p) == super::names::key(name)),
        });
        found.ok_or_else(|| no_such("no such slide", format!("{wanted:?}")))
    }

    pub(crate) fn slide_part(entry: &SlideEntry) -> Result<String, WriterError> {
        entry
            .part
            .clone()
            .ok_or_else(|| no_such("a slide entry with no part", entry.rel_id.clone()))
    }

    /// Insert an entry at `position` in `sldIdLst`, with a new id.
    pub(crate) fn insert_slide_entry(
        &mut self,
        rel_id: &str,
        position: usize,
    ) -> Result<String, WriterError> {
        let entries = self.slide_entries()?;
        if position > entries.len() {
            return Err(no_such("position past the end", position.to_string()));
        }
        let mut used: BTreeSet<u64> = entries
            .iter()
            .filter_map(|e| e.id)
            .filter(|n| (u64::from(SLIDE_ID_MIN)..=u64::from(SLIDE_ID_MAX)).contains(n))
            .collect();
        used.extend(self.log.retired_slide_ids.iter().copied());
        let id = next_slide_id(&used)?.to_string();
        let uri = self.r_uri();
        let before = entries.get(position).map(|e| e.element);
        let list = self.ensure_list("sldIdLst")?;
        let main = self.main.clone();
        let tree = self.tree(&main)?;
        let name = sibling_name(tree, list, "sldId");
        let new = tree.new_element(&name);
        set_attr(tree, new, "id", &id, &main)?;
        let (attr, declare) = r_attr_name(tree, list, uri);
        if let Some(uri) = declare {
            set_attr(tree, new, "xmlns:r", &uri, &main)?;
        }
        set_attr(tree, new, &attr, rel_id, &main)?;
        tree.insert_child(list, before, new);
        Ok(id)
    }

    /// Whether the slide's section entry is to be kept when the list element goes.
    pub(crate) fn remove_slide_element(&mut self, element: ElementId) -> Result<(), WriterError> {
        let main = self.main.clone();
        let tree = self.tree(&main)?;
        if let Some(parent) = tree.parent_element(element) {
            tree.remove_child(parent, element);
        }
        Ok(())
    }

    /// Master and layout ids in the one pool (`INV-ID-03`; one pool is Inferred), with
    /// those given up in this save.
    pub(crate) fn pool_ids(&mut self) -> Result<BTreeSet<u64>, WriterError> {
        let main = self.main.clone();
        let mut used: BTreeSet<u64> = self.log.retired_layout_ids.clone();
        let (masters, rows): (Vec<u64>, Vec<String>) = {
            let tree = self.tree(&main)?;
            let root = tree.root();
            let list = pml_child(tree, root, "sldMasterIdLst");
            let mut masters = Vec::new();
            let mut rel_ids = Vec::new();
            if let Some(list) = list {
                for el in pml_children(tree, list, "sldMasterId") {
                    if let Some(n) = tree
                        .plain_attribute(el, "id")
                        .and_then(|v| v.parse::<u64>().ok())
                    {
                        masters.push(n);
                    }
                    if let Some(rel) = rel_attribute(tree, el, "id") {
                        rel_ids.push(rel);
                    }
                }
            }
            (masters, rel_ids)
        };
        used.extend(masters);
        let rels = self.relationships(&main)?;
        for rel_id in rows {
            let Some(part) = rels
                .iter()
                .find(|r| r.id == rel_id)
                .and_then(|r| self.resolve(&main, r))
            else {
                continue;
            };
            let tree = self.tree(&part)?;
            if let Some(list) = pml_child(tree, tree.root(), "sldLayoutIdLst") {
                for el in pml_children(tree, list, "sldLayoutId") {
                    if let Some(n) = tree
                        .plain_attribute(el, "id")
                        .and_then(|v| v.parse::<u64>().ok())
                    {
                        used.insert(n);
                    }
                }
            }
        }
        used.retain(|n| (u64::from(SLIDE_MASTER_ID_MIN)..=POOL_MAX).contains(n));
        Ok(used)
    }

    /// A new id from the pool, marked used.
    pub(crate) fn mint_pool_id(&mut self) -> Result<u64, WriterError> {
        let used = self.pool_ids()?;
        let id = next_pool_id(&used)?;
        self.log.retired_layout_ids.insert(id);
        Ok(id)
    }

    /// Add `r:id` and `id` as a new child named `local` of `list`.
    pub(crate) fn append_id_entry(
        &mut self,
        part: &str,
        list: ElementId,
        local: &str,
        id: Option<u64>,
        rel_id: &str,
    ) -> Result<(), WriterError> {
        let uri = self.r_uri();
        let tree = self.tree(part)?;
        let name = sibling_name(tree, list, local);
        let new = tree.new_element(&name);
        if let Some(id) = id {
            set_attr(tree, new, "id", &id.to_string(), part)?;
        }
        let (attr, declare) = r_attr_name(tree, list, uri);
        if let Some(uri) = declare {
            set_attr(tree, new, "xmlns:r", &uri, part)?;
        }
        set_attr(tree, new, &attr, rel_id, part)?;
        tree.insert_child(list, None, new);
        Ok(())
    }

    /// The sections as they stand, or `None` for a deck with no list.
    pub(crate) fn sections(&mut self) -> Result<Option<Vec<SectionFacts>>, WriterError> {
        let main = self.main.clone();
        let tree = self.tree(&main)?;
        let root = tree.root();
        Ok(section_list(tree, root).map(|(_, _, list)| read_sections(tree, list)))
    }

    /// Write a section's entries, touching nothing if they already say this.
    pub(crate) fn write_sections(&mut self, wanted: &[SectionFacts]) -> Result<(), WriterError> {
        let main = self.main.clone();
        let tree = self.tree(&main)?;
        let root = tree.root();
        let Some((_, _, list)) = section_list(tree, root) else {
            return Ok(());
        };
        // Sections sharing an id, as a hostile deck may have, are told apart by order: the
        // n-th with an id takes the n-th row with it, so one row is never written to all.
        let mut rows: BTreeMap<&str, std::collections::VecDeque<&SectionFacts>> = BTreeMap::new();
        for row in wanted {
            rows.entry(row.id.as_str()).or_default().push_back(row);
        }
        for section in tree.child_elements(list) {
            if tree.local_name(section) != "section" {
                continue;
            }
            let id = tree.plain_attribute(section, "id").unwrap_or_default();
            let Some(row) = rows.get_mut(id.as_str()).and_then(|r| r.pop_front()) else {
                continue;
            };
            let slides = match tree.child(section, "sldIdLst") {
                Some(slides) => slides,
                None => {
                    let name = sibling_name(tree, section, "sldIdLst");
                    let new = tree.new_element(&name);
                    let first = tree.child_elements(section).into_iter().next();
                    tree.insert_child(section, first, new);
                    new
                }
            };
            let current = tree.child_elements(slides);
            let ids: Vec<String> = current
                .iter()
                .filter_map(|&e| tree.plain_attribute(e, "id"))
                .collect();
            if ids == row.slides {
                continue;
            }
            let name = sibling_name(tree, slides, "sldId");
            let mut replacement = Vec::with_capacity(row.slides.len());
            for slide in &row.slides {
                let existing = current
                    .iter()
                    .copied()
                    .find(|&e| tree.plain_attribute(e, "id").as_deref() == Some(slide));
                let el = match existing {
                    Some(el) => el,
                    None => {
                        let new = tree.new_element(&name);
                        set_attr(tree, new, "id", slide, &main)?;
                        new
                    }
                };
                replacement.push(el);
            }
            tree.replace_children(slides, &current, &replacement);
        }
        Ok(())
    }

    /// Put a slide in the section `INV-SL-06` (Inferred) says: that of the slide before
    /// it, and at position 0, the first section.
    pub(crate) fn place_in_section(
        &mut self,
        slide: &str,
        previous: Option<&str>,
    ) -> Result<(), WriterError> {
        let Some(mut sections) = self.sections()? else {
            return Ok(());
        };
        for s in &mut sections {
            s.slides.retain(|id| id != slide);
        }
        // A previous slide in no section arrived that way. The new one still goes
        // somewhere, so it is not one more slide in none: after the nearest slide before it
        // that is in a section, so reading the sections still gives the slide order
        // (`INV-SL-05`); with none, first in the first section.
        let earlier: Vec<String> = match previous {
            Some(p) => {
                let entries = self.slide_entries()?;
                let upto = entries.iter().position(|e| e.raw_id == p);
                entries
                    .into_iter()
                    .take(upto.map_or(0, |i| i.saturating_add(1)))
                    .map(|e| e.raw_id)
                    .collect()
            }
            None => Vec::new(),
        };
        let anchor = earlier.iter().rev().find_map(|p| {
            sections
                .iter()
                .position(|s| s.slides.iter().any(|id| id == p))
                .map(|at| (at, p))
        });
        match anchor {
            Some((at, p)) => {
                if let Some(s) = sections.get_mut(at) {
                    let after = s.slides.iter().position(|id| id == p).map_or(0, |i| i + 1);
                    s.slides.insert(after, slide.to_owned());
                }
            }
            None => {
                if let Some(s) = sections.first_mut() {
                    s.slides.insert(0, slide.to_owned());
                }
            }
        }
        self.write_sections(&sections)
    }

    /// Record where each new slide now sits, after an operation that placed it.
    pub(crate) fn note_new_slide_sections(&mut self) -> Result<(), WriterError> {
        let sections = self.sections()?.unwrap_or_default();
        let mut placed = std::collections::BTreeMap::new();
        for id in &self.log.new_slide_ids {
            if let Some(section) = sections.iter().find(|s| s.slides.contains(id)) {
                placed.insert(id.clone(), section.id.clone());
            }
        }
        self.log.section_of_new = placed;
        Ok(())
    }

    /// Take a slide out of every section (`INV-SL-04`, `INV-SP-09`).
    pub(crate) fn drop_from_sections(&mut self, slide: &str) -> Result<(), WriterError> {
        let Some(mut sections) = self.sections()? else {
            return Ok(());
        };
        for s in &mut sections {
            s.slides.retain(|id| id != slide);
        }
        self.write_sections(&sections)
    }

    /// Reorder `sldIdLst` so reading the sections gives it (`INV-SL-05`, Inferred).
    pub(crate) fn order_slides_by_sections(&mut self) -> Result<(), WriterError> {
        let Some(sections) = self.sections()? else {
            return Ok(());
        };
        let entries = self.slide_entries()?;
        let in_sections: BTreeSet<&String> = sections.iter().flat_map(|s| &s.slides).collect();
        let mut order: Vec<ElementId> = Vec::with_capacity(entries.len());
        for section in &sections {
            for slide in &section.slides {
                if let Some(entry) = entries.iter().find(|e| &e.raw_id == slide) {
                    order.push(entry.element);
                }
            }
        }
        for entry in &entries {
            if !in_sections.contains(&entry.raw_id) {
                order.push(entry.element);
            }
        }
        let current: Vec<ElementId> = entries.iter().map(|e| e.element).collect();
        if order == current || order.len() != current.len() {
            return Ok(());
        }
        let main = self.main.clone();
        let tree = self.tree(&main)?;
        if let Some(list) = pml_child(tree, tree.root(), "sldIdLst") {
            tree.replace_children(list, &current, &order);
        }
        Ok(())
    }

    /// Create the section list: one section, holding every slide (`INV-SL-05`).
    pub(crate) fn create_section_list(&mut self, id: &str, name: &str) -> Result<(), WriterError> {
        let slides: Vec<String> = self
            .slide_entries()?
            .into_iter()
            .map(|e| e.raw_id)
            .collect();
        let ext_list = self.ensure_list("extLst")?;
        let main = self.main.clone();
        let tree = self.tree(&main)?;
        let ext_name = sibling_name(tree, ext_list, "ext");
        let ext = tree.new_element(&ext_name);
        set_attr(tree, ext, "uri", SECTION_EXT_URI, &main)?;
        let list = tree.new_element("p14:sectionLst");
        set_attr(tree, list, "xmlns:p14", P14_NAMESPACE, &main)?;
        let section = tree.new_element("p14:section");
        set_attr(tree, section, "name", name, &main)?;
        set_attr(tree, section, "id", id, &main)?;
        let holder = tree.new_element("p14:sldIdLst");
        for slide in &slides {
            let el = tree.new_element("p14:sldId");
            set_attr(tree, el, "id", slide, &main)?;
            tree.insert_child(holder, None, el);
        }
        tree.insert_child(section, None, holder);
        tree.insert_child(list, None, section);
        tree.insert_child(ext, None, list);
        tree.insert_child(ext_list, None, ext);
        Ok(())
    }

    /// Add a section to a list that exists, after `after` (an index), or last.
    pub(crate) fn append_section(
        &mut self,
        id: &str,
        name: &str,
        after: Option<usize>,
    ) -> Result<(), WriterError> {
        let main = self.main.clone();
        let tree = self.tree(&main)?;
        let root = tree.root();
        let Some((_, _, list)) = section_list(tree, root) else {
            return Err(WriterError::Export(ExportError::MalformedElement {
                part: main,
            }));
        };
        let sections: Vec<ElementId> = tree
            .child_elements(list)
            .into_iter()
            .filter(|&s| tree.local_name(s) == "section")
            .collect();
        let before = after.and_then(|at| sections.get(at.saturating_add(1)).copied());
        let name_of = sibling_name(tree, list, "section");
        let section = tree.new_element(&name_of);
        set_attr(tree, section, "name", name, &main)?;
        set_attr(tree, section, "id", id, &main)?;
        let holder_name = sibling_name(tree, list, "sldIdLst");
        let holder = tree.new_element(&holder_name);
        tree.insert_child(section, None, holder);
        // Without a section to go before, after the last section rather than after any
        // extension list the section list may carry.
        let before = before.or_else(|| {
            tree.child_elements(list)
                .into_iter()
                .find(|&c| tree.local_name(c) == "extLst")
        });
        tree.insert_child(list, before, section);
        Ok(())
    }

    /// Remove a section element. The last one takes the `ext` with it, keeping other
    /// entries (`INV-SL-02`, Documented).
    pub(crate) fn remove_section_element(&mut self, id: &str) -> Result<(), WriterError> {
        let main = self.main.clone();
        let tree = self.tree(&main)?;
        let root = tree.root();
        let Some((ext_list, ext, list)) = section_list(tree, root) else {
            return Ok(());
        };
        let sections: Vec<ElementId> = tree
            .child_elements(list)
            .into_iter()
            .filter(|&s| tree.local_name(s) == "section")
            .collect();
        for &section in &sections {
            if tree.plain_attribute(section, "id").as_deref() == Some(id) {
                tree.remove_child(list, section);
            }
        }
        let left = tree
            .child_elements(list)
            .into_iter()
            .any(|s| tree.local_name(s) == "section");
        if !left {
            tree.remove_child(ext_list, ext);
            if tree.child_elements(ext_list).is_empty() {
                tree.remove_child(root, ext_list);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slide_id_is_one_above_the_top_or_the_lowest_free_at_the_cap() {
        let set = |v: &[u64]| v.iter().copied().collect::<BTreeSet<u64>>();
        assert_eq!(next_slide_id(&set(&[])).unwrap(), 256);
        assert_eq!(next_slide_id(&set(&[256, 300])).unwrap(), 301);
        assert_eq!(next_slide_id(&set(&[256, 2_147_483_647])).unwrap(), 257);
    }

    #[test]
    fn a_pool_id_is_one_above_the_top_or_the_lowest_free_at_the_cap() {
        let set = |v: &[u64]| v.iter().copied().collect::<BTreeSet<u64>>();
        assert_eq!(next_pool_id(&set(&[])).unwrap(), 2_147_483_648);
        assert_eq!(next_pool_id(&set(&[2_147_483_649])).unwrap(), 2_147_483_650);
        assert_eq!(
            next_pool_id(&set(&[2_147_483_648, 4_294_967_295])).unwrap(),
            2_147_483_649
        );
    }
}
