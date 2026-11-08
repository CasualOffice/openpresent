//! Group and Ungroup (`PAR-03`, Shape Format > Group, Ctrl+G and Ctrl+Shift+G).
//!
//! OpenCalc has no shape groups to copy from (owner instruction 42: checked, its only
//! "group" is row and column outlines), so this is new, built on the tree operations
//! Arrange already uses. Group puts the selected top-level shapes, in their z-order,
//! into a new `p:grpSp` at the place of the frontmost one, its child space the same as
//! the slide's (`chOff` = `off`, `chExt` = `ext`), so no child's transform changes and
//! nothing moves. Ungroup puts a group's children back in its place, each transform
//! carried through the group's mapping into the space the group was in. Each is one
//! `Batch` of `DeleteNode` and `InsertNode`, one undo step.
//!
//! Saving: [`Viewer::group_pass`] runs before the shapes pass and moves the file's own
//! elements, byte for byte, into a new `p:grpSp` or out of a removed one, so what the
//! model does not know about a shape travels with it. The patch pass then writes the
//! moved children's new transforms.
//!
//! As PowerPoint, placeholders, tables and `noGrp` shapes cannot be grouped, nor a
//! `noUngrp` group ungrouped. Shared sessions refuse both
//! verbs for now: the wire has no form for moving a node with its subtree (`PAR-03`).

use std::collections::{BTreeMap, BTreeSet};

use casual_slide_model::presentation::{Coordinate, POSITIVE_COORDINATE_MAX, Point, PositiveSize};
use casual_slide_model::shape::{NodeKind, ObjectId, ShapeNode, Transform};
use casual_slide_model::slide::SlidePart;
use casual_slide_ooxml::{ElementId, OoxmlLimits, PresentationPackage, XmlTree};
use casual_slide_render::Deck;
use casual_slide_render::display::Affine;
use casual_slide_render::scene::group_mapping;
use casual_slide_style::Chain;
use casual_slide_transaction::{Operation, TxnError};

use crate::build::{NAMESPACES, TREE_HEAD, highest, read_slide, spid};
use crate::{BridgeError, Viewer, group};

const PML: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";
const DML: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";

/// The four corners of a transform's box on the space `m` maps it onto, as the
/// renderer places it: the axis-aligned bounds `(left, top, right, bottom)`.
fn bounds(t: &casual_slide_style::EffectiveTransform, m: &Affine) -> (f64, f64, f64, f64) {
    let (w, h) = (t.extent.cx as f64, t.extent.cy as f64);
    let (cx, cy) = (t.offset.x.0 as f64 + w / 2.0, t.offset.y.0 as f64 + h / 2.0);
    let r = (f64::from(t.rotation) / 60_000.0).to_radians();
    let (s, c) = r.sin_cos();
    let mut out = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let (px, py) = (dx * w / 2.0, dy * h / 2.0);
        let (x, y) = m.apply(cx + px * c - py * s, cy + px * s + py * c);
        out = (out.0.min(x), out.1.min(y), out.2.max(x), out.3.max(y));
    }
    out
}

fn emu(v: f64) -> i64 {
    // Bounded by the clamp below; a non-finite value becomes 0.
    if v.is_finite() {
        (v.round() as i64).clamp(-POSITIVE_COORDINATE_MAX, POSITIVE_COORDINATE_MAX)
    } else {
        0
    }
}

/// The groups from the root down to the node `node`'s parent, outermost first, and the
/// parent itself (the root when the node is at the top).
fn parent_path(root: &ShapeNode, node: u32) -> Option<Vec<&ShapeNode>> {
    fn walk<'a>(at: &'a ShapeNode, node: u32, path: &mut Vec<&'a ShapeNode>) -> bool {
        if path.len() > casual_slide_render::scene::MAX_GROUP_DEPTH {
            return false;
        }
        path.push(at);
        if at.children.iter().any(|c| c.id.node == node) {
            return true;
        }
        for child in &at.children {
            if child.kind == NodeKind::Group && walk(child, node, path) {
                return true;
            }
        }
        path.pop();
        false
    }
    let mut path = Vec::new();
    walk(root, node, &mut path).then_some(path)
}

