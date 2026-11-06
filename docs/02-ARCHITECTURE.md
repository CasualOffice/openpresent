# 02 — Architecture

OpenSlide is OpenCalc's architecture applied to a different document: **a headless
engine with host adapters.** The engine owns the state, the layout and the display
list. The host owns the window, fonts, I/O and policy.

## The stack

```
 Host app ───▶ SDK / WASM bridge                 casual-slide-sdk, -wasm
               ├ Presentation runtime            -model, -style, -transaction
               ├ Layout runtime                  -geometry, -text, -layout, -chart
               ├ Animation runtime               -anim
               ├ Display list (backend-neutral)  in -layout
               └ Render backends                 Canvas2D (JS painter), -render (PNG, PDF)

 Cross-cutting: -package, -ooxml, -import, -export
 Beside it:     server/ (collab, host, WOPI, MCP)   webapp/   desktop/   sdk/
```

```
.pptx → bounded OPC admission → PresentationML part graph
      → semantic import + retained source → model + compatibility report
      → [edit] op → changed-object set
      → resolve inheritance → geometry + text layout → display list
      → paint (canvas, PNG, PDF)
      → export: byte-identical if unedited; untouched parts verbatim if edited
```

Security is designed against [07](07-SECURITY-THREAT-MODEL.md); the enterprise bar is [10](10-ENTERPRISE-REQUIREMENTS.md).

## Inherited from OpenCalc unchanged

Each of these has already been paid for there; see its docs for the reasoning.

- Rust core compiling unchanged to native and `wasm32-unknown-unknown`; no `#[cfg]`
  forks; platform needs arrive as host-supplied values (its ADR-019).
- Bounded admission of untrusted packages (its docs/21, ADR-006).
- Dual-axis loss accounting and a compatibility report (ADR-007).
- A serializable, backend-neutral display list with one painter per backend
  (ADR-008, ADR-021). Its JS painter costs about 0.6 ms per frame for 1434 items
  across the WASM boundary (`webapp/editor.paintlist.js`), so this seam is proven.
- Server-ordered OT with snapshot-plus-ops, resume, pipelining, leader per document
  and relay from any node (ADR-011, 012, 014 to 017, 031).
- A vanilla-JS editor with no framework or bundler: data-driven ribbon, command
  registry, Fluent icons (MIT), `--oc-*`-style tokens, shadow-root custom element.
- Time and randomness supplied, never sampled. Same gates and tracker discipline.

## Where a copy would be wrong

A presentation is not a grid, and these five differences are why this is a design
and not a rename. Each has a home in a pillar document.

**1. Text layout moves into the engine, in WASM.** OpenCalc draws with
`ctx.fillText` and keeps shaping out of the bundle (ADR-018), which is right for
cells nobody clicks a caret into by pixel. Here wrapping decides autofit, autofit
decides what fits, and two collaborators must see the same line breaks and the same
caret. Text layout is the product. This reverses ADR-018 for this project and is the
largest cost difference. See [42](42-TEXT-AND-RENDERING-ARCHITECTURE.md).

**2. Inheritance is the model's central fact.** A title's size may come from the
slide, its layout, its master, the master's text styles, or the presentation
defaults, and colours resolve through a theme and a colour map. Flattening on import
would make a master edit impossible and destroy round-trip. See
[22](22-NORMALIZED-SCHEMA.md).

**3. Collaboration needs character-level text OT.** OpenCalc's unit of conflict is
the cell. Here two people type in one text box, one drags a shape while another
restyles it, and slides reorder. See [24](24-OPERATIONS-AND-COLLABORATION.md).

**4. Time is a dimension.** Slide show needs a timing tree compiled to a sampled
timeline, and per-shape composition at 60 fps. OpenCalc has no equivalent. See
[42](42-TEXT-AND-RENDERING-ARCHITECTURE.md) §Slide show.

**5. Fidelity has better granularity.** OPC stores one part per slide, so an edit
can leave every other slide byte-identical. OpenCalc's byte-identical guarantee ends
at the first edit; ours can hold per part, which is a stronger and cheaper contract.

## What is simpler than OpenCalc

No formula language, no calc engine, no dependency graph, no pivot tables. Together
`casual-calc-formula`, `-eval` and the pivot code are the largest part of that
engine (`-eval` alone is about 29k lines). Their complexity budget goes to text and
fidelity instead.

## Lessons taken from OpenCalc's history

Designed in from the start, because each cost a repair there:

| What went wrong there | What we do from day one |
| --- | --- |
| Whole-vector replace for charts needed a protocol bump (`CHT-07`, protocol v8) | Per-element ops for every collection |
| Interned ids crossed the wire and meant different things per replica (ADR-025) | Values and stable ids on the wire; interning stays local |
| Primitives painted twice, Rust and JS, and diverged (`RND-10`) | One display list and painters that decide nothing, from the first pixel |
| `editor.core.js` reached 12k lines | Editor modules by feature, with a size gate |
| 60-odd verbs existed only in context menus, ungoverned (docs/91) | Every verb is a registered command first |
| `-selection` was scaffolded and is still 4 lines | No crate without a first consumer |
| Duplicate tracker ids and doc claims drifting from code | Copy the `check-*` gates in Phase 0 |
