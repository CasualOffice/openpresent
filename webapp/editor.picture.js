// The Picture Format tab (`UX-003`, docs/53): shown while a picture is selected, as
// PowerPoint's contextual tab. Crop puts black crop handles on the picture's sides
// and corners; dragging one cuts that side (or pads it, past the edge) and moves the
// frame with it, so what stays visible stays where it was. Enter, Esc, Crop again or
// a press elsewhere ends cropping. Reset Picture and Reset Picture & Size, and the
// basic Picture Styles, apply on one click.
//
// Every verb is a registered command and every change one engine op with its inverse
// (`casual-slide-wasm` `picture.rs`), so Undo and Redo are the shared ones. Nothing
// here reads a document string.

import { register } from "./commands.js";
import { state, request, status } from "./editor.core.js";
import { queueSync, selectTab } from "./ribbon.js";
import { errorDialog } from "./editor.dialogs.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";

const byId = (id) => document.getElementById(id);
const PICTURE = 1024;            // `ShapeBox::PICTURE`
const MAX_CROP = 99000;          // `casual-slide-wasm` `MAX_CROP`
const MAX_PAD = 1000000;         // `MAX_PAD`
// The crop handles: [x, y] of the side or corner, 0 left/top, 1 right/bottom.
const HANDLES = [[0, 0], [0.5, 0], [1, 0], [1, 0.5], [1, 1], [0.5, 1], [0, 1], [0, 0.5]];

let sel = null;                  // the selected shape, as the selection paints it
let cropping = null;             // { index, node, crop: [l,t,r,b], box }
let drag = null;
let layer = null;
let wasPicture = false;

export const pictureSelected = () => !!state.deck && !!sel && !!(sel.flags & PICTURE) && selection.selectedNodes().length === 1;
const scale = () => byId("slide-frame").clientWidth / state.deck.width;

async function call(fn, args) {
  const r = await request({ op: "build", fn, args }, { priority: 9 });
  selection.noteUndoState(r.undo);
  return r;
}

async function edit(what, fn, args) {
  const s = sel;
  if (!s) return;
  try {
    const r = await call(fn, [s.index, s.node, ...args]);
    if (r.value === false) { status(`${what}: nothing changed`); return; }
    await slides.redrawSlide(s.index);
    status(what);
  } catch (e) {
    errorDialog(`OpenPresent could not change that picture: ${what.toLowerCase()}.`, e.message);
  }
}

/* ── Crop ───────────────────────────────────────────────────────────────── */

function cropLayer() {
  if (!layer || !layer.isConnected) {
    layer = document.createElement("div");
    layer.className = "pic-crop-layer";
    byId("slide-frame").appendChild(layer);
  }
  return layer;
}

function paintCrop() {
  const l = cropLayer();
  l.textContent = "";
  if (!cropping) return;
  const k = scale();
  const b = drag ? drag.box : cropping.box;
  const frame = document.createElement("div");
  frame.className = "pic-crop-frame";
  Object.assign(frame.style, { left: `${b.x * k}px`, top: `${b.y * k}px`, width: `${b.cx * k}px`, height: `${b.cy * k}px` });
  l.appendChild(frame);
  HANDLES.forEach(([hx, hy], i) => {
    const h = document.createElement("div");
    h.className = `pic-crop-h ${hx === 0.5 || hy === 0.5 ? "pic-crop-side" : "pic-crop-corner"}`;
    h.dataset.cropHandle = String(i);
    if (hx === 0.5) h.classList.add("pic-crop-hz");
    if (hy === 0.5) h.classList.add("pic-crop-vt");
    Object.assign(h.style, { left: `${(b.x + hx * b.cx) * k}px`, top: `${(b.y + hy * b.cy) * k}px` });
    h.title = "Drag to crop";
    h.addEventListener("pointerdown", (e) => startDrag(e, i));
    l.appendChild(h);
  });
}

async function startCrop() {
  const s = sel;
  if (!pictureSelected()) return;
  if (cropping) { endCrop(); return; }
  try {
    const r = await call("picture_crop", [s.index, s.node]);
    cropping = { index: s.index, node: s.node, crop: r.value.map(Number), box: { x: s.x, y: s.y, cx: s.cx, cy: s.cy } };
    paintCrop();
    status("Crop: drag a black handle; Enter or Esc when done");
    queueSync();
  } catch (e) {
    errorDialog("OpenPresent could not crop that picture.", e.message);
  }
}

function endCrop() {
  if (!cropping) return;
  cropping = null;
  drag = null;
  paintCrop();
  status("Ready");
  queueSync();
}

