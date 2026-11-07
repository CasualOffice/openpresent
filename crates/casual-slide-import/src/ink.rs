//! Read ink (Tier 1K, `docs/spec/INK.md`): the part a `p:contentPart` names.
//!
//! A part that cannot be read is set aside and reported, and stays in the package as it
//! was; the slide's fallback picture is a shape on the slide, read by Tier 1D and kept as
//! it is. A stroke's points are held as the file wrote them, so a hostile trace costs its
//! length, bounded by `max_value_bytes`. `max_traces` bounds how many there are. Evidence:
//! W3C InkML 1.0; `MS-ODRAWXML` 2.9 (`msink`, `iact`); `INV-FB-06` in `docs/36`.

pub mod schema;

use casual_slide_model::diagram::SetAside;
use casual_slide_model::ink::InkPart;
use casual_slide_model::ink::schema as m;
use casual_slide_model::shape::{ObjectId, ShapeNode};
use casual_slide_model::slide::SlidePart;
use casual_slide_ooxml::{PresentationPackage, Relationship, XmlTree};

use crate::color::Findings as ColorFindings;
use crate::diagram::{
    GraphicError, GraphicFinding, GraphicLimits, SchemaReader, read_tree, resolve_rel,
};
use crate::drawing::DrawingLimits;
use crate::properties::{
    Elem, Findings, ImportError, ImportFindingKind as K, ImportLimits, Reader,
};
use crate::text::TextCounts;

/// Read a tree as a standalone element of ink. `None` if its root is none of them.
pub fn import_ink_root(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    limits: &GraphicLimits,
    import_limits: &ImportLimits,
    findings: &mut Findings,
) -> Result<Option<m::Root>, GraphicError> {
    let mut r = Reader::new(tree, part, import_limits, findings);
    let root = Elem::root(tree, part).map_err(ImportError::from)?;
    let mut color = ColorFindings::default();
    let mut counts = TextCounts::default();
    let mut c = SchemaReader::new(
        &mut r,
        rels,
        &mut color,
        &mut counts,
        DrawingLimits::default(),
        *limits,
    );
    schema::read_root(&mut c, &root)
}

/// Read an ink part. This never refuses: a part that cannot be read is set aside.
pub fn import_ink_part(
    package: &mut PresentationPackage,
    part: &str,
    limits: &GraphicLimits,
    import_limits: &ImportLimits,
    findings: &mut Findings,
) -> InkPart {
    let mut out = InkPart {
        part: part.to_owned(),
        ..InkPart::default()
    };
    let read = read_tree(package, part).and_then(|tree| {
        let rels = package.relationships_of(part).to_vec();
        import_ink_root(&tree, part, &rels, limits, import_limits, findings)
    });
    let code = match read {
        Ok(Some(root)) => {
            out.root = Some(root);
            return out;
        }
        Ok(None) => "CS-IMP-0003",
        Err(err) => err.code(),
    };
    findings.note(K::Graphic(GraphicFinding::PartSetAside), part, "ink", None);
    out.set_aside = Some(SetAside {
        part: part.to_owned(),
        code: code.to_owned(),
    });
    out
}

/// Every ink part of a slide-like part: each content part's, directly on the slide and in
/// the `mc:Choice` its fallback picture stands in for. A relationship naming nothing is
/// reported.
pub fn import_slide_ink(
    package: &mut PresentationPackage,
    slide_part: &str,
    part: &SlidePart,
    limits: &GraphicLimits,
    import_limits: &ImportLimits,
    findings: &mut Findings,
) -> Vec<(ObjectId, InkPart)> {
    let mut out = Vec::new();
    for node in part
        .shape_tree()
        .into_iter()
        .flat_map(ShapeNode::descendants)
    {
        let direct = node.rel_id.iter().map(|id| (node.id, id.as_str()));
        let chosen = node
            .alternates
            .iter()
            .flat_map(|a| a.content_parts.iter())
            .map(|id| (node.id, id.as_str()));
        for (owner, id) in direct.chain(chosen) {
            match resolve_rel(package, slide_part, id) {
                Some(name) => out.push((
                    owner,
                    import_ink_part(package, &name, limits, import_limits, findings),
                )),
                None => findings.note(
                    K::DanglingRelationship,
                    slide_part,
                    "contentPart",
                    Some("r:id"),
                ),
            }
        }
    }
    out
}
