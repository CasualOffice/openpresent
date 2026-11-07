//! Patch equations in place (Tier 1K): `a14:m`, and the `mc:AlternateContent` around one.
//!
//! An equation with a fallback is written with both branches (`INV-FB-09`). An edit of the
//! equation alone would leave its fallback runs saying something else, and there is no
//! math layout to redraw them, so it is refused (`INV-FB-03`). Editing both is fine.

pub mod schema;

use casual_slide_import::diagram::GraphicLimits;
use casual_slide_import::math::{A14, import_math_root};
use casual_slide_import::properties::{Elem, Findings, ImportLimits};
use casual_slide_model::math::schema as m;
use casual_slide_model::math::{InlineMath, MathAlternate, MathItem, TextMath};
use casual_slide_model::text::TextItem;
use casual_slide_ooxml::{MARKUP_COMPATIBILITY, OoxmlLimits, Relationship, XmlTree};

use crate::diagram::{R, sync_items};
use crate::properties::{ExportError, Writer};
use crate::slide::SlideExportError;
use crate::text::patch_items;

const MC: &[&str] = &[MARKUP_COMPATIBILITY];
const NS_M: &[&str] = casual_slide_import::math::schema::NS_M;

/// `a14:m`'s content.
// spec-audit: owner=m
pub(crate) fn patch_text_math(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&TextMath>,
    a: &TextMath,
) -> R {
    sync_items(
        w,
        el,
        &[],
        &[(NS_M, "m", "oMath"), (NS_M, "m", "oMathPara")],
        b.map_or(&[][..], |b| b.items.as_slice()),
        &a.items,
        |i| match i {
            MathItem::OMath(_) => (NS_M, "m", "oMath"),
            MathItem::OMathPara(_) => (NS_M, "m", "oMathPara"),
        },
        |w, el, b, a| match a {
            MathItem::OMath(a) => {
                let before = match b {
                    Some(MathItem::OMath(b)) => Some(&**b),
                    _ => None,
                };
                schema::patch_o_math(w, el, before, a)
            }
            MathItem::OMathPara(a) => {
                let before = match b {
                    Some(MathItem::OMathPara(b)) => Some(&**b),
                    _ => None,
                };
                schema::patch_o_math_para(w, el, before, a)
            }
        },
    )
}

fn refused(w: &Writer<'_>, what: &'static str) -> ExportError {
    ExportError::UnsupportedEdit {
        what,
        part: w.part.to_owned(),
    }
}

/// Which element an inline equation is written as.
pub(crate) fn inline_name(m: &InlineMath) -> (&'static [&'static str], &'static str, &'static str) {
    if m.alternate.is_some() {
        (MC, "mc", "AlternateContent")
    } else {
        // spec-audit: x:drawing/2010/main:<m>
        (A14, "a14", "m")
    }
}

/// Bring an inline equation to a value. `el` is the `a14:m`, or the `mc:AlternateContent`.
pub(crate) fn patch_inline_math(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&InlineMath>,
    a: &InlineMath,
) -> R {
    let Some(alternate) = &a.alternate else {
        let Some(math) = &a.math else {
            return Err(refused(w, "an inline equation with no a14:m"));
        };
        return patch_text_math(w, el, b.and_then(|b| b.math.as_ref()), math);
    };
    if let Some(b) = b
        && b.math != a.math
        && b.alternate.as_ref().and_then(|x| x.fallback.as_ref()) == alternate.fallback.as_ref()
        && alternate.fallback.as_ref().is_some_and(|f| !f.is_empty())
    {
        return Err(refused(
            w,
            "an equation edit that leaves its fallback showing something else",
        ));
    }
    patch_alternate(w, el, b, a.math.as_ref(), alternate)
}

fn patch_alternate(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&InlineMath>,
    math: Option<&TextMath>,
    alternate: &MathAlternate,
) -> R {
    let before = b.and_then(|b| b.alternate.as_ref());
    let choice = match el.first_child(w.tree, w.part, MC, "Choice")? {
        Some(found) => found,
        None => {
            let fresh = w.new_child(el, MC, "mc", "Choice")?;
            w.tree.insert_child(el.id, None, fresh.id);
            fresh
        }
    };
    if before.and_then(|x| x.requires.as_ref()) != alternate.requires.as_ref() {
        match &alternate.requires {
            Some(requires) => {
                // `Requires` names prefixes, so each must be in scope here. `a14` is the
                // one this writer knows; another must already be declared.
                for prefix in requires.split_ascii_whitespace() {
                    if choice.scope.uri(prefix).is_none() {
                        if prefix != "a14" {
                            return Err(refused(w, "Requires naming a prefix not in scope"));
                        }
                        w.attr(
                            &choice,
                            "xmlns:a14",
                            None::<&String>,
                            Some(&A14.first().copied().unwrap_or_default().to_owned()),
                        )?;
                    }
                }
                w.attr(&choice, "Requires", None::<&String>, Some(requires))?;
            }
            None => {
                w.attr(
                    &choice,
                    "Requires",
                    before.and_then(|x| x.requires.as_ref()),
                    None,
                )?;
            }
        }
    }
    // The namespace just declared has to be seen by what comes next.
    let choice = el
        .first_child(w.tree, w.part, MC, "Choice")?
        .unwrap_or(choice);
    if let Some(math) = math {
        let holder = match choice.first_child(w.tree, w.part, A14, "m")? {
            Some(found) => found,
            None => {
                let fresh = w.new_child(&choice, A14, "a14", "m")?;
                w.tree.insert_child(choice.id, None, fresh.id);
                fresh
            }
        };
        patch_text_math(w, &holder, b.and_then(|b| b.math.as_ref()), math)?;
    } else {
        w.remove_all(&choice, A14, "m")?;
    }
    let old_fallback = before.and_then(|x| x.fallback.as_ref());
    match &alternate.fallback {
        None => w.remove_all(el, MC, "Fallback")?,
        Some(items) => {
            let fallback = match el.first_child(w.tree, w.part, MC, "Fallback")? {
                Some(found) => found,
                None => {
                    let fresh = w.new_child(el, MC, "mc", "Fallback")?;
                    w.tree.insert_child(el.id, None, fresh.id);
                    fresh
                }
            };
            let empty: Vec<TextItem> = Vec::new();
            patch_items(w, &fallback, old_fallback.unwrap_or(&empty), items)?;
        }
    }
    Ok(())
}

/// Bring a tree holding standalone OMML to a value, read back whole.
#[allow(clippy::too_many_arguments)] // As `patch_slide_part`.
pub fn patch_math_part(
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
        import_math_root(tree, part, rels, limits, import_limits, &mut findings)
    };
    let fault = |e: casual_slide_import::diagram::GraphicError| SlideExportError::Unsupported {
        what: match e.code() {
            "CS-GFX-0001" | "CS-GFX-0002" => "a part over a bound",
            _ => "a part that does not read",
        },
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
