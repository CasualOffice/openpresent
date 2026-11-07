//! Preflight: every implemented `INV-*` of `docs/36`, run on the input and on the draft.
//!
//! A rule either reads one package ([`state_rules`], [`written_part_rules`]), and is run
//! on both, so what the save added is the difference; or it compares the two
//! ([`change_rules`]). Violations are compared by rule, part and detail, and details hold
//! ids and counts only. **A violation that arrived is reported and left alone**, including
//! one a clone inherits from its source. Each rule's mark in a comment is the one `docs/36`
//! gives it, and an Inferred or Unverified rule is on `PV-01`'s list.

use std::collections::{BTreeMap, BTreeSet};

use casual_slide_model::presentation::{SLIDE_ID_MAX, SLIDE_ID_MIN, SLIDE_MASTER_ID_MIN};
use casual_slide_ooxml::{
    Conformance, ElementId, Relationship, TargetMode, XmlTree, part_name_key,
    relationship_type_to_strict, relationship_type_to_transitional, resolve_internal,
};

use super::clone::is_media;
use super::draft::{Draft, rel_kind};
use super::error::WriterError;
use super::format::main_type_for;
use super::names::{Families, family_of, is_mintable_segment, is_rels_part, key};
use super::view::{View, prefix_is_relationships};
use crate::presentation::PRESENTATION_ORDER;

const POOL_MAX: u64 = 4_294_967_295;
/// What a `Deck` patch states for itself, and so answers for (`Draft::model_owns_app`).
const APP_COUNT_RULES: &[&str] = &["INV-AP-01", "INV-AP-02", "INV-AP-03"];

/// One broken rule. Details hold ids and counts, never document content.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Violation {
    /// The invariant, as `INV-PN-03`.
    pub rule: &'static str,
    /// The part it is about.
    pub part: String,
    /// A fixed phrase, or an id.
    pub detail: String,
}

/// What preflight found, having refused nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preflight {
    /// Violations the input already had, which are left alone.
    pub arrived: Vec<Violation>,
}

pub(crate) type Found = BTreeSet<Violation>;

pub(crate) fn add(found: &mut Found, rule: &'static str, part: &str, detail: impl Into<String>) {
    found.insert(Violation {
        rule,
        part: part.to_owned(),
        detail: detail.into(),
    });
}

fn is_slide_like(view: &View, part: &str) -> bool {
    view.content_type(part).is_some_and(|ct| {
        let ct = ct.to_ascii_lowercase();
        [
            ".slide+xml",
            ".slidelayout+xml",
            ".slidemaster+xml",
            ".notesslide+xml",
            ".notesmaster+xml",
            ".handoutmaster+xml",
        ]
        .iter()
        .any(|suffix| ct.ends_with(suffix))
    })
}

/// `xsd:ID`, near enough: an NCName.
fn is_xsd_id(id: &str) -> bool {
    let mut chars = id.chars();
    let first = chars.next().is_some_and(|c| c.is_alphabetic() || c == '_');
    first && chars.all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

/// `INV-PN-01` (Documented), for one name.
fn name_is_well_formed(name: &str) -> bool {
    if part_name_key(name).is_none() {
        return false;
    }
    name.split('/').all(|segment| {
        if segment.is_empty() || segment.ends_with('.') {
            return false;
        }
        let bytes = segment.as_bytes();
        let mut i = 0usize;
        while let Some(&b) = bytes.get(i) {
            if b == b'%' {
                let hi = bytes
                    .get(i.saturating_add(1))
                    .and_then(|c| char::from(*c).to_digit(16));
                let lo = bytes
                    .get(i.saturating_add(2))
                    .and_then(|c| char::from(*c).to_digit(16));
                if let (Some(hi), Some(lo)) = (hi, lo) {
                    let value = hi.saturating_mul(16).saturating_add(lo);
                    let unreserved = char::from_u32(value).is_some_and(|c| {
                        c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_' | '~')
                    });
                    if unreserved || matches!(value, 0x2F | 0x5C) {
                        return false;
                    }
                }
            }
            i = i.saturating_add(1);
        }
        true
    })
}

pub(crate) fn is_section_guid(id: &str) -> bool {
    let Some(inner) = id.strip_prefix('{').and_then(|s| s.strip_suffix('}')) else {
        return false;
    };
    let groups: Vec<&str> = inner.split('-').collect();
    let lengths = [8usize, 4, 4, 4, 12];
    groups.len() == lengths.len()
        && groups.iter().zip(lengths).all(|(g, n)| {
            g.len() == n
                && g.chars()
                    .all(|c| c.is_ascii_digit() || ('A'..='F').contains(&c))
        })
}

/// A number that is only an id, so a detail can never carry file text.
fn id_detail(raw: &str) -> String {
    if raw.chars().all(|c| c.is_ascii_digit()) && raw.len() <= 20 {
        raw.to_owned()
    } else {
        "not a number".to_owned()
    }
}

fn rule_set_names(view: &View, found: &mut Found) {
    for name in view.names.values() {
        // INV-PN-01 (Documented).
        if !name_is_well_formed(name) {
            add(found, "INV-PN-01", name, "not a valid part name");
        }
        // INV-PN-02: what arrived non-ASCII is kept and named here.
        if !name.is_ascii() {
            add(found, "INV-PN-02", name, "not ASCII");
        }
        // INV-PN-04 (Documented).
        let k = key(name);
        let mut at = k.as_str();
        while let Some((parent, _)) = at.rsplit_once('/') {
            if view.names.contains_key(parent) {
                add(
                    found,
                    "INV-PN-04",
                    name,
                    "derivable from another part's name",
                );
            }
            at = parent;
        }
    }
    // INV-PN-07 (name mapping Documented, cascade Inferred): a `.rels` with no source.
    for name in &view.orphan_rels {
        add(
            found,
            "INV-PN-07",
            name,
            "relationships of a part that is not there",
        );
    }
}

fn rule_set_rels(view: &View, found: &mut Found) {
    for (source, facts) in &view.rels {
        // INV-RI-06 (Documented).
        if facts.has_xml_base {
            add(
                found,
                "INV-RI-06",
                &facts.part,
                "xml:base in a relationships part",
            );
        }
        if is_rels_part(source) {
            add(
                found,
                "INV-RI-06",
                source,
                "a relationships part has relationships",
            );
        }
        // INV-RI-01 (Documented).
        for id in &facts.duplicate_ids {
            add(
                found,
                "INV-RI-01",
                &facts.part,
                format!("repeated id {}", safe_id(id)),
            );
        }
        let slide_like = !source.is_empty() && is_slide_like(view, source);
        for rel in &facts.items {
            if !is_xsd_id(&rel.id) {
                add(
                    found,
                    "INV-RI-01",
                    &facts.part,
                    format!("id {} is not an xsd:ID", safe_id(&rel.id)),
                );
            }
            let kind = rel_kind(&rel.rel_type);
            // INV-RI-08 (Documented).
            if slide_like && kind.ends_with("metadata/thumbnail") {
                add(
                    found,
                    "INV-RI-08",
                    source,
                    "a thumbnail relationship on a slide-like part",
                );
            }
            if source == &view.main && kind == "slideLayout" {
                add(
                    found,
                    "INV-RI-08",
                    source,
                    "a presentation relationship to a layout",
                );
            }
            if rel.mode != TargetMode::Internal || rel.target.starts_with('#') {
                continue;
            }
            let Some(target) = resolve_internal(source, &rel.target) else {
                continue;
            };
            let Some(target_key) = part_name_key(&target) else {
                continue;
            };
            // INV-RI-06: a relationships part is never a target.
            if is_rels_part(&target) {
                add(
                    found,
                    "INV-RI-06",
                    &facts.part,
                    format!("{} targets a relationships part", safe_id(&rel.id)),
                );
            }
            // INV-PN-07: a relationship to a part that is not there.
            if !view.names.contains_key(&target_key) {
                add(
                    found,
                    "INV-PN-07",
                    source,
                    format!("{} dangles", safe_id(&rel.id)),
                );
            }
        }
    }
}

/// An id, kept to a safe alphabet, for a message.
fn safe_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
        .take(40)
        .collect()
}

