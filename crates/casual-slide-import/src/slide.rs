//! Slides, layouts, masters, notes slides and the notes and handout masters
//! (Tier 1D): the deck's structure from relationships alone, each part read from
//! its [`XmlTree`] when it is first asked for, and the limits that bound both.
//!
//! [`index_deck`] reads no slide: it follows relationships the package already
//! holds, so a 500-slide deck is indexed for the cost of its `.rels`. The chain is
//! walked in a fixed order and to a fixed length (slide, layout, master), and each
//! step must reach a part of the kind it names, so a relationship cycle costs
//! nothing and loops nowhere (`SEC-X07`). [`LazySlides`] then reads a part on first
//! use; [`import_slide_parts`] reads them all.
//!
//! Absent stays absent, and a part that cannot be read is reported and left out.

use std::collections::{HashMap, HashSet};
use std::fmt;

use casual_slide_model::presentation::Presentation;
use casual_slide_model::shape::ObjectId;
use casual_slide_model::slide::{
    Background, BackgroundFill, BackgroundProperties, CommonSlideData, DeckStructure, HeaderFooter,
    MasterTextStyles, PartEntry, SLIDE_LAYOUT_ID_MIN, SlideKind, SlideLayoutId, SlidePart,
    SlideParts,
};
use casual_slide_ooxml::{
    OoxmlError, PresentationPackage, Relationship, TargetMode, XmlTree, part_name_key,
    resolve_compatibility, resolve_internal,
};

use crate::chart::ChartLimits;
use crate::color::Findings as ColorFindings;
use crate::drawing::DrawingLimits;
use crate::presentation::is_optional_part_fault;
use crate::properties::{
    Elem, Findings, ImportError, ImportFindingKind as K, ImportLimits, Reader, ns,
};
use crate::shape::{ShapeReader, SourceMap};
use crate::table::{
    TableBound, TableCounts, TableLimits, check_style_references, read_deck_table_styles,
};
use crate::text::{TextCounts, TextReader};
use crate::theme::{read_color_map, read_color_map_override, read_style_matrix_ref};
use crate::timing::{MotionReader, TimingLimits, branch_index, check_references};

const TRANSITIONAL_RELS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
const STRICT_RELS: &str = "http://purl.oclc.org/ooxml/officeDocument/relationships/";

/// Bounds on what slide-like parts can make this reader hold. Over a bound, the
/// part, or for the structure the deck, is refused with the bound's code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlideLimits {
    /// Shape-tree nodes in one part, the tree's root included. `CS-SLD-0001`.
    pub max_shapes_per_part: usize,
    /// Groups nested in groups, below the tree's root. `CS-SLD-0002`.
    pub max_group_depth: usize,
    /// Slides in the deck's structure. `CS-SLD-0003`.
    pub max_slides: usize,
    /// Slide-like parts of every kind in the deck's structure. `CS-SLD-0004`.
    pub max_parts: usize,
    /// Entries in one master's `sldLayoutIdLst`. `CS-SLD-0004`.
    pub max_layout_ids: usize,
    /// Bounds on tables and table styles.
    pub table: TableLimits,
    /// Bounds on chart parts.
    pub chart: ChartLimits,
    /// Bounds on one drawing property of a shape or a run. `CS-DRW-0001`.
    pub drawing: DrawingLimits,
    /// Bounds on a part's timing and transition. `CS-TMG-0001`.
    pub timing: TimingLimits,
}

impl Default for SlideLimits {
    fn default() -> Self {
        Self {
            max_shapes_per_part: 100_000,
            max_group_depth: 64,
            max_slides: 100_000,
            max_parts: 250_000,
            max_layout_ids: 10_000,
            table: TableLimits::default(),
            chart: ChartLimits::default(),
            drawing: DrawingLimits::default(),
            timing: TimingLimits::default(),
        }
    }
}

/// Why a slide-like part, or the deck's structure, could not be read. Names parts
/// and counts, never content.
#[derive(Debug)]
#[non_exhaustive]
pub enum SlideError {
    /// The XML, or a shared import bound, refused.
    Import(ImportError),
    /// More shape-tree nodes than [`SlideLimits::max_shapes_per_part`].
    TooManyShapes {
        /// The part.
        part: String,
        /// The bound.
        limit: usize,
    },
    /// Groups nested deeper than [`SlideLimits::max_group_depth`].
    GroupTooDeep {
        /// The part.
        part: String,
        /// The bound.
        limit: usize,
    },
    /// More slides than [`SlideLimits::max_slides`].
    TooManySlides {
        /// The bound.
        limit: usize,
    },
    /// More slide-like parts, or layout ids, than allowed.
    TooManyParts {
        /// What was counted.
        what: &'static str,
        /// The bound.
        limit: usize,
    },
    /// No part has this ordinal in the deck's structure.
    NoSuchPart {
        /// The ordinal asked for.
        ordinal: u32,
    },
    /// More of something in a table, or in the table style list, than
    /// [`TableLimits`] allows.
    Table {
        /// Which bound.
        bound: TableBound,
        /// The part.
        part: String,
        /// The bound's value.
        limit: usize,
    },
}

