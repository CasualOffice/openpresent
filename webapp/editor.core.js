// The editor core: its state, the engine client, and the commands it registers.
//
// Shaped after OpenCalc's `webapp/editor.core.js` at `b23c67c` (webapp/README.md):
// one module owns the document state and registers every verb in the command
// registry at boot, and the topic modules (`editor.files.js`, `editor.slides.js`,
// `editor.dialogs.js`) do one job each. Unlike OpenCalc's, the engine runs in a
// worker (`render.worker.js`), because drawing a slide is the heavy part here and
// a 500-slide deck must not freeze the window while its thumbnails draw.
//
// This is the first vertical slice (owner instruction 44): open a deck and view
// its slides. Every other PowerPoint command the ribbon names is registered
// disabled, saying so, and arrives working with its own slice.

import { register, run, all as allCommands } from "./commands.js";
import { start as startRibbon, queueSync, openBackstage, icon } from "./ribbon.js";
import { TABS, FILE_RAIL, modelCommandIds } from "./ribbon.model.js";
import { initFiles, openFilePicker } from "./editor.files.js";
import * as slides from "./editor.slides.js";
import { initDialogs, aboutDialog, reportDialog, shortcutsDialog, errorDialog } from "./editor.dialogs.js";

const byId = (id) => document.getElementById(id);

/// The document state. `deck` is `null` until one opens: `{ name, count, width,
/// height }`, the size in EMU.
export const state = {
  deck: null,
  current: 0,
  zoom: "fit",          // "fit" or a percentage
  notes: true,
  reading: false,
  engineVersion: "",
};

/* ── The engine client ────────────────────────────────────────────────────
 *
 * One request in the worker at a time, from a queue the page orders: the slide
 * the user is looking at goes before thumbnails, so a click is answered at once
 * even while a long deck's strip is still drawing. */

const worker = new Worker(new URL("./render.worker.js", import.meta.url), { type: "module" });
let seq = 0;
let inFlight = null;
const queue = [];          // { id, msg, transfer, priority, resolve, reject }

function pump() {
  if (inFlight || !queue.length) return;
  queue.sort((a, b) => b.priority - a.priority || a.id - b.id);
  inFlight = queue.shift();
  worker.postMessage({ id: inFlight.id, ...inFlight.msg }, inFlight.transfer || []);
}

worker.onmessage = (e) => {
  const r = e.data || {};
  const job = inFlight;
  inFlight = null;
  if (job && job.id === r.id) {
    if (r.ok) job.resolve(r); else job.reject(new Error(r.error || "the engine failed"));
  }
  pump();
};
worker.onerror = (e) => {
  e.preventDefault?.();
  const job = inFlight;
  inFlight = null;
  const err = new Error("The engine could not start in this browser.");
  if (job) job.reject(err);
  for (const j of queue.splice(0)) j.reject(err);
  status("The engine could not start in this browser.");
};

/// Ask the engine for something. `priority`: higher runs first.
export function request(msg, { transfer, priority = 0 } = {}) {
  return new Promise((resolve, reject) => {
    queue.push({ id: ++seq, msg, transfer, priority, resolve, reject });
    pump();
  });
}

/// Drop queued requests a newer view has made pointless (thumbnails of a deck
/// that is no longer open). The one in the worker finishes.
export function dropQueued(pred) {
  for (let i = queue.length - 1; i >= 0; i--) {
    if (pred(queue[i].msg)) queue.splice(i, 1)[0].reject(new Error("superseded"));
  }
}

/* ── Status ─────────────────────────────────────────────────────────────── */

export function status(text) { byId("tb-status").textContent = text; }

/// Repaint everything that reads the state: the registry's controls, the slide
/// indicator, the status bar's buttons.
export function changed() {
  const d = state.deck;
  byId("slide-indicator").textContent = d ? `Slide ${state.current + 1} of ${d.count}` : "";
  byId("notes-toggle").setAttribute("aria-pressed", state.notes ? "true" : "false");
  byId("notes-pane").hidden = !state.notes || state.reading;
  byId("zoom-level").textContent = state.zoom === "fit" ? "Fit" : `${state.zoom}%`;
  for (const id of ["zoom-in", "zoom-out", "zoom-level", "zoom-fit"]) byId(id).disabled = !d;
  document.documentElement.classList.toggle("op-reading", state.reading);
  document.documentElement.classList.toggle("op-has-deck", !!d);
  for (const b of byId("view-switch").querySelectorAll("button")) {
    const c = allCommands().find((x) => x.id === b.dataset.cmd);
    b.disabled = !c || !c.enabled();
    if (c?.checked) b.setAttribute("aria-pressed", c.checked() ? "true" : "false");
  }
  queueSync();
}

