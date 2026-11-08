//! Home > Layout on an existing slide (`PAR-07`, `RIB-02`): the slide moves to another
//! layout and its placeholders are remapped as PowerPoint does. OpenCalc has no slide
//! layouts to copy from (owner instruction 42: checked); the matching follows the
//! slide paste of `PAR-10` (`clipboard.rs`), which already maps placeholders to a
//! layout by kind.
//!
//! Each placeholder of the slide is matched to one of the new layout's: a title to a
//! title, a subtitle to a subtitle, a body or content placeholder to a body or content
//! one, any other kind to the same kind, the same `idx` first. A matched placeholder
//! takes the new one's `p:ph` and gives up its own position, so it sits where the new
//! layout puts it; its text and formatting stay. One with no match keeps its text
//! where it was drawn, its position written into it, or goes if it was empty. A text
//! placeholder of the new layout nothing matched is added, empty, as New Slide adds
//! it. Date, footer and slide number are left as they are. One undo step: a `Batch` of
//! `SetSlideLayout` and the shape operations. Shared sessions refuse it for now.

use casual_slide_model::shape::{
    NodeKind, ObjectId, Placeholder, PlaceholderType, ShapeNode, Transform,
};
use casual_slide_render::Deck;
use casual_slide_style::Chain;
use casual_slide_transaction::{Operation, TxnError};

use crate::build::highest;
use crate::{BridgeError, Viewer};

/// What a placeholder is for when matching: title, subtitle, body or content, or its
/// own kind. `None` for the date, footer, slide number and header, left alone.
fn role(p: &Placeholder) -> Option<u8> {
    Some(match p.placeholder_type {
        Some(PlaceholderType::Title | PlaceholderType::CtrTitle) => 0,
        Some(PlaceholderType::SubTitle) => 1,
        None | Some(PlaceholderType::Body | PlaceholderType::Obj) => 2,
        Some(
            PlaceholderType::Dt
            | PlaceholderType::Ftr
            | PlaceholderType::SldNum
            | PlaceholderType::Hdr,
        ) => return None,
        Some(other) => 3_u8.saturating_add(
            u8::try_from(
                PlaceholderType::ALL
                    .iter()
                    .position(|t| *t == other)
                    .unwrap_or(0),
            )
            .unwrap_or(0),
        ),
    })
}

/// Whether a shape holds any text.
fn has_text(node: &ShapeNode) -> bool {
    node.text_body.as_deref().is_some_and(|b| {
        b.paragraphs.iter().any(|p| {
            p.items
                .iter()
                .any(|i| casual_slide_transaction::item_length(i) > 0)
        })
    })
}

