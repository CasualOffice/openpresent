// The task pane on the right: Format Shape, as PowerPoint's (`UX-003`, docs/53).
//
// PowerPoint opens Format Shape from a group's dialog launcher (Drawing, Shape
// Styles, Size) or a shape's right-click menu. Two tabs, Shape Options and Text
// Options; Shape Options has three pages, Fill & Line, Effects and Size &
// Properties, each a stack of sections that open and close.
//
// Size and position are read from the selection and edited here (`UX-003`): a
// value typed in and committed with Enter or by leaving the box moves or sizes the
// shape, one undo step, as a drag does. Fill and Line set No fill or a solid colour,
// No line or a solid line of a colour and a width, through the same engine calls as
// Shape Fill and Shape Outline. What this build cannot change yet (gradients,
// pictures, patterns, effects, dashes from here) stays present and disabled, saying
// so, never faked (owner instruction 46).

import { state, changed, request, status } from "./editor.core.js";
import { register } from "./commands.js";
import { icon } from "./ribbon.js";
import * as selection from "./editor.selection.js";
import * as slides from "./editor.slides.js";
import { openPop, palette, colourMenu } from "./editor.shapes.js";
import { errorDialog } from "./editor.dialogs.js";

const byId = (id) => document.getElementById(id);
const EMU_PER_INCH = 914400;
const NOT_YET = "Not available yet in this build";

let pane = null;
let page = "fill";        // fill | effects | size | text
let fields = [];          // [{ input, read: () => string }]

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

/// The selected shape's box, in EMU: `{ node, x, y, cx, cy, rot, flags }`, or `null`.
function selectedBox() {
  if (!state.deck || !selection.hasSelection()) return null;
  return selection.selectedShape();
}

const inches = (emu) => `${(emu / EMU_PER_INCH).toFixed(2)}"`;

/// A length typed as PowerPoint takes it: inches by default, or with `"`, `in`,
/// `cm`, `mm` or `pt`. EMU, or null.
function parseLength(text) {
  const m = /^\s*(-?\d+(?:\.\d+)?)\s*("|in|cm|mm|pt)?\s*$/i.exec(text);
  if (!m) return null;
  const v = parseFloat(m[1]);
  const per = { '"': EMU_PER_INCH, in: EMU_PER_INCH, cm: 360000, mm: 36000, pt: 12700 }[(m[2] || '"').toLowerCase()];
  return Math.round(v * per);
}

/// One Fill or Line edit of the selected shape, then the slide drawn again.
async function look(label, fn, args) {
  const s = selectedBox();
  if (!s) return;
  const index = state.current;
  try {
    const r = await request({ op: "build", fn, args: [index, s.node, ...args] }, { priority: 9 });
    selection.noteUndoState(r.undo);
    if (r.value) { await slides.redrawSlide(index); status(label); } else status(`${label}: nothing changed`);
  } catch (e) {
    errorDialog(`OpenPresent could not change that shape: ${label.toLowerCase()}.`, e.message);
  }
}

/* ── Building blocks ────────────────────────────────────────────────────── */

/// A section that opens and closes, as PowerPoint's ▸ Fill, ▸ Line.
function section(title, open, body) {
  const s = h("section", "fp-sec");
  const head = h("button", "fp-sec-head");
  head.type = "button";
  head.setAttribute("aria-expanded", open ? "true" : "false");
  head.append(icon("chevron_right", 12), h("span", null, title));
  const content = h("div", "fp-sec-body");
  content.hidden = !open;
  body(content);
  head.addEventListener("click", () => {
    content.hidden = !content.hidden;
    head.setAttribute("aria-expanded", content.hidden ? "false" : "true");
  });
  s.append(head, content);
  return s;
}

/// Radio buttons; `live` maps an option to what choosing it does, and the rest are
/// present and disabled.
function radios(box, name, options, live = {}) {
  for (const o of options) {
    const run = live[o];
    const l = h("label", run ? "fp-radio" : "fp-radio is-off");
    const r = h("input");
    r.type = "radio";
    r.name = name;
    r.disabled = !run;
    if (run) r.addEventListener("change", () => { if (r.checked) run(); });
    else l.title = NOT_YET;
    l.append(r, h("span", null, o));
    box.appendChild(l);
  }
}

/// A colour button that opens PowerPoint's colour menu and gives the pick to `onPick`.
function colourField(box, label, noneLabel, onPick) {
  const row = h("div", "fp-row");
  row.appendChild(h("span", "fp-label", label));
  const b = h("button", "fp-color");
  b.type = "button";
  b.setAttribute("aria-label", label);
  b.append(h("span", "fp-swatch"), icon("chevron_down", 10));
  b.addEventListener("click", async () => {
    const colours = await palette();
    openPop(b, "shp-menu", (p) => colourMenu(p, colours, noneLabel, onPick));
  });
  row.appendChild(b);
  box.appendChild(row);
}

/// A labelled field. With `read`, it shows a value from the selection, and with
/// `write` too it takes a typed one, committed on Enter or on leaving the box;
/// without either, it is a disabled control of the kind PowerPoint shows there.
function field(box, label, kind = "text", read = null, write = null) {
  const row = h("label", "fp-row");
  row.appendChild(h("span", "fp-label", label));
  let input;
  if (kind === "color") {
    input = h("button", "fp-color");
    input.type = "button";
    input.append(h("span", "fp-swatch"), icon("chevron_down", 10));
  } else if (kind === "check") {
    input = h("input");
    input.type = "checkbox";
  } else {
    input = h("input", "fp-input");
    input.type = "text";
  }
  if (read && write) {
    const commit = () => {
      const s = selectedBox();
      if (!s || input.value === read(s)) return;
      if (write(input.value, s) === false) { input.value = read(s); status(`${label}: enter a number, as 1.5" or 4 cm`); }
    };
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") { e.preventDefault(); commit(); }
      else if (e.key === "Escape") { const s = selectedBox(); if (s) input.value = read(s); }
    });
    input.addEventListener("change", commit);
    fields.push({ input, read });
  } else if (read) {
    input.readOnly = true;
    fields.push({ input, read });
  } else {
    input.disabled = true;
    row.title = NOT_YET;
    row.classList.add("is-off");
  }
  row.appendChild(input);
  box.appendChild(row);
}

