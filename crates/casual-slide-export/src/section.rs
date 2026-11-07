//! Patch the section list and a slide-like part's zooms and marks (Tier 1M). What a section,
//! a zoom or a mark holds beyond what the model says stays in the tree: a section's `extLst`, a
//! zoom's `mc:Fallback`, a frame's position, `xfrm` in a zoom's `spPr`.
//!
//! Sections match by `id`, so taking one away leaves the rest of their elements alone. A slide
//! moved, added or deleted changes `sldId`s and nothing else. The writer's slide operations
//! keep this list in step on their own (`INV-SL-04`, `INV-SL-06`); this is for an edit of the
//! list itself, and preflight holds either to `INV-SL-*`.
//!
//! The first section list in a deck goes in an `ext` with the `uri` of `INV-SL-01`. Removing
//! the last takes the `ext` away (`INV-SL-02`), and any other `ext` stays.
//!
//! A zoom or mark added or removed moves shape ids and fallbacks together, which are edit
//! operations for Phase 4, and is refused as `CS-EXP-0009`.

use casual_slide_import::comment::ReviewLimits;
use casual_slide_import::properties::{Elem, Findings, ImportLimits, ns};
use casual_slide_import::section::{import_part_marks, mark_elements};
use casual_slide_model::drawing::ShapeProperties;
use casual_slide_model::section::{
    ClassificationMark, DesignerMark, DesignerTag, PartMarks, Section, SectionList,
    SectionZoomObject, SlideZoomObject, SummaryLayout, SummaryZoomObject, Zoom, ZoomFrame,
    ZoomProperties,
};
use casual_slide_ooxml::{OoxmlLimits, Relationship, XmlTree};

use crate::comment::{
    ACS, P14, P16, P166, P184, P202, PSEZ, PSLZ, PSUZ, malformed, sync_items, unsupported,
};
use crate::drawing::{Node, Space, patch};
use crate::effect::effect_properties_node;
use crate::fill::{blip_fill_node, fill_node};
use crate::geometry::geometry_node;
use crate::line::line_node;
use crate::presentation::PRESENTATION_ORDER;
use crate::properties::{ExportError, Writer, verify};

type Result<T = ()> = std::result::Result<T, ExportError>;

/// The section list's `ext`, as `docs/36` has it (`INV-SL-01`).
const SECTION_URI: &str = "{521415D9-36F7-43E2-AB2F-B90AF26B5E84}";

// spec-audit: owner=section
fn section_node(s: &Section) -> Node {
    Node::new(P14, "section")
        .lex("name", s.name.as_ref())
        .lex("id", s.id.as_ref())
        .opt(s.sld_id_lst.as_ref().map(|list| {
            Node::new(P14, "sldIdLst").list(
                list.iter()
                    .map(|id| Node::new(P14, "sldId").lex("id", Some(id))),
            )
        }))
}

/// Bring the main part's section list from `before` to `after`.
// spec-audit: owner=sectionLst
// spec-audit: x:powerpoint/2010/main:<sectionLst>
pub(crate) fn patch_sections(
    w: &mut Writer<'_>,
    root: &Elem,
    before: Option<&SectionList>,
    after: Option<&SectionList>,
) -> Result {
    if before == after {
        return Ok(());
    }
    if after.is_some_and(|l| l.sections.is_empty()) {
        // INV-SL-02: the schema wants at least one. Say none by removing the list.
        return Err(unsupported(w, "a section list holding no section"));
    }
    let lists = root.children(w.tree, w.part)?;
    let ext_list = lists.into_iter().find(|c| c.is(ns::PML, "extLst"));
    let mut found = None;
    if let Some(list) = &ext_list {
        for ext in list.children(w.tree, w.part)? {
            for item in ext.children(w.tree, w.part)? {
                if found.is_none() && item.is(P14.uris(), "sectionLst") {
                    found = Some((ext.clone(), item));
                }
            }
        }
    }
    match (found, after) {
        (Some((ext, item)), None) => {
            w.tree.remove_child(ext.id, item.id);
            w.changes += 1;
            if let Some(list) = &ext_list
                && ext.children(w.tree, w.part)?.is_empty()
            {
                w.tree.remove_child(list.id, ext.id);
                if list.children(w.tree, w.part)?.is_empty() {
                    w.tree.remove_child(root.id, list.id);
                }
            }
            Ok(())
        }
        (None, None) => Ok(()),
        (found, Some(after)) => {
            let item = match found {
                Some((_, item)) => item,
                None => {
                    let list = match ext_list {
                        Some(list) => list,
                        None => {
                            let list = w.new_child(root, ns::PML, "p", "extLst")?;
                            w.insert_in_order(root, PRESENTATION_ORDER, "extLst", &list)?;
                            list
                        }
                    };
                    let ext = w.new_child(&list, ns::PML, "p", "ext")?;
                    w.tree.insert_child(list.id, None, ext.id);
                    w.attr(&ext, "uri", None::<&String>, Some(&SECTION_URI.to_owned()))?;
                    let item = w.new_child(&ext, P14.uris(), P14.prefix(), "sectionLst")?;
                    w.tree.insert_child(ext.id, None, item.id);
                    item
                }
            };
            sync_items(
                w,
                &item,
                P14,
                "section",
                before.map_or(&[][..], |l| l.sections.as_slice()),
                &after.sections,
                |s| s.id.as_ref().map(|i| i.to_ascii_uppercase()),
                |w, el, b, a| patch(w, el, b.map(section_node).as_ref(), &section_node(a)),
            )
        }
    }
}

