//! Read SmartArt (Tier 1K, `docs/spec/DGM.md`), and what the generated readers of
//! diagrams, equations and ink share.
//!
//! One diagram is five parts reached through a slide's relationships. Each is read on its
//! own: a part that cannot be read is set aside and reported, stays in the package as it
//! was, and the others still read. Nothing here refuses a deck.
//!
//! **A layout definition is data (`SEC-C12`).** It reads like any part, bounded by its
//! size and by [`GraphicLimits`], and nothing evaluates its `forEach`, `choose`, rules or
//! constraints. What is drawn is the stored `dsp:drawing`.
//!
//! Evidence: ECMA-376 Part 1 21.4 and `MS-ODRAWXML` 2.13 (`dsp`, `dgm14`), by the schemas in
//! `schemas/`. Which part a drawing relationship names follows `dsp:dataModelExt/@relId`.

pub mod schema;

use std::fmt;

use casual_slide_model::color::Color;
use casual_slide_model::d3::{Scene3D, Shape3D};
use casual_slide_model::diagram::schema as m;
use casual_slide_model::diagram::{
    Diagram, DiagramRole, DrawingIdentity, Extension, Extensions, Formatting, Inconsistency,
    NvGroupProps, NvShapeProps, SetAside, ShapeProps,
};
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode, Transform};
use casual_slide_model::slide::SlidePart;
use casual_slide_model::text::{CharacterProperties, Text3D, TextBody, TextListStyle};
use casual_slide_model::theme::ShapeStyle;
use casual_slide_ooxml::OoxmlError;
use casual_slide_ooxml::{
    PresentationPackage, Relationship, TargetMode, XmlTree, resolve_internal,
};

use crate::color::{Findings as ColorFindings, is_color_element, read_color};
use crate::drawing::{
    Ctx, DrawingLimits, is_fill_element, note_dag_siblings, read_effects, read_fill, read_line,
};
use crate::presentation::read_positive_size;
use crate::properties::{
    Elem, Findings, ImportError, ImportFindingKind as K, ImportLimits, Reader, ns,
};
use crate::shape::read_point;
use crate::text::{TextCounts, TextReader};
use crate::theme::read_shape_style;

/// What the generated readers use.
pub(crate) mod prelude {
    pub(crate) use super::{
        GraphicBound, Res, SchemaReader, is_color, is_text3d, read_character, read_color_item,
        read_ext_lst, read_formatting, read_identity, read_list_style, read_nv_group,
        read_nv_shape, read_scene3d, read_sp_pr, read_sp3d, read_style, read_text_body,
        read_text3d, read_transform,
    };
    pub(crate) use crate::properties::{Elem, ns};
    pub(crate) use casual_slide_model::properties::{Float64, Lexical};
}

/// The relationship type of a diagram's drawing part.
pub const DIAGRAM_DRAWING_RELATIONSHIP: &str =
    "http://schemas.microsoft.com/office/2007/relationships/diagramDrawing";

/// Bounds on diagrams, equations and ink. Over one, the part is set aside and reported;
/// in a slide's text, the equation stays in the tree and is reported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphicLimits {
    /// Elements nested below a part's root. Layout nodes and equations nest. `CS-GFX-0001`.
    pub max_depth: usize,
    /// Elements read in one part, or across a slide's equations. `CS-GFX-0002`.
    pub max_elements: usize,
    /// `dgm:pt` in one part: the point cap of `SEC-C12`. `CS-GFX-0003`.
    pub max_points: usize,
    /// `dgm:cxn` in one part. `CS-GFX-0004`.
    pub max_connections: usize,
    /// `dgm:layoutNode` in one part. `CS-GFX-0005`.
    pub max_layout_nodes: usize,
    /// `dsp:sp` in one part. `CS-GFX-0006`.
    pub max_drawing_shapes: usize,
    /// `inkml:trace` in one part. `CS-GFX-0007`.
    pub max_traces: usize,
}

