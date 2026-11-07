//! Patch SmartArt in place (ADR-013, Tier 1K), and what the generated writers of diagrams,
//! equations and ink share.
//!
//! The generated writers change only the nodes whose values differ. Lists and repeating
//! choices match by position, so what a child holds beyond the model stays with it.
//!
//! **Staleness (`INV-FB-01`).** `dsp:drawing` is the last layout, and nothing here can make
//! another (`DGM-01`). So a data edit that would leave the drawing behind is refused as
//! `CS-SAV-0009`, with nothing saved: an edit the drawing already shows is fine, which is
//! what [`Diagram::set_point_text`] makes. An edit of a layout, style or colour definition
//! leaves the drawing as it was, and is reported stale as `CS-SAV-0007`, not refused.
//! A part set aside on import cannot be edited, and comes back byte for byte.

pub mod schema;

use std::collections::HashSet;

use casual_slide_import::color::is_color_element;
use casual_slide_import::diagram::{
    GraphicError, GraphicLimits, import_diagram, import_diagram_root,
};
use casual_slide_import::properties::{Elem, Findings, ImportLimits, ns};
use casual_slide_model::color::Color;
use casual_slide_model::d3::{Scene3D, Shape3D};
use casual_slide_model::diagram::schema as m;
use casual_slide_model::diagram::{
    Diagram, DiagramRole, DrawingIdentity, Extension, Extensions, Formatting, NvGroupProps,
    NvShapeProps, ShapeProps,
};
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::Transform;
use casual_slide_model::text::{CharacterProperties, Text3D, TextBody, TextListStyle};
use casual_slide_model::theme::ShapeStyle;
use casual_slide_ooxml::{Conformance, OoxmlLimits, PresentationPackage, Relationship, XmlTree};
use casual_slide_package::PartEdits;

use crate::d3::{flat_text_node, scene3d_node, sp3d_node};
use crate::drawing::{Node, Space, color_node, patch, patch_style, properties_node};
use crate::effect::effect_properties_node;
use crate::fill::fill_node;
use crate::line::line_node;
use crate::presentation::SaveOutcome;
use crate::properties::{ExportError, Writer};
use crate::slide::{SlideExportError, patch_transform, sync_ordered};
use crate::text::{patch_character, patch_list_style, patch_text_body};

/// What the generated writers use.
pub(crate) mod prelude {
    pub(crate) use super::{
        R, attr_member, attr_name, patch_character_part, patch_color_list, patch_ext_lst,
        patch_formatting, patch_identity, patch_list_style_part, patch_nv_group, patch_nv_shape,
        patch_scene3d, patch_sp_pr, patch_sp3d, patch_style_part, patch_text_body_part,
        patch_text3d, patch_transform_part, rel_attr_named, sync_items, sync_many, sync_own_text,
    };
    pub(crate) use crate::properties::Writer;
    pub(crate) use casual_slide_import::properties::{Elem, ns};
    pub(crate) use casual_slide_model::properties::Lexical;
}

pub(crate) type R = Result<(), ExportError>;

/// A namespace list, a preferred prefix, and a local name.
pub(crate) type ItemName = (&'static [&'static str], &'static str, &'static str);

/// An attribute in the relationships namespace, under whatever prefix the part uses, and
/// checked against the part's relationships.
pub(crate) fn rel_attr_named(
    w: &mut Writer<'_>,
    el: &Elem,
    local: &str,
    before: Option<&String>,
    after: Option<&String>,
) -> R {
    if before == after {
        return Ok(());
    }
    let existing = el.name_of_ns(ns::REL, local).map(str::to_owned);
    let Some(value) = after else {
        if let Some(name) = existing {
            w.tree.remove_attribute(el.id, &name);
            w.changes += 1;
        }
        return Ok(());
    };
    if !value.is_empty()
        && w.known_rels
            .as_ref()
            .is_some_and(|known| !known.contains(value))
    {
        return Err(ExportError::UnknownRelationship {
            part: w.part.to_owned(),
        });
    }
    let name = match existing {
        Some(name) => name,
        None => format!("{}:{local}", w.declared_prefix(el, ns::REL, "r")?),
    };
    w.attr(el, &name, None::<&String>, Some(value))
}

