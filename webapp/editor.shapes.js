// PowerPoint's shape experience (`SHAPE-01`): Insert > Shapes opens the gallery with
// every preset the engine has, in PowerPoint's groups, each icon drawn from our own
// geometry engine; a pick arms a drawing tool (drag to size, a click for the default
// size, Shift for proportions or 45° lines, Esc to cancel; lines run point to point);
// a selected shape shows its yellow adjust handles, which drag live; and the Shape
// Format tab sets Shape Fill, Shape Outline and a theme Shape Style.
//
// Every verb is a registered command and every change one engine op with its
// inverse, so Undo and Redo are the shared `edit.undo`/`edit.redo`. Nothing here
// reads a document string: names and tooltips are the catalogue's own, and icons
// are path data the engine computed, set as an attribute, never as markup.

import { register } from "./commands.js";
import { state, request, status, changed } from "./editor.core.js";
import { TABS } from "./ribbon.model.js";
import { selectTab, queueSync } from "./ribbon.js";
import { errorDialog } from "./editor.dialogs.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";
import { withOthers } from "./shapes.catalog.js";

const byId = (id) => document.getElementById(id);
const SVG = "http://www.w3.org/2000/svg";
const EMU_PER_INCH = 914400;
const ICON_UNITS = 32;            // the size icons are evaluated at
const RECENT_KEY = "op.shapes.recent";
const RECENT_MAX = 12;
const GROUP_BIT = 64;             // `ShapeBox::GROUP`
const CONNECTORS = new Set(["line", "lineInv", "straightConnector1", "bentConnector2", "bentConnector3",
  "bentConnector4", "bentConnector5", "curvedConnector2", "curvedConnector3", "curvedConnector4", "curvedConnector5"]);

let catalogue = null;             // { groups, icons: Map(prst → path data) }
let tool = null;                  // the armed shape: { prst, name, ends }
let sel = null;                   // the selected shape, as `editor.selection.js` paints it

function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

const hasSlide = () => !!state.deck && state.deck.count > 0;
const formattable = () => hasSlide() && !!sel && !(sel.flags & GROUP_BIT);
const isLine = (prst) => CONNECTORS.has(prst);

/// One engine call through the worker's build table; afterwards the undo state.
async function call(fn, args) {
  const r = await request({ op: "build", fn, args }, { priority: 9 });
  selection.noteUndoState(r.undo);
  return r;
}

/* ── The catalogue and its icons ────────────────────────────────────────── */

async function load() {
  if (catalogue) return catalogue;
  const r = await request({ op: "icons", size: ICON_UNITS }, { priority: 9 });
  const icons = new Map();
  r.names.forEach((n, i) => icons.set(n, r.icons[i]));
  catalogue = { groups: withOthers(r.names), icons };
  return catalogue;
}

/// An icon from the engine's path data: one line a path, a fill flag, a stroke flag,
/// then SVG path commands of numbers only.
function iconSvg(data, px = 22) {
  const svg = document.createElementNS(SVG, "svg");
  svg.setAttribute("viewBox", `-2 -2 ${ICON_UNITS + 4} ${ICON_UNITS + 4}`);
  svg.setAttribute("width", String(px));
  svg.setAttribute("height", String(px));
  svg.setAttribute("aria-hidden", "true");
  for (const line of (data || "").split("\n")) {
    if (line.length < 3) continue;
    const d = line.slice(3);
    if (!/^[MLQCZ0-9 .\-]*$/.test(d)) continue;
    const p = document.createElementNS(SVG, "path");
    p.setAttribute("d", d);
    const fill = line[0], stroke = line[1] === "s";
    p.setAttribute("fill", fill === "n" ? "none" : "currentColor");
    p.setAttribute("fill-opacity", { f: "0.12", l: "0.04", L: "0.08", d: "0.4", D: "0.25" }[fill] || "0");
    p.setAttribute("stroke", stroke ? "currentColor" : "none");
    p.setAttribute("stroke-width", "1.4");
    p.setAttribute("vector-effect", "non-scaling-stroke");
    p.setAttribute("stroke-linejoin", "round");
    svg.appendChild(p);
  }
  return svg;
}

