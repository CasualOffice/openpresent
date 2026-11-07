//! Patch document properties into an [`XmlTree`], and the helpers every Tier 1A and
//! 1B writer here shares (ADR-013).
//!
//! A part is never rebuilt from the model. What the tree holds now is read as the
//! model would see it; only what differs from the wanted value is touched, so an
//! unedited model changes nothing, and everything unmodelled stays where it is.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

use casual_slide_import::properties::{
    Elem, Findings, ImportError, ImportLimits, Reader, import_app_properties,
    import_core_properties, import_custom_properties, ns, read_custom_property,
    variant_text_is_valid,
};
use casual_slide_model::properties::{
    AppProperties, CoreProperties, CustomProperties, CustomProperty, Keywords, Lexical, Variant,
    VtVector, W3cDateTime,
};
use casual_slide_ooxml::{ElementId, OoxmlError, XmlTree};

/// Why a model could not be written. Nothing was saved. Names parts, never content.
#[derive(Debug)]
#[non_exhaustive]
pub enum ExportError {
    /// The package or the XML refused.
    Ooxml(OoxmlError),
    /// The part could not be read back.
    Import(ImportError),
    /// A string holds a character XML 1.0 cannot carry.
    InvalidCharacter {
        /// The part.
        part: String,
    },
    /// A new value is outside its type's range.
    ValueOutOfRange {
        /// What the value is.
        what: &'static str,
        /// The part.
        part: String,
    },
    /// A new `r:id` names no relationship in the part.
    UnknownRelationship {
        /// The part.
        part: String,
    },
    /// The element to edit is not well formed.
    MalformedElement {
        /// The part.
        part: String,
    },
    /// The model drops a whole part, which this tier does not do.
    PartRemovalUnsupported {
        /// The part.
        part: String,
    },
    /// The model has a value for a part that exists and cannot be read.
    UnreadablePart {
        /// The part.
        part: String,
    },
    /// Reading the patched part back did not give the wanted model. Refused, as
    /// the bytes would not say what the caller asked.
    RoundTrip {
        /// The part.
        part: String,
    },
    /// A variant nested deeper than the reader accepts.
    VariantTooDeep {
        /// The part.
        part: String,
    },
    /// The model asks for an edit this tier does not write, or one that would leave
    /// required content out.
    UnsupportedEdit {
        /// What.
        what: &'static str,
        /// The part.
        part: String,
    },
}

impl ExportError {
    /// The stable diagnostic code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Ooxml(err) => err.code(),
            Self::Import(err) => err.code(),
            Self::InvalidCharacter { .. } => "CS-EXP-0001",
            Self::ValueOutOfRange { .. } => "CS-EXP-0002",
            Self::UnknownRelationship { .. } => "CS-EXP-0003",
            Self::MalformedElement { .. } => "CS-EXP-0004",
            Self::PartRemovalUnsupported { .. } => "CS-EXP-0005",
            Self::UnreadablePart { .. } => "CS-EXP-0006",
            Self::RoundTrip { .. } => "CS-EXP-0007",
            Self::VariantTooDeep { .. } => "CS-EXP-0008",
            Self::UnsupportedEdit { .. } => "CS-EXP-0009",
        }
    }
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.code();
        match self {
            Self::Ooxml(err) => write!(f, "{err}"),
            Self::Import(err) => write!(f, "{err}"),
            Self::InvalidCharacter { part } => {
                write!(f, "[{code}] {part:?}: a character XML cannot carry")
            }
            Self::ValueOutOfRange { what, part } => {
                write!(f, "[{code}] {part:?}: {what} out of range")
            }
            Self::UnknownRelationship { part } => {
                write!(f, "[{code}] {part:?}: an r:id with no relationship")
            }
            Self::MalformedElement { part } => {
                write!(f, "[{code}] {part:?}: element to edit is not well formed")
            }
            Self::PartRemovalUnsupported { part } => {
                write!(
                    f,
                    "[{code}] {part:?}: removing a whole part is not supported"
                )
            }
            Self::UnreadablePart { part } => {
                write!(
                    f,
                    "[{code}] {part:?}: cannot edit a part that cannot be read"
                )
            }
            Self::RoundTrip { part } => {
                write!(
                    f,
                    "[{code}] {part:?}: patched part does not read back as asked"
                )
            }
            Self::VariantTooDeep { part } => {
                write!(f, "[{code}] {part:?}: variant nested too deeply")
            }
            Self::UnsupportedEdit { what, part } => {
                write!(f, "[{code}] {part:?}: editing {what} is not supported")
            }
        }
    }
}