/// What the groups of `path` (the root first, which maps nothing) do to coordinates in
/// the last one's child space.
fn mapping(path: &[&ShapeNode]) -> Affine {
    path.iter()
        .skip(1)
        .fold(Affine::IDENTITY, |m, g| match &g.transform {
            Some(t) => m.then_after(&group_mapping(t)),
            None => m,
        })
}

impl Viewer {
    /// Group the top-level shapes `nodes` of slide `index` into a new group at the place
    /// of the frontmost: one undo step. Returns the group's node.
    ///
    /// # Errors
    ///
    /// `CS-WSM-0006` for fewer than two shapes, one not at the top of the slide, a
    /// placeholder, a table, a shape locked with `noGrp` or one without a place of its
    /// own; `CS-TXN-0015` in a
    /// shared session.
    pub fn group(&mut self, index: usize, nodes: &[u32]) -> Result<u32, BridgeError> {
        if self.is_shared() {
            return Err(BridgeError::Edit(TxnError::NotShared));
        }
        let ordinal = self.slide_ordinal(index)?;
        let parts = self.deck.parts();
        let part = parts.get(ordinal).ok_or(BridgeError::BadArgument)?;
        let root = part.shape_tree().ok_or(BridgeError::BadArgument)?;
        let wanted: BTreeSet<u32> = nodes.iter().copied().collect();
        let members: Vec<(usize, &ShapeNode)> = root
            .children
            .iter()
            .enumerate()
            .filter(|(_, c)| wanted.contains(&c.id.node))
            .collect();
        if members.len() < 2 || members.len() != wanted.len() {
            return Err(BridgeError::BadArgument);
        }
        let chain = Chain::of(parts, ordinal);
        let mut b = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for (_, shape) in &members {
            let locked = shape.locks.and_then(|l| l.no_grp) == Some(true);
            if locked
                || shape.placeholder().is_some()
                || shape.table.is_some()
                || shape.transform.is_none()
            {
                return Err(BridgeError::BadArgument);
            }
            let t = chain
                .transform(shape)
                .ok_or(BridgeError::BadArgument)?
                .value;
            let s = bounds(&t, &Affine::IDENTITY);
            b = (b.0.min(s.0), b.1.min(s.1), b.2.max(s.2), b.3.max(s.3));
        }
        let (x, y) = (emu(b.0), emu(b.1));
        let (cx, cy) = (emu(b.2 - b.0).max(0), emu(b.3 - b.1).max(0));
        let (top_spid, top_node) = highest(part);
        let id = top_spid.checked_add(1).ok_or(BridgeError::BadArgument)?;
        let name = format!("Group {}", id.saturating_sub(1));
        let element = format!(
            "<p:grpSp><p:nvGrpSpPr><p:cNvPr id=\"{id}\" name=\"{name}\"/><p:cNvGrpSpPr/><p:nvPr/>\
             </p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"{x}\" y=\"{y}\"/><a:ext cx=\"{cx}\" \
             cy=\"{cy}\"/><a:chOff x=\"{x}\" y=\"{y}\"/><a:chExt cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm>\
             </p:grpSpPr></p:grpSp>"
        );
        let xml = format!(
            "<p:sld {NAMESPACES}><p:cSld><p:spTree>{TREE_HEAD}{element}</p:spTree></p:cSld></p:sld>"
        );
        let read = read_slide(xml.as_bytes(), "ppt/slides/openpresent-group.xml", ordinal)?;
        let mut grp = read
            .shape_tree()
            .and_then(|t| t.children.first())
            .cloned()
            .ok_or(BridgeError::Build("CS-WSM-0007"))?;
        let node_number = top_node.checked_add(1).ok_or(BridgeError::BadArgument)?;
        grp.id = ObjectId {
            part: ordinal,
            node: node_number,
        };
        grp.children = members.iter().map(|(_, s)| (*s).clone()).collect();
        let front = members.last().map_or(0, |(i, _)| *i);
        let at = front.saturating_add(1).saturating_sub(members.len());
        let parent = root.id;
        let mut ops: Vec<Operation> = members
            .iter()
            .map(|(_, s)| Operation::DeleteNode { node: s.id })
            .collect();
        ops.push(Operation::InsertNode {
            parent,
            index: at,
            node: Box::new(grp),
        });
        self.typing = None;
        self.history
            .apply(self.deck.parts_mut(), Operation::Batch(ops))?;
        self.reports.remove(&index);
        Ok(node_number)
    }