/// How to spell a qualified attribute here: as the element has it, or under a prefix in
/// scope, declared if it must be and only if there is a value to write.
pub(crate) fn attr_name(
    w: &mut Writer<'_>,
    el: &Elem,
    uris: &[&str],
    prefix: &str,
    local: &str,
    writing: bool,
) -> Result<String, ExportError> {
    if let Some(found) = el.name_of_ns(uris, local) {
        return Ok(found.to_owned());
    }
    if !writing {
        return Ok(local.to_owned());
    }
    Ok(format!("{}:{local}", w.declared_prefix(el, uris, prefix)?))
}

/// An attribute of a union or a list: a token the type does not admit is refused here,
/// rather than written and found on reading it back.
pub(crate) fn attr_member<T: Lexical + PartialEq>(
    w: &mut Writer<'_>,
    el: &Elem,
    name: &str,
    before: Option<&T>,
    after: Option<&T>,
    member: fn(&str, &str) -> bool,
) -> R {
    if before != after
        && let Some(value) = after
    {
        let text = value.to_lexical();
        let local = name.rsplit(':').next().unwrap_or(name);
        if !text.split_ascii_whitespace().all(|t| member(local, t)) {
            return Err(ExportError::ValueOutOfRange {
                what: "a value its type does not admit",
                part: w.part.to_owned(),
            });
        }
    }
    w.attr(el, name, before, after)
}

/// An element's own text.
pub(crate) fn sync_own_text<T: Lexical + PartialEq>(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&T>,
    after: Option<&T>,
) -> R {
    if before == after {
        return Ok(());
    }
    set_text(w, el, &after.map_or(String::new(), Lexical::to_lexical))
}

fn set_text(w: &mut Writer<'_>, el: &Elem, value: &str) -> R {
    w.set_text(el, value)
}

