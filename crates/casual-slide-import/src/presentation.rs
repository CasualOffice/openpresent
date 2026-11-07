//! Read the presentation part, `presProps` and `viewProps` from an [`XmlTree`], and
//! a whole deck's worth of Tier 1A and 1B from a package.
//!
//! A relationship id is checked against the part's `.rels`, and a target is never
//! followed from here: a font or a tag part is not opened because the list names it.

use std::collections::{HashMap, HashSet};

use casual_slide_model::presentation::{
    COORDINATE_MIN, CommonSlideViewProperties, CommonViewProperties, Coordinate, CustomShow,
    CustomerDataList, Deck, EmbeddedFont, Guide, HtmlPublishProperties, Kinsoku, ModifyVerifier,
    NormalViewPortion, NormalViewProperties, OutlineSlide, OutlineViewProperties,
    POSITIVE_COORDINATE_MAX, Percentage, PhotoAlbum, Point, PositiveSize, Presentation,
    PresentationProperties, PrintProperties, Ratio, SLIDE_ID_MAX, SLIDE_ID_MIN,
    SLIDE_MASTER_ID_MIN, SLIDE_SIZE_MAX, SLIDE_SIZE_MIN, Scale, ShowProperties, ShowType, SlideId,
    SlideListChoice, SlideMasterId, SlideSize, SorterViewProperties, TextFont, ViewProperties,
    WebProperties,
};
use casual_slide_model::properties::DocumentProperties;
use casual_slide_ooxml::{
    OoxmlError, PresentationPackage, Relationship, TargetMode, XmlTree, part_names_equivalent,
    resolve_internal,
};

use crate::color::Findings as ColorFindings;
use crate::properties::{
    Elem, Findings, ImportError, ImportFindingKind, ImportLimits, Reader, import_app_properties,
    import_core_properties, import_custom_properties, ns,
};
use crate::text::{TextCounts, TextReader};

const TRANSITIONAL_RELS: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/";
const STRICT_RELS: &str = "http://purl.oclc.org/ooxml/officeDocument/relationships/";

/// The relationship types of the parts this tier reads.
pub mod rel_types {
    /// Core properties, the same in both forms.
    pub const CORE: &[&str] =
        &["http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties"];
    /// Extended properties.
    pub const APP: &[&str] = &[
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties",
        "http://purl.oclc.org/ooxml/officeDocument/relationships/extendedProperties",
    ];
    /// Custom properties.
    pub const CUSTOM: &[&str] = &[
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties",
        "http://purl.oclc.org/ooxml/officeDocument/relationships/customProperties",
    ];
    /// `presProps`.
    pub const PRES_PROPS: &[&str] = &[
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/presProps",
        "http://purl.oclc.org/ooxml/officeDocument/relationships/presProps",
    ];
    /// `viewProps`.
    pub const VIEW_PROPS: &[&str] = &[
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/viewProps",
        "http://purl.oclc.org/ooxml/officeDocument/relationships/viewProps",
    ];
}

/// A part's relationships by id, to check an `r:id` against.
#[derive(Debug)]
pub struct Rels<'a>(HashMap<&'a str, &'a Relationship>);

impl<'a> Rels<'a> {
    /// Index a part's relationships.
    pub fn new(rels: &'a [Relationship]) -> Self {
        Self(rels.iter().map(|r| (r.id.as_str(), r)).collect())
    }

    /// Whether an id exists.
    pub fn contains(&self, id: &str) -> bool {
        self.0.contains_key(id)
    }

    /// Report an id that names nothing or, when `expected` is given, something
    /// other than an internal part of that relationship type.
    fn check(&self, r: &mut Reader<'_>, el: &Elem, id: &str, expected: Option<&str>) {
        let Some(rel) = self.0.get(id) else {
            r.note(ImportFindingKind::DanglingRelationship, el, Some("r:id"));
            return;
        };
        if let Some(expected) = expected {
            let suffix = rel
                .rel_type
                .strip_prefix(TRANSITIONAL_RELS)
                .or_else(|| rel.rel_type.strip_prefix(STRICT_RELS));
            if suffix != Some(expected) || rel.mode != TargetMode::Internal {
                r.note(ImportFindingKind::WrongRelationship, el, Some("r:id"));
            }
        }
    }
}

/// `r:id`, required, checked and reported.
fn rel_id(
    r: &mut Reader<'_>,
    el: &mut Elem,
    rels: &Rels<'_>,
    expected: Option<&str>,
) -> Result<Option<String>, ImportError> {
    let value = el.take_ns(ns::REL, "id");
    let Some(value) = value else {
        r.note(ImportFindingKind::MissingRequired, el, Some("r:id"));
        return Ok(None);
    };
    rels.check(r, el, &value, expected);
    Ok(Some(value))
}

/// `p:sldId`. An id out of range is kept and reported: the entry names a real
/// slide. One that cannot be read at all leaves the entry out of the model.
// spec-audit: owner=sldId
pub fn read_slide_id(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &Rels<'_>,
) -> Result<Option<SlideId>, ImportError> {
    let mut el = el.clone();
    let id = r.required::<u32>(&mut el, "id", |_| true)?;
    if id.is_some_and(|v| !(SLIDE_ID_MIN..=SLIDE_ID_MAX).contains(&v)) {
        r.note(ImportFindingKind::OutOfRange, &el, Some("id"));
    }
    let rel_id = rel_id(r, &mut el, rels, Some("slide"))?;
    r.finish(&el);
    retained_children(r, &el)?;
    Ok(id.zip(rel_id).map(|(id, rel_id)| SlideId { id, rel_id }))
}

