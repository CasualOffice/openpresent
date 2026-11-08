// Insert > Symbols > Symbol (`UX-006`): a grid of common symbols. With a text box
// open the symbol goes in at the caret, as typing; otherwise a new text box holding
// it is placed on the slide, as PowerPoint inserts a symbol with nothing selected.
//
// Insert > Text > WordArt (`UX-006`): the engine's styles (`WORDART_STYLES`), each a
// text box saying "Your text here" with its fill, outline and text warp, placed in the
// middle of the slide and selected, one undo step. The warp is saved; this build's
// renderer draws the text unwarped.
//
// Insert > Link (Ctrl+K) on the selected shape (`UX-006`): a web or mail address the
// shape links to on a click in PowerPoint's show. The address is stored in the file,
// never fetched or opened here; the engine takes only `http:`, `https:` and `mailto:`.

import { register } from "./commands.js";
import { state, request } from "./editor.core.js";
import { showModal, closeModal, errorDialog } from "./editor.dialogs.js";
import * as selection from "./editor.selection.js";
import * as slides from "./editor.slides.js";
import * as text from "./editor.text.js";

// Our own pick of common symbols, by Unicode code point: punctuation, maths, arrows,
// currency, Greek, and marks.
const SYMBOLS = [
  0x00a9, 0x00ae, 0x2122, 0x00b0, 0x00b1, 0x00d7, 0x00f7, 0x2260, 0x2264, 0x2265, 0x2248, 0x221e,
  0x221a, 0x2211, 0x220f, 0x222b, 0x2202, 0x2206, 0x03c0, 0x00b5, 0x2030, 0x00bd, 0x00bc, 0x00be,
  0x20ac, 0x00a3, 0x00a5, 0x00a2, 0x20b9, 0x00a7, 0x00b6, 0x2022, 0x2026, 0x2013, 0x2014, 0x2018,
  0x2019, 0x201c, 0x201d, 0x00ab, 0x00bb, 0x2190, 0x2192, 0x2191, 0x2193, 0x2194, 0x21d2, 0x21d4,
  0x2605, 0x2606, 0x2713, 0x2717, 0x2660, 0x2663, 0x2665, 0x2666, 0x03b1, 0x03b2, 0x03b3, 0x03b4,
  0x03b8, 0x03bb, 0x03c3, 0x03c6, 0x03c9, 0x03a9, 0x2200, 0x2203, 0x2208, 0x2229, 0x222a, 0x2234,
];

async function insertSymbol(ch) {
  if (text.editing()) { text.pasteText(ch); return; }
  const index = state.current;
  try {
    const W = state.deck.width, H = state.deck.height;
    const box = [W * 0.45, H * 0.44, W * 0.1, H * 0.12].map(Math.round);
    const r = await request({ op: "build", fn: "insert_shape", args: [index, 0, ...box] }, { priority: 9 });
    const node = r.value;
    selection.selectNode(node);
    await slides.redrawSlide(index);
    await selection.refresh();
    const s = selection.selectedShape();
    if (!s || s.node !== node) return;
    const t = await request({ op: "text", fn: "text_insert", args: [s.part, node, 0, ch, false] }, { priority: 9 });
    if (t.undo) selection.noteUndoState(t.undo);
    await slides.redrawSlide(index);
  } catch (e) {
    errorDialog("OpenPresent could not insert the symbol.", e.message);
  }
}

function symbolDialog() {
  showModal("Symbol", (body) => {
    const grid = document.createElement("div");
    grid.className = "sym-grid";
    grid.setAttribute("role", "grid");
    for (const cp of SYMBOLS) {
      const ch = String.fromCodePoint(cp);
      const b = document.createElement("button");
      b.type = "button";
      b.className = "sym-cell";
      b.textContent = ch;
      b.title = `U+${cp.toString(16).toUpperCase().padStart(4, "0")}`;
      b.addEventListener("click", () => { closeModal(); insertSymbol(ch); });
      grid.appendChild(b);
    }
    body.appendChild(grid);
  });
}