/// Bring a run of like children to the wanted list, by position. An unchanged child is
/// not touched. A child of another name is replaced where it stands.
#[allow(clippy::too_many_arguments)] // The list, its names, and how to write one.
fn sync_seq<T: PartialEq>(
    w: &mut Writer<'_>,
    parent: &Elem,
    order: &[&str],
    existing: Vec<Elem>,
    before: &[T],
    after: &[T],
    name_of: impl Fn(&T) -> ItemName,
    mut write: impl FnMut(&mut Writer<'_>, &Elem, Option<&T>, &T) -> R,
) -> R {
    if existing.len() != before.len() {
        return Err(ExportError::MalformedElement {
            part: w.part.to_owned(),
        });
    }
    if before == after {
        return Ok(());
    }
    let mut last = None;
    for (at, wanted) in after.iter().enumerate() {
        let (uris, prefix, local) = name_of(wanted);
        match (before.get(at), existing.get(at)) {
            (Some(old), Some(holder)) => {
                if holder.is(uris, local) {
                    if old != wanted {
                        write(w, holder, Some(old), wanted)?;
                    }
                    last = Some(holder.id);
                } else {
                    let fresh = w.new_child(parent, uris, prefix, local)?;
                    w.tree.insert_child(parent.id, Some(holder.id), fresh.id);
                    w.tree.remove_child(parent.id, holder.id);
                    w.changes += 1;
                    write(w, &fresh, None, wanted)?;
                    last = Some(fresh.id);
                }
            }
            _ => {
                let fresh = w.new_child(parent, uris, prefix, local)?;
                let anchor = match last {
                    Some(id) => {
                        let kids = parent.children(w.tree, w.part)?;
                        let at = kids.iter().position(|k| k.id == id);
                        at.and_then(|i| kids.get(i + 1)).map(|k| k.id)
                    }
                    None => None,
                };
                if last.is_some() {
                    w.tree.insert_child(parent.id, anchor, fresh.id);
                } else {
                    w.insert_in_order(parent, order, local, &fresh)?;
                }
                write(w, &fresh, None, wanted)?;
                last = Some(fresh.id);
            }
        }
    }
    for holder in existing.iter().skip(after.len()) {
        w.tree.remove_child(parent.id, holder.id);
        w.changes += 1;
    }
    Ok(())
}

/// A repeating choice.
#[allow(clippy::too_many_arguments)] // As `sync_seq`.
pub(crate) fn sync_items<T: PartialEq>(
    w: &mut Writer<'_>,
    parent: &Elem,
    order: &[&str],
    names: &[ItemName],
    before: &[T],
    after: &[T],
    name_of: impl Fn(&T) -> ItemName,
    write: impl FnMut(&mut Writer<'_>, &Elem, Option<&T>, &T) -> R,
) -> R {
    let existing = parent
        .children(w.tree, w.part)?
        .into_iter()
        .filter(|c| names.iter().any(|(uris, _, local)| c.is(uris, local)))
        .collect();
    sync_seq(w, parent, order, existing, before, after, name_of, write)
}

/// A child that repeats.
#[allow(clippy::too_many_arguments)] // As `sync_seq`.
pub(crate) fn sync_many<T: PartialEq>(
    w: &mut Writer<'_>,
    parent: &Elem,
    uris: &'static [&'static str],
    prefix: &'static str,
    order: &[&str],
    local: &'static str,
    before: &[T],
    after: &[T],
    write: impl FnMut(&mut Writer<'_>, &Elem, Option<&T>, &T) -> R,
) -> R {
    let existing = parent
        .children(w.tree, w.part)?
        .into_iter()
        .filter(|c| c.is(uris, local))
        .collect();
    sync_seq(
        w,
        parent,
        order,
        existing,
        before,
        after,
        |_| (uris, prefix, local),
        write,
    )
}

const SP_PR_ORDER: &[&str] = &[
    "xfrm",
    "custGeom",
    "prstGeom",
    "noFill",
    "solidFill",
    "gradFill",
    "blipFill",
    "pattFill",
    "grpFill",
    "ln",
    "effectLst",
    "effectDag",
    "scene3d",
    "sp3d",
    "extLst",
];

/// `a:CT_ShapeProperties`, written by Tier 1E's writers.
pub(crate) fn patch_sp_pr(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&ShapeProps>,
    a: &ShapeProps,
) -> R {
    w.attr(
        el,
        "bwMode",
        b.and_then(|b| b.bw_mode.as_ref()),
        a.bw_mode.as_ref(),
    )?;
    sync_ordered(
        w,
        el,
        ns::DML,
        "a",
        SP_PR_ORDER,
        "xfrm",
        b.and_then(|b| b.transform.as_ref()),
        a.transform.as_ref(),
        patch_transform_part,
    )?;
    let empty = casual_slide_model::drawing::ShapeProperties::default();
    let before = properties_node(b.map_or(&empty, |b| &b.properties), "spPr");
    let after = properties_node(&a.properties, "spPr");
    patch(w, el, Some(&before), &after)
}

/// `a:CT_Transform2D`, and a group's.
pub(crate) fn patch_transform_part(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&Transform>,
    a: &Transform,
) -> R {
    patch_transform(w, el, true, b, a)
}

pub(crate) fn patch_text_body_part(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&TextBody>,
    a: &TextBody,
) -> R {
    patch_text_body(w, el, b, a)
}

pub(crate) fn patch_list_style_part(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&TextListStyle>,
    a: &TextListStyle,
) -> R {
    patch_list_style(w, el, b, a)
}

pub(crate) fn patch_character_part(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&CharacterProperties>,
    a: &CharacterProperties,
) -> R {
    patch_character(w, el, b, a)
}

pub(crate) fn patch_style_part(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&ShapeStyle>,
    a: &ShapeStyle,
) -> R {
    patch_style(w, el, b, a)
}

// spec-audit: owner=cNvPr
pub(crate) fn patch_identity(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&DrawingIdentity>,
    a: &DrawingIdentity,
) -> R {
    w.attr(el, "id", b.and_then(|b| b.id.as_ref()), a.id.as_ref())?;
    w.attr(el, "name", b.and_then(|b| b.name.as_ref()), a.name.as_ref())?;
    w.attr(
        el,
        "descr",
        b.and_then(|b| b.descr.as_ref()),
        a.descr.as_ref(),
    )?;
    w.attr(
        el,
        "hidden",
        b.and_then(|b| b.hidden.as_ref()),
        a.hidden.as_ref(),
    )?;
    w.attr(
        el,
        "title",
        b.and_then(|b| b.title.as_ref()),
        a.title.as_ref(),
    )
}

// spec-audit: owner=cNvSpPr
pub(crate) fn patch_nv_shape(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&NvShapeProps>,
    a: &NvShapeProps,
) -> R {
    w.attr(
        el,
        "txBox",
        b.and_then(|b| b.tx_box.as_ref()),
        a.tx_box.as_ref(),
    )
}

pub(crate) fn patch_nv_group(
    _: &mut Writer<'_>,
    _: &Elem,
    _: Option<&NvGroupProps>,
    _: &NvGroupProps,
) -> R {
    Ok(())
}

pub(crate) fn patch_scene3d(w: &mut Writer<'_>, el: &Elem, b: Option<&Scene3D>, a: &Scene3D) -> R {
    let of = |s: &Scene3D| scene3d_node(s, Space::Dml, "scene3d");
    patch(w, el, b.map(of).as_ref(), &of(a))
}

pub(crate) fn patch_sp3d(w: &mut Writer<'_>, el: &Elem, b: Option<&Shape3D>, a: &Shape3D) -> R {
    let of = |s: &Shape3D| sp3d_node(s, Space::Dml, "sp3d");
    patch(w, el, b.map(of).as_ref(), &of(a))
}

/// The `EG_Text3D` choice, among whatever else the parent holds.
pub(crate) fn patch_text3d(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&Text3D>,
    a: Option<&Text3D>,
) -> R {
    if b == a {
        return Ok(());
    }
    let of = |t: &Text3D| match t {
        Text3D::Shape(s) => sp3d_node(s, Space::Dml, "sp3d"),
        Text3D::Flat(f) => flat_text_node(f),
    };
    let local = |t: &Text3D| match t {
        Text3D::Shape(_) => "sp3d",
        Text3D::Flat(_) => "flatTx",
    };
    if let Some(old) = b
        && a.is_none_or(|a| local(a) != local(old))
    {
        w.remove_all(el, ns::DML, local(old))?;
    }
    let Some(wanted) = a else {
        return Ok(());
    };
    let target = match el.first_child(w.tree, w.part, ns::DML, local(wanted))? {
        Some(found) => found,
        None => {
            let fresh = w.new_child(el, ns::DML, "a", local(wanted))?;
            // `sp3d` and `flatTx` end what they stand in, before any `extLst`.
            w.insert_in_order(el, &["sp3d", "flatTx", "extLst"], local(wanted), &fresh)?;
            fresh
        }
    };
    let same = b.filter(|old| local(old) == local(wanted)).map(of);
    patch(w, &target, same.as_ref(), &of(wanted))
}

/// `a:EG_ColorChoice`, repeated: the colours the reader could read, by position.
pub(crate) fn patch_color_list(w: &mut Writer<'_>, el: &Elem, b: &[Color], a: &[Color]) -> R {
    if b == a {
        return Ok(());
    }
    let mut existing = Vec::new();
    for child in el.children(w.tree, w.part)? {
        if is_color_element(w.tree, child.id) {
            let mut quiet = casual_slide_import::color::Findings::default();
            if casual_slide_import::color::read_color(w.tree, child.id, &mut quiet).is_some() {
                existing.push(child);
            }
        }
    }
    sync_seq(
        w,
        el,
        &[],
        existing,
        b,
        a,
        |c| (ns::DML, "a", color_node(c).local),
        |w, el, old, new| patch(w, el, old.map(color_node).as_ref(), &color_node(new)),
    )
}

/// `a:CT_BackgroundFormatting`, `a:CT_WholeE2oFormatting`.
pub(crate) fn patch_formatting(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&Formatting>,
    a: &Formatting,
) -> R {
    let of = |f: &Formatting| {
        Node::a("bg")
            .opt(f.fill.as_ref().map(fill_node))
            .opt(f.line.as_ref().map(|l| line_node(l, Space::Dml, "ln")))
            .opt(f.effects.as_ref().map(effect_properties_node))
    };
    patch(w, el, b.map(of).as_ref(), &of(a))
}

/// `extLst`: the extensions this tier knows, by position. Any other stays.
pub(crate) fn patch_ext_lst(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&Extensions>,
    a: &Extensions,
) -> R {
    let before = b.map_or(&[][..], |b| b.items.as_slice());
    let mut known = Vec::new();
    for ext in el.children(w.tree, w.part)? {
        if !ext.is(ns::DML, "ext") {
            continue;
        }
        let kids = ext.children(w.tree, w.part)?;
        if kids
            .iter()
            .any(casual_slide_import::diagram::schema::is_root)
        {
            known.push(ext);
        }
    }
    sync_seq(
        w,
        el,
        &[],
        known,
        before,
        &a.items,
        |_| (ns::DML, "a", "ext"),
        patch_extension,
    )
}

fn patch_extension(w: &mut Writer<'_>, el: &Elem, b: Option<&Extension>, a: &Extension) -> R {
    w.attr(el, "uri", b.and_then(|b| b.uri.as_ref()), a.uri.as_ref())?;
    let (uris, prefix, local) = schema::root_name(&a.body);
    // An `a:ext` holds one known extension, so another's goes, or the reader would find it.
    for kid in el.children(w.tree, w.part)? {
        if casual_slide_import::diagram::schema::is_root(&kid) && !kid.is(uris, local) {
            w.tree.remove_child(el.id, kid.id);
            w.changes += 1;
        }
    }
    let same = b.filter(|b| schema::root_name(&b.body).2 == local);
    let holder = match el.first_child(w.tree, w.part, uris, local)? {
        Some(holder) => holder,
        None => {
            let fresh = w.new_child(el, uris, prefix, local)?;
            w.tree.insert_child(el.id, None, fresh.id);
            fresh
        }
    };
    schema::patch_root(w, &holder, same.map(|b| &b.body), &a.body)
}

/// Why a whole-part patch stopped.
fn read_error(err: GraphicError, part: &str) -> SlideExportError {
    match err {
        GraphicError::Import(err) => SlideExportError::Export(ExportError::from(err)),
        GraphicError::Bound { .. } => SlideExportError::Export(ExportError::MalformedElement {
            part: part.to_owned(),
        }),
    }
}

/// Bring a tree holding a standalone element of `diagram` to a value. Only the nodes that
/// differ change. The result is read back whole and must equal `after`.
#[allow(clippy::too_many_arguments)] // As `patch_slide_part`.
pub fn patch_diagram_part(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &m::Root,
    limits: &GraphicLimits,
    import_limits: &ImportLimits,
    xml: &OoxmlLimits,
) -> Result<usize, SlideExportError> {
    let read = |tree: &XmlTree| {
        let mut findings = Findings::new(import_limits.max_findings);
        import_diagram_root(tree, part, rels, limits, import_limits, &mut findings)
    };
    let before = read(tree).map_err(|e| read_error(e, part))?;
    if before.as_ref() == Some(after) {
        return Ok(0);
    }
    let root = Elem::root(tree, part).map_err(ExportError::from)?;
    if before.is_none() || schema::root_name(after).2 != root.local {
        return Err(SlideExportError::Unsupported {
            what: "a part of another kind",
            part: part.to_owned(),
        });
    }
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect::<HashSet<_>>()),
        changes: 0,
    };
    schema::patch_root(&mut w, &root, before.as_ref(), after)?;
    let changes = w.changes;
    let again = XmlTree::parse(&tree.to_bytes(), part, xml).map_err(ExportError::from)?;
    if read(&again).map_err(|e| read_error(e, part))?.as_ref() != Some(after) {
        return Err(SlideExportError::Export(ExportError::RoundTrip {
            part: part.to_owned(),
        }));
    }
    Ok(changes)
}

