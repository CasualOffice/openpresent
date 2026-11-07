//! Read comments (Tier 1M): legacy `p:cmLst` and `p:cmAuthorLst`, modern `p188:cmLst` and
//! `p188:authorLst`, with threads, mentions, reactions and tasks.
//!
//! **Comment text, author names and user ids are document data, and never reach a finding.**
//! A finding holds a part's name and an element or attribute name from the schemas.
//!
//! A part that cannot be read, is wrongly rooted, or crosses a bound in [`ReviewLimits`], is
//! set aside: named in the report and in [`DeckComments::set_aside`], retained byte for byte,
//! and the deck opens (owner instruction 24). So a deck-level read here cannot fail.
//!
//! An extension is matched by what it holds and not by its `uri`, as real decks spell one
//! `uri` in several ways (`fixtures/real`).
//!
//! Evidence: `MS-PPTX` schemas in `schemas/ms-extensions`. Relationship and content types
//! are as `fixtures/real` has them. That an `assignedTo` or `authorId` should name a listed
//! author, and a thread's `parentCm` a comment in the same part, is Inferred.

use std::collections::{HashMap, HashSet};

use casual_slide_model::comment::{
    Author, AuthorList, Comment, CommentAnchor, CommentAuthor, CommentAuthorList, CommentList,
    CommentReply, CommentStatus, CommentThreading, DateTime, DeckComments, Moniker, ParentComment,
    PresenceInfo, Reaction, ReactionInstance, Reactions, SlideComments, TaskAction, TaskDetails,
    TaskHistory, TaskHistoryEvent, ThreadedComment, ThreadedCommentList,
};
use casual_slide_model::presentation::Percentage;
use casual_slide_model::text::Written;
use casual_slide_ooxml::{
    PresentationPackage, Relationship, TargetMode, XmlTree, part_name_key, resolve_internal,
};

use crate::color::Findings as ColorFindings;
use crate::drawing::DrawingLimits;
use crate::properties::{
    Elem, Findings, ImportError, ImportFindingKind as K, ImportLimits, Reader, is_guid, ns,
};
use crate::shape::read_point;
use crate::text::{TextCounts, TextReader};

/// `p14:`, which holds the section list.
pub const P14: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2010/main"];
/// `p15:`, which holds a legacy thread and an author's presence.
pub const P15: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2012/main"];
/// `p188:`, modern comments.
pub const P188: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2018/8/main"];
/// `p223:`, reactions.
pub const P223: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2022/03/main"];
/// `p228:`, tasks.
pub const P228: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2022/08/main"];
/// `pc:`, what a slide anchor is written in.
pub const PC: &[&str] = &["http://schemas.microsoft.com/office/powerpoint/2013/main/command"];
/// `ac:`, what a shape or text anchor is written in.
pub const AC: &[&str] = &["http://schemas.microsoft.com/office/drawing/2013/main/command"];

/// The legacy `comments` relationship, Transitional and Strict.
pub const COMMENTS_RELATIONSHIP: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/comments",
];
/// The `commentAuthors` relationship, Transitional and Strict.
pub const COMMENT_AUTHORS_RELATIONSHIP: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/commentAuthors",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/commentAuthors",
];
/// The modern `comments` relationship.
pub const THREADED_COMMENTS_RELATIONSHIP: &[&str] =
    &["http://schemas.microsoft.com/office/2018/10/relationships/comments"];
/// The `authors` relationship.
pub const AUTHORS_RELATIONSHIP: &[&str] =
    &["http://schemas.microsoft.com/office/2018/10/relationships/authors"];

/// Bounds on what this tier's parts can make a reader hold. Past one, that part is set
/// aside and named, and the deck opens. Each is past anything PowerPoint is known to write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReviewLimits {
    /// `cm` in one part. `CS-RVW-0001`.
    pub max_comments_per_part: usize,
    /// `cmAuthor` or `author` in one list. `CS-RVW-0001`.
    pub max_authors: usize,
    /// `reply` under one comment. `CS-RVW-0001`.
    pub max_replies_per_comment: usize,
    /// `rxn` plus `instance` under one holder. `CS-RVW-0001`.
    pub max_reactions_per_holder: usize,
    /// `event` in one task. `CS-RVW-0001`.
    pub max_task_events: usize,
    /// Monikers in one anchor. `CS-RVW-0001`.
    pub max_monikers: usize,
    /// Comment and tag parts in one deck, and custom XML items. Past it, the rest are named
    /// as set aside. `CS-RVW-0001`.
    pub max_review_parts: usize,
    /// `section` in one list. `CS-RVW-0001`, and the list is left out.
    pub max_sections: usize,
    /// `sldId` across one list. `CS-RVW-0001`, and the list is left out.
    pub max_section_slides: usize,
    /// `tag` in one part. `CS-RVW-0001`.
    pub max_tags: usize,
    /// Zooms, marks and `custDataLst` entries in one slide-like part. Past it, the rest stay
    /// in the tree, unread. `CS-RVW-0001`.
    pub max_marks_per_part: usize,
    /// `schemaRef`, or `schema`, in one part. `CS-RVW-0001`.
    pub max_schema_refs: usize,
    /// Sensitivity labels in one deck. `CS-RVW-0001`.
    pub max_labels: usize,
}