fn rule_set_content_types(view: &View, found: &mut Found) {
    // INV-CT-07 (Documented).
    for name in &view.types.root_attributes {
        add(found, "INV-CT-07", "[Content_Types].xml", safe_id(name));
    }
    // INV-CT-02 (Documented).
    let mut seen = BTreeSet::new();
    for (ext, _) in &view.types.defaults {
        if !seen.insert(ext.to_ascii_lowercase()) {
            add(
                found,
                "INV-CT-02",
                "[Content_Types].xml",
                format!("Default {}", safe_id(ext)),
            );
        }
    }
    let mut seen = BTreeSet::new();
    for (name, _) in &view.types.overrides {
        let k = key(name);
        if !seen.insert(k.clone()) {
            add(found, "INV-CT-02", name, "Override repeated");
        }
        // INV-CT-05 (Inferred).
        if !view.names.contains_key(&k) {
            add(found, "INV-CT-05", name, "Override with no part");
        }
    }
    // INV-CT-01 (Documented).
    for name in view.names.values() {
        if !view.is_structural(name) && view.content_type(name).is_none() {
            add(found, "INV-CT-01", name, "no Default or Override");
        }
    }
}

fn rule_set_shared(view: &View, found: &mut Found) {
    let main = view.main.as_str();
    // INV-SP-05 (Inferred).
    let mut slides: BTreeSet<String> = view
        .of_kind(main, "slide")
        .into_iter()
        .filter_map(|r| view.resolved(main, r))
        .collect();
    slides.extend(view.parts_of_type(".slide+xml"));
    for slide in &slides {
        let layouts = view.of_kind(slide, "slideLayout").len();
        if layouts != 1 {
            add(
                found,
                "INV-SP-05",
                slide,
                format!("{layouts} layout relationships"),
            );
        }
    }
    // INV-SP-06 (Documented; refusal Inferred).
    if view.presentation.masters.is_empty() {
        add(found, "INV-SP-06", main, "no master");
    }
    let mut on_lists: BTreeMap<String, usize> = BTreeMap::new();
    for master in &view.masters {
        for entry in &master.layouts {
            match view.target_of(&master.part, &entry.rel_id) {
                None => add(
                    found,
                    "INV-SP-06",
                    &master.part,
                    format!("layout entry {} lands nowhere", safe_id(&entry.rel_id)),
                ),
                Some(layout) => {
                    let n = on_lists.entry(key(&layout)).or_insert(0);
                    *n = n.saturating_add(1);
                    let back = view.first_target(&layout, "slideMaster");
                    if back.as_deref().map(key) != Some(key(&master.part)) {
                        add(
                            found,
                            "INV-SP-06",
                            &layout,
                            "does not point back to its master",
                        );
                    }
                }
            }
        }
        for rel in view.of_kind(&master.part, "slideLayout") {
            if !master.layouts.iter().any(|e| e.rel_id == rel.id) {
                add(
                    found,
                    "INV-SP-06",
                    &master.part,
                    format!("{} is not on the layout list", safe_id(&rel.id)),
                );
            }
        }
    }
    if !view.masters.is_empty() && on_lists.is_empty() {
        add(found, "INV-SP-06", main, "no layout");
    }
    for layout in view.parts_of_type(".slidelayout+xml") {
        match on_lists.get(&key(&layout)).copied().unwrap_or(0) {
            1 => {}
            n => add(found, "INV-SP-06", &layout, format!("on {n} master lists")),
        }
    }
    // INV-SP-07 (Documented for one per master; never sharing Inferred).
    let mut holders: BTreeMap<String, usize> = BTreeMap::new();
    let mut owners: Vec<String> = view.masters.iter().map(|m| m.part.clone()).collect();
    owners.extend(
        view.presentation
            .notes_masters
            .iter()
            .chain(&view.presentation.handout_masters)
            .filter_map(|id| view.target_of(main, id)),
    );
    for owner in &owners {
        let themes = view.of_kind(owner, "theme");
        if themes.len() > 1 {
            add(
                found,
                "INV-SP-07",
                owner,
                format!("{} themes", themes.len()),
            );
        }
        for rel in themes {
            if let Some(theme) = view.resolved(owner, rel) {
                let n = holders.entry(theme).or_insert(0);
                *n = n.saturating_add(1);
            }
        }
    }
    for (theme, n) in holders {
        if n > 1 {
            add(found, "INV-SP-07", &theme, "shared");
        }
    }
    // INV-SP-08 (Unverified): the presentation's own theme is not left dangling.
    for rel in view.of_kind(main, "theme") {
        if view.resolved(main, rel).is_none() {
            add(found, "INV-SP-08", main, "theme relationship dangling");
        }
    }
}

