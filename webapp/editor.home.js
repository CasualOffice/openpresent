// The Home tab's Font and Paragraph groups (`UX-003`): the Font and Font Size boxes
// showing the character at the caret and applying a choice, Increase and Decrease
// Font Size, Font Color, Text Highlight Color, the four alignments and Bullets.
//
// As PowerPoint: with a box open they act on its selected text (alignment and
// Bullets on the paragraphs the selection touches, the caret's alone included);
// with text shapes selected and no box open, on all their text. Each is one engine
// call, one undo step, saved by the writer as a patch of the run and paragraph
// properties.
//
// The Font box is OpenCalc's (`webapp/editor.core.js` at `b23c67c`, `fontFamilies`,
// `wireCombo` and the `toolbar.font` command; owner instruction 54): the list is the
// engine's substitution table (`font_families`), every row drawn in its own face
// through the engine's CSS stack (`font_css_stack`), a substituted family saying in
// its tooltip which face it renders as, and a typed name outside the list still
// applies. One change: OpenCalc's page declares the bundled faces with `@font-face`
// files; here the engine already carries them, so the page registers them as
// `FontFace`s from the engine's bytes when the list first opens, and ships no second
// copy.

import { state, request, status } from "./editor.core.js";
import { register } from "./commands.js";
import { queueSync } from "./ribbon.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";
import * as text from "./editor.text.js";
import { openPop, closePop, palette, colourMenu } from "./editor.shapes.js";
import { errorDialog } from "./editor.dialogs.js";

/// PowerPoint's Font Size list, as the engine's `SIZE_LADDER`.
const SIZE_LADDER = [8, 9, 10, 10.5, 11, 12, 14, 16, 18, 20, 24, 28, 32, 36, 40, 44, 48, 54, 60, 66, 72, 80, 88, 96];
/// PowerPoint's Text Highlight Color choices.
const HIGHLIGHTS = [["FFFF00", "Yellow"], ["00FF00", "Bright Green"], ["00FFFF", "Turquoise"], ["FF00FF", "Pink"], ["0000FF", "Blue"],
  ["FF0000", "Red"], ["000080", "Dark Blue"], ["008080", "Teal"], ["008000", "Green"], ["800080", "Violet"],
  ["800000", "Dark Red"], ["808000", "Dark Yellow"], ["808080", "Gray 50%"], ["C0C0C0", "Gray 25%"], ["000000", "Black"]];
const kindNote = { exact: "", metric: " — renders as %s (metric-compatible)", generic: " — renders as %s (closest match)" };

function el(tag, cls, txt) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (txt != null) e.textContent = txt;
  return e;
}

/* ── What the controls show ─────────────────────────────────────────────── */

let shown = null;       // [font, size, align, bulleted] for a selected shape, no box open
let shownKey = "";

/// The state at the caret, or of the selected text shape.
function current() {
  if (text.editing()) return text.caretRunState();
  return selection.hasSelection() ? shown : null;
}

/// The selected text shapes when no box is open: what the controls act on.
const textShapes = () => (text.editing() ? [] : selection.selectedShapes().filter((s) => s.part != null));
const canFormat = () => !!state.deck && (text.editing() || textShapes().length > 0);

async function readShown() {
  const s = textShapes().at(-1);
  const key = s ? `${state.current}:${s.node}` : "";
  if (key === shownKey) return;
  shownKey = key;
  if (!s) { shown = null; queueSync(); return; }
  try {
    shown = (await request({ op: "text", fn: "text_run_state", args: [s.part, s.node, 1] }, { priority: 7 })).value;
  } catch { shown = null; }
  queueSync();
}

/* ── Applying ───────────────────────────────────────────────────────────── */

/// Run `fn` and its arguments over the open box's selection, or over all the text
/// of each selected text shape, as one step each.
async function apply(fn, args, label, { whole = false } = {}) {
  if (text.editing()) {
    await text.formatSelection(async (part, node, start, end) => {
      const r = await request({ op: "text", fn, args: [part, node, start, end, ...args] }, { priority: 8 });
      selection.noteUndoState(r.undo);
      return r.value;
    }, whole);
    status(label);
    return;
  }
  const shapes = textShapes();
  if (!shapes.length) return;
  const index = state.current;
  let any = false;
  try {
    for (const s of shapes) {
      const n = (await request({ op: "text", fn: "text_length", args: [s.part, s.node] }, { priority: 8 })).value;
      if (!n && fn !== "text_set_paragraphs") continue;
      const r = await request({ op: "text", fn, args: [s.part, s.node, 0, n, ...args] }, { priority: 8 });
      selection.noteUndoState(r.undo);
      any = any || r.value;
    }
  } catch (e) {
    errorDialog(`OpenPresent could not apply ${label.toLowerCase()}.`, e.message);
  }
  shownKey = "";
  if (any) { await slides.redrawSlide(index); status(label); } else status(`${label}: nothing changed`);
  readShown();
}

