//! Patch a shape's geometry, fill, line, effects and 3D into its tree (ADR-013, Tier 1E).
//!
//! The model is turned into a small description of the elements it stands for, before
//! and after, and only what differs between the two descriptions touches the tree. So
//! an unedited property changes nothing, spelling included; an edited one changes its
//! own attributes and children; unknown attributes, foreign children and other `ext`
//! entries stay where they are. Replacing a choice (a solid fill by a gradient) replaces
//! that element, and whatever was in it went with the edit that named it.
//!
//! Range is the reader's: `patch_slide_part` reads every patched part back and refuses
//! the save if it does not read as asked, so a value the reader would not believe is
//! refused by the reader's own rule, as `CS-EXP-0007`. One check is the writer's own
//! (`DRAW-01`): a file may hold more than PowerPoint accepts, and keeps it, but a new
//! `ST_PositiveCoordinate` past 2147483647 (`MS-OI29500` 2.1.1330) or a new dash stop of
//! zero or less (2.1.1292) is refused as `CS-EXP-0002`, with nothing saved.

use std::collections::{HashMap, HashSet, VecDeque};

use casual_slide_import::drawing::{A14, ASK};
use casual_slide_import::media::{AIF, ASVG, P14, P173, WOE};
use casual_slide_import::properties::{Elem, ns};
use casual_slide_import::timing::MC;
use casual_slide_import::transition::{P15, P159};
use casual_slide_model::color::{Angle, Color, ColorValue, Percentage, format_percentage};
use casual_slide_model::drawing::{HiddenProperties, ShapeProperties};
use casual_slide_model::fill::{Blip, Fill};
use casual_slide_model::presentation::Coordinate;
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::NodeKind;
use casual_slide_model::theme::{FontRef, ShapeStyle, StyleMatrixRef};

use crate::effect::effect_properties_node;
use crate::fill::{blip_fill_node, fill_node};
use crate::geometry::geometry_node;
use crate::line::line_node;
use crate::properties::{ExportError, Writer};

/// Which namespace an element is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Space {
    Dml,
    Pml,
    A14,
    Ask,
    Asvg,
    Woe,
    Aif,
    P14,
    P173,
    Mc,
    P15,
    P159,
    /// A namespace this file does not need by name: its URIs and its usual prefix.
    Other(&'static [&'static str], &'static str),
}

const SPACES: [Space; 24] = [
    Space::Dml,
    Space::Pml,
    Space::A14,
    Space::Ask,
    Space::Asvg,
    Space::Woe,
    Space::Aif,
    Space::P14,
    Space::P173,
    Space::Mc,
    Space::P15,
    Space::P159,
    crate::comment::P188,
    crate::comment::P223,
    crate::comment::P228,
    crate::comment::P166,
    crate::comment::PSLZ,
    crate::comment::PSEZ,
    crate::comment::PSUZ,
    crate::comment::P16,
    crate::comment::P184,
    crate::comment::ACS,
    crate::comment::P202,
    crate::comment::CLBL,
];

impl Space {
    pub(crate) fn uris(self) -> &'static [&'static str] {
        match self {
            Self::Dml => ns::DML,
            Self::Pml => ns::PML,
            Self::A14 => A14,
            Self::Ask => ASK,
            Self::Asvg => ASVG,
            Self::Woe => WOE,
            Self::Aif => AIF,
            Self::P14 => P14,
            Self::P173 => P173,
            Self::Mc => MC,
            Self::P15 => P15,
            Self::P159 => P159,
            Self::Other(uris, _) => uris,
        }
    }

    pub(crate) fn prefix(self) -> &'static str {
        match self {
            Self::Dml => "a",
            Self::Pml => "p",
            Self::A14 => "a14",
            Self::Ask => "ask",
            Self::Asvg => "asvg",
            Self::Woe => "woe",
            Self::Aif => "aif",
            Self::P14 => "p14",
            Self::P173 => "p173",
            Self::Mc => "mc",
            Self::P15 => "p15",
            Self::P159 => "p159",
            Self::Other(_, prefix) => prefix,
        }
    }
}

/// An attribute's value.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    /// As `Lexical` writes it.
    Text(String),
    /// A percentage, whose spelling follows what it replaces or the conformance class.
    Percent(i32),
    /// A relationship id: the attribute is in the relationships namespace.
    Rel(String),
    /// An attribute in `p14`, under whatever prefix the part uses.
    P14(String),
    /// `ST_PositiveCoordinate`, which PowerPoint stops short of the schema's maximum of.
    PositiveCoordinate(i64),
    /// A dash stop, which PowerPoint wants above zero.
    DashStop(i32),
}