fn rule_set_ids(view: &View, found: &mut Found) {
    let main = view.main.as_str();
    let p = &view.presentation;
    // INV-ID-01 (Documented).
    let mut seen = BTreeSet::new();
    for entry in &p.slides {
        let in_range = entry
            .id
            .is_some_and(|n| (u64::from(SLIDE_ID_MIN)..=u64::from(SLIDE_ID_MAX)).contains(&n));
        if !in_range {
            add(
                found,
                "INV-ID-01",
                main,
                format!("slide id {} out of range", id_detail(&entry.raw)),
            );
        }
        if !seen.insert(entry.raw.clone()) {
            add(
                found,
                "INV-ID-01",
                main,
                format!("slide id {} repeated", id_detail(&entry.raw)),
            );
        }
    }
    // INV-ID-03 (Documented for range and uniqueness; one pool Inferred).
    let mut pool = BTreeSet::new();
    let lists = std::iter::once((main, &p.masters))
        .chain(view.masters.iter().map(|m| (m.part.as_str(), &m.layouts)));
    for (part, list) in lists {
        for entry in list {
            // `sldMasterId/@id` is optional in the schema, and an absent one has no number.
            if entry.raw.is_empty() {
                continue;
            }
            let in_range = entry
                .id
                .is_some_and(|n| (u64::from(SLIDE_MASTER_ID_MIN)..=POOL_MAX).contains(&n));
            if !in_range {
                add(
                    found,
                    "INV-ID-03",
                    part,
                    format!("id {} out of range", id_detail(&entry.raw)),
                );
            }
            if !pool.insert(entry.raw.clone()) {
                add(
                    found,
                    "INV-ID-03",
                    part,
                    format!("id {} repeated in the pool", id_detail(&entry.raw)),
                );
            }
        }
    }
    // INV-ID-05 (Documented).
    let mut shows = BTreeSet::new();
    for show in &p.custom_shows {
        if !shows.insert(show.id.clone()) {
            add(
                found,
                "INV-ID-05",
                main,
                format!("custom show id {} repeated", id_detail(&show.id)),
            );
        }
    }
    for (part, id) in &view.custom_show_refs {
        if !shows.contains(id) {
            add(
                found,
                "INV-ID-05",
                part,
                format!("custom show {} does not exist", id_detail(id)),
            );
        }
    }
    // INV-ID-08 (Documented).
    let mut last = 0usize;
    for child in &p.child_order {
        if let Some(at) = PRESENTATION_ORDER.iter().position(|l| l == child) {
            if at < last {
                add(
                    found,
                    "INV-ID-08",
                    main,
                    format!("{} out of schema order", safe_id(child)),
                );
            }
            last = last.max(at);
        }
    }
    // INV-ID-09 (Documented).
    if let Some(first) = &p.first_slide_num {
        let ok = first.parse::<i64>().is_ok_and(|n| (0..=9999).contains(&n));
        if !ok {
            add(
                found,
                "INV-ID-09",
                main,
                format!("firstSlideNum {}", id_detail(first)),
            );
        }
    }
}

fn rule_set_sections(view: &View, found: &mut Found) {
    let main = view.main.as_str();
    let Some(sections) = &view.presentation.sections else {
        return;
    };
    // INV-SL-02 (Documented).
    if sections.is_empty() {
        add(found, "INV-SL-02", main, "an empty section list");
    }
    let slide_ids: BTreeSet<&String> = view.presentation.slides.iter().map(|e| &e.raw).collect();
    let mut guids = BTreeSet::new();
    let mut counts: BTreeMap<&String, usize> = BTreeMap::new();
    let mut read_order = Vec::new();
    for section in sections {
        // INV-SL-07 (Documented).
        if !is_section_guid(&section.id) {
            add(
                found,
                "INV-SL-07",
                main,
                "a section id is not a brace-wrapped upper-case GUID",
            );
        }
        if !guids.insert(section.id.clone()) {
            add(found, "INV-SL-07", main, "a section id is repeated");
        }
        for slide in &section.slides {
            // INV-SL-04 (Documented type; integrity Inferred).
            if !slide_ids.contains(slide) {
                add(
                    found,
                    "INV-SL-04",
                    main,
                    format!("section entry {} has no slide", id_detail(slide)),
                );
            }
            let n = counts.entry(slide).or_insert(0);
            *n = n.saturating_add(1);
            read_order.push(slide);
        }
    }
    // INV-SL-05 (Inferred; what PowerPoint does otherwise is Unverified).
    for slide in &view.presentation.slides {
        match counts.get(&slide.raw).copied().unwrap_or(0) {
            1 => {}
            0 => add(
                found,
                "INV-SL-05",
                main,
                format!("slide {} in no section", id_detail(&slide.raw)),
            ),
            _ => add(
                found,
                "INV-SL-05",
                main,
                format!("slide {} in two sections", id_detail(&slide.raw)),
            ),
        }
    }
    let listed: Vec<&String> = view
        .presentation
        .slides
        .iter()
        .map(|e| &e.raw)
        .filter(|id| counts.contains_key(id))
        .collect();
    let mut once = BTreeSet::new();
    read_order.retain(|id| once.insert(*id) && slide_ids.contains(id));
    if read_order != listed {
        add(
            found,
            "INV-SL-05",
            main,
            "reading the sections does not give the slide order",
        );
    }
}

fn rule_set_notes(view: &View, found: &mut Found) {
    let main = view.main.as_str();
    let p = &view.presentation;
    // INV-NM-01 (Documented).
    let on_list: BTreeSet<String> = p
        .notes_masters
        .iter()
        .filter_map(|id| view.target_of(main, id))
        .collect();
    if p.notes_masters.len() > 1 || view.parts_of_type(".notesmaster+xml").len() > 1 {
        add(found, "INV-NM-01", main, "more than one notes master");
    }
    let explicit: Vec<String> = view
        .of_kind(main, "notesMaster")
        .into_iter()
        .filter_map(|r| view.resolved(main, r))
        .collect();
    for master in &explicit {
        if !on_list.contains(master) {
            add(found, "INV-NM-01", master, "not on notesMasterIdLst");
        }
    }
    // INV-NM-03 (Documented for root and clrMap; placeholders Inferred; form Unverified).
    for (master, missing) in &view.notes_master_gaps {
        add(found, "INV-NM-03", master, *missing);
    }
    // INV-NM-04 (Documented).
    for notes in view.parts_of_type(".notesslide+xml") {
        if view.of_kind(&notes, "notesMaster").is_empty() {
            add(
                found,
                "INV-NM-04",
                &notes,
                "no relationship to the notes master",
            );
        }
        if view.of_kind(&notes, "slide").is_empty() {
            add(found, "INV-NM-04", &notes, "no relationship to its slide");
        }
    }
}

/// Rules that read one package.
pub(crate) fn state_rules(view: &View) -> Found {
    let mut found = Found::new();
    rule_set_names(view, &mut found);
    rule_set_rels(view, &mut found);
    rule_set_content_types(view, &mut found);
    rule_set_shared(view, &mut found);
    rule_set_ids(view, &mut found);
    rule_set_sections(view, &mut found);
    rule_set_notes(view, &mut found);
    super::bookkeeping::state_rules(view, &mut found);
    found
}

/// What one XML part holds that the rules below read.
#[derive(Debug, Default)]
struct Scan {
    rel_ids: BTreeSet<String>,
    shape_ids: BTreeSet<String>,
    repeated_shapes: BTreeSet<String>,
    timing_targets: BTreeSet<String>,
    repeated_authors: BTreeSet<String>,
    repeated_creation_ids: BTreeSet<String>,
}

