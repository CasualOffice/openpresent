//! Patch comment parts (Tier 1M). The model before and after becomes a description of the
//! elements it stands for, and only what differs touches the tree, as for every other part
//! here. So what the model does not hold stays where it is: an anchor's monikers, an
//! `extLst` entry nothing here names, an attribute this reader reported.
//!
//! Comments, replies and authors match by `id` where every one has an id of its own, and by
//! place otherwise, so taking one away leaves the others' elements alone. An anchor can be
//! written for a new comment. A changed anchor on one that exists is refused as `CS-EXP-0009`,
//! as what it points at is the part of it this tier only reads.
//!
//! A new `extLst` entry needs its `uri`. `presenceInfo`'s is in `fixtures/real`;
//! `threadingInfo`'s is Inferred. No deck here holds a modern comment, so those of reactions
//! and tasks are not known: an entry can be patched where a file has one, and adding the first
//! is refused as `CS-EXP-0009`, until `PV-01` or a real deck says what to write.

use std::collections::HashMap;

use casual_slide_import::color::Findings as ColorFindings;
use casual_slide_import::comment::{
    ReviewLimits, import_authors, import_comment_authors, import_comments,
    import_threaded_comments,
};
use casual_slide_import::properties::{Elem, Findings, ImportLimits, ns};
use casual_slide_model::comment::{
    Author, AuthorList, Comment, CommentAnchor, CommentAuthor, CommentAuthorList, CommentList,
    CommentReply, CommentThreading, Moniker, PresenceInfo, Reactions, TaskAction, TaskDetails,
    ThreadedComment, ThreadedCommentList,
};
use casual_slide_model::presentation::Point;
use casual_slide_model::text::TextBody;
use casual_slide_ooxml::{OoxmlLimits, Relationship, XmlTree};

use crate::drawing::{Node, Space, patch};
use crate::properties::{ExportError, Writer, verify};
use crate::text::patch_text_body;

type Result<T = ()> = std::result::Result<T, ExportError>;

/// `p14:`.
pub(crate) const P14: Space = Space::P14;
/// `p15:`.
pub(crate) const P15: Space = Space::P15;
/// `p188:`.
pub(crate) const P188: Space = Space::Other(casual_slide_import::comment::P188, "p188");
/// `p223:`.
pub(crate) const P223: Space = Space::Other(casual_slide_import::comment::P223, "p223");
/// `p228:`.
pub(crate) const P228: Space = Space::Other(casual_slide_import::comment::P228, "p228");
/// `p166:`.
pub(crate) const P166: Space = Space::Other(casual_slide_import::section::P166, "p166");
/// `pslz:`.
pub(crate) const PSLZ: Space = Space::Other(casual_slide_import::section::PSLZ, "pslz");
/// `psez:`.
pub(crate) const PSEZ: Space = Space::Other(casual_slide_import::section::PSEZ, "psez");
/// `psuz:`.
pub(crate) const PSUZ: Space = Space::Other(casual_slide_import::section::PSUZ, "psuz");
/// `p16:`.
pub(crate) const P16: Space = Space::Other(casual_slide_import::section::P16, "p16");
/// `p184:`.
pub(crate) const P184: Space = Space::Other(casual_slide_import::section::P184, "p184");
/// `acs:`.
pub(crate) const ACS: Space = Space::Other(casual_slide_import::section::ACS, "acs");
/// `p202:`.
pub(crate) const P202: Space = Space::Other(casual_slide_import::section::P202, "p202");
/// `clbl:`.
pub(crate) const CLBL: Space = Space::Other(casual_slide_import::custom_data::CLBL, "clbl");

/// A `presenceInfo` entry's `uri`, as `fixtures/real` has it.
const PRESENCE_URI: &str = "{19B8F6BF-5375-455C-9EA6-DF929625EA0E}";
/// A `threadingInfo` entry's `uri`. Inferred: no deck here holds one.
const THREADING_URI: &str = "{C676402C-5697-4E1C-873F-D02D1690AC5C}";

