//! Patch a chart-family part in place (ADR-013, Tier 1J).
//!
//! The model is turned into a description of the elements it stands for, before and after,
//! and only what differs touches the tree. So an unedited chart changes nothing, an edit
//! changes its own nodes, and `c:extLst` entries, `mc:AlternateContent`, unknown attributes
//! and what the model only names ([`Retained`]) stay where they are.
//!
//! Each type's description is generated with its reader from the schema, by
//! `tools/chart-codegen/generate.py`, into `chart/`. The patch itself is here, and is one
//! function for every type. DrawingML inside a chart goes through Tier 1E's and 1F's writers.
//!
//! A new [`Retained`] is refused as `CS-EXP-0009`: the model holds its name and nothing else,
//! so it cannot be written, only kept or removed. Every patched part is read back, and a
//! value the reader would not believe is refused as `CS-EXP-0007`, with nothing saved.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};

use casual_slide_import::chart::{ChartLimits, NS_MC, import_chart_part};
use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::drawing::DrawingLimits;
use casual_slide_import::properties::{Elem, Findings, ImportLimits, ns};
use casual_slide_model::chart::{
    AlternateContentPart, AlternateStyle, AxisId, ChartPart, Retained,
};
use casual_slide_model::color::Color;
use casual_slide_model::drawing::ShapeProperties;
use casual_slide_model::properties::Lexical;
use casual_slide_model::text::TextBody;
use casual_slide_ooxml::{Conformance, OoxmlLimits, PresentationPackage, Relationship, XmlTree};
use casual_slide_package::PartEdits;

use crate::drawing::{color_node, patch as patch_drawing, properties_node};
use crate::presentation::SaveOutcome;
use crate::properties::{ExportError, Writer};
use crate::slide::SlideExportError;
use crate::text::patch_text_body;

#[path = "chart/c.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated, one shape repeated: see tools/chart-codegen/generate.py.
pub(crate) mod c;
#[path = "chart/c14.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
pub(crate) mod c14;
#[path = "chart/c15.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
pub(crate) mod c15;
#[path = "chart/c16.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
pub(crate) mod c16;
#[path = "chart/c16ac.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
pub(crate) mod c16ac;
#[path = "chart/c16r2.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
pub(crate) mod c16r2;
#[path = "chart/c16r3.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
pub(crate) mod c16r3;
#[path = "chart/cdr.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
pub(crate) mod cdr;
#[path = "chart/cdr14.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
pub(crate) mod cdr14;
#[path = "chart/cs.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
pub(crate) mod cs;
#[path = "chart/cx.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
pub(crate) mod cx;
#[path = "chart/ext.rs"]
#[rustfmt::skip]
#[allow(clippy::all)] // Generated.
mod ext;

pub(crate) use ext::ext_item;

/// An element's namespace and the prefix to use if none is in scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Space {
    uris: &'static [&'static str],
    prefix: &'static str,
}

const fn space(uris: &'static [&'static str], prefix: &'static str) -> Space {
    Space { uris, prefix }
}

use casual_slide_import::chart as names;

pub(crate) const C: Space = space(names::NS_C, "c");
pub(crate) const CDR: Space = space(names::NS_CDR, "cdr");
pub(crate) const C14: Space = space(names::NS_C14, "c14");
pub(crate) const CDR14: Space = space(names::NS_CDR14, "cdr14");
pub(crate) const C15: Space = space(names::NS_C15, "c15");
pub(crate) const CS: Space = space(names::NS_CS, "cs");
pub(crate) const C16: Space = space(names::NS_C16, "c16");
pub(crate) const C16AC: Space = space(names::NS_C16AC, "c16ac");
pub(crate) const CX: Space = space(names::NS_CX, "cx");
pub(crate) const C16R3: Space = space(names::NS_C16R3, "c16r3");
pub(crate) const C16R2: Space = space(names::NS_C16R2, "c16r2");
const MC: Space = space(NS_MC, "mc");
#[allow(non_upper_case_globals)] // Read by generated code as a namespace's lower-case module is.
pub(crate) const dml: Space = space(ns::DML, "a");

/// What an attribute is.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Attr {
    Plain(String),
    /// `r:id` and its like.
    Rel(String),
    /// In a namespace of its own.
    Qualified(Space, String),
}