impl SlideError {
    /// The stable diagnostic code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Import(err) => err.code(),
            Self::TooManyShapes { .. } => "CS-SLD-0001",
            Self::GroupTooDeep { .. } => "CS-SLD-0002",
            Self::TooManySlides { .. } => "CS-SLD-0003",
            Self::TooManyParts { .. } => "CS-SLD-0004",
            Self::NoSuchPart { .. } => "CS-SLD-0005",
            Self::Table { bound, .. } => bound.code(),
        }
    }
}

impl fmt::Display for SlideError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::Import(err) => write!(f, "{err}"),
            Self::TooManyShapes { part, limit } => {
                write!(f, "[{code}] {part:?}: more than {limit} shapes")
            }
            Self::GroupTooDeep { part, limit } => {
                write!(f, "[{code}] {part:?}: groups nested deeper than {limit}")
            }
            Self::TooManySlides { limit } => write!(f, "[{code}] more than {limit} slides"),
            Self::TooManyParts { what, limit } => {
                write!(f, "[{code}] {what} over the limit {limit}")
            }
            Self::NoSuchPart { ordinal } => write!(f, "[{code}] no part {ordinal}"),
            Self::Table { bound, part, limit } => {
                write!(
                    f,
                    "[{code}] {part:?}: {} over the limit {limit}",
                    bound.what()
                )
            }
        }
    }
}

impl std::error::Error for SlideError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Import(err) => Some(err),
            _ => None,
        }
    }
}

impl From<ImportError> for SlideError {
    fn from(err: ImportError) -> Self {
        Self::Import(err)
    }
}

impl From<OoxmlError> for SlideError {
    fn from(err: OoxmlError) -> Self {
        Self::Import(ImportError::Ooxml(err))
    }
}

/// Whether a failure belongs to one part rather than to the deck: a malformed or
/// wrongly rooted part. A bound or a hostile shape never is.
pub fn is_part_fault(err: &SlideError) -> bool {
    matches!(err, SlideError::Import(e) if is_optional_part_fault(e))
}

fn is_too_deep(err: &SlideError) -> bool {
    matches!(
        err,
        SlideError::Import(ImportError::Ooxml(OoxmlError::TooDeep { .. }))
    )
}

/// One slide-like part read, with what was noticed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedPart {
    /// The model.
    pub value: SlidePart,
    /// The report.
    pub findings: Findings,
    /// What the colour map readers noticed.
    pub color_findings: ColorFindings,
    /// The part arrived damaged: what is here is what could be read, and an empty part if
    /// nothing could (`REC-04`). Its original bytes are retained.
    pub damaged: bool,
}

fn finish_children(r: &mut Reader<'_>, el: &Elem) -> Result<(), SlideError> {
    for child in r.children(el)? {
        r.retained(&child);
    }
    Ok(())
}

/// Read one slide-like part. `ordinal` is the part's place in the deck's
/// structure, which its ids carry; `rels` are its relationships.
#[allow(clippy::too_many_arguments)]
pub fn import_slide_part(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    kind: SlideKind,
    ordinal: u32,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
    findings: &mut Findings,
    color: &mut ColorFindings,
) -> Result<SlidePart, SlideError> {
    read_part(
        tree,
        part,
        rels,
        kind,
        ordinal,
        limits,
        import_limits,
        findings,
        color,
        None,
    )
}

/// As [`import_slide_part`], also saying where each node came from, for an export
/// that patches the tree in place. Reported findings are discarded.
#[allow(clippy::too_many_arguments)]
pub fn import_slide_part_mapped(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    kind: SlideKind,
    ordinal: u32,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
) -> Result<(SlidePart, Elem, SourceMap), SlideError> {
    let mut findings = Findings::new(import_limits.max_findings);
    let mut color = ColorFindings::default();
    let mut map = SourceMap::default();
    let value = read_part(
        tree,
        part,
        rels,
        kind,
        ordinal,
        limits,
        import_limits,
        &mut findings,
        &mut color,
        Some(&mut map),
    )?;
    let root = Elem::root(tree, part)?;
    Ok((value, root, map))
}