impl Default for GraphicLimits {
    fn default() -> Self {
        Self {
            max_depth: 128,
            max_elements: 4_194_304,
            max_points: 65_536,
            max_connections: 131_072,
            max_layout_nodes: 65_536,
            max_drawing_shapes: 65_536,
            max_traces: 262_144,
        }
    }
}

/// Which bound was crossed. Each has its own stable code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicBound {
    /// Nesting below the root.
    Depth,
    /// Elements read.
    Elements,
    /// Points in a data model.
    Points,
    /// Connections in a data model.
    Connections,
    /// Layout nodes in a definition.
    LayoutNodes,
    /// Shapes in a drawing.
    DrawingShapes,
    /// Traces in an ink part.
    Traces,
}

impl GraphicBound {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::Depth => "CS-GFX-0001",
            Self::Elements => "CS-GFX-0002",
            Self::Points => "CS-GFX-0003",
            Self::Connections => "CS-GFX-0004",
            Self::LayoutNodes => "CS-GFX-0005",
            Self::DrawingShapes => "CS-GFX-0006",
            Self::Traces => "CS-GFX-0007",
        }
    }

    fn what(self) -> &'static str {
        match self {
            Self::Depth => "nesting",
            Self::Elements => "elements",
            Self::Points => "points",
            Self::Connections => "connections",
            Self::LayoutNodes => "layout nodes",
            Self::DrawingShapes => "drawing shapes",
            Self::Traces => "traces",
        }
    }

    fn limit(self, limits: &GraphicLimits) -> usize {
        match self {
            Self::Depth => limits.max_depth,
            Self::Elements => limits.max_elements,
            Self::Points => limits.max_points,
            Self::Connections => limits.max_connections,
            Self::LayoutNodes => limits.max_layout_nodes,
            Self::DrawingShapes => limits.max_drawing_shapes,
            Self::Traces => limits.max_traces,
        }
    }
}

/// Why a part of a diagram, an equation or an ink part could not be read.
#[derive(Debug)]
pub enum GraphicError {
    /// From the XML or another tier's reader.
    Import(ImportError),
    /// Over a bound in [`GraphicLimits`].
    Bound {
        /// Which.
        bound: GraphicBound,
        /// The part.
        part: String,
        /// The bound's value.
        limit: usize,
    },
}

impl GraphicError {
    /// The stable diagnostic code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Import(err) => err.code(),
            Self::Bound { bound, .. } => bound.code(),
        }
    }
}

impl fmt::Display for GraphicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Import(err) => write!(f, "{err}"),
            Self::Bound { bound, part, limit } => write!(
                f,
                "[{}] {part:?}: {} over the limit {limit}",
                bound.code(),
                bound.what()
            ),
        }
    }
}

impl std::error::Error for GraphicError {}

impl From<ImportError> for GraphicError {
    fn from(err: ImportError) -> Self {
        Self::Import(err)
    }
}

impl From<OoxmlError> for GraphicError {
    fn from(err: OoxmlError) -> Self {
        Self::Import(ImportError::from(err))
    }
}

pub(crate) type Res<T> = Result<T, GraphicError>;

/// What reading a diagram, an equation or ink noticed. The deck opens and the tree keeps
/// all of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GraphicFinding {
    /// A part that could not be read: left as it was, and out of the model.
    PartSetAside,
    /// Fewer of a diagram's five parts than it should have.
    MissingPart,
    /// Two points share a `@modelId`.
    DuplicatePoint,
    /// A connection names a point that is not there.
    DanglingConnection,
    /// A `@presAssocID` names a point that is not there.
    DanglingPresentation,
    /// A drawing shape names a point that is not there.
    DanglingShape,
    /// A point's definition id differs from the definition part's `@uniqueId`.
    DefinitionMismatch,
    /// An equation in a slide over a bound. It stays in the tree.
    EquationOverLimit,
}

impl GraphicFinding {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::PartSetAside => "CS-GFF-0001",
            Self::MissingPart => "CS-GFF-0002",
            Self::DuplicatePoint => "CS-GFF-0003",
            Self::DanglingConnection => "CS-GFF-0004",
            Self::DanglingPresentation => "CS-GFF-0005",
            Self::DanglingShape => "CS-GFF-0006",
            Self::DefinitionMismatch => "CS-GFF-0007",
            Self::EquationOverLimit => "CS-GFF-0008",
        }
    }
}

