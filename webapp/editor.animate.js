// The Transitions and Animations tabs, and the Animation Pane (`VS-10`).
//
// Transitions: the gallery gives the current slide None, Fade, Push, Wipe, Split or
// Cut at PowerPoint's default duration; Duration changes it; Apply To All gives every
// slide this slide's transition. Animations: the Animation Styles gallery makes the
// selected shape's effect Appear, Fade or Fly In (None removes it); Add Animation adds
// any of the six this build writes (Appear, Fade, Fly In; Pulse; Disappear, Fade Out)
// after the others; Start sets when the chosen effect starts; Move Earlier and Move
// Later reorder it. The Animation Pane, a task pane like Format Shape, lists the order
// and selects, reorders and deletes.
//
// Every verb is a registered command and one engine op with its inverse, so Undo and
// Redo are the shared `edit.undo`/`edit.redo`. Shape names are document text and reach
// the DOM through `textContent` only. Gallery tiles this build cannot write yet (Morph,
// Reveal, Float In…) say so in the status bar and change nothing.

import { register } from "./commands.js";
import { state, request, changed, status } from "./editor.core.js";
import { showModal, closeModal, errorDialog } from "./editor.dialogs.js";
import { icon } from "./ribbon.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";
import * as panes from "./editor.panes.js";

const byId = (id) => document.getElementById(id);
const has = () => !!state.deck && state.deck.count > 0;
const hasShape = () => has() && selection.selectedNode() != null;

/// Transition kinds, by the engine's numbers.
const TRANSITIONS = { None: 0, Cut: 1, Fade: 2, Push: 3, Wipe: 4, Split: 5 };
const TRANSITION_NAMES = Object.fromEntries(Object.entries(TRANSITIONS).map(([k, v]) => [v, k]));
const TRANSITION_MS = { 0: 0, 1: 0, 2: 700, 3: 1000, 4: 1000, 5: 1500 };

/// Effects, by the engine's numbers, with their class.
export const EFFECTS = [
  { code: 0, name: "Appear", cls: "Entrance" },
  { code: 1, name: "Fade", cls: "Entrance" },
  { code: 2, name: "Fly In", cls: "Entrance" },
  { code: 3, name: "Pulse", cls: "Emphasis" },
  { code: 4, name: "Disappear", cls: "Exit" },
  { code: 5, name: "Fade Out", cls: "Exit" },
];
const STARTS = ["On Click", "With Previous", "After Previous"];
const CLASS_NAMES = ["Other", "Entrance", "Emphasis", "Exit"];

let transition = { kind: 0, ms: 0 };   // the current slide's, as read
let list = [];                          // the current slide's effects
let names = new Map();                  // node → shape name
let chosen = -1;                        // the pane's selected effect
let readFor = -1;                       // the slide `transition` and `list` describe
let readError = "";

async function call(fn, args = []) {
  const r = await request({ op: "animate", fn, args }, { priority: 9 });
  if (r.undo) selection.noteUndoState(r.undo);
  return r.value;
}

async function guarded(what, f) {
  if (!has()) return;
  try {
    await f();
  } catch (e) {
    errorDialog(`OpenPresent could not ${what}.`, e.message);
  }
}

/// Read the current slide's transition and effects again.
export async function read() {
  if (!has()) { list = []; readFor = -1; return; }
  const index = state.current;
  readFor = index;
  try {
    const t = await call("transition_choice", [index]);
    transition = { kind: t[0], ms: t[1] };
    readError = "";
    const v = await call("slide_animations", [index]);
    const next = [];
    for (let i = 0; i + 8 < v.length; i += 9) {
      next.push({ node: v[i], kind: v[i + 1], cls: v[i + 2], start: v[i + 3], ms: v[i + 4], box: [v[i + 5], v[i + 6], v[i + 7], v[i + 8]] });
    }
    list = next;
    names = new Map();
    for (const e of list) {
      if (e.node >= 0 && !names.has(e.node)) names.set(e.node, await call("shape_name", [index, e.node]));
    }
  } catch (e) {
    list = [];
    readError = e.message;
  }
  if (chosen >= list.length) chosen = list.length - 1;
  renderPane();
  changed();
}

async function edited(redraw = true) {
  await read();
  if (redraw) await slides.redrawSlide(state.current);
}

/* ── Transitions ────────────────────────────────────────────────────────── */

function setTransition(kind, ms = TRANSITION_MS[kind]) {
  return guarded("set the transition", async () => {
    await call("set_slide_transition", [state.current, kind, ms]);
    await edited(false);
    status(kind ? `Transition: ${TRANSITION_NAMES[kind]}` : "No transition");
  });
}

function pickTransition(name) {
  if (!(name in TRANSITIONS)) { status(`${name} is not available yet; this build writes None, Cut, Fade, Push, Wipe and Split`); return; }
  setTransition(TRANSITIONS[name]);
}

