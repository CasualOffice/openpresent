//! Patch the presentation part, `presProps` and `viewProps`, and save a deck through
//! `PresentationPackage::save_with` (ADR-013).
//!
//! An unedited model produces no edit and returns the input bytes. Every patched
//! part is read back and compared with what was asked before anything is saved.

use std::collections::BTreeMap;

use casual_slide_import::presentation::{
    PartLocation, Rels, import_presentation, import_presentation_properties,
    import_view_properties, is_optional_part_fault, locate_part, read_custom_show,
    read_embedded_font, read_guide, read_master_id, read_outline_slide, read_rel_only,
    read_slide_id, rel_types,
};
use casual_slide_import::properties::{
    Elem, Findings, ImportError, ImportLimits, Reader, import_app_properties,
    import_core_properties, import_custom_properties, ns,
};
use casual_slide_model::presentation::{
    CommonSlideViewProperties, CommonViewProperties, Coordinate, CustomShow, CustomerDataList,
    Deck, EmbeddedFont, Guide, HtmlPublishProperties, Kinsoku, ModifyVerifier, NormalViewPortion,
    NormalViewProperties, OutlineSlide, OutlineViewProperties, Percentage, PhotoAlbum, Point,
    PositiveSize, Presentation, PresentationProperties, PrintProperties, Ratio, SLIDE_ID_MAX,
    SLIDE_ID_MIN, SLIDE_MASTER_ID_MIN, Scale, ShowProperties, ShowType, SlideId, SlideListChoice,
    SlideMasterId, SlideSize, SorterViewProperties, TextFont, ViewProperties, WebProperties,
};
use casual_slide_model::properties::Lexical;
use casual_slide_ooxml::{
    Conformance, OoxmlLimits, PresentationPackage, Relationship, TargetMode, XmlTree,
    part_names_equivalent,
};

use crate::properties::{
    ExportError, Writer, patch_app_properties, patch_core_properties, patch_custom_properties,
    verify,
};
use crate::writer::{
    SavePlan, WriterError, WriterLimits, next_rel_id, read_relationships, target_for,
};

pub(crate) const PRESENTATION_ORDER: &[&str] = &[
    "sldMasterIdLst",
    "notesMasterIdLst",
    "handoutMasterIdLst",
    "sldIdLst",
    "sldSz",
    "notesSz",
    "smartTags",
    "embeddedFontLst",
    "custShowLst",
    "photoAlbum",
    "custDataLst",
    "kinsoku",
    "defaultTextStyle",
    "modifyVerifier",
    "extLst",
];
const PROPERTIES_ORDER: &[&str] = &["htmlPubPr", "webPr", "prnPr", "showPr", "clrMru", "extLst"];
const VIEW_ORDER: &[&str] = &[
    "normalViewPr",
    "slideViewPr",
    "outlineViewPr",
    "notesTextViewPr",
    "sorterViewPr",
    "notesViewPr",
    "gridSpacing",
    "extLst",
];
const SHOW_ORDER: &[&str] = &[
    "present", "browse", "kiosk", "sldAll", "sldRg", "custShow", "penClr", "extLst",
];
const HTML_ORDER: &[&str] = &["sldAll", "sldRg", "custShow", "extLst"];
const FONT_ORDER: &[&str] = &["font", "regular", "bold", "italic", "boldItalic"];
const NORMAL_ORDER: &[&str] = &["restoredLeft", "restoredTop", "extLst"];
const COMMON_VIEW_ORDER: &[&str] = &["scale", "origin"];
const COMMON_SLIDE_VIEW_ORDER: &[&str] = &["cViewPr", "guideLst"];
const OUTLINE_ORDER: &[&str] = &["cViewPr", "sldLst", "extLst"];
const HOLDER_ORDER: &[&str] = &["cViewPr", "cSldViewPr", "extLst"];

fn out_of_range(w: &Writer<'_>, what: &'static str) -> ExportError {
    ExportError::ValueOutOfRange {
        what,
        part: w.part.to_owned(),
    }
}

/// A percentage attribute, in the form the file already uses, or `N%` in Strict.
fn percent_attr(
    w: &mut Writer<'_>,
    el: &Elem,
    name: &str,
    before: Option<&Percentage>,
    after: Option<&Percentage>,
) -> Result<(), ExportError> {
    if before == after {
        return Ok(());
    }
    let Some(value) = after else {
        return w.attr::<Percentage>(el, name, before, None);
    };
    let percent_form = w.strict || el.get(name).is_some_and(|v| v.contains('%'));
    let text = if percent_form {
        value.to_percent_form()
    } else {
        value.to_lexical()
    };
    w.attr(el, name, Some(&String::new()), Some(&text))
}

fn scratch_reader<'a>(
    tree: &'a XmlTree,
    part: &'a str,
    limits: &'a ImportLimits,
    findings: &'a mut Findings,
) -> Reader<'a> {
    Reader::new(tree, part, limits, findings)
}

