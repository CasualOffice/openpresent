# Provenance

Started from `crates/casual-calc-transaction` in OpenCalc (`../opencalc`) at commit `b23c67c`,
on 2026-11-08 (owner instruction 42, `VS-05`). Same owner, Apache-2.0.

Taken from it, and changed only where slides need it:

- The contract: every operation applies purely to the model and returns its **inverse**, so
  undo and redo are inverse replay and never a second implementation that can drift.
- `History`: paired undo and redo stacks over `apply`; an operation that changed nothing
  leaves no undo entry and keeps the redo stack; `undo_mark` and `group_since` fold a gesture
  (here, a burst of typing) into one undo step; `edits_applied` counts up and never down, so
  a host can ask whether there is unsaved work. The bound on depth is kept.
- `Batch`: one undo step, applied all or nothing.

Changed: the operations are slides' text operations from `docs/24` (insert text, delete
range, format range) over a shape's text body, addressed by `ObjectId` and a character
offset in the body, where a paragraph break counts as one character. The inverse of each is
`Restore`, which puts back the paragraphs the operation touched exactly as they were, so an
undo gives back the very model that was read and the writer then changes nothing.

Not taken: cell operations, structural row and column shifts, formula rewriting, the wire
format, sessions, transform and versions. Transform and the wire arrive with collaboration
(`VS-09`).
