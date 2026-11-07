//! Copying a shape within a slide (`INV-ID-06`): new ids for everything in the copy, a
//! timing `spid` that follows them, and a new `a16:creationId`, which the host supplies
//! because randomness is never sampled here. The copy keeps every unmodelled attribute.
//! Comments inside a copied shape are not carried; the original keeps them.

use std::collections::BTreeMap;

use casual_slide_ooxml::{ElementId, XmlTree};

use super::draft::{Draft, set_attr};
use super::error::WriterError;
use super::ops::{SlideRef, all_elements};
use super::view::{pml_child, pml_children};

fn bad(what: &'static str, subject: impl Into<String>) -> WriterError {
    WriterError::BadOperation {
        what,
        subject: subject.into(),
    }
}

/// A copy of the subtree at `from`, not yet in the tree. An explicit stack: the depth is
/// bounded when a part is parsed, and the copy cannot exceed it.
fn copy_subtree(tree: &mut XmlTree, from: ElementId, part: &str) -> Result<ElementId, WriterError> {
    let start = |tree: &mut XmlTree, el: ElementId| -> Result<ElementId, WriterError> {
        let name = tree.qualified_name(el).to_owned();
        let new = tree.new_element(&name);
        for (key, value) in tree.declared_attributes(el) {
            set_attr(tree, new, &key, &value, part)?;
        }
        Ok(new)
    };
    let root = start(tree, from)?;
    let mut stack = vec![(from, root)];
    while let Some((source, target)) = stack.pop() {
        let children = tree.child_elements(source);
        if children.is_empty() {
            if let Some(text) = tree.own_text(source).filter(|t| !t.is_empty()) {
                tree.set_text(target, &text);
            }
            continue;
        }
        for child in children {
            let new = start(tree, child)?;
            tree.insert_child(target, None, new);
            stack.push((child, new));
        }
    }
    Ok(root)
}

fn descendants(tree: &XmlTree, root: ElementId) -> Vec<ElementId> {
    let mut out = Vec::new();
    let mut stack = vec![root];
    while let Some(el) = stack.pop() {
        out.push(el);
        stack.extend(tree.child_elements(el));
    }
    out
}

/// The `childTnLst` of the main sequence, where each click's `par` hangs.
fn main_sequence(tree: &XmlTree, root: ElementId) -> Option<ElementId> {
    let timing = pml_child(tree, root, "timing")?;
    descendants(tree, timing).into_iter().find_map(|el| {
        if tree.local_name(el) == "cTn"
            && tree.plain_attribute(el, "nodeType").as_deref() == Some("mainSeq")
        {
            pml_child(tree, el, "childTnLst")
        } else {
            None
        }
    })
}