const setRun = (what, value, label) => apply("text_set_run", [what, value], label);

/* ── The Font list (OpenCalc's) ─────────────────────────────────────────── */

let fonts = null;       // { list: [{n, f, k}], stacks: [css] }

/// The engine's font list, its faces registered with the page once.
async function fontList() {
  if (fonts) return fonts;
  const r = await request({ op: "fonts" }, { priority: 9 });
  if (typeof FontFace !== "undefined") {
    for (const [family, bytes] of Object.entries(r.faces || {})) {
      try {
        const face = new FontFace(family, bytes, { weight: "400", style: "normal", display: "swap" });
        document.fonts.add(face);
        face.load().catch(() => {});
      } catch { /* the row falls back to the page's font */ }
    }
  }
  fonts = { list: r.list, stacks: r.stacks };
  return fonts;
}

/// A combo's list, as OpenCalc's `wireCombo`: an editable box that filters the rows
/// as it is typed in and still applies any typed value; arrows, Enter and Escape.
/// `values`: [{ v, label, title?, font? }].
function comboPop(id, values, now, applyValue) {
  openPop([id], "shp-menu op-combo-pop", (p) => {
    const input = el("input", "op-combo-input");
    input.type = "text";
    input.value = now || "";
    input.setAttribute("aria-label", id === "font.name" ? "Font" : "Font Size");
    const menu = el("div", "op-combo-list");
    menu.setAttribute("role", "listbox");
    p.append(input, menu);
    let rows = [];
    let active = -1;
    const highlight = (i) => {
      if (!rows.length) return;
      active = (i + rows.length) % rows.length;
      rows.forEach((b, n) => b.classList.toggle("active", n === active));
      rows[active].scrollIntoView({ block: "nearest" });
    };
    const choose = (v) => { closePop(); applyValue(v); };
    const build = (filter) => {
      const f = (filter || "").trim().toLowerCase();
      const list = f ? values.filter((o) => o.label.toLowerCase().includes(f)) : values;
      const cur = (now || "").trim().toLowerCase();
      menu.textContent = "";
      active = -1;
      rows = list.map((o, i) => {
        const b = el("button", "combo-item", o.label);
        b.type = "button";
        b.setAttribute("role", "option");
        b.dataset.v = o.v;
        if (o.title) b.title = o.title;
        if (o.font) b.style.fontFamily = o.font;
        if (o.v.toLowerCase() === cur) { b.classList.add("checked"); active = i; }
        b.addEventListener("mousedown", (e) => e.preventDefault());
        b.addEventListener("click", (e) => { e.stopPropagation(); choose(o.v); });
        menu.appendChild(b);
        return b;
      });
    };
    build("");
    input.addEventListener("input", () => build(input.value));
    input.addEventListener("keydown", (e) => {
      e.stopPropagation();
      if (e.key === "ArrowDown") { highlight(active + 1); e.preventDefault(); }
      else if (e.key === "ArrowUp") { highlight(active - 1); e.preventDefault(); }
      else if (e.key === "Enter") {
        e.preventDefault();
        const typed = input.value.trim();
        if (rows[active] && rows[active].dataset.v.toLowerCase().includes(typed.toLowerCase())) choose(rows[active].dataset.v);
        else if (typed) choose(typed);
      } else if (e.key === "Escape") { e.preventDefault(); closePop(); }
    });
    requestAnimationFrame(() => {
      input.focus({ preventScroll: true });
      input.select();
      if (active >= 0) rows[active].scrollIntoView({ block: "nearest" });
    });
  });
}

async function fontPicker() {
  if (!canFormat()) return;
  let f;
  try { f = await fontList(); } catch (e) { errorDialog("The fonts could not be listed.", e.message); return; }
  const values = f.list.map((x, i) => ({ v: x.n, label: x.n, font: f.stacks[i], title: x.n + (kindNote[x.k] || "").replace("%s", x.f) }));
  comboPop("font.name", values, current()?.[0] || "", (v) => setRun(0, v, `Font: ${v}`));
}