#[allow(clippy::too_many_arguments)]
// spec-audit: owner=sld,sldLayout,sldMaster,notes,notesMaster,handoutMaster
// `kind.root()` names these: p:<sld> p:<sldLayout> p:<sldMaster> p:<notes> p:<notesMaster> p:<handoutMaster>
// spec-audit: p:<sld>
// spec-audit: p:<sldLayout>
// spec-audit: p:<sldMaster>
// spec-audit: p:<notes>
// spec-audit: p:<notesMaster>
// spec-audit: p:<handoutMaster>
fn read_part(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    kind: SlideKind,
    ordinal: u32,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
    findings: &mut Findings,
    color: &mut ColorFindings,
    mut map: Option<&mut SourceMap>,
) -> Result<SlidePart, SlideError> {
    let compatibility = resolve_compatibility(tree);
    let mut reader = Reader::new(tree, part, import_limits, findings);
    let r = &mut reader;
    let mut root = r.root(ns::PML, kind.root())?;
    let mut p = SlidePart::new(ObjectId::part(ordinal), kind);
    if matches!(
        kind,
        SlideKind::Slide | SlideKind::Layout | SlideKind::Notes
    ) {
        p.show_master_shapes = r.attr(&mut root, "showMasterSp")?;
        p.show_master_placeholder_animations = r.attr(&mut root, "showMasterPhAnim")?;
    }
    match kind {
        SlideKind::Slide => p.show = r.attr(&mut root, "show")?,
        SlideKind::Layout => {
            p.matching_name = r.attr(&mut root, "matchingName")?;
            p.layout_type = r.attr(&mut root, "type")?;
            p.preserve = r.attr(&mut root, "preserve")?;
            p.user_drawn = r.attr(&mut root, "userDrawn")?;
        }
        SlideKind::Master => p.preserve = r.attr(&mut root, "preserve")?,
        _ => {}
    }
    r.finish(&root);
    let mut seen = HashSet::new();
    let has_motion = matches!(
        kind,
        SlideKind::Slide | SlideKind::Layout | SlideKind::Master
    );
    let branches = branch_index(&compatibility.alternates);
    for el in r.children(&root)? {
        if has_motion
            && MotionReader::new(&mut *r, rels, &mut *color, limits.timing, &branches)
                .root_child(&el, &mut p)
        {
            continue;
        }
        if !el.in_namespace(ns::PML) {
            r.retained(&el);
            continue;
        }
        let name = el.local.as_str();
        let modelled = match name {
            "cSld" => true,
            "clrMap" => kind.is_top_level(),
            "clrMapOvr" => !kind.is_top_level(),
            "hf" => matches!(
                kind,
                SlideKind::Layout
                    | SlideKind::Master
                    | SlideKind::NotesMaster
                    | SlideKind::HandoutMaster
            ),
            "sldLayoutIdLst" | "txStyles" => kind == SlideKind::Master,
            "notesStyle" => kind == SlideKind::NotesMaster,
            _ => false,
        };
        if !modelled {
            // `extLst`: Tier 1M.
            r.retained(&el);
            continue;
        }
        if !seen.insert(name.to_owned()) {
            r.note(K::DuplicateElement, &el, None);
            continue;
        }
        match name {
            "cSld" => {
                let mut shapes = ShapeReader {
                    r: &mut *r,
                    rels,
                    limits,
                    part_ordinal: ordinal,
                    alternates: &compatibility.alternates,
                    cursor: 0,
                    next_node: 0,
                    spids: HashSet::new(),
                    map: map.as_deref_mut(),
                    color: &mut *color,
                    text_counts: TextCounts::default(),
                    table_counts: TableCounts::default(),
                };
                p.common = Some(read_common(&mut shapes, &el)?);
            }
            "clrMap" => p.color_map = Some(read_color_map(tree, el.id, color)),
            "clrMapOvr" => p.color_map_override = read_color_map_override(tree, el.id, color),
            "hf" => p.header_footer = Some(read_header_footer(r, &el)?),
            "sldLayoutIdLst" => p.layout_ids = Some(read_layout_ids(r, &el, rels, limits)?),
            "txStyles" => {
                p.text_styles = Some(read_text_styles(r, &el, rels, color, limits.drawing)?)
            }
            _ => {
                let mut text = TextReader {
                    r: &mut *r,
                    rels,
                    color: &mut *color,
                    counts: &mut TextCounts::default(),
                    drawing: limits.drawing,
                };
                p.notes_style = Some(text.list_style(&el)?);
            }
        }
    }
    if p.common.is_none() {
        r.note(K::MissingRequired, &root, None);
    }
    if let Some(timing) = &mut p.timing {
        check_references(r, timing);
    }
    Ok(p)
}

// spec-audit: owner=cSld
fn read_common(
    shapes: &mut ShapeReader<'_, '_, '_>,
    el: &Elem,
) -> Result<CommonSlideData, SlideError> {
    let mut el = el.clone();
    let name = shapes.r.attr(&mut el, "name")?;
    shapes.r.finish(&el);
    let mut common = CommonSlideData {
        name,
        background: None,
        shape_tree: None,
        controls: Vec::new(),
    };
    let mut seen = HashSet::new();
    for child in shapes.r.children(&el)? {
        if child.is(ns::PML, "controls") && common.controls.is_empty() {
            common.controls = crate::retained::read_controls(shapes.r, shapes.rels, &child)?;
            continue;
        }
        let known = child.in_namespace(ns::PML) && matches!(child.local.as_str(), "bg" | "spTree");
        if !known {
            // `custDataLst`, `extLst`: Tier 1M.
            shapes.r.retained(&child);
            continue;
        }
        if !seen.insert(child.local.clone()) {
            shapes.r.note(K::DuplicateElement, &child, None);
            continue;
        }
        if child.local == "bg" {
            common.background = Some(read_background(shapes.r, &child, shapes.color)?);
        } else {
            common.shape_tree = Some(shapes.tree(&child)?);
        }
    }
    if common.shape_tree.is_none() {
        shapes.r.note(K::MissingRequired, &el, Some("spTree"));
    }
    Ok(common)
}

