// The slides: the thumbnail pane, the slide on the stage, zoom, and the slide
// show. This module is to a deck what OpenCalc's `editor.sheets.js` is to a
// workbook's sheets: it draws what the engine reports and keeps no model.
//
// Every picture is drawn by the engine (`casual-slide-render` through the
// worker) as PNG and shown as an `<img>` from a `blob:` URL. No document string
// reaches the DOM except the deck's file name, through `textContent`.

import { state, request, dropQueued, changed, status } from "./editor.core.js";
import { reportDialog } from "./editor.dialogs.js";

const byId = (id) => document.getElementById(id);
const EMU_PER_INCH = 914400;
/// The engine's widest picture (`MAX_WIDTH_PX` in `casual-slide-wasm`).
const MAX_WIDTH = 4096;
const THUMB_CSS_WIDTH = 168;

let generation = 0;         // moves on each deck, so late answers for an old one are dropped
let observer = null;
const thumbUrls = [];       // object URLs, revoked when a new deck opens
let stageUrl = null;
let stageToken = 0;
let relayoutTimer = 0;

const dpr = () => Math.min(window.devicePixelRatio || 1, 3);
const aspect = () => state.deck.height / state.deck.width;

/// Ask the engine for slide `index` drawn `width` device pixels wide; resolves
/// to a `blob:` URL, or `null` if it failed or the deck changed meanwhile.
async function draw(index, width, priority) {
  const gen = generation;
  try {
    const r = await request({ op: "png", index, width: Math.max(1, Math.min(MAX_WIDTH, Math.round(width))) }, { priority });
    if (gen !== generation) return null;
    return URL.createObjectURL(new Blob([r.png], { type: "image/png" }));
  } catch (e) {
    if (gen === generation && e.message !== "superseded") status(`Slide ${index + 1} could not be drawn: ${e.message}`);
    return null;
  }
}

/* ── Thumbnails ─────────────────────────────────────────────────────────── */

function buildThumbs() {
  const list = byId("thumbs");
  list.textContent = "";
  observer?.disconnect();
  // Drawn as they scroll into view, so a 500-slide deck costs only what is seen.
  observer = new IntersectionObserver((entries) => {
    for (const en of entries) {
      if (!en.isIntersecting) continue;
      const li = en.target;
      observer.unobserve(li);
      const i = Number(li.dataset.index);
      draw(i, THUMB_CSS_WIDTH * dpr(), 1).then((url) => {
        if (!url) return;
        thumbUrls.push(url);
        const img = li.querySelector("img");
        img.src = url;
        img.classList.add("is-drawn");
      });
    }
  }, { root: byId("slide-pane"), rootMargin: "400px 0px" });
  const ratio = `${state.deck.width} / ${state.deck.height}`;
  for (let i = 0; i < state.deck.count; i++) {
    const li = document.createElement("li");
    li.className = "thumb";
    li.dataset.index = String(i);
    li.setAttribute("role", "option");
    li.setAttribute("aria-label", `Slide ${i + 1}`);
    li.setAttribute("aria-selected", "false");
    const num = document.createElement("span");
    num.className = "thumb-num";
    num.textContent = String(i + 1);
    const frame = document.createElement("span");
    frame.className = "thumb-frame";
    frame.style.aspectRatio = ratio;   // CSSOM, which the CSP allows
    const img = document.createElement("img");
    img.alt = "";
    img.draggable = false;
    frame.appendChild(img);
    li.append(num, frame);
    li.addEventListener("click", () => go(i));
    list.appendChild(li);
    observer.observe(li);
  }
}

function markThumb() {
  const list = byId("thumbs");
  for (const li of list.querySelectorAll(".thumb[aria-selected='true']")) li.setAttribute("aria-selected", "false");
  const li = list.children[state.current];
  if (!li) return;
  li.setAttribute("aria-selected", "true");
  li.scrollIntoView({ block: "nearest" });
}

/* ── The stage ──────────────────────────────────────────────────────────── */