function recent() {
  try {
    const v = JSON.parse(localStorage.getItem(RECENT_KEY) || "[]");
    return Array.isArray(v) ? v.filter((i) => i && typeof i.prst === "string" && typeof i.name === "string").slice(0, RECENT_MAX) : [];
  } catch {
    return [];
  }
}

function remember(item) {
  const list = [item, ...recent().filter((i) => !(i.prst === item.prst && i.name === item.name))].slice(0, RECENT_MAX);
  try { localStorage.setItem(RECENT_KEY, JSON.stringify(list.map(({ prst, name, ends }) => ({ prst, name, ends: ends | 0 })))); } catch { /* private mode */ }
}

/* ── Popovers ───────────────────────────────────────────────────────────── */

let pop = null;

function closePop() {
  if (!pop) return;
  pop.remove();
  pop = null;
  document.removeEventListener("pointerdown", outside, true);
  document.removeEventListener("keydown", popKey, true);
}

function outside(e) { if (pop && !pop.contains(e.target)) closePop(); }
function popKey(e) { if (e.key === "Escape") { e.stopPropagation(); closePop(); } }

/// The button that ran command `id`, to drop a popover from; else the stage.
function anchorFor(ids) {
  for (const id of ids) {
    for (const b of document.querySelectorAll(`.op-ribbon [data-cmd="${CSS.escape(id)}"]`)) {
      const r = b.getBoundingClientRect();
      if (r.width && r.height) return r;
    }
  }
  return byId("stage").getBoundingClientRect();
}

function openPop(ids, cls, build) {
  closePop();
  pop = el("div", `shp-pop ${cls}`);
  pop.setAttribute("role", "dialog");
  build(pop);
  document.body.appendChild(pop);
  const r = anchorFor(ids);
  const w = pop.offsetWidth, h = pop.offsetHeight;
  const left = Math.max(4, Math.min(r.left, window.innerWidth - 4 - w));
  let top = r.bottom + 2;
  if (top + h > window.innerHeight - 4) top = Math.max(4, window.innerHeight - 4 - h);
  pop.style.left = `${left}px`;
  pop.style.top = `${top}px`;
  setTimeout(() => {
    document.addEventListener("pointerdown", outside, true);
    document.addEventListener("keydown", popKey, true);
  }, 0);
  pop.querySelector("button")?.focus({ preventScroll: true });
}

/* ── Insert > Shapes ────────────────────────────────────────────────────── */

async function gallery() {
  if (!hasSlide()) return;
  let cat;
  try {
    cat = await load();
  } catch (e) {
    errorDialog("OpenPresent could not list the shapes.", e.message);
    return;
  }
  openPop(["insert.shapes", "format.insert-shapes"], "shp-gallery", (p) => {
    const groups = [];
    const used = recent();
    if (used.length) groups.push({ name: "Recently Used Shapes", items: used });
    groups.push(...cat.groups);
    for (const grp of groups) {
      p.appendChild(el("div", "shp-head", grp.name));
      const grid = el("div", "shp-grid");
      grid.setAttribute("role", "group");
      grid.setAttribute("aria-label", grp.name);
      for (const item of grp.items) {
        const b = el("button", "shp-item");
        b.type = "button";
        b.title = item.name;
        b.setAttribute("aria-label", item.name);
        b.appendChild(iconSvg(cat.icons.get(item.prst)));
        if (item.ends) {
          // An arrowhead mark, as the gallery's Line Arrow icons have one.
          b.classList.add(item.ends === 2 ? "shp-ends-2" : "shp-ends-1");
        }
        b.addEventListener("click", () => { closePop(); arm(item); });
        grid.appendChild(b);
      }
      p.appendChild(grid);
    }
  });
}