impl std::error::Error for ExportError {}

impl From<OoxmlError> for ExportError {
    fn from(err: OoxmlError) -> Self {
        Self::Ooxml(err)
    }
}

impl From<ImportError> for ExportError {
    fn from(err: ImportError) -> Self {
        Self::Import(err)
    }
}

/// XML 1.0's `Char` production. A control character written as-is, or as a
/// reference, makes the part not XML, which PowerPoint refuses to open.
fn is_xml_text(s: &str) -> bool {
    s.chars().all(|c| {
        matches!(c, '\t' | '\n' | '\r') || (c >= '\u{20}' && !matches!(c, '\u{FFFE}' | '\u{FFFF}'))
    })
}

/// One part being patched.
#[derive(Debug)]
pub struct Writer<'a> {
    /// The tree.
    pub tree: &'a mut XmlTree,
    /// The part's name.
    pub part: &'a str,
    /// Whether the deck is Strict, which decides namespaces and percentage forms.
    pub strict: bool,
    /// Relationship ids a new `r:id` may name. `None` when the part has no `r:id`s.
    pub known_rels: Option<HashSet<String>>,
    /// How many nodes have been changed.
    pub changes: usize,
}

impl Writer<'_> {
    /// The namespace to use for something new.
    pub fn uri<'u>(&self, uris: &[&'u str]) -> &'u str {
        let at = usize::from(self.strict && uris.len() > 1);
        uris.get(at).copied().unwrap_or_default()
    }

    /// Refuse a string XML cannot carry.
    pub fn check_text(&self, s: &str) -> Result<(), ExportError> {
        if is_xml_text(s) {
            Ok(())
        } else {
            Err(ExportError::InvalidCharacter {
                part: self.part.to_owned(),
            })
        }
    }

    fn set_raw(&mut self, id: ElementId, name: &str, value: &str) -> Result<(), ExportError> {
        self.check_text(value)?;
        if self.tree.set_attribute(id, name, value) {
            self.changes += 1;
            Ok(())
        } else {
            Err(ExportError::MalformedElement {
                part: self.part.to_owned(),
            })
        }
    }

    /// Bring one attribute to its wanted value, or leave it alone if it is there.
    pub fn attr<T: Lexical + PartialEq>(
        &mut self,
        el: &Elem,
        name: &str,
        before: Option<&T>,
        after: Option<&T>,
    ) -> Result<(), ExportError> {
        if before == after {
            return Ok(());
        }
        match after {
            Some(value) => self.set_raw(el.id, name, &value.to_lexical()),
            None => {
                self.tree.remove_attribute(el.id, name);
                self.changes += 1;
                Ok(())
            }
        }
    }

    /// `r:id`, under whatever prefix the part uses, checked against the part's rels.
    pub fn rel_attr(
        &mut self,
        el: &Elem,
        before: Option<&str>,
        after: Option<&str>,
    ) -> Result<(), ExportError> {
        if before == after {
            return Ok(());
        }
        let existing = el.name_of_ns(ns::REL, "id").map(str::to_owned);
        let Some(value) = after else {
            if let Some(name) = existing {
                self.tree.remove_attribute(el.id, &name);
                self.changes += 1;
            }
            return Ok(());
        };
        if self
            .known_rels
            .as_ref()
            .is_some_and(|known| !known.contains(value))
        {
            return Err(ExportError::UnknownRelationship {
                part: self.part.to_owned(),
            });
        }
        let name = match existing {
            Some(name) => name,
            None => format!("{}:id", self.declared_prefix(el, ns::REL, "r")?),
        };
        self.set_raw(el.id, &name, value)
    }

    /// A prefix for this namespace usable on `el`, declared on it if none is in scope.
    pub fn declared_prefix(
        &mut self,
        el: &Elem,
        uris: &[&str],
        preferred: &str,
    ) -> Result<String, ExportError> {
        if let Some(p) = el.scope.prefix_for(uris, true) {
            return Ok(p.to_owned());
        }
        let prefix = free_prefix(el, preferred);
        let uri = self.uri(uris);
        self.set_raw(el.id, &format!("xmlns:{prefix}"), uri)?;
        Ok(prefix)
    }

    /// A new element, not yet in the tree, named for use under `parent`.
    pub fn new_child(
        &mut self,
        parent: &Elem,
        uris: &[&str],
        preferred: &str,
        local: &str,
    ) -> Result<Elem, ExportError> {
        let id;
        match parent.scope.prefix_for(uris, false) {
            Some("") => id = self.tree.new_element(local),
            Some(p) => {
                let name = format!("{p}:{local}");
                id = self.tree.new_element(&name);
            }
            None => {
                let prefix = free_prefix(parent, preferred);
                id = self.tree.new_element(&format!("{prefix}:{local}"));
                let uri = self.uri(uris);
                self.set_raw(id, &format!("xmlns:{prefix}"), uri)?;
            }
        }
        self.changes += 1;
        Ok(Elem::read(self.tree, id, &parent.scope, self.part)?)
    }

    /// Put a new element in schema order among `parent`'s children. `order` is the
    /// parent's sequence. An element not in it is not a landmark.
    pub fn insert_in_order(
        &mut self,
        parent: &Elem,
        order: &[&str],
        local: &str,
        child: &Elem,
    ) -> Result<(), ExportError> {
        let rank = |name: &str| order.iter().position(|o| *o == name);
        let mine = rank(local);
        let before = parent
            .children(self.tree, self.part)?
            .into_iter()
            .find(|c| {
                // PresentationML holds DrawingML children, so they are landmarks too.
                // A foreign element with a schema's name is not.
                (c.uri == parent.uri || c.in_namespace(ns::PML) || c.in_namespace(ns::DML))
                    && match (rank(&c.local), mine) {
                        (Some(theirs), Some(mine)) => theirs > mine,
                        _ => false,
                    }
            })
            .map(|c| c.id);
        self.tree.insert_child(parent.id, before, child.id);
        Ok(())
    }

    /// Bring one optional child to its wanted state: removed, or created in schema
    /// order and patched. Equal values touch nothing.
    #[allow(clippy::too_many_arguments)]
    pub fn sync_child<T: PartialEq>(
        &mut self,
        parent: &Elem,
        uris: &[&str],
        preferred: &str,
        order: &[&str],
        local: &str,
        before: Option<&T>,
        after: Option<&T>,
        patch: impl FnOnce(&mut Self, &Elem, Option<&T>, &T) -> Result<(), ExportError>,
    ) -> Result<(), ExportError> {
        if before == after {
            return Ok(());
        }
        let Some(wanted) = after else {
            return self.remove_all(parent, uris, local);
        };
        let existing = parent.first_child(self.tree, self.part, uris, local)?;
        let el = match existing {
            Some(el) => el,
            None => {
                let el = self.new_child(parent, uris, preferred, local)?;
                self.insert_in_order(parent, order, local, &el)?;
                el
            }
        };
        patch(self, &el, before, wanted)
    }

    /// Remove every child of this name. A file may repeat one, and the reader would
    /// otherwise find the next, so the model's absent would not be the file's.
    pub fn remove_all(
        &mut self,
        parent: &Elem,
        uris: &[&str],
        local: &str,
    ) -> Result<(), ExportError> {
        for child in parent.children(self.tree, self.part)? {
            if child.is(uris, local) {
                self.tree.remove_child(parent.id, child.id);
                self.changes += 1;
            }
        }
        Ok(())
    }

    /// Bring a child holding one typed text value to its wanted state.
    pub fn sync_text<T: Lexical + PartialEq>(
        &mut self,
        parent: &Elem,
        uris: &[&str],
        preferred: &str,
        local: &str,
        before: Option<&T>,
        after: Option<&T>,
    ) -> Result<(), ExportError> {
        self.sync_child(
            parent,
            uris,
            preferred,
            &[],
            local,
            before,
            after,
            |w, el, _, wanted| w.set_text(el, &wanted.to_lexical()),
        )
    }

    /// Replace an element's content with text.
    pub fn set_text(&mut self, el: &Elem, value: &str) -> Result<(), ExportError> {
        self.check_text(value)?;
        self.tree.set_text(el.id, value);
        self.changes += 1;
        Ok(())
    }

    /// Bring a list of like children to the wanted list.
    ///
    /// `existing` pairs each child the model can see with its value. Children it
    /// cannot see stay where they are. An unchanged entry is not touched, one with
    /// the same `key` is patched in place, so what it holds beyond the model
    /// survives, and order changes only if the list's order did.
    pub fn sync_list<T: PartialEq + fmt::Debug>(
        &mut self,
        parent: &Elem,
        existing: &[(T, Elem)],
        after: &[T],
        key: impl Fn(&T) -> String,
        mut make: impl FnMut(&mut Self) -> Result<Elem, ExportError>,
        mut patch: impl FnMut(&mut Self, &Elem, Option<&T>, &T) -> Result<(), ExportError>,
    ) -> Result<(), ExportError> {
        // By hash, as a hostile list could otherwise cost its square. The debug
        // form is the whole value, and stays in memory.
        let mut exact: HashMap<String, VecDeque<usize>> = HashMap::new();
        let mut keyed: HashMap<String, VecDeque<usize>> = HashMap::new();
        for (i, (value, _)) in existing.iter().enumerate() {
            exact.entry(format!("{value:?}")).or_default().push_back(i);
            keyed.entry(key(value)).or_default().push_back(i);
        }
        let mut used = vec![false; existing.len()];
        let mut assigned: Vec<Option<usize>> = Vec::with_capacity(after.len());
        for value in after {
            let found = exact
                .get_mut(&format!("{value:?}"))
                .and_then(VecDeque::pop_front);
            if let Some(i) = found
                && let Some(slot) = used.get_mut(i)
            {
                *slot = true;
            }
            assigned.push(found);
        }
        let mut sequence = Vec::with_capacity(after.len());
        for (value, found) in after.iter().zip(assigned) {
            let id = match found.and_then(|i| existing.get(i)) {
                Some((_, el)) => el.id,
                None => {
                    let reuse = keyed.get_mut(&key(value)).and_then(|q| {
                        while let Some(i) = q.pop_front() {
                            if used.get(i) == Some(&false) {
                                return Some(i);
                            }
                        }
                        None
                    });
                    match reuse.and_then(|i| existing.get(i).map(|e| (i, e))) {
                        Some((i, (old, el))) => {
                            if let Some(slot) = used.get_mut(i) {
                                *slot = true;
                            }
                            patch(self, el, Some(old), value)?;
                            el.id
                        }
                        None => {
                            let el = make(self)?;
                            patch(self, &el, None, value)?;
                            el.id
                        }
                    }
                }
            };
            sequence.push(id);
        }
        let old: Vec<ElementId> = existing.iter().map(|(_, e)| e.id).collect();
        if old != sequence {
            self.tree.replace_children(parent.id, &old, &sequence);
            self.changes += 1;
        }
        Ok(())
    }
}

