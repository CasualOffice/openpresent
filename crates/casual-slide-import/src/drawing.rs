//! Read what a shape's properties hold beyond its transform (Tier 1E), and the helpers
//! the fill, line, effect, geometry and 3D readers share.
//!
//! Four outcomes for what a file says, by the same rule as the rest of this crate. A
//! usable value goes in the model. An unusable one is reported and left out. An element
//! nothing here knows is reported and left in the tree. A list over a bound in
//! [`DrawingLimits`] leaves out the **whole property** it is in, as `CS-DRW-0001`: a
//! partial gradient would be a wrong gradient, and the tree still holds all of it, so
//! the deck opens, saves, and loses nothing.
//!
//! Reading the same element twice gives the same value, and these readers follow the
//! input only to [`DrawingLimits::max_effect_depth`]: fills hold blips hold effects hold
//! fills, which the schema does not bound.
//!
//! Evidence: `MS-OI29500` v20260818, section numbers on each use. Anything else about
//! Office's behaviour here is marked Inferred.

use std::collections::HashSet;

use casual_slide_model::color::{
    Angle, Color, Percentage, ValueDomain, parse_int, parse_percentage,
};
use casual_slide_model::drawing::{HiddenProperties, ShapeProperties};
use casual_slide_model::effect::EffectProperties;
use casual_slide_model::fill::{Blip, BlipFill, Fill};
use casual_slide_model::line::Line;
use casual_slide_model::presentation::{COORDINATE_MIN, Coordinate, POSITIVE_COORDINATE_MAX};
use casual_slide_model::properties::Lexical;
use casual_slide_ooxml::Relationship;

use crate::color::{
    Findings as ColorFindings, is_color_element, read_color_choice, read_optional_color,
};
use crate::properties::{Elem, ImportError, ImportFindingKind as K, Reader, ns};

/// `a14:`, Office 2010 drawing, then the namespace pre-release Office 2010 wrote for the
/// same elements: `fixtures/real/sdk-animation.pptx` holds `a14:useLocalDpi` in it, as
/// the released one is written. Only the first is ever written new.
pub const A14: &[&str] = &[
    "http://schemas.microsoft.com/office/drawing/2010/main",
    "http://schemas.microsoft.com/office/drawing/2007/7/7/main",
];
/// `ask:`, Office 2018 sketched lines.
pub const ASK: &[&str] = &["http://schemas.microsoft.com/office/drawing/2018/sketchyshapes"];

/// `ST_FixedAngle` is exclusive of this, on both sides.
const FIXED_ANGLE_LIMIT: i32 = 5_400_000;
/// The largest `ST_FOVAngle`.
const FOV_ANGLE_LIMIT: i32 = 10_800_000;

/// Bounds on one property of one shape. Over one, the property is left out and reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawingLimits {
    /// `gs` in one `gsLst`. PowerPoint's UI stops at ten.
    pub max_gradient_stops: usize,
    /// `ds` in one `custDash`.
    pub max_dash_stops: usize,
    /// Effects in one list, blip or container.
    pub max_effects: usize,
    /// Nesting of effects in containers, and of fills in effects in blips.
    pub max_effect_depth: usize,
    /// `imgEffect` in one `imgLayer`, and marks in one `backgroundRemoval`.
    pub max_picture_effects: usize,
    /// `gd` in one `avLst` or `gdLst`.
    pub max_guides: usize,
    /// `ahXY` and `ahPolar` in one `ahLst`.
    pub max_handles: usize,
    /// `cxn` in one `cxnLst`.
    pub max_sites: usize,
    /// `path` in one `pathLst`.
    pub max_paths: usize,
    /// Commands in one path.
    pub max_path_commands: usize,
    /// Elements read for one property, which bounds the walk whatever the shape of it.
    pub max_items_per_property: usize,
}