fn scan_part(tree: &XmlTree) -> Scan {
    let mut scan = Scan::default();
    let mut authors = BTreeSet::new();
    let mut creation_ids = BTreeSet::new();
    // A SmartArt drawing's shapes are all id 0 in the corpus: the rule is the slide's.
    let holds_shapes = matches!(
        tree.local_name(tree.root()),
        "sld" | "sldLayout" | "sldMaster" | "notes" | "notesMaster" | "handoutMaster"
    );
    // Each element with whether it is in a fallback branch, which repeats ids by design.
    let mut stack: Vec<(ElementId, bool)> = vec![(tree.root(), false)];
    while let Some((el, fallback)) = stack.pop() {
        let local = tree.local_name(el);
        let fallback = fallback || local == "Fallback";
        let is_shape = holds_shapes && local == "cNvPr";
        let is_author = local == "cmAuthor";
        let is_creation_id = holds_shapes && local == "creationId" && !fallback;
        for (name, value) in tree.declared_attributes(el) {
            if let Some((prefix, _)) = name.split_once(':') {
                // An empty reference is how a SmartArt layout says it has no image yet.
                if !value.is_empty() && prefix_is_relationships(tree, el, prefix) {
                    scan.rel_ids.insert(value);
                }
                continue;
            }
            if name == "spid" {
                scan.timing_targets.insert(value);
            } else if name == "id" && is_shape && !fallback {
                if !scan.shape_ids.insert(value.clone()) {
                    scan.repeated_shapes.insert(value);
                }
            } else if name == "id" && is_shape {
                scan.shape_ids.insert(value);
            } else if name == "id" && is_creation_id {
                if !creation_ids.insert(value.clone()) {
                    scan.repeated_creation_ids.insert(value);
                }
            } else if name == "id" && is_author && !authors.insert(value.clone()) {
                scan.repeated_authors.insert(value);
            }
        }
        for child in tree.child_elements(el) {
            stack.push((child, fallback));
        }
    }
    scan
}

fn scan_rules(view: &View, part: &str, label: &str, scan: &Scan, found: &mut Found) {
    let rels = view.rels.get(part);
    // INV-RI-04 (Documented as an SDK rule; general form Inferred).
    for id in &scan.rel_ids {
        if !rels.is_some_and(|r| r.items.iter().any(|item| &item.id == id)) {
            add(
                found,
                "INV-RI-04",
                label,
                format!("{} has no relationship", safe_id(id)),
            );
        }
    }
    // INV-ID-06 (spid Documented as an SDK rule; uniqueness Inferred; prompt Unverified).
    for id in &scan.repeated_shapes {
        add(
            found,
            "INV-ID-06",
            label,
            format!("shape id {} repeated", safe_id(id)),
        );
    }
    for id in &scan.timing_targets {
        if !scan.shape_ids.contains(id) {
            add(
                found,
                "INV-ID-06",
                label,
                format!("spid {} names no shape", safe_id(id)),
            );
        }
    }
    for id in &scan.repeated_creation_ids {
        add(
            found,
            "INV-ID-06",
            label,
            format!("creationId {} repeated", safe_id(id)),
        );
    }
    // INV-ID-07 (Documented as an SDK rule).
    for id in &scan.repeated_authors {
        add(
            found,
            "INV-ID-07",
            label,
            format!("author id {} repeated", safe_id(id)),
        );
    }
}

/// The part a clone came from, followed to the input.
fn origin<'c>(clones: &'c BTreeMap<String, String>, part: &'c str) -> &'c str {
    let mut at = part;
    for _ in 0..16 {
        match clones.get(at) {
            Some(source) => at = source,
            None => break,
        }
    }
    at
}

/// Rules that read the bodies of parts this save wrote, on both sides.
fn written_part_rules(
    draft: &mut Draft<'_>,
    before: &View,
    after: &View,
    was: &mut Found,
    now: &mut Found,
) -> Result<(), WriterError> {
    let is_xml = |view: &View, name: &str| {
        !view.is_structural(name)
            && (name.to_ascii_lowercase().ends_with(".xml")
                || view
                    .content_type(name)
                    .is_some_and(|ct| ct.to_ascii_lowercase().ends_with("xml")))
    };
    let written: Vec<String> = draft
        .overlay_names()
        .filter(|n| is_xml(after, n))
        .cloned()
        .collect();
    let clones = draft.log.clone_of.clone();
    let mut looked_at = BTreeSet::new();
    for name in &written {
        // A body that does not parse was written by no patch in this crate.
        let bytes = draft.xml_bytes(name)?;
        let tree =
            XmlTree::parse(&bytes, name, &draft.xml).map_err(|_| WriterError::Preflight {
                violations: vec![Violation {
                    rule: "F4",
                    part: name.clone(),
                    detail: "a written part does not parse".to_owned(),
                }],
            })?;
        let label = origin(&clones, name);
        scan_rules(after, name, label, &scan_part(&tree), now);
        let source = label.to_owned();
        if draft.input_contains(&source) && looked_at.insert(source.clone()) {
            let bytes = draft.package.read_xml_part(&source)?;
            if let Ok(tree) = XmlTree::parse(&bytes, &source, &draft.xml) {
                scan_rules(before, &source, &source, &scan_part(&tree), was);
            }
        }
    }
    Ok(())
}

/// Referrers of each part in a package, by key, leaving out the main part's own
/// relationship to its theme, which follows the first master rather than editing it.
fn referrers(view: &View) -> BTreeMap<String, Vec<(String, String)>> {
    let mut out: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (source, facts) in &view.rels {
        for rel in &facts.items {
            if source == &view.main && rel_kind(&rel.rel_type) == "theme" {
                continue;
            }
            if let Some(target) = view.resolved(source, rel) {
                out.entry(key(&target))
                    .or_default()
                    .push((source.clone(), rel_kind(&rel.rel_type).to_owned()));
            }
        }
    }
    out
}

fn top_rel_number(items: &[Relationship]) -> u64 {
    items
        .iter()
        .filter_map(|r| r.id.strip_prefix("rId")?.parse::<u64>().ok())
        .max()
        .unwrap_or(0)
}