const FILLS: &[&str] = &[
    "noFill",
    "solidFill",
    "gradFill",
    "blipFill",
    "pattFill",
    "grpFill",
];

// spec-audit: owner=bg
fn read_background(
    r: &mut Reader<'_>,
    el: &Elem,
    color: &mut ColorFindings,
) -> Result<Background, SlideError> {
    let mut el = el.clone();
    let mut bg = Background {
        bw_mode: r.attr(&mut el, "bwMode")?,
        fill: None,
    };
    r.finish(&el);
    for child in r.children(&el)? {
        let choice =
            child.in_namespace(ns::PML) && matches!(child.local.as_str(), "bgPr" | "bgRef");
        if !choice {
            r.retained(&child);
            continue;
        }
        if bg.fill.is_some() {
            r.note(K::DuplicateElement, &child, None);
            continue;
        }
        if child.local == "bgRef" {
            // Its colour's findings keep their own codes.
            let reference = read_style_matrix_ref(r.tree, child.id, color);
            bg.fill = Some(BackgroundFill::Reference(reference));
            continue;
        }
        let mut pr = child.clone();
        let mut props = BackgroundProperties {
            shade_to_title: r.attr(&mut pr, "shadeToTitle")?,
            ..BackgroundProperties::default()
        };
        r.finish(&pr);
        for item in r.children(&pr)? {
            if item.in_namespace(ns::DML) {
                let name = item.local.as_str();
                props.has_fill |= FILLS.contains(&name);
                props.has_effects |= matches!(name, "effectLst" | "effectDag");
            }
            // The fill and effects themselves are Tier 1E.
            r.retained(&item);
        }
        bg.fill = Some(BackgroundFill::Properties(props));
    }
    if bg.fill.is_none() {
        r.note(K::MissingRequired, &el, Some("bgPr"));
    }
    Ok(bg)
}

/// `p:hf`.
// spec-audit: owner=hf
pub fn read_header_footer(r: &mut Reader<'_>, el: &Elem) -> Result<HeaderFooter, SlideError> {
    let mut el = el.clone();
    let hf = HeaderFooter {
        slide_number: r.attr(&mut el, "sldNum")?,
        header: r.attr(&mut el, "hdr")?,
        footer: r.attr(&mut el, "ftr")?,
        date_time: r.attr(&mut el, "dt")?,
    };
    r.finish(&el);
    finish_children(r, &el)?;
    Ok(hf)
}

// spec-audit: owner=sldLayoutIdLst
fn read_layout_ids(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &[Relationship],
    limits: &SlideLimits,
) -> Result<Vec<SlideLayoutId>, SlideError> {
    r.finish(el);
    if r.tree.child_elements(el.id).len() > limits.max_layout_ids {
        return Err(SlideError::TooManyParts {
            what: "layout ids",
            limit: limits.max_layout_ids,
        });
    }
    let mut out = Vec::new();
    let mut ids = HashSet::new();
    for child in r.children(el)? {
        if !child.is(ns::PML, "sldLayoutId") {
            r.retained(&child);
            continue;
        }
        let mut c = child.clone();
        let raw = c.take("id");
        let id = match raw {
            None => None,
            Some(raw) => {
                match <u32 as casual_slide_model::properties::Lexical>::parse_lexical(&raw) {
                    None => {
                        r.note(K::MalformedValue, &c, Some("id"));
                        continue;
                    }
                    Some(v) => {
                        if v < SLIDE_LAYOUT_ID_MIN {
                            r.note(K::OutOfRange, &c, Some("id"));
                        }
                        if !ids.insert(v) {
                            r.note(K::DuplicateId, &c, Some("id"));
                        }
                        Some(v)
                    }
                }
            }
        };
        let rel_id = c.take_ns(ns::REL, "id");
        r.finish(&c);
        finish_children(r, &c)?;
        let Some(rel_id) = rel_id else {
            r.note(K::MissingRequired, &c, Some("r:id"));
            continue;
        };
        match rels.iter().find(|x| x.id == rel_id) {
            None => r.note(K::DanglingRelationship, &c, Some("r:id")),
            Some(rel) if rel_suffix(rel) != Some("slideLayout") => {
                r.note(K::WrongRelationship, &c, Some("r:id"));
            }
            Some(_) => {}
        }
        out.push(SlideLayoutId { id, rel_id });
    }
    Ok(out)
}

