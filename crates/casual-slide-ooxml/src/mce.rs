//! Markup compatibility: ECMA-376 Part 3, clause 9.
//!
//! [`resolve_compatibility`] answers which branch of each `mc:AlternateContent`
//! applies and what is ignored, and changes nothing. The tree keeps every branch
//! byte for byte, and reading is what picks one, by one fixed rule, so a scanner
//! and the editor cannot be shown different documents by accident: the report
//! names the branches not taken, whose content is retained and never interpreted.
//!
//! The standard says a processor "shall signal" a `MustUnderstand` mismatch and
//! "should indicate" other non-conformance. Here the first is carried as
//! [`Compatibility::must_understand`] for the read boundary to refuse, and the
//! second as [`Compatibility::nonconformances`]. An element outside the understood
//! list and not ignorable is no mismatch at this layer (9.4, step 5): it is listed
//! in [`Compatibility::unknown_namespaces`], and retained.
//!
//! Depth follows the input, and is bounded by the limit `XmlTree::parse` enforces.

use std::collections::{BTreeMap, BTreeSet};

use crate::namespaces::{MARKUP_COMPATIBILITY, is_understood};
use crate::xml::{ElementId, XmlTree};

const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

/// The branch of an `mc:AlternateContent` that applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Branch {
    /// The first `Choice` whose required namespaces are all understood. `index`
    /// counts `Choice` elements from zero.
    Choice {
        /// Which `Choice`.
        index: usize,
        /// The element.
        element: ElementId,
    },
    /// No `Choice` applied, and there is a `Fallback`.
    Fallback {
        /// The element.
        element: ElementId,
    },
    /// Nothing applies, so the construct reads as absent.
    None,
}

/// How one `mc:AlternateContent` resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlternateResolution {
    /// The `AlternateContent` element.
    pub element: ElementId,
    /// What applies.
    pub branch: Branch,
    /// The branches not taken. Retained in the bytes, and never interpreted.
    pub not_taken: Vec<ElementId>,
}

/// A way a part breaks Part 3's syntax. Reported, and never a reason to guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NonConformance {
    /// `Ignorable`, `ProcessContent`, `MustUnderstand` or `Requires` names a prefix
    /// with no in-scope namespace.
    UndeclaredPrefix,
    /// One of those lists names the markup compatibility namespace itself.
    ListsCompatibilityNamespace,
    /// A `ProcessContent` token is not `prefix:local` or `prefix:*`.
    MalformedProcessContent,
    /// `ProcessContent` names a namespace not declared ignorable.
    ProcessContentNotIgnorable,
    /// An element's prefix has no in-scope namespace. [`XmlTree::parse`] refuses
    /// such a part, so this is seen only in a tree an edit has made so.
    UndeclaredElementPrefix,
    /// An `AlternateContent` with no `Choice`.
    NoChoice,
    /// A `Choice` with no `Requires`, or an empty one. It is never selected.
    MissingRequires,
    /// A `Choice` after a `Fallback`, or a second `Fallback`. Never selected.
    BranchOutOfOrder,
    /// A child of `AlternateContent` that is neither a branch nor ignored.
    UnexpectedChild,
    /// `Choice` or `Fallback` outside an `AlternateContent`.
    MisplacedBranch,
    /// An element in the markup compatibility namespace that does not exist.
    UnknownCompatibilityElement,
    /// An attribute in the markup compatibility namespace that does not exist.
    UnknownCompatibilityAttribute,
    /// A compatibility element with an attribute it may not have.
    BadCompatibilityElementAttribute,
    /// An unwrapped element with `xml:base`, `xml:lang` or `xml:space`.
    UnwrappedWithXmlAttribute,
}

impl NonConformance {
    /// A stable name, shared with `tools/f4/validate.py`.
    pub fn name(self) -> &'static str {
        match self {
            Self::UndeclaredPrefix => "undeclared-prefix",
            Self::ListsCompatibilityNamespace => "lists-compatibility-namespace",
            Self::MalformedProcessContent => "malformed-process-content",
            Self::ProcessContentNotIgnorable => "process-content-not-ignorable",
            Self::UndeclaredElementPrefix => "undeclared-element-prefix",
            Self::NoChoice => "no-choice",
            Self::MissingRequires => "missing-requires",
            Self::BranchOutOfOrder => "branch-out-of-order",
            Self::UnexpectedChild => "unexpected-child",
            Self::MisplacedBranch => "misplaced-branch",
            Self::UnknownCompatibilityElement => "unknown-compatibility-element",
            Self::UnknownCompatibilityAttribute => "unknown-compatibility-attribute",
            Self::BadCompatibilityElementAttribute => "bad-compatibility-element-attribute",
            Self::UnwrappedWithXmlAttribute => "unwrapped-with-xml-attribute",
        }
    }
}

