//! Collaboration: the seam a co-editing server and its clients meet the engine through
//! (`VS-09`, `docs/24` §The server).
//!
//! Copied from OpenCalc's `casual-calc-transaction` `session.rs` and `protocol.rs` at
//! `b23c67c` (PROVENANCE.md), and made generic. OpenCalc's sessions name its `Workbook`
//! and `Operation` directly, which is why its server imports five engine crates; here
//! they name only the two traits below, so the server compiles against no model at all
//! and OpenCalc could adopt the same traits later (`docs/24`).
//!
//! - [`Engine`] is what the **server** needs: an operation type that crosses the wire,
//!   and how one operation is rebased over another (transform). The server never
//!   applies an operation and never reads a deck: it orders, transforms and relays.
//! - [`Replica`] is what a **client** needs besides: apply an operation to its copy of
//!   the document, and write and read the snapshot a joiner starts from.
//!
//! The rules (ADR-011 in OpenCalc, kept): one server orders every edit; a client
//! applies its own edits at once and sends them in chunks; the server rebases each
//! chunk over what was committed since the chunk's base and relays it as it landed; a
//! client rebases what arrives over its own unacknowledged edits. Every replica then
//! applies the same operations in the same order, so they converge, provided
//! [`Engine::transform`] meets TP1 (`docs/24` §Text OT), which `tests.rs` checks over
//! the real model.

use std::collections::BTreeMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

mod undo;
pub mod wire;

pub use undo::rebase_history;

#[cfg(test)]
mod tests;

/// Which of two concurrent operations the server ordered first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The operation being rebased was ordered before the one it is rebased over.
    Earlier,
    /// It was ordered after: it wins a tie (the server's order wins, `docs/24`).
    Later,
}

impl Side {
    /// The other side.
    #[must_use]
    pub fn flip(self) -> Self {
        match self {
            Self::Earlier => Self::Later,
            Self::Later => Self::Earlier,
        }
    }
}

/// The server's half of the seam: an operation that crosses the wire, and transform.
pub trait Engine {
    /// One operation as it travels. Plain data: no interned id crosses, as in
    /// OpenCalc (`docs/24` §Values on the wire).
    type Op: Clone + core::fmt::Debug + PartialEq + Serialize + DeserializeOwned;

    /// The conversation's version; two peers that disagree stop at the handshake.
    const PROTOCOL: u32;

    /// `op`, rebased so it applies after `over`. `side` says which the server ordered
    /// first. Must meet TP1: `over` then `transform(op, over, Later)` leaves the same
    /// document as `op` then `transform(over, op, Earlier)`.
    fn transform(op: &Self::Op, over: &Self::Op, side: Side) -> Self::Op;

    /// Whether `op` changes nothing, so it need not be applied or relayed.
    fn is_noop(op: &Self::Op) -> bool;
}

/// The client's half of the seam: a copy of the document.
pub trait Replica<E: Engine> {
    /// Why an operation or a snapshot could not be applied.
    type Error: core::fmt::Debug;

    /// Apply an operation that arrived from the server.
    fn apply_remote(&mut self, op: &E::Op) -> Result<(), Self::Error>;

    /// The snapshot a joiner starts from: the document's bytes.
    fn write_snapshot(&self) -> Result<Vec<u8>, Self::Error>;

    /// Replace this replica's document with a snapshot's.
    fn read_snapshot(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}

/// An error in a collaborative session. Each has a stable code (`docs/20`).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SessionError {
    /// A chunk named a revision the server does not hold: older than its log or newer
    /// than its present.
    UnknownRevision {
        /// The revision named.
        claimed: u64,
        /// The oldest the server can rebase from.
        oldest: u64,
        /// The server's present.
        current: u64,
    },
    /// An operation from the server could not be applied here, so this replica no
    /// longer agrees and must join again.
    Desynced,
}

impl SessionError {
    /// The stable diagnostic code (`docs/20`).
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnknownRevision { .. } => "CS-COL-0001",
            Self::Desynced => "CS-COL-0002",
        }
    }
}

