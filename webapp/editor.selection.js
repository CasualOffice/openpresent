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
// the outline over the engine's picture. A drag moves the outline and a translucent
// copy of the shape cut from the slide picture, by CSS transform, so it follows the
// pointer at once; the edit is sent once, when the pointer is released (docs/24:
// intermediate positions are not edits), and the engine redraws the slide once.
//
// As PowerPoint: a press inside a shape that holds text puts the caret there
// (`editor.text.js`); a press on its border, or on a shape without text, selects it
// and drags it.
//
// Several shapes (`UX-003`): Shift-click or Ctrl-click adds or removes one, a drag
// from an empty part of the slide draws a marquee that selects every shape wholly
// inside it, and a drag of any selected shape moves them all. A group move, delete,
// nudge, align or distribute is one engine call and one undo step. Handles show
// only on a single selection, as PowerPoint's.

import { state, request, status, changed } from "./editor.core.js";
import { register, run } from "./commands.js";
import * as slides from "./editor.slides.js";
import { errorDialog } from "./editor.dialogs.js";
import * as text from "./editor.text.js";

const byId = (id) => document.getElementById(id);

// `ShapeBox`'s flag bits in `casual-slide-wasm`.
const NO_MOVE = 4, NO_RESIZE = 8, NO_ROTATE = 16, KEEP_ASPECT = 32;
const PLACEHOLDER = 128;              // `ShapeBox::PLACEHOLDER`: only these prompt
const ROT_UNIT = 60000;               // `@rot` is in 60000ths of a degree
const NUDGE = 76200;                  // PowerPoint's grid step, 1/12 inch, in EMU
const NUDGE_FINE = 12700;             // one point, with Ctrl
const MIN_EXTENT = 12700;             // the smallest a drag makes a side, one point
const HANDLES = [[-1, -1], [0, -1], [1, -1], [1, 0], [1, 1], [0, 1], [-1, 1], [-1, 0]];
const BORDER_PX = 6;                  // how far inside its edge a press still grabs a text shape

let shapes = [];        // [{ node, x, y, cx, cy, rot (degrees), flags, part?, empty? }]
let selected = null;    // the primary node: handles, the Format pane and the panes read it
let others = [];        // the rest of a multiple selection, nodes
let drag = null;        // { kind, start, box, handle?, moved }
let undoState = [false, false];
let shapesToken = 0;
let shapesFor = null;   // the slide index `shapes` was read for; null until read
let pendingPress = null; // a press made before the shapes were read: { p, shift }

const shape = () => shapes.find((s) => s.node === selected) || null;
/// Every selected shape, the primary last.
const chosen = () => {
  const p = shape();
  const rest = others.map((n) => shapes.find((s) => s.node === n)).filter((s) => s && s !== p);
  return p ? [...rest, p] : rest;
};
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

/// Whether a point lies inside `s` by more than the border a press grabs.
function interior(s, p) {
  const [lx, ly] = local(s, p);
  const b = BORDER_PX / scale();
  return Math.abs(lx) <= s.cx / 2 - b && Math.abs(ly) <= s.cy / 2 - b;
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
    l.addEventListener("pointercancel", () => { drag = null; dropGhost(); paint(); });
  }
  return l;
}

let frameQueued = false;
/// Paint on the next frame, once, however many pointer moves arrive before it.
function paintSoon() {
  if (frameQueued) return;
  frameQueued = true;
  requestAnimationFrame(() => { frameQueued = false; paint(); });
}

/* ── The drag's ghost ───────────────────────────────────────────────────── */
//
// A copy of the slide picture, clipped to the shape's box where the drag began and
// carried by one CSS transform onto the box under the pointer. The picture is the
// engine's, so it is the shape as drawn; nothing is drawn per frame.

let ghost = null;

/// The affine map taking the unit square, centred on the origin, to box `b`, in px.
function boxMatrix(b, k) {
  const r = (b.rot * Math.PI) / 180;
  const cos = Math.cos(r), sin = Math.sin(r);
  const w = b.cx * k, h = b.cy * k;
  return [cos * w, sin * w, -sin * h, cos * h, (b.x + b.cx / 2) * k, (b.y + b.cy / 2) * k];
}

function invert([a, b, c, d, e, f]) {
  const det = a * d - b * c;
  if (!det) return null;
  return [d / det, -b / det, -c / det, a / det, (c * f - d * e) / det, (b * e - a * f) / det];
}

