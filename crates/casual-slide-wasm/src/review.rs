//! Review in the browser (`UX-011`): modern comments, editable notes, and the Selection
//! Pane's list of a slide's shapes.
//!
//! **Comments.** Read at open through Tier 1M's `read_deck_comments`, edited as whole lists
//! through `SetComments` and `SetAuthors`, so every edit is one undo step, and saved by
//! patching each comment part to its list (`patch_threaded_comments`), or attaching a new
//! one built the same way. Ids and times come from the host: nothing here samples a clock
//! or a random source. A mention is the text typed; nothing is sent anywhere.
//!
//! **Notes.** A slide's notes body is edited with the text operations every shape uses. A
//! slide with none gets a notes slide, `AddPart` and `SetNotes` as one undo step, built
//! from the writer's own [`casual_slide_export::writer::new_notes_part`], and the save
//! writes it through `Op::AddNotes` and then patches it as it patches a slide.
//!
//! **Selection Pane.** The slide's top-level shapes, front first as PowerPoint lists them;
//! a name and `@hidden` change through `SetNonVisual`, and the order through
//! `DeleteNode` and `InsertNode`, which a save writes by moving the shapes' elements in
//! `spTree` and nothing else.
//!
//! Document text crosses to the host as JSON strings, for `textContent` only.

use std::collections::{BTreeMap, BTreeSet};

use casual_slide_export::comment::{patch_authors, patch_threaded_comments};
use casual_slide_export::writer::{Op, SavePlan, SlideRef, new_notes_part};
use casual_slide_import::comment::{
    AUTHORS_RELATIONSHIP, ReviewLimits, THREADED_COMMENTS_RELATIONSHIP, read_deck_comments,
};
use casual_slide_import::properties::{Findings, ImportLimits};
use casual_slide_model::comment::{
    Author, AuthorList, CommentAnchor, CommentReply, CommentStatus, DateTime, Moniker,
    ThreadedComment, ThreadedCommentList,
};
use casual_slide_model::presentation::{Coordinate, Point};
use casual_slide_model::properties::Lexical;
use casual_slide_model::shape::{NodeKind, ObjectId, PlaceholderType, ShapeNode};
use casual_slide_model::slide::{PartEntry, SlideKind};
use casual_slide_model::text::{BodyProperties, Paragraph, TextBody, TextItem, TextRun};
use casual_slide_ooxml::{OoxmlLimits, PresentationPackage, XmlTree, resolve_internal};
use casual_slide_render::Deck;
use casual_slide_transaction::{Operation, Review, TxnError};
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

use crate::build::read_slide_kind;
use crate::{BridgeError, Viewer, with_viewer};

/// The longest comment or reply, in characters. PowerPoint's own box stops well short.
pub const MAX_COMMENT_CHARS: usize = 10_000;
/// The longest shape name the Selection Pane takes, in characters.
pub const MAX_RENAME_CHARS: usize = 255;

const COMMENTS_TYPE: &str = "application/vnd.ms-powerpoint.comments+xml";
const AUTHORS_TYPE: &str = "application/vnd.ms-powerpoint.authors+xml";
const P188_NS: &str = "http://schemas.microsoft.com/office/powerpoint/2018/8/main";
const A_NS: &str = "http://schemas.openxmlformats.org/drawingml/2006/main";
const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Where the comment parts and the author list were found, as opened.
#[derive(Debug, Clone, Default)]
pub(crate) struct ReviewParts {
    /// Each slide's `sldIdLst` id, by slide ordinal.
    slide_ids: BTreeMap<u32, u32>,
}

fn first_target(package: &PresentationPackage, source: &str, kinds: &[&str]) -> Option<String> {
    package
        .relationships_of(source)
        .iter()
        .filter(|r| kinds.contains(&r.rel_type.as_str()))
        .find_map(|r| resolve_internal(source, &r.target))
}

/// Each slide part's `sldIdLst` id, by part name.
fn slide_ids(package: &mut PresentationPackage) -> BTreeMap<String, u32> {
    let main = package.main_part().to_owned();
    let rels = package.relationships_of(&main).to_vec();
    let Ok(bytes) = package.read_xml_part(&main) else {
        return BTreeMap::new();
    };
    let Ok(tree) = XmlTree::parse(&bytes, &main, package.limits()) else {
        return BTreeMap::new();
    };
    let mut out = BTreeMap::new();
    let mut stack = vec![tree.root()];
    while let Some(el) = stack.pop() {
        if tree.local_name(el) == "sldId" {
            let id = tree.plain_attribute(el, "id").and_then(|v| v.parse().ok());
            let rel = tree
                .attributes(el, &main)
                .ok()
                .and_then(|all| all.into_iter().find(|(k, _)| k.ends_with(":id")))
                .map(|(_, v)| v);
            if let (Some(id), Some(rel)) = (id, rel)
                && let Some(r) = rels.iter().find(|r| r.id == rel)
                && let Some(part) = resolve_internal(&main, &r.target)
            {
                out.insert(part, id);
            }
            continue;
        }
        stack.extend(tree.child_elements(el));
    }
    out
}