/// One part of a diagram, an equation or ink being read.
pub(crate) struct SchemaReader<'a, 'r> {
    pub r: &'a mut Reader<'r>,
    pub rels: &'a [Relationship],
    pub color: &'a mut ColorFindings,
    pub text_counts: &'a mut TextCounts,
    pub drawing: DrawingLimits,
    pub limits: GraphicLimits,
    pub depth: usize,
    pub elements: usize,
    points: usize,
    connections: usize,
    layout_nodes: usize,
    drawing_shapes: usize,
    traces: usize,
}

impl<'a, 'r> SchemaReader<'a, 'r> {
    pub(crate) fn new(
        r: &'a mut Reader<'r>,
        rels: &'a [Relationship],
        color: &'a mut ColorFindings,
        text_counts: &'a mut TextCounts,
        drawing: DrawingLimits,
        limits: GraphicLimits,
    ) -> Self {
        Self {
            r,
            rels,
            color,
            text_counts,
            drawing,
            limits,
            depth: 0,
            elements: 0,
            points: 0,
            connections: 0,
            layout_nodes: 0,
            drawing_shapes: 0,
            traces: 0,
        }
    }

    fn over(&self, bound: GraphicBound) -> GraphicError {
        GraphicError::Bound {
            bound,
            part: self.r.part.to_owned(),
            limit: bound.limit(&self.limits),
        }
    }

    /// One element deeper. A file cannot make this recurse past `max_depth`, nor read past
    /// `max_elements`.
    pub(crate) fn enter(&mut self) -> Res<()> {
        self.depth = self.depth.saturating_add(1);
        self.elements = self.elements.saturating_add(1);
        if self.depth > self.limits.max_depth {
            return Err(self.over(GraphicBound::Depth));
        }
        if self.elements > self.limits.max_elements {
            return Err(self.over(GraphicBound::Elements));
        }
        Ok(())
    }

    /// One element shallower.
    pub(crate) fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Spend one of a bound's own.
    pub(crate) fn charge(&mut self, bound: GraphicBound) -> Res<()> {
        let count = match bound {
            GraphicBound::Points => &mut self.points,
            GraphicBound::Connections => &mut self.connections,
            GraphicBound::LayoutNodes => &mut self.layout_nodes,
            GraphicBound::DrawingShapes => &mut self.drawing_shapes,
            GraphicBound::Traces => &mut self.traces,
            GraphicBound::Depth | GraphicBound::Elements => return Ok(()),
        };
        *count = count.saturating_add(1);
        if *count > bound.limit(&self.limits) {
            return Err(self.over(bound));
        }
        Ok(())
    }

    /// How this element spells a qualified attribute: `m:val`, under whatever prefix the part
    /// uses. If it has none, the name to report as missing.
    pub(crate) fn attr_name(&self, el: &Elem, uris: &[&str], local: &str) -> String {
        el.name_of_ns(uris, local).unwrap_or(local).to_owned()
    }

    pub(crate) fn attr<T: Lexical>(&mut self, el: &mut Elem, name: &str) -> Res<Option<T>> {
        Ok(self.r.attr(el, name)?)
    }

    pub(crate) fn attr_required<T: Lexical>(
        &mut self,
        el: &mut Elem,
        name: &str,
    ) -> Res<Option<T>> {
        Ok(self.r.required(el, name, |_| true)?)
    }

    pub(crate) fn attr_range<T: Lexical + PartialOrd>(
        &mut self,
        el: &mut Elem,
        name: &str,
        required: bool,
        range: impl std::ops::RangeBounds<T>,
    ) -> Res<Option<T>> {
        if required {
            Ok(self.r.required(el, name, |v| range.contains(v))?)
        } else {
            Ok(self.r.attr_in(el, name, |v| range.contains(v))?)
        }
    }