/// What an element holds beyond attributes and children.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Payload {
    None,
    Shape(Box<ShapeProperties>),
    Text(Box<TextBody>),
    Color(Box<Color>),
    Retained(Retained),
}

/// An element, as the model says it should be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct N {
    space: Space,
    local: Cow<'static, str>,
    order: &'static [&'static str],
    attrs: Vec<(&'static str, Attr)>,
    kids: Vec<N>,
    text: Option<String>,
    payload: Payload,
}

impl N {
    pub(crate) fn new(space: Space, local: &'static str, order: &'static [&'static str]) -> Self {
        Self {
            space,
            local: Cow::Borrowed(local),
            order,
            attrs: Vec::new(),
            kids: Vec::new(),
            text: None,
            payload: Payload::None,
        }
    }

    pub(crate) fn attr<T: Lexical>(&mut self, name: &'static str, value: Option<&T>) {
        if let Some(value) = value {
            self.attrs.push((name, Attr::Plain(value.to_lexical())));
        }
    }

    pub(crate) fn rel_attr(&mut self, name: &'static str, value: Option<&String>) {
        if let Some(value) = value {
            self.attrs.push((name, Attr::Rel(value.clone())));
        }
    }

    pub(crate) fn qualified_attr<T: Lexical>(
        &mut self,
        space: Space,
        name: &'static str,
        value: Option<&T>,
    ) {
        if let Some(value) = value {
            self.attrs
                .push((name, Attr::Qualified(space, value.to_lexical())));
        }
    }

    pub(crate) fn set_text<T: Lexical>(&mut self, value: Option<&T>) {
        self.text = value.map(Lexical::to_lexical);
    }

    pub(crate) fn push(&mut self, kid: N) {
        self.kids.push(kid);
    }

    pub(crate) fn text_element<T: Lexical>(space: Space, local: &'static str, value: &T) -> Self {
        let mut n = Self::new(space, local, &[]);
        n.text = Some(value.to_lexical());
        n
    }

    pub(crate) fn shape(space: Space, local: &'static str, value: &ShapeProperties) -> Self {
        let mut n = Self::new(space, local, &[]);
        n.payload = Payload::Shape(Box::new(value.clone()));
        n
    }

    pub(crate) fn text_body(space: Space, local: &'static str, value: &TextBody) -> Self {
        let mut n = Self::new(space, local, &[]);
        n.payload = Payload::Text(Box::new(value.clone()));
        n
    }

    pub(crate) fn color(value: &Color) -> Self {
        let mut n = Self::new(dml, value.value.element_name(), &[]);
        n.payload = Payload::Color(Box::new(value.clone()));
        n
    }

    pub(crate) fn retained(value: &Retained) -> Self {
        let mut n = Self::new(dml, "", &[]);
        n.local = Cow::Owned(value.local.clone());
        n.payload = Payload::Retained(value.clone());
        n
    }

    fn value(&self, name: &str) -> Option<&Attr> {
        self.attrs.iter().find(|(n, _)| *n == name).map(|(_, v)| v)
    }

    fn is(&self, el: &Elem) -> bool {
        if el.local != self.local {
            return false;
        }
        match &self.payload {
            Payload::Retained(r) => el.uri == r.uri,
            _ => el.in_namespace(self.space.uris),
        }
    }

    /// Two nodes are of one element name.
    fn same_name(&self, other: &N) -> bool {
        self.local == other.local
            && match (&self.payload, &other.payload) {
                (Payload::Retained(a), Payload::Retained(b)) => a.uri == b.uri,
                (Payload::Retained(_), _) | (_, Payload::Retained(_)) => false,
                _ => self.space == other.space,
            }
    }

    fn rank(&self, local: &str) -> Option<usize> {
        self.order.iter().position(|o| *o == local)
    }
}

/// `c:axId` or `c:crossAx`: `c:CT_UnsignedInt` by the schema, signed by Mac PowerPoint.
// spec-audit: owner=axId,crossAx
pub(crate) fn axis_id(v: &AxisId, name: &'static str, space: Space) -> N {
    let mut n = N::new(space, name, &[]);
    n.attr("val", v.val.as_ref());
    n
}

/// `mc:AlternateContent` around `c14:style`.
pub(crate) fn alternate_style(v: &AlternateStyle) -> N {
    let mut n = N::new(MC, "AlternateContent", &["Choice", "Fallback"]);
    let mut choice = N::new(MC, "Choice", &[]);
    choice.attr("Requires", v.requires.as_ref());
    if let Some(style) = &v.choice {
        choice.push(c14::c14_style(style, "style", C14));
    }
    n.push(choice);
    if let Some(style) = &v.fallback {
        let mut fallback = N::new(MC, "Fallback", &[]);
        fallback.push(c::c_style(style, "style", C));
        n.push(fallback);
    }
    n
}

/// `mc:AlternateContent` around `cdr14:contentPart`.
pub(crate) fn alternate_content_part(v: &AlternateContentPart) -> N {
    let mut n = N::new(MC, "AlternateContent", &["Choice", "Fallback"]);
    let mut choice = N::new(MC, "Choice", &[]);
    choice.attr("Requires", v.requires.as_ref());
    if let Some(part) = &v.choice {
        choice.push(cdr14::cdr14_content_part(part, "contentPart", CDR14));
    }
    n.push(choice);
    let mut fallback = N::new(MC, "Fallback", &[]);
    for kid in &v.fallback {
        fallback.push(N::retained(kid));
    }
    n.push(fallback);
    n
}

/// A part's root, as the model says it should be.
fn root_node(part: &ChartPart) -> N {
    match part {
        ChartPart::Chart(v) => c::c_chart_space(v, "chartSpace", C),
        ChartPart::ChartEx(v) => cx::cx_chart_space(v, "chartSpace", CX),
        ChartPart::UserShapes(v) => cdr::cdr_drawing(v, "userShapes", C),
        ChartPart::Style(v) => cs::cs_chart_style(v, "chartStyle", CS),
        ChartPart::Colors(v) => cs::cs_color_style(v, "colorStyle", CS),
    }
}

/// What a patch does with a new [`Retained`], which it can only name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewRetained {
    /// Refuse, as the save path does.
    Refuse,
    /// Write it empty. For [`rewrite_from_model`], whose read back is what is being tested.
    Empty,
}

