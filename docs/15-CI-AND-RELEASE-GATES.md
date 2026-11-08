# 15 — CI and release gates

CI is part of the architecture. The jobs below are a **contract**: stable names, and a
change is not mergeable until they pass. Adapted from OpenCalc's docs/15 at `b23c67c`,
keeping its number. Owner: `GOV-01`.

**Current state:** Phase 0. Seventeen jobs run in six workflows, listed below. Nothing is
released, and no release workflow exists yet.

## Test levels

Each level answers a question the one below it cannot.

| Level | Where | What it proves | Runs |
| --- | --- | --- | --- |
| Unit | Beside the code, `#[cfg(test)]` | One function's behaviour, including every refusal path | `test`, every push |
| Integration | `crates/*/tests/` | A crate's public surface, end to end | `test` |
| Corpus | `fixtures/manifest.tsv`, read by `corpus.rs`, `decks.rs`, `roundtrip.rs` | Every fixture meets its recorded expectation: opens with the findings named, or refuses with the code named. F1, F2 and the forced rewrite run over every deck that opens | `test` |
| Hostile | `fixtures/hostile/` | Every `SEC-*` threat and every limit in [21](21-PARSER-LIMITS.md) has an input that crosses it | `test` |
| Fuzz | `fuzz/`, one target per parser | Nothing panics, hangs or over-allocates on input nobody thought of | `fuzz-build` per push; `fuzz-campaign` weekly |
| Schema validity (F4) | `tools/f4/validate.py`, xmllint and Ecma's XSDs, with what PowerPoint writes allowed in Transitional and recorded in `tools/f4/allowances.tsv` (`F4-02`, `F4-03`) | Every XML part we read or write is valid against the standard, or against those recorded allowances | `test` on Linux, where it is required |
| Office validity (`F-006`) | `tools/ooxml-sdk/`, Microsoft's Open XML SDK | The same parts against Office's reading of the schemas. Disagreements with F4 are recorded in `false-positives.tsv`, never silenced | `ooxml-sdk` |
| Real decks (`F-007`) | `fixtures/real/`, `tools/real-decks/` | Decks written by PowerPoint, not by us, import and survive a forced rewrite | `test` for committed decks; `tools/real-decks/check.py` locally for the rest |
| Second opinion (F7) | `tools/fidelity/`; `tools/phase1-evidence.py export` | LibreOffice's rendering, where it differs; a question, never a verdict. The export run reopens every edited corpus deck in it | `p1-export`; the rest by hand until a job exists |
| Repository | `tools/check-*.py` | The documents, tracker, workflows and dependency graph keep their own rules | `checks`, `gates`, pre-commit hook |

The fidelity ladder F1 to F8 is defined in [51](51-SPEC-CHECKLIST-AND-FIDELITY.md). F5, F6 and F8
need PowerPoint itself and wait on `PV-01`; until then no gate may claim a PowerPoint match.

## Jobs

The list is a contract **in both directions**: every job here exists, and every job a workflow
runs is here. `tools/check-workflows.py` checks both, since OpenCalc's first three new jobs
reached its workflow and not its table.