fn as_root(diagram: &Diagram, role: DiagramRole) -> Option<m::Root> {
    Some(match role {
        DiagramRole::Data => m::Root::DataModel(Box::new(diagram.data.clone()?)),
        DiagramRole::Layout => m::Root::LayoutDef(Box::new(diagram.layout.clone()?)),
        DiagramRole::Style => m::Root::StyleDef(Box::new(diagram.style.clone()?)),
        DiagramRole::Colors => m::Root::ColorsDef(Box::new(diagram.colors.clone()?)),
        DiagramRole::Drawing => m::Root::DspDrawing(Box::new(diagram.drawing.clone()?)),
    })
}

/// A data model with every point's text taken out, and the drawing's text likewise, which
/// is what must not change if a data edit is to leave the drawing as it stands.
fn without_text(diagram: &Diagram) -> (Option<m::DataModel>, Option<m::DspDrawing>) {
    let mut data = diagram.data.clone();
    if let Some(list) = data.as_mut().and_then(|d| d.pt_lst.as_deref_mut()) {
        for pt in &mut list.pt {
            pt.t = None;
        }
    }
    let mut drawing = diagram.drawing.clone();
    if let Some(tree) = drawing.as_mut().and_then(|d| d.sp_tree.as_deref_mut()) {
        let mut todo = vec![tree];
        while let Some(group) = todo.pop() {
            for item in &mut group.items {
                match item {
                    m::DspGroupShapeItem::GrpSp(g) => todo.push(g),
                    m::DspGroupShapeItem::Sp(s) => s.tx_body = None,
                }
            }
        }
    }
    (data, drawing)
}

