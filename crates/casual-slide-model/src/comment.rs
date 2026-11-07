//! Comments as plain typed values (`docs/spec/COMMENT.md`): legacy `p:cmLst` and
//! `p:cmAuthorLst`, modern `p188:cmLst` and `p188:authorLst`, with threads, mentions,
//! reactions and tasks.
//!
//! **Comment text, author names and user ids are document data.** Nothing here reaches a
//! finding or a message. Each field is what the file said, or `None`.
//!
//! What stays in the tree: every `extLst` entry not named here, and an anchor's moniker
//! elements beyond what [`Moniker`] says of them.
//!
//! Evidence: `MS-PPTX` schemas in `schemas/ms-extensions`. What a mention looks like, and
//! that a task's `assignedTo` names authors in the same list, is Inferred. `fixtures/real`
//! holds author lists and no modern comment, so PowerPoint's use of tasks and reactions is
//! Unverified until `PV-01`.

use crate::color::xml_enum;
use crate::presentation::{Percentage, Point};
use crate::properties::Lexical;
use crate::text::{TextBody, Written};

/// An `xsd:dateTime`, kept as it was written. Compared as text: two spellings of one
/// instant are two values, as a comment's time is never computed with here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime(String);

impl DateTime {
    /// The text this value was parsed from.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn digits(s: &str, n: usize) -> Option<&str> {
    let head = s.get(..n)?;
    head.bytes()
        .all(|b| b.is_ascii_digit())
        .then(|| s.get(n..))
        .flatten()
}

fn expect(s: &str, c: char) -> Option<&str> {
    s.strip_prefix(c)
}

impl Lexical for DateTime {
    fn parse_lexical(s: &str) -> Option<Self> {
        let t = crate::properties::trim_xml(s);
        let rest = t.strip_prefix('-').unwrap_or(t);
        let rest = expect(digits(rest, 4)?, '-')?;
        let rest = expect(digits(rest, 2)?, '-')?;
        let rest = expect(digits(rest, 2)?, 'T')?;
        let rest = expect(digits(rest, 2)?, ':')?;
        let rest = expect(digits(rest, 2)?, ':')?;
        let mut rest = digits(rest, 2)?;
        if let Some(fraction) = rest.strip_prefix('.') {
            let len = fraction.bytes().take_while(u8::is_ascii_digit).count();
            if len == 0 {
                return None;
            }
            rest = fraction.get(len..)?;
        }
        let zoned = match rest.as_bytes().first() {
            None => true,
            Some(b'Z') => rest.len() == 1,
            Some(b'+' | b'-') => rest.get(1..).is_some_and(|z| {
                digits(z, 2)
                    .and_then(|z| expect(z, ':'))
                    .and_then(|z| digits(z, 2))
                    .is_some_and(str::is_empty)
            }),
            Some(_) => false,
        };
        zoned.then(|| Self(t.to_owned()))
    }

    fn to_lexical(&self) -> String {
        self.0.clone()
    }
}

// ---------------------------------------------------------------- legacy comments

/// `p:CT_CommentAuthorList`, the `commentAuthors` part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommentAuthorList {
    /// `cmAuthor`.
    pub authors: Vec<CommentAuthor>,
}

/// `p:CT_CommentAuthor`. Other `extLst` entries stay in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommentAuthor {
    /// `@id`, which a comment's `authorId` names.
    pub id: Option<u32>,
    /// `@name`.
    pub name: Option<String>,
    /// `@initials`.
    pub initials: Option<String>,
    /// `@lastIdx`.
    pub last_idx: Option<u32>,
    /// `@clrIdx`.
    pub clr_idx: Option<u32>,
    /// `p15:presenceInfo`, from `extLst`.
    pub presence: Option<PresenceInfo>,
}

/// `p15:CT_PresenceInfo`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PresenceInfo {
    /// `@userId`.
    pub user_id: Option<String>,
    /// `@providerId`.
    pub provider_id: Option<String>,
}

/// `p:CT_CommentList`, a `comments` part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommentList {
    /// `cm`.
    pub comments: Vec<Comment>,
}

