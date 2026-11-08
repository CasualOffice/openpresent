// The Alt Text pane (`PAR-44`): Shape Format > Alt Text, Picture Format > Alt Text and
// the right-click menu, as PowerPoint for the web opens it at the right. One box: the
// selected shape's description for screen readers, `cNvPr/@descr`. What is typed is
// set a moment after typing stops, and when the box loses the focus, one undo step a
// setting. A pane in the task pane host (`editor.panehost.js`), built as the
// Selection Pane is; document text reaches the box as its `value`, never as HTML.

import { state, request } from "./editor.core.js";
import { register } from "./commands.js";
import * as host from "./editor.panehost.js";
import * as selection from "./editor.selection.js";
import { errorDialog } from "./editor.dialogs.js";

const MAX_CHARS = 4096;   // `MAX_ALT_TEXT_CHARS`

let body = null;
let box = null;
let none = null;
let target = null;        // { index, node } the box is for
let timer = 0;
let typed = false;      // typed in since the box was given its shape

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

const one = () => !!state.deck && selection.selectedNodes().length === 1 && selection.selectedNode() != null;

async function commit() {
  clearTimeout(timer);
  if (!target || !box) return;
  const { index, node } = target;
  const text = box.value;
  if (box.dataset.saved === text) return;
  try {
    const r = await request({ op: "build", fn: "set_shape_alt_text", args: [index, node, text] }, { priority: 9 });
    if (r.undo) selection.noteUndoState(r.undo);
    if (box && target && target.node === node) { box.dataset.saved = text; typed = false; }
  } catch (e) {
    errorDialog("OpenPresent could not set the alt text.", e.message);
  }
}

async function read() {
  if (!body) return;
  const node = one() ? selection.selectedNode() : null;
  const index = state.current;
  if (target && (target.node !== node || target.index !== index)) await commit();
  if (node == null) {
    target = null;
    box.hidden = true;
    none.hidden = false;
    return;
  }
  const same = target && target.node === node && target.index === index;
  // While the box is being typed in, the box is the truth; otherwise read again, so an
  // undo shows.
  if (same && (document.activeElement === box || box.value !== box.dataset.saved)) return;
  if (!same) { box.value = ""; box.dataset.saved = ""; typed = false; }
  target = { index, node };
  box.hidden = false;
  none.hidden = true;
  try {
    const r = await request({ op: "build", fn: "shape_alt_text", args: [index, node] }, { priority: 8 });
    if (!box || !target || target.node !== node) return;
    // Keys typed before the answer came are kept, and set as typing stops.
    if (!typed) box.value = r.value || "";
    box.dataset.saved = r.value || "";
  } catch {
    target = null;
  }
}

function build(b) {
  body = h("div", "alt-pane");
  const label = h("label", "alt-label", "How would you describe this object and its context to someone who is blind or has low vision?");
  box = document.createElement("textarea");
  box.className = "fp-input alt-box";
  box.rows = 6;
  box.maxLength = MAX_CHARS;
  box.dataset.autofocus = "";
  box.setAttribute("aria-label", "Alt text");
  box.addEventListener("input", () => { typed = true; clearTimeout(timer); timer = setTimeout(commit, 600); });
  box.addEventListener("blur", commit);
  // Keys typed here are the box's, not the slide's.
  box.addEventListener("keydown", (e) => e.stopPropagation());
  label.htmlFor = "alt-text-box";
  box.id = "alt-text-box";
  none = h("p", "fp-none", "Select one shape or picture to give it alt text.");
  body.append(label, box, none);
  b.appendChild(body);
  target = null;
  read();
}

host.registerPane("alt-text", {
  title: "Alt Text",
  build,
  refresh: () => read(),
  closed: () => { commit(); body = null; box = null; target = null; },
});

export function init() {
  register("shape.alt-text", { label: "Alt Text", enabled: one, checked: () => host.isOpen("alt-text"), run: () => host.toggle("alt-text") });
  let pending = 0;
  document.addEventListener("op-changed", () => {
    if (!body) return;
    clearTimeout(pending);
    pending = setTimeout(read, 60);
  });
}
