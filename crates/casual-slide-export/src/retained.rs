//! Patch tags, custom XML properties and schema libraries, and save everything of Tier 1M.
//!
//! What is retained whole is not touched here at all: a part this module was not asked to
//! change goes out as its compressed bytes (`docs/22`, the fidelity contract). So change
//! tracking, revision info and every custom XML item's body are byte-identical through any
//! edit, whichever part it names.
//!
//! A new part for a deck that has none is the writer's, as a relationship and a content type
//! must come with it, and is refused here as `CS-EXP-0009`.

use casual_slide_import::comment::ReviewLimits;
use casual_slide_import::properties::{Elem, Findings, ImportLimits};
use casual_slide_import::retained::{
    DS, ImportedReview, SL, import_datastore_item, import_schema_library, import_tags,
};
use casual_slide_model::properties::Lexical;
use casual_slide_model::retained::{DatastoreItem, DatastoreSchemaRef, Schema, SchemaLibrary, StringTag, TagList};
use casual_slide_ooxml::{Conformance, OoxmlLimits, PresentationPackage, XmlTree};
use casual_slide_package::PartEdits;

use crate::comment::{
    counted, patch_authors, patch_comment_authors, patch_comments, patch_threaded_comments,
    sync_items,
};
use crate::drawing::{Node, Space, patch};
use crate::presentation::SaveOutcome;
use crate::properties::{ExportError, Writer, verify};
use crate::section::patch_part_marks;

type Result<T = ()> = std::result::Result<T, ExportError>;

// spec-audit: owner=tag
fn tag_node(t: &StringTag) -> Node {
    Node::new(Space::Pml, "tag")
        .lex("name", t.name.as_ref())
        .lex("val", t.val.as_ref())
}

/// Patch a `tags` tree to say `after`. Returns the nodes changed.
// spec-audit: owner=tagLst
// spec-audit: p:<tagLst>
pub fn patch_tags(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    after: &TagList,
    limits: &ReviewLimits,
    import: &ImportLimits,
    xml: &OoxmlLimits,
) -> Result<usize> {
    let read =
        |t: &XmlTree| import_tags(t, part, limits, import, &mut Findings::new(import.max_findings));
    let before = read(tree)?;
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
    sync_items(
        &mut w,
        &root,
        Space::Pml,
        "tag",
        &before.tags,
        &after.tags,
        |_| None,
        |w, el, b, a| patch(w, el, b.map(tag_node).as_ref(), &tag_node(a)),
    )?;
    let changes = w.changes;
    verify(tree, part, xml, after, read)?;
    Ok(changes)
}

/// An attribute in the element's own namespace, as `ds:` and `sl:` attributes are.
fn qualified_attr<T: Lexical + PartialEq>(
    w: &mut Writer<'_>,
    el: &Elem,
    uris: &[&str],
    prefix: &str,
    local: &str,
    before: Option<&T>,
    after: Option<&T>,
) -> Result {
    if before == after {
        return Ok(());
    }
    let name = match el.name_of_ns(uris, local) {
        Some(name) => name.to_owned(),
        None => format!("{}:{local}", w.declared_prefix(el, uris, prefix)?),
    };
    w.attr(el, &name, before, after)
}

// spec-audit: owner=schemaRef
fn patch_schema_ref(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&DatastoreSchemaRef>,
    a: &DatastoreSchemaRef,
) -> Result {
    qualified_attr(w, el, DS, "ds", "uri", b.and_then(|b| b.uri.as_ref()), a.uri.as_ref())
}

