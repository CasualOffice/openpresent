---
name: openslide-worker
description: Brief a parallel OpenSlide session. Use when splitting independent work across worktree-isolated agents.
---
Parallelism is decided by invariants: two workers may run together only if they cannot touch the same invariant or file. Two tasks in one file are one worker, in sequence. Read-only audits can run freely.

Commit a shared base first, with skeleton files for each worker, so they do not collide on `Cargo.toml` or `lib.rs`.

Each brief has: the job and its tracker row; documents to read; **the files it owns, and that everything else is off limits**, including `docs/14` and `docs/spec`, which the main session updates; design decisions already fixed; the acceptance test, which for tier work is the tier's whole exit gate in docs/06, not just the code; the gates to run; that it commits on its worktree branch with the attribution line and does not push; what to report: branch, a test seen red then green with verbatim output, rows truly covered and not, defects found. Tell it not to stop for questions: make the sensible call and note it. **A failing gate is reported, never routed around:** no `--no-verify`, no symlinks or edits outside its files.

Use fresh workers. Everything reuse seems to buy comes from a better brief. Do not ask a worker to verify another worker's claims by plausibility.