/* ── Drawing ────────────────────────────────────────────────────────────── */

let drawLayer = null;
let drawing = null;   // { start: [x, y] EMU, end, id }

const scale = () => byId("slide-frame").clientWidth / state.deck.width;

function pointer(e) {
  const r = byId("slide-frame").getBoundingClientRect();
  const k = scale();
  return [(e.clientX - r.left) / k, (e.clientY - r.top) / k];
}

function arm(item) {
  if (!hasSlide()) return;
  disarm();
  tool = item;
  const frame = byId("slide-frame");
  drawLayer = el("div", "shp-draw");
  drawLayer.addEventListener("pointerdown", onDrawDown);
  drawLayer.addEventListener("pointermove", onDrawMove);
  drawLayer.addEventListener("pointerup", onDrawUp);
  drawLayer.addEventListener("pointercancel", () => disarm());
  frame.appendChild(drawLayer);
  document.addEventListener("keydown", drawKey, true);
  status(`${item.name}: drag on the slide to draw it, or click for the default size. Esc cancels.`);
}

function disarm() {
  tool = null;
  drawing = null;
  drawLayer?.remove();
  drawLayer = null;
  document.removeEventListener("keydown", drawKey, true);
}

function drawKey(e) {
  if (e.key === "Escape" && tool) {
    e.preventDefault();
    e.stopPropagation();
    disarm();
    status("Ready");
  }
}

/// The box and flips a drag from `a` to `b` gives, Shift applied.
function dragBox(a, b, shift, line) {
  let dx = b[0] - a[0], dy = b[1] - a[1];
  if (shift) {
    if (line) {
      // Lines snap to every 45°.
      const ang = Math.round(Math.atan2(dy, dx) / (Math.PI / 4)) * (Math.PI / 4);
      const len = Math.hypot(dx, dy);
      dx = Math.round(Math.cos(ang) * len);
      dy = Math.round(Math.sin(ang) * len);
    } else {
      const s = Math.max(Math.abs(dx), Math.abs(dy));
      dx = Math.sign(dx || 1) * s;
      dy = Math.sign(dy || 1) * s;
    }
  }
  return {
    x: Math.min(a[0], a[0] + dx), y: Math.min(a[1], a[1] + dy),
    cx: Math.abs(dx), cy: Math.abs(dy),
    flipH: dx < 0, flipV: dy < 0,
  };
}

function onDrawDown(e) {
  if (e.button !== 0 || !tool) return;
  e.preventDefault();
  e.stopPropagation();
  drawLayer.setPointerCapture(e.pointerId);
  const p = pointer(e);
  drawing = { start: p, end: p, px: [e.clientX, e.clientY] };
}

function onDrawMove(e) {
  if (!drawing || !tool) return;
  drawing.end = pointer(e);
  drawing.shift = e.shiftKey;
  preview();
}

function preview() {
  drawLayer.textContent = "";
  const line = isLine(tool.prst);
  const b = dragBox(drawing.start, drawing.end, drawing.shift, line);
  const k = scale();
  const svg = document.createElementNS(SVG, "svg");
  svg.setAttribute("class", "shp-preview");
  svg.style.left = `${b.x * k}px`;
  svg.style.top = `${b.y * k}px`;
  const w = Math.max(1, b.cx * k), h = Math.max(1, b.cy * k);
  svg.setAttribute("width", String(w));
  svg.setAttribute("height", String(h));
  svg.style.overflow = "visible";
  if (line) {
    const l = document.createElementNS(SVG, "line");
    l.setAttribute("x1", b.flipH ? String(w) : "0");
    l.setAttribute("y1", b.flipV ? String(h) : "0");
    l.setAttribute("x2", b.flipH ? "0" : String(w));
    l.setAttribute("y2", b.flipV ? "0" : String(h));
    svg.appendChild(l);
  } else {
    const icon = iconSvg(catalogue?.icons.get(tool.prst), 0);
    icon.setAttribute("viewBox", `0 0 ${ICON_UNITS} ${ICON_UNITS}`);
    icon.setAttribute("preserveAspectRatio", "none");
    icon.setAttribute("width", String(w));
    icon.setAttribute("height", String(h));
    if (b.flipH || b.flipV) icon.style.transform = `scale(${b.flipH ? -1 : 1}, ${b.flipV ? -1 : 1})`;
    svg.appendChild(icon);
  }
  drawLayer.appendChild(svg);
}