/// The list child `local` as it stands, with each entry the model can see.
fn existing_list<T>(
    w: &Writer<'_>,
    parent: &Elem,
    limits: &ImportLimits,
    child: &str,
    mut read: impl FnMut(&mut Reader<'_>, &Elem) -> Result<Option<T>, ImportError>,
) -> Result<Vec<(T, Elem)>, ExportError> {
    let mut findings = Findings::new(limits.max_findings);
    let mut r = scratch_reader(w.tree, w.part, limits, &mut findings);
    let mut out = Vec::new();
    for el in r.children(parent)? {
        if el.is(ns::PML, child)
            && let Some(value) = read(&mut r, &el)?
        {
            out.push((value, el));
        }
    }
    Ok(out)
}

// spec-audit: owner=sldId
fn patch_slide_id(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&SlideId>,
    after: &SlideId,
) -> Result<(), ExportError> {
    if before.map(|b| b.id) != Some(after.id) && !(SLIDE_ID_MIN..=SLIDE_ID_MAX).contains(&after.id)
    {
        return Err(out_of_range(w, "slide id"));
    }
    w.attr(el, "id", before.map(|b| &b.id), Some(&after.id))?;
    w.rel_attr(el, before.map(|b| b.rel_id.as_str()), Some(&after.rel_id))
}

// spec-audit: owner=sldMasterId
fn patch_master_id(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&SlideMasterId>,
    after: &SlideMasterId,
) -> Result<(), ExportError> {
    if before.and_then(|b| b.id) != after.id && after.id.is_some_and(|v| v < SLIDE_MASTER_ID_MIN) {
        return Err(out_of_range(w, "slide master id"));
    }
    w.attr(
        el,
        "id",
        before.and_then(|b| b.id.as_ref()),
        after.id.as_ref(),
    )?;
    w.rel_attr(el, before.map(|b| b.rel_id.as_str()), Some(&after.rel_id))
}

#[allow(clippy::ptr_arg)] // The shape `sync_child` and `sync_list` call, with `T = String`.
// spec-audit: owner=notesMasterId,handoutMasterId,regular,bold,italic,boldItalic,smartTags,sld
fn patch_rel_entry(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&String>,
    after: &String,
) -> Result<(), ExportError> {
    w.rel_attr(el, before.map(String::as_str), Some(after))
}

// spec-audit: owner=sldSz
fn patch_slide_size(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&SlideSize>,
    after: &SlideSize,
) -> Result<(), ExportError> {
    w.attr(el, "cx", before.map(|b| &b.cx), Some(&after.cx))?;
    w.attr(el, "cy", before.map(|b| &b.cy), Some(&after.cy))?;
    w.attr(
        el,
        "type",
        before.and_then(|b| b.size_type.as_ref()),
        after.size_type.as_ref(),
    )
}

// spec-audit: owner=notesSz,gridSpacing
fn patch_positive_size(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&PositiveSize>,
    after: &PositiveSize,
) -> Result<(), ExportError> {
    w.attr(el, "cx", before.map(|b| &b.cx), Some(&after.cx))?;
    w.attr(el, "cy", before.map(|b| &b.cy), Some(&after.cy))
}

// spec-audit: owner=latin,ea,cs,sym,buFont
pub(crate) fn patch_text_font(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&TextFont>,
    after: &TextFont,
) -> Result<(), ExportError> {
    w.attr(
        el,
        "typeface",
        before.map(|b| &b.typeface),
        Some(&after.typeface),
    )?;
    w.attr(
        el,
        "panose",
        before.and_then(|b| b.panose.as_ref()),
        after.panose.as_ref(),
    )?;
    w.attr(
        el,
        "pitchFamily",
        before.and_then(|b| b.pitch_family.as_ref()),
        after.pitch_family.as_ref(),
    )?;
    w.attr(
        el,
        "charset",
        before.and_then(|b| b.charset.as_ref()),
        after.charset.as_ref(),
    )
}

// spec-audit: owner=embeddedFont
fn patch_embedded_font(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&EmbeddedFont>,
    after: &EmbeddedFont,
) -> Result<(), ExportError> {
    w.sync_child(
        el,
        ns::PML,
        "p",
        FONT_ORDER,
        "font",
        before.map(|b| &b.font),
        Some(&after.font),
        patch_text_font,
    )?;
    let styles: [(&str, Option<&String>, Option<&String>); 4] = [
        (
            "regular",
            before.and_then(|b| b.regular.as_ref()),
            after.regular.as_ref(),
        ),
        (
            "bold",
            before.and_then(|b| b.bold.as_ref()),
            after.bold.as_ref(),
        ),
        (
            "italic",
            before.and_then(|b| b.italic.as_ref()),
            after.italic.as_ref(),
        ),
        (
            "boldItalic",
            before.and_then(|b| b.bold_italic.as_ref()),
            after.bold_italic.as_ref(),
        ),
    ];
    for (local, old, new) in styles {
        w.sync_child(
            el,
            ns::PML,
            "p",
            FONT_ORDER,
            local,
            old,
            new,
            patch_rel_entry,
        )?;
    }
    Ok(())
}

// spec-audit: owner=custShow
fn patch_custom_show(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&CustomShow>,
    after: &CustomShow,
    limits: &ImportLimits,
) -> Result<(), ExportError> {
    w.attr(el, "name", before.map(|b| &b.name), Some(&after.name))?;
    w.attr(el, "id", before.map(|b| &b.id), Some(&after.id))?;
    if before.map(|b| &b.slides) == Some(&after.slides) {
        return Ok(());
    }
    let list = match el.first_child(w.tree, w.part, ns::PML, "sldLst")? {
        Some(list) => list,
        None => {
            let list = w.new_child(el, ns::PML, "p", "sldLst")?;
            w.tree.insert_child(el.id, None, list.id);
            list
        }
    };
    let no_rels = Rels::new(&[]);
    let existing = existing_list(w, &list, limits, "sld", |r, e| {
        read_rel_only(r, e, &no_rels, None)
    })?;
    w.sync_list(
        &list,
        &existing,
        &after.slides,
        String::clone,
        |w| w.new_child(&list, ns::PML, "p", "sld"),
        patch_rel_entry,
    )
}

// spec-audit: owner=photoAlbum
fn patch_photo_album(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&PhotoAlbum>,
    after: &PhotoAlbum,
) -> Result<(), ExportError> {
    let b = before.copied().unwrap_or_default();
    w.attr(el, "bw", b.bw.as_ref(), after.bw.as_ref())?;
    w.attr(
        el,
        "showCaptions",
        b.show_captions.as_ref(),
        after.show_captions.as_ref(),
    )?;
    w.attr(el, "layout", b.layout.as_ref(), after.layout.as_ref())?;
    w.attr(el, "frame", b.frame.as_ref(), after.frame.as_ref())
}

// spec-audit: owner=custDataLst
fn patch_customer_data(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&CustomerDataList>,
    after: &CustomerDataList,
    limits: &ImportLimits,
) -> Result<(), ExportError> {
    let no_rels = Rels::new(&[]);
    let existing = existing_list(w, el, limits, "custData", |r, e| {
        read_rel_only(r, e, &no_rels, None)
    })?;
    w.sync_list(
        el,
        &existing,
        &after.customer_data,
        String::clone,
        |w| w.new_child(el, ns::PML, "p", "custData"),
        patch_rel_entry,
    )?;
    // The schema puts `tags` last, and a new entry lands after the last element.
    if let Some(tags) = el.first_child(w.tree, w.part, ns::PML, "tags")?
        && w.tree.child_elements(el.id).last() != Some(&tags.id)
    {
        w.tree.remove_child(el.id, tags.id);
        w.tree.insert_child(el.id, None, tags.id);
        w.changes += 1;
    }
    w.sync_child(
        el,
        ns::PML,
        "p",
        &["custData", "tags"],
        "tags",
        before.and_then(|b| b.tags.as_ref()),
        after.tags.as_ref(),
        patch_rel_entry,
    )
}

fn patch_kinsoku(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&Kinsoku>,
    after: &Kinsoku,
) -> Result<(), ExportError> {
    w.attr(
        el,
        "lang",
        before.and_then(|b| b.lang.as_ref()),
        after.lang.as_ref(),
    )?;
    w.attr(
        el,
        "invalStChars",
        before.map(|b| &b.invalid_start_chars),
        Some(&after.invalid_start_chars),
    )?;
    w.attr(
        el,
        "invalEndChars",
        before.map(|b| &b.invalid_end_chars),
        Some(&after.invalid_end_chars),
    )
}

fn patch_modify_verifier(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&ModifyVerifier>,
    after: &ModifyVerifier,
) -> Result<(), ExportError> {
    let b = before.cloned().unwrap_or_default();
    macro_rules! attrs {
        ($($name:literal => $field:ident),+ $(,)?) => {$(
            w.attr(el, $name, b.$field.as_ref(), after.$field.as_ref())?;
        )+};
    }
    attrs!(
        "algorithmName" => algorithm_name,
        "hashValue" => hash_value,
        "saltValue" => salt_value,
        "spinValue" => spin_value,
        "cryptProviderType" => crypt_provider_type,
        "cryptAlgorithmClass" => crypt_algorithm_class,
        "cryptAlgorithmType" => crypt_algorithm_type,
        "cryptAlgorithmSid" => crypt_algorithm_sid,
        "spinCount" => spin_count,
        "saltData" => salt_data,
        "hashData" => hash_data,
        "cryptProvider" => crypt_provider,
        "algIdExt" => alg_id_ext,
        "algIdExtSource" => alg_id_ext_source,
        "cryptProviderTypeExt" => crypt_provider_type_ext,
        "cryptProviderTypeExtSource" => crypt_provider_type_ext_source,
    );
    Ok(())
}

/// Patch a tree of `p:presentation` to say `after`. Returns the nodes changed.
// spec-audit: owner=presentation
pub fn patch_presentation(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &Presentation,
    limits: &ImportLimits,
) -> Result<usize, ExportError> {
    let mut quiet = Findings::new(limits.max_findings);
    let before = import_presentation(tree, part, rels, limits, &mut quiet)?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
        changes: 0,
    };
    let w = &mut w;
    percent_attr(
        w,
        &root,
        "serverZoom",
        before.server_zoom.as_ref(),
        after.server_zoom.as_ref(),
    )?;
    macro_rules! attrs {
        ($($name:literal => $field:ident),+ $(,)?) => {$(
            w.attr(&root, $name, before.$field.as_ref(), after.$field.as_ref())?;
        )+};
    }
    attrs!(
        "firstSlideNum" => first_slide_num,
        "showSpecialPlsOnTitleSld" => show_special_pls_on_title_sld,
        "rtl" => rtl,
        "removePersonalInfoOnSave" => remove_personal_info_on_save,
        "compatMode" => compat_mode,
        "strictFirstAndLastChars" => strict_first_and_last_chars,
        "embedTrueTypeFonts" => embed_true_type_fonts,
        "saveSubsetFonts" => save_subset_fonts,
        "autoCompressPictures" => auto_compress_pictures,
        "bookmarkIdSeed" => bookmark_id_seed,
        "conformance" => conformance,
    );
    let no_rels = Rels::new(&[]);
    macro_rules! child {
        ($local:literal, $field:ident, $patch:expr) => {
            w.sync_child(
                &root,
                ns::PML,
                "p",
                PRESENTATION_ORDER,
                $local,
                before.$field.as_ref(),
                after.$field.as_ref(),
                $patch,
            )?
        };
    }
    child!("sldMasterIdLst", slide_masters, |w,
                                             list,
                                             _,
                                             wanted: &Vec<
        SlideMasterId,
    >| {
        let existing = existing_list(w, list, limits, "sldMasterId", |r, e| {
            read_master_id(r, e, &no_rels)
        })?;
        w.sync_list(
            list,
            &existing,
            wanted,
            |m| m.rel_id.clone(),
            |w| w.new_child(list, ns::PML, "p", "sldMasterId"),
            patch_master_id,
        )
    });
    for (local, entry, before_list, after_list) in [
        (
            "notesMasterIdLst",
            "notesMasterId",
            before.notes_masters.as_ref(),
            after.notes_masters.as_ref(),
        ),
        (
            "handoutMasterIdLst",
            "handoutMasterId",
            before.handout_masters.as_ref(),
            after.handout_masters.as_ref(),
        ),
    ] {
        w.sync_child(
            &root,
            ns::PML,
            "p",
            PRESENTATION_ORDER,
            local,
            before_list,
            after_list,
            |w, list, _, wanted: &Vec<String>| {
                let existing = existing_list(w, list, limits, entry, |r, e| {
                    read_rel_only(r, e, &no_rels, None)
                })?;
                w.sync_list(
                    list,
                    &existing,
                    wanted,
                    String::clone,
                    |w| w.new_child(list, ns::PML, "p", entry),
                    patch_rel_entry,
                )
            },
        )?;
    }
    child!("sldIdLst", slides, |w, list, _, wanted: &Vec<SlideId>| {
        let existing = existing_list(w, list, limits, "sldId", |r, e| {
            read_slide_id(r, e, &no_rels)
        })?;
        w.sync_list(
            list,
            &existing,
            wanted,
            |s| s.rel_id.clone(),
            |w| w.new_child(list, ns::PML, "p", "sldId"),
            patch_slide_id,
        )
    });
    child!("sldSz", slide_size, patch_slide_size);
    child!("notesSz", notes_size, patch_positive_size);
    child!("smartTags", smart_tags, patch_rel_entry);
    child!("embeddedFontLst", embedded_fonts, |w,
                                               list,
                                               _,
                                               wanted: &Vec<
        EmbeddedFont,
    >| {
        let existing = existing_list(w, list, limits, "embeddedFont", |r, e| {
            read_embedded_font(r, e, &no_rels)
        })?;
        w.sync_list(
            list,
            &existing,
            wanted,
            |f| f.font.typeface.clone(),
            |w| w.new_child(list, ns::PML, "p", "embeddedFont"),
            patch_embedded_font,
        )
    });
    child!(
        "custShowLst",
        custom_shows,
        |w, list, _, wanted: &Vec<CustomShow>| {
            let mut budget = limits.max_custom_show_slides;
            let existing = existing_list(w, list, limits, "custShow", |r, e| {
                read_custom_show(r, e, &no_rels, &mut budget)
            })?;
            w.sync_list(
                list,
                &existing,
                wanted,
                |s| s.id.to_string(),
                |w| w.new_child(list, ns::PML, "p", "custShow"),
                |w, el, old, new| patch_custom_show(w, el, old, new, limits),
            )
        }
    );
    child!("photoAlbum", photo_album, patch_photo_album);
    child!("custDataLst", customer_data, |w, el, old, new| {
        patch_customer_data(w, el, old, new, limits)
    });
    child!("kinsoku", kinsoku, patch_kinsoku);
    child!(
        "defaultTextStyle",
        default_text_style,
        crate::text::patch_list_style
    );
    child!("modifyVerifier", modify_verifier, patch_modify_verifier);
    crate::section::patch_sections(w, &root, before.sections.as_ref(), after.sections.as_ref())?;
    Ok(w.changes)
}

