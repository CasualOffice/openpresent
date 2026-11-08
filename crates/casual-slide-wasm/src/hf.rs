//! Insert > Header & Footer (`UX-006`): the date, the footer text and the slide
//! number on a slide, or on every slide with Apply to All, as PowerPoint puts them:
//! each a copy of its layout's `dt`, `ftr` or `sldNum` placeholder on the slide, with
//! no position of its own, so it sits where the layout and master put it. The slide
//! number is a `slidenum` field; the date is fixed text the host supplies (time is
//! supplied, never sampled here). Unticked, the slide's own copy is removed. One undo
//! step. "Don't show on title slide" skips slides on a layout of type `title`.

use casual_slide_model::shape::{ObjectId, PlaceholderType, ShapeNode};
use casual_slide_model::slide::SlideLayoutType;
use casual_slide_transaction::Operation;

use crate::build::{NAMESPACES, TREE_HEAD, highest, read_slide};
use crate::{BridgeError, Viewer};

/// The longest footer or date text taken, in characters.
pub const MAX_FOOTER_CHARS: usize = 255;

/// The field id PowerPoint gives a slide number (any GUID will do; this one is fixed so
/// the same deck saves the same bytes).
const SLIDENUM_FIELD_ID: &str = "{B6F15528-21DE-4FAA-801E-634DDDAF4B2B}";

/// What Header & Footer asks for: each `None` to take that item away.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderFooter {
    /// The date, as fixed text.
    pub date: Option<String>,
    /// The footer text.
    pub footer: Option<String>,
    /// Show the slide number.
    pub slide_number: bool,
    /// Don't show on title slide.
    pub not_on_title: bool,
}

fn escape(text: &str) -> String {
    let mut out = String::new();
    for c in text
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_FOOTER_CHARS)
    {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
    out
}

const KINDS: [(PlaceholderType, &str, &str); 3] = [
    (PlaceholderType::Dt, "dt", "Date Placeholder"),
    (PlaceholderType::Ftr, "ftr", "Footer Placeholder"),
    (
        PlaceholderType::SldNum,
        "sldNum",
        "Slide Number Placeholder",
    ),
];

fn find(tree: &ShapeNode, kind: PlaceholderType) -> Option<&ShapeNode> {
    tree.children
        .iter()
        .find(|n| n.placeholder().and_then(|p| p.placeholder_type) == Some(kind))
}

impl Viewer {
    /// Header & Footer on slide `index`, or on every slide with `all`: one undo step.
    /// `false` when nothing changed.
    ///
    /// # Errors
    ///
    /// `CS-RND-0003` for a slide that does not exist.
    pub fn header_footer(
        &mut self,
        index: usize,
        all: bool,
        hf: &HeaderFooter,
    ) -> Result<bool, BridgeError> {
        let targets: Vec<usize> = if all {
            (0..self.slide_count()).collect()
        } else {
            vec![index]
        };
        let mut ops = Vec::new();
        for i in targets {
            let ordinal = self.slide_ordinal(i)?;
            let parts = self.deck.parts();
            let Some(slide) = parts.get(ordinal) else {
                continue;
            };
            let Some(tree) = slide.shape_tree() else {
                continue;
            };
            let layout = parts
                .structure
                .entry(ordinal)
                .and_then(|e| e.layout)
                .and_then(|l| parts.get(l));
            let title = layout.is_some_and(|l| l.layout_type == Some(SlideLayoutType::Title));
            let layout_tree = layout.and_then(|l| l.shape_tree());
            let (w, h) = self.deck.size();
            let (mut spid, mut node) = highest(slide);
            for (kind, token, base) in KINDS {
                let text = match kind {
                    PlaceholderType::Dt => hf.date.clone(),
                    PlaceholderType::Ftr => hf.footer.clone(),
                    _ => hf.slide_number.then(|| (i + 1).to_string()),
                }
                .filter(|_| !(hf.not_on_title && title));
                let mine = find(tree, kind);
                match (text, mine) {
                    (None, Some(n)) => ops.push(Operation::DeleteNode { node: n.id }),
                    (Some(text), Some(n)) => {
                        // Already there: its text becomes this, as one paragraph.
                        let built = self.footer_node(
                            ordinal, n.id.node, spid, kind, token, base, &text, None, None,
                        )?;
                        if let Some(body) = built.text_body.clone()
                            && let Some(own) = n.text_body.as_ref()
                            && own.paragraphs != body.paragraphs
                        {
                            ops.push(Operation::Restore {
                                object: n.id,
                                first: 0,
                                count: own.paragraphs.len(),
                                paragraphs: body.paragraphs,
                            });
                        }
                    }
                    (Some(text), None) => {
                        // The layout's placeholder of the kind, else the master's, names
                        // the `idx` to match. An absent `idx` means 0, the title's, so a
                        // copy with none to name, or no placeholder of the kind to
                        // inherit from, gets PowerPoint's default place along the bottom
                        // of the slide instead, in proportion to its size.
                        // A slide's placeholder inherits from its layout's only, so with
                        // none there it is placed itself, under PowerPoint's usual idx
                        // (10, 11, 12), which matches nothing to inherit.
                        let found = layout_tree
                            .and_then(|t| find(t, kind))
                            .and_then(|l| l.placeholder().and_then(|p| p.idx));
                        let idx = found.or(Some(match kind {
                            PlaceholderType::Dt => 10,
                            PlaceholderType::Ftr => 11,
                            _ => 12,
                        }));
                        spid = spid.checked_add(1).ok_or(BridgeError::BadArgument)?;
                        node = node.checked_add(1).ok_or(BridgeError::BadArgument)?;
                        let place = found.is_none().then(|| {
                            let (x, cx) = match kind {
                                PlaceholderType::Dt => (6.875, 22.5),
                                PlaceholderType::Ftr => (33.125, 33.75),
                                _ => (70.625, 22.5),
                            };
                            let at = |v: i64, pct: f64| (v as f64 * pct / 100.0).round() as i64;
                            [at(w, x), at(h, 92.68), at(w, cx), at(h, 5.32)]
                        });
                        let built = self.footer_node(
                            ordinal, node, spid, kind, token, base, &text, idx, place,
                        )?;
                        ops.push(Operation::InsertNode {
                            parent: tree.id,
                            index: tree.children.len(),
                            node: Box::new(built),
                        });
                    }
                    (None, None) => {}
                }
            }
        }
        if ops.is_empty() {
            return Ok(false);
        }
        self.edit(Operation::Batch(ops), false)
    }