function sizePicker() {
  if (!canFormat()) return;
  const values = SIZE_LADDER.map((n) => ({ v: String(n), label: String(n) }));
  comboPop("font.size", values, current()?.[1] || "", (v) => {
    const n = parseFloat(v);
    if (!Number.isFinite(n) || n < 1 || n > 4000) { status("Enter a size from 1 to 4000 points."); return; }
    setRun(1, String(n), `Font Size: ${n}`);
  });
}

/* ── Colours ────────────────────────────────────────────────────────────── */

async function fontColour() {
  if (!canFormat()) return;
  const colours = await palette();
  openPop(["font.color"], "shp-menu", (p) => {
    colourMenu(p, colours, "Automatic", (spec) => setRun(2, spec === "none" ? "" : spec, "Font Color"));
  });
}

function highlight() {
  if (!canFormat()) return;
  openPop(["font.highlight"], "shp-menu", (p) => {
    const grid = el("div", "shp-colours");
    for (const [hex, name] of HIGHLIGHTS) {
      const b = el("button", "shp-swatch");
      b.type = "button";
      b.title = name;
      b.setAttribute("aria-label", name);
      b.style.background = `#${hex}`;
      b.addEventListener("click", () => { closePop(); setRun(3, `srgb ${hex}`, "Text Highlight Color"); });
      grid.appendChild(b);
    }
    p.appendChild(grid);
    p.appendChild(el("div", "shp-sep"));
    const none = el("button", "shp-menu-item", "No Color");
    none.type = "button";
    none.addEventListener("click", () => { closePop(); setRun(3, "", "Text Highlight Color"); });
    p.appendChild(none);
  });
}

/* ── Lists and line spacing (`PAR-06`) ──────────────────────────────────── */

/// The Bullets gallery, as PowerPoint's: the engine's style name, the character it
/// writes (Wingdings' for the symbol ones), and the name.
export const BULLET_STYLES = [
  ["round", "•", "Filled Round Bullets"], ["hollow", "o", "Hollow Round Bullets"],
  ["square", "§", "Filled Square Bullets"], ["hollow-square", "q", "Hollow Square Bullets"],
  ["star", "v", "Star Bullets"], ["arrow", "Ø", "Arrow Bullets"], ["check", "ü", "Checkmark Bullets"],
];
/// The Numbering gallery: the scheme the engine writes and how it reads.
export const NUMBER_STYLES = [
  ["arabicPeriod", "1. 2. 3."], ["arabicParenR", "1) 2) 3)"], ["romanUcPeriod", "I. II. III."],
  ["alphaUcPeriod", "A. B. C."], ["alphaLcParenR", "a) b) c)"], ["alphaLcPeriod", "a. b. c."], ["romanLcPeriod", "i. ii. iii."],
];
/// The Line Spacing menu, in lines.
export const LINE_SPACINGS = ["1.0", "1.5", "2.0", "2.5", "3.0"];

/// Increase (`up`) or Decrease (`down`) List Level, for the ribbon and Tab and Shift+Tab.
export const listLevel = (way) => apply("text_set_paragraphs", [3, way], way === "up" ? "Increase List Level" : "Decrease List Level", { whole: true });

function registerLists() {
  const para = (args, label) => () => apply("text_set_paragraphs", args, label, { whole: true });
  register("para.bullets.none", { label: "None", enabled: canFormat, checked: () => canFormat() && !current()?.[4], run: para([1, "off"], "Bullets: None") });
  for (const [name, ch, label] of BULLET_STYLES) {
    register(`para.bullets.${name}`, { label, enabled: canFormat, checked: () => canFormat() && current()?.[4] === `c:${ch}`, run: para([1, name], label) });
  }
  register("para.numbering", {
    label: "Numbering", enabled: canFormat,
    checked: () => canFormat() && !!current()?.[4]?.startsWith("n:"),
    run: para([2, "toggle"], "Numbering"),
  });
  register("para.numbering.none", { label: "None", enabled: canFormat, checked: () => canFormat() && !current()?.[4], run: para([2, "off"], "Numbering: None") });
  for (const [scheme, label] of NUMBER_STYLES) {
    register(`para.numbering.${scheme}`, { label, enabled: canFormat, checked: () => canFormat() && current()?.[4] === `n:${scheme}`, run: para([2, scheme], `Numbering: ${label}`) });
  }
  register("para.indent-more", { label: "Increase List Level", shortcut: "Tab", enabled: canFormat, run: () => listLevel("up") });
  register("para.indent-less", { label: "Decrease List Level", shortcut: "Shift+Tab", enabled: canFormat, run: () => listLevel("down") });
  for (const v of LINE_SPACINGS) {
    register(`para.line-spacing.${v}`, {
      label: v, enabled: canFormat,
      checked: () => canFormat() && !!current()?.[6] && Number(current()[6]) === Number(v),
      run: para([4, v], `Line Spacing: ${v}`),
    });
  }
}

