# 05 — What a phase, tier and task are

This is the rule for opening and closing work. [06](06-ROADMAP-AND-DELIVERY.md) applies it to each phase;
[14](14-EXECUTION-TRACKER.md) records the state. A word used here means only what is defined here.

## The three levels

| Level | What it is | Size | Defined by | Example |
| --- | --- | --- | --- | --- |
| **Phase** | A capability the product gains, stated as what a user or host can do afterwards | Months | Its charter in 06 | Phase 2: any deck renders headlessly, measured against a reference |
| **Tier** | A part of a phase with a checklist scope | Weeks | A row set in [spec/](spec/README.md), or a behaviour list | Tier 1C: colour and theme, 473 rows |
| **Task** | One reviewable change with one acceptance test | Days or less | A tracker row | `SEC-013`: a diluted zip bomb is refused |

A task belongs to exactly one tier or phase. Tracker rows that are wider than a task are **tier rows**, and are
split into tasks when the tier opens.

**Slices (owner instruction 44).** After Phase 1, the unit above a task is a **vertical slice**: a thin feature a user can use, cut through model, layout, render, UI and save, with an outcome, an end point and a time box ([06](06-ROADMAP-AND-DELIVERY.md) §After Phase 1). A slice opens and closes as a phase does below, and only the owner closes one. Depth is added in later passes, each a task.

## Phase lifecycle

```
Not opened ──entry met──▶ Open ──exit gate met, owner signs──▶ Closed
                           │  ▲
                           ▼  │ regression found: reopened, with a row
                         Blocked
```

| State | Meaning | Recorded |
| --- | --- | --- |
| **Not opened** | No task may start | — |
| **Open** | Entry criteria met, or a waiver recorded | Date, and what opened it |
| **Blocked** | A named dependency stops the exit gate | The blocking row |
| **Closed** | Every exit criterion has evidence, and the owner signed | Date, evidence links, sign-off |

**Entry criteria** state what must be true to start. They exist so that work does not rest on something unsettled.

**Exit criteria** must each be checkable by someone who did not do the work, by running a command or reading a
named artefact. "Mostly done", "implemented" and "looks right" are not criteria.

**Only the owner closes a phase.** An agent assembles the evidence and proposes closure. An agent never closes one.

**Waivers.** Starting early is sometimes right, and hiding it is not. A waiver names the unmet criterion, the risk, who
accepted it, and what happens to work done under it if the criterion fails. A waiver never removes an exit criterion;
the phase still cannot close until the criterion is met.

**Reopening.** If a closed phase's gate later fails, it reopens. Its row says what regressed.

## Task lifecycle

A task row is the only thing that authorizes work. It has an id, scope, status, severity, mechanism and gate.

| Status | Meaning | Needed to enter |
| --- | --- | --- |
| `Open` | Defined, not started | Id, scope, and an acceptance test stated in advance |
| `In progress` | Being worked | Someone owns it. The files it may touch are known |
| `Blocked` | Cannot proceed | The blocking id or decision is named |
| `Done` | Proven | All of Definition of Done below |

### Definition of Done

1. **The acceptance test was seen to fail without the change,** and pass with it. The output is recorded.
2. **Someone else checks it by running it:** the fix is reverted, the test goes red, and the fix is restored. A worker's report is not evidence.
3. All local gates pass: format, lint, tests, WASM, docs, repo gates. Once CI exists, CI is green.
4. For format work, the spec checklist rows it covers are updated, and each has a fixture.
5. For parsers, limits have hostile fixtures and a fuzz target exists.
6. Docs changed in the same change agree with the code.
7. Anything found and not fixed has become a new row.

Standards and which documents to update are in [CLAUDE.md](../CLAUDE.md).

**Partially done is `In progress`.** A row's mechanism says what remains.

## Tier lifecycle

A tier opens when its dependencies in 06 are closed or waived. **On opening** it gets: a read of the matching
[MS-OI29500](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-oi29500/1fd4a662-8623-49c0-82f0-18fa91b413b8) notes, tasks for
its rows, and fixtures planned. **It closes on the per-tier gate in 06.** A tier close needs no owner signature; a phase close does.

## Evidence

| Kind | Where |
| --- | --- |
| Test proof | The tracker row: test name, and the failure output or how it was reproduced |
| Coverage | `docs/spec/checklist.tsv`, checked by the audit tool |
| Fidelity | The ladder in [51](51-SPEC-CHECKLIST-AND-FIDELITY.md), with its level stated. A LibreOffice result is never recorded as a PowerPoint result |
| Phase close | A section in 06: date, each criterion, its evidence, the owner's sign-off |

## What this forbids

- Starting work with no row.
- Marking `Done` on a report, a green run of a test never seen red, or code that merely exists.
- Closing a phase with an unmet criterion, because waivers cover starting and not finishing.
- Changing an exit criterion to fit what was built. Changing one needs the owner, and the change is recorded with its reason.