async function onDrawUp(e) {
  if (!drawing || !tool) return;
  const item = tool;
  const d = drawing;
  d.end = pointer(e);
  const line = isLine(item.prst);
  const moved = Math.hypot(e.clientX - d.px[0], e.clientY - d.px[1]) > 3;
  let b;
  if (moved) {
    b = dragBox(d.start, d.end, e.shiftKey, line);
  } else {
    // A click alone: PowerPoint's default size, one inch, from the point clicked.
    b = line
      ? { x: d.start[0], y: d.start[1], cx: EMU_PER_INCH, cy: 0, flipH: false, flipV: false }
      : { x: d.start[0], y: d.start[1], cx: EMU_PER_INCH, cy: EMU_PER_INCH, flipH: false, flipV: false };
  }
  disarm();
  await insert(item, b);
}

async function insert(item, b) {
  const index = state.current;
  const flags = (b.flipH ? 1 : 0) | (b.flipV ? 2 : 0) | ((item.ends | 0) << 2);
  try {
    const r = await call("insert_preset", [index, item.prst, item.name,
      Math.round(b.x), Math.round(b.y), Math.round(b.cx), Math.round(b.cy), flags]);
    remember(item);
    selection.selectNode(r.value);
    await slides.redrawSlide(index);
    wantFormatTab = true;   // once the new shape is selected, as PowerPoint opens it
    status(`${item.name} inserted`);
  } catch (e) {
    errorDialog("OpenPresent could not insert that shape.", e.message);
  }
}

/* ── The yellow adjust handles ──────────────────────────────────────────── */

let adjLayer = null;
let adjToken = 0;
let gesture = null;   // { index, node, handle, first, pending, busy, el }
let wantFormatTab = false;

function adjustLayer() {
  if (!adjLayer || !adjLayer.isConnected) {
    adjLayer = el("div", "shp-adj-layer");
    byId("slide-frame").appendChild(adjLayer);
  }
  return adjLayer;
}

async function paintHandles(s) {
  sel = s;
  queueSync();
  if (s && wantFormatTab) {
    wantFormatTab = false;
    // After the ribbon's next sync has shown the contextual tab.
    requestAnimationFrame(() => requestAnimationFrame(() => selectTab("shape-format")));
  }
  if (gesture) return;
  const l = adjustLayer();
  const token = ++adjToken;
  if (!s || state.reading || tool) { l.textContent = ""; return; }
  let pts;
  try {
    pts = (await request({ op: "build", fn: "shape_handles", args: [s.index, s.node] }, { priority: 8 })).value;
  } catch {
    pts = [];
  }
  if (token !== adjToken || gesture) return;
  l.textContent = "";
  const k = scale();
  for (let i = 0; i + 1 < pts.length; i += 2) {
    const h = el("div", "shp-adj");
    h.style.left = `${pts[i] * k}px`;
    h.style.top = `${pts[i + 1] * k}px`;
    h.title = "Drag to change the shape";
    h.dataset.handle = String(i / 2);
    h.addEventListener("pointerdown", (e) => startAdjust(e, s, i / 2, h));
    l.appendChild(h);
  }
}

function startAdjust(e, s, handle, h) {
  if (e.button !== 0) return;
  e.preventDefault();
  e.stopPropagation();
  h.setPointerCapture(e.pointerId);
  gesture = { index: s.index, node: s.node, handle, first: true, pending: null, busy: false, el: h };
  h.classList.add("is-dragging");
  h.addEventListener("pointermove", moveAdjust);
  h.addEventListener("pointerup", endAdjust);
  h.addEventListener("pointercancel", endAdjust);
}

