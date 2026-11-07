//! Read `a:CT_LineProperties` (`docs/spec/LINE.md`), and the sketch extension.

use casual_slide_model::color::{Percentage, ValueDomain};
use casual_slide_model::line::{
    DashStop, LINE_WIDTH_LIMIT, Line, LineDash, LineEnd, LineJoin, LineSketch, LineSketchType,
};

use crate::drawing::{ASK, Ctx, DrawingFinding, Res};
use crate::properties::{Elem, ImportFindingKind as K, ns};

impl Ctx<'_, '_> {
    /// `a:CT_LineProperties`, under any name.
    // spec-audit: owner=ln,hiddenLine,prstDash,miter
    pub(crate) fn line(&mut self, el: &Elem) -> Res<Line> {
        let mut e = el.clone();
        let mut l = Line {
            width: self
                .r
                .attr_in::<i32>(&mut e, "w", |w| (0..=LINE_WIDTH_LIMIT).contains(w))?,
            cap: self.attr(&mut e, "cap")?,
            compound: self.attr(&mut e, "cmpd")?,
            align: self.attr(&mut e, "algn")?,
            ..Line::default()
        };
        self.r.finish(&e);
        self.sequence(el, |c, kid| {
            if !kid.in_namespace(ns::DML) {
                return Ok(false);
            }
            let local = kid.local.as_str();
            match local {
                "noFill" | "solidFill" | "gradFill" | "pattFill" => {
                    if l.fill.is_some() {
                        c.r.note(K::DuplicateElement, kid, None);
                    } else {
                        l.fill = Some(c.fill(kid)?);
                    }
                }
                "prstDash" | "custDash" if l.dash.is_some() => {
                    c.r.note(K::DuplicateElement, kid, None);
                }
                "prstDash" => {
                    let mut k = kid.clone();
                    l.dash = Some(LineDash::Preset(c.req(&mut k, "val")?));
                    c.r.finish(&k);
                    c.no_kids(kid)?;
                }
                "custDash" => l.dash = Some(LineDash::Custom(c.dash_stops(kid)?)),
                "round" | "bevel" | "miter" if l.join.is_some() => {
                    c.r.note(K::DuplicateElement, kid, None);
                }
                "round" | "bevel" => {
                    c.r.finish(kid);
                    c.no_kids(kid)?;
                    l.join = Some(if local == "round" {
                        LineJoin::Round
                    } else {
                        LineJoin::Bevel
                    });
                }
                "miter" => {
                    let mut k = kid.clone();
                    let limit = c.pct(&mut k, "lim", ValueDomain::PositivePercentage)?;
                    c.r.finish(&k);
                    c.no_kids(kid)?;
                    l.join = Some(LineJoin::Miter(limit));
                }
                "headEnd" => l.head_end = Some(c.line_end(kid)?),
                "tailEnd" => l.tail_end = Some(c.line_end(kid)?),
                "extLst" => {
                    for item in c.extension_items(kid)? {
                        // spec-audit: x:drawing/2018/sketchyshapes:<lineSketchStyleProps>
                        if item.is(ASK, "lineSketchStyleProps") && l.sketch.is_none() {
                            l.sketch = Some(c.sketch(&item)?);
                        } else {
                            c.r.retained(&item);
                        }
                    }
                }
                _ => return Ok(false),
            }
            Ok(true)
        })?;
        Ok(l)
    }

    // spec-audit: owner=headEnd,tailEnd
    fn line_end(&mut self, el: &Elem) -> Res<LineEnd> {
        let mut e = el.clone();
        let end = LineEnd {
            end_type: self.attr(&mut e, "type")?,
            width: self.attr(&mut e, "w")?,
            length: self.attr(&mut e, "len")?,
        };
        self.r.finish(&e);
        self.no_kids(el)?;
        Ok(end)
    }

    // spec-audit: owner=custDash,ds
    fn dash_stops(&mut self, el: &Elem) -> Res<Vec<DashStop>> {
        self.r.finish(el);
        let mut stops = Vec::new();
        for kid in self.kids(el, self.limits.max_dash_stops)? {
            if !kid.is(ns::DML, "ds") {
                self.r.retained(&kid);
                continue;
            }
            let mut k = kid.clone();
            let stop = DashStop {
                dash: self.req_pct(&mut k, "d", ValueDomain::PositivePercentage)?,
                space: self.req_pct(&mut k, "sp", ValueDomain::PositivePercentage)?,
            };
            let bad = |v: Option<Percentage>| v.is_some_and(|p| p.0 <= 0);
            if bad(stop.dash) || bad(stop.space) {
                self.r
                    .note(K::Drawing(DrawingFinding::NonPositiveDashStop), &kid, None);
            }
            self.r.finish(&k);
            self.no_kids(&kid)?;
            stops.push(stop);
        }
        Ok(stops)
    }

    // spec-audit: owner=lineSketchStyleProps,type
    fn sketch(&mut self, el: &Elem) -> Res<LineSketch> {
        let mut e = el.clone();
        let mut s = LineSketch {
            seed_attribute: self.attr(&mut e, "sd")?,
            ..LineSketch::default()
        };
        self.r.finish(&e);
        for kid in self.all_kids(el)? {
            if kid.in_namespace(ns::DML) && matches!(kid.local.as_str(), "custGeom" | "prstGeom") {
                if s.geometry.is_some() {
                    self.r.note(K::DuplicateElement, &kid, None);
                } else {
                    s.geometry = Some(self.geometry(&kid)?);
                }
            } else if kid.is(ASK, "type") {
                if s.sketch_type.is_some() {
                    self.r.note(K::DuplicateElement, &kid, None);
                    continue;
                }
                self.r.finish(&kid);
                let mut choice = None;
                for t in self.all_kids(&kid)? {
                    let found = match t.local.as_str() {
                        "lineSketchNone" => Some(LineSketchType::None),
                        "lineSketchCurved" => Some(LineSketchType::Curved),
                        "lineSketchFreehand" => Some(LineSketchType::Freehand),
                        "lineSketchScribble" => Some(LineSketchType::Scribble),
                        _ => None,
                    };
                    match found.filter(|_| t.in_namespace(ASK)) {
                        Some(found) if choice.is_none() => choice = Some(found),
                        Some(_) => self.r.note(K::DuplicateElement, &t, None),
                        None => self.r.retained(&t),
                    }
                }
                s.sketch_type = Some(choice);
            } else if kid.is(ASK, "seed") {
                if s.seed.is_some() {
                    self.r.note(K::DuplicateElement, &kid, None);
                } else {
                    s.seed = self.r.typed_text(&kid)?;
                }
            } else {
                self.r.retained(&kid);
            }
        }
        Ok(s)
    }
}
