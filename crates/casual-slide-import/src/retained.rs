//! Read who a retained object is and what stands in for it, and name every retained part
//! (`SEC-C07`, Tier 1L). An OLE object, a control, a VML drawing and a 3D model are bytes:
//! nothing here decodes one, starts one, or follows a link out of the file.

use std::collections::BTreeSet;

use casual_slide_model::retained::{RetainedKind, RetainedObject};
use casual_slide_ooxml::{
    MARKUP_COMPATIBILITY, OoxmlError, PresentationPackage, Relationship, TargetMode,
    part_names_equivalent, resolve_internal,
};

use crate::properties::{Elem, ImportError, ImportFindingKind as K, Reader, ns};
use crate::slide::rel_suffix;

const MC: &[&str] = &[MARKUP_COMPATIBILITY];
const MODEL3D: &[&str] = &["http://schemas.microsoft.com/office/drawing/2017/model3d"];

/// What a retained relationship must be, by the end of its type.
fn accepts(kind: RetainedKind, rel: &Relationship) -> bool {
    let suffix = rel_suffix(rel);
    match kind {
        RetainedKind::OleEmbedded | RetainedKind::OleLinked => {
            matches!(suffix, Some("oleObject" | "package"))
        }
        RetainedKind::Control => suffix == Some("control"),
        RetainedKind::Model3d => rel.rel_type.ends_with("/model3d"),
    }
}

fn check_object_rel(r: &mut Reader<'_>, rels: &[Relationship], el: &Elem, o: &RetainedObject) {
    let attribute = if o.kind == RetainedKind::Model3d {
        "r:embed"
    } else {
        "r:id"
    };
    let Some(id) = o.rel_id.as_deref() else {
        r.note(K::MissingRequired, el, Some(attribute));
        return;
    };
    match rels.iter().find(|rel| rel.id == id) {
        None => r.note(K::DanglingRelationship, el, Some(attribute)),
        Some(rel) if !accepts(o.kind, rel) => r.note(K::WrongRelationship, el, Some(attribute)),
        // A link may leave the file, as it is never followed. Nothing else may.
        Some(rel) if rel.mode == TargetMode::External && o.kind != RetainedKind::OleLinked => {
            r.note(K::WrongRelationship, el, Some(attribute));
        }
        Some(_) => {}
    }
}

fn check_links(r: &mut Reader<'_>, rels: &[Relationship], el: &Elem, o: &RetainedObject) {
    check_object_rel(r, rels, el, o);
    if let Some(id) = o.fallback_rel_id.as_deref() {
        match rels.iter().find(|rel| rel.id == id) {
            None => r.note(K::DanglingRelationship, el, Some("r:embed")),
            Some(rel) if rel_suffix(rel) != Some("image") || rel.mode == TargetMode::External => {
                r.note(K::WrongRelationship, el, Some("r:embed"));
            }
            Some(_) => {}
        }
    }
}

/// `a:blip/@r:embed` in a `p:pic`, read without taking anything: the picture itself is
/// retained and reported whole.
fn picture_fallback(r: &mut Reader<'_>, pic: &Elem) -> Result<Option<String>, ImportError> {
    for fill in r.children(pic)? {
        if fill.local == "blipFill" {
            for mut blip in r.children(&fill)? {
                if blip.is(ns::DML, "blip") {
                    return Ok(blip.take_ns(ns::REL, "embed"));
                }
            }
        }
    }
    Ok(None)
}

/// The later reading fills only what the earlier left unsaid: the two branches of an
/// `mc:AlternateContent` hold one object, and often only one says `@spid` or holds the picture.
fn merge(into: &mut RetainedObject, other: RetainedObject) {
    into.spid = into.spid.take().or(other.spid);
    into.display_name = into.display_name.take().or(other.display_name);
    into.prog_id = into.prog_id.take().or(other.prog_id);
    into.show_as_icon = into.show_as_icon.or(other.show_as_icon);
    into.img_w = into.img_w.or(other.img_w);
    into.img_h = into.img_h.or(other.img_h);
    into.rel_id = into.rel_id.take().or(other.rel_id);
    into.fallback_rel_id = into.fallback_rel_id.take().or(other.fallback_rel_id);
}

/// `ST_PositiveCoordinate32`, which a value outside of is reported and left out.
// spec-audit: owner=oleObj,control
fn image_size(
    r: &mut Reader<'_>,
    attrs: &mut Elem,
    name: &str,
) -> Result<Option<i64>, ImportError> {
    let value: Option<i64> = r.attr(attrs, name)?;
    Ok(value.filter(|v| {
        let fits = (0..=i64::from(i32::MAX)).contains(v);
        if !fits {
            r.note(K::OutOfRange, attrs, Some(name));
        }
        fits
    }))
}

