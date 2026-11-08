// The Design tab (`VS-12`): Themes, Variants, Slide Size and Format Background, as
// PowerPoint has them. Each verb is a registered command and each edit one engine
// operation, so it is one undo step and saves through the one writer.
//
// Shaped after OpenCalc's ribbon galleries (`webapp/ribbon.model.js` at `b23c67c`,
// webapp/README.md): the catalogue comes from the engine (`design_themes`), the page
// draws swatches from it and keeps no model. The themes are our own palettes, not
// Microsoft's. No document string reaches the DOM: the names shown are the engine's
// built-in ones, set with `textContent`.

import { state, request, status, changed } from "./editor.core.js";
import { register } from "./commands.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";
import { showModal, closeModal, errorDialog } from "./editor.dialogs.js";

const EMU_PER_INCH = 914400;
/// `ST_SlideSizeCoordinate`'s range, in inches: 1 to 56.
const MIN_INCHES = 1, MAX_INCHES = 56;
const SIZES = {
  widescreen: [12192000, 6858000],   // 13.333 by 7.5 inches, 16:9
  standard: [9144000, 6858000],      // 10 by 7.5 inches, 4:3
};

let themes = null;    // [{ name, major, minor, variants: [[12 × "RRGGBB"]] }]
let current = 0;      // the built-in theme last applied, whose variants Variants shows
let busy = false;

function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

async function catalogue() {
  if (themes) return themes;
  const r = await request({ op: "themes" }, { priority: 9 });
  themes = r.themes.map((line) => {
    const [name, major, minor, variants] = line.split("|");
    return { name, major, minor, variants: variants.split(";").map((v) => v.split(",")) };
  });
  return themes;
}

/// Run one Design edit in the engine, then draw what it changed.
async function edit(what, fn, args, wide) {
  if (busy || !state.deck) return;
  busy = true;
  changed();
  try {
    const r = await request({ op: "design", fn, args }, { priority: 9 });
    selection.noteUndoState(r.undo);
    if (!r.value) { status(`${what}: nothing changed`); return; }
    if (wide || r.size[0] !== state.deck.width || r.size[1] !== state.deck.height) slides.redrawAll(r.size);
    else slides.redrawSlide(state.current);
    status(what);
  } catch (e) {
    errorDialog(`OpenPresent could not finish: ${what.toLowerCase()}.`, e.message);
  } finally {
    busy = false;
    changed();
  }
}

/* ── Themes and Variants ───────────────────────────────────────────────── */

function swatches(colors) {
  const row = el("span", "dz-swatches");
  // Accents 1 to 6, as PowerPoint's gallery shows a theme.
  for (const c of colors.slice(4, 10)) {
    const s = el("span", "dz-swatch");
    s.style.backgroundColor = `#${c}`;
    row.appendChild(s);
  }
  return row;
}

function card(label, font, colors, onPick) {
  const b = el("button", "dz-card");
  b.type = "button";
  b.setAttribute("aria-label", label);
  const face = el("span", "dz-face", "Aa");
  face.style.fontFamily = `"${font}", sans-serif`;
  face.style.color = `#${colors[2]}`;
  face.style.backgroundColor = `#${colors[1]}`;
  b.append(face, swatches(colors), el("span", "dz-name", label));
  b.addEventListener("click", () => { closeModal(); onPick(); });
  return b;
}

async function themesDialog() {
  let list;
  try { list = await catalogue(); } catch (e) { errorDialog("The themes could not be listed.", e.message); return; }
  showModal("Themes", (body) => {
    body.appendChild(el("p", "op-muted", "A theme sets every master's colours and fonts. Undo takes it back."));
    const grid = el("div", "dz-grid");
    list.forEach((t, i) => grid.appendChild(card(t.name, t.major, t.variants[0], () => {
      current = i;
      edit(`Theme: ${t.name}`, "design_apply_theme", [i, 0, false], true);
    })));
    body.appendChild(grid);
  });
}

async function variantsDialog() {
  let list;
  try { list = await catalogue(); } catch (e) { errorDialog("The variants could not be listed.", e.message); return; }
  const t = list[current];
  showModal(`Variants: ${t.name}`, (body) => {
    body.appendChild(el("p", "op-muted", "A variant changes the colours only; the fonts stay."));
    const grid = el("div", "dz-grid");
    t.variants.forEach((v, j) => grid.appendChild(card(`${t.name} ${j + 1}`, t.major, v, () => {
      edit(`Variant: ${t.name} ${j + 1}`, "design_apply_theme", [current, j, true], true);
    })));
    body.appendChild(grid);
  });
}

/* ── Slide Size ─────────────────────────────────────────────────────────── */

/// PowerPoint asks how content fits a new size: Maximize or Ensure Fit.
function fitDialog(cx, cy, label) {
  if (cx === state.deck.width && cy === state.deck.height) { status("The slides are that size already"); return; }
  showModal("Slide Size", (body) => {
    body.appendChild(el("p", "op-confirm-text", "You are scaling to a new slide size. Do you want to maximize the size of your content, or scale it down to make sure it fits on the new slide?"));
    const actions = el("div", "op-confirm-actions");
    for (const [text, fit, primary] of [["Maximize", 1, false], ["Ensure Fit", 0, true]]) {
      const b = el("button", primary ? "op-btn primary" : "op-btn", text);
      b.type = "button";
      b.addEventListener("click", () => { closeModal(); edit(`Slide Size: ${label}`, "design_slide_size", [cx, cy, fit], true); });
      actions.appendChild(b);
    }
    body.appendChild(actions);
  });
}

