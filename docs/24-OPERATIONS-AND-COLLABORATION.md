# 24 — Operations, undo and collaboration

Single-user editing needs no server, and OT is dormant at one editor, as in OpenCalc
(its docs/56). The op set is still designed for transform from its first version, because
that cannot be retrofitted cheaply.

## The op set

Closed at a version; growing it is a wire break (its ADR-024). Every op applies purely and
returns its inverse.

| Family | Ops | Conflict rule |
| --- | --- | --- |
| **Slides** | Insert, delete, move, duplicate, set layout, set hidden | Ordered list by stable id; insert-after anchors |
| **Tree** | Insert, delete, reparent, z-reorder, group, ungroup | Same; delete beats concurrent edit of the subtree |
| **Properties** | Set one property on one object: transform, fill, line, effect, adjust value, any scalar | Per-property; the server's order wins, different properties never conflict |
| **Text** | Insert text, delete range, format range, set paragraph property, split and merge paragraph | Character OT inside one body |
| **Table** | Insert or delete row and column, merge, split, set cell property; cell text uses text ops | Structural like OpenCalc's; text as above |
| **Timing** | Add, remove, reorder, set animation property | Ordered list by stable id |
| **Design** | Theme, master and layout edits through the same tree and property ops | As above |
| **Notes, comments, sections** | Text ops on notes; per-element ops otherwise | As above |