function chooser(title, items, pick) {
  showModal(title, (body) => {
    const grid = document.createElement("div");
    grid.className = "op-layouts";
    for (const [label, value, hint] of items) {
      const b = document.createElement("button");
      b.type = "button";
      b.className = "op-btn op-layout";
      b.textContent = label;
      if (hint) b.title = hint;
      b.addEventListener("click", () => { closeModal(); pick(value); });
      grid.appendChild(b);
    }
    body.appendChild(grid);
  });
}

function transitionGallery() {
  chooser("Transition to This Slide", Object.entries(TRANSITIONS).map(([n, k]) => [n, k]), (k) => setTransition(k));
}

function durationDialog(title, seconds, apply) {
  showModal(title, (body) => {
    const label = document.createElement("label");
    label.className = "fp-row";
    label.appendChild(document.createTextNode("Duration (seconds)"));
    const input = document.createElement("input");
    input.type = "number";
    input.min = "0.01";
    input.max = "59";
    input.step = "0.25";
    input.className = "fp-input";
    input.value = seconds.toFixed(2);
    label.appendChild(input);
    const ok = document.createElement("button");
    ok.type = "button";
    ok.className = "op-btn primary";
    ok.textContent = "OK";
    const go = () => {
      const s = Number(input.value);
      if (!(s > 0 && s < 60)) { input.focus(); return; }
      closeModal();
      apply(Math.round(s * 1000));
    };
    ok.addEventListener("click", go);
    input.addEventListener("keydown", (e) => { if (e.key === "Enter") go(); });
    body.append(label, ok);
  });
}

/* ── Animations ─────────────────────────────────────────────────────────── */

const GALLERY_EFFECTS = { Appear: 0, Fade: 1, "Fly In": 2 };

function replaceEffect(kind) {
  const node = selection.selectedNode();
  if (node == null) { status("Select a shape on the slide to animate it"); return; }
  return guarded("animate the shape", async () => {
    await call("replace_animation", [state.current, node, kind]);
    await edited();
    if (!panes.isOpen() && kind >= 0) status(`Animation: ${EFFECTS[kind].name}`);
  });
}

function pickAnimation(name) {
  if (name === "None") { replaceEffect(-1); return; }
  if (!(name in GALLERY_EFFECTS)) { status(`${name} is not available yet; Add Animation lists the effects this build writes`); return; }
  replaceEffect(GALLERY_EFFECTS[name]);
}

function addEffect(kind, start = 0) {
  const node = selection.selectedNode();
  if (node == null) return;
  return guarded("add the animation", async () => {
    await call("add_animation", [state.current, node, kind, start]);
    await edited();
    chosen = list.length - 1;
    renderPane();
  });
}

function addAnimationDialog() {
  chooser("Add Animation", EFFECTS.map((e) => [`${e.name} (${e.cls})`, e.code]), (k) => addEffect(k));
}

/// The effect the Start and Move commands act on: the pane's choice, else the
/// selected shape's first.
function target() {
  if (chosen >= 0 && chosen < list.length) return chosen;
  const node = selection.selectedNode();
  return node == null ? -1 : list.findIndex((e) => e.node === node);
}

function setStart(start, at = target()) {
  if (at < 0) return;
  return guarded("set when the animation starts", async () => {
    await call("set_animation_start", [state.current, at, start]);
    await edited(false);
  });
}

function moveEffect(from, to) {
  if (from < 0 || to < 0 || to >= list.length || from === to) return;
  return guarded("reorder the animations", async () => {
    await call("move_animation", [state.current, from, to]);
    chosen = to;
    await edited(false);
  });
}

function removeEffect(at) {
  if (at < 0) return;
  return guarded("remove the animation", async () => {
    await call("remove_animation", [state.current, at]);
    await edited();
  });
}

/* ── The Animation Pane ─────────────────────────────────────────────────── */

let pane = null;

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

function smallButton(ic, label, onClick) {
  const b = h("button", "fp-close");
  b.type = "button";
  b.title = label;
  b.setAttribute("aria-label", label);
  b.appendChild(icon(ic, 16));
  b.addEventListener("click", onClick);
  return b;
}

function buildPane() {
  pane = h("aside", "task-pane anim-pane");
  pane.id = "anim-pane";
  pane.setAttribute("aria-label", "Animation Pane");
  pane.hidden = true;
  const head = h("div", "fp-head");
  head.appendChild(h("h2", "fp-title", "Animation Pane"));
  head.appendChild(smallButton("dismiss", "Close the Animation Pane", closePane));
  const tools = h("div", "ap-tools");
  tools.append(
    smallButton("chevron_up", "Move Earlier", () => moveEffect(chosen, chosen - 1)),
    smallButton("chevron_down", "Move Later", () => moveEffect(chosen, chosen + 1)),
    smallButton("eraser", "Remove", () => removeEffect(chosen)),
  );
  const ol = h("ol", "ap-list");
  ol.setAttribute("role", "listbox");
  ol.setAttribute("aria-label", "Animations on this slide");
  ol.tabIndex = 0;
  ol.addEventListener("keydown", (e) => {
    if (e.key === "ArrowDown" && chosen < list.length - 1) { chosen++; renderPane(); e.preventDefault(); }
    else if (e.key === "ArrowUp" && chosen > 0) { chosen--; renderPane(); e.preventDefault(); }
    else if (e.key === "Delete" || e.key === "Backspace") { e.preventDefault(); e.stopPropagation(); removeEffect(chosen); }
  });
  const none = h("p", "fp-none");
  pane.append(head, tools, ol, none);
  pane.addEventListener("keydown", (e) => { if (e.key === "Escape") { e.stopPropagation(); closePane(); } });
  byId("workspace").appendChild(pane);
}

