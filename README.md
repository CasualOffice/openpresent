# OpenPresent

[![CI](https://github.com/CasualOffice/openpresent/actions/workflows/ci.yml/badge.svg)](https://github.com/CasualOffice/openpresent/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Status: pre-alpha](https://img.shields.io/badge/status-pre--alpha-orange.svg)](#status)

**A PowerPoint-compatible presentation engine and editor that you can run anywhere.**
A deterministic Rust engine reads, models, lays out and writes `.pptx`, and runs natively or in
the browser through WebAssembly. On top of it sits an editor built to work like PowerPoint, on the
web and as a desktop app, so a PowerPoint user needs no teaching.

The engine is built first, and fidelity comes before features: a deck opened and saved by OpenPresent
should keep everything PowerPoint put in it.

> **Pre-alpha.** The file engine is under construction; nothing renders yet, and there is no editor.
> See [Status](#status) for exactly what works today.

Part of [CasualOffice](https://github.com/CasualOffice), with
[OpenCalc](https://github.com/CasualOffice/opencalc) (spreadsheets) and
[OpenDoc](https://github.com/CasualOffice/opendoc) (documents), whose architecture it shares.

---

## Why another presentation editor?

Most open editors can open a `.pptx`. Fewer can save one back without quietly losing what they did not
understand: animations they do not model, extensions from newer Office versions, a SmartArt layout,
a chart's embedded workbook. The damage shows up later, for the next person who opens the file in PowerPoint.

OpenPresent is designed around that problem:

- **Nothing is lost silently.** Every part of a deck is either modelled or kept byte for byte, and
  anything that cannot be kept is named in a compatibility report.
- **Edits are surgical.** Saving an edited deck rewrites only the XML nodes that changed. Every
  untouched part keeps its exact bytes.
- **Coverage is measured, not claimed.** Every element, attribute and enum value in the Office Open XML
  schemas, 8,739 in all, is a row in a [spec checklist](docs/spec/README.md). A row is marked supported
  only when code and a test deck prove it.
- **Every file is treated as hostile.** Zip bombs, entity expansion, path tricks, tracking links and
  credential-leaking UNC paths are refused or reported. Nothing is fetched and no macro ever runs.
- **The engine owns text layout.** Every collaborator, every export and every browser sees the same
  line breaks.

## Status

| Area | State |
| --- | --- |
| Package reading: bounded ZIP, part graph, content types, relationships | ✅ Tier 1A closed |
| Opening damaged decks: salvage, XML recovery, repair | 🟡 Built, tier open; its fuzz targets not yet run |
| Saving: unedited byte-identical, edits patch only what changed; the writer | ✅ Tier 1N closed |
| Markup compatibility, Strict and Transitional, percent-encoded part names | 🟡 Built, tier open |
| Geometry: all 187 preset shapes and 41 text warps | ✅ Built |
| Presentation part, view state, colour, themes | ✅ Tiers 1B and 1C closed |
| Slides, masters, layouts, shape tree, inheritance | ✅ Tier 1D closed |
| Fills, lines, effects, 3D, text | ✅ Tiers 1E and 1F closed |
| Tables | ✅ Tier 1G closed |
| Pictures, media | 🟡 Built, tier open |
| Animations and transitions | 🟡 Built, tier open |
| Charts, SmartArt, equations, ink | 🟡 Built, tier open; limits not yet mutation-tested |
| OLE, VML, 3D models: kept with their fallback | 🟡 Built, tier open |
| Comments, sections, zoom, labels, tags, custom XML | ✅ Tier 1M closed |
| Rendering to PNG and PDF | 🟡 Phase 2 started: slides to PNG, and to PDF with real, extractable text (`VS-04`) |
| Viewing a deck in the browser: thumbnails, slide, slide show (`VS-01`) | 🟡 Built, untested in a browser; CI publishes it with the gallery |
| Desktop app: the same viewer in a native window, Open panel, double-click a `.pptx` (`VS-03`) | 🟡 Built; CI bundles it for Linux, macOS and Windows, not yet seen green |
| Save and export in the browser: Save (byte-identical unedited), PDF, slide as PNG (`VS-04`) | 🟡 Built; its engine tests run on CI, untested in a browser |
| Editing, co-editing | ⬜ Later slices |

**All Phase 1 tiers, 1A to 1N and 1R, are closed** (2026-11-08). Of 8,739 checklist rows, 7,843 are marked `y` or `r` in model, import and export; 469 more are `n/a` in a column that does not apply to them; the 427 others each carry a named reason in `docs/spec/open-rows.tsv` or are accounted for by their members, and `tools/spec-audit` checks every one. What each tier still lacks is in the [tracker](docs/14-EXECUTION-TRACKER.md) under "Tier gates".

The full plan, with how each phase opens and closes, is the [roadmap](docs/06-ROADMAP-AND-DELIVERY.md).
Every task is in the [execution tracker](docs/14-EXECUTION-TRACKER.md). 66 real PowerPoint-written decks open and save with every XML part byte-identical. PowerPoint itself has not yet checked our output; until then, nothing here claims to match it.

## Try it

You need Rust 1.88 or later. The toolchain is pinned in `rust-toolchain.toml`.

```sh
git clone https://github.com/CasualOffice/openpresent
cd openpresent
cargo test --workspace

# Inspect any deck: structure, conformance, and anything suspicious in it
cargo run -p casual-slide-ooxml --example inspect -- path/to/deck.pptx
```

The inspector prints the deck's kind and part count, plus each finding: external content that was not
fetched, unsafe links, macro projects, embedded objects, and parts nothing points to.

## How it is built

```
 Host (browser, desktop, server)
   └─ SDK / WebAssembly bridge
        ├─ Model, inheritance resolver, operations     casual-slide-model, -style, -transaction
        ├─ Geometry, text, layout, display list         casual-slide-geometry, -layout
        └─ Render backends: canvas, PNG, PDF            casual-slide-render
   Import and export                                    casual-slide-package, -ooxml, -import, -export
```

Each layer depends only on the layers below it, and the engine has no network capability at all. Some
crates above are still to come. The [architecture](docs/02-ARCHITECTURE.md) explains what is shared with
OpenCalc and what had to be different.

| Crate | What it does today |
| --- | --- |
| `casual-slide-package` | Bounded ZIP and OPC admission: size, ratio and entry limits, safe part names, raw-copy rewriting |
| `casual-slide-ooxml` | Part graph, the single bounded XML reader, a byte-faithful XML tree, markup compatibility |
| `casual-slide-model` | The presentation as plain typed values |
| `casual-slide-import`, `-export` | XML to model, and model changes patched back into XML |
| `casual-slide-geometry` | Preset and custom shape geometry, deterministic on every platform |

## Documentation

| Read | For |
| --- | --- |
| [docs/00](docs/00-README.md) | The index of every design document |
| [Roadmap](docs/06-ROADMAP-AND-DELIVERY.md) | Phases, tiers, and their exit criteria |
| [Security threat model](docs/07-SECURITY-THREAT-MODEL.md) | Every threat, its control, and its test |
| [Spec checklist](docs/spec/README.md) | Every tag, property and value in the format |
| [Fidelity](docs/51-SPEC-CHECKLIST-AND-FIDELITY.md) | How fidelity is measured |
| [PowerPoint fit-gap](docs/96-POWERPOINT-FIT-GAP.md) | All 318 features, and our plan for each |

## Contributing

The project is designed before it is built, and every change follows one cycle: design, build, run it,
look for what is wrong, fix it, with a test that fails without the fix. [AGENTS.md](AGENTS.md) is the
contract and [CLAUDE.md](CLAUDE.md) the working rules, for people and coding agents alike. Commits follow
[Conventional Commits](https://www.conventionalcommits.org/) and cite a tracker row.

Run the gates before committing:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
python3 tools/check-repo.py
```

Found a security issue? Please report it privately to the maintainers rather than in a public issue.

## License

[Apache-2.0](LICENSE). The vendored Office Open XML schemas are © Ecma International and
Microsoft and are used under their published terms; see [schemas/README.md](schemas/README.md).

PowerPoint and Microsoft Office are trademarks of Microsoft Corporation. OpenPresent is not affiliated
with or endorsed by Microsoft.
