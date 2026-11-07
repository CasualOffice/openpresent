//! Read sections, zoom objects, classification marks and Designer's marks (Tier 1M).
//!
//! `p14:sectionLst` comes from the main part's `extLst`. The rest come from a slide-like
//! part by a walk of its tree that is apart from the shape reader, so that what a slide says
//! about its shapes is unchanged whether or not anyone asks about zooms. A node in a branch of
//! `mc:AlternateContent` is read here for what it says: that is where a zoom is.
//!
//! A section list out of step with `sldIdLst` opens and is kept, and is reported, because
//! what PowerPoint does with one is Unverified (`INV-SL-05`). An over-long list is reported
//! and left out, and stays in the tree.
//!
//! Evidence: `MS-PPTX` schemas in `schemas/ms-extensions`; `INV-SL-*`, `INV-FB-05` in `docs/36`.
//! Where each mark may stand in a part is Inferred: a mark is read wherever an `extLst` holds it.

use std::collections::HashSet;

use casual_slide_model::drawing::ShapeProperties;
use casual_slide_model::presentation::{Percentage, Presentation};
use casual_slide_model::section::{
    ClassificationMark, ClassificationOutcome, DesignerMark, DesignerTag, PartMarks, Section,
    SectionList, SectionZoomObject, SlideZoomObject, SummaryLayout, SummaryZoomObject, Zoom,
    ZoomFallback, ZoomFrame, ZoomImageType, ZoomProperties,
};
use casual_slide_ooxml::{Relationship, XmlTree};

use crate::comment::{P14, ReviewFinding, ReviewLimits, extension_items, first, guid_attr};
use crate::drawing::{Ctx, DrawingLimits, read_blip_fill};
use crate::properties::{
    Elem, Findings, ImportError, ImportFindingKind as K, ImportLimits, Reader, ns,
};

/// `p166:`.
pub const P166: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2016/6/main"];
/// `pslz:`.
pub const PSLZ: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2016/slidezoom"];
/// `psez:`.
pub const PSEZ: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2016/sectionzoom"];
/// `psuz:`.
pub const PSUZ: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2016/summaryzoom"];
/// `p16:`.
pub const P16: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2015/main"];
/// `p184:`.
pub const P184: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2018/4/main"];
/// `acs:`.
pub const ACS: &[&str] = &["http://schemas.microsoft.com/office/drawing/2020/classificationShape"];
/// `p202:`.
pub const P202: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2020/02/main"];
/// `p188:`, for `commentRel`.
pub use crate::comment::P188;
/// `mc:`.
pub const MC: &[&str] = &["http://schemas.openxmlformats.org/markup-compatibility/2006"];

/// The main part's `extLst`, with the section list out of it. Anything else stays in the tree,
/// and is reported as retained. Called from the presentation reader.
pub(crate) fn read_presentation_extensions(
    r: &mut Reader<'_>,
    el: &Elem,
    limits: &ReviewLimits,
) -> Result<Option<SectionList>, ImportError> {
    let mut sections = None;
    let mut any_other = false;
    for item in extension_items(r, el)? {
        if item.is(P14, "sectionLst") && sections.is_none() {
            sections = read_section_list(r, &item, limits)?;
        } else if !item.is(P14, "sectionLst") {
            any_other = true;
            r.retained(&item);
        } else {
            r.note(K::DuplicateElement, &item, None);
        }
    }
    if sections.is_none() || any_other {
        // Something is kept that the model does not say: `extLst` is what a report names.
        r.retained(el);
    }
    Ok(sections)
}