/// `p:sldMasterId`.
// spec-audit: owner=sldMasterId
pub fn read_master_id(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &Rels<'_>,
) -> Result<Option<SlideMasterId>, ImportError> {
    let mut el = el.clone();
    let raw = el.take("id");
    let id = match raw {
        None => None,
        Some(raw) => {
            let parsed = <u32 as casual_slide_model::properties::Lexical>::parse_lexical(&raw);
            match parsed {
                None => {
                    r.note(ImportFindingKind::MalformedValue, &el, Some("id"));
                    return Ok(None);
                }
                Some(v) => {
                    if v < SLIDE_MASTER_ID_MIN {
                        r.note(ImportFindingKind::OutOfRange, &el, Some("id"));
                    }
                    Some(v)
                }
            }
        }
    };
    let rel_id = rel_id(r, &mut el, rels, Some("slideMaster"))?;
    r.finish(&el);
    retained_children(r, &el)?;
    Ok(rel_id.map(|rel_id| SlideMasterId { id, rel_id }))
}

/// An element holding only `r:id` of interest: `notesMasterId`, `sld`, `tags` and so on.
// spec-audit: owner=notesMasterId,handoutMasterId,smartTags,sld,regular,bold,italic,boldItalic
pub fn read_rel_only(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &Rels<'_>,
    expected: Option<&str>,
) -> Result<Option<String>, ImportError> {
    let mut el = el.clone();
    let id = rel_id(r, &mut el, rels, expected)?;
    r.finish(&el);
    retained_children(r, &el)?;
    Ok(id)
}

/// Report every child of an element that has no modelled children.
fn retained_children(r: &mut Reader<'_>, el: &Elem) -> Result<(), ImportError> {
    for child in r.children(el)? {
        r.retained(&child);
    }
    Ok(())
}

fn seen_before(r: &mut Reader<'_>, seen: &mut HashSet<String>, el: &Elem) -> bool {
    if seen.insert(el.local.clone()) {
        false
    } else {
        r.note(ImportFindingKind::DuplicateElement, el, None);
        true
    }
}

/// `a:CT_PositiveSize2D`. `None` unless both sides are usable.
// spec-audit: owner=notesSz,gridSpacing
pub fn read_positive_size(
    r: &mut Reader<'_>,
    el: &Elem,
) -> Result<Option<PositiveSize>, ImportError> {
    let mut el = el.clone();
    let range = |v: &i64| (0..=POSITIVE_COORDINATE_MAX).contains(v);
    let cx = r.required::<i64>(&mut el, "cx", range)?;
    let cy = r.required::<i64>(&mut el, "cy", range)?;
    r.finish_leaf(&el)?;
    Ok(cx.zip(cy).map(|(cx, cy)| PositiveSize { cx, cy }))
}

/// `p:sldSz`. `None` unless both sides are in range.
// spec-audit: owner=sldSz
pub fn read_slide_size(r: &mut Reader<'_>, el: &Elem) -> Result<Option<SlideSize>, ImportError> {
    let mut el = el.clone();
    let range = |v: &i32| (SLIDE_SIZE_MIN..=SLIDE_SIZE_MAX).contains(v);
    let cx = r.required::<i32>(&mut el, "cx", range)?;
    let cy = r.required::<i32>(&mut el, "cy", range)?;
    let size_type = r.attr(&mut el, "type")?;
    r.finish_leaf(&el)?;
    Ok(cx.zip(cy).map(|(cx, cy)| SlideSize { cx, cy, size_type }))
}

pub(crate) fn read_text_font(
    r: &mut Reader<'_>,
    el: &Elem,
) -> Result<Option<TextFont>, ImportError> {
    let mut el = el.clone();
    let typeface = r.required::<String>(&mut el, "typeface", |_| true)?;
    let panose = r.attr_in::<String>(&mut el, "panose", |p| {
        p.len() == 20 && p.bytes().all(|b| b.is_ascii_hexdigit())
    })?;
    let pitch_family = r.attr(&mut el, "pitchFamily")?;
    let charset = r.attr(&mut el, "charset")?;
    r.finish_leaf(&el)?;
    Ok(typeface.map(|typeface| TextFont {
        typeface,
        panose,
        pitch_family,
        charset,
    }))
}

/// `p:embeddedFont`. `None` unless its `font` is usable. Only ids are read: no font
/// data is reached by this (`SEC-C05`).
// spec-audit: owner=embeddedFont
pub fn read_embedded_font(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &Rels<'_>,
) -> Result<Option<EmbeddedFont>, ImportError> {
    let mut font = None;
    let mut entry = EmbeddedFont::default();
    let mut seen = HashSet::new();
    r.finish(el);
    for child in r.children(el)? {
        if !child.in_namespace(ns::PML) {
            r.retained(&child);
            continue;
        }
        if seen_before(r, &mut seen, &child) {
            continue;
        }
        match child.local.as_str() {
            "font" => font = read_text_font(r, &child)?,
            "regular" => entry.regular = read_rel_only(r, &child, rels, Some("font"))?,
            "bold" => entry.bold = read_rel_only(r, &child, rels, Some("font"))?,
            "italic" => entry.italic = read_rel_only(r, &child, rels, Some("font"))?,
            "boldItalic" => entry.bold_italic = read_rel_only(r, &child, rels, Some("font"))?,
            _ => r.retained(&child),
        }
    }
    let Some(font) = font else {
        r.note(ImportFindingKind::MissingRequired, el, Some("font"));
        return Ok(None);
    };
    entry.font = font;
    Ok(Some(entry))
}