// spec-audit: owner=txStyles
fn read_text_styles(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &[Relationship],
    color: &mut ColorFindings,
    drawing: DrawingLimits,
) -> Result<MasterTextStyles, SlideError> {
    r.finish(el);
    let mut styles = MasterTextStyles::default();
    for child in r.children(el)? {
        let slot = match child.local.as_str() {
            _ if !child.in_namespace(ns::PML) => None,
            "titleStyle" => Some(&mut styles.title),
            "bodyStyle" => Some(&mut styles.body),
            "otherStyle" => Some(&mut styles.other),
            _ => None,
        };
        let Some(slot) = slot else {
            r.retained(&child);
            continue;
        };
        if slot.is_some() {
            r.note(K::DuplicateElement, &child, None);
            continue;
        }
        let mut text = TextReader {
            r: &mut *r,
            rels,
            color: &mut *color,
            counts: &mut TextCounts::default(),
            drawing,
        };
        *slot = Some(text.list_style(&child)?);
    }
    Ok(styles)
}

/// The last segment of a relationship type in either conformance class.
pub fn rel_suffix(rel: &Relationship) -> Option<&str> {
    rel.rel_type
        .strip_prefix(TRANSITIONAL_RELS)
        .or_else(|| rel.rel_type.strip_prefix(STRICT_RELS))
}

/// Builds the structure, one part at a time.
struct Indexer<'p> {
    package: &'p PresentationPackage,
    names: HashMap<String, String>,
    by_name: HashMap<String, u32>,
    structure: DeckStructure,
    limits: &'p SlideLimits,
}

impl Indexer<'_> {
    /// The internal part a relationship of `source` reaches, as the archive spells it.
    fn target(&self, source: &str, rel: &Relationship) -> Option<String> {
        if rel.mode != TargetMode::Internal {
            return None;
        }
        let target = resolve_internal(source, &rel.target)?;
        self.names.get(&part_name_key(&target)?).cloned()
    }

    /// The first relationship of `source` of this type, and the part it reaches.
    fn follow(&self, source: &str, kind: &str, findings: &mut Findings) -> Option<Option<String>> {
        let mut found = self
            .package
            .relationships_of(source)
            .iter()
            .filter(|r| rel_suffix(r) == Some(kind));
        let rel = found.next()?;
        if found.next().is_some() {
            findings.note_role(K::DuplicateRelationship, source, kind);
        }
        let target = self.target(source, rel);
        if target.is_none() {
            findings.note_role(K::UnusableRelationship, source, kind);
        }
        Some(target)
    }

    /// The ordinal of a part of this kind, adding it if new. `None`, reported, if
    /// its content type says it is something else.
    fn add(
        &mut self,
        kind: SlideKind,
        part: &str,
        source: &str,
        findings: &mut Findings,
    ) -> Result<Option<u32>, SlideError> {
        let declared = self.package.content_types().resolve(part);
        if declared != Some(kind.content_type()) {
            findings.note_role(K::WrongRelationship, source, kind.rel_type());
            return Ok(None);
        }
        if let Some(&ordinal) = self.by_name.get(part) {
            let same = self
                .structure
                .entry(ordinal)
                .is_some_and(|e| e.kind == kind);
            if !same {
                findings.note_role(K::WrongRelationship, source, kind.rel_type());
                return Ok(None);
            }
            return Ok(Some(ordinal));
        }
        if self.structure.parts.len() >= self.limits.max_parts {
            // A limit stops the indexing, not the deck, unless the host wants refusal.
            if !self.package.is_strict() {
                findings.note_role(K::UnresolvedReference, source, kind.rel_type());
                return Ok(None);
            }
            return Err(SlideError::TooManyParts {
                what: "slide-like parts",
                limit: self.limits.max_parts,
            });
        }
        let ordinal =
            u32::try_from(self.structure.parts.len()).map_err(|_| SlideError::TooManyParts {
                what: "slide-like parts",
                limit: self.limits.max_parts,
            })?;
        self.structure.parts.push(PartEntry {
            id: ObjectId::part(ordinal),
            kind,
            part: part.to_owned(),
            layout: None,
            master: None,
            notes: None,
            theme: None,
        });
        self.by_name.insert(part.to_owned(), ordinal);
        match kind {
            SlideKind::Master => self.structure.masters.push(ordinal),
            SlideKind::Layout => self.structure.layouts.push(ordinal),
            SlideKind::Notes => self.structure.notes.push(ordinal),
            _ => {}
        }
        Ok(Some(ordinal))
    }

    /// Follow `source`'s relationship of `kind`, and add what it reaches.
    fn add_related(
        &mut self,
        source: &str,
        kind: SlideKind,
        findings: &mut Findings,
    ) -> Result<Option<Option<u32>>, SlideError> {
        let Some(target) = self.follow(source, kind.rel_type(), findings) else {
            return Ok(None);
        };
        Ok(Some(match target {
            Some(part) => self.add(kind, &part, source, findings)?,
            None => None,
        }))
    }

    fn entry_mut(&mut self, ordinal: u32) -> Option<&mut PartEntry> {
        self.structure.parts.get_mut(usize::try_from(ordinal).ok()?)
    }

    fn part_name(&self, ordinal: u32) -> String {
        self.structure
            .entry(ordinal)
            .map(|e| e.part.clone())
            .unwrap_or_default()
    }
}

