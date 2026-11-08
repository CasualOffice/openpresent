// Building a deck (`VS-08`): New Slide with a layout picker drawn from the deck's own
// layouts, Duplicate, Delete and Hide Slide, reordering by dragging thumbnails, and
// Insert Text Box and Pictures; Insert > Shapes is `editor.shapes.js` (`SHAPE-01`). Every verb is a registered command; each is
// one engine op with its inverse, so Undo and Redo are the shared `edit.undo`/`edit.redo`.
//
// Layout names are document text: they reach the DOM through `textContent` only.

import { register, run } from "./commands.js";
import { state, request, changed, status } from "./editor.core.js";
import { showModal, closeModal, errorDialog } from "./editor.dialogs.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";

const byId = (id) => document.getElementById(id);
const has = () => !!state.deck;
const hasSlide = () => has() && state.deck.count > 0;

let hidden = [];           // per slide, from the engine
let layouts = [];          // [{ ordinal, name }]

/// One build call through the worker; afterwards the undo state and slide count.
async function call(fn, args = [], transfer) {
  const r = await request({ op: "build", fn, args }, { priority: 9, transfer });
  selection.noteUndoState(r.undo);
  return r;
}

async function guarded(what, f) {
  if (!has()) return;
  try {
    await f();
  } catch (e) {
    errorDialog(`OpenPresent could not ${what}.`, e.message);
  }
}

/// The slide list changed: rebuild the pane with `count` slides and show `index`.
function relist(count, index) {
  slides.reloadList(count, index);   // its rebuilt pane marks hidden slides again
}

/// Mark hidden slides' thumbnails, as PowerPoint strikes their numbers through.
export async function markHidden() {
  if (!has()) return;
  try {
    hidden = (await request({ op: "build", fn: "hidden_slides", args: [] }, { priority: 7 })).value.map(Boolean);
  } catch {
    hidden = [];
  }
  const list = byId("thumbs");
  for (const li of list.children) {
    const i = Number(li.dataset.index);
    li.classList.toggle("is-hidden", !!hidden[i]);
    li.setAttribute("aria-label", hidden[i] ? `Slide ${i + 1}, hidden` : `Slide ${i + 1}`);
  }
  changed();
}

async function readLayouts() {
  const v = (await request({ op: "build", fn: "slide_layouts", args: [] }, { priority: 9 })).value;
  layouts = [];
  for (let i = 0; i + 1 < v.length; i += 2) layouts.push({ ordinal: Number(v[i]), name: v[i + 1] });
  return layouts;
}

/* ── Slides ─────────────────────────────────────────────────────────────── */

async function newSlideOn(layout) {
  const at = state.deck.count ? state.current + 1 : 0;
  const r = await call("new_slide", [at, layout]);
  await relist(r.count, r.value);
}

/// New Slide (Ctrl+M): after the current slide, on its layout, as PowerPoint; after
/// a title slide, on Title and Content where the deck has it.
function newSlide() {
  return guarded("add a slide", async () => {
    const all = await readLayouts();
    if (!all.length) return;
    let layout = all[0].ordinal;
    if (state.deck.count) {
      const current = (await request({ op: "build", fn: "slide_layout", args: [state.current] }, { priority: 9 })).value;
      // An `i64` from the bridge arrives as a BigInt.
      if (current >= 0) layout = Number(current);
      const name = all.find((l) => l.ordinal === layout)?.name;
      const content = all.find((l) => l.name === "Title and Content");
      if (name === "Title Slide" && content) layout = content.ordinal;
    }
    await newSlideOn(layout);
  });
}