// spec-audit: owner=sectionLst
// spec-audit: x:powerpoint/2010/main:<sectionLst>
fn read_section_list(
    r: &mut Reader<'_>,
    el: &Elem,
    limits: &ReviewLimits,
) -> Result<Option<SectionList>, ImportError> {
    r.finish(el);
    let kids = r.tree.child_elements(el.id).len();
    if kids > limits.max_sections {
        r.note(K::Review(ReviewFinding::OverBound), el, None);
        return Ok(None);
    }
    let mut list = SectionList::default();
    let mut ids = HashSet::new();
    let mut budget = limits.max_section_slides;
    for child in r.children(el)? {
        if !child.is(P14, "section") {
            r.retained(&child);
            continue;
        }
        let Some(section) = read_section(r, &child, &mut budget)? else {
            r.note(K::Review(ReviewFinding::OverBound), el, None);
            return Ok(None);
        };
        if let Some(id) = &section.id
            && !ids.insert(id.to_ascii_uppercase())
        {
            r.note(K::Review(ReviewFinding::RepeatedId), &child, Some("id"));
        }
        list.sections.push(section);
    }
    Ok(Some(list))
}

// spec-audit: owner=section
fn read_section(
    r: &mut Reader<'_>,
    el: &Elem,
    budget: &mut usize,
) -> Result<Option<Section>, ImportError> {
    let mut e = el.clone();
    let mut out = Section {
        name: r.required(&mut e, "name", |_: &String| true)?,
        id: r.required(&mut e, "id", guid_attr)?,
        sld_id_lst: None,
    };
    r.finish(&e);
    let mut seen = HashSet::new();
    for child in r.children(el)? {
        if child.is(P14, "sldIdLst") {
            if !first(r, &mut seen, "sldIdLst", &child) {
                continue;
            }
            let Some(ids) = read_section_slides(r, &child, budget)? else {
                return Ok(None);
            };
            out.sld_id_lst = Some(ids);
        } else {
            // `extLst` included.
            r.retained(&child);
        }
    }
    if !seen.contains("sldIdLst") {
        r.note(K::MissingRequired, el, None);
    }
    Ok(Some(out))
}

// spec-audit: owner=sldIdLst
fn read_section_slides(
    r: &mut Reader<'_>,
    el: &Elem,
    budget: &mut usize,
) -> Result<Option<Vec<u32>>, ImportError> {
    r.finish(el);
    if r.tree.child_elements(el.id).len() > *budget {
        return Ok(None);
    }
    let mut out = Vec::new();
    for child in r.children(el)? {
        if !child.is(P14, "sldId") {
            r.retained(&child);
            continue;
        }
        *budget = budget.saturating_sub(1);
        if let Some(id) = read_section_slide(r, &child)? {
            out.push(id);
        }
    }
    Ok(Some(out))
}

// spec-audit: owner=sldId
fn read_section_slide(r: &mut Reader<'_>, el: &Elem) -> Result<Option<u32>, ImportError> {
    let mut e = el.clone();
    let id = r.required(&mut e, "id", |_: &u32| true)?;
    r.finish(&e);
    Ok(id)
}

/// Report how a section list disagrees with `sldIdLst`. Each kind once: an id is not repeated
/// in a report, and a hostile list can hold every kind many times.
pub(crate) fn check_sections(p: &Presentation, part: &str, findings: &mut Findings) {
    let Some(list) = &p.sections else { return };
    let slides: Vec<u32> = p.slides.iter().flatten().map(|s| s.id).collect();
    let sync = list.check_against(&slides);
    let mut say = |bad: bool, kind: ReviewFinding| {
        if bad {
            findings.note(K::Review(kind), part, "sectionLst", None);
        }
    };
    say(!sync.unknown.is_empty(), ReviewFinding::SectionUnknownSlide);
    say(
        !sync.unlisted.is_empty(),
        ReviewFinding::SectionUnlistedSlide,
    );
    say(
        !sync.repeated.is_empty(),
        ReviewFinding::SectionRepeatedSlide,
    );
    say(sync.out_of_order, ReviewFinding::SectionOutOfOrder);
}