function multiply([a1, b1, c1, d1, e1, f1], [a2, b2, c2, d2, e2, f2]) {
  return [a1 * a2 + c1 * b2, b1 * a2 + d1 * b2, a1 * c2 + c1 * d2, b1 * c2 + d1 * d2, a1 * e2 + c1 * f2 + e1, b1 * e2 + d1 * f2 + f1];
}

function makeGhost(origin) {
  dropGhost();
  const src = byId("slide-img").currentSrc || byId("slide-img").src;
  if (!src) return;
  const k = scale();
  const g = document.createElement("div");
  g.className = "sel-ghost";
  const img = document.createElement("img");
  img.src = src;
  img.alt = "";
  img.draggable = false;
  g.appendChild(img);
  // Clip to the shape's box, rotated as it is drawn.
  const m = boxMatrix(origin, k);
  const corners = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];
  const pts = corners.map(([u, v]) => `${m[0] * u + m[2] * v + m[4]}px ${m[1] * u + m[3] * v + m[5]}px`);
  img.style.clipPath = `polygon(${pts.join(", ")})`;
  ghost = { el: g, from: invert(m) };
  layer().prepend(g);
}

function moveGhost(b) {
  if (!ghost || !ghost.from) return;
  const t = multiply(boxMatrix(b, scale()), ghost.from);
  ghost.el.style.transform = `matrix(${t.map((v) => v.toFixed(4)).join(",")})`;
}

function dropGhost() {
  ghost?.el.remove();
  ghost = null;
}

/// Keep the ghost where it was dropped until the slide picture shows the edit.
function dropGhostWhenDrawn() {
  const g = ghost;
  if (!g) return;
  ghost = null;
  const img = byId("slide-img");
  const done = () => { g.el.remove(); img.removeEventListener("load", done); };
  img.addEventListener("load", done);
  setTimeout(done, 3000);
}

/// Who draws over the selection, as the yellow adjust handles (`editor.shapes.js`):
/// each is told the selected shape after every paint, or `null` while a drag runs.
const painters = [];
export function onPaint(f) { painters.push(f); }

function paint() {
  paintSelection();
  const s = drag || text.editing() || others.length ? null : shape();
  for (const f of painters) f(s ? { ...s, index: state.current } : null);
}

function paintSelection() {
  const l = layer();
  for (const el of [...l.children]) if (!el.classList.contains("sel-ghost")) el.remove();
  if (state.reading || !state.deck) return;
  // Empty text placeholders show where to click, as PowerPoint's dashed prompt; a
  // drawn shape or text box with no text shows none (`SHAPE-02`).
  const k = scale();
  for (const s of shapes) {
    if (!s.empty || !(s.flags & PLACEHOLDER) || (text.editing() && s.node === selected)) continue;
    const p = document.createElement("div");
    p.className = "sel-prompt";
    p.style.left = `${s.x * k}px`;
    p.style.top = `${s.y * k}px`;
    p.style.width = `${s.cx * k}px`;
    p.style.height = `${s.cy * k}px`;
    p.style.transform = `rotate(${s.rot}deg)`;
    const label = document.createElement("span");
    label.textContent = "Click to add text";
    p.appendChild(label);
    l.appendChild(p);
  }
  if (drag?.kind === "marquee" && drag.moved) {
    const m = document.createElement("div");
    m.className = "sel-marquee";
    const [x0, y0] = drag.start, [x1, y1] = drag.at;
    m.style.left = `${Math.min(x0, x1) * k}px`;
    m.style.top = `${Math.min(y0, y1) * k}px`;
    m.style.width = `${Math.abs(x1 - x0) * k}px`;
    m.style.height = `${Math.abs(y1 - y0) * k}px`;
    l.appendChild(m);
  }
  const all = chosen();
  if (all.length > 1) {
    // A multiple selection: an outline round each, no handles, as PowerPoint's.
    const moved = drag?.kind === "group" && drag.moved ? drag.boxes : null;
    for (const s of all) {
      const b = moved?.get(s.node) || s;
      const box = document.createElement("div");
      box.className = s.node === selected ? "sel-box is-multi is-primary" : "sel-box is-multi";
      box.style.left = `${b.x * k}px`;
      box.style.top = `${b.y * k}px`;
      box.style.width = `${b.cx * k}px`;
      box.style.height = `${b.cy * k}px`;
      box.style.transform = `rotate(${b.rot}deg)`;
      l.appendChild(box);
    }
    return;
  }
  const b = drag?.box || shape();
  if (!b) return;
  if (ghost && drag) moveGhost(b);
  const box = document.createElement("div");
  box.className = text.editing() ? "sel-box is-editing" : "sel-box";
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
  const s = chosen().length === 1 ? shape() : null;
  e.preventDefault();
  e.currentTarget.setPointerCapture(e.pointerId);
  if (s && handle) {
    text.stop();
    drag = handle === "rotate"
      ? { kind: "rotate", start: p, origin: s, box: { ...s } }
      : { kind: "resize", start: p, origin: s, box: { ...s }, handle: handle.split(",").map(Number) };
    return;
  }
  // The slide is drawn but its shapes are not read yet: hold the press, and the
  // keys typed after it, until they are (`VS-19`).
  if (shapesFor !== state.current) {
    pendingPress = { p, shift: e.shiftKey };
    text.hold();
    return;
  }
  press(p, e.shiftKey, e.shiftKey || e.ctrlKey || e.metaKey);
}

