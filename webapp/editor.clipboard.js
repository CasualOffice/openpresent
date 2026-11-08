// Cut, Copy, Paste and Duplicate (`UX-003`).
//
// Copied in approach from OpenCalc's `webapp/editor.clipboard.js` at `b23c67c`
// (webapp/README.md, owner instruction 42): the engine keeps its own copy of what was
// copied, which carries everything the system clipboard's plain text cannot, and the
// page writes a plain-text stand-in to the system clipboard. A paste uses the
// engine's copy while the system clipboard still holds that stand-in (or cannot be
// read), and the system clipboard's text otherwise, as OpenCalc's `doPaste` decides
// between `session_clip_paste` and `session_paste_tsv`. What changes for slides: the
// copy is shapes or text, not cells; text pasted from another application while no
// box is open becomes a new text box, as PowerPoint's; and the engine's copy lives
// outside the open deck, so shapes paste into a new one.
//
// Keys come as the browser's own `copy`, `cut` and `paste` events, which carry the
// system clipboard without asking for a permission; the ribbon's buttons use the
// async clipboard API for writing only. Paste from the ribbon uses the engine's copy,
// since reading the system clipboard there would ask the user for permission.

import { state, request, status, changed } from "./editor.core.js";
import { register } from "./commands.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";
import * as text from "./editor.text.js";
import { errorDialog } from "./editor.dialogs.js";

const byId = (id) => document.getElementById(id);
/// A new text box for pasted text: PowerPoint's default, 4 by 0.4 inches.
const PASTED_BOX = [3657600, 365760];

let lastClipText = null;   // the stand-in written for the engine's last copy
let held = 0;              // shapes the engine's clipboard holds

/// Whether keys and events belong to the slide editor, not a dialog or a field.
function ours(target) {
  if (!state.deck || !byId("op-modal").hidden || slides.showing()) return false;
  if (text.editing()) return true;
  const t = target || document.activeElement;
  if (t && !t.classList?.contains("text-input") && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName))) return false;
  return !t?.closest?.(".task-pane, .rb-pop, .shp-pop");
}

async function call(fn, args) {
  return request({ op: "build", fn, args }, { priority: 9 });
}

/// The text of the selected shapes, joined by line feeds: the system clipboard's
/// stand-in for them, as PowerPoint puts a shape's text there.
async function shapesText(shapes) {
  const parts = [];
  for (const s of shapes) {
    if (s.part == null) continue;
    try {
      const n = (await request({ op: "text", fn: "text_length", args: [s.part, s.node] }, { priority: 9 })).value;
      if (n) parts.push((await request({ op: "text", fn: "text_slice", args: [s.part, s.node, 0, n] }, { priority: 9 })).value.replace(/\v/g, "\n"));
    } catch { /* no text */ }
  }
  return parts.join("\n");
}

/// Copy: the open box's selected text, or the selected shapes. Returns the stand-in
/// text for the system clipboard, or null when nothing was copied.
async function copy() {
  if (text.editing()) {
    const t = await text.selectedText();
    if (!t) { status("Select text to copy it."); return null; }
    lastClipText = null;
    return t;
  }
  const shapes = selection.selectedShapes();
  if (!shapes.length) return null;
  try {
    const r = await call("copy_shapes", [state.current, shapes.map((s) => s.node)]);
    held = r.value;
  } catch (e) {
    errorDialog("OpenPresent cannot copy that yet.", e.message);
    return null;
  }
  lastClipText = await shapesText(shapes);
  status(held === 1 ? "Copied 1 shape" : `Copied ${held} shapes`);
  return lastClipText;
}

async function cut() {
  const editing = text.editing();
  const t = await copy();
  if (t == null) return null;
  if (editing) text.cutSelection();
  else await selection.deleteSelection();
  return t;
}

/// Paste the engine's shapes onto the current slide and select them.
async function pasteShapes() {
  const index = state.current;
  try {
    const r = await call("paste_shapes", [index]);
    if (!r.value.length) { status("The clipboard is empty"); return; }
    selection.noteUndoState(r.undo);
    selection.selectNodes(r.value);
    await slides.redrawSlide(index);
    status(r.value.length === 1 ? "Pasted 1 shape" : `Pasted ${r.value.length} shapes`);
  } catch (e) {
    errorDialog("OpenPresent cannot paste that here yet.", e.message);
  }
}

