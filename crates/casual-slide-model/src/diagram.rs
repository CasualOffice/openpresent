//! SmartArt as plain typed values (`docs/spec/DGM.md`): the five linked parts of one
//! diagram, each typed in [`schema`].
//!
//! A diagram's `dsp:drawing` is the last layout PowerPoint made, and it is what is drawn
//! (`SEC-C12`). A layout definition is read as data and nothing here runs it: no
//! `forEach`, `choose` or constraint is evaluated, so a hostile definition costs its size
//! and no more. Point counts are bounded by `GraphicLimits` in the importer.
//!
//! A part that cannot be read is [`SetAside`] and stays in the package untouched, and the
//! rest of the diagram still reads.

pub mod schema;

use std::collections::BTreeSet;

use crate::drawing::ShapeProperties;
use crate::effect::EffectProperties;
use crate::fill::Fill;
use crate::line::Line;
use crate::properties::Lexical;
use crate::shape::{BlackWhiteMode, Transform};
use crate::text::{TextBody, TextItem};

/// An `xsd:list`: tokens separated by whitespace.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XmlList<T>(pub Vec<T>);

impl<T: Lexical> Lexical for XmlList<T> {
    fn parse_lexical(s: &str) -> Option<Self> {
        s.split_ascii_whitespace()
            .map(T::parse_lexical)
            .collect::<Option<Vec<_>>>()
            .map(Self)
    }
    fn to_lexical(&self) -> String {
        let tokens: Vec<String> = self.0.iter().map(Lexical::to_lexical).collect();
        tokens.join(" ")
    }
}

/// `a:CT_ShapeProperties` where a diagram holds one: Tier 1E's types.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShapeProps {
    /// `@bwMode`.
    pub bw_mode: Option<BlackWhiteMode>,
    /// `xfrm`.
    pub transform: Option<Transform>,
    /// Geometry, fill, line, effects and 3D.
    pub properties: ShapeProperties,
}

/// `a:CT_BackgroundFormatting` and `a:CT_WholeE2oFormatting`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Formatting {
    /// `EG_FillProperties`.
    pub fill: Option<Fill>,
    /// `ln`.
    pub line: Option<Line>,
    /// `EG_EffectProperties`.
    pub effects: Option<EffectProperties>,
}

/// `a:CT_NonVisualDrawingProps` in a diagram's drawing. Its hyperlinks and `extLst` stay
/// in the tree and are reported.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DrawingIdentity {
    /// `@id`.
    pub id: Option<u32>,
    /// `@name`.
    pub name: Option<String>,
    /// `@descr`.
    pub descr: Option<String>,
    /// `@hidden`.
    pub hidden: Option<bool>,
    /// `@title`.
    pub title: Option<String>,
}

/// `a:CT_NonVisualDrawingShapeProps`. Its locks stay in the tree and are reported.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NvShapeProps {
    /// `@txBox`.
    pub tx_box: Option<bool>,
}

/// `a:CT_NonVisualGroupDrawingShapeProps`. Its locks stay in the tree and are reported.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NvGroupProps {}

/// `extLst` in a diagram's parts, where the extension is one the schemas name. Another
/// extension stays in the tree and is reported.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Extensions {
    /// The known ones, in file order.
    pub items: Vec<Extension>,
}

/// `a:ext`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extension {
    /// `@uri`.
    pub uri: Option<String>,
    /// What it holds: `dsp:dataModelExt`, `dgm14:cNvPr` and so on.
    pub body: schema::Root,
}

/// Which of a diagram's five parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagramRole {
    /// `r:dm`: `dgm:dataModel`.
    Data,
    /// `r:lo`: `dgm:layoutDef`.
    Layout,
    /// `r:qs`: `dgm:styleDef`.
    Style,
    /// `r:cs`: `dgm:colorsDef`.
    Colors,
    /// `dsp:dataModelExt/@relId`: `dsp:drawing`.
    Drawing,
}

impl DiagramRole {
    /// All five, in the order parts are read.
    pub const ALL: [Self; 5] = [
        Self::Data,
        Self::Layout,
        Self::Style,
        Self::Colors,
        Self::Drawing,
    ];
}