fn free_prefix(el: &Elem, preferred: &str) -> String {
    if el.scope.uri(preferred).is_none() {
        return preferred.to_owned();
    }
    (1u32..)
        .map(|n| format!("{preferred}{n}"))
        .find(|p| el.scope.uri(p).is_none())
        .unwrap_or_else(|| preferred.to_owned())
}

/// Read what is in a tree again, quietly, to compare with what is wanted.
fn quiet(limits: &ImportLimits) -> Findings {
    Findings::new(limits.max_findings)
}

/// After a patch, the bytes must read back as what was asked. A value the reader
/// would report and drop is caught here, before it is saved.
pub fn verify<T: PartialEq>(
    tree: &XmlTree,
    part: &str,
    limits: &casual_slide_ooxml::OoxmlLimits,
    wanted: &T,
    read: impl FnOnce(&XmlTree) -> Result<T, ImportError>,
) -> Result<(), ExportError> {
    let again = XmlTree::parse(&tree.to_bytes(), part, limits)?;
    if &read(&again)? == wanted {
        Ok(())
    } else {
        Err(ExportError::RoundTrip {
            part: part.to_owned(),
        })
    }
}

/// Patch a tree of `coreProperties` to say `after`. Returns the nodes changed.
pub fn patch_core_properties(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    after: &CoreProperties,
    limits: &ImportLimits,
) -> Result<usize, ExportError> {
    let before = import_core_properties(tree, part, limits, &mut quiet(limits))?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: None,
        changes: 0,
    };
    macro_rules! text {
        ($uris:expr, $prefix:literal, $local:literal, $field:ident) => {
            w.sync_text(
                &root,
                $uris,
                $prefix,
                $local,
                before.$field.as_ref(),
                after.$field.as_ref(),
            )?
        };
    }
    text!(ns::CORE, "cp", "category", category);
    text!(ns::CORE, "cp", "contentStatus", content_status);
    text!(ns::DC, "dc", "creator", creator);
    text!(ns::DC, "dc", "description", description);
    text!(ns::DC, "dc", "identifier", identifier);
    text!(ns::DC, "dc", "language", language);
    text!(ns::CORE, "cp", "lastModifiedBy", last_modified_by);
    text!(ns::CORE, "cp", "lastPrinted", last_printed);
    text!(ns::CORE, "cp", "revision", revision);
    text!(ns::DC, "dc", "subject", subject);
    text!(ns::DC, "dc", "title", title);
    text!(ns::CORE, "cp", "version", version);
    patch_date(
        &mut w,
        &root,
        "created",
        before.created.as_ref(),
        after.created.as_ref(),
    )?;
    patch_date(
        &mut w,
        &root,
        "modified",
        before.modified.as_ref(),
        after.modified.as_ref(),
    )?;
    w.sync_child(
        &root,
        ns::CORE,
        "cp",
        &[],
        "keywords",
        before.keywords.as_ref(),
        after.keywords.as_ref(),
        patch_keywords,
    )?;
    Ok(w.changes)
}

