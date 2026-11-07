//! Read equations (Tier 1K, `docs/spec/MATH.md`): `a14:m` in a paragraph, alone or in an
//! `mc:AlternateContent`, and OMML standing alone.
//!
//! `mc:AlternateContent` here keeps both branches in the model: the equation and its
//! fallback runs. Tier 1F held one as a name and nothing else (`T1F-04`), so a body holding
//! one could not be written from the model.
//!
//! An equation over a bound in `GraphicLimits` does not stop the slide: it stays in the
//! tree as [`TextItem::Retained`], reported. Evidence: ECMA-376 Part 1 22.1;
//! `MS-ODRAWXML` for `a14:m`; `INV-FB-03` in `docs/36`.

pub mod schema;

use casual_slide_model::math::schema as m;
use casual_slide_model::math::{InlineMath, MathAlternate, MathItem, TextMath};
use casual_slide_model::text::TextItem;
use casual_slide_ooxml::{MARKUP_COMPATIBILITY, Relationship, XmlTree};

use crate::color::Findings as ColorFindings;
use crate::diagram::{GraphicError, GraphicFinding, GraphicLimits, Res, SchemaReader};
use crate::drawing::DrawingLimits;
use crate::properties::{
    Elem, Findings, ImportError, ImportFindingKind as K, ImportLimits, Reader, ns,
};
use crate::text::{TextCounts, TextReader};

/// `a14:`.
pub const A14: &[&str] = &["http://schemas.microsoft.com/office/drawing/2010/main"];
const MC: &[&str] = &[MARKUP_COMPATIBILITY];

/// `a14:m`.
// spec-audit: owner=m
pub(crate) fn read_text_math(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<TextMath> {
    c.enter()?;
    c.finish(el);
    let mut out = TextMath::default();
    for child in c.children(el)? {
        if child.is(schema::NS_M, "oMath") {
            out.items
                .push(MathItem::OMath(Box::new(schema::read_o_math(c, &child)?)));
        } else if child.is(schema::NS_M, "oMathPara") {
            out.items
                .push(MathItem::OMathPara(Box::new(schema::read_o_math_para(
                    c, &child,
                )?)));
        } else {
            c.retained(&child);
        }
    }
    c.leave();
    Ok(out)
}

/// What a whole `mc:AlternateContent` in a paragraph holds, if it is an equation's: one
/// `Choice`, holding only an `a14:m`, and at most one `Fallback`. Anything else is not for
/// this reader, and stays [`TextItem::Retained`].
struct Shape {
    choice: Elem,
    math: Elem,
    fallback: Option<Elem>,
}

fn equation_shape(r: &Reader<'_>, el: &Elem) -> Result<Option<Shape>, ImportError> {
    let (mut choice, mut fallback) = (None, None);
    for child in r.children(el)? {
        if child.is(MC, "Choice") && choice.is_none() {
            choice = Some(child);
        } else if child.is(MC, "Fallback") && fallback.is_none() {
            fallback = Some(child);
        } else {
            return Ok(None);
        }
    }
    let Some(choice) = choice else {
        return Ok(None);
    };
    let inner = r.children(&choice)?;
    let [math] = inner.as_slice() else {
        return Ok(None);
    };
    // spec-audit: x:drawing/2010/main:<m>
    if !math.is(A14, "m") {
        return Ok(None);
    }
    Ok(Some(Shape {
        math: math.clone(),
        choice,
        fallback,
    }))
}

impl TextReader<'_, '_> {
    fn math_reader_run<T>(
        &mut self,
        read: impl FnOnce(&mut SchemaReader<'_, '_>) -> Res<T>,
    ) -> Result<Option<T>, ImportError> {
        let limits = self.r.limits.graphic;
        let spent = self.counts.math_elements;
        let mut c = SchemaReader::new(
            &mut *self.r,
            self.rels,
            &mut *self.color,
            &mut *self.counts,
            self.drawing,
            limits,
        );
        c.elements = spent;
        let out = read(&mut c);
        let spent = c.elements;
        self.counts.math_elements = spent;
        match out {
            Ok(v) => Ok(Some(v)),
            Err(GraphicError::Bound { .. }) => Ok(None),
            Err(GraphicError::Import(err)) => Err(err),
        }
    }

    /// `a14:m` in a paragraph.
    pub(crate) fn inline_math(&mut self, el: &Elem) -> Result<TextItem, ImportError> {
        let Some(math) = self.math_reader_run(|c| read_text_math(c, el))? else {
            self.r
                .note(K::Graphic(GraphicFinding::EquationOverLimit), el, None);
            return Ok(TextItem::Retained(el.local.clone()));
        };
        Ok(TextItem::Math(Box::new(InlineMath {
            math: Some(math),
            alternate: None,
        })))
    }

    /// `mc:AlternateContent` in a paragraph.
    pub(crate) fn inline_alternate(&mut self, el: &Elem) -> Result<TextItem, ImportError> {
        let Some(shape) = equation_shape(self.r, el)? else {
            self.r.retained(el);
            return Ok(TextItem::Retained(el.local.clone()));
        };
        let Some(math) = self.math_reader_run(|c| read_text_math(c, &shape.math))? else {
            self.r
                .note(K::Graphic(GraphicFinding::EquationOverLimit), el, None);
            return Ok(TextItem::Retained(el.local.clone()));
        };
        let requires = shape.choice.get("Requires").map(str::to_owned);
        let fallback = match &shape.fallback {
            None => None,
            Some(fallback) => {
                let mut items = Vec::new();
                for child in self.r.children(fallback)? {
                    items.push(self.fallback_item(&child)?);
                }
                Some(items)
            }
        };
        Ok(TextItem::Math(Box::new(InlineMath {
            math: Some(math),
            alternate: Some(MathAlternate { requires, fallback }),
        })))
    }

    fn fallback_item(&mut self, child: &Elem) -> Result<TextItem, ImportError> {
        let dml = child.in_namespace(ns::DML);
        Ok(match child.local.as_str() {
            "r" if dml => TextItem::Run(self.run(child)?),
            "br" if dml => TextItem::Break(self.line_break(child)?),
            "fld" if dml => TextItem::Field(self.field(child)?),
            other => {
                self.r.retained(child);
                TextItem::Retained(other.to_owned())
            }
        })
    }
}

/// Read a tree as a standalone OMML element: `m:oMath`, `m:oMathPara` or `m:mathPr`.
/// `None` if its root is none of them.
pub fn import_math_root(
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
