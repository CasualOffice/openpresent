// The slide show and Presenter View (`VS-07`): presenting as PowerPoint does.
//
// From Beginning (F5) and From Current Slide (Shift+F5) fill the screen with one
// slide; Presenter View (Alt+F5) adds a second window with the current and next
// slide, the speaker notes, a timer and the slide count. The keys are
// PowerPoint's: next by click, Space, Enter, N, Right, Down or Page Down; back by
// P, Backspace, Left, Up or Page Up; a number then Enter jumps to that slide; B
// or `.` blacks the screen and W or `,` whites it; Esc or `-` ends. Hidden slides
// (`@show="0"`) are skipped, as PowerPoint skips them, unless jumped to by number.
//
// Every slide with a transition gets a simple fade over the transition's own
// duration. Any effect other than a fade or a cut is played as that fade, and
// when the show ends the status bar names which (`VS-07`; the effects are a later
// slice). Animations do not play; each slide shows as it finally stands.
//
// The deck stays in the engine. This module asks the worker for pictures, the
// show plan and the notes, and keeps no model. The notes, document text, reach
// the presenter window through `textContent` only (`presenter.js`).

import { state, request, status, changed } from "./editor.core.js";

const byId = (id) => document.getElementById(id);
const MAX_WIDTH = 4096;
const SHOW_PRIORITY = 6;
const PRESENTER_PRIORITY = 4;

/// The show in progress, or `null`. `index` past the last slide is the black
/// "end of slide show" screen.
let show = null;
let token = 0;
let presenter = null;   // the Presenter View window, while it is open

const dpr = () => Math.min(window.devicePixelRatio || 1, 3);
const aspect = () => state.deck.height / state.deck.width;

export function showing() { return show !== null; }

async function picture(index, width, priority) {
  try {
    const r = await request({ op: "png", index, width: Math.max(1, Math.min(MAX_WIDTH, Math.round(width))) }, { priority });
    return URL.createObjectURL(new Blob([r.png], { type: "image/png" }));
  } catch (e) {
    if (e.message !== "superseded") status(`Slide ${index + 1} could not be drawn: ${e.message}`);
    return null;
  }
}

/* ── The plan ───────────────────────────────────────────────────────────── */

/// Which slides are hidden, and each one's transition, from the engine.
async function loadPlan() {
  const count = state.deck.count;
  const plan = Array.from({ length: count }, () => ({ hidden: false, transition: false, asIs: true, ms: 0, effect: "" }));
  try {
    const r = await request({ op: "plan" }, { priority: 9 });
    for (let i = 0; i < count; i++) {
      const flags = r.plan[i * 2] | 0;
      plan[i] = { hidden: !!(flags & 1), transition: !!(flags & 2), asIs: !!(flags & 4), ms: r.plan[i * 2 + 1] | 0, effect: r.effects?.[i] || "" };
    }
  } catch (e) {
    // Without the plan the show still runs: every slide, no transitions.
    status(`The slide show could not read which slides are hidden: ${e.message}`);
  }
  return plan;
}

const visible = (i) => show && i >= 0 && i < show.plan.length && !show.plan[i].hidden;

/// The next slide shown after `i` in direction `d`, skipping hidden ones; the
/// end screen (`count`) past the last, and `-1` before the first.
function step(i, d) {
  const n = show.plan.length;
  for (let j = i + d; j >= 0 && j < n; j += d) if (visible(j)) return j;
  return d > 0 ? n : -1;
}

/* ── Drawing ────────────────────────────────────────────────────────────── */

const layers = () => [byId("show-img"), byId("show-fade")];

function clearLayer(img) {
  img.hidden = true;
  img.classList.remove("is-in");
  img.removeAttribute("src");
}

/// The bottom picture becomes `url`; the one it showed before is released.
function setBase(url, alt) {
  const base = byId("show-img");
  if (show.baseUrl && show.baseUrl !== url) URL.revokeObjectURL(show.baseUrl);
  show.baseUrl = url;
  base.src = url;
  base.alt = alt;
  base.hidden = false;
}

/// Drop a fade in progress, and release its picture.
function clearOver() {
  if (show.overUrl && show.overUrl !== show.baseUrl) URL.revokeObjectURL(show.overUrl);
  show.overUrl = null;
  clearLayer(byId("show-fade"));
}

