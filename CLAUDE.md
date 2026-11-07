# CLAUDE.md

The working rules for this repository. [AGENTS.md](AGENTS.md) holds the full contract and applies to every agent.
Sister repo `../opencalc` is the architecture reference: copy from it with provenance, and never modify it.

## Owner instructions come first

**[docs/03](docs/03-OWNER-INSTRUCTIONS.md) records everything the owner has instructed. Follow it without deviating.** Record each new instruction there in the same turn. If something seems wrong or conflicts, raise it; do not quietly choose. Within those instructions, make decisions yourself, thinking of the goal and the future, rather than asking.

## Do not repeat these mistakes

**Read [docs/04](docs/04-MISTAKES-AND-RULES.md) before starting work; its rules bind every session.** The ones that cost most:
- Until Phase 1 closes, work only on model, import and export. No rendering, painting or fonts.
- Biggest remaining work starts first. Independent stages all start at once.
- Opening and editing the user's file comes first: neutralize attacks, refuse only what cannot be read.
- Every status update gives the phase percentage, from the checklist, merged only. Short, plain language.
- Say "stopped", "done" or "tested" only after checking by running. Read gate output before committing.
- Nothing that can pop up or crash on the owner's desktop.
- Check machine load (`uptime`) before adding a session and at every merge. Above twice the core count, start nothing and land finished work. Sessions run only `cargo test -p` on what they changed, with `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2`, and never the full gates: the main session runs those once at merge.
- No new gates, audit rules or repository checks unless the owner asks (instruction 31).
- Decide, record it, carry on. Commit often, merge as soon as gates pass, keep the disk clear.

## What this is

An **enterprise-grade** PowerPoint-compatible engine and editor, on **web and desktop**. Not an MVP or a demo.
**Fidelity is the most critical property.** Import, export and modelling come first, then layout and rendering, then UI.

## One roadmap, one tracker

- **[docs/06](docs/06-ROADMAP-AND-DELIVERY.md) is the only roadmap. [docs/14](docs/14-EXECUTION-TRACKER.md) is the only tracker.** No parallel plans, TODO files, or lists in conversation. `tools/check-repo.py` enforces it.
- **No work without a tracker row.** Read the id space first; ids are never reused.
- **[docs/05](docs/05-PHASES-AND-TASKS.md) defines phases, tiers, tasks, and how each opens and closes.** Only the owner closes a phase. Starting early needs a recorded waiver, and exit criteria are never bent to fit what was built.
- **Findings become rows immediately,** including those out of scope.
- **Built is not done (owner instruction 22).** A tier counts only when every item of its exit gate in docs/06 is met. Do not move on and leave a tier at "built": finish its leftovers, its fixtures and its status marks.

## Update every document

**A change is not done until every document it affects is true, in the same commit.** Check:

| If you changed | Update |
| --- | --- |
| Any work's status | [docs/14](docs/14-EXECUTION-TRACKER.md); the phase checkbox in docs/06 if it meets a criterion |
| Spec coverage | Status columns in `docs/spec/checklist.tsv` |
| A threat's control or test | [docs/07](docs/07-SECURITY-THREAT-MODEL.md) |
| A limit or error code | [docs/20](docs/20-ERROR-CODE-REGISTRY.md), docs/07 |
| A crate, or its dependencies | [docs/19](docs/19-WORKSPACE-SCAFFOLD-DESIGN.md) |
| A design decision | [docs/08](docs/08-ADR-REGISTER.md) and the owning design note |
| The model, ops or rendering | [docs/22](docs/22-NORMALIZED-SCHEMA.md), [24](docs/24-OPERATIONS-AND-COLLABORATION.md), [42](docs/42-TEXT-AND-RENDERING-ARCHITECTURE.md) |
| A new document | [docs/00](docs/00-README.md) |
| Phase or project state | "Current state" in AGENTS.md, status in README.md |

A document stating something the code does not do is a defect: fix the code or add a row. Never quietly lower a promise. Say "measured" only when it was, and never call a LibreOffice result a PowerPoint match.

## The cycle

Every task: **design → execute → check → find issues → fix → commit, with tests.**

- **Check means running it,** not re-reading it.
- **Find issues** asks what would have to be true for this to be wrong, then goes and looks.
- **A test counts only if it was seen to fail without the change.** Definition of Done is in docs/05.
- Substantial designs are discussed first. An ADR trigger needs an Accepted ADR.

## Engineering standards

Industry-standard practice applies everywhere, with these specifics.

