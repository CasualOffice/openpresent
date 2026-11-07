//! The part graph of an admitted presentation.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use casual_slide_package::{Package, PartEdits};

use crate::content_types::{CONTENT_TYPES_PART, ContentTypes};
use crate::error::OoxmlError;
use crate::limits::OoxmlLimits;
use crate::mce::{Compatibility, resolve_compatibility};
use crate::namespaces::{to_strict, to_transitional};
use crate::part_name::{PartIndex, entry_key, part_name_key};
use crate::rels::{
    ExternalKind, Relationship, TargetMode, classify_external, parse_relationships, rels_part_for,
    resolve_internal, source_of_rels,
};
use crate::xml::{XmlTree, walk, wrong_root};

const TRANSITIONAL_RELS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
const STRICT_RELS: &str = "http://purl.oclc.org/ooxml/officeDocument/relationships/";
const OFFICE_RELS_2006: &str = "http://schemas.microsoft.com/office/2006/relationships/";
const SIGNATURE_ORIGIN: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/digital-signature/origin";
const COMPOUND_FILE_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

const PML: &str = "application/vnd.openxmlformats-officedocument.presentationml.";
const PPT_MACRO: &str = "application/vnd.ms-powerpoint.";

/// Relationship types in ECMA-376 that a presentation may use (`docs/spec/parts.md`).
/// Transitional files spell two with a hyphen, as real decks do; Strict does not.
const KNOWN_RELS: &[&str] = &[
    "officeDocument",
    "extended-properties",
    "custom-properties",
    "extendedProperties",
    "customProperties",
    "metadata/thumbnail",
    "slide",
    "slideLayout",
    "slideMaster",
    "notesSlide",
    "notesMaster",
    "handoutMaster",
    "presProps",
    "viewProps",
    "tableStyles",
    "tags",
    "theme",
    "themeOverride",
    "comments",
    "commentAuthors",
    "slideUpdateInfo",
    "slideUpdateUrl",
    "image",
    "audio",
    "video",
    "movie",
    "hyperlink",
    "chart",
    "chartUserShapes",
    "diagramData",
    "diagramLayout",
    "diagramQuickStyle",
    "diagramColors",
    "oleObject",
    "package",
    "control",
    "font",
    "customXml",
    "customXmlProps",
    "printerSettings",
    "vmlDrawing",
    "htmlPubSaveAs",
    "attachedTemplate",
];

/// Which form of the standard a file is written in. A file is written back in
/// the form it arrived in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conformance {
    /// `schemas.openxmlformats.org`, which is almost every real deck.
    Transitional,
    /// `purl.oclc.org`.
    Strict,
}

impl Conformance {
    /// A namespace as this file spells it, for anything newly written into it.
    /// Reading goes the other way, through [`to_transitional`].
    pub fn spell(self, uri: &str) -> &str {
        match self {
            Self::Transitional => to_transitional(uri),
            Self::Strict => to_strict(uri),
        }
    }
}

/// What the main part's content type says the file is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationKind {
    /// `.pptx`
    Presentation,
    /// `.ppsx`
    Slideshow,
    /// `.potx`
    Template,
    /// `.pptm`
    MacroPresentation,
    /// `.ppsm`
    MacroSlideshow,
    /// `.potm`
    MacroTemplate,
}

impl PresentationKind {
    fn from_content_type(ct: &str) -> Option<Self> {
        Some(
            match ct
                .strip_prefix(PML)
                .or_else(|| ct.strip_prefix(PPT_MACRO))?
            {
                "presentation.main+xml" => Self::Presentation,
                "slideshow.main+xml" => Self::Slideshow,
                "template.main+xml" => Self::Template,
                "presentation.macroEnabled.main+xml" => Self::MacroPresentation,
                "slideshow.macroEnabled.main+xml" => Self::MacroSlideshow,
                "template.macroEnabled.main+xml" => Self::MacroTemplate,
                _ => return None,
            },
        )
    }

    /// Whether this kind may carry a VBA project, which is never run.
    pub fn is_macro_enabled(self) -> bool {
        matches!(
            self,
            Self::MacroPresentation | Self::MacroSlideshow | Self::MacroTemplate
        )
    }
}