// spec-audit: owner=spPr
fn zoom_shape_properties(p: &ShapeProperties) -> Node {
    Node::new(P166, "spPr")
        .opt(p.geometry.as_ref().map(geometry_node))
        .opt(p.fill.as_ref().map(fill_node))
        .opt(p.line.as_ref().map(|l| line_node(l, Space::Dml, "ln")))
        .opt(p.effects.as_ref().map(effect_properties_node))
        .opt(
            p.scene3d
                .as_ref()
                .map(|s| crate::d3::scene3d_node(s, Space::Dml, "scene3d")),
        )
        .opt(
            p.sp3d
                .as_ref()
                .map(|s| crate::d3::sp3d_node(s, Space::Dml, "sp3d")),
        )
}

// spec-audit: owner=zmPr
fn zoom_props_node(p: &ZoomProperties) -> Node {
    Node::new(P166, "zmPr")
        .lex("id", p.id.as_ref())
        .lex("returnToParent", p.return_to_parent.as_ref())
        .lex(
            "imageType",
            p.image_type.map(|t| t.as_xml().to_owned()).as_ref(),
        )
        .lex("transitionDur", p.transition_dur.as_ref())
        .lex("showBg", p.show_bg.as_ref())
        .opt(p.blip_fill.as_ref().map(|f| blip_fill_node(f, P166)))
        .opt(p.sp_pr.as_ref().map(zoom_shape_properties))
}

// spec-audit: owner=sldZmObj
fn slide_object_node(o: &SlideZoomObject) -> Node {
    Node::new(PSLZ, "sldZmObj")
        .lex("sldId", o.sld_id.as_ref())
        .lex("cId", o.c_id.as_ref())
        .opt(o.zm_pr.as_ref().map(zoom_props_node))
}

// spec-audit: owner=sectionZmObj
fn section_object_node(o: &SectionZoomObject) -> Node {
    Node::new(PSEZ, "sectionZmObj")
        .lex("sectionId", o.section_id.as_ref())
        .opt(o.zm_pr.as_ref().map(zoom_props_node))
}

// spec-audit: owner=summaryZmObj
fn summary_object_node(o: &SummaryZoomObject) -> Node {
    Node::new(PSUZ, "summaryZmObj")
        .lex("sectionId", o.section_id.as_ref())
        .lex("title", o.title.as_ref())
        .lex("descr", o.descr.as_ref())
        .lex("offsetFactorX", o.offset_factor_x.as_ref())
        .lex("offsetFactorY", o.offset_factor_y.as_ref())
        .lex("scaleFactorX", o.scale_factor_x.as_ref())
        .lex("scaleFactorY", o.scale_factor_y.as_ref())
        .opt(o.zm_pr.as_ref().map(zoom_props_node))
}

// spec-audit: owner=sldZm,sectionZm,summaryZm
// spec-audit: x:powerpoint/2016/slidezoom:<sldZm>
// spec-audit: x:powerpoint/2016/sectionzoom:<sectionZm>
// spec-audit: x:powerpoint/2016/summaryzoom:<summaryZm>
fn zoom_node(z: &ZoomFrame) -> Node {
    match &z.zoom {
        Zoom::Slide(o) => Node::new(PSLZ, "sldZm").opt(o.as_ref().map(slide_object_node)),
        Zoom::Section(o) => Node::new(PSEZ, "sectionZm").opt(o.as_ref().map(section_object_node)),
        Zoom::Summary { objects, layout } => Node::new(PSUZ, "summaryZm")
            .list(objects.iter().map(summary_object_node))
            .opt(layout.map(|l| match l {
                SummaryLayout::Grid => Node::new(PSUZ, "gridLayout"),
                SummaryLayout::Fixed => Node::new(PSUZ, "fixedLayout"),
            })),
    }
}