| Job | Workflow | Command (intent) | Enforces |
| --- | --- | --- | --- |
| `checks` | `ci.yml` | fmt, clippy `-D warnings`, rustdoc `-D warnings`, wasm32 check, `check-repo.py`, `check-gate-selftest.py`, checklist and fixture staleness, spec audit, fidelity tool tests | Formatting, lints, docs, WebAssembly, every repository gate |
| `commit-messages` | `ci.yml` | `tools/check-commits.py --range` over the pushed or proposed commits | Conventional Commits |
| `ooxml-sdk` | `ci.yml` | `tools/ooxml-sdk/validate.py --self-test`, then over `generated/`, `producers/` and `real/` with `OOXML_SDK_REQUIRED=1` | `F-006`: a seeded invalid part is rejected; no unrecorded SDK finding |
| `test` | `ci.yml` | `cargo test --workspace --all-features --locked` on Linux, macOS and Windows; F4 required on Linux | Correctness, determinism, the corpus, hostile fixtures, F1 to F4 |
| `msrv` | `ci.yml` | `cargo +1.88.0 check --workspace --locked` | The minimum Rust version |
| `dependency-policy` | `ci.yml` | `cargo deny check bans licenses sources`, `cargo audit --deny warnings` over the root and `fuzz/` lockfiles | Supply-chain policy |
| `fuzz-build` | `ci.yml` | `cargo +nightly fuzz build` | Fuzz targets compile |
| `gates` | `post-merge.yml` | `check-repo.py`, `check-gate-selftest.py`, fixture manifest, on every push to main | The merge commit itself, which no branch run saw |
| `fuzz-campaign` | `fuzz.yml` | Every target from `cargo fuzz list`, seven minutes each, weekly and on demand; crashing inputs kept as an artifact | Parsers against new inputs |
| `p1-budget-hostile` | `phase1-exit.yml` | `tools/phase1-evidence.py budget`, then `hostile`; this workflow runs on every push to main, pull request and merge group, and on demand | `P1X-01`: the 500-slide budget; every hostile file opened and edited, or refused, with no crash or hang |
| `p1-damage` | `phase1-exit.yml` | `tools/phase1-evidence.py damage`: the damage corpus with `DAMAGE_FULL=1 F4_REQUIRED=1` | `P1X-01`: every damaged deck opens or is refused by code, and what it writes passes F4 |
| `p1-export` | `phase1-exit.yml` | `tools/phase1-evidence.py export` with `PHASE1_LO=host`: every corpus deck written edited, then F4, the SDK, and LibreOffice from apt, headless under `unshare --net` | `P1X-01`: exported decks valid and reopened by LibreOffice; the only place it runs (instructions 23, 33, 38) |
| `p1-gates` | `phase1-exit.yml` | The gates in CLAUDE.md, exactly as listed, with F4 and the SDK required | `P1X-01`: the closing list's gates item |
| `site-build` | `pages.yml` | Builds `casual-slide-wasm` with `wasm-pack` (0.14.0) and `render-deck`; copies `webapp/` to the site's root with the engine in `pkg/`, as OpenCalc's `pages.yml` does, so the marketing pages are at `/` and the editor at `editor.html`; draws slide 1 of four `fixtures/real` decks with `render-deck` into `shots/` for the front page; then checks every page and module the site needs, and each picture, is there and not empty. Kept as the `site` artifact. Runs on push to main, pull requests and on demand | `SITE-01`: the site and editor deploy complete, with real renders and no mock-ups; `VS-01`: the editor on the web |
| `gallery-build` | `pages.yml` | Builds `render-deck` as an example of `casual-slide-render`, renders `fixtures/real` and a few generated decks with it and with LibreOffice from apt under `unshare -n`, then `tools/gallery/build.py` writes `index.html`; kept as the `gallery` artifact. Runs on push to main, pull requests and on demand | `FH-02`: our slides beside LibreOffice's, labelled "LibreOffice", never "PowerPoint"; an empty gallery fails |
| `pages-deploy` | `pages.yml` | The only job in any workflow that deploys to GitHub Pages: from main only, with `pages: write` on this job alone, when `site-build` passed. Puts the `site` artifact at the root and the `gallery` artifact at `gallery/`; when the gallery failed, a page there says so and links the run. Then `actions/deploy-pages`. Pages' source is "GitHub Actions" | `SITE-01`, `FH-02`: the site at https://casualoffice.github.io/openpresent/, the editor at `editor.html`, the gallery at `gallery/` |
| `desktop-bundle` | `desktop.yml` | On Linux, macOS and Windows, on push to main, pull requests and on demand: builds `casual-slide-wasm` with `wasm-pack` (0.14.0) into `webapp/pkg`, then `cargo tauri build` of `desktop/` as `.deb`, `.app` and NSIS, and checks the bundle is on disk and not empty. On Linux also fmt, clippy `-D warnings` and tests of `desktop/`, the built binary run under xvfb with a fixture deck on its command line (`--self-check`, which passes only when the slide is drawn), and the `.deb`'s desktop entry checked to take a file. Copied from OpenCalc's `desktop` job at `5d42a07` | `VS-03`: the desktop app bundles on three OSs and draws as the web does |
| `p1-evidence` | `phase1-exit.yml` | `phase1-evidence.py merge` of each job's record into `tools/phase1-evidence.json`, kept as the `phase1-evidence` artifact, then `tools/phase1-exit.py --quick --gates-result <p1-gates result> --sign-off-not-failing`: the gates item is met only when `p1-gates` succeeded, and owner sign-off, the owner's box, prints NOT MET without failing the job (`CI-22`); any other NOT MET fails it | The closing list, read from evidence made on GitHub |

**Every job is read-only** (`permissions: contents: read`), **every action is pinned to a full
commit SHA** with its tag in a comment, and a superseded pull-request run is cancelled while a
run on main never is. `check-workflows.py` refuses an unpinned action and a workflow without
`permissions:`.