/// The largest `ST_PositiveCoordinate` PowerPoint accepts (`MS-OI29500` 2.1.1330).
pub(crate) const POWERPOINT_COORDINATE_MAX: i64 = 2_147_483_647;

/// An element, as the model says it should be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Node {
    space: Space,
    pub(crate) local: &'static str,
    attrs: Vec<(&'static str, Value)>,
    kids: Vec<Node>,
    text: Option<String>,
    /// What this element's `extLst` holds, by item, each in an `ext` of its own.
    exts: Vec<Node>,
    /// The `uri` of the `ext` to hold this, where only the file can say.
    ext_uri: Option<String>,
}

impl Node {
    pub fn new(space: Space, local: &'static str) -> Self {
        Self {
            space,
            local,
            attrs: Vec::new(),
            kids: Vec::new(),
            text: None,
            exts: Vec::new(),
            ext_uri: None,
        }
    }

    /// The `uri` of the `ext` to hold this, where [`extension_uri`] has none.
    pub fn ext_uri(mut self, uri: Option<&String>) -> Self {
        self.ext_uri = uri.cloned();
        self
    }

    /// An extension's item, named with its conventional prefix.
    pub fn extension(qualified: &'static str) -> Self {
        match qualified.split_once(':') {
            Some(("asvg", local)) => Self::new(Space::Asvg, local),
            Some(("woe", local)) => Self::new(Space::Woe, local),
            Some(("aif", local)) => Self::new(Space::Aif, local),
            Some((_, local)) => Self::new(Space::P173, local),
            None => Self::a(qualified),
        }
    }

    /// In DrawingML.
    pub fn a(local: &'static str) -> Self {
        Self::new(Space::Dml, local)
    }

    pub fn lex<T: Lexical>(mut self, name: &'static str, value: Option<&T>) -> Self {
        if let Some(value) = value {
            self.attrs.push((name, Value::Text(value.to_lexical())));
        }
        self
    }

    /// An `ST_PositiveCoordinate`.
    pub fn pos(mut self, name: &'static str, value: Option<&Coordinate>) -> Self {
        if let Some(value) = value {
            self.attrs.push((name, Value::PositiveCoordinate(value.0)));
        }
        self
    }

    /// A dash stop's length.
    pub fn dash(mut self, name: &'static str, value: Option<Percentage>) -> Self {
        if let Some(value) = value {
            self.attrs.push((name, Value::DashStop(value.0)));
        }
        self
    }

    pub fn pct(mut self, name: &'static str, value: Option<Percentage>) -> Self {
        if let Some(value) = value {
            self.attrs.push((name, Value::Percent(value.0)));
        }
        self
    }

    pub fn ang(mut self, name: &'static str, value: Option<Angle>) -> Self {
        if let Some(value) = value {
            self.attrs.push((name, Value::Text(value.0.to_string())));
        }
        self
    }

    pub fn rel(mut self, name: &'static str, value: Option<&String>) -> Self {
        if let Some(value) = value {
            self.attrs.push((name, Value::Rel(value.clone())));
        }
        self
    }

    /// An attribute in the 2010 namespace.
    pub fn p14(mut self, name: &'static str, value: Option<&String>) -> Self {
        if let Some(value) = value {
            self.attrs.push((name, Value::P14(value.clone())));
        }
        self
    }

    pub fn kid(mut self, kid: Node) -> Self {
        self.kids.push(kid);
        self
    }

    pub fn opt(mut self, kid: Option<Node>) -> Self {
        self.kids.extend(kid);
        self
    }

    pub fn list(mut self, kids: impl IntoIterator<Item = Node>) -> Self {
        self.kids.extend(kids);
        self
    }

    pub fn color(self, color: Option<&Color>) -> Self {
        self.opt(color.map(color_node))
    }

    pub fn text(mut self, text: Option<String>) -> Self {
        self.text = text;
        self
    }

    pub fn ext(mut self, item: Option<Node>) -> Self {
        self.exts.extend(item);
        self
    }

    /// Whether this element holds this attribute.
    fn value(&self, name: &str) -> Option<&Value> {
        self.attrs.iter().find(|(n, _)| *n == name).map(|(_, v)| v)
    }

    /// The element's local name.
    pub fn local(&self) -> &'static str {
        self.local
    }

    pub(crate) fn same_name(&self, other: &Node) -> bool {
        self.local == other.local && self.space == other.space
    }

    fn is(&self, el: &Elem) -> bool {
        el.is(self.space.uris(), self.local)
    }
}