/// Locate every slide-like part from the presentation and the relationships the
/// package holds. Reads no part. A slide whose layout, a layout whose master, or a
/// notes slide whose master is missing or of the wrong kind is reported as
/// `CS-IMF-0008` or `CS-IMF-0007` and has none.
pub fn index_deck(
    package: &PresentationPackage,
    presentation: &Presentation,
    limits: &SlideLimits,
    findings: &mut Findings,
) -> Result<DeckStructure, SlideError> {
    let main = package.main_part().to_owned();
    let names = package
        .part_names()
        .into_iter()
        .filter_map(|n| Some((part_name_key(&n)?, n)))
        .collect();
    let mut ix = Indexer {
        package,
        names,
        by_name: HashMap::new(),
        structure: DeckStructure::default(),
        limits,
    };
    let main_rels = package.relationships_of(&main);
    let by_id = |id: &str| main_rels.iter().find(|r| r.id == id);
    let reach = |ix: &Indexer<'_>, id: &str, kind: SlideKind, findings: &mut Findings| {
        let rel = by_id(id).filter(|r| rel_suffix(r) == Some(kind.rel_type()));
        let target = rel.and_then(|r| ix.target(&main, r));
        if target.is_none() {
            findings.note_role(K::UnresolvedReference, &main, kind.rel_type());
        }
        target
    };

    // Masters, and each one's theme and layouts, in relationship order.
    for entry in presentation.slide_masters.iter().flatten() {
        if let Some(part) = reach(&ix, &entry.rel_id, SlideKind::Master, findings) {
            ix.add(SlideKind::Master, &part, &main, findings)?;
        }
    }
    let mut at = 0;
    while let Some(&master) = ix.structure.masters.get(at) {
        at += 1;
        let part = ix.part_name(master);
        let theme = ix.follow(&part, "theme", findings).flatten();
        if let Some(e) = ix.entry_mut(master) {
            e.theme = theme;
        }
        let layouts: Vec<Relationship> = package
            .relationships_of(&part)
            .iter()
            .filter(|r| rel_suffix(r) == Some("slideLayout"))
            .cloned()
            .collect();
        for rel in layouts {
            match ix.target(&part, &rel) {
                Some(layout) => {
                    ix.add(SlideKind::Layout, &layout, &part, findings)?;
                }
                None => findings.note_role(K::UnusableRelationship, &part, "slideLayout"),
            }
        }
    }

    // Slides, each one's layout and notes.
    let slide_count = presentation.slides.as_ref().map_or(0, Vec::len);
    if slide_count > limits.max_slides {
        if package.is_strict() {
            return Err(SlideError::TooManySlides {
                limit: limits.max_slides,
            });
        }
        findings.note_role(K::UnresolvedReference, &main, "slide");
    }
    let mut seen_slides = HashSet::new();
    for entry in presentation.slides.iter().flatten().take(limits.max_slides) {
        let Some(part) = reach(&ix, &entry.rel_id, SlideKind::Slide, findings) else {
            continue;
        };
        let Some(slide) = ix.add(SlideKind::Slide, &part, &main, findings)? else {
            continue;
        };
        ix.structure.slides.push(slide);
        if !seen_slides.insert(slide) {
            findings.note(K::DuplicateId, &main, "sldId", Some("r:id"));
            continue;
        }
        let layout = match ix.add_related(&part, SlideKind::Layout, findings)? {
            Some(layout) => layout,
            None => {
                findings.note_role(K::UnresolvedReference, &part, "slideLayout");
                None
            }
        };
        let notes = ix.add_related(&part, SlideKind::Notes, findings)?.flatten();
        if let Some(e) = ix.entry_mut(slide) {
            e.layout = layout;
            e.notes = notes;
        }
    }

    // The notes and handout masters.
    let first = |list: &Option<Vec<String>>| list.as_ref().and_then(|l| l.first().cloned());
    if let Some(id) = first(&presentation.notes_masters)
        && let Some(part) = reach(&ix, &id, SlideKind::NotesMaster, findings)
    {
        ix.structure.notes_master = ix.add(SlideKind::NotesMaster, &part, &main, findings)?;
    }
    if let Some(id) = first(&presentation.handout_masters)
        && let Some(part) = reach(&ix, &id, SlideKind::HandoutMaster, findings)
    {
        ix.structure.handout_master = ix.add(SlideKind::HandoutMaster, &part, &main, findings)?;
    }

    // Each layout's master, and each notes slide's notes master, by its own
    // relationship. One step each: nothing beyond is followed.
    let layouts = ix.structure.layouts.clone();
    for layout in layouts {
        let part = ix.part_name(layout);
        let master = match ix.add_related(&part, SlideKind::Master, findings)? {
            Some(master) => master,
            None => {
                findings.note_role(K::UnresolvedReference, &part, "slideMaster");
                None
            }
        };
        if let Some(e) = ix.entry_mut(layout) {
            e.master = master;
        }
    }
    let notes = ix.structure.notes.clone();
    for n in notes {
        let part = ix.part_name(n);
        let master = ix
            .add_related(&part, SlideKind::NotesMaster, findings)?
            .flatten();
        if let Some(e) = ix.entry_mut(n) {
            e.master = master;
        }
    }
    // A master the presentation relates to but no list names is not read. It stays in the
    // package, and the report says so.
    if ix.structure.notes_master.is_none() && ix.follow(&main, "notesMaster", findings).is_some() {
        findings.note_role(K::RetainedElement, &main, "notesMaster");
    }
    if ix.structure.handout_master.is_none()
        && ix.follow(&main, "handoutMaster", findings).is_some()
    {
        findings.note_role(K::RetainedElement, &main, "handoutMaster");
    }
    // A master reached only from a layout has not had its theme found.
    for &master in &ix.structure.masters.clone() {
        let needs = ix
            .structure
            .entry(master)
            .is_some_and(|e| e.theme.is_none());
        if needs {
            let part = ix.part_name(master);
            let theme = ix.follow(&part, "theme", &mut Findings::new(0)).flatten();
            if let Some(e) = ix.entry_mut(master) {
                e.theme = theme;
            }
        }
    }
    Ok(ix.structure)
}