    /// Ungroup group `node` of slide `index`: its children take its place, each where it
    /// was drawn. One undo step. Returns the children's nodes, back to front.
    ///
    /// # Errors
    ///
    /// `CS-WSM-0006` for a node that is not a group, or one locked with `noUngrp`;
    /// `CS-TXN-0015` in a shared session.
    pub fn ungroup(&mut self, index: usize, node: u32) -> Result<Vec<u32>, BridgeError> {
        if self.is_shared() {
            return Err(BridgeError::Edit(TxnError::NotShared));
        }
        let ordinal = self.slide_ordinal(index)?;
        let parts = self.deck.parts();
        let root = parts
            .get(ordinal)
            .and_then(SlidePart::shape_tree)
            .ok_or(BridgeError::BadArgument)?;
        let path = parent_path(root, node).ok_or(BridgeError::BadArgument)?;
        let parent = *path.last().ok_or(BridgeError::BadArgument)?;
        let (at, grp) = parent
            .children
            .iter()
            .enumerate()
            .find(|(_, c)| c.id.node == node)
            .ok_or(BridgeError::BadArgument)?;
        // `noUngrp`, as PowerPoint, keeps a group whole.
        if grp.kind != NodeKind::Group || grp.locks.and_then(|l| l.no_ungrp) == Some(true) {
            return Err(BridgeError::BadArgument);
        }
        let outer = mapping(&path);
        let mut inner_path = path.clone();
        inner_path.push(grp);
        let inner = mapping(&inner_path);
        let mirrors = |m: &Affine| m.a * m.d - m.b * m.c < 0.0;
        // The group's own mapping: what is undone moving a child out of it.
        let flips = mirrors(&inner) != mirrors(&outer);
        let chain = Chain::of(parts, ordinal);
        let mut ops = vec![Operation::DeleteNode { node: grp.id }];
        let mut out = Vec::new();
        for (i, child) in grp.children.iter().enumerate() {
            let mut moved = child.clone();
            if let Some(t) = chain.transform(child).map(|t| t.value) {
                // The child's box on the slide, then in the parent's space.
                let w = t.extent.cx as f64;
                let h = t.extent.cy as f64;
                let (sx, sy) = (inner.a.hypot(inner.b), inner.c.hypot(inner.d));
                let (mx, my) =
                    inner.apply(t.offset.x.0 as f64 + w / 2.0, t.offset.y.0 as f64 + h / 2.0);
                let degrees = inner.b.atan2(inner.a).to_degrees();
                let rot = f64::from(t.rotation) / 60_000.0 + degrees;
                let slide = (
                    emu(mx - w * sx / 2.0),
                    emu(my - h * sy / 2.0),
                    emu(w * sx),
                    emu(h * sy),
                );
                let rot_units =
                    i32::try_from(emu(rot * 60_000.0).rem_euclid(21_600_000)).unwrap_or(0);
                let ((x, y, cx, cy), rotation) = if path.len() > 1 {
                    group::to_child_space(&outer, slide, rot_units)
                } else {
                    (slide, rot_units)
                };
                let mut nt: Transform = child.transform.unwrap_or_default();
                nt.offset = Some(Point {
                    x: Coordinate(x),
                    y: Coordinate(y),
                });
                nt.extent = Some(PositiveSize {
                    cx: cx.max(0),
                    cy: cy.max(0),
                });
                let rotation = rotation.rem_euclid(21_600_000);
                // Absent stays absent.
                if rotation != 0 || nt.rotation.is_some() {
                    nt.rotation = Some(rotation);
                }
                if flips {
                    nt.flip_h = if t.flip_h { None } else { Some(true) };
                }
                // A group child keeps its own child space: only its place changes.
                moved.transform = Some(nt);
            }
            out.push(moved.id.node);
            ops.push(Operation::InsertNode {
                parent: parent.id,
                index: at.saturating_add(i),
                node: Box::new(moved),
            });
        }
        self.typing = None;
        self.history
            .apply(self.deck.parts_mut(), Operation::Batch(ops))?;
        self.reports.remove(&index);
        Ok(out)
    }

