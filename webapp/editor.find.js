// Home > Editing: Find (Ctrl+F) and Replace (Ctrl+H), and Home > Slides: Reset
// (`UX-006`).
//
// Find searches every text body of every slide, in slide order and then shape order,
// through the engine (`find_text`); Find Next goes to the next match's slide and
// selects its shape. Replace changes the match found and goes on; Replace All changes
// every match as one undo step. The text typed here and the matches are document
// content and reach the page through `value` and `textContent` only.
//
// Reset puts the slide's placeholders back on their layout's position, size and look,
// as PowerPoint's Reset Slide does; text stays.

import { register } from "./commands.js";
import { state, request, status, changed } from "./editor.core.js";
import { showModal, closeModal, errorDialog } from "./editor.dialogs.js";
import * as selection from "./editor.selection.js";
import * as slides from "./editor.slides.js";

const has = () => !!state.deck;
let last = { query: "", with: "", matchCase: false, at: -1, matches: [] };

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

async function textCall(fn, args) {
  const r = await request({ op: "text", fn, args }, { priority: 9 });
  if (r.undo) selection.noteUndoState(r.undo);
  return r.value;
}

async function search() {
  const v = await textCall("find_text", [last.query, last.matchCase]);
  const m = [];
  for (let i = 0; i + 4 < v.length; i += 5) m.push({ index: v[i], part: v[i + 1], node: v[i + 2], start: v[i + 3], end: v[i + 4] });
  last.matches = m;
  return m;
}

async function findNext(note) {
  if (!last.query) return;
  const m = await search();
  if (!m.length) { note.textContent = `"${last.query}" was not found.`; return; }
  // The first match after the one shown, from the current slide on, as PowerPoint.
  const from = last.at >= 0 ? last.at + 1 : m.findIndex((x) => x.index >= state.current);
  last.at = from >= 0 && from < m.length ? from : 0;
  const hit = m[last.at];
  if (hit.index !== state.current) slides.go(hit.index);
  selection.selectNode(hit.node);
  await selection.refresh();
  note.textContent = `Match ${last.at + 1} of ${m.length}, on slide ${hit.index + 1}.`;
}

async function replaceOne(note) {
  const hit = last.matches[last.at];
  if (!hit) { await findNext(note); return; }
  try {
    await textCall("replace_text_one", [hit.part, hit.node, hit.start, hit.end, last.with]);
    await slides.redrawSlide(hit.index);
    last.at -= 1;   // the next match now sits where this one's successor was
    await findNext(note);
  } catch (e) {
    errorDialog("OpenPresent could not replace that.", e.message);
  }
}

async function replaceAll(note) {
  if (!last.query) return;
  try {
    const n = await textCall("replace_text_all", [last.query, last.with, last.matchCase]);
    for (let i = 0; i < state.deck.count; i++) slides.redrawSlide(i);
    last.at = -1;
    note.textContent = n ? `Replaced ${n} ${n === 1 ? "match" : "matches"}.` : `"${last.query}" was not found.`;
    status(note.textContent);
  } catch (e) {
    errorDialog("OpenPresent could not replace those.", e.message);
  }
}

function dialog(replace) {
  showModal(replace ? "Replace" : "Find", (body) => {
    const row = (label, input) => { const l = h("label", "fp-row"); l.append(document.createTextNode(label), input); return l; };
    const what = h("input", "fp-input");
    what.type = "text";
    what.value = last.query;
    what.setAttribute("aria-label", "Find what");
    const withInput = h("input", "fp-input");
    withInput.type = "text";
    withInput.value = last.with;
    withInput.setAttribute("aria-label", "Replace with");
    const caseBox = h("input");
    caseBox.type = "checkbox";
    caseBox.checked = last.matchCase;
    const caseRow = h("label", "fp-row");
    caseRow.append(caseBox, document.createTextNode(" Match case"));
    const note = h("p", "fp-none");
    note.setAttribute("role", "status");
    const read = () => {
      if (what.value !== last.query || caseBox.checked !== last.matchCase) last.at = -1;
      last.query = what.value;
      last.with = withInput.value;
      last.matchCase = caseBox.checked;
    };
    const button = (label, primary, f) => {
      const b = h("button", primary ? "op-btn primary" : "op-btn", label);
      b.type = "button";
      b.addEventListener("click", () => { read(); f(note); });
      return b;
    };
    const buttons = h("div", "fd-buttons");
    buttons.append(button("Find Next", !replace, findNext));
    if (replace) buttons.append(button("Replace", false, replaceOne), button("Replace All", true, replaceAll));
    else buttons.append(button("Replace…", false, () => { closeModal(); dialog(true); }));
    body.append(row("Find what ", what));
    if (replace) body.append(row("Replace with ", withInput));
    body.append(caseRow, buttons, note);
    what.addEventListener("keydown", (e) => { if (e.key === "Enter") { e.preventDefault(); read(); findNext(note); } });
    setTimeout(() => { what.focus(); what.select(); }, 0);
  });
}

async function resetSlide() {
  const index = state.current;
  try {
    const r = await request({ op: "build", fn: "reset_slide", args: [index] }, { priority: 9 });
    if (r.undo) selection.noteUndoState(r.undo);
    await slides.redrawSlide(index);
    await selection.refresh();
    status(r.value ? "Slide reset to its layout" : "The slide already matches its layout");
    changed();
  } catch (e) {
    errorDialog("OpenPresent could not reset the slide.", e.message);
  }
}

export function init() {
  register("edit.find", { label: "Find", shortcut: "Ctrl+F", enabled: has, run: () => dialog(false) });
  register("edit.replace", { label: "Replace", shortcut: "Ctrl+H", enabled: has, run: () => dialog(true) });
  register("home.reset", { label: "Reset Slide", enabled: has, run: resetSlide });
}

/// Ctrl+F and Ctrl+H, from the editor's keys.
export function onKey(e) {
  const mod = e.ctrlKey || e.metaKey;
  if (!mod || e.altKey || e.shiftKey || !has()) return false;
  if (e.key === "f" || e.key === "F") { e.preventDefault(); dialog(false); return true; }
  if (e.key === "h" || e.key === "H") { e.preventDefault(); dialog(true); return true; }
  return false;
}