const COMMENT_ORDER: &[&str] = &[
    "sldMkLst",
    "deMkLst",
    "txMkLst",
    "unknownAnchor",
    "pos",
    "text",
    "replyLst",
    "txBody",
    "extLst",
];
const REPLY_ORDER: &[&str] = &["txBody", "extLst"];
const EVENT_ORDER: &[&str] = &[
    "atrbtn",
    "anchr",
    "asgn",
    "add",
    "title",
    "date",
    "pcntCmplt",
    "unasgnAll",
    "undo",
    "unknown",
    "extLst",
];

/// The schema's sequence for an element of this tier, for a new child among its siblings.
pub(crate) fn order_in(parent: &str) -> Option<&'static [&'static str]> {
    Some(match parent {
        "cm" => COMMENT_ORDER,
        "reply" => REPLY_ORDER,
        "event" => EVENT_ORDER,
        "task" | "taskDetails" => &["history", "extLst"],
        "threadingInfo" => &["parentCm"],
        "section" => &["sldIdLst", "extLst"],
        "zmPr" => &["blipFill", "spPr"],
        "sldZmObj" | "sectionZmObj" | "summaryZmObj" => &["zmPr", "extLst"],
        "summaryZm" => &["summaryZmObj", "gridLayout", "fixedLayout", "extLst"],
        "designPr" => &["designTagLst", "extLst"],
        _ => return None,
    })
}

pub(crate) fn unsupported(w: &Writer<'_>, what: &'static str) -> ExportError {
    ExportError::UnsupportedEdit {
        what,
        part: w.part.to_owned(),
    }
}

pub(crate) fn malformed(w: &Writer<'_>) -> ExportError {
    ExportError::MalformedElement {
        part: w.part.to_owned(),
    }
}

/// The children of this name, as many as the reader found.
pub(crate) fn counted(
    w: &Writer<'_>,
    parent: &Elem,
    uris: &[&str],
    local: &str,
    expected: usize,
) -> Result<Vec<Elem>> {
    let out: Vec<Elem> = parent
        .children(w.tree, w.part)?
        .into_iter()
        .filter(|c| c.is(uris, local))
        .collect();
    if out.len() == expected {
        Ok(out)
    } else {
        Err(malformed(w))
    }
}

