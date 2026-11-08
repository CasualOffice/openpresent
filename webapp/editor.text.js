// Text editing on the stage (`VS-05`): click into a text box, see a caret, type,
// delete, select, Bold, Italic and Underline, undo and redo.
//
// The engine keeps the text and lays it out; this module keeps only where the
// caret is (`editor.selection.js`) and draws it. A click becomes a hit test in
// the engine, the caret and the selection come back as slide coordinates and are
// drawn in an SVG over the slide picture, and keystrokes go to a hidden textarea
// so the browser's own input methods work. Every edit is one engine call, applied
// to the deck's model through `casual-slide-transaction`, so it can be undone.
// No document string reaches the DOM: the textarea is emptied after each input.
//
// Shaped after OpenCalc's in-cell editor (`webapp/editor.edit.js` at `b23c67c`):
// a hidden input owns focus and IME, and the canvas only draws.

import { state, request, changed, status } from "./editor.core.js";
import { selection, select, clearSelection, textRange } from "./editor.selection.js";
import { register } from "./commands.js";
import * as slides from "./editor.slides.js";

const SVG = "http://www.w3.org/2000/svg";
const byId = (id) => document.getElementById(id);

/// Text calls run in order, ahead of thumbnails, behind nothing but opening.
const TEXT_PRIORITY = 8;

let overlay = null;    // <svg> over the slide
let input = null;      // hidden <textarea>
let undoBits = 0;      // bit 1 undo, bit 2 redo
let formatBits = 0;    // bit 1 bold, 2 italic, 4 underline
let composing = false;
let dragging = false;
let blink = 0;

async function call(fn, ...args) {
  const r = await request({ op: "text", fn, args }, { priority: TEXT_PRIORITY });
  undoBits = r.undo | 0;
  return r.value;
}

const obj = () => selection.object;

/* ── Coordinates ────────────────────────────────────────────────────────── */

/// A client point to slide EMU.
function toSlide(clientX, clientY) {
  const r = byId("slide-frame").getBoundingClientRect();
  return [((clientX - r.left) / r.width) * state.deck.width, ((clientY - r.top) / r.height) * state.deck.height];
}

/* ── Drawing the caret and selection ────────────────────────────────────── */

function clearOverlay() {
  if (overlay) overlay.replaceChildren();
}

async function paint() {
  clearOverlay();
  clearInterval(blink);
  if (!selection.editing || !obj() || !state.deck) return;
  const [part, node] = obj();
  const index = state.current;
  const [start, end] = textRange();
  try {
    if (start !== end) {
      const quads = await call("text_selection", index, part, node, start, end);
      for (let i = 0; i + 8 <= quads.length; i += 8) {
        const poly = document.createElementNS(SVG, "polygon");
        poly.setAttribute("points", quads.slice(i, i + 8).join(" "));
        poly.setAttribute("class", "text-sel");
        overlay.appendChild(poly);
      }
    }
    const [x1, y1, x2, y2] = await call("text_caret", index, part, node, selection.focus);
    const line = document.createElementNS(SVG, "line");
    for (const [k, v] of [["x1", x1], ["y1", y1], ["x2", x2], ["y2", y2]]) line.setAttribute(k, String(v));
    line.setAttribute("class", "text-caret");
    line.setAttribute("vector-effect", "non-scaling-stroke");
    overlay.appendChild(line);
    // The textarea follows the caret, so an IME's window opens beside it.
    const r = byId("slide-frame").getBoundingClientRect();
    input.style.left = `${(x1 / state.deck.width) * r.width}px`;
    input.style.top = `${(y1 / state.deck.height) * r.height}px`;
    let on = true;
    blink = setInterval(() => { on = !on; line.style.visibility = on ? "visible" : "hidden"; }, 530);
    formatBits = await call("text_format_state", part, node, selection.focus);
  } catch (e) {
    status(`The caret could not be placed: ${e.message}`);
  }
  changed();
}

/// After an edit: redraw the slide and its thumbnail, then the caret.
async function afterEdit() {
  byId("doc-state").textContent = "Edited";
  await slides.redraw();
  slides.refreshThumb(state.current);
  await paint();
}

/* ── Entering and leaving a text box ────────────────────────────────────── */

function stopEditing() {
  if (!selection.editing) return;
  clearSelection();
  call("edit_break_typing").catch(() => {});
  clearOverlay();
  clearInterval(blink);
  input.blur();
  changed();
}