impl Default for ReviewLimits {
    fn default() -> Self {
        Self {
            max_comments_per_part: 100_000,
            max_authors: 100_000,
            max_replies_per_comment: 100_000,
            max_reactions_per_holder: 10_000,
            max_task_events: 100_000,
            max_monikers: 1_024,
            max_review_parts: 100_000,
            max_sections: 10_000,
            max_section_slides: 100_000,
            max_tags: 100_000,
            max_marks_per_part: 100_000,
            max_schema_refs: 10_000,
            max_labels: 1_024,
        }
    }
}

/// What this tier's readers notice beyond the crate's common findings. Nothing here stops
/// a deck opening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReviewFinding {
    /// A count over a bound in [`ReviewLimits`]: the part is set aside, or the list left out.
    OverBound,
    /// A comment names an author the list does not hold, or the list is absent.
    UnknownAuthor,
    /// A reply or thread names a parent comment the part does not hold.
    UnknownParent,
    /// A comment or section `id` repeats.
    RepeatedId,
    /// A section names a slide `sldIdLst` does not hold (`INV-SL-04`).
    SectionUnknownSlide,
    /// A slide in `sldIdLst` is in no section (`INV-SL-05`).
    SectionUnlistedSlide,
    /// A slide is named more than once across the sections (`INV-SL-05`).
    SectionRepeatedSlide,
    /// Reading the sections in order gives another order from `sldIdLst` (`INV-SL-05`).
    SectionOutOfOrder,
    /// A zoom names a slide or section the deck does not hold.
    ZoomUnknownTarget,
    /// A zoom has no fallback: another reader draws nothing for it (`INV-FB-05`).
    ZoomNoFallback,
    /// A slide's `commentRel` and its relationships to comment parts disagree.
    CommentRelMismatch,
    /// A part kept whole, modelled no further: change tracking, revision info, an item's body.
    RetainedWhole,
}

impl ReviewFinding {
    /// The stable diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::OverBound => "CS-RVW-0001",
            Self::UnknownAuthor => "CS-RVW-0002",
            Self::UnknownParent => "CS-RVW-0003",
            Self::RepeatedId => "CS-RVW-0004",
            Self::SectionUnknownSlide => "CS-RVW-0005",
            Self::SectionUnlistedSlide => "CS-RVW-0006",
            Self::SectionRepeatedSlide => "CS-RVW-0007",
            Self::SectionOutOfOrder => "CS-RVW-0008",
            Self::ZoomUnknownTarget => "CS-RVW-0009",
            Self::ZoomNoFallback => "CS-RVW-0010",
            Self::CommentRelMismatch => "CS-RVW-0011",
            Self::RetainedWhole => "CS-RVW-0012",
        }
    }
}

/// Fill a slot from the first occurrence, reporting any other.
pub(crate) fn first<'s>(
    r: &mut Reader<'_>,
    seen: &mut HashSet<&'s str>,
    name: &'s str,
    el: &Elem,
) -> bool {
    let fresh = seen.insert(name);
    if !fresh {
        r.note(K::DuplicateElement, el, None);
    }
    fresh
}

// `Reader::required` hands a validator `&String`, so this signature is fixed.
#[allow(clippy::ptr_arg)]
pub(crate) fn guid_attr(s: &String) -> bool {
    is_guid(s)
}

pub(crate) fn fixed_percentage(p: &Written<Percentage>) -> bool {
    (0..=100_000).contains(&p.value().0)
}

/// `p:CT_CommentAuthorList`.
// spec-audit: owner=cmAuthorLst
// spec-audit: p:<cmAuthorLst>
pub fn import_comment_authors(
    tree: &XmlTree,
    part: &str,
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
) -> Result<CommentAuthorList, ImportError> {
    let mut reader = Reader::new(tree, part, import, findings);
    let r = &mut reader;
    let root = r.root(ns::PML, "cmAuthorLst")?;
    r.finish(&root);
    let mut list = CommentAuthorList::default();
    let mut ids = HashSet::new();
    for child in r.bounded_children(&root, limits.max_authors, "comment authors")? {
        if !child.is(ns::PML, "cmAuthor") {
            r.retained(&child);
            continue;
        }
        let author = read_cm_author(r, &child)?;
        if let Some(id) = author.id
            && !ids.insert(id)
        {
            r.note(K::DuplicateId, &child, Some("id"));
        }
        list.authors.push(author);
    }
    Ok(list)
}

