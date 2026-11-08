// The File tab's own verbs (`FILE-01`): rename in the title bar, File > Info's
// properties and Advanced Properties, Save As, and Print.
//
// Rename is OpenCalc's title-strip rename (`webapp/editor.core.js`, `UX-CHR-10`, at
// `b23c67c`; webapp/README.md): the name is a button, swapped for an input while
// renaming, Enter or a click away commits and Esc cancels. Properties are the engine's
// (`casual-slide-wasm` `file.rs`): each field change is one edit with an undo step, and
// a save patches `docProps` through the writer. Every value from the file reaches the
// page through `textContent` or an input's `value`, never as markup.

import { register, run } from "./commands.js";
import { state, request, status, changed } from "./editor.core.js";
import { download } from "./editor.files.js";
import { showModal, closeModal, errorDialog } from "./editor.dialogs.js";
import { noteUndoState } from "./editor.selection.js";

const byId = (id) => document.getElementById(id);

function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

/* ── The name ──────────────────────────────────────────────────────────── */

/// The types a deck may be saved as, PowerPoint's names for them, and their MIME type.
const TYPES = {
  pptx: ["PowerPoint Presentation", "application/vnd.openxmlformats-officedocument.presentationml.presentation"],
  potx: ["PowerPoint Template", "application/vnd.openxmlformats-officedocument.presentationml.template"],
  ppsx: ["PowerPoint Show", "application/vnd.openxmlformats-officedocument.presentationml.slideshow"],
  pptm: ["PowerPoint Macro-Enabled Presentation", "application/vnd.ms-powerpoint.presentation.macroEnabled.12"],
  potm: ["PowerPoint Macro-Enabled Template", "application/vnd.ms-powerpoint.template.macroEnabled.12"],
  ppsm: ["PowerPoint Macro-Enabled Show", "application/vnd.ms-powerpoint.slideshow.macroEnabled.12"],
};

/// The longest name kept, in characters.
const MAX_NAME = 200;

/// The extension of `name`, if it is one of [`TYPES`].
export function extensionOf(name) {
  const m = /\.([A-Za-z]{4})$/.exec(name || "");
  const ext = m ? m[1].toLowerCase() : "";
  return Object.hasOwn(TYPES, ext) ? ext : "";
}

/// `name` without its presentation extension.
export function stemOf(name) {
  const ext = extensionOf(name);
  return ext ? name.slice(0, -(ext.length + 1)) : name;
}

/// A name a user typed, made safe to be a file name: no path, no characters a file
/// system refuses, no control characters, bounded; empty when nothing is left.
function cleanName(typed) {
  return String(typed || "")
    .replace(/[\u0000-\u001f\u007f]/g, "")
    .replace(/[\\/:*?"<>|]/g, "-")
    .replace(/^[\s.]+|[\s.]+$/g, "")
    .slice(0, MAX_NAME);
}

/// Give the open deck a new name. Its extension stays the deck's unless the typed
/// name carries one of the presentation types. Returns the name it now has.
export function rename(typed) {
  const d = state.deck;
  if (!d) return "";
  const cleaned = cleanName(typed);
  if (!cleaned) return d.name;
  const ext = extensionOf(cleaned) || extensionOf(d.name) || "pptx";
  const name = `${stemOf(cleaned) || stemOf(d.name)}.${ext}`;
  d.name = name;
  byId("doc-name").textContent = name;
  document.title = `${name} - OpenPresent`;
  status(`Renamed to ${name}`);
  changed();
  return name;
}

/// Start renaming in the title bar, as PowerPoint for the web's file name does.
export function startRename() {
  const nameEl = byId("doc-name");
  const input = byId("doc-rename");
  if (!state.deck || !nameEl || !input) return;
  input.value = stemOf(state.deck.name);
  // Sized to the label it replaces, so the strip does not jump (OpenCalc's rule).
  input.style.width = `${Math.min(420, Math.max(160, nameEl.offsetWidth + 28))}px`;
  input.hidden = false;
  nameEl.hidden = true;
  input.focus();
  input.select();
}

function initRename() {
  const nameEl = byId("doc-name");
  const input = byId("doc-rename");
  const stop = (commit) => {
    if (input.hidden) return;
    const typed = input.value;   // read before hiding
    input.hidden = true;
    nameEl.hidden = false;
    if (commit) rename(typed);
    byId("stage")?.focus({ preventScroll: true });
  };
  nameEl.addEventListener("click", startRename);
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter") { e.preventDefault(); stop(true); }
    else if (e.key === "Escape") { e.preventDefault(); stop(false); }
    // The editor's keys must not see a keystroke meant for the name.
    e.stopPropagation();
  });
  input.addEventListener("blur", () => stop(true));
}