/// Read one part of the structure from the package.
pub fn load_part(
    package: &mut PresentationPackage,
    structure: &DeckStructure,
    ordinal: u32,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
) -> Result<ImportedPart, SlideError> {
    let entry = structure
        .entry(ordinal)
        .ok_or(SlideError::NoSuchPart { ordinal })?;
    let (tree, _) = package.read_resolved(&entry.part)?;
    let rels = package.relationships_of(&entry.part).to_vec();
    let mut findings = Findings::new(import_limits.max_findings);
    let mut color = ColorFindings::default();
    let value = import_slide_part(
        &tree,
        &entry.part,
        &rels,
        entry.kind,
        ordinal,
        limits,
        import_limits,
        &mut findings,
        &mut color,
    )?;
    Ok(ImportedPart {
        value,
        findings,
        color_findings: color,
        damaged: package.is_damaged(&entry.part),
    })
}

/// Read one part as [`load_part`] does, except that a part this cannot use, whatever the
/// reason, is set aside and read as the empty part in its place, so slide order survives
/// (`REC-04`). Under `strict`, the refusal stands.
pub fn load_part_recovering(
    package: &mut PresentationPackage,
    structure: &DeckStructure,
    ordinal: u32,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
) -> Result<ImportedPart, SlideError> {
    match load_part(package, structure, ordinal, limits, import_limits) {
        Err(err) if !package.is_strict() && !matches!(err, SlideError::NoSuchPart { .. }) => {
            let entry = structure
                .entry(ordinal)
                .ok_or(SlideError::NoSuchPart { ordinal })?;
            package.set_part_aside(&entry.part)?;
            let mut part = load_part(package, structure, ordinal, limits, import_limits)?;
            part.findings.note(K::UnreadablePart, &entry.part, "", None);
            part.damaged = true;
            Ok(part)
        }
        other => other,
    }
}

/// The deck's slide-like parts, each read when first asked for and kept.
#[derive(Debug, Clone)]
pub struct LazySlides {
    /// Where everything is.
    pub structure: DeckStructure,
    loaded: Vec<Option<ImportedPart>>,
    limits: SlideLimits,
    import_limits: ImportLimits,
}

impl LazySlides {
    /// Nothing read yet.
    pub fn new(structure: DeckStructure, limits: SlideLimits, import_limits: ImportLimits) -> Self {
        let loaded = vec![None; structure.parts.len()];
        Self {
            structure,
            loaded,
            limits,
            import_limits,
        }
    }

    /// A part, read now if it has not been.
    pub fn part(
        &mut self,
        package: &mut PresentationPackage,
        ordinal: u32,
    ) -> Result<&ImportedPart, SlideError> {
        let at = usize::try_from(ordinal).map_err(|_| SlideError::NoSuchPart { ordinal })?;
        let slot = self
            .loaded
            .get(at)
            .ok_or(SlideError::NoSuchPart { ordinal })?;
        if slot.is_none() {
            let part = load_part_recovering(
                package,
                &self.structure,
                ordinal,
                &self.limits,
                &self.import_limits,
            )?;
            if let Some(slot) = self.loaded.get_mut(at) {
                *slot = Some(part);
            }
        }
        self.loaded
            .get(at)
            .and_then(Option::as_ref)
            .ok_or(SlideError::NoSuchPart { ordinal })
    }