impl Default for DrawingLimits {
    fn default() -> Self {
        Self {
            max_gradient_stops: 1_024,
            max_dash_stops: 1_024,
            max_effects: 1_024,
            max_effect_depth: 32,
            max_picture_effects: 4_096,
            max_guides: 4_096,
            max_handles: 256,
            max_sites: 4_096,
            max_paths: 1_024,
            max_path_commands: 262_144,
            max_items_per_property: 1_000_000,
        }
    }
}

/// What a drawing property's reader noticed beyond the crate's common findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DrawingFinding {
    /// A list over a [`DrawingLimits`] bound, or over `casual-slide-geometry`'s. The
    /// whole property is left out of the model and stays in the tree.
    OverLimit,
    /// A dash stop of zero or less, which the schema allows and Office does not
    /// (`MS-OI29500` 2.1.1292). Kept.
    NonPositiveDashStop,
    /// `effectDag` beside a fill or `effectLst`, whose behaviour Office leaves
    /// unspecified (`MS-OI29500` 2.1.1295). Both kept.
    DagWithSiblings,
    /// Custom geometry `casual-slide-geometry`'s compile had to repair (an unknown
    /// operator or guide, too few operands), or an adjust naming none of the preset's.
    /// Kept as written.
    GeometryRepaired,
    /// A repeated container name, or an `effect/@ref` naming no container and none of
    /// the special references (`MS-OI29500` 2.1.1291, 2.1.1294). Kept.
    EffectReference,
    /// A percentage with more precision than the model holds was rounded in the model.
    /// The tree keeps the text.
    PercentRounded,
}

impl DrawingFinding {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::OverLimit => "CS-DRW-0001",
            Self::NonPositiveDashStop => "CS-DRW-0002",
            Self::DagWithSiblings => "CS-DRW-0003",
            Self::GeometryRepaired => "CS-DRW-0004",
            Self::EffectReference => "CS-DRW-0005",
            Self::PercentRounded => "CS-DRW-0006",
        }
    }
}

/// Why a reader stopped.
#[derive(Debug)]
pub(crate) enum Stop {
    /// A shared bound refused the part.
    Refused(ImportError),
    /// A list over a bound: the property is left out.
    Over,
}

impl From<ImportError> for Stop {
    fn from(err: ImportError) -> Self {
        Self::Refused(err)
    }
}

impl From<casual_slide_ooxml::OoxmlError> for Stop {
    fn from(err: casual_slide_ooxml::OoxmlError) -> Self {
        Self::Refused(ImportError::Ooxml(err))
    }
}

pub(crate) type Res<T> = Result<T, Stop>;

/// One property being read.
pub(crate) struct Ctx<'c, 'r> {
    pub r: &'c mut Reader<'r>,
    pub rels: &'c [Relationship],
    pub limits: DrawingLimits,
    pub color: ColorFindings,
    depth: usize,
    items: usize,
}

impl<'c, 'r> Ctx<'c, 'r> {
    pub fn new(r: &'c mut Reader<'r>, rels: &'c [Relationship], limits: DrawingLimits) -> Self {
        Self {
            r,
            rels,
            limits,
            color: ColorFindings::default(),
            depth: 0,
            items: 0,
        }
    }

    /// Run a reader over one whole property. Over a bound, `None` and a report.
    pub fn guard<T>(
        &mut self,
        el: &Elem,
        read: impl FnOnce(&mut Self) -> Res<T>,
    ) -> Result<Option<T>, ImportError> {
        self.depth = 0;
        self.items = 0;
        let out = match read(self) {
            Ok(value) => Some(value),
            Err(Stop::Over) => {
                self.r.note(K::Drawing(DrawingFinding::OverLimit), el, None);
                None
            }
            Err(Stop::Refused(err)) => return Err(err),
        };
        self.drain_color();
        Ok(out)
    }