/* ── File > Info: properties ───────────────────────────────────────────── */

/// The summary fields, in PowerPoint's Info order, and Advanced Properties' Summary.
const FIELDS = [
  ["title", "Title", "Add a title"],
  ["tags", "Tags", "Add a tag"],
  ["comments", "Comments", "Add comments"],
  ["categories", "Categories", "Add a category"],
  ["subject", "Subject", "Specify the subject"],
  ["company", "Company", "Specify the company"],
  ["manager", "Manager", "Specify the manager"],
  ["author", "Author", "Add an author"],
];

/// One engine call for File > Info. An edit answers the undo state, which goes to the
/// one history's buttons (`noteUndoState`).
function fileCall(fn, args) {
  return request({ op: "file", fn, args }, { priority: 6 });
}

async function readInfo() {
  const r = await fileCall("document_info", []);
  return JSON.parse(r.value);
}

/// A W3CDTF date as the reader's locale writes it, or the text as stored.
function dateLabel(s) {
  if (!s) return "";
  const d = new Date(s);
  return Number.isFinite(d.getTime()) ? d.toLocaleString() : s;
}

/// Set one summary field; the Info panel and the modal both commit through here.
async function commitField(key, value) {
  try {
    const r = await fileCall("set_document_property", [key, value]);
    if (r.value) {
      status(`${FIELDS.find((f) => f[0] === key)?.[1] || "Property"} changed`);
      noteUndoState(r.undo);
    }
    return true;
  } catch (e) {
    errorDialog("That property could not be changed.", e.message);
    return false;
  }
}

/// One field: a label and an input that commits on Enter or when it loses focus.
function fieldRow(key, label, hint, value, multiline) {
  const row = el("label", "op-prop-row");
  row.appendChild(el("span", "op-prop-label", label));
  const input = el(multiline ? "textarea" : "input", "op-prop-input");
  if (!multiline) input.type = "text";
  else input.rows = 2;
  input.value = value || "";
  input.placeholder = hint;
  input.dataset.prop = key;
  input.spellcheck = false;
  let committed = input.value;
  const commit = async () => {
    if (input.value === committed) return;
    committed = input.value;
    await commitField(key, input.value);
  };
  input.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && !(multiline && e.shiftKey)) { e.preventDefault(); commit(); }
    if (e.key === "Escape") { input.value = committed; }
    e.stopPropagation();
  });
  input.addEventListener("blur", commit);
  row.appendChild(input);
  return row;
}

function readOnlyRow(label, value) {
  const row = el("div", "op-prop-row");
  row.append(el("span", "op-prop-label", label), el("span", "op-prop-value", value || "—"));
  return row;
}

/// Draw File > Info's Properties panel into `host`.
async function drawInfo(host) {
  if (!state.deck) { host.replaceChildren(el("p", "op-muted", "Open a presentation to see its properties.")); return; }
  let info;
  try { info = await readInfo(); } catch (e) { host.replaceChildren(el("p", "op-muted", `The properties could not be read. ${e.message}`)); return; }
  host.replaceChildren();
  host.appendChild(el("h2", "rb-section-title", "Properties"));
  const grid = el("div", "op-props");
  grid.appendChild(readOnlyRow("Slides", String(info.slides)));
  grid.appendChild(readOnlyRow("Words", String(info.words)));
  for (const [key, label, hint] of FIELDS) {
    if (key === "author") continue;
    grid.appendChild(fieldRow(key, label, hint, info[key], key === "comments"));
  }
  host.appendChild(grid);
  host.appendChild(el("h2", "rb-section-title", "Related Dates"));
  const dates = el("div", "op-props");
  dates.appendChild(readOnlyRow("Last Modified", dateLabel(info.modified)));
  dates.appendChild(readOnlyRow("Created", dateLabel(info.created)));
  host.appendChild(dates);
  host.appendChild(el("h2", "rb-section-title", "Related People"));
  const people = el("div", "op-props");
  const [k, label, hint] = FIELDS.find((f) => f[0] === "author");
  people.appendChild(fieldRow(k, label, hint, info.author));
  people.appendChild(readOnlyRow("Last Modified By", info.lastModifiedBy));
  host.appendChild(people);
}

let infoPage = null;

function infoHost(page) {
  let host = page.querySelector(".op-info-props");
  if (!host) {
    host = el("div", "rb-section op-info-props");
    page.appendChild(host);
  }
  return host;
}

/* ── Advanced Properties ───────────────────────────────────────────────── */

