// The View tab's Presentation Views (`UX-006`): Normal, Outline View, Slide Sorter,
// Notes Page and Reading View, as PowerPoint switches between them.
//
// Slide Sorter is the thumbnail pane drawn as a grid in place of the stage; a slide
// is dragged to a new place there as in the pane (`editor.build.js`), and a double
// click opens it in Normal. Notes Page is the slide over its speaker notes on a page.
// Outline View puts each slide's title and body text in the left column, as
// PowerPoint's outline: a title line, and the body a paragraph to a line; leaving a
// field writes it back through the engine as one undo step (`text_replace_all`).
// The text is document content and reaches the page through `value` and
// `textContent` only. Reading View is `state.reading` (`editor.core.js`).

import { register } from "./commands.js";
import { state, request, changed, status } from "./editor.core.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";
import { errorDialog } from "./editor.dialogs.js";

const byId = (id) => document.getElementById(id);
const VIEWS = ["normal", "outline", "sorter", "notes"];
let view = "normal";
let host = null;
let token = 0;
let shownIndex = -1, shownCount = -1;   // what the page or the outline was drawn for

export function current() { return view; }

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

/// Switch to `next`. Reading View ends, as choosing a view in PowerPoint ends it.
export function setView(next) {
  if (!VIEWS.includes(next)) return;
  view = next;
  state.reading = false;
  const root = document.documentElement;
  for (const v of VIEWS) root.classList.toggle(`op-view-${v}`, v === next && v !== "normal");
  if (next !== "normal") selection.deselect?.();
  render();
  changed();
  slides.relayout();
}

function ensureHost() {
  if (host) return host;
  host = h("section", "view-host");
  host.id = "view-host";
  host.hidden = true;
  byId("workspace").appendChild(host);
  return host;
}

function render() {
  const el = ensureHost();
  const t = ++token;
  el.textContent = "";
  el.hidden = view !== "notes" && view !== "outline";
  el.dataset.view = view;
  if (!state.deck) return;
  shownIndex = state.current;
  shownCount = state.deck.count;
  if (view === "notes") renderNotesPage(el, t);
  else if (view === "outline") renderOutline(el, t);
  else if (view === "sorter") byId("thumbs").querySelector(".thumb[aria-selected='true']")?.scrollIntoView({ block: "nearest" });
}

/* ── Notes Page ─────────────────────────────────────────────────────────── */

async function renderNotesPage(el, t) {
  const page = h("div", "np-page");
  const pic = h("img", "np-slide");
  pic.alt = `Slide ${state.current + 1}`;
  const notes = h("p", "np-notes");
  page.append(pic, notes);
  el.appendChild(page);
  try {
    const index = state.current;
    const [png, r] = await Promise.all([
      request({ op: "png", index, width: Math.round(720 * Math.min(window.devicePixelRatio || 1, 2)) }, { priority: 8 }),
      request({ op: "notes", index }, { priority: 8 }),
    ]);
    if (t !== token) return;
    const url = URL.createObjectURL(new Blob([png.png], { type: "image/png" }));
    pic.addEventListener("load", () => URL.revokeObjectURL(url), { once: true });
    pic.src = url;
    notes.textContent = r.notes || "This slide has no speaker notes.";
    notes.classList.toggle("is-empty", !r.notes);
  } catch (e) {
    if (t === token) notes.textContent = `This page could not be drawn: ${e.message}`;
  }
}

/* ── Outline View ───────────────────────────────────────────────────────── */

async function call(fn, args) {
  const r = await request({ op: "text", fn, args }, { priority: 8 });
  if (r.undo) selection.noteUndoState(r.undo);
  return r.value;
}

async function textOf(part, node) {
  if (node < 0) return null;
  const n = await call("text_length", [part, node]);
  return n ? call("text_slice", [part, node, 0, n]) : "";
}