struct Patch<'p, 'w> {
    w: &'p mut Writer<'w>,
    new_retained: NewRetained,
}

impl Patch<'_, '_> {
    fn write_attr(&mut self, el: &Elem, name: &str, value: &Attr) -> Result<(), ExportError> {
        let (name, text) = match value {
            Attr::Plain(text) => (name.to_owned(), text.clone()),
            Attr::Rel(id) => {
                if !id.is_empty()
                    && self
                        .w
                        .known_rels
                        .as_ref()
                        .is_some_and(|known| !known.contains(id))
                {
                    return Err(ExportError::UnknownRelationship {
                        part: self.w.part.to_owned(),
                    });
                }
                let written = match el.name_of_ns(ns::REL, name) {
                    Some(written) => written.to_owned(),
                    None => format!("{}:{name}", self.w.declared_prefix(el, ns::REL, "r")?),
                };
                (written, id.clone())
            }
            Attr::Qualified(space, text) => {
                let written = match el.name_of_ns(space.uris, name) {
                    Some(written) => written.to_owned(),
                    None if el.get(name).is_some() => name.to_owned(),
                    None => format!(
                        "{}:{name}",
                        self.w.declared_prefix(el, space.uris, space.prefix)?
                    ),
                };
                (written, text.clone())
            }
        };
        self.w.attr(el, &name, None::<&String>, Some(&text))
    }

    fn remove_attr(&mut self, el: &Elem, name: &str, value: &Attr) {
        let written = match value {
            Attr::Plain(_) => Some(name.to_owned()),
            Attr::Rel(_) => el.name_of_ns(ns::REL, name).map(str::to_owned),
            Attr::Qualified(space, _) => el
                .name_of_ns(space.uris, name)
                .or(el.get(name).map(|_| name))
                .map(str::to_owned),
        };
        if let Some(written) = written {
            self.w.tree.remove_attribute(el.id, &written);
            self.w.changes += 1;
        }
    }