/* ── Pages ──────────────────────────────────────────────────────────────── */

const PAGES = {
  fill: (body) => {
    body.append(
      section("Fill", true, (b) => {
        radios(b, "fp-fill", ["No fill", "Solid fill", "Gradient fill", "Picture or texture fill", "Pattern fill", "Slide background fill"], {
          "No fill": () => look("No Fill", "set_shape_fill", ["none"]),
          "Solid fill": () => look("Solid Fill", "set_shape_fill", ["scheme accent1"]),
        });
        colourField(b, "Color", "No Fill", (spec) => look("Shape Fill", "set_shape_fill", [spec]));
        field(b, "Transparency");
      }),
      section("Line", true, (b) => {
        radios(b, "fp-line", ["No line", "Solid line", "Gradient line"], {
          "No line": () => look("No Line", "set_shape_outline", [0, "none"]),
          "Solid line": () => look("Solid Line", "set_shape_outline", [0, "scheme accent1"]),
        });
        colourField(b, "Color", "No Outline", (spec) => look("Shape Outline", "set_shape_outline", [0, spec]));
        field(b, "Transparency");
        const width = h("div");
        field(width, "Width", "text", () => "", (v) => {
          const emu = parseLength(/^\s*[\d.]+\s*$/.test(v) ? `${v} pt` : v);
          if (emu == null || emu <= 0 || emu > 20116800) return false;
          look("Outline Weight", "set_shape_outline", [1, String(emu)]);
          return true;
        });
        width.querySelector("input")?.setAttribute("placeholder", "pt");
        b.append(...width.childNodes);
        field(b, "Dash type");
        field(b, "Cap type");
        field(b, "Join type");
      }),
    );
  },
  effects: (body) => {
    for (const [name, fs] of [["Shadow", ["Presets", "Color", "Transparency", "Size", "Blur", "Angle", "Distance"]], ["Reflection", ["Presets", "Transparency", "Size", "Distance"]], ["Glow", ["Presets", "Color", "Size", "Transparency"]], ["Soft Edges", ["Presets", "Size"]], ["3-D Format", ["Top bevel", "Bottom bevel", "Depth", "Material"]], ["3-D Rotation", ["Presets", "X Rotation", "Y Rotation", "Z Rotation"]]]) {
      body.appendChild(section(name, false, (b) => { for (const f of fs) field(b, f, f === "Color" ? "color" : "text"); }));
    }
  },
  size: (body) => {
    body.append(
      section("Size", true, (b) => {
        const size = (key) => (v, s) => {
          const emu = parseLength(v);
          if (emu == null || emu < 0 || emu > 51206400) return false;
          if (s.flags & 8) { status("This shape's size is locked"); return true; }
          selection.place({ [key]: emu });
          return true;
        };
        field(b, "Height", "text", (s) => inches(s.cy), size("cy"));
        field(b, "Width", "text", (s) => inches(s.cx), size("cx"));
        field(b, "Rotation", "text", (s) => `${Math.round(((s.rot % 360) + 360) % 360)}°`, (v, s) => {
          const deg = parseFloat(String(v).replace("°", ""));
          if (!Number.isFinite(deg)) return false;
          if (s.flags & 16) { status("This shape's rotation is locked"); return true; }
          selection.place({ rot: ((deg % 360) + 360) % 360 });
          return true;
        });
        field(b, "Lock aspect ratio", "check");
      }),
      section("Position", true, (b) => {
        const pos = (key) => (v, s) => {
          const emu = parseLength(v);
          if (emu == null || Math.abs(emu) > 51206400) return false;
          if (s.flags & 4) { status("This shape's position is locked"); return true; }
          selection.place({ [key]: emu });
          return true;
        };
        field(b, "Horizontal position", "text", (s) => inches(s.x), pos("x"));
        field(b, "Vertical position", "text", (s) => inches(s.y), pos("y"));
        field(b, "From", "text", () => "Top Left Corner");
      }),
      section("Text Box", false, (b) => {
        for (const f of ["Vertical alignment", "Text direction", "Autofit", "Left margin", "Right margin", "Top margin", "Bottom margin"]) field(b, f);
        field(b, "Wrap text in shape", "check");
      }),
      section("Alt Text", false, (b) => { field(b, "Description"); }),
    );
  },
  text: (body) => {
    body.append(
      section("Text Fill", true, (b) => { radios(b, "fp-tfill", ["No fill", "Solid fill", "Gradient fill"]); field(b, "Color", "color"); }),
      section("Text Outline", false, (b) => { radios(b, "fp-tline", ["No line", "Solid line", "Gradient line"]); field(b, "Color", "color"); field(b, "Width"); }),
    );
  },
};