/// Elements of a slide-like part this tier reads, each with the id of the shape it is under,
/// in document order. One walk for the reader and the writer, so both name the same element.
#[derive(Debug, Clone, Default)]
pub struct MarkElements {
    /// `sldZm`, `sectionZm` or `summaryZm`, with its frame's id and its fallback.
    pub zooms: Vec<(Elem, Option<u32>, ZoomFallback)>,
    /// `p184:classification` or `acs:classification`.
    pub classifications: Vec<(Elem, Option<u32>)>,
    /// `p16:designElem` or `p202:designPr`.
    pub designer: Vec<(Elem, Option<u32>)>,
    /// `p188:commentRel`.
    pub comment_rels: Vec<Elem>,
    /// `custData` or `tags`, from every `custDataLst`.
    pub customer_data: Vec<Elem>,
    /// How many there were past `max_marks_per_part`, which stay in the tree, unread.
    pub over: usize,
}

/// Find this tier's elements. A `mc:Fallback` is read only where no `mc:Choice` stands beside
/// it: it says again what the choice says. The stack is on the heap, and the tree is already
/// bounded in size and depth by its parser.
pub fn mark_elements(
    tree: &XmlTree,
    part: &str,
    limits: &ReviewLimits,
) -> Result<MarkElements, ImportError> {
    let mut out = MarkElements::default();
    let mut spent = 0_usize;
    let mut charge = |over: &mut usize| {
        spent = spent.saturating_add(1);
        let fits = spent <= limits.max_marks_per_part;
        if !fits {
            *over = over.saturating_add(1);
        }
        fits
    };
    let root = Elem::root(tree, part)?;
    let mut stack = vec![(root, None::<u32>, ZoomFallback::Absent)];
    let kids = |el: &Elem| el.children(tree, part);
    while let Some((el, shape, fallback)) = stack.pop() {
        let mut shape = own_shape_id(&el).or(shape);
        if el.local.starts_with("nv") && el.local.ends_with("Pr") {
            let first_id = kids(&el)?
                .iter()
                .find(|k| k.local == "cNvPr")
                .and_then(own_shape_id);
            shape = first_id.or(shape);
        }
        let mut held = fallback;
        let mut children = kids(&el)?;
        if el.is(MC, "AlternateContent") {
            held = fallback_of(tree, part, &children)?;
            if children.iter().any(|b| b.is(MC, "Choice")) {
                children.retain(|b| !b.is(MC, "Fallback"));
            }
        } else if el.local == "extLst" {
            for ext in children {
                if ext.local != "ext" {
                    continue;
                }
                for item in kids(&ext)? {
                    if item.is(P188, "commentRel") {
                        out.comment_rels.push(item);
                    } else if item.is(P184, "classification") || item.is(ACS, "classification") {
                        if charge(&mut out.over) {
                            out.classifications.push((item, shape));
                        }
                    } else if (item.is(P16, "designElem") || item.is(P202, "designPr"))
                        && charge(&mut out.over)
                    {
                        out.designer.push((item, shape));
                    }
                }
            }
            continue;
        } else if el.is(ns::PML, "custDataLst") {
            for child in children {
                if (child.is(ns::PML, "custData") || child.is(ns::PML, "tags"))
                    && charge(&mut out.over)
                {
                    out.customer_data.push(child);
                }
            }
            continue;
        } else if el.is(ns::PML, "graphicFrame")
            && let Some(zoom) = zoom_in(tree, part, &el)?
        {
            if charge(&mut out.over) {
                let id = kids(&el)?
                    .iter()
                    .find(|k| k.local == "nvGraphicFramePr")
                    .map(kids)
                    .transpose()?
                    .and_then(|n| n.iter().find(|k| k.local == "cNvPr").and_then(own_shape_id));
                out.zooms.push((zoom, id, held));
            }
            continue;
        }
        for kid in children.into_iter().rev() {
            stack.push((kid, shape, held));
        }
    }
    Ok(out)
}

/// One slide-like part being read.
struct Marks<'a, 'r> {
    r: &'a mut Reader<'r>,
    rels: &'a [Relationship],
    limits: &'a ReviewLimits,
    out: PartMarks,
    spent: usize,
}