fn relationship_rules(draft: &Draft<'_>, before: &View, after: &View, found: &mut Found) {
    let strict = draft.conformance == Conformance::Strict;
    let log = &draft.log;
    for (source, facts) in &after.rels {
        let existed = source.is_empty() || before.names.contains_key(&key(source));
        let old = before.rels.get(source);
        let old_items = old.map(|o| o.items.as_slice()).unwrap_or(&[]);
        let is_clone = log.clone_of.contains_key(source);
        for rel in &facts.items {
            if old_items.iter().any(|o| o.id == rel.id) {
                continue;
            }
            // INV-RI-09 (Documented for comparison; written form Inferred).
            let form = if strict {
                relationship_type_to_strict(&rel.rel_type)
            } else {
                relationship_type_to_transitional(&rel.rel_type)
            };
            if !is_clone && form != rel.rel_type {
                add(
                    found,
                    "INV-RI-09",
                    source,
                    format!("{} is in the other form", safe_id(&rel.id)),
                );
            }
            if !existed || is_clone {
                continue;
            }
            // INV-RI-03 (Inferred).
            let top = top_rel_number(old_items);
            let good = rel
                .id
                .strip_prefix("rId")
                .and_then(|n| n.parse::<u64>().ok())
                .is_some_and(|n| n > top || top == u64::MAX);
            if !good {
                add(
                    found,
                    "INV-RI-03",
                    source,
                    format!("{} is not above the part's highest", safe_id(&rel.id)),
                );
            }
            // INV-RI-07 (Documented for relative; form Inferred).
            if rel.mode == TargetMode::Internal && !rel.target.starts_with('#') {
                let siblings: Vec<&Relationship> = old_items
                    .iter()
                    .filter(|o| o.mode == TargetMode::Internal && !o.target.starts_with('#'))
                    .collect();
                if !siblings.is_empty() {
                    let all_absolute = siblings.iter().all(|o| o.target.starts_with('/'));
                    let any_absolute = siblings.iter().any(|o| o.target.starts_with('/'));
                    let absolute = rel.target.starts_with('/');
                    if (all_absolute && !absolute) || (!any_absolute && absolute) {
                        add(
                            found,
                            "INV-RI-07",
                            source,
                            format!("{} is not in its siblings' form", safe_id(&rel.id)),
                        );
                    }
                }
            }
            // INV-RI-10 (Inferred).
            let same = facts.items.iter().any(|other| {
                other.id != rel.id
                    && other.rel_type == rel.rel_type
                    && other.mode == rel.mode
                    && after.resolved(source, other).is_some()
                    && after.resolved(source, other) == after.resolved(source, rel)
            });
            if same {
                add(
                    found,
                    "INV-RI-10",
                    source,
                    format!("{} repeats a relationship to one target", safe_id(&rel.id)),
                );
            }
        }
    }
    for (source, facts) in &before.rels {
        let alive = source.is_empty() || after.names.contains_key(&key(source));
        if !alive {
            continue;
        }
        let empty = Vec::new();
        let now = after.rels.get(source).map_or(&empty, |r| &r.items);
        for rel in &facts.items {
            let pair = (source.clone(), rel.id.clone());
            match now.iter().find(|n| n.id == rel.id) {
                None if log.removed_rels.contains(&pair) => {}
                None => {
                    // INV-RI-02 (Inferred): the same relationship under another id.
                    let renumbered = now.iter().any(|n| {
                        n.rel_type == rel.rel_type
                            && n.mode == rel.mode
                            && n.target == rel.target
                            && !facts.items.iter().any(|o| o.id == n.id)
                    });
                    if renumbered {
                        add(
                            found,
                            "INV-RI-02",
                            source,
                            format!("{} was renumbered", safe_id(&rel.id)),
                        );
                    } else {
                        // INV-RI-05 (Inferred): nothing was asked to drop it.
                        add(
                            found,
                            "INV-RI-05",
                            source,
                            format!("{} was dropped", safe_id(&rel.id)),
                        );
                    }
                }
                Some(n) => {
                    // INV-RI-07 (Inferred from F2): a retained target stays byte for byte.
                    if n.target != rel.target && !log.retargeted.contains(&pair) {
                        add(
                            found,
                            "INV-RI-07",
                            source,
                            format!("{} was rewritten", safe_id(&rel.id)),
                        );
                    }
                    // INV-RI-09.
                    if n.rel_type != rel.rel_type {
                        add(
                            found,
                            "INV-RI-09",
                            source,
                            format!("{} changed type", safe_id(&rel.id)),
                        );
                    }
                }
            }
        }
    }
}

fn name_rules(
    draft: &mut Draft<'_>,
    before: &View,
    after: &View,
    found: &mut Found,
) -> Result<(), WriterError> {
    let mut families = Families::default();
    for name in before.names.values() {
        families.note(name);
    }
    let added = draft.log.added.clone();
    let removed = draft.log.removed.clone();
    let clones = draft.log.clone_of.clone();
    let authored = draft.log.authored.clone();
    for name in &added {
        // INV-PN-02 (Documented reason; narrower set Inferred), INV-PE-02.
        if !name.split('/').all(is_mintable_segment) {
            add(
                found,
                "INV-PN-02",
                name,
                "a new name outside the allowed characters",
            );
        }
        // INV-PN-05 (Inferred): one above the highest, and never a removed name.
        if is_rels_part(name) {
            continue;
        }
        let k = key(name);
        let file = k.rsplit('/').next().unwrap_or(&k);
        let (_, number, _) = family_of(file);
        let reused = removed.contains(name);
        let below = number.is_some_and(|n| n <= families.highest_of(name));
        if reused || below {
            add(found, "INV-PN-05", name, "a number the family has used");
        }
    }
    // INV-PN-07: a removed part leaves nothing behind.
    for name in &removed {
        if after.names.contains_key(&key(name)) {
            continue;
        }
        if after.rels_entry(name).is_some() {
            add(found, "INV-PN-07", name, "its relationships part was left");
        }
    }
    // INV-PN-06 (Inferred from F2): a new part with the bytes of a removed one, which no
    // clone or operation accounts for, is the old part renamed.
    let gone: Vec<&String> = removed
        .iter()
        .filter(|n| !after.names.contains_key(&key(n)) && !is_rels_part(n))
        .collect();
    for name in added
        .iter()
        .filter(|n| !clones.contains_key(*n) && !authored.contains(*n) && !is_rels_part(n))
    {
        if gone.is_empty() {
            break;
        }
        let bytes = draft.part_bytes(name)?;
        for old in &gone {
            if draft.package.read_part(old)? == bytes {
                add(found, "INV-PN-06", old, "renamed");
            }
        }
    }
    Ok(())
}

fn copied_shape_rules(draft: &mut Draft<'_>, found: &mut Found) -> Result<(), WriterError> {
    // INV-ID-06: an animated copy's animation names the copy, not the original.
    let copies = draft.log.copied_shapes.clone();
    for copy in copies.iter().filter(|c| c.animated) {
        let bytes = draft.xml_bytes(&copy.part)?;
        let Ok(tree) = XmlTree::parse(&bytes, &copy.part, &draft.xml) else {
            continue;
        };
        let count = |id: &str| {
            super::ops::all_elements(&tree)
                .into_iter()
                .filter(|&el| {
                    tree.local_name(el) == "spTgt"
                        && tree.plain_attribute(el, "spid").as_deref() == Some(id)
                })
                .count()
        };
        if count(&copy.old) > 0 && count(&copy.new) == 0 {
            add(
                found,
                "INV-ID-06",
                &copy.part,
                format!("spid not remapped to {}", safe_id(&copy.new)),
            );
        }
    }
    Ok(())
}

