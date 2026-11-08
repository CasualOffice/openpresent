// The task pane on the right: Format Shape, as PowerPoint's (`UX-003`, docs/53).
//
// PowerPoint opens Format Shape from a group's dialog launcher (Drawing, Shape
// Styles, Size) or a shape's right-click menu. Two tabs, Shape Options and Text
// Options; Shape Options has three pages, Fill & Line, Effects and Size &
// Properties, each a stack of sections that open and close.
//
// This build cannot yet change a fill, a line or an effect from here, so those
// controls are present and disabled, saying so, never faked (owner instruction
// 46). Size and position are read from the selection on the stage and shown;
// typing them in arrives with the edit that commits them.

import { state, changed } from "./editor.core.js";
import { register } from "./commands.js";
import { icon } from "./ribbon.js";
import * as selection from "./editor.selection.js";

const byId = (id) => document.getElementById(id);
const EMU_PER_INCH = 914400;
const NOT_YET = "Not available yet: this build changes shapes by dragging on the slide";

let pane = null;
let page = "fill";        // fill | effects | size | text
let fields = [];          // [{ input, read: () => string }]

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

/// The selected shape's box, in EMU, as the stage draws it: `{ x, y, cx, cy, rot }`,
/// or `null`. Read from the outline `editor.selection.js` paints, so this pane
/// needs nothing from that module but the fact that something is selected.
function selectedBox() {
  if (!state.deck || !selection.hasSelection()) return null;
  const box = document.querySelector("#sel-layer .sel-box");
  const frame = byId("slide-frame");
  if (!box || !frame || !frame.clientWidth) return null;
  const k = frame.clientWidth / state.deck.width;       // px per EMU
  const px = (v) => parseFloat(v) || 0;
  const rot = /rotate\(([-\d.]+)deg\)/.exec(box.style.transform);
  return { x: px(box.style.left) / k, y: px(box.style.top) / k, cx: px(box.style.width) / k, cy: px(box.style.height) / k, rot: rot ? parseFloat(rot[1]) : 0 };
}

const inches = (emu) => `${(emu / EMU_PER_INCH).toFixed(2)}"`;

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

function radios(box, name, options) {
  for (const o of options) {
    const l = h("label", "fp-radio is-off");
    const r = h("input");
    r.type = "radio";
    r.name = name;
    r.disabled = true;
    l.title = NOT_YET;
    l.append(r, h("span", null, o));
    box.appendChild(l);
  }
}

/// A labelled field. With `read`, it shows a value from the selection; without,
/// it is a disabled control of the kind PowerPoint shows there.
function field(box, label, kind = "text", read = null) {
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
  if (read) {
    input.readOnly = true;
    input.title = "Drag the shape's handles on the slide to change it";
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
        radios(b, "fp-fill", ["No fill", "Solid fill", "Gradient fill", "Picture or texture fill", "Pattern fill", "Slide background fill"]);
        field(b, "Color", "color");
        field(b, "Transparency");
      }),
      section("Line", true, (b) => {
        radios(b, "fp-line", ["No line", "Solid line", "Gradient line"]);
        field(b, "Color", "color");
        field(b, "Transparency");
        field(b, "Width");
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
        field(b, "Height", "text", (s) => inches(s.cy));
        field(b, "Width", "text", (s) => inches(s.cx));
        field(b, "Rotation", "text", (s) => `${Math.round(((s.rot % 360) + 360) % 360)}°`);
        field(b, "Lock aspect ratio", "check");
      }),
      section("Position", true, (b) => {
        field(b, "Horizontal position", "text", (s) => inches(s.x));
        field(b, "Vertical position", "text", (s) => inches(s.y));
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