async function renderOutline(el, t) {
  const list = h("ol", "ov-list");
  list.setAttribute("aria-label", "Outline");
  el.appendChild(list);
  for (let i = 0; i < state.deck.count; i++) {
    let o, title, body;
    try {
      o = (await call("slide_outline", [i])).map(Number);
      [title, body] = await Promise.all([textOf(o[0], o[1]), textOf(o[0], o[2])]);
    } catch (e) {
      if (t !== token) return;
      list.appendChild(h("li", "ov-slide", `Slide ${i + 1} could not be read: ${e.message}`));
      continue;
    }
    if (t !== token) return;
    const li = h("li", "ov-slide");
    li.dataset.index = String(i);
    li.classList.toggle("is-current", i === state.current);
    li.appendChild(h("span", "ov-num", String(i + 1)));
    const fields = h("div", "ov-fields");
    if (title != null) fields.appendChild(field("input", title, `Slide ${i + 1} title`, i, o[0], o[1]));
    else fields.appendChild(h("span", "ov-none", "(no title placeholder)"));
    if (body != null) fields.appendChild(field("textarea", body, `Slide ${i + 1} text`, i, o[0], o[2]));
    li.appendChild(fields);
    list.appendChild(li);
  }
}

function field(tag, value, label, index, part, node) {
  const f = h(tag, tag === "input" ? "ov-title" : "ov-body");
  if (tag === "input") f.type = "text";
  else f.rows = Math.max(1, value.split("\n").length);
  f.value = value;
  f.setAttribute("aria-label", label);
  f.placeholder = tag === "input" ? "Click to add title" : "Click to add text";
  f.spellcheck = false;
  let saved = value;
  f.addEventListener("focus", () => { if (state.current !== index) slides.go(index); markCurrent(index); });
  f.addEventListener("input", () => { if (tag === "textarea") f.rows = Math.max(1, f.value.split("\n").length); });
  f.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (tag === "input" && e.key === "Enter") { e.preventDefault(); f.blur(); }
    if (e.key === "Escape") { f.value = saved; f.blur(); }
  });
  f.addEventListener("change", async () => {
    // A title is one line; a line feed there would start a second paragraph.
    const text = tag === "input" ? f.value.replace(/[\r\n]+/g, " ") : f.value.replace(/\r\n?/g, "\n");
    if (text === saved) return;
    try {
      await call("text_replace_all", [part, node, text]);
      saved = text;
      await slides.redrawSlide(index);
      status(`Slide ${index + 1} updated`);
    } catch (e) {
      f.value = saved;
      errorDialog("OpenPresent could not change that text.", e.message);
    }
  });
  return f;
}

function markCurrent(index) {
  for (const li of host?.querySelectorAll(".ov-slide") || []) li.classList.toggle("is-current", Number(li.dataset.index) === index);
}

/* ── Commands ───────────────────────────────────────────────────────────── */

export function init() {
  const has = () => !!state.deck;
  register("view.normal", { label: "Normal", checked: () => view === "normal" && !state.reading, run: () => setView("normal") });
  register("view.outline", { label: "Outline View", enabled: has, checked: () => view === "outline", run: () => setView("outline") });
  register("view.slide-sorter", { label: "Slide Sorter", enabled: has, checked: () => view === "sorter", run: () => setView("sorter") });
  register("view.notes-page", { label: "Notes Page", enabled: has, checked: () => view === "notes", run: () => setView("notes") });
  // Slide Sorter: a double click opens the slide in Normal, as PowerPoint's.
  byId("thumbs").addEventListener("dblclick", (e) => {
    if (view !== "sorter") return;
    const li = e.target.closest?.(".thumb");
    if (!li) return;
    slides.go(Number(li.dataset.index));
    setView("normal");
  });
  // Another slide, another deck, an undo: the page and outline follow.
  document.addEventListener("op-changed", () => {
    if (!state.deck) return;
    if (view === "notes" && shownIndex !== state.current) render();
    else if (view === "outline" && shownCount !== state.deck.count) render();
    else if (view === "outline") markCurrent(state.current);
  });
  document.addEventListener("op-slides-built", () => { if (view === "outline" || view === "notes") render(); });
}
