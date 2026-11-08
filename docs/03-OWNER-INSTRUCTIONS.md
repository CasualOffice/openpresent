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
| 17 | 2026-11-07 | Push directly to `main`; no pull requests for now. Supersedes CLAUDE.md's rule against committing to `main` until the owner says otherwise | [CLAUDE.md](../CLAUDE.md), `openslide-merge` skill |
| 18 | 2026-11-07 | Update the README and repo overview. While Phase 0 completes, plan out all of Phase 1 | README, GitHub description and topics; [06](06-ROADMAP-AND-DELIVERY.md) Phase 1 plan; task rows in [14](14-EXECUTION-TRACKER.md) |
| 19 | 2026-11-07 | Fix the open findings and proceed | Close-out session and Phase 1 Wave 1 |
| 20 | 2026-11-07 | Fonts and painting are not the priority. The foundation comes first: stop that work until the foundation is built. Supersedes the walking skeleton after Tier 1D | `SKEL-01` blocked, fonts removed, [06](06-ROADMAP-AND-DELIVERY.md) |
| 21 | 2026-11-07 | Phase 0 is done, sign-off given. Start completing Phase 1 | [06](06-ROADMAP-AND-DELIVERY.md) |
| 22 | 2026-11-07 | Do not leave work at "built". Complete it: a tier counts only when it meets its exit gate | [CLAUDE.md](../CLAUDE.md), [05](05-PHASES-AND-TASKS.md) |
| 23 | 2026-11-07 | LibreOffice keeps crashing on the owner's machine: stop it. Tools must not start it unless asked on purpose | `OPENPRESENT_ALLOW_SOFFICE`, `FID-04` |
| 24 | 2026-11-07 | We must open even malformed files, and edit them, including ones Microsoft discards or fails to open. That is an advantage, not a fault. Supersedes refusing malformed input | [29](29-RECOVERY.md), ADR-015, `REC-*` |
| 25 | 2026-11-07 | Keep the list of known disagreements with Microsoft's validator: keep testing everything the standard allows, write down each disagreement with its reason, and recheck all of them against real PowerPoint before release. Editing `tools/ooxml-sdk/false-positives.tsv` is approved for this | `F-010`, `PV-01` |
| 26 | 2026-11-07 | Write down your mistakes and their solutions, so they do not happen again | [04](04-MISTAKES-AND-RULES.md), [CLAUDE.md](../CLAUDE.md) |
| 27 | 2026-11-07 | Complete Phase 1 soon | [06](06-ROADMAP-AND-DELIVERY.md), [14](14-EXECUTION-TRACKER.md) |
| 28 | 2026-11-07 | Write the rule for not overloading the machine and keep it in memory, so the owner need not repeat it | [04](04-MISTAKES-AND-RULES.md) rule 22, [CLAUDE.md](../CLAUDE.md) |
| 29 | 2026-11-07 | Waste no time and no tokens | [04](04-MISTAKES-AND-RULES.md) rule 23 |
| 30 | 2026-11-07 | Sessions do not run all the gates: they finish their work. Stop wasting system resources, money and time | [CLAUDE.md](../CLAUDE.md), `openslide-worker` skill |
| 31 | 2026-11-07 | Stop writing gates for every little thing. No new gate without the owner asking for it | [CLAUDE.md](../CLAUDE.md) |
| 32 | 2026-11-07 | Run one broad gate, not a gate for every small thing and test. CI on `main` is that gate; a merge only has to compile | [CLAUDE.md](../CLAUDE.md) |
| 33 | 2026-11-08 | Corrects how instruction 23 was read: do not crash LibreOffice, but keep using it to validate. The owner never said to stop | `FID-04`, F7 |
| 34 | 2026-11-08 | A flipped byte in a part that still parses: keep the bytes and tell the user, rather than replace the part with a stand-in, even though that file stays invalid until the part is repaired | [29](29-RECOVERY.md), [06](06-ROADMAP-AND-DELIVERY.md) |
| 35 | 2026-11-08 | Withdrawn by the owner: said about another repo. Applies to nothing here, and `DEC-03` stands | `DEC-03` |
| 36 | 2026-11-08 | Drop Keynote decks from the Phase 1 corpus requirement: Apple-native output is not a priority | [06](06-ROADMAP-AND-DELIVERY.md), `CORP-01` |
| 37 | 2026-11-08 | Corrected by the owner the same day: no local CI scans; CI runs on GitHub. Workflows stay enabled, and sessions run only the named tests they need | [CLAUDE.md](../CLAUDE.md), [15](15-CI-AND-RELEASE-GATES.md) |
| 38 | 2026-11-08 | No tests, scans or CI on the owner's machine at all. Everything runs on GitHub, including the Phase 1 exit runs | [CLAUDE.md](../CLAUDE.md), `.github/workflows` |
| 39 | 2026-11-08 | Every test runs as GitHub CI on merge to `main` and on pull requests, as OpenCalc does, including the Phase 1 exit runs | `.github/workflows`, [15](15-CI-AND-RELEASE-GATES.md) |
| 40 | 2026-11-08 | When LibreOffice fails on a deck whose unedited input fails the same way, that is LibreOffice's problem, not ours, and does not hold the exports item back | `tools/phase1-exit.py`, `FID-04` |
| 41 | 2026-11-08 | Phase 2 needs proper planning before it starts: scope, targets, decisions and an ETA, done deliberately and not rushed | [06](06-ROADMAP-AND-DELIVERY.md) |
| 42 | 2026-11-08 | Don't reintroduce problems OpenCalc already solved. Each Phase 2 task starts from OpenCalc's working solution, copied with provenance, and changes only what slides need | [06](06-ROADMAP-AND-DELIVERY.md), [CLAUDE.md](../CLAUDE.md) |
| 43 | 2026-11-08 | A visible outcome within 3 hours: start Phase 2 now with the plan's recommended decisions, aiming at real slides rendered to images the owner can see. This waives the rule that rendering waits for Phase 1's sign-off | [06](06-ROADMAP-AND-DELIVERY.md), `DEC-16` to `DEC-20` |
| 44 | 2026-11-08 | Plan vertically: ship thin end-to-end slices a user can see and use, then deepen them, instead of perfecting each layer before the next | [06](06-ROADMAP-AND-DELIVERY.md), [05](05-PHASES-AND-TASKS.md) |
| 45 | 2026-11-08 | The webapp is built like OpenCalc's: same structure, modules, command registry and look, as a PowerPoint clone the way OpenCalc clones Excel | `webapp/`, [CLAUDE.md](../CLAUDE.md) |
| 46 | 2026-11-08 | Detail of 45: build the web app the way OpenCalc's is: its structure, file layout (`editor.*.js`, `commands.js` registry, CSS, dialogs, files, deploy and download pages), conventions and look, copied with provenance, changing only what a presentation needs: PowerPoint's ribbon tabs (Home, Insert, Design, Transitions, Animations, Slide Show, Review, View), the thumbnail pane, slide canvas, notes pane and status bar. Wire only what works; show other commands present but disabled through the registry, never faked. Read OpenCalc's UI docs first. This reverses OpenCalc's rule (its docs/91 §8) of leaving unbuilt commands out | `webapp/`, `VS-01` |
| 47 | 2026-11-08 | A marketing site and the editor, deployed on GitHub Pages exactly as OpenCalc does it: its landing, docs, download and deploy pages copied with provenance and adapted, in the cyan accent; every claim true today and none of a PowerPoint match; real renders, never mock-ups; one workflow deploys, with the gallery folded in. Pages is switched on with source "GitHub Actions" at https://casualoffice.github.io/openpresent/ | `SITE-01`, `webapp/`, `.github/workflows/pages.yml` |
| 48 | 2026-11-08 | Fixing CI must be quick: run only the failing tests locally, one by one, to reproduce and confirm a fix; never the full suite on this machine. Time boxes are short: 30 minutes for CI fixes | [CLAUDE.md](../CLAUDE.md) |

## Boundaries set by the environment

| Date | What | Status |
| --- | --- | --- |
| 2026-11-07 | Creating a GitHub remote and pushing was denied by the session's permission system | Resolved: the owner created the repo and pushed. Not worked around |