const KINDS = [["text", "Text", 0], ["date", "Date", 2], ["number", "Number", 1], ["yesno", "Yes or no", 3]];

async function advancedProperties() {
  if (!state.deck) return;
  let info;
  try { info = await readInfo(); } catch (e) { errorDialog("The properties could not be read.", e.message); return; }
  showModal(`${state.deck.name} Properties`, (body) => {
    const tabs = el("div", "op-tabs");
    tabs.setAttribute("role", "tablist");
    const panes = {};
    const show = (id) => {
      for (const b of tabs.children) b.setAttribute("aria-selected", b.dataset.tab === id ? "true" : "false");
      for (const [pid, p] of Object.entries(panes)) p.hidden = pid !== id;
    };
    for (const [id, label] of [["summary", "Summary"], ["statistics", "Statistics"], ["custom", "Custom"]]) {
      const b = el("button", "op-tab", label);
      b.type = "button";
      b.dataset.tab = id;
      b.setAttribute("role", "tab");
      b.addEventListener("click", () => show(id));
      tabs.appendChild(b);
    }
    body.appendChild(tabs);

    const summary = el("div", "op-props");
    for (const [key, label, hint] of FIELDS) summary.appendChild(fieldRow(key, label, hint, info[key], key === "comments"));
    panes.summary = summary;

    const stats = el("div", "op-props");
    stats.appendChild(readOnlyRow("Created", dateLabel(info.created)));
    stats.appendChild(readOnlyRow("Modified", dateLabel(info.modified)));
    stats.appendChild(readOnlyRow("Last saved by", info.lastModifiedBy));
    stats.appendChild(readOnlyRow("Slides", String(info.slides)));
    stats.appendChild(readOnlyRow("Words", String(info.words)));
    stats.appendChild(readOnlyRow("Type", `${TYPES[info.format]?.[0] || info.format} (.${info.format})`));
    panes.statistics = stats;

    panes.custom = customPane(info.custom || []);
    for (const p of Object.values(panes)) body.appendChild(p);
    const actions = el("div", "op-confirm-actions");
    const ok = el("button", "op-btn primary", "OK");
    ok.type = "button";
    ok.addEventListener("click", () => {
      // Commit a field still focused before closing.
      document.activeElement?.blur?.();
      closeModal();
    });
    actions.appendChild(ok);
    body.appendChild(actions);
    show("summary");
  });
}

function customPane(list) {
  const pane = el("div", "op-custom");
  const form = el("div", "op-custom-form");
  const name = el("input", "op-prop-input");
  name.type = "text";
  name.placeholder = "Name";
  name.setAttribute("aria-label", "Name");
  name.maxLength = 255;
  const kind = el("select", "op-prop-input");
  kind.setAttribute("aria-label", "Type");
  for (const [v, label] of KINDS) { const o = el("option", null, label); o.value = v; kind.appendChild(o); }
  const value = el("input", "op-prop-input");
  value.type = "text";
  value.placeholder = "Value";
  value.setAttribute("aria-label", "Value");
  const setValueType = () => {
    value.type = kind.value === "date" ? "date" : kind.value === "number" ? "number" : "text";
    value.placeholder = kind.value === "yesno" ? "Yes or No" : "Value";
  };
  kind.addEventListener("change", setValueType);
  const add = el("button", "op-btn", "Add");
  add.type = "button";
  const remove = el("button", "op-btn", "Delete");
  remove.type = "button";
  for (const i of [name, value]) i.addEventListener("keydown", (e) => { e.stopPropagation(); if (e.key === "Enter") { e.preventDefault(); add.click(); } });
  form.append(name, kind, value, add, remove);
  pane.appendChild(form);
  const note = el("p", "op-muted op-custom-note", "");
  pane.appendChild(note);

  const table = el("div", "op-custom-list");
  table.setAttribute("role", "listbox");
  table.setAttribute("aria-label", "Custom properties");
  pane.appendChild(table);
  let rows = list;
  const draw = () => {
    table.replaceChildren();
    const head = el("div", "op-custom-row op-custom-head");
    head.append(el("span", null, "Name"), el("span", null, "Value"), el("span", null, "Type"));
    table.appendChild(head);
    if (!rows.length) table.appendChild(el("div", "op-muted op-custom-empty", "No custom properties"));
    for (const [n, k, v] of rows) {
      const row = el("button", "op-custom-row");
      row.type = "button";
      row.setAttribute("role", "option");
      const kl = KINDS.find((x) => x[0] === k)?.[1] || "Kept as is";
      row.append(el("span", null, n), el("span", null, k === "other" ? "(not editable here)" : v), el("span", null, kl));
      row.addEventListener("click", () => {
        name.value = n;
        if (k !== "other") { kind.value = k; setValueType(); value.value = v; }
        for (const r of table.querySelectorAll(".op-custom-row")) r.setAttribute("aria-selected", r === row ? "true" : "false");
      });
      table.appendChild(row);
    }
  };
  const refresh = async () => { rows = (await readInfo()).custom || []; draw(); };
  add.addEventListener("click", async () => {
    const code = KINDS.find((x) => x[0] === kind.value)?.[2] ?? 0;
    try {
      const r = await fileCall("set_custom_property", [name.value, code, value.value]);
      note.textContent = r.value ? `${name.value.trim()} saved with the presentation.` : "Nothing changed.";
      if (r.value) noteUndoState(r.undo);
      await refresh();
    } catch {
      note.textContent = !name.value.trim() ? "Type a name first." : `That value is not a ${KINDS.find((x) => x[0] === kind.value)[1].toLowerCase()}.`;
    }
  });
  remove.addEventListener("click", async () => {
    try {
      const r = await fileCall("remove_custom_property", [name.value]);
      note.textContent = r.value ? `${name.value.trim()} deleted.` : "No custom property has that name.";
      if (r.value) noteUndoState(r.undo);
      await refresh();
    } catch (e) {
      note.textContent = e.message;
    }
  });
  draw();
  return pane;
}