function render() {
  const body = pane.querySelector(".fp-body");
  body.textContent = "";
  fields = [];
  PAGES[page](body);
  for (const b of pane.querySelectorAll("[data-page]")) {
    const on = b.dataset.page === page || (b.dataset.page === "shape" && page !== "text");
    b.setAttribute("aria-selected", on ? "true" : "false");
  }
  pane.querySelector(".fp-icons").hidden = page === "text";
  refresh();
}

/// Fill the read-out fields from the selection; say so when there is none.
export function refresh() {
  if (!pane || pane.hidden) return;
  const s = selectedBox();
  pane.querySelector(".fp-none").hidden = !!s;
  pane.querySelector(".fp-body").hidden = !s;
  if (!s) return;
  for (const f of fields) {
    // A value being typed is not overwritten under the cursor.
    if (document.activeElement === f.input) continue;
    const v = f.read(s);
    if (f.input.value !== v) f.input.value = v;
  }
}

function build() {
  pane = h("aside", "task-pane");
  pane.id = "format-pane";
  pane.setAttribute("aria-label", "Format Shape");
  pane.hidden = true;
  const head = h("div", "fp-head");
  head.appendChild(h("h2", "fp-title", "Format Shape"));
  const x = h("button", "fp-close");
  x.type = "button";
  x.title = "Close";
  x.setAttribute("aria-label", "Close the Format Shape pane");
  x.appendChild(icon("dismiss", 16));
  x.addEventListener("click", close);
  head.appendChild(x);

  const tabs = h("div", "fp-tabs");
  tabs.setAttribute("role", "tablist");
  for (const [id, label] of [["shape", "Shape Options"], ["text", "Text Options"]]) {
    const t = h("button", "fp-tab", label);
    t.type = "button";
    t.dataset.page = id;
    t.setAttribute("role", "tab");
    t.addEventListener("click", () => { page = id === "text" ? "text" : "fill"; render(); });
    tabs.appendChild(t);
  }
  const icons = h("div", "fp-icons");
  icons.setAttribute("role", "tablist");
  for (const [id, ic, label] of [["fill", "color_fill", "Fill & Line"], ["effects", "square_shadow", "Effects"], ["size", "slide_size", "Size & Properties"]]) {
    const b = h("button", "fp-icon");
    b.type = "button";
    b.dataset.page = id;
    b.title = label;
    b.setAttribute("role", "tab");
    b.setAttribute("aria-label", label);
    b.appendChild(icon(ic, 20));
    b.addEventListener("click", () => { page = id; render(); });
    icons.appendChild(b);
  }
  const none = h("p", "fp-none", "Select a shape on the slide to format it.");
  const body = h("div", "fp-body");
  pane.append(head, tabs, icons, none, body);
  pane.addEventListener("keydown", (e) => { if (e.key === "Escape") { e.stopPropagation(); close(); } });
  byId("workspace").appendChild(pane);
}

export function open(which = "fill") {
  if (!pane) build();
  page = which;
  pane.hidden = false;
  render();
  changed();
  pane.querySelector(".fp-close").focus({ preventScroll: true });
}

export function close() {
  if (!pane || pane.hidden) return;
  pane.hidden = true;
  changed();
  byId("stage")?.focus({ preventScroll: true });
}

export const isOpen = () => !!pane && !pane.hidden;

export function registerPaneCommands() {
  const has = () => !!state.deck && selection.hasSelection();
  register("format.shape-pane", { label: "Format Shape…", enabled: has, checked: () => isOpen() && page !== "size", run: () => (isOpen() && page !== "size" ? close() : open("fill")) });
  register("format.size-pane", { label: "Size and Position…", enabled: has, run: () => open("size") });
  register("format.text-pane", { label: "Format Text Effects…", enabled: has, run: () => open("text") });
  // The stage redraws the outline after every edit; follow it.
  new MutationObserver(() => refresh()).observe(byId("slide-frame"), { subtree: true, childList: true, attributes: true, attributeFilter: ["style"] });
}