/// Show `show.index`; `fade` plays the slide's transition into it.
async function draw(fade) {
  const box = byId("show");
  const [base, over] = layers();
  const t = ++token;
  box.classList.toggle("is-end", show.index >= show.plan.length);
  if (show.index >= show.plan.length) {
    clearOver();
    if (show.baseUrl) URL.revokeObjectURL(show.baseUrl);
    show.baseUrl = null;
    clearLayer(base);
    updatePresenter();
    return;
  }
  const index = show.index;
  const w = Math.min(box.clientWidth || window.innerWidth, (box.clientHeight || window.innerHeight) / aspect());
  const url = await picture(index, w * dpr(), SHOW_PRIORITY);
  if (!url || t !== token || !show) { if (url) URL.revokeObjectURL(url); return; }
  const p = show.plan[index];
  if (p.transition && !p.asIs && p.effect) show.standIns.add(p.effect);
  const alt = `Slide ${index + 1}`;
  clearOver();
  if (fade && p.transition && p.ms > 0 && !base.hidden) {
    // The new slide fades in over the one shown, then takes its place.
    show.overUrl = url;
    over.style.transitionDuration = `${p.ms}ms`;
    over.src = url;
    over.alt = alt;
    over.hidden = false;
    void over.offsetWidth;          // start from transparent
    over.classList.add("is-in");
    const done = () => {
      if (!show || show.overUrl !== url) return;
      setBase(url, alt);
      show.overUrl = null;
      clearLayer(over);
    };
    over.addEventListener("transitionend", done, { once: true });
    setTimeout(done, p.ms + 100);   // a fade the browser skipped still ends
  } else {
    setBase(url, alt);
  }
  updatePresenter();
}

function setBlank(colour) {
  show.blank = colour;
  const box = byId("show");
  box.classList.toggle("is-black", colour === "black");
  box.classList.toggle("is-white", colour === "white");
  updatePresenter();
}

/* ── Moving ─────────────────────────────────────────────────────────────── */

function next() {
  if (show.index >= show.plan.length) { end(); return; }
  show.index = step(show.index, 1);
  draw(true);
}

function previous() {
  const j = step(Math.min(show.index, show.plan.length), -1);
  if (j < 0) return;
  show.index = j;
  draw(false);
}

/// Jump to slide `n`, counted from 1, hidden or not, as PowerPoint does.
function jump(n) {
  if (!(n >= 1 && n <= show.plan.length)) return;
  show.index = n - 1;
  draw(true);
}

/// Start the slide show at slide `from` (from 0). From the beginning, the first
/// slide that is not hidden; from the current slide, that slide even if hidden.
export async function start(from, { presenterView = false } = {}) {
  if (!state.deck || show) return;
  // Opened before anything is awaited, so the browser counts it as the user's click.
  if (presenterView) openPresenter();
  const box = byId("show");
  show = { plan: [], index: 0, baseUrl: null, overUrl: null, blank: null, typed: "", standIns: new Set(), started: Date.now(), from };
  box.hidden = false;
  box.focus();
  changed();
  try { await box.requestFullscreen?.(); } catch { /* shown as a window-filling layer instead */ }
  const plan = await loadPlan();
  if (!show) return;
  show.plan = plan;
  show.index = from === 0 && plan.length && plan[0].hidden ? step(0, 1) : Math.min(from, plan.length);
  draw(false);
}

/// End the show, back to Normal view at the slide last shown.
export function end() {
  if (!show) return;
  const last = Math.min(show.index, show.plan.length - 1);
  const standIns = [...show.standIns].sort();
  const urls = [show.baseUrl, show.overUrl];
  show = null;
  token++;
  const box = byId("show");
  box.hidden = true;
  box.classList.remove("is-end", "is-black", "is-white");
  for (const img of layers()) clearLayer(img);
  for (const u of new Set(urls)) if (u) URL.revokeObjectURL(u);
  if (document.fullscreenElement) document.exitFullscreen?.().catch(() => {});
  closePresenter();
  onEnd(Math.max(0, last));
  changed();
  // What the show stood in for, so it is never silent (`VS-07`).
  if (standIns.length) status(`Shown as a simple fade, not drawn yet: ${standIns.join(", ")}`);
}

let onEnd = () => {};

/* ── Keys ───────────────────────────────────────────────────────────────── */

const NEXT = new Set(["ArrowRight", "ArrowDown", "PageDown", " ", "n", "N"]);
const BACK = new Set(["ArrowLeft", "ArrowUp", "PageUp", "Backspace", "p", "P"]);

