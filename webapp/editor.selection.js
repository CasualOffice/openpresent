// Shape selection on the stage: click to select, drag to move, PowerPoint's eight
// resize handles and one rotate handle, arrow-key nudge, Delete (`VS-06`).
//
// Shaped after OpenCalc's `webapp/editor.selection.js` at `b23c67c` (webapp/README.md,
// owner instruction 42): the selection lives here and nowhere else, and every verb
// goes through the command registry. The geometry is new, since a slide's objects are
// rotated boxes, not cells.
//
// The engine keeps the model. This module asks it for the slide's selectable shapes
// (numbers only: node, box in EMU, rotation, lock flags; no document text) and draws
// the outline over the engine's picture. A drag moves only that outline; the edit is
// sent once, when the pointer is released (docs/24: intermediate positions are not
// edits), and the engine redraws the slide.

import { state, request, status, changed } from "./editor.core.js";
import { register, run } from "./commands.js";
import * as slides from "./editor.slides.js";
import { errorDialog } from "./editor.dialogs.js";

const byId = (id) => document.getElementById(id);

// `ShapeBox`'s flag bits in `casual-slide-wasm`.
const NO_MOVE = 4, NO_RESIZE = 8, NO_ROTATE = 16, KEEP_ASPECT = 32;
const ROT_UNIT = 60000;               // `@rot` is in 60000ths of a degree
const NUDGE = 76200;                  // PowerPoint's grid step, 1/12 inch, in EMU
const NUDGE_FINE = 12700;             // one point, with Ctrl
const MIN_EXTENT = 12700;             // the smallest a drag makes a side, one point
const HANDLES = [[-1, -1], [0, -1], [1, -1], [1, 0], [1, 1], [0, 1], [-1, 1], [-1, 0]];

let shapes = [];        // [{ node, x, y, cx, cy, rot (degrees), flags }]
let selected = null;    // node
let drag = null;        // { kind, start, box, handle?, moved }
let undoState = [false, false];
let shapesToken = 0;

const shape = () => shapes.find((s) => s.node === selected) || null;
const scale = () => byId("slide-frame").clientWidth / state.deck.width;   // px per EMU

/* ── Geometry ───────────────────────────────────────────────────────────── */

function rotateVec(x, y, deg) {
  const r = (deg * Math.PI) / 180;
  const c = Math.cos(r), s = Math.sin(r);
  return [x * c - y * s, x * s + y * c];
}

/// The pointer in slide EMU.
function pointer(e) {
  const r = byId("slide-frame").getBoundingClientRect();
  const k = scale();
  return [(e.clientX - r.left) / k, (e.clientY - r.top) / k];
}

/// The point in the shape's own frame: centre at the origin, rotation removed.
function local(s, [px, py]) {
  return rotateVec(px - (s.x + s.cx / 2), py - (s.y + s.cy / 2), -s.rot);
}

function hit([px, py]) {
  // Topmost first: the last shape in z-order is drawn on top. A line has no
  // width, so every box is grown by a few pixels to be clickable.
  const slop = 4 / scale();
  for (let i = shapes.length - 1; i >= 0; i--) {
    const s = shapes[i];
    const [lx, ly] = local(s, [px, py]);
    if (Math.abs(lx) <= s.cx / 2 + slop && Math.abs(ly) <= s.cy / 2 + slop) return s;
  }
  return null;
}

/// The box a resize drag gives: the handle opposite stays where it is.
function resized(b, [hx, hy], p, keepAspect) {
  const [lx, ly] = local(b, p);
  const ax = (-hx * b.cx) / 2, ay = (-hy * b.cy) / 2;
  let w = hx ? Math.max(MIN_EXTENT, (lx - ax) * hx) : b.cx;
  let h = hy ? Math.max(MIN_EXTENT, (ly - ay) * hy) : b.cy;
  if (keepAspect && hx && hy && b.cx > 0 && b.cy > 0) {
    const k = Math.max(w / b.cx, h / b.cy);
    w = b.cx * k;
    h = b.cy * k;
  }
  const lcx = hx ? ax + (hx * w) / 2 : 0;
  const lcy = hy ? ay + (hy * h) / 2 : 0;
  const [dx, dy] = rotateVec(lcx, lcy, b.rot);
  const cx = b.x + b.cx / 2 + dx, cy = b.y + b.cy / 2 + dy;
  return { ...b, x: cx - w / 2, y: cy - h / 2, cx: w, cy: h };
}

/* ── Drawing the outline ────────────────────────────────────────────────── */

