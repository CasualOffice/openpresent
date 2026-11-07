//! Read document properties from an [`XmlTree`] into the model, and the helpers
//! every Tier 1A and 1B reader here shares.
//!
//! Three outcomes for what a file says. A value that can be used goes in the
//! model. One that cannot is reported and left out, and stays in the tree, so a
//! save writes it back as it was. A count over a limit refuses the part.
//!
//! Names match on namespace, never on prefix or local name alone: `foo:title` in
//! another namespace is not `dc:title`, and `r:id` is not `id`.

use std::collections::{BTreeSet, HashSet};
use std::fmt;
use std::rc::Rc;

use casual_slide_model::properties::{
    AppProperties, ArrayBaseType, CoreProperties, CustomProperties, CustomProperty, Float32,
    Float64, Keyword, Keywords, Lexical, Variant, VectorBaseType, VtArray, VtVector, W3cDateTime,
    trim_xml,
};
use casual_slide_ooxml::{ElementId, OoxmlError, XmlTree};

use crate::schema_names::NAMES;
use crate::text::TextLimits;

/// Namespace names. Transitional first, then Strict, where they differ.
pub mod ns {
    /// `p:`
    pub const PML: &[&str] = &[
        "http://schemas.openxmlformats.org/presentationml/2006/main",
        "http://purl.oclc.org/ooxml/presentationml/main",
    ];
    /// `a:`
    pub const DML: &[&str] = &[
        "http://schemas.openxmlformats.org/drawingml/2006/main",
        "http://purl.oclc.org/ooxml/drawingml/main",
    ];
    /// `r:`
    pub const REL: &[&str] = &[
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
        "http://purl.oclc.org/ooxml/officeDocument/relationships",
    ];
    /// Extended properties.
    pub const EP: &[&str] = &[
        "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties",
        "http://purl.oclc.org/ooxml/officeDocument/extendedProperties",
    ];
    /// Custom properties.
    pub const CP: &[&str] = &[
        "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties",
        "http://purl.oclc.org/ooxml/officeDocument/customProperties",
    ];
    /// `vt:`
    pub const VT: &[&str] = &[
        "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes",
        "http://purl.oclc.org/ooxml/officeDocument/docPropsVTypes",
    ];
    /// Core properties, the same in both forms.
    pub const CORE: &[&str] =
        &["http://schemas.openxmlformats.org/package/2006/metadata/core-properties"];
    /// `dc:`
    pub const DC: &[&str] = &["http://purl.org/dc/elements/1.1/"];
    /// `dcterms:`
    pub const DCTERMS: &[&str] = &["http://purl.org/dc/terms/"];
    /// `xsi:`
    pub const XSI: &[&str] = &["http://www.w3.org/2001/XMLSchema-instance"];
}

/// Bounds on what a file can make this reader hold, on top of the part bounds in
/// `OoxmlLimits`. Over a bound, the part is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportLimits {
    /// Slide ids in `sldIdLst`, and outline entries in `viewProps`.
    pub max_slides: usize,
    /// Entries in each master id list.
    pub max_masters: usize,
    /// Custom shows.
    pub max_custom_shows: usize,
    /// Slides across all of a part's custom shows.
    pub max_custom_show_slides: usize,
    /// Embedded fonts.
    pub max_embedded_fonts: usize,
    /// `custData` entries.
    pub max_customer_data: usize,
    /// Guides in one list.
    pub max_guides: usize,
    /// Custom properties.
    pub max_custom_properties: usize,
    /// `value` children of `keywords`.
    pub max_keywords: usize,
    /// Vector and array items across a part. A `vt:vector/@size` is never believed.
    pub max_variant_items: usize,
    /// Nesting of `vt:variant` and `vt:vector`.
    pub max_variant_depth: usize,
    /// Bytes in one attribute value or text value.
    pub max_value_bytes: usize,
    /// Distinct findings kept. One more says the list was cut.
    pub max_findings: usize,
    /// Bounds on text bodies and list styles.
    pub text: TextLimits,
}

impl Default for ImportLimits {
    fn default() -> Self {
        Self {
            max_slides: 100_000,
            max_masters: 10_000,
            max_custom_shows: 10_000,
            max_custom_show_slides: 500_000,
            max_embedded_fonts: 4_096,
            max_customer_data: 100_000,
            max_guides: 65_536,
            max_custom_properties: 100_000,
            max_keywords: 100_000,
            max_variant_items: 1_000_000,
            max_variant_depth: 16,
            max_value_bytes: 16 << 20,
            max_findings: 10_000,
            text: TextLimits::default(),
        }
    }
}

/// Which text bound was crossed. Each has its own stable code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBound {
    /// Paragraphs in one text body.
    ParagraphsInBody,
    /// Runs, breaks and fields in one paragraph.
    RunsInParagraph,
    /// Paragraphs across one part.
    ParagraphsInPart,
    /// Runs, breaks and fields across one part.
    RunsInPart,
    /// Tab stops in one list.
    TabStops,
}

impl TextBound {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::ParagraphsInBody => "CS-TXT-0001",
            Self::RunsInParagraph => "CS-TXT-0002",
            Self::ParagraphsInPart => "CS-TXT-0003",
            Self::RunsInPart => "CS-TXT-0004",
            Self::TabStops => "CS-TXT-0005",
        }
    }

    fn what(self) -> &'static str {
        match self {
            Self::ParagraphsInBody => "paragraphs in a text body",
            Self::RunsInParagraph => "runs in a paragraph",
            Self::ParagraphsInPart => "paragraphs in the part",
            Self::RunsInPart => "runs in the part",
            Self::TabStops => "tab stops in a list",
        }
    }
}

