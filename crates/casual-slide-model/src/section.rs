//! Sections, zoom objects and classification marks as plain typed values
//! (`docs/spec/EXT.md`): `p14:sectionLst`, `pslz:sldZm`, `psez:sectionZm`, `psuz:summaryZm`,
//! `p184:classification`, `acs:classification`, and the sensitivity labels of `ENT-04`.
//!
//! Each field is what the file said, or `None`. Nothing here says how a zoom shows or
//! plays: that is Phase 8. A zoom's fallback stays in the tree, and is reported here only by
//! what it is, as it is what other readers draw (`INV-FB-05`).
//!
//! Evidence: `MS-PPTX` schemas in `schemas/ms-extensions`; `INV-SL-*` in `docs/36`. That each
//! slide is in exactly one section is Inferred (`INV-SL-05`). Sensitivity labels are Inferred
//! from Microsoft's public description of them, with no deck here holding one.

use crate::color::xml_enum;
use crate::drawing::ShapeProperties;
use crate::fill::BlipFill;
use crate::presentation::Percentage;

/// `p14:CT_SectionList`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SectionList {
    /// `section`.
    pub sections: Vec<Section>,
}

/// `p14:CT_Section`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Section {
    /// `@name`. Document data.
    pub name: Option<String>,
    /// `@id`.
    pub id: Option<String>,
    /// `sldIdLst`. `None` if the section has none, which the schema does not allow and a
    /// file can do.
    pub sld_id_lst: Option<Vec<u32>>,
}

/// How a section list disagrees with `sldIdLst`. A list out of step opens, and is kept as it
/// is, because what PowerPoint does with one is Unverified (`INV-SL-05`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SectionSync {
    /// Slide ids a section names and `sldIdLst` does not.
    pub unknown: Vec<u32>,
    /// Slides in `sldIdLst` no section names.
    pub unlisted: Vec<u32>,
    /// Slide ids named by more than one entry.
    pub repeated: Vec<u32>,
    /// Whether reading the sections in order gives a different order from `sldIdLst`.
    pub out_of_order: bool,
}

impl SectionSync {
    /// Whether the list and the slides agree.
    pub fn in_sync(&self) -> bool {
        self.unknown.is_empty()
            && self.unlisted.is_empty()
            && self.repeated.is_empty()
            && !self.out_of_order
    }
}

impl SectionList {
    /// Hold the list against `sldIdLst`'s ids, in order. Linear, as a hostile list can be long.
    pub fn check_against(&self, slides: &[u32]) -> SectionSync {
        use std::collections::{HashMap, HashSet};
        let known: HashSet<u32> = slides.iter().copied().collect();
        let mut times: HashMap<u32, usize> = HashMap::new();
        let mut sync = SectionSync::default();
        let mut order = Vec::new();
        for id in self.sections.iter().flat_map(|s| s.sld_id_lst.iter().flatten()) {
            let seen = times.entry(*id).or_insert(0);
            *seen = seen.saturating_add(1);
            if *seen == 2 {
                sync.repeated.push(*id);
            }
            if !known.contains(id) {
                if *seen == 1 {
                    sync.unknown.push(*id);
                }
            } else if *seen == 1 {
                order.push(*id);
            }
        }
        sync.unlisted = slides
            .iter()
            .copied()
            .filter(|s| !times.contains_key(s))
            .collect();
        let listed: Vec<u32> = slides
            .iter()
            .copied()
            .filter(|s| times.contains_key(s))
            .collect();
        sync.out_of_order = listed != order;
        sync
    }

    /// Take a slide out of every section, as deleting it does (`INV-SL-04`).
    pub fn remove_slide(&mut self, slide: u32) {
        for section in &mut self.sections {
            if let Some(list) = &mut section.sld_id_lst {
                list.retain(|id| *id != slide);
            }
        }
    }
}

xml_enum! {
    /// `p166:ST_ZoomObjectImageType`.
    ZoomImageType {
        Preview = "preview",
        Cover = "cover",
    }
}

/// `p166:CT_ZoomObjectProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ZoomProperties {
    /// `blipFill`.
    pub blip_fill: Option<BlipFill>,
    /// `spPr`.
    pub sp_pr: Option<ShapeProperties>,
    /// `@id`.
    pub id: Option<String>,
    /// `@returnToParent`.
    pub return_to_parent: Option<bool>,
    /// `@imageType`.
    pub image_type: Option<ZoomImageType>,
    /// `@transitionDur`, as written: `ST_UniversalTimeOffset`.
    pub transition_dur: Option<String>,
    /// `@showBg`.
    pub show_bg: Option<bool>,
}

/// `pslz:CT_SlideZoomObject`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SlideZoomObject {
    /// `zmPr`.
    pub zm_pr: Option<ZoomProperties>,
    /// `@sldId`.
    pub sld_id: Option<u32>,
    /// `@cId`.
    pub c_id: Option<u32>,
}

/// `psez:CT_SectionZoomObject`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SectionZoomObject {
    /// `zmPr`.
    pub zm_pr: Option<ZoomProperties>,
    /// `@sectionId`.
    pub section_id: Option<String>,
}