/// A press at slide point `p` on the shapes as read. `add`: Shift or Ctrl was held,
/// so the press adds the shape to the selection or takes it out (`UX-003`).
function press(p, shift, add = false) {
  const found = hit(p);
  if (add && !text.editing()) {
    text.stop();
    if (found) toggleInSelection(found.node);
    else drag = { kind: "marquee", start: p, at: p, add: true };
    byId("stage").focus({ preventScroll: true });
    return false;
  }
  // A press on a shape of a multiple selection moves them all.
  if (found && chosen().length > 1 && chosen().some((s) => s.node === found.node)) {
    text.stop();
    const all = chosen().map((s) => ({ ...s }));
    selected = found.node;
    others = all.map((s) => s.node).filter((n) => n !== found.node);
    drag = { kind: "group", start: p, origins: all };
    byId("stage").focus({ preventScroll: true });
    // On the next frame: painting now would detach the outline this press landed on
    // before the press reaches the stage, which reads a detached target as off the slide.
    paintSoon();
    return false;
  }
  // A press inside a shape that holds text puts the caret there (`VS-05`).
  if (found && found.part != null && interior(found, p)) {
    others = [];
    selected = found.node;
    drag = { kind: "text" };
    text.pointerDown(found.part, found.node, p, shift);
    paint();
    changed();
    return true;
  }
  text.stop();
  select(found ? found.node : null);
  byId("stage").focus({ preventScroll: true });
  if (found) drag = { kind: "move", start: p, origin: found, box: { ...found } };
  // A press on an empty part of the slide draws a marquee.
  else drag = { kind: "marquee", start: p, at: p, add: false };
  return false;
}

/// Shift-click: add the shape to the selection, or take it out.
function toggleInSelection(node) {
  const all = chosen().map((s) => s.node);
  const next = all.includes(node) ? all.filter((n) => n !== node) : [...all, node];
  selected = next.length ? next[next.length - 1] : null;
  others = next.slice(0, -1);
  paintSoon();   // as a group press: the outline pressed stays attached
  changed();
}

/// Select every shape wholly inside the marquee from `a` to `b`, as PowerPoint does.
function marqueeSelect(a, b, add) {
  const [x0, x1] = [Math.min(a[0], b[0]), Math.max(a[0], b[0])];
  const [y0, y1] = [Math.min(a[1], b[1]), Math.max(a[1], b[1])];
  const inside = shapes.filter((s) => s.x >= x0 && s.y >= y0 && s.x + s.cx <= x1 && s.y + s.cy <= y1).map((s) => s.node);
  const base = add ? chosen().map((s) => s.node) : [];
  const next = [...base, ...inside.filter((n) => !base.includes(n))];
  selected = next.length ? next[next.length - 1] : null;
  others = next.slice(0, -1);
  paint();
  changed();
}

/// Replay a press held while the shapes were read; the keys held with it follow
/// into the box it opens, or are dropped if it opened none.
function replayPress() {
  const held = pendingPress;
  pendingPress = null;
  if (!held) return;
  const opened = press(held.p, held.shift);
  // The button may be up by now: a replayed press does not drag.
  drag = null;
  text.pointerUp();
  text.release(opened);
}