async function onPointerDown(e) {
  if (!state.deck || state.reading || e.button !== 0) return;
  if (!e.target.closest("#slide-frame")) { stopEditing(); return; }
  const [x, y] = toSlide(e.clientX, e.clientY);
  const index = state.current;
  if (e.shiftKey && selection.editing) {
    const [part, node] = obj();
    select({ focus: await call("text_offset_near", index, part, node, x, y) });
    paint();
    return;
  }
  let hit;
  try { hit = await call("text_hit", index, x, y); } catch { hit = []; }
  if (!hit.length) { stopEditing(); return; }
  e.preventDefault();
  const object = [hit[0], hit[1]];
  await call("edit_break_typing");
  select({ slide: index, object, editing: true, anchor: hit[2], focus: hit[2] });
  input.focus({ preventScroll: true });
  dragging = true;
  paint();
}

async function onPointerMove(e) {
  if (!dragging || !selection.editing || !(e.buttons & 1)) return;
  const [x, y] = toSlide(e.clientX, e.clientY);
  const [part, node] = obj();
  const focus = await call("text_offset_near", state.current, part, node, x, y);
  if (focus !== selection.focus) { select({ focus }); paint(); }
}

/* ── Edits ──────────────────────────────────────────────────────────────── */

let queue = Promise.resolve();
/// Edits run one after another, each seeing the caret the last one left.
function serial(fn) {
  queue = queue.then(fn).catch((e) => status(`The edit could not be made: ${e.message}`));
  return queue;
}

/// Replace the selection with `text`; `typing` joins keystrokes in one undo step.
function insert(text, typing = true) {
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    let [start, end] = textRange();
    if (start !== end) {
      await call("text_delete", part, node, start, end);
      typing = false;
    }
    await call("text_insert", part, node, start, text, typing);
    // The caret moves past what was inserted; dropped control characters are
    // found by the length, which is the engine's.
    const length = await call("text_length", part, node);
    const at = Math.min(length, start + [...text].length);
    select({ anchor: at, focus: at });
    await afterEdit();
  });
}

function remove(direction) {
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    let [start, end] = textRange();
    if (start === end) {
      const length = await call("text_length", part, node);
      if (direction < 0) start = Math.max(0, start - 1); else end = Math.min(length, end + 1);
      if (start === end) return;
    }
    await call("text_delete", part, node, start, end);
    select({ anchor: start, focus: start });
    await afterEdit();
  });
}

function toggle(which) {
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    const [start, end] = textRange();
    if (start === end) { status("Select text to format it."); return; }
    if (await call("text_toggle", part, node, start, end, which)) await afterEdit();
  });
}

function history(fn) {
  return serial(async () => {
    if (!state.deck) return;
    const changedObject = await call(fn);
    if (!changedObject.length) return;
    if (selection.editing) {
      const [part, node] = obj();
      try {
        const length = await call("text_length", part, node);
        select({ anchor: Math.min(selection.anchor, length), focus: Math.min(selection.focus, length) });
      } catch { clearSelection(); }
    }
    await afterEdit();
  });
}

/// Move the caret; `extend` keeps the anchor, as Shift does.
function move(how, extend) {
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    const [start, end] = textRange();
    let focus;
    if (how === "left" || how === "right") {
      const length = await call("text_length", part, node);
      if (!extend && start !== end) focus = how === "left" ? start : end;
      else focus = Math.max(0, Math.min(length, selection.focus + (how === "left" ? -1 : 1)));
    } else {
      const code = { home: 0, end: 1, up: 2, down: 3 }[how];
      focus = await call("text_navigate", state.current, part, node, selection.focus, code);
    }
    await call("edit_break_typing");
    select(extend ? { focus } : { anchor: focus, focus });
    await paint();
  });
}

function selectAll() {
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    select({ anchor: 0, focus: await call("text_length", part, node) });
    await paint();
  });
}

/* ── Keys ───────────────────────────────────────────────────────────────── */