/// A part that could not be read, left in the package as it was. `code` is the stable
/// diagnostic code that says why.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetAside {
    /// The part, or the relationship id if it names no part.
    pub part: String,
    /// `docs/20`.
    pub code: String,
}

/// One diagram: what a graphic frame's `dgm:relIds` reaches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagram {
    /// `dgm:dataModel`.
    pub data: Option<schema::DataModel>,
    /// `dgm:layoutDef`.
    pub layout: Option<schema::DiagramDefinition>,
    /// `dgm:styleDef`.
    pub style: Option<schema::StyleDefinition>,
    /// `dgm:colorsDef`.
    pub colors: Option<schema::ColorTransform>,
    /// `dsp:drawing`.
    pub drawing: Option<schema::DspDrawing>,
    /// Each part's name, by role, as the archive spells it.
    pub parts: Vec<(DiagramRole, String)>,
    /// What could not be read, by role.
    pub set_aside: Vec<(DiagramRole, SetAside)>,
}

/// What does not agree across a diagram's parts. Reported, and the deck opens; nothing
/// is repaired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Inconsistency {
    /// One of the five parts is not there.
    MissingPart(DiagramRole),
    /// Two points share a `@modelId`.
    DuplicatePoint,
    /// A connection names a point the data model does not hold.
    DanglingConnection,
    /// A `@presAssocID` names no point.
    DanglingPresentation,
    /// A drawing shape's `@modelId` names no point.
    DanglingShape,
    /// `prSet/@loTypeId`, `@qsTypeId` or `@csTypeId` differs from the part's `@uniqueId`.
    DefinitionMismatch(DiagramRole),
}

impl Diagram {
    /// A part's name, by role.
    pub fn part(&self, role: DiagramRole) -> Option<&str> {
        self.parts
            .iter()
            .find(|(r, _)| *r == role)
            .map(|(_, p)| p.as_str())
    }

    /// Whether a part was set aside.
    pub fn is_set_aside(&self, role: DiagramRole) -> bool {
        self.set_aside.iter().any(|(r, _)| *r == role)
    }

    /// The data model's points, if any.
    pub fn points(&self) -> &[schema::Pt] {
        self.data
            .as_ref()
            .and_then(|d| d.pt_lst.as_ref())
            .map_or(&[], |l| l.pt.as_slice())
    }

    /// The data model's connections, if any.
    pub fn connections(&self) -> &[schema::Cxn] {
        self.data
            .as_ref()
            .and_then(|d| d.cxn_lst.as_ref())
            .map_or(&[], |l| l.cxn.as_slice())
    }

    /// Every shape of the drawing, in document order.
    pub fn drawing_shapes(&self) -> Vec<&schema::DspShape> {
        let mut out = Vec::new();
        if let Some(tree) = self.drawing.as_ref().and_then(|d| d.sp_tree.as_deref()) {
            let mut todo = vec![tree];
            while let Some(group) = todo.pop() {
                for item in group.items.iter().rev() {
                    match item {
                        schema::DspGroupShapeItem::GrpSp(g) => todo.push(g),
                        schema::DspGroupShapeItem::Sp(s) => out.push(&**s),
                    }
                }
            }
        }
        out
    }