**Built** (`casual-slide-transaction`, slices 5 and 6): `SetTransform`, `DeleteNode` and `InsertNode` from Properties and Tree; `InsertText`, `DeleteRange` and `FormatRange` (bold, italic, underline; and, for the Home tab's Font group (`UX-003`), size, Latin font, fill as Font Color and highlight) and `FormatParagraphs` (alignment, bullet and bullet font, left margin and indent of whole paragraphs, its inverse a `Restore`) from Text; and `Restore`, which replaces a body's paragraphs `[first, first + count)` and is the inverse of every text op: the paragraphs it touched, exactly as they were, so an undo gives back the model as read and the writer then changes nothing. `Restore` transforms like a delete and insert of its range. From Slides (slice 8, `VS-08`): `AddPart` and its inverse `DropPart` (a new or duplicated slide's part, always the next ordinal), `InsertSlide` and `RemoveSlide` (the slide list; a removed slide's part stays, so undo lists it again), `MoveSlide`, and `SetSlideShow` (Hide Slide; showing again removes `@show`). New Slide and Duplicate are a `Batch` of `AddPart` and `InsertSlide`; Insert Text Box, Shape and Picture are `InsertNode` of a node read from a template, so it is what the reader gives. From Motion (slice 10, `VS-10`, `animate.rs`): `SetTransition` (a slide's `p:transition`, `None` removes it; Apply To All is a `Batch` of them), `SetTiming` (a slide's whole `p:timing`, the inverse of every effect op, so undo restores the tree as read, ids and all), and `AddEffect`, `RemoveEffect`, `MoveEffect` and `SetEffectStart` over the main sequence: an effect from a file keeps its behaviours whole and is only regrouped, clicks and After Previous steps are rebuilt, and every `cTn/@id` is numbered again with `tn/@val` following. A timing in another shape is refused (`CS-TXN-0008`). Paste and Duplicate of shapes (`UX-003`) are a `Batch` of `InsertNode` with fresh ids, a copied picture's bytes minted as new media; moving or deleting several shapes is a `Batch` of `SetTransform` or `DeleteNode`, one undo step. Set layout is not built. Transform (OT) and the wire are not built yet (`VS-09`).

**Built for Design** (`casual-slide-transaction` `design.rs`, slice 12): `SetColorScheme` and `SetFontScheme` set one master's theme scheme whole, `SetSlideSize` the deck's `sldSz`, and `SetBackground` one part's `cSld/bg`; each sets a value and returns the one it replaced as its inverse. They reach themes and the size through the `Document` trait, which a host implements; the slide parts alone are a document without them, so a design op there is `CS-TXN-0006`. Slide Size is a `Batch` of `SetSlideSize` and a `SetTransform` per top-level shape of every slide, layout and master, so content scaling undoes with it. These are whole-value property sets: under OT they conflict per property as the Properties family does. `SetDocumentProperties` (`FILE-01`) sets the three property parts (core, app, custom) whole, reached through `Document::properties_mut`, and returns what it replaced; File > Info makes one per field. It has no wire form yet, so a shared session refuses it.

**Built for rich objects** (`casual-slide-transaction` `rich.rs`, slice 11, `VS-11`): `SetTable` sets a graphic frame's table whole and `SetChart` a chart part's model whole, each returning the value it replaced as its inverse; charts are reached through `Document::chart_mut`. Typing in a cell, Insert and Delete Row and Column, and a Table Design style are each one `SetTable` of a table built by `rich`'s pure functions (refused on merged cells, `CS-TXN-0013`). Insert Table and Insert Chart are `InsertNode` of a frame read from a template. Whole-value sets: under OT a concurrent edit of the same table or chart conflicts as one property; finer cell ops are a later pass.

**Built for review** (`casual-slide-transaction` `review.rs`, `UX-011`): `SetComments` sets a slide's modern comment list whole and `SetAuthors` the author list, reached through `Document::review_mut`; `SetNotes` links a slide to a notes slide added by `AddPart` (one batch, one undo step; `DropPart` now takes back an unlinked notes slide); `SetNonVisual` sets a shape's `cNvPr` values, its name and `@hidden`, keeping its `@id`. Z-order is `DeleteNode` then `InsertNode`. Each returns the value it replaced. None is sent in a shared session yet (`CS-TXN-0015`). The bridge saves comments by patching or attaching a `modernComment` part, notes through `Op::AddNotes` then the slide patch, and z-order by moving the shapes' elements in `spTree`.

`InsertRun` (`COL-01`) inserts text whose every run and line break has the given run
properties rather than a neighbour's: typed text as it crosses to another editor, and the
undo of a delete in a shared session.

`Batch` makes one undo step and one atomic submission. A drag is a single op committed at
the end; intermediate positions are presence, not edits, so a 60 fps drag never floods the
log. Other users see it live through presence, and it lands once.

**Values on the wire.** No interned id crosses (its ADR-025). Collections are never replaced
whole (its `CHT-07`).

## Text OT

Positions are `(body, paragraph-independent character offset)`; paragraph breaks count as
characters, so split and merge are inserts and deletes of a break with a paragraph-property
payload. Formatting is a range op that transforms like a retained span. TP1 is a property
test over the op generators, run the way OpenCalc runs its transform tests.

The conflict that needs a stated rule: format over a concurrently deleted range formats what
remains, and text inserted inside a formatted range inherits from the character before it,
as typing does.

Whole-body replace was rejected: two people in one text box would overwrite each other,
which is the most common collaboration in a slide deck.

## Undo

Local undo reverts my own last step, rebased over what others have done since, following
OpenCalc's collaborative undo policy (its docs/69) and its decision that clients rebase
rather than merge (its ADR-031). If the object is gone the step is skipped and reported.

**Built** (`COL-04`): in a shared session `History` keeps each step's undo in a form that
travels (`casual-slide-transaction` `shared.rs`): typing is undone by a delete, a delete by
typing the deleted runs back as `InsertRun`s with their own run properties, a format by the
old value per stretch of characters, and shape transforms, inserts and deletes by their own
inverses. Every arrival is folded into the undo and redo steps with the server's transform
(`collab::rebase_history`): the newest step over the arrival, the arrival over that step,
the next step over what it became. An undo is then an ordinary edit, sent and ordered as
any other, so nothing on the wire says it is one (OpenCalc's docs/69). Alone, undo stays a
`Restore`, exact. What a shared undo of a delete does not give back is `COL-08`.

## The server

Fork, with a seam. OpenCalc's server is 18.3k lines and imports its transaction, import,
export, formula, model and eval crates. Transport, cluster, relay, presence, tokens and
lifecycle look engine-independent; the document-facing code does not. Plan:

1. Copy into `server/casual-slide-collab-server`.
2. Define a trait in `-transaction`: apply, transform, snapshot read and write, protocol messages.
3. Move every engine-specific call behind it and compile against the slide engine.

The trait is written so OpenCalc could adopt it later. It is not required to, and nothing
here changes that repo.

### The audit (step 1, `VS-09`, 2026-11-08)

Read-only, OpenCalc at `b23c67c`. `server/casual-calc-collab-server` is 16,126 lines of Rust
in 23 files, 8,800 of them tests. Engine-specific, by what each file imports:

| Part | Lines (code, tests) | Engine-specific? |
| --- | --- | --- |
| `document.rs` | 610 | **Yes, wholly**: imports, recalculates, exports a workbook and applies every operation to it |
| `net.rs` (transport, ordering, fan-out) | 3,090, 4,347 | **Partly**: 10 of its 3,090 lines name the engine, all through the transaction crate's `session` and `protocol` (`ClientId`, `Submission`, `Commit`, `ClientMessage`, `ServerMessage`, `SnapshotPolicy`, `MAX_FRAME_BYTES`); the rest is axum, tokio and policy |
| `token.rs` | 612, 482 | **Slightly**: an access level is checked against the operation kinds (`SheetFields::COMMENTS`), else JWT claims |
| `relay.rs`, `cluster/` | 202; 2,075, 2,340 | Carries `WireOperation` and `Submission` as data; Redis, leases, election engine-free |
| `presence.rs` | 414 | Names `Draft` and `ClientId` only |
| `verify.rs`, `config.rs`, `http.rs`, `lifecycle.rs`, `memory.rs`, `main.rs` | 2,742, 1,334 | No; `main.rs` names `SnapshotPolicy` |

So the engine-specific surface is the document session, about 5% of the code, plus a few
types threaded through the rest. Behind it in the transaction crate: `session.rs` (1,210),
`protocol.rs` (1,198), `wire.rs` (1,122), `transform.rs` (1,638): the client and server
sessions are generic in shape and only name `Workbook` and `Operation`.

### What `VS-09` built

`casual-slide-transaction::collab` (ADR-020) is the seam: `Engine` (a wire operation, its
transform, the protocol version) and `Replica` (apply an arrival, write and read a snapshot),
with OpenCalc's `ClientSession` and `ServerSession` made generic over them and its protocol
messages. Slides' `WireOp` carries Text (insert, delete, format), Properties (set
transform) and Tree (insert a node, given as its XML element and read on arrival through the
hardened reader, and delete one); every other operation is refused in a shared session
before it is applied (`CS-TXN-0015`), so copies never part because an edit could not be
sent. Rules: per property, the later in server order stands; character OT in one body; a
parent's children as a list. **Two rules are stated here for the first time:** a delete
takes text typed inside it or at its edges, because typed text takes its format from a
neighbour when applied and a delete changes the neighbour; and a format grows over text
typed where that text would take the format, judged by the sender at the range's edges
(`typing_takes_after`). TP1 is a property test over the real model, including transforms of
what transform made; a two-client simulation through `ServerSession` converges.

**Protocol 2** (`COL-01`, `COL-06`, `COL-04`) replaced those two rules. Typed text crosses
with the run properties it took at the sender (`InsertText`'s `run`, an `a:rPr` the writer
wrote), so where it lands no longer decides its format: text typed inside a concurrently
deleted range stays, at the cut; a format grows over text typed strictly inside it, the
insert picking the format up (`formats`), and never at its edges. Format carries every Font
group attribute: size as a number, font, colour and highlight as an `a:rPr` holding that one
element. An inserted node's XML is written by the writer from the model, as a save writes a
new shape, so Paste and Duplicate travel; a picture's bytes travel with its node, base64,
at most `MAX_WIRE_MEDIA` (32 MiB), sniffed as PNG or JPEG on arrival. Each participant mints
node ids, `cNvPr/@id`s and relationship ids from its own range. Formats now converge too:
the two-client simulation compares every character's bold, italic, underline and size.

`server/casual-slide-collab-server` is the copied server, in part (its `PROVENANCE.md`): one
WebSocket per participant, a join first, ordering, acknowledgement, relay, presence and
colours. **It holds no model**: the first participant seeds a room with the deck's bytes, and
a joiner receives them with the log. Its CI job (`collab.yml`) starts it and runs two
clients over real sockets.

The browser half (`COL-04`) followed: the bridge's shared session in `casual-slide-wasm`
`collab.rs`, `webapp/collab.js` and `editor.presence.js`, checked in two Chrome tabs; its CI
browser run is left. Not done: tokens, resume, the cluster and the
integrator's save are `COL-03` and `COL-05`; OpenCalc's chained pipelining diverged in the
simulation, so a client keeps one chunk in flight (`COL-02`); shape look, slides, motion,
design, tables and charts are refused while shared (`COL-07`). Undo in a shared session
works for typing, deletes, formats, moves, inserts and deletes of shapes (§Undo).

## Presence

Remote carets and text selections, selected objects with the user's colour, and in-flight
drags. Selection maps through ops in `-transaction`. Presence is never logged and never in
the undo stack.