function renderPane() {
  if (!pane || pane.hidden) return;
  const ol = pane.querySelector(".ap-list");
  const none = pane.querySelector(".fp-none");
  ol.textContent = "";
  none.hidden = list.length > 0 && !readError;
  none.textContent = readError ? `This slide's animations cannot be edited here yet. ${readError}` : "Select a shape on the slide, then choose an effect from Animation Styles or Add Animation.";
  let click = 0;
  list.forEach((e, i) => {
    if (e.start === 0) click++;
    const li = h("li", "ap-item");
    li.setAttribute("role", "option");
    li.setAttribute("aria-selected", i === chosen ? "true" : "false");
    li.dataset.cls = CLASS_NAMES[e.cls] || "Other";
    li.appendChild(h("span", "ap-click", e.start === 0 ? String(click) : ""));
    const kind = e.kind >= 0 ? EFFECTS[e.kind].name : "Other effect";
    const who = e.node >= 0 ? (names.get(e.node) || "Shape") : "No shape";
    const label = h("span", "ap-label");
    label.append(h("span", "ap-kind", kind), h("span", "ap-name", who));
    li.appendChild(label);
    const sel = h("select", "ap-start");
    sel.setAttribute("aria-label", "Start");
    STARTS.forEach((s, k) => { const o = h("option", null, s); o.value = String(k); sel.appendChild(o); });
    sel.value = String(e.start);
    sel.addEventListener("change", () => setStart(Number(sel.value), i));
    sel.addEventListener("click", (ev) => ev.stopPropagation());
    li.appendChild(sel);
    li.title = `${kind}: ${who}, ${STARTS[e.start]}, ${(e.ms / 1000).toFixed(2)} s`;
    li.addEventListener("click", () => {
      chosen = i;
      if (e.node >= 0) selection.selectNode(e.node);
      selection.refresh?.();
      renderPane();
      changed();
    });
    ol.appendChild(li);
  });
}

function openPane() {
  if (!pane) buildPane();
  panes.close();
  pane.hidden = false;
  read();
  changed();
}

function closePane() {
  if (!pane || pane.hidden) return;
  pane.hidden = true;
  changed();
  byId("stage")?.focus({ preventScroll: true });
}

const paneOpen = () => !!pane && !pane.hidden;

/* ── Commands ───────────────────────────────────────────────────────────── */

export function init() {
  register("transitions.gallery", { label: "Transitions", enabled: has, pick: pickTransition, selected: () => TRANSITION_NAMES[transition.kind] ?? "", run: transitionGallery });
  register("transitions.timing", {
    label: "Duration", enabled: () => has() && transition.kind > 1,
    value: () => (transition.kind > 1 ? `${(transition.ms / 1000).toFixed(2)}` : ""),
    run: () => durationDialog("Transition Duration", transition.ms / 1000 || 1, (ms) => setTransition(transition.kind, ms)),
  });
  register("transitions.apply-to-all", {
    label: "Apply To All", enabled: () => has() && state.deck.count > 1,
    run: () => guarded("apply the transition to all slides", async () => {
      await call("transition_to_all", [state.current]);
      await edited(false);
      status("Every slide has this slide's transition");
    }),
  });
  register("animations.gallery", { label: "Animation Styles", enabled: hasShape, pick: pickAnimation, run: addAnimationDialog });
  register("animations.add", { label: "Add Animation", enabled: hasShape, run: addAnimationDialog });
  register("animations.pane", { label: "Animation Pane", enabled: has, checked: paneOpen, run: () => (paneOpen() ? closePane() : openPane()) });
  register("animations.start", {
    label: "Start", enabled: () => has() && target() >= 0,
    value: () => { const t = target(); return t >= 0 ? STARTS[list[t].start] : ""; },
    run: () => chooser("Start", STARTS.map((s, k) => [s, k]), (k) => setStart(k)),
  });
  register("animations.earlier", { label: "Move Earlier", enabled: () => target() > 0, run: () => moveEffect(target(), target() - 1) });
  register("animations.later", { label: "Move Later", enabled: () => { const t = target(); return t >= 0 && t < list.length - 1; }, run: () => moveEffect(target(), target() + 1) });
  register("animations.remove", { label: "Remove Animation", enabled: () => target() >= 0, run: () => removeEffect(target()) });
  // Undo and redo, another slide, another deck: read again.
  document.addEventListener("op-slides-built", () => { read(); });
  document.addEventListener("op-changed", () => {
    if (state.deck && readFor !== state.current) { chosen = -1; read(); }
    else if (!state.deck) readFor = -1;
  });
}
