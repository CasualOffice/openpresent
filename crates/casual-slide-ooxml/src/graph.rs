//! The part graph of an admitted presentation.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use casual_slide_package::{
    Package, PackageError, PartEdits, Salvaged, part_entry_key, salvage, scrub_name,
};

use crate::content_types::{CONTENT_TYPES_PART, ContentTypes};
use crate::error::OoxmlError;
use crate::limits::OoxmlLimits;
use crate::mce::{Compatibility, resolve_compatibility};
use crate::namespaces::{to_strict, to_transitional};
use crate::part_name::{PartIndex, entry_key, part_name_key};
use crate::recovery::{RecoveryReport, Repair, ReportLine};
use crate::rels::{
    ExternalKind, Relationship, TargetMode, classify_external, parse_relationships, rels_part_for,
    resolve_internal, source_of_rels,
};
use crate::repair::{
    Repaired, STRICT_RELS, TRANSITIONAL_RELS, chain_is_broken, entry_is_good, part_needs_repair,
    repair_package,
};
use crate::xml::{XmlTree, walk, wrong_root};

const OFFICE_RELS_2006: &str = "http://schemas.microsoft.com/office/2006/relationships/";
const SIGNATURE_ORIGIN: &str =
    "http://schemas.openxmlformats.org/package/2006/relationships/digital-signature/origin";
const COMPOUND_FILE_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

const PML: &str = "application/vnd.openxmlformats-officedocument.presentationml.";
const PPT_MACRO: &str = "application/vnd.ms-powerpoint.";

/// Relationship types in `MS-PPTX` that a presentation's audio, video and captions use.
const MICROSOFT_MEDIA_RELS: &[&str] = &[
    "http://schemas.microsoft.com/office/2007/relationships/media",
    "http://schemas.microsoft.com/office/2017/04/relationships/track",
];

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
    /// Damage was repaired ([`docs/29`](crate::RecoveryReport)).
    Repaired(Repair),
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
            Self::Repaired(repair) => repair.code(),
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

/// How to open a package.
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenOptions {
    /// The bounds, which hold in either mode.
    pub limits: OoxmlLimits,
    /// Refuse a fault rather than recover from it, for a deployment that prefers
    /// rejection, such as a mail gateway. The default is recovery (`docs/29`).
    pub strict: bool,
}

/// An admitted presentation: its part graph and the report on it.
///
/// **What is read is one clean package.** A damaged deck is repaired into an image in
/// memory, which everything above reads and an edit writes from, while the bytes as they
/// arrived are kept for an unedited save (`docs/29-RECOVERY.md`).
#[derive(Debug)]
pub struct PresentationPackage {
    package: Package,
    /// The file exactly as it arrived.
    original: Arc<[u8]>,
    /// Refuse faults rather than repair them.
    strict: bool,
    /// XML parts seen to need no repair, or made so, by key.
    checked: BTreeSet<String>,
    /// Every part's stream and XML has been seen to be sound.
    prepared: bool,
    /// What repairs have done so far.
    repairs: BTreeSet<Finding>,
    /// Parts whose content is not what arrived, by key.
    damaged: BTreeSet<String>,
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
    /// Open `bytes`, recovering from what is wrong with them: strict first, and on failure
    /// a repair. Refuses only what `docs/29` lists: a Compound File, the whole-file size
    /// limit, and what holds no ZIP entry at all. An unedited save returns `bytes`.
    pub fn open(bytes: Vec<u8>, limits: OoxmlLimits) -> Result<Self, OoxmlError> {
        Self::open_with(
            bytes,
            OpenOptions {
                limits,
                strict: false,
            },
        )
    }

    /// Open `bytes` refusing any fault, each with its own code, and repairing nothing.
    pub fn open_strict(bytes: Vec<u8>, limits: OoxmlLimits) -> Result<Self, OoxmlError> {
        Self::open_with(
            bytes,
            OpenOptions {
                limits,
                strict: true,
            },
        )
    }