// spec-audit: owner=cmAuthor
fn read_cm_author(r: &mut Reader<'_>, el: &Elem) -> Result<CommentAuthor, ImportError> {
    let mut e = el.clone();
    let mut out = CommentAuthor {
        id: r.required(&mut e, "id", |_: &u32| true)?,
        name: r.required(&mut e, "name", |_: &String| true)?,
        initials: r.required(&mut e, "initials", |_: &String| true)?,
        last_idx: r.required(&mut e, "lastIdx", |_: &u32| true)?,
        clr_idx: r.required(&mut e, "clrIdx", |_: &u32| true)?,
        presence: None,
    };
    r.finish(&e);
    for child in r.children(el)? {
        if !child.is(ns::PML, "extLst") {
            r.retained(&child);
            continue;
        }
        for item in extension_items(r, &child)? {
            if item.is(P15, "presenceInfo") && out.presence.is_none() {
                out.presence = Some(read_presence(r, &item)?);
            } else {
                r.retained(&item);
            }
        }
    }
    Ok(out)
}

// spec-audit: owner=presenceInfo
// spec-audit: x:powerpoint/2012/main:<presenceInfo>
fn read_presence(r: &mut Reader<'_>, el: &Elem) -> Result<PresenceInfo, ImportError> {
    let mut e = el.clone();
    let out = PresenceInfo {
        user_id: r.required(&mut e, "userId", |_: &String| true)?,
        provider_id: r.required(&mut e, "providerId", |_: &String| true)?,
    };
    r.finish(&e);
    Ok(out)
}

/// What an `extLst`'s entries hold. An entry holding nothing is retained, and said.
pub(crate) fn extension_items(r: &mut Reader<'_>, el: &Elem) -> Result<Vec<Elem>, ImportError> {
    r.finish(el);
    let mut items = Vec::new();
    for ext in r.children(el)? {
        if !ext.in_namespace(ns::PML) && !ext.in_namespace(ns::DML) || ext.local != "ext" {
            r.retained(&ext);
            continue;
        }
        let mut holder = ext.clone();
        holder.take("uri");
        r.finish(&holder);
        let held = r.children(&ext)?;
        if held.is_empty() {
            r.retained(&ext);
        }
        items.extend(held);
    }
    Ok(items)
}

/// `p:CT_CommentList`.
// spec-audit: owner=cmLst
// spec-audit: p:<cmLst>
pub fn import_comments(
    tree: &XmlTree,
    part: &str,
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
) -> Result<CommentList, ImportError> {
    let mut reader = Reader::new(tree, part, import, findings);
    let r = &mut reader;
    let root = r.root(ns::PML, "cmLst")?;
    r.finish(&root);
    let mut list = CommentList::default();
    for child in r.bounded_children(&root, limits.max_comments_per_part, "comments")? {
        if !child.is(ns::PML, "cm") {
            r.retained(&child);
            continue;
        }
        list.comments.push(read_legacy_comment(r, &child)?);
    }
    let known: HashSet<(Option<u32>, Option<u32>)> =
        list.comments.iter().map(|c| (c.author_id, c.idx)).collect();
    let orphan = list.comments.iter().any(|c| {
        c.threading
            .as_ref()
            .and_then(|t| t.parent)
            .is_some_and(|p| !known.contains(&(p.author_id, p.idx)))
    });
    if orphan {
        r.findings.note(
            K::Review(ReviewFinding::UnknownParent),
            part,
            "parentCm",
            None,
        );
    }
    Ok(list)
}

// spec-audit: owner=cm
fn read_legacy_comment(r: &mut Reader<'_>, el: &Elem) -> Result<Comment, ImportError> {
    let mut e = el.clone();
    let mut out = Comment {
        author_id: r.required(&mut e, "authorId", |_: &u32| true)?,
        dt: r.attr(&mut e, "dt")?,
        idx: r.required(&mut e, "idx", |v: &u32| *v >= 1)?,
        ..Comment::default()
    };
    r.finish(&e);
    let mut seen = HashSet::new();
    for child in r.children(el)? {
        if !child.in_namespace(ns::PML) {
            r.retained(&child);
            continue;
        }
        match child.local.as_str() {
            "pos" => {
                if first(r, &mut seen, "pos", &child) {
                    out.pos = read_point(r, &child)?;
                }
            }
            "text" => {
                if first(r, &mut seen, "text", &child) {
                    r.finish(&child);
                    out.text = r.text(&child)?;
                }
            }
            "extLst" => {
                if first(r, &mut seen, "extLst", &child) {
                    for item in extension_items(r, &child)? {
                        if item.is(P15, "threadingInfo") && out.threading.is_none() {
                            out.threading = Some(read_threading(r, &item)?);
                        } else {
                            r.retained(&item);
                        }
                    }
                }
            }
            _ => r.retained(&child),
        }
    }
    Ok(out)
}

// spec-audit: owner=threadingInfo
// spec-audit: x:powerpoint/2012/main:<threadingInfo>
fn read_threading(r: &mut Reader<'_>, el: &Elem) -> Result<CommentThreading, ImportError> {
    let mut e = el.clone();
    let mut out = CommentThreading {
        time_zone_bias: r.attr(&mut e, "timeZoneBias")?,
        parent: None,
    };
    r.finish(&e);
    let mut seen = HashSet::new();
    for child in r.children(el)? {
        if !child.is(P15, "parentCm") {
            r.retained(&child);
        } else if first(r, &mut seen, "parentCm", &child) {
            out.parent = Some(read_parent(r, &child)?);
        }
    }
    Ok(out)
}