/// A colour and its transforms.
pub(crate) fn color_node(color: &Color) -> Node {
    let node = Node::a(color.value.element_name());
    let node = match color.value {
        ColorValue::ScRgb { r, g, b } => node.pct("r", Some(r)).pct("g", Some(g)).pct("b", Some(b)),
        ColorValue::Srgb(rgb) => node.lex("val", Some(&rgb.to_hex())),
        ColorValue::Hsl { hue, sat, lum } => node
            .ang("hue", Some(hue))
            .pct("sat", Some(sat))
            .pct("lum", Some(lum)),
        ColorValue::System { val, last } => node
            .lex("val", Some(&val.as_xml().to_owned()))
            .lex("lastClr", last.map(|l| l.to_hex()).as_ref()),
        ColorValue::Scheme(s) => node.lex("val", Some(&s.as_xml().to_owned())),
        ColorValue::Preset(p) => node.lex("val", Some(&p.as_xml().to_owned())),
    };
    node.list(color.transforms.iter().map(|t| {
        let domain = t.kind.domain();
        let kid = Node::a(t.kind.as_xml());
        if domain == casual_slide_model::color::ValueDomain::None {
            kid
        } else if domain.is_percentage() {
            kid.pct("val", Some(Percentage(t.value)))
        } else {
            kid.ang("val", Some(Angle(t.value)))
        }
    }))
}

/// Children of which one may be present, so changing which one replaces it.
const CHOICES: &[&[&str]] = &[
    &[
        "scrgbClr",
        "srgbClr",
        "hslClr",
        "sysClr",
        "schemeClr",
        "prstClr",
    ],
    &[
        "noFill",
        "solidFill",
        "gradFill",
        "blipFill",
        "pattFill",
        "grpFill",
    ],
    &["custGeom", "prstGeom"],
    &["effectLst", "effectDag"],
    &["prstDash", "custDash"],
    &["round", "bevel", "miter"],
    &["tile", "stretch"],
    &[
        "audioCd",
        "wavAudioFile",
        "audioFile",
        "videoFile",
        "quickTimeFile",
    ],
];

/// `path` means a shade only in a gradient; in `pathLst` it is one of a list.
const SHADES: &[&str] = &["lin", "path"];

fn choices_of(parent: &str) -> impl Iterator<Item = &'static [&'static str]> {
    let shade = (parent == "gradFill").then_some(SHADES);
    CHOICES
        .iter()
        .copied()
        .chain(shade)
        .chain(crate::table::choices_in(parent).iter().copied())
}

/// Where a child goes among its siblings: the schema's sequence for the parent. `None`
/// for what the parent lists no order for, which goes at the end.
fn rank(parent: &str, local: &str) -> Option<usize> {
    let order: &[&str] = match parent {
        "spPr" | "grpSpPr" => &[
            "xfrm",
            "custGeom",
            "prstGeom",
            "noFill",
            "solidFill",
            "gradFill",
            "blipFill",
            "pattFill",
            "grpFill",
            "ln",
            "effectLst",
            "effectDag",
            "scene3d",
            "sp3d",
            "extLst",
        ],
        "pic" => &["nvPicPr", "blipFill", "spPr", "style", "extLst"],
        "fmtScheme" => &[
            "fillStyleLst",
            "lnStyleLst",
            "effectStyleLst",
            "bgFillStyleLst",
        ],
        "effectStyle" => &["effectLst", "effectDag", "scene3d", "sp3d"],
        "nvPr" => &[
            "ph",
            "audioCd",
            "wavAudioFile",
            "audioFile",
            "videoFile",
            "quickTimeFile",
            "custDataLst",
            "extLst",
        ],
        "audioCd" => &["st", "end", "extLst"],
        "tracksInfo" => &["trackLst"],
        "trackLst" => &["track"],
        "media" => &["trim", "fade", "bmkLst", "extLst"],
        "hlinkClick" | "hlinkHover" | "hlinkMouseOver" => &["snd", "extLst"],
        "gradFill" => &["gsLst", "lin", "path", "tileRect"],
        "blipFill" => &["blip", "srcRect", "tile", "stretch"],
        "pattFill" => &["fgClr", "bgClr"],
        "style" => &["lnRef", "fillRef", "effectRef", "fontRef"],
        "clrChange" => &["clrFrom", "clrTo"],
        "ln" | "hiddenLine" | "uLn" | "lnL" | "lnR" | "lnT" | "lnB" | "lnTlToBr" | "lnBlToTr" => &[
            "noFill",
            "solidFill",
            "gradFill",
            "pattFill",
            "prstDash",
            "custDash",
            "round",
            "bevel",
            "miter",
            "headEnd",
            "tailEnd",
            "extLst",
        ],
        "custGeom" => &["avLst", "gdLst", "ahLst", "cxnLst", "rect", "pathLst"],
        "scene3d" | "hiddenScene3d" => &["camera", "lightRig", "backdrop", "extLst"],
        "backdrop" => &["anchor", "norm", "up", "extLst"],
        "sp3d" | "hiddenSp3d" => &["bevelT", "bevelB", "extrusionClr", "contourClr", "extLst"],
        "effectLst" => &[
            "blur",
            "fillOverlay",
            "glow",
            "innerShdw",
            "outerShdw",
            "prstShdw",
            "reflection",
            "softEdge",
        ],
        "backgroundRemoval" => &["foregroundMark", "backgroundMark"],
        "lineSketchStyleProps" => &["custGeom", "prstGeom", "type", "seed", "extLst"],
        // Any number of effects, in any order, then `extLst`.
        "blip" => return Some(usize::from(local == "extLst")),
        _ => match crate::table::order_in(parent).or_else(|| crate::comment::order_in(parent)) {
            Some(order) => order,
            None => return crate::timing::rank(parent, local),
        },
    };
    order.iter().position(|o| *o == local)
}

