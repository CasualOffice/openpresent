//! Shapes inside a group (`UX-015`): as PowerPoint, a first click selects the group
//! and a second the shape inside it, whose text a click or typing then edits.
//!
//! A grouped shape's transform is in its group's child space (`chOff`, `chExt`), so
//! the editor is given its box on the slide, mapped through every group above it as
//! the renderer draws it, and a box the editor sends back is mapped into that space.

use casual_slide_model::shape::{NodeKind, ShapeNode};
use casual_slide_model::slide::SlidePart;
use casual_slide_render::display::Affine;
use casual_slide_render::scene::{MAX_GROUP_DEPTH, group_mapping};
use casual_slide_style::Chain;

use crate::{BridgeError, ShapeBox, Viewer};

/// The groups from the shape tree's root down to the parent of `node`, outermost
/// first; `None` when `node` is not under `root`.
fn ancestors(root: &ShapeNode, node: u32) -> Option<Vec<&ShapeNode>> {
    fn walk<'a>(at: &'a ShapeNode, node: u32, path: &mut Vec<&'a ShapeNode>) -> bool {
        if path.len() > MAX_GROUP_DEPTH {
            return false;
        }
        for child in &at.children {
            if child.id.node == node {
                return true;
            }
            if child.kind == NodeKind::Group {
                path.push(child);
                if walk(child, node, path) {
                    return true;
                }
                path.pop();
            }
        }
        false
    }
    let mut path = Vec::new();
    walk(root, node, &mut path).then_some(path)
}

/// What the groups in `path` do to their children's coordinates, outermost applied
/// last, as the renderer composes them. A group with no placement adds nothing.
fn mapping_of(path: &[&ShapeNode]) -> Affine {
    path.iter()
        .fold(Affine::IDENTITY, |m, g| match &g.transform {
            Some(t) => m.then_after(&group_mapping(t)),
            None => m,
        })
}

/// The mapping onto the slide of the space `node`'s transform is in: `None` for a
/// shape at the top of the tree, or one not found.
pub(crate) fn parent_mapping(root: &ShapeNode, node: u32) -> Option<Affine> {
    let path = ancestors(root, node)?;
    (!path.is_empty()).then(|| mapping_of(&path))
}

/// The mapping's scale along each axis and its rotation in degrees, and whether it
/// mirrors.
fn decompose(m: &Affine) -> (f64, f64, f64, bool) {
    let sx = m.a.hypot(m.b);
    let sy = m.c.hypot(m.d);
    let degrees = m.b.atan2(m.a).to_degrees();
    (sx, sy, degrees, m.a * m.d - m.b * m.c < 0.0)
}

fn round(v: f64) -> i64 {
    // Within `i64` for any box a part can hold; a non-finite value becomes 0.
    if v.is_finite() { v.round() as i64 } else { 0 }
}

/// A rotation in 60000ths of a degree, in `[0, 21600000)`.
fn rotation_units(degrees: f64) -> i32 {
    let units = round(degrees * 60_000.0).rem_euclid(21_600_000);
    i32::try_from(units).unwrap_or(0)
}

/// The selectable box of `shape`, its own transform carried onto the slide by `m`;
/// `None` for a shape hidden, `noSelect` or without a placement.
pub(crate) fn shape_box(chain: &Chain<'_>, shape: &ShapeNode, m: &Affine) -> Option<ShapeBox> {
    let hidden = shape.non_visual.as_ref().and_then(|n| n.hidden) == Some(true);
    let locks = shape.locks.unwrap_or_default();
    if hidden || locks.no_select == Some(true) {
        return None;
    }
    let t = chain.transform(shape)?.value;
    let (sx, sy, degrees, mirrors) = decompose(m);
    let (w, h) = (t.extent.cx as f64 * sx, t.extent.cy as f64 * sy);
    let (mx, my) = m.apply(
        t.offset.x.0 as f64 + t.extent.cx as f64 / 2.0,
        t.offset.y.0 as f64 + t.extent.cy as f64 / 2.0,
    );
    let bit = |on: Option<bool>, b: u32| if on == Some(true) { b } else { 0 };
    let flags = bit(Some(t.flip_h != mirrors), ShapeBox::FLIP_H)
        | bit(Some(t.flip_v), ShapeBox::FLIP_V)
        | bit(locks.no_move, ShapeBox::NO_MOVE)
        | bit(locks.no_resize, ShapeBox::NO_RESIZE)
        | bit(locks.no_rot, ShapeBox::NO_ROTATE)
        | bit(locks.no_change_aspect, ShapeBox::KEEP_ASPECT)
        | bit(Some(shape.kind == NodeKind::Group), ShapeBox::GROUP)
        | bit(Some(shape.placeholder().is_some()), ShapeBox::PLACEHOLDER)
        | bit(Some(shape.table.is_some()), ShapeBox::TABLE)
        | bit(Some(shape.chart.is_some()), ShapeBox::CHART)
        | bit(
            Some(shape.kind == NodeKind::Picture && shape.properties.picture.is_some()),
            ShapeBox::PICTURE,
        );
    Some(ShapeBox {
        node: shape.id.node,
        x: round(mx - w / 2.0),
        y: round(my - h / 2.0),
        cx: round(w),
        cy: round(h),
        rotation: rotation_units(f64::from(t.rotation) / 60_000.0 + degrees),
        flags,
    })
}