/// Read what a slide-like part holds of this tier. Nothing refuses but the XML of the part
/// itself: past `max_marks_per_part` the rest stay in the tree, unread, and that is reported.
pub fn import_part_marks(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
) -> Result<PartMarks, ImportError> {
    let found = mark_elements(tree, part, limits)?;
    let root = Elem::root(tree, part)?;
    let mut reader = Reader::new(tree, part, import, findings);
    if found.over > 0 {
        reader.note(K::Review(ReviewFinding::OverBound), &root, None);
    }
    let mut walk = Marks {
        r: &mut reader,
        rels,
        limits,
        out: PartMarks::default(),
        spent: 0,
    };
    for (el, id, fallback) in &found.zooms {
        let zoom = walk.zoom(el)?;
        if *fallback == ZoomFallback::Absent {
            walk.r
                .note(K::Review(ReviewFinding::ZoomNoFallback), el, None);
        }
        walk.out.zooms.push(ZoomFrame {
            shape_id: *id,
            zoom,
            fallback: *fallback,
        });
    }
    for (el, id) in &found.classifications {
        walk.classification(el, *id);
    }
    for (el, id) in &found.designer {
        if el.is(P16, "designElem") {
            walk.design_element(el, *id)?;
        } else {
            walk.design_properties(el, *id)?;
        }
    }
    for el in &found.comment_rels {
        walk.comment_rel(el);
    }
    for el in &found.customer_data {
        walk.customer_data(el);
    }
    Ok(walk.out)
}

impl Marks<'_, '_> {
    fn charge(&mut self, el: &Elem) -> bool {
        self.spent = self.spent.saturating_add(1);
        if self.spent == self.limits.max_marks_per_part.saturating_add(1) {
            self.r.note(K::Review(ReviewFinding::OverBound), el, None);
        }
        self.spent <= self.limits.max_marks_per_part
    }

    // spec-audit: owner=custDataLst
    // spec-audit: p:CT_CustomerDataList/custData
    // spec-audit: p:CT_CustomerDataList/tags
    // spec-audit: owner=custData,tags
    fn customer_data(&mut self, el: &Elem) {
        let mut c = el.clone();
        match c.take_ns(ns::REL, "id") {
            Some(id) => self.out.customer_data.push(id),
            None => self.r.note(K::MissingRequired, el, Some("id")),
        }
        self.r.finish(&c);
    }

    // spec-audit: owner=commentRel
    // spec-audit: x:powerpoint/2018/8/main:<commentRel>
    fn comment_rel(&mut self, el: &Elem) {
        let mut e = el.clone();
        let id = e.take_ns(ns::REL, "id");
        if id.is_none() {
            self.r.note(K::MissingRequired, el, Some("id"));
        }
        self.r.finish(&e);
        if self.out.comment_rel.is_none() {
            self.out.comment_rel = id;
        } else {
            self.r.note(K::DuplicateElement, el, None);
        }
    }

    // spec-audit: owner=classification
    // spec-audit: x:powerpoint/2018/4/main:<classification>
    // spec-audit: x:drawing/2020/classificationShape:<classification>
    fn classification(&mut self, el: &Elem, shape: Option<u32>) {
        let mut e = el.clone();
        let name = if el.is(P184, "classification") {
            "val"
        } else {
            "classificationOutcomeType"
        };
        let raw = e.take(name);
        let outcome = raw.as_deref().and_then(ClassificationOutcome::from_xml);
        if raw.is_some() && outcome.is_none() {
            self.r.note(K::MalformedValue, el, Some(name));
        }
        self.r.finish(&e);
        self.out.classifications.push(ClassificationMark {
            shape_id: shape,
            outcome,
        });
    }

    // spec-audit: owner=designElem
    // spec-audit: x:powerpoint/2015/main:<designElem>
    fn design_element(&mut self, el: &Elem, shape: Option<u32>) -> Result<(), ImportError> {
        let mut e = el.clone();
        let val = self.r.attr(&mut e, "val")?;
        self.r.finish(&e);
        self.out.designer.push(DesignerMark::DesignElement {
            shape_id: shape,
            val,
        });
        Ok(())
    }