    /// A slide's copy of a footer placeholder, read from XML as the file would hold it.
    #[allow(clippy::too_many_arguments)] // One call site; the parts of one element.
    fn footer_node(
        &self,
        ordinal: u32,
        node: u32,
        spid: u32,
        kind: PlaceholderType,
        token: &str,
        base: &str,
        text: &str,
        idx: Option<u32>,
        place: Option<[i64; 4]>,
    ) -> Result<ShapeNode, BridgeError> {
        let idx = idx.map_or(String::new(), |i| format!(" idx=\"{i}\""));
        let (sp_pr, p_pr) = match place {
            None => ("<p:spPr/>".to_owned(), String::new()),
            Some([x, y, cx, cy]) => {
                let algn = match kind {
                    PlaceholderType::Dt => "l",
                    PlaceholderType::Ftr => "ctr",
                    _ => "r",
                };
                (
                    format!(
                        "<p:spPr><a:xfrm><a:off x=\"{x}\" y=\"{y}\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm></p:spPr>"
                    ),
                    format!("<a:pPr algn=\"{algn}\"/>"),
                )
            }
        };
        let text = escape(text);
        // Placed by default, the text is PowerPoint's 12 point footer size.
        let size = if place.is_some() { " sz=\"1200\"" } else { "" };
        let run = if kind == PlaceholderType::SldNum {
            format!(
                "<a:fld id=\"{SLIDENUM_FIELD_ID}\" type=\"slidenum\"><a:rPr lang=\"en-US\"{size}/><a:t>{text}</a:t></a:fld>"
            )
        } else {
            format!("<a:r><a:rPr lang=\"en-US\"{size} dirty=\"0\"/><a:t>{text}</a:t></a:r>")
        };
        let name = format!("{base} {}", spid.saturating_sub(1));
        let element = format!(
            "<p:sp><p:nvSpPr><p:cNvPr id=\"{spid}\" name=\"{name}\"/><p:cNvSpPr/><p:nvPr><p:ph type=\"{token}\"{idx}/></p:nvPr></p:nvSpPr>\
             {sp_pr}<p:txBody><a:bodyPr/><a:lstStyle/><a:p>{p_pr}{run}</a:p></p:txBody></p:sp>"
        );
        let xml = format!(
            "<p:sld {NAMESPACES}><p:cSld><p:spTree>{TREE_HEAD}{element}</p:spTree></p:cSld></p:sld>"
        );
        let read = read_slide(xml.as_bytes(), "ppt/slides/openpresent-footer.xml", ordinal)?;
        let mut built = read
            .shape_tree()
            .and_then(|t| t.children.first())
            .cloned()
            .ok_or(BridgeError::Build("CS-WSM-0007"))?;
        built.id = ObjectId {
            part: ordinal,
            node,
        };
        built.children.clear();
        Ok(built)
    }
}
