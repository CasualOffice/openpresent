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
//
// Groups (`UX-015`), as PowerPoint: a first click selects the group, a second
// selects the shape inside it, or puts the caret in its text, and typing with that
// shape selected types into it. The engine gives each shape inside a group its box
// on the slide, and maps a box sent back into the group's own coordinates.
//
// Smart guides (`PAR-15`): while shapes move, their left, centre and right edges and
// their top, middle and bottom snap to another shape's, or to the slide's edges and
// centre lines, when within a few pixels, and a red dashed line shows each match, as
// PowerPoint's. Alt held during the drag turns snapping off.

import { state, request, status, changed } from "./editor.core.js";
import { register, run } from "./commands.js";
import * as slides from "./editor.slides.js";
import { errorDialog } from "./editor.dialogs.js";
import * as text from "./editor.text.js";

const byId = (id) => document.getElementById(id);

// `ShapeBox`'s flag bits in `casual-slide-wasm`.
const NO_MOVE = 4, NO_RESIZE = 8, NO_ROTATE = 16, KEEP_ASPECT = 32, GROUP = 64, TABLE = 256;
const PLACEHOLDER = 128;              // `ShapeBox::PLACEHOLDER`: only these prompt
const ROT_UNIT = 60000;               // `@rot` is in 60000ths of a degree
const NUDGE = 76200;                  // PowerPoint's grid step, 1/12 inch, in EMU
const NUDGE_FINE = 12700;             // one point, with Ctrl
const MIN_EXTENT = 12700;             // the smallest a drag makes a side, one point
const HANDLES = [[-1, -1], [0, -1], [1, -1], [1, 0], [1, 1], [0, 1], [-1, 1], [-1, 0]];
const BORDER_PX = 6;                  // how far inside its edge a press still grabs a text shape
const SNAP_PX = 6;                    // how near, in screen pixels, a smart guide pulls

let shapes = [];        // [{ node, x, y, cx, cy, rot (degrees), flags, part?, empty? }]
let selected = null;    // the primary node: handles, the Format pane and the panes read it
let others = [];        // the rest of a multiple selection, nodes
let drag = null;        // { kind, start, box, handle?, moved }
let undoState = [false, false];
let shapesToken = 0;
let shapesFor = null;   // the slide index `shapes` was read for; null until read
let pendingPress = null; // a press made before the shapes were read: { p, shift }
/// The groups a click has gone into, outermost first, each with the shapes directly
/// inside it as the engine placed them on the slide (`UX-015`): a press inside the
/// group picks among these.
let levels = [];        // [{ group, kids: [shape] }]

/// A selectable shape by node: one at the top of the slide, or inside a group gone into.
function known(node) {
  const top = shapes.find((s) => s.node === node);
  if (top) return top;
  for (const l of levels) {
    const k = l.kids.find((s) => s.node === node);
    if (k) return k;
  }
  return null;
}
const shape = () => (selected == null ? null : known(selected));
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

function hit([px, py], list = shapes) {
  // Topmost first: the last shape in z-order is drawn on top. A line has no
  // width, so every box is grown by a few pixels to be clickable.
  const slop = 4 / scale();
  for (let i = list.length - 1; i >= 0; i--) {
    const s = list[i];
    if (covers(s, [px, py], slop)) return s;
  }
  return null;
}

/// Whether a press at `p` lands on `s`. A group is pressed only where a shape inside
/// it is, as PowerPoint's: the empty parts of its box let the press through to the
/// shapes under it (`UX-015`).
function covers(s, p, slop, depth = 0) {
  const [lx, ly] = local(s, p);
  if (Math.abs(lx) > s.cx / 2 + slop || Math.abs(ly) > s.cy / 2 + slop) return false;
  if (!(s.flags & GROUP) || !s.kids?.length || depth > 16) return true;
  return s.kids.some((k) => covers(k, p, slop, depth + 1));
}