impl core::fmt::Display for SessionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownRevision {
                claimed,
                oldest,
                current,
            } => write!(
                f,
                "{}: revision {claimed} is outside {oldest}..={current}",
                self.code()
            ),
            Self::Desynced => write!(f, "{}: this copy no longer agrees", self.code()),
        }
    }
}

impl std::error::Error for SessionError {}

/// A participant, numbered by the server. The engine never invents identity.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct ClientId(pub u64);

/// What a chunk was written on top of (OpenCalc's ADR-016).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Base {
    /// Written against this revision: the first chunk after a join or a resume.
    Revision(u64),
    /// Written on top of this client's previous chunk, wherever it landed.
    Chained,
}

/// A chunk of a client's own edits, offered to the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", bound = "")]
pub struct Submission<E: Engine> {
    /// Who sent it.
    pub client: ClientId,
    /// Which chunk from that client, counted from one: with `client`, an idempotency
    /// key, so a resend after a lost acknowledgement is not applied twice.
    pub seq: u64,
    /// What the operations were written against.
    pub base: Base,
    /// The operations, in the order the client made them.
    pub ops: Vec<E::Op>,
}

/// What committing a chunk did.
#[derive(Debug, Clone, PartialEq)]
pub enum Commit<E: Engine> {
    /// Newly committed: the operations as they landed, and the revision reached.
    Applied {
        /// The rebased operations, which every other participant is sent.
        ops: Vec<E::Op>,
        /// The revision the document reached.
        revision: u64,
    },
    /// Already committed by an earlier delivery of this chunk; nothing applied twice.
    Duplicate {
        /// Where it landed the first time.
        revision: u64,
    },
}

/// How many chunks a client keeps in flight before it stops making more.
///
/// One: stop and wait. OpenCalc pipelines up to 32 with a `Chained` base (its
/// ADR-016), but a chained chunk is rebased only over what was committed after the
/// chunk before it landed, while it was written before the client had seen what was
/// committed ahead of that chunk; the two-client test here diverged under it
/// (`COL-02`). With one chunk in flight, each chunk names the revision it was written
/// against, which includes everything the client has received.
const MAX_OUTSTANDING: usize = 1;

/// The largest frame either end sends or accepts, in bytes. A snapshot travels in
/// one, base64 in JSON, so this bounds the deck a room can share.
pub const MAX_FRAME_BYTES: usize = 48 * 1024 * 1024;

#[derive(Debug, Clone)]
struct Outstanding<Op> {
    seq: u64,
    ops: Vec<Op>,
}

/// A client's side of a session: its unacknowledged edits, and the rebase of what
/// arrives over them. Copied from OpenCalc's `ClientSession`.
#[derive(Debug, Clone)]
pub struct ClientSession<E: Engine> {
    client: ClientId,
    revision: u64,
    sent: Vec<Outstanding<E::Op>>,
    pending: Vec<E::Op>,
    chunks: u64,
    desynced: bool,
}

impl<E: Engine> ClientSession<E> {
    /// A session joined as `client` at `revision`.
    #[must_use]
    pub fn new(client: ClientId, revision: u64) -> Self {
        Self {
            client,
            revision,
            sent: Vec::new(),
            pending: Vec::new(),
            chunks: 0,
            desynced: false,
        }
    }

    /// Carry on after a reconnect the server resumed.
    pub fn resume(&mut self, client: ClientId, revision: u64) {
        self.client = client;
        self.revision = revision;
    }

    /// This participant.
    #[must_use]
    pub fn id(&self) -> ClientId {
        self.client
    }

    /// The last revision this replica has.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Whether any own edit is not yet acknowledged.
    #[must_use]
    pub fn has_unacknowledged(&self) -> bool {
        !self.sent.is_empty() || !self.pending.is_empty()
    }

    /// Note an edit already applied to the local document, to be sent.
    pub fn record(&mut self, op: E::Op) {
        if !E::is_noop(&op) {
            self.pending.push(op);
        }
    }

