//! Inserting a new shape (`VS-08`): an empty `p:sp`, `p:cxnSp` or `p:pic` with its id
//! and name, put into the shape tree in z-order. Nothing here writes the shape's look or
//! text: the slide part's patch does that afterwards from the model, so a new shape goes
//! through the same patch, read-back and round-trip check as an edited one (ADR-013).

use casual_slide_ooxml::ElementId;

use super::draft::{Draft, set_attr, sibling_name};
use super::error::WriterError;
use super::ops::{NewShape, SlideRef, all_elements};
use super::view::pml_child;

const DML: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// The shape tree's children that are not shapes.
const NOT_SHAPES: [&str; 3] = ["nvGrpSpPr", "grpSpPr", "extLst"];

fn bad(what: &'static str, subject: impl Into<String>) -> WriterError {
    WriterError::BadOperation {
        what,
        subject: subject.into(),
    }
}

impl Draft<'_> {
    pub(crate) fn insert_shape(
        &mut self,
        slide: &SlideRef,
        index: usize,
        shape: &NewShape,
        shape_id: u32,
        name: &str,
    ) -> Result<(), WriterError> {
        let (_, entry) = self.slide(slide)?;
        let part = Self::slide_part(&entry)?;
        let embed = match shape {
            NewShape::Picture(media) => {
                if !media
                    .content_type
                    .to_ascii_lowercase()
                    .starts_with("image/")
                {
                    return Err(bad("a picture that is not an image", part));
                }
                let target =
                    self.media_part(&media.bytes, &media.extension, &media.content_type)?;
                Some(self.reuse_relationship(&part, "image", &target)?)
            }
            NewShape::Shape | NewShape::Connector => None,
        };
        let conformance = self.conformance;
        let dml = conformance.spell(DML);
        let rel_uri = conformance.spell(REL);
        let tree = self.tree(&part)?;
        let sp_tree = pml_child(tree, tree.root(), "cSld")
            .and_then(|c| pml_child(tree, c, "spTree"))
            .ok_or_else(|| bad("the slide has no shape tree", part.clone()))?;
        let wanted = shape_id.to_string();
        let taken = all_elements(tree).into_iter().any(|el| {
            tree.local_name(el) == "cNvPr"
                && tree.plain_attribute(el, "id").as_deref() == Some(wanted.as_str())
        });
        if taken || shape_id == 0 {
            return Err(bad("a shape id the slide already uses", wanted));
        }
        let p = |local: &str| sibling_name(tree, sp_tree, local);
        let (root_local, nv_local, kind_local) = match shape {
            NewShape::Shape => ("sp", "nvSpPr", "cNvSpPr"),
            NewShape::Connector => ("cxnSp", "nvCxnSpPr", "cNvCxnSpPr"),
            NewShape::Picture(_) => ("pic", "nvPicPr", "cNvPicPr"),
        };
        let a = match tree.prefix_for_namespace(sp_tree, dml).as_deref() {
            Some("") => String::new(),
            Some(prefix) => format!("{prefix}:"),
            None => "a:".to_owned(),
        };
        let declare_a = tree.prefix_for_namespace(sp_tree, dml).is_none();
        let r_prefix = tree.prefix_for_namespace(sp_tree, rel_uri);
        let (r_attr, declare_r) = match r_prefix.as_deref() {
            Some(prefix) if !prefix.is_empty() => (format!("{prefix}:embed"), false),
            _ => ("r:embed".to_owned(), true),
        };
        let names = (
            p(root_local),
            p(nv_local),
            p("cNvPr"),
            p(kind_local),
            p("nvPr"),
        );
        let (sp_pr, fill_name) = (p("spPr"), p("blipFill"));
        let root = tree.new_element(&names.0);
        let nv = tree.new_element(&names.1);
        let c_nv = tree.new_element(&names.2);
        set_attr(tree, c_nv, "id", &wanted, &part)?;
        set_attr(tree, c_nv, "name", name, &part)?;
        let kind = tree.new_element(&names.3);
        let app = tree.new_element(&names.4);
        tree.insert_child(nv, None, c_nv);
        tree.insert_child(nv, None, kind);
        tree.insert_child(nv, None, app);
        tree.insert_child(root, None, nv);
        if let Some(rel_id) = embed {
            if declare_a {
                set_attr(tree, root, "xmlns:a", dml, &part)?;
            }
            if declare_r {
                set_attr(tree, root, "xmlns:r", rel_uri, &part)?;
            }
            let fill = tree.new_element(&fill_name);
            let blip = tree.new_element(&format!("{a}blip"));
            set_attr(tree, blip, &r_attr, &rel_id, &part)?;
            let stretch = tree.new_element(&format!("{a}stretch"));
            let rect = tree.new_element(&format!("{a}fillRect"));
            tree.insert_child(stretch, None, rect);
            tree.insert_child(fill, None, blip);
            tree.insert_child(fill, None, stretch);
            tree.insert_child(root, None, fill);
        }
        let properties = tree.new_element(&sp_pr);
        tree.insert_child(root, None, properties);
        // Before the `index`-th shape, or before the tree's own `extLst`.
        let shapes: Vec<ElementId> = tree
            .child_elements(sp_tree)
            .into_iter()
            .filter(|&el| !NOT_SHAPES.contains(&tree.local_name(el)))
            .collect();
        let before = match shapes.get(index) {
            Some(&el) => Some(el),
            None => pml_child(tree, sp_tree, "extLst"),
        };
        tree.insert_child(sp_tree, before, root);
        Ok(())
    }
}