/// The sibling a new child of this name goes before, by the schema's order.
pub(crate) fn landmark(
    w: &Writer<'_>,
    parent: &Elem,
    local: &str,
) -> Result<Option<Elem>, ExportError> {
    let Some(mine) = rank(&parent.local, local) else {
        return Ok(None);
    };
    Ok(parent.children(w.tree, w.part)?.into_iter().find(|c| {
        (c.in_namespace(ns::DML)
            || c.in_namespace(ns::PML)
            || c.in_namespace(P14)
            || c.uri == parent.uri)
            && rank(&parent.local, &c.local).is_some_and(|theirs| theirs > mine)
    }))
}

fn out_of_range(w: &Writer<'_>, what: &'static str) -> ExportError {
    ExportError::ValueOutOfRange {
        what,
        part: w.part.to_owned(),
    }
}

/// A new child: built, placed, then patched from nothing.
pub(crate) fn insert_new(
    w: &mut Writer<'_>,
    parent: &Elem,
    before: Option<&Elem>,
    node: &Node,
) -> Result<(), ExportError> {
    let el = w.new_child(parent, node.space.uris(), node.space.prefix(), node.local)?;
    let anchor = match before {
        Some(el) => Some(el.id),
        None => landmark(w, parent, node.local)?.map(|l| l.id),
    };
    w.tree.insert_child(parent.id, anchor, el.id);
    patch(w, &el, None, node)
}

fn write_attr(w: &mut Writer<'_>, el: &Elem, name: &str, value: &Value) -> Result<(), ExportError> {
    let (name, text) = match value {
        Value::Text(text) => (name.to_owned(), text.clone()),
        Value::PositiveCoordinate(v) => {
            if *v > POWERPOINT_COORDINATE_MAX {
                return Err(out_of_range(w, "a coordinate past what PowerPoint accepts"));
            }
            (name.to_owned(), v.to_string())
        }
        Value::DashStop(v) => {
            if *v <= 0 {
                return Err(out_of_range(w, "a dash stop of zero or less"));
            }
            let spelled = w.strict || el.get(name).is_some_and(|old| old.contains('%'));
            let text = if spelled {
                format_percentage(*v)
            } else {
                v.to_string()
            };
            (name.to_owned(), text)
        }
        Value::Percent(v) => {
            let spelled = w.strict || el.get(name).is_some_and(|old| old.contains('%'));
            let text = if spelled {
                format_percentage(*v)
            } else {
                v.to_string()
            };
            (name.to_owned(), text)
        }
        Value::P14(text) => {
            let written = match el.name_of_ns(P14, name) {
                Some(written) => written.to_owned(),
                None => format!("{}:{name}", w.declared_prefix(el, P14, "p14")?),
            };
            (written, text.clone())
        }
        Value::Rel(id) => {
            if !id.is_empty()
                && w.known_rels
                    .as_ref()
                    .is_some_and(|known| !known.contains(id))
            {
                return Err(ExportError::UnknownRelationship {
                    part: w.part.to_owned(),
                });
            }
            let written = match el.name_of_ns(ns::REL, name) {
                Some(written) => written.to_owned(),
                None => format!("{}:{name}", w.declared_prefix(el, ns::REL, "r")?),
            };
            (written, id.clone())
        }
    };
    w.attr(el, &name, None::<&String>, Some(&text))
}