/// `p:custShow`. `None` unless it has a usable name and id. `budget` is the slides
/// left across the part's shows.
// spec-audit: owner=custShow
pub fn read_custom_show(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &Rels<'_>,
    budget: &mut usize,
) -> Result<Option<CustomShow>, ImportError> {
    let mut el = el.clone();
    let name = r.required::<String>(&mut el, "name", |_| true)?;
    let id = r.required::<u32>(&mut el, "id", |_| true)?;
    r.finish(&el);
    let mut slides = Vec::new();
    let mut seen = HashSet::new();
    for child in r.children(&el)? {
        if !child.is(ns::PML, "sldLst") {
            r.retained(&child);
            continue;
        }
        if seen_before(r, &mut seen, &child) {
            continue;
        }
        r.finish(&child);
        for sld in r.children(&child)? {
            if !sld.is(ns::PML, "sld") {
                r.retained(&sld);
                continue;
            }
            *budget = budget
                .checked_sub(1)
                .ok_or_else(|| r.too_many("custom show slides", r.limits.max_custom_show_slides))?;
            if let Some(rel) = read_rel_only(r, &sld, rels, Some("slide"))? {
                slides.push(rel);
            }
        }
    }
    Ok(name
        .zip(id)
        .map(|(name, id)| CustomShow { name, id, slides }))
}

// spec-audit: owner=photoAlbum
fn read_photo_album(r: &mut Reader<'_>, el: &Elem) -> Result<PhotoAlbum, ImportError> {
    let mut el = el.clone();
    let album = PhotoAlbum {
        bw: r.attr(&mut el, "bw")?,
        show_captions: r.attr(&mut el, "showCaptions")?,
        layout: r.attr(&mut el, "layout")?,
        frame: r.attr(&mut el, "frame")?,
    };
    r.finish(&el);
    retained_children(r, &el)?;
    Ok(album)
}

/// `p:kinsoku`. `None` unless both required sets are there.
// spec-audit: owner=kinsoku
pub fn read_kinsoku(r: &mut Reader<'_>, el: &Elem) -> Result<Option<Kinsoku>, ImportError> {
    let mut el = el.clone();
    let lang = r.attr(&mut el, "lang")?;
    let start = r.required::<String>(&mut el, "invalStChars", |_| true)?;
    let end = r.required::<String>(&mut el, "invalEndChars", |_| true)?;
    r.finish_leaf(&el)?;
    Ok(start.zip(end).map(|(start, end)| Kinsoku {
        lang,
        invalid_start_chars: start,
        invalid_end_chars: end,
    }))
}

// spec-audit: owner=modifyVerifier
fn read_modify_verifier(r: &mut Reader<'_>, el: &Elem) -> Result<ModifyVerifier, ImportError> {
    let mut el = el.clone();
    let v = ModifyVerifier {
        algorithm_name: r.attr(&mut el, "algorithmName")?,
        hash_value: r.attr(&mut el, "hashValue")?,
        salt_value: r.attr(&mut el, "saltValue")?,
        spin_value: r.attr(&mut el, "spinValue")?,
        crypt_provider_type: r.attr(&mut el, "cryptProviderType")?,
        crypt_algorithm_class: r.attr(&mut el, "cryptAlgorithmClass")?,
        crypt_algorithm_type: r.attr(&mut el, "cryptAlgorithmType")?,
        crypt_algorithm_sid: r.attr(&mut el, "cryptAlgorithmSid")?,
        spin_count: r.attr(&mut el, "spinCount")?,
        salt_data: r.attr(&mut el, "saltData")?,
        hash_data: r.attr(&mut el, "hashData")?,
        crypt_provider: r.attr(&mut el, "cryptProvider")?,
        alg_id_ext: r.attr(&mut el, "algIdExt")?,
        alg_id_ext_source: r.attr(&mut el, "algIdExtSource")?,
        crypt_provider_type_ext: r.attr(&mut el, "cryptProviderTypeExt")?,
        crypt_provider_type_ext_source: r.attr(&mut el, "cryptProviderTypeExtSource")?,
    };
    r.finish_leaf(&el)?;
    Ok(v)
}

// spec-audit: owner=custDataLst
fn read_customer_data(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &Rels<'_>,
) -> Result<CustomerDataList, ImportError> {
    r.finish(el);
    let mut list = CustomerDataList::default();
    for child in r.bounded_children(el, r.limits.max_customer_data, "customer data")? {
        if child.is(ns::PML, "custData") {
            if let Some(id) = read_rel_only(r, &child, rels, None)? {
                list.customer_data.push(id);
            }
        } else if child.is(ns::PML, "tags") {
            if list.tags.is_some() {
                r.note(ImportFindingKind::DuplicateElement, &child, None);
            } else {
                list.tags = read_rel_only(r, &child, rels, Some("tags"))?;
            }
        } else {
            r.retained(&child);
        }
    }
    Ok(list)
}

/// Read `p:presentation`. `rels` are the part's own relationships.
pub fn import_presentation(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    limits: &ImportLimits,
    findings: &mut Findings,
) -> Result<Presentation, ImportError> {
    let mut color = ColorFindings::default();
    import_presentation_with_colors(tree, part, rels, limits, findings, &mut color)
}