/// One key, from the show itself or the presenter window. True if it was the show's.
export function key(e) {
  if (!show) return false;
  const k = e.key;
  if (k === "Escape" || k === "-") { end(); return true; }
  if (!show.plan.length) return true;   // the plan is still loading
  if (/^[0-9]$/.test(k)) { show.typed = (show.typed + k).slice(-6); return true; }
  if (k === "Enter" && show.typed) { const n = Number(show.typed); show.typed = ""; if (show.blank) setBlank(null); jump(n); return true; }
  show.typed = "";
  if (k === "b" || k === "B" || k === ".") { setBlank(show.blank === "black" ? null : "black"); return true; }
  if (k === "w" || k === "W" || k === ",") { setBlank(show.blank === "white" ? null : "white"); return true; }
  // On a blank screen any other key brings the slide back, and does nothing else.
  if (show.blank && (NEXT.has(k) || BACK.has(k) || k === "Enter" || k === "Home" || k === "End")) { setBlank(null); return true; }
  if (NEXT.has(k) || k === "Enter") { next(); return true; }
  if (BACK.has(k)) { previous(); return true; }
  if (k === "Home") { show.index = step(-1, 1); draw(false); return true; }
  if (k === "End") { const j = step(show.plan.length, -1); if (j >= 0) { show.index = j; draw(false); } return true; }
  return false;
}

function click(forward) {
  if (!show || !show.plan.length) return;
  if (show.blank) { setBlank(null); return; }
  if (forward) next(); else previous();
}

/* ── Presenter View ─────────────────────────────────────────────────────── */

let presenterUrls = [];

function openPresenter() {
  if (presenter && !presenter.closed) { presenter.focus(); return; }
  let w = null;
  try { w = window.open("./presenter.html", "openpresent-presenter", "popup,width=1100,height=700"); } catch { w = null; }
  if (!w) { status("Presenter View needs a second window, and this browser blocked it"); return; }
  presenter = w;
}

function closePresenter() {
  for (const u of presenterUrls.splice(0)) URL.revokeObjectURL(u);
  if (presenter && !presenter.closed) presenter.close();
  presenter = null;
}

let presenterToken = 0;

/// Bring the presenter window up to date: current and next slide, notes, count.
async function updatePresenter() {
  const api = presenter && !presenter.closed ? presenter.openpresentPresenter : null;
  if (!api || !show) return;
  const t = ++presenterToken;
  const n = show.plan.length;
  const i = show.index;
  const nextIndex = i < n ? step(i, 1) : n;
  const view = {
    started: show.started,
    index: i,
    count: n,
    ended: i >= n,
    blank: show.blank,
    effect: i < n && show.plan[i].transition && !show.plan[i].asIs ? show.plan[i].effect : "",
    full: false,
    current: null,
    next: null,
    nextLabel: nextIndex < n ? `Slide ${nextIndex + 1}` : "End of slide show",
    notes: "",
  };
  api.update(view);   // the count and timer at once; pictures and notes follow
  const [current, upcoming, notes] = await Promise.all([
    i < n ? picture(i, 960 * dpr(), PRESENTER_PRIORITY) : null,
    nextIndex < n ? picture(nextIndex, 480 * dpr(), PRESENTER_PRIORITY) : null,
    i < n ? request({ op: "notes", index: i }, { priority: PRESENTER_PRIORITY }).then((r) => r.notes, () => "") : "",
  ]);
  const fresh = [current, upcoming].filter(Boolean);
  if (t !== presenterToken || !presenter || presenter.closed || !show) { for (const u of fresh) URL.revokeObjectURL(u); return; }
  const old = presenterUrls;
  presenterUrls = fresh;
  presenter.openpresentPresenter?.update({ ...view, full: true, current, next: upcoming, notes });
  // Kept one update longer than shown, so the window never shows a revoked picture.
  setTimeout(() => { for (const u of old) URL.revokeObjectURL(u); }, 1000);
}

/// What `presenter.js` calls on the window that opened it.
const presenterBridge = {
  ready() { updatePresenter(); },
  key(k) { key({ key: k }); },
  next() { click(true); },
  previous() { click(false); },
  end() { end(); },
};

/* ── Wiring ─────────────────────────────────────────────────────────────── */

/// `after(index)` is called with the slide last shown when the show ends.
export function init(after) {
  onEnd = after;
  globalThis.openpresentShow = presenterBridge;
  const box = byId("show");
  box.addEventListener("click", () => click(true));
  box.addEventListener("contextmenu", (e) => { e.preventDefault(); click(false); });
  box.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    if (key(e)) e.preventDefault();
  });
  // Leaving full screen with the browser's own Escape ends the show too.
  // With Presenter View open, moving to its window may leave full screen; the show goes on.
  document.addEventListener("fullscreenchange", () => {
    if (!document.fullscreenElement && show && !(presenter && !presenter.closed)) end();
  });
  window.addEventListener("resize", () => { if (show && show.plan.length) draw(false); });
  window.addEventListener("pagehide", closePresenter);
}