    // spec-audit: owner=designPr
    // spec-audit: x:powerpoint/2020/02/main:<designPr>
    fn design_properties(&mut self, el: &Elem, shape: Option<u32>) -> Result<(), ImportError> {
        let mut e = el.clone();
        let edt_design_elem = self.r.attr(&mut e, "edtDesignElem")?;
        self.r.finish(&e);
        let mut design_tag_lst = None;
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if child.is(P202, "designTagLst") {
                if first(self.r, &mut seen, "designTagLst", &child) {
                    design_tag_lst = Some(self.design_tags(&child)?);
                }
            } else {
                self.r.retained(&child);
            }
        }
        self.out.designer.push(DesignerMark::DrawingProperties {
            shape_id: shape,
            edt_design_elem,
            design_tag_lst,
        });
        Ok(())
    }

    // spec-audit: owner=designTagLst
    // spec-audit: x:powerpoint/2020/02/main:<designTagLst>
    fn design_tags(&mut self, el: &Elem) -> Result<Vec<DesignerTag>, ImportError> {
        self.r.finish(el);
        let mut out = Vec::new();
        for child in self.r.children(el)? {
            if child.is(P202, "designTag") && self.charge(&child) {
                let mut c = child.clone();
                out.push(DesignerTag {
                    name: self.r.required(&mut c, "name", |_: &String| true)?,
                    val: self.r.required(&mut c, "val", |_: &String| true)?,
                });
                self.r.finish(&c);
            } else if !child.is(P202, "designTag") {
                self.r.retained(&child);
            }
        }
        Ok(out)
    }

    fn zoom(&mut self, el: &Elem) -> Result<Zoom, ImportError> {
        self.r.finish(el);
        if el.is(PSLZ, "sldZm") {
            self.slide_zoom(el)
        } else if el.is(PSEZ, "sectionZm") {
            self.section_zoom(el)
        } else {
            self.summary_zoom(el)
        }
    }

    // spec-audit: owner=sldZm
    // spec-audit: x:powerpoint/2016/slidezoom:<sldZm>
    fn slide_zoom(&mut self, el: &Elem) -> Result<Zoom, ImportError> {
        let mut object = None;
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if child.is(PSLZ, "sldZmObj") {
                if first(self.r, &mut seen, "sldZmObj", &child) {
                    object = Some(self.slide_zoom_object(&child)?);
                }
            } else {
                self.r.retained(&child);
            }
        }
        Ok(Zoom::Slide(object))
    }

    // spec-audit: owner=sldZmObj
    fn slide_zoom_object(&mut self, el: &Elem) -> Result<SlideZoomObject, ImportError> {
        let mut e = el.clone();
        let out = SlideZoomObject {
            sld_id: self.r.required(&mut e, "sldId", |_: &u32| true)?,
            c_id: self.r.attr(&mut e, "cId")?,
            zm_pr: self.zoom_child(el, PSLZ)?,
        };
        self.r.finish(&e);
        Ok(out)
    }

    // spec-audit: owner=sectionZm
    // spec-audit: x:powerpoint/2016/sectionzoom:<sectionZm>
    fn section_zoom(&mut self, el: &Elem) -> Result<Zoom, ImportError> {
        let mut object = None;
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if child.is(PSEZ, "sectionZmObj") {
                if first(self.r, &mut seen, "sectionZmObj", &child) {
                    object = Some(self.section_zoom_object(&child)?);
                }
            } else {
                self.r.retained(&child);
            }
        }
        Ok(Zoom::Section(object))
    }

    // spec-audit: owner=sectionZmObj
    fn section_zoom_object(&mut self, el: &Elem) -> Result<SectionZoomObject, ImportError> {
        let mut e = el.clone();
        let out = SectionZoomObject {
            section_id: self.r.required(&mut e, "sectionId", guid_attr)?,
            zm_pr: self.zoom_child(el, PSEZ)?,
        };
        self.r.finish(&e);
        Ok(out)
    }

    // spec-audit: owner=summaryZm
    // spec-audit: x:powerpoint/2016/summaryzoom:<summaryZm>
    fn summary_zoom(&mut self, el: &Elem) -> Result<Zoom, ImportError> {
        let mut objects = Vec::new();
        let mut layout = None;
        for child in self.r.children(el)? {
            if child.is(PSUZ, "summaryZmObj") {
                if self.charge(&child) {
                    objects.push(self.summary_zoom_object(&child)?);
                }
            } else if child.is(PSUZ, "gridLayout") || child.is(PSUZ, "fixedLayout") {
                if layout.is_some() {
                    self.r.note(K::DuplicateElement, &child, None);
                } else if child.local == "gridLayout" {
                    layout = Some(SummaryLayout::Grid);
                } else {
                    layout = Some(SummaryLayout::Fixed);
                }
            } else {
                self.r.retained(&child);
            }
        }
        Ok(Zoom::Summary { objects, layout })
    }

    // spec-audit: owner=summaryZmObj
    fn summary_zoom_object(&mut self, el: &Elem) -> Result<SummaryZoomObject, ImportError> {
        let mut e = el.clone();
        let r = &mut *self.r;
        let out = SummaryZoomObject {
            section_id: r.required(&mut e, "sectionId", guid_attr)?,
            title: r.attr(&mut e, "title")?,
            descr: r.attr(&mut e, "descr")?,
            offset_factor_x: r.attr::<Percentage>(&mut e, "offsetFactorX")?,
            offset_factor_y: r.attr::<Percentage>(&mut e, "offsetFactorY")?,
            scale_factor_x: r.attr::<Percentage>(&mut e, "scaleFactorX")?,
            scale_factor_y: r.attr::<Percentage>(&mut e, "scaleFactorY")?,
            zm_pr: self.zoom_child(el, PSUZ)?,
        };
        self.r.finish(&e);
        Ok(out)
    }

    /// `zmPr`, in the namespace of the zoom that holds it, as the schemas qualify it.
    fn zoom_child(
        &mut self,
        el: &Elem,
        space: &[&str],
    ) -> Result<Option<ZoomProperties>, ImportError> {
        let mut out = None;
        let mut seen = HashSet::new();
        for child in self.r.children(el)? {
            if child.is(space, "zmPr") {
                if first(self.r, &mut seen, "zmPr", &child) {
                    out = Some(self.zoom_properties(&child)?);
                }
            } else {
                // `extLst` included.
                self.r.retained(&child);
            }
        }
        if out.is_none() {
            self.r.note(K::MissingRequired, el, None);
        }
        Ok(out)
    }

    // spec-audit: owner=zmPr
    fn zoom_properties(&mut self, el: &Elem) -> Result<ZoomProperties, ImportError> {
        let mut e = el.clone();
        let r = &mut *self.r;
        let mut out = ZoomProperties {
            id: r.required(&mut e, "id", guid_attr)?,
            return_to_parent: r.attr(&mut e, "returnToParent")?,
            transition_dur: r.attr(&mut e, "transitionDur")?,
            show_bg: r.attr(&mut e, "showBg")?,
            ..ZoomProperties::default()
        };
        if let Some(raw) = e.take("imageType") {
            out.image_type = ZoomImageType::from_xml(&raw);
            if out.image_type.is_none() {
                r.note(K::MalformedValue, &e, Some("imageType"));
            }
        }
        r.finish(&e);
        let mut seen = HashSet::new();
        for child in r.children(el)? {
            let r = &mut *self.r;
            if !child.in_namespace(P166) {
                r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "blipFill" => {
                    if first(r, &mut seen, "blipFill", &child) {
                        out.blip_fill =
                            read_blip_fill(r, self.rels, DrawingLimits::default(), &child)?;
                    }
                }
                "spPr" => {
                    if first(r, &mut seen, "spPr", &child) {
                        out.sp_pr = Some(read_zoom_shape_properties(r, self.rels, &child)?);
                    }
                }
                _ => r.retained(&child),
            }
        }
        Ok(out)
    }
}