// spec-audit: owner=classification
fn classification_node(m: &ClassificationMark, el: &Elem) -> Node {
    let token = m.outcome.map(|o| o.as_xml().to_owned());
    if el.is(P184.uris(), "classification") {
        Node::new(P184, "classification").lex("val", token.as_ref())
    } else {
        Node::new(ACS, "classification").lex("classificationOutcomeType", token.as_ref())
    }
}

// spec-audit: owner=designTag
fn tag_node(t: &DesignerTag) -> Node {
    Node::new(P202, "designTag")
        .lex("name", t.name.as_ref())
        .lex("val", t.val.as_ref())
}

// spec-audit: owner=designElem,designPr,designTagLst
// spec-audit: x:powerpoint/2015/main:<designElem>
// spec-audit: x:powerpoint/2020/02/main:<designPr>
// spec-audit: x:powerpoint/2020/02/main:<designTagLst>
fn designer_node(m: &DesignerMark) -> Node {
    match m {
        DesignerMark::DesignElement { val, .. } => {
            Node::new(P16, "designElem").lex("val", val.as_ref())
        }
        DesignerMark::DrawingProperties {
            edt_design_elem,
            design_tag_lst,
            ..
        } => Node::new(P202, "designPr")
            .lex("edtDesignElem", edt_design_elem.as_ref())
            .opt(
                design_tag_lst
                    .as_ref()
                    .map(|l| Node::new(P202, "designTagLst").list(l.iter().map(tag_node))),
            ),
    }
}

/// Patch a slide-like part's zooms and marks to say `after`. Returns the nodes changed.
/// `commentRel` and `custDataLst` are relationship references, which follow from the writer's
/// relationship operations, and an edit of one here is refused.
#[allow(clippy::too_many_arguments)]
pub fn patch_part_marks(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &PartMarks,
    limits: &ReviewLimits,
    import: &ImportLimits,
    xml: &OoxmlLimits,
) -> Result<usize> {
    let read = |t: &XmlTree| {
        import_part_marks(
            t,
            part,
            rels,
            limits,
            import,
            &mut Findings::new(import.max_findings),
        )
    };
    let before = read(tree)?;
    if &before == after {
        return Ok(0);
    }
    let found = mark_elements(tree, part, limits)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
        changes: 0,
    };
    if before.comment_rel != after.comment_rel || before.customer_data != after.customer_data {
        return Err(unsupported(&w, "a relationship reference in a part's marks"));
    }
    if before.zooms.len() != after.zooms.len()
        || before.classifications.len() != after.classifications.len()
        || before.designer.len() != after.designer.len()
    {
        return Err(unsupported(&w, "adding or removing a zoom or a mark"));
    }
    if found.zooms.len() != after.zooms.len()
        || found.classifications.len() != after.classifications.len()
        || found.designer.len() != after.designer.len()
    {
        return Err(malformed(&w));
    }
    for ((el, _, _), (b, a)) in found.zooms.iter().zip(before.zooms.iter().zip(&after.zooms)) {
        if b.fallback != a.fallback || b.shape_id != a.shape_id {
            return Err(unsupported(&w, "a zoom's fallback or frame"));
        }
        if std::mem::discriminant(&b.zoom) != std::mem::discriminant(&a.zoom) {
            return Err(unsupported(&w, "changing a zoom's kind"));
        }
        patch(&mut w, el, Some(&zoom_node(b)), &zoom_node(a))?;
    }
    for ((el, _), (b, a)) in found
        .classifications
        .iter()
        .zip(before.classifications.iter().zip(&after.classifications))
    {
        patch(
            &mut w,
            el,
            Some(&classification_node(b, el)),
            &classification_node(a, el),
        )?;
    }
    for ((el, _), (b, a)) in found
        .designer
        .iter()
        .zip(before.designer.iter().zip(&after.designer))
    {
        let (node_b, node_a) = (designer_node(b), designer_node(a));
        if !node_b.same_name(&node_a) {
            return Err(unsupported(&w, "changing what Designer's mark is"));
        }
        patch(&mut w, el, Some(&node_b), &node_a)?;
    }
    let changes = w.changes;
    verify(tree, part, xml, after, read)?;
    Ok(changes)
}