/* ── Opening ────────────────────────────────────────────────────────────── */

let opening = false;

/// Open a deck from its bytes. A deck the engine refuses leaves the one open
/// before in place, as the engine itself does.
export async function openDeck(name, bytes) {
  if (opening) return;
  opening = true;
  status(`Opening ${name}…`);
  byId("doc-state").textContent = "Opening…";
  try {
    const r = await request({ op: "open", bytes }, { transfer: [bytes], priority: 10 });
    state.deck = { name, count: r.count, width: r.width, height: r.height };
    state.engineVersion = r.version;
    state.current = 0;
    byId("doc-name").textContent = name;
    byId("doc-state").textContent = "Read-only";
    document.title = `${name} - OpenPresent`;
    status(r.count === 1 ? "1 slide" : `${r.count} slides`);
    slides.load();
  } catch (e) {
    byId("doc-state").textContent = state.deck ? "Read-only" : "";
    status(state.deck ? `${state.deck.count} slides` : "Ready");
    errorDialog(`OpenPresent can't read ${name}. It was read in this browser only, and nothing was uploaded.`, e.message);
  } finally {
    opening = false;
    changed();
  }
}

/* ── Commands ───────────────────────────────────────────────────────────── */

const hasDeck = () => !!state.deck;
const NOT_YET = "not available yet. This build opens and shows presentations";

function setTheme(v) {
  if (v === "auto") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = v;
  try { localStorage.setItem("op.theme", v); } catch { /* private mode */ }
}

function registerCommands() {
  register("file.open", { label: "Open", shortcut: "Ctrl+O", run: openFilePicker });
  register("file.render-report", { label: "What this slide leaves out", enabled: hasDeck, run: () => reportDialog(state.current) });

  register("slide.next", { label: "Next Slide", shortcut: "Page Down", enabled: () => hasDeck() && state.current < state.deck.count - 1, run: () => slides.go(state.current + 1) });
  register("slide.previous", { label: "Previous Slide", shortcut: "Page Up", enabled: () => hasDeck() && state.current > 0, run: () => slides.go(state.current - 1) });
  register("slide.first", { label: "First Slide", shortcut: "Home", enabled: hasDeck, run: () => slides.go(0) });
  register("slide.last", { label: "Last Slide", shortcut: "End", enabled: hasDeck, run: () => slides.go(state.deck.count - 1) });

  register("slideshow.from-beginning", { label: "From Beginning", shortcut: "F5", enabled: hasDeck, run: () => slides.startShow(0) });
  register("slideshow.from-current", { label: "From Current Slide", shortcut: "Shift+F5", enabled: hasDeck, run: () => slides.startShow(state.current) });

  register("view.normal", { label: "Normal", checked: () => !state.reading, run: () => { state.reading = false; changed(); slides.relayout(); } });
  register("view.reading", { label: "Reading View", enabled: hasDeck, checked: () => state.reading, run: () => { state.reading = !state.reading; changed(); slides.relayout(); } });
  register("view.notes", { label: "Notes", checked: () => state.notes, run: () => { state.notes = !state.notes; changed(); slides.relayout(); } });
  register("view.zoom", { label: "Zoom", enabled: hasDeck, run: () => {} });
  register("view.zoom.fit", { label: "Fit to Window", enabled: hasDeck, checked: () => state.zoom === "fit", run: () => slides.setZoom("fit") });
  for (const z of [50, 75, 100, 150, 200]) {
    register(`view.zoom.${z}`, { label: `${z}%`, enabled: hasDeck, checked: () => state.zoom === z, run: () => slides.setZoom(z) });
  }
  for (const [v, label] of [["auto", "Automatic"], ["light", "Light"], ["dark", "Dark"]]) {
    register(`view.theme.${v}`, { label: `Theme: ${label}`, run: () => setTheme(v) });
  }

  register("help.keyboard-shortcuts", { label: "Keyboard Shortcuts", run: shortcutsDialog });
  register("help.about", { label: "About OpenPresent", run: () => aboutDialog(state.engineVersion) });
  register("file.info", { label: "Info", run: () => openBackstage("Info") });

  // Every other command PowerPoint has and this build does not: present, and
  // disabled with the reason, never faked (owner, 2026-11-08).
  const labels = new Map();
  const walk = (it) => {
    if (it.kind === "rows") { it.rows.forEach((r) => r.forEach(walk)); return; }
    if (!labels.has(it.id)) labels.set(it.id, it.label.replace(/\s*\(.*\)\s*$/, ""));
  };
  for (const t of TABS) for (const g of t.groups) g.items.forEach(walk);
  for (const r of FILE_RAIL) for (const s of r.sections || []) for (const it of s.items) if (!labels.has(it.id)) labels.set(it.id, it.label);
  const built = new Set(allCommands().map((c) => c.id));
  for (const id of [...modelCommandIds(), "edit.undo", "edit.redo", "home.duplicate-slide", "home.reuse-slides"]) {
    if (built.has(id)) continue;
    const label = labels.get(id) || ({ "edit.undo": "Undo", "edit.redo": "Redo", "home.duplicate-slide": "Duplicate Slide", "home.reuse-slides": "Reuse Slides" })[id] || id;
    register(id, { label, enabled: () => false, reason: NOT_YET, run: () => {} });
  }
}

