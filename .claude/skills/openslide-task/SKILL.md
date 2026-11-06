---
name: openslide-task
description: Run one tracked OpenSlide task from row to commit. Use when starting any implementation, fix or doc task in this repository.
---
1. Find the row in `docs/14-EXECUTION-TRACKER.md`, or add one after reading the id space. Set it `In progress`. State the acceptance test before writing code. Work with no row is not allowed.
2. Check the tier or phase is open in `docs/06-ROADMAP-AND-DELIVERY.md`. If not, it needs a recorded waiver from the owner.
3. Read the design note that owns the area. If an ADR trigger in `docs/08` fires, stop for an Accepted ADR.
4. Write the failing test. Run it and keep the failure output.
5. Implement to `CLAUDE.md`'s engineering standards.
6. Check by running, then ask what would have to be true for this to be wrong, and go and look. Revert the fix to confirm the test goes red, then restore.
7. Run the gates listed in `CLAUDE.md`.
8. Use `openslide-docs` to update every affected document, in the same commit.
9. Mark `Done` only per Definition of Done in `docs/05`, with evidence in the row. If partial, stay `In progress` and say what remains. New findings become rows.
10. Commit with Conventional Commits, citing the row.
