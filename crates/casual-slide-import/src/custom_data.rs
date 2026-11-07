//! Read tags, custom XML properties and sensitivity labels, and name what is kept whole
//! (Tier 1M): change tracking, revision info, and the bodies of custom XML items.
//!
//! [`import_review`] reads everything of this tier from a deck, and cannot fail: a part it
//! cannot read is set aside, retained byte for byte, and named (owner instruction 24).
//!
//! **Retained whole means the bytes are never parsed by the model.** Only a part's name and
//! kind are reported. An item's body is read only to learn whether its root is a schema
//! library, and nothing else of it reaches the model.
//!
//! Evidence: `MS-PPTX` for `changesInfo` and `revisionInfo`, and `fixtures/real` for their
//! relationship types. The `classificationlabels` relationship and `LabelInfo`'s attributes,
//! and `MSIP_Label_*` custom properties, are Inferred from Microsoft's public description of
//! sensitivity labels: no deck here holds one.

use std::collections::HashSet;

use casual_slide_model::presentation::Deck;
use casual_slide_model::properties::Variant;
use casual_slide_model::custom_data::{
    CustomXmlItem, DatastoreItem, DatastoreSchemaRef, DeckCustomData, RetainedKind, RetainedPart,
    Schema, SchemaLibrary, StringTag, TagList, TagPart,
};
use casual_slide_model::section::{DeckLabels, LabelSource, PartMarks, SensitivityLabel};
use casual_slide_model::comment::DeckComments;
use casual_slide_ooxml::{PresentationPackage, XmlTree};

use crate::color::Findings as ColorFindings;
use crate::comment::{
    ReviewFinding, ReviewLimits, first, guid_attr, name_map, read_deck_comments,
    read_part_or_set_aside, targets_of,
};
use crate::properties::{Elem, Findings, ImportError, ImportFindingKind as K, ImportLimits, Reader, ns};
use crate::section::{check_zoom_targets, import_part_marks};

/// `ds:`.
pub const DS: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/customXml",
    "http://purl.oclc.org/ooxml/officeDocument/customXml",
];
/// `sl:`.
pub const SL: &[&str] = &[
    "http://schemas.openxmlformats.org/schemaLibrary/2006/main",
    "http://purl.oclc.org/ooxml/schemaLibrary/main",
];
/// `clbl:`, Inferred.
pub const CLBL: &[&str] = &["http://schemas.microsoft.com/office/2020/mipLabelMetadata"];

/// The `tags` relationship, Transitional and Strict.
pub const TAGS_RELATIONSHIP: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/tags",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/tags",
];
/// The `customXml` relationship, Transitional and Strict.
pub const CUSTOM_XML_RELATIONSHIP: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/customXml",
];
/// The `customXmlProps` relationship, Transitional and Strict.
pub const CUSTOM_XML_PROPS_RELATIONSHIP: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/customXmlProps",
];
/// The `changesInfo` relationship, as `fixtures/real` has it.
pub const CHANGES_INFO_RELATIONSHIP: &[&str] =
    &["http://schemas.microsoft.com/office/2016/11/relationships/changesInfo"];
/// The `revisionInfo` relationship, as `fixtures/real` has it.
pub const REVISION_INFO_RELATIONSHIP: &[&str] =
    &["http://schemas.microsoft.com/office/2015/10/relationships/revisionInfo"];
/// The sensitivity label relationship, Inferred.
pub const LABELS_RELATIONSHIP: &[&str] =
    &["http://schemas.microsoft.com/office/2020/02/relationships/classificationlabels"];

/// `p:CT_TagList`.
// spec-audit: owner=tagLst
// spec-audit: p:<tagLst>
pub fn import_tags(
    tree: &XmlTree,
    part: &str,
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
) -> Result<TagList, ImportError> {
    let mut reader = Reader::new(tree, part, import, findings);
    let r = &mut reader;
    let root = r.root(ns::PML, "tagLst")?;
    r.finish(&root);
    let mut out = TagList::default();
    for child in r.bounded_children(&root, limits.max_tags, "tags")? {
        if child.is(ns::PML, "tag") {
            out.tags.push(read_tag(r, &child)?);
        } else {
            r.retained(&child);
        }
    }
    Ok(out)
}