/// Bring a list of like elements from `before` to `after`. Matched by key when each has its
/// own, and by place if not. What goes takes only its own element, and what is new goes
/// before the next that stays, or last of its name.
#[allow(clippy::too_many_arguments)] // Each says something the others do not; a struct would only name them twice.
pub(crate) fn sync_items<T: PartialEq>(
    w: &mut Writer<'_>,
    parent: &Elem,
    space: Space,
    local: &'static str,
    before: &[T],
    after: &[T],
    key: impl Fn(&T) -> Option<String>,
    mut patch_item: impl FnMut(&mut Writer<'_>, &Elem, Option<&T>, &T) -> Result,
) -> Result {
    if before == after {
        return Ok(());
    }
    let elements = counted(w, parent, space.uris(), local, before.len())?;
    let keys_of = |list: &[T]| -> Option<HashMap<String, usize>> {
        let mut map = HashMap::with_capacity(list.len());
        for (i, item) in list.iter().enumerate() {
            if map.insert(key(item)?, i).is_some() {
                return None;
            }
        }
        Some(map)
    };
    let by_key = keys_of(before).zip(keys_of(after)).map(|(b, _)| b);
    let source: Vec<Option<usize>> = after
        .iter()
        .enumerate()
        .map(|(i, item)| match &by_key {
            Some(map) => key(item).and_then(|k| map.get(&k).copied()),
            None => (i < before.len()).then_some(i),
        })
        .collect();
    let kept = source.iter().flatten();
    if kept.clone().zip(kept.skip(1)).any(|(x, y)| x > y) {
        return Err(unsupported(w, "reordering a list that holds what the model cannot see"));
    }
    let mut used = vec![false; before.len()];
    for at in source.iter().flatten() {
        if let Some(slot) = used.get_mut(*at) {
            *slot = true;
        }
    }
    for (i, el) in elements.iter().enumerate() {
        if !used.get(i).copied().unwrap_or(false) {
            w.tree.remove_child(parent.id, el.id);
            w.changes += 1;
        }
    }
    for (j, item) in after.iter().enumerate() {
        match source.get(j).copied().flatten() {
            Some(i) => {
                if let (Some(el), Some(b)) = (elements.get(i), before.get(i))
                    && b != item
                {
                    patch_item(w, el, Some(b), item)?;
                }
            }
            None => {
                let next = source
                    .iter()
                    .skip(j)
                    .flatten()
                    .next()
                    .and_then(|i| elements.get(*i))
                    .map(|e| e.id);
                let new = w.new_child(parent, space.uris(), space.prefix(), local)?;
                w.tree.insert_child(parent.id, next, new.id);
                patch_item(w, &new, None, item)?;
            }
        }
    }
    Ok(())
}

/// Bring one `extLst` entry from `before` to `after`, matched by what it holds. A new entry
/// goes in an `extLst` in `list_space`, which is created if need be. `uri` is what a new one
/// is written with, if that is known.
#[allow(clippy::too_many_arguments)] // As for `sync_items`.
pub(crate) fn patch_ext_item(
    w: &mut Writer<'_>,
    el: &Elem,
    list_space: Space,
    uri: Option<&str>,
    before: Option<&Node>,
    after: Option<&Node>,
    item_space: Space,
    item_local: &'static str,
) -> Result {
    if before == after {
        return Ok(());
    }
    let list = el
        .children(w.tree, w.part)?
        .into_iter()
        .find(|c| c.local == "extLst");
    let mut found = None;
    if let Some(list) = &list {
        for ext in list.children(w.tree, w.part)? {
            for item in ext.children(w.tree, w.part)? {
                if found.is_none() && item.is(item_space.uris(), item_local) {
                    found = Some((ext.clone(), item));
                }
            }
        }
    }
    match (found, after) {
        (Some((_, item)), Some(a)) => patch(w, &item, before, a),
        (Some((ext, item)), None) => {
            w.tree.remove_child(ext.id, item.id);
            w.changes += 1;
            if let Some(list) = &list
                && ext.children(w.tree, w.part)?.is_empty()
            {
                w.tree.remove_child(list.id, ext.id);
                if list.children(w.tree, w.part)?.is_empty() {
                    w.tree.remove_child(el.id, list.id);
                }
            }
            Ok(())
        }
        (None, None) => Ok(()),
        (None, Some(a)) => {
            let Some(uri) = uri else {
                return Err(unsupported(w, "adding this extension, whose uri is not known"));
            };
            let list = match list {
                Some(list) => list,
                None => {
                    let list =
                        w.new_child(el, list_space.uris(), list_space.prefix(), "extLst")?;
                    w.tree.insert_child(el.id, None, list.id);
                    list
                }
            };
            let ext = w.new_child(&list, ns::PML, "p", "ext")?;
            w.tree.insert_child(list.id, None, ext.id);
            w.attr(&ext, "uri", None::<&String>, Some(&uri.to_owned()))?;
            let item = w.new_child(&ext, item_space.uris(), item_space.prefix(), item_local)?;
            w.tree.insert_child(ext.id, None, item.id);
            patch(w, &item, None, a)
        }
    }
}

pub(crate) fn point_node(space: Space, local: &'static str, p: &Point) -> Node {
    Node::new(space, local)
        .lex("x", Some(&p.x))
        .lex("y", Some(&p.y))
}

// ---------------------------------------------------------------- legacy

// spec-audit: owner=presenceInfo
fn presence_node(p: &PresenceInfo) -> Node {
    Node::new(P15, "presenceInfo")
        .lex("userId", p.user_id.as_ref())
        .lex("providerId", p.provider_id.as_ref())
}

// spec-audit: owner=cmAuthor
fn cm_author_node(a: &CommentAuthor) -> Node {
    Node::new(Space::Pml, "cmAuthor")
        .lex("id", a.id.as_ref())
        .lex("name", a.name.as_ref())
        .lex("initials", a.initials.as_ref())
        .lex("lastIdx", a.last_idx.as_ref())
        .lex("clrIdx", a.clr_idx.as_ref())
}

fn patch_cm_author(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&CommentAuthor>,
    a: &CommentAuthor,
) -> Result {
    patch(w, el, b.map(cm_author_node).as_ref(), &cm_author_node(a))?;
    patch_ext_item(
        w,
        el,
        Space::Pml,
        Some(PRESENCE_URI),
        b.and_then(|b| b.presence.as_ref()).map(presence_node).as_ref(),
        a.presence.as_ref().map(presence_node).as_ref(),
        P15,
        "presenceInfo",
    )
}

/// Patch a `commentAuthors` tree to say `after`. Returns the nodes changed.
// spec-audit: owner=cmAuthorLst
// spec-audit: p:<cmAuthorLst>
pub fn patch_comment_authors(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    after: &CommentAuthorList,
    limits: &ReviewLimits,
    import: &ImportLimits,
    xml: &OoxmlLimits,
) -> Result<usize> {
    let read = |t: &XmlTree| {
        import_comment_authors(t, part, limits, import, &mut Findings::new(import.max_findings))
    };
    let before = read(tree)?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: None,
        changes: 0,
    };
    sync_items(
        &mut w,
        &root,
        Space::Pml,
        "cmAuthor",
        &before.authors,
        &after.authors,
        |a| a.id.map(|i| i.to_string()),
        patch_cm_author,
    )?;
    let changes = w.changes;
    verify(tree, part, xml, after, read)?;
    Ok(changes)
}