    /// Open `bytes` as `options` says.
    pub fn open_with(bytes: Vec<u8>, options: OpenOptions) -> Result<Self, OoxmlError> {
        let OpenOptions { limits, strict } = options;
        if bytes.starts_with(&COMPOUND_FILE_MAGIC) {
            return Err(OoxmlError::CompoundFile);
        }
        let source: Arc<[u8]> = Arc::from(bytes);
        if strict {
            let mut opened = Self::admit(Arc::clone(&source), limits)?;
            opened.strict = true;
            return Ok(opened);
        }
        // The one limit that refuses: it protects the machine.
        let size = source.len() as u64;
        if size > limits.package.max_input_bytes {
            return Err(PackageError::InputTooLarge {
                size,
                limit: limits.package.max_input_bytes,
            }
            .into());
        }
        match Self::admit(Arc::clone(&source), limits) {
            Ok(mut opened) => {
                let names = opened.part_names();
                let broken = chain_is_broken(
                    &opened.main_part,
                    &opened.relationships,
                    &names,
                    &opened.content_types,
                );
                if broken {
                    opened.repair_now(&BTreeSet::new())?;
                }
                Ok(opened)
            }
            Err(_) => Self::recover(source, limits),
        }
    }

    /// Repair what strict admission refused, and open the result.
    fn recover(source: Arc<[u8]>, limits: OoxmlLimits) -> Result<Self, OoxmlError> {
        let mut findings = BTreeSet::new();
        let package = Package::open_shared(Arc::clone(&source), limits.package)
            .ok()
            .and_then(|mut package| package.verify_entries().then_some(package));
        let mut package = match package {
            Some(package) => package,
            None => Self::salvaged(&source, &limits, &mut findings)?,
        };
        let repaired = repair_package(&mut package, &limits, &BTreeSet::new(), usize::MAX)?;
        let mut opened = Self::from_repaired(repaired, limits, &mut findings)?;
        opened.original = source;
        Ok(opened)
    }

    /// The container's own damage, read past.
    fn salvaged(
        source: &[u8],
        limits: &OoxmlLimits,
        findings: &mut BTreeSet<Finding>,
    ) -> Result<Package, OoxmlError> {
        let Salvaged { image, faults, .. } = salvage(source, limits.package, |name, bytes| {
            entry_is_good(name, bytes, limits)
        })?;
        for fault in faults {
            findings.insert(Finding {
                kind: FindingKind::Repaired(Repair::Container(fault.kind)),
                part: fault.entry,
                rel_id: None,
            });
        }
        Ok(Package::open(image, limits.package)?)
    }

    /// Open a repaired image, strictly: it is clean or the repair has a bug, and a bug
    /// here is a refusal the damage corpus sees, never an unchecked file.
    fn from_repaired(
        repaired: Repaired,
        limits: OoxmlLimits,
        findings: &mut BTreeSet<Finding>,
    ) -> Result<Self, OoxmlError> {
        let Repaired {
            image,
            findings: done,
            damaged,
        } = repaired;
        findings.extend(done);
        let mut opened = Self::admit(Arc::from(image), limits)?;
        opened.prepared = true;
        opened.repairs = std::mem::take(findings);
        opened.findings.extend(opened.repairs.iter().cloned());
        opened.findings.sort();
        opened.damaged = damaged.iter().map(|n| part_entry_key(n)).collect();
        for name in opened.part_names() {
            opened.checked.insert(part_entry_key(&name));
        }
        Ok(opened)
    }

    /// Replace this package with its repaired self, keeping what arrived. Run on first
    /// sight of damage: everything is mended at once, so there is one image and no overlay.
    fn repair_now(&mut self, aside: &BTreeSet<String>) -> Result<(), OoxmlError> {
        self.repair_capped(aside, usize::MAX)
    }

