# webapp: the OpenPresent viewer and site

A static site: a landing page, docs, deploy and download pages, and the editor page,
which today opens a `.pptx` and shows its slides (`VS-01`, the first vertical slice,
owner instruction 44). The engine runs as WebAssembly in a worker; the deck is read
from a file the user picks or drops and never leaves the browser. CI builds it and
publishes it to https://casualoffice.github.io/openpresent/ through
`.github/workflows/pages.yml`, as OpenCalc publishes its own (`SITE-01`): this folder at the
site's root, the editor at `editor.html`, the slide gallery at `gallery/`, and the slides
on the front page drawn by `render-deck` into `shots/` at build time.

## Build and serve

```sh
wasm-pack build crates/casual-slide-wasm --release --target web --out-dir "$PWD/webapp/pkg"
```

Then serve `webapp/` over HTTP (modules and a module worker do not load from
`file://`). `webapp/pkg/` is build output and git-ignored.

## Files

| File | Job |
| --- | --- |
| `editor.html` | The editor page: title bar, ribbon anchor, thumbnail pane, stage, notes pane, status bar. Strict CSP |
| `editor.js` | The one module the page names |
| `editor.core.js` | State, the engine client and its priority queue, every command registered, keys, boot |
| `editor.files.js` | File picker, drag and drop; `download`, through which Save and every Export pass (the desktop shell's hook, `window.__openpresentNative.save(bytes, ext, name)`) |
| `editor.slides.js` | Thumbnails, the stage, zoom, Reading View, the slide show |
| `editor.selection.js` | Shape selection on the stage (`VS-06`): click, Tab, drag to move, 8 resize and 1 rotate handle, nudge, Delete, undo and redo; after OpenCalc's `editor.selection.js`. One op per drag, at its end |
| `editor.text.js` | Text editing on the stage (`VS-05`): double click into a text box, caret, typing, Backspace and Delete, selection by drag, Shift+arrows and Ctrl+A, Bold, Italic and Underline (Home ribbon and Ctrl+B/I/U), undo and redo over the one history. The engine lays out and hit-tests; a hidden textarea takes keys and IME; caret and selection are drawn in an SVG in slide EMU. After OpenCalc's in-cell editor |
| `editor.dialogs.js` | The modal: errors, the per-slide report, shortcuts, about |
| `render.worker.js` | The engine in a worker: open, draw a slide to PNG, report, save, PDF |
| `commands.js` | The command registry |
| `ribbon.js`, `ribbon.model.js` | The ribbon, File page and Quick Access Toolbar, drawn from the model |
| `editor.css`, `ribbon.css`, `ribbon.tokens.css` | Tokens (accent `DEC-03`), regions, the ribbon |
| `icons.fluent.svg` | Fluent UI System Icons sprite |
| `index.html`, `docs.html`, `deploy.html`, `download.html` | The site: landing, user guide, self-hosting, desktop downloads. Strict CSP, no inline style or script |
| `style.css`, `site.js` | The site's stylesheet and its small script (reveals, mobile menu, docs contents); never loaded by the editor |

Every ribbon command PowerPoint has is present. The ones this build cannot run are
registered disabled, with the reason in their tooltip (owner, 2026-11-08), never faked.

## In the desktop app

`desktop/` (`VS-03`) embeds this folder unchanged. Two hooks are its only wiring:
`editor.core.js` sets `globalThis.openpresentEditor = { openDeck, runCommand }`, and
`openFilePicker` in `editor.files.js` calls `globalThis.__openpresentNative.open()`, the
platform's Open panel, when the desktop's bridge has installed it. In a browser neither exists
and nothing changes.

## Provenance

Copied from OpenCalc (`../opencalc/webapp`) at commit `b23c67c` on 2026-11-08, same
owner, Apache-2.0 (owner instruction 42), with `oc` prefixes renamed `op`:

- `commands.js`: verbatim but for the prefix and its header note.
- `ribbon.js`: cut to one page shell. Kept: model-driven controls run through the
  registry, folding by rank, the File page, Options, Ribbon Display Options,
  search (Alt+Q), Ctrl+F1. Not yet: embedded shells, combos, toolbar popups,
  KeyTips, the phone bar. Changed: disabled rather than hidden commands; the
  sprite is parsed with `DOMParser` instead of `innerHTML`.
- `ribbon.model.js`: OpenCalc's builders and File page model, with PowerPoint's
  tabs and groups (docs/47, docs/49).
- `ribbon.css`, `ribbon.tokens.css`: trimmed to what the shell draws; the icon font
  removed.
- `editor.css`: OpenCalc's token block with the accent changed to cyan; the regions
  are new.
- `editor.js`: its single-entry rule.
- `editor.dialogs.js`: its `.oc-modal`, built with `textContent`.
- `render.worker.js`: the idea of OpenCalc's `file.worker.js`, the engine off the
  page's thread.
- `icons.fluent.svg`: OpenCalc's generated sprite, verbatim, from
  `@fluentui/svg-icons` 1.1.344 (MIT, Copyright Microsoft Corporation), as its
  header says.
- `style.css`: OpenCalc's, with every green turned to cyan (the brand values are
  `editor.css`'s), the Inter web font left out so the system face is used, and a
  `.shots` block added for the real renders.
- `site.js`: verbatim but for its header note.
- `index.html`, `docs.html`, `download.html`, `deploy.html`: OpenCalc's pages' structure,
  classes, header, footer and tone. The words are new, since only true claims may stand
  (`SITE-01`, `PV-01`): `docs.html` is a user guide rather than an SDK guide, `download.html`
  points at CI artifacts as there is no release, and `deploy.html` covers static hosting,
  as there is no server. The animated hero and illustrations are not carried; the pictures
  are real renders. Inline `style` attributes and the download page's inline script are
  dropped, so the pages hold a strict CSP.