    /// Carry what the colour reader noticed into this part's report, under the colour
    /// reader's own codes: a slide part reports in one list, and `CS-CLR` says the same
    /// thing there as on a theme part.
    fn drain_color(&mut self) {
        let kept = std::mem::take(&mut self.color);
        for f in kept.items {
            let attribute = (!f.detail.is_empty()).then_some(f.detail);
            self.r
                .findings
                .note(K::Color(f.kind), self.r.part, &f.element, attribute);
        }
    }

    /// One level deeper in a structure the schema does not bound.
    pub fn enter(&mut self) -> Res<()> {
        self.depth = self.depth.saturating_add(1);
        if self.depth > self.limits.max_effect_depth {
            return Err(Stop::Over);
        }
        Ok(())
    }

    pub fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// An element's children, charged to the property. Over `limit`, stop.
    pub fn kids(&mut self, el: &Elem, limit: usize) -> Res<Vec<Elem>> {
        let count = self.r.tree.child_elements(el.id).len();
        self.items = self.items.saturating_add(count);
        if count > limit || self.items > self.limits.max_items_per_property {
            return Err(Stop::Over);
        }
        Ok(self.r.children(el)?)
    }

    /// An element's children where the list is not bounded by the schema's shape: the
    /// property's budget is the bound.
    pub fn all_kids(&mut self, el: &Elem) -> Res<Vec<Elem>> {
        self.kids(el, self.limits.max_items_per_property)
    }

    /// Report children of an element that should have none.
    pub fn no_kids(&mut self, el: &Elem) -> Res<()> {
        for kid in self.all_kids(el)? {
            self.r.retained(&kid);
        }
        Ok(())
    }

    /// The children of a schema sequence, each reported if repeated. `read` says
    /// whether it took one; what it does not take is reported as retained.
    pub fn sequence(
        &mut self,
        el: &Elem,
        mut read: impl FnMut(&mut Self, &Elem) -> Res<bool>,
    ) -> Res<()> {
        let mut seen = HashSet::new();
        for kid in self.all_kids(el)? {
            if seen.contains(&kid.local) && kid.in_namespace(ns::DML) {
                self.r.note(K::DuplicateElement, &kid, None);
                continue;
            }
            if read(self, &kid)? {
                seen.insert(kid.local.clone());
            } else {
                self.r.retained(&kid);
            }
        }
        Ok(())
    }

    fn percent_text(&mut self, e: &mut Elem, name: &str, raw: &str) -> Option<Percentage> {
        let Some(read) = parse_percentage(raw) else {
            self.r.note(K::MalformedValue, e, Some(name));
            return None;
        };
        if read.rounded {
            self.r
                .note(K::Drawing(DrawingFinding::PercentRounded), e, Some(name));
        }
        Some(Percentage(read.value))
    }

    /// An attribute that is a percentage in this domain.
    pub fn pct(
        &mut self,
        e: &mut Elem,
        name: &str,
        domain: ValueDomain,
    ) -> Res<Option<Percentage>> {
        let Some(raw) = self.r.attr::<String>(e, name)? else {
            return Ok(None);
        };
        let Some(value) = self.percent_text(e, name, &raw) else {
            return Ok(None);
        };
        if domain.contains(value.0) {
            Ok(Some(value))
        } else {
            self.r.note(K::OutOfRange, e, Some(name));
            Ok(None)
        }
    }

    /// As [`Self::pct`], with absence reported.
    pub fn req_pct(
        &mut self,
        e: &mut Elem,
        name: &str,
        domain: ValueDomain,
    ) -> Res<Option<Percentage>> {
        if e.get(name).is_none() {
            self.r.note(K::MissingRequired, e, Some(name));
        }
        self.pct(e, name, domain)
    }

    /// A percentage in an inclusive range.
    pub fn pct_range(
        &mut self,
        e: &mut Elem,
        name: &str,
        range: std::ops::RangeInclusive<i32>,
    ) -> Res<Option<Percentage>> {
        let value = self.pct(e, name, ValueDomain::Percentage)?;
        Ok(value.filter(|v| {
            let ok = range.contains(&v.0);
            if !ok {
                self.r.note(K::OutOfRange, e, Some(name));
            }
            ok
        }))
    }