    /// The next chunk to send, if any.
    pub fn flush(&mut self) -> Option<Submission<E>> {
        if self.desynced || self.pending.is_empty() || self.sent.len() >= MAX_OUTSTANDING {
            return None;
        }
        let base = if self.sent.is_empty() {
            Base::Revision(self.revision)
        } else {
            Base::Chained
        };
        self.chunks += 1;
        let chunk = Outstanding {
            seq: self.chunks,
            ops: core::mem::take(&mut self.pending),
        };
        let submission = Submission {
            client: self.client,
            seq: chunk.seq,
            base,
            ops: chunk.ops.clone(),
        };
        self.sent.push(chunk);
        Some(submission)
    }

    /// Every chunk in flight again, for a resumed connection.
    #[must_use]
    pub fn resend(&self) -> Vec<Submission<E>> {
        if self.desynced {
            return Vec::new();
        }
        self.sent
            .iter()
            .enumerate()
            .map(|(i, chunk)| Submission {
                client: self.client,
                seq: chunk.seq,
                base: if i == 0 {
                    Base::Revision(self.revision)
                } else {
                    Base::Chained
                },
                ops: chunk.ops.clone(),
            })
            .collect()
    }

    /// The server committed every chunk through `through`; the document is at
    /// `revision`.
    pub fn acknowledge(&mut self, through: u64, revision: u64) {
        self.sent.retain(|chunk| chunk.seq > through);
        self.revision = revision;
    }

    /// Whether an arrival failed to apply, so the replica must join again.
    #[must_use]
    pub fn is_desynced(&self) -> bool {
        self.desynced
    }

    /// Apply an operation another participant made, as the server ordered it, rebased
    /// over this client's own edits not yet acknowledged; and rebase those over it.
    /// Returns the operation as it was applied here.
    pub fn receive<R: Replica<E>>(
        &mut self,
        replica: &mut R,
        incoming: &E::Op,
        revision: u64,
    ) -> Result<E::Op, SessionError> {
        if self.desynced {
            return Err(SessionError::Desynced);
        }
        let mut arriving = incoming.clone();
        let mut rebased: Vec<E::Op> = Vec::new();
        let outstanding = self.sent.iter().flat_map(|chunk| chunk.ops.iter());
        for local in outstanding.chain(self.pending.iter()) {
            let next = E::transform(&arriving, local, Side::Earlier);
            rebased.push(E::transform(local, &arriving, Side::Later));
            arriving = next;
        }
        if !E::is_noop(&arriving) && replica.apply_remote(&arriving).is_err() {
            self.desynced = true;
            return Err(SessionError::Desynced);
        }
        let outstanding = self.sent.iter_mut().flat_map(|chunk| chunk.ops.iter_mut());
        for (slot, op) in outstanding.chain(self.pending.iter_mut()).zip(rebased) {
            *slot = op;
        }
        self.revision = revision;
        Ok(arriving)
    }
}

/// The server's side of a session: the ordered log and who has had what accepted.
/// Copied from OpenCalc's `ServerSession`, without the document: the server here
/// never applies an operation, so it holds no model of the deck.
#[derive(Debug, Clone)]
pub struct ServerSession<E: Engine> {
    revision: u64,
    log: Vec<E::Op>,
    first: u64,
    accepted: BTreeMap<ClientId, (u64, u64)>,
}

impl<E: Engine> Default for ServerSession<E> {
    fn default() -> Self {
        Self {
            revision: 0,
            log: Vec::new(),
            first: 0,
            accepted: BTreeMap::new(),
        }
    }
}

impl<E: Engine> ServerSession<E> {
    /// A new session at revision 0.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The revision the document has reached.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Every operation committed since `revision`, if the log still holds them.
    #[must_use]
    pub fn history_since(&self, revision: u64) -> Option<&[E::Op]> {
        if revision < self.first || revision > self.revision {
            return None;
        }
        let skip = usize::try_from(revision - self.first).ok()?;
        self.log.get(skip..)
    }