/// `p:CT_Comment`. Other `extLst` entries stay in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Comment {
    /// `@authorId`.
    pub author_id: Option<u32>,
    /// `@dt`.
    pub dt: Option<DateTime>,
    /// `@idx`.
    pub idx: Option<u32>,
    /// `pos`.
    pub pos: Option<Point>,
    /// `text`. Document data.
    pub text: Option<String>,
    /// `p15:threadingInfo`, from `extLst`.
    pub threading: Option<CommentThreading>,
}

/// `p15:CT_CommentThreading`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommentThreading {
    /// `@timeZoneBias`.
    pub time_zone_bias: Option<i32>,
    /// `parentCm`.
    pub parent: Option<ParentComment>,
}

/// `p15:CT_ParentCommentIdentifier`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParentComment {
    /// `@authorId`.
    pub author_id: Option<u32>,
    /// `@idx`.
    pub idx: Option<u32>,
}

// ---------------------------------------------------------------- modern comments

/// `p188:CT_AuthorList`, the `authors` part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthorList {
    /// `author`.
    pub authors: Vec<Author>,
}

/// `p188:CT_Author`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Author {
    /// `@id`.
    pub id: Option<String>,
    /// `@name`.
    pub name: Option<String>,
    /// `@initials`.
    pub initials: Option<String>,
    /// `@userId`.
    pub user_id: Option<String>,
    /// `@providerId`.
    pub provider_id: Option<String>,
}

xml_enum! {
    /// `p188:ST_CommentStatus`.
    CommentStatus {
        Active = "active",
        Resolved = "resolved",
        Closed = "closed",
    }
}

/// `p188:CT_CommentList`, a modern comments part.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThreadedCommentList {
    /// `cm`.
    pub comments: Vec<ThreadedComment>,
}

/// What a modern comment's anchor is (`p188:EG_CommentAnchor`). Which moniker list holds it
/// says what kind of thing it points at. A list's own content stays in the tree, and is
/// reported here as read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentAnchor {
    /// `sldMkLst`: a slide.
    Slide(Vec<Moniker>),
    /// `deMkLst`: a drawing element. The schema allows several.
    Drawing(Vec<Moniker>),
    /// `txMkLst`: a range of text. The schema allows several.
    Text(Vec<Moniker>),
    /// `unknownAnchor`.
    Unknown,
}

/// One element of a moniker list: its local name and attributes, in file order. Read for
/// anyone asking what a comment points at; written back only by the tree, as `REVIEW` retains
/// these (`docs/spec/REVIEW.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moniker {
    /// The element's local name, `sldMk` or `spMk` or another.
    pub kind: String,
    /// Its attributes, name and value.
    pub attributes: Vec<(String, String)>,
}

/// `p188:CT_Comment`, with `p188:AG_CommentProperties` and `p188:EG_CommentProperties`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThreadedComment {
    /// `sldMkLst`, `deMkLst`, `txMkLst` or `unknownAnchor`.
    pub anchor: Option<CommentAnchor>,
    /// `pos`.
    pub pos: Option<Point>,
    /// `replyLst`. A list present and empty is not an absent one.
    pub reply_lst: Option<Vec<CommentReply>>,
    /// `txBody`. Document data.
    pub tx_body: Option<TextBody>,
    /// `@id`.
    pub id: Option<String>,
    /// `@authorId`.
    pub author_id: Option<String>,
    /// `@status`.
    pub status: Option<CommentStatus>,
    /// `@created`.
    pub created: Option<DateTime>,
    /// `@startDate`.
    pub start_date: Option<DateTime>,
    /// `@dueDate`.
    pub due_date: Option<DateTime>,
    /// `@assignedTo`, a list of author ids.
    pub assigned_to: Option<Vec<String>>,
    /// `@complete`.
    pub complete: Option<Written<Percentage>>,
    /// `@title`. Document data.
    pub title: Option<String>,
    /// `p223:reactions`, from `extLst`.
    pub reactions: Option<Reactions>,
    /// `p228:taskDetails`, from `extLst`.
    pub task: Option<TaskDetails>,
}