/// Patch a `customXmlProps` tree to say `after`. Returns the nodes changed.
// spec-audit: owner=datastoreItem
// spec-audit: ds:<datastoreItem>
pub fn patch_datastore_item(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    after: &DatastoreItem,
    limits: &ReviewLimits,
    import: &ImportLimits,
    xml: &OoxmlLimits,
) -> Result<usize> {
    let read = |t: &XmlTree| {
        import_datastore_item(t, part, limits, import, &mut Findings::new(import.max_findings))
    };
    let before = read(tree)?;
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
    let w = &mut w;
    qualified_attr(w, &root, DS, "ds", "itemID", before.item_id.as_ref(), after.item_id.as_ref())?;
    if before.schema_refs != after.schema_refs {
        let found = root.first_child(w.tree, w.part, DS, "schemaRefs")?;
        match (found, after.schema_refs.as_ref()) {
            (Some(list), None) => {
                w.tree.remove_child(root.id, list.id);
                w.changes += 1;
            }
            (found, Some(refs)) => {
                let list = match found {
                    Some(list) => list,
                    None => {
                        let list = w.new_child(&root, DS, "ds", "schemaRefs")?;
                        w.tree.insert_child(root.id, None, list.id);
                        list
                    }
                };
                let before_refs = before.schema_refs.as_deref().unwrap_or(&[]);
                let existing = counted(w, &list, DS, "schemaRef", before_refs.len())?;
                for el in existing.iter().skip(refs.len()) {
                    w.tree.remove_child(list.id, el.id);
                    w.changes += 1;
                }
                for (i, a) in refs.iter().enumerate() {
                    match (existing.get(i), before_refs.get(i)) {
                        (Some(el), b) => patch_schema_ref(w, el, b, a)?,
                        (None, _) => {
                            let el = w.new_child(&list, DS, "ds", "schemaRef")?;
                            w.tree.insert_child(list.id, None, el.id);
                            patch_schema_ref(w, &el, None, a)?;
                        }
                    }
                }
            }
            (None, None) => {}
        }
    }
    let changes = w.changes;
    verify(tree, part, xml, after, read)?;
    Ok(changes)
}

// spec-audit: owner=schema
fn patch_schema(w: &mut Writer<'_>, el: &Elem, b: Option<&Schema>, a: &Schema) -> Result {
    let d = Schema::default();
    let b = b.unwrap_or(&d);
    qualified_attr(w, el, SL, "sl", "uri", b.uri.as_ref(), a.uri.as_ref())?;
    qualified_attr(
        w,
        el,
        SL,
        "sl",
        "manifestLocation",
        b.manifest_location.as_ref(),
        a.manifest_location.as_ref(),
    )?;
    qualified_attr(
        w,
        el,
        SL,
        "sl",
        "schemaLocation",
        b.schema_location.as_ref(),
        a.schema_location.as_ref(),
    )?;
    qualified_attr(
        w,
        el,
        SL,
        "sl",
        "schemaLanguage",
        b.schema_language.as_ref(),
        a.schema_language.as_ref(),
    )
}

/// Patch a schema library tree to say `after`. Returns the nodes changed.
// spec-audit: owner=schemaLibrary
// spec-audit: sl:<schemaLibrary>
pub fn patch_schema_library(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    after: &SchemaLibrary,
    limits: &ReviewLimits,
    import: &ImportLimits,
    xml: &OoxmlLimits,
) -> Result<usize> {
    let read = |t: &XmlTree| {
        import_schema_library(t, part, limits, import, &mut Findings::new(import.max_findings))
    };
    let before = read(tree)?;
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
    let w = &mut w;
    let existing = counted(w, &root, SL, "schema", before.schemas.len())?;
    for el in existing.iter().skip(after.schemas.len()) {
        w.tree.remove_child(root.id, el.id);
        w.changes += 1;
    }
    for (i, a) in after.schemas.iter().enumerate() {
        match (existing.get(i), before.schemas.get(i)) {
            (Some(el), b) => patch_schema(w, el, b, a)?,
            (None, _) => {
                let el = w.new_child(&root, SL, "sl", "schema")?;
                w.tree.insert_child(root.id, None, el.id);
                patch_schema(w, &el, None, a)?;
            }
        }
    }
    let changes = w.changes;
    verify(tree, part, xml, after, read)?;
    Ok(changes)
}

/// One part's tree, to patch.
fn read_tree(package: &mut PresentationPackage, part: &str) -> Result<(XmlTree, Vec<u8>)> {
    let xml = *package.limits();
    let bytes = package.read_part(part)?;
    let tree = XmlTree::parse(&bytes, part, &xml)?;
    Ok((tree, bytes))
}