    /// Bring an element from what `before` says to what `after` says.
    fn node(&mut self, el: &Elem, before: Option<&N>, after: &N) -> Result<(), ExportError> {
        if before == Some(after) {
            return Ok(());
        }
        match (&after.payload, before.map(|b| &b.payload)) {
            (Payload::Shape(a), b) => {
                let b = match b {
                    Some(Payload::Shape(b)) => Some(properties_node(b, "spPr")),
                    _ => None,
                };
                return patch_drawing(self.w, el, b.as_ref(), &properties_node(a, "spPr"));
            }
            (Payload::Text(a), b) => {
                let b = match b {
                    Some(Payload::Text(b)) => Some(&**b),
                    _ => None,
                };
                return patch_text_body(self.w, el, b, a);
            }
            (Payload::Color(a), b) => {
                let b = match b {
                    Some(Payload::Color(b)) => Some(color_node(b)),
                    _ => None,
                };
                return patch_drawing(self.w, el, b.as_ref(), &color_node(a));
            }
            (Payload::Retained(_), _) | (Payload::None, _) => {}
        }
        for (name, value) in &after.attrs {
            if before.and_then(|b| b.value(name)) != Some(value) {
                self.write_attr(el, name, value)?;
            }
        }
        for (name, value) in before.iter().flat_map(|b| &b.attrs) {
            if after.value(name).is_none() {
                self.remove_attr(el, name, value);
            }
        }
        if before.and_then(|b| b.text.as_ref()) != after.text.as_ref() {
            self.w.set_text(el, after.text.as_deref().unwrap_or(""))?;
        }
        self.kids(el, before.map_or(&[], |b| &b.kids), after)
    }

    fn insert(
        &mut self,
        parent: &Elem,
        anchor: Option<&Elem>,
        order_of: &N,
        n: &N,
    ) -> Result<(), ExportError> {
        if matches!(n.payload, Payload::Retained(_)) && self.new_retained == NewRetained::Refuse {
            return Err(ExportError::UnsupportedEdit {
                what: "a chart element only named in the model",
                part: self.w.part.to_owned(),
            });
        }
        let dynamic: Vec<&str>;
        let (uris, prefix): (&[&str], &str) = match &n.payload {
            Payload::Retained(r) => {
                dynamic = r.uri.iter().map(String::as_str).collect();
                (&dynamic, "x")
            }
            _ => (n.space.uris, n.space.prefix),
        };
        let el = self.w.new_child(parent, uris, prefix, &n.local)?;
        let anchor = match anchor {
            Some(a) => Some(a.id),
            None => {
                let mine = order_of.rank(&n.local);
                let kids = parent.children(self.w.tree, self.w.part)?;
                mine.and_then(|mine| {
                    kids.iter()
                        .find(|k| order_of.rank(&k.local).is_some_and(|theirs| theirs > mine))
                        .map(|k| k.id)
                })
            }
        };
        self.w.tree.insert_child(parent.id, anchor, el.id);
        if !matches!(n.payload, Payload::Retained(_)) {
            self.node(&el, None, n)?;
        }
        Ok(())
    }