/// `psuz:CT_SummaryZoomObject`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SummaryZoomObject {
    /// `zmPr`.
    pub zm_pr: Option<ZoomProperties>,
    /// `@sectionId`.
    pub section_id: Option<String>,
    /// `@title`. Document data.
    pub title: Option<String>,
    /// `@descr`. Document data.
    pub descr: Option<String>,
    /// `@offsetFactorX`.
    pub offset_factor_x: Option<Percentage>,
    /// `@offsetFactorY`.
    pub offset_factor_y: Option<Percentage>,
    /// `@scaleFactorX`.
    pub scale_factor_x: Option<Percentage>,
    /// `@scaleFactorY`.
    pub scale_factor_y: Option<Percentage>,
}

/// `psuz:CT_SummaryZoom`'s layout, a choice of two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryLayout {
    /// `gridLayout`, `psuz:CT_GridLayout`.
    Grid,
    /// `fixedLayout`, `psuz:CT_FixedLayout`.
    Fixed,
}

/// What a zoom frame holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Zoom {
    /// `sldZm`, whose `sldZmObj` is `pslz:CT_SlideZoom`. Required by the schema.
    Slide(Option<SlideZoomObject>),
    /// `sectionZm`, whose `sectionZmObj` is `psez:CT_SectionZoom`. Required by the schema.
    Section(Option<SectionZoomObject>),
    /// `summaryZm`, with `summaryZmObj` and `gridLayout` or `fixedLayout`, `psuz:CT_SummaryZoom`.
    Summary {
        /// `summaryZmObj`.
        objects: Vec<SummaryZoomObject>,
        /// `gridLayout` or `fixedLayout`.
        layout: Option<SummaryLayout>,
    },
}

/// What a zoom falls back to in `mc:Fallback`, for a reader that has no zoom. It stays in
/// the tree, and every reader but PowerPoint draws it (`INV-FB-05`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomFallback {
    /// None: the file has no `mc:Fallback`, or it is empty.
    Absent,
    /// A `p:pic`, as `sldZm` and `sectionZm` have.
    Picture,
    /// A `p:grpSp`, as `summaryZm` has.
    Group,
    /// Something else.
    Other,
}

/// One zoom on a slide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoomFrame {
    /// The frame's `cNvPr/@id`.
    pub shape_id: Option<u32>,
    /// What it holds.
    pub zoom: Zoom,
    /// Its fallback.
    pub fallback: ZoomFallback,
}

xml_enum! {
    /// `p184:ST_ClassificationOutcomeType`, and the same names in
    /// `acs:ST_ClassificationOutcomeType`: where a label's visible mark stands.
    ClassificationOutcome {
        None = "none",
        Header = "hdr",
        Footer = "ftr",
        Watermark = "watermark",
    }
}

/// A shape marked as a label's visible mark, `p184:CT_ClassificationOutcome` or
/// `acs:CT_ClassificationOutcome`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassificationMark {
    /// The `cNvPr/@id` of the shape that holds it.
    pub shape_id: Option<u32>,
    /// `@val`, or `@classificationOutcomeType`.
    pub outcome: Option<ClassificationOutcome>,
}

/// What Designer says about a shape: `p16:CT_DesignElement` and `p202:CT_DesignerDrawingProps`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesignerMark {
    /// `designElem`, with `@val`.
    DesignElement {
        /// The `cNvPr/@id` of the shape that holds it.
        shape_id: Option<u32>,
        /// `@val`.
        val: Option<bool>,
    },
    /// `designPr`.
    DrawingProperties {
        /// The `cNvPr/@id` of the shape that holds it.
        shape_id: Option<u32>,
        /// `@edtDesignElem`.
        edt_design_elem: Option<bool>,
        /// `designTagLst`, as `p202:CT_DesignerTagList`.
        design_tag_lst: Option<Vec<DesignerTag>>,
    },
}

/// `p202:CT_DesignerTag`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesignerTag {
    /// `@name`. Document data.
    pub name: Option<String>,
    /// `@val`. Document data.
    pub val: Option<String>,
}

/// A sensitivity label on a deck (`ENT-04`). Read and kept; nothing here enforces one, as
/// that is host policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SensitivityLabel {
    /// Where it was read from.
    pub source: LabelSource,
    /// The label's id.
    pub id: Option<String>,
    /// The tenant's id.
    pub site_id: Option<String>,
    /// Whether it is on.
    pub enabled: Option<bool>,
    /// How it was set.
    pub method: Option<String>,
    /// Its name, where the file gives one. Document data.
    pub name: Option<String>,
    /// Which marks it applies, as written. `contentBits` is Inferred to be a bit set.
    pub content_bits: Option<u32>,
    /// Whether the label was removed.
    pub removed: Option<bool>,
}

/// Where a sensitivity label was read from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LabelSource {
    /// `docMetadata/LabelInfo.xml`.
    #[default]
    LabelPart,
    /// `MSIP_Label_*` custom properties, which older writers leave.
    CustomProperties,
}

/// A deck's labels, as read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeckLabels {
    /// Every label found.
    pub labels: Vec<SensitivityLabel>,
    /// The label part, if it could not be read. Set aside and named.
    pub set_aside: Option<String>,
}

/// What one slide-like part holds of this tier, beyond its shape tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartMarks {
    /// `p188:commentRel`'s `r:id`, under `p188:CT_CommentRelationship`.
    pub comment_rel: Option<String>,
    /// Its zoom objects, in document order.
    pub zooms: Vec<ZoomFrame>,
    /// Its classification marks, in document order.
    pub classifications: Vec<ClassificationMark>,
    /// What Designer said, in document order.
    pub designer: Vec<DesignerMark>,
    /// `tags` and `custData` `r:id`s from every `custDataLst` in the part, in document order.
    pub customer_data: Vec<String>,
}
