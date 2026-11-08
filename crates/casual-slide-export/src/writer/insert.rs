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
const CHART_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/chart";
const TABLE_URI: &str = "http://schemas.openxmlformats.org/drawingml/2006/table";
const CHART_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.drawingml.chart+xml";
const XLSX_CONTENT_TYPE: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
/// The most rows or columns a new table has, as the editor's own bound.
const MAX_NEW_TABLE_SIDE: usize = 75;

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
            NewShape::Shape
            | NewShape::Placeholder { .. }
            | NewShape::Connector
            | NewShape::Table { .. } => None,
            NewShape::Chart { chart, workbook } => {
                let chart_part = self.families.mint("ppt/charts", "chart", "xml")?;
                let book =
                    self.families
                        .mint("ppt/embeddings", "Microsoft_Excel_Worksheet", "xlsx")?;
                self.add_part(&book, Some(XLSX_CONTENT_TYPE), workbook.clone())?;
                let book_rel = self.add_relationship(&chart_part, "package", &book)?;
                let xml = chart.replace(NewShape::WORKBOOK_REL, &book_rel);
                self.add_part(&chart_part, Some(CHART_CONTENT_TYPE), xml.into_bytes())?;
                Some(self.add_relationship(&part, "chart", &chart_part)?)
            }
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
        let pml = sibling_name(tree, sp_tree, "graphicFrame")
            .trim_end_matches("graphicFrame")
            .to_owned();
        let p = |local: &str| sibling_name(tree, sp_tree, local);
        if let NewShape::Table { columns, rows } = shape
            && (columns.is_empty()
                || rows.is_empty()
                || columns.len() > MAX_NEW_TABLE_SIDE
                || rows.len() > MAX_NEW_TABLE_SIDE)
        {
            return Err(bad("a table with no rows or columns, or too many", part));
        }
        let (root_local, nv_local, kind_local) = match shape {
            NewShape::Table { .. } | NewShape::Chart { .. } => {
                let a = match tree.prefix_for_namespace(sp_tree, dml).as_deref() {
                    Some("") => String::new(),
                    Some(prefix) => format!("{prefix}:"),
                    None => "a:".to_owned(),
                };
                let declare_a = tree.prefix_for_namespace(sp_tree, dml).is_none();
                let root = tree.new_element(&format!("{pml}graphicFrame"));
                if declare_a {
                    set_attr(tree, root, "xmlns:a", dml, &part)?;
                }
                let nv = tree.new_element(&format!("{pml}nvGraphicFramePr"));
                let c_nv = tree.new_element(&format!("{pml}cNvPr"));
                set_attr(tree, c_nv, "id", &wanted, &part)?;
                set_attr(tree, c_nv, "name", name, &part)?;
                let kind = tree.new_element(&format!("{pml}cNvGraphicFramePr"));
                let locks = tree.new_element(&format!("{a}graphicFrameLocks"));
                set_attr(tree, locks, "noGrp", "1", &part)?;
                tree.insert_child(kind, None, locks);
                let app = tree.new_element(&format!("{pml}nvPr"));
                for child in [c_nv, kind, app] {
                    tree.insert_child(nv, None, child);
                }
                let xfrm = tree.new_element(&format!("{pml}xfrm"));
                let off = tree.new_element(&format!("{a}off"));
                set_attr(tree, off, "x", "0", &part)?;
                set_attr(tree, off, "y", "0", &part)?;
                let ext = tree.new_element(&format!("{a}ext"));
                set_attr(tree, ext, "cx", "0", &part)?;
                set_attr(tree, ext, "cy", "0", &part)?;
                tree.insert_child(xfrm, None, off);
                tree.insert_child(xfrm, None, ext);
                let graphic = tree.new_element(&format!("{a}graphic"));
                let data = tree.new_element(&format!("{a}graphicData"));
                tree.insert_child(graphic, None, data);
                match (shape, embed.as_deref()) {
                    (NewShape::Table { columns, rows }, _) => {
                        set_attr(tree, data, "uri", TABLE_URI, &part)?;
                        let tbl = tree.new_element(&format!("{a}tbl"));
                        let grid = tree.new_element(&format!("{a}tblGrid"));
                        for w in columns {
                            let col = tree.new_element(&format!("{a}gridCol"));
                            set_attr(tree, col, "w", &w.max(&0).to_string(), &part)?;
                            tree.insert_child(grid, None, col);
                        }
                        tree.insert_child(tbl, None, grid);
                        for h in rows {
                            let tr = tree.new_element(&format!("{a}tr"));
                            set_attr(tree, tr, "h", &h.max(&0).to_string(), &part)?;
                            for _ in columns {
                                let tc = tree.new_element(&format!("{a}tc"));
                                let body = tree.new_element(&format!("{a}txBody"));
                                for local in ["bodyPr", "lstStyle", "p"] {
                                    let child = tree.new_element(&format!("{a}{local}"));
                                    tree.insert_child(body, None, child);
                                }
                                let tc_pr = tree.new_element(&format!("{a}tcPr"));
                                tree.insert_child(tc, None, body);
                                tree.insert_child(tc, None, tc_pr);
                                tree.insert_child(tr, None, tc);
                            }
                            tree.insert_child(tbl, None, tr);
                        }
                        tree.insert_child(data, None, tbl);
                    }
                    (_, Some(rel_id)) => {
                        set_attr(tree, data, "uri", CHART_NS, &part)?;
                        let c = tree.new_element("c:chart");
                        set_attr(tree, c, "xmlns:c", CHART_NS, &part)?;
                        set_attr(tree, c, "xmlns:r", rel_uri, &part)?;
                        set_attr(tree, c, "r:id", rel_id, &part)?;
                        tree.insert_child(data, None, c);
                    }
                    _ => return Err(bad("a chart with no part", part)),
                }
                for child in [nv, xfrm, graphic] {
                    tree.insert_child(root, None, child);
                }
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
                return Ok(());
            }
            NewShape::Shape | NewShape::Placeholder { .. } => ("sp", "nvSpPr", "cNvSpPr"),
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
        let (sp_pr, fill_name, ph_name) = (p("spPr"), p("blipFill"), p("ph"));
        let root = tree.new_element(&names.0);
        let nv = tree.new_element(&names.1);
        let c_nv = tree.new_element(&names.2);
        set_attr(tree, c_nv, "id", &wanted, &part)?;
        set_attr(tree, c_nv, "name", name, &part)?;
        let kind = tree.new_element(&names.3);
        let app = tree.new_element(&names.4);
        tree.insert_child(nv, None, c_nv);
        tree.insert_child(nv, None, kind);
        if let NewShape::Placeholder { kind, idx } = shape {
            if !matches!(kind.as_str(), "dt" | "ftr" | "sldNum" | "hdr") {
                return Err(bad(
                    "a placeholder type this writer does not add",
                    kind.clone(),
                ));
            }
            let ph = tree.new_element(&ph_name);
            set_attr(tree, ph, "type", kind, &part)?;
            if let Some(idx) = idx {
                set_attr(tree, ph, "idx", &idx.to_string(), &part)?;
            }
            tree.insert_child(app, None, ph);
        }
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