/// Text from another application, with no box open: a new text box holding it.
async function pasteTextBox(t) {
  const index = state.current;
  const [cx, cy] = PASTED_BOX;
  const x = Math.max(0, Math.round((state.deck.width - cx) / 2));
  const y = Math.max(0, Math.round((state.deck.height - cy) / 2));
  try {
    const r = await call("insert_shape", [index, 0, x, y, cx, cy]);
    const node = r.value;
    const hit = await request({ op: "text", fn: "text_hit", args: [index, x + cx / 2, y + cy / 2] }, { priority: 9 });
    const part = hit.value[0];
    const done = await request({ op: "text", fn: "text_insert", args: [part, node, 0, t.replace(/\r\n?/g, "\n"), false] }, { priority: 9 });
    selection.noteUndoState(done.undo);
    selection.selectNode(node);
    await slides.redrawSlide(index);
    status("Pasted as a text box");
  } catch (e) {
    errorDialog("OpenPresent could not paste that text.", e.message);
  }
}

/// Paste: `osText` is the system clipboard's text, or null when it could not be read.
async function paste(osText) {
  if (text.editing()) {
    if (osText) text.pasteText(osText);
    return;
  }
  if (held && (osText == null || osText === "" || osText === lastClipText)) { await pasteShapes(); return; }
  if (osText) { await pasteTextBox(osText); return; }
  status("The clipboard is empty");
}

async function duplicate() {
  const nodes = selection.selectedNodes();
  if (!nodes.length) return;
  const index = state.current;
  try {
    const r = await call("duplicate_shapes", [index, nodes]);
    selection.noteUndoState(r.undo);
    selection.selectNodes(r.value);
    await slides.redrawSlide(index);
    status("Duplicated");
  } catch (e) {
    errorDialog("OpenPresent cannot duplicate that yet.", e.message);
  }
}

/// The ribbon's Copy and Cut: the system clipboard is written through the async
/// API, which needs no permission inside a click.
async function viaRibbon(fn) {
  const t = await fn();
  if (t == null) return;
  try { await navigator.clipboard.writeText(t); } catch { status("Copied here; the system clipboard was refused, so other applications will not see it"); }
  changed();
}

export function init() {
  const canCopy = () => !!state.deck && (text.editing() || selection.hasSelection());
  register("edit.copy", { label: "Copy", shortcut: "Ctrl+C", enabled: canCopy, run: () => viaRibbon(copy) });
  register("edit.cut", { label: "Cut", shortcut: "Ctrl+X", enabled: canCopy, run: () => viaRibbon(cut) });
  register("edit.paste", { label: "Paste", shortcut: "Ctrl+V", enabled: () => !!state.deck && (held > 0 || text.editing()), run: () => paste(null) });
  register("edit.duplicate", { label: "Duplicate", shortcut: "Ctrl+D", enabled: () => !!state.deck && selection.hasSelection() && !text.editing(), run: duplicate });

  // The keys: the browser's own events, which carry the system clipboard.
  document.addEventListener("copy", (e) => {
    if (!ours(e.target)) return;
    if (!text.editing() && !selection.hasSelection()) return;
    e.preventDefault();
    // The event's clipboard must be written now; the engine answers later, so
    // the stand-in follows through the async API.
    copy().then((t) => { if (t != null) navigator.clipboard.writeText(t).catch(() => {}); changed(); });
  });
  document.addEventListener("cut", (e) => {
    if (!ours(e.target)) return;
    if (!text.editing() && !selection.hasSelection()) return;
    e.preventDefault();
    cut().then((t) => { if (t != null) navigator.clipboard.writeText(t).catch(() => {}); changed(); });
  });
  document.addEventListener("paste", (e) => {
    if (!ours(e.target)) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    paste(e.clipboardData ? e.clipboardData.getData("text/plain") : null);
  }, true);
}