### Each job seen failing

A job that cannot fail proves nothing, so each was given a seeded violation and seen red, with the
clean tree green around it. **Measured 2026-11-07 by running each job's own command locally at
`a96e88b` plus these gates, on macOS arm64. Not yet seen failing on GitHub's runners**, where the
same commands run; `F-003` stays open until CI has run them. Every seed was reverted.

| Job | Step | Seed | Result |
| --- | --- | --- | --- |
| `checks` | `cargo fmt --check` | A badly spaced `pub fn` in `-model` | Exit 1 |
| `checks` | clippy `-D warnings` | An unused variable | `error: unused variable: x` |
| `checks` | rustdoc `-D warnings` | A doc link to an item that does not exist | `error: unresolved link to NoSuchItem` |
| `checks` | wasm32 check | `compile_error!` under `cfg(target_arch = "wasm32")` | Fails on wasm32; the native check of the same tree passes |
| `checks` | `check-repo.py`, `check-gate-selftest.py` | One violation per gate, 22 in all; separately, two gates mutated | Each seed fails its gate; each mutated gate turns the self-test red |
| `test` | `cargo test` | `max_xml_depth` raised from 256 to 1,000,000 | `every_fixture_meets_its_expectation` overflows its stack on `hostile/deep-nesting.bin`, and the run aborts. So this limit is what stands between a hostile deck and a crash |
| `msrv` | `cargo +1.88.0 check` | A call to `File::lock`, stable from 1.89 | `E0658` on 1.88.0; passes on the 1.96.0 dev toolchain |
| `dependency-policy` | `cargo deny` | `-model` licensed `GPL-3.0-only` | `licenses FAILED`; `bans` and `sources` still ok |
| `dependency-policy` | `cargo audit` | `smallvec 0.6.9` added to a copy of the lockfile | Exit 1 naming RUSTSEC-2019-0009, among others; the real lockfile exits 0. Run with `--no-fetch --stale`, since the advisory fetch failed on the network that day, so the database was the local copy |
| `fuzz-build` | `cargo +nightly fuzz build` | A type error in the `part_name` target. One target built, not all | `E0308`; the restored target builds |
| `commit-messages` | `check-commits.py` | The message `Fix stuff` | Exit 1, naming it; the 21 commits then in history pass |
| `ooxml-sdk` | `validate.py --self-test` | An invalid element and attribute in a slide; separately, redaction disabled in the tool | Rejected on that part alone; the mutation is reported as a value reaching the report |

Not seen failing: `gates` and `fuzz-campaign`, which run commands covered above. `fuzz-campaign` only
goes red when it finds a crash, so it has no seed that is not a planted bug in a parser.

## Repository gates

`python3 tools/check-repo.py` runs them all, in the pre-commit hook and in `checks`. Each fails
non-zero and names every problem.

| Gate | Holds |
| --- | --- |
| `check-repo.py` itself | Tracker ids unique; one tracker and one roadmap; every tracked doc indexed in [00](00-README.md); relative links resolve; no raw XML reader outside the walker |
| `check-tracker-shape.py` | Every row of [14](14-EXECUTION-TRACKER.md) has its table's column count, and a status from `Open`, `In progress`, `Blocked`, `Done` |
| `check-doc-references.py` | Every `docs/NN`, `docs/NN-NAME.md`, `ADR-NNN` and tracker id cited in documents, crate sources and tools exists. Dead references found when the gate arrived are listed in `tools/doc-references.known`, each owned by a row; the list can only shrink |
| `check-crate-deps.py` | No crate under `crates/` depends on a server, or on any path outside `crates/` (`CI-01`) |
| `check-fuzz-lock.py` | Every separate workspace's lockfile (`fuzz/`, `spikes/`, `tools/real-decks/`) agrees with its manifests |
| `check-error-codes.py` | Every stable code in the code is in [20](20-ERROR-CODE-REGISTRY.md), and every code there is in the code (`DOC-010`) |
| `check-parser-limits.py` | Every limit field and `MAX_*` constant in the code is in [21](21-PARSER-LIMITS.md) with the default the code holds |
| `check-chart-codegen.py` | The generated chart modules are what `tools/chart-codegen/generate.py` writes from the schemas (`T1J-01`) |
| `check-workflows.py` | The job contract above, SHA pins, permissions, no retries, and the release rules below |
| `check-commits.py` | Conventional Commits; run by the `commit-msg` hook and `commit-messages`, not by `check-repo.py`, since it judges history rather than the tree |