/// A box on the slide, and its rotation, in the child space `m` maps onto the slide:
/// what [`shape_box`] undoes.
pub(crate) fn to_child_space(
    m: &Affine,
    (x, y, cx, cy): (i64, i64, i64, i64),
    rotation: i32,
) -> ((i64, i64, i64, i64), i32) {
    let (sx, sy, degrees, _) = decompose(m);
    let det = m.a * m.d - m.b * m.c;
    if !det.is_finite() || det.abs() < 1e-12 || sx < 1e-12 || sy < 1e-12 {
        return ((x, y, cx, cy), rotation);
    }
    let (px, py) = (
        x as f64 + cx as f64 / 2.0 - m.e,
        y as f64 + cy as f64 / 2.0 - m.f,
    );
    let (lx, ly) = ((m.d * px - m.c * py) / det, (m.a * py - m.b * px) / det);
    let (w, h) = (cx as f64 / sx, cy as f64 / sy);
    (
        (round(lx - w / 2.0), round(ly - h / 2.0), round(w), round(h)),
        rotation_units(f64::from(rotation) / 60_000.0 - degrees),
    )
}

impl Viewer {
    /// The shapes directly inside group `group` of slide `index`, back to front, each
    /// box on the slide as the group draws it: what a second click in a group picks
    /// from. Empty for a node that is not a group.
    ///
    /// # Errors
    ///
    /// `CS-RND-0003` for a slide that does not exist.
    pub fn group_shapes(&self, index: usize, group: u32) -> Result<Vec<ShapeBox>, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let parts = self.deck.parts();
        let chain = Chain::of(parts, ordinal);
        let Some(root) = parts.get(ordinal).and_then(SlidePart::shape_tree) else {
            return Ok(Vec::new());
        };
        let Some(mut path) = ancestors(root, group) else {
            return Ok(Vec::new());
        };
        let Some(node) = path
            .last()
            .map_or(&root.children[..], |g| &g.children[..])
            .iter()
            .find(|n| n.id.node == group && n.kind == NodeKind::Group)
        else {
            return Ok(Vec::new());
        };
        path.push(node);
        let m = mapping_of(&path);
        Ok(node
            .children
            .iter()
            .filter_map(|shape| shape_box(&chain, shape, &m))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_mapped_into_a_scaled_rotated_group_maps_back() {
        let m = Affine::translate(1000.0, 2000.0)
            .then_after(&Affine::rotate_degrees(30.0))
            .then_after(&Affine::scale(2.0, 0.5));
        let (sx, sy, deg, _) = decompose(&m);
        assert!((sx - 2.0).abs() < 1e-9 && (sy - 0.5).abs() < 1e-9 && (deg - 30.0).abs() < 1e-9);
        // A child box at (100, 200) 400 by 600, rotated 10 degrees, on the slide.
        let (cx, cy) = m.apply(300.0, 500.0);
        let slide = (
            round(cx - 400.0),
            round(cy - 150.0),
            800,
            300,
            rotation_units(40.0),
        );
        let (b, rot) = to_child_space(&m, (slide.0, slide.1, slide.2, slide.3), slide.4);
        assert_eq!(b, (100, 200, 400, 600));
        assert_eq!(rot, 600_000);
    }
}