/// Why a part could not be read. Clean, and names parts and counts, never content.
#[derive(Debug)]
#[non_exhaustive]
pub enum ImportError {
    /// The package or the XML refused.
    Ooxml(OoxmlError),
    /// More of something than [`ImportLimits`] allows.
    TooMany {
        /// What was counted.
        what: &'static str,
        /// The part.
        part: String,
        /// The bound.
        limit: usize,
    },
    /// A value longer than [`ImportLimits::max_value_bytes`].
    ValueTooLong {
        /// The part.
        part: String,
        /// The bound.
        limit: usize,
    },
    /// A part's root is not the element, in the namespace, that its role requires.
    WrongRoot {
        /// The part.
        part: String,
        /// The element expected.
        expected: &'static str,
    },
    /// More text than [`crate::text::TextLimits`] allows.
    TextBound {
        /// Which bound.
        bound: TextBound,
        /// The part.
        part: String,
        /// The bound's value.
        limit: usize,
    },
}

impl ImportError {
    /// The stable diagnostic code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Ooxml(err) => err.code(),
            Self::TooMany { .. } => "CS-IMP-0001",
            Self::ValueTooLong { .. } => "CS-IMP-0002",
            Self::WrongRoot { .. } => "CS-IMP-0003",
            Self::TextBound { bound, .. } => bound.code(),
        }
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ooxml(err) => write!(f, "{err}"),
            Self::TooMany { what, part, limit } => {
                write!(
                    f,
                    "[{}] {part:?}: {what} over the limit {limit}",
                    self.code()
                )
            }
            Self::ValueTooLong { part, limit } => {
                write!(f, "[{}] {part:?}: a value over {limit} bytes", self.code())
            }
            Self::WrongRoot { part, expected } => {
                write!(f, "[{}] {part:?}: root is not {expected}", self.code())
            }
            Self::TextBound { bound, part, limit } => {
                write!(
                    f,
                    "[{}] {part:?}: {} over the limit {limit}",
                    self.code(),
                    bound.what()
                )
            }
        }
    }
}

impl std::error::Error for ImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ooxml(err) => Some(err),
            _ => None,
        }
    }
}

impl From<OoxmlError> for ImportError {
    fn from(err: OoxmlError) -> Self {
        Self::Ooxml(err)
    }
}

/// What was noticed while reading. Nothing here stopped the import, and everything
/// it names stays in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImportFindingKind {
    /// A value that does not parse as its type. Left out of the model.
    MalformedValue,
    /// A value that parses but is outside its type's range. Left out of the model,
    /// except a slide or master id, which still names a real part and so is kept.
    OutOfRange,
    /// A required attribute or child is missing, so the entry is not in the model.
    MissingRequired,
    /// An element that may appear once appears again. The first is used.
    DuplicateElement,
    /// An id repeated where ids must be unique.
    DuplicateId,
    /// An `r:id` with no relationship in the part's `.rels`.
    DanglingRelationship,
    /// An `r:id` whose relationship is of the wrong type, or points outside the package.
    WrongRelationship,
    /// A reference to something else in the deck that is not there.
    UnresolvedReference,
    /// A `vt:vector/@size` that disagrees with the items.
    CountMismatch,
    /// An element this tier does not model. Preserved, not modelled.
    RetainedElement,
    /// An attribute this tier does not model. Preserved, not modelled.
    RetainedAttribute,
    /// An optional part that could not be read. It is left out of the model and
    /// written back as it was.
    UnreadablePart,
    /// More than one relationship where only one is allowed. The first is used.
    DuplicateRelationship,
    /// A property part's relationship names a part that is not there, or leaves the package.
    UnusableRelationship,
    /// The list of findings reached its bound.
    FindingsTruncated,
    /// What a drawing property's reader noticed, under its own codes.
    Drawing(crate::drawing::DrawingFinding),
    /// What the colour reader noticed in a shape's properties, under its own codes.
    Color(crate::color::FindingKind),
    /// What the timing reader noticed, under its own codes.
    Timing(crate::timing::TimingFinding),
}

impl ImportFindingKind {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::MalformedValue => "CS-IMF-0001",
            Self::OutOfRange => "CS-IMF-0002",
            Self::MissingRequired => "CS-IMF-0003",
            Self::DuplicateElement => "CS-IMF-0004",
            Self::DuplicateId => "CS-IMF-0005",
            Self::DanglingRelationship => "CS-IMF-0006",
            Self::WrongRelationship => "CS-IMF-0007",
            Self::UnresolvedReference => "CS-IMF-0008",
            Self::CountMismatch => "CS-IMF-0009",
            Self::RetainedElement => "CS-IMF-0010",
            Self::RetainedAttribute => "CS-IMF-0011",
            Self::UnreadablePart => "CS-IMF-0012",
            Self::DuplicateRelationship => "CS-IMF-0013",
            Self::UnusableRelationship => "CS-IMF-0014",
            Self::FindingsTruncated => "CS-IMF-0015",
            Self::Drawing(finding) => finding.code(),
            Self::Color(finding) => finding.code(),
            Self::Timing(finding) => finding.code(),
        }
    }
}

/// What a report says for a name the schemas do not have.
pub const UNKNOWN_NAME: &str = "(unknown)";