// spec-audit: owner=oleObj,control
fn read_ole_attributes(
    r: &mut Reader<'_>,
    el: &Elem,
    kind: RetainedKind,
) -> Result<RetainedObject, ImportError> {
    let mut attrs = el.clone();
    let mut o = RetainedObject::new(kind);
    o.spid = attrs.take("spid");
    o.display_name = attrs.take("name");
    o.show_as_icon = r.attr(&mut attrs, "showAsIcon")?;
    o.img_w = image_size(r, &mut attrs, "imgW")?;
    o.img_h = image_size(r, &mut attrs, "imgH")?;
    if kind != RetainedKind::Control {
        o.prog_id = attrs.take("progId");
    }
    o.rel_id = attrs.take_ns(ns::REL, "id");
    r.finish(&attrs);
    Ok(o)
}

/// `p:oleObj` or `p:control`: identity, whether it is a link, and the picture standing in.
fn read_object(
    r: &mut Reader<'_>,
    el: &Elem,
    kind: RetainedKind,
) -> Result<RetainedObject, ImportError> {
    let mut o = read_ole_attributes(r, el, kind)?;
    for child in r.children(el)? {
        if child.is(ns::PML, "link") && kind != RetainedKind::Control {
            o.kind = RetainedKind::OleLinked;
        } else if child.is(ns::PML, "pic") {
            let found = picture_fallback(r, &child)?;
            o.fallback_rel_id = o.fallback_rel_id.take().or(found);
        }
        // `p:embed` is the default kind. Each stays in the file, and is reported.
        r.retained(&child);
    }
    r.retained(el);
    Ok(o)
}

/// `am3d:model3d`: the model's relationship and the raster it was last drawn as. Its
/// camera, lights and transform stay in the file until `D3-01`.
// spec-audit: owner=model3d
fn read_model(r: &mut Reader<'_>, el: &Elem) -> Result<RetainedObject, ImportError> {
    let mut attrs = el.clone();
    let mut o = RetainedObject::new(RetainedKind::Model3d);
    o.rel_id = attrs.take_ns(ns::REL, "embed");
    for child in r.children(el)? {
        if child.is(MODEL3D, "raster") {
            for mut blip in r.children(&child)? {
                if blip.is(MODEL3D, "blip") {
                    o.fallback_rel_id = blip.take_ns(ns::REL, "embed");
                }
            }
        }
    }
    Ok(o)
}

/// The children of an `mc:AlternateContent`'s branches in order, or the element itself.
fn branches(r: &mut Reader<'_>, el: &Elem) -> Result<Vec<Elem>, ImportError> {
    let mut out = Vec::new();
    if el.is(MC, "AlternateContent") {
        for branch in r.children(el)? {
            out.extend(r.children(&branch)?);
        }
    } else {
        out.push(el.clone());
    }
    Ok(out)
}

/// What a graphic frame's `a:graphicData` child holds, if it is an object or a model.
/// Both branches of an `mc:AlternateContent` count, whichever would be taken, as the picture
/// is usually only in the fallback. The caller has already reported `el`.
pub(crate) fn read_frame_content(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    el: &Elem,
) -> Result<Option<RetainedObject>, ImportError> {
    let mut found: Option<RetainedObject> = None;
    for item in branches(r, el)? {
        let read = if item.is(ns::PML, "oleObj") {
            read_object(r, &item, RetainedKind::OleEmbedded)?
        } else if item.is(MODEL3D, "model3d") {
            r.retained(&item);
            read_model(r, &item)?
        } else {
            continue;
        };
        match &mut found {
            Some(first) => merge(first, read),
            None => found = Some(read),
        }
    }
    if let Some(o) = &found {
        check_links(r, rels, el, o);
    }
    Ok(found)
}

/// `p:controls`: each control, the branches of one `mc:AlternateContent` being one control.
pub(crate) fn read_controls(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    el: &Elem,
) -> Result<Vec<RetainedObject>, ImportError> {
    let mut out = Vec::new();
    r.retained(el);
    for child in r.children(el)? {
        let mut one: Option<RetainedObject> = None;
        for item in branches(r, &child)? {
            if !item.is(ns::PML, "control") {
                r.retained(&item);
                continue;
            }
            let read = read_object(r, &item, RetainedKind::Control)?;
            match &mut one {
                Some(first) => merge(first, read),
                None => one = Some(read),
            }
        }
        if let Some(o) = one {
            check_links(r, rels, &child, &o);
            out.push(o);
        }
    }
    Ok(out)
}