/* ── Save As ───────────────────────────────────────────────────────────── */

let saving = false;

/// File > Save As: a name and a type, then the file through the save-as grid. The
/// deck takes the new name, as PowerPoint's does.
async function saveAs() {
  if (!state.deck || saving) return;
  let format = extensionOf(state.deck.name) || "pptx";
  try { format = (await readInfo()).format || format; } catch { /* the name's */ }
  const macros = format.endsWith("m");
  showModal("Save As", (body) => {
    const form = el("div", "op-saveas");
    const nameRow = el("label", "op-prop-row");
    nameRow.appendChild(el("span", "op-prop-label", "File name"));
    const name = el("input", "op-prop-input");
    name.type = "text";
    name.value = stemOf(state.deck.name);
    name.spellcheck = false;
    nameRow.appendChild(name);
    const typeRow = el("label", "op-prop-row");
    typeRow.appendChild(el("span", "op-prop-label", "Save as type"));
    const type = el("select", "op-prop-input");
    const offered = macros ? ["pptx", "potx", "ppsx", "pptm", "potm", "ppsm"] : ["pptx", "potx", "ppsx"];
    for (const ext of offered) {
      const o = el("option", null, `${TYPES[ext][0]} (*.${ext})`);
      o.value = ext;
      type.appendChild(o);
    }
    type.value = offered.includes(extensionOf(state.deck.name)) ? extensionOf(state.deck.name) : "pptx";
    typeRow.appendChild(type);
    const note = el("p", "op-muted", "");
    const explain = () => {
      note.textContent = macros && !type.value.endsWith("m")
        ? "This presentation has a macro project. A type without macros leaves it out of the copy; it is never run."
        : "The file is made in this browser and downloaded. Nothing is uploaded.";
    };
    type.addEventListener("change", explain);
    explain();
    form.append(nameRow, typeRow, note);
    body.appendChild(form);
    const actions = el("div", "op-confirm-actions");
    const cancel = el("button", "op-btn", "Cancel");
    cancel.type = "button";
    cancel.addEventListener("click", closeModal);
    const ok = el("button", "op-btn primary", "Save");
    ok.type = "button";
    const go = async () => {
      const stem = cleanName(stemOf(name.value)) || stemOf(state.deck.name);
      const ext = type.value;
      closeModal();
      await saveAsType(`${stem}.${ext}`, ext, macros && !ext.endsWith("m"));
    };
    ok.addEventListener("click", go);
    name.addEventListener("keydown", (e) => { e.stopPropagation(); if (e.key === "Enter") { e.preventDefault(); go(); } });
    actions.append(cancel, ok);
    body.appendChild(actions);
  });
  const input = byId("op-modal-body").querySelector("input");
  input?.focus();
  input?.select();
}

async function saveAsType(name, ext, dropMacros) {
  saving = true;
  status("Saving…");
  try {
    const r = await request({ op: "save-as", ext, dropMacros }, { priority: 5 });
    const written = await download(new Uint8Array(r.bytes), name, TYPES[ext][1]);
    if (written === null) { status("Save As cancelled"); return; }
    rename(name);
    status(`Saved: ${written}`);
    document.dispatchEvent(new CustomEvent("op-saved"));
  } catch (e) {
    status("Save As failed");
    errorDialog("OpenPresent could not save the presentation as that type.", e.message);
  } finally {
    saving = false;
    changed();
  }
}