/// Bring an element from what `before` says to what `after` says.
pub(crate) fn patch(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&Node>,
    after: &Node,
) -> Result<(), ExportError> {
    if before == Some(after) {
        return Ok(());
    }
    for (name, value) in &after.attrs {
        if before.and_then(|b| b.value(name)) != Some(value) {
            write_attr(w, el, name, value)?;
        }
    }
    for (name, value) in before.iter().flat_map(|b| &b.attrs) {
        if after.value(name).is_none() {
            let written = match value {
                Value::Rel(_) => el.name_of_ns(ns::REL, name).map(str::to_owned),
                Value::P14(_) => el.name_of_ns(P14, name).map(str::to_owned),
                _ => Some((*name).to_owned()),
            };
            if let Some(written) = written {
                w.tree.remove_attribute(el.id, &written);
                w.changes += 1;
            }
        }
    }
    if before.and_then(|b| b.text.as_ref()) != after.text.as_ref() {
        match &after.text {
            Some(text) => w.set_text(el, text)?,
            None => {
                w.set_text(el, "")?;
            }
        }
    }
    patch_kids(w, el, before.map_or(&[], |b| &b.kids), &after.kids)?;
    patch_exts(w, el, before.map_or(&[], |b| &b.exts), &after.exts)
}

fn patch_kids(
    w: &mut Writer<'_>,
    el: &Elem,
    before: &[Node],
    after: &[Node],
) -> Result<(), ExportError> {
    if before == after {
        return Ok(());
    }
    let mut tree_kids: Vec<Elem> = el.children(w.tree, w.part)?;

    // A choice that changed: every element of that choice goes, what the model could
    // not see included, or the reader would find it instead of the new one.
    let names = |list: &[Node], group: &[&str]| -> Vec<&'static str> {
        list.iter()
            .filter(|n| n.space == Space::Dml && group.contains(&n.local))
            .map(|n| n.local)
            .collect()
    };
    let mut replaced: Vec<&'static [&'static str]> = Vec::new();
    for group in choices_of(&el.local) {
        if names(before, group) != names(after, group) {
            for kid in &tree_kids {
                if kid.in_namespace(ns::DML) && group.contains(&kid.local.as_str()) {
                    w.tree.remove_child(el.id, kid.id);
                    w.changes += 1;
                }
            }
            replaced.push(group);
        }
    }
    let gone = |n: &Node| n.space == Space::Dml && replaced.iter().any(|g| g.contains(&n.local));
    tree_kids.retain(|k| {
        !(k.in_namespace(ns::DML) && replaced.iter().any(|g| g.contains(&k.local.as_str())))
    });

    // Each `before` child owns the next tree child of its name.
    let mut owners: HashMap<(Space, &str), VecDeque<usize>> = HashMap::new();
    let mut element_of: Vec<Option<usize>> = Vec::with_capacity(before.len());
    {
        let mut by_name: HashMap<(&str, Space), VecDeque<usize>> = HashMap::new();
        for (i, kid) in tree_kids.iter().enumerate() {
            for space in SPACES {
                if kid.in_namespace(space.uris()) {
                    by_name
                        .entry((kid.local.as_str(), space))
                        .or_default()
                        .push_back(i);
                }
            }
        }
        for (i, n) in before.iter().enumerate() {
            let found = if gone(n) {
                None
            } else {
                by_name
                    .get_mut(&(n.local, n.space))
                    .and_then(VecDeque::pop_front)
            };
            element_of.push(found);
            if found.is_some() {
                owners.entry((n.space, n.local)).or_default().push_back(i);
            }
        }
    }

    // Each `after` child takes the `before` child of the same name and occurrence.
    let mut paired: Vec<Option<usize>> = Vec::with_capacity(after.len());
    for n in after {
        paired.push(
            owners
                .get_mut(&(n.space, n.local))
                .and_then(VecDeque::pop_front),
        );
    }
    let reordered = paired
        .iter()
        .flatten()
        .zip(paired.iter().flatten().skip(1))
        .any(|(x, y)| x > y);
    if reordered {
        // A list whose order changed is rebuilt: its order is what the edit named.
        for owned in element_of.iter().flatten() {
            if let Some(kid) = tree_kids.get(*owned) {
                w.tree.remove_child(el.id, kid.id);
                w.changes += 1;
            }
        }
        for n in after {
            insert_new(w, el, None, n)?;
        }
        return Ok(());
    }

    // Surplus and removed: every child of a name no longer wanted, repeats included.
    let mut used = vec![false; before.len()];
    for p in paired.iter().flatten() {
        if let Some(slot) = used.get_mut(*p) {
            *slot = true;
        }
    }
    let mut keep = vec![true; tree_kids.len()];
    for (i, owned) in element_of.iter().enumerate() {
        if used.get(i).copied().unwrap_or(false) {
            continue;
        }
        let Some(owned) = *owned else { continue };
        if let Some(kid) = tree_kids.get(owned) {
            w.tree.remove_child(el.id, kid.id);
            w.changes += 1;
        }
        if let Some(slot) = keep.get_mut(owned) {
            *slot = false;
        }
    }
    // Each name once, so a long list costs its length and not its square.
    let wanted: HashSet<(Space, &str)> = after.iter().map(|a| (a.space, a.local)).collect();
    let mut seen_names = HashSet::new();
    let unwanted: Vec<&Node> = before
        .iter()
        .filter(|n| !wanted.contains(&(n.space, n.local)) && seen_names.insert((n.space, n.local)))
        .collect();
    for (i, kid) in tree_kids.iter().enumerate() {
        if keep.get(i).copied().unwrap_or(false) && unwanted.iter().any(|n| n.is(kid)) {
            w.tree.remove_child(el.id, kid.id);
            w.changes += 1;
            if let Some(slot) = keep.get_mut(i) {
                *slot = false;
            }
        }
    }

    // What the model could not see, by name, for a new child of that name to use.
    let owned: HashSet<usize> = element_of.iter().flatten().copied().collect();
    let mut unseen: HashMap<(Space, &str), VecDeque<usize>> = HashMap::new();
    for (i, kid) in tree_kids.iter().enumerate() {
        if owned.contains(&i) || !keep.get(i).copied().unwrap_or(false) {
            continue;
        }
        for space in SPACES {
            if kid.in_namespace(space.uris()) {
                unseen
                    .entry((space, kid.local.as_str()))
                    .or_default()
                    .push_back(i);
            }
        }
    }
    // For each `after` child, the tree child of the next one that stays.
    let mut following: Vec<Option<usize>> = vec![None; after.len()];
    let mut next = None;
    for j in (0..after.len()).rev() {
        if let Some(slot) = following.get_mut(j) {
            *slot = next;
        }
        if let Some(i) = paired.get(j).copied().flatten() {
            next = element_of.get(i).copied().flatten().or(next);
        }
    }

    for (j, n) in after.iter().enumerate() {
        if let Some(i) = paired.get(j).copied().flatten() {
            let kid = element_of
                .get(i)
                .copied()
                .flatten()
                .and_then(|o| tree_kids.get(o));
            if let (Some(kid), Some(b)) = (kid, before.get(i)) {
                patch(w, kid, Some(b), n)?;
            }
            continue;
        }
        let adopted = unseen
            .get_mut(&(n.space, n.local))
            .and_then(VecDeque::pop_front)
            .and_then(|i| tree_kids.get(i));
        if let Some(kid) = adopted {
            patch(w, kid, None, n)?;
            continue;
        }
        // Before the next child that stays, or by the schema if none does.
        let before_kid = following
            .get(j)
            .copied()
            .flatten()
            .and_then(|o| tree_kids.get(o));
        insert_new(w, el, before_kid, n)?;
    }
    Ok(())
}