/// As [`import_presentation`], also saying what the colour readers noticed in
/// `defaultTextStyle`, under their own codes.
// spec-audit: owner=presentation
pub fn import_presentation_with_colors(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    limits: &ImportLimits,
    findings: &mut Findings,
    color: &mut ColorFindings,
) -> Result<Presentation, ImportError> {
    let mut r = Reader::new(tree, part, limits, findings);
    let r = &mut r;
    let rel_list = rels;
    let rels = Rels::new(rels);
    let mut root = r.root(ns::PML, "presentation")?;
    let mut p = Presentation {
        server_zoom: r.attr(&mut root, "serverZoom")?,
        first_slide_num: r.attr(&mut root, "firstSlideNum")?,
        show_special_pls_on_title_sld: r.attr(&mut root, "showSpecialPlsOnTitleSld")?,
        rtl: r.attr(&mut root, "rtl")?,
        remove_personal_info_on_save: r.attr(&mut root, "removePersonalInfoOnSave")?,
        compat_mode: r.attr(&mut root, "compatMode")?,
        strict_first_and_last_chars: r.attr(&mut root, "strictFirstAndLastChars")?,
        embed_true_type_fonts: r.attr(&mut root, "embedTrueTypeFonts")?,
        save_subset_fonts: r.attr(&mut root, "saveSubsetFonts")?,
        auto_compress_pictures: r.attr(&mut root, "autoCompressPictures")?,
        bookmark_id_seed: r.attr_in(&mut root, "bookmarkIdSeed", |v: &u32| {
            (1..=SLIDE_ID_MAX).contains(v)
        })?,
        conformance: r.attr(&mut root, "conformance")?,
        ..Presentation::default()
    };
    r.finish(&root);
    let mut seen = HashSet::new();
    for el in r.children(&root)? {
        if !el.in_namespace(ns::PML) {
            r.retained(&el);
            continue;
        }
        if seen_before(r, &mut seen, &el) {
            continue;
        }
        match el.local.as_str() {
            "sldMasterIdLst" => {
                r.finish(&el);
                let mut list = Vec::new();
                for c in r.bounded_children(&el, limits.max_masters, "slide masters")? {
                    if !c.is(ns::PML, "sldMasterId") {
                        r.retained(&c);
                    } else if let Some(m) = read_master_id(r, &c, &rels)? {
                        list.push(m);
                    }
                }
                check_unique(r, &el, list.iter().filter_map(|m| m.id));
                p.slide_masters = Some(list);
            }
            "notesMasterIdLst" => {
                p.notes_masters = Some(read_master_refs(
                    r,
                    &el,
                    &rels,
                    "notesMasterId",
                    "notesMaster",
                )?);
            }
            "handoutMasterIdLst" => {
                p.handout_masters = Some(read_master_refs(
                    r,
                    &el,
                    &rels,
                    "handoutMasterId",
                    "handoutMaster",
                )?);
            }
            "sldIdLst" => {
                r.finish(&el);
                let mut list = Vec::new();
                for c in r.bounded_children(&el, limits.max_slides, "slides")? {
                    if !c.is(ns::PML, "sldId") {
                        r.retained(&c);
                    } else if let Some(s) = read_slide_id(r, &c, &rels)? {
                        list.push(s);
                    }
                }
                check_unique(r, &el, list.iter().map(|s| s.id));
                p.slides = Some(list);
            }
            "sldSz" => p.slide_size = read_slide_size(r, &el)?,
            "notesSz" => p.notes_size = read_positive_size(r, &el)?,
            "smartTags" => p.smart_tags = read_rel_only(r, &el, &rels, None)?,
            "embeddedFontLst" => {
                r.finish(&el);
                let mut list = Vec::new();
                for c in r.bounded_children(&el, limits.max_embedded_fonts, "embedded fonts")? {
                    if !c.is(ns::PML, "embeddedFont") {
                        r.retained(&c);
                    } else if let Some(f) = read_embedded_font(r, &c, &rels)? {
                        list.push(f);
                    }
                }
                p.embedded_fonts = Some(list);
            }
            "custShowLst" => {
                r.finish(&el);
                let mut list = Vec::new();
                let mut budget = limits.max_custom_show_slides;
                for c in r.bounded_children(&el, limits.max_custom_shows, "custom shows")? {
                    if !c.is(ns::PML, "custShow") {
                        r.retained(&c);
                    } else if let Some(s) = read_custom_show(r, &c, &rels, &mut budget)? {
                        list.push(s);
                    }
                }
                check_unique(r, &el, list.iter().map(|s| s.id));
                p.custom_shows = Some(list);
            }
            "photoAlbum" => p.photo_album = Some(read_photo_album(r, &el)?),
            "custDataLst" => p.customer_data = Some(read_customer_data(r, &el, &rels)?),
            "kinsoku" => p.kinsoku = read_kinsoku(r, &el)?,
            "modifyVerifier" => p.modify_verifier = Some(read_modify_verifier(r, &el)?),
            "defaultTextStyle" => {
                let mut text = TextReader {
                    r: &mut *r,
                    rels: rel_list,
                    color: &mut *color,
                    counts: &mut TextCounts::default(),
                    drawing: crate::drawing::DrawingLimits::default(),
                };
                p.default_text_style = Some(text.list_style(&el)?);
            }
            "extLst" => {
                p.sections = crate::section::read_presentation_extensions(
                    r,
                    &el,
                    &crate::comment::ReviewLimits::default(),
                )?;
            }
            // Whatever else: preserved, not modelled.
            _ => r.retained(&el),
        }
    }
    Ok(p)
}

fn read_master_refs(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &Rels<'_>,
    child_name: &str,
    rel_type: &str,
) -> Result<Vec<String>, ImportError> {
    r.finish(el);
    let mut list = Vec::new();
    for c in r.bounded_children(el, r.limits.max_masters, "masters")? {
        if !c.is(ns::PML, child_name) {
            r.retained(&c);
        } else if let Some(id) = read_rel_only(r, &c, rels, Some(rel_type))? {
            if !list.is_empty() {
                // The schema allows one. Each still names a part, so each is kept.
                r.note(ImportFindingKind::DuplicateElement, &c, None);
            }
            list.push(id);
        }
    }
    Ok(list)
}

fn check_unique(r: &mut Reader<'_>, el: &Elem, ids: impl Iterator<Item = u32>) {
    let mut seen = HashSet::new();
    for id in ids {
        if !seen.insert(id) {
            r.note(ImportFindingKind::DuplicateId, el, Some("id"));
            return;
        }
    }
}

/// One choice of `p:EG_SlideListChoice`, if `el` is one.
// spec-audit: owner=showPr,htmlPubPr
fn read_slide_list_choice(
    r: &mut Reader<'_>,
    el: &Elem,
) -> Result<Option<Option<SlideListChoice>>, ImportError> {
    let mut el = el.clone();
    let choice = match el.local.as_str() {
        "sldAll" => Some(SlideListChoice::All),
        "sldRg" => {
            let start = r.required::<u32>(&mut el, "st", |_| true)?;
            let end = r.required::<u32>(&mut el, "end", |_| true)?;
            let range = start.zip(end);
            if range.is_some_and(|(s, e)| s > e) {
                r.note(ImportFindingKind::OutOfRange, &el, Some("end"));
            }
            range.map(|(start, end)| SlideListChoice::Range { start, end })
        }
        "custShow" => r
            .required::<u32>(&mut el, "id", |_| true)?
            .map(SlideListChoice::CustomShow),
        _ => return Ok(None),
    };
    r.finish_leaf(&el)?;
    Ok(Some(choice))
}