/// The slide's size on screen, in CSS pixels, for the zoom in force.
function stageCssWidth() {
  const stage = byId("stage");
  if (state.zoom === "fit") {
    const pad = 48;
    const w = Math.max(40, stage.clientWidth - pad);
    const h = Math.max(30, stage.clientHeight - pad);
    return Math.max(40, Math.min(w, h / aspect()));
  }
  return (state.deck.width / EMU_PER_INCH) * 96 * (state.zoom / 100);
}

async function drawStage() {
  if (!state.deck) return;
  const token = ++stageToken;
  const index = state.current;
  const css = stageCssWidth();
  const frame = byId("slide-frame");
  frame.style.width = `${css}px`;
  frame.style.height = `${css * aspect()}px`;
  const url = await draw(index, css * dpr(), 5);
  if (!url) return;
  if (token !== stageToken) { URL.revokeObjectURL(url); return; }
  const img = byId("slide-img");
  img.src = url;
  img.alt = `Slide ${index + 1}`;
  if (stageUrl) URL.revokeObjectURL(stageUrl);
  stageUrl = url;
  showReport(index);
}

/// The status bar's count of what this slide leaves out, which opens the list.
async function showReport(index) {
  const btn = byId("report-btn");
  try {
    const r = await request({ op: "report", index }, { priority: 4 });
    if (index !== state.current) return;
    const n = r.lines.length;
    btn.hidden = n === 0;
    btn.textContent = n === 1 ? "1 feature not drawn" : `${n} features not drawn`;
  } catch {
    btn.hidden = true;
  }
}

/// Redraw the stage after its size changed, once the change settles.
export function relayout() {
  clearTimeout(relayoutTimer);
  relayoutTimer = setTimeout(drawStage, 120);
}

/* ── Navigation ─────────────────────────────────────────────────────────── */

/// Show slide `i`, clamped to the deck.
export function go(i) {
  if (!state.deck) return;
  const next = Math.max(0, Math.min(state.deck.count - 1, i));
  if (next === state.current && byId("slide-img").src) return;
  state.current = next;
  markThumb();
  changed();
  drawStage();
}

export function setZoom(z) {
  if (!state.deck) return;
  state.zoom = z;
  byId("stage").classList.toggle("is-zoomed", z !== "fit");
  changed();
  drawStage();
}

const ZOOMS = [25, 50, 75, 100, 150, 200, 300];
function stepZoom(dir) {
  if (!state.deck) return;
  const current = state.zoom === "fit" ? (stageCssWidth() / ((state.deck.width / EMU_PER_INCH) * 96)) * 100 : state.zoom;
  const next = dir > 0 ? ZOOMS.find((z) => z > current + 0.5) : [...ZOOMS].reverse().find((z) => z < current - 0.5);
  if (next) setZoom(next);
}

/// A new deck is open: rebuild the pane and show its first slide.
export function load() {
  generation++;
  dropQueued((m) => m.op === "png" || m.op === "report");
  for (const u of thumbUrls.splice(0)) URL.revokeObjectURL(u);
  if (stageUrl) { URL.revokeObjectURL(stageUrl); stageUrl = null; }
  byId("slide-img").removeAttribute("src");
  byId("report-btn").hidden = true;
  byId("empty-state").hidden = true;
  byId("slide-frame").hidden = false;
  state.current = 0;
  buildThumbs();
  markThumb();
  changed();
  drawStage();
  byId("stage").focus({ preventScroll: true });
}

/* ── Slide show ─────────────────────────────────────────────────────────── */

let showIndex = 0;
let showUrl = null;
let showToken = 0;

export function showing() { return !byId("show").hidden; }