/// A name as a report may repeat it. A part chooses its element, attribute and prefix
/// names, so any of them can hold text, and an unknown name is an unknown name: the
/// schemas' own, with a prefix from the conventional few, is all that is repeated
/// (`SEC-018`). What the name says is in the part, which keeps it.
pub(crate) fn reportable_name(name: &str) -> &str {
    const PREFIXES: [&str; 12] = [
        "a", "c", "cp", "dc", "dcterms", "dgm", "mc", "p", "pic", "r", "vt", "xsi",
    ];
    let (prefix, local) = match name.split_once(':') {
        Some((prefix, local)) => (Some(prefix), local),
        None => (None, name),
    };
    let known = prefix.is_none_or(|p| PREFIXES.contains(&p)) && NAMES.binary_search(&local).is_ok();
    if known || name.is_empty() {
        name
    } else {
        UNKNOWN_NAME
    }
}

/// One entry in the import report. Structure only: never a value from the document.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImportFinding {
    /// What was noticed.
    pub kind: ImportFindingKind,
    /// The part.
    pub part: String,
    /// The element's local name if the schemas have it, else [`UNKNOWN_NAME`].
    pub element: String,
    /// The attribute, as written, if one is concerned and the schemas have it, else
    /// [`UNKNOWN_NAME`].
    pub attribute: Option<String>,
}

/// The findings of an import, distinct, in a stable order, and bounded.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Findings {
    items: BTreeSet<ImportFinding>,
    limit: usize,
}

impl Findings {
    /// An empty report keeping at most `limit` distinct findings.
    pub fn new(limit: usize) -> Self {
        Self {
            items: BTreeSet::new(),
            limit,
        }
    }

    /// Note something. `element` and `attribute` may come from the file, so a name the
    /// schemas do not have is reported as [`UNKNOWN_NAME`].
    pub fn note(
        &mut self,
        kind: ImportFindingKind,
        part: &str,
        element: &str,
        attribute: Option<&str>,
    ) {
        self.insert(ImportFinding {
            kind,
            part: part.to_owned(),
            element: reportable_name(element).to_owned(),
            attribute: attribute.map(|a| reportable_name(a).to_owned()),
        });
    }

    /// Note something about a relationship role, by the name this crate gives it.
    pub(crate) fn note_role(&mut self, kind: ImportFindingKind, part: &str, role: &str) {
        self.insert(ImportFinding {
            kind,
            part: part.to_owned(),
            element: role.to_owned(),
            attribute: None,
        });
    }

    fn insert(&mut self, finding: ImportFinding) {
        if self.items.len() < self.limit {
            self.items.insert(finding);
        } else if !self.items.contains(&finding) {
            self.items.insert(ImportFinding {
                kind: ImportFindingKind::FindingsTruncated,
                part: String::new(),
                element: String::new(),
                attribute: None,
            });
        }
    }

    /// Everything noticed, in order.
    pub fn iter(&self) -> impl Iterator<Item = &ImportFinding> {
        self.items.iter()
    }

    /// Whether a finding of this kind names this element.
    pub fn has(&self, kind: ImportFindingKind, element: &str) -> bool {
        self.items
            .iter()
            .any(|f| f.kind == kind && f.element == element)
    }

    /// How many distinct findings there are.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether nothing was noticed.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// The namespace declarations in scope. Shared until an element declares its own.
#[derive(Debug, Clone, Default)]
pub struct Scope(Rc<Vec<(String, String)>>);

impl Scope {
    /// The namespace a prefix means here. `""` asks for the default namespace.
    pub fn uri(&self, prefix: &str) -> Option<&str> {
        if prefix == "xml" {
            return Some("http://www.w3.org/XML/1998/namespace");
        }
        self.0
            .iter()
            .rev()
            .find(|(p, _)| p == prefix)
            .map(|(_, u)| u.as_str())
            .filter(|u| !u.is_empty())
    }

    /// A prefix in scope for any of these namespaces, and not shadowed. `""` means
    /// the default namespace, which an attribute cannot use.
    pub fn prefix_for(&self, uris: &[&str], for_attribute: bool) -> Option<&str> {
        self.0
            .iter()
            .rev()
            .map(|(p, _)| p.as_str())
            .filter(|p| !(for_attribute && p.is_empty()))
            .find(|p| self.uri(p).is_some_and(|u| uris.contains(&u)))
    }
}

/// An element as a reader sees it: its name resolved, its attributes taken as they
/// are used, so whatever is left at the end is what was not modelled.
#[derive(Debug, Clone)]
pub struct Elem {
    /// The node.
    pub id: ElementId,
    /// The local name.
    pub local: String,
    /// The namespace, if the prefix resolves.
    pub uri: Option<String>,
    /// What is in scope inside this element.
    pub scope: Scope,
    attrs: Vec<(String, String)>,
}

impl Elem {
    /// Read an element given what is in scope around it.
    pub fn read(
        tree: &XmlTree,
        id: ElementId,
        parent: &Scope,
        part: &str,
    ) -> Result<Self, OoxmlError> {
        let attrs = tree.attributes(id, part)?;
        let declared = attrs
            .iter()
            .filter_map(|(k, v)| {
                let prefix = match k.as_str() {
                    "xmlns" => "",
                    other => other.strip_prefix("xmlns:")?,
                };
                Some((prefix.to_owned(), v.clone()))
            })
            .collect::<Vec<_>>();
        let scope = if declared.is_empty() {
            parent.clone()
        } else {
            let mut all = parent.0.as_ref().clone();
            all.extend(declared);
            Scope(Rc::new(all))
        };
        let qualified = tree.qualified_name(id);
        let (prefix, local) = qualified.split_once(':').unwrap_or(("", qualified));
        let uri = scope.uri(prefix).map(str::to_owned);
        let local = local.to_owned();
        Ok(Self {
            id,
            local,
            uri,
            scope,
            attrs,
        })
    }