/// Replace a slide list choice among `el`'s children.
// spec-audit: owner=showPr,htmlPubPr
fn patch_slide_list(
    w: &mut Writer<'_>,
    el: &Elem,
    order: &[&str],
    before: Option<&SlideListChoice>,
    after: Option<&SlideListChoice>,
) -> Result<(), ExportError> {
    if before == after {
        return Ok(());
    }
    for local in ["sldAll", "sldRg", "custShow"] {
        w.remove_all(el, ns::PML, local)?;
    }
    let Some(choice) = after else {
        return Ok(());
    };
    let local = match choice {
        SlideListChoice::All => "sldAll",
        SlideListChoice::Range { .. } => "sldRg",
        SlideListChoice::CustomShow(_) => "custShow",
    };
    let new = w.new_child(el, ns::PML, "p", local)?;
    w.insert_in_order(el, order, local, &new)?;
    match *choice {
        SlideListChoice::All => {}
        SlideListChoice::Range { start, end } => {
            w.attr(&new, "st", None, Some(&start))?;
            w.attr(&new, "end", None, Some(&end))?;
        }
        SlideListChoice::CustomShow(id) => w.attr(&new, "id", None, Some(&id))?,
    }
    Ok(())
}

// spec-audit: owner=htmlPubPr
fn patch_html_publish(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&HtmlPublishProperties>,
    after: &HtmlPublishProperties,
) -> Result<(), ExportError> {
    let b = before.cloned().unwrap_or_default();
    w.attr(
        el,
        "showSpeakerNotes",
        b.show_speaker_notes.as_ref(),
        after.show_speaker_notes.as_ref(),
    )?;
    w.attr(el, "target", b.target.as_ref(), after.target.as_ref())?;
    w.attr(el, "title", b.title.as_ref(), after.title.as_ref())?;
    w.rel_attr(el, b.rel_id.as_deref(), after.rel_id.as_deref())?;
    patch_slide_list(
        w,
        el,
        HTML_ORDER,
        b.slide_list.as_ref(),
        after.slide_list.as_ref(),
    )
}