/// The deck's modern comments and authors as opened. What cannot be read opens as nothing,
/// as Tier 1M's reader sets it aside (owner instruction 24).
pub(crate) fn read(deck: &Deck) -> (Review, ReviewParts) {
    let mut review = Review::default();
    let mut parts = ReviewParts::default();
    let Ok(mut package) = PresentationPackage::open(deck.source_bytes(), OoxmlLimits::default())
    else {
        return (review, parts);
    };
    let structure = &deck.parts().structure;
    let ordinal_of = |name: &str| {
        structure
            .parts
            .iter()
            .find(|e| e.kind == SlideKind::Slide && e.part == name)
            .map(|e| e.id.part)
    };
    for (part, id) in slide_ids(&mut package) {
        if let Some(o) = ordinal_of(&part) {
            parts.slide_ids.insert(o, id);
        }
    }
    let slides: Vec<String> = structure
        .slides
        .iter()
        .filter_map(|&o| structure.entry(o))
        .map(|e| e.part.clone())
        .collect();
    let import = ImportLimits::default();
    let mut findings = Findings::new(import.max_findings);
    let mut color = casual_slide_import::color::Findings::default();
    let found = read_deck_comments(
        &mut package,
        &slides,
        &ReviewLimits::default(),
        &import,
        &mut findings,
        &mut color,
    );
    review.authors = found.authors;
    for t in found.threaded {
        if let Some(o) = ordinal_of(&t.slide_part) {
            // A second part on one slide stays as it is, unedited.
            review.comments.entry(o).or_insert(t.list);
        }
    }
    (review, parts)
}

/// Plain text, a paragraph a line, as a comment's body.
fn body_of(text: &str) -> TextBody {
    TextBody {
        properties: Some(BodyProperties::default()),
        list_style: None,
        paragraphs: text
            .split('\n')
            .map(|line| Paragraph {
                properties: None,
                items: if line.is_empty() {
                    Vec::new()
                } else {
                    vec![TextItem::Run(TextRun {
                        properties: None,
                        text: Some(line.to_owned()),
                    })]
                },
                end_properties: None,
            })
            .collect(),
    }
}

fn text_of(body: Option<&TextBody>) -> String {
    let mut out = String::new();
    for (i, p) in body.iter().flat_map(|b| &b.paragraphs).enumerate() {
        if i > 0 {
            out.push('\n');
        }
        for item in &p.items {
            match item {
                TextItem::Run(r) => out.push_str(r.text.as_deref().unwrap_or("")),
                TextItem::Field(f) => out.push_str(f.text.as_deref().unwrap_or("")),
                TextItem::Break(_) => out.push('\n'),
                TextItem::Math(_) | TextItem::Retained(_) => {}
            }
        }
    }
    out
}

/// Text as typed: control characters XML cannot carry dropped, at most
/// [`MAX_COMMENT_CHARS`], and refused when nothing is left.
fn clean(text: &str, max: usize) -> Result<String, BridgeError> {
    let t: String = text
        .chars()
        .filter(|&c| c == '\n' || !c.is_control())
        .take(max)
        .collect();
    let t = t.trim().to_owned();
    if t.is_empty() {
        return Err(BridgeError::BadArgument);
    }
    Ok(t)
}

/// A GUID in braces, as the host minted it. Anything else is refused, so a document
/// string never becomes an id.
fn guid(text: &str) -> Result<String, BridgeError> {
    let inner = text
        .strip_prefix('{')
        .and_then(|t| t.strip_suffix('}'))
        .ok_or(BridgeError::BadArgument)?;
    let groups: Vec<&str> = inner.split('-').collect();
    let lens = [8, 4, 4, 4, 12];
    let ok = groups.len() == lens.len()
        && groups
            .iter()
            .zip(lens)
            .all(|(g, n)| g.len() == n && g.bytes().all(|b| b.is_ascii_hexdigit()));
    if !ok {
        return Err(BridgeError::BadArgument);
    }
    Ok(text.to_ascii_uppercase())
}

fn same_id(a: Option<&String>, b: &str) -> bool {
    a.is_some_and(|a| a.eq_ignore_ascii_case(b))
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .flat_map(char::to_uppercase)
        .take(3)
        .collect()
}

/// A modern-comment moniker's element for a shape of this kind.
fn moniker_kind(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Picture => "picMk",
        NodeKind::Group => "grpSpMk",
        NodeKind::GraphicFrame => "graphicFrameMk",
        NodeKind::Connector => "cxnSpMk",
        _ => "spMk",
    }
}

fn moniker(kind: &str, attributes: &[(&str, String)]) -> Moniker {
    Moniker {
        kind: kind.to_owned(),
        attributes: attributes
            .iter()
            .map(|(k, v)| ((*k).to_owned(), v.clone()))
            .collect(),
    }
}

/// The shape spid a drawing anchor names, if any.
fn anchored_spid(c: &ThreadedComment) -> Option<u32> {
    match &c.anchor {
        Some(CommentAnchor::Drawing(m) | CommentAnchor::Text(m)) => m
            .iter()
            .filter(|m| m.kind != "docMk" && m.kind != "sldMk")
            .find_map(|m| {
                m.attributes
                    .iter()
                    .find(|(k, _)| k == "id")
                    .and_then(|(_, v)| v.parse().ok())
            }),
        _ => None,
    }
}

/// Set each new comment's `sldMk/@sldId`: a slide added this session has its id only once
/// the file lists it.
fn set_slide_id(c: &mut ThreadedComment, id: u32) {
    let list = match &mut c.anchor {
        Some(CommentAnchor::Slide(m) | CommentAnchor::Drawing(m) | CommentAnchor::Text(m)) => m,
        _ => return,
    };
    for m in list.iter_mut().filter(|m| m.kind == "sldMk") {
        for (k, v) in &mut m.attributes {
            if k == "sldId" {
                *v = id.to_string();
            }
        }
    }
}

/// Who is writing: the host's local setting.
#[derive(Debug)]
pub struct Writer<'a> {
    /// The author's id, a GUID the host minted once and keeps.
    pub id: &'a str,
    /// The name the user gave.
    pub name: &'a str,
}

impl Viewer {
    fn review_apply(&mut self, op: Operation) -> Result<bool, BridgeError> {
        self.typing = None;
        let changed = self.history.apply(
            &mut crate::design::DeckDoc(&mut self.deck, &mut self.review),
            op,
        )?;
        Ok(changed)
    }

