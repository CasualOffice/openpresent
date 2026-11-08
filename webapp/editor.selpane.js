// The Selection Pane (`UX-011`): Home > Arrange > Selection Pane, and Shape Format's.
//
// As PowerPoint's: the slide's shapes, front first; a click selects one, a double click
// or F2 renames it, the eye hides or shows it, Show All and Hide All, and dragging a row
// (or Alt+Shift+Up and Down) moves it in z-order. Each change is one engine call and one
// undo step; a save writes the name, `@hidden`, and the order by moving the shapes'
// elements in the slide. Shape names are document text and reach the DOM through
// `textContent` and input values only.

import { state, request, status } from "./editor.core.js";
import { register } from "./commands.js";
import { icon } from "./ribbon.js";
import * as host from "./editor.panehost.js";
import * as selection from "./editor.selection.js";
import * as slides from "./editor.slides.js";
import { errorDialog } from "./editor.dialogs.js";

const has = () => !!state.deck && state.deck.count > 0;
const KIND = { spMk: "Shape", picMk: "Picture", grpSpMk: "Group", graphicFrameMk: "Object", cxnSpMk: "Connector" };

let body = null;
let items = [];
let lastJson = "";
let readFor = -1;

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

async function call(fn, args) {
  const r = await request({ op: "review", fn, args }, { priority: 9 });
  if (r.undo) selection.noteUndoState(r.undo);
  return r.value;
}

async function edit(what, fn, args) {
  try {
    const changedIt = await call(fn, [state.current, ...args]);
    if (changedIt) {
      await slides.redrawSlide(state.current);
      await read(true);
    }
    return changedIt;
  } catch (e) {
    errorDialog(`OpenPresent could not ${what}.`, e.message);
    return false;
  }
}

async function read(force = false) {
  if (!body || !has()) return;
  const index = state.current;
  try {
    const json = await call("selection_list", [index]);
    if (!force && json === lastJson && readFor === index) { mark(); return; }
    lastJson = json;
    readFor = index;
    items = JSON.parse(json);
    render();
  } catch (e) {
    status(`The shapes could not be listed: ${e.message}`);
  }
}

/// Mark the row of the selected shape.
function mark() {
  if (!body) return;
  const chosen = new Set(selection.selectedNodes());
  for (const li of body.querySelectorAll(".sp-item")) li.setAttribute("aria-selected", chosen.has(Number(li.dataset.node)) ? "true" : "false");
}

/// z-order place, 0 the back, of the row at list position `i` (the list is front first).
const zOf = (i) => items.length - 1 - i;

function rename(li, item) {
  const label = li.querySelector(".sp-name");
  const input = h("input", "fp-input sp-rename");
  input.type = "text";
  input.value = item.name;
  input.maxLength = 255;
  input.setAttribute("aria-label", "Shape name");
  label.replaceWith(input);
  input.focus();
  input.select();
  let done = false;
  const finish = async (keep) => {
    if (done) return;
    done = true;
    const name = input.value.trim();
    if (keep && name && name !== item.name) {
      await edit("rename the shape", "rename_shape", [item.node, name]);
      status(`Renamed to ${name}`);
    } else render();
  };
  input.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.key === "Enter") { e.preventDefault(); finish(true); }
    else if (e.key === "Escape") { e.preventDefault(); finish(false); }
  });
  input.addEventListener("blur", () => finish(true));
}

let dragFrom = -1;