// spec-audit: owner=htmlPubPr
fn read_html_publish(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &Rels<'_>,
) -> Result<HtmlPublishProperties, ImportError> {
    let mut el = el.clone();
    let mut h = HtmlPublishProperties {
        show_speaker_notes: r.attr(&mut el, "showSpeakerNotes")?,
        target: r.attr(&mut el, "target")?,
        title: r.attr(&mut el, "title")?,
        rel_id: rel_id(r, &mut el, rels, Some("htmlPubSaveAs"))?,
        slide_list: None,
    };
    r.finish(&el);
    let mut chosen = false;
    for child in r.children(&el)? {
        if !child.in_namespace(ns::PML) {
            r.retained(&child);
            continue;
        }
        match read_slide_list_choice(r, &child)? {
            Some(_) if chosen => r.note(ImportFindingKind::DuplicateElement, &child, None),
            Some(choice) => {
                chosen = true;
                h.slide_list = choice;
            }
            None => r.retained(&child),
        }
    }
    Ok(h)
}

// spec-audit: owner=webPr
fn read_web(r: &mut Reader<'_>, el: &Elem) -> Result<WebProperties, ImportError> {
    let mut el = el.clone();
    let w = WebProperties {
        show_animation: r.attr(&mut el, "showAnimation")?,
        resize_graphics: r.attr(&mut el, "resizeGraphics")?,
        allow_png: r.attr(&mut el, "allowPng")?,
        rely_on_vml: r.attr(&mut el, "relyOnVml")?,
        organize_in_folders: r.attr(&mut el, "organizeInFolders")?,
        use_long_filenames: r.attr(&mut el, "useLongFilenames")?,
        image_size: r.attr(&mut el, "imgSz")?,
        encoding: r.attr(&mut el, "encoding")?,
        color: r.attr(&mut el, "clr")?,
    };
    r.finish(&el);
    retained_children(r, &el)?;
    Ok(w)
}

// spec-audit: owner=prnPr
fn read_print(r: &mut Reader<'_>, el: &Elem) -> Result<PrintProperties, ImportError> {
    let mut el = el.clone();
    let p = PrintProperties {
        print_what: r.attr(&mut el, "prnWhat")?,
        color_mode: r.attr(&mut el, "clrMode")?,
        hidden_slides: r.attr(&mut el, "hiddenSlides")?,
        scale_to_fit_paper: r.attr(&mut el, "scaleToFitPaper")?,
        frame_slides: r.attr(&mut el, "frameSlides")?,
    };
    r.finish(&el);
    retained_children(r, &el)?;
    Ok(p)
}

// spec-audit: owner=showPr
fn read_show(r: &mut Reader<'_>, el: &Elem) -> Result<ShowProperties, ImportError> {
    let mut el = el.clone();
    let mut s = ShowProperties {
        loop_show: r.attr(&mut el, "loop")?,
        show_narration: r.attr(&mut el, "showNarration")?,
        show_animation: r.attr(&mut el, "showAnimation")?,
        use_timings: r.attr(&mut el, "useTimings")?,
        show_type: None,
        slide_list: None,
    };
    r.finish(&el);
    let (mut typed, mut listed) = (false, false);
    for mut child in r.children(&el)? {
        if !child.in_namespace(ns::PML) {
            r.retained(&child);
            continue;
        }
        let show_type = match child.local.as_str() {
            "present" => Some(ShowType::Present),
            "browse" => Some(ShowType::Browse {
                show_scrollbar: r.attr(&mut child, "showScrollbar")?,
            }),
            "kiosk" => Some(ShowType::Kiosk {
                restart: r.attr(&mut child, "restart")?,
            }),
            _ => None,
        };
        if let Some(show_type) = show_type {
            if typed {
                r.note(ImportFindingKind::DuplicateElement, &child, None);
            } else {
                typed = true;
                s.show_type = Some(show_type);
            }
            r.finish_leaf(&child)?;
            continue;
        }
        match read_slide_list_choice(r, &child)? {
            Some(_) if listed => r.note(ImportFindingKind::DuplicateElement, &child, None),
            Some(choice) => {
                listed = true;
                s.slide_list = choice;
            }
            // `penClr` and `extLst`: preserved, not modelled.
            None => r.retained(&child),
        }
    }
    Ok(s)
}

/// Read `p:presentationPr`. `rels` are the part's own relationships.
// spec-audit: owner=presentationPr
pub fn import_presentation_properties(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    limits: &ImportLimits,
    findings: &mut Findings,
) -> Result<PresentationProperties, ImportError> {
    let mut r = Reader::new(tree, part, limits, findings);
    let r = &mut r;
    let rels = Rels::new(rels);
    let root = r.root(ns::PML, "presentationPr")?;
    r.finish(&root);
    let mut p = PresentationProperties::default();
    let mut seen = HashSet::new();
    for el in r.children(&root)? {
        if !el.in_namespace(ns::PML) {
            r.retained(&el);
            continue;
        }
        if seen_before(r, &mut seen, &el) {
            continue;
        }
        match el.local.as_str() {
            "htmlPubPr" => p.html_publish = Some(read_html_publish(r, &el, &rels)?),
            "webPr" => p.web = Some(read_web(r, &el)?),
            "prnPr" => p.print = Some(read_print(r, &el)?),
            "showPr" => p.show = Some(read_show(r, &el)?),
            // `clrMru` and `extLst`: preserved, not modelled.
            _ => r.retained(&el),
        }
    }
    Ok(p)
}