/// A 3D model in a branch of `mc:AlternateContent` that was not taken, with `pic` the picture
/// the taken branch drew in its place. A consumer that does not understand `am3d` takes the
/// fallback, so that picture's node is what carries the model.
pub(crate) fn model_beside_picture(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    alternate: &Elem,
    pic: &Elem,
) -> Result<Option<RetainedObject>, ImportError> {
    for branch in r.children(alternate)? {
        for frame in r.children(&branch)? {
            if !frame.is(ns::PML, "graphicFrame") {
                continue;
            }
            for graphic in r.children(&frame)? {
                if !graphic.is(ns::DML, "graphic") {
                    continue;
                }
                for data in r.children(&graphic)? {
                    for model in r.children(&data)? {
                        if model.is(MODEL3D, "model3d") {
                            let mut o = read_model(r, &model)?;
                            o.fallback_rel_id = picture_fallback(r, pic)?.or(o.fallback_rel_id);
                            check_links(r, rels, alternate, &o);
                            return Ok(Some(o));
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Which family a retained part belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RetainedPartKind {
    /// An OLE object or an embedded package.
    Ole,
    /// An ActiveX control's part, or its binary.
    Control,
    /// A legacy VML drawing.
    Vml,
    /// A 3D model.
    Model3d,
}

/// What became of a retained part. It stays in the saved file whichever this is.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RetainedStatus {
    /// Its bytes are there and intact.
    Kept,
    /// It is a link out of the file, which is never followed.
    External,
    /// It is missing or its bytes are damaged. Set aside, with why, and the deck opens.
    SetAside {
        /// The stable code (docs/20).
        code: &'static str,
    },
}

/// One retained part, for the report. Part names only, never document content.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RetainedPart {
    /// Which family.
    pub kind: RetainedPartKind,
    /// The part holding the relationship, or empty for a part nothing refers to.
    pub source: String,
    /// The relationship's id, or empty for a part nothing refers to.
    pub rel_id: String,
    /// The part's name. Empty for a link, whose target is document content, or a missing part.
    pub target: String,
    /// What became of it.
    pub status: RetainedStatus,
}

fn kind_of_relationship(rel: &Relationship) -> Option<RetainedPartKind> {
    match rel_suffix(rel) {
        Some("oleObject" | "package") => Some(RetainedPartKind::Ole),
        Some("control") => Some(RetainedPartKind::Control),
        Some("vmlDrawing") => Some(RetainedPartKind::Vml),
        _ if rel.rel_type.ends_with("/activeXControlBinary") => Some(RetainedPartKind::Control),
        _ if rel.rel_type.ends_with("/model3d") => Some(RetainedPartKind::Model3d),
        _ => None,
    }
}

fn kind_of_name(name: &str) -> Option<RetainedPartKind> {
    let lower = name.to_ascii_lowercase();
    if lower.contains("/_rels/") {
        None
    } else if lower.ends_with(".vml") {
        Some(RetainedPartKind::Vml)
    } else if lower.starts_with("ppt/embeddings/") {
        Some(RetainedPartKind::Ole)
    } else if lower.starts_with("ppt/activex/") {
        Some(RetainedPartKind::Control)
    } else if lower.ends_with(".glb") {
        Some(RetainedPartKind::Model3d)
    } else {
        None
    }
}

/// Name every retained part in the deck, referred to or not, and set aside any that is
/// missing or damaged. Each is read once, to check its bytes, by a second reader of the
/// file, so this charges nothing to `package`'s read budget, and the bytes are dropped
/// unlooked at. When that second reader's budget runs out, what is left is set aside.
pub fn retained_report(package: &PresentationPackage) -> Vec<RetainedPart> {
    let mut names = package.part_names();
    names.sort();
    let mut checker = None;
    let mut targets = BTreeSet::new();
    let mut out = Vec::new();
    for source in &names {
        for rel in package.relationships_of(source) {
            let Some(kind) = kind_of_relationship(rel) else {
                continue;
            };
            let (target, status) = if rel.mode == TargetMode::External {
                (String::new(), RetainedStatus::External)
            } else {
                let actual = resolve_internal(source, &rel.target)
                    .and_then(|t| names.iter().find(|n| part_names_equivalent(n, &t)));
                match actual {
                    Some(name) => {
                        targets.insert(name.clone());
                        (name.clone(), check_bytes(package, &mut checker, name))
                    }
                    None => (
                        String::new(),
                        RetainedStatus::SetAside {
                            code: missing_code(),
                        },
                    ),
                }
            };
            out.push(RetainedPart {
                kind,
                source: source.clone(),
                rel_id: rel.reportable_id().unwrap_or_default().to_owned(),
                target,
                status,
            });
        }
    }
    for name in &names {
        if !targets.contains(name)
            && let Some(kind) = kind_of_name(name)
        {
            out.push(RetainedPart {
                kind,
                source: String::new(),
                rel_id: String::new(),
                target: name.clone(),
                status: check_bytes(package, &mut checker, name),
            });
        }
    }
    out.sort();
    out
}

fn missing_code() -> &'static str {
    OoxmlError::MissingPart {
        name: String::new(),
    }
    .code()
}

fn check_bytes(
    package: &PresentationPackage,
    checker: &mut Option<PresentationPackage>,
    name: &str,
) -> RetainedStatus {
    if checker.is_none() {
        match PresentationPackage::open(package.save_unedited(), *package.limits()) {
            Ok(opened) => *checker = Some(opened),
            Err(err) => return RetainedStatus::SetAside { code: err.code() },
        }
    }
    let Some(reader) = checker.as_mut() else {
        return RetainedStatus::SetAside {
            code: missing_code(),
        };
    };
    match reader.read_part(name) {
        // It reads, because a repair read past what was wrong with it. Not intact.
        Ok(_) => match reader
            .recovery_report()
            .lines
            .iter()
            .find(|line| line.part == name)
        {
            Some(line) => RetainedStatus::SetAside { code: line.code },
            None => RetainedStatus::Kept,
        },
        Err(err) => RetainedStatus::SetAside { code: err.code() },
    }
}