/// Download a Copy: the deck as it stands, under its name; the deck is not renamed
/// and its saved state does not change.
async function downloadCopy() {
  if (!state.deck || saving) return;
  saving = true;
  status("Downloading a copy…");
  try {
    const ext = extensionOf(state.deck.name) || "pptx";
    const r = await request({ op: "save-as", ext, dropMacros: false }, { priority: 5 });
    const written = await download(new Uint8Array(r.bytes), state.deck.name, TYPES[ext][1]);
    status(written === null ? "Download cancelled" : `Downloaded a copy: ${written}`);
  } catch (e) {
    status("Download failed");
    errorDialog("OpenPresent could not make a copy.", e.message);
  } finally {
    saving = false;
    changed();
  }
}

/* ── Print ─────────────────────────────────────────────────────────────── */

/// The width each slide is drawn at for printing: sharp on paper at 150 dpi.
const PRINT_WIDTH_PX = 1600;
let printing = false;

/// File > Print: every slide but the hidden ones drawn, one a page, then the
/// browser's own Print. Only the sheet prints (`@media print` in `editor.css`).
async function print() {
  if (!state.deck || printing) return;
  printing = true;
  changed();
  const sheet = byId("print-sheet");
  const urls = [];
  try {
    let hidden = [];
    try { hidden = (await request({ op: "build", fn: "hidden_slides", args: [] }, { priority: 5 })).value || []; } catch { hidden = []; }
    sheet.replaceChildren();
    const loads = [];
    for (let i = 0; i < state.deck.count; i++) {
      if (hidden[i]) continue;
      status(`Preparing slide ${i + 1} of ${state.deck.count} to print…`);
      const r = await request({ op: "png", index: i, width: PRINT_WIDTH_PX }, { priority: 5 });
      const url = URL.createObjectURL(new Blob([r.png], { type: "image/png" }));
      urls.push(url);
      const page = el("div", "print-page");
      const img = el("img", "print-slide");
      img.alt = `Slide ${i + 1}`;
      loads.push(new Promise((res) => { img.onload = res; img.onerror = res; }));
      img.src = url;
      page.appendChild(img);
      sheet.appendChild(page);
    }
    await Promise.all(loads);
    status("Printing");
    const done = () => {
      sheet.replaceChildren();
      for (const u of urls) URL.revokeObjectURL(u);
      status("Ready");
    };
    window.addEventListener("afterprint", done, { once: true });
    window.print();
  } catch (e) {
    status("Print failed");
    errorDialog("OpenPresent could not prepare the slides to print.", e.message);
  } finally {
    printing = false;
    changed();
  }
}

/* ── Commands and wiring ───────────────────────────────────────────────── */

export function registerFileCommands() {
  const has = () => !!state.deck;
  // Save writes the type the name says: after Save As .potx, Save keeps it a template.
  // The type it was opened as goes through the grid's `=` cell, a plain save.
  register("file.save", { label: "Save", shortcut: "Ctrl+S", enabled: () => has() && !saving, run: () => {
    const ext = extensionOf(state.deck.name) || "pptx";
    return saveAsType(state.deck.name, ext, false);
  } });
  register("file.rename", { label: "Rename", enabled: has, run: startRename });
  register("file.save-as", { label: "Save As", shortcut: "F12", enabled: () => has() && !saving, run: saveAs });
  register("file.save-copy", { label: "Download a Copy", enabled: () => has() && !saving, run: downloadCopy });
  register("file.properties", { label: "Advanced Properties", enabled: has, run: advancedProperties });
  register("file.print", { label: "Print", shortcut: "Ctrl+P", enabled: () => has() && !printing, run: print });
}

export function initFile() {
  initRename();
  document.addEventListener("op-backstage-page", (e) => {
    const { id, page } = e.detail || {};
    if (id !== "info" || !page) { infoPage = null; return; }
    infoPage = page;
    drawInfo(infoHost(page));
  });
  // An undo or redo can change a property; the Info page, when it shows, follows.
  document.addEventListener("op-changed", () => {
    if (infoPage && !infoPage.hidden && !infoPage.contains(document.activeElement)) drawInfo(infoHost(infoPage));
  });
  document.addEventListener("keydown", (e) => {
    const mod = e.ctrlKey || e.metaKey;
    if (!byId("op-modal").hidden) return;
    if (e.key === "F12" && !mod && !e.altKey) { e.preventDefault(); run("file.save-as"); }
    else if (mod && !e.altKey && !e.shiftKey && (e.key === "p" || e.key === "P")) { e.preventDefault(); run("file.print"); }
  });
}