**Rust**
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/). `cargo fmt` and `cargo clippy -- -D warnings` are clean. No `#[allow]` without a comment saying why.
- `unsafe` is forbidden. Toolchain and MSRV are pinned. Must compile to `wasm32-unknown-unknown`, with no `#[cfg]` forks in the engine.
- **Untrusted input never panics.** No `unwrap`, `expect`, indexing or unchecked arithmetic on values from a file. Return a typed error with a stable code from docs/20. Tests may unwrap.
- Errors are typed enums, never strings. Messages carry part names and counts, never document content.
- Public items have doc comments saying what and why. Keep public surfaces narrow; `pub(crate)` by default.
- Pure functions where possible. Time and randomness are supplied, never sampled. Same input ⇒ same bytes: use ordered collections where order can reach output.
- Layers depend downward only, per docs/19. No new crate without a first consumer.
- **Clean room for copyleft competitors.** ONLYOFFICE and PPTist are AGPL, and LibreOffice is MPL. Take ideas from their documentation and behaviour; never copy or adapt their code. OpenCalc, ours, is the only code to copy from.
- New dependencies need a reason, a licence that passes `deny.toml`, and an ADR for the engine.

**Parsing and security** ([docs/07](docs/07-SECURITY-THREAT-MODEL.md))
- **Recover before refusing (owner instruction 24, [docs/29](docs/29-RECOVERY.md)).** Malformed input gets a best reading and the deck opens; hostile constructs are neutralized, never obeyed. Opening what Microsoft refuses is an advantage.
- Every file is hostile. Every parser is bounded, has hostile fixtures, and has a fuzz target. All XML goes through `casual-slide-ooxml`'s walker.
- No macro execution, no automatic network fetch, no document string reaching the DOM as HTML. No secrets in code or logs.

**Fidelity**
- Absent stays absent; never materialize a default. Edits patch named nodes (ADR-013); never re-serialize a part from the model.
- Nothing is lost silently. Anything unmodelled is retained and reported. The word "lossless" is banned unless the dimension is named.

**Tests**
- Unit tests beside the code; integration in `tests/`; corpus-driven where a fixture can express it. Names describe behaviour. Tests are deterministic, with no network and no sleeps.
- Every bug fix starts with a failing test.

**JavaScript** (`webapp/`): ES modules, no framework or bundler, as OpenCalc. `textContent` for document strings. Strict CSP. Every verb is a registered command.

**Use LibreOffice to validate, and never let it crash on the owner's desktop** (owner instructions 23 and 33, `FID-04`). Run it in a Linux container, or on CI, where a crash shows no dialog. Tools refuse without `OPENPRESENT_ALLOW_SOFFICE=1`, so a desktop run can't start by accident.

**Python** (`tools/`): standard library where possible; tools fail non-zero and name the problem. Parse untrusted XML only after refusing a DOCTYPE.

**Git**
- [Conventional Commits](https://www.conventionalcommits.org/): `feat(scope):`, `fix(scope):`, `docs:`, `chore:`, citing tracker ids. One coherent change per commit, with the reason in the message.
- **Work lands directly on `main`, with no pull requests for now** (owner instruction 17). Parallel workers still use their own worktree branches; the main session verifies and merges them to `main`, then pushes. Never force-push `main`. Never skip hooks. Enable them with `git config core.hooksPath .githooks`.
- Never commit build output, confidential decks, or unreviewed third-party files.

**Nothing runs on the owner's machine: no tests, scans or CI (owner instructions 37, 38).** Everything runs on GitHub, including the Phase 1 exit runs. Sessions may run `cargo check` to confirm code compiles, and nothing more.

**One broad gate (owner instruction 32).** CI on `main` runs the list below. A merge needs only to compile, then push, and a red CI is fixed first. Sessions do not run this list.

**The list CI runs**

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo check --target wasm32-unknown-unknown --workspace --locked
python3 tools/check-repo.py
python3 tools/spec-checklist/generate.py --check
python3 fixtures/tools/generate.py --check
python3 tools/spec-audit/audit.py --check
python3 tools/ooxml-sdk/validate.py fixtures/generated fixtures/producers fixtures/real fixtures/rr/good fixtures/rr/sa   # when dotnet is present; CI always runs it
```

## Working in parallel

Run independent work in parallel worktree-isolated sessions rather than serially. A worker gets one job, the files it owns, and its acceptance test. **A worker's report is not evidence:** the main session verifies by running, reverting the fix to see the test go red, then merging. Two workers never touch one invariant. See the `openslide-worker` and `openslide-merge` skills.

## Skills

| Skill | When |
| --- | --- |
| `openslide-task` | Starting any tracked task |
| `openslide-tier` | Opening or closing a Phase 1 or 2 tier |
| `openslide-parser` | Adding or changing anything that reads a file |
| `openslide-worker` | Briefing a parallel session |
| `openslide-merge` | Verifying and merging a worker's branch |
| `openslide-docs` | Before committing, to bring every document into line; or to audit them |

Start reading at [docs/00](docs/00-README.md).