// spec-audit: owner=sx,sy
fn read_ratio(r: &mut Reader<'_>, el: &Elem) -> Result<Option<Ratio>, ImportError> {
    let mut el = el.clone();
    let n = r.required::<i64>(&mut el, "n", |_| true)?;
    let d = r.required::<i64>(&mut el, "d", |_| true)?;
    r.finish_leaf(&el)?;
    Ok(n.zip(d).map(|(n, d)| Ratio { n, d }))
}

// spec-audit: owner=scale
fn read_scale(r: &mut Reader<'_>, el: &Elem) -> Result<Option<Scale>, ImportError> {
    r.finish(el);
    let (mut x, mut y) = (None, None);
    let mut seen = HashSet::new();
    for child in r.children(el)? {
        if !child.in_namespace(ns::DML) {
            r.retained(&child);
            continue;
        }
        if seen_before(r, &mut seen, &child) {
            continue;
        }
        match child.local.as_str() {
            "sx" => x = read_ratio(r, &child)?,
            "sy" => y = read_ratio(r, &child)?,
            _ => r.retained(&child),
        }
    }
    let scale = x.zip(y).map(|(x, y)| Scale { x, y });
    if scale.is_none() {
        r.note(ImportFindingKind::MissingRequired, el, None);
    }
    Ok(scale)
}

// spec-audit: owner=origin
fn read_point(r: &mut Reader<'_>, el: &Elem) -> Result<Option<Point>, ImportError> {
    let mut el = el.clone();
    let range = |c: &Coordinate| (COORDINATE_MIN..=POSITIVE_COORDINATE_MAX).contains(&c.0);
    let x = r.required::<Coordinate>(&mut el, "x", range)?;
    let y = r.required::<Coordinate>(&mut el, "y", range)?;
    r.finish_leaf(&el)?;
    Ok(x.zip(y).map(|(x, y)| Point { x, y }))
}

/// `p:cViewPr`.
// spec-audit: owner=cViewPr
pub fn read_common_view(
    r: &mut Reader<'_>,
    el: &Elem,
) -> Result<CommonViewProperties, ImportError> {
    let mut el = el.clone();
    let mut v = CommonViewProperties {
        var_scale: r.attr(&mut el, "varScale")?,
        scale: None,
        origin: None,
    };
    r.finish(&el);
    let mut seen = HashSet::new();
    for child in r.children(&el)? {
        if !child.in_namespace(ns::PML) {
            r.retained(&child);
            continue;
        }
        if seen_before(r, &mut seen, &child) {
            continue;
        }
        match child.local.as_str() {
            "scale" => v.scale = read_scale(r, &child)?,
            "origin" => v.origin = read_point(r, &child)?,
            _ => r.retained(&child),
        }
    }
    Ok(v)
}

/// `p:guide`.
// spec-audit: owner=guide
pub fn read_guide(r: &mut Reader<'_>, el: &Elem) -> Result<Guide, ImportError> {
    let mut el = el.clone();
    let range = |c: &Coordinate| i32::try_from(c.0).is_ok();
    let g = Guide {
        orient: r.attr(&mut el, "orient")?,
        pos: r.attr_in(&mut el, "pos", range)?,
    };
    r.finish_leaf(&el)?;
    Ok(g)
}

// spec-audit: owner=cSldViewPr
fn read_common_slide_view(
    r: &mut Reader<'_>,
    el: &Elem,
) -> Result<CommonSlideViewProperties, ImportError> {
    let mut el = el.clone();
    let mut v = CommonSlideViewProperties {
        snap_to_grid: r.attr(&mut el, "snapToGrid")?,
        snap_to_objects: r.attr(&mut el, "snapToObjects")?,
        show_guides: r.attr(&mut el, "showGuides")?,
        ..CommonSlideViewProperties::default()
    };
    r.finish(&el);
    let mut seen = HashSet::new();
    for child in r.children(&el)? {
        if !child.in_namespace(ns::PML) {
            r.retained(&child);
            continue;
        }
        if seen_before(r, &mut seen, &child) {
            continue;
        }
        match child.local.as_str() {
            "cViewPr" => v.common = Some(read_common_view(r, &child)?),
            "guideLst" => {
                r.finish(&child);
                let mut guides = Vec::new();
                for g in r.bounded_children(&child, r.limits.max_guides, "guides")? {
                    if g.is(ns::PML, "guide") {
                        guides.push(read_guide(r, &g)?);
                    } else {
                        r.retained(&g);
                    }
                }
                v.guides = Some(guides);
            }
            _ => r.retained(&child),
        }
    }
    Ok(v)
}

/// A holder whose one modelled child is `name`: `slideViewPr/cSldViewPr` and the like.
fn holder_child(r: &mut Reader<'_>, el: &Elem, name: &str) -> Result<Option<Elem>, ImportError> {
    r.finish(el);
    let mut found = None;
    for child in r.children(el)? {
        if !child.is(ns::PML, name) {
            r.retained(&child);
        } else if found.is_some() {
            r.note(ImportFindingKind::DuplicateElement, &child, None);
        } else {
            found = Some(child);
        }
    }
    Ok(found)
}

// spec-audit: owner=restoredLeft,restoredTop
fn read_normal_portion(
    r: &mut Reader<'_>,
    el: &Elem,
) -> Result<Option<NormalViewPortion>, ImportError> {
    let mut el = el.clone();
    let size = r.required::<Percentage>(&mut el, "sz", |p| (0..=100_000).contains(&p.0))?;
    let auto_adjust = r.attr(&mut el, "autoAdjust")?;
    r.finish_leaf(&el)?;
    Ok(size.map(|size| NormalViewPortion { size, auto_adjust }))
}