// spec-audit: owner=threadingInfo
fn threading_node(t: &CommentThreading) -> Node {
    Node::new(P15, "threadingInfo")
        .lex("timeZoneBias", t.time_zone_bias.as_ref())
        .opt(t.parent.map(|p| parent_node(&p)))
}

// spec-audit: owner=parentCm
fn parent_node(p: &casual_slide_model::comment::ParentComment) -> Node {
    Node::new(P15, "parentCm")
        .lex("authorId", p.author_id.as_ref())
        .lex("idx", p.idx.as_ref())
}

// spec-audit: owner=cm
fn legacy_node(c: &Comment) -> Node {
    Node::new(Space::Pml, "cm")
        .lex("authorId", c.author_id.as_ref())
        .lex("dt", c.dt.as_ref())
        .lex("idx", c.idx.as_ref())
        .opt(c.pos.as_ref().map(|p| point_node(Space::Pml, "pos", p)))
        .opt(
            c.text
                .as_ref()
                .map(|t| Node::new(Space::Pml, "text").text(Some(t.clone()))),
        )
}

fn patch_legacy(w: &mut Writer<'_>, el: &Elem, b: Option<&Comment>, a: &Comment) -> Result {
    patch(w, el, b.map(legacy_node).as_ref(), &legacy_node(a))?;
    patch_ext_item(
        w,
        el,
        Space::Pml,
        Some(THREADING_URI),
        b.and_then(|b| b.threading.as_ref()).map(threading_node).as_ref(),
        a.threading.as_ref().map(threading_node).as_ref(),
        P15,
        "threadingInfo",
    )
}

/// Patch a legacy `comments` tree to say `after`. Returns the nodes changed.
// spec-audit: owner=cmLst
// spec-audit: p:<cmLst>
pub fn patch_comments(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    after: &CommentList,
    limits: &ReviewLimits,
    import: &ImportLimits,
    xml: &OoxmlLimits,
) -> Result<usize> {
    let read = |t: &XmlTree| {
        import_comments(t, part, limits, import, &mut Findings::new(import.max_findings))
    };
    let before = read(tree)?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: None,
        changes: 0,
    };
    sync_items(
        &mut w,
        &root,
        Space::Pml,
        "cm",
        &before.comments,
        &after.comments,
        |_| None,
        patch_legacy,
    )?;
    let changes = w.changes;
    verify(tree, part, xml, after, read)?;
    Ok(changes)
}