function onMove(e) {
  if (!drag) {
    const l = byId("sel-layer");
    if (!l || !state.deck) return;
    const p = pointer(e);
    const s = hit(p);
    l.style.cursor = !s ? "" : s.part != null && interior(s, p) ? "text" : "move";
    return;
  }
  const p = pointer(e);
  if (drag.kind === "text") { text.pointerMove(p); return; }
  if (drag.kind === "marquee") {
    drag.at = p;
    drag.moved = drag.moved || Math.hypot(p[0] - drag.start[0], p[1] - drag.start[1]) * scale() >= 3;
    paintSoon();
    return;
  }
  if (drag.kind === "group") {
    if (!drag.moved && Math.hypot(p[0] - drag.start[0], p[1] - drag.start[1]) * scale() < 3) return;
    let dx = p[0] - drag.start[0], dy = p[1] - drag.start[1];
    if (e.shiftKey) { if (Math.abs(dx) > Math.abs(dy)) dy = 0; else dx = 0; }
    drag.boxes = new Map(drag.origins.filter((o) => !(o.flags & NO_MOVE)).map((o) => [o.node, { ...o, x: o.x + dx, y: o.y + dy }]));
    drag.moved = true;
    paintSoon();
    return;
  }
  const o = drag.origin;
  if (!drag.moved) {
    // A press that does not move is a click; the ghost starts with the drag.
    const slop = 3 / scale();
    if (drag.kind === "move" && Math.hypot(p[0] - drag.start[0], p[1] - drag.start[1]) < slop) return;
    makeGhost(o);
  }
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
  paintSoon();
}

function onUp() {
  const d = drag;
  drag = null;
  if (d?.kind === "text") { text.pointerUp(); return; }
  if (d?.kind === "marquee") { if (d.moved) marqueeSelect(d.start, d.at, d.add); else paint(); return; }
  // A click on one shape of several, with no drag, selects that one alone, as PowerPoint.
  if (d?.kind === "group") { if (d.moved) commitMany([...d.boxes.values()]); else select(selected); return; }
  if (!d || !d.moved) { dropGhost(); paint(); return; }
  commit(d.box);
}

/* ── Edits ──────────────────────────────────────────────────────────────── */

/// A text edit's undo state (`VS-05`): the one history serves both slices.
export function noteUndoState(u) {
  afterEdit({ undo: u });
}

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
  moveGhost(b);
  dropGhostWhenDrawn();
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

/// Send several boxes as one edit, one undo step (`UX-003`).
async function commitMany(boxes) {
  if (!boxes.length) { paint(); return; }
  const index = state.current;
  for (const b of boxes) { const s = shapes.find((x) => x.node === b.node); if (s) Object.assign(s, b); }
  paint();
  const flat = [];
  for (const b of boxes) flat.push(b.node, Math.round(b.x), Math.round(b.y), Math.round(b.cx), Math.round(b.cy), Math.round(b.rot * ROT_UNIT));
  try {
    const r = await request({ op: "build", fn: "set_shape_transforms", args: [index, flat] }, { priority: 9 });
    afterEdit(r);
  } catch (e) {
    errorDialog("OpenPresent could not move those shapes.", e.message);
  }
  slides.redrawSlide(index);
}

export async function deleteSelection() {
  const all = chosen();
  const s = shape();
  if (!s) return;
  const index = state.current;
  try {
    const r = all.length > 1
      ? await request({ op: "build", fn: "delete_shapes", args: [index, all.map((x) => x.node)] }, { priority: 9 })
      : await request({ op: "delete", index, node: s.node }, { priority: 9 });
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
    // A theme, the slide size or several backgrounds (`VS-12`): every slide is drawn again.
    if (r.slide === -2 && r.count === state.deck.count) { slides.redrawAll(r.size); return; }
    // A slide added, deleted, moved or hidden comes back too (`VS-08`): the pane is
    // read again whenever the count changed or no single slide was named.
    if (typeof r.count === "number" && (r.count !== state.deck.count || r.slide < 0)) {
      slides.reloadList(r.count, r.slide >= 0 ? r.slide : state.current);
      return;
    }
    document.dispatchEvent(new CustomEvent("op-slides-built"));   // a hide undone
    if (r.slide >= 0) {
      if (r.slide !== state.current) slides.go(r.slide);
      slides.redrawSlide(r.slide);
    }
  } catch (e) {
    errorDialog(`OpenPresent could not ${op}.`, e.message);
  }
}

function nudge(dx, dy) {
  const all = chosen().filter((s) => !(s.flags & NO_MOVE));
  if (all.length > 1) { commitMany(all.map((s) => ({ ...s, x: s.x + dx, y: s.y + dy }))); return; }
  const s = shape();
  if (!s || (s.flags & NO_MOVE)) return;
  commit({ ...s, x: s.x + dx, y: s.y + dy });
}