    fn kids(&mut self, el: &Elem, before: &[N], parent: &N) -> Result<(), ExportError> {
        let after = &parent.kids;
        if before == after.as_slice() {
            return Ok(());
        }
        let tree_kids = el.children(self.w.tree, self.w.part)?;
        // Each `before` child owns the next tree child of its name.
        let mut by_local: HashMap<&str, VecDeque<usize>> = HashMap::new();
        for (i, kid) in tree_kids.iter().enumerate() {
            by_local.entry(kid.local.as_str()).or_default().push_back(i);
        }
        let mut take = |n: &N| -> Option<usize> {
            let queue = by_local.get_mut(n.local.as_ref())?;
            let at = queue
                .iter()
                .position(|i| tree_kids.get(*i).is_some_and(|k| n.is(k)))?;
            queue.remove(at)
        };
        let element_of: Vec<Option<usize>> = before.iter().map(&mut take).collect();
        // Each `after` child takes the `before` child of the same name and occurrence, or
        // a child of that name the model could not see.
        let mut used = vec![false; before.len()];
        let mut paired: Vec<Option<usize>> = Vec::with_capacity(after.len());
        let mut by_name: HashMap<&str, VecDeque<usize>> = HashMap::new();
        for (i, n) in before.iter().enumerate() {
            if element_of.get(i).copied().flatten().is_some() {
                by_name.entry(n.local.as_ref()).or_default().push_back(i);
            }
        }
        for n in after {
            let found = by_name.get_mut(n.local.as_ref()).and_then(|queue| {
                let at = queue
                    .iter()
                    .position(|i| before.get(*i).is_some_and(|b| b.same_name(n)))?;
                queue.remove(at)
            });
            if let Some(i) = found
                && let Some(slot) = used.get_mut(i)
            {
                *slot = true;
            }
            paired.push(found);
        }
        // An order the edit changed is rebuilt: its order is what the edit named.
        let order: Vec<usize> = paired
            .iter()
            .flatten()
            .filter_map(|i| element_of.get(*i).copied().flatten())
            .collect();
        if order.windows(2).any(|w| matches!(w, [x, y] if x > y)) {
            for kid in element_of
                .iter()
                .flatten()
                .filter_map(|i| tree_kids.get(*i))
            {
                self.w.tree.remove_child(el.id, kid.id);
                self.w.changes += 1;
            }
            for n in after {
                self.insert(el, None, parent, n)?;
            }
            return Ok(());
        }
        // Removed: what the edit took out, and repeats of a name no longer wanted, which
        // the reader would otherwise find in its place.
        for (i, slot) in used.iter().enumerate() {
            if *slot {
                continue;
            }
            if let Some(kid) = element_of
                .get(i)
                .copied()
                .flatten()
                .and_then(|o| tree_kids.get(o))
            {
                self.w.tree.remove_child(el.id, kid.id);
                self.w.changes += 1;
            }
        }
        let mut gone = HashSet::new();
        for n in before {
            let wanted = after.iter().any(|a| a.same_name(n));
            if wanted || !gone.insert(n.local.clone()) {
                continue;
            }
            if let Some(queue) = by_local_rest(&tree_kids, &element_of, n) {
                for kid in queue {
                    self.w.tree.remove_child(el.id, kid.id);
                    self.w.changes += 1;
                }
            }
        }
        // For each `after` child, the tree child of the next one that stays.
        let mut following: Vec<Option<usize>> = vec![None; after.len()];
        let mut next = None;
        for j in (0..after.len()).rev() {
            if let Some(slot) = following.get_mut(j) {
                *slot = next;
            }
            if let Some(i) = paired.get(j).copied().flatten() {
                next = element_of.get(i).copied().flatten().or(next);
            }
        }
        let owned: HashSet<usize> = element_of.iter().flatten().copied().collect();
        let mut adopted: HashSet<usize> = HashSet::new();
        for (j, n) in after.iter().enumerate() {
            if let Some(i) = paired.get(j).copied().flatten() {
                let kid = element_of
                    .get(i)
                    .copied()
                    .flatten()
                    .and_then(|o| tree_kids.get(o));
                if let (Some(kid), Some(b)) = (kid, before.get(i)) {
                    self.node(kid, Some(b), n)?;
                }
                continue;
            }
            // A child of this name the model could not see, such as one holding a bad value.
            let unseen = tree_kids
                .iter()
                .enumerate()
                .find(|(i, k)| n.is(k) && !owned.contains(i) && !adopted.contains(i));
            if let Some((i, kid)) = unseen {
                adopted.insert(i);
                self.node(kid, None, n)?;
                continue;
            }
            let anchor = following
                .get(j)
                .copied()
                .flatten()
                .and_then(|o| tree_kids.get(o));
            self.insert(el, anchor, parent, n)?;
        }
        Ok(())
    }
}

/// The children of `n`'s name that no `before` child owned.
fn by_local_rest<'t>(
    tree_kids: &'t [Elem],
    element_of: &[Option<usize>],
    n: &N,
) -> Option<Vec<&'t Elem>> {
    let owned: HashSet<usize> = element_of.iter().flatten().copied().collect();
    let rest: Vec<&Elem> = tree_kids
        .iter()
        .enumerate()
        .filter(|(i, k)| n.is(k) && !owned.contains(i))
        .map(|(_, k)| k)
        .collect();
    (!rest.is_empty()).then_some(rest)
}