/// What was noticed about a package. None of these stops it opening. Each goes
/// to the compatibility report, and all of the content they refer to is retained.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FindingKind {
    /// A non-link relationship points outside the package: an image, video,
    /// template or object that a naive reader would fetch. Not fetched.
    ExternalContent(ExternalKind),
    /// A hyperlink no user should be offered: a script, UNC or `file:` target.
    UnsafeLink(ExternalKind),
    /// A relationship climbs above the package root. Not followed.
    TargetEscapesPackage,
    /// A relationship names a part that is not there.
    DanglingTarget,
    /// A part nothing points to. Retained.
    OrphanPart,
    /// A part with no declared content type. Invalid OPC.
    UndeclaredContentType,
    /// A relationship type not in ECMA-376's list for presentations. Retained.
    UnknownRelationshipType,
    /// A VBA project. Retained as bytes and never run.
    MacroProject,
    /// A digital signature. An edit invalidates it.
    Signature,
    /// An embedded object or package. Retained as bytes and never opened.
    EmbeddedObject,
    /// A target or entry name that decodes to traversal, or to no valid name at
    /// all. A target is not followed; an entry is retained and unreachable.
    DisguisedPartName,
    /// The part breaks ECMA-376 Part 3's syntax. Nothing was guessed at.
    MarkupCompatibility,
}

impl FindingKind {
    /// The stable diagnostic code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::ExternalContent(_) => "CS-FND-0001",
            Self::UnsafeLink(_) => "CS-FND-0002",
            Self::TargetEscapesPackage => "CS-FND-0003",
            Self::DanglingTarget => "CS-FND-0004",
            Self::OrphanPart => "CS-FND-0005",
            Self::UndeclaredContentType => "CS-FND-0006",
            Self::UnknownRelationshipType => "CS-FND-0007",
            Self::MacroProject => "CS-FND-0008",
            Self::Signature => "CS-FND-0009",
            Self::EmbeddedObject => "CS-FND-0010",
            Self::DisguisedPartName => "CS-FND-0011",
            Self::MarkupCompatibility => "CS-FND-0012",
        }
    }
}

/// One entry in the compatibility report.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// What was noticed.
    pub kind: FindingKind,
    /// The part concerned. Empty for the package itself.
    pub part: String,
    /// The relationship id, where one is involved.
    pub rel_id: Option<String>,
}

/// An admitted presentation: its part graph and the report on it.
#[derive(Debug)]
pub struct PresentationPackage {
    package: Package,
    index: PartIndex,
    limits: OoxmlLimits,
    content_types: ContentTypes,
    main_part: String,
    kind: PresentationKind,
    conformance: Conformance,
    /// Relationships by source part. The package's own are under the empty name.
    relationships: BTreeMap<String, Vec<Relationship>>,
    findings: Vec<Finding>,
}

impl PresentationPackage {
    /// Admit `bytes` and walk everything reachable, reading only `.rels` parts,
    /// `[Content_Types].xml` and the main part's root.
    pub fn open(bytes: Vec<u8>, limits: OoxmlLimits) -> Result<Self, OoxmlError> {
        if bytes.starts_with(&COMPOUND_FILE_MAGIC) {
            return Err(OoxmlError::CompoundFile);
        }
        let mut package = Package::open(bytes, limits.package)?;
        let index = PartIndex::build(package.entry_names())?;
        let ct_name = index
            .find(CONTENT_TYPES_PART)
            .ok_or_else(|| OoxmlError::MissingPart {
                name: CONTENT_TYPES_PART.to_owned(),
            })?
            .to_owned();
        let cap = limits.max_xml_part_bytes as u64;
        let content_types =
            ContentTypes::parse(&package.read_part_capped(&ct_name, cap)?, &limits)?;

        let root_rels = read_rels(&mut package, &index, "", &limits, &mut 0)?;
        let (main_part, conformance) = find_main_part(&root_rels)?;
        let main_part = part_name_key(&main_part)
            .and_then(|key| index.find_key(&key))
            .ok_or(OoxmlError::MissingPart {
                name: main_part.clone(),
            })?
            .to_owned();

        let content_type = content_types.resolve(&main_part).unwrap_or_default();
        let kind = PresentationKind::from_content_type(content_type).ok_or(
            OoxmlError::NotAPresentation {
                part: main_part.clone(),
            },
        )?;
        let main = package.read_part_capped(&main_part, cap)?;
        walk(&main, &main_part, &limits, |e, depth| {
            if depth == 1 && e.local_name().as_ref() != b"presentation" {
                return Err(wrong_root(
                    &main_part,
                    "the root element is not presentation",
                ));
            }
            Ok(())
        })?;
        let compatibility =
            checked_compatibility(&XmlTree::parse(&main, &main_part, &limits)?, &main_part)?;

        let (relationships, mut findings) =
            walk_graph(&mut package, &index, &limits, &content_types)?;
        if !compatibility.nonconformances.is_empty() {
            findings.push(Finding {
                kind: FindingKind::MarkupCompatibility,
                part: main_part.clone(),
                rel_id: None,
            });
            findings.sort();
        }
        Ok(Self {
            package,
            index,
            limits,
            content_types,
            main_part,
            kind,
            conformance,
            relationships,
            findings,
        })
    }