    fn repair_capped(&mut self, aside: &BTreeSet<String>, cap: usize) -> Result<(), OoxmlError> {
        let limits = self.limits;
        let mut findings = std::mem::take(&mut self.repairs);
        if !self.package.verify_entries() {
            let source = self.package.source_bytes().to_vec();
            self.package = Self::salvaged(&source, &limits, &mut findings)?;
        }
        let repaired = repair_package(&mut self.package, &limits, aside, cap)?;
        let mut opened = Self::from_repaired(repaired, limits, &mut findings)?;
        opened.original = Arc::clone(&self.original);
        opened.damaged.extend(std::mem::take(&mut self.damaged));
        *self = opened;
        Ok(())
    }

    /// Admit `bytes` under strict rules and walk everything reachable, reading only `.rels`
    /// parts, `[Content_Types].xml` and the main part's root.
    fn admit(bytes: Arc<[u8]>, limits: OoxmlLimits) -> Result<Self, OoxmlError> {
        let original = Arc::clone(&bytes);
        let mut package = Package::open_shared(bytes, limits.package)?;
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
            original,
            strict: false,
            checked: BTreeSet::new(),
            prepared: false,
            repairs: BTreeSet::new(),
            damaged: BTreeSet::new(),
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

    /// Everything noticed, in a stable order, repairs included.
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// Whether this package refuses faults rather than repairing them.
    pub fn is_strict(&self) -> bool {
        self.strict
    }

    /// Whether anything was repaired.
    pub fn was_repaired(&self) -> bool {
        !self.repairs.is_empty()
    }

    /// Whether a part's content is not what arrived: it was mended, or made.
    pub fn is_damaged(&self, part: &str) -> bool {
        self.damaged.contains(&part_entry_key(part))
    }

    /// What to tell the user, in plain language. Part names and fixed text: no document
    /// text. Empty for a deck that arrived healthy.
    pub fn recovery_report(&self) -> RecoveryReport {
        let lines = self
            .repairs
            .iter()
            .filter_map(|f| match f.kind {
                FindingKind::Repaired(repair) => Some(ReportLine {
                    code: repair.code(),
                    part: scrub_name(&f.part),
                    text: repair.plain(),
                }),
                _ => None,
            })
            .collect();
        RecoveryReport { lines }
    }

    /// A reader above could not use this part: put a stand-in in its place, retaining its
    /// bytes. A slide becomes an empty slide marked damaged. Refused under `strict`.
    pub fn set_part_aside(&mut self, name: &str) -> Result<(), OoxmlError> {
        self.set_parts_aside(&[name.to_owned()], usize::MAX)
    }

    /// [`PresentationPackage::set_part_aside`] for several parts at once. A main part among
    /// them is rebuilt with at most `list_cap` slides and masters, for a reader whose own
    /// limit was what it could not use.
    pub fn set_parts_aside(&mut self, names: &[String], list_cap: usize) -> Result<(), OoxmlError> {
        if self.strict {
            return Err(OoxmlError::MissingPart {
                name: names.first().cloned().unwrap_or_default(),
            });
        }
        let mut aside = BTreeSet::new();
        for name in names {
            aside.insert(self.actual_name(name)?);
        }
        self.repair_capped(&aside, list_cap)
    }

    /// Make every part sound, before an edit or a repair writes from them. Nothing is
    /// decided by this on a healthy deck, though each stream is read once. Under `strict`,
    /// nothing happens.
    pub fn prepare_for_edit(&mut self) -> Result<(), OoxmlError> {
        if self.strict || self.prepared {
            return Ok(());
        }
        let sound = self.package.verify_entries() && self.xml_is_sound();
        if !sound {
            self.repair_now(&BTreeSet::new())?;
        }
        self.prepared = true;
        Ok(())
    }

    fn xml_is_sound(&mut self) -> bool {
        let limits = self.limits;
        let cap = limits.max_xml_part_bytes as u64;
        for name in self.package.entry_names() {
            let lower = name.to_ascii_lowercase();
            let is_xml = lower.ends_with(".xml") || lower.ends_with(".rels");
            if !is_xml || name.ends_with('/') || self.checked.contains(&part_entry_key(&name)) {
                continue;
            }
            let Ok(bytes) = self.package.read_part_capped(&name, cap) else {
                return false;
            };
            if part_needs_repair(&bytes, &name, &limits) {
                return false;
            }
            self.checked.insert(part_entry_key(&name));
        }
        true
    }

    /// The bytes everything here reads from: what arrived, or the repaired image.
    pub fn base_bytes(&self) -> &[u8] {
        self.package.source_bytes()
    }

    /// The deck as a clean file, with nothing edited: an explicit repair.
    pub fn save_repaired(&mut self) -> Result<Vec<u8>, OoxmlError> {
        self.prepare_for_edit()?;
        Ok(self.package.source_bytes().to_vec())
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
        match self.package.read_part(&actual) {
            // A stream that will not read: mend the container, and look again.
            Err(PackageError::NotAPackage) if !self.strict => {
                self.repair_now(&BTreeSet::new())?;
                let actual = self.actual_name(name)?;
                Ok(self.package.read_part(&actual)?)
            }
            other => Ok(other?),
        }
    }

    /// Read a part that is to be parsed as XML. A part declaring more than the
    /// walker accepts (`max_xml_part_bytes`) is refused as `CS-PKG-0009` with
    /// nothing decompressed, rather than read in full for the walker to refuse
    /// (`PKG-04`).
    ///
    /// **Strict first, recovery on failure for that part.** A part the walker refuses, or
    /// one over the bound, starts the repair, and what comes back is the best reading that
    /// the walker admits. Under `strict`, the refusal is the answer.
    pub fn read_xml_part(&mut self, name: &str) -> Result<Vec<u8>, OoxmlError> {
        let actual = self.actual_name(name)?;
        let cap = self.limits.max_xml_part_bytes as u64;
        if self.strict {
            return Ok(self.package.read_part_capped(&actual, cap)?);
        }
        let key = part_entry_key(&actual);
        let first = self.package.read_part_capped(&actual, cap);
        let good = match &first {
            Ok(bytes) => {
                self.checked.contains(&key) || !part_needs_repair(bytes, &actual, &self.limits)
            }
            Err(_) => false,
        };
        if good {
            self.checked.insert(key);
            return Ok(first?);
        }
        if self.checked.contains(&key) {
            // Mended already, and still not to be had: this is a reader's limit.
            return Ok(first?);
        }
        self.repair_now(&BTreeSet::new())?;
        let actual = self.actual_name(name)?;
        Ok(self.package.read_part_capped(&actual, cap)?)
    }

    /// Bytes charged to the package's read budget so far: each part read, once.
    /// Opening charges only the structure it reads, never media.
    pub fn consumed(&self) -> u64 {
        self.package.consumed()
    }

    /// Read a part that is not XML, such as a picture or a clip, within a caller's ceiling.
    /// A part declaring more than `max` is refused as `CS-PKG-0009` with nothing
    /// decompressed.
    pub fn read_part_capped(&mut self, name: &str, max: u64) -> Result<Vec<u8>, OoxmlError> {
        let actual = self.actual_name(name)?;
        Ok(self.package.read_part_capped(&actual, max)?)
    }

    /// Read an XML part for use: its tree, keeping every branch and byte, and what
    /// markup compatibility says applies. Refuses a must-understand namespace that
    /// is not understood, in the part as read and not in a branch not taken.
    ///
    /// Under recovery, a repair has already taken the demand out of a part that had one.
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

    /// The file exactly as it arrived, damage included.
    pub fn save_unedited(&self) -> Vec<u8> {
        self.original.to_vec()
    }

    /// Write the file with `edits`. Every other part keeps its compressed bytes. With no
    /// edits, the file as it arrived. With any, a clean file: what was damaged is mended
    /// first, and what arrived of it is retained under `recovered/`.
    pub fn save_with(&mut self, edits: &PartEdits) -> Result<Vec<u8>, OoxmlError> {
        if edits.is_empty() {
            return Ok(self.save_unedited());
        }
        self.prepare_for_edit()?;
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
        // `MS-PPTX` "Media Part" and "Track Part".
        _ if MICROSOFT_MEDIA_RELS.contains(&r.rel_type.as_str()) => {}
        _ => note(FindingKind::UnknownRelationshipType, part, id),
    }
}