/// The `uri` for an extension this writer may add. Those from real decks are as
/// PowerPoint writes them (`fixtures/real`); the sketch's is Inferred, from memory of
/// PowerPoint's output and no deck here. Others are patched where a file has them, and
/// cannot be added.
fn extension_uri(local: &str) -> Option<&'static str> {
    Some(match local {
        "hiddenFill" => "{909E8E84-426E-40DD-AFC4-6F175D3DCCD1}",
        "hiddenLine" => "{91240B29-F687-4F45-9708-019B960494DF}",
        "hiddenEffects" => "{AF507438-7753-43E0-B8FC-AC1667EBCBE1}",
        "useLocalDpi" => "{28A0092B-C50C-407E-A947-70E740481C1C}",
        "imgProps" => "{BEBA8EAE-BF5A-486C-A8C5-ECC9F3942E4B}",
        "svgBlip" => "{96DAC541-7B7A-43D3-8B79-37D633B846F1}",
        "lineSketchStyleProps" => "{C807C97D-BFC1-408E-A445-0C87EB9F89A2}",
        _ => return None,
    })
}

/// Find an extension's item, with its `ext` and `extLst`.
fn find_item(
    w: &Writer<'_>,
    el: &Elem,
    node: &Node,
) -> Result<Option<(Elem, Elem, Elem)>, ExportError> {
    let Some(list) = el.first_child(w.tree, w.part, ns::DML, "extLst")? else {
        return Ok(None);
    };
    for ext in list.children(w.tree, w.part)? {
        if !ext.is(ns::DML, "ext") {
            continue;
        }
        for item in ext.children(w.tree, w.part)? {
            if node.is(&item) {
                return Ok(Some((list, ext, item)));
            }
        }
    }
    Ok(None)
}