// ---------------------------------------------------------------- modern

// spec-audit: owner=author
fn author_node(a: &Author) -> Node {
    Node::new(P188, "author")
        .lex("id", a.id.as_ref())
        .lex("name", a.name.as_ref())
        .lex("initials", a.initials.as_ref())
        .lex("userId", a.user_id.as_ref())
        .lex("providerId", a.provider_id.as_ref())
}

/// Patch an `authors` tree to say `after`. Returns the nodes changed.
// spec-audit: owner=authorLst
// spec-audit: x:powerpoint/2018/8/main:<authorLst>
pub fn patch_authors(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    after: &AuthorList,
    limits: &ReviewLimits,
    import: &ImportLimits,
    xml: &OoxmlLimits,
) -> Result<usize> {
    let read =
        |t: &XmlTree| import_authors(t, part, limits, import, &mut Findings::new(import.max_findings));
    let before = read(tree)?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: None,
        changes: 0,
    };
    sync_items(
        &mut w,
        &root,
        P188,
        "author",
        &before.authors,
        &after.authors,
        |a| a.id.as_ref().map(|i| i.to_ascii_uppercase()),
        |w, el, b, a| patch(w, el, b.map(author_node).as_ref(), &author_node(a)),
    )?;
    let changes = w.changes;
    verify(tree, part, xml, after, read)?;
    Ok(changes)
}

// spec-audit: owner=reactions
// spec-audit: x:powerpoint/2022/03/main:<reactions>
fn reactions_node(r: &Reactions) -> Node {
    Node::new(P223, "reactions").list(r.rxn.iter().map(|x| {
        Node::new(P223, "rxn")
            .lex("type", x.reaction_type.as_ref())
            .list(x.instance.iter().map(|i| {
                Node::new(P223, "instance")
                    .lex("time", i.time.as_ref())
                    .lex("authorId", i.author_id.as_ref())
            }))
    }))
}

// spec-audit: owner=event
fn action_node(a: &TaskAction) -> Node {
    match a {
        TaskAction::Assign(id) => Node::new(P228, "asgn").lex("authorId", id.as_ref()),
        TaskAction::Add => Node::new(P228, "add"),
        TaskAction::Title(v) => Node::new(P228, "title").lex("val", v.as_ref()),
        TaskAction::Date { st_dt, end_dt } => Node::new(P228, "date")
            .lex("stDt", st_dt.as_ref())
            .lex("endDt", end_dt.as_ref()),
        TaskAction::PercentComplete(v) => Node::new(P228, "pcntCmplt").lex("val", v.as_ref()),
        TaskAction::UnassignAll => Node::new(P228, "unasgnAll"),
        TaskAction::Undo(id) => Node::new(P228, "undo").lex("id", id.as_ref()),
        TaskAction::Unknown => Node::new(P228, "unknown"),
    }
}

// spec-audit: owner=taskDetails
// spec-audit: x:powerpoint/2022/08/main:<taskDetails>
fn task_node(t: &TaskDetails) -> Node {
    Node::new(P228, "taskDetails").opt(t.history.as_ref().map(|h| {
        Node::new(P228, "history").list(h.event.iter().map(|e| {
            Node::new(P228, "event")
                .lex("time", e.time.as_ref())
                .lex("id", e.id.as_ref())
                .opt(
                    e.atrbtn
                        .as_ref()
                        .map(|id| Node::new(P228, "atrbtn").lex("authorId", Some(id))),
                )
                .opt(e.anchr.as_ref().map(|id| {
                    Node::new(P228, "anchr").kid(Node::new(P228, "comment").lex("id", Some(id)))
                }))
                .opt(e.action.as_ref().map(action_node))
        }))
    }))
}