/// What markup compatibility says about one part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Compatibility {
    /// Each `AlternateContent` reached, in document order. One inside a branch not
    /// taken, an ignored element or an extension is not reached.
    pub alternates: Vec<AlternateResolution>,
    /// Elements removed with their content (9.2).
    pub ignored_elements: Vec<ElementId>,
    /// Elements replaced by their content (9.2).
    pub unwrapped_elements: Vec<ElementId>,
    /// How many attributes are ignored.
    pub ignored_attributes: usize,
    /// `extLst` extensions, whose content this processor leaves alone (clause 8).
    pub extensions: Vec<ElementId>,
    /// Namespaces declared must-understand, reached, and not understood.
    pub must_understand: BTreeSet<String>,
    /// Non-ignorable namespaces in use that are not understood. Retained.
    pub unknown_namespaces: BTreeSet<String>,
    /// How the part breaks Part 3's syntax.
    pub nonconformances: BTreeSet<NonConformance>,
}

/// Resolve a part against the fixed list in [`is_understood`]. Pure: the tree is
/// not touched, and the same tree always gives the same answer.
pub fn resolve_compatibility(tree: &XmlTree) -> Compatibility {
    resolve_with(tree, &is_understood)
}

/// Resolve a part against any application configuration, as Part 3 calls the set
/// of understood namespaces.
pub fn resolve_with(tree: &XmlTree, understood: &dyn Fn(&str) -> bool) -> Compatibility {
    let mut resolver = Resolver {
        tree,
        understood,
        prefixes: BTreeMap::new(),
        ignorable: BTreeMap::new(),
        process: BTreeMap::new(),
        out: Compatibility::default(),
    };
    resolver.visit(tree.root());
    resolver.out
}

/// What entering an element added to the scope, for leaving it.
#[derive(Default)]
struct Undo {
    prefixes: Vec<String>,
    ignorable: Vec<String>,
    process: Vec<(String, String)>,
}

struct Attribute {
    /// `None` when unqualified.
    namespace: Option<String>,
    local: String,
    value: String,
}

/// Scope lookups are by map, so a hostile pile of declarations costs a logarithm
/// per name and not a scan of everything in scope.
struct Resolver<'a> {
    tree: &'a XmlTree,
    understood: &'a dyn Fn(&str) -> bool,
    prefixes: BTreeMap<String, Vec<String>>,
    ignorable: BTreeMap<String, usize>,
    process: BTreeMap<(String, String), usize>,
    out: Compatibility,
}