fn content_type_rules(draft: &Draft<'_>, before: &View, after: &View, found: &mut Found) {
    // INV-CT-10 (Unverified): the main part's type agrees with the file asked for.
    if let Some(extension) = &draft.log.requested_extension
        && after.content_type(&after.main) != main_type_for(extension)
    {
        add(
            found,
            "INV-CT-10",
            &after.main,
            format!("its type is not that of .{}", safe_id(extension)),
        );
    }
    // INV-CT-08 (Inferred): the main type changes only when a format is asked for.
    if draft.log.requested_extension.is_none()
        && before.content_type(&before.main) != after.content_type(&after.main)
    {
        add(
            found,
            "INV-CT-08",
            &after.main,
            "its type changed with no format asked for",
        );
    }
    // INV-CT-06's exception: dropping VBA leaves no `Default` naming its type, unused.
    if draft.log.vba_dropped {
        for (ext, ct) in &after.types.defaults {
            let used = after.names.keys().any(|k| {
                k.rsplit('/')
                    .next()
                    .and_then(|f| f.rsplit_once('.'))
                    .is_some_and(|(_, e)| e.eq_ignore_ascii_case(ext))
            });
            if ct.eq_ignore_ascii_case("application/vnd.ms-office.vbaproject") && !used {
                add(
                    found,
                    "INV-CT-06",
                    "[Content_Types].xml",
                    format!("Default {} still names VBA", safe_id(ext)),
                );
            }
        }
    }
    // INV-CT-03 (Documented), INV-CT-04 (Inferred).
    let overridden: BTreeSet<String> = after.types.overrides.iter().map(|(n, _)| key(n)).collect();
    for (part, wanted) in &draft.log.expected_types {
        if !after.names.contains_key(&key(part)) {
            continue;
        }
        let got = after.content_type(part);
        if !got.is_some_and(|g| g.eq_ignore_ascii_case(wanted)) {
            add(
                found,
                "INV-CT-03",
                part,
                "does not resolve to the type it was added with",
            );
        }
        if wanted.to_ascii_lowercase().ends_with("+xml") && !overridden.contains(&key(part)) {
            add(found, "INV-CT-04", part, "an XML part relies on a Default");
        }
    }
    // INV-CT-06 (Documented that unused is allowed; the macro exception Inferred, and
    // not yet exercised: dropping VBA is T1N-06), INV-CT-09 (Inferred).
    for (ext, ct) in &before.types.defaults {
        if after
            .types
            .defaults
            .iter()
            .any(|(e, _)| e.eq_ignore_ascii_case(ext))
        {
            continue;
        }
        if ext.eq_ignore_ascii_case("rels") {
            add(
                found,
                "INV-CT-09",
                "[Content_Types].xml",
                "the rels Default was removed",
            );
        } else if !ct.to_ascii_lowercase().contains("vbaproject") {
            add(
                found,
                "INV-CT-06",
                "[Content_Types].xml",
                format!("Default {} was removed", safe_id(ext)),
            );
        }
    }
    // INV-CT-08 (Inferred): no other type changes.
    for (name, ct) in &before.types.overrides {
        let k = key(name);
        if k == key(&before.main) {
            continue;
        }
        if let Some((_, new)) = after.types.overrides.iter().find(|(n, _)| key(n) == k)
            && !new.eq_ignore_ascii_case(ct)
        {
            add(found, "INV-CT-08", name, "its Override changed type");
        }
    }
}

fn shared_part_rules(
    draft: &mut Draft<'_>,
    before: &View,
    after: &View,
    found: &mut Found,
) -> Result<(), WriterError> {
    let removed = draft.log.removed.clone();
    let reach = draft.input_reach().clone();
    let orphans: Vec<String> = after
        .names
        .iter()
        .filter(|(k, n)| !reach.contains(*k) && !after.is_structural(n))
        .map(|(_, n)| n.clone())
        .collect();
    let mut roots = vec![String::new()];
    roots.extend(orphans);
    let kept = after.reach_of(&roots);
    let incoming = referrers(before);
    // What relationships still aim at, whether or not the part is there.
    let aimed_at: BTreeSet<String> = after
        .rels
        .iter()
        .flat_map(|(source, facts)| {
            facts.items.iter().filter_map(move |rel| {
                if rel.mode != TargetMode::Internal || rel.target.starts_with('#') {
                    return None;
                }
                part_name_key(&resolve_internal(source, &rel.target)?)
            })
        })
        .collect();
    for name in &removed {
        if after.names.contains_key(&key(name)) || is_rels_part(name) {
            continue;
        }
        // INV-SP-01 (Inferred): what arrived orphaned stays.
        if !reach.contains(&key(name)) {
            add(
                found,
                "INV-SP-01",
                name,
                "a part that arrived orphaned was removed",
            );
        }
        // INV-SP-02 (Inferred walk): media goes with its last incoming relationship.
        let is_media_part = incoming
            .get(&key(name))
            .is_some_and(|sources| sources.iter().any(|(_, kind)| is_media(kind)));
        if is_media_part && aimed_at.contains(&key(name)) {
            add(
                found,
                "INV-SP-02",
                name,
                "media removed while something still points to it",
            );
        }
    }
    // INV-SP-01: what an edit left unreachable goes. INV-SP-09 (Inferred) names a
    // deleted slide's notes and comments.
    let gone_slides: Vec<&String> = removed
        .iter()
        .filter(|n| !after.names.contains_key(&key(n)))
        .collect();
    for (k, name) in &after.names {
        if reach.contains(k) && !kept.contains(k) && !after.is_structural(name) {
            let belongs_to_slide = gone_slides.iter().any(|slide| {
                before.rels.get(*slide).is_some_and(|facts| {
                    facts.items.iter().any(|rel| {
                        let kind = rel_kind(&rel.rel_type);
                        (kind == "notesSlide" || kind == "comments" || kind.ends_with("/comments"))
                            && before.resolved(slide, rel).is_some_and(|t| key(&t) == *k)
                    })
                })
            });
            let rule = if belongs_to_slide {
                "INV-SP-09"
            } else {
                "INV-SP-01"
            };
            add(found, rule, name, "left unreachable by this save");
        }
    }
    // INV-SP-03 (Inferred): new media with the bytes of existing media.
    let new_media: Vec<String> = draft
        .log
        .added
        .iter()
        .filter(|n| key(n).starts_with("ppt/media/") && !draft.log.clone_of.contains_key(*n))
        .cloned()
        .collect();
    if !new_media.is_empty() {
        let media: Vec<String> = after
            .names
            .iter()
            .filter(|(k, _)| k.starts_with("ppt/media/"))
            .map(|(_, n)| n.clone())
            .collect();
        for new in &new_media {
            let bytes = draft.part_bytes(new)?;
            for other in &media {
                if key(other) != key(new) && draft.part_bytes(other)? == bytes {
                    add(found, "INV-SP-03", new, "the same bytes as a media part");
                }
            }
        }
    }
    // INV-SP-04 (Inferred): a part with more than one referrer is not edited in place.
    let edited: Vec<String> = draft
        .overlay_names()
        .filter(|n| draft.input_contains(n) && !after.is_structural(n))
        .cloned()
        .collect();
    for name in edited {
        let Some(sources) = incoming.get(&key(&name)) else {
            continue;
        };
        let shared_kind = sources.iter().any(|(_, kind)| {
            is_media(kind)
                || matches!(
                    kind.as_str(),
                    "theme" | "chart" | "diagramData" | "oleObject" | "package"
                )
        });
        let distinct: BTreeSet<&String> = sources.iter().map(|(s, _)| s).collect();
        if shared_kind && distinct.len() > 1 {
            add(
                found,
                "INV-SP-04",
                &name,
                "edited in place with more than one referrer",
            );
        }
    }
    // INV-SP-08 (Unverified): the presentation's theme follows the first master's when
    // this save changes which theme that is.
    let main = after.main.clone();
    let first_theme = |view: &View| {
        view.masters
            .first()
            .and_then(|m| view.first_target(&m.part, "theme"))
    };
    let first_was = first_theme(before);
    let first_now = first_theme(after);
    if first_was != first_now
        && let (Some(theme), Some(mine)) = (&first_now, after.first_target(&main, "theme"))
        && key(theme) != key(&mine)
    {
        add(
            found,
            "INV-SP-08",
            &main,
            "does not follow the first master's theme",
        );
    }
    Ok(())
}