    /// Slide `index`'s modern comments, as JSON for the Comments pane: each thread's id,
    /// author, time, text, status, the node it is anchored to (or -1), its position in
    /// EMU, and its replies.
    pub fn comments_json(&self, index: usize) -> Result<String, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let authors = self.review.authors.as_ref();
        let who = |id: Option<&String>| -> Value {
            let a = authors.and_then(|l| {
                l.authors
                    .iter()
                    .find(|a| same_id(a.id.as_ref(), id.map_or("", |s| s)))
            });
            json!({
                "name": a.and_then(|a| a.name.clone()).unwrap_or_default(),
                "initials": a.and_then(|a| a.initials.clone()).unwrap_or_default(),
                "id": id.cloned().unwrap_or_default(),
            })
        };
        let spids: BTreeMap<u32, u32> = self
            .deck
            .parts()
            .get(ordinal)
            .and_then(|p| p.shape_tree())
            .map(|root| {
                root.children
                    .iter()
                    .filter_map(|n| Some((n.non_visual.as_ref()?.spid?, n.id.node)))
                    .collect()
            })
            .unwrap_or_default();
        let list = self.review.comments.get(&ordinal);
        let threads: Vec<Value> = list
            .iter()
            .flat_map(|l| &l.comments)
            .map(|c| {
                let node = anchored_spid(c)
                    .and_then(|s| spids.get(&s).copied())
                    .map_or(-1, i64::from);
                let replies: Vec<Value> = c
                    .reply_lst
                    .iter()
                    .flatten()
                    .map(|r| {
                        json!({
                            "id": r.id.clone().unwrap_or_default(),
                            "author": who(r.author_id.as_ref()),
                            "created": r.created.as_ref().map(|d| d.as_str().to_owned()).unwrap_or_default(),
                            "text": text_of(r.tx_body.as_ref()),
                        })
                    })
                    .collect();
                json!({
                    "id": c.id.clone().unwrap_or_default(),
                    "author": who(c.author_id.as_ref()),
                    "created": c.created.as_ref().map(|d| d.as_str().to_owned()).unwrap_or_default(),
                    "text": text_of(c.tx_body.as_ref()),
                    "resolved": matches!(c.status, Some(CommentStatus::Resolved | CommentStatus::Closed)),
                    "node": node,
                    "x": c.pos.map(|p| p.x.0),
                    "y": c.pos.map(|p| p.y.0),
                    "replies": replies,
                })
            })
            .collect();
        Ok(Value::Array(threads).to_string())
    }

    /// How many comment threads each slide has, in slide order, for the thumbnails.
    pub fn comment_counts(&self) -> Vec<u32> {
        self.deck
            .parts()
            .structure
            .slides
            .iter()
            .map(|o| {
                self.review
                    .comments
                    .get(o)
                    .map_or(0, |l| u32::try_from(l.comments.len()).unwrap_or(u32::MAX))
            })
            .collect()
    }

    /// The author list with `writer` in it, if it is not already.
    fn authors_with(&self, writer: &Writer<'_>) -> Result<Option<AuthorList>, BridgeError> {
        let id = guid(writer.id)?;
        let mut list = self.review.authors.clone().unwrap_or_default();
        let name = clean(writer.name, MAX_RENAME_CHARS).unwrap_or_else(|_| "Author".to_owned());
        match list
            .authors
            .iter_mut()
            .find(|a| same_id(a.id.as_ref(), &id))
        {
            Some(a) if a.name.as_deref() == Some(name.as_str()) => return Ok(None),
            Some(a) => {
                a.initials = Some(initials(&name));
                a.user_id = Some(name.clone());
                a.name = Some(name);
            }
            None => list.authors.push(Author {
                id: Some(id),
                initials: Some(initials(&name)),
                user_id: Some(name.clone()),
                provider_id: Some("None".to_owned()),
                name: Some(name),
            }),
        }
        Ok(Some(list))
    }

    fn comments_op(
        &self,
        ordinal: u32,
        writer: &Writer<'_>,
        list: ThreadedCommentList,
    ) -> Result<Operation, BridgeError> {
        let mut ops = Vec::new();
        if let Some(authors) = self.authors_with(writer)? {
            ops.push(Operation::SetAuthors {
                authors: Some(Box::new(authors)),
            });
        }
        ops.push(Operation::SetComments {
            ordinal,
            comments: Some(Box::new(list)),
        });
        Ok(Operation::Batch(ops))
    }

    /// New Comment on slide `index`, on shape `node` or on the slide when `node` is
    /// `None`, at `at` in EMU. One undo step, the author added to the list if new.
    #[allow(clippy::too_many_arguments)] // What a new comment is: each is the host's.
    pub fn add_comment(
        &mut self,
        index: usize,
        node: Option<u32>,
        at: (i64, i64),
        text: &str,
        writer: &Writer<'_>,
        id: &str,
        created: &str,
    ) -> Result<bool, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let text = clean(text, MAX_COMMENT_CHARS)?;
        let id = guid(id)?;
        let created = DateTime::parse_lexical(created).ok_or(BridgeError::BadArgument)?;
        let slide_id = self
            .review_parts
            .slide_ids
            .get(&ordinal)
            .copied()
            .unwrap_or(0);
        let base = vec![
            moniker("docMk", &[]),
            moniker(
                "sldMk",
                &[("cId", "0".to_owned()), ("sldId", slide_id.to_string())],
            ),
        ];
        let anchor = match node {
            None => CommentAnchor::Slide(base),
            Some(n) => {
                let shape = self
                    .deck
                    .parts()
                    .get(ordinal)
                    .and_then(|p| {
                        p.node(ObjectId {
                            part: ordinal,
                            node: n,
                        })
                    })
                    .ok_or(TxnError::NoSuchObject(ObjectId {
                        part: ordinal,
                        node: n,
                    }))?;
                let nv = shape.non_visual.as_ref();
                let spid = nv.and_then(|v| v.spid).ok_or(BridgeError::BadArgument)?;
                let mut attrs = vec![("id", spid.to_string())];
                if let Some(c) = nv.and_then(|v| v.creation_id.clone()) {
                    attrs.push(("creationId", c));
                }
                let mut m = base;
                m.push(moniker(moniker_kind(shape.kind), &attrs));
                CommentAnchor::Drawing(m)
            }
        };
        let clamp = |v: i64| Coordinate(v.clamp(0, crate::POSITIVE_COORDINATE_MAX));
        let comment = ThreadedComment {
            anchor: Some(anchor),
            pos: Some(Point {
                x: clamp(at.0),
                y: clamp(at.1),
            }),
            tx_body: Some(body_of(&text)),
            id: Some(id),
            author_id: Some(guid(writer.id)?),
            created: Some(created),
            ..ThreadedComment::default()
        };
        let mut list = self
            .review
            .comments
            .get(&ordinal)
            .cloned()
            .unwrap_or_default();
        list.comments.push(comment);
        let op = self.comments_op(ordinal, writer, list)?;
        self.review_apply(op)
    }

    fn edit_thread(
        &mut self,
        index: usize,
        thread: &str,
        writer: Option<&Writer<'_>>,
        f: impl FnOnce(&mut ThreadedCommentList, usize) -> Result<(), BridgeError>,
    ) -> Result<bool, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let mut list = self
            .review
            .comments
            .get(&ordinal)
            .cloned()
            .ok_or(BridgeError::BadArgument)?;
        let at = list
            .comments
            .iter()
            .position(|c| same_id(c.id.as_ref(), thread))
            .ok_or(BridgeError::BadArgument)?;
        f(&mut list, at)?;
        let op = match writer {
            Some(w) => self.comments_op(ordinal, w, list)?,
            None => Operation::SetComments {
                ordinal,
                comments: Some(Box::new(list)),
            },
        };
        self.review_apply(op)
    }

    /// Reply to thread `thread` of slide `index`. One undo step.
    pub fn reply_comment(
        &mut self,
        index: usize,
        thread: &str,
        text: &str,
        writer: &Writer<'_>,
        id: &str,
        created: &str,
    ) -> Result<bool, BridgeError> {
        let reply = CommentReply {
            tx_body: Some(body_of(&clean(text, MAX_COMMENT_CHARS)?)),
            id: Some(guid(id)?),
            author_id: Some(guid(writer.id)?),
            created: Some(DateTime::parse_lexical(created).ok_or(BridgeError::BadArgument)?),
            ..CommentReply::default()
        };
        self.edit_thread(index, thread, Some(writer), |list, at| {
            let c = list.comments.get_mut(at).ok_or(BridgeError::BadArgument)?;
            c.reply_lst.get_or_insert_with(Vec::new).push(reply);
            Ok(())
        })
    }

    /// Resolve thread `thread`, or reopen it. Reopened, `@status` is removed, as a thread
    /// that never had one is active.
    pub fn resolve_comment(
        &mut self,
        index: usize,
        thread: &str,
        resolved: bool,
    ) -> Result<bool, BridgeError> {
        self.edit_thread(index, thread, None, |list, at| {
            let c = list.comments.get_mut(at).ok_or(BridgeError::BadArgument)?;
            c.status = resolved.then_some(CommentStatus::Resolved);
            Ok(())
        })
    }

    /// Delete thread `thread`, or only its reply `reply` when one is named.
    pub fn delete_comment(
        &mut self,
        index: usize,
        thread: &str,
        reply: Option<&str>,
    ) -> Result<bool, BridgeError> {
        self.edit_thread(index, thread, None, |list, at| {
            match reply {
                None => {
                    list.comments.remove(at);
                }
                Some(r) => {
                    let c = list.comments.get_mut(at).ok_or(BridgeError::BadArgument)?;
                    let replies = c.reply_lst.as_mut().ok_or(BridgeError::BadArgument)?;
                    let k = replies
                        .iter()
                        .position(|x| same_id(x.id.as_ref(), r))
                        .ok_or(BridgeError::BadArgument)?;
                    replies.remove(k);
                }
            }
            Ok(())
        })
    }

    /// Save step: every comment list and the author list that changed, patched into the
    /// part they came from, or attached as a new part (`UX-011`).
    pub(crate) fn review_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        if self.review == self.original_review {
            return Ok(bytes);
        }
        let save = |code: &'static str| BridgeError::Save(code);
        let file = Deck::open(bytes.clone())?;
        let mut package = PresentationPackage::open(bytes.clone(), OoxmlLimits::default())
            .map_err(|e| save(e.code()))?;
        let strict = package.is_strict();
        let xml = *package.limits();
        let limits = ReviewLimits::default();
        let import = ImportLimits::default();
        let ids = slide_ids(&mut package);
        let model = self.deck.parts();
        let mut plan = SavePlan::new();
        let mut any = false;
        for (i, &o) in model.structure.slides.iter().enumerate() {
            let want = self.review.comments.get(&o);
            if want == self.original_review.comments.get(&o) {
                continue;
            }
            let Some(name) = file
                .parts()
                .structure
                .slide(i)
                .and_then(|f| file.parts().structure.entry(f))
                .map(|e| e.part.clone())
            else {
                return Err(save("CS-WSM-0009"));
            };
            let mut list = want.cloned().unwrap_or_default();
            let known: BTreeSet<String> = self
                .original_review
                .comments
                .get(&o)
                .iter()
                .flat_map(|l| &l.comments)
                .filter_map(|c| c.id.clone())
                .collect();
            if let Some(&id) = ids.get(&name) {
                for c in &mut list.comments {
                    if !c.id.as_ref().is_some_and(|i| known.contains(i)) {
                        set_slide_id(c, id);
                    }
                }
            }
            match first_target(&package, &name, THREADED_COMMENTS_RELATIONSHIP) {
                Some(part) => {
                    let rels = package.relationships_of(&part).to_vec();
                    let b = package.read_xml_part(&part).map_err(|e| save(e.code()))?;
                    let mut tree = XmlTree::parse(&b, &part, &xml).map_err(|e| save(e.code()))?;
                    patch_threaded_comments(
                        &mut tree, &part, strict, &rels, &list, &limits, &import, &xml,
                    )
                    .map_err(|e| save(e.code()))?;
                    plan = plan.replace_part(part, tree.to_bytes());
                }
                None if list.comments.is_empty() => continue,
                None => {
                    let empty = format!(
                        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
                         <p188:cmLst xmlns:a=\"{A_NS}\" xmlns:r=\"{R_NS}\" xmlns:p188=\"{P188_NS}\"/>"
                    );
                    let part = "ppt/comments/new.xml";
                    let mut tree =
                        XmlTree::parse(empty.as_bytes(), part, &xml).map_err(|e| save(e.code()))?;
                    patch_threaded_comments(
                        &mut tree,
                        part,
                        strict,
                        &[],
                        &list,
                        &limits,
                        &import,
                        &xml,
                    )
                    .map_err(|e| save(e.code()))?;
                    plan = plan.op(Op::AttachPart {
                        source: name.clone(),
                        kind: THREADED_COMMENTS_RELATIONSHIP
                            .first()
                            .copied()
                            .unwrap_or_default()
                            .to_owned(),
                        dir: "ppt/comments".to_owned(),
                        stem: "modernComment_".to_owned(),
                        extension: "xml".to_owned(),
                        content_type: COMMENTS_TYPE.to_owned(),
                        bytes: tree.to_bytes(),
                    });
                }
            }
            any = true;
        }
        if self.review.authors != self.original_review.authors {
            let list = self.review.authors.clone().unwrap_or_default();
            let main = package.main_part().to_owned();
            match first_target(&package, &main, AUTHORS_RELATIONSHIP) {
                Some(part) => {
                    let b = package.read_xml_part(&part).map_err(|e| save(e.code()))?;
                    let mut tree = XmlTree::parse(&b, &part, &xml).map_err(|e| save(e.code()))?;
                    patch_authors(&mut tree, &part, strict, &list, &limits, &import, &xml)
                        .map_err(|e| save(e.code()))?;
                    plan = plan.replace_part(part, tree.to_bytes());
                }
                None => {
                    let empty = format!(
                        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
                         <p188:authorLst xmlns:a=\"{A_NS}\" xmlns:r=\"{R_NS}\" xmlns:p188=\"{P188_NS}\"/>"
                    );
                    let part = "ppt/authors.xml";
                    let mut tree =
                        XmlTree::parse(empty.as_bytes(), part, &xml).map_err(|e| save(e.code()))?;
                    patch_authors(&mut tree, part, strict, &list, &limits, &import, &xml)
                        .map_err(|e| save(e.code()))?;
                    plan = plan.op(Op::AttachPart {
                        source: main,
                        kind: AUTHORS_RELATIONSHIP
                            .first()
                            .copied()
                            .unwrap_or_default()
                            .to_owned(),
                        dir: "ppt".to_owned(),
                        stem: "authors".to_owned(),
                        extension: "xml".to_owned(),
                        content_type: AUTHORS_TYPE.to_owned(),
                        bytes: tree.to_bytes(),
                    });
                }
            }
            any = true;
        }
        if !any {
            return Ok(bytes);
        }
        Self::run(&plan, bytes)
    }

    /* ── Notes ─────────────────────────────────────────────────────────── */

    /// Slide `index`'s notes body placeholder, `[part, node]`. With `create`, a slide with
    /// no notes slide, or one with no body, gets a notes slide first, one undo step;
    /// without, such a slide gives `None`.
    pub fn notes_body(
        &mut self,
        index: usize,
        create: bool,
    ) -> Result<Option<ObjectId>, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        if let Some(found) = self.find_notes_body(ordinal) {
            return Ok(Some(found));
        }
        let linked = self
            .deck
            .parts()
            .structure
            .entry(ordinal)
            .and_then(|e| e.notes);
        if !create || linked.is_some() {
            return Ok(None);
        }
        let notes = self.next_part_ordinal()?;
        let name = format!("ppt/notesSlides/openpresent-new-{notes}.xml");
        let xml = new_notes_part(self.deck.conformance());
        let part = read_slide_kind(&xml, &name, notes, SlideKind::Notes)?;
        let entry = PartEntry {
            id: ObjectId::part(notes),
            kind: SlideKind::Notes,
            part: name,
            layout: None,
            master: self.deck.parts().structure.notes_master,
            notes: None,
            theme: None,
        };
        self.review_apply(Operation::Batch(vec![
            Operation::AddPart {
                entry: Box::new(entry),
                part: Box::new(part),
            },
            Operation::SetNotes {
                ordinal,
                notes: Some(notes),
            },
        ]))?;
        Ok(self.find_notes_body(ordinal))
    }

    fn next_part_ordinal(&self) -> Result<u32, BridgeError> {
        u32::try_from(self.deck.parts().parts.len()).map_err(|_| BridgeError::BadArgument)
    }

    fn find_notes_body(&self, slide: u32) -> Option<ObjectId> {
        let parts = self.deck.parts();
        let notes = parts.structure.entry(slide)?.notes?;
        let root = parts.get(notes)?.shape_tree()?;
        root.descendants()
            .find(|n| {
                n.text_body.is_some()
                    && n.placeholder()
                        .is_some_and(|p| p.placeholder_type == Some(PlaceholderType::Body))
            })
            .map(|n| n.id)
    }

    /// A text body's paragraphs as JSON for the notes editor: each a list of
    /// `[text, bits]`, bits 1 bold, 2 italic, 4 underline, as the run states them. A line
    /// break is `"\u{b}"`; an equation or element this build does not model counts nothing.
    pub fn body_json(&self, object: ObjectId) -> Result<String, BridgeError> {
        let node = self
            .deck
            .parts()
            .get(object.part)
            .and_then(|p| p.node(object))
            .ok_or(TxnError::NoSuchObject(object))?;
        let body = node.text_body.as_deref().ok_or(BridgeError::NoText)?;
        let bits = |p: Option<&casual_slide_model::text::CharacterProperties>| {
            let p = p.cloned().unwrap_or_default();
            u32::from(p.bold == Some(true))
                | (u32::from(p.italic == Some(true)) << 1)
                | (u32::from(
                    p.underline
                        .is_some_and(|u| u != casual_slide_model::text::UnderlineType::None),
                ) << 2)
        };
        let paragraphs: Vec<Value> = body
            .paragraphs
            .iter()
            .map(|p| {
                let runs: Vec<Value> = p
                    .items
                    .iter()
                    .filter_map(|item| match item {
                        TextItem::Run(r) => Some(json!([
                            r.text.as_deref().unwrap_or(""),
                            bits(r.properties.as_deref())
                        ])),
                        TextItem::Field(f) => Some(json!([
                            f.text.as_deref().unwrap_or(""),
                            bits(f.properties.as_deref())
                        ])),
                        TextItem::Break(_) => Some(json!(["\u{b}", 0])),
                        TextItem::Math(_) | TextItem::Retained(_) => None,
                    })
                    .collect();
                Value::Array(runs)
            })
            .collect();
        Ok(Value::Array(paragraphs).to_string())
    }

    /// Save step: a notes slide added this session is written with `Op::AddNotes`, empty,
    /// and then patched with the slides (`patch_pass`). One left empty is not written.
    pub(crate) fn notes_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let original = self.original_count();
        let model = self.deck.parts();
        let file = Deck::open(bytes.clone())?;
        let mut plan = SavePlan::new();
        let mut any = false;
        for (i, &o) in model.structure.slides.iter().enumerate() {
            let Some(n) = model.structure.entry(o).and_then(|e| e.notes) else {
                continue;
            };
            if n < original {
                continue;
            }
            let empty = self
                .find_notes_body(o)
                .and_then(|b| model.get(b.part)?.node(b))
                .and_then(|n| n.text_body.as_deref())
                .is_none_or(|b| text_of(Some(b)).trim().is_empty());
            let fs = file.parts().structure.slide(i);
            let has = fs
                .and_then(|f| file.parts().structure.entry(f))
                .and_then(|e| e.notes)
                .is_some();
            if empty || has {
                continue;
            }
            plan = plan.op(Op::AddNotes {
                slide: SlideRef::Index(i),
                text: String::new(),
            });
            any = true;
        }
        if !any {
            return Ok(bytes);
        }
        Self::run(&plan, bytes)
    }

    /// The notes slides of the file and of the model, paired by their slides' places, for
    /// `patch_pass`.
    pub(crate) fn notes_pairs<'a>(
        &'a self,
        file: &'a casual_slide_model::slide::SlideParts,
    ) -> Vec<(
        String,
        &'a casual_slide_model::slide::SlidePart,
        &'a casual_slide_model::slide::SlidePart,
    )> {
        let model = self.deck.parts();
        let mut out = Vec::new();
        for (i, &o) in model.structure.slides.iter().enumerate() {
            let m = model
                .structure
                .entry(o)
                .and_then(|e| e.notes)
                .and_then(|n| model.get(n));
            let f = file
                .structure
                .slide(i)
                .and_then(|s| file.structure.entry(s))
                .and_then(|e| e.notes)
                .and_then(|n| Some((file.structure.entry(n)?, file.get(n)?)));
            if let (Some(m), Some((entry, f))) = (m, f) {
                out.push((entry.part.clone(), f, m));
            }
        }
        out
    }

    /* ── The Selection Pane ────────────────────────────────────────────── */

    /// Slide `index`'s top-level shapes for the Selection Pane, front first: each one's
    /// node, name, whether it is hidden, and its kind.
    pub fn selection_list(&self, index: usize) -> Result<String, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let Some(root) = self.deck.parts().get(ordinal).and_then(|p| p.shape_tree()) else {
            return Ok("[]".to_owned());
        };
        let items: Vec<Value> = root
            .children
            .iter()
            .rev()
            .map(|n| {
                let nv = n.non_visual.as_ref();
                json!({
                    "node": n.id.node,
                    "name": nv.and_then(|v| v.name.clone()).unwrap_or_default(),
                    "hidden": nv.and_then(|v| v.hidden) == Some(true),
                    "kind": moniker_kind(n.kind),
                })
            })
            .collect();
        Ok(Value::Array(items).to_string())
    }

    fn listed_shape(&self, ordinal: u32, node: u32) -> Result<&ShapeNode, BridgeError> {
        let id = ObjectId {
            part: ordinal,
            node,
        };
        self.deck
            .parts()
            .get(ordinal)
            .and_then(|p| p.shape_tree())
            .and_then(|r| r.children.iter().find(|c| c.id == id))
            .ok_or(BridgeError::Edit(TxnError::NoSuchObject(id)))
    }

    /// Rename shape `node`, or hide or show it: one undo step.
    pub fn set_shape_shown(
        &mut self,
        index: usize,
        node: u32,
        name: Option<&str>,
        hidden: Option<bool>,
    ) -> Result<bool, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let shape = self.listed_shape(ordinal, node)?;
        let mut nv = shape.non_visual.clone().ok_or(BridgeError::CannotDelete)?;
        if let Some(name) = name {
            let name: String = name
                .chars()
                .filter(|c| !c.is_control())
                .take(MAX_RENAME_CHARS)
                .collect();
            nv.name = Some(name.trim().to_owned());
        }
        if let Some(hidden) = hidden {
            // Shown again, `@hidden` is removed: absent stays absent.
            nv.hidden = hidden.then_some(true);
        }
        self.review_apply(Operation::SetNonVisual {
            node: ObjectId {
                part: ordinal,
                node,
            },
            non_visual: Some(Box::new(nv)),
        })
    }

    /// Move shape `node` to place `to` in z-order, 0 the back: one undo step.
    pub fn move_shape_z(
        &mut self,
        index: usize,
        node: u32,
        to: usize,
    ) -> Result<bool, BridgeError> {
        let ordinal = self.slide_ordinal(index)?;
        let root = self
            .deck
            .parts()
            .get(ordinal)
            .and_then(|p| p.shape_tree())
            .ok_or(BridgeError::BadArgument)?;
        let id = ObjectId {
            part: ordinal,
            node,
        };
        let from = root
            .children
            .iter()
            .position(|c| c.id == id)
            .ok_or(TxnError::NoSuchObject(id))?;
        let to = to.min(root.children.len().saturating_sub(1));
        if from == to {
            return Ok(false);
        }
        let moved = root
            .children
            .get(from)
            .cloned()
            .ok_or(TxnError::NoSuchObject(id))?;
        if moved.non_visual.as_ref().and_then(|n| n.spid).is_none() {
            return Err(BridgeError::CannotDelete);
        }
        let parent = root.id;
        self.review_apply(Operation::Batch(vec![
            Operation::DeleteNode { node: id },
            Operation::InsertNode {
                parent,
                index: to,
                node: Box::new(moved),
            },
        ]))
    }

    /// Save step: each slide's shapes put in the model's z-order, by moving their elements
    /// within `spTree`. Nothing else in the part changes.
    pub(crate) fn order_pass(&self, bytes: Vec<u8>) -> Result<Vec<u8>, BridgeError> {
        let save = |code: &'static str| BridgeError::Save(code);
        let file = Deck::open(bytes.clone())?;
        let mut wanted = Vec::new();
        for (name, f, m) in self.pairs(file.parts())? {
            let (Some(ft), Some(mt)) = (f.shape_tree(), m.shape_tree()) else {
                continue;
            };
            let in_file: BTreeSet<u32> =
                ft.children.iter().filter_map(crate::build::spid).collect();
            let model_order: Vec<u32> = mt
                .children
                .iter()
                .filter_map(crate::build::spid)
                .filter(|s| in_file.contains(s))
                .collect();
            let file_order: Vec<u32> = ft
                .children
                .iter()
                .filter_map(crate::build::spid)
                .filter(|s| model_order.contains(s))
                .collect();
            if model_order != file_order {
                wanted.push((name, model_order));
            }
        }
        if wanted.is_empty() {
            return Ok(bytes);
        }
        let mut package = PresentationPackage::open(bytes.clone(), OoxmlLimits::default())
            .map_err(|e| save(e.code()))?;
        let xml = *package.limits();
        let mut plan = SavePlan::new();
        for (name, order) in wanted {
            let b = package.read_xml_part(&name).map_err(|e| save(e.code()))?;
            let mut tree = XmlTree::parse(&b, &name, &xml).map_err(|e| save(e.code()))?;
            reorder_tree(&mut tree, &order).ok_or(save("CS-WSM-0009"))?;
            plan = plan.replace_part(name, tree.to_bytes());
        }
        Self::run(&plan, bytes)
    }
}

