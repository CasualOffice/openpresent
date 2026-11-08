// The task pane host (`UX-011`): one pane at the right of the slide, as PowerPoint
// for the web shows Format Shape, Comments, the Animation Pane and the Selection Pane.
//
// Copied in approach from OpenCalc's side panel (`../opencalc/webapp/editor.core.js`,
// `openPanel`/`closePanel` at `side-panel`, and its `.side-panel` styles): one host
// element with a title, a close button and a body; opening another tool re-targets the
// host rather than stacking a second panel; closing gives the focus back to the work.
// Added here: a registry, so a pane is one module that registers `{ title, build,
// refresh, closed }`; a resize handle on the left edge whose width is remembered in
// this browser; Esc closes; Ctrl+F6 reaches it (`editor.menus.js`).
//
// A pane builds its content into the body it is given each time it opens, and is told
// to refresh when the deck changes. Document text reaches the DOM through `textContent`.

import { changed } from "./editor.core.js";
import { icon } from "./ribbon.js";

const byId = (id) => document.getElementById(id);
const WIDTH_KEY = "openpresent.paneWidth";
const MIN_WIDTH = 240;
const MAX_WIDTH = 560;

const registry = new Map();
let host = null;
let active = null;
let lastFocus = null;

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

function savedWidth() {
  let w = 300;
  try { w = Number(localStorage.getItem(WIDTH_KEY)) || 300; } catch { /* storage off: the default */ }
  return Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, w));
}

function build() {
  host = h("aside", "task-pane pane-host");
  host.id = "pane-host";
  host.hidden = true;
  host.tabIndex = -1;
  host.style.width = `${savedWidth()}px`;
  const grip = h("div", "pane-grip");
  grip.setAttribute("role", "separator");
  grip.setAttribute("aria-orientation", "vertical");
  grip.setAttribute("aria-label", "Resize the pane");
  grip.tabIndex = 0;
  const setWidth = (w) => {
    const v = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, Math.round(w)));
    host.style.width = `${v}px`;
    try { localStorage.setItem(WIDTH_KEY, String(v)); } catch { /* not remembered */ }
  };
  grip.addEventListener("pointerdown", (e) => {
    e.preventDefault();
    grip.setPointerCapture(e.pointerId);
    const startX = e.clientX, startW = host.getBoundingClientRect().width;
    const move = (ev) => setWidth(startW + (startX - ev.clientX));
    const up = () => { grip.removeEventListener("pointermove", move); grip.removeEventListener("pointerup", up); };
    grip.addEventListener("pointermove", move);
    grip.addEventListener("pointerup", up);
  });
  grip.addEventListener("keydown", (e) => {
    const w = host.getBoundingClientRect().width;
    if (e.key === "ArrowLeft") { setWidth(w + 16); e.preventDefault(); }
    else if (e.key === "ArrowRight") { setWidth(w - 16); e.preventDefault(); }
  });
  const head = h("div", "fp-head");
  const title = h("h2", "fp-title");
  title.id = "pane-host-title";
  host.setAttribute("aria-labelledby", title.id);
  const x = h("button", "fp-close");
  x.type = "button";
  x.title = "Close (Esc)";
  x.appendChild(icon("dismiss", 16));
  x.addEventListener("click", () => close());
  head.append(title, x);
  const body = h("div", "pane-body");
  host.append(grip, head, body);
  host.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !e.defaultPrevented) { e.stopPropagation(); close(); }
  });
  byId("workspace").appendChild(host);
}

/// Register a pane: `{ title, build(body, opts), refresh?(), closed?() }`.
export function registerPane(id, spec) { registry.set(id, spec); }

/// The open pane's id, or null.
export const activePane = () => active;
export const isOpen = (id) => active === id;

/// Open pane `id`, replacing whatever pane is open, as PowerPoint switches its task
/// pane. `opts` goes to the pane's `build`. The focus moves into the pane.
export function open(id, opts = {}) {
  const spec = registry.get(id);
  if (!spec) return;
  if (!host) build();
  if (active && active !== id) registry.get(active)?.closed?.();
  if (!active) lastFocus = document.activeElement;
  active = id;
  host.dataset.pane = id;
  byId("pane-host-title").textContent = spec.title;
  host.querySelector(".fp-close").setAttribute("aria-label", `Close the ${spec.title} pane`);
  const body = host.querySelector(".pane-body");
  body.textContent = "";
  host.hidden = false;
  spec.build(body, opts);
  changed();
  if (opts.focus !== false) {
    const first = body.querySelector("[data-autofocus]") || host.querySelector(".fp-close");
    first?.focus({ preventScroll: true });
  }
}

/// Close the open pane, or pane `id` only if it is the one open.
export function close(id) {
  if (!host || !active || (id && id !== active)) return;
  const spec = registry.get(active);
  active = null;
  host.hidden = true;
  host.querySelector(".pane-body").textContent = "";
  spec?.closed?.();
  changed();
  const back = lastFocus && document.contains(lastFocus) ? lastFocus : byId("stage");
  back?.focus({ preventScroll: true });
}

/// Open pane `id`, or close it if it is the one open.
export function toggle(id, opts) { if (active === id) close(); else open(id, opts); }

/// Ask the open pane to show the deck as it is now.
export function refresh() {
  if (active) registry.get(active)?.refresh?.();
}
