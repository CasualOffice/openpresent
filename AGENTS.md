# AGENTS.md — Agent contract for OpenSlide

Read this in full before doing anything else. Owner instructions are in [docs/03](docs/03-OWNER-INSTRUCTIONS.md) and come first. Working rules, engineering standards and skills are in [CLAUDE.md](CLAUDE.md). The product is being renamed OpenPresent (`DEC-01`, `NAME-01`); crates use the prefix `casual-slide-`, which does not depend on it.

## Repository boundary

- **This repository is the target.** All work happens here.
- **`../opencalc` is the sister repo and is reference-only.** It is the
  architecture this project follows. Read it, copy from it with provenance
  ([19](docs/19-WORKSPACE-SCAFFOLD-DESIGN.md) §Provenance), **do not modify it.**

## Mission

An **enterprise-grade** presentation editor, not an MVP or a demo: a deterministic, embeddable, loss-aware
`.pptx` engine in Rust, running client-side in WebAssembly, under an editor that
works like PowerPoint for the web — same ribbon, flows, shortcuts and dialogs —
so a PowerPoint user needs no teaching.

**Fidelity is the most critical property, and import, export and modelling come first**
([06](docs/06-ROADMAP-AND-DELIVERY.md), [51](docs/51-SPEC-CHECKLIST-AND-FIDELITY.md)). Coverage is
tracked per tag and property in [docs/spec](docs/spec/README.md). Every file is hostile until
proven otherwise ([07](docs/07-SECURITY-THREAT-MODEL.md)).

Only what Microsoft owns is replaced: name, logo, colour scheme, icon artwork,
fonts ([47](docs/47-POWERPOINT-SHELL-AND-FEATURE-MAP.md)).

## The cycle

Every task, none of it optional:

    design → execute → check → find issues → fix → push, with tests

- **Check** means running the thing, not re-reading it.
- **Find issues** asks what would have to be true for this to be wrong, then
  goes and looks.
- **With tests** means a test seen to fail without the fix.
- **Discuss and finalize** substantial designs before implementing. An ADR
  trigger ([08](docs/08-ADR-REGISTER.md)) needs an Accepted ADR first.
- **Track everything** in [14](docs/14-EXECUTION-TRACKER.md). No untracked work.

## Priorities (the earlier wins)

1. **Correctness and safety** — never corrupt a deck.
2. **Determinism** — same input and version ⇒ same model, layout, display list, bytes.
3. **Security and resource bounds** — every parser bounded; no macro execution;
   no automatic network fetches (linked images and media included).
4. **Fidelity** — preserve intent and unknown data; measured, not asserted.
5. **Performance** — designed-in targets ([42](docs/42-TEXT-AND-RENDERING-ARCHITECTURE.md)).
6. **API stability** — public surfaces narrower than internals.
7. **UX** — PowerPoint for the web is the specification.
8. **Maintainability.**

## Design rules

- **Mutation only through operations.** Every edit is an op returning its inverse.
- **The engine owns layout.** Line breaks, caret positions and autofit come from
  the engine, never from the browser, so collaborators, export and PNG agree.
- **Inheritance is resolved at read time, never flattened into the model.**
- **The host owns policy.** Fonts, I/O, network, persistence, transport.
- **No DOM or canvas as source of truth.** Views project the display list.
- **No silent data loss.** Anything unmodelled is retained verbatim or named in
  the compatibility report. The word "lossless" is banned unless the exact
  fidelity dimension is named.
- **A verb we lack is left off the ribbon**, not drawn dead.
- **A document stating a contract the code does not keep is a tracker row**, not
  a doc edit.
- **Report outcomes faithfully.** Preserved-but-not-modelled and
  parsed-but-not-rendered are said exactly that way.

## Current state

**Phases 0 and 1 are open, under waiver W-1** ([06](docs/06-ROADMAP-AND-DELIVERY.md)). The package reader and part graph are built. Tiers 1A, 1B and 1C are built but not closed, and nothing renders yet. All ADRs are Accepted.