// spec-audit: owner=tag
fn read_tag(r: &mut Reader<'_>, el: &Elem) -> Result<StringTag, ImportError> {
    let mut e = el.clone();
    let out = StringTag {
        name: r.required(&mut e, "name", |_: &String| true)?,
        val: r.required(&mut e, "val", |_: &String| true)?,
    };
    r.finish(&e);
    for child in r.children(el)? {
        r.retained(&child);
    }
    Ok(out)
}

/// `ds:CT_DatastoreItem`.
// spec-audit: owner=datastoreItem
// spec-audit: ds:<datastoreItem>
pub fn import_datastore_item(
    tree: &XmlTree,
    part: &str,
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
) -> Result<DatastoreItem, ImportError> {
    let mut reader = Reader::new(tree, part, import, findings);
    let r = &mut reader;
    let root = r.root(DS, "datastoreItem")?;
    let mut e = root.clone();
    let item_id = e.take_ns(DS, "itemID");
    if item_id.is_none() {
        r.note(K::MissingRequired, &root, Some("itemID"));
    }
    let good = item_id.as_ref().filter(|i| guid_attr(i)).cloned();
    if item_id.is_some() && good.is_none() {
        r.note(K::MalformedValue, &root, Some("itemID"));
    }
    r.finish(&e);
    let mut out = DatastoreItem {
        item_id: good,
        schema_refs: None,
    };
    let mut seen = HashSet::new();
    for child in r.children(&root)? {
        if !child.is(DS, "schemaRefs") {
            r.retained(&child);
        } else if first(r, &mut seen, "schemaRefs", &child) {
            out.schema_refs = Some(read_schema_refs(r, &child, limits)?);
        }
    }
    Ok(out)
}

// spec-audit: owner=schemaRefs
fn read_schema_refs(
    r: &mut Reader<'_>,
    el: &Elem,
    limits: &ReviewLimits,
) -> Result<Vec<DatastoreSchemaRef>, ImportError> {
    r.finish(el);
    let mut out = Vec::new();
    for child in r.bounded_children(el, limits.max_schema_refs, "schema references")? {
        if child.is(DS, "schemaRef") {
            out.push(read_schema_ref(r, &child));
        } else {
            r.retained(&child);
        }
    }
    Ok(out)
}

// spec-audit: owner=schemaRef
fn read_schema_ref(r: &mut Reader<'_>, el: &Elem) -> DatastoreSchemaRef {
    let mut e = el.clone();
    let uri = e.take_ns(DS, "uri");
    if uri.is_none() {
        r.note(K::MissingRequired, el, Some("uri"));
    }
    r.finish(&e);
    DatastoreSchemaRef { uri }
}

/// `sl:CT_SchemaLibrary`.
// spec-audit: owner=schemaLibrary
// spec-audit: sl:<schemaLibrary>
pub fn import_schema_library(
    tree: &XmlTree,
    part: &str,
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
) -> Result<SchemaLibrary, ImportError> {
    let mut reader = Reader::new(tree, part, import, findings);
    let r = &mut reader;
    let root = r.root(SL, "schemaLibrary")?;
    r.finish(&root);
    let mut out = SchemaLibrary::default();
    for child in r.bounded_children(&root, limits.max_schema_refs, "schemas")? {
        if child.is(SL, "schema") {
            out.schemas.push(read_schema(r, &child));
        } else {
            r.retained(&child);
        }
    }
    Ok(out)
}

// spec-audit: owner=schema
fn read_schema(r: &mut Reader<'_>, el: &Elem) -> Schema {
    let mut e = el.clone();
    let out = Schema {
        uri: e.take_ns(SL, "uri"),
        manifest_location: e.take_ns(SL, "manifestLocation"),
        schema_location: e.take_ns(SL, "schemaLocation"),
        schema_language: e.take_ns(SL, "schemaLanguage"),
    };
    r.finish(&e);
    out
}