// spec-audit: owner=webPr
fn patch_web(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&WebProperties>,
    after: &WebProperties,
) -> Result<(), ExportError> {
    let b = before.cloned().unwrap_or_default();
    macro_rules! attrs {
        ($($name:literal => $field:ident),+ $(,)?) => {$(
            w.attr(el, $name, b.$field.as_ref(), after.$field.as_ref())?;
        )+};
    }
    attrs!(
        "showAnimation" => show_animation,
        "resizeGraphics" => resize_graphics,
        "allowPng" => allow_png,
        "relyOnVml" => rely_on_vml,
        "organizeInFolders" => organize_in_folders,
        "useLongFilenames" => use_long_filenames,
        "imgSz" => image_size,
        "encoding" => encoding,
        "clr" => color,
    );
    Ok(())
}

// spec-audit: owner=prnPr
fn patch_print(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&PrintProperties>,
    after: &PrintProperties,
) -> Result<(), ExportError> {
    let b = before.copied().unwrap_or_default();
    w.attr(
        el,
        "prnWhat",
        b.print_what.as_ref(),
        after.print_what.as_ref(),
    )?;
    w.attr(
        el,
        "clrMode",
        b.color_mode.as_ref(),
        after.color_mode.as_ref(),
    )?;
    w.attr(
        el,
        "hiddenSlides",
        b.hidden_slides.as_ref(),
        after.hidden_slides.as_ref(),
    )?;
    w.attr(
        el,
        "scaleToFitPaper",
        b.scale_to_fit_paper.as_ref(),
        after.scale_to_fit_paper.as_ref(),
    )?;
    w.attr(
        el,
        "frameSlides",
        b.frame_slides.as_ref(),
        after.frame_slides.as_ref(),
    )
}

// spec-audit: owner=present,browse,kiosk
fn patch_show_type(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&ShowType>,
    after: Option<&ShowType>,
) -> Result<(), ExportError> {
    if before == after {
        return Ok(());
    }
    let local_of = |t: &ShowType| match t {
        ShowType::Present => "present",
        ShowType::Browse { .. } => "browse",
        ShowType::Kiosk { .. } => "kiosk",
    };
    // The same kind keeps its element, and only its attribute moves.
    if let (Some(old), Some(new)) = (before, after)
        && local_of(old) == local_of(new)
        && let Some(existing) = el.first_child(w.tree, w.part, ns::PML, local_of(new))?
    {
        return patch_show_type_attr(w, &existing, Some(old), new);
    }
    for local in ["present", "browse", "kiosk"] {
        w.remove_all(el, ns::PML, local)?;
    }
    let Some(new) = after else {
        return Ok(());
    };
    let local = local_of(new);
    let created = w.new_child(el, ns::PML, "p", local)?;
    w.insert_in_order(el, SHOW_ORDER, local, &created)?;
    patch_show_type_attr(w, &created, None, new)
}

// spec-audit: owner=browse,kiosk
fn patch_show_type_attr(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&ShowType>,
    after: &ShowType,
) -> Result<(), ExportError> {
    match after {
        ShowType::Present => Ok(()),
        ShowType::Browse { show_scrollbar } => {
            let old = match before {
                Some(ShowType::Browse { show_scrollbar }) => show_scrollbar.as_ref(),
                _ => None,
            };
            w.attr(el, "showScrollbar", old, show_scrollbar.as_ref())
        }
        ShowType::Kiosk { restart } => {
            let old = match before {
                Some(ShowType::Kiosk { restart }) => restart.as_ref(),
                _ => None,
            };
            w.attr(el, "restart", old, restart.as_ref())
        }
    }
}

// spec-audit: owner=showPr
fn patch_show(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&ShowProperties>,
    after: &ShowProperties,
) -> Result<(), ExportError> {
    let b = before.copied().unwrap_or_default();
    w.attr(el, "loop", b.loop_show.as_ref(), after.loop_show.as_ref())?;
    w.attr(
        el,
        "showNarration",
        b.show_narration.as_ref(),
        after.show_narration.as_ref(),
    )?;
    w.attr(
        el,
        "showAnimation",
        b.show_animation.as_ref(),
        after.show_animation.as_ref(),
    )?;
    w.attr(
        el,
        "useTimings",
        b.use_timings.as_ref(),
        after.use_timings.as_ref(),
    )?;
    patch_show_type(w, el, b.show_type.as_ref(), after.show_type.as_ref())?;
    patch_slide_list(
        w,
        el,
        SHOW_ORDER,
        b.slide_list.as_ref(),
        after.slide_list.as_ref(),
    )
}