fn patch_date(
    w: &mut Writer<'_>,
    root: &Elem,
    local: &str,
    before: Option<&W3cDateTime>,
    after: Option<&W3cDateTime>,
) -> Result<(), ExportError> {
    w.sync_child(
        root,
        ns::DCTERMS,
        "dcterms",
        &[],
        local,
        before,
        after,
        |w, el, _, wanted| {
            // OPC requires the type on these two. One already there is left as it is.
            if el.name_of_ns(ns::XSI, "type").is_none() {
                let xsi = w.declared_prefix(el, ns::XSI, "xsi")?;
                let el = Elem::read(w.tree, el.id, &el.scope, w.part)?;
                let terms = w.declared_prefix(&el, ns::DCTERMS, "dcterms")?;
                w.set_raw(el.id, &format!("{xsi}:type"), &format!("{terms}:W3CDTF"))?;
            }
            w.set_text(el, wanted.as_str())
        },
    )
}

fn patch_keywords(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&Keywords>,
    after: &Keywords,
) -> Result<(), ExportError> {
    w.attr(
        el,
        "xml:lang",
        before.and_then(|b| b.lang.as_ref()),
        after.lang.as_ref(),
    )?;
    let same_content = before.is_some_and(|b| b.text == after.text && b.values == after.values);
    if same_content {
        return Ok(());
    }
    if after.values.is_empty() {
        return w.set_text(el, &after.text);
    }
    // The one element whose content the model holds whole: rewrite its content.
    w.set_text(el, &after.text)?;
    for keyword in &after.values {
        let child = w.new_child(el, ns::CORE, "cp", "value")?;
        w.tree.insert_child(el.id, None, child.id);
        if let Some(lang) = &keyword.lang {
            w.set_raw(child.id, "xml:lang", lang)?;
        }
        w.set_text(&child, &keyword.text)?;
    }
    Ok(())
}

