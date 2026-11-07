//! Patch ink parts in place (Tier 1K).
//!
//! A stroke's fallback picture is a shape on the slide, with bytes this crate cannot redraw,
//! so `save_ink_part` says what it left stale (`INV-FB-06`); moving and resizing a content
//! part patches the slide, not this part. An ink part set aside on import cannot be edited,
//! and comes back byte for byte.

pub mod schema;

use casual_slide_import::diagram::GraphicLimits;
use casual_slide_import::ink::{import_ink_part, import_ink_root};
use casual_slide_import::properties::{Elem, Findings, ImportLimits};
use casual_slide_model::ink::InkPart;
use casual_slide_model::ink::schema as m;
use casual_slide_ooxml::{Conformance, OoxmlLimits, PresentationPackage, Relationship, XmlTree};
use casual_slide_package::PartEdits;

use crate::presentation::SaveOutcome;
use crate::properties::{ExportError, Writer};
use crate::slide::SlideExportError;

/// Bring a tree holding a standalone element of ink to a value, read back whole.
#[allow(clippy::too_many_arguments)] // As `patch_slide_part`.
pub fn patch_ink_part(
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
        import_ink_root(tree, part, rels, limits, import_limits, &mut findings)
    };
    let fault = |_| SlideExportError::Unsupported {
        what: "a part that does not read",
        part: part.to_owned(),
    };
    let before = read(tree).map_err(fault)?;
    if before.as_ref() == Some(after) {
        return Ok(0);
    }
    let Some(before) = before else {
        return Err(SlideExportError::Unsupported {
            what: "a part of another kind",
            part: part.to_owned(),
        });
    };
    if std::mem::discriminant(&before) != std::mem::discriminant(after) {
        return Err(SlideExportError::Unsupported {
            what: "a part of another kind",
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
    schema::patch_root(&mut w, &root, Some(&before), after)?;
    let changes = w.changes;
    let again = XmlTree::parse(&tree.to_bytes(), part, xml).map_err(ExportError::from)?;
    if read(&again).map_err(fault)?.as_ref() != Some(after) {
        return Err(SlideExportError::Export(ExportError::RoundTrip {
            part: part.to_owned(),
        }));
    }
    Ok(changes)
}

/// Save an ink part over the package it came from. Only its part changes; with nothing
/// changed, the input bytes come back. `stale_fallback` tells the host the slide's picture
/// still shows the old strokes (`INV-FB-06`, `CS-SAV-0007`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InkSave {
    /// The saved deck, the parts changed, and whether a signature broke.
    pub outcome: SaveOutcome,
    /// The strokes changed, so a fallback picture shows the old ones.
    pub stale_fallback: bool,
}

/// See [`InkSave`].
pub fn save_ink_part(
    package: &mut PresentationPackage,
    after: &InkPart,
    limits: &GraphicLimits,
    import_limits: &ImportLimits,
) -> Result<InkSave, SlideExportError> {
    let mut findings = Findings::new(import_limits.max_findings);
    let before = import_ink_part(package, &after.part, limits, import_limits, &mut findings);
    if before.root == after.root {
        return Ok(InkSave {
            outcome: SaveOutcome {
                bytes: package.save_unedited(),
                changed_parts: Vec::new(),
                signature_invalidated: false,
            },
            stale_fallback: false,
        });
    }
    let (true, Some(root)) = (before.set_aside.is_none(), after.root.as_ref()) else {
        return Err(SlideExportError::Unsupported {
            what: "an edit of a part set aside on import, or taking a part away",
            part: after.part.clone(),
        });
    };
    let strict = package.conformance() == Conformance::Strict;
    let xml: OoxmlLimits = *package.limits();
    let bytes = package
        .read_xml_part(&after.part)
        .map_err(ExportError::from)?;
    let mut tree = XmlTree::parse(&bytes, &after.part, &xml).map_err(ExportError::from)?;
    let rels = package.relationships_of(&after.part).to_vec();
    patch_ink_part(
        &mut tree,
        &after.part,
        strict,
        &rels,
        root,
        limits,
        import_limits,
        &xml,
    )?;
    let mut edits = PartEdits::default();
    edits.replace(after.part.clone(), tree.to_bytes());
    let signed = package
        .findings()
        .iter()
        .any(|f| f.kind == casual_slide_ooxml::FindingKind::Signature);
    Ok(InkSave {
        outcome: SaveOutcome {
            bytes: package.save_with(&edits).map_err(ExportError::from)?,
            changed_parts: vec![after.part.clone()],
            signature_invalidated: signed,
        },
        stale_fallback: true,
    })
}