impl Resolver<'_> {
    fn knows(&self, uri: &str) -> bool {
        uri == MARKUP_COMPATIBILITY || uri == XML_NS || (self.understood)(uri)
    }

    fn lookup(&self, prefix: &str) -> Option<&str> {
        if prefix == "xml" {
            return Some(XML_NS);
        }
        self.prefixes
            .get(prefix)
            .and_then(|stack| stack.last())
            .map(String::as_str)
            .filter(|uri| !uri.is_empty())
    }

    fn declares_ignorable(&self, uri: &str) -> bool {
        self.ignorable.contains_key(uri)
    }

    /// Bring an element's declarations into scope, and read its attributes.
    fn enter(&mut self, id: ElementId) -> (Vec<Attribute>, Undo) {
        let raw = self.tree.declared_attributes(id);
        let mut undo = Undo::default();
        for (name, value) in &raw {
            let prefix = match name.split_once(':') {
                Some(("xmlns", prefix)) => prefix,
                None if name == "xmlns" => "",
                _ => continue,
            };
            self.prefixes
                .entry(prefix.to_owned())
                .or_default()
                .push(value.clone());
            undo.prefixes.push(prefix.to_owned());
        }
        let mut attributes = Vec::with_capacity(raw.len());
        for (name, value) in raw {
            if name == "xmlns" || name.starts_with("xmlns:") {
                continue;
            }
            let (namespace, local) = match name.split_once(':') {
                Some((prefix, local)) => (
                    Some(self.lookup(prefix).unwrap_or_default().to_owned()),
                    local.to_owned(),
                ),
                None => (None, name),
            };
            attributes.push(Attribute {
                namespace,
                local,
                value,
            });
        }
        for a in &attributes {
            if a.namespace.as_deref() != Some(MARKUP_COMPATIBILITY) || a.local != "Ignorable" {
                continue;
            }
            for prefix in a.value.split_ascii_whitespace() {
                if let Some(uri) = self.list_entry(prefix) {
                    *self.ignorable.entry(uri.clone()).or_default() += 1;
                    undo.ignorable.push(uri);
                }
            }
        }
        for a in &attributes {
            if a.namespace.as_deref() != Some(MARKUP_COMPATIBILITY) || a.local != "ProcessContent" {
                continue;
            }
            for token in a.value.split_ascii_whitespace() {
                let Some((prefix, local)) = token
                    .split_once(':')
                    .filter(|(p, l)| !p.is_empty() && !l.is_empty() && !l.contains(':'))
                else {
                    self.out
                        .nonconformances
                        .insert(NonConformance::MalformedProcessContent);
                    continue;
                };
                let Some(uri) = self.list_entry(prefix) else {
                    continue;
                };
                if !self.declares_ignorable(&uri) {
                    self.out
                        .nonconformances
                        .insert(NonConformance::ProcessContentNotIgnorable);
                    continue;
                }
                let pair = (uri, local.to_owned());
                *self.process.entry(pair.clone()).or_default() += 1;
                undo.process.push(pair);
            }
        }
        (attributes, undo)
    }

    fn leave(&mut self, undo: Undo) {
        for prefix in undo.prefixes {
            if let Some(stack) = self.prefixes.get_mut(&prefix) {
                stack.pop();
                if stack.is_empty() {
                    self.prefixes.remove(&prefix);
                }
            }
        }
        for uri in undo.ignorable {
            release(&mut self.ignorable, uri);
        }
        for pair in undo.process {
            release(&mut self.process, pair);
        }
    }

    /// A prefix from a compatibility list, as a namespace that may be listed.
    fn list_entry(&mut self, prefix: &str) -> Option<String> {
        let uri = self.lookup(prefix).map(str::to_owned);
        match uri.as_deref() {
            None => {
                self.out
                    .nonconformances
                    .insert(NonConformance::UndeclaredPrefix);
                None
            }
            Some(MARKUP_COMPATIBILITY) => {
                self.out
                    .nonconformances
                    .insert(NonConformance::ListsCompatibilityNamespace);
                None
            }
            Some(_) => uri,
        }
    }

    /// An element's namespace, and its local name.
    fn name_of(&mut self, id: ElementId) -> (Option<String>, String) {
        let qualified = self.tree.qualified_name(id).to_owned();
        match qualified.split_once(':') {
            Some((prefix, local)) => {
                let uri = self.lookup(prefix).map(str::to_owned);
                if uri.is_none() {
                    self.out
                        .nonconformances
                        .insert(NonConformance::UndeclaredElementPrefix);
                }
                (uri, local.to_owned())
            }
            None => (self.lookup("").map(str::to_owned), qualified),
        }
    }

    fn is_process_content(&self, uri: &str, local: &str) -> bool {
        let pair = |l: &str| (uri.to_owned(), l.to_owned());
        self.process.contains_key(&pair(local)) || self.process.contains_key(&pair("*"))
    }

    fn check_must_understand(&mut self, attributes: &[Attribute]) {
        for a in attributes {
            if a.namespace.as_deref() != Some(MARKUP_COMPATIBILITY) || a.local != "MustUnderstand" {
                continue;
            }
            for prefix in a.value.split_ascii_whitespace() {
                if let Some(uri) = self.list_entry(prefix)
                    && !self.knows(&uri)
                {
                    self.out.must_understand.insert(uri);
                }
            }
        }
    }

    fn visit_children(&mut self, id: ElementId) {
        for child in self.tree.child_elements(id) {
            self.visit(child);
        }
    }

    fn visit(&mut self, id: ElementId) {
        let (attributes, undo) = self.enter(id);
        self.visit_entered(id, &attributes);
        self.leave(undo);
    }

    fn visit_entered(&mut self, id: ElementId, attributes: &[Attribute]) {
        let (namespace, local) = self.name_of(id);
        let namespace = namespace.as_deref();

        if let Some(uri) = namespace
            && !self.knows(uri)
            && self.declares_ignorable(uri)
        {
            if !self.is_process_content(uri, &local) {
                self.out.ignored_elements.push(id);
                return;
            }
            self.out.unwrapped_elements.push(id);
            self.check_must_understand(attributes);
            let has_xml_attribute = attributes.iter().any(|a| {
                a.namespace.as_deref() == Some(XML_NS)
                    && matches!(a.local.as_str(), "base" | "lang" | "space")
            });
            if has_xml_attribute {
                self.out
                    .nonconformances
                    .insert(NonConformance::UnwrappedWithXmlAttribute);
            }
            self.visit_children(id);
            return;
        }

        if namespace == Some(MARKUP_COMPATIBILITY) {
            match local.as_str() {
                "AlternateContent" => {
                    self.alternate(id, attributes);
                    return;
                }
                "Choice" | "Fallback" => {
                    self.out
                        .nonconformances
                        .insert(NonConformance::MisplacedBranch);
                }
                _ => {
                    self.out
                        .nonconformances
                        .insert(NonConformance::UnknownCompatibilityElement);
                }
            }
        } else if let Some(uri) = namespace
            && !self.knows(uri)
        {
            self.out.unknown_namespaces.insert(uri.to_owned());
        }

        if local == "ext"
            && namespace.is_some_and(|uri| self.knows(uri))
            && self
                .tree
                .parent_element(id)
                .is_some_and(|parent| self.tree.local_name(parent) == "extLst")
        {
            self.out.extensions.push(id);
            return;
        }

        self.check_must_understand(attributes);
        for a in attributes {
            match a.namespace.as_deref() {
                None | Some("") => {}
                Some(MARKUP_COMPATIBILITY) => {
                    if !matches!(
                        a.local.as_str(),
                        "Ignorable"
                            | "ProcessContent"
                            | "MustUnderstand"
                            | "PreserveElements"
                            | "PreserveAttributes"
                    ) {
                        self.out
                            .nonconformances
                            .insert(NonConformance::UnknownCompatibilityAttribute);
                    }
                }
                Some(uri) if self.knows(uri) => {}
                Some(uri) if self.declares_ignorable(uri) => self.out.ignored_attributes += 1,
                Some(uri) => {
                    self.out.unknown_namespaces.insert(uri.to_owned());
                }
            }
        }
        self.visit_children(id);
    }

    /// Check a compatibility element's attributes (7.1, 7.5, 7.6 and 7.7).
    fn check_branch_attributes(&mut self, attributes: &[Attribute], allowed_unqualified: &str) {
        for a in attributes {
            let ok = match a.namespace.as_deref() {
                None => a.local == allowed_unqualified,
                Some(MARKUP_COMPATIBILITY) => true,
                Some(XML_NS | "") => false,
                Some(uri) => self.declares_ignorable(uri),
            };
            if !ok {
                self.out
                    .nonconformances
                    .insert(NonConformance::BadCompatibilityElementAttribute);
            }
        }
    }

    fn alternate(&mut self, id: ElementId, attributes: &[Attribute]) {
        self.check_branch_attributes(attributes, "");
        self.check_must_understand(attributes);

        let mut branch = Branch::None;
        let mut not_taken = Vec::new();
        let mut choices = 0usize;
        let mut fallback_seen = false;
        let mut selected = None;

        for child in self.tree.child_elements(id) {
            let (child_attributes, undo) = self.enter(child);
            let (namespace, local) = self.name_of(child);
            if namespace.as_deref() == Some(MARKUP_COMPATIBILITY) && local == "Choice" {
                let index = choices;
                choices += 1;
                self.check_branch_attributes(&child_attributes, "Requires");
                if fallback_seen {
                    self.out
                        .nonconformances
                        .insert(NonConformance::BranchOutOfOrder);
                }
                let satisfied = self.requires_satisfied(&child_attributes);
                if selected.is_none() && !fallback_seen && satisfied {
                    selected = Some(child);
                    branch = Branch::Choice {
                        index,
                        element: child,
                    };
                } else {
                    not_taken.push(child);
                }
            } else if namespace.as_deref() == Some(MARKUP_COMPATIBILITY) && local == "Fallback" {
                self.check_branch_attributes(&child_attributes, "");
                if fallback_seen {
                    self.out
                        .nonconformances
                        .insert(NonConformance::BranchOutOfOrder);
                }
                if selected.is_none() && !fallback_seen {
                    selected = Some(child);
                    branch = Branch::Fallback { element: child };
                } else {
                    not_taken.push(child);
                }
                fallback_seen = true;
            } else if namespace
                .as_deref()
                .is_some_and(|uri| !self.knows(uri) && self.declares_ignorable(uri))
            {
                self.out.ignored_elements.push(child);
            } else {
                self.out
                    .nonconformances
                    .insert(NonConformance::UnexpectedChild);
            }
            self.leave(undo);
        }
        if choices == 0 {
            self.out.nonconformances.insert(NonConformance::NoChoice);
        }
        self.out.alternates.push(AlternateResolution {
            element: id,
            branch,
            not_taken,
        });
        if let Some(element) = selected {
            let (branch_attributes, undo) = self.enter(element);
            self.check_must_understand(&branch_attributes);
            self.visit_children(element);
            self.leave(undo);
        }
    }

    /// Whether every namespace a `Choice` requires is understood. A `Choice` with
    /// no list, an empty one, or a prefix that binds nothing is not selectable.
    fn requires_satisfied(&mut self, attributes: &[Attribute]) -> bool {
        let requires = attributes
            .iter()
            .find(|a| a.namespace.is_none() && a.local == "Requires")
            .map(|a| a.value.as_str())
            .unwrap_or_default();
        let mut any = false;
        let mut satisfied = true;
        for prefix in requires.split_ascii_whitespace() {
            any = true;
            match self.list_entry(prefix) {
                Some(uri) if self.knows(&uri) => {}
                _ => satisfied = false,
            }
        }
        if !any {
            self.out
                .nonconformances
                .insert(NonConformance::MissingRequires);
        }
        any && satisfied
    }
}