    fn angle_in(
        &mut self,
        e: &mut Elem,
        name: &str,
        allowed: impl Fn(i32) -> bool,
    ) -> Res<Option<Angle>> {
        let Some(raw) = self.r.attr::<String>(e, name)? else {
            return Ok(None);
        };
        let Some(value) = parse_int(&raw) else {
            self.r.note(K::MalformedValue, e, Some(name));
            return Ok(None);
        };
        if !allowed(value) {
            self.r.note(K::OutOfRange, e, Some(name));
            return Ok(None);
        }
        Ok(Some(Angle(value)))
    }

    /// `ST_PositiveFixedAngle`.
    pub fn pos_angle(&mut self, e: &mut Elem, name: &str) -> Res<Option<Angle>> {
        self.angle_in(e, name, |v| ValueDomain::PositiveFixedAngle.contains(v))
    }

    /// `ST_FixedAngle`.
    pub fn fixed_angle(&mut self, e: &mut Elem, name: &str) -> Res<Option<Angle>> {
        self.angle_in(e, name, |v| v > -FIXED_ANGLE_LIMIT && v < FIXED_ANGLE_LIMIT)
    }

    /// `ST_FOVAngle`.
    pub fn fov_angle(&mut self, e: &mut Elem, name: &str) -> Res<Option<Angle>> {
        self.angle_in(e, name, |v| (0..=FOV_ANGLE_LIMIT).contains(&v))
    }

    /// `ST_Coordinate`.
    pub fn coord(&mut self, e: &mut Elem, name: &str) -> Res<Option<Coordinate>> {
        Ok(self.r.attr_in::<Coordinate>(e, name, |c| {
            (COORDINATE_MIN..=POSITIVE_COORDINATE_MAX).contains(&c.0)
        })?)
    }

    /// `ST_Coordinate`, required.
    pub fn req_coord(&mut self, e: &mut Elem, name: &str) -> Res<Option<Coordinate>> {
        Ok(self.r.required::<Coordinate>(e, name, |c| {
            (COORDINATE_MIN..=POSITIVE_COORDINATE_MAX).contains(&c.0)
        })?)
    }

    /// `ST_PositiveCoordinate`, by the schema. `MS-OI29500` 2.1.1330: PowerPoint itself
    /// stops at 2147483647. A file may say more and keeps it, and the writer will not
    /// write a new value past that (`DRAW-01`).
    pub fn pos_coord(&mut self, e: &mut Elem, name: &str) -> Res<Option<Coordinate>> {
        Ok(self
            .r
            .attr_in::<Coordinate>(e, name, |c| (0..=POSITIVE_COORDINATE_MAX).contains(&c.0))?)
    }

    /// Any lexical value.
    pub fn attr<T: Lexical>(&mut self, e: &mut Elem, name: &str) -> Res<Option<T>> {
        Ok(self.r.attr(e, name)?)
    }

    /// Any lexical value, with absence reported.
    pub fn req<T: Lexical>(&mut self, e: &mut Elem, name: &str) -> Res<Option<T>> {
        Ok(self.r.required(e, name, |_| true)?)
    }

    /// A relationship attribute, checked against the part's relationships. Nothing it
    /// names is read or followed.
    pub fn rel(&mut self, e: &mut Elem, local: &'static str) -> Option<String> {
        let id = e.take_ns(ns::REL, local)?;
        if !id.is_empty() && !self.rels.iter().any(|r| r.id == id) {
            self.r.note(K::DanglingRelationship, e, Some(local));
        }
        Some(id)
    }

