// Text editing on the stage (`VS-05`): click into a text box, see a caret, type,
// delete, select, Bold, Italic and Underline, undo and redo.
//
// The engine keeps the text and lays it out; this module keeps only where the
// caret is and draws it. As PowerPoint, one click inside a shape that holds text
// puts the caret there; `editor.selection.js` (`VS-06`) routes the press, since it
// knows the shapes, and a press on a border selects and drags the shape instead.
// A click becomes a hit test in the engine, the caret and the selection come back
// as slide coordinates and are drawn in an SVG over the slide picture, and
// keystrokes go to a hidden textarea so the browser's own input methods work.
// Every edit is one engine call, applied to the deck's model through
// `casual-slide-transaction`, so it can be undone.
// No document string reaches the DOM: the textarea is emptied after each input.
//
// Fluid typing: keys typed while the engine is busy are joined into one insert,
// the stage is redrawn once per insert and the caret drawn with the new picture,
// and the thumbnail and the shape list wait until typing pauses.
//
// Shaped after OpenCalc's in-cell editor (`webapp/editor.edit.js` at `b23c67c`):
// a hidden input owns focus and IME, and the canvas only draws.

import { state, request, changed, status } from "./editor.core.js";
import { register, run } from "./commands.js";
import * as slides from "./editor.slides.js";
import * as shapes from "./editor.selection.js";

const SVG = "http://www.w3.org/2000/svg";
const byId = (id) => document.getElementById(id);

/// Text calls run in order, ahead of thumbnails, behind nothing but opening.
const TEXT_PRIORITY = 8;
/// How long typing pauses before the thumbnail and the shape list are read again.
const SETTLE_MS = 400;

let overlay = null;    // <svg> over the slide
let input = null;      // hidden <textarea>
let formatBits = 0;    // bit 1 bold, 2 italic, 4 underline
let composing = false;
let dragging = false;
let blink = 0;
let settleTimer = 0;

/// The open text box: the shape as `[part, node]` and the caret's anchor and focus,
/// character offsets in its body (`docs/24`). No document text.
const selection = { object: null, editing: false, anchor: 0, focus: 0 };
const select = (patch) => Object.assign(selection, patch);
const clearSelection = () => select({ object: null, editing: false, anchor: 0, focus: 0 });
const textRange = () => [Math.min(selection.anchor, selection.focus), Math.max(selection.anchor, selection.focus)];

/// Whether a text box is open, for the shape selection to stand aside.
export const editing = () => selection.editing;

async function call(fn, ...args) {
  const r = await request({ op: "text", fn, args }, { priority: TEXT_PRIORITY });
  if (r.undo && ["text_insert", "text_delete", "text_toggle"].includes(fn)) shapes.noteUndoState(r.undo);
  return r.value;
}

const obj = () => selection.object;

/* ── Drawing the caret and selection ────────────────────────────────────── */

function clearOverlay() {
  if (overlay) overlay.replaceChildren();
}

/// Where the caret and the selection are, from the engine: numbers only.
async function geometry() {
  if (!selection.editing || !obj() || !state.deck) return null;
  const [part, node] = obj();
  const index = state.current;
  const [start, end] = textRange();
  const quads = start !== end ? await call("text_selection", index, part, node, start, end) : [];
  const caret = await call("text_caret", index, part, node, selection.focus);
  formatBits = await call("text_format_state", part, node, selection.focus);
  return { quads, caret };
}

/// Draw what `geometry` found: the selection, then a blinking caret.
function draw(g) {
  clearOverlay();
  clearInterval(blink);
  if (!g || !selection.editing) return;
  const { quads, caret } = g;
  for (let i = 0; i + 8 <= quads.length; i += 8) {
    const poly = document.createElementNS(SVG, "polygon");
    poly.setAttribute("points", quads.slice(i, i + 8).join(" "));
    poly.setAttribute("class", "text-sel");
    overlay.appendChild(poly);
  }
  const [x1, y1, x2, y2] = caret;
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
  // A ribbon button took focus: the keys come back to the text.
  if (document.activeElement !== input) input.focus({ preventScroll: true });
}

async function paint() {
  try {
    draw(await geometry());
  } catch (e) {
    status(`The caret could not be placed: ${e.message}`);
  }
  changed();
}

/// After an edit: redraw the slide, and draw the caret with the new picture. The
/// thumbnail and the shape list follow once typing pauses.
async function afterEdit() {
  const [part, node] = obj();
  const [g] = await Promise.all([geometry().catch(() => null), slides.redraw({ quiet: true })]);
  draw(g);
  changed();
  const length = await call("text_length", part, node).catch(() => 1);
  shapes.noteEmpty(node, length === 0);
  clearTimeout(settleTimer);
  const index = state.current;
  settleTimer = setTimeout(() => {
    slides.refreshThumb(index);
    if (index === state.current) shapes.refresh();
  }, SETTLE_MS);
}

/* ── Entering and leaving a text box ────────────────────────────────────── */

/// Close the open text box, if any. The shape stays selected, as PowerPoint's.
let opens = 0;   // moves on with each open and close, so a late open is dropped

export function stop() {
  opens++;
  if (!selection.editing) return;
  clearSelection();
  call("edit_break_typing").catch(() => {});
  clearOverlay();
  clearInterval(blink);
  input.blur();
  shapes.repaint();
  changed();
}