    /// The root element of a part.
    pub fn root(tree: &XmlTree, part: &str) -> Result<Self, OoxmlError> {
        Self::read(tree, tree.root(), &Scope::default(), part)
    }

    /// The child elements, in order.
    pub fn children(&self, tree: &XmlTree, part: &str) -> Result<Vec<Self>, OoxmlError> {
        tree.child_elements(self.id)
            .into_iter()
            .map(|c| Self::read(tree, c, &self.scope, part))
            .collect()
    }

    /// The first child with this name, as a reader would use it.
    pub fn first_child(
        &self,
        tree: &XmlTree,
        part: &str,
        uris: &[&str],
        local: &str,
    ) -> Result<Option<Self>, OoxmlError> {
        for c in tree.child_elements(self.id) {
            let child = Self::read(tree, c, &self.scope, part)?;
            if child.is(uris, local) {
                return Ok(Some(child));
            }
        }
        Ok(None)
    }

    /// Whether this element has this name.
    pub fn is(&self, uris: &[&str], local: &str) -> bool {
        self.local == local && self.in_namespace(uris)
    }

    /// Whether this element is in one of these namespaces.
    pub fn in_namespace(&self, uris: &[&str]) -> bool {
        self.uri.as_deref().is_some_and(|u| uris.contains(&u))
    }

    /// An attribute not yet taken, by the name as written.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// Take an attribute by the name as written, so `id` never finds `r:id`.
    pub fn take(&mut self, name: &str) -> Option<String> {
        let at = self.attrs.iter().position(|(k, _)| k == name)?;
        Some(self.attrs.remove(at).1)
    }

    /// Take an attribute by namespace and local name.
    pub fn take_ns(&mut self, uris: &[&str], local: &str) -> Option<String> {
        let at = self.attrs.iter().position(|(k, _)| {
            k.split_once(':').is_some_and(|(p, l)| {
                p != "xmlns" && l == local && self.scope.uri(p).is_some_and(|u| uris.contains(&u))
            })
        })?;
        Some(self.attrs.remove(at).1)
    }

    /// The name as written of an attribute, by namespace and local name.
    pub fn name_of_ns(&self, uris: &[&str], local: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| {
                k.split_once(':').is_some_and(|(p, l)| {
                    p != "xmlns"
                        && l == local
                        && self.scope.uri(p).is_some_and(|u| uris.contains(&u))
                })
            })
            .map(|(k, _)| k.as_str())
    }
}

/// One part being read.
#[derive(Debug)]
pub struct Reader<'a> {
    /// The tree.
    pub tree: &'a XmlTree,
    /// The part's name.
    pub part: &'a str,
    /// The bounds.
    pub limits: &'a ImportLimits,
    /// The report.
    pub findings: &'a mut Findings,
    items: usize,
}

impl<'a> Reader<'a> {
    /// Start reading a part.
    pub fn new(
        tree: &'a XmlTree,
        part: &'a str,
        limits: &'a ImportLimits,
        findings: &'a mut Findings,
    ) -> Self {
        Self {
            tree,
            part,
            limits,
            findings,
            items: 0,
        }
    }

    /// The root, which must be this element in one of these namespaces.
    pub fn root(&self, uris: &[&str], local: &'static str) -> Result<Elem, ImportError> {
        let root = Elem::root(self.tree, self.part)?;
        if !root.is(uris, local) {
            return Err(ImportError::WrongRoot {
                part: self.part.to_owned(),
                expected: local,
            });
        }
        Ok(root)
    }

    /// The child elements, in order.
    pub fn children(&self, el: &Elem) -> Result<Vec<Elem>, ImportError> {
        Ok(el.children(self.tree, self.part)?)
    }

    /// The children, refused if there are more than `limit`.
    pub fn bounded_children(
        &self,
        el: &Elem,
        limit: usize,
        what: &'static str,
    ) -> Result<Vec<Elem>, ImportError> {
        if self.tree.child_elements(el.id).len() > limit {
            return Err(self.too_many(what, limit));
        }
        self.children(el)
    }

    /// The refusal for a count over its bound.
    pub fn too_many(&self, what: &'static str, limit: usize) -> ImportError {
        ImportError::TooMany {
            what,
            part: self.part.to_owned(),
            limit,
        }
    }

    /// Note a finding about an element.
    pub fn note(&mut self, kind: ImportFindingKind, el: &Elem, attribute: Option<&str>) {
        self.findings.note(kind, self.part, &el.local, attribute);
    }

    fn checked_len(&self, value: &str) -> Result<(), ImportError> {
        if value.len() > self.limits.max_value_bytes {
            return Err(ImportError::ValueTooLong {
                part: self.part.to_owned(),
                limit: self.limits.max_value_bytes,
            });
        }
        Ok(())
    }