async function drawShow() {
  const show = byId("show");
  const img = byId("show-img");
  const token = ++showToken;
  if (showIndex >= state.deck.count) {
    // PowerPoint's last screen: black, saying the show is over.
    img.hidden = true;
    show.dataset.end = "End of slide show, click to exit.";
    show.classList.add("is-end");
    return;
  }
  show.classList.remove("is-end");
  img.hidden = false;
  const w = Math.min(show.clientWidth || window.innerWidth, (show.clientHeight || window.innerHeight) / aspect());
  const url = await draw(showIndex, w * dpr(), 6);
  if (!url || token !== showToken) { if (url) URL.revokeObjectURL(url); return; }
  img.src = url;
  img.alt = `Slide ${showIndex + 1}`;
  if (showUrl) URL.revokeObjectURL(showUrl);
  showUrl = url;
}

/// Start the slide show at slide `from`.
export async function startShow(from) {
  if (!state.deck) return;
  const show = byId("show");
  showIndex = from;
  show.hidden = false;
  show.focus();
  try { await show.requestFullscreen?.(); } catch { /* shown as a window-filling layer instead */ }
  drawShow();
}

function endShow() {
  const show = byId("show");
  if (show.hidden) return;
  show.hidden = true;
  if (document.fullscreenElement) document.exitFullscreen?.().catch(() => {});
  if (showUrl) { URL.revokeObjectURL(showUrl); showUrl = null; }
  byId("show-img").removeAttribute("src");
  go(Math.min(showIndex, state.deck.count - 1));
  byId("stage").focus({ preventScroll: true });
}

function showStep(d) {
  if (showIndex + d < 0) return;
  if (showIndex >= state.deck.count && d > 0) { endShow(); return; }
  showIndex += d;
  drawShow();
}

/* ── Wiring ─────────────────────────────────────────────────────────────── */

export function init() {
  new ResizeObserver(() => { if (state.deck) relayout(); }).observe(byId("stage"));
  byId("zoom-in").addEventListener("click", () => stepZoom(1));
  byId("zoom-out").addEventListener("click", () => stepZoom(-1));
  byId("zoom-level").addEventListener("click", () => setZoom("fit"));
  byId("zoom-fit").addEventListener("click", () => setZoom("fit"));
  byId("report-btn").addEventListener("click", () => reportDialog(state.current));
  byId("thumbs").addEventListener("keydown", (e) => {
    const step = { ArrowDown: 1, ArrowRight: 1, ArrowUp: -1, ArrowLeft: -1 }[e.key];
    if (!step || !state.deck) return;
    e.preventDefault();
    e.stopPropagation();
    go(state.current + step);
  });
  // The stage: a click on the slide's left or right part moves, as a reading view.
  byId("stage").addEventListener("click", (e) => {
    if (!state.deck || state.zoom !== "fit" || !e.target.closest("#slide-frame")) return;
    const r = byId("slide-frame").getBoundingClientRect();
    go(state.current + (e.clientX < r.left + r.width / 3 ? -1 : 1));
  });
  byId("stage").addEventListener("wheel", (e) => {
    if (!state.deck || state.zoom !== "fit" || e.ctrlKey || Math.abs(e.deltaY) < 30) return;
    e.preventDefault();
    go(state.current + (e.deltaY > 0 ? 1 : -1));
  }, { passive: false });

  const show = byId("show");
  show.addEventListener("click", () => showStep(1));
  show.addEventListener("contextmenu", (e) => { e.preventDefault(); showStep(-1); });
  show.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.key === "Escape") { e.preventDefault(); endShow(); return; }
    const step = { ArrowRight: 1, ArrowDown: 1, PageDown: 1, " ": 1, Enter: 1, n: 1, ArrowLeft: -1, ArrowUp: -1, PageUp: -1, Backspace: -1, p: -1 }[e.key];
    if (step) { e.preventDefault(); showStep(step); }
    else if (e.key === "Home") { e.preventDefault(); showIndex = 0; drawShow(); }
    else if (e.key === "End") { e.preventDefault(); showIndex = state.deck.count - 1; drawShow(); }
  });
  // Leaving full screen with the browser's own Escape ends the show too.
  document.addEventListener("fullscreenchange", () => { if (!document.fullscreenElement && showing()) endShow(); });
  window.addEventListener("resize", () => { if (showing()) drawShow(); });
}