/// What a saved diagram leaves stale, for the host's report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagramSave {
    /// The saved deck, the parts changed, and whether a signature broke.
    pub outcome: SaveOutcome,
    /// `CS-SAV-0007`: definitions edited, so the drawing is what the last layout made.
    pub stale_drawing: bool,
}

/// Whether a data edit would leave `dsp:drawing` showing something else. A text edit the
/// drawing shows too is not stale, and neither is one where the drawing is set aside or
/// absent, since then nothing shows it at all.
fn data_edit_is_stale(before: &Diagram, after: &Diagram) -> bool {
    if before.data == after.data || before.drawing.is_none() {
        return false;
    }
    let (data_b, _) = without_text(before);
    let (data_a, _) = without_text(after);
    if data_b != data_a {
        return true;
    }
    // Text only: each changed point must have its body shown changed alike.
    let shapes: Vec<_> = after.drawing_shapes();
    for (old, new) in before.points().iter().zip(after.points()) {
        if old.t == new.t {
            continue;
        }
        let Some(id) = new.model_id.as_ref() else {
            return true;
        };
        let holds = |s: &&&m::DspShape| {
            s.model_id.as_ref().is_some_and(|mid| {
                mid.eq_ignore_ascii_case(id)
                    || after.points().iter().any(|p| {
                        p.model_id
                            .as_ref()
                            .is_some_and(|pid| pid.eq_ignore_ascii_case(mid))
                            && p.pr_set
                                .as_deref()
                                .and_then(|set| set.pres_assoc_id.as_ref())
                                .is_some_and(|assoc| assoc.eq_ignore_ascii_case(id))
                    })
            })
        };
        let text_of = |body: Option<&TextBody>| -> Option<Vec<Option<String>>> {
            body.map(|b| {
                b.paragraphs
                    .iter()
                    .flat_map(|p| &p.items)
                    .filter_map(|i| match i {
                        casual_slide_model::text::TextItem::Run(r) => Some(r.text.clone()),
                        _ => None,
                    })
                    .collect()
            })
        };
        let wanted = text_of(new.t.as_deref());
        let shown: Vec<_> = shapes.iter().filter(|s| holds(s)).collect();
        if shown.is_empty()
            || shown
                .iter()
                .any(|s| text_of(s.tx_body.as_deref()) != wanted)
        {
            return true;
        }
    }
    false
}