/// New Slide's gallery: the deck's own layouts, by name.
function layoutPicker() {
  return guarded("list the layouts", async () => {
    const all = await readLayouts();
    showModal("New Slide", (body) => {
      const p = document.createElement("p");
      p.textContent = "Choose a layout for the new slide.";
      body.appendChild(p);
      const grid = document.createElement("div");
      grid.className = "op-layouts";
      for (const l of all) {
        const b = document.createElement("button");
        b.type = "button";
        b.className = "op-btn op-layout";
        b.textContent = l.name;
        b.addEventListener("click", () => { closeModal(); guarded("add a slide", () => newSlideOn(l.ordinal)); });
        grid.appendChild(b);
      }
      body.appendChild(grid);
      grid.querySelector("button")?.classList.add("primary");
    });
  });
}

function duplicateSlide() {
  return guarded("duplicate the slide", async () => {
    const r = await call("duplicate_slide", [state.current]);
    await relist(r.count, r.value);
  });
}

function deleteSlide() {
  return guarded("delete the slide", async () => {
    const at = state.current;
    const r = await call("delete_slide", [at]);
    await relist(r.count, Math.min(at, r.count - 1));
  });
}

function toggleHidden() {
  return guarded("hide the slide", async () => {
    await call("set_slide_hidden", [state.current, !hidden[state.current]]);
    await markHidden();
  });
}

async function moveSlide(from, to) {
  if (from === to) return;
  await guarded("move the slide", async () => {
    const r = await call("move_slide", [from, to]);
    await relist(r.count, to);
  });
}

/* ── Insert ─────────────────────────────────────────────────────────────── */

const KINDS = { textbox: 0, rectangle: 1, oval: 2, line: 3, arrow: 4 };

/// A new shape's box, in EMU, centred on the slide, as a click-to-insert gives.
function defaultBox(kind) {
  const W = state.deck.width, H = state.deck.height;
  if (kind === "textbox") return [W / 3, H * 0.42, W / 3, H * 0.08];
  if (kind === "line" || kind === "arrow") return [W * 3 / 8, H / 2, W / 4, 0];
  const s = Math.min(W, H) / 4;
  return [(W - s) / 2, (H - s) / 2, s, s];
}

async function inserted(r) {
  selection.selectNode(r.value);
  await slides.redrawSlide(state.current);
}

function insertShape(kind) {
  return guarded("insert that", async () => {
    const [x, y, cx, cy] = defaultBox(kind).map(Math.round);
    await inserted(await call("insert_shape", [state.current, KINDS[kind], x, y, cx, cy]));
  });
}

let picker = null;
function pickPicture() {
  if (!picker) {
    picker = document.createElement("input");
    picker.type = "file";
    picker.accept = "image/png,image/jpeg";
    picker.hidden = true;
    document.body.appendChild(picker);
    picker.addEventListener("change", () => {
      const f = picker.files?.[0];
      if (f) insertPicture(f);
    });
  }
  picker.value = "";
  picker.click();
}

/// Insert > Pictures: the picture at its own proportions, fitted inside most of the
/// slide and centred. The engine decides from the bytes whether it is a picture.
function insertPicture(file) {
  return guarded("insert the picture", async () => {
    status(`Inserting ${file.name}…`);
    const bytes = await file.arrayBuffer();
    let ratio = 1;
    try {
      const bmp = await createImageBitmap(new Blob([bytes]));
      ratio = bmp.height / Math.max(1, bmp.width);
      bmp.close();
    } catch { /* the engine refuses what is not a picture */ }
    const W = state.deck.width, H = state.deck.height;
    let cx = W * 0.6, cy = cx * ratio;
    if (cy > H * 0.8) { cy = H * 0.8; cx = cy / ratio; }
    const box = [(W - cx) / 2, (H - cy) / 2, cx, cy].map(Math.round);
    await inserted(await call("insert_picture", [state.current, bytes, ...box], [bytes]));
    status("Picture inserted");
  });
}

/* ── Reordering by drag ─────────────────────────────────────────────────── */