// spec-audit: owner=cm
fn threaded_node(c: &ThreadedComment) -> Node {
    Node::new(P188, "cm")
        .lex("id", c.id.as_ref())
        .lex("authorId", c.author_id.as_ref())
        .lex("status", c.status.map(|s| s.as_xml().to_owned()).as_ref())
        .lex("created", c.created.as_ref())
        .lex("startDate", c.start_date.as_ref())
        .lex("dueDate", c.due_date.as_ref())
        .lex("assignedTo", c.assigned_to.as_ref().map(|l| l.join(" ")).as_ref())
        .lex("complete", c.complete.as_ref())
        .lex("title", c.title.as_ref())
        .opt(c.pos.as_ref().map(|p| point_node(P188, "pos", p)))
}

// spec-audit: owner=reply
fn reply_node(c: &CommentReply) -> Node {
    Node::new(P188, "reply")
        .lex("id", c.id.as_ref())
        .lex("authorId", c.author_id.as_ref())
        .lex("status", c.status.map(|s| s.as_xml().to_owned()).as_ref())
        .lex("created", c.created.as_ref())
}

/// A body in a comment or reply. A new one goes at its schema position.
fn patch_body(
    w: &mut Writer<'_>,
    el: &Elem,
    order: &[&str],
    b: Option<&TextBody>,
    a: Option<&TextBody>,
) -> Result {
    if b == a {
        return Ok(());
    }
    let found = el.first_child(w.tree, w.part, P188.uris(), "txBody")?;
    match (found, a) {
        (Some(body), Some(a)) => patch_text_body(w, &body, b, a),
        (Some(body), None) => {
            w.tree.remove_child(el.id, body.id);
            w.changes += 1;
            Ok(())
        }
        (None, Some(a)) => {
            let body = w.new_child(el, P188.uris(), P188.prefix(), "txBody")?;
            w.insert_in_order(el, order, "txBody", &body)?;
            patch_text_body(w, &body, None, a)
        }
        (None, None) => Ok(()),
    }
}

fn patch_reply(w: &mut Writer<'_>, el: &Elem, b: Option<&CommentReply>, a: &CommentReply) -> Result {
    patch(w, el, b.map(reply_node).as_ref(), &reply_node(a))?;
    patch_body(
        w,
        el,
        REPLY_ORDER,
        b.and_then(|b| b.tx_body.as_ref()),
        a.tx_body.as_ref(),
    )?;
    patch_ext_item(
        w,
        el,
        P188,
        None,
        b.and_then(|b| b.reactions.as_ref()).map(reactions_node).as_ref(),
        a.reactions.as_ref().map(reactions_node).as_ref(),
        P223,
        "reactions",
    )
}

/// The element an anchor's moniker is in: the `pc` namespace holds the document and slide
/// kinds, and `ac` the drawing kinds.
fn moniker_space(kind: &str) -> (&'static [&'static str], &'static str) {
    use casual_slide_import::comment::{AC, PC};
    if kind.starts_with("doc") || kind.starts_with("sld") || kind.starts_with("cm") {
        (PC, "pc")
    } else {
        (AC, "ac")
    }
}

fn write_anchor(
    w: &mut Writer<'_>,
    el: &Elem,
    uris: &'static [&'static str],
    prefix: &str,
    local: &str,
    monikers: &[Moniker],
) -> Result {
    let list = w.new_child(el, uris, prefix, local)?;
    w.insert_in_order(el, COMMENT_ORDER, local, &list)?;
    for m in monikers {
        let (uris, prefix) = moniker_space(&m.kind);
        let item = w.new_child(&list, uris, prefix, &m.kind)?;
        w.tree.insert_child(list.id, None, item.id);
        for (name, value) in &m.attributes {
            w.attr(&item, name, None::<&String>, Some(value))?;
        }
    }
    Ok(())
}