/// The shape a press at `p` picks, `top` being the slide's own shape there: inside
/// each group already gone into, the shape under the press, as PowerPoint's second
/// click; `depth` is how many groups it is inside.
function deepHit(top, p) {
  let found = top, depth = 0;
  while (found && depth < levels.length && levels[depth].group === found.node) {
    const kid = hit(p, levels[depth].kids);
    if (!kid) break;
    found = kid;
    depth++;
  }
  return { found, depth };
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

/* ── Smart guides (`PAR-15`) ────────────────────────────────────────────── */

/// The upright box a shape covers, rotation included: `[left, top, right, bottom]`.
function bounds(s) {
  if (!s.rot) return [s.x, s.y, s.x + s.cx, s.y + s.cy];
  const r = (s.rot * Math.PI) / 180;
  const w = Math.abs(s.cx * Math.cos(r)) + Math.abs(s.cy * Math.sin(r));
  const h = Math.abs(s.cx * Math.sin(r)) + Math.abs(s.cy * Math.cos(r));
  const mx = s.x + s.cx / 2, my = s.y + s.cy / 2;
  return [mx - w / 2, my - h / 2, mx + w / 2, my + h / 2];
}

/// Snap a move of the shapes `moving` by `[dx, dy]` to the other shapes' and the
/// slide's edges and centres: the move it becomes, and the guides to draw, each
/// `{ v: true, at, from, to }` (a vertical line at x `at`) or the horizontal kind.
export function snapMove(moving, others, [dx, dy], [width, height], tolerance) {
  if (!moving.length) return { dx, dy, guides: [] };
  const all = moving.map(bounds);
  const box = [Math.min(...all.map((b) => b[0])) + dx, Math.min(...all.map((b) => b[1])) + dy,
    Math.max(...all.map((b) => b[2])) + dx, Math.max(...all.map((b) => b[3])) + dy];
  // Each target line: its position and the extent along it that a guide spans.
  const xs = [[0, 0, height], [width / 2, 0, height], [width, 0, height]];
  const ys = [[0, 0, width], [height / 2, 0, width], [height, 0, width]];
  for (const o of others) {
    const [l, t, r, b] = bounds(o);
    for (const x of [l, (l + r) / 2, r]) xs.push([x, t, b]);
    for (const y of [t, (t + b) / 2, b]) ys.push([y, l, r]);
  }
  const best = (edges, lines) => {
    let pull = null;
    for (const e of edges) for (const [at] of lines) {
      const d = at - e;
      if (Math.abs(d) <= tolerance && (pull === null || Math.abs(d) < Math.abs(pull))) pull = d;
    }
    return pull;
  };
  const sx = best([box[0], (box[0] + box[2]) / 2, box[2]], xs);
  const sy = best([box[1], (box[1] + box[3]) / 2, box[3]], ys);
  if (sx !== null) { dx += sx; box[0] += sx; box[2] += sx; }
  if (sy !== null) { dy += sy; box[1] += sy; box[3] += sy; }
  const guides = [];
  const near = (a, b) => Math.abs(a - b) < 0.5;
  if (sx !== null) {
    for (const e of [box[0], (box[0] + box[2]) / 2, box[2]]) for (const [at, from, to] of xs) {
      if (near(at, e)) guides.push({ v: true, at, from: Math.min(from, box[1]), to: Math.max(to, box[3]) });
    }
  }
  if (sy !== null) {
    for (const e of [box[1], (box[1] + box[3]) / 2, box[3]]) for (const [at, from, to] of ys) {
      if (near(at, e)) guides.push({ v: false, at, from: Math.min(from, box[0]), to: Math.max(to, box[2]) });
    }
  }
  return { dx, dy, guides };
}

/// A move of `origins` by `[dx, dy]` as the pointer gives it, snapped unless Alt is down.
function snapped(origins, dx, dy, e) {
  if (e.altKey) return { dx, dy, guides: [] };
  const moving = new Set(origins.map((o) => o.node));
  const rest = shapes.filter((s) => !moving.has(s.node));
  return snapMove(origins, rest, [dx, dy], [state.deck.width, state.deck.height], SNAP_PX / scale());
}

function paintGuides(l, guides, k) {
  for (const g of guides || []) {
    const d = document.createElement("div");
    d.className = g.v ? "sel-guide is-v" : "sel-guide is-h";
    if (g.v) {
      d.style.left = `${g.at * k}px`;
      d.style.top = `${g.from * k}px`;
      d.style.height = `${(g.to - g.from) * k}px`;
    } else {
      d.style.top = `${g.at * k}px`;
      d.style.left = `${g.from * k}px`;
      d.style.width = `${(g.to - g.from) * k}px`;
    }
    l.appendChild(d);
  }
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

/// Who opens a table's cell on a press inside it (`editor.objects.js`).
let tablePress = null;
export function onTablePress(f) { tablePress = f; }

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
  if (drag?.moved && drag.guides) paintGuides(l, drag.guides, k);
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
  // A shape inside a group: the group shows as a dashed box round it, as PowerPoint's.
  for (const g of levels) {
    const o = known(g.group);
    if (!o || o.node === b.node) continue;
    const d = document.createElement("div");
    d.className = "sel-box is-group";
    d.style.left = `${o.x * k}px`;
    d.style.top = `${o.y * k}px`;
    d.style.width = `${o.cx * k}px`;
    d.style.height = `${o.cy * k}px`;
    d.style.transform = `rotate(${o.rot}deg)`;
    l.appendChild(d);
  }
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
  const top = hit(p);
  if (add && !text.editing()) {
    const found = top;
    text.stop();
    if (found) toggleInSelection(found.node);
    else drag = { kind: "marquee", start: p, at: p, add: true };
    byId("stage").focus({ preventScroll: true });
    return false;
  }
  // A press on a shape of a multiple selection moves them all.
  if (top && chosen().length > 1 && chosen().some((s) => s.node === top.node)) {
    const found = top;
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
  // Inside a group already selected, the shape under the press (`UX-015`).
  const { found, depth } = deepHit(top, p);
  levels = levels.slice(0, depth);
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
  if (found && found.flags & GROUP) enterGroup(found);
  // A press inside a table, off its border, types in the cell under it, as
  // PowerPoint's; it selected the table only, and typing went nowhere (`UX-015`).
  if (found && found.flags & TABLE && tablePress && interior(found, p)) {
    tablePress(found, p);
    return false;
  }
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
    const s = deepHit(hit(p), p).found;
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
    const movable = drag.origins.filter((o) => !(o.flags & NO_MOVE));
    ({ dx, dy, guides: drag.guides } = snapped(movable, dx, dy, e));
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
    ({ dx, dy, guides: drag.guides } = snapped([o], dx, dy, e));
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

/// Undo once, as Undo does, and resolve when the slide is drawn again: Paste Special
/// takes a paste back this way to paste it another way (`PAR-11`).
export const undoStep = () => history("undo");

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
  if (node == null) levels = [];
  paint();
  changed();
}

export function hasSelection() { return !!shape(); }
/// The stage's shapes as last read, and the slide they were read for (`COL-04`:
/// another participant's selection is outlined from them).
export function allShapes() { return shapesFor === state.current ? shapes : []; }
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

/// A right-click at `e`, before its menu opens: a shape not selected is selected
/// first, as PowerPoint does, so the menu acts on what was clicked; one already
/// selected keeps the selection it is part of, and text being typed in stays so.
/// A right-click on an empty part of the slide clears the selection (`UX-003`).
export function selectForMenu(e) {
  if (!state.deck || shapesFor !== state.current) return;
  const p = pointer(e);
  const { found, depth } = deepHit(hit(p), p);
  if (found && chosen().some((s) => s.node === found.node)) return;
  if (!found && text.editing()) return;
  text.stop();
  levels = levels.slice(0, depth);
  select(found ? found.node : null);
  if (found && found.flags & GROUP) enterGroup(found);
  byId("stage").focus({ preventScroll: true });
}

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
    const next = await boxes(r.shapes, index);
    // The groups gone into, as read again: a shape inside may have moved or gone.
    const inside = [];
    for (const l of levels) {
      const parent = inside.length ? inside[inside.length - 1].kids : next;
      const g = parent.find((s) => s.node === l.group);
      if (!g) break;
      inside.push({ group: l.group, kids: g.kids || [] });
    }
    if (token !== shapesToken || index !== state.current) return;
    shapes = next;
    levels = inside;
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

/// Shapes from the engine's seven numbers each, with which hold text and which are
/// empty, so a press is routed at once.
async function boxes(flat, index, depth = 0) {
  const out = [];
  for (let i = 0; i + 6 < flat.length; i += 7) {
    const [node, x, y, cx, cy, rot, flags] = flat.slice(i, i + 7);
    out.push({ node, x, y, cx, cy, rot: rot / ROT_UNIT, flags });
  }
  await Promise.all(out.map(async (s) => {
    // What a group holds, so a press is routed through its empty parts at once.
    if (s.flags & GROUP) { s.kids = depth < 16 ? await kidsOf(index, s.node, depth + 1) : []; return; }
    try {
      const h = await request({ op: "text", fn: "text_hit", args: [index, s.x + s.cx / 2, s.y + s.cy / 2] }, { priority: 8 });
      if (h.value.length < 2 || h.value[1] !== s.node) return;
      s.part = h.value[0];
      const n = await request({ op: "text", fn: "text_length", args: [s.part, s.node] }, { priority: 8 });
      s.empty = n.value === 0;
    } catch { /* no text */ }
  }));
  return out;
}

/// The shapes directly inside group `group` of slide `index`, boxes on the slide.
async function kidsOf(index, group, depth) {
  try {
    const r = await request({ op: "build", fn: "group_shapes", args: [index, group] }, { priority: 8 });
    return await boxes(r.value, index, depth);
  } catch {
    return [];
  }
}

/// Group `g` was selected: the next press inside it picks a shape inside (`UX-015`).
function enterGroup(g) {
  if (!levels.some((l) => l.group === g.node)) levels.push({ group: g.node, kids: g.kids || [] });
}

/// A new deck, or another slide: nothing is selected.
export function reset(deckChanged) {
  selected = null;
  others = [];
  levels = [];
  drag = null;
  dropGhost();
  shapes = [];
  shapesFor = null;
  if (deckChanged) undoState = [false, false];
  paint();
}

/// Keys while the stage has a selection. Returns true when the key was used.
export function onKey(e) {
  // A key typed just after a click into text, while the box opens, is the box's: taken
  // here, it typed at the text's end and dropped the click's caret (`UX-015`).
  if (text.ownsKeys()) return false;
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