/// Sensitivity label metadata. Inferred: see the module documentation.
pub fn import_label_info(
    tree: &XmlTree,
    part: &str,
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
) -> Result<Vec<SensitivityLabel>, ImportError> {
    let mut reader = Reader::new(tree, part, import, findings);
    let r = &mut reader;
    let root = r.root(CLBL, "labelList")?;
    r.finish(&root);
    let mut out = Vec::new();
    for child in r.bounded_children(&root, limits.max_labels, "labels")? {
        if !child.is(CLBL, "label") {
            r.retained(&child);
            continue;
        }
        let mut e = child.clone();
        let label = SensitivityLabel {
            source: LabelSource::LabelPart,
            id: r.attr_in(&mut e, "id", guid_attr)?,
            site_id: r.attr_in(&mut e, "siteId", guid_attr)?,
            enabled: r.attr(&mut e, "enabled")?,
            method: r.attr(&mut e, "method")?,
            name: None,
            content_bits: r.attr(&mut e, "contentBits")?,
            removed: r.attr(&mut e, "removed")?,
        };
        r.finish(&e);
        out.push(label);
    }
    Ok(out)
}

/// Labels left in custom properties. A property's name holds a label's id, so these are the
/// only property names read here.
pub fn labels_in_custom_properties(deck: &Deck, limits: &ReviewLimits) -> Vec<SensitivityLabel> {
    let mut out: Vec<SensitivityLabel> = Vec::new();
    let Some(custom) = &deck.properties.custom else {
        return out;
    };
    for property in &custom.properties {
        let Some(name) = &property.name else { continue };
        let Some(rest) = name.strip_prefix("MSIP_Label_") else {
            continue;
        };
        let Some((id, field)) = rest.split_once('_') else {
            continue;
        };
        let at = match out.iter().position(|l| l.id.as_deref() == Some(id)) {
            Some(at) => at,
            None => {
                if out.len() >= limits.max_labels {
                    continue;
                }
                out.push(SensitivityLabel {
                    source: LabelSource::CustomProperties,
                    id: Some(id.to_owned()),
                    ..SensitivityLabel::default()
                });
                out.len() - 1
            }
        };
        let Some(label) = out.get_mut(at) else { continue };
        let text = match &property.value {
            Some(Variant::Lpwstr(v) | Variant::Lpstr(v)) => Some(v.clone()),
            _ => None,
        };
        match field {
            "Enabled" => label.enabled = text.as_deref().map(|t| t.eq_ignore_ascii_case("true")),
            "Name" => label.name = text,
            "SiteId" => label.site_id = text,
            "Method" => label.method = text,
            "Removed" => label.removed = text.as_deref().map(|t| t.eq_ignore_ascii_case("true")),
            "ContentBits" => label.content_bits = text.as_deref().and_then(|t| t.parse().ok()),
            _ => {}
        }
    }
    out
}