fn release<K: Ord>(map: &mut BTreeMap<K, usize>, key: K) {
    if let Some(count) = map.get_mut(&key) {
        *count = count.saturating_sub(1);
        if *count == 0 {
            map.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::OoxmlLimits;

    const MC: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";
    const P14: &str = "http://schemas.microsoft.com/office/powerpoint/2010/main";
    const PML: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";

    fn tree(xml: &str) -> XmlTree {
        XmlTree::parse(xml.as_bytes(), "t", &OoxmlLimits::default()).unwrap()
    }

    fn doc(body: &str) -> String {
        format!(
            "<r xmlns=\"{PML}\" xmlns:mc=\"{MC}\" xmlns:p14=\"{P14}\" xmlns:p=\"{PML}\" \
             xmlns:x=\"urn:x\" xmlns:y=\"urn:y\">{body}</r>"
        )
    }

    fn branch_marker(t: &XmlTree, c: &Compatibility, i: usize) -> Option<String> {
        let element = match c.alternates.get(i)?.branch {
            Branch::Choice { element, .. } | Branch::Fallback { element } => element,
            Branch::None => return Some("none".to_owned()),
        };
        Some(t.local_name(*t.child_elements(element).first()?).to_owned())
    }

    /// `SEC-X05`: one fixed rule, whatever the order suggests.
    #[test]
    fn the_first_choice_whose_requirements_are_understood_wins() {
        let t = tree(&doc("<mc:AlternateContent>\
             <mc:Choice Requires=\"p14\"><a/></mc:Choice>\
             <mc:Choice Requires=\"p\"><b/></mc:Choice>\
             <mc:Choice Requires=\"p\"><c/></mc:Choice>\
             <mc:Fallback><d/></mc:Fallback>\
             </mc:AlternateContent>"));
        let c = resolve_compatibility(&t);
        assert_eq!(c.alternates.len(), 1);
        assert!(matches!(
            c.alternates[0].branch,
            Branch::Choice { index: 1, .. }
        ));
        assert_eq!(branch_marker(&t, &c, 0).unwrap(), "b");
        assert_eq!(c.alternates[0].not_taken.len(), 3);
        assert!(c.nonconformances.is_empty());
    }

    #[test]
    fn every_required_namespace_must_be_understood() {
        let t = tree(&doc("<mc:AlternateContent>\
             <mc:Choice Requires=\" p  p14 \"><a/></mc:Choice>\
             <mc:Fallback><d/></mc:Fallback>\
             </mc:AlternateContent>"));
        let c = resolve_compatibility(&t);
        assert_eq!(branch_marker(&t, &c, 0).unwrap(), "d");
    }

    #[test]
    fn nothing_applies_when_no_choice_can_and_there_is_no_fallback() {
        let t = tree(&doc(
            "<mc:AlternateContent><mc:Choice Requires=\"p14\"><a/></mc:Choice></mc:AlternateContent>",
        ));
        let c = resolve_compatibility(&t);
        assert_eq!(c.alternates[0].branch, Branch::None);
        assert_eq!(c.alternates[0].not_taken.len(), 1);
    }

    /// `SEC-X05`: names are compared by URI, so a prefix proves nothing.
    #[test]
    fn prefixes_are_not_believed() {
        // `p14` here is bound to the understood PresentationML namespace.
        let t = tree(&format!(
            "<r xmlns:mc=\"{MC}\" xmlns:p14=\"{PML}\"><mc:AlternateContent>\
             <mc:Choice Requires=\"p14\"><a/></mc:Choice><mc:Fallback><d/></mc:Fallback>\
             </mc:AlternateContent></r>"
        ));
        let c = resolve_compatibility(&t);
        assert_eq!(branch_marker(&t, &c, 0).unwrap(), "a");

        // `p` is bound to a look-alike, and the nearest declaration counts.
        let t = tree(&format!(
            "<r xmlns:mc=\"{MC}\" xmlns:p=\"{PML}\"><mc:AlternateContent>\
             <mc:Choice xmlns:p=\"{PML}/\" Requires=\"p\"><a/></mc:Choice>\
             <mc:Fallback><d/></mc:Fallback></mc:AlternateContent></r>"
        ));
        let c = resolve_compatibility(&t);
        assert_eq!(branch_marker(&t, &c, 0).unwrap(), "d");

        // Not the real namespace, so not the real construct.
        let t = tree(
            "<r xmlns:mc=\"urn:not-compat\"><mc:AlternateContent>\
             <mc:Choice Requires=\"mc\"/></mc:AlternateContent></r>",
        );
        let c = resolve_compatibility(&t);
        assert!(c.alternates.is_empty());
        assert!(c.unknown_namespaces.contains("urn:not-compat"));
    }

    /// A different prefix for the real namespace is the real construct.
    #[test]
    fn any_prefix_bound_to_the_namespace_works() {
        let t = tree(&format!(
            "<r xmlns:zz=\"{MC}\" xmlns:p=\"{PML}\"><zz:AlternateContent>\
             <zz:Choice Requires=\"p\"><a/></zz:Choice></zz:AlternateContent></r>"
        ));
        let c = resolve_compatibility(&t);
        assert_eq!(branch_marker(&t, &c, 0).unwrap(), "a");
    }

    /// `SEC-X05`: a branch not taken is not interpreted, so it can neither refuse
    /// the part nor reach the report; and resolving leaves both branches in place.
    #[test]
    fn a_branch_not_taken_is_retained_and_never_interpreted() {
        let source = doc("<mc:AlternateContent>\
             <mc:Choice Requires=\"p14\">\
             <evil mc:MustUnderstand=\"p14\" xmlns:u=\"urn:u\"><u:payload/>\
             <mc:AlternateContent><mc:Fallback/></mc:AlternateContent></evil>\
             </mc:Choice>\
             <mc:Fallback><fine/></mc:Fallback>\
             </mc:AlternateContent>");
        let t = tree(&source);
        let before = t.clone();
        let c = resolve_compatibility(&t);
        assert_eq!(c.alternates.len(), 1);
        assert_eq!(branch_marker(&t, &c, 0).unwrap(), "fine");
        assert!(c.must_understand.is_empty());
        assert!(c.unknown_namespaces.is_empty());
        assert!(c.nonconformances.is_empty());
        assert_eq!(t, before);
        assert_eq!(t.to_bytes(), source.as_bytes());
    }

    #[test]
    fn an_unknown_must_understand_is_reported_where_it_is_reached() {
        let t = tree(&doc("<a mc:MustUnderstand=\"p14 p\"/>"));
        assert!(resolve_compatibility(&t).must_understand.contains(P14));

        // On a selected branch, and on the AlternateContent itself (9.4, step 3c).
        for body in [
            "<mc:AlternateContent><mc:Choice Requires=\"p\" mc:MustUnderstand=\"p14\"/></mc:AlternateContent>",
            "<mc:AlternateContent mc:MustUnderstand=\"p14\"><mc:Choice Requires=\"p14\"/></mc:AlternateContent>",
            "<mc:AlternateContent><mc:Fallback mc:MustUnderstand=\"p14\"/><mc:Choice Requires=\"p14\"/></mc:AlternateContent>",
        ] {
            let c = resolve_compatibility(&tree(&doc(body)));
            assert!(c.must_understand.contains(P14), "{body}");
        }

        // Understood, in an ignored element, or in an extension: no mismatch.
        for body in [
            "<a mc:MustUnderstand=\"p\"/>",
            "<x:a mc:Ignorable=\"x\" mc:MustUnderstand=\"p14\"/>",
            "<extLst><ext mc:MustUnderstand=\"p14\"/></extLst>",
        ] {
            let c = resolve_compatibility(&tree(&doc(body)));
            assert!(c.must_understand.is_empty(), "{body}");
        }
    }

    #[test]
    fn ignorable_and_process_content_mark_what_the_standard_says() {
        // Part 3's example in 9.2, without an application-defined extension.
        let t = tree(&format!(
            "<example xmlns=\"{PML}\" xmlns:mce=\"{MC}\" xmlns:i1=\"urn:i1\" \
             mce:Ignorable=\"i1\" mce:ProcessContent=\"i1:bar\" i1:foo=\"\">\
             <i1:foo><fooChild/></i1:foo>\
             <i1:bar><barChild/></i1:bar>\
             <i1:baz i1:baz=\"\"><i1:bazChild/></i1:baz>\
             </example>"
        ));
        let c = resolve_compatibility(&t);
        let names = |ids: &[ElementId]| -> Vec<String> {
            ids.iter().map(|&i| t.local_name(i).to_owned()).collect()
        };
        assert_eq!(names(&c.ignored_elements), ["foo", "baz"]);
        assert_eq!(names(&c.unwrapped_elements), ["bar"]);
        assert_eq!(c.ignored_attributes, 1);
        assert!(c.unknown_namespaces.is_empty());
    }

    #[test]
    fn a_wildcard_unwraps_and_scope_ends_with_the_element() {
        let c = resolve_compatibility(&tree(&doc(
            "<k mc:Ignorable=\"x\" mc:ProcessContent=\"x:*\"><x:a><x:b/></x:a></k><x:c/>",
        )));
        assert_eq!(c.unwrapped_elements.len(), 2);
        assert!(c.ignored_elements.is_empty());
        assert!(c.unknown_namespaces.contains("urn:x"));
    }

    #[test]
    fn an_understood_namespace_is_never_ignored() {
        let c = resolve_compatibility(&tree(&doc("<p:a mc:Ignorable=\"p\" p:b=\"\"/>")));
        assert!(c.ignored_elements.is_empty());
        assert_eq!(c.ignored_attributes, 0);
    }

    /// Part 3's example in 9.3, both configurations.
    #[test]
    fn nested_alternate_content_resolves_inside_the_selected_branch_only() {
        let t = tree(&format!(
            "<r xmlns:mce=\"{MC}\" xmlns:n1=\"urn:n1\" xmlns:n2=\"urn:n2\" xmlns:n3=\"urn:n3\">\
             <mce:AlternateContent>\
             <mce:Choice Requires=\"n1 n2\"><mce:AlternateContent>\
             <mce:Choice Requires=\"n3\"><c11/></mce:Choice><mce:Fallback><f11/></mce:Fallback>\
             </mce:AlternateContent></mce:Choice>\
             <mce:Choice Requires=\"n1\"><mce:AlternateContent>\
             <mce:Choice Requires=\"n3\"><c21/></mce:Choice><mce:Fallback><f21/></mce:Fallback>\
             </mce:AlternateContent></mce:Choice>\
             <mce:Fallback><f1/></mce:Fallback>\
             </mce:AlternateContent></r>"
        ));
        let all = |uri: &str| ["urn:n1", "urn:n2", "urn:n3"].contains(&uri);
        let c = resolve_with(&t, &all);
        assert_eq!(c.alternates.len(), 2);
        assert_eq!(branch_marker(&t, &c, 1).unwrap(), "c11");

        let two = |uri: &str| ["urn:n1", "urn:n2"].contains(&uri);
        let c = resolve_with(&t, &two);
        assert_eq!(branch_marker(&t, &c, 1).unwrap(), "f11");

        let none = |_: &str| false;
        let c = resolve_with(&t, &none);
        assert_eq!(c.alternates.len(), 1);
        assert_eq!(branch_marker(&t, &c, 0).unwrap(), "f1");
    }

    /// `SEC-X05`: a construct readers might disagree over is never selected, and
    /// always reported.
    #[test]
    fn broken_constructs_are_reported_and_not_guessed_at() {
        use NonConformance as N;
        for (body, want, marker) in [
            (
                "<mc:AlternateContent><mc:Choice><a/></mc:Choice><mc:Fallback><d/></mc:Fallback></mc:AlternateContent>",
                N::MissingRequires,
                "d",
            ),
            (
                "<mc:AlternateContent><mc:Choice Requires=\" \"><a/></mc:Choice><mc:Fallback><d/></mc:Fallback></mc:AlternateContent>",
                N::MissingRequires,
                "d",
            ),
            (
                "<mc:AlternateContent><mc:Choice Requires=\"nope\"><a/></mc:Choice><mc:Fallback><d/></mc:Fallback></mc:AlternateContent>",
                N::UndeclaredPrefix,
                "d",
            ),
            (
                "<mc:AlternateContent><mc:Choice Requires=\"mc\"><a/></mc:Choice><mc:Fallback><d/></mc:Fallback></mc:AlternateContent>",
                N::ListsCompatibilityNamespace,
                "d",
            ),
            (
                "<mc:AlternateContent><mc:Fallback><d/></mc:Fallback><mc:Choice Requires=\"p\"><a/></mc:Choice></mc:AlternateContent>",
                N::BranchOutOfOrder,
                "d",
            ),
            (
                "<mc:AlternateContent><mc:Choice Requires=\"p14\"/><mc:Fallback><d/></mc:Fallback><mc:Fallback><e/></mc:Fallback></mc:AlternateContent>",
                N::BranchOutOfOrder,
                "d",
            ),
            (
                "<mc:AlternateContent><mc:Fallback><d/></mc:Fallback></mc:AlternateContent>",
                N::NoChoice,
                "d",
            ),
            (
                "<mc:AlternateContent><q/><mc:Choice Requires=\"p\"><d/></mc:Choice></mc:AlternateContent>",
                N::UnexpectedChild,
                "d",
            ),
            (
                "<mc:AlternateContent k=\"1\"><mc:Choice Requires=\"p\"><d/></mc:Choice></mc:AlternateContent>",
                N::BadCompatibilityElementAttribute,
                "d",
            ),
            (
                "<mc:AlternateContent><mc:Choice Requires=\"p\" xml:lang=\"en\"><d/></mc:Choice></mc:AlternateContent>",
                N::BadCompatibilityElementAttribute,
                "d",
            ),
        ] {
            let t = tree(&doc(body));
            let c = resolve_compatibility(&t);
            assert!(c.nonconformances.contains(&want), "{body}: {c:?}");
            assert_eq!(branch_marker(&t, &c, 0).unwrap(), marker, "{body}");
        }
        for (body, want) in [
            ("<mc:Choice Requires=\"p\"/>", N::MisplacedBranch),
            ("<mc:Fallback/>", N::MisplacedBranch),
            ("<mc:Other/>", N::UnknownCompatibilityElement),
            ("<a mc:Other=\"1\"/>", N::UnknownCompatibilityAttribute),
            ("<a mc:Ignorable=\"nope\"/>", N::UndeclaredPrefix),
            ("<a mc:MustUnderstand=\"nope\"/>", N::UndeclaredPrefix),
            ("<a mc:Ignorable=\"mc\"/>", N::ListsCompatibilityNamespace),
            ("<a mc:ProcessContent=\"x\"/>", N::MalformedProcessContent),
            (
                "<a mc:ProcessContent=\"x:b\"/>",
                N::ProcessContentNotIgnorable,
            ),
            (
                "<x:a mc:Ignorable=\"x\" mc:ProcessContent=\"x:a\" xml:lang=\"en\"/>",
                N::UnwrappedWithXmlAttribute,
            ),
        ] {
            let c = resolve_compatibility(&tree(&doc(body)));
            assert!(c.nonconformances.contains(&want), "{body}: {c:?}");
        }
        // An undeclared element prefix no longer reaches here: the walker refuses
        // the part (`XML-03`).
        let err = XmlTree::parse(doc("<nope:a/>").as_bytes(), "t", &OoxmlLimits::default());
        assert_eq!(err.unwrap_err().code(), "CS-XML-0004");
    }

    #[test]
    fn an_extension_is_left_alone_but_an_xfrm_ext_is_ordinary() {
        let c = resolve_compatibility(&tree(&doc(
            "<extLst><ext uri=\"{1}\"><mc:AlternateContent/><z:b xmlns:z=\"urn:z\"/><p14:c/></ext></extLst>",
        )));
        assert_eq!(c.extensions.len(), 1);
        assert!(c.alternates.is_empty());
        assert!(c.nonconformances.is_empty());
        assert!(c.unknown_namespaces.is_empty());

        let c = resolve_compatibility(&tree(&doc("<xfrm><ext><p14:c/></ext></xfrm>")));
        assert!(c.extensions.is_empty());
        assert!(c.unknown_namespaces.contains(P14));
    }

    #[test]
    fn a_root_alternate_content_resolves() {
        let t = tree(&format!(
            "<mc:AlternateContent xmlns:mc=\"{MC}\" xmlns:p=\"{PML}\">\
             <mc:Choice Requires=\"p\"><p:sld/></mc:Choice></mc:AlternateContent>"
        ));
        let c = resolve_compatibility(&t);
        assert_eq!(branch_marker(&t, &c, 0).unwrap(), "sld");
    }
}