function render() {
  const ol = body.querySelector(".sp-list");
  const focusedNode = document.activeElement?.closest?.(".sp-item")?.dataset.node;
  ol.textContent = "";
  body.querySelector(".fp-none").hidden = items.length > 0;
  items.forEach((item, i) => {
    const li = h("li", "sp-item");
    li.dataset.node = String(item.node);
    li.setAttribute("role", "option");
    li.tabIndex = 0;
    li.draggable = true;
    li.classList.toggle("is-hidden", item.hidden);
    const name = h("span", "sp-name", item.name || KIND[item.kind] || "Shape");
    name.title = `${KIND[item.kind] || "Shape"}: ${item.name}`;
    const eye = h("button", "fp-close sp-eye");
    eye.type = "button";
    const verb = item.hidden ? "Show" : "Hide";
    eye.title = verb;
    eye.setAttribute("aria-label", `${verb} ${item.name || "shape"}`);
    eye.setAttribute("aria-pressed", item.hidden ? "true" : "false");
    eye.appendChild(icon(item.hidden ? "eye_off" : "eye", 16));
    eye.addEventListener("click", (e) => { e.stopPropagation(); edit(item.hidden ? "show the shape" : "hide the shape", "set_shape_hidden", [item.node, !item.hidden]); });
    li.append(name, eye);
    li.addEventListener("click", () => {
      if (item.hidden) { status("A hidden shape cannot be selected on the slide; show it first"); return; }
      selection.selectNode(item.node);
      slides.redrawSlide(state.current).then(mark);
    });
    li.addEventListener("dblclick", () => rename(li, item));
    li.addEventListener("keydown", (e) => {
      if (e.target !== li) return;
      e.stopPropagation();
      if (e.key === "F2") { e.preventDefault(); rename(li, item); }
      else if (e.key === "Enter" || e.key === " ") { e.preventDefault(); li.click(); }
      else if (e.altKey && e.shiftKey && (e.key === "ArrowUp" || e.key === "ArrowDown")) {
        e.preventDefault();
        const to = e.key === "ArrowUp" ? i - 1 : i + 1;
        if (to >= 0 && to < items.length) edit("reorder the shapes", "move_shape_z", [item.node, zOf(to)]);
      } else if (e.key === "ArrowDown") { e.preventDefault(); li.nextElementSibling?.focus(); }
      else if (e.key === "ArrowUp") { e.preventDefault(); li.previousElementSibling?.focus(); }
    });
    li.addEventListener("dragstart", (e) => { dragFrom = i; e.dataTransfer.effectAllowed = "move"; e.dataTransfer.setData("text/plain", ""); li.classList.add("is-dragging"); });
    li.addEventListener("dragend", () => { dragFrom = -1; li.classList.remove("is-dragging"); for (const x of ol.children) x.classList.remove("drop-before", "drop-after"); });
    li.addEventListener("dragover", (e) => {
      if (dragFrom < 0) return;
      e.preventDefault();
      const r = li.getBoundingClientRect();
      const before = e.clientY < r.top + r.height / 2;
      for (const x of ol.children) x.classList.remove("drop-before", "drop-after");
      li.classList.add(before ? "drop-before" : "drop-after");
    });
    li.addEventListener("drop", (e) => {
      if (dragFrom < 0) return;
      e.preventDefault();
      const r = li.getBoundingClientRect();
      const before = e.clientY < r.top + r.height / 2;
      let to = before ? i : i + 1;
      if (to > dragFrom) to -= 1;
      const from = dragFrom;
      dragFrom = -1;
      if (to !== from) edit("reorder the shapes", "move_shape_z", [items[from].node, zOf(to)]);
    });
    ol.appendChild(li);
  });
  mark();
  if (focusedNode) ol.querySelector(`.sp-item[data-node="${CSS.escape(focusedNode)}"]`)?.focus({ preventScroll: true });
}

async function setAll(hidden) {
  const todo = items.filter((x) => x.hidden !== hidden);
  for (const x of todo) await call("set_shape_hidden", [state.current, x.node, hidden]).catch(() => {});
  await slides.redrawSlide(state.current);
  await read(true);
}

function build(b) {
  body = h("div", "sp-pane");
  const tools = h("div", "sp-tools");
  for (const [label, hidden] of [["Show All", false], ["Hide All", true]]) {
    const btn = h("button", "op-btn", label);
    btn.type = "button";
    btn.addEventListener("click", () => setAll(hidden));
    tools.appendChild(btn);
  }
  const ol = h("ol", "sp-list");
  ol.setAttribute("role", "listbox");
  ol.setAttribute("aria-label", "Shapes on this slide, front first");
  ol.dataset.autofocus = "";
  ol.tabIndex = -1;
  ol.addEventListener("focus", () => ol.querySelector(".sp-item")?.focus());
  body.append(tools, ol, h("p", "fp-none", "There are no shapes on this slide."));
  b.appendChild(body);
  lastJson = "";
  read(true);
}

host.registerPane("selection", { title: "Selection", build, refresh: () => read(), closed: () => { body = null; readFor = -1; } });

export function init() {
  register("arrange.selection-pane", { label: "Selection Pane", enabled: has, checked: () => host.isOpen("selection"), run: () => host.toggle("selection") });
  let pending = 0;
  document.addEventListener("op-changed", () => {
    if (!body) return;
    clearTimeout(pending);
    pending = setTimeout(() => read(readFor !== state.current), 60);
  });
}