/// The `cNvPr/@id` of a top-level shape element.
fn element_spid(tree: &XmlTree, el: casual_slide_ooxml::ElementId) -> Option<u32> {
    let nv = tree
        .child_elements(el)
        .into_iter()
        .find(|&c| tree.local_name(c).starts_with("nv"))?;
    let c = tree
        .child_elements(nv)
        .into_iter()
        .find(|&c| tree.local_name(c) == "cNvPr")?;
    tree.plain_attribute(c, "id")?.parse().ok()
}

/// Put `spTree`'s shape elements named in `order` in that order, in the places they held.
fn reorder_tree(tree: &mut XmlTree, order: &[u32]) -> Option<()> {
    let root = tree.root();
    let c_sld = tree
        .child_elements(root)
        .into_iter()
        .find(|&c| tree.local_name(c) == "cSld")?;
    let sp_tree = tree
        .child_elements(c_sld)
        .into_iter()
        .find(|&c| tree.local_name(c) == "spTree")?;
    let kids = tree.child_elements(sp_tree);
    let by_spid: BTreeMap<u32, casual_slide_ooxml::ElementId> = kids
        .iter()
        .filter_map(|&k| Some((element_spid(tree, k)?, k)))
        .filter(|(s, _)| order.contains(s))
        .collect();
    if by_spid.len() != order.len() {
        return None;
    }
    let mut next = order.iter().filter_map(|s| by_spid.get(s).copied());
    let final_order: Vec<_> = kids
        .iter()
        .map(|&k| {
            if element_spid(tree, k).is_some_and(|s| by_spid.contains_key(&s)) {
                next.next().unwrap_or(k)
            } else {
                k
            }
        })
        .collect();
    for &k in &kids {
        tree.remove_child(sp_tree, k);
    }
    for k in final_order {
        tree.insert_child(sp_tree, None, k);
    }
    Some(())
}