/// The crop and frame after moving handle `i` by `dx`, `dy` EMU from `c`, `b`. The
/// whole image's size is what the frame shows over what the crop keeps.
function cropped(c, b, i, dx, dy) {
  const [hx, hy] = HANDLES[i];
  let [l, t, r, bt] = c;
  let { x, y, cx, cy } = b;
  const wholeW = cx / Math.max(1e-6, 1 - (l + r) / 100000);
  const wholeH = cy / Math.max(1e-6, 1 - (t + bt) / 100000);
  const clampSide = (v, other) => Math.max(-MAX_PAD, Math.min(MAX_CROP - other, v));
  if (hx === 0) {
    const nl = clampSide(l + (dx / wholeW) * 100000, r);
    const d = ((nl - l) / 100000) * wholeW;
    l = nl; x += d; cx -= d;
  } else if (hx === 1) {
    const nr = clampSide(r - (dx / wholeW) * 100000, l);
    cx += ((r - nr) / 100000) * wholeW; r = nr;
  }
  if (hy === 0) {
    const nt = clampSide(t + (dy / wholeH) * 100000, bt);
    const d = ((nt - t) / 100000) * wholeH;
    t = nt; y += d; cy -= d;
  } else if (hy === 1) {
    const nb = clampSide(bt - (dy / wholeH) * 100000, t);
    cy += ((bt - nb) / 100000) * wholeH; bt = nb;
  }
  return { crop: [l, t, r, bt].map(Math.round), box: { x: Math.round(x), y: Math.round(y), cx: Math.round(cx), cy: Math.round(cy) } };
}

function startDrag(e, i) {
  if (e.button !== 0 || !cropping) return;
  e.preventDefault();
  e.stopPropagation();
  // The handles are drawn again on every move, so the gesture is followed on the
  // document rather than on the handle it started on.
  drag = { i, start: [e.clientX, e.clientY], box: { ...cropping.box }, crop: cropping.crop.slice() };
  const move = (ev) => {
    const k = scale();
    const r = cropped(cropping.crop, cropping.box, i, (ev.clientX - drag.start[0]) / k, (ev.clientY - drag.start[1]) / k);
    drag.box = r.box;
    drag.crop = r.crop;
    paintCrop();
  };
  const up = async () => {
    document.removeEventListener("pointermove", move);
    document.removeEventListener("pointerup", up);
    const d = drag;
    drag = null;
    if (!cropping || !d) return;
    const c = cropping;
    if (d.crop.every((v, j) => v === c.crop[j])) { paintCrop(); return; }
    try {
      await call("set_picture_crop", [c.index, c.node, ...d.crop, d.box.x, d.box.y, d.box.cx, d.box.cy]);
      c.crop = d.crop;
      c.box = d.box;
      selection.selectNode(c.node);
      await slides.redrawSlide(c.index);
      status("Crop");
    } catch (err) {
      errorDialog("OpenPresent could not crop that picture.", err.message);
    }
    paintCrop();
  };
  document.addEventListener("pointermove", move);
  document.addEventListener("pointerup", up);
}

/* ── The tab ────────────────────────────────────────────────────────────── */

function onSelection(s) {
  sel = s;
  const isPic = pictureSelected();
  if (cropping && (!s || s.node !== cropping.node)) endCrop();
  // The selection paints again when the slide frame changes size (a zoom, a pane
  // opening): the crop handles are in pixels, so they follow it (`UX-013`).
  else if (cropping && !drag) paintCrop();
  // As PowerPoint opens Picture Format when a picture is picked.
  if (isPic && !wasPicture) requestAnimationFrame(() => requestAnimationFrame(() => selectTab("picture-format")));
  wasPicture = isPic;
  queueSync();
}

export function init() {
  register("picture.crop", { label: "Crop", enabled: pictureSelected, checked: () => !!cropping, run: startCrop });
  register("picture.reset", { label: "Reset Picture", enabled: pictureSelected, run: () => edit("Reset Picture", "reset_picture", [false]) });
  register("picture.reset-size", { label: "Reset Picture & Size", enabled: pictureSelected, run: () => edit("Reset Picture & Size", "reset_picture", [true]) });
  register("picture.styles", {
    label: "Picture Styles", enabled: pictureSelected,
    run: () => edit("Picture Style", "set_picture_style", ["Simple Frame, White"]),
    pick: (name) => edit("Picture Style", "set_picture_style", [name]),
  });
  selection.onPaint(onSelection);
  document.addEventListener("keydown", (e) => {
    if (!cropping) return;
    if (e.key === "Escape" || e.key === "Enter") { e.preventDefault(); e.stopPropagation(); endCrop(); }
  }, true);
  window.addEventListener("resize", () => { if (cropping) paintCrop(); });
}