/// Read every tag part and custom XML item in a deck. See the module documentation.
pub fn read_deck_custom_data(
    package: &mut PresentationPackage,
    slide_parts: &[String],
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
) -> DeckCustomData {
    let names = name_map(package);
    let main = package.main_part().to_owned();
    let mut out = DeckCustomData::default();
    let mut set_aside = Vec::new();
    let mut seen_tags = HashSet::new();
    let sources = std::iter::once(main.clone()).chain(slide_parts.iter().cloned());
    for source in sources {
        let rels = package.relationships_of(&source).to_vec();
        for part in targets_of(&names, &source, &rels, TAGS_RELATIONSHIP, findings) {
            if !seen_tags.insert(part.clone()) {
                continue;
            }
            if seen_tags.len() > limits.max_review_parts {
                findings.note(K::Review(ReviewFinding::OverBound), &part, "", None);
                set_aside.push(part);
                continue;
            }
            let read = read_part_or_set_aside(package, &part, findings, &mut set_aside, |t, _, f| {
                import_tags(t, &part, limits, import, f)
            });
            if let Some(list) = read {
                out.tags.push(TagPart { part, list });
            }
        }
    }
    let rels = package.relationships_of(&main).to_vec();
    let items = targets_of(&names, &main, &rels, CUSTOM_XML_RELATIONSHIP, findings);
    for (i, part) in items.into_iter().enumerate() {
        if i >= limits.max_review_parts {
            findings.note(K::Review(ReviewFinding::OverBound), &part, "", None);
            set_aside.push(part);
            continue;
        }
        let mut item = CustomXmlItem {
            part: part.clone(),
            ..CustomXmlItem::default()
        };
        let item_rels = package.relationships_of(&part).to_vec();
        let props = targets_of(&names, &part, &item_rels, CUSTOM_XML_PROPS_RELATIONSHIP, findings);
        if let Some(props) = props.first() {
            item.properties_part = Some(props.clone());
            item.properties = read_part_or_set_aside(package, props, findings, &mut set_aside, |t, _, f| {
                import_datastore_item(t, props, limits, import, f)
            });
        }
        // An item is whatever its owner put there, so it may not be XML we can read, or
        // XML at all. That is not damage: nothing here needs it to be.
        item.schema_library = read_item_library(package, &part, limits, import, findings);
        findings.note(K::Review(ReviewFinding::RetainedWhole), &part, "customXml", None);
        out.custom_xml.push(item);
    }
    out.set_aside = set_aside;
    out
}

fn read_item_library(
    package: &mut PresentationPackage,
    part: &str,
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
) -> Option<SchemaLibrary> {
    let xml = *package.limits();
    let bytes = package.read_xml_part(part).ok()?;
    let tree = XmlTree::parse(&bytes, part, &xml).ok()?;
    let root = Elem::root(&tree, part).ok()?;
    if !root.is(SL, "schemaLibrary") {
        return None;
    }
    import_schema_library(&tree, part, limits, import, findings).ok()
}

/// Name every part of this tier kept whole, in a stable order.
pub fn retained_parts(package: &PresentationPackage, findings: &mut Findings) -> Vec<RetainedPart> {
    let names = name_map(package);
    let main = package.main_part().to_owned();
    let rels = package.relationships_of(&main).to_vec();
    let mut out = Vec::new();
    let mut take = |types: &[&str], kind: RetainedKind, findings: &mut Findings| {
        for part in targets_of(&names, &main, &rels, types, findings) {
            findings.note(K::Review(ReviewFinding::RetainedWhole), &part, kind.name(), None);
            out.push(RetainedPart { part, kind });
        }
    };
    take(CHANGES_INFO_RELATIONSHIP, RetainedKind::ChangesInfo, findings);
    take(REVISION_INFO_RELATIONSHIP, RetainedKind::RevisionInfo, findings);
    take(CUSTOM_XML_RELATIONSHIP, RetainedKind::CustomXml, &mut Findings::new(0));
    let root_rels = package.relationships_of("").to_vec();
    for part in targets_of(&names, "", &root_rels, LABELS_RELATIONSHIP, findings) {
        out.push(RetainedPart {
            part,
            kind: RetainedKind::LabelInfo,
        });
    }
    out.sort();
    out
}

/// Everything of this tier a deck holds, beyond what the main part's reader holds. Reading it
/// reads every slide, so this is asked for and is not part of opening a deck.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedReview {
    /// Comments, in both forms.
    pub comments: DeckComments,
    /// Tags and custom XML.
    pub custom: DeckCustomData,
    /// Sensitivity labels.
    pub labels: DeckLabels,
    /// Each slide-like part's marks, for parts that hold any.
    pub marks: Vec<(String, PartMarks)>,
    /// Slide-like parts that could not be walked. Set aside and named.
    pub marks_set_aside: Vec<String>,
    /// Parts kept whole.
    pub retained: Vec<RetainedPart>,
    /// The report.
    pub findings: Findings,
    /// What the colour readers noticed in comment text.
    pub color_findings: ColorFindings,
}