    /// The main part's name, usually `ppt/presentation.xml`.
    pub fn main_part(&self) -> &str {
        &self.main_part
    }

    /// What kind of file this is.
    pub fn kind(&self) -> PresentationKind {
        self.kind
    }

    /// Strict or Transitional.
    pub fn conformance(&self) -> Conformance {
        self.conformance
    }

    /// The bounds this package was admitted under.
    pub fn limits(&self) -> &OoxmlLimits {
        &self.limits
    }

    /// The content types as declared.
    pub fn content_types(&self) -> &ContentTypes {
        &self.content_types
    }

    /// The relationships a part declares. `""` is the package itself.
    pub fn relationships_of(&self, part: &str) -> &[Relationship] {
        self.relationships.get(part).map_or(&[], Vec::as_slice)
    }

    /// Everything noticed, in a stable order.
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// Every entry name, in archive order.
    pub fn part_names(&self) -> Vec<String> {
        self.package.entry_names()
    }

    /// Read a part by any spelling equivalent to its name. Bounded by what the
    /// part declares and by the package's budget. For a part to be parsed as XML,
    /// use [`PresentationPackage::read_xml_part`], which refuses an oversized part
    /// before decompressing any of it.
    pub fn read_part(&mut self, name: &str) -> Result<Vec<u8>, OoxmlError> {
        let actual = self.actual_name(name)?;
        Ok(self.package.read_part(&actual)?)
    }

    /// Read a part that is to be parsed as XML. A part declaring more than the
    /// walker accepts (`max_xml_part_bytes`) is refused as `CS-PKG-0009` with
    /// nothing decompressed, rather than read in full for the walker to refuse
    /// (`PKG-04`).
    pub fn read_xml_part(&mut self, name: &str) -> Result<Vec<u8>, OoxmlError> {
        let actual = self.actual_name(name)?;
        let cap = self.limits.max_xml_part_bytes as u64;
        Ok(self.package.read_part_capped(&actual, cap)?)
    }

    /// Bytes charged to the package's read budget so far: each part read, once.
    /// Opening charges only the structure it reads, never media.
    pub fn consumed(&self) -> u64 {
        self.package.consumed()
    }

    /// Read an XML part for use: its tree, keeping every branch and byte, and what
    /// markup compatibility says applies. Refuses a must-understand namespace that
    /// is not understood, in the part as read and not in a branch not taken.
    pub fn read_resolved(&mut self, name: &str) -> Result<(XmlTree, Compatibility), OoxmlError> {
        let bytes = self.read_xml_part(name)?;
        let actual = self.actual_name(name)?;
        let tree = XmlTree::parse(&bytes, &actual, &self.limits)?;
        let compatibility = checked_compatibility(&tree, &actual)?;
        Ok((tree, compatibility))
    }

    fn actual_name(&self, name: &str) -> Result<String, OoxmlError> {
        self.index
            .find(name)
            .map(str::to_owned)
            .ok_or_else(|| OoxmlError::MissingPart {
                name: name.to_owned(),
            })
    }

    /// The file exactly as it arrived.
    pub fn save_unedited(&self) -> Vec<u8> {
        self.package.source_bytes().to_vec()
    }

    /// Write the file with `edits`. Every other part keeps its compressed bytes.
    pub fn save_with(&mut self, edits: &PartEdits) -> Result<Vec<u8>, OoxmlError> {
        Ok(self.package.rewrite(edits)?)
    }
}

fn checked_compatibility(tree: &XmlTree, part: &str) -> Result<Compatibility, OoxmlError> {
    let compatibility = resolve_compatibility(tree);
    match compatibility.must_understand.iter().next() {
        Some(_) => Err(OoxmlError::MustUnderstand {
            part: part.to_owned(),
        }),
        None => Ok(compatibility),
    }
}

fn find_main_part(root: &[Relationship]) -> Result<(String, Conformance), OoxmlError> {
    let missing = || OoxmlError::MissingPart {
        name: "main document relationship".to_owned(),
    };
    let main = root
        .iter()
        .find_map(|r| {
            let conformance = match r.rel_type.strip_suffix("officeDocument")? {
                TRANSITIONAL_RELS => Conformance::Transitional,
                STRICT_RELS => Conformance::Strict,
                _ => return None,
            };
            (r.mode == TargetMode::Internal).then_some((r, conformance))
        })
        .ok_or_else(missing)?;
    let part = resolve_internal("", &main.0.target).ok_or_else(missing)?;
    Ok((part, main.1))
}