    /// Order a chunk: rebase it over everything committed since its base, append it
    /// to the log, and say how it landed. A resend of a committed chunk is
    /// [`Commit::Duplicate`].
    pub fn commit(&mut self, submission: &Submission<E>) -> Result<Commit<E>, SessionError> {
        if let Some(&(seq, revision)) = self.accepted.get(&submission.client)
            && submission.seq <= seq
        {
            return Ok(Commit::Duplicate { revision });
        }
        let base = match submission.base {
            Base::Revision(revision) => revision,
            Base::Chained => match self.accepted.get(&submission.client) {
                Some(&(_, landed)) => landed,
                None => {
                    return Err(SessionError::UnknownRevision {
                        claimed: 0,
                        oldest: self.first,
                        current: self.revision,
                    });
                }
            },
        };
        let Some(since) = self.history_since(base) else {
            return Err(SessionError::UnknownRevision {
                claimed: base,
                oldest: self.first,
                current: self.revision,
            });
        };
        let mut history: Vec<E::Op> = since.to_vec();
        let mut rebased = Vec::with_capacity(submission.ops.len());
        for op in &submission.ops {
            let mut current = op.clone();
            for committed in &mut history {
                let next = E::transform(&current, committed, Side::Later);
                *committed = E::transform(committed, &current, Side::Earlier);
                current = next;
            }
            rebased.push(current);
        }
        for op in &rebased {
            self.log.push(op.clone());
            self.revision += 1;
        }
        self.accepted
            .insert(submission.client, (submission.seq, self.revision));
        Ok(Commit::Applied {
            ops: rebased,
            revision: self.revision,
        })
    }
}

/// Where a participant is and what they have selected: presence, never an operation,
/// never logged and never undone (`docs/24` §Presence).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Presence {
    /// The slide they are on, by its index in the deck.
    pub slide: u32,
    /// The shapes they have selected, by node.
    pub shapes: Vec<u32>,
    /// The shape whose text they are in, by node, with the caret and the selection's
    /// other end as character offsets.
    pub caret: Option<Caret>,
}

/// A caret inside one text body.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Caret {
    /// The shape, by node.
    pub node: u32,
    /// Where the caret is.
    pub offset: u32,
    /// The other end of the selection; the caret alone when equal.
    pub anchor: u32,
}

/// What a client says to the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", bound = "")]
pub enum ClientMessage<E: Engine> {
    /// Join a room. The first to join a room seeds it with `seed`, the deck's bytes
    /// in base64; a later joiner's seed is ignored.
    Join {
        /// [`Engine::PROTOCOL`].
        protocol: u32,
        /// The name shown beside this participant's cursor.
        name: String,
        /// The deck, when this participant may be the first.
        seed: Option<String>,
    },
    /// A chunk of own edits.
    Submit(Submission<E>),
    /// Where this participant is.
    Presence(Presence),
    /// Still here.
    Heartbeat,
    /// Is the connection alive? Answered with [`ServerMessage::Pong`].
    Ping {
        /// Echoed back.
        nonce: u64,
    },
    /// Leaving.
    Leave,
}

/// What the server says to a client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", bound = "")]
pub enum ServerMessage<E: Engine> {
    /// Joined: the deck as seeded, base64, and every operation since, which the client
    /// applies in order to reach `revision`.
    Welcome {
        /// [`Engine::PROTOCOL`].
        protocol: u32,
        /// This participant's number.
        client: ClientId,
        /// Its colour, `#rrggbb`.
        colour: String,
        /// The seed.
        snapshot: String,
        /// Operations since the seed.
        ops: Vec<E::Op>,
        /// The revision they reach.
        revision: u64,
    },
    /// Own chunks through `through` are committed; the document is at `revision`.
    Ack {
        /// The last chunk committed.
        through: u64,
        /// The revision reached.
        revision: u64,
    },
    /// Another participant's operations, as they landed.
    Apply {
        /// The operations.
        ops: Vec<E::Op>,
        /// The revision they reach.
        revision: u64,
    },
    /// Another participant's presence.
    Presence {
        /// Who.
        client: ClientId,
        /// Their name.
        name: String,
        /// Their colour.
        colour: String,
        /// Where they are.
        #[serde(flatten)]
        presence: Presence,
    },
    /// A participant left.
    Departed {
        /// Who.
        client: ClientId,
    },
    /// The answer to a ping.
    Pong {
        /// The ping's nonce.
        nonce: u64,
    },
    /// A message was refused; the connection stays.
    Refused {
        /// Why, as a stable code.
        reason: String,
    },
    /// The session ended for this connection.
    Stopped {
        /// Why, as a stable code.
        reason: String,
    },
}