/// `p166:spPr`, which is `a:CT_ShapeProperties`. `xfrm` stays in the tree, and is reported, as
/// a zoom's picture is placed by its frame.
fn read_zoom_shape_properties(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    el: &Elem,
) -> Result<ShapeProperties, ImportError> {
    r.finish(el);
    let mut props = ShapeProperties::default();
    let mut slots = HashSet::new();
    for child in r.children(el)? {
        let mut drawing = Ctx::new(&mut *r, rels, DrawingLimits::default());
        if !drawing.property(&child, &mut props, false, &mut slots)? {
            r.retained(&child);
        }
    }
    Ok(props)
}

/// `cNvPr/@id` where this element is `cNvPr`.
fn own_shape_id(el: &Elem) -> Option<u32> {
    if el.local == "cNvPr" {
        el.get("id").and_then(|v| v.parse().ok())
    } else {
        None
    }
}

/// The zoom a graphic frame holds, if it holds one.
fn zoom_in(tree: &XmlTree, part: &str, frame: &Elem) -> Result<Option<Elem>, ImportError> {
    for graphic in frame.children(tree, part)? {
        if !graphic.is(ns::DML, "graphic") {
            continue;
        }
        for data in graphic.children(tree, part)? {
            if !data.is(ns::DML, "graphicData") {
                continue;
            }
            for inner in data.children(tree, part)? {
                if inner.is(PSLZ, "sldZm")
                    || inner.is(PSEZ, "sectionZm")
                    || inner.is(PSUZ, "summaryZm")
                {
                    return Ok(Some(inner));
                }
            }
        }
    }
    Ok(None)
}