    /// An attribute of a union or a list: each token must be one the type admits.
    pub(crate) fn attr_member<T: Lexical>(
        &mut self,
        el: &mut Elem,
        name: &str,
        required: bool,
        member: fn(&str, &str) -> bool,
    ) -> Res<Option<T>> {
        let allowed = |value: &T| {
            let text = value.to_lexical();
            let mut tokens = text.split_ascii_whitespace().peekable();
            if tokens.peek().is_none() {
                return true;
            }
            let local = name.rsplit(':').next().unwrap_or(name);
            tokens.all(|t| member(local, t))
        };
        if required {
            Ok(self.r.required(el, name, allowed)?)
        } else {
            Ok(self.r.attr_in(el, name, allowed)?)
        }
    }

    /// An attribute in the relationships namespace, under whatever prefix the part uses.
    /// A dangling id is reported, and kept.
    pub(crate) fn rel_attr(&mut self, el: &mut Elem, local: &str) -> Option<String> {
        let value = el.take_ns(ns::REL, local)?;
        if !value.is_empty() && !self.rels.iter().any(|r| r.id == value) {
            self.r.note(K::DanglingRelationship, el, Some(local));
        }
        Some(value)
    }

    pub(crate) fn finish(&mut self, el: &Elem) {
        self.r.finish(el);
    }

    pub(crate) fn children(&self, el: &Elem) -> Res<Vec<Elem>> {
        Ok(self.r.children(el)?)
    }

    pub(crate) fn retained(&mut self, el: &Elem) {
        self.r.retained(el);
    }

    pub(crate) fn duplicate(&mut self, el: &Elem) {
        self.r.note(K::DuplicateElement, el, None);
    }

    pub(crate) fn text<T: Lexical>(&mut self, el: &Elem) -> Res<Option<T>> {
        Ok(self.r.typed_text(el)?)
    }

    pub(crate) fn text_reader(&mut self) -> TextReader<'_, 'r> {
        TextReader {
            r: &mut *self.r,
            rels: self.rels,
            color: &mut *self.color,
            counts: &mut *self.text_counts,
            drawing: self.drawing,
        }
    }

    fn ctx(&mut self) -> Ctx<'_, 'r> {
        Ctx::new(&mut *self.r, self.rels, self.drawing)
    }
}

use casual_slide_model::properties::Lexical;