impl Viewer {
    /// Put slide `index` on layout `layout` (an ordinal from [`Viewer::layouts`]),
    /// remapping its placeholders: one undo step. `false` when it is already on it.
    ///
    /// # Errors
    ///
    /// `CS-WSM-0006` for a part that is not a layout; `CS-TXN-0015` in a shared
    /// session; the template's own code if the layout cannot be read.
    pub fn change_layout(&mut self, index: usize, layout: u32) -> Result<bool, BridgeError> {
        if self.is_shared() {
            return Err(BridgeError::Edit(TxnError::NotShared));
        }
        let ordinal = self.slide_ordinal(index)?;
        if !self.deck.parts().structure.layouts.contains(&layout) {
            return Err(BridgeError::BadArgument);
        }
        if self.layout_of(index) == Some(layout) {
            return Ok(false);
        }
        // The new layout's placeholders as New Slide would put them on a slide.
        let (_, template) = self.layout_slide(layout, ordinal)?;
        let parts = self.deck.parts();
        let part = parts.get(ordinal).ok_or(BridgeError::BadArgument)?;
        let root = part.shape_tree().ok_or(BridgeError::BadArgument)?;
        let chain = Chain::of(parts, ordinal);
        let offered: Vec<&ShapeNode> = template
            .shape_tree()
            .map(|t| {
                t.children
                    .iter()
                    .filter(|n| n.placeholder().is_some())
                    .collect()
            })
            .unwrap_or_default();
        let mut taken = vec![false; offered.len()];
        let mut ops = vec![Operation::SetSlideLayout { ordinal, layout }];
        let mut removed = 0usize;
        for (at, node) in root.children.iter().enumerate() {
            let Some(ph) = node.placeholder() else {
                continue;
            };
            let Some(kind) = role(ph) else {
                continue;
            };
            let candidates = || {
                offered.iter().enumerate().filter(|(i, o)| {
                    !taken.get(*i).copied().unwrap_or(true)
                        && o.placeholder().and_then(role) == Some(kind)
                })
            };
            let found = candidates()
                .find(|(_, o)| o.placeholder().map(|p| p.idx) == Some(ph.idx))
                .or_else(|| candidates().next())
                .map(|(i, o)| (i, *o));
            let place = at.saturating_sub(removed);
            match found {
                Some((i, o)) => {
                    if let Some(t) = taken.get_mut(i) {
                        *t = true;
                    }
                    let mut moved = node.clone();
                    if let (Some(app), Some(new)) = (moved.app.as_mut(), o.placeholder()) {
                        app.placeholder = Some(*new);
                    }
                    moved.transform = None;
                    if moved != *node {
                        ops.push(Operation::DeleteNode { node: node.id });
                        ops.push(Operation::InsertNode {
                            parent: root.id,
                            index: place,
                            node: Box::new(moved),
                        });
                    }
                }
                None if !has_text(node) && node.kind == NodeKind::Shape => {
                    ops.push(Operation::DeleteNode { node: node.id });
                    removed = removed.saturating_add(1);
                }
                None => {
                    // Kept where it was drawn: its inherited place written into it.
                    if node.transform.is_none()
                        && let Some(t) = chain.transform(node).map(|t| t.value)
                    {
                        ops.push(Operation::SetTransform {
                            node: node.id,
                            transform: Some(Transform {
                                offset: Some(t.offset),
                                extent: Some(t.extent),
                                rotation: (t.rotation != 0).then_some(t.rotation),
                                flip_h: t.flip_h.then_some(true),
                                flip_v: t.flip_v.then_some(true),
                                ..Transform::default()
                            }),
                        });
                    }
                }
            }
        }
        // The new layout's text placeholders nothing matched, added empty on top.
        let (mut next_spid, mut next_node) = highest(part);
        let mut end = root.children.len().saturating_sub(removed);
        for (i, o) in offered.iter().enumerate() {
            if taken.get(i).copied().unwrap_or(true) || o.placeholder().and_then(role).is_none() {
                continue;
            }
            next_spid = next_spid.checked_add(1).ok_or(BridgeError::BadArgument)?;
            next_node = next_node.checked_add(1).ok_or(BridgeError::BadArgument)?;
            let mut add = (*o).clone();
            add.id = ObjectId {
                part: ordinal,
                node: next_node,
            };
            if let Some(nv) = add.non_visual.as_mut() {
                nv.spid = Some(next_spid);
            }
            ops.push(Operation::InsertNode {
                parent: root.id,
                index: end,
                node: Box::new(add),
            });
            end = end.saturating_add(1);
        }
        self.typing = None;
        let changed = self
            .history
            .apply(self.deck.parts_mut(), Operation::Batch(ops))?;
        self.reports.remove(&index);
        Ok(changed)
    }

    /// Save step: each slide whose layout changed has its layout relationship pointed
    /// at the new layout part.
    pub(crate) fn layout_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let file = Deck::open(bytes.clone())?;
        let model = self.deck.parts();
        let fs = &file.parts().structure;
        let mut wanted = Vec::new();
        for (i, &o) in fs.slides.iter().enumerate() {
            let name_of = |s: &casual_slide_model::slide::DeckStructure, l: Option<u32>| {
                l.and_then(|l| s.entry(l)).map(|e| e.part.clone())
            };
            let Some(entry) = fs.entry(o) else { continue };
            let now = model
                .structure
                .slide(i)
                .and_then(|m| model.structure.entry(m))
                .and_then(|e| name_of(&model.structure, e.layout));
            let was = name_of(fs, entry.layout);
            if let (Some(now), Some(was)) = (now, was)
                && now != was
            {
                wanted.push((entry.part.clone(), now));
            }
        }
        if wanted.is_empty() {
            return Ok(bytes);
        }
        let mut plan = casual_slide_export::writer::SavePlan::new();
        for (slide, layout) in wanted {
            plan = plan.op(casual_slide_export::writer::Op::SetLayout {
                slide: casual_slide_export::writer::SlideRef::Part(slide),
                layout,
            });
        }
        Self::run(&plan, bytes)
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