    /// The colour an element holds, its other children reported. `required`: absence is
    /// reported too.
    pub fn holder_color(&mut self, el: &Elem, required: bool) -> Res<Option<Color>> {
        let tree = self.r.tree;
        for kid in self.all_kids(el)? {
            if !is_color_element(tree, kid.id) {
                self.r.retained(&kid);
            }
        }
        Ok(if required {
            read_color_choice(tree, el.id, &mut self.color)
        } else {
            read_optional_color(tree, el.id, &mut self.color)
        })
    }

    /// The entries of an `extLst`, by what they hold and not by their `uri`: real decks
    /// spell one `uri` in several cases, and without braces (`fixtures/real`).
    pub fn extension_items(&mut self, el: &Elem) -> Res<Vec<Elem>> {
        self.r.finish(el);
        let mut items = Vec::new();
        for ext in self.all_kids(el)? {
            if !ext.is(ns::DML, "ext") {
                self.r.retained(&ext);
                continue;
            }
            items.extend(self.all_kids(&ext)?);
        }
        Ok(items)
    }

    /// One child of `spPr` or `grpSpPr` other than `xfrm`. `false` if it is not one this
    /// node's kind can hold. `seen` holds the choice groups already taken.
    pub fn property(
        &mut self,
        el: &Elem,
        props: &mut ShapeProperties,
        group: bool,
        seen: &mut HashSet<&'static str>,
    ) -> Result<bool, ImportError> {
        if !el.in_namespace(ns::DML) {
            return Ok(false);
        }
        let local = el.local.as_str();
        let slot = match local {
            "custGeom" | "prstGeom" if !group => "geometry",
            "ln" if !group => "ln",
            "sp3d" if !group => "sp3d",
            "effectLst" | "effectDag" => "effects",
            "scene3d" => "scene3d",
            "extLst" => "extLst",
            _ if is_fill_name(local) => "fill",
            _ => return Ok(false),
        };
        if !seen.insert(slot) {
            self.r.note(K::DuplicateElement, el, None);
            return Ok(true);
        }
        match slot {
            "geometry" => props.geometry = self.guard(el, |c| c.geometry(el))?,
            "fill" => props.fill = self.guard(el, |c| c.fill(el))?,
            "ln" => props.line = self.guard(el, |c| c.line(el))?.map(Box::new),
            "effects" => props.effects = self.guard(el, |c| c.effect_properties(el))?,
            "scene3d" => props.scene3d = self.guard(el, |c| c.scene3d(el))?.map(Box::new),
            "sp3d" => props.sp3d = self.guard(el, |c| c.sp3d(el))?.map(Box::new),
            _ => props.hidden = self.guard(el, |c| c.hidden(el))?.flatten().map(Box::new),
        }
        Ok(true)
    }