function onKeyDown(e) {
  if (!selection.editing || composing || e.isComposing) return;
  const mod = e.ctrlKey || e.metaKey;
  const key = e.key.length === 1 ? e.key.toLowerCase() : e.key;
  let handled = true;
  if (mod && !e.altKey && key === "b") toggle(0);
  else if (mod && !e.altKey && key === "i") toggle(1);
  else if (mod && !e.altKey && key === "u") toggle(2);
  else if (mod && !e.altKey && key === "z" && !e.shiftKey) history("edit_undo");
  else if (mod && !e.altKey && (key === "y" || (key === "z" && e.shiftKey))) history("edit_redo");
  else if (mod && !e.altKey && key === "a") selectAll();
  else if (mod && !e.altKey && key === "s") import("./commands.js").then((c) => c.run("file.save"));
  else if (e.key === "Backspace") remove(-1);
  else if (e.key === "Delete") remove(1);
  else if (e.key === "Enter") insert(e.shiftKey ? "\v" : "\n", false);
  else if (e.key === "Tab" && !mod) insert("\t");
  else if (e.key === "ArrowLeft") move("left", e.shiftKey);
  else if (e.key === "ArrowRight") move("right", e.shiftKey);
  else if (e.key === "ArrowUp") move("up", e.shiftKey);
  else if (e.key === "ArrowDown") move("down", e.shiftKey);
  else if (e.key === "Home") move("home", e.shiftKey);
  else if (e.key === "End") move("end", e.shiftKey);
  else if (e.key === "Escape") stopEditing();
  else handled = false;
  if (handled) { e.preventDefault(); e.stopPropagation(); }
}

function onInput(e) {
  if (composing || e.isComposing) return;
  const text = input.value;
  input.value = "";
  if (text) insert(text);
}

/* ── Commands ───────────────────────────────────────────────────────────── */

/// Bold, Italic and Underline in the Home ribbon, and Undo and Redo, through the
/// registry. Called by the core before it fills in the commands not built yet.
export function registerTextCommands() {
  const editing = () => !!state.deck && selection.editing;
  register("font.bold", { label: "Bold", shortcut: "Ctrl+B", enabled: editing, checked: () => editing() && !!(formatBits & 1), run: () => toggle(0) });
  register("font.italic", { label: "Italic", shortcut: "Ctrl+I", enabled: editing, checked: () => editing() && !!(formatBits & 2), run: () => toggle(1) });
  register("font.underline", { label: "Underline", shortcut: "Ctrl+U", enabled: editing, checked: () => editing() && !!(formatBits & 4), run: () => toggle(2) });
  register("edit.undo", { label: "Undo", shortcut: "Ctrl+Z", enabled: () => !!state.deck && !!(undoBits & 1), run: () => history("edit_undo") });
  register("edit.redo", { label: "Redo", shortcut: "Ctrl+Y", enabled: () => !!state.deck && !!(undoBits & 2), run: () => history("edit_redo") });
}

/// Undo and redo from the page's own keys, when no text box is open.
export function handleGlobalKey(e) {
  const mod = e.ctrlKey || e.metaKey;
  if (!mod || e.altKey || !state.deck) return false;
  const key = e.key.toLowerCase();
  if (key === "z" && !e.shiftKey) { history("edit_undo"); return true; }
  if (key === "y" || (key === "z" && e.shiftKey)) { history("edit_redo"); return true; }
  return false;
}

/// A new deck, or another slide: no text is open.
export function reset() {
  clearSelection();
  clearOverlay();
  clearInterval(blink);
  undoBits = state.deck ? undoBits : 0;
}

/* ── Wiring ─────────────────────────────────────────────────────────────── */

export function init() {
  const frame = byId("slide-frame");
  overlay = document.createElementNS(SVG, "svg");
  overlay.setAttribute("class", "text-overlay");
  overlay.setAttribute("aria-hidden", "true");
  overlay.setAttribute("preserveAspectRatio", "none");
  frame.appendChild(overlay);
  input = document.createElement("textarea");
  input.className = "text-input";
  input.setAttribute("aria-label", "Text");
  input.setAttribute("autocomplete", "off");
  input.setAttribute("autocorrect", "off");
  input.setAttribute("spellcheck", "false");
  frame.appendChild(input);

  // The overlay's coordinates are the slide's, in EMU.
  const size = () => { if (state.deck) overlay.setAttribute("viewBox", `0 0 ${state.deck.width} ${state.deck.height}`); };
  new ResizeObserver(size).observe(frame);
  new MutationObserver(size).observe(frame, { attributes: true, attributeFilter: ["hidden"] });

  byId("stage").addEventListener("mousedown", onPointerDown);
  window.addEventListener("mousemove", onPointerMove);
  window.addEventListener("mouseup", () => { dragging = false; });
  input.addEventListener("keydown", onKeyDown);
  input.addEventListener("input", onInput);
  input.addEventListener("compositionstart", () => { composing = true; });
  input.addEventListener("compositionend", (e) => {
    composing = false;
    input.value = "";
    if (e.data) insert(e.data);
  });
  input.addEventListener("paste", (e) => {
    e.preventDefault();
    const text = e.clipboardData?.getData("text/plain");
    if (text) insert(text.replace(/\r\n?/g, "\n"), false);
  });
}
