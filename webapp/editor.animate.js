// The Transitions and Animations tabs, and the Animation Pane (`VS-10`).
//
// Transitions (`VS-10`, `UX-004`): the gallery gives the current slide None, Fade,
// Push, Wipe, Split, Reveal, Cut, Uncover, Cover or Zoom at PowerPoint's default
// duration; Effect Options picks its direction (or Smoothly, Through Black); Duration
// changes it; On Mouse Click and After set how the slide advances; Apply To All gives
// every slide this slide's transition; Preview plays it on the slide. Animations: the Animation Styles gallery makes the
// selected shape's effect Appear, Fade or Fly In (None removes it); Add Animation adds
// any of the six this build writes (Appear, Fade, Fly In; Pulse; Disappear, Fade Out)
// after the others; Start sets when the chosen effect starts; Move Earlier and Move
// Later reorder it. The Animation Pane, a pane of the pane host (`UX-011`), lists the order
// and selects, reorders and deletes.
//
// Every verb is a registered command and one engine op with its inverse, so Undo and
// Redo are the shared `edit.undo`/`edit.redo`. Shape names are document text and reach
// the DOM through `textContent` only. The galleries show only what this build writes.

import { register } from "./commands.js";
import { state, request, changed, status } from "./editor.core.js";
import { showModal, closeModal, errorDialog } from "./editor.dialogs.js";
import { icon } from "./ribbon.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";
import * as host from "./editor.panehost.js";
import * as show from "./editor.show.js";

const byId = (id) => document.getElementById(id);
const has = () => !!state.deck && state.deck.count > 0;
const hasShape = () => has() && selection.selectedNode() != null;

/// Transition kinds, by the engine's numbers (`TransitionKind::from_code`).
const TRANSITIONS = { None: 0, Cut: 1, Fade: 2, Push: 3, Wipe: 4, Split: 5, Cover: 6, Uncover: 7, Zoom: 8, Reveal: 9 };
const TRANSITION_NAMES = Object.fromEntries(Object.entries(TRANSITIONS).map(([k, v]) => [v, k]));
const TRANSITION_MS = { 0: 0, 1: 0, 2: 700, 3: 1000, 4: 1000, 5: 1500, 6: 1000, 7: 1000, 8: 500, 9: 1000 };
/// Effect Options, by the engine's numbers (`TransitionOption::from_code`), in
/// PowerPoint's words; and which each kind offers, its default first, as the engine's
/// `TransitionKind::options`.
export const TRANSITION_OPTIONS = ["Smoothly", "Through Black", "From Right", "From Bottom", "From Left", "From Top", "From Bottom-Right", "From Bottom-Left", "From Top-Right", "From Top-Left", "Horizontal Out", "Horizontal In", "Vertical Out", "Vertical In", "In", "Out", "Through Black From Right", "Through Black From Left"];
const SIDES = [2, 3, 4, 5];
const KIND_OPTIONS = { 0: [], 1: [0, 1], 2: [0, 1], 3: SIDES, 4: SIDES, 5: [10, 11, 12, 13], 6: [2, 3, 4, 5, 6, 7, 8, 9], 7: [2, 3, 4, 5, 6, 7, 8, 9], 8: [14, 15], 9: [2, 4, 16, 17] };

/// Effects, by the engine's numbers, with their class.
export const EFFECTS = [
  { code: 0, name: "Appear", cls: "Entrance" },
  { code: 1, name: "Fade", cls: "Entrance" },
  { code: 2, name: "Fly In", cls: "Entrance" },
  { code: 3, name: "Pulse", cls: "Emphasis" },
  { code: 4, name: "Disappear", cls: "Exit" },
  { code: 5, name: "Fade Out", cls: "Exit" },
  { code: 6, name: "Float In", cls: "Entrance" },
  { code: 7, name: "Wipe", cls: "Entrance" },
  { code: 8, name: "Split", cls: "Entrance" },
  { code: 9, name: "Zoom", cls: "Entrance" },
  { code: 10, name: "Grow/Shrink", cls: "Emphasis" },
  { code: 11, name: "Spin", cls: "Emphasis" },
];
/// Effect Options for an effect, by the engine's numbers (`EffectOption::from_code`),
/// and which each kind offers, its default first, as `EffectKind::options`.
export const EFFECT_OPTIONS = ["From Bottom", "From Left", "From Right", "From Top", "Vertical Out", "Vertical In", "Horizontal Out", "Horizontal In"];
const EFFECT_KIND_OPTIONS = { 2: [0, 1, 2, 3], 6: [0, 3], 7: [0, 1, 2, 3], 8: [4, 5, 6, 7] };
const STARTS = ["On Click", "With Previous", "After Previous"];
const CLASS_NAMES = ["Other", "Entrance", "Emphasis", "Exit"];