    // spec-audit: owner=shadowObscured
    fn hidden(&mut self, el: &Elem) -> Res<Option<HiddenProperties>> {
        let mut h = HiddenProperties::default();
        let mut any = false;
        let mut seen = HashSet::new();
        for item in self.extension_items(el)? {
            if !item.in_namespace(A14) {
                self.r.retained(&item);
                continue;
            }
            // spec-audit: x:drawing/2010/main:<hiddenFill>
            // spec-audit: x:drawing/2010/main:<hiddenLine>
            // spec-audit: x:drawing/2010/main:<hiddenEffects>
            // spec-audit: x:drawing/2010/main:<hiddenScene3d>
            // spec-audit: x:drawing/2010/main:<hiddenSp3d>
            // spec-audit: x:drawing/2010/main:<shadowObscured>
            let known = matches!(
                item.local.as_str(),
                "hiddenFill"
                    | "hiddenLine"
                    | "hiddenEffects"
                    | "hiddenScene3d"
                    | "hiddenSp3d"
                    | "shadowObscured"
            );
            if !known {
                self.r.retained(&item);
                continue;
            }
            if !seen.insert(item.local.clone()) {
                self.r.note(K::DuplicateElement, &item, None);
                continue;
            }
            any = true;
            match item.local.as_str() {
                "hiddenFill" => {
                    self.r.finish(&item);
                    for kid in self.all_kids(&item)? {
                        if is_fill_element(&kid) && h.fill.is_none() {
                            h.fill = Some(self.fill(&kid)?);
                        } else {
                            self.r.retained(&kid);
                        }
                    }
                }
                "hiddenLine" => h.line = Some(self.line(&item)?),
                "hiddenEffects" => {
                    self.r.finish(&item);
                    for kid in self.all_kids(&item)? {
                        let known = kid.in_namespace(ns::DML)
                            && matches!(kid.local.as_str(), "effectLst" | "effectDag");
                        if known && h.effects.is_none() {
                            h.effects = Some(self.effect_properties(&kid)?);
                        } else {
                            self.r.retained(&kid);
                        }
                    }
                }
                "hiddenScene3d" => h.scene3d = Some(self.scene3d(&item)?),
                "hiddenSp3d" => h.sp3d = Some(self.sp3d(&item)?),
                _ => {
                    let mut e = item.clone();
                    h.shadow_obscured = self.attr(&mut e, "val")?;
                    self.r.finish(&e);
                }
            }
        }
        Ok(any.then_some(h))
    }
}

/// Whether this local name is one of `EG_FillProperties`.
pub fn is_fill_name(local: &str) -> bool {
    matches!(
        local,
        "noFill" | "solidFill" | "gradFill" | "blipFill" | "pattFill" | "grpFill"
    )
}

/// Whether this element is one of `EG_FillProperties`.
pub fn is_fill_element(el: &Elem) -> bool {
    el.in_namespace(ns::DML) && is_fill_name(&el.local)
}

/// Read one `EG_FillProperties` element, for a reader outside the shape tree: a theme's
/// format scheme, a table cell, a run. `None`, reported, over a bound.
pub fn read_fill(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    limits: DrawingLimits,
    el: &Elem,
) -> Result<Option<Fill>, ImportError> {
    Ctx::new(r, rels, limits).guard(el, |c| c.fill(el))
}

/// Read one `a:CT_LineProperties` element, under any name. `None`, reported, over a bound.
pub fn read_line(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    limits: DrawingLimits,
    el: &Elem,
) -> Result<Option<Line>, ImportError> {
    Ctx::new(r, rels, limits).guard(el, |c| c.line(el))
}

/// Read `effectLst` or `effectDag`. `None`, reported, over a bound.
pub fn read_effects(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    limits: DrawingLimits,
    el: &Elem,
) -> Result<Option<EffectProperties>, ImportError> {
    Ctx::new(r, rels, limits).guard(el, |c| c.effect_properties(el))
}

/// Read a picture's `p:blipFill`, or any `a:CT_BlipFillProperties`.
pub fn read_blip_fill(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    limits: DrawingLimits,
    el: &Elem,
) -> Result<Option<BlipFill>, ImportError> {
    Ctx::new(r, rels, limits).guard(el, |c| c.blip_fill(el))
}

/// `a:CT_Blip` on its own, as a picture bullet holds it. `None`, reported, over a bound.
pub(crate) fn read_blip(
    r: &mut Reader<'_>,
    rels: &[Relationship],
    limits: DrawingLimits,
    el: &Elem,
) -> Result<Option<Blip>, ImportError> {
    Ctx::new(r, rels, limits).guard(el, |c| c.blip(el))
}

/// `MS-OI29500` 2.1.1295: what Office leaves unspecified beside an `effectDag`.
pub(crate) fn note_dag_siblings(r: &mut Reader<'_>, el: &Elem, props: &ShapeProperties) {
    if matches!(props.effects, Some(EffectProperties::Dag(_))) && props.fill.is_some() {
        r.note(K::Drawing(DrawingFinding::DagWithSiblings), el, None);
    }
}