/// A part's relationships, charged against the package-wide total.
fn read_rels(
    package: &mut Package,
    index: &PartIndex,
    part: &str,
    limits: &OoxmlLimits,
    total: &mut usize,
) -> Result<Vec<Relationship>, OoxmlError> {
    let name = rels_part_for(part);
    let Some(actual) = index.find(&name).map(str::to_owned) else {
        return Ok(Vec::new());
    };
    let xml = package.read_part_capped(&actual, limits.max_xml_part_bytes as u64)?;
    let rels = parse_relationships(&xml, &actual, limits)?;
    *total += rels.len();
    if *total > limits.max_total_relationships {
        return Err(OoxmlError::TooManyRelationships {
            limit: limits.max_total_relationships,
        });
    }
    Ok(rels)
}

type Graph = (BTreeMap<String, Vec<Relationship>>, Vec<Finding>);

/// Breadth first over internal relationships. A visited set means a cycle, or a
/// part reached by many routes, costs one visit.
fn walk_graph(
    package: &mut Package,
    index: &PartIndex,
    limits: &OoxmlLimits,
    content_types: &ContentTypes,
) -> Result<Graph, OoxmlError> {
    let mut relationships = BTreeMap::new();
    let mut findings = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([String::new()]);
    let mut total = 0usize;
    let mut note = |kind: FindingKind, part: &str, rel_id: Option<&str>| {
        findings.insert(Finding {
            kind,
            part: part.to_owned(),
            rel_id: rel_id.map(str::to_owned),
        });
    };

    while let Some(part) = queue.pop_front() {
        if !visited.insert(entry_key(&part)) {
            continue;
        }
        let rels = read_rels(package, index, &part, limits, &mut total)?;
        for r in &rels {
            let id = r.reportable_id();
            inspect_type(r, &part, &mut note);
            match r.mode {
                TargetMode::External => {
                    let kind = classify_external(&r.target);
                    if is_link(&r.rel_type) {
                        if !kind.is_openable_link() && kind != ExternalKind::PowerPointAction {
                            note(FindingKind::UnsafeLink(kind), &part, id);
                        }
                    } else {
                        note(FindingKind::ExternalContent(kind), &part, id);
                    }
                }
                TargetMode::Internal => {
                    // An internal hyperlink can be an in-deck jump with no part.
                    if r.target.starts_with('#') {
                        continue;
                    }
                    match resolve_internal(&part, &r.target) {
                        None => note(FindingKind::TargetEscapesPackage, &part, id),
                        Some(target) => match part_name_key(&target) {
                            None => note(FindingKind::DisguisedPartName, &part, id),
                            Some(key) => match index.find_key(&key) {
                                None => note(FindingKind::DanglingTarget, &part, id),
                                Some(actual) => queue.push_back(actual.to_owned()),
                            },
                        },
                    }
                }
            }
        }
        if !rels.is_empty() {
            relationships.insert(part, rels);
        }
    }

    let content_types_key = entry_key(CONTENT_TYPES_PART);
    for name in package.entry_names() {
        let key = entry_key(&name);
        if key == content_types_key || name.ends_with('/') {
            continue;
        }
        if part_name_key(&name).is_none() {
            note(FindingKind::DisguisedPartName, &name, None);
        }
        // On the key, so `_RELS/.RELS` and `%5Frels/.rels` are recognized too.
        if let Some(source) = source_of_rels(&key) {
            if !visited.contains(&source) {
                note(FindingKind::OrphanPart, &name, None);
            }
            continue;
        }
        if !visited.contains(&key) {
            note(FindingKind::OrphanPart, &name, None);
        }
        if content_types.resolve(&name).is_none() {
            note(FindingKind::UndeclaredContentType, &name, None);
        }
    }
    Ok((relationships, findings.into_iter().collect()))
}

fn suffix(rel_type: &str) -> Option<&str> {
    rel_type
        .strip_prefix(TRANSITIONAL_RELS)
        .or_else(|| rel_type.strip_prefix(STRICT_RELS))
}

fn is_link(rel_type: &str) -> bool {
    suffix(rel_type) == Some("hyperlink")
}

fn inspect_type(
    r: &Relationship,
    part: &str,
    note: &mut impl FnMut(FindingKind, &str, Option<&str>),
) {
    let id = r.reportable_id();
    match suffix(&r.rel_type) {
        Some("oleObject" | "package" | "control") => {
            note(FindingKind::EmbeddedObject, part, id);
        }
        Some(s) if KNOWN_RELS.contains(&s) => {}
        _ if r.rel_type.strip_prefix(OFFICE_RELS_2006) == Some("vbaProject") => {
            note(FindingKind::MacroProject, part, id);
        }
        _ if r.rel_type == SIGNATURE_ORIGIN => note(FindingKind::Signature, part, id),
        // Core properties live in a namespace of their own.
        _ if r.rel_type.ends_with("/metadata/core-properties") => {}
        _ if r.rel_type.ends_with("/metadata/thumbnail") => {}
        _ => note(FindingKind::UnknownRelationshipType, part, id),
    }
}