fn patch_anchor(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&ThreadedComment>,
    a: &ThreadedComment,
) -> Result {
    use casual_slide_import::comment::{AC, PC};
    match b {
        Some(b) if b.anchor == a.anchor => Ok(()),
        Some(_) => Err(unsupported(w, "changing what a comment is anchored to")),
        None => match &a.anchor {
            Some(CommentAnchor::Slide(m)) => write_anchor(w, el, PC, "pc", "sldMkLst", m),
            Some(CommentAnchor::Drawing(m)) => write_anchor(w, el, AC, "ac", "deMkLst", m),
            Some(CommentAnchor::Text(m)) => write_anchor(w, el, AC, "ac", "txMkLst", m),
            Some(CommentAnchor::Unknown) => {
                let list = w.new_child(el, P188.uris(), P188.prefix(), "unknownAnchor")?;
                w.insert_in_order(el, COMMENT_ORDER, "unknownAnchor", &list)
            }
            None => Err(unsupported(w, "a new comment with no anchor")),
        },
    }
}

fn patch_threaded(
    w: &mut Writer<'_>,
    el: &Elem,
    b: Option<&ThreadedComment>,
    a: &ThreadedComment,
) -> Result {
    patch_anchor(w, el, b, a)?;
    patch(w, el, b.map(threaded_node).as_ref(), &threaded_node(a))?;
    let before_replies = b.and_then(|b| b.reply_lst.as_ref());
    if before_replies != a.reply_lst.as_ref() {
        let found = el.first_child(w.tree, w.part, P188.uris(), "replyLst")?;
        match (found, a.reply_lst.as_ref()) {
            (Some(list), None) => {
                w.tree.remove_child(el.id, list.id);
                w.changes += 1;
            }
            (found, Some(replies)) => {
                let list = match found {
                    Some(list) => list,
                    None => {
                        let list = w.new_child(el, P188.uris(), P188.prefix(), "replyLst")?;
                        w.insert_in_order(el, COMMENT_ORDER, "replyLst", &list)?;
                        list
                    }
                };
                sync_items(
                    w,
                    &list,
                    P188,
                    "reply",
                    before_replies.map_or(&[][..], Vec::as_slice),
                    replies,
                    |r| r.id.as_ref().map(|i| i.to_ascii_uppercase()),
                    patch_reply,
                )?;
            }
            (None, None) => {}
        }
    }
    patch_body(
        w,
        el,
        COMMENT_ORDER,
        b.and_then(|b| b.tx_body.as_ref()),
        a.tx_body.as_ref(),
    )?;
    patch_ext_item(
        w,
        el,
        P188,
        None,
        b.and_then(|b| b.reactions.as_ref()).map(reactions_node).as_ref(),
        a.reactions.as_ref().map(reactions_node).as_ref(),
        P223,
        "reactions",
    )?;
    patch_ext_item(
        w,
        el,
        P188,
        None,
        b.and_then(|b| b.task.as_ref()).map(task_node).as_ref(),
        a.task.as_ref().map(task_node).as_ref(),
        P228,
        "taskDetails",
    )
}

/// Patch a modern comments tree to say `after`. Returns the nodes changed.
// spec-audit: owner=cmLst
// spec-audit: x:powerpoint/2018/8/main:<cmLst>
#[allow(clippy::too_many_arguments)]
pub fn patch_threaded_comments(
    tree: &mut XmlTree,
    part: &str,
    strict: bool,
    rels: &[Relationship],
    after: &ThreadedCommentList,
    limits: &ReviewLimits,
    import: &ImportLimits,
    xml: &OoxmlLimits,
) -> Result<usize> {
    let read = |t: &XmlTree| {
        import_threaded_comments(
            t,
            part,
            rels,
            limits,
            import,
            &mut Findings::new(import.max_findings),
            &mut ColorFindings::default(),
        )
    };
    let before = read(tree)?;
    if &before == after {
        return Ok(0);
    }
    let root = Elem::root(tree, part)?;
    let mut w = Writer {
        tree,
        part,
        strict,
        known_rels: Some(rels.iter().map(|r| r.id.clone()).collect()),
        changes: 0,
    };
    sync_items(
        &mut w,
        &root,
        P188,
        "cm",
        &before.comments,
        &after.comments,
        |c| c.id.as_ref().map(|i| i.to_ascii_uppercase()),
        patch_threaded,
    )?;
    let changes = w.changes;
    verify(tree, part, xml, after, read)?;
    Ok(changes)
}
