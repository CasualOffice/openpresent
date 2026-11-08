# webapp: the OpenPresent editor and site

A static site: a landing page, docs, deploy and download pages, and the editor page,
which opens a `.pptx` (or starts from a blank deck) and edits, presents, co-edits, saves
and exports it in PowerPoint for the web's shell (slices 1 to 12 of docs/06, `UX-003`,
`UX-011`). What works and what is left is in the root README's Status; true as of `8011d28`. The engine runs as WebAssembly in a worker; the deck is read
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
`file://`), for example `python3 -m http.server -d webapp 8000` and open
`http://127.0.0.1:8000/editor.html`. That is all self-hosting needs. To co-edit, let the
collaboration server serve the folder instead: `OPENPRESENT_COLLAB_FILES=../../webapp cargo run`
in `server/casual-slide-collab-server` (127.0.0.1:8787 by default). `webapp/pkg/` is build
output and git-ignored.

## Files

| File | Job |
| --- | --- |
| `editor.html` | The editor page: title bar, ribbon anchor, thumbnail pane, stage, notes pane, status bar. Strict CSP |
| `editor.js` | The one module the page names |
| `editor.core.js` | State, the engine client and its priority queue, every command registered, keys, boot |
| `editor.files.js` | File picker, drag and drop; `download`, through which Save and every Export pass (the desktop shell's hook, `window.__openpresentNative.save(bytes, ext, name)`) |
| `editor.slides.js` | Thumbnails, the stage, zoom, Reading View, the notes pane |
| `editor.show.js` | The slide show (`VS-07`): From Beginning, From Current Slide and Presenter View; PowerPoint's keys, number then Enter, black and white screens, hidden slides skipped; fade, push, wipe, split and cut played as themselves and other transitions as a fade, reported; a slide's animations click by click (`VS-10`) |
| `editor.animate.js` | The Transitions and Animations tabs and the Animation Pane (`VS-10`): transition gallery, Duration, Apply To All; Animation Styles, Add Animation, Start, Move Earlier and Later; the pane lists the order and reorders and deletes |
| `presenter.html`, `presenter.js`, `presenter.css` | Presenter View's window: current and next slide, notes as text, timer, clock, slide count; driven by `editor.show.js` through `window.opener` |
| `editor.design.js` | The Design tab (`VS-12`): Themes and Variants galleries of our own built-in themes, Slide Size (4:3, 16:9, custom; Maximize or Ensure Fit) and Format Background (solid, gradient, reset; one slide or all), each a registered command and one undo step; after OpenCalc's ribbon galleries |
| `editor.selection.js` | Shape selection on the stage (`VS-06`): click, Tab, drag to move, 8 resize and 1 rotate handle, nudge, Delete, undo and redo; after OpenCalc's `editor.selection.js`. One op per drag, at its end. Smart guides while moving (`PAR-15`): edges and centres snap to other shapes' and the slide's within 6 pixels, red dashed lines show each match, Alt turns it off. Groups as PowerPoint's (`UX-015`): a first click selects the group, a second the shape inside or the caret in its text; a press passes through a group's empty parts |
| `editor.objects.js` | Rich objects (`VS-11`): Insert > Table (grid picker), double click a cell to type, Insert and Delete Rows and Columns, Table Styles; Insert > Chart (column, bar, line, pie) and Edit Data, a data sheet shaped after OpenCalc's `editor.charts.js` and chart panel at `b23c67c`. Each edit is one engine op in the one history |
| `collab.js` | Co-editing's socket and protocol (`COL-04`): copied from OpenCalc's `webapp/collab.js` at `b23c67c` and changed only for slides: engine calls go through the worker and are handled one message at a time; a join carries the deck as the room's seed, and every welcome reopens the room's seed and replays its log; no resume or redirect (`COL-05`) |
| `editor.presence.js` | Share, who is here, and others' selection outline, caret and name in their colour over the slide (`COL-04`); after OpenCalc's `editor.presence.js` at `b23c67c`. The link is this page with `?room=`; the socket is this origin, so `connect-src 'self'` holds |
| `editor.build.js` | Building a deck (`VS-08`): New Slide (Ctrl+M) on the current slide's layout and New Slide from Layout from the deck's own layouts, Duplicate, Delete (also Delete in the pane) and Hide Slide, reorder by dragging thumbnails, Insert Text Box, Shapes (rectangle, oval, line, arrow) and Pictures (PNG, JPEG). Each is one engine op in the one history. The editor starts on the built-in blank deck, and File > New opens it again |
| `editor.text.js` | Text editing on the stage (`VS-05`, `VS-17`): one click into a text box, as PowerPoint, caret, typing, Backspace and Delete, selection by drag, Shift+arrows and Ctrl+A, double click a word and triple click a paragraph, Ctrl+arrows by word and paragraph, Ctrl+Home and End, Ctrl+Backspace and Delete by word (`PAR-12`, the browser's `Intl.Segmenter` for word breaks, CJK included), Tab and Shift+Tab as list levels at a paragraph's start (`PAR-06`), Bold, Italic and Underline (Home ribbon and Ctrl+B/I/U), undo and redo over the one history. The engine lays out and hit-tests; a hidden textarea takes keys and IME; caret and selection are drawn in an SVG in slide EMU. After OpenCalc's in-cell editor |
| `editor.shapes.js`, `shapes.catalog.js` | Insert > Shapes (`SHAPE-01`): every one of the 187 presets in PowerPoint's groups, icons drawn by the geometry engine, a drawing tool, yellow adjust handles, Shape Fill, Shape Outline and Shape Styles on the Shape Format tab |
| `editor.picture.js` | The Picture Format tab (`UX-003`): Crop with handles, Reset Picture, Reset Picture & Size, basic Picture Styles |
| `editor.panehost.js` | The task pane host (`UX-011`): one pane at the right, a registry so a pane is one module, title, close, Esc, a resize grip, Ctrl+F6; after OpenCalc's side panel |
| `editor.comments.js` | The Comments pane (`UX-011`): modern threaded comments on a slide or shape, replies, resolve, reopen, delete, markers on the slide; an @mention is text, sent nowhere |
| `editor.notes.js` | Speaker notes typed in the pane under the slide (`UX-011`), through the same text operations as shapes; a notes slide is created on the first keystroke |
| `editor.selpane.js` | The Selection Pane (`UX-011`): list, rename, hide or show, reorder by drag |
| `editor.author.js` | Who comments are signed as (`UX-011`): a local setting asked in the Comments pane or File > Options |
| `editor.dialogs.js` | The modal: errors, the per-slide report, shortcuts, about |
| `editor.file.js` | The File tab (`FILE-01`): rename in the title bar (OpenCalc's `doc-name` and `doc-rename` from `editor.core.js` at `b23c67c`), File > Info's properties edited in place and Advanced Properties (Summary, Statistics, Custom), Save (the type the name says), Save As (name and type through the engine's save-as grid, F12), Download a Copy, Print (every shown slide one a page, the browser's print, Ctrl+P). Each property change is one engine op in the one history |
| `editor.drafts.js` | Autosave and recovery (`SAVE-01`): copied from OpenCalc's `editor.drafts.js` at `b23c67c`: drafts in this browser's IndexedDB after 5 s of quiet, at most 60 s apart, and when the tab hides; one slot a tab over a `BroadcastChannel`; a recovery bar that offers, never applies, and only when the editor opens (`UX-014`). Changed: the engine is a worker, so the counter and bytes are requests; Restore opens here when the deck on screen is unedited, else in a new tab; undo history is not kept. The leave warning (`PAR-01`): OpenCalc's `beforeunload`, asking only while the engine's edit counter, carried on every worker answer, differs from the last save |
| `editor.recent.js` | Recent presentations (`PAR-19`): every deck opened from this device kept, bytes and all, in this browser's IndexedDB (OpenCalc's draft store's shape; OpenCalc has no recent list), listed on File > Open and New, pinned first, then the twelve most recent; Pin, Remove, and open again from the list. Nothing uploaded |
| `editor.menus.js` | Right-click menus for the slide, a thumbnail, a shape and text, from `CONTEXT_MENUS` in the model; Shift+F10 and the Menu key; Ctrl+F6 between regions (`UX-003`, docs/53) |
| `editor.panes.js` | The Format Shape task pane: Shape Options and Text Options; Fill & Line, Effects, Size & Properties; size, position, rotation, no or solid fill and line, line colour and width edited here, the rest disabled until editable (`UX-003`) |
| `editor.home.js` | The Home tab's Font and Paragraph groups (`UX-003`): Font and Size boxes with the caret's values, Increase and Decrease, Font Color, Highlight, alignment, Bullets, and (`PAR-06`) the Bullets and Numbering galleries, Increase and Decrease List Level and Line Spacing. The Font box is OpenCalc's `fontFamilies`, `wireCombo` and `toolbar.font` from `editor.core.js` at `b23c67c` (owner instruction 54), its faces registered from the engine's bytes |
| `editor.clipboard.js` | Cut, Copy, Paste and Duplicate of shapes and text (`UX-003`), after OpenCalc's `editor.clipboard.js` at `b23c67c`: the engine keeps the copy, the system clipboard a plain-text stand-in. Formatted paste from other applications and Paste Special (`PAR-11`), its HTML read as OpenCalc's `cellsFromClipboardHtml` reads it; pictures from other applications; slides copied and pasted in the thumbnail pane, Ctrl+ and Shift+click to select several, within and between decks (`PAR-10`) |
| `editor.keys.js` | PowerPoint's keyboard shortcuts (`PAR-13`): one table from keys to command ids, run through the registry, ahead of the browser's own; a key whose command is not built says so in the status bar |
| `render.worker.js` | The engine in a worker: open, draw a slide to PNG, report, save, PDF, edits, and the show's plan and notes |
| `commands.js` | The command registry |
| `ribbon.js`, `ribbon.model.js` | The ribbon (PowerPoint for the web's tabs, groups and order, docs/53), contextual Shape Format, the tab row's Comments, Editing, Present and Share, File page and Quick Access Toolbar, drawn from the model |
| `editor.css`, `ribbon.css`, `ribbon.tokens.css` | Tokens (accent `DEC-03`), regions, the ribbon |
| `icons.fluent.svg` | Fluent UI System Icons sprite |
| `index.html`, `docs.html`, `deploy.html`, `download.html` | The site: landing, user guide, self-hosting, desktop downloads. Strict CSP, no inline style or script |
| `style.css`, `site.js` | The site's stylesheet and its small script (reveals, the hero's demo strip, Try a sample deck, mobile menu, docs contents); never loaded by the editor |

At build time `pages.yml` adds `shots/` (slides drawn by `render-deck`, and `editor.png`, the editor as the runner's Chrome shows it), `samples/sample-deck.pptx` (a CC BY deck from `fixtures/real`), and through `tools/site/build.py` the site's address in canonical and Open Graph URLs, `robots.txt` and `sitemap.xml` (`SITE-03`).

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
  search (Alt+Q), Ctrl+F1. Added for PowerPoint (`UX-003`): combos, in-ribbon
  galleries, dialog launchers, contextual tabs, labelled small buttons, folding
  that reopens what fits, the tab row's end and `contextMenu`. KeyTips (`UX-003`). Not yet: embedded
  shells, toolbar popups, the phone bar. Changed: disabled rather than hidden commands; the
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
- `icons.fluent.svg`: OpenCalc's generated sprite, from
  `@fluentui/svg-icons` 1.1.344 (MIT, Copyright Microsoft Corporation), as its
  header says, with 88 more for PowerPoint's commands. Rebuilt by OpenCalc's
  `tools/icons/`, copied to this repository with its licence: `npm ci && npm run
  build` there, or `FLUENT_ICONS=<unpacked package> node build-sprite.mjs`.
- `style.css`: OpenCalc's, with every green turned to cyan (the brand values are
  `editor.css`'s), the Inter web font left out so the system face is used, and a
  `.shots` block added for the real renders.
- `site.js`: verbatim but for its header note.
- `index.html`, `docs.html`, `download.html`, `deploy.html`: OpenCalc's pages' structure,
  classes, header, footer and tone. The words are new, since only true claims may stand
  (`SITE-01`, `PV-01`): `docs.html` is a user guide rather than an SDK guide, `download.html`
  points at CI artifacts as there is no release, and `deploy.html` covers static hosting,
  as there was no server when it was written (co-editing's server is `SITE-04`). The animated hero and illustrations are not carried; the pictures
  are real renders. Inline `style` attributes and the download page's inline script are
  dropped, so the pages hold a strict CSP.
