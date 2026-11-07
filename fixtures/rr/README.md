# Repair-risk fixtures

One pair per invariant of `docs/36`, for `PV-01` to open. Nothing here has been opened in PowerPoint.

- `good/RR-*.pptx`: the deck after the stated edit, through the writer. F4 and the SDK validator pass them, against the deck each started from.
- `seeded/RR-*.pptx`: the same plan with the one invariant broken. Preflight refuses this; the file exists because the break was written with the check left out, and `manifest.tsv` says what became of it as an input.
- `manifest.tsv`: fixture, invariant, input deck, good file, and the seeded deck's fate.

`crates/casual-slide-export/tests/writer_rr.rs` writes these with `RR_WRITE=1`, and a plain run holds the writer to them, part by part. Their inputs are `fixtures/generated/rr-*`, from `fixtures/tools/generate_rr.py`. Seeded decks are invalid on purpose: they are not corpus, and no validator gate runs on them.