// spec-audit: owner=normalViewPr
fn read_normal_view(r: &mut Reader<'_>, el: &Elem) -> Result<NormalViewProperties, ImportError> {
    let mut el = el.clone();
    let mut v = NormalViewProperties {
        show_outline_icons: r.attr(&mut el, "showOutlineIcons")?,
        snap_vert_splitter: r.attr(&mut el, "snapVertSplitter")?,
        vert_bar_state: r.attr(&mut el, "vertBarState")?,
        horz_bar_state: r.attr(&mut el, "horzBarState")?,
        prefer_single_view: r.attr(&mut el, "preferSingleView")?,
        restored_left: None,
        restored_top: None,
    };
    r.finish(&el);
    let mut seen = HashSet::new();
    for child in r.children(&el)? {
        if !child.in_namespace(ns::PML) {
            r.retained(&child);
            continue;
        }
        if seen_before(r, &mut seen, &child) {
            continue;
        }
        match child.local.as_str() {
            "restoredLeft" => v.restored_left = read_normal_portion(r, &child)?,
            "restoredTop" => v.restored_top = read_normal_portion(r, &child)?,
            _ => r.retained(&child),
        }
    }
    Ok(v)
}

/// `p:sld` in the outline view.
// spec-audit: owner=sld
pub fn read_outline_slide(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &Rels<'_>,
) -> Result<Option<OutlineSlide>, ImportError> {
    let mut el = el.clone();
    let rel = rel_id(r, &mut el, rels, Some("slide"))?;
    let collapse = r.attr(&mut el, "collapse")?;
    r.finish_leaf(&el)?;
    Ok(rel.map(|rel_id| OutlineSlide { rel_id, collapse }))
}

// spec-audit: owner=outlineViewPr
fn read_outline_view(
    r: &mut Reader<'_>,
    el: &Elem,
    rels: &Rels<'_>,
) -> Result<OutlineViewProperties, ImportError> {
    r.finish(el);
    let mut v = OutlineViewProperties::default();
    let mut seen = HashSet::new();
    for child in r.children(el)? {
        if !child.in_namespace(ns::PML) {
            r.retained(&child);
            continue;
        }
        if seen_before(r, &mut seen, &child) {
            continue;
        }
        match child.local.as_str() {
            "cViewPr" => v.common = Some(read_common_view(r, &child)?),
            "sldLst" => {
                r.finish(&child);
                let mut slides = Vec::new();
                for s in r.bounded_children(&child, r.limits.max_slides, "outline slides")? {
                    if !s.is(ns::PML, "sld") {
                        r.retained(&s);
                    } else if let Some(entry) = read_outline_slide(r, &s, rels)? {
                        slides.push(entry);
                    }
                }
                v.slides = Some(slides);
            }
            _ => r.retained(&child),
        }
    }
    Ok(v)
}

// spec-audit: owner=sorterViewPr
fn read_sorter_view(r: &mut Reader<'_>, el: &Elem) -> Result<SorterViewProperties, ImportError> {
    let mut el = el.clone();
    let show_formatting = r.attr(&mut el, "showFormatting")?;
    let common = holder_child(r, &el, "cViewPr")?;
    let common = common.map(|c| read_common_view(r, &c)).transpose()?;
    Ok(SorterViewProperties {
        show_formatting,
        common,
    })
}

/// Read `p:viewPr`. `rels` are the part's own relationships.
// spec-audit: owner=viewPr
pub fn import_view_properties(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    limits: &ImportLimits,
    findings: &mut Findings,
) -> Result<ViewProperties, ImportError> {
    let mut r = Reader::new(tree, part, limits, findings);
    let r = &mut r;
    let rels = Rels::new(rels);
    let mut root = r.root(ns::PML, "viewPr")?;
    let mut v = ViewProperties {
        last_view: r.attr(&mut root, "lastView")?,
        show_comments: r.attr(&mut root, "showComments")?,
        ..ViewProperties::default()
    };
    r.finish(&root);
    let mut seen = HashSet::new();
    for el in r.children(&root)? {
        if !el.in_namespace(ns::PML) {
            r.retained(&el);
            continue;
        }
        if seen_before(r, &mut seen, &el) {
            continue;
        }
        match el.local.as_str() {
            "normalViewPr" => v.normal = Some(read_normal_view(r, &el)?),
            "slideViewPr" => {
                let inner = holder_child(r, &el, "cSldViewPr")?;
                v.slide = inner.map(|c| read_common_slide_view(r, &c)).transpose()?;
            }
            "outlineViewPr" => v.outline = Some(read_outline_view(r, &el, &rels)?),
            "notesTextViewPr" => {
                let inner = holder_child(r, &el, "cViewPr")?;
                v.notes_text = inner.map(|c| read_common_view(r, &c)).transpose()?;
            }
            "sorterViewPr" => v.sorter = Some(read_sorter_view(r, &el)?),
            "notesViewPr" => {
                let inner = holder_child(r, &el, "cSldViewPr")?;
                v.notes = inner.map(|c| read_common_slide_view(r, &c)).transpose()?;
            }
            "gridSpacing" => v.grid_spacing = read_positive_size(r, &el)?,
            // `extLst`: preserved, not modelled.
            _ => r.retained(&el),
        }
    }
    Ok(v)
}

/// A deck as read, and what was noticed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedDeck {
    /// The model.
    pub deck: Deck,
    /// The report.
    pub findings: Findings,
    /// What the colour readers noticed, in `defaultTextStyle`.
    pub color_findings: ColorFindings,
}

/// Where a role's part is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartLocation {
    /// No relationship of this role.
    Absent,
    /// This part, spelled as the archive spells it.
    At(String),
    /// A relationship exists, and names nothing usable.
    Unusable,
}