/// The element name of a variant.
fn variant_local(v: &Variant) -> &'static str {
    match v {
        Variant::Variant(_) => "variant",
        Variant::Vector(_) => "vector",
        Variant::Array(_) => "array",
        Variant::Blob(_) => "blob",
        Variant::OBlob(_) => "oblob",
        Variant::Empty => "empty",
        Variant::Null => "null",
        Variant::I1(_) => "i1",
        Variant::I2(_) => "i2",
        Variant::I4(_) => "i4",
        Variant::I8(_) => "i8",
        Variant::Int(_) => "int",
        Variant::Ui1(_) => "ui1",
        Variant::Ui2(_) => "ui2",
        Variant::Ui4(_) => "ui4",
        Variant::Ui8(_) => "ui8",
        Variant::Uint(_) => "uint",
        Variant::R4(_) => "r4",
        Variant::R8(_) => "r8",
        Variant::Decimal(_) => "decimal",
        Variant::Lpstr(_) => "lpstr",
        Variant::Lpwstr(_) => "lpwstr",
        Variant::Bstr(_) => "bstr",
        Variant::Date(_) => "date",
        Variant::Filetime(_) => "filetime",
        Variant::Bool(_) => "bool",
        Variant::Cy(_) => "cy",
        Variant::Error(_) => "error",
        Variant::Stream(_) => "stream",
        Variant::OStream(_) => "ostream",
        Variant::Storage(_) => "storage",
        Variant::OStorage(_) => "ostorage",
        Variant::Vstream { .. } => "vstream",
        Variant::Clsid(_) => "clsid",
    }
}