/// Save a diagram over the package it came from. Only the parts that differ change, and
/// only their differing nodes; with nothing changed, the input bytes come back.
#[allow(clippy::too_many_arguments)] // As `save_table_styles`.
pub fn save_diagram(
    package: &mut PresentationPackage,
    slide_part: &str,
    rel_ids: &m::RelIds,
    after: &Diagram,
    limits: &GraphicLimits,
    import_limits: &ImportLimits,
) -> Result<DiagramSave, SlideExportError> {
    let mut findings = Findings::new(import_limits.max_findings);
    let before = import_diagram(
        package,
        slide_part,
        rel_ids,
        limits,
        import_limits,
        &mut findings,
    );
    if data_edit_is_stale(&before, after) {
        return Err(SlideExportError::Blocked {
            what: "a data edit the stored drawing does not show, with no layout engine to redraw it",
            part: slide_part.to_owned(),
        });
    }
    let strict = package.conformance() == Conformance::Strict;
    let xml = *package.limits();
    let mut edits = PartEdits::default();
    let mut changed = Vec::new();
    let mut definitions_changed = false;
    for role in DiagramRole::ALL {
        let (old, new) = (as_root(&before, role), as_root(after, role));
        if old == new {
            continue;
        }
        let (Some(name), Some(new)) = (before.part(role), new) else {
            return Err(SlideExportError::Unsupported {
                what: "adding or removing one of a diagram's parts",
                part: slide_part.to_owned(),
            });
        };
        if before.is_set_aside(role) {
            return Err(SlideExportError::Unsupported {
                what: "an edit of a part set aside on import",
                part: name.to_owned(),
            });
        }
        let bytes = package.read_xml_part(name).map_err(ExportError::from)?;
        let mut tree = XmlTree::parse(&bytes, name, &xml).map_err(ExportError::from)?;
        let rels = package.relationships_of(name).to_vec();
        let n = patch_diagram_part(
            &mut tree,
            name,
            strict,
            &rels,
            &new,
            limits,
            import_limits,
            &xml,
        )?;
        let out = tree.to_bytes();
        if n > 0 && out != bytes {
            edits.replace(name.to_owned(), out);
            changed.push(name.to_owned());
            if matches!(
                role,
                DiagramRole::Layout | DiagramRole::Style | DiagramRole::Colors
            ) {
                definitions_changed = true;
            }
        }
    }
    if changed.is_empty() {
        return Ok(DiagramSave {
            outcome: SaveOutcome {
                bytes: package.save_unedited(),
                changed_parts: Vec::new(),
                signature_invalidated: false,
            },
            stale_drawing: false,
        });
    }
    let signed = package
        .findings()
        .iter()
        .any(|f| f.kind == casual_slide_ooxml::FindingKind::Signature);
    Ok(DiagramSave {
        outcome: SaveOutcome {
            bytes: package.save_with(&edits).map_err(ExportError::from)?,
            changed_parts: changed,
            signature_invalidated: signed,
        },
        stale_drawing: definitions_changed,
    })
}
