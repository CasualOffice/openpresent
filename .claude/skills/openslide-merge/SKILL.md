---
name: openslide-merge
description: Verify and merge a parallel worker's branch into OpenSlide. Use when a worktree session reports completion.
---
A worker's report is not evidence.

1. `git diff --stat base..branch`: confirm it stayed in its owned files. Strays are rejected or examined.
2. Check out the branch. Run the gates in `CLAUDE.md`.
3. Pick the most important tests. Revert the fix, watch them go red, restore. Spend commands, not another agent.
4. Spot-check claimed coverage against `docs/spec/checklist.tsv`. A claimed row needs code and a fixture. If nothing you check is refuted, wonder whether you checked hard enough.
5. Merge into `main`. Re-run the gates on the merged result, then push `main` (no pull requests, owner instruction 17).
6. Update `docs/14` and the checklist, and anything else via `openslide-docs`. Record what the worker found, and what made it invisible. Findings not fixed become rows.
7. Commit, then remove the worktree once merged.