/// INV-SP-10 (Inferred): a slide's clone shares what it shares and owns what it owns,
/// with the same relationship ids, so a `relId` in a cloned part still resolves.
fn clone_rules(
    draft: &mut Draft<'_>,
    before: &View,
    after: &View,
    found: &mut Found,
) -> Result<(), WriterError> {
    let clones = draft.log.clone_of.clone();
    let main = after.main.clone();
    let slides: BTreeSet<String> = after
        .of_kind(&main, "slide")
        .into_iter()
        .filter_map(|r| after.resolved(&main, r))
        .collect();
    for (clone, source) in &clones {
        if !slides.contains(clone) {
            continue;
        }
        let Some(old) = before.rels.get(source) else {
            continue;
        };
        let empty = Vec::new();
        let new = after.rels.get(clone).map_or(&empty, |f| &f.items);
        for rel in &old.items {
            let pair = (clone.clone(), rel.id.clone());
            if draft.log.removed_rels.contains(&pair) || draft.log.retargeted.contains(&pair) {
                // A later operation changed it on purpose: a link to a deleted slide.
                continue;
            }
            let Some(now) = new.iter().find(|n| n.id == rel.id) else {
                add(
                    found,
                    "INV-SP-10",
                    clone,
                    format!("{} was not carried over", safe_id(&rel.id)),
                );
                continue;
            };
            let kind = rel_kind(&rel.rel_type);
            let (Some(from), Some(to)) = (before.resolved(source, rel), after.resolved(clone, now))
            else {
                continue;
            };
            let same = key(&from) == key(&to);
            let shared_by_rule = is_media(kind)
                || matches!(
                    kind,
                    "slideLayout" | "slide" | "hyperlink" | "theme" | "notesMaster" | "slideMaster"
                );
            if same != shared_by_rule && from != *source {
                add(
                    found,
                    "INV-SP-10",
                    clone,
                    format!("{} breaks the sharing rule", safe_id(&rel.id)),
                );
            }
        }
        // `relId` in a cloned data part names a relationship of the slide, and must land
        // on the clone's own drawing (INV-RI-02's reading).
        for rel in new {
            if rel_kind(&rel.rel_type) != "diagramData" {
                continue;
            }
            let Some(data) = after.resolved(clone, rel) else {
                continue;
            };
            let bytes = draft.xml_bytes(&data)?;
            let Ok(tree) = XmlTree::parse(&bytes, &data, &draft.xml) else {
                continue;
            };
            for el in super::ops::all_elements(&tree) {
                if tree.local_name(el) != "dataModelExt" {
                    continue;
                }
                let Some(id) = tree.plain_attribute(el, "relId") else {
                    continue;
                };
                let source_has = before
                    .rels
                    .get(source)
                    .is_some_and(|f| f.items.iter().any(|r| r.id == id));
                if !source_has {
                    continue;
                }
                let lands = after.target_of(clone, &id);
                if !lands.as_ref().is_some_and(|t| clones.contains_key(t)) {
                    add(
                        found,
                        "INV-SP-10",
                        clone,
                        format!(
                            "relId {} does not land on the clone's drawing",
                            safe_id(&id)
                        ),
                    );
                }
            }
        }
    }
    Ok(())
}

fn id_rules(draft: &Draft<'_>, before: &View, after: &View, found: &mut Found) {
    let main = after.main.as_str();
    let (b, a) = (&before.presentation, &after.presentation);
    // A model handed to the plan may renumber on purpose, and then sections and zooms
    // are what stop it (INV-SL-04). What these two judge is what the writer allocates.
    if draft.log.model_edited_main {
        return;
    }
    // INV-ID-02 (Inferred): an existing id never changes, and a new one is above the top.
    let now: BTreeSet<(&String, &String)> = a.slides.iter().map(|e| (&e.rel_id, &e.raw)).collect();
    for entry in &b.slides {
        let moved = a.slides.iter().any(|n| n.rel_id == entry.rel_id)
            && !now.contains(&(&entry.rel_id, &entry.raw));
        if moved {
            add(
                found,
                "INV-ID-02",
                main,
                format!("slide id {} changed", id_detail(&entry.raw)),
            );
        }
    }
    let was: BTreeSet<&String> = b.slides.iter().map(|e| &e.raw).collect();
    let top = b
        .slides
        .iter()
        .filter_map(|e| e.id)
        .filter(|n| *n <= u64::from(SLIDE_ID_MAX))
        .max()
        .unwrap_or(0);
    for entry in &a.slides {
        if was.contains(&entry.raw) {
            continue;
        }
        let good = entry
            .id
            .is_some_and(|n| n > top || top == u64::from(SLIDE_ID_MAX));
        if !good {
            add(
                found,
                "INV-ID-02",
                main,
                format!(
                    "new slide id {} is not above the top",
                    id_detail(&entry.raw)
                ),
            );
        }
    }
    // INV-ID-05 (Documented): a new custom show's id is consecutive.
    let shows = |view: &View| -> Vec<u64> {
        view.presentation
            .custom_shows
            .iter()
            .filter_map(|s| s.id.parse().ok())
            .collect()
    };
    let old_shows = shows(before);
    let mut next = old_shows.iter().max().map_or(0, |t| t.saturating_add(1));
    let mut new_shows: Vec<u64> = shows(after)
        .into_iter()
        .filter(|id| !old_shows.contains(id))
        .collect();
    new_shows.sort_unstable();
    for id in new_shows {
        if id != next {
            add(
                found,
                "INV-ID-05",
                main,
                format!("custom show id {id} is not consecutive"),
            );
        }
        next = id.saturating_add(1);
    }
    // INV-ID-04 (Inferred), the same for the shared pool.
    let pool = |view: &View| -> BTreeSet<u64> {
        view.presentation
            .masters
            .iter()
            .chain(view.masters.iter().flat_map(|m| &m.layouts))
            .filter_map(|e| e.id)
            .collect()
    };
    let was = pool(before);
    let top = was
        .iter()
        .copied()
        .filter(|n| *n <= POOL_MAX)
        .max()
        .unwrap_or(0);
    for id in pool(after) {
        if !was.contains(&id) && id <= top && top != POOL_MAX {
            add(
                found,
                "INV-ID-04",
                main,
                format!("new id {id} is not above the top"),
            );
        }
        if !was.contains(&id) && id > POOL_MAX {
            add(
                found,
                "INV-ID-04",
                main,
                format!("new id {id} wrapped past the cap"),
            );
        }
    }
}