/* ── Commands ───────────────────────────────────────────────────────────── */

export function init() {
  register("font.name", {
    label: "Font", enabled: canFormat, run: fontPicker,
    value: () => (canFormat() ? current()?.[0] || "" : ""),
  });
  register("font.size", {
    label: "Font Size", enabled: canFormat, run: sizePicker,
    value: () => (canFormat() ? current()?.[1] || "" : ""),
  });
  register("font.increase", { label: "Increase Font Size", shortcut: "Ctrl+Shift+>", enabled: canFormat, run: () => apply("text_grow", [true], "Increase Font Size") });
  register("font.decrease", { label: "Decrease Font Size", shortcut: "Ctrl+Shift+<", enabled: canFormat, run: () => apply("text_grow", [false], "Decrease Font Size") });
  register("font.color", { label: "Font Color", enabled: canFormat, run: fontColour });
  // The Font group's rest (`PAR-20`): each over the selection, or all the text of
  // each selected shape, one undo step a shape.
  const bit = (b) => () => !!(text.caretFormatBits() & b);
  register("font.strike", { label: "Strikethrough", enabled: canFormat, checked: bit(8), run: () => apply("text_toggle", [3], "Strikethrough") });
  register("font.superscript", { label: "Superscript", shortcut: "Ctrl+Shift+=", enabled: canFormat, checked: bit(16), run: () => apply("text_toggle", [4], "Superscript") });
  register("font.subscript", { label: "Subscript", shortcut: "Ctrl+=", enabled: canFormat, checked: bit(32), run: () => apply("text_toggle", [5], "Subscript") });
  register("font.clear", { label: "Clear All Formatting", shortcut: "Ctrl+Space", enabled: canFormat, run: () => setRun(5, "", "Clear All Formatting") });
  register("font.case", { label: "Change Case", enabled: canFormat, run: () => {} });
  for (const [mode, label] of [["sentence", "Sentence case."], ["lower", "lowercase"], ["upper", "UPPERCASE"], ["title", "Capitalize Each Word"], ["toggle", "tOGGLE cASE"]]) {
    register(`font.case.${mode}`, { label, enabled: canFormat, run: () => apply("text_change_case", [mode], "Change Case") });
  }
  register("font.spacing", { label: "Character Spacing", enabled: canFormat, run: () => {} });
  // PowerPoint's presets, in points.
  for (const [id, label, pts] of [["very-tight", "Very Tight", "-3"], ["tight", "Tight", "-1.5"], ["normal", "Normal", ""], ["loose", "Loose", "3"], ["very-loose", "Very Loose", "6"]]) {
    register(`font.spacing.${id}`, { label, enabled: canFormat, run: () => setRun(4, pts, `Character Spacing: ${label}`) });
  }
  register("font.highlight", { label: "Text Highlight Color", enabled: canFormat, run: highlight });
  for (const [id, token, label, key] of [["para.left", "l", "Align Left", "Ctrl+L"], ["para.center", "ctr", "Center", "Ctrl+E"], ["para.right", "r", "Align Right", "Ctrl+R"], ["para.justify", "just", "Justify", "Ctrl+J"]]) {
    register(id, {
      label, shortcut: key, enabled: canFormat,
      checked: () => canFormat() && current()?.[2] === token,
      run: () => apply("text_set_paragraphs", [0, token], label, { whole: true }),
    });
  }
  register("para.bullets", {
    label: "Bullets", enabled: canFormat,
    checked: () => canFormat() && current()?.[3] === "1",
    run: () => apply("text_set_paragraphs", [1, "toggle"], "Bullets", { whole: true }),
  });
  registerLists();
  document.addEventListener("op-changed", () => { readShown(); });
}