/// Patch a tree of `p:presentationPr` to say `after`. Returns the nodes changed.
// spec-audit: owner=presentationPr
pub fn patch_presentation_properties(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &PresentationProperties,
    limits: &ImportLimits,
) -> Result<usize, ExportError> {
    let mut quiet = Findings::new(limits.max_findings);
    let before = import_presentation_properties(tree, part, rels, limits, &mut quiet)?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
        changes: 0,
    };
    let w = &mut w;
    w.sync_child(
        &root,
        ns::PML,
        "p",
        PROPERTIES_ORDER,
        "htmlPubPr",
        before.html_publish.as_ref(),
        after.html_publish.as_ref(),
        patch_html_publish,
    )?;
    w.sync_child(
        &root,
        ns::PML,
        "p",
        PROPERTIES_ORDER,
        "webPr",
        before.web.as_ref(),
        after.web.as_ref(),
        patch_web,
    )?;
    w.sync_child(
        &root,
        ns::PML,
        "p",
        PROPERTIES_ORDER,
        "prnPr",
        before.print.as_ref(),
        after.print.as_ref(),
        patch_print,
    )?;
    w.sync_child(
        &root,
        ns::PML,
        "p",
        PROPERTIES_ORDER,
        "showPr",
        before.show.as_ref(),
        after.show.as_ref(),
        patch_show,
    )?;
    Ok(w.changes)
}

// spec-audit: owner=sx,sy
fn patch_ratio(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&Ratio>,
    after: &Ratio,
) -> Result<(), ExportError> {
    w.attr(el, "n", before.map(|b| &b.n), Some(&after.n))?;
    w.attr(el, "d", before.map(|b| &b.d), Some(&after.d))
}

// spec-audit: owner=scale
fn patch_scale(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&Scale>,
    after: &Scale,
) -> Result<(), ExportError> {
    let order = &["sx", "sy"];
    w.sync_child(
        el,
        ns::DML,
        "a",
        order,
        "sx",
        before.map(|b| &b.x),
        Some(&after.x),
        patch_ratio,
    )?;
    w.sync_child(
        el,
        ns::DML,
        "a",
        order,
        "sy",
        before.map(|b| &b.y),
        Some(&after.y),
        patch_ratio,
    )
}

// spec-audit: owner=origin
fn patch_point(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&Point>,
    after: &Point,
) -> Result<(), ExportError> {
    w.attr::<Coordinate>(el, "x", before.map(|b| &b.x), Some(&after.x))?;
    w.attr::<Coordinate>(el, "y", before.map(|b| &b.y), Some(&after.y))
}

// spec-audit: owner=cViewPr
fn patch_common_view(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&CommonViewProperties>,
    after: &CommonViewProperties,
) -> Result<(), ExportError> {
    let b = before.copied().unwrap_or_default();
    w.attr(
        el,
        "varScale",
        b.var_scale.as_ref(),
        after.var_scale.as_ref(),
    )?;
    w.sync_child(
        el,
        ns::PML,
        "p",
        COMMON_VIEW_ORDER,
        "scale",
        b.scale.as_ref(),
        after.scale.as_ref(),
        patch_scale,
    )?;
    w.sync_child(
        el,
        ns::PML,
        "p",
        COMMON_VIEW_ORDER,
        "origin",
        b.origin.as_ref(),
        after.origin.as_ref(),
        patch_point,
    )
}

// spec-audit: owner=guide
fn patch_guide(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&Guide>,
    after: &Guide,
) -> Result<(), ExportError> {
    let b = before.copied().unwrap_or_default();
    w.attr(el, "orient", b.orient.as_ref(), after.orient.as_ref())?;
    w.attr(el, "pos", b.pos.as_ref(), after.pos.as_ref())
}

// spec-audit: owner=cSldViewPr
fn patch_common_slide_view(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&CommonSlideViewProperties>,
    after: &CommonSlideViewProperties,
    limits: &ImportLimits,
) -> Result<(), ExportError> {
    let b = before.cloned().unwrap_or_default();
    w.attr(
        el,
        "snapToGrid",
        b.snap_to_grid.as_ref(),
        after.snap_to_grid.as_ref(),
    )?;
    w.attr(
        el,
        "snapToObjects",
        b.snap_to_objects.as_ref(),
        after.snap_to_objects.as_ref(),
    )?;
    w.attr(
        el,
        "showGuides",
        b.show_guides.as_ref(),
        after.show_guides.as_ref(),
    )?;
    w.sync_child(
        el,
        ns::PML,
        "p",
        COMMON_SLIDE_VIEW_ORDER,
        "cViewPr",
        b.common.as_ref(),
        after.common.as_ref(),
        patch_common_view,
    )?;
    w.sync_child(
        el,
        ns::PML,
        "p",
        COMMON_SLIDE_VIEW_ORDER,
        "guideLst",
        b.guides.as_ref(),
        after.guides.as_ref(),
        |w, list, _, wanted: &Vec<Guide>| {
            let existing =
                existing_list(w, list, limits, "guide", |r, e| read_guide(r, e).map(Some))?;
            w.sync_list(
                list,
                &existing,
                wanted,
                |_| String::new(),
                |w| w.new_child(list, ns::PML, "p", "guide"),
                patch_guide,
            )
        },
    )
}

/// `slideViewPr/cSldViewPr` and `notesViewPr/cSldViewPr`.
// spec-audit: owner=slideViewPr,notesViewPr
fn patch_slide_view_holder(
    w: &mut Writer<'_>,
    root: &Elem,
    local: &str,
    before: Option<&CommonSlideViewProperties>,
    after: Option<&CommonSlideViewProperties>,
    limits: &ImportLimits,
) -> Result<(), ExportError> {
    w.sync_child(
        root,
        ns::PML,
        "p",
        VIEW_ORDER,
        local,
        before,
        after,
        |w, holder, old, new| {
            w.sync_child(
                holder,
                ns::PML,
                "p",
                HOLDER_ORDER,
                "cSldViewPr",
                old,
                Some(new),
                |w, el, old, new| patch_common_slide_view(w, el, old, new, limits),
            )
        },
    )
}

// spec-audit: owner=restoredLeft,restoredTop
fn patch_normal_portion(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&NormalViewPortion>,
    after: &NormalViewPortion,
) -> Result<(), ExportError> {
    percent_attr(w, el, "sz", before.map(|b| &b.size), Some(&after.size))?;
    w.attr(
        el,
        "autoAdjust",
        before.and_then(|b| b.auto_adjust.as_ref()),
        after.auto_adjust.as_ref(),
    )
}