/// The text of a variant that is only text.
fn variant_text(v: &Variant) -> Option<String> {
    Some(match v {
        Variant::Blob(s)
        | Variant::OBlob(s)
        | Variant::Decimal(s)
        | Variant::Lpstr(s)
        | Variant::Lpwstr(s)
        | Variant::Bstr(s)
        | Variant::Date(s)
        | Variant::Filetime(s)
        | Variant::Cy(s)
        | Variant::Error(s)
        | Variant::Stream(s)
        | Variant::OStream(s)
        | Variant::Storage(s)
        | Variant::OStorage(s)
        | Variant::Clsid(s) => s.clone(),
        Variant::I1(n) => n.to_lexical(),
        Variant::I2(n) => n.to_lexical(),
        Variant::I4(n) | Variant::Int(n) => n.to_lexical(),
        Variant::I8(n) => n.to_lexical(),
        Variant::Ui1(n) => n.to_lexical(),
        Variant::Ui2(n) => n.to_lexical(),
        Variant::Ui4(n) | Variant::Uint(n) => n.to_lexical(),
        Variant::Ui8(n) => n.to_lexical(),
        Variant::R4(n) => n.to_lexical(),
        Variant::R8(n) => n.to_lexical(),
        Variant::Bool(b) => b.to_lexical(),
        _ => return None,
    })
}

const MAX_WRITE_DEPTH: usize = 16;

/// Write a variant as a new last child of `parent`.
fn write_variant(
    w: &mut Writer<'_>,
    parent: &Elem,
    v: &Variant,
    depth: usize,
) -> Result<(), ExportError> {
    if depth > MAX_WRITE_DEPTH {
        return Err(ExportError::VariantTooDeep {
            part: w.part.to_owned(),
        });
    }
    if !variant_text_is_valid(v) {
        return Err(ExportError::ValueOutOfRange {
            what: "variant text",
            part: w.part.to_owned(),
        });
    }
    let el = w.new_child(parent, ns::VT, "vt", variant_local(v))?;
    w.tree.insert_child(parent.id, None, el.id);
    if let Some(text) = variant_text(v) {
        return w.set_text(&el, &text);
    }
    match v {
        Variant::Variant(inner) => write_variant(w, &el, inner, depth + 1),
        Variant::Vector(vector) => fill_vector(w, &el, vector, depth),
        Variant::Array(array) => {
            w.set_raw(el.id, "lBounds", &array.lower_bound.to_lexical())?;
            w.set_raw(el.id, "uBounds", &array.upper_bound.to_lexical())?;
            w.set_raw(el.id, "baseType", &array.base_type.to_lexical())?;
            for item in &array.items {
                write_variant(w, &el, item, depth + 1)?;
            }
            Ok(())
        }
        Variant::Vstream { version, data } => {
            if let Some(version) = version {
                w.set_raw(el.id, "version", version)?;
            }
            w.set_text(&el, data)
        }
        _ => Ok(()),
    }
}