function moveAdjust(e) {
  if (!gesture) return;
  const k = scale();
  const p = pointer(e);
  gesture.el.style.left = `${p[0] * k}px`;
  gesture.el.style.top = `${p[1] * k}px`;
  gesture.pending = p;
  pump();
}

/// Send the latest point; one in flight at a time, so the engine never falls behind.
/// Once the pointer is up and the last point sent, the gesture ends as one undo step.
async function pump() {
  const g = gesture;
  if (!g || g.busy) return;
  if (g.pending) {
    const [x, y] = g.pending;
    g.pending = null;
    g.busy = true;
    try {
      await call("drag_shape_handle", [g.index, g.node, g.handle, x, y, g.first]);
      g.first = false;
      if (g.index === state.current && !g.ending) await slides.redraw();
    } catch {
      status("That handle cannot move there");
    } finally {
      g.busy = false;
    }
    pump();
  } else if (g.ending) {
    gesture = null;
    try {
      await call("edit_break_typing", []);
    } catch { /* nothing to end */ }
    await slides.redrawSlide(g.index);
    changed();
  }
}

function endAdjust(e) {
  const g = gesture;
  if (!g || g.ending) return;
  g.el.removeEventListener("pointermove", moveAdjust);
  if (e.type === "pointerup") g.pending = pointer(e);
  g.ending = true;
  pump();
}

/* ── Shape Format: colours ──────────────────────────────────────────────── */

const SLOT_NAMES = ["Background 1", "Text 1", "Background 2", "Text 2",
  "Accent 1", "Accent 2", "Accent 3", "Accent 4", "Accent 5", "Accent 6"];
const SLOT_TOKENS = ["bg1", "tx1", "bg2", "tx2", "accent1", "accent2", "accent3", "accent4", "accent5", "accent6"];
const STANDARD = [["C00000", "Dark Red"], ["FF0000", "Red"], ["FFC000", "Orange"], ["FFFF00", "Yellow"],
  ["92D050", "Light Green"], ["00B050", "Green"], ["00B0F0", "Light Blue"], ["0070C0", "Blue"],
  ["002060", "Dark Blue"], ["7030A0", "Purple"]];

function hexToHsl(hex) {
  const n = parseInt(hex, 16);
  const r = ((n >> 16) & 255) / 255, g = ((n >> 8) & 255) / 255, b = (n & 255) / 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  const l = (max + min) / 2;
  if (max === min) return [0, 0, l];
  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
  let h = max === r ? (g - b) / d + (g < b ? 6 : 0) : max === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [h / 6, s, l];
}

function hslToCss([h, s, l]) {
  return `hsl(${Math.round(h * 360)} ${Math.round(s * 100)}% ${Math.round(Math.max(0, Math.min(1, l)) * 100)}%)`;
}

/// PowerPoint's five tints and shades under a theme colour, by how light it is.
function variations(hex) {
  const l = hexToHsl(hex || "808080")[2];
  if (l >= 0.99) return [[95000, 0, "Darker 5%"], [85000, 0, "Darker 15%"], [75000, 0, "Darker 25%"], [65000, 0, "Darker 35%"], [50000, 0, "Darker 50%"]];
  if (l <= 0.01) return [[50000, 50000, "Lighter 50%"], [65000, 35000, "Lighter 35%"], [75000, 25000, "Lighter 25%"], [85000, 15000, "Lighter 15%"], [95000, 5000, "Lighter 5%"]];
  if (l > 0.8) return [[90000, 0, "Darker 10%"], [75000, 0, "Darker 25%"], [50000, 0, "Darker 50%"], [25000, 0, "Darker 75%"], [10000, 0, "Darker 90%"]];
  if (l < 0.2) return [[10000, 90000, "Lighter 90%"], [25000, 75000, "Lighter 75%"], [50000, 50000, "Lighter 50%"], [75000, 25000, "Lighter 25%"], [90000, 10000, "Lighter 10%"]];
  return [[20000, 80000, "Lighter 80%"], [40000, 60000, "Lighter 60%"], [60000, 40000, "Lighter 40%"], [75000, 0, "Darker 25%"], [50000, 0, "Darker 50%"]];
}