// spec-audit: owner=normalViewPr
fn patch_normal_view(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&NormalViewProperties>,
    after: &NormalViewProperties,
) -> Result<(), ExportError> {
    let b = before.copied().unwrap_or_default();
    macro_rules! attrs {
        ($($name:literal => $field:ident),+ $(,)?) => {$(
            w.attr(el, $name, b.$field.as_ref(), after.$field.as_ref())?;
        )+};
    }
    attrs!(
        "showOutlineIcons" => show_outline_icons,
        "snapVertSplitter" => snap_vert_splitter,
        "vertBarState" => vert_bar_state,
        "horzBarState" => horz_bar_state,
        "preferSingleView" => prefer_single_view,
    );
    w.sync_child(
        el,
        ns::PML,
        "p",
        NORMAL_ORDER,
        "restoredLeft",
        b.restored_left.as_ref(),
        after.restored_left.as_ref(),
        patch_normal_portion,
    )?;
    w.sync_child(
        el,
        ns::PML,
        "p",
        NORMAL_ORDER,
        "restoredTop",
        b.restored_top.as_ref(),
        after.restored_top.as_ref(),
        patch_normal_portion,
    )
}

// spec-audit: owner=sld
fn patch_outline_slide(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&OutlineSlide>,
    after: &OutlineSlide,
) -> Result<(), ExportError> {
    w.rel_attr(el, before.map(|b| b.rel_id.as_str()), Some(&after.rel_id))?;
    w.attr(
        el,
        "collapse",
        before.and_then(|b| b.collapse.as_ref()),
        after.collapse.as_ref(),
    )
}

// spec-audit: owner=outlineViewPr
fn patch_outline_view(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&OutlineViewProperties>,
    after: &OutlineViewProperties,
    limits: &ImportLimits,
) -> Result<(), ExportError> {
    let b = before.cloned().unwrap_or_default();
    w.sync_child(
        el,
        ns::PML,
        "p",
        OUTLINE_ORDER,
        "cViewPr",
        b.common.as_ref(),
        after.common.as_ref(),
        patch_common_view,
    )?;
    w.sync_child(
        el,
        ns::PML,
        "p",
        OUTLINE_ORDER,
        "sldLst",
        b.slides.as_ref(),
        after.slides.as_ref(),
        |w, list, _, wanted: &Vec<OutlineSlide>| {
            let no_rels = Rels::new(&[]);
            let existing = existing_list(w, list, limits, "sld", |r, e| {
                read_outline_slide(r, e, &no_rels)
            })?;
            w.sync_list(
                list,
                &existing,
                wanted,
                |s| s.rel_id.clone(),
                |w| w.new_child(list, ns::PML, "p", "sld"),
                patch_outline_slide,
            )
        },
    )
}

/// Patch a tree of `p:viewPr` to say `after`. Returns the nodes changed.
// spec-audit: owner=viewPr
pub fn patch_view_properties(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &ViewProperties,
    limits: &ImportLimits,
) -> Result<usize, ExportError> {
    let mut quiet = Findings::new(limits.max_findings);
    let before = import_view_properties(tree, part, rels, limits, &mut quiet)?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
        changes: 0,
    };
    let w = &mut w;
    w.attr(
        &root,
        "lastView",
        before.last_view.as_ref(),
        after.last_view.as_ref(),
    )?;
    w.attr(
        &root,
        "showComments",
        before.show_comments.as_ref(),
        after.show_comments.as_ref(),
    )?;
    w.sync_child(
        &root,
        ns::PML,
        "p",
        VIEW_ORDER,
        "normalViewPr",
        before.normal.as_ref(),
        after.normal.as_ref(),
        patch_normal_view,
    )?;
    patch_slide_view_holder(
        w,
        &root,
        "slideViewPr",
        before.slide.as_ref(),
        after.slide.as_ref(),
        limits,
    )?;
    w.sync_child(
        &root,
        ns::PML,
        "p",
        VIEW_ORDER,
        "outlineViewPr",
        before.outline.as_ref(),
        after.outline.as_ref(),
        |w, el, old, new| patch_outline_view(w, el, old, new, limits),
    )?;
    w.sync_child(
        &root,
        ns::PML,
        "p",
        VIEW_ORDER,
        "notesTextViewPr",
        before.notes_text.as_ref(),
        after.notes_text.as_ref(),
        |w, holder, old, new| {
            w.sync_child(
                holder,
                ns::PML,
                "p",
                HOLDER_ORDER,
                "cViewPr",
                old,
                Some(new),
                patch_common_view,
            )
        },
    )?;
    w.sync_child(
        &root,
        ns::PML,
        "p",
        VIEW_ORDER,
        "sorterViewPr",
        before.sorter.as_ref(),
        after.sorter.as_ref(),
        |w, el, old, new: &SorterViewProperties| {
            w.attr(
                el,
                "showFormatting",
                old.and_then(|o| o.show_formatting.as_ref()),
                new.show_formatting.as_ref(),
            )?;
            w.sync_child(
                el,
                ns::PML,
                "p",
                HOLDER_ORDER,
                "cViewPr",
                old.and_then(|o| o.common.as_ref()),
                new.common.as_ref(),
                patch_common_view,
            )
        },
    )?;
    patch_slide_view_holder(
        w,
        &root,
        "notesViewPr",
        before.notes.as_ref(),
        after.notes.as_ref(),
        limits,
    )?;
    w.sync_child(
        &root,
        ns::PML,
        "p",
        VIEW_ORDER,
        "gridSpacing",
        before.grid_spacing.as_ref(),
        after.grid_spacing.as_ref(),
        patch_positive_size,
    )?;
    Ok(w.changes)
}

/// What a save did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveOutcome {
    /// The package.
    pub bytes: Vec<u8>,
    /// The parts whose bytes differ, or are new, in order.
    pub changed_parts: Vec<String>,
    /// The deck carried a digital signature, and an edit has invalidated it.
    pub signature_invalidated: bool,
}

const CONTENT_TYPES_NS: &[&str] = &["http://schemas.openxmlformats.org/package/2006/content-types"];
const RELS_NS: &[&str] = &["http://schemas.openxmlformats.org/package/2006/relationships"];
const DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n";

/// The saving of one deck: part edits, and the structural parts new parts touch.
struct Save<'a> {
    package: &'a mut PresentationPackage,
    limits: ImportLimits,
    xml_limits: OoxmlLimits,
    strict: bool,
    edits: BTreeMap<String, Vec<u8>>,
    rels: BTreeMap<String, XmlTree>,
    content_types: Option<XmlTree>,
}

/// A part the model may hold a value for.
struct Role<'r> {
    source: &'r str,
    rel_types: &'r [&'r str],
    /// The name a new part takes, relative to the package root.
    new_part: String,
    /// `Target`, relative to `source`.
    new_target: String,
    content_type: &'r str,
    skeleton: String,
}

fn rels_part_for(part: &str) -> String {
    match part.rfind('/') {
        Some(i) => {
            let (dir, file) = part.split_at(i);
            format!("{dir}/_rels{file}.rels")
        }
        None if part.is_empty() => "_rels/.rels".to_owned(),
        None => format!("_rels/{part}.rels"),
    }
}