async function wordartDialog() {
  let names = [];
  try { names = (await request({ op: "build", fn: "wordart_styles", args: [] }, { priority: 9 })).value; } catch (e) { errorDialog("OpenPresent could not list the WordArt styles.", e.message); return; }
  showModal("WordArt", (body) => {
    const grid = document.createElement("div");
    grid.className = "op-layouts";
    names.forEach((name, i) => {
      const b = document.createElement("button");
      b.type = "button";
      b.className = "op-btn op-layout";
      b.textContent = name;
      b.addEventListener("click", () => { closeModal(); insertWordArt(i); });
      grid.appendChild(b);
    });
    body.appendChild(grid);
  });
}

async function insertWordArt(style) {
  const index = state.current;
  try {
    const W = state.deck.width, H = state.deck.height;
    const box = [W * 0.15, H * 0.4, W * 0.7, H * 0.2].map(Math.round);
    const r = await request({ op: "build", fn: "insert_wordart", args: [index, style, ...box] }, { priority: 9 });
    if (r.undo) selection.noteUndoState(r.undo);
    selection.selectNode(r.value);
    await slides.redrawSlide(index);
    await selection.refresh();
  } catch (e) {
    errorDialog("OpenPresent could not insert the WordArt.", e.message);
  }
}

const oneShape = () => !!state.deck && selection.selectedNodes().length === 1 && selection.selectedNode() != null && !text.editing();

async function linkDialog() {
  const node = selection.selectedNode();
  const index = state.current;
  let now = null;
  try { now = (await request({ op: "build", fn: "shape_link", args: [index, node] }, { priority: 9 })).value ?? null; } catch { now = null; }
  showModal("Insert Link", (body) => {
    const label = document.createElement("label");
    label.className = "fp-row";
    label.appendChild(document.createTextNode("Address "));
    const input = document.createElement("input");
    input.type = "url";
    input.className = "fp-input";
    input.placeholder = "https://";
    input.value = now || "";
    input.setAttribute("aria-label", "Address");
    label.appendChild(input);
    const note = document.createElement("p");
    note.className = "fp-none";
    note.textContent = now === "" ? "This shape has a link from the file." : "Stored in the presentation; OpenPresent never opens it.";
    const row = document.createElement("div");
    row.className = "fd-buttons";
    const ok = document.createElement("button");
    ok.type = "button";
    ok.className = "op-btn primary";
    ok.textContent = "OK";
    const go = async () => {
      const url = input.value.trim();
      if (!url) { input.focus(); return; }
      try {
        const r = await request({ op: "build", fn: "set_shape_link", args: [index, node, url] }, { priority: 9 });
        if (r.undo) selection.noteUndoState(r.undo);
        closeModal();
      } catch {
        note.textContent = "Use a web address (https://…) or a mail address (mailto:…), without spaces.";
        input.focus();
      }
    };
    ok.addEventListener("click", go);
    input.addEventListener("keydown", (e) => { if (e.key === "Enter") { e.preventDefault(); go(); } });
    row.appendChild(ok);
    if (now) {
      // A link added in this session can go again; one from the file stays (`RIB-04`).
      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "op-btn";
      remove.textContent = "Remove Link";
      remove.addEventListener("click", async () => {
        try {
          const r = await request({ op: "build", fn: "set_shape_link", args: [index, node, ""] }, { priority: 9 });
          if (r.undo) selection.noteUndoState(r.undo);
          closeModal();
        } catch (e) { errorDialog("OpenPresent could not remove the link.", e.message); }
      });
      row.appendChild(remove);
    }
    body.append(label, row, note);
    setTimeout(() => input.focus(), 0);
  });
}

/// Ctrl+K with one shape selected, from the editor's keys.
export function onKey(e) {
  const mod = e.ctrlKey || e.metaKey;
  if (!mod || e.altKey || e.shiftKey || (e.key !== "k" && e.key !== "K") || !oneShape()) return false;
  e.preventDefault();
  linkDialog();
  return true;
}

export function init() {
  register("insert.link", { label: "Link", shortcut: "Ctrl+K", enabled: oneShape, run: linkDialog });
  register("insert.wordart", { label: "WordArt", enabled: () => !!state.deck && state.deck.count > 0, run: wordartDialog });
  register("insert.symbol", { label: "Symbol", enabled: () => !!state.deck && state.deck.count > 0, run: symbolDialog });
}