/// Place the primary shape at box `b` (EMU): the Format Shape pane's Size and
/// Position fields (`UX-003`).
export function place(b) {
  const s = shape();
  if (!s) return;
  commit({ ...s, ...b });
}

/// Align (`left`, `center`, `right`, `top`, `middle`, `bottom`) or distribute
/// (`horizontal`, `vertical`) the selection, as PowerPoint's Arrange > Align: one
/// shape aligns to the slide, several to the box round them all. Boxes are the
/// shapes' own, unrotated, as the engine reports them.
function arrange(how) {
  const all = chosen().filter((s) => !(s.flags & NO_MOVE));
  if (!all.length) return;
  const toSlide = chosen().length === 1;
  const left = toSlide ? 0 : Math.min(...all.map((s) => s.x));
  const top = toSlide ? 0 : Math.min(...all.map((s) => s.y));
  const right = toSlide ? state.deck.width : Math.max(...all.map((s) => s.x + s.cx));
  const bottom = toSlide ? state.deck.height : Math.max(...all.map((s) => s.y + s.cy));
  let out;
  if (how === "horizontal" || how === "vertical") {
    const h = how === "horizontal";
    const sorted = [...all].sort((a, b) => (h ? a.x - b.x : a.y - b.y));
    const span = h ? right - left : bottom - top;
    const used = sorted.reduce((n, s) => n + (h ? s.cx : s.cy), 0);
    const gap = sorted.length > 1 ? (span - used) / (sorted.length - 1) : 0;
    let at = h ? left : top;
    out = sorted.map((s) => {
      const b = h ? { ...s, x: at } : { ...s, y: at };
      at += (h ? s.cx : s.cy) + gap;
      return b;
    });
  } else {
    out = all.map((s) => ({
      ...s,
      x: { left, center: (left + right) / 2 - s.cx / 2, right: right - s.cx }[how] ?? s.x,
      y: { top, middle: (top + bottom) / 2 - s.cy / 2, bottom: bottom - s.cy }[how] ?? s.y,
    }));
  }
  out = out.filter((b) => {
    const s = all.find((x) => x.node === b.node);
    return Math.round(s.x) !== Math.round(b.x) || Math.round(s.y) !== Math.round(b.y);
  });
  if (!out.length) { status("Already aligned"); return; }
  if (out.length === 1 && chosen().length === 1) commit(out[0]); else commitMany(out);
}

/* ── Selection ──────────────────────────────────────────────────────────── */

function select(node) {
  selected = node;
  others = [];
  paint();
  changed();
}

export function hasSelection() { return !!shape(); }
/// Every selected shape's node, the primary last (`UX-003`).
export function selectedNodes() { return chosen().map((s) => s.node); }
/// Every selected shape, the primary last.
export function selectedShapes() { return chosen().map((s) => ({ ...s })); }
/// Select these nodes once the stage's shapes are next read: a paste or duplicate.
export function selectNodes(nodes) { pendingSelect = nodes.length ? [...nodes] : null; }
/// The selected shape's node, or `null` (`VS-10`'s Animations tab acts on it).
export function selectedNode() { return shape()?.node ?? null; }

/// The selected shape's box and flags, or null (`VS-11`: tables and charts read it).
export function selectedShape() { return shape(); }

/// Clear the selection: a press off the slide.
export function deselect() { if (selected != null || others.length) select(null); }

let pendingSelect = null;
/// Select `node` once the stage's shapes are next read: a shape just inserted (`VS-08`).
export function selectNode(node) { pendingSelect = node; }