    /// Save step, before the shapes pass: each slide's shape elements put into the
    /// groups the model has them in, by moving the file's own elements. A group the
    /// model added is written new; one it removed gives its children to its parent.
    pub(crate) fn group_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let save = |code: &'static str| BridgeError::Save(code);
        let file = Deck::open(bytes.clone())?;
        let mut wanted = Vec::new();
        for (name, f, m) in self.pairs(file.parts())? {
            let (Some(ft), Some(mt)) = (f.shape_tree(), m.shape_tree()) else {
                continue;
            };
            let (fp, mp) = (parents(ft), parents(mt));
            let moved = mp.iter().any(|(s, p)| fp.get(s).is_some_and(|q| q != p))
                || mt.descendants().any(|n| {
                    n.kind == NodeKind::Group
                        && spid(n).is_some_and(|s| !fp.contains_key(&s))
                        && n.children
                            .iter()
                            .any(|c| spid(c).is_some_and(|s| fp.contains_key(&s)))
                });
            if moved {
                wanted.push((
                    name,
                    mt.clone(),
                    fp.keys().copied().collect::<BTreeSet<_>>(),
                ));
            }
        }
        if wanted.is_empty() {
            return Ok(bytes);
        }
        let mut package = PresentationPackage::open(bytes.clone(), OoxmlLimits::default())
            .map_err(|e| save(e.code()))?;
        let xml = *package.limits();
        let mut plan = casual_slide_export::writer::SavePlan::new();
        for (name, model, in_file) in wanted {
            let b = package.read_xml_part(&name).map_err(|e| save(e.code()))?;
            let mut tree = XmlTree::parse(&b, &name, &xml).map_err(|e| save(e.code()))?;
            regroup_tree(&mut tree, &model, &in_file).ok_or(save("CS-WSM-0009"))?;
            plan = plan.replace_part(name, tree.to_bytes());
        }
        Self::run(&plan, bytes)
    }
}