function initDrag() {
  const list = byId("thumbs");
  let drag = null;
  const target = (e) => {
    const li = document.elementFromPoint(e.clientX, e.clientY)?.closest?.("#thumbs .thumb");
    if (!li) return null;
    const r = li.getBoundingClientRect();
    const i = Number(li.dataset.index);
    return e.clientY > r.top + r.height / 2 ? i + 1 : i;   // a gap: before slide i
  };
  const mark = (gap) => {
    for (const li of list.children) {
      const i = Number(li.dataset.index);
      li.classList.toggle("drop-before", gap === i);
      li.classList.toggle("drop-after", gap === list.children.length && i === list.children.length - 1);
    }
  };
  list.addEventListener("pointerdown", (e) => {
    const li = e.target.closest?.(".thumb");
    if (!li || e.button !== 0) return;
    drag = { from: Number(li.dataset.index), y: e.clientY, moving: false, id: e.pointerId };
  });
  list.addEventListener("pointermove", (e) => {
    if (!drag || e.pointerId !== drag.id) return;
    if (!drag.moving && Math.abs(e.clientY - drag.y) < 6) return;
    if (!drag.moving) { drag.moving = true; list.setPointerCapture(e.pointerId); list.classList.add("is-dragging"); }
    mark(target(e));
  });
  const end = (e) => {
    const d = drag;
    drag = null;
    list.classList.remove("is-dragging");
    mark(-1);
    if (!d || !d.moving) return;
    const gap = target(e);
    if (gap == null) return;
    // A gap after the slide's own place counts after it is taken out.
    const to = gap > d.from ? gap - 1 : gap;
    // The click that follows a drag must not go back to the old place.
    list.addEventListener("click", (c) => c.stopPropagation(), { capture: true, once: true });
    moveSlide(d.from, Math.max(0, Math.min(state.deck.count - 1, to)));
  };
  list.addEventListener("pointerup", end);
  list.addEventListener("pointercancel", () => { drag = null; list.classList.remove("is-dragging"); mark(-1); });
  // Delete in the thumbnail pane deletes the slide, as PowerPoint's.
  list.addEventListener("keydown", (e) => {
    if ((e.key === "Delete" || e.key === "Backspace") && hasSlide()) { e.preventDefault(); e.stopPropagation(); run("home.delete-slide"); }
  });
}

export function init() {
  register("home.new-slide", { label: "New Slide", shortcut: "Ctrl+M", enabled: has, run: newSlide });
  register("home.new-slide-layout", { label: "New Slide from Layout", enabled: has, run: layoutPicker });
  register("home.duplicate-slide", { label: "Duplicate Slide", enabled: hasSlide, run: duplicateSlide });
  register("home.delete-slide", { label: "Delete Slide", enabled: hasSlide, run: deleteSlide });
  register("slideshow.hide-slide", { label: "Hide Slide", enabled: hasSlide, checked: () => !!hidden[state.current], run: toggleHidden });
  register("insert.text-box", { label: "Text Box", enabled: hasSlide, run: () => insertShape("textbox") });
  register("insert.shape.rectangle", { label: "Rectangle", enabled: hasSlide, run: () => insertShape("rectangle") });
  register("insert.shape.oval", { label: "Oval", enabled: hasSlide, run: () => insertShape("oval") });
  register("insert.shape.line", { label: "Line", enabled: hasSlide, run: () => insertShape("line") });
  register("insert.shape.arrow", { label: "Arrow", enabled: hasSlide, run: () => insertShape("arrow") });
  register("insert.pictures", { label: "Pictures", enabled: hasSlide, run: pickPicture });
  initDrag();
  document.addEventListener("op-slides-built", () => { markHidden(); });
}

/// Ctrl+M, from the editor's key handler.
export function onKey(e) {
  const mod = e.ctrlKey || e.metaKey;
  if (mod && !e.altKey && !e.shiftKey && (e.key === "m" || e.key === "M")) {
    e.preventDefault();
    run("home.new-slide");
    return true;
  }
  return false;
}
