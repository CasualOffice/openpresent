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
OpenCalc's collaborative undo policy (its docs/69) and owner ruling that clients rebase
rather than merge (its ADR-031). If the object is gone the step is skipped and reported.

## The server

Fork, with a seam. OpenCalc's server is 18.3k lines and imports its transaction, import,
export, formula, model and eval crates. Transport, cluster, relay, presence, tokens and
lifecycle look engine-independent; the document-facing code does not. Plan:

1. Copy into `server/casual-slide-collab-server`.
2. Define a trait in `-transaction`: apply, transform, snapshot read and write, protocol messages.
3. Move every engine-specific call behind it and compile against the slide engine.

The trait is written so OpenCalc could adopt it later. It is not required to, and nothing
here changes that repo. I have sized this by imports, not by reading all 18k lines, so
Phase 9 opens with a read-only audit before the estimate is trusted.

## Presence

Remote carets and text selections, selected objects with the user's colour, and in-flight
drags. Selection maps through ops in `-transaction`. Presence is never logged and never in
the undo stack.