/// Each shape's `spid` and its parent group's (`0` for the slide's tree), at any depth.
fn parents(root: &ShapeNode) -> BTreeMap<u32, u32> {
    fn walk(at: &ShapeNode, parent: u32, out: &mut BTreeMap<u32, u32>, depth: usize) {
        if depth > casual_slide_render::scene::MAX_GROUP_DEPTH {
            return;
        }
        for c in &at.children {
            if let Some(s) = spid(c) {
                out.insert(s, parent);
                if c.kind == NodeKind::Group {
                    walk(c, s, out, depth.saturating_add(1));
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(root, 0, &mut out, 0);
    out
}

/// The `cNvPr/@id` of a shape element.
fn element_spid(tree: &XmlTree, el: ElementId) -> Option<u32> {
    let nv = tree
        .child_elements(el)
        .into_iter()
        .find(|&c| tree.local_name(c).starts_with("nv"))?;
    let c = tree.child(nv, "cNvPr")?;
    tree.plain_attribute(c, "id")?.parse().ok()
}

/// Every shape element under `at`, by `spid`, at any depth.
fn index_elements(tree: &XmlTree, at: ElementId, out: &mut BTreeMap<u32, ElementId>, depth: usize) {
    if depth > casual_slide_render::scene::MAX_GROUP_DEPTH {
        return;
    }
    for c in tree.child_elements(at) {
        if let Some(s) = element_spid(tree, c) {
            out.insert(s, c);
            if tree.local_name(c) == "grpSp" {
                index_elements(tree, c, out, depth.saturating_add(1));
            }
        }
    }
}

/// Bring the file's `spTree` to the model's grouping: removed groups first, children
/// lifted into their place; then each new group, innermost first, written at its first
/// child's place with its children moved into it in the model's order.
fn regroup_tree(tree: &mut XmlTree, model: &ShapeNode, in_file: &BTreeSet<u32>) -> Option<()> {
    let root = tree.root();
    let c_sld = tree.child(root, "cSld")?;
    let sp_tree = tree.child(c_sld, "spTree")?;
    let mut by_spid = BTreeMap::new();
    index_elements(tree, sp_tree, &mut by_spid, 0);
    let in_model = parents(model);
    // Ungroup: a group of the file the model no longer has.
    let gone: Vec<u32> = by_spid
        .iter()
        .filter(|(s, e)| tree.local_name(**e) == "grpSp" && !in_model.contains_key(s))
        .map(|(s, _)| *s)
        .collect();
    for s in gone {
        let g = *by_spid.get(&s)?;
        let parent = tree.parent_element(g)?;
        let kids: Vec<ElementId> = tree
            .child_elements(g)
            .into_iter()
            .filter(|&k| element_spid(tree, k).is_some_and(|k| in_model.contains_key(&k)))
            .collect();
        for k in &kids {
            tree.remove_child(g, *k);
            tree.insert_child(parent, Some(g), *k);
        }
        tree.remove_child(parent, g);
        by_spid.remove(&s);
    }
    // Group: a group of the model the file does not have, innermost first.
    let mut new_groups = Vec::new();
    collect_new(model, in_file, &mut new_groups, 0);
    let p = tree
        .prefix_for_namespace(sp_tree, PML)
        .unwrap_or_else(|| "p".to_owned());
    let a = tree
        .prefix_for_namespace(sp_tree, DML)
        .unwrap_or_else(|| "a".to_owned());
    let q = |prefix: &str, local: &str| {
        if prefix.is_empty() {
            local.to_owned()
        } else {
            format!("{prefix}:{local}")
        }
    };
    for g in new_groups {
        let s = spid(g)?;
        let kids: Vec<ElementId> = g
            .children
            .iter()
            .map(|c| spid(c).and_then(|k| by_spid.get(&k).copied()))
            .collect::<Option<_>>()?;
        let first = *kids.first()?;
        let parent = tree.parent_element(first)?;
        let el = tree.insert_element(parent, Some(first), &q(&p, "grpSp"), &[]);
        let nv = tree.insert_element(el, None, &q(&p, "nvGrpSpPr"), &[]);
        let name = g
            .non_visual
            .as_ref()
            .and_then(|n| n.name.clone())
            .unwrap_or_default();
        let id = s.to_string();
        tree.insert_element(nv, None, &q(&p, "cNvPr"), &[("id", &id), ("name", &name)]);
        tree.insert_element(nv, None, &q(&p, "cNvGrpSpPr"), &[]);
        tree.insert_element(nv, None, &q(&p, "nvPr"), &[]);
        let pr = tree.insert_element(el, None, &q(&p, "grpSpPr"), &[]);
        if let Some(t) = g.transform {
            let xf = tree.insert_element(pr, None, &q(&a, "xfrm"), &[]);
            let pt = |v: Option<Point>| v.map(|p| (p.x.0.to_string(), p.y.0.to_string()));
            let sz = |v: Option<PositiveSize>| v.map(|s| (s.cx.to_string(), s.cy.to_string()));
            for (local, xy, names) in [
                ("off", pt(t.offset), ("x", "y")),
                ("ext", sz(t.extent), ("cx", "cy")),
                ("chOff", pt(t.child_offset), ("x", "y")),
                ("chExt", sz(t.child_extent), ("cx", "cy")),
            ] {
                if let Some((u, v)) = xy {
                    tree.insert_element(xf, None, &q(&a, local), &[(names.0, &u), (names.1, &v)]);
                }
            }
        }
        for k in kids {
            let from = tree.parent_element(k)?;
            tree.remove_child(from, k);
            tree.insert_child(el, None, k);
        }
        by_spid.insert(s, el);
    }
    Some(())
}

/// The model's groups the file does not have, children before parents.
fn collect_new<'a>(
    at: &'a ShapeNode,
    in_file: &BTreeSet<u32>,
    out: &mut Vec<&'a ShapeNode>,
    depth: usize,
) {
    if depth > casual_slide_render::scene::MAX_GROUP_DEPTH {
        return;
    }
    for c in &at.children {
        if c.kind == NodeKind::Group {
            collect_new(c, in_file, out, depth.saturating_add(1));
            if spid(c).is_some_and(|s| !in_file.contains(&s)) {
                out.push(c);
            }
        }
    }
}

#[cfg(test)]
#[path = "regroup_tests.rs"]
mod tests;