**The gates are tested.** `tools/check-gate-selftest.py` seeds one violation of each gate into a
scratch copy and requires that gate to fail on it and pass without it, and refuses a gate that
defines its own tracker-row pattern instead of importing `tools/tracker_rows.py`. A gate that
cannot fail is not a gate; OpenCalc lost 47 rows to two that could not see them.

Enable the hooks once per clone with `git config core.hooksPath .githooks`. Never skip them.

## Flake policy: no retries

**No job, step or test is retried.** A retry turns an intermittent failure, the kind most worth
knowing about, into a pass. `check-workflows.py` refuses retry actions, retry options and
`continue-on-error: true`.

A test that fails intermittently is a defect:

1. It gets a tracker row the day it is seen, P1 or above, naming the job and the run.
2. It is fixed, or deleted with the row saying what coverage was lost. It is never retried,
   ignored with `#[ignore]` without a row, or made non-blocking.
3. Tests are deterministic by construction: no network, no sleeps, no wall clock, no
   unseeded randomness, ordered collections wherever order reaches output.

A registry or runner outage is not a flake in our code, but it is not retried either. The run
is re-run by hand, and a second outage gets a row.

## What each phase adds

A phase does not close until its gates exist and pass ([05](05-PHASES-AND-TASKS.md)). When a
needed gate does not exist, it is named here with a row, never silently skipped.

| Phase | Gates added |
| --- | --- |
| 0 | Everything above. Still open: the coverage gate from the spec audit (`SPEC-01`), cross-platform determinism in CI (`GEO-05`, `DET-01`) |
| 1 | Per tier: a fixture per checklist area, a hostile fixture per limit, a fuzz target per new parser, F4 and the SDK validator clean on every export; real decks as input (`F-007`), with corpus targets (`CORP-01`); one repair-risk fixture per writer invariant (`WR-02`) |
| 2 | Golden display lists; native against WebAssembly display-list hash (`DET-01`); F7 comparison as a job; F6 thresholds per fixture (`THR-01`) once `PV-01` gives references |
| 3 | `browser-smoke`: the viewer loads, paints and navigates in a real browser, building the engine itself; bundle-size budget; benchmark smoke with baselines (`CAP-01`); browser matrix (`BRW-01`); pseudo-locale (`I18N-01`) |
| 4 to 7 | Every operation's inverse and TP1 properties per editing phase (`OP-02`); crash-safe save (`SAVE-01`); UX sweep and reachability (`UXT-01`); accessibility checker rules (`A11Y-01`) |
| 8 | Timing-tree goldens sampled at fixed times |
| 9 | Collaboration server tests, wire compatibility across versions (`VER-01`), load test |
| 10 | Desktop builds per platform, signing, updater; release workflows below |

## Releases

**Releases fire on a tag, and only on a tag.** Nothing is published from a branch, a schedule or
a merge. As in OpenCalc, tags are **component-scoped**, never a bare `v1.0.0`, because two
release workflows watching one tag namespace both fire on either tag:

| Tag | Releases | Workflow |
| --- | --- | --- |
| `engine-v*` | `casual-slide-*` crates | reserved |
| `sdk-v*` | The browser SDK | reserved |
| `desktop-v*` | The desktop app | reserved |
| `server-v*` | The collaboration server | reserved |

**A manual run is a dry run by default.** A release workflow that accepts `workflow_dispatch`
has a `dry_run` input defaulting to `true`, which builds, packages and verifies everything and
publishes nothing. Publishing needs a tag, or someone setting `dry_run` to false on purpose.
`check-workflows.py` holds any `release*.yml` to both rules.

A release also needs: every job above green on the tagged commit; the version in the tag equal
to the version in the packages, refused otherwise; credentials only on a protected `release`
environment; an updated [CHANGELOG](../CHANGELOG.md); reproducible artifacts. **The first
public release is blocked by `DEC-07` (legal review) and `PV-01` (PowerPoint verification).**
No single contributor may weaken a release gate to publish ([GOVERNANCE](../GOVERNANCE.md)).

## Not built yet

- Coverage gate: a checklist row with no fixture cannot be `y` (`SPEC-01`).
- Cross-platform determinism job (`DET-01`, `GEO-05`).
- `browser-smoke` and bundle budget, Phase 3.
- A scheduled `cargo audit`. `dependency-policy` audits the root and fuzz lockfiles on every
  push, but an advisory published between pushes waits for the next one.
- Release workflows, reserved above.