/// A press inside text shape `[part, node]` at slide point `p`, routed here by the
/// shape selection: open the box with the caret under the pointer, or move the
/// caret in the open one; `extend` keeps the anchor, as Shift does.
export function pointerDown(part, node, p, extend) {
  const same = selection.editing && obj()[0] === part && obj()[1] === node;
  // Focus now, inside the press, so the keys that follow are not lost.
  input.focus({ preventScroll: true });
  dragging = true;
  const ticket = ++opens;
  serial(async () => {
    const at = await call("text_offset_near", state.current, part, node, p[0], p[1]);
    if (ticket !== opens) return;
    if (!same) {
      await call("edit_break_typing");
      select({ object: [part, node], editing: true, anchor: at, focus: at });
      shapes.repaint();
    } else {
      select(extend ? { focus: at } : { anchor: at, focus: at });
    }
    await paint();
  });
}

let moveQueued = null;
/// The pointer moved with the button down in a text box: extend the selection.
export function pointerMove(p) {
  if (!dragging || !selection.editing) return;
  // Only the latest point matters; the engine is asked once per answer.
  const pending = moveQueued;
  moveQueued = p;
  if (pending) return;
  serial(async () => {
    const q = moveQueued;
    moveQueued = null;
    if (!q || !selection.editing) return;
    const [part, node] = obj();
    const focus = await call("text_offset_near", state.current, part, node, q[0], q[1]);
    if (focus !== selection.focus) { select({ focus }); await paint(); }
  });
}

export function pointerUp() { dragging = false; }

/// A key typed while a text shape is selected but not open: open it at the end
/// of its text and type the key there, as PowerPoint does.
export function typeInto(part, node, key) {
  input.focus({ preventScroll: true });
  const ticket = ++opens;
  serial(async () => {
    const length = await call("text_length", part, node);
    if (ticket !== opens) return;
    await call("edit_break_typing");
    select({ object: [part, node], editing: true, anchor: length, focus: length });
    shapes.repaint();
  });
  insert(key);
}

/* ── Edits ──────────────────────────────────────────────────────────────── */

let queue = Promise.resolve();
/// Edits run one after another, each seeing the caret the last one left.
function serial(fn) {
  queue = queue.then(fn).catch((e) => status(`The edit could not be made: ${e.message}`));
  return queue;
}

let typed = "";
/// Replace the selection with `text`; `typing` joins keystrokes in one undo step.
/// Keys typed while an edit is in flight join the next insert, so the stage is
/// drawn once for them all.
function insert(text, typing = true) {
  if (typing) {
    const first = !typed;
    typed += text;
    if (!first) return queue;
    return serial(async () => {
      const t = typed;
      typed = "";
      await insertNow(t, true);
    });
  }
  return serial(() => insertNow(text, false));
}

async function insertNow(text, typing) {
  if (!selection.editing || !text) return;
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
  // From the ribbon, the button took focus: the next key is the text's again.
  if (selection.editing) input.focus({ preventScroll: true });
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    const [start, end] = textRange();
    if (start === end) { status("Select text to format it."); return; }
    if (await call("text_toggle", part, node, start, end, which)) await afterEdit();
  });
}

function history(id) {
  call("edit_break_typing").catch(() => {});
  run(id);
}

/// The stage was drawn again, after an undo, a redo or a move: keep the caret
/// inside the body, or close the box if its shape is gone.
export function refresh() {
  if (!selection.editing) return;
  serial(async () => {
    const [part, node] = obj();
    try {
      const length = await call("text_length", part, node);
      select({ anchor: Math.min(selection.anchor, length), focus: Math.min(selection.focus, length) });
      await paint();
    } catch {
      stop();
    }
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
  else if (mod && !e.altKey && key === "z" && !e.shiftKey) history("edit.undo");
  else if (mod && !e.altKey && (key === "y" || (key === "z" && e.shiftKey))) history("edit.redo");
  else if (mod && !e.altKey && key === "a") selectAll();
  else if (mod && !e.altKey && key === "s") run("file.save");
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
  else if (e.key === "Escape") { stop(); byId("stage").focus({ preventScroll: true }); }
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

/// Bold, Italic and Underline in the Home ribbon, through the registry. Undo and
/// Redo are `editor.selection.js`'s, over the one history. Called by the core
/// before it fills in the commands not built yet.
export function registerTextCommands() {
  const open = () => !!state.deck && selection.editing;
  register("font.bold", { label: "Bold", shortcut: "Ctrl+B", enabled: open, checked: () => open() && !!(formatBits & 1), run: () => toggle(0) });
  register("font.italic", { label: "Italic", shortcut: "Ctrl+I", enabled: open, checked: () => open() && !!(formatBits & 2), run: () => toggle(1) });
  register("font.underline", { label: "Underline", shortcut: "Ctrl+U", enabled: open, checked: () => open() && !!(formatBits & 4), run: () => toggle(2) });
}

/// A new deck, or another slide: no text is open.
export function reset() {
  if (selection.editing) input?.blur();
  clearSelection();
  clearOverlay();
  clearInterval(blink);
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

  // Presses on the slide are routed by `editor.selection.js`; one on the grey
  // around it closes the box and clears the selection, as PowerPoint's.
  byId("stage").addEventListener("pointerdown", (e) => {
    if (e.button !== 0 || e.target.closest("#slide-frame")) return;
    stop();
    shapes.deselect();
  });
  window.addEventListener("pointerup", () => { dragging = false; });
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