/// Find the part a role's relationship points at, reporting what is wrong with it.
/// Nothing external is followed.
pub fn locate_part(
    package: &PresentationPackage,
    source: &str,
    rel_types: &[&str],
    findings: &mut Findings,
) -> PartLocation {
    let mut found = package
        .relationships_of(source)
        .iter()
        .filter(|r| rel_types.contains(&r.rel_type.as_str()));
    let Some(rel) = found.next() else {
        return PartLocation::Absent;
    };
    let name = rel_types[0].rsplit('/').next().unwrap_or_default();
    if found.next().is_some() {
        findings.note_role(ImportFindingKind::DuplicateRelationship, source, name);
    }
    let unusable = |findings: &mut Findings| {
        findings.note_role(ImportFindingKind::UnusableRelationship, source, name);
        PartLocation::Unusable
    };
    if rel.mode != TargetMode::Internal {
        return unusable(findings);
    }
    let Some(target) = resolve_internal(source, &rel.target) else {
        return unusable(findings);
    };
    match package
        .part_names()
        .into_iter()
        .find(|n| part_names_equivalent(n, &target))
    {
        Some(actual) => PartLocation::At(actual),
        None => unusable(findings),
    }
}

/// Whether a failure belongs to one optional part, rather than to the whole deck.
/// Refusals of a hostile shape, or of a limit, are never this.
pub fn is_optional_part_fault(err: &ImportError) -> bool {
    matches!(
        err,
        ImportError::WrongRoot { .. } | ImportError::Ooxml(OoxmlError::MalformedXml { .. })
    )
}

fn read_optional<T>(
    package: &mut PresentationPackage,
    source: &str,
    rel_types: &[&str],
    findings: &mut Findings,
    read: impl FnOnce(&XmlTree, &str, &[Relationship], &mut Findings) -> Result<T, ImportError>,
) -> Result<Option<T>, ImportError> {
    let PartLocation::At(part) = locate_part(package, source, rel_types, findings) else {
        return Ok(None);
    };
    let bytes = package.read_xml_part(&part)?;
    let limits = *package.limits();
    let rels = package.relationships_of(&part).to_vec();
    let result = XmlTree::parse(&bytes, &part, &limits)
        .map_err(ImportError::from)
        .and_then(|tree| read(&tree, &part, &rels, findings));
    match result {
        Ok(value) => Ok(Some(value)),
        Err(err) if is_optional_part_fault(&err) => {
            findings.note(ImportFindingKind::UnreadablePart, &part, "", None);
            Ok(None)
        }
        Err(err) => Err(err),
    }
}

fn cross_check(deck: &Deck, part: &str, findings: &mut Findings) {
    let p = &deck.presentation;
    let deck_slides: HashSet<&str> = p
        .slides
        .iter()
        .flatten()
        .map(|s| s.rel_id.as_str())
        .collect();
    let in_deck = p
        .custom_shows
        .iter()
        .flatten()
        .all(|show| show.slides.iter().all(|s| deck_slides.contains(s.as_str())));
    if !in_deck {
        findings.note(ImportFindingKind::UnresolvedReference, part, "sld", None);
    }
    let shows: HashSet<u32> = p.custom_shows.iter().flatten().map(|s| s.id).collect();
    let props = deck.presentation_properties.as_ref();
    let choices = [
        props.and_then(|x| x.show.and_then(|s| s.slide_list)),
        props.and_then(|x| x.html_publish.as_ref().and_then(|h| h.slide_list)),
    ];
    let count = deck_slides.len();
    for choice in choices.into_iter().flatten() {
        match choice {
            SlideListChoice::CustomShow(id) if !shows.contains(&id) => {
                findings.note(
                    ImportFindingKind::UnresolvedReference,
                    part,
                    "custShow",
                    None,
                );
            }
            SlideListChoice::Range { end, .. }
                if usize::try_from(end).map_or(true, |e| e > count) =>
            {
                findings.note(ImportFindingKind::OutOfRange, part, "sldRg", Some("end"));
            }
            _ => {}
        }
    }
}

/// Read everything Tier 1A and 1B model from an admitted package.
///
/// The main part must be readable. Any other part, if malformed or of the wrong
/// kind, is reported and left out, and is written back as it was.
pub fn import_deck(
    package: &mut PresentationPackage,
    limits: &ImportLimits,
) -> Result<ImportedDeck, ImportError> {
    let mut findings = Findings::new(limits.max_findings);
    let main = package.main_part().to_owned();
    let bytes = package.read_xml_part(&main)?;
    let tree = XmlTree::parse(&bytes, &main, package.limits())?;
    let mut color_findings = ColorFindings::default();
    let presentation = import_presentation_with_colors(
        &tree,
        &main,
        package.relationships_of(&main),
        limits,
        &mut findings,
        &mut color_findings,
    )?;
    let presentation_properties = read_optional(
        package,
        &main,
        rel_types::PRES_PROPS,
        &mut findings,
        |tree, part, rels, f| import_presentation_properties(tree, part, rels, limits, f),
    )?;
    let view_properties = read_optional(
        package,
        &main,
        rel_types::VIEW_PROPS,
        &mut findings,
        |tree, part, rels, f| import_view_properties(tree, part, rels, limits, f),
    )?;
    let core = read_optional(
        package,
        "",
        rel_types::CORE,
        &mut findings,
        |t, part, _, f| import_core_properties(t, part, limits, f),
    )?;
    let app = read_optional(
        package,
        "",
        rel_types::APP,
        &mut findings,
        |t, part, _, f| import_app_properties(t, part, limits, f),
    )?;
    let custom = read_optional(
        package,
        "",
        rel_types::CUSTOM,
        &mut findings,
        |t, part, _, f| import_custom_properties(t, part, limits, f),
    )?;
    let deck = Deck {
        presentation,
        presentation_properties,
        view_properties,
        properties: DocumentProperties { core, app, custom },
    };
    cross_check(&deck, &main, &mut findings);
    crate::section::check_sections(&deck.presentation, &main, &mut findings);
    Ok(ImportedDeck {
        deck,
        findings,
        color_findings,
    })
}