    /// What does not agree across the parts, each once, in a stable order. Linear in the
    /// parts' size. Nothing is run: no layout is evaluated to find these.
    pub fn inconsistencies(&self) -> BTreeSet<Inconsistency> {
        use Inconsistency as I;
        let mut found = BTreeSet::new();
        let held = [
            (DiagramRole::Data, self.data.is_some()),
            (DiagramRole::Layout, self.layout.is_some()),
            (DiagramRole::Style, self.style.is_some()),
            (DiagramRole::Colors, self.colors.is_some()),
            (DiagramRole::Drawing, self.drawing.is_some()),
        ];
        for (role, here) in held {
            if !here {
                found.insert(I::MissingPart(role));
            }
        }
        let mut ids = BTreeSet::new();
        for pt in self.points() {
            if let Some(id) = &pt.model_id
                && !ids.insert(norm_id(id))
            {
                found.insert(I::DuplicatePoint);
            }
        }
        let known = |id: &Option<String>| id.as_ref().is_none_or(|id| ids.contains(&norm_id(id)));
        for cxn in self.connections() {
            if !known(&cxn.src_id)
                || !known(&cxn.dest_id)
                || !known(&cxn.par_trans_id.as_ref().filter(|i| *i != "0").cloned())
                || !known(&cxn.sib_trans_id.as_ref().filter(|i| *i != "0").cloned())
            {
                found.insert(I::DanglingConnection);
            }
        }
        let differs = |id: &Option<String>, def: Option<&String>| {
            id.as_ref()
                .zip(def)
                .is_some_and(|(id, def)| !id.is_empty() && id != def)
        };
        for pt in self.points() {
            if let Some(set) = pt.pr_set.as_deref() {
                let parent = &set.pres_assoc_id;
                if parent.as_ref().is_some_and(|p| !p.is_empty()) && !known(parent) {
                    found.insert(I::DanglingPresentation);
                }
                if differs(
                    &set.lo_type_id,
                    self.layout.as_ref().and_then(|l| l.unique_id.as_ref()),
                ) {
                    found.insert(I::DefinitionMismatch(DiagramRole::Layout));
                }
                if differs(
                    &set.qs_type_id,
                    self.style.as_ref().and_then(|s| s.unique_id.as_ref()),
                ) {
                    found.insert(I::DefinitionMismatch(DiagramRole::Style));
                }
                if differs(
                    &set.cs_type_id,
                    self.colors.as_ref().and_then(|c| c.unique_id.as_ref()),
                ) {
                    found.insert(I::DefinitionMismatch(DiagramRole::Colors));
                }
            }
        }
        if !ids.is_empty() && self.drawing_shapes().iter().any(|s| !known(&s.model_id)) {
            found.insert(I::DanglingShape);
        }
        found
    }

    /// Put text in a point and in every drawing shape that shows it, so the data model and
    /// the drawing say the same thing. The first run of each body takes the text and keeps
    /// its properties; the other runs and paragraphs go. Geometry is not recomputed, which
    /// is the one thing a layout engine would add (`DGM-01`). `false` if no point has that
    /// id, or no shape shows it.
    pub fn set_point_text(&mut self, model_id: &str, text: &str) -> bool {
        let id = norm_id(model_id);
        let mut shown: BTreeSet<String> = BTreeSet::new();
        shown.insert(id.clone());
        for pt in self.points() {
            if let (Some(own), Some(set)) = (&pt.model_id, pt.pr_set.as_deref())
                && set.pres_assoc_id.as_deref().map(norm_id).as_ref() == Some(&id)
            {
                shown.insert(norm_id(own));
            }
        }
        let Some(list) = self.data.as_mut().and_then(|d| d.pt_lst.as_deref_mut()) else {
            return false;
        };
        let Some(point) = list
            .pt
            .iter_mut()
            .find(|p| p.model_id.as_deref().is_some_and(|m| norm_id(m) == id))
        else {
            return false;
        };
        let Some(body) = point.t.as_deref_mut() else {
            return false;
        };
        set_body_text(body, text);
        let Some(tree) = self.drawing.as_mut().and_then(|d| d.sp_tree.as_deref_mut()) else {
            return false;
        };
        let mut hit = false;
        let mut todo = vec![tree];
        while let Some(group) = todo.pop() {
            for item in &mut group.items {
                match item {
                    schema::DspGroupShapeItem::GrpSp(g) => todo.push(g),
                    schema::DspGroupShapeItem::Sp(s) => {
                        let mine = s
                            .model_id
                            .as_deref()
                            .is_some_and(|m| shown.contains(&norm_id(m)));
                        if mine && let Some(body) = s.tx_body.as_deref_mut() {
                            set_body_text(body, text);
                            hit = true;
                        }
                    }
                }
            }
        }
        hit
    }
}

/// A model id: a number, or a GUID in any case.
fn norm_id(id: &str) -> String {
    id.trim().to_ascii_lowercase()
}

fn set_body_text(body: &mut TextBody, text: &str) {
    body.paragraphs.truncate(1);
    let Some(paragraph) = body.paragraphs.first_mut() else {
        return;
    };
    let mut kept = None;
    for item in paragraph.items.drain(..) {
        if let TextItem::Run(run) = item {
            kept = Some(run);
            break;
        }
    }
    let mut run = kept.unwrap_or_default();
    run.text = Some(text.to_owned());
    paragraph.items = vec![TextItem::Run(run)];
}