async function palette() {
  try {
    return (await request({ op: "build", fn: "slide_palette", args: [state.current] }, { priority: 9 })).value;
  } catch {
    return [];
  }
}

function swatch(css, title, spec, onPick) {
  const b = el("button", "shp-swatch");
  b.type = "button";
  b.title = title;
  b.setAttribute("aria-label", title);
  b.style.background = css;
  b.addEventListener("click", () => { closePop(); onPick(spec); });
  return b;
}

/// PowerPoint's colour menu: Theme Colors with their tints, Standard Colors, a none
/// item and More Colors. `onPick` gets the engine's colour form, or `none`.
function colourMenu(p, colours, noneLabel, onPick) {
  const none = el("button", "shp-menu-item", noneLabel);
  none.type = "button";
  none.addEventListener("click", () => { closePop(); onPick("none"); });
  p.appendChild(el("div", "shp-head", "Theme Colors"));
  const grid = el("div", "shp-colours");
  for (let i = 0; i < SLOT_TOKENS.length; i++) {
    const hex = colours[i] || "";
    grid.appendChild(swatch(hex ? `#${hex}` : "transparent", SLOT_NAMES[i], `scheme ${SLOT_TOKENS[i]}`, onPick));
  }
  const hsl = SLOT_TOKENS.map((_, i) => hexToHsl(colours[i] || "808080"));
  for (let row = 0; row < 5; row++) {
    for (let i = 0; i < SLOT_TOKENS.length; i++) {
      const [mod, off, label] = variations(colours[i])[row];
      const [h, s, l] = hsl[i];
      const spec = `scheme ${SLOT_TOKENS[i]} lumMod ${mod}${off ? ` lumOff ${off}` : ""}`;
      grid.appendChild(swatch(hslToCss([h, s, l * (mod / 100000) + off / 100000]), `${SLOT_NAMES[i]}, ${label}`, spec, onPick));
    }
  }
  p.appendChild(grid);
  p.appendChild(el("div", "shp-head", "Standard Colors"));
  const std = el("div", "shp-colours");
  for (const [hex, name] of STANDARD) std.appendChild(swatch(`#${hex}`, name, `srgb ${hex}`, onPick));
  p.appendChild(std);
  p.appendChild(el("div", "shp-sep"));
  p.appendChild(none);
  const more = el("label", "shp-menu-item shp-more", "More Colors…");
  const input = el("input");
  input.type = "color";
  input.className = "shp-color-input";
  input.addEventListener("change", () => {
    const hex = input.value.replace("#", "").toUpperCase();
    if (/^[0-9A-F]{6}$/.test(hex)) { closePop(); onPick(`srgb ${hex}`); }
  });
  more.appendChild(input);
  p.appendChild(more);
}

/// One Shape Format edit of the selected shape, then the slide drawn again.
async function format(what, fn, args) {
  const s = sel;
  if (!s) return;
  try {
    const r = await call(fn, [s.index, s.node, ...args]);
    if (!r.value) { status(`${what}: nothing changed`); return; }
    await slides.redrawSlide(s.index);
    status(what);
  } catch (e) {
    errorDialog(`OpenPresent could not change that shape: ${what.toLowerCase()}.`, e.message);
  }
}

async function fillMenu() {
  if (!formattable()) return;
  const colours = await palette();
  openPop(["format.shape-fill", "drawing.fill"], "shp-menu", (p) => {
    colourMenu(p, colours, "No Fill", (spec) => format("Shape Fill", "set_shape_fill", [spec]));
  });
}