/* ── Keys ───────────────────────────────────────────────────────────────── */

function typing(t) {
  return t && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName));
}

function bindKeys() {
  document.addEventListener("keydown", (e) => {
    if (typing(e.target) || slides.showing()) return;
    if (!byId("op-modal").hidden || document.documentElement.classList.contains("op-backstage-open")) return;
    const mod = e.ctrlKey || e.metaKey;
    if (mod && !e.altKey && (e.key === "o" || e.key === "O")) { e.preventDefault(); openFilePicker(); return; }
    if (e.key === "F5") { e.preventDefault(); if (state.deck) slides.startShow(e.shiftKey ? state.current : 0); return; }
    if (mod || e.altKey || !state.deck) return;
    // Over the ribbon's own controls the arrows are theirs.
    if (e.target.closest?.(".op-ribbon, .rb-pop")) return;
    // Space and Enter on a focused button press it; they are not "next slide".
    if (e.target.tagName === "BUTTON" && (e.key === " " || e.key === "Enter")) return;
    const step = { ArrowRight: 1, ArrowDown: 1, PageDown: 1, " ": 1, ArrowLeft: -1, ArrowUp: -1, PageUp: -1 }[e.key];
    if (step) { e.preventDefault(); slides.go(state.current + step); return; }
    if (e.key === "Home") { e.preventDefault(); slides.go(0); }
    else if (e.key === "End") { e.preventDefault(); slides.go(state.deck.count - 1); }
    else if (e.key === "Escape" && state.reading) { state.reading = false; changed(); slides.relayout(); }
  });
}

/* ── Status bar ─────────────────────────────────────────────────────────── */

/// PowerPoint's view buttons at the right of the status bar, each a command.
function buildStatusBar() {
  const views = byId("view-switch");
  for (const [id, ic, label] of [
    ["view.normal", "tab_desktop_multiple", "Normal"],
    ["view.slide-sorter", "grid", "Slide Sorter"],
    ["view.reading", "book_open", "Reading View"],
    ["slideshow.from-current", "play", "Slide Show (Shift+F5)"],
  ]) {
    const b = document.createElement("button");
    b.type = "button";
    b.className = "bar-btn";
    b.dataset.cmd = id;
    b.title = label;
    b.setAttribute("aria-label", label.replace(/\s*\(.*\)$/, ""));
    b.appendChild(icon(ic, 16));
    b.addEventListener("click", () => { run(id); changed(); });
    views.appendChild(b);
  }
  byId("zoom-fit").appendChild(icon("zoom_in", 14));
  byId("notes-toggle").addEventListener("click", () => run("view.notes"));
}

/* ── Boot ───────────────────────────────────────────────────────────────── */

async function boot() {
  try {
    const t = localStorage.getItem("op.theme");
    if (t && t !== "auto") document.documentElement.dataset.theme = t;
  } catch { /* private mode */ }
  registerCommands();
  initDialogs(request);
  slides.init();
  initFiles({ onOpen: openDeck });
  buildStatusBar();
  bindKeys();
  await startRibbon(byId("workspace"));
  changed();
  status("Ready");
}

boot();