impl Save<'_> {
    fn parse_structural(&mut self, name: &str) -> Result<XmlTree, ExportError> {
        let bytes = self.package.read_xml_part(name)?;
        Ok(XmlTree::parse(&bytes, name, &self.xml_limits)?)
    }

    /// Give a new part its relationship, content type and bytes.
    fn add_part(&mut self, role: &Role<'_>, bytes: Vec<u8>) -> Result<(), ExportError> {
        let names = self.package.part_names();
        let taken = |n: &str| names.iter().any(|x| part_names_equivalent(x, n));
        if taken(&role.new_part) {
            return Err(ExportError::UnreadablePart {
                part: role.new_part.clone(),
            });
        }
        let rels_name = rels_part_for(role.source);
        let actual = names
            .iter()
            .find(|n| part_names_equivalent(n, &rels_name))
            .cloned();
        let mut tree = match self.rels.remove(&rels_name) {
            Some(tree) => tree,
            None => match &actual {
                Some(name) => self.parse_structural(&name.clone())?,
                None => XmlTree::parse(
                    format!("{DECL}<Relationships xmlns=\"{}\"/>", RELS_NS[0]).as_bytes(),
                    &rels_name,
                    &self.xml_limits,
                )?,
            },
        };
        // INV-RI-03, INV-RI-07: one above the part's highest, in its siblings' form.
        let existing = read_relationships(&tree);
        let id = next_rel_id(existing.iter().map(|r| r.id.as_str()));
        let internal = || existing.iter().filter(|r| r.mode == TargetMode::Internal);
        let absolute = internal().next().is_some() && internal().all(|r| r.target.starts_with('/'));
        let target = if absolute {
            target_for(role.source, &role.new_part, true)
        } else {
            role.new_target.clone()
        };
        let rel_type = role.rel_types[usize::from(self.strict && role.rel_types.len() > 1)];
        let root = Elem::root(&tree, &rels_name)?;
        {
            let mut w = Writer {
                tree: &mut tree,
                part: &rels_name,
                strict: self.strict,
                known_rels: None,
                changes: 0,
            };
            let new = w.new_child(&root, RELS_NS, "rel", "Relationship")?;
            w.attr(&new, "Id", None, Some(&id))?;
            w.attr(&new, "Type", None, Some(&rel_type.to_owned()))?;
            w.attr(&new, "Target", None, Some(&target))?;
            w.tree.insert_child(root.id, None, new.id);
        }
        self.rels.insert(rels_name, tree);

        const CT_PART: &str = "[Content_Types].xml";
        let mut ct = match self.content_types.take() {
            Some(tree) => tree,
            None => self.parse_structural(CT_PART)?,
        };
        let root = Elem::root(&ct, CT_PART)?;
        {
            let mut w = Writer {
                tree: &mut ct,
                part: CT_PART,
                strict: self.strict,
                known_rels: None,
                changes: 0,
            };
            // A phantom `Override` for this name would otherwise be declared twice
            // (`INV-CT-02`): the generated Strict deck had one, and preflight saw it.
            let part_name = format!("/{}", role.new_part);
            let existing = w.tree.child_elements(root.id).into_iter().find(|&el| {
                w.tree
                    .plain_attribute(el, "PartName")
                    .is_some_and(|n| n.eq_ignore_ascii_case(&part_name))
            });
            let content_type = role.content_type.to_owned();
            match existing {
                Some(el) => {
                    let el = Elem::read(w.tree, el, &root.scope, CT_PART)?;
                    w.attr(
                        &el,
                        "ContentType",
                        Some(&String::new()),
                        Some(&content_type),
                    )?;
                }
                None => {
                    let new = w.new_child(&root, CONTENT_TYPES_NS, "ct", "Override")?;
                    w.attr(&new, "PartName", None, Some(&part_name))?;
                    w.attr(&new, "ContentType", None, Some(&content_type))?;
                    w.tree.insert_child(root.id, None, new.id);
                }
            }
        }
        self.content_types = Some(ct);
        self.edits.insert(role.new_part.clone(), bytes);
        Ok(())
    }

    /// Bring one optional part to its wanted state.
    fn sync_part<T: PartialEq + Clone>(
        &mut self,
        role: &Role<'_>,
        after: Option<&T>,
        read: impl Fn(&XmlTree, &str, &[Relationship], &ImportLimits) -> Result<T, ImportError>,
        patch: impl Fn(
            &mut XmlTree,
            &str,
            bool,
            &[Relationship],
            &T,
            &ImportLimits,
        ) -> Result<usize, ExportError>,
    ) -> Result<(), ExportError> {
        let mut scratch = Findings::new(self.limits.max_findings);
        let location = locate_part(self.package, role.source, role.rel_types, &mut scratch);
        let part = match location {
            PartLocation::At(part) => part,
            PartLocation::Absent => {
                let Some(wanted) = after else { return Ok(()) };
                let mut tree =
                    XmlTree::parse(role.skeleton.as_bytes(), &role.new_part, &self.xml_limits)?;
                patch(
                    &mut tree,
                    &role.new_part,
                    self.strict,
                    &[],
                    wanted,
                    &self.limits,
                )?;
                let limits = self.limits;
                verify(&tree, &role.new_part, &self.xml_limits, wanted, |t| {
                    read(t, &role.new_part, &[], &limits)
                })?;
                return self.add_part(role, tree.to_bytes());
            }
            PartLocation::Unusable => {
                return match after {
                    None => Ok(()),
                    Some(_) => Err(ExportError::UnreadablePart {
                        part: role.new_part.clone(),
                    }),
                };
            }
        };
        let rels = self.package.relationships_of(&part).to_vec();
        let bytes = self.package.read_xml_part(&part)?;
        let tree = XmlTree::parse(&bytes, &part, &self.xml_limits)
            .map_err(ImportError::from)
            .and_then(|t| read(&t, &part, &rels, &self.limits).map(|_| t));
        let mut tree = match tree {
            Ok(tree) => tree,
            Err(err) if is_optional_part_fault(&err) => {
                return match after {
                    None => Ok(()),
                    Some(_) => Err(ExportError::UnreadablePart { part }),
                };
            }
            Err(err) => return Err(err.into()),
        };
        let Some(wanted) = after else {
            return Err(ExportError::PartRemovalUnsupported { part });
        };
        let changes = patch(&mut tree, &part, self.strict, &rels, wanted, &self.limits)?;
        if changes > 0 {
            let limits = self.limits;
            verify(&tree, &part, &self.xml_limits, wanted, |t| {
                read(t, &part, &rels, &limits)
            })?;
            self.edits.insert(part, tree.to_bytes());
        }
        Ok(())
    }
}