// spec-audit: owner=parentCm
fn read_parent(r: &mut Reader<'_>, el: &Elem) -> Result<ParentComment, ImportError> {
    let mut e = el.clone();
    let out = ParentComment {
        author_id: r.required(&mut e, "authorId", |_: &u32| true)?,
        idx: r.required(&mut e, "idx", |_: &u32| true)?,
    };
    r.finish(&e);
    Ok(out)
}

/// `p188:CT_AuthorList`.
// spec-audit: owner=authorLst
// spec-audit: x:powerpoint/2018/8/main:<authorLst>
pub fn import_authors(
    tree: &XmlTree,
    part: &str,
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
) -> Result<AuthorList, ImportError> {
    let mut reader = Reader::new(tree, part, import, findings);
    let r = &mut reader;
    let root = r.root(P188, "authorLst")?;
    r.finish(&root);
    let mut list = AuthorList::default();
    let mut ids = HashSet::new();
    for child in r.bounded_children(&root, limits.max_authors, "authors")? {
        if !child.is(P188, "author") {
            r.retained(&child);
            continue;
        }
        let author = read_author(r, &child)?;
        if let Some(id) = &author.id
            && !ids.insert(id.to_ascii_uppercase())
        {
            r.note(K::DuplicateId, &child, Some("id"));
        }
        list.authors.push(author);
    }
    Ok(list)
}

// spec-audit: owner=author
fn read_author(r: &mut Reader<'_>, el: &Elem) -> Result<Author, ImportError> {
    let mut e = el.clone();
    let out = Author {
        id: r.required(&mut e, "id", guid_attr)?,
        name: r.required(&mut e, "name", |_: &String| true)?,
        initials: r.attr(&mut e, "initials")?,
        user_id: r.required(&mut e, "userId", |_: &String| true)?,
        provider_id: r.required(&mut e, "providerId", |_: &String| true)?,
    };
    r.finish(&e);
    for child in r.children(el)? {
        r.retained(&child);
    }
    Ok(out)
}

/// One modern comment part being read.
struct Threaded<'a, 'r> {
    r: &'a mut Reader<'r>,
    rels: &'a [Relationship],
    limits: &'a ReviewLimits,
    color: &'a mut ColorFindings,
    text_counts: TextCounts,
}

/// `p188:CT_CommentList`.
// spec-audit: owner=cmLst
// spec-audit: x:powerpoint/2018/8/main:<cmLst>
pub fn import_threaded_comments(
    tree: &XmlTree,
    part: &str,
    rels: &[Relationship],
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
    color: &mut ColorFindings,
) -> Result<ThreadedCommentList, ImportError> {
    let mut reader = Reader::new(tree, part, import, findings);
    let root = reader.root(P188, "cmLst")?;
    reader.finish(&root);
    let mut t = Threaded {
        r: &mut reader,
        rels,
        limits,
        color,
        text_counts: TextCounts::default(),
    };
    let mut list = ThreadedCommentList::default();
    let mut ids = HashSet::new();
    for child in
        t.r.bounded_children(&root, limits.max_comments_per_part, "comments")?
    {
        if !child.is(P188, "cm") {
            t.r.retained(&child);
            continue;
        }
        let comment = t.comment(&child)?;
        let replies = comment
            .reply_lst
            .iter()
            .flatten()
            .filter_map(|c| c.id.as_ref());
        for id in comment.id.iter().chain(replies) {
            if !ids.insert(id.to_ascii_uppercase()) {
                t.r.note(K::Review(ReviewFinding::RepeatedId), &child, Some("id"));
            }
        }
        list.comments.push(comment);
    }
    Ok(list)
}

