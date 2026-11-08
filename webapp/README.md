# webapp: the OpenPresent viewer and site

A static site: a landing page, deploy and download pages, and the editor page,
which today opens a `.pptx` and shows its slides (`VS-01`, the first vertical slice,
owner instruction 44). The engine runs as WebAssembly in a worker; the deck is read
from a file the user picks or drops and never leaves the browser. CI builds it and
publishes it beside the slide gallery, at `app/`, through `.github/workflows/gallery.yml`.

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
| `editor.files.js` | File picker, drag and drop |
| `editor.slides.js` | Thumbnails, the stage, zoom, Reading View, the slide show |
| `editor.dialogs.js` | The modal: errors, the per-slide report, shortcuts, about |
| `render.worker.js` | The engine in a worker: open, draw a slide to PNG, report |
| `commands.js` | The command registry |
| `ribbon.js`, `ribbon.model.js` | The ribbon, File page and Quick Access Toolbar, drawn from the model |
| `editor.css`, `ribbon.css`, `ribbon.tokens.css` | Tokens (accent `DEC-03`), regions, the ribbon |
| `icons.fluent.svg` | Fluent UI System Icons sprite |
| `index.html`, `deploy.html`, `download.html`, `style.css` | The site |

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
- `style.css` and the site pages: OpenCalc's structure and class names; the content is new.