    fn parse_attr<T: Lexical>(
        &mut self,
        el: &Elem,
        name: &str,
        raw: Option<String>,
        allowed: impl Fn(&T) -> bool,
    ) -> Result<Option<T>, ImportError> {
        let Some(raw) = raw else { return Ok(None) };
        self.checked_len(&raw)?;
        let Some(value) = T::parse_lexical(&raw) else {
            self.note(ImportFindingKind::MalformedValue, el, Some(name));
            return Ok(None);
        };
        if !allowed(&value) {
            self.note(ImportFindingKind::OutOfRange, el, Some(name));
            return Ok(None);
        }
        Ok(Some(value))
    }

    /// An attribute's value. A malformed one is reported and left out.
    pub fn attr<T: Lexical>(
        &mut self,
        el: &mut Elem,
        name: &str,
    ) -> Result<Option<T>, ImportError> {
        let raw = el.take(name);
        self.parse_attr(el, name, raw, |_| true)
    }

    /// As [`Self::attr`], with values outside `allowed` reported and left out too.
    pub fn attr_in<T: Lexical>(
        &mut self,
        el: &mut Elem,
        name: &str,
        allowed: impl Fn(&T) -> bool,
    ) -> Result<Option<T>, ImportError> {
        let raw = el.take(name);
        self.parse_attr(el, name, raw, allowed)
    }

    /// An attribute the schema requires. Absence is reported.
    pub fn required<T: Lexical>(
        &mut self,
        el: &mut Elem,
        name: &str,
        allowed: impl Fn(&T) -> bool,
    ) -> Result<Option<T>, ImportError> {
        let raw = el.take(name);
        if raw.is_none() {
            self.note(ImportFindingKind::MissingRequired, el, Some(name));
        }
        self.parse_attr(el, name, raw, allowed)
    }

    /// Report what the element still holds. Namespace declarations are not content.
    pub fn finish(&mut self, el: &Elem) {
        for (name, _) in &el.attrs {
            if name != "xmlns" && !name.starts_with("xmlns:") {
                self.findings.note(
                    ImportFindingKind::RetainedAttribute,
                    self.part,
                    &el.local,
                    Some(name),
                );
            }
        }
    }

    /// Report an element nothing here models.
    pub fn retained(&mut self, el: &Elem) {
        self.note(ImportFindingKind::RetainedElement, el, None);
    }

    /// An element's text, with no child elements expected.
    pub fn text(&mut self, el: &Elem) -> Result<Option<String>, ImportError> {
        if !self.tree.child_elements(el.id).is_empty() {
            self.note(ImportFindingKind::MalformedValue, el, None);
            return Ok(None);
        }
        let Some(text) = self.tree.own_text(el.id) else {
            self.note(ImportFindingKind::MalformedValue, el, None);
            return Ok(None);
        };
        self.checked_len(&text)?;
        Ok(Some(text))
    }

    /// An element's text as a typed value.
    pub fn typed_text<T: Lexical>(&mut self, el: &Elem) -> Result<Option<T>, ImportError> {
        let Some(text) = self.text(el)? else {
            return Ok(None);
        };
        let value = T::parse_lexical(&text);
        if value.is_none() {
            self.note(ImportFindingKind::MalformedValue, el, None);
        }
        Ok(value)
    }

    fn charge_items(&mut self, n: usize) -> Result<(), ImportError> {
        self.items = self.items.saturating_add(n);
        if self.items > self.limits.max_variant_items {
            return Err(self.too_many("variant items", self.limits.max_variant_items));
        }
        Ok(())
    }
}

/// Fill a slot from the first occurrence of an element, and report any other.
/// `true` if this is the first.
fn first_time(
    r: &mut Reader<'_>,
    seen: &mut HashSet<&'static str>,
    name: &'static str,
    el: &Elem,
) -> bool {
    if seen.insert(name) {
        true
    } else {
        r.note(ImportFindingKind::DuplicateElement, el, None);
        false
    }
}

/// Read `coreProperties`.
pub fn import_core_properties(
    tree: &XmlTree,
    part: &str,
    limits: &ImportLimits,
    findings: &mut Findings,
) -> Result<CoreProperties, ImportError> {
    let mut r = Reader::new(tree, part, limits, findings);
    let root = r.root(ns::CORE, "coreProperties")?;
    r.finish(&root);
    let mut core = CoreProperties::default();
    let mut seen = HashSet::new();
    for mut el in r.children(&root)? {
        let name: &'static str = match (
            el.in_namespace(ns::CORE),
            el.in_namespace(ns::DC),
            el.in_namespace(ns::DCTERMS),
            el.local.as_str(),
        ) {
            (true, _, _, "category") => "category",
            (true, _, _, "contentStatus") => "contentStatus",
            (true, _, _, "keywords") => "keywords",
            (true, _, _, "lastModifiedBy") => "lastModifiedBy",
            (true, _, _, "lastPrinted") => "lastPrinted",
            (true, _, _, "revision") => "revision",
            (true, _, _, "version") => "version",
            (_, true, _, "creator") => "creator",
            (_, true, _, "description") => "description",
            (_, true, _, "identifier") => "identifier",
            (_, true, _, "language") => "language",
            (_, true, _, "subject") => "subject",
            (_, true, _, "title") => "title",
            (_, _, true, "created") => "created",
            (_, _, true, "modified") => "modified",
            _ => {
                r.retained(&el);
                continue;
            }
        };
        if !first_time(&mut r, &mut seen, name, &el) {
            continue;
        }
        match name {
            "category" => core.category = r.text(&el)?,
            "contentStatus" => core.content_status = r.text(&el)?,
            "creator" => core.creator = r.text(&el)?,
            "description" => core.description = r.text(&el)?,
            "identifier" => core.identifier = r.text(&el)?,
            "language" => core.language = r.text(&el)?,
            "lastModifiedBy" => core.last_modified_by = r.text(&el)?,
            "revision" => core.revision = r.text(&el)?,
            "subject" => core.subject = r.text(&el)?,
            "title" => core.title = r.text(&el)?,
            "version" => core.version = r.text(&el)?,
            "lastPrinted" => core.last_printed = r.typed_text::<W3cDateTime>(&el)?,
            "created" | "modified" => {
                // OPC requires this type on both.
                let declared = el.take_ns(ns::XSI, "type");
                let is_w3cdtf = declared.as_deref().is_some_and(|t| {
                    t.split_once(':').is_some_and(|(p, l)| {
                        l == "W3CDTF" && el.scope.uri(p).is_some_and(|u| ns::DCTERMS.contains(&u))
                    })
                });
                if !is_w3cdtf {
                    r.note(ImportFindingKind::MissingRequired, &el, Some("xsi:type"));
                }
                let value = r.typed_text::<W3cDateTime>(&el)?;
                if name == "created" {
                    core.created = value;
                } else {
                    core.modified = value;
                }
            }
            _ => core.keywords = read_keywords(&mut r, &mut el)?,
        }
        r.finish(&el);
    }
    Ok(core)
}

