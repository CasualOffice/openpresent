# 03 — Owner instructions

**Binding, and first in authority.** Every instruction from the owner is recorded here, close to their words, with
where it is applied. Work follows these without deviation. Where a document, a design or an agent's judgement disagrees,
this page wins. If an instruction seems wrong or conflicts with another, raise it with the owner; do not quietly choose.

Add a row in the same turn an instruction is given. Rows are never edited away; a later instruction supersedes an earlier one
by saying so.

| # | Date | Instruction | Applied in |
| --- | --- | --- | --- |
| 1 | 2026-11-07 | Build a PowerPoint Online clone: similar UI, similar UX, as far as we can, with the same features. OpenCalc is the sister repo and our architecture | [01](01-ORD.md), [02](02-ARCHITECTURE.md), [47](47-POWERPOINT-SHELL-AND-FEATURE-MAP.md) |
| 2 | 2026-11-07 | Build in this folder. Copy from the sister repo and make use of it | `DEC-02`, [19](19-WORKSPACE-SCAFFOLD-DESIGN.md) |
| 3 | 2026-11-07 | Study PowerPoint's complete feature list and create the roadmap. **Fidelity is most critical**, with layout and modelling. **Import, export and modelling are the first priority.** We build our own modelling | [48](48-POWERPOINT-FEATURE-STUDY.md), [06](06-ROADMAP-AND-DELIVERY.md), [51](51-SPEC-CHECKLIST-AND-FIDELITY.md) |
| 4 | 2026-11-07 | **Enterprise grade, not an MVP** | [10](10-ENTERPRISE-REQUIREMENTS.md), [06](06-ROADMAP-AND-DELIVERY.md) |
| 5 | 2026-11-07 | Full spec checklist, with properties and tags, in the roadmap. A fully mapped-out, complete roadmap | [spec/](spec/README.md), [06](06-ROADMAP-AND-DELIVERY.md) |
| 6 | 2026-11-07 | Take proper precautions: zip bombs and other security threats | [07](07-SECURITY-THREAT-MODEL.md) |
| 7 | 2026-11-07 | Proceed and start implementation; `git init` first | Done; [14](14-EXECUTION-TRACKER.md) |
| 8 | 2026-11-07 | Support **both web and desktop**, as OpenCalc does | `DEC-06` |
| 9 | 2026-11-07 | Work faster. **Use parallel sessions**; do not block everything on one thread or one mistake | [CLAUDE.md](../CLAUDE.md), `openslide-worker`, `openslide-merge` skills |
| 10 | 2026-11-07 | Phases must clearly define their opening and closing, and what defines a phase or task. Verify the docs. Analyse missing gaps, and what to add or modify | [05](05-PHASES-AND-TASKS.md), [06](06-ROADMAP-AND-DELIVERY.md), `DOC-003`, `DOC-004` |
| 11 | 2026-11-07 | Follow industry-standard coding practices. Follow **one roadmap and one tracker**. Update all documents. Keep these in `CLAUDE.md`. Add appropriate skills | [CLAUDE.md](../CLAUDE.md), `.claude/skills/`, `tools/check-repo.py` |
| 12 | 2026-11-07 | The goal is a clone. Make appropriate decisions without asking, thinking of the goal and the future, and do not regret them later | `DEC-*` in [14](14-EXECUTION-TRACKER.md) |
| 13 | 2026-11-07 | LibreOffice is already installed: use it. Do the UI and UX study through online research | `DEC-04`, `UX-001` |
| 14 | 2026-11-07 | Note everything instructed, and do not deviate | This page, [CLAUDE.md](../CLAUDE.md) |
| 15 | 2026-11-07 | Use cyan as the colour | `DEC-03` |
| 16 | 2026-11-07 | Complete Phase 0 soon | Phase 0 exit criteria prioritised over other work; workers commit each criterion as soon as it is done |
| 17 | 2026-11-07 | Push directly to `main`; no pull requests for now. Supersedes CLAUDE.md's rule against committing to `main` until the owner says otherwise | [CLAUDE.md](../CLAUDE.md), `openslide-merge` skill || 18 | 2026-11-07 | Update the README and repo overview. While Phase 0 completes, plan out all of Phase 1 | README, GitHub description and topics; [06](06-ROADMAP-AND-DELIVERY.md) Phase 1 plan; task rows in [14](14-EXECUTION-TRACKER.md) |

## Boundaries set by the environment

| Date | What | Status |
| --- | --- | --- |
| 2026-11-07 | Creating a GitHub remote and pushing was denied by the session's permission system | Resolved: the owner created the repo and pushed. Not worked around |