fn read(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    limits: &ChartLimits,
    import_limits: &ImportLimits,
) -> Result<ChartPart, ExportError> {
    let mut findings = Findings::new(import_limits.max_findings);
    let mut color = ColorFindings::default();
    import_chart_part(
        tree,
        part,
        rels,
        limits,
        DrawingLimits::default(),
        import_limits,
        &mut findings,
        &mut color,
    )
    .map_err(|err| match err {
        casual_slide_import::chart::ChartError::Import(err) => ExportError::Import(err),
        casual_slide_import::chart::ChartError::Bound { part, .. } => {
            ExportError::RoundTrip { part }
        }
    })
}

/// Bring a chart-family part's tree to what the model says. The number of nodes changed.
#[allow(clippy::too_many_arguments)] // As `patch_table_styles`: one part's whole context.
pub fn patch_chart_part(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &ChartPart,
    limits: &ChartLimits,
    import_limits: &ImportLimits,
    xml_limits: &OoxmlLimits,
) -> Result<usize, ExportError> {
    let before = read(tree, part, rels, limits, import_limits)?;
    if &before == after {
        return Ok(0);
    }
    if std::mem::discriminant(&before) != std::mem::discriminant(after) {
        return Err(ExportError::UnsupportedEdit {
            what: "a chart part changed to another kind of part",
            part: part.to_owned(),
        });
    }
    let root = Elem::root(tree, part).map_err(ExportError::from)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
        changes: 0,
    };
    let mut patch = Patch {
        w: &mut w,
        new_retained: NewRetained::Refuse,
    };
    patch.node(&root, Some(&root_node(&before)), &root_node(after))?;
    let changes = w.changes;
    let again = XmlTree::parse(&tree.to_bytes(), part, xml_limits).map_err(ExportError::from)?;
    if &read(&again, part, rels, limits, import_limits)? != after {
        return Err(ExportError::RoundTrip {
            part: part.to_owned(),
        });
    }
    Ok(changes)
}

/// Write a model into a tree holding only an empty root, by the path a new child takes in
/// [`patch_chart_part`]. Never used to save: edits patch named nodes (ADR-013). It is how
/// tests hold the writer to every row: what comes out must read back as the model.
pub fn rewrite_from_model(
    tree: &mut XmlTree,
    part: &str,
    rels: &[Relationship],
    model: &ChartPart,
) -> Result<usize, ExportError> {
    let root = Elem::root(tree, part).map_err(ExportError::from)?;
    let mut w = Writer {
        tree,
        part,
        strict: false,
        known_rels: None,
        changes: 0,
    };
    let _ = rels;
    let mut patch = Patch {
        w: &mut w,
        new_retained: NewRetained::Empty,
    };
    patch.node(&root, None, &root_node(model))?;
    Ok(w.changes)
}

/// Save edited chart parts over the package they came from. Only those parts change; with
/// nothing changed, the input bytes come back.
pub fn save_chart_parts(
    package: &mut PresentationPackage,
    edited: &[(String, ChartPart)],
    limits: &ChartLimits,
    import_limits: &ImportLimits,
) -> Result<SaveOutcome, SlideExportError> {
    let strict = package.conformance() == Conformance::Strict;
    let xml = *package.limits();
    let mut edits = PartEdits::default();
    let mut changed_parts = Vec::new();
    for (part, after) in edited {
        let bytes = package.read_part(part).map_err(ExportError::from)?;
        let mut tree = XmlTree::parse(&bytes, part, &xml).map_err(ExportError::from)?;
        let rels = package.relationships_of(part).to_vec();
        let changes = patch_chart_part(
            &mut tree,
            part,
            strict,
            &rels,
            after,
            limits,
            import_limits,
            &xml,
        )?;
        let out = tree.to_bytes();
        if changes > 0 && out != bytes {
            edits.replace(part.clone(), out);
            changed_parts.push(part.clone());
        }
    }
    if changed_parts.is_empty() {
        return Ok(SaveOutcome {
            bytes: package.save_unedited(),
            changed_parts,
            signature_invalidated: false,
        });
    }
    let signed = package
        .findings()
        .iter()
        .any(|f| f.kind == casual_slide_ooxml::FindingKind::Signature);
    Ok(SaveOutcome {
        bytes: package.save_with(&edits).map_err(ExportError::from)?,
        changed_parts,
        signature_invalidated: signed,
    })
}