/// `p188:CT_CommentReply`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommentReply {
    /// `txBody`. Document data.
    pub tx_body: Option<TextBody>,
    /// `@id`.
    pub id: Option<String>,
    /// `@authorId`.
    pub author_id: Option<String>,
    /// `@status`.
    pub status: Option<CommentStatus>,
    /// `@created`.
    pub created: Option<DateTime>,
    /// `p223:reactions`, from `extLst`.
    pub reactions: Option<Reactions>,
}

impl ThreadedComment {
    /// Who a comment's text mentions: the `id` of each `a:fld` of type `mention`, in order.
    /// That form is Inferred: `MS-PPTX` gives no schema for a mention.
    pub fn mentions(&self) -> Vec<&str> {
        mentions_in(self.tx_body.as_ref())
    }
}

impl CommentReply {
    /// Who a reply's text mentions, as [`ThreadedComment::mentions`].
    pub fn mentions(&self) -> Vec<&str> {
        mentions_in(self.tx_body.as_ref())
    }
}

fn mentions_in(body: Option<&TextBody>) -> Vec<&str> {
    let mut out = Vec::new();
    for paragraph in body.iter().flat_map(|b| &b.paragraphs) {
        for item in &paragraph.items {
            if let crate::text::TextItem::Field(field) = item
                && field.field_type.as_deref() == Some("mention")
                && let Some(id) = &field.id
            {
                out.push(id.as_str());
            }
        }
    }
    out
}

/// `p223:CT_Reactions`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reactions {
    /// `rxn`.
    pub rxn: Vec<Reaction>,
}

/// `p223:CT_Reaction`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reaction {
    /// `@type`.
    pub reaction_type: Option<String>,
    /// `instance`.
    pub instance: Vec<ReactionInstance>,
}

/// `p223:CT_ReactionInstance`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReactionInstance {
    /// `@time`.
    pub time: Option<DateTime>,
    /// `@authorId`.
    pub author_id: Option<String>,
}

/// `p228:CT_TaskDetails`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskDetails {
    /// `history`. Required by the schema, and absent when damaged.
    pub history: Option<TaskHistory>,
}

/// `p228:CT_TaskHistory`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskHistory {
    /// `event`.
    pub event: Vec<TaskHistoryEvent>,
}

/// `p228:CT_TaskHistoryEvent`. `extLst` stays in the tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaskHistoryEvent {
    /// `atrbtn`, whose `@authorId` says who did this.
    pub atrbtn: Option<String>,
    /// `anchr`, whose `comment/@id` says which comment it was on.
    pub anchr: Option<String>,
    /// What happened.
    pub action: Option<TaskAction>,
    /// `@time`.
    pub time: Option<DateTime>,
    /// `@id`.
    pub id: Option<String>,
}

/// What a task's history event records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskAction {
    /// `asgn`, with its `@authorId`.
    Assign(Option<String>),
    /// `add`.
    Add,
    /// `title`, with its `@val`. Document data.
    Title(Option<String>),
    /// `date`.
    Date {
        /// `@stDt`.
        st_dt: Option<DateTime>,
        /// `@endDt`.
        end_dt: Option<DateTime>,
    },
    /// `pcntCmplt`, with its `@val`.
    PercentComplete(Option<Written<Percentage>>),
    /// `unasgnAll`.
    UnassignAll,
    /// `undo`, with its `@id`.
    Undo(Option<String>),
    /// `unknown`.
    Unknown,
}

// ---------------------------------------------------------------- what a deck holds

/// One slide's comments, wherever the part was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideComments<T> {
    /// The comment part's name.
    pub part: String,
    /// The slide part that holds the relationship to it.
    pub slide_part: String,
    /// What it says.
    pub list: T,
}

/// Every comment in a deck, in its two forms. A deck can hold both: one of `fixtures/real`
/// holds both author lists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeckComments {
    /// `commentAuthors`.
    pub comment_authors: Option<CommentAuthorList>,
    /// `authors`.
    pub authors: Option<AuthorList>,
    /// Legacy comment parts, by slide.
    pub legacy: Vec<SlideComments<CommentList>>,
    /// Modern comment parts, by slide.
    pub threaded: Vec<SlideComments<ThreadedCommentList>>,
    /// Comment and author parts that could not be read. Set aside: retained byte for byte,
    /// and named in the report. Each opens as nothing (owner instruction 24).
    pub set_aside: Vec<String>,
}