impl Draft<'_> {
    pub(crate) fn copy_shape(
        &mut self,
        slide: &SlideRef,
        shape_id: u32,
        creation_id: Option<&str>,
    ) -> Result<(), WriterError> {
        let (_, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        let wanted = shape_id.to_string();
        let tree = self.tree(&part)?;
        let elements = all_elements(tree);
        let c_nv = elements
            .iter()
            .copied()
            .find(|&el| {
                tree.local_name(el) == "cNvPr"
                    && tree.plain_attribute(el, "id").as_deref() == Some(wanted.as_str())
            })
            .ok_or_else(|| bad("no such shape", wanted.clone()))?;
        let shape = tree
            .parent_element(c_nv)
            .and_then(|nv| tree.parent_element(nv))
            .ok_or_else(|| bad("no such shape", wanted.clone()))?;
        let parent = tree
            .parent_element(shape)
            .ok_or_else(|| bad("no such shape", wanted.clone()))?;
        let mut next = elements
            .iter()
            .filter(|&&el| tree.local_name(el) == "cNvPr")
            .filter_map(|&el| tree.plain_attribute(el, "id")?.parse::<u64>().ok())
            .max()
            .unwrap_or(1);

        let copy = copy_subtree(tree, shape, &part)?;
        let mut remap: BTreeMap<String, String> = BTreeMap::new();
        for el in descendants(tree, copy) {
            match tree.local_name(el) {
                "cNvPr" => {
                    let Some(old) = tree.plain_attribute(el, "id") else {
                        continue;
                    };
                    // One id in both branches of `mc:AlternateContent` stays one id.
                    let new = match remap.get(&old) {
                        Some(new) => new.clone(),
                        None => {
                            next = next.saturating_add(1);
                            let new = next.to_string();
                            remap.insert(old, new.clone());
                            new
                        }
                    };
                    set_attr(tree, el, "id", &new, &part)?;
                }
                "creationId" if tree.plain_attribute(el, "id").is_some() => {
                    let id = creation_id.ok_or_else(|| {
                        bad("a shape with a creationId needs a new one", wanted.clone())
                    })?;
                    set_attr(tree, el, "id", id, &part)?;
                }
                _ => {}
            }
        }
        let siblings = tree.child_elements(parent);
        let before = siblings
            .iter()
            .position(|&s| s == shape)
            .and_then(|at| siblings.get(at.saturating_add(1)))
            .copied();
        tree.insert_child(parent, before, copy);

        // The animation comes with it, if whole clicks animate only this shape.
        let root = tree.root();
        let mut animated = false;
        if let Some(sequence) = main_sequence(tree, root) {
            let mut next_node = descendants(tree, sequence)
                .into_iter()
                .filter(|&el| tree.local_name(el) == "cTn")
                .filter_map(|el| tree.plain_attribute(el, "id")?.parse::<u64>().ok())
                .max()
                .unwrap_or(0);
            if let Some(timing) = pml_child(tree, root, "timing") {
                for el in descendants(tree, timing) {
                    if tree.local_name(el) == "cTn"
                        && let Some(n) = tree
                            .plain_attribute(el, "id")
                            .and_then(|v| v.parse::<u64>().ok())
                    {
                        next_node = next_node.max(n);
                    }
                }
            }
            for click in pml_children(tree, sequence, "par") {
                let targets: Vec<String> = descendants(tree, click)
                    .into_iter()
                    .filter(|&el| tree.local_name(el) == "spTgt")
                    .filter_map(|el| tree.plain_attribute(el, "spid"))
                    .collect();
                if targets.is_empty() || !targets.iter().all(|t| remap.contains_key(t)) {
                    continue;
                }
                let new = copy_subtree(tree, click, &part)?;
                for el in descendants(tree, new) {
                    match tree.local_name(el) {
                        "spTgt" => {
                            if let Some(mapped) = tree
                                .plain_attribute(el, "spid")
                                .and_then(|old| remap.get(&old).cloned())
                            {
                                set_attr(tree, el, "spid", &mapped, &part)?;
                            }
                        }
                        "cTn" if tree.plain_attribute(el, "id").is_some() => {
                            next_node = next_node.saturating_add(1);
                            set_attr(tree, el, "id", &next_node.to_string(), &part)?;
                        }
                        _ => {}
                    }
                }
                tree.insert_child(sequence, None, new);
                animated = true;
            }
            if animated
                && let Some(builds) = pml_child(tree, root, "timing")
                    .and_then(|timing| pml_child(tree, timing, "bldLst"))
            {
                for build in tree.child_elements(builds) {
                    let Some(mapped) = tree
                        .plain_attribute(build, "spid")
                        .and_then(|old| remap.get(&old).cloned())
                    else {
                        continue;
                    };
                    let new = copy_subtree(tree, build, &part)?;
                    set_attr(tree, new, "spid", &mapped, &part)?;
                    tree.insert_child(builds, None, new);
                }
            }
        }
        for (old, new) in &remap {
            self.log.copied_shapes.push(super::draft::CopiedShape {
                part: part.clone(),
                old: old.clone(),
                new: new.clone(),
                animated,
            });
        }
        Ok(())
    }
}