/* ── The bridge ─────────────────────────────────────────────────────────── */

/// Slide `index`'s comment threads, JSON (`UX-011`).
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn slide_comments(index: u32) -> Result<String, JsError> {
    with_viewer(|v| v.comments_json(index as usize))
}

/// How many comment threads each slide has, in slide order.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck.
#[wasm_bindgen]
pub fn comment_counts() -> Result<Vec<u32>, JsError> {
    with_viewer(|v| Ok(v.comment_counts()))
}

/// New Comment on slide `index`: on shape `node`, or the slide with a negative `node`, at
/// `x`, `y` in EMU, by the author `author_id` named `author`, with the host's `id` and
/// `created` time. One undo step.
///
/// # Errors
///
/// `CS-WSM-0006` for empty text, an id that is not a GUID or a time that is not
/// `xsd:dateTime`; the edit's own code.
#[allow(clippy::too_many_arguments)] // A comment's parts, each from the host.
#[wasm_bindgen]
pub fn add_comment(
    index: u32,
    node: i32,
    x: f64,
    y: f64,
    text: &str,
    author_id: &str,
    author: &str,
    id: &str,
    created: &str,
) -> Result<bool, JsError> {
    with_viewer(|v| {
        let w = Writer {
            id: author_id,
            name: author,
        };
        let node = u32::try_from(node).ok();
        v.add_comment(
            index as usize,
            node,
            (x as i64, y as i64),
            text,
            &w,
            id,
            created,
        )
    })
}