fn read_keywords(r: &mut Reader<'_>, el: &mut Elem) -> Result<Option<Keywords>, ImportError> {
    let lang = r.attr::<String>(el, "xml:lang")?;
    let Some(text) = r.tree.own_text(el.id) else {
        r.note(ImportFindingKind::MalformedValue, el, None);
        return Ok(None);
    };
    r.checked_len(&text)?;
    let mut values = Vec::new();
    for mut child in r.bounded_children(el, r.limits.max_keywords, "keywords")? {
        if !child.is(ns::CORE, "value") {
            r.retained(&child);
            continue;
        }
        let lang = r.attr::<String>(&mut child, "xml:lang")?;
        let Some(text) = r.text(&child)? else {
            return Ok(None);
        };
        r.finish(&child);
        values.push(Keyword { lang, text });
    }
    Ok(Some(Keywords { lang, text, values }))
}

/// Read extended `Properties`.
pub fn import_app_properties(
    tree: &XmlTree,
    part: &str,
    limits: &ImportLimits,
    findings: &mut Findings,
) -> Result<AppProperties, ImportError> {
    let mut r = Reader::new(tree, part, limits, findings);
    let root = r.root(ns::EP, "Properties")?;
    r.finish(&root);
    let mut app = AppProperties::default();
    let mut seen = HashSet::new();
    for el in r.children(&root)? {
        if !el.in_namespace(ns::EP) {
            r.retained(&el);
            continue;
        }
        macro_rules! field {
            ($name:literal, $slot:ident, $t:ty) => {{
                if first_time(&mut r, &mut seen, $name, &el) {
                    app.$slot = r.typed_text::<$t>(&el)?;
                    r.finish(&el);
                }
            }};
        }
        macro_rules! vector {
            ($name:literal, $slot:ident) => {{
                if first_time(&mut r, &mut seen, $name, &el) {
                    app.$slot = read_vector_holder(&mut r, &el)?;
                    r.finish(&el);
                }
            }};
        }
        match el.local.as_str() {
            "Template" => field!("Template", template, String),
            "Manager" => field!("Manager", manager, String),
            "Company" => field!("Company", company, String),
            "Pages" => field!("Pages", pages, i32),
            "Words" => field!("Words", words, i32),
            "Characters" => field!("Characters", characters, i32),
            "PresentationFormat" => field!("PresentationFormat", presentation_format, String),
            "Lines" => field!("Lines", lines, i32),
            "Paragraphs" => field!("Paragraphs", paragraphs, i32),
            "Slides" => field!("Slides", slides, i32),
            "Notes" => field!("Notes", notes, i32),
            "TotalTime" => field!("TotalTime", total_time, i32),
            "HiddenSlides" => field!("HiddenSlides", hidden_slides, i32),
            "MMClips" => field!("MMClips", mm_clips, i32),
            "ScaleCrop" => field!("ScaleCrop", scale_crop, bool),
            "HeadingPairs" => vector!("HeadingPairs", heading_pairs),
            "TitlesOfParts" => vector!("TitlesOfParts", titles_of_parts),
            "LinksUpToDate" => field!("LinksUpToDate", links_up_to_date, bool),
            "CharactersWithSpaces" => field!("CharactersWithSpaces", characters_with_spaces, i32),
            "SharedDoc" => field!("SharedDoc", shared_doc, bool),
            "HyperlinkBase" => field!("HyperlinkBase", hyperlink_base, String),
            "HLinks" => vector!("HLinks", hlinks),
            "HyperlinksChanged" => field!("HyperlinksChanged", hyperlinks_changed, bool),
            "DigSig" => {
                if first_time(&mut r, &mut seen, "DigSig", &el) {
                    app.dig_sig = read_dig_sig(&mut r, &el)?;
                    r.finish(&el);
                }
            }
            "Application" => field!("Application", application, String),
            "AppVersion" => field!("AppVersion", app_version, String),
            "DocSecurity" => field!("DocSecurity", doc_security, i32),
            _ => r.retained(&el),
        }
    }
    Ok(app)
}

