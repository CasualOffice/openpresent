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

**Website and editor: https://casualoffice.github.io/openpresent/**. Open a deck in the
[editor](https://casualoffice.github.io/openpresent/editor.html), or see our renders beside
LibreOffice's in the [slide gallery](https://casualoffice.github.io/openpresent/gallery/index.html).

> **Pre-alpha.** It opens, edits, presents, co-edits and exports decks, each in a first, thin form.
> See [Status](#status) for exactly what works today and what is left.

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

Docs are true as of `8011d28` (2026-11-09).

**Phases 0 and 1 are closed** (Phase 1 signed off by the owner on 2026-11-08): every Phase 1 tier, 1A to 1N and 1R, is closed. Of 8,739 checklist rows, 7,843 are marked `y` or `r` in model, import and export; 469 more are `n/a` in a column that does not apply to them; the 427 others each carry a named reason in `docs/spec/open-rows.tsv` or are accounted for by their members, and `tools/spec-audit` checks every one. 66 real PowerPoint-written decks open and save with every XML part byte-identical.

Since then the work runs as **vertical slices** ([roadmap](docs/06-ROADMAP-AND-DELIVERY.md) §After Phase 1). Slices 1 to 12 are built and merged; **none is closed**, because each still lacks part of its end point, most often a CI browser run. What is left is named in each slice's [tracker](docs/14-EXECUTION-TRACKER.md) row.

| What you can do | State | Left |
| --- | --- | --- |
| Open a `.pptx` in the browser: thumbnails, zoom, Reading View, report of what a slide leaves out (slice 1) | ✅ Built, [live](https://casualoffice.github.io/openpresent/editor.html) | CI browser run over every real deck (`VS-01`) |
| See tables, charts, SmartArt and OLE fallbacks, shadows and glows drawn (slice 2) | 🟡 In progress | Render tiers' gates (`VS-02`, `FH-04`) |
| Desktop app on Linux, macOS and Windows, double-click a `.pptx` (slice 3) | ✅ Built; bundles green on CI at `50ea80d` | Dependency policy over its lockfile, a drawn-slide check on macOS and Windows (`DESK-01`, `DESK-02`) |
| Save (byte-identical when unedited), export PDF with real text, slide as PNG (slice 4) | ✅ Done (`VS-04`) | The owner's sign-off |
| Edit text: click, type, IME, select, Bold/Italic/Underline, fonts and sizes, colour, highlight, alignment, bullets, undo (slice 5, `UX-003`) | ✅ Built | CI browser run, IME check (`VS-15`) |
| Select, move, resize, rotate, multi-select, align, delete shapes, with undo (slice 6) | ✅ Built | Drag budget, groups, CI browser run (`VS-14`) |
| Present: slide show, Presenter View, notes, PowerPoint's keys (slice 7) | ✅ Built | Automatic advance, show settings, CI run (`VS-16`) |
| Build slides: New Slide from a layout, duplicate, hide, reorder, delete; insert text box, picture and all 187 shapes (slice 8, `SHAPE-01`) | ✅ Built | Change layout, shape effects, freeform (`VS-08`, `SHAPE-02`) |
| Co-edit: Share a link, see each other's edits, carets and presence (slice 9) | 🟡 Built; the CI two-page run is red on `main` (`CI-23`) | Resume, tokens, more edits on the wire (`COL-02` to `COL-08`) |
| Transitions and animations: six of each, played in the show, Animation Pane (slice 10) | ✅ Built | Effect options, timing, goldens (`VS-10`) |
| Insert and edit tables and charts (column, bar, line, pie), Edit Data (slice 11) | ✅ Built | Cell typing through the text engine, merge, more chart types (`VS-20`) |
| Themes, variants, slide size, background (slice 12) | ✅ Built | Slide Master editing (`DSN-01`) |
| File tab: rename, Info properties, Save As `.pptx`, `.potx`, `.ppsx`, Print; autosave and recovery in the browser | ✅ Built | Modified date stamp, atomic save, undo kept in drafts (`FILE-01`, `SAVE-01`) |
| Side panes: Comments (threaded, replies, resolve, saved as modern comments), editable speaker notes, the Selection Pane (`UX-011`) | 🟡 Built, seen in a browser | What its row lists (`UX-011`) |
| Hosts: WOPI, SDK, MCP (slice 13) | ⬜ Not started | `VS-13` |

**Fidelity, measured.** On every push to `main`, CI draws each gallery slide with our renderer and with LibreOffice and compares them ([docs/51](docs/51-SPEC-CHECKLIST-AND-FIDELITY.md)). At `1ba6443` (run 42511686894, LibreOffice 26.2.6.3, 111 slides): **on average 5.4% of pixels differ (94.6% agree)**, median 4.4%, SSIM 0.869; 64 slides within 5%, 14 within 1%. This is **agreement with LibreOffice, not a PowerPoint match**: PowerPoint itself has not yet checked our output (`PV-01`), and until it has nothing here claims to match it. See the [gallery](https://casualoffice.github.io/openpresent/gallery/).

**PowerPoint parity, audited.** The parity audit ([docs/96](docs/96-POWERPOINT-FIT-GAP.md) §Parity today, first at `7539ea5`, re-audited and kept current since) marks 352 PowerPoint features against the live editor: **55 work (15.6%)**, 53 work in part, 59 are present but disabled, 185 are absent, none faked. Of the 280 we mean to build, 55 work (19.6%). The first audit found 35 (9.9%).

## Try it

**In the browser:** open the [editor](https://casualoffice.github.io/openpresent/editor.html), then drop a `.pptx` on it or start from the blank deck. The deck is read and drawn inside your browser and is never uploaded.

**From source.** You need Rust 1.88 or later (pinned in `rust-toolchain.toml`).

```sh
git clone https://github.com/CasualOffice/openpresent
cd openpresent
cargo test --workspace

# Inspect any deck: structure, conformance, and anything suspicious in it
cargo run -p casual-slide-ooxml --example inspect -- path/to/deck.pptx

# Draw every slide of a deck to PNG, or the deck to PDF
cargo run --release -p casual-slide-render --example render-deck -- path/to/deck.pptx out/
cargo run --release -p casual-slide-render --example export-pdf -- path/to/deck.pptx out.pdf
```

The inspector prints the deck's kind and part count, plus each finding: external content that was not
fetched, unsafe links, macro projects, embedded objects, and parts nothing points to.

**Build and serve the editor yourself.** The editor is static files plus the engine as WebAssembly:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.14.0 --locked
wasm-pack build crates/casual-slide-wasm --release --target web --out-dir "$PWD/webapp/pkg"
python3 -m http.server -d webapp 8000     # then open http://127.0.0.1:8000/editor.html
```

Any static web server will do (modules and the worker do not load from `file://`); this is how the site is self-hosted, and `pages.yml` does the same for GitHub Pages ([webapp/README.md](webapp/README.md)).

**Co-edit.** The collaboration server serves the editor and relays edits; it never reads a deck:

```sh
# after building webapp/pkg as above
cd server/casual-slide-collab-server
OPENPRESENT_COLLAB_FILES=../../webapp cargo run --release
# listens on 127.0.0.1:8787 (OPENPRESENT_COLLAB_BIND to change it)
```

Open `http://127.0.0.1:8787/editor.html`, choose **Share**, and open the link in a second window. The server has no sign-in yet: the room key in the link is the only capability, so keep it on a trusted network (`COL-03`).

**Desktop.** `desktop/` is a Tauri app around the same editor. CI bundles it for Linux, macOS and Windows on every push (`desktop.yml`, artifacts on the run); to build locally, build `webapp/pkg` as above, then `cargo install tauri-cli --version "^2.0.0" --locked` and `cargo tauri build` in `desktop/`.

## How it is built

```
 Host (browser, desktop, collaboration server)
   └─ casual-slide-wasm: the bridge the editor's worker calls
        ├─ Operations, undo, co-editing wire                   casual-slide-transaction
        ├─ Inheritance resolver, geometry, text layout          casual-slide-style, -geometry, -text
        └─ Render backends: PNG and PDF                         casual-slide-render
   Import and export                                            casual-slide-package, -ooxml, -import, -export, -model
```

Each layer depends only on the layers below it, and the engine has no network capability at all. The
[architecture](docs/02-ARCHITECTURE.md) explains what is shared with OpenCalc and what had to be different;
[docs/19](docs/19-WORKSPACE-SCAFFOLD-DESIGN.md) has the crate graph as built.

| Crate | What it does today |
| --- | --- |
| `casual-slide-package` | Bounded ZIP and OPC admission: size, ratio and entry limits, safe part names, salvage, raw-copy rewriting |
| `casual-slide-ooxml` | Part graph, the single bounded XML reader, a byte-faithful XML tree, markup compatibility, recovery and repair |
| `casual-slide-model` | The presentation as plain typed values |
| `casual-slide-import`, `-export` | XML to model, and model changes patched back into XML by the one writer |
| `casual-slide-geometry` | Preset and custom shape geometry, deterministic on every platform |
| `casual-slide-style` | Effective properties through the master, layout and theme chain, resolved at read time |
| `casual-slide-text` | Fonts, shaping, line breaking and paragraph layout into glyph runs; caret stops |
| `casual-slide-render` | Slides to PNG and PDF: shapes, text, pictures, tables, charts, effects |
| `casual-slide-transaction` | Edit operations with exact inverses, undo history, and the co-editing seam |
| `casual-slide-wasm` | The browser bridge: open, draw, edit, save, export |
| `server/casual-slide-collab-server` | Orders and relays co-editing operations; holds no model (its own workspace) |
| `desktop/` | The Tauri desktop app around `webapp/` (its own workspace) |

## Documentation

| Read | For |
| --- | --- |
| [docs/00](docs/00-README.md) | The index of every design document |
| [Roadmap](docs/06-ROADMAP-AND-DELIVERY.md) | Phases, tiers, and their exit criteria |
| [Security threat model](docs/07-SECURITY-THREAT-MODEL.md) | Every threat, its control, and its test |
| [Spec checklist](docs/spec/README.md) | Every tag, property and value in the format |
| [Fidelity](docs/51-SPEC-CHECKLIST-AND-FIDELITY.md) | How fidelity is measured |
| [PowerPoint fit-gap](docs/96-POWERPOINT-FIT-GAP.md) | All 352 features, our plan for each, and the parity audit of the editor today |
| [Changelog](CHANGELOG.md) | What landed, by date |

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
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo check --target wasm32-unknown-unknown --workspace --locked
python3 tools/check-repo.py
```

The full list, exactly as CI runs it, is in [CLAUDE.md](CLAUDE.md).

Found a security issue? Please report it privately to the maintainers rather than in a public issue.

## License

[Apache-2.0](LICENSE). The vendored Office Open XML schemas are © Ecma International and
Microsoft and are used under their published terms; see [schemas/README.md](schemas/README.md).

PowerPoint and Microsoft Office are trademarks of Microsoft Corporation. OpenPresent is not affiliated
with or endorsed by Microsoft.