fn patch_exts(
    w: &mut Writer<'_>,
    el: &Elem,
    before: &[Node],
    after: &[Node],
) -> Result<(), ExportError> {
    if before == after {
        return Ok(());
    }
    for b in before {
        if after.iter().any(|a| a.same_name(b)) {
            continue;
        }
        if let Some((list, ext, item)) = find_item(w, el, b)? {
            w.tree.remove_child(ext.id, item.id);
            w.changes += 1;
            // The holders go with their last entry; another's entry keeps them.
            if ext.children(w.tree, w.part)?.is_empty() {
                w.tree.remove_child(list.id, ext.id);
                if list.children(w.tree, w.part)?.is_empty() {
                    w.tree.remove_child(el.id, list.id);
                }
            }
        }
    }
    for a in after {
        let b = before.iter().find(|b| b.same_name(a));
        if let Some((_, _, item)) = find_item(w, el, a)? {
            patch(w, &item, b, a)?;
            continue;
        }
        let Some(uri) = extension_uri(a.local).or(a.ext_uri.as_deref()) else {
            return Err(out_of_range(w, "adding this extension"));
        };
        let list = match el.first_child(w.tree, w.part, ns::DML, "extLst")? {
            Some(list) => list,
            None => {
                let list = w.new_child(el, ns::DML, "a", "extLst")?;
                let anchor = landmark(w, el, "extLst")?.map(|l| l.id);
                w.tree.insert_child(el.id, anchor, list.id);
                list
            }
        };
        let ext = w.new_child(&list, ns::DML, "a", "ext")?;
        w.tree.insert_child(list.id, None, ext.id);
        w.attr(&ext, "uri", None::<&String>, Some(&uri.to_owned()))?;
        insert_new(w, &ext, None, a)?;
    }
    Ok(())
}

// spec-audit: x:drawing/2010/main:<hiddenFill>
// spec-audit: x:drawing/2010/main:<hiddenLine>
// spec-audit: x:drawing/2010/main:<hiddenEffects>
// spec-audit: x:drawing/2010/main:<hiddenScene3d>
// spec-audit: x:drawing/2010/main:<hiddenSp3d>
// spec-audit: x:drawing/2010/main:<shadowObscured>
fn hidden_nodes(h: &HiddenProperties) -> Vec<Node> {
    let mut out = Vec::new();
    if let Some(fill) = &h.fill {
        out.push(Node::new(Space::A14, "hiddenFill").kid(fill_node(fill)));
    }
    if let Some(line) = &h.line {
        out.push(line_node(line, Space::A14, "hiddenLine"));
    }
    if let Some(effects) = &h.effects {
        out.push(Node::new(Space::A14, "hiddenEffects").kid(effect_properties_node(effects)));
    }
    if let Some(scene) = &h.scene3d {
        out.push(crate::d3::scene3d_node(scene, Space::A14, "hiddenScene3d"));
    }
    if let Some(sp3d) = &h.sp3d {
        out.push(crate::d3::sp3d_node(sp3d, Space::A14, "hiddenSp3d"));
    }
    if let Some(value) = &h.shadow_obscured {
        out.push(Node::new(Space::A14, "shadowObscured").lex("val", Some(value)));
    }
    out
}

pub(crate) fn properties_node(p: &ShapeProperties, holder: &'static str) -> Node {
    let mut n = Node::new(Space::Pml, holder)
        .opt(p.geometry.as_ref().map(geometry_node))
        .opt(p.fill.as_ref().map(fill_node))
        .opt(p.line.as_ref().map(|l| line_node(l, Space::Dml, "ln")))
        .opt(p.effects.as_ref().map(effect_properties_node))
        .opt(
            p.scene3d
                .as_ref()
                .map(|s| crate::d3::scene3d_node(s, Space::Dml, "scene3d")),
        )
        .opt(
            p.sp3d
                .as_ref()
                .map(|s| crate::d3::sp3d_node(s, Space::Dml, "sp3d")),
        );
    if let Some(hidden) = &p.hidden {
        for item in hidden_nodes(hidden) {
            n = n.ext(Some(item));
        }
    }
    n
}