/// Make a `vt:vector` say this: attributes set, content replaced.
fn fill_vector(
    w: &mut Writer<'_>,
    el: &Elem,
    vector: &VtVector,
    depth: usize,
) -> Result<(), ExportError> {
    w.set_raw(el.id, "baseType", &vector.base_type.to_lexical())?;
    w.set_raw(el.id, "size", &vector.items.len().to_string())?;
    w.tree.clear_content(el.id);
    for item in &vector.items {
        write_variant(w, el, item, depth + 1)?;
    }
    Ok(())
}

fn patch_vector_holder(
    w: &mut Writer<'_>,
    holder: &Elem,
    _: Option<&VtVector>,
    after: &VtVector,
) -> Result<(), ExportError> {
    let el = match holder.first_child(w.tree, w.part, ns::VT, "vector")? {
        Some(el) => el,
        None => {
            let el = w.new_child(holder, ns::VT, "vt", "vector")?;
            w.tree.insert_child(holder.id, None, el.id);
            el
        }
    };
    fill_vector(w, &el, after, 0)
}

#[allow(clippy::ptr_arg)] // The shape `sync_child` calls, with `T = String`.
fn patch_dig_sig(
    w: &mut Writer<'_>,
    holder: &Elem,
    _: Option<&String>,
    after: &String,
) -> Result<(), ExportError> {
    let el = match holder.first_child(w.tree, w.part, ns::VT, "blob")? {
        Some(el) => el,
        None => {
            let el = w.new_child(holder, ns::VT, "vt", "blob")?;
            w.tree.insert_child(holder.id, None, el.id);
            el
        }
    };
    w.set_text(&el, after)
}

/// Patch a tree of extended `Properties` to say `after`. Returns the nodes changed.
pub fn patch_app_properties(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    after: &AppProperties,
    limits: &ImportLimits,
) -> Result<usize, ExportError> {
    let before = import_app_properties(tree, part, limits, &mut quiet(limits))?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: None,
        changes: 0,
    };
    macro_rules! text {
        ($($local:literal => $field:ident),+ $(,)?) => {$(
            w.sync_text(&root, ns::EP, "ep", $local, before.$field.as_ref(), after.$field.as_ref())?;
        )+};
    }
    text!(
        "Template" => template,
        "Manager" => manager,
        "Company" => company,
        "Pages" => pages,
        "Words" => words,
        "Characters" => characters,
        "PresentationFormat" => presentation_format,
        "Lines" => lines,
        "Paragraphs" => paragraphs,
        "Slides" => slides,
        "Notes" => notes,
        "TotalTime" => total_time,
        "HiddenSlides" => hidden_slides,
        "MMClips" => mm_clips,
        "ScaleCrop" => scale_crop,
        "LinksUpToDate" => links_up_to_date,
        "CharactersWithSpaces" => characters_with_spaces,
        "SharedDoc" => shared_doc,
        "HyperlinkBase" => hyperlink_base,
        "HyperlinksChanged" => hyperlinks_changed,
        "Application" => application,
        "AppVersion" => app_version,
        "DocSecurity" => doc_security,
    );
    macro_rules! vector {
        ($($local:literal => $field:ident),+ $(,)?) => {$(
            w.sync_child(
                &root, ns::EP, "ep", &[], $local,
                before.$field.as_ref(), after.$field.as_ref(), patch_vector_holder,
            )?;
        )+};
    }
    vector!(
        "HeadingPairs" => heading_pairs,
        "TitlesOfParts" => titles_of_parts,
        "HLinks" => hlinks,
    );
    w.sync_child(
        &root,
        ns::EP,
        "ep",
        &[],
        "DigSig",
        before.dig_sig.as_ref(),
        after.dig_sig.as_ref(),
        patch_dig_sig,
    )?;
    Ok(w.changes)
}