function layer() {
  let l = byId("sel-layer");
  if (!l) {
    l = document.createElement("div");
    l.id = "sel-layer";
    l.className = "sel-layer";
    byId("slide-frame").appendChild(l);
    l.addEventListener("pointerdown", onDown);
    l.addEventListener("pointermove", onMove);
    l.addEventListener("pointerup", onUp);
    l.addEventListener("pointercancel", () => { drag = null; paint(); });
  }
  return l;
}

function paint() {
  const l = layer();
  l.textContent = "";
  const b = drag?.box || shape();
  if (!b || state.reading) return;
  const k = scale();
  const box = document.createElement("div");
  box.className = "sel-box";
  box.style.left = `${b.x * k}px`;
  box.style.top = `${b.y * k}px`;
  box.style.width = `${b.cx * k}px`;
  box.style.height = `${b.cy * k}px`;
  box.style.transform = `rotate(${b.rot}deg)`;
  if (!(b.flags & NO_RESIZE)) {
    for (const [hx, hy] of HANDLES) {
      const h = document.createElement("div");
      h.className = "sel-handle";
      h.dataset.handle = `${hx},${hy}`;
      h.style.left = `${((hx + 1) / 2) * 100}%`;
      h.style.top = `${((hy + 1) / 2) * 100}%`;
      h.style.cursor = cursorFor(hx, hy, b.rot);
      box.appendChild(h);
    }
  }
  if (!(b.flags & NO_ROTATE)) {
    const r = document.createElement("div");
    r.className = "sel-rotate";
    r.dataset.handle = "rotate";
    box.appendChild(r);
  }
  l.appendChild(box);
}

function cursorFor(hx, hy, rot) {
  const names = ["ns-resize", "nesw-resize", "ew-resize", "nwse-resize"];
  const angle = (Math.atan2(hy, hx) * 180) / Math.PI + rot + 90;
  const step = Math.round((((angle % 180) + 180) % 180) / 45) % 4;
  return names[step];
}

/* ── Pointer ────────────────────────────────────────────────────────────── */

function onDown(e) {
  if (!state.deck || state.reading || e.button !== 0) return;
  const p = pointer(e);
  const handle = e.target.dataset?.handle;
  const s = shape();
  if (s && handle) {
    drag = handle === "rotate"
      ? { kind: "rotate", start: p, origin: s, box: { ...s } }
      : { kind: "resize", start: p, origin: s, box: { ...s }, handle: handle.split(",").map(Number) };
  } else {
    const found = hit(p);
    select(found ? found.node : null);
    if (!found) return;
    drag = { kind: "move", start: p, origin: found, box: { ...found } };
  }
  e.preventDefault();
  e.currentTarget.setPointerCapture(e.pointerId);
  byId("stage").focus({ preventScroll: true });
}

function onMove(e) {
  if (!drag) {
    const l = byId("sel-layer");
    if (l) l.style.cursor = hit(pointer(e)) ? "move" : "";
    return;
  }
  const p = pointer(e);
  const o = drag.origin;
  if (drag.kind === "move") {
    if (o.flags & NO_MOVE) return;
    let dx = p[0] - drag.start[0], dy = p[1] - drag.start[1];
    if (e.shiftKey) { if (Math.abs(dx) > Math.abs(dy)) dy = 0; else dx = 0; }
    drag.box = { ...o, x: o.x + dx, y: o.y + dy };
  } else if (drag.kind === "resize") {
    drag.box = resized(o, drag.handle, p, e.shiftKey || !!(o.flags & KEEP_ASPECT));
  } else {
    const cx = o.x + o.cx / 2, cy = o.y + o.cy / 2;
    let deg = (Math.atan2(p[1] - cy, p[0] - cx) * 180) / Math.PI + 90;
    if (e.shiftKey) deg = Math.round(deg / 15) * 15;
    drag.box = { ...o, rot: ((deg % 360) + 360) % 360 };
  }
  drag.moved = true;
  paint();
}

function onUp() {
  const d = drag;
  drag = null;
  if (!d || !d.moved) { paint(); return; }
  commit(d.box);
}

/* ── Edits ──────────────────────────────────────────────────────────────── */

function afterEdit(r) {
  if (r.undo) undoState = [!!r.undo[0], !!r.undo[1]];
  byId("doc-state").textContent = undoState[0] ? "Edited" : "";
  changed();
}

/// Send one box to the engine: one op, one undo step.
async function commit(b) {
  const index = state.current;
  const s = shape();
  if (s) Object.assign(s, b);            // show it where it was dropped at once
  paint();
  try {
    const r = await request({
      op: "transform", index, node: b.node,
      x: Math.round(b.x), y: Math.round(b.y), cx: Math.round(b.cx), cy: Math.round(b.cy),
      rot: Math.round(b.rot * ROT_UNIT),
    }, { priority: 9 });
    afterEdit(r);
  } catch (e) {
    errorDialog("OpenPresent could not change that shape.", e.message);
  }
  slides.redrawSlide(index);
}