function numberField(label, value) {
  const wrap = el("label", "dz-field", label);
  const input = el("input");
  input.type = "number";
  input.min = String(MIN_INCHES);
  input.max = String(MAX_INCHES);
  input.step = "0.01";
  input.value = value.toFixed(2);
  wrap.appendChild(input);
  return [wrap, input];
}

function customSizeDialog() {
  showModal("Slide Size", (body) => {
    const [wl, w] = numberField("Width (in)", state.deck.width / EMU_PER_INCH);
    const [hl, h] = numberField("Height (in)", state.deck.height / EMU_PER_INCH);
    body.append(wl, hl);
    const note = el("p", "op-muted", `Each side from ${MIN_INCHES} to ${MAX_INCHES} inches.`);
    body.appendChild(note);
    const actions = el("div", "op-confirm-actions");
    const cancel = el("button", "op-btn", "Cancel");
    cancel.type = "button";
    cancel.addEventListener("click", closeModal);
    const ok = el("button", "op-btn primary", "OK");
    ok.type = "button";
    ok.addEventListener("click", () => {
      const a = Number(w.value), b = Number(h.value);
      const fits = (x) => Number.isFinite(x) && x >= MIN_INCHES && x <= MAX_INCHES;
      if (!fits(a) || !fits(b)) { note.textContent = `Enter each side from ${MIN_INCHES} to ${MAX_INCHES} inches.`; return; }
      closeModal();
      fitDialog(Math.round(a * EMU_PER_INCH), Math.round(b * EMU_PER_INCH), "Custom");
    });
    actions.append(cancel, ok);
    body.appendChild(actions);
  });
}

/* ── Format Background ──────────────────────────────────────────────────── */

const hex = (input) => parseInt(input.value.slice(1), 16) >>> 0;

function backgroundDialog() {
  showModal("Format Background", (body) => {
    const kind = el("div", "dz-radios");
    const radios = {};
    for (const [value, text] of [["solid", "Solid fill"], ["gradient", "Gradient fill"]]) {
      const l = el("label", "dz-radio");
      const r = el("input");
      r.type = "radio";
      r.name = "dz-fill";
      r.value = value;
      r.checked = value === "solid";
      radios[value] = r;
      l.append(r, document.createTextNode(` ${text}`));
      kind.appendChild(l);
    }
    const color = (label, value) => {
      const l = el("label", "dz-field", label);
      const i = el("input");
      i.type = "color";
      i.value = value;
      l.appendChild(i);
      return [l, i];
    };
    const [c1l, c1] = color("Color", "#ffffff");
    const [c2l, c2] = color("Second color", "#1f6fb2");
    const angleLabel = el("label", "dz-field", "Angle");
    const angle = el("select");
    for (const a of [0, 45, 90, 135, 180, 225, 270, 315]) {
      const o = el("option", null, `${a}°`);
      o.value = String(a);
      angle.appendChild(o);
    }
    angle.value = "90";
    angleLabel.appendChild(angle);
    const sync = () => { const g = radios.gradient.checked; c2l.hidden = !g; angleLabel.hidden = !g; };
    radios.solid.addEventListener("change", sync);
    radios.gradient.addEventListener("change", sync);
    sync();
    body.append(kind, c1l, c2l, angleLabel);

    const apply = (index, label) => {
      const g = radios.gradient.checked;
      closeModal();
      edit(label, "design_background", [index, g ? 1 : 0, hex(c1), hex(c2), Number(angle.value)], index < 0);
    };
    const actions = el("div", "op-confirm-actions");
    for (const [text, cls, fn] of [
      ["Reset Background", "op-btn", () => { closeModal(); edit("Reset Background", "design_background", [state.current, 2, 0, 0, 0], false); }],
      ["Apply to All", "op-btn", () => apply(-1, "Background: all slides")],
      ["Apply", "op-btn primary", () => apply(state.current, `Background: slide ${state.current + 1}`)],
    ]) {
      const b = el("button", cls, text);
      b.type = "button";
      b.addEventListener("click", fn);
      actions.appendChild(b);
    }
    body.appendChild(actions);
  });
}

/* ── Commands ───────────────────────────────────────────────────────────── */

export function registerDesignCommands() {
  const can = () => !!state.deck && !busy && !slides.showing();
  // A tile applies its theme on one click, as PowerPoint's gallery; More lists them all (`UX-003`).
  register("design.themes", {
    label: "Themes", enabled: can, run: themesDialog,
    pick: async (name) => {
      let list;
      try { list = await catalogue(); } catch (e) { errorDialog("The themes could not be listed.", e.message); return; }
      const i = list.findIndex((t) => t.name === name);
      if (i < 0) { themesDialog(); return; }
      current = i;
      edit(`Theme: ${name}`, "design_apply_theme", [i, 0, false], true);
    },
  });
  register("design.variants", { label: "Variants", enabled: can, run: variantsDialog });
  register("design.slide-size", { label: "Slide Size", enabled: can, run: () => {} });
  register("design.slide-size.standard", { label: "Standard (4:3)", enabled: can, checked: () => !!state.deck && state.deck.width === SIZES.standard[0] && state.deck.height === SIZES.standard[1], run: () => fitDialog(...SIZES.standard, "Standard (4:3)") });
  register("design.slide-size.widescreen", { label: "Widescreen (16:9)", enabled: can, checked: () => !!state.deck && state.deck.width === SIZES.widescreen[0] && state.deck.height === SIZES.widescreen[1], run: () => fitDialog(...SIZES.widescreen, "Widescreen (16:9)") });
  register("design.slide-size.custom", { label: "Custom Slide Size…", enabled: can, run: customSizeDialog });
  register("design.format-background", { label: "Format Background", enabled: can, run: backgroundDialog });
}