    /// The slide at a place in the deck, read now if it has not been.
    pub fn slide(
        &mut self,
        package: &mut PresentationPackage,
        index: usize,
    ) -> Result<&ImportedPart, SlideError> {
        let ordinal = self
            .structure
            .slide(index)
            .ok_or(SlideError::NoSuchPart { ordinal: u32::MAX })?;
        self.part(package, ordinal)
    }

    /// Whether a part has been read.
    pub fn is_loaded(&self, ordinal: u32) -> bool {
        usize::try_from(ordinal)
            .ok()
            .and_then(|at| self.loaded.get(at))
            .is_some_and(Option::is_some)
    }

    /// How many parts have been read.
    pub fn loaded_count(&self) -> usize {
        self.loaded.iter().filter(|p| p.is_some()).count()
    }
}

/// Every slide-like part, read, and the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedSlides {
    /// The model.
    pub parts: SlideParts,
    /// The structure's findings, then each part's, merged.
    pub findings: Findings,
    /// What the colour map readers noticed, across parts.
    pub color_findings: ColorFindings,
    /// The `tableStyles` part, if there is one and it can be read.
    pub table_styles: Option<casual_slide_model::table::TableStyleList>,
    /// The parts that arrived damaged, by ordinal, so a host can mark them.
    pub damaged: Vec<u32>,
    /// Every chart-family part, each read or set aside, and each embedded workbook unread.
    pub charts: crate::chart::DeckCharts,
}

/// Index the deck and read every slide-like part. A part that cannot be used, by being
/// malformed, the wrong kind, or over a bound, is reported as `CS-IMF-0012`, set aside and
/// read as an empty part, so slide order survives (`REC-04`). Under `strict`, a part with
/// a fault is left out, and a bound refuses the deck.
pub fn import_slide_parts(
    package: &mut PresentationPackage,
    presentation: &Presentation,
    limits: &SlideLimits,
    import_limits: &ImportLimits,
) -> Result<ImportedSlides, SlideError> {
    let mut findings = Findings::new(import_limits.max_findings);
    let structure = index_deck(package, presentation, limits, &mut findings)?;
    let mut color_findings = ColorFindings::default();
    let mut parts = Vec::with_capacity(structure.parts.len());
    let mut damaged = Vec::new();
    let mut loaded = Vec::with_capacity(structure.parts.len());
    for entry in &structure.parts {
        loaded.push(load_part(
            package,
            &structure,
            entry.id.part,
            limits,
            import_limits,
        ));
    }
    // Everything that will not read is set aside at once, which is one repair however many.
    if !package.is_strict() {
        let unusable: Vec<String> = structure
            .parts
            .iter()
            .zip(&loaded)
            .filter(|(_, read)| read.is_err())
            .map(|(entry, _)| entry.part.clone())
            .collect();
        if !unusable.is_empty() {
            package.set_parts_aside(&unusable, usize::MAX)?;
            for part in &unusable {
                findings.note(K::UnreadablePart, part, "", None);
            }
            for (entry, read) in structure.parts.iter().zip(&mut loaded) {
                if read.is_err() {
                    *read = load_part(package, &structure, entry.id.part, limits, import_limits);
                }
            }
        }
    }
    for (entry, read) in structure.parts.iter().zip(loaded) {
        match read {
            Ok(part) => {
                if part.damaged {
                    damaged.push(entry.id.part);
                }
                for f in part.findings.iter() {
                    findings.note(f.kind, &f.part, &f.element, f.attribute.as_deref());
                }
                color_findings.total = color_findings
                    .total
                    .saturating_add(part.color_findings.total);
                color_findings.items.extend(
                    part.color_findings.items.into_iter().take(
                        color_findings
                            .bounds
                            .findings
                            .saturating_sub(color_findings.items.len()),
                    ),
                );
                parts.push(Some(part.value));
            }
            // A part nested past the XML limit is that part's, however it got there.
            Err(err) if is_part_fault(&err) || is_too_deep(&err) => {
                findings.note(K::UnreadablePart, &entry.part, "", None);
                parts.push(None);
            }
            Err(err) => return Err(err),
        }
    }
    let table_styles = read_deck_table_styles(
        package,
        limits,
        import_limits,
        &mut findings,
        &mut color_findings,
    )?;
    let parts = SlideParts { structure, parts };
    check_style_references(&parts, table_styles.as_ref(), &mut findings);
    let charts = crate::chart::read_deck_charts(
        package,
        &limits.chart,
        limits.drawing,
        import_limits,
        &mut findings,
        &mut color_findings,
    );
    Ok(ImportedSlides {
        parts,
        findings,
        color_findings,
        table_styles,
        damaged,
        charts,
    })
}