const WEIGHTS = [0.25, 0.5, 0.75, 1, 1.5, 2.25, 3, 4.5, 6];
const DASHES = [["solid", "Solid"], ["sysDot", "Round Dot"], ["sysDash", "Square Dot"], ["dash", "Dash"],
  ["dashDot", "Dash Dot"], ["lgDash", "Long Dash"], ["lgDashDot", "Long Dash Dot"], ["lgDashDotDot", "Long Dash Dot Dot"]];
const DASH_ARRAYS = { solid: "", sysDot: "1 2", sysDash: "3 2", dash: "6 4", dashDot: "6 3 1 3", lgDash: "12 4", lgDashDot: "12 3 1 3", lgDashDotDot: "12 3 1 3 1 3" };
const ARROWS = [["none", "No Arrowheads"], ["tail", "Arrow at End"], ["head", "Arrow at Start"], ["both", "Arrows at Both Ends"]];

function lineSample(width, dash) {
  const svg = document.createElementNS(SVG, "svg");
  svg.setAttribute("width", "60");
  svg.setAttribute("height", "10");
  svg.setAttribute("aria-hidden", "true");
  const l = document.createElementNS(SVG, "line");
  l.setAttribute("x1", "2"); l.setAttribute("y1", "5"); l.setAttribute("x2", "58"); l.setAttribute("y2", "5");
  l.setAttribute("stroke", "currentColor");
  l.setAttribute("stroke-width", String(Math.max(0.5, Math.min(6, width))));
  if (dash) l.setAttribute("stroke-dasharray", dash);
  svg.appendChild(l);
  return svg;
}

function menuRow(p, label, sample, onClick) {
  const b = el("button", "shp-menu-item");
  b.type = "button";
  if (sample) b.appendChild(sample);
  b.appendChild(el("span", null, label));
  b.addEventListener("click", () => { closePop(); onClick(); });
  p.appendChild(b);
}

async function outlineMenu() {
  if (!formattable()) return;
  const colours = await palette();
  const outline = (what, value, label) => format(label, "set_shape_outline", [what, value]);
  openPop(["format.shape-outline", "drawing.outline"], "shp-menu", (p) => {
    colourMenu(p, colours, "No Outline", (spec) => outline(0, spec, "Shape Outline"));
    p.appendChild(el("div", "shp-head", "Weight"));
    for (const w of WEIGHTS) {
      menuRow(p, `${w} pt`, lineSample(w, ""), () => outline(1, String(Math.round(w * 12700)), "Outline Weight"));
    }
    p.appendChild(el("div", "shp-head", "Dashes"));
    for (const [token, label] of DASHES) {
      menuRow(p, label, lineSample(1.5, DASH_ARRAYS[token]), () => outline(2, token, "Outline Dashes"));
    }
    p.appendChild(el("div", "shp-head", "Arrows"));
    for (const [token, label] of ARROWS) menuRow(p, label, null, () => outline(3, token, "Arrows"));
  });
}

/* ── Shape Format: Shape Styles ─────────────────────────────────────────── */

const STYLE_ROWS = ["Colored Outline", "Colored Fill", "Light 1 Outline, Colored Fill", "Subtle Effect", "Moderate Effect", "Intense Effect"];
const STYLE_COLUMNS = ["Dark 1", "Accent 1", "Accent 2", "Accent 3", "Accent 4", "Accent 5", "Accent 6"];