/// `a:CT_ShapeProperties`, read by Tier 1E's readers.
pub(crate) fn read_sp_pr(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<ShapeProps> {
    c.enter()?;
    let mut e = el.clone();
    let mut out = ShapeProps {
        bw_mode: c.attr(&mut e, "bwMode")?,
        ..ShapeProps::default()
    };
    c.finish(&e);
    let mut slots = std::collections::HashSet::new();
    let mut seen = false;
    for child in c.children(&e)? {
        if child.is(ns::DML, "xfrm") {
            if seen {
                c.duplicate(&child);
            } else {
                seen = true;
                out.transform = Some(read_transform(c, &child)?);
            }
            continue;
        }
        if !c
            .ctx()
            .property(&child, &mut out.properties, false, &mut slots)?
        {
            c.retained(&child);
        }
    }
    note_dag_siblings(c.r, &e, &out.properties);
    c.leave();
    Ok(out)
}

/// `a:CT_Transform2D`, and a group's.
pub(crate) fn read_transform(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<Transform> {
    let mut e = el.clone();
    let mut t = Transform {
        rotation: c.attr(&mut e, "rot")?,
        flip_h: c.attr(&mut e, "flipH")?,
        flip_v: c.attr(&mut e, "flipV")?,
        ..Transform::default()
    };
    c.finish(&e);
    let mut seen = std::collections::HashSet::new();
    for child in c.children(&e)? {
        let name = child.local.as_str();
        if !child.in_namespace(ns::DML) || !matches!(name, "off" | "ext" | "chOff" | "chExt") {
            c.retained(&child);
            continue;
        }
        if !seen.insert(name.to_owned()) {
            c.duplicate(&child);
            continue;
        }
        match name {
            "off" => t.offset = read_point(c.r, &child)?,
            "ext" => t.extent = read_positive_size(c.r, &child)?,
            "chOff" => t.child_offset = read_point(c.r, &child)?,
            _ => t.child_extent = read_positive_size(c.r, &child)?,
        }
    }
    Ok(t)
}

pub(crate) fn read_text_body(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<TextBody> {
    Ok(c.text_reader().body(el)?)
}

pub(crate) fn read_list_style(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<TextListStyle> {
    Ok(c.text_reader().list_style(el)?)
}

pub(crate) fn read_character(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<CharacterProperties> {
    Ok(c.text_reader().character(el)?)
}

pub(crate) fn read_style(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<ShapeStyle> {
    Ok(read_shape_style(c.r.tree, el.id, &mut *c.color))
}

/// `a:CT_NonVisualDrawingProps`: identity only. Its hyperlinks and `extLst` are reported.
// spec-audit: owner=cNvPr
pub(crate) fn read_identity(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<DrawingIdentity> {
    let mut e = el.clone();
    let out = DrawingIdentity {
        id: c.attr(&mut e, "id")?,
        name: c.attr(&mut e, "name")?,
        descr: c.attr(&mut e, "descr")?,
        hidden: c.attr(&mut e, "hidden")?,
        title: c.attr(&mut e, "title")?,
    };
    c.finish(&e);
    for child in c.children(&e)? {
        c.retained(&child);
    }
    Ok(out)
}

// spec-audit: owner=cNvSpPr
pub(crate) fn read_nv_shape(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<NvShapeProps> {
    let mut e = el.clone();
    let out = NvShapeProps {
        tx_box: c.attr(&mut e, "txBox")?,
    };
    c.finish(&e);
    for child in c.children(&e)? {
        c.retained(&child);
    }
    Ok(out)
}

pub(crate) fn read_nv_group(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<NvGroupProps> {
    c.finish(el);
    for child in c.children(el)? {
        c.retained(&child);
    }
    Ok(NvGroupProps {})
}

pub(crate) fn read_scene3d(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<Scene3D> {
    let read = c.ctx().guard(el, |x| x.scene3d(el))?;
    Ok(read.unwrap_or_default())
}

pub(crate) fn read_sp3d(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<Shape3D> {
    let read = c.ctx().guard(el, |x| x.sp3d(el))?;
    Ok(read.unwrap_or_default())
}

pub(crate) fn is_text3d(_: &SchemaReader<'_, '_>, el: &Elem) -> bool {
    el.in_namespace(ns::DML) && matches!(el.local.as_str(), "sp3d" | "flatTx")
}

pub(crate) fn read_text3d(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<Option<Text3D>> {
    if el.local == "sp3d" {
        let read = c.ctx().guard(el, |x| x.sp3d(el))?;
        Ok(read.map(|s| Text3D::Shape(Box::new(s))))
    } else {
        let read = c.ctx().guard(el, |x| x.flat_text(el))?;
        Ok(read.map(Text3D::Flat))
    }
}

pub(crate) fn is_color(c: &SchemaReader<'_, '_>, el: &Elem) -> bool {
    is_color_element(c.r.tree, el.id)
}

pub(crate) fn read_color_item(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<Option<Color>> {
    c.enter()?;
    let out = read_color(c.r.tree, el.id, &mut *c.color);
    c.leave();
    Ok(out)
}

/// `a:CT_BackgroundFormatting`, `a:CT_WholeE2oFormatting`.
pub(crate) fn read_formatting(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<Formatting> {
    c.finish(el);
    let mut out = Formatting::default();
    for child in c.children(el)? {
        if is_fill_element(&child) && out.fill.is_none() {
            out.fill = read_fill(c.r, c.rels, c.drawing, &child)?;
        } else if child.is(ns::DML, "ln") && out.line.is_none() {
            out.line = read_line(c.r, c.rels, c.drawing, &child)?;
        } else if child.in_namespace(ns::DML)
            && matches!(child.local.as_str(), "effectLst" | "effectDag")
            && out.effects.is_none()
        {
            out.effects = read_effects(c.r, c.rels, c.drawing, &child)?;
        } else {
            c.retained(&child);
        }
    }
    Ok(out)
}

/// An `extLst`: an extension the schemas of this tier name is read as itself, and any
/// other stays in the tree and is reported.
pub(crate) fn read_ext_lst(c: &mut SchemaReader<'_, '_>, el: &Elem) -> Res<Extensions> {
    c.enter()?;
    c.finish(el);
    let mut out = Extensions::default();
    for ext in c.children(el)? {
        if !ext.is(ns::DML, "ext") {
            c.retained(&ext);
            continue;
        }
        let mut e = ext.clone();
        let uri: Option<String> = c.attr(&mut e, "uri")?;
        c.finish(&e);
        let mut known = false;
        for inner in c.children(&e)? {
            if !known && schema::is_root(&inner) {
                if let Some(body) = schema::read_root(c, &inner)? {
                    out.items.push(Extension {
                        uri: uri.clone(),
                        body,
                    });
                    known = true;
                    continue;
                }
            }
            c.retained(&inner);
        }
    }
    c.leave();
    Ok(out)
}

/// Read a tree as a standalone element of `diagram`. `None` if its root is none of them.
pub fn import_diagram_root(
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

/// The part a relationship of `source` names, as the archive spells it.
pub(crate) fn resolve_rel(package: &PresentationPackage, source: &str, id: &str) -> Option<String> {
    let rel = package
        .relationships_of(source)
        .iter()
        .find(|r| r.id == id)?;
    if rel.mode != TargetMode::Internal {
        return None;
    }
    let target = resolve_internal(source, &rel.target)?;
    let key = casual_slide_ooxml::part_name_key(&target)?;
    package
        .part_names()
        .into_iter()
        .find(|n| casual_slide_ooxml::part_name_key(n).as_deref() == Some(&key))
}

/// Read a part and parse it: a fault is what the caller sets aside.
pub(crate) fn read_tree(
    package: &mut PresentationPackage,
    part: &str,
) -> Result<XmlTree, GraphicError> {
    let limits = *package.limits();
    let bytes = package.read_xml_part(part)?;
    Ok(XmlTree::parse(&bytes, part, &limits)?)
}

fn set_aside(
    diagram: &mut Diagram,
    role: DiagramRole,
    part: &str,
    code: &str,
    findings: &mut Findings,
) {
    findings.note(
        K::Graphic(GraphicFinding::PartSetAside),
        part,
        part_root(role),
        None,
    );
    diagram.set_aside.push((
        role,
        SetAside {
            part: part.to_owned(),
            code: code.to_owned(),
        },
    ));
}

fn part_root(role: DiagramRole) -> &'static str {
    match role {
        DiagramRole::Data => "dataModel",
        DiagramRole::Layout => "layoutDef",
        DiagramRole::Style => "styleDef",
        DiagramRole::Colors => "colorsDef",
        DiagramRole::Drawing => "drawing",
    }
}

/// Read a diagram from the slide-like part that holds its frame. This never refuses:
/// what cannot be read is set aside and reported, and the deck opens.
pub fn import_diagram(
    package: &mut PresentationPackage,
    slide_part: &str,
    rel_ids: &m::RelIds,
    limits: &GraphicLimits,
    import_limits: &ImportLimits,
    findings: &mut Findings,
) -> Diagram {
    let mut diagram = Diagram::default();
    let links = [
        (DiagramRole::Data, rel_ids.dm.as_deref()),
        (DiagramRole::Layout, rel_ids.lo.as_deref()),
        (DiagramRole::Style, rel_ids.qs.as_deref()),
        (DiagramRole::Colors, rel_ids.cs.as_deref()),
    ];
    for (role, id) in links {
        read_role(
            package,
            slide_part,
            role,
            id,
            limits,
            import_limits,
            findings,
            &mut diagram,
        );
    }
    // The drawing is named by the data model, by a relationship of the slide's.
    let drawing_rel = diagram
        .data
        .as_ref()
        .and_then(|d| d.ext_lst.as_deref())
        .and_then(|l| {
            l.items.iter().find_map(|e| match &e.body {
                m::Root::DspDataModelExt(x) => x.rel_id.clone(),
                _ => None,
            })
        });
    read_role(
        package,
        slide_part,
        DiagramRole::Drawing,
        drawing_rel.as_deref(),
        limits,
        import_limits,
        findings,
        &mut diagram,
    );
    let at = diagram
        .part(DiagramRole::Data)
        .unwrap_or(slide_part)
        .to_owned();
    for fault in diagram.inconsistencies() {
        let (kind, element) = match fault {
            Inconsistency::MissingPart(role) => {
                // A part already set aside has said why.
                if diagram.is_set_aside(role) {
                    continue;
                }
                (GraphicFinding::MissingPart, part_root(role))
            }
            Inconsistency::DuplicatePoint => (GraphicFinding::DuplicatePoint, "pt"),
            Inconsistency::DanglingConnection => (GraphicFinding::DanglingConnection, "cxn"),
            Inconsistency::DanglingPresentation => (GraphicFinding::DanglingPresentation, "prSet"),
            Inconsistency::DanglingShape => (GraphicFinding::DanglingShape, "sp"),
            Inconsistency::DefinitionMismatch(_) => (GraphicFinding::DefinitionMismatch, "prSet"),
        };
        findings.note(K::Graphic(kind), &at, element, None);
    }
    diagram
}

#[allow(clippy::too_many_arguments)] // One call site; the bounds and the report are distinct.
fn read_role(
    package: &mut PresentationPackage,
    slide_part: &str,
    role: DiagramRole,
    id: Option<&str>,
    limits: &GraphicLimits,
    import_limits: &ImportLimits,
    findings: &mut Findings,
    diagram: &mut Diagram,
) {
    let Some(id) = id else {
        return;
    };
    let Some(part) = resolve_rel(package, slide_part, id) else {
        findings.note(
            K::DanglingRelationship,
            slide_part,
            part_root(role),
            Some("r:id"),
        );
        return;
    };
    diagram.parts.push((role, part.clone()));
    let tree = match read_tree(package, &part) {
        Ok(tree) => tree,
        Err(err) => return set_aside(diagram, role, &part, err.code(), findings),
    };
    let rels = package.relationships_of(&part).to_vec();
    let read = import_diagram_root(&tree, &part, &rels, limits, import_limits, findings);
    let root = match read {
        Ok(Some(root)) => root,
        Ok(None) => return set_aside(diagram, role, &part, "CS-IMP-0003", findings),
        Err(err) => return set_aside(diagram, role, &part, err.code(), findings),
    };
    match (role, root) {
        (DiagramRole::Data, m::Root::DataModel(v)) => diagram.data = Some(*v),
        (DiagramRole::Layout, m::Root::LayoutDef(v)) => diagram.layout = Some(*v),
        (DiagramRole::Style, m::Root::StyleDef(v)) => diagram.style = Some(*v),
        (DiagramRole::Colors, m::Root::ColorsDef(v)) => diagram.colors = Some(*v),
        (DiagramRole::Drawing, m::Root::DspDrawing(v)) => diagram.drawing = Some(*v),
        // A part of the wrong kind where the relationship says another.
        _ => set_aside(diagram, role, &part, "CS-IMP-0003", findings),
    }
}

/// Every diagram of a slide-like part, by the frame that holds it.
pub fn import_slide_diagrams(
    package: &mut PresentationPackage,
    slide_part: &str,
    part: &SlidePart,
    limits: &GraphicLimits,
    import_limits: &ImportLimits,
    findings: &mut Findings,
) -> Vec<(ObjectId, Diagram)> {
    let mut out = Vec::new();
    for node in part
        .shape_tree()
        .into_iter()
        .flat_map(ShapeNode::descendants)
    {
        if node.kind != NodeKind::GraphicFrame {
            continue;
        }
        if let Some(ids) = node.diagram.as_deref() {
            out.push((
                node.id,
                import_diagram(package, slide_part, ids, limits, import_limits, findings),
            ));
        }
    }
    out
}