fn section_rules(draft: &Draft<'_>, before: &View, after: &View, found: &mut Found) {
    let main = after.main.as_str();
    let log = &draft.log;
    let (was, now) = (
        before.presentation.sections.as_ref(),
        after.presentation.sections.as_ref(),
    );
    // INV-SL-01 (Documented): a deck with none gets none.
    if was.is_none() && now.is_some() && !log.section_list_requested {
        add(found, "INV-SL-01", main, "a section list was minted");
    }
    // INV-SL-03 (Documented): a section may be empty, so emptying one keeps it.
    if let Some(was) = was {
        for section in was {
            let kept = now.is_some_and(|n| n.iter().any(|s| s.id == section.id));
            if !kept && !log.removed_sections.contains(&section.id) {
                add(
                    found,
                    "INV-SL-03",
                    main,
                    "a section was removed without being asked",
                );
            }
        }
    }
    // INV-SL-06 (Inferred): a new slide joins the section of the slide before it. Where
    // that is was settled when it was placed; later operations may change who is before it.
    if let Some(now) = now {
        for (slide, wanted) in &log.section_of_new {
            let holder = now.iter().find(|s| s.slides.contains(slide));
            if holder.is_some_and(|s| &s.id != wanted) {
                add(
                    found,
                    "INV-SL-06",
                    main,
                    format!("slide {} is not with the one before it", id_detail(slide)),
                );
            }
        }
    }
}

fn notes_rules(draft: &Draft<'_>, before: &View, after: &View, found: &mut Found) {
    let main = after.main.as_str();
    let (b, a) = (&before.presentation, &after.presentation);
    // INV-NM-05 (Inferred): the master stays, and no handout master is minted.
    if !b.notes_masters.is_empty() && a.notes_masters.is_empty() {
        add(found, "INV-NM-05", main, "the notes master was removed");
    }
    if b.handout_masters.is_empty() && !a.handout_masters.is_empty() && !draft.log.model_edited_main
    {
        add(found, "INV-NM-05", main, "a handout master was minted");
    }
    let had = !before.parts_of_type(".notesmaster+xml").is_empty();
    // INV-NM-02 (Inferred): the first notes bring all of the master in one change.
    for master in after.parts_of_type(".notesmaster+xml") {
        if had {
            continue;
        }
        let theme = after.first_target(&master, "theme");
        match &theme {
            None => add(found, "INV-NM-02", &master, "no theme"),
            Some(theme) => {
                let other_users = after.rels.iter().any(|(source, facts)| {
                    source != &master
                        && source != &after.main
                        && facts.items.iter().any(|r| {
                            after
                                .resolved(source, r)
                                .is_some_and(|t| key(&t) == key(theme))
                        })
                });
                if other_users {
                    add(found, "INV-NM-02", &master, "shares its theme");
                }
            }
        }
        let overridden = |part: &str| {
            after
                .types
                .overrides
                .iter()
                .any(|(n, _)| key(n) == key(part))
        };
        if !overridden(&master) || !theme.as_deref().is_some_and(overridden) {
            add(found, "INV-NM-02", &master, "an Override is missing");
        }
        if !after.of_kind(main, "notesMaster").iter().any(|r| {
            after
                .resolved(main, r)
                .is_some_and(|t| key(&t) == key(&master))
        }) || a.notes_masters.is_empty()
        {
            add(
                found,
                "INV-NM-02",
                &master,
                "the presentation does not list it",
            );
        }
    }
}

/// Rules that compare the input and the draft.
fn change_rules(draft: &mut Draft<'_>, before: &View, after: &View) -> Result<Found, WriterError> {
    let mut found = Found::new();
    name_rules(draft, before, after, &mut found)?;
    relationship_rules(draft, before, after, &mut found);
    content_type_rules(draft, before, after, &mut found);
    copied_shape_rules(draft, &mut found)?;
    shared_part_rules(draft, before, after, &mut found)?;
    clone_rules(draft, before, after, &mut found)?;
    id_rules(draft, before, after, &mut found);
    section_rules(draft, before, after, &mut found);
    notes_rules(draft, before, after, &mut found);
    super::bookkeeping::change_rules(draft, before, after, &mut found);
    super::bookkeeping::retype_rules(draft, before, after, &mut found);
    super::format::macro_action_rules(draft, &mut found)?;
    Ok(found)
}

/// Judge the draft against the input.
pub(crate) fn run(
    draft: &mut Draft<'_>,
    before: &View,
    after: &View,
) -> Result<Preflight, WriterError> {
    let mut was = state_rules(before);
    let mut now = Found::new();
    let clones = draft.log.clone_of.clone();
    // A violation on a clone is its source's, so a deck is not refused for copying
    // what it arrived with.
    for mut violation in state_rules(after) {
        violation.part = origin(&clones, &violation.part).to_owned();
        now.insert(violation);
    }
    written_part_rules(draft, before, after, &mut was, &mut now)?;
    let changes = change_rules(draft, before, after)?;
    let mut introduced: Vec<Violation> = now.difference(&was).cloned().collect();
    introduced.extend(changes);
    introduced.sort();
    introduced.dedup();
    if let Some(app) = &after.app
        && draft.model_owns_app(&app.part)
    {
        introduced.retain(|v| !APP_COUNT_RULES.contains(&v.rule));
    }
    if !introduced.is_empty() {
        return Err(WriterError::Preflight {
            violations: introduced,
        });
    }
    let arrived = now.intersection(&was).cloned().collect();
    Ok(Preflight { arrived })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_part_name_breaking_opc_is_caught() {
        for bad in ["ppt/slide.", "ppt//a.xml", "ppt/a%2Fb.xml", "ppt/%41.xml"] {
            assert!(!name_is_well_formed(bad), "{bad}");
        }
        for good in [
            "ppt/slides/slide1.xml",
            "ppt/media/a%20b.png",
            "_rels/.rels",
        ] {
            assert!(name_is_well_formed(good), "{good}");
        }
    }

    #[test]
    fn a_section_id_is_a_brace_wrapped_upper_case_guid() {
        assert!(is_section_guid("{7A1BB3C4-1111-4222-8333-AABBCCDDEEFF}"));
        assert!(!is_section_guid("{7a1bb3c4-1111-4222-8333-aabbccddeeff}"));
        assert!(!is_section_guid("7A1BB3C4-1111-4222-8333-AABBCCDDEEFF"));
    }

    #[test]
    fn an_xsd_id_does_not_start_with_a_digit() {
        assert!(is_xsd_id("rId1"));
        assert!(!is_xsd_id("1a"));
        assert!(!is_xsd_id(""));
    }

    #[test]
    fn details_never_carry_file_text() {
        assert_eq!(id_detail("secret words"), "not a number");
        assert_eq!(safe_id("a<b c>"), "abc");
    }
}