/// A thumbnail of theme style `row` in colour `hex`, as the gallery shows it: the
/// theme's look, drawn approximately, never a picture of Microsoft's.
function styleThumb(row, hex, light, dark) {
  const t = el("span", "shp-style-thumb", "Abc");
  const c = `#${hex || "808080"}`;
  const [h, s, l] = hexToHsl(hex || "808080");
  const lighter = hslToCss([h, s, Math.min(1, l + (1 - l) * 0.6)]);
  const darker = hslToCss([h, s, l * 0.5]);
  const looks = [
    { background: `#${light}`, border: `2px solid ${c}`, color: `#${dark}` },
    { background: c, border: `1.5px solid ${darker}`, color: `#${light}` },
    { background: c, border: `2px solid #${light}`, color: `#${light}`, boxShadow: "0 1px 3px rgba(0,0,0,.45)" },
    { background: `linear-gradient(${lighter}, ${c})`, border: `1px solid ${c}`, color: `#${dark}` },
    { background: `linear-gradient(${c}, ${darker})`, border: "none", color: `#${light}`, boxShadow: "0 2px 3px rgba(0,0,0,.4)" },
    { background: `linear-gradient(${lighter}, ${c} 50%, ${darker})`, border: "none", color: `#${light}`, boxShadow: "0 3px 5px rgba(0,0,0,.5)" },
  ];
  Object.assign(t.style, looks[row]);
  return t;
}

async function stylesMenu() {
  if (!formattable()) return;
  const colours = await palette();
  const light = colours[0] || "FFFFFF", dark = colours[1] || "000000";
  const base = [dark, ...colours.slice(4, 10)];
  openPop(["format.shape-styles", "drawing.quick-styles"], "shp-menu shp-styles", (p) => {
    p.appendChild(el("div", "shp-head", "Theme Styles"));
    const grid = el("div", "shp-style-grid");
    for (let row = 0; row < STYLE_ROWS.length; row++) {
      for (let col = 0; col < STYLE_COLUMNS.length; col++) {
        const b = el("button", "shp-style");
        b.type = "button";
        const label = `${STYLE_ROWS[row]} - ${STYLE_COLUMNS[col]}`;
        b.title = label;
        b.setAttribute("aria-label", label);
        b.appendChild(styleThumb(row, base[col], light, dark));
        b.addEventListener("click", () => { closePop(); format("Shape Style", "set_shape_style", [row, col]); });
        grid.appendChild(b);
      }
    }
    p.appendChild(grid);
  });
}

/* ── The Shape Format tab ───────────────────────────────────────────────── */

// A contextual tab: there while a shape is selected (`tab.contextual` in `ribbon.js`).
// It is added to the ribbon's model here, so the Shapes work lives in one module.
TABS.push({ id: "shape-format", label: "Shape Format", key: "JD", contextual: "format.tab", groups: [
  { label: "Insert Shapes", fold: 2, icon: "shapes", items: [
    { kind: "large", id: "insert.shapes", icon: "shapes", label: "Shapes" },
    { kind: "large", id: "insert.text-box", icon: "textbox", label: "Text Box" },
  ] },
  { label: "Shape Styles", fold: 3, icon: "style_guide", items: [
    { kind: "large", id: "format.shape-styles", icon: "style_guide", label: "Shape Styles" },
    { kind: "rows", rows: [
      [{ kind: "small", id: "format.shape-fill", icon: "paint_bucket", label: "Shape Fill" }],
      [{ kind: "small", id: "format.shape-outline", icon: "border_outside", label: "Shape Outline" }],
      [{ kind: "small", id: "format.shape-effects", icon: "asterisk", label: "Shape Effects" }],
    ] },
  ] },
] });

export function init() {
  register("insert.shapes", { label: "Shapes", enabled: hasSlide, run: gallery });
  register("format.tab", { label: "Shape Format", enabled: formattable, run: () => selectTab("shape-format") });
  for (const [ids, label, runIt] of [
    [["format.shape-fill", "drawing.fill"], "Shape Fill", fillMenu],
    [["format.shape-outline", "drawing.outline"], "Shape Outline", outlineMenu],
    [["format.shape-styles", "drawing.quick-styles"], "Shape Styles", stylesMenu],
  ]) {
    for (const id of ids) register(id, { label, enabled: formattable, run: runIt });
  }
  selection.onPaint(paintHandles);
  window.addEventListener("resize", () => paintHandles(sel));
}
