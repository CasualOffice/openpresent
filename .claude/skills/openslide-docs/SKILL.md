---
name: openslide-docs
description: Bring all OpenSlide documents into line with a change, or audit them. Use before every commit that changes behaviour or status, and when asked to verify docs.
---
**Before a commit:** walk the table under "Update every document" in `CLAUDE.md` against `git diff`. For each changed thing, update the document in the same commit. Then run `python3 tools/check-repo.py`, and `tools/spec-checklist/generate.py --check` if schemas or status changed.

Rules:
- `docs/06` is the only roadmap and `docs/14` the only tracker. Add nothing parallel to them.
- Present tense means true now. Designs not built say so.
- Numbers come from a command, not memory. "Measured" means measured, on a named environment.
- A contract the code does not keep is a tracker row, not a softened sentence.
- New documents take the next number and go in `docs/00`.
- **Docs are true as of a commit (owner instruction 58).** `README.md` §Status, `AGENTS.md` §Current state and `docs/00` each carry one line, `Docs are true as of <short hash> (<date>).`, naming the `main` commit they were last checked against. Move it only after checking.

**Auditing:** use a fresh session, so the check is not anchored on the author. Check each claim by running or reading something, with `file:line`. Report what was checked as well as what failed. Each finding becomes a fix or a row.