impl Threaded<'_, '_> {
    // spec-audit: owner=cm
    fn comment(&mut self, el: &Elem) -> Result<ThreadedComment, ImportError> {
        let mut e = el.clone();
        let r = &mut *self.r;
        let mut out = ThreadedComment {
            id: r.required(&mut e, "id", guid_attr)?,
            author_id: r.required(&mut e, "authorId", guid_attr)?,
            status: read_status(r, &mut e)?,
            created: r.required(&mut e, "created", |_: &DateTime| true)?,
            start_date: r.attr(&mut e, "startDate")?,
            due_date: r.attr(&mut e, "dueDate")?,
            complete: r.attr_in(&mut e, "complete", fixed_percentage)?,
            title: r.attr(&mut e, "title")?,
            ..ThreadedComment::default()
        };
        if let Some(raw) = e.take("assignedTo") {
            let ids: Vec<String> = raw.split_ascii_whitespace().map(str::to_owned).collect();
            if ids.iter().all(|i| is_guid(i)) {
                out.assigned_to = Some(ids);
            } else {
                r.note(K::MalformedValue, &e, Some("assignedTo"));
            }
        }
        r.finish(&e);
        let mut seen = HashSet::new();
        let mut anchors: Vec<(CommentAnchorKind, Vec<Moniker>)> = Vec::new();
        for child in r.children(el)? {
            let r = &mut *self.r;
            if child.is(PC, "sldMkLst") {
                anchors.push((
                    CommentAnchorKind::Slide,
                    read_monikers(r, &child, self.limits)?,
                ));
            } else if child.is(AC, "deMkLst") {
                anchors.push((
                    CommentAnchorKind::Drawing,
                    read_monikers(r, &child, self.limits)?,
                ));
            } else if child.is(AC, "txMkLst") {
                anchors.push((
                    CommentAnchorKind::Text,
                    read_monikers(r, &child, self.limits)?,
                ));
            } else if child.is(P188, "unknownAnchor") {
                r.finish(&child);
                anchors.push((CommentAnchorKind::Unknown, Vec::new()));
            } else if !child.is(P188, child.local.as_str()) {
                r.retained(&child);
            } else {
                match child.local.as_str() {
                    "pos" => {
                        if first(r, &mut seen, "pos", &child) {
                            out.pos = read_point(r, &child)?;
                        }
                    }
                    "replyLst" => {
                        if first(r, &mut seen, "replyLst", &child) {
                            r.finish(&child);
                            out.reply_lst = Some(self.replies(&child)?);
                        }
                    }
                    "txBody" => {
                        if first(r, &mut seen, "txBody", &child) {
                            out.tx_body = Some(self.body(&child)?);
                        }
                    }
                    "extLst" => {
                        if first(r, &mut seen, "extLst", &child) {
                            for item in extension_items(r, &child)? {
                                let r = &mut *self.r;
                                if item.is(P223, "reactions") && out.reactions.is_none() {
                                    out.reactions = Some(read_reactions(r, &item, self.limits)?);
                                } else if item.is(P228, "taskDetails") && out.task.is_none() {
                                    out.task = Some(read_task(r, &item, self.limits)?);
                                } else {
                                    r.retained(&item);
                                }
                            }
                        }
                    }
                    _ => r.retained(&child),
                }
            }
        }
        out.anchor = combine_anchors(self.r, el, anchors);
        Ok(out)
    }

    // spec-audit: owner=replyLst
    fn replies(&mut self, el: &Elem) -> Result<Vec<CommentReply>, ImportError> {
        let mut out = Vec::new();
        for child in self
            .r
            .bounded_children(el, self.limits.max_replies_per_comment, "replies")?
        {
            if child.is(P188, "reply") {
                out.push(self.reply(&child)?);
            } else {
                self.r.retained(&child);
            }
        }
        Ok(out)
    }

    // spec-audit: owner=reply
    fn reply(&mut self, el: &Elem) -> Result<CommentReply, ImportError> {
        let mut e = el.clone();
        let r = &mut *self.r;
        let mut out = CommentReply {
            id: r.required(&mut e, "id", guid_attr)?,
            author_id: r.required(&mut e, "authorId", guid_attr)?,
            status: read_status(r, &mut e)?,
            created: r.required(&mut e, "created", |_: &DateTime| true)?,
            ..CommentReply::default()
        };
        r.finish(&e);
        let mut seen = HashSet::new();
        for child in r.children(el)? {
            let r = &mut *self.r;
            if !child.is(P188, child.local.as_str()) {
                r.retained(&child);
                continue;
            }
            match child.local.as_str() {
                "txBody" => {
                    if first(r, &mut seen, "txBody", &child) {
                        out.tx_body = Some(self.body(&child)?);
                    }
                }
                "extLst" => {
                    if first(r, &mut seen, "extLst", &child) {
                        for item in extension_items(r, &child)? {
                            let r = &mut *self.r;
                            if item.is(P223, "reactions") && out.reactions.is_none() {
                                out.reactions = Some(read_reactions(r, &item, self.limits)?);
                            } else {
                                r.retained(&item);
                            }
                        }
                    }
                }
                _ => r.retained(&child),
            }
        }
        Ok(out)
    }