/// Reply to thread `thread` of slide `index`. One undo step.
///
/// # Errors
///
/// As [`add_comment`], and `CS-WSM-0006` for a thread the slide does not hold.
#[allow(clippy::too_many_arguments)] // A reply's parts, each from the host.
#[wasm_bindgen]
pub fn reply_comment(
    index: u32,
    thread: &str,
    text: &str,
    author_id: &str,
    author: &str,
    id: &str,
    created: &str,
) -> Result<bool, JsError> {
    with_viewer(|v| {
        let w = Writer {
            id: author_id,
            name: author,
        };
        v.reply_comment(index as usize, thread, text, &w, id, created)
    })
}

/// Resolve thread `thread`, or reopen it. One undo step.
///
/// # Errors
///
/// `CS-WSM-0006` for a thread the slide does not hold.
#[wasm_bindgen]
pub fn resolve_comment(index: u32, thread: &str, resolved: bool) -> Result<bool, JsError> {
    with_viewer(|v| v.resolve_comment(index as usize, thread, resolved))
}

/// Delete thread `thread`, or its reply `reply` when that is not empty. One undo step.
///
/// # Errors
///
/// `CS-WSM-0006` for a thread or reply the slide does not hold.
#[wasm_bindgen]
pub fn delete_comment(index: u32, thread: &str, reply: &str) -> Result<bool, JsError> {
    with_viewer(|v| v.delete_comment(index as usize, thread, (!reply.is_empty()).then_some(reply)))
}