/// Save everything of this tier over the package it came from. Only the parts that differ
/// change, and only their named nodes. With nothing changed, the input bytes come back, which
/// a part that was set aside cannot stop: it is not named, so it is not touched.
pub fn save_review(
    package: &mut PresentationPackage,
    after: &ImportedReview,
    limits: &ReviewLimits,
    import: &ImportLimits,
) -> Result<SaveOutcome> {
    let strict = package.conformance() == Conformance::Strict;
    let xml = *package.limits();
    let mut edits = PartEdits::default();
    let mut changed = Vec::new();
    let mut finish = |edits: &mut PartEdits, part: &str, tree: XmlTree, n: usize, was: Vec<u8>| {
        let out = tree.to_bytes();
        if n > 0 && out != was {
            edits.replace(part.to_owned(), out);
            changed.push(part.to_owned());
        }
    };
    let main = package.main_part().to_owned();
    if let Some(wanted) = &after.comments.comment_authors {
        let Some(part) = first_target(package, &main, casual_slide_import::comment::COMMENT_AUTHORS_RELATIONSHIP) else {
            return Err(new_part(&main));
        };
        let (mut tree, was) = read_tree(package, &part)?;
        let n = patch_comment_authors(&mut tree, &part, strict, wanted, limits, import, &xml)?;
        finish(&mut edits, &part, tree, n, was);
    }
    if let Some(wanted) = &after.comments.authors {
        let Some(part) = first_target(package, &main, casual_slide_import::comment::AUTHORS_RELATIONSHIP) else {
            return Err(new_part(&main));
        };
        let (mut tree, was) = read_tree(package, &part)?;
        let n = patch_authors(&mut tree, &part, strict, wanted, limits, import, &xml)?;
        finish(&mut edits, &part, tree, n, was);
    }
    for slide in &after.comments.legacy {
        let (mut tree, was) = read_tree(package, &slide.part)?;
        let n = patch_comments(&mut tree, &slide.part, strict, &slide.list, limits, import, &xml)?;
        finish(&mut edits, &slide.part, tree, n, was);
    }
    for slide in &after.comments.threaded {
        let rels = package.relationships_of(&slide.part).to_vec();
        let (mut tree, was) = read_tree(package, &slide.part)?;
        let n = patch_threaded_comments(
            &mut tree, &slide.part, strict, &rels, &slide.list, limits, import, &xml,
        )?;
        finish(&mut edits, &slide.part, tree, n, was);
    }
    for tags in &after.custom.tags {
        let (mut tree, was) = read_tree(package, &tags.part)?;
        let n = patch_tags(&mut tree, &tags.part, strict, &tags.list, limits, import, &xml)?;
        finish(&mut edits, &tags.part, tree, n, was);
    }
    for item in &after.custom.custom_xml {
        if let (Some(part), Some(wanted)) = (&item.properties_part, &item.properties) {
            let (mut tree, was) = read_tree(package, part)?;
            let n = patch_datastore_item(&mut tree, part, strict, wanted, limits, import, &xml)?;
            finish(&mut edits, part, tree, n, was);
        }
        if let Some(wanted) = &item.schema_library {
            let (mut tree, was) = read_tree(package, &item.part)?;
            let n = patch_schema_library(&mut tree, &item.part, strict, wanted, limits, import, &xml)?;
            finish(&mut edits, &item.part, tree, n, was);
        }
    }
    for (part, wanted) in &after.marks {
        let rels = package.relationships_of(part).to_vec();
        let (mut tree, was) = read_tree(package, part)?;
        let n = patch_part_marks(&mut tree, part, strict, &rels, wanted, limits, import, &xml)?;
        finish(&mut edits, part, tree, n, was);
    }
    if changed.is_empty() {
        return Ok(SaveOutcome {
            bytes: package.save_unedited(),
            changed_parts: Vec::new(),
            signature_invalidated: false,
        });
    }
    let signed = package
        .findings()
        .iter()
        .any(|f| f.kind == casual_slide_ooxml::FindingKind::Signature);
    Ok(SaveOutcome {
        bytes: package.save_with(&edits)?,
        changed_parts: changed,
        signature_invalidated: signed,
    })
}

fn new_part(main: &str) -> ExportError {
    ExportError::UnsupportedEdit {
        what: "a new part for what the deck has none of",
        part: main.to_owned(),
    }
}

fn first_target(package: &PresentationPackage, source: &str, types: &[&str]) -> Option<String> {
    let mut quiet = Findings::new(0);
    match casual_slide_import::presentation::locate_part(package, source, types, &mut quiet) {
        casual_slide_import::presentation::PartLocation::At(part) => Some(part),
        _ => None,
    }
}