let transition = { kind: 0, ms: 0, option: -1, onClick: true, after: null };   // the current slide's, as read
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
    transition = { kind: t[0], ms: t[1], option: t[2] === 99 ? -1 : t[2], onClick: t[3] !== 0, after: t[4] > 0 ? t[4] - 1 : null };
    readError = "";
    const v = await call("slide_animations", [index]);
    const next = [];
    for (let i = 0; i + 10 < v.length; i += 11) {
      next.push({ node: v[i], kind: v[i + 1], cls: v[i + 2], start: v[i + 3], ms: v[i + 4], box: [v[i + 5], v[i + 6], v[i + 7], v[i + 8]], option: v[i + 9], delay: v[i + 10] });
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

function setTransition(kind, ms = TRANSITION_MS[kind], option = 99) {
  return guarded("set the transition", async () => {
    await call("set_slide_transition", [state.current, kind, option, ms]);
    await edited(false);
    const opt = transition.option >= 0 && (KIND_OPTIONS[kind] || []).length > 1 ? `, ${TRANSITION_OPTIONS[transition.option]}` : "";
    status(kind ? `Transition: ${TRANSITION_NAMES[kind]}${opt}` : "No transition");
  });
}

function pickTransition(name) {
  if (!(name in TRANSITIONS)) return;
  setTransition(TRANSITIONS[name]);
}

/// Effect Options: the same effect in another direction, at the same duration.
function setOption(code) {
  if (!(KIND_OPTIONS[transition.kind] || []).includes(code)) return;
  return setTransition(transition.kind, transition.ms || TRANSITION_MS[transition.kind], code);
}

function setAdvance(onClick, after) {
  return guarded("set how the slide advances", async () => {
    await call("set_transition_advance", [state.current, onClick, after == null ? 0xffffffff : after]);
    await edited(false);
  });
}

/// Advance Slide After: a time in seconds, or none.
function afterDialog() {
  showModal("Advance Slide", (body) => {
    const on = document.createElement("label");
    on.className = "fp-row";
    const box = document.createElement("input");
    box.type = "checkbox";
    box.checked = transition.after != null;
    on.append(box, document.createTextNode(" After"));
    const label = document.createElement("label");
    label.className = "fp-row";
    label.appendChild(document.createTextNode("Seconds"));
    const input = document.createElement("input");
    input.type = "number";
    input.min = "0";
    input.max = "86399";
    input.step = "0.25";
    input.className = "fp-input";
    input.value = ((transition.after ?? 0) / 1000).toFixed(2);
    input.addEventListener("input", () => { box.checked = true; });
    label.appendChild(input);
    const ok = document.createElement("button");
    ok.type = "button";
    ok.className = "op-btn primary";
    ok.textContent = "OK";
    const go = () => {
      const sec = Number(input.value);
      if (box.checked && !(sec >= 0 && sec < 86400)) { input.focus(); return; }
      closeModal();
      setAdvance(transition.onClick, box.checked ? Math.round(sec * 1000) : null);
    };
    ok.addEventListener("click", go);
    input.addEventListener("keydown", (e) => { if (e.key === "Enter") go(); });
    body.append(on, label, ok);
  });
}

/// Preview: the transition into this slide, played on the slide in the editor
/// from the slide before it (from black on the first), as the show plays it.
async function previewTransition() {
  const index = state.current;
  const frame = byId("slide-frame");
  const now = byId("slide-img");
  if (!now?.src || frame.querySelector(".tr-preview")) return;
  let plan;
  try { plan = await request({ op: "plan" }, { priority: 9 }); } catch (e) { errorDialog("OpenPresent could not preview the transition.", e.message); return; }
  const flags = plan.plan[index * 2] | 0, ms = plan.plan[index * 2 + 1] | 0;
  if (!(flags & 2) || ms <= 0) { status("This slide has no transition to preview"); return; }
  const effect = plan.effects?.[index] || "", dir = plan.directions?.[index] || "";
  let before = null;
  if (index > 0) {
    try {
      const r = await request({ op: "png", index: index - 1, width: Math.max(1, Math.round(frame.clientWidth * Math.min(window.devicePixelRatio || 1, 2))) }, { priority: 8 });
      before = URL.createObjectURL(new Blob([r.png], { type: "image/png" }));
    } catch { before = null; }
  }
  if (state.current !== index) { if (before) URL.revokeObjectURL(before); return; }
  const layer = h("div", "tr-preview");
  const base = h("img", "tr-preview-img");
  base.alt = "";
  if (before) base.src = before; else base.hidden = true;
  const over = h("img", "tr-preview-img");
  over.alt = "";
  over.src = now.src;
  layer.append(base, over);
  frame.appendChild(layer);
  const frames = (flags & 4) ? show.transitionFrames(effect, dir) : null;
  const opts = { duration: ms, easing: "ease-in-out", fill: "forwards" };
  if (frames?.baseOnTop) base.style.zIndex = "2";
  const a = over.animate(frames ? frames.over : [{ opacity: 0 }, { opacity: 1 }], opts);
  if (frames?.base && before) base.animate(frames.base, opts);
  const finish = () => { layer.remove(); if (before) URL.revokeObjectURL(before); };
  a.finished.then(finish, finish);
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

const GALLERY_EFFECTS = Object.fromEntries(EFFECTS.filter((e) => e.cls !== "Exit").map((e) => [e.name, e.code]));

function replaceEffect(kind) {
  const node = selection.selectedNode();
  if (node == null) { status("Select a shape on the slide to animate it"); return; }
  return guarded("animate the shape", async () => {
    await call("replace_animation", [state.current, node, kind]);
    await edited();
    if (!host.isOpen("format") && kind >= 0) status(`Animation: ${EFFECTS[kind].name}`);
  });
}

function pickAnimation(name) {
  if (name === "None") { replaceEffect(-1); return; }
  if (!(name in GALLERY_EFFECTS)) return;
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

/// Effect Options, Duration and Delay of effect `at` (`UX-006`): built again in its
/// place, one undo step.
function setTiming(at, { option = 99, ms, delay }) {
  const e = list[at];
  if (!e || e.kind < 0) return;
  return guarded("change the animation", async () => {
    await call("set_animation", [state.current, at, option, ms ?? e.ms, delay ?? e.delay]);
    await edited(false);
  });
}

const targetEffect = () => { const t = target(); return t >= 0 && list[t].kind >= 0 ? list[t] : null; };

function secondsDialog(title, seconds, min, apply) {
  showModal(title, (body) => {
    const label = document.createElement("label");
    label.className = "fp-row";
    label.appendChild(document.createTextNode("Seconds"));
    const input = document.createElement("input");
    input.type = "number";
    input.min = String(min);
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
      const v = Number(input.value);
      if (!(v >= min && v < 60)) { input.focus(); return; }
      closeModal();
      apply(Math.round(v * 1000));
    };
    ok.addEventListener("click", go);
    input.addEventListener("keydown", (e) => { if (e.key === "Enter") go(); });
    body.append(label, ok);
  });
}

/// Animations > Preview: the slide's effects played on the slide in the editor, in
/// order, each its shape cut from the slide's picture and moved as the show moves it.
let previewing = false;
async function previewAnimations() {
  const index = state.current;
  const frame = byId("slide-frame");
  const now = byId("slide-img");
  const shown = list.filter((e) => e.node >= 0);
  if (!now?.src || previewing || !shown.length) return;
  previewing = true;
  const layer = h("div", "tr-preview");
  layer.style.background = "transparent";
  frame.appendChild(layer);
  try {
    for (const e of shown) {
      if (state.current !== index) break;
      const r = await request({ op: "png-hiding", index, width: Math.max(1, Math.round(frame.clientWidth)), nodes: [e.node] }, { priority: 8 });
      const under = URL.createObjectURL(new Blob([r.png], { type: "image/png" }));
      const base = h("img", "tr-preview-img");
      base.alt = "";
      base.src = under;
      const part = h("img", "tr-preview-img");
      part.alt = "";
      part.src = now.src;
      const [x, y, cx, cy] = e.box;
      const W = state.deck.width, H = state.deck.height;
      const pct = (v, of) => `${Math.max(0, Math.min(100, (v / of) * 100))}%`;
      part.style.clipPath = `inset(${pct(y, H)} ${pct(W - x - cx, W)} ${pct(H - y - cy, H)} ${pct(x, W)})`;
      part.style.transformOrigin = `${((x + cx / 2) / W) * 100}% ${((y + cy / 2) / H) * 100}%`;
      layer.textContent = "";
      layer.append(base, part);
      await Promise.all([base.decode().catch(() => {}), part.decode().catch(() => {})]);
      const frames = show.effectFrames(e, W, H);
      if (frames) await part.animate(frames, { duration: Math.max(1, e.ms), delay: e.delay || 0, easing: "ease-out", fill: "both" }).finished.catch(() => {});
      URL.revokeObjectURL(under);
    }
  } catch (err) {
    errorDialog("OpenPresent could not preview the animations.", err.message);
  } finally {
    layer.remove();
    previewing = false;
  }
}

/// Animation Painter: pick up the selected shape's effects; the next shape selected
/// gets them in place of its own (`UX-006`). Esc or a second click puts it down.
let painterFrom = null;
function armPainter() {
  if (painterFrom != null) { painterFrom = null; status(""); changed(); return; }
  const node = selection.selectedNode();
  if (node == null || !list.some((e) => e.node === node && e.kind >= 0)) { status("Select an animated shape first"); return; }
  painterFrom = { node, index: state.current };
  status("Animation Painter: click the shape to give these animations to");
  changed();
}
function paintTo(s) {
  if (!painterFrom || !s || s.index !== painterFrom.index || s.node === painterFrom.node) return;
  const from = painterFrom.node;
  painterFrom = null;
  guarded("copy the animations", async () => {
    await call("paint_animation", [state.current, from, s.node]);
    await edited(false);
    status("Animations copied");
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

function buildPane(body) {
  pane = h("div", "anim-pane");
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
  pane.append(tools, ol, none);
  body.appendChild(pane);
  read();
}

host.registerPane("animation", { title: "Animation Pane", build: buildPane, refresh: () => renderPane(), closed: () => { pane = null; } });

function renderPane() {
  if (!pane) return;
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

function openPane() { host.open("animation"); }

function closePane() { host.close("animation"); }

const paneOpen = () => host.isOpen("animation");

/* ── Commands ───────────────────────────────────────────────────────────── */

export function init() {
  register("transitions.gallery", { label: "Transitions", enabled: has, pick: pickTransition, selected: () => TRANSITION_NAMES[transition.kind] ?? "", run: transitionGallery });
  const offers = () => KIND_OPTIONS[transition.kind] || [];
  TRANSITION_OPTIONS.forEach((label, code) => {
    register(`transitions.option.${code}`, {
      label, enabled: () => has() && offers().includes(code), hidden: () => !offers().includes(code),
      checked: () => transition.option === code, run: () => setOption(code),
    });
  });
  register("transitions.effect-options", {
    label: "Effect Options", enabled: () => has() && offers().length > 1,
    run: () => chooser("Effect Options", offers().map((c) => [TRANSITION_OPTIONS[c], c]), (c) => setOption(c)),
  });
  register("transitions.preview", { label: "Preview", enabled: () => has() && transition.kind > 1, run: previewTransition });
  register("transitions.on-click", {
    label: "On Mouse Click", enabled: has, checked: () => transition.onClick,
    run: () => setAdvance(!transition.onClick, transition.after),
  });
  register("transitions.after", {
    label: "After", enabled: has, checked: () => transition.after != null,
    value: () => (transition.after != null ? `${(transition.after / 1000).toFixed(2)}` : ""),
    run: afterDialog,
  });
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
  register("animations.gallery", { label: "Animation Styles", enabled: hasShape, pick: pickAnimation, run: addAnimationDialog,
    selected: () => { const n = selection.selectedNode(); const e = list.find((x) => x.node === n); return e && e.kind >= 0 ? EFFECTS[e.kind].name : n == null ? "" : "None"; } });
  const effectOffers = () => { const e = targetEffect(); return e ? EFFECT_KIND_OPTIONS[e.kind] || [] : []; };
  EFFECT_OPTIONS.forEach((label, code) => {
    register(`animations.option.${code}`, {
      label, enabled: () => has() && effectOffers().includes(code), hidden: () => !effectOffers().includes(code),
      checked: () => targetEffect()?.option === code, run: () => setTiming(target(), { option: code }),
    });
  });
  register("animations.effect-options", {
    label: "Effect Options", enabled: () => has() && effectOffers().length > 1,
    run: () => chooser("Effect Options", effectOffers().map((c) => [EFFECT_OPTIONS[c], c]), (c) => setTiming(target(), { option: c })),
  });
  register("animations.timing", {
    label: "Duration", enabled: () => { const e = targetEffect(); return has() && !!e && e.kind !== 0 && e.kind !== 4; },
    value: () => { const e = targetEffect(); return e ? (e.ms / 1000).toFixed(2) : ""; },
    run: () => secondsDialog("Animation Duration", (targetEffect()?.ms || 500) / 1000, 0.01, (ms) => setTiming(target(), { ms })),
  });
  register("animations.delay", {
    label: "Delay", enabled: () => has() && !!targetEffect(),
    value: () => { const e = targetEffect(); return e ? (e.delay / 1000).toFixed(2) : ""; },
    run: () => secondsDialog("Animation Delay", (targetEffect()?.delay || 0) / 1000, 0, (delay) => setTiming(target(), { delay })),
  });
  register("animations.preview", { label: "Preview", enabled: () => has() && list.some((e) => e.node >= 0), run: previewAnimations });
  register("animations.painter", { label: "Animation Painter", enabled: () => has() && (painterFrom != null || !!targetEffect()), checked: () => painterFrom != null, run: armPainter });
  selection.onPaint((s) => paintTo(s));
  document.addEventListener("keydown", (e) => { if (e.key === "Escape" && painterFrom) { painterFrom = null; status(""); changed(); } }, true);
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