/// Slide `index`'s notes body, `[part, node]`, made first with `create`; empty when there
/// is none.
///
/// # Errors
///
/// `CS-WSM-0007` when the notes slide could not be built; the edit's own code.
#[wasm_bindgen]
pub fn notes_body(index: u32, create: bool) -> Result<Vec<u32>, JsError> {
    with_viewer(|v| {
        Ok(v.notes_body(index as usize, create)?
            .map(|o| vec![o.part, o.node])
            .unwrap_or_default())
    })
}

/// A text body's paragraphs and runs, JSON, for the notes editor.
///
/// # Errors
///
/// `CS-WSM-0005` when the shape has no text.
#[wasm_bindgen]
pub fn text_body_runs(part: u32, node: u32) -> Result<String, JsError> {
    with_viewer(|v| v.body_json(ObjectId { part, node }))
}

/// Slide `index`'s shapes for the Selection Pane, front first, JSON.
///
/// # Errors
///
/// `CS-WSM-0001` with no deck; `CS-RND-0003` for a slide that does not exist.
#[wasm_bindgen]
pub fn selection_list(index: u32) -> Result<String, JsError> {
    with_viewer(|v| v.selection_list(index as usize))
}

/// Rename shape `node` of slide `index`. One undo step.
///
/// # Errors
///
/// The edit's own code.
#[wasm_bindgen]
pub fn rename_shape(index: u32, node: u32, name: &str) -> Result<bool, JsError> {
    with_viewer(|v| v.set_shape_shown(index as usize, node, Some(name), None))
}

/// Hide shape `node` of slide `index`, or show it. One undo step.
///
/// # Errors
///
/// The edit's own code.
#[wasm_bindgen]
pub fn set_shape_hidden(index: u32, node: u32, hidden: bool) -> Result<bool, JsError> {
    with_viewer(|v| v.set_shape_shown(index as usize, node, None, Some(hidden)))
}

/// Move shape `node` of slide `index` to place `to` in z-order, 0 the back. One undo step.
///
/// # Errors
///
/// The edit's own code.
#[wasm_bindgen]
pub fn move_shape_z(index: u32, node: u32, to: u32) -> Result<bool, JsError> {
    with_viewer(|v| v.move_shape_z(index as usize, node, to as usize))
}

#[cfg(test)]
#[path = "review_tests.rs"]
mod tests;