fn blip_of(fill: Option<&Fill>) -> Option<&Blip> {
    match fill {
        Some(Fill::Blip(f)) => f.blip.as_ref(),
        _ => None,
    }
}

/// An SVG keeps its fallback: an edit taking the blip's own picture away and leaving
/// `asvg:svgBlip` would leave one older readers cannot show (`MS-ODRAWXML` "Pictures").
/// A file that arrived that way is reported on reading and left alone.
pub(crate) fn check_svg_fallback(
    w: &Writer<'_>,
    before: Option<&Blip>,
    after: Option<&Blip>,
) -> Result<(), ExportError> {
    let lost = |b: &Blip| b.svg.is_some() && b.embed.is_none() && b.link.is_none();
    if after.is_some_and(lost) && !before.is_some_and(lost) {
        return Err(out_of_range(w, "an SVG with no fallback"));
    }
    Ok(())
}

fn matrix_ref(local: &'static str, r: &StyleMatrixRef) -> Node {
    Node::a(local)
        .lex("idx", r.idx.as_ref())
        .color(r.color.as_ref())
}

fn font_ref(r: &FontRef) -> Node {
    Node::a("fontRef")
        .lex("idx", r.idx.map(|i| i.as_xml().to_owned()).as_ref())
        .color(r.color.as_ref())
}

fn style_node(s: &ShapeStyle) -> Node {
    Node::new(Space::Pml, "style")
        .opt(s.line.as_ref().map(|r| matrix_ref("lnRef", r)))
        .opt(s.fill.as_ref().map(|r| matrix_ref("fillRef", r)))
        .opt(s.effect.as_ref().map(|r| matrix_ref("effectRef", r)))
        .opt(s.font.as_ref().map(font_ref))
}

/// `p:style`. The schema requires all four references, so a style short of one is
/// refused here, unless it came that way and that part did not change.
pub(crate) fn patch_style(
    w: &mut Writer<'_>,
    el: &Elem,
    before: Option<&ShapeStyle>,
    after: &ShapeStyle,
) -> Result<(), ExportError> {
    let short = |s: &ShapeStyle| {
        s.line.is_none() || s.fill.is_none() || s.effect.is_none() || s.font.is_none()
    };
    if short(after) && !before.is_some_and(short) {
        return Err(ExportError::UnsupportedEdit {
            what: "a shape style short of a reference",
            part: w.part.to_owned(),
        });
    }
    patch(w, el, before.map(style_node).as_ref(), &style_node(after))
}

/// Bring a node's properties from `before` to `after`.
pub(crate) fn patch_properties(
    w: &mut Writer<'_>,
    el: &Elem,
    kind: NodeKind,
    before: &ShapeProperties,
    after: &ShapeProperties,
) -> Result<(), ExportError> {
    let holder = match kind {
        NodeKind::Shape | NodeKind::Connector | NodeKind::Picture => "spPr",
        NodeKind::Group => "grpSpPr",
        NodeKind::GraphicFrame | NodeKind::ContentPart => {
            return Err(out_of_range(w, "drawing properties on this kind of node"));
        }
    };
    let (b, a) = (
        properties_node(before, holder),
        properties_node(after, holder),
    );
    if b != a {
        let Some(props) = el.first_child(w.tree, w.part, ns::PML, holder)? else {
            return Err(out_of_range(w, "drawing properties on a node with no spPr"));
        };
        patch(w, &props, Some(&b), &a)?;
    }
    check_svg_fallback(
        w,
        blip_of(before.fill.as_ref()),
        blip_of(after.fill.as_ref()),
    )?;
    check_svg_fallback(
        w,
        before.picture.as_ref().and_then(|f| f.blip.as_ref()),
        after.picture.as_ref().and_then(|f| f.blip.as_ref()),
    )?;
    if before.picture != after.picture {
        if kind != NodeKind::Picture {
            return Err(out_of_range(w, "a picture fill on a node not a picture"));
        }
        let of = |p: &ShapeProperties| {
            let root = Node::new(Space::Pml, "pic");
            root.opt(p.picture.as_ref().map(|f| blip_fill_node(f, Space::Pml)))
        };
        patch(w, el, Some(&of(before)), &of(after))?;
    }
    Ok(())
}