    fn body(&mut self, el: &Elem) -> Result<casual_slide_model::text::TextBody, ImportError> {
        let mut text = TextReader {
            r: &mut *self.r,
            rels: self.rels,
            color: &mut *self.color,
            counts: &mut self.text_counts,
            drawing: DrawingLimits::default(),
        };
        text.body(el)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CommentAnchorKind {
    Slide,
    Drawing,
    Text,
    Unknown,
}

/// A comment has one kind of anchor; `deMkLst` and `txMkLst` may repeat. Anything past the
/// first kind stays in the tree, and is reported.
fn combine_anchors(
    r: &mut Reader<'_>,
    el: &Elem,
    anchors: Vec<(CommentAnchorKind, Vec<Moniker>)>,
) -> Option<CommentAnchor> {
    let mut it = anchors.into_iter();
    let Some((kind, mut monikers)) = it.next() else {
        r.note(K::MissingRequired, el, None);
        return None;
    };
    for (other, more) in it {
        if other == kind && matches!(kind, CommentAnchorKind::Drawing | CommentAnchorKind::Text) {
            monikers.extend(more);
        } else {
            r.note(K::DuplicateElement, el, None);
        }
    }
    Some(match kind {
        CommentAnchorKind::Slide => CommentAnchor::Slide(monikers),
        CommentAnchorKind::Drawing => CommentAnchor::Drawing(monikers),
        CommentAnchorKind::Text => CommentAnchor::Text(monikers),
        CommentAnchorKind::Unknown => CommentAnchor::Unknown,
    })
}

/// An anchor's moniker list: what each child is called and says. These are data of a
/// reader's interest only and are written back by the tree, so nothing about one is reported.
// spec-audit: owner=sldMkLst,deMkLst,txMkLst
fn read_monikers(
    r: &mut Reader<'_>,
    el: &Elem,
    limits: &ReviewLimits,
) -> Result<Vec<Moniker>, ImportError> {
    r.finish(el);
    let mut out = Vec::new();
    for child in r.bounded_children(el, limits.max_monikers, "monikers")? {
        let attributes = r
            .tree
            .attributes(child.id, r.part)?
            .into_iter()
            .filter(|(n, _)| n != "xmlns" && !n.starts_with("xmlns:"))
            .collect();
        out.push(Moniker {
            kind: child.local.clone(),
            attributes,
        });
    }
    Ok(out)
}

fn read_status(r: &mut Reader<'_>, e: &mut Elem) -> Result<Option<CommentStatus>, ImportError> {
    let Some(raw) = e.take("status") else {
        return Ok(None);
    };
    let found = CommentStatus::from_xml(&raw);
    if found.is_none() {
        r.note(K::MalformedValue, e, Some("status"));
    }
    Ok(found)
}

// spec-audit: owner=reactions
// spec-audit: x:powerpoint/2022/03/main:<reactions>
fn read_reactions(
    r: &mut Reader<'_>,
    el: &Elem,
    limits: &ReviewLimits,
) -> Result<Reactions, ImportError> {
    r.finish(el);
    let limit = limits.max_reactions_per_holder;
    let mut spent = 0_usize;
    let mut out = Reactions::default();
    for child in r.bounded_children(el, limit, "reactions")? {
        if !child.is(P223, "rxn") {
            r.retained(&child);
            continue;
        }
        spent = spent.saturating_add(1);
        out.rxn.push(read_reaction(r, &child, limit, &mut spent)?);
    }
    Ok(out)
}

// spec-audit: owner=rxn
fn read_reaction(
    r: &mut Reader<'_>,
    el: &Elem,
    limit: usize,
    spent: &mut usize,
) -> Result<Reaction, ImportError> {
    let mut e = el.clone();
    let mut out = Reaction {
        reaction_type: r.required(&mut e, "type", |_: &String| true)?,
        instance: Vec::new(),
    };
    r.finish(&e);
    for child in r.bounded_children(el, limit, "reactions")? {
        if !child.is(P223, "instance") {
            r.retained(&child);
            continue;
        }
        *spent = spent.saturating_add(1);
        if *spent > limit {
            return Err(r.too_many("reactions", limit));
        }
        out.instance.push(read_reaction_instance(r, &child)?);
    }
    Ok(out)
}

// spec-audit: owner=instance
fn read_reaction_instance(r: &mut Reader<'_>, el: &Elem) -> Result<ReactionInstance, ImportError> {
    let mut e = el.clone();
    let out = ReactionInstance {
        time: r.required(&mut e, "time", |_: &DateTime| true)?,
        author_id: r.required(&mut e, "authorId", guid_attr)?,
    };
    r.finish(&e);
    for child in r.children(el)? {
        r.retained(&child);
    }
    Ok(out)
}

// spec-audit: owner=taskDetails
// spec-audit: x:powerpoint/2022/08/main:<taskDetails>
fn read_task(
    r: &mut Reader<'_>,
    el: &Elem,
    limits: &ReviewLimits,
) -> Result<TaskDetails, ImportError> {
    r.finish(el);
    let mut out = TaskDetails::default();
    let mut seen = HashSet::new();
    for child in r.children(el)? {
        if !child.is(P228, "history") {
            r.retained(&child);
            continue;
        }
        if !first(r, &mut seen, "history", &child) {
            continue;
        }
        r.finish(&child);
        let mut history = TaskHistory::default();
        for event in r.bounded_children(&child, limits.max_task_events, "task events")? {
            if event.is(P228, "event") {
                history.event.push(read_event(r, &event)?);
            } else {
                r.retained(&event);
            }
        }
        out.history = Some(history);
    }
    if out.history.is_none() {
        r.note(K::MissingRequired, el, None);
    }
    Ok(out)
}

// spec-audit: owner=event
fn read_event(r: &mut Reader<'_>, el: &Elem) -> Result<TaskHistoryEvent, ImportError> {
    let mut e = el.clone();
    let mut out = TaskHistoryEvent {
        time: r.required(&mut e, "time", |_: &DateTime| true)?,
        id: r.required(&mut e, "id", guid_attr)?,
        ..TaskHistoryEvent::default()
    };
    r.finish(&e);
    let mut seen = HashSet::new();
    let mut taken = false;
    for child in r.children(el)? {
        if !child.is(P228, child.local.as_str()) {
            r.retained(&child);
            continue;
        }
        let mut c = child.clone();
        match child.local.as_str() {
            "atrbtn" => {
                if first(r, &mut seen, "atrbtn", &child) {
                    out.atrbtn = r.required(&mut c, "authorId", guid_attr)?;
                    r.finish(&c);
                }
            }
            "anchr" => {
                if first(r, &mut seen, "anchr", &child) {
                    out.anchr = read_task_anchor(r, &child)?;
                }
            }
            "extLst" => r.retained(&child),
            "asgn" | "add" | "title" | "date" | "pcntCmplt" | "unasgnAll" | "undo" | "unknown" => {
                if taken {
                    r.note(K::DuplicateElement, &child, None);
                    continue;
                }
                taken = true;
                out.action = Some(read_task_action(r, &mut c)?);
                r.finish(&c);
            }
            _ => r.retained(&child),
        }
    }
    Ok(out)
}

// spec-audit: owner=anchr
fn read_task_anchor(r: &mut Reader<'_>, el: &Elem) -> Result<Option<String>, ImportError> {
    r.finish(el);
    let mut id = None;
    let mut seen = HashSet::new();
    for child in r.children(el)? {
        if child.is(P228, "comment") {
            if first(r, &mut seen, "comment", &child) {
                let mut c = child.clone();
                id = r.required(&mut c, "id", guid_attr)?;
                r.finish(&c);
            }
        } else {
            r.retained(&child);
        }
    }
    Ok(id)
}

// spec-audit: owner=event
fn read_task_action(r: &mut Reader<'_>, c: &mut Elem) -> Result<TaskAction, ImportError> {
    Ok(match c.local.clone().as_str() {
        "asgn" => TaskAction::Assign(r.required(c, "authorId", guid_attr)?),
        "add" => TaskAction::Add,
        "title" => TaskAction::Title(r.required(c, "val", |_: &String| true)?),
        "date" => TaskAction::Date {
            st_dt: r.attr(c, "stDt")?,
            end_dt: r.attr(c, "endDt")?,
        },
        "pcntCmplt" => TaskAction::PercentComplete(r.required(c, "val", fixed_percentage)?),
        "unasgnAll" => TaskAction::UnassignAll,
        "undo" => TaskAction::Undo(r.required(c, "id", guid_attr)?),
        _ => TaskAction::Unknown,
    })
}

/// Every internal part a relationship of this kind reaches from `source`, as the archive
/// spells it. What leaves the package, or reaches nothing, is reported and not followed.
pub(crate) fn targets_of(
    names: &HashMap<String, String>,
    source: &str,
    rels: &[Relationship],
    rel_types: &[&str],
    findings: &mut Findings,
) -> Vec<String> {
    let mut out = Vec::new();
    for rel in rels
        .iter()
        .filter(|r| rel_types.contains(&r.rel_type.as_str()))
    {
        let target = (rel.mode == TargetMode::Internal)
            .then(|| resolve_internal(source, &rel.target))
            .flatten()
            .and_then(|t| part_name_key(&t))
            .and_then(|key| names.get(&key));
        match target {
            Some(name) => out.push(name.clone()),
            None => findings.note_role(
                K::UnusableRelationship,
                source,
                rel_types
                    .first()
                    .and_then(|t| t.rsplit('/').next())
                    .unwrap_or(""),
            ),
        }
    }
    out
}

/// The archive's names, by key.
pub(crate) fn name_map(package: &PresentationPackage) -> HashMap<String, String> {
    package
        .part_names()
        .into_iter()
        .filter_map(|n| part_name_key(&n).map(|k| (k, n)))
        .collect()
}

/// How one part's read ended, for a deck-level reader.
pub(crate) fn read_part_or_set_aside<T>(
    package: &mut PresentationPackage,
    part: &str,
    findings: &mut Findings,
    set_aside: &mut Vec<String>,
    read: impl FnOnce(&XmlTree, &[Relationship], &mut Findings) -> Result<T, ImportError>,
) -> Option<T> {
    let rels = package.relationships_of(part).to_vec();
    let limits = *package.limits();
    let bytes = package.read_xml_part(part);
    // A repair has been here, and what reads now is what stands in for the part: the part
    // itself is what the caller has to be told of, and it is kept under `recovered/`.
    let result = if package.is_damaged(part) {
        Err(K::UnreadablePart)
    } else {
        bytes
            .map_err(ImportError::from)
            .and_then(|bytes| XmlTree::parse(&bytes, part, &limits).map_err(ImportError::from))
            .and_then(|tree| read(&tree, &rels, findings))
            .map_err(|err| match err {
                ImportError::TooMany { .. } => K::Review(ReviewFinding::OverBound),
                _ => K::UnreadablePart,
            })
    };
    match result {
        Ok(value) => Some(value),
        Err(kind) => {
            findings.note(kind, part, "", None);
            if !set_aside.iter().any(|p| p == part) {
                set_aside.push(part.to_owned());
            }
            None
        }
    }
}

/// Read every comment and author list in a deck. Nothing here refuses: see the module
/// documentation. `slide_parts` are the slide-like parts to look under, in order.
pub fn read_deck_comments(
    package: &mut PresentationPackage,
    slide_parts: &[String],
    limits: &ReviewLimits,
    import: &ImportLimits,
    findings: &mut Findings,
    color: &mut ColorFindings,
) -> DeckComments {
    let names = name_map(package);
    let main = package.main_part().to_owned();
    let mut out = DeckComments::default();
    let mut set_aside = Vec::new();
    let rels = package.relationships_of(&main).to_vec();
    let legacy_authors = targets_of(&names, &main, &rels, COMMENT_AUTHORS_RELATIONSHIP, findings);
    if let Some(part) = legacy_authors.first() {
        let p = part.clone();
        out.comment_authors =
            read_part_or_set_aside(package, &p, findings, &mut set_aside, |t, _, f| {
                import_comment_authors(t, &p, limits, import, f)
            });
    }
    let authors = targets_of(&names, &main, &rels, AUTHORS_RELATIONSHIP, findings);
    if let Some(part) = authors.first() {
        let p = part.clone();
        out.authors = read_part_or_set_aside(package, &p, findings, &mut set_aside, |t, _, f| {
            import_authors(t, &p, limits, import, f)
        });
    }
    let mut count = 0_usize;
    for slide in slide_parts {
        let rels = package.relationships_of(slide).to_vec();
        for part in targets_of(&names, slide, &rels, COMMENTS_RELATIONSHIP, findings) {
            count = count.saturating_add(1);
            if count > limits.max_review_parts {
                findings.note(K::Review(ReviewFinding::OverBound), &part, "", None);
                set_aside.push(part);
                continue;
            }
            let read =
                read_part_or_set_aside(package, &part, findings, &mut set_aside, |t, _, f| {
                    import_comments(t, &part, limits, import, f)
                });
            if let Some(list) = read {
                out.legacy.push(SlideComments {
                    part,
                    slide_part: slide.clone(),
                    list,
                });
            }
        }
        for part in targets_of(
            &names,
            slide,
            &rels,
            THREADED_COMMENTS_RELATIONSHIP,
            findings,
        ) {
            count = count.saturating_add(1);
            if count > limits.max_review_parts {
                findings.note(K::Review(ReviewFinding::OverBound), &part, "", None);
                set_aside.push(part);
                continue;
            }
            let read =
                read_part_or_set_aside(package, &part, findings, &mut set_aside, |t, r, f| {
                    import_threaded_comments(t, &part, r, limits, import, f, color)
                });
            if let Some(list) = read {
                out.threaded.push(SlideComments {
                    part,
                    slide_part: slide.clone(),
                    list,
                });
            }
        }
    }
    check_authors(&out, findings);
    out.set_aside = set_aside;
    out
}

/// Report a comment naming an author no list holds. Reported once a part: which comment is
/// the part's business, and an id is not repeated in a report.
fn check_authors(deck: &DeckComments, findings: &mut Findings) {
    let legacy: HashSet<u32> = deck
        .comment_authors
        .iter()
        .flat_map(|l| &l.authors)
        .filter_map(|a| a.id)
        .collect();
    for slide in &deck.legacy {
        let unknown = slide
            .list
            .comments
            .iter()
            .filter_map(|c| c.author_id)
            .any(|id| !legacy.contains(&id));
        if unknown {
            findings.note(
                K::Review(ReviewFinding::UnknownAuthor),
                &slide.part,
                "cm",
                Some("authorId"),
            );
        }
    }
    let modern: HashSet<String> = deck
        .authors
        .iter()
        .flat_map(|l| &l.authors)
        .filter_map(|a| a.id.as_ref().map(|i| i.to_ascii_uppercase()))
        .collect();
    let known = |id: &String| modern.contains(&id.to_ascii_uppercase());
    for slide in &deck.threaded {
        for c in &slide.list.comments {
            let replies = c
                .reply_lst
                .iter()
                .flatten()
                .filter_map(|r| r.author_id.as_ref());
            let assigned = c.assigned_to.iter().flatten();
            let unknown = c
                .author_id
                .iter()
                .chain(replies)
                .chain(assigned)
                .any(|id| !known(id));
            if unknown {
                findings.note(
                    K::Review(ReviewFinding::UnknownAuthor),
                    &slide.part,
                    "cm",
                    Some("authorId"),
                );
            }
        }
    }
}