fn read_dig_sig(r: &mut Reader<'_>, el: &Elem) -> Result<Option<String>, ImportError> {
    for child in r.children(el)? {
        if child.is(ns::VT, "blob") {
            return match read_variant(r, &child, 0)? {
                Some(Variant::Blob(data)) => Ok(Some(data)),
                _ => Ok(None),
            };
        }
    }
    r.note(ImportFindingKind::MissingRequired, el, None);
    Ok(None)
}

fn read_vector_holder(r: &mut Reader<'_>, el: &Elem) -> Result<Option<VtVector>, ImportError> {
    for child in r.children(el)? {
        if child.is(ns::VT, "vector") {
            return match read_variant(r, &child, 0)? {
                Some(Variant::Vector(v)) => Ok(Some(v)),
                _ => Ok(None),
            };
        }
    }
    r.note(ImportFindingKind::MissingRequired, el, None);
    Ok(None)
}

fn is_base64(s: &str) -> bool {
    s.bytes().all(|b| {
        b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'=' | b' ' | b'\t' | b'\r' | b'\n')
    })
}

fn is_decimal(s: &str) -> bool {
    let s = trim_xml(s);
    let s = s.strip_prefix(['+', '-']).unwrap_or(s);
    let (whole, frac) = s.split_once('.').unwrap_or((s, ""));
    (!whole.is_empty() || !frac.is_empty())
        && whole.bytes().all(|b| b.is_ascii_digit())
        && frac.bytes().all(|b| b.is_ascii_digit())
}

/// `\s*[0-9]*\.[0-9]{4}\s*`
fn is_currency(s: &str) -> bool {
    let Some((whole, frac)) = trim_xml(s).split_once('.') else {
        return false;
    };
    frac.len() == 4
        && whole.bytes().all(|b| b.is_ascii_digit())
        && frac.bytes().all(|b| b.is_ascii_digit())
}

/// `\s*0x[0-9A-Za-z]{8}\s*`
fn is_error_code(s: &str) -> bool {
    trim_xml(s)
        .strip_prefix("0x")
        .is_some_and(|h| h.len() == 8 && h.bytes().all(|b| b.is_ascii_alphanumeric()))
}

/// `ST_Guid`, in either case: the schema says upper, and lower is found in files.
pub fn is_guid(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 38
        && b[0] == b'{'
        && b[37] == b'}'
        && b[1..37].iter().enumerate().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => *c == b'-',
            _ => c.is_ascii_hexdigit(),
        })
}

/// Whether a text form of a variant is what its type says. Export relies on this
/// too: a value it would not read back is one it must not write.
pub fn variant_text_is_valid(v: &Variant) -> bool {
    match v {
        Variant::Blob(s)
        | Variant::OBlob(s)
        | Variant::Stream(s)
        | Variant::OStream(s)
        | Variant::Storage(s)
        | Variant::OStorage(s)
        | Variant::Vstream { data: s, .. } => is_base64(s),
        Variant::Decimal(s) => is_decimal(s),
        Variant::Date(s) | Variant::Filetime(s) => W3cDateTime::is_full_date_time(s),
        Variant::Cy(s) => is_currency(s),
        Variant::Error(s) => is_error_code(s),
        Variant::Clsid(s) => is_guid(trim_xml(s)),
        _ => true,
    }
}

/// One `vt:` value. A value that cannot be used is reported and is `None`, and a
/// vector or array holding one is `None` whole, as half of one would misstate it.
pub fn read_variant(
    r: &mut Reader<'_>,
    el: &Elem,
    depth: usize,
) -> Result<Option<Variant>, ImportError> {
    if depth > r.limits.max_variant_depth {
        return Err(r.too_many("variant nesting", r.limits.max_variant_depth));
    }
    if !el.in_namespace(ns::VT) {
        r.retained(el);
        return Ok(None);
    }
    macro_rules! typed {
        ($variant:ident, $t:ty) => {
            r.typed_text::<$t>(el)?.map(Variant::$variant)
        };
    }
    let mut el = el.clone();
    let value = match el.local.as_str() {
        "variant" => {
            let children = r.children(&el)?;
            match children.as_slice() {
                [only] => read_variant(r, only, depth + 1)?.map(|v| Variant::Variant(Box::new(v))),
                _ => {
                    r.note(ImportFindingKind::MalformedValue, &el, None);
                    None
                }
            }
        }
        "vector" => read_vector(r, &mut el, depth)?,
        "array" => read_array(r, &mut el, depth)?,
        "empty" => Some(Variant::Empty),
        "null" => Some(Variant::Null),
        "i1" => typed!(I1, i8),
        "i2" => typed!(I2, i16),
        "i4" => typed!(I4, i32),
        "i8" => typed!(I8, i64),
        "int" => typed!(Int, i32),
        "ui1" => typed!(Ui1, u8),
        "ui2" => typed!(Ui2, u16),
        "ui4" => typed!(Ui4, u32),
        "ui8" => typed!(Ui8, u64),
        "uint" => typed!(Uint, u32),
        "r4" => typed!(R4, Float32),
        "r8" => typed!(R8, Float64),
        "bool" => typed!(Bool, bool),
        "lpstr" => typed!(Lpstr, String),
        "lpwstr" => typed!(Lpwstr, String),
        "bstr" => typed!(Bstr, String),
        "blob" => typed!(Blob, String),
        "oblob" => typed!(OBlob, String),
        "stream" => typed!(Stream, String),
        "ostream" => typed!(OStream, String),
        "storage" => typed!(Storage, String),
        "ostorage" => typed!(OStorage, String),
        "decimal" => typed!(Decimal, String),
        "date" => typed!(Date, String),
        "filetime" => typed!(Filetime, String),
        "cy" => typed!(Cy, String),
        "error" => typed!(Error, String),
        "clsid" => typed!(Clsid, String),
        "vstream" => {
            let version = r.attr::<String>(&mut el, "version")?;
            r.text(&el)?.map(|data| Variant::Vstream { version, data })
        }
        _ => {
            r.retained(&el);
            return Ok(None);
        }
    };
    r.finish(&el);
    if value.as_ref().is_some_and(|v| !variant_text_is_valid(v)) {
        r.note(ImportFindingKind::MalformedValue, &el, None);
        return Ok(None);
    }
    Ok(value)
}

