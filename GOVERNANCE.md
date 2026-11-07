# Governance

Adapted from OpenCalc's `GOVERNANCE.md` at `b23c67c`. The project is maintained under
the CasualOffice GitHub organization, alongside OpenCalc and OpenDoc.

## Authority

1. **The owner's instructions**, recorded in [docs/03](docs/03-OWNER-INSTRUCTIONS.md),
   come first. Where a document, a design or anyone's judgement disagrees, that page
   wins, and the disagreement is raised with the owner rather than quietly settled.
2. **Accepted ADRs** in [docs/08](docs/08-ADR-REGISTER.md).
3. **The design documents** indexed in [docs/00](docs/00-README.md).

Only the owner closes a phase ([docs/05](docs/05-PHASES-AND-TASKS.md)).

## Roles

**Owner**: sets direction, decides what is delegated, closes phases, and holds the
decisions that need a lawyer (`DEC-07`).

**Maintainers** own repository administration, releases, security response and final
compatibility decisions. Until named maintainers are recorded here, before the first
public preview, repository write access is the authoritative signal.

**Subsystem owners** review changes in their area (the package reader, the part graph,
the model, import and export, geometry, text, rendering, the shell) and keep its
design, tests, fixtures and tracker rows true.

**Contributors**, human or agent, propose designs and changes through
[CONTRIBUTING.md](CONTRIBUTING.md). An agent works under [AGENTS.md](AGENTS.md), and
**a worker's report is not evidence**: the main session verifies by running it.

## Decisions

Substantial decisions are designed first:

1. state the required outcome and its constraints;
2. record the research and the alternatives;
3. write a design note, or an ADR if a trigger in docs/08 applies;
4. discuss and resolve objections;
5. mark the decision Accepted;
6. track it in [docs/14](docs/14-EXECUTION-TRACKER.md), the only tracker;
7. implement and verify.

Maintainers seek technical consensus. Where there is none, the maintainer responsible
for the affected boundary records the decision and its consequences in an ADR.

## Protected areas

These need maintainer review, because a change to them is expensive to reverse:

- public APIs and the SDK surface;
- the presentation model and the operation schema ([docs/22](docs/22-NORMALIZED-SCHEMA.md),
  [docs/24](docs/24-OPERATIONS-AND-COLLABORATION.md));
- **crate boundaries** and the dependency DAG ([docs/19](docs/19-WORKSPACE-SCAFFOLD-DESIGN.md));
- parser limits and security policy ([docs/21](docs/21-PARSER-LIMITS.md),
  [docs/07](docs/07-SECURITY-THREAT-MODEL.md)), and anything that would need `unsafe`,
  which is forbidden;
- the byte-faithful part tree and save path (ADR-013);
- the display-list contract and render backends;
- performance budgets ([docs/42](docs/42-TEXT-AND-RENDERING-ARCHITECTURE.md));
- vendored schemas, fonts and fixtures with third-party rights ([NOTICE](NOTICE));
- CI gates and release automation ([docs/15](docs/15-CI-AND-RELEASE-GATES.md)).

## Releases

A release needs every gate in [docs/15](docs/15-CI-AND-RELEASE-GATES.md), an updated
[CHANGELOG.md](CHANGELOG.md) and tracker, compatibility notes and reproducible
artifacts. **The first public release is also blocked by `DEC-07` and `PV-01`.** No
single contributor may weaken a release gate in order to publish.
