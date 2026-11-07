# Contributing

Adapted from OpenCalc's `CONTRIBUTING.md` at `b23c67c`. Read [AGENTS.md](AGENTS.md)
and [CLAUDE.md](CLAUDE.md) first; they are the full contract, and this file is its
checklist. Owner instructions in [docs/03](docs/03-OWNER-INSTRUCTIONS.md) come before
both.

## Before you implement

- [ ] The work has a row in [docs/14](docs/14-EXECUTION-TRACKER.md), with a stable id
      never used before. **No untracked work merges.**
- [ ] The outcome is written down in a numbered design note, indexed in
      [docs/00](docs/00-README.md).
- [ ] If the change trips an ADR trigger in [docs/08](docs/08-ADR-REGISTER.md), the ADR
      is **Accepted**.
- [ ] The acceptance test is named, and it can fail.
- [ ] Code that reads a file follows the parser rules in
      [docs/07](docs/07-SECURITY-THREAT-MODEL.md): bounded, a hostile fixture per
      limit, a fuzz target, a stable code from [docs/20](docs/20-ERROR-CODE-REGISTRY.md),
      and its limit listed in [docs/21](docs/21-PARSER-LIMITS.md).

## While you implement

The cycle is **design, execute, check, find issues, fix, commit, with tests**.

- One coherent change per commit.
- Mutation only through operations; every operation returns its inverse.
- A test counts only if it was seen to fail without the change. A bug fix starts with
  the failing test.
- Absent stays absent. Nothing unmodelled is lost silently.
- Copy code only from OpenCalc, with provenance. Never from ONLYOFFICE, PPTist or
  LibreOffice (clean room).

## Before you commit

Enable the hooks once per clone: `git config core.hooksPath .githooks`. **Never skip
them**, and never route around a failing gate: report it.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo check --target wasm32-unknown-unknown --workspace --locked
python3 tools/check-repo.py
python3 tools/spec-checklist/generate.py --check
python3 fixtures/tools/generate.py --check
```

`tools/check-repo.py` runs every repository gate, each listed in
[docs/15](docs/15-CI-AND-RELEASE-GATES.md). Then make every document the change affects
true, in the same commit (the table in CLAUDE.md lists them).

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/), checked by
`tools/check-commits.py` in the `commit-msg` hook and in CI:

```
feat(ooxml): refuse duplicate attributes in the walker (XML-02)
```

Types: `feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `perf`, `ci`, `build`,
`spike`, `revert`. Scope is optional and lower case. The summary is imperative, cites
tracker ids, and is at most 100 characters. The body says why.

## Pull requests

Every PR body says:

- **Problem**: what and why.
- **Design**: the docs note and ADR.
- **Verification**: the gates above, and the test seen red.
- **Impact**: API, fidelity, security and limits, performance, UX, documents.
- **Tracker**: the ids it moves.

## Fixtures and third-party files

Never commit build output, a confidential deck, or a third-party file whose rights
have not been reviewed. A real deck needs its URL, licence, date and SHA-256 in
`fixtures/real/README.md`; unclear rights means it is tested
locally and never committed. Every new third-party file joins the inventory in
[NOTICE](NOTICE).

## Quality bar

Enterprise grade. Correctness and determinism are not negotiable: a change that can
corrupt a deck or lose data silently does not merge, however useful it is otherwise.