fn read_items(
    r: &mut Reader<'_>,
    el: &Elem,
    depth: usize,
) -> Result<Option<Vec<Variant>>, ImportError> {
    let children = r.children(el)?;
    r.charge_items(children.len())?;
    let mut items = Vec::new();
    let mut whole = true;
    for child in &children {
        match read_variant(r, child, depth + 1)? {
            Some(v) => items.push(v),
            None => whole = false,
        }
    }
    Ok(whole.then_some(items))
}

fn read_vector(
    r: &mut Reader<'_>,
    el: &mut Elem,
    depth: usize,
) -> Result<Option<Variant>, ImportError> {
    let base_type = r.required::<VectorBaseType>(el, "baseType", |_| true)?;
    // Only a claim. Nothing is ever sized from it.
    let size = r.required::<u32>(el, "size", |_| true)?;
    let items = read_items(r, el, depth)?;
    if let (Some(size), Some(items)) = (size, &items)
        && usize::try_from(size).ok() != Some(items.len())
    {
        r.note(ImportFindingKind::CountMismatch, el, Some("size"));
    }
    Ok(match (base_type, items) {
        (Some(base_type), Some(items)) => Some(Variant::Vector(VtVector { base_type, items })),
        _ => None,
    })
}

fn read_array(
    r: &mut Reader<'_>,
    el: &mut Elem,
    depth: usize,
) -> Result<Option<Variant>, ImportError> {
    let lower_bound = r.required::<i32>(el, "lBounds", |_| true)?;
    let upper_bound = r.required::<i32>(el, "uBounds", |_| true)?;
    let base_type = r.required::<ArrayBaseType>(el, "baseType", |_| true)?;
    let items = read_items(r, el, depth)?;
    Ok(match (lower_bound, upper_bound, base_type, items) {
        (Some(lower_bound), Some(upper_bound), Some(base_type), Some(items)) => {
            Some(Variant::Array(VtArray {
                lower_bound,
                upper_bound,
                base_type,
                items,
            }))
        }
        _ => None,
    })
}

/// One `property` element, or `None` if it has no usable `fmtid` and `pid`.
pub fn read_custom_property(
    r: &mut Reader<'_>,
    el: &Elem,
    pids: Option<&mut HashSet<i32>>,
) -> Result<Option<CustomProperty>, ImportError> {
    let mut el = el.clone();
    let fmtid = r.required::<String>(&mut el, "fmtid", |s| is_guid(trim_xml(s)))?;
    let pid = r.required::<i32>(&mut el, "pid", |_| true)?;
    let name = r.attr::<String>(&mut el, "name")?;
    let link_target = r.attr::<String>(&mut el, "linkTarget")?;
    r.finish(&el);
    let (Some(fmtid), Some(pid)) = (fmtid, pid) else {
        return Ok(None);
    };
    // The specification reserves 0 and 1. Kept, as the entry is still a property.
    if pid < 2 {
        r.note(ImportFindingKind::OutOfRange, &el, Some("pid"));
    }
    if let Some(pids) = pids
        && !pids.insert(pid)
    {
        r.note(ImportFindingKind::DuplicateId, &el, Some("pid"));
    }
    let children = r.children(&el)?;
    let value = match children.as_slice() {
        [] => {
            r.note(ImportFindingKind::MissingRequired, &el, None);
            None
        }
        [only] => read_variant(r, only, 0)?,
        [first, ..] => {
            r.note(ImportFindingKind::DuplicateElement, &el, None);
            read_variant(r, first, 0)?
        }
    };
    Ok(Some(CustomProperty {
        fmtid,
        pid,
        name,
        link_target,
        value,
    }))
}

/// Read custom `Properties`.
pub fn import_custom_properties(
    tree: &XmlTree,
    part: &str,
    limits: &ImportLimits,
    findings: &mut Findings,
) -> Result<CustomProperties, ImportError> {
    let mut r = Reader::new(tree, part, limits, findings);
    let root = r.root(ns::CP, "Properties")?;
    r.finish(&root);
    let mut properties = Vec::new();
    let mut pids = HashSet::new();
    let children = r.bounded_children(&root, limits.max_custom_properties, "custom properties")?;
    for el in &children {
        if !el.is(ns::CP, "property") {
            r.retained(el);
            continue;
        }
        if let Some(p) = read_custom_property(&mut r, el, Some(&mut pids))? {
            properties.push(p);
        }
    }
    Ok(CustomProperties { properties })
}