fn patch_custom_property(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&CustomProperty>,
    after: &CustomProperty,
) -> Result<(), ExportError> {
    w.attr(el, "fmtid", before.map(|b| &b.fmtid), Some(&after.fmtid))?;
    w.attr(el, "pid", before.map(|b| &b.pid), Some(&after.pid))?;
    w.attr(
        el,
        "name",
        before.and_then(|b| b.name.as_ref()),
        after.name.as_ref(),
    )?;
    w.attr(
        el,
        "linkTarget",
        before.and_then(|b| b.link_target.as_ref()),
        after.link_target.as_ref(),
    )?;
    let old = before.and_then(|b| b.value.as_ref());
    if old == after.value.as_ref() {
        return Ok(());
    }
    let children = el.children(w.tree, w.part)?;
    // The same type and only text: change the text, so the element stays as written.
    if let (Some(old), Some(new), [only]) = (old, &after.value, children.as_slice())
        && variant_local(old) == variant_local(new)
        && let Some(text) = variant_text(new)
    {
        if !variant_text_is_valid(new) {
            return Err(ExportError::ValueOutOfRange {
                what: "variant text",
                part: w.part.to_owned(),
            });
        }
        return w.set_text(only, &text);
    }
    for child in &children {
        w.tree.remove_child(el.id, child.id);
        w.changes += 1;
    }
    if let Some(new) = &after.value {
        write_variant(w, el, new, 0)?;
    }
    Ok(())
}

/// Patch a tree of custom `Properties` to say `after`. Returns the nodes changed.
pub fn patch_custom_properties(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    after: &CustomProperties,
    limits: &ImportLimits,
) -> Result<usize, ExportError> {
    let before = import_custom_properties(tree, part, limits, &mut quiet(limits))?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part)?;
    let mut scratch = quiet(limits);
    let mut existing = Vec::new();
    {
        let mut r = Reader::new(tree, part, limits, &mut scratch);
        for el in r.children(&root)? {
            if el.is(ns::CP, "property")
                && let Some(p) = read_custom_property(&mut r, &el, None)?
            {
                existing.push((p, el));
            }
        }
    }
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: None,
        changes: 0,
    };
    w.sync_list(
        &root,
        &existing,
        &after.properties,
        |p| p.pid.to_string(),
        |w| w.new_child(&root, ns::CP, "op", "property"),
        patch_custom_property,
    )?;
    Ok(w.changes)
}

#[cfg(test)]
mod insert_order_tests {
    use super::*;

    const OPEN: &str = "<p:scale xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" \
        xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:x=\"urn:x\">";

    fn insert(body: &str) -> String {
        let xml = format!("{OPEN}{body}</p:scale>");
        let mut tree = XmlTree::parse(
            xml.as_bytes(),
            "p",
            &casual_slide_ooxml::OoxmlLimits::default(),
        )
        .unwrap();
        let root = Elem::root(&tree, "p").unwrap();
        let mut w = Writer {
            tree: &mut tree,
            part: "p",
            strict: false,
            known_rels: None,
            changes: 0,
        };
        let child = w.new_child(&root, ns::DML, "a", "sx").unwrap();
        w.insert_in_order(&root, &["sx", "sy"], "sx", &child)
            .unwrap();
        String::from_utf8(tree.to_bytes()).unwrap()
    }

    /// `T1D-03`: `a:sy` under `p:scale` is the parent's other namespace, and still comes
    /// after `a:sx`. Comparing the parent's namespace alone wrote it first, which is
    /// out of schema order.
    #[test]
    fn a_child_in_the_other_ooxml_namespace_is_a_landmark() {
        let out = insert("<a:sy n=\"1\" d=\"1\"/>");
        assert!(
            out.find("<a:sx").unwrap() < out.find("<a:sy").unwrap(),
            "{out}"
        );
    }

    #[test]
    fn a_foreign_element_with_a_schema_name_is_not_a_landmark() {
        let out = insert("<x:sy/>");
        assert!(
            out.find("<x:sy").unwrap() < out.find("<a:sx").unwrap(),
            "{out}"
        );
    }
}