fn main_dir(main: &str) -> &str {
    main.rfind('/').map_or("", |i| main.get(..=i).unwrap_or(""))
}

/// Save `after` over the package it came from. What did not change is not touched;
/// with nothing changed, the input bytes come back. Parts whose bytes are not
/// edited keep their compressed bytes.
pub fn save_deck(
    package: &mut PresentationPackage,
    after: &Deck,
    limits: &ImportLimits,
) -> Result<SaveOutcome, ExportError> {
    let plan = SavePlan::new()
        .with_limits(*limits, WriterLimits::default())
        .acknowledge_signature()
        .deck(after);
    let outcome = plan.save(package).map_err(|err| match err {
        WriterError::Export(err) => err,
        other => ExportError::Refused(Box::new(other)),
    })?;
    Ok(SaveOutcome {
        bytes: outcome.bytes,
        changed_parts: outcome.changed_parts,
        signature_invalidated: outcome.signature_invalidated,
    })
}

/// The parts that bring the package to `after`: presentation, properties, and the
/// structural parts new ones touch. Nothing is written; [`SavePlan`] does that.
pub(crate) fn deck_changes(
    package: &mut PresentationPackage,
    after: &Deck,
    limits: &ImportLimits,
) -> Result<Vec<(String, Vec<u8>)>, ExportError> {
    let xml_limits = *package.limits();
    let strict = package.conformance() == Conformance::Strict;
    let main = package.main_part().to_owned();
    let dir = main_dir(&main).to_owned();
    let mut edits = BTreeMap::new();
    let main_rels = package.relationships_of(&main).to_vec();
    let main_bytes = package.read_xml_part(&main)?;
    let mut tree = XmlTree::parse(&main_bytes, &main, &xml_limits)?;
    let changes = patch_presentation(
        &mut tree,
        &main,
        strict,
        &main_rels,
        &after.presentation,
        limits,
    )?;
    if changes > 0 {
        verify(&tree, &main, &xml_limits, &after.presentation, |t| {
            let mut quiet = Findings::new(limits.max_findings);
            import_presentation(t, &main, &main_rels, limits, &mut quiet)
        })?;
        edits.insert(main.clone(), tree.to_bytes());
    }

    let (pml, core_skeleton, app_skeleton, custom_skeleton) = {
        let at = usize::from(strict);
        (
            ns::PML[at],
            format!(
                "{DECL}<cp:coreProperties xmlns:cp=\"{}\" xmlns:dc=\"{}\" xmlns:dcterms=\"{}\" xmlns:xsi=\"{}\"/>",
                ns::CORE[0],
                ns::DC[0],
                ns::DCTERMS[0],
                ns::XSI[0]
            ),
            format!(
                "{DECL}<Properties xmlns=\"{}\" xmlns:vt=\"{}\"/>",
                ns::EP[at],
                ns::VT[at]
            ),
            format!(
                "{DECL}<Properties xmlns=\"{}\" xmlns:vt=\"{}\"/>",
                ns::CP[at],
                ns::VT[at]
            ),
        )
    };
    let rel_uri = ns::REL[usize::from(strict)];
    let dml = ns::DML[usize::from(strict)];
    let mut save = Save {
        package,
        limits: *limits,
        xml_limits,
        strict,
        edits,
        rels: BTreeMap::new(),
        content_types: None,
    };

    let quietly = |limits: &ImportLimits| Findings::new(limits.max_findings);
    let pml_root = |local: &str| {
        format!("{DECL}<p:{local} xmlns:a=\"{dml}\" xmlns:r=\"{rel_uri}\" xmlns:p=\"{pml}\"/>")
    };
    save.sync_part(
        &Role {
            source: &main,
            rel_types: rel_types::PRES_PROPS,
            new_part: format!("{dir}presProps.xml"),
            new_target: "presProps.xml".to_owned(),
            content_type: "application/vnd.openxmlformats-officedocument.presentationml.presProps+xml",
            skeleton: pml_root("presentationPr"),
        },
        after.presentation_properties.as_ref(),
        |t, part, rels, l| import_presentation_properties(t, part, rels, l, &mut quietly(l)),
        patch_presentation_properties,
    )?;
    save.sync_part(
        &Role {
            source: &main,
            rel_types: rel_types::VIEW_PROPS,
            new_part: format!("{dir}viewProps.xml"),
            new_target: "viewProps.xml".to_owned(),
            content_type: "application/vnd.openxmlformats-officedocument.presentationml.viewProps+xml",
            skeleton: pml_root("viewPr"),
        },
        after.view_properties.as_ref(),
        |t, part, rels, l| import_view_properties(t, part, rels, l, &mut quietly(l)),
        patch_view_properties,
    )?;
    save.sync_part(
        &Role {
            source: "",
            rel_types: rel_types::CORE,
            new_part: "docProps/core.xml".to_owned(),
            new_target: "docProps/core.xml".to_owned(),
            content_type: "application/vnd.openxmlformats-package.core-properties+xml",
            skeleton: core_skeleton,
        },
        after.properties.core.as_ref(),
        |t, part, _, l| import_core_properties(t, part, l, &mut quietly(l)),
        |t, part, s, _, wanted, l| patch_core_properties(t, part, s, wanted, l),
    )?;
    save.sync_part(
        &Role {
            source: "",
            rel_types: rel_types::APP,
            new_part: "docProps/app.xml".to_owned(),
            new_target: "docProps/app.xml".to_owned(),
            content_type: "application/vnd.openxmlformats-officedocument.extended-properties+xml",
            skeleton: app_skeleton,
        },
        after.properties.app.as_ref(),
        |t, part, _, l| import_app_properties(t, part, l, &mut quietly(l)),
        |t, part, s, _, wanted, l| patch_app_properties(t, part, s, wanted, l),
    )?;
    save.sync_part(
        &Role {
            source: "",
            rel_types: rel_types::CUSTOM,
            new_part: "docProps/custom.xml".to_owned(),
            new_target: "docProps/custom.xml".to_owned(),
            content_type: "application/vnd.openxmlformats-officedocument.custom-properties+xml",
            skeleton: custom_skeleton,
        },
        after.properties.custom.as_ref(),
        |t, part, _, l| import_custom_properties(t, part, l, &mut quietly(l)),
        |t, part, s, _, wanted, l| patch_custom_properties(t, part, s, wanted, l),
    )?;

    let Save {
        mut edits,
        rels,
        content_types,
        ..
    } = save;
    for (name, tree) in rels {
        edits.insert(name, tree.to_bytes());
    }
    if let Some(tree) = content_types {
        edits.insert("[Content_Types].xml".to_owned(), tree.to_bytes());
    }
    Ok(edits.into_iter().collect())
}
