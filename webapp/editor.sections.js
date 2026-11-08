// Home > Section (`UX-006`): Add Section, Rename Section, Remove Section, Collapse
// All and Expand All, and each section's header in the thumbnail pane, which a click
// collapses or expands, as PowerPoint's.
//
// A header sits inside the thumbnail of the section's first slide, so the pane's
// items stay one a slide and every index into it holds. Section names are document
// text and reach the page through `textContent` and `value` only. Each edit is one
// engine op and one undo step; a section id is a GUID made here with the browser's
// `crypto.randomUUID`, as the engine takes randomness from its host.

import { register } from "./commands.js";
import { state, request, status } from "./editor.core.js";
import { showModal, closeModal, errorDialog } from "./editor.dialogs.js";
import * as selection from "./editor.selection.js";

const byId = (id) => document.getElementById(id);
const has = () => !!state.deck && state.deck.count > 0;
let sections = [];            // [{ name, first }]
const collapsed = new Set();  // by first slide's place, for this view only

const guid = () => `{${crypto.randomUUID().toUpperCase()}}`;

async function call(fn, args) {
  const r = await request({ op: "build", fn, args }, { priority: 8 });
  if (r.undo) selection.noteUndoState(r.undo);
  return r.value;
}

export async function refresh() {
  if (!state.deck) { sections = []; paint(); return; }
  try {
    const v = await call("slide_sections", []);
    const next = [];
    for (let i = 0; i + 1 < v.length; i += 2) next.push({ name: v[i], first: Number(v[i + 1]) });
    sections = next;
  } catch { sections = []; }
  paint();
}

/// The section holding slide `i`, by its place in `sections`; -1 for none.
function sectionOf(i) {
  let at = -1;
  sections.forEach((s, k) => { if (s.first <= i) at = k; });
  return at;
}

function paint() {
  const list = byId("thumbs");
  for (const h of list.querySelectorAll(".thumb-sec")) h.remove();
  for (const li of list.children) li.classList.remove("has-sec", "sec-hidden", "sec-collapsed");
  sections.forEach((s, k) => {
    const li = list.children[s.first];
    if (!li) return;
    const head = document.createElement("button");
    head.type = "button";
    head.className = "thumb-sec";
    const end = k + 1 < sections.length ? sections[k + 1].first : list.children.length;
    const shut = collapsed.has(s.first);
    head.setAttribute("aria-expanded", shut ? "false" : "true");
    head.textContent = `${shut ? "▸" : "▾"} ${s.name} (${end - s.first})`;
    head.title = shut ? "Expand Section" : "Collapse Section";
    head.addEventListener("click", (e) => {
      e.stopPropagation();
      if (collapsed.has(s.first)) collapsed.delete(s.first); else collapsed.add(s.first);
      paint();
    });
    head.addEventListener("dblclick", (e) => { e.stopPropagation(); renameDialog(k); });
    li.classList.add("has-sec");
    li.prepend(head);
    if (shut) {
      li.classList.add("sec-collapsed");
      for (let i = s.first + 1; i < end; i++) list.children[i]?.classList.add("sec-hidden");
    }
  });
}

function nameDialog(title, value, apply) {
  showModal(title, (body) => {
    const label = document.createElement("label");
    label.className = "fp-row";
    label.appendChild(document.createTextNode("Section name "));
    const input = document.createElement("input");
    input.type = "text";
    input.className = "fp-input";
    input.maxLength = 255;
    input.value = value;
    label.appendChild(input);
    const ok = document.createElement("button");
    ok.type = "button";
    ok.className = "op-btn primary";
    ok.textContent = title.startsWith("Rename") ? "Rename" : "Add";
    const go = () => { const v = input.value.trim(); if (!v) { input.focus(); return; } closeModal(); apply(v); };
    ok.addEventListener("click", go);
    input.addEventListener("keydown", (e) => { if (e.key === "Enter") { e.preventDefault(); go(); } });
    body.append(label, ok);
    setTimeout(() => { input.focus(); input.select(); }, 0);
  });
}

async function edit(what, fn, args) {
  try {
    await call(fn, args);
    await refresh();
  } catch (e) {
    errorDialog(`OpenPresent could not ${what}.`, e.message);
  }
}

function addDialog() {
  const at = state.current;
  if (sections.some((s) => s.first === at)) { status("This slide already starts a section"); return; }
  nameDialog("Add Section", "Untitled Section", (name) => edit("add the section", "section_add", [at, name, guid(), guid()]));
}

function renameDialog(k = sectionOf(state.current)) {
  const s = sections[k];
  if (!s) return;
  nameDialog("Rename Section", s.name, (name) => edit("rename the section", "section_rename", [k, name]));
}

function removeSection() {
  const k = sectionOf(state.current);
  if (k < 0) return;
  edit("remove the section", "section_remove", [k]);
}

export function init() {
  const inOne = () => has() && sectionOf(state.current) >= 0;
  register("slide.section.add", { label: "Add Section", enabled: has, run: addDialog });
  register("slide.section.rename", { label: "Rename Section", enabled: inOne, run: () => renameDialog() });
  register("slide.section.remove", { label: "Remove Section", enabled: inOne, run: removeSection });
  register("slide.section.collapse-all", { label: "Collapse All", enabled: () => has() && sections.length > 0, run: () => { for (const s of sections) collapsed.add(s.first); paint(); } });
  register("slide.section.expand-all", { label: "Expand All", enabled: () => has() && sections.length > 0, run: () => { collapsed.clear(); paint(); } });
  // A rebuilt pane, another deck, an undo or redo: read the list again.
  document.addEventListener("op-slides-built", () => { collapsed.clear(); refresh(); });
  // The read is a small list and draws nothing, so it follows every change, settled.
  let timer = 0;
  document.addEventListener("op-changed", () => {
    clearTimeout(timer);
    timer = setTimeout(refresh, 250);
  });
}