/// The slide-like parts of a deck, by following relationships from the main part: slides,
/// and each slide's layout and master and notes. An unreachable part holds nothing here.
pub fn slide_like_parts(package: &PresentationPackage) -> Vec<String> {
    const KINDS: [&str; 6] = [
        "slide",
        "slideLayout",
        "slideMaster",
        "notesSlide",
        "notesMaster",
        "handoutMaster",
    ];
    let names = name_map(package);
    let mut scratch = Findings::new(0);
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut queue = vec![package.main_part().to_owned()];
    while let Some(source) = queue.pop() {
        let rels = package.relationships_of(&source).to_vec();
        for rel in &rels {
            let kind = crate::slide::rel_suffix(rel);
            if !kind.is_some_and(|k| KINDS.contains(&k)) {
                continue;
            }
            let single = [rel.rel_type.as_str()];
            for part in targets_of(&names, &source, std::slice::from_ref(rel), &single, &mut scratch) {
                if seen.insert(part.clone()) {
                    queue.push(part.clone());
                    out.push(part);
                }
            }
        }
    }
    out.sort();
    out
}

/// Read everything of this tier. Never refuses: see the module documentation.
pub fn import_review(
    package: &mut PresentationPackage,
    deck: &Deck,
    limits: &ReviewLimits,
    import: &ImportLimits,
) -> ImportedReview {
    let mut findings = Findings::new(import.max_findings);
    let mut color = ColorFindings::default();
    let slide_parts = slide_like_parts(package);
    let comments = read_deck_comments(package, &slide_parts, limits, import, &mut findings, &mut color);
    let custom = read_deck_custom_data(package, &slide_parts, limits, import, &mut findings);
    let retained = retained_parts(package, &mut findings);
    let mut marks = Vec::new();
    let mut marks_set_aside = Vec::new();
    for part in &slide_parts {
        let read = read_part_or_set_aside(package, part, &mut findings, &mut marks_set_aside, |t, r, f| {
            import_part_marks(t, part, r, limits, import, f)
        });
        if let Some(m) = read
            && m != PartMarks::default()
        {
            marks.push((part.clone(), m));
        }
    }
    check_zoom_targets(&deck.presentation, &marks, &mut findings);
    check_comment_rels(package, &marks, &mut findings);
    let mut labels = DeckLabels {
        labels: labels_in_custom_properties(deck, limits),
        set_aside: None,
    };
    let names = name_map(package);
    let root_rels = package.relationships_of("").to_vec();
    for part in targets_of(&names, "", &root_rels, LABELS_RELATIONSHIP, &mut findings) {
        let mut aside = Vec::new();
        let read = read_part_or_set_aside(package, &part, &mut findings, &mut aside, |t, _, f| {
            import_label_info(t, &part, limits, import, f)
        });
        match read {
            Some(found) => labels.labels.extend(found),
            None => labels.set_aside = aside.pop(),
        }
    }
    ImportedReview {
        comments,
        custom,
        labels,
        marks,
        marks_set_aside,
        retained,
        findings,
        color_findings: color,
    }
}

/// A `commentRel` should name a comment relationship of its own part.
fn check_comment_rels(
    package: &PresentationPackage,
    marks: &[(String, PartMarks)],
    findings: &mut Findings,
) {
    for (part, m) in marks {
        let Some(id) = &m.comment_rel else { continue };
        let good = package.relationships_of(part).iter().any(|r| {
            &r.id == id && crate::comment::THREADED_COMMENTS_RELATIONSHIP.contains(&r.rel_type.as_str())
        });
        if !good {
            findings.note(K::Review(ReviewFinding::CommentRelMismatch), part, "commentRel", Some("id"));
        }
    }
}