async function deleteSelected() {
  const s = shape();
  if (!s) return;
  const index = state.current;
  try {
    const r = await request({ op: "delete", index, node: s.node }, { priority: 9 });
    afterEdit(r);
    select(null);
    slides.redrawSlide(index);
  } catch (e) {
    status("That shape cannot be deleted yet");
    errorDialog("OpenPresent cannot delete this shape yet.", e.message);
  }
}

async function history(op) {
  try {
    const r = await request({ op }, { priority: 9 });
    afterEdit(r);
    if (r.slide >= 0) {
      if (r.slide !== state.current) slides.go(r.slide);
      slides.redrawSlide(r.slide);
    }
  } catch (e) {
    errorDialog(`OpenPresent could not ${op}.`, e.message);
  }
}

function nudge(dx, dy) {
  const s = shape();
  if (!s || (s.flags & NO_MOVE)) return;
  commit({ ...s, x: s.x + dx, y: s.y + dy });
}

/* ── Selection ──────────────────────────────────────────────────────────── */

function select(node) {
  selected = node;
  paint();
  changed();
}

export function hasSelection() { return !!shape(); }

/// The slide on the stage changed or was redrawn: read its shapes again,
/// keeping the selection when the shape is still there.
export async function refresh() {
  if (!state.deck) return;
  const token = ++shapesToken;
  const index = state.current;
  try {
    const r = await request({ op: "shapes", index }, { priority: 8 });
    if (token !== shapesToken || index !== state.current) return;
    shapes = [];
    for (let i = 0; i + 6 < r.shapes.length; i += 7) {
      const [node, x, y, cx, cy, rot, flags] = r.shapes.slice(i, i + 7);
      shapes.push({ node, x, y, cx, cy, rot: rot / ROT_UNIT, flags });
    }
    if (!shape()) selected = null;
    paint();
    changed();
  } catch {
    shapes = [];
  }
}

/// A new deck, or another slide: nothing is selected.
export function reset(deckChanged) {
  selected = null;
  drag = null;
  shapes = [];
  if (deckChanged) undoState = [false, false];
  paint();
}

/// Keys while the stage has a selection. Returns true when the key was used.
export function onKey(e) {
  const mod = e.ctrlKey || e.metaKey;
  if (mod && !e.altKey && (e.key === "z" || e.key === "Z") && !e.shiftKey) { e.preventDefault(); run("edit.undo"); return true; }
  if (mod && !e.altKey && (e.key === "y" || e.key === "Y" || ((e.key === "z" || e.key === "Z") && e.shiftKey))) { e.preventDefault(); run("edit.redo"); return true; }
  if (!shape() || e.altKey) return false;
  if (e.key === "Delete" || e.key === "Backspace") { e.preventDefault(); run("shape.delete"); return true; }
  if (e.key === "Escape") { e.preventDefault(); select(null); return true; }
  if (e.key === "Tab") {
    e.preventDefault();
    const i = shapes.findIndex((s) => s.node === selected);
    const next = shapes[(i + (e.shiftKey ? shapes.length - 1 : 1)) % shapes.length];
    select(next ? next.node : null);
    return true;
  }
  const step = e.ctrlKey || e.metaKey ? NUDGE_FINE : NUDGE;
  const d = { ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step] }[e.key];
  if (d) { e.preventDefault(); nudge(d[0], d[1]); return true; }
  return false;
}

export function init() {
  const has = () => !!state.deck;
  register("edit.undo", { label: "Undo", shortcut: "Ctrl+Z", enabled: () => has() && undoState[0], run: () => history("undo") });
  register("edit.redo", { label: "Redo", shortcut: "Ctrl+Y", enabled: () => has() && undoState[1], run: () => history("redo") });
  register("shape.delete", { label: "Delete", shortcut: "Delete", enabled: () => has() && !!shape(), run: deleteSelected });
  for (const [id, label, dx, dy] of [
    ["shape.nudge-left", "Nudge Left", -NUDGE, 0], ["shape.nudge-right", "Nudge Right", NUDGE, 0],
    ["shape.nudge-up", "Nudge Up", 0, -NUDGE], ["shape.nudge-down", "Nudge Down", 0, NUDGE],
  ]) {
    register(id, { label, enabled: () => has() && !!shape() && !(shape().flags & NO_MOVE), run: () => nudge(dx, dy) });
  }
  window.addEventListener("resize", () => paint());
}

export { paint as repaint };
