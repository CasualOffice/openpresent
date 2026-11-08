# Provenance

Started from `crates/casual-calc-transaction` in OpenCalc (`../opencalc`) at commit `b23c67c`,
on 2026-11-08 (owner instruction 42, `VS-05`). Same owner, Apache-2.0.

Taken from it, and changed only where slides need it:

- The contract: every operation applies purely to the model and returns its **inverse**, so
  undo and redo are inverse replay and never a second implementation that can drift.
- `History`: paired undo and redo stacks over `apply`; an operation that changed nothing
  leaves no undo entry and keeps the redo stack; `undo_mark` and `group_since` fold a gesture
  (here, a burst of typing) into one undo step; `edits_applied` counts up and never down, so
  a host can ask whether there is unsaved work. The bound on depth is kept; the hold of a
  mark past the bound (`HOLD_CEILING`) is not, since a typing burst is far under it.
- `Batch`: one undo step, applied all or nothing.

Changed: the operations are slides' text operations from `docs/24` (insert text, delete
range, format range) over a shape's text body, addressed by `ObjectId` and a character
offset in the body, where a paragraph break counts as one character. The inverse of each is
`Restore`, which puts back the paragraphs the operation touched exactly as they were, so an
undo gives back the very model that was read and the writer then changes nothing.

Also here: slice 6's shape operations (`VS-06`: set a node's transform, delete and insert
a node), first written in `casual-slide-wasm/src/txn.rs` in the same shape and moved here
with their tests (`shape_tests.rs`), so there is one op set and one undo stack.

Not taken: cell operations, structural row and column shifts, formula rewriting and
versions.

Collaboration (`VS-09`, `collab/`): `session.rs`'s `ClientSession` and `ServerSession` and
`protocol.rs`'s messages, made generic over the `Engine` and `Replica` traits, so the server
names no model. Changed: one chunk in flight rather than 32 chained (`COL-02`); no `Draft`,
`Resume`, snapshot policy or chunk budget; the server holds no document. Slides' wire
operations and their transform (`collab/wire.rs`) are new, after `docs/24`'s rules.