fn fallback_of(tree: &XmlTree, part: &str, branches: &[Elem]) -> Result<ZoomFallback, ImportError> {
    for branch in branches {
        if !branch.is(MC, "Fallback") {
            continue;
        }
        let kids = branch.children(tree, part)?;
        let Some(kid) = kids.first() else {
            return Ok(ZoomFallback::Absent);
        };
        return Ok(if kid.is(ns::PML, "pic") {
            ZoomFallback::Picture
        } else if kid.is(ns::PML, "grpSp") {
            ZoomFallback::Group
        } else {
            ZoomFallback::Other
        });
    }
    Ok(ZoomFallback::Absent)
}

/// Report each zoom naming a slide or section the deck does not hold, by part.
pub fn check_zoom_targets(
    p: &Presentation,
    marks: &[(String, PartMarks)],
    findings: &mut Findings,
) {
    let slides: HashSet<u32> = p.slides.iter().flatten().map(|s| s.id).collect();
    let sections: HashSet<String> = p
        .sections
        .iter()
        .flat_map(|l| &l.sections)
        .filter_map(|s| s.id.as_ref().map(|i| i.to_ascii_uppercase()))
        .collect();
    let section = |id: &Option<String>| {
        id.as_ref()
            .is_some_and(|i| sections.contains(&i.to_ascii_uppercase()))
    };
    let unknown = K::Review(ReviewFinding::ZoomUnknownTarget);
    for (part, m) in marks {
        for frame in &m.zooms {
            let good = match &frame.zoom {
                Zoom::Slide(Some(o)) => o.sld_id.is_some_and(|i| slides.contains(&i)),
                Zoom::Section(Some(o)) => section(&o.section_id),
                Zoom::Summary { objects, .. } => objects.iter().all(|o| section(&o.section_id)),
                Zoom::Slide(None) | Zoom::Section(None) => true,
            };
            if !good {
                findings.note(unknown, part, "zoom", None);
            }
        }
    }
}