/// The slide on the stage changed or was redrawn: read its shapes again,
/// keeping the selection when the shape is still there.
export async function refresh() {
  if (!state.deck) return;
  const token = ++shapesToken;
  const index = state.current;
  try {
    const r = await request({ op: "shapes", index }, { priority: 8 });
    if (token !== shapesToken || index !== state.current) return;
    const next = [];
    for (let i = 0; i + 6 < r.shapes.length; i += 7) {
      const [node, x, y, cx, cy, rot, flags] = r.shapes.slice(i, i + 7);
      next.push({ node, x, y, cx, cy, rot: rot / ROT_UNIT, flags });
    }
    // Which shapes hold text, and which are empty, so a press is routed at once.
    await Promise.all(next.map(async (s) => {
      try {
        const h = await request({ op: "text", fn: "text_hit", args: [index, s.x + s.cx / 2, s.y + s.cy / 2] }, { priority: 8 });
        if (h.value.length < 2 || h.value[1] !== s.node) return;
        s.part = h.value[0];
        const n = await request({ op: "text", fn: "text_length", args: [s.part, s.node] }, { priority: 8 });
        s.empty = n.value === 0;
      } catch { /* no text */ }
    }));
    if (token !== shapesToken || index !== state.current) return;
    shapes = next;
    shapesFor = index;
    if (pendingSelect != null) {
      const list = Array.isArray(pendingSelect) ? pendingSelect : [pendingSelect];
      selected = list[list.length - 1];
      others = list.slice(0, -1);
      pendingSelect = null;
    }
    if (!shape()) selected = null;
    others = others.filter((n) => n !== selected && next.some((s) => s.node === n));
    paint();
    changed();
    replayPress();
  } catch {
    shapes = [];
    pendingPress = null;
    text.release(false);
  }
}

/// A new deck, or another slide: nothing is selected.
export function reset(deckChanged) {
  selected = null;
  others = [];
  drag = null;
  dropGhost();
  shapes = [];
  shapesFor = null;
  if (deckChanged) undoState = [false, false];
  paint();
}

/// Keys while the stage has a selection. Returns true when the key was used.
export function onKey(e) {
  if (text.editing()) return false;
  const mod = e.ctrlKey || e.metaKey;
  if (mod && !e.altKey && (e.key === "z" || e.key === "Z") && !e.shiftKey) { e.preventDefault(); run("edit.undo"); return true; }
  if (mod && !e.altKey && (e.key === "y" || e.key === "Y" || ((e.key === "z" || e.key === "Z") && e.shiftKey))) { e.preventDefault(); run("edit.redo"); return true; }
  if (!shape() || e.altKey) return false;
  if (e.key === "Delete" || e.key === "Backspace") { e.preventDefault(); run("shape.delete"); return true; }
  if (e.key === "Escape") { e.preventDefault(); select(null); return true; }
  if (mod && !e.shiftKey && (e.key === "d" || e.key === "D")) { e.preventDefault(); run("edit.duplicate"); return true; }
  // A key typed with a text shape selected starts typing in it, as PowerPoint.
  const s = shape();
  if (s.part != null && e.key.length === 1 && !mod && chosen().length === 1) {
    e.preventDefault();
    text.typeInto(s.part, s.node, e.key);
    return true;
  }
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
  register("shape.delete", { label: "Delete", shortcut: "Delete", enabled: () => has() && !!shape(), run: deleteSelection });
  const any = () => has() && !!shape();
  for (const [how, label] of [["left", "Align Left"], ["center", "Align Center"], ["right", "Align Right"], ["top", "Align Top"], ["middle", "Align Middle"], ["bottom", "Align Bottom"]]) {
    register(`arrange.align-${how}`, { label, enabled: any, run: () => arrange(how) });
  }
  const several = () => has() && chosen().length > 2;
  register("arrange.distribute-horizontal", { label: "Distribute Horizontally", enabled: several, run: () => arrange("horizontal") });
  register("arrange.distribute-vertical", { label: "Distribute Vertically", enabled: several, run: () => arrange("vertical") });
  register("edit.select-all", {
    label: "Select All", shortcut: "Ctrl+A",
    enabled: () => has() && shapes.length > 0 && !text.editing(),
    run: () => {
      selected = shapes.length ? shapes[shapes.length - 1].node : null;
      others = shapes.slice(0, -1).map((s) => s.node);
      paint();
      changed();
    },
  });
  for (const [id, label, dx, dy] of [
    ["shape.nudge-left", "Nudge Left", -NUDGE, 0], ["shape.nudge-right", "Nudge Right", NUDGE, 0],
    ["shape.nudge-up", "Nudge Up", 0, -NUDGE], ["shape.nudge-down", "Nudge Down", 0, NUDGE],
  ]) {
    register(id, { label, enabled: () => has() && !!shape() && !(shape().flags & NO_MOVE), run: () => nudge(dx, dy) });
  }
  window.addEventListener("resize", () => paint());
}

/// A text shape's emptiness changed with an edit: its prompt shows or goes.
export function noteEmpty(node, empty) {
  const s = shapes.find((x) => x.node === node);
  if (s) s.empty = empty;
}

export { paint as repaint };
