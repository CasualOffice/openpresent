// Rich objects (`VS-11`): Insert > Table with a grid picker, typing in a table's cells,
// Insert and Delete Rows and Columns, Table Design's styles from the deck's own
// `tableStyles`; Insert > Chart (column, bar, line, pie) and Edit Data, a small data
// sheet. Every verb is a registered command, and each edit is one engine op with its
// inverse, so Undo and Redo are the shared `edit.undo`/`edit.redo`.
//
// The data sheet is shaped after OpenCalc's chart panel (`webapp/editor.charts.js` and
// `buildChartPanel` in its `editor.core.js` at `b23c67c`, webapp/README.md, owner
// instruction 42): plain text fields committed on OK, Enter to accept, built from
// nodes. Here it is a grid, categories down and series across, as PowerPoint's Edit
// Data sheet. Cell text, style names and chart labels are document text: they reach
// the DOM through `textContent` and form values only.
//
// A cell is typed in a field laid over it, committed when the field is left: a thin
// pass. Caret-level editing inside a cell through slice 5's engine is a leftover.

import { register } from "./commands.js";
import { state, request, status, changed } from "./editor.core.js";
import { showModal, closeModal, errorDialog } from "./editor.dialogs.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";

const byId = (id) => document.getElementById(id);
const TABLE = 256, CHART = 512;                 // `ShapeBox` flags
const PICK_ROWS = 8, PICK_COLS = 10;            // PowerPoint's Insert Table grid
const CHART_KINDS = [["Column", 0], ["Bar", 1], ["Line", 2], ["Pie", 3]];

const has = () => !!state.deck && state.deck.count > 0;
const sel = () => selection.selectedShape();
const tableSel = () => { const s = sel(); return s && (s.flags & TABLE) ? s : null; };
const chartSel = () => { const s = sel(); return s && (s.flags & CHART) ? s : null; };
export const tableSelected = () => !!state.deck && !!tableSel();
export const chartSelected = () => !!state.deck && !!chartSel();

let activeCell = { row: 0, col: 0 };            // the cell last typed in

function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

function button(label, cls = "op-btn") {
  const b = el("button", cls, label);
  b.type = "button";
  return b;
}

async function call(fn, args = []) {
  const r = await request({ op: "build", fn, args }, { priority: 9 });
  selection.noteUndoState(r.undo);
  return r;
}

async function ask(fn, args = []) {
  return (await request({ op: "build", fn, args }, { priority: 9 })).value;
}

async function guarded(what, f) {
  if (!has()) return;
  try {
    await f();
  } catch (e) {
    errorDialog(`OpenPresent could not ${what}.`, e.message);
  }
}

async function redraw(select) {
  if (select != null) selection.selectNode(select);
  await slides.redrawSlide(state.current);
  changed();
}

/* ── Insert ─────────────────────────────────────────────────────────────── */

function insertTable(rows, cols) {
  return guarded("insert the table", async () => {
    const W = state.deck.width, H = state.deck.height;
    const cx = Math.round(W * 0.8), cy = Math.round(Math.min(H * 0.6, 370840 * rows));
    const r = await call("insert_table", [state.current, rows, cols, Math.round((W - cx) / 2), Math.round(H * 0.25), cx, cy]);
    activeCell = { row: 0, col: 0 };
    await redraw(r.value);
    status(`Table inserted: ${cols} × ${rows}`);
  });
}

/// Insert > Table: PowerPoint's grid. Hover to size it, click to insert; or give any
/// size up to PowerPoint's 75 below it, as Insert Table… does.
function tablePicker() {
  showModal("Insert Table", (body) => {
    const label = el("p", "op-muted", "1 × 1 Table");
    const grid = el("div", "op-table-pick");
    const cells = [];
    const mark = (r, c) => {
      for (const b of cells) b.classList.toggle("is-on", Number(b.dataset.r) <= r && Number(b.dataset.c) <= c);
      label.textContent = `${c + 1} × ${r + 1} Table`;
    };
    for (let r = 0; r < PICK_ROWS; r++) {
      for (let c = 0; c < PICK_COLS; c++) {
        const b = button("", "op-table-cell");
        b.dataset.r = r;
        b.dataset.c = c;
        b.setAttribute("aria-label", `${c + 1} by ${r + 1} table`);
        b.addEventListener("mouseenter", () => mark(r, c));
        b.addEventListener("focus", () => mark(r, c));
        b.addEventListener("click", () => { closeModal(); insertTable(r + 1, c + 1); });
        cells.push(b);
        grid.appendChild(b);
      }
    }
    body.appendChild(label);
    body.appendChild(grid);
    const row = el("div", "op-confirm-actions");
    const num = (v, name) => {
      const i = el("input", "op-table-size");
      i.type = "number"; i.min = "1"; i.max = "75"; i.value = String(v);
      i.setAttribute("aria-label", name);
      return i;
    };
    const cols = num(5, "Number of columns"), rows = num(2, "Number of rows");
    const ok = button("Insert", "op-btn");
    ok.addEventListener("click", () => {
      const c = Math.max(1, Math.min(75, Number(cols.value) | 0)), r = Math.max(1, Math.min(75, Number(rows.value) | 0));
      closeModal();
      insertTable(r, c);
    });
    row.append(el("span", "op-muted", "Columns"), cols, el("span", "op-muted", "Rows"), rows, ok);
    body.appendChild(row);
    mark(0, 0);
  });
}

function insertChart(kind) {
  return guarded("insert the chart", async () => {
    const W = state.deck.width, H = state.deck.height;
    const cx = Math.round(W * 0.6), cy = Math.round(H * 0.6);
    const r = await call("insert_chart", [state.current, kind, Math.round((W - cx) / 2), Math.round((H - cy) / 2), cx, cy]);
    await redraw(r.value);
    status("Chart inserted");
    await editData(r.value);
  });
}

/// Insert > Chart: the four kinds this build writes.
function chartPicker() {
  showModal("Insert Chart", (body) => {
    const grid = el("div", "op-layouts");
    for (const [label, kind] of CHART_KINDS) {
      const b = button(label, "op-btn op-layout");
      b.addEventListener("click", () => { closeModal(); insertChart(kind); });
      grid.appendChild(b);
    }
    body.appendChild(grid);
    grid.querySelector("button")?.classList.add("primary");
  });
}

/* ── Tables ─────────────────────────────────────────────────────────────── */

async function tableShape(node) {
  const v = await ask("table_shape", [state.current, node]);
  const rows = v[0], cols = v[1];
  // The rows as drawn, grown to fit their text, when the engine gives them: by the
  // stated heights alone a click landed a row or more below (`UX-015`).
  const drawn = v.length >= 2 + cols + 2 * rows ? v.slice(2 + cols + rows, 2 + cols + 2 * rows) : null;
  return { rows, cols, widths: v.slice(2, 2 + cols), heights: drawn || v.slice(2 + cols, 2 + cols + rows) };
}

/// The cell under a point of the slide, EMU, by the grid's widths and stated heights.
function cellAt(s, t, x, y) {
  const rowH = (i) => t.heights[i] || s.cy / t.rows;
  let col = 0, acc = s.x;
  for (; col < t.cols - 1; col++) { acc += t.widths[col]; if (x < acc) break; }
  let row = 0;
  acc = s.y;
  for (; row < t.rows - 1; row++) { acc += rowH(row); if (y < acc) break; }
  return { row, col };
}

function cellBox(s, t, row, col) {
  let x = s.x, y = s.y;
  for (let c = 0; c < col; c++) x += t.widths[c];
  for (let r = 0; r < row; r++) y += t.heights[r] || s.cy / t.rows;
  return { x, y, cx: t.widths[col] || s.cx / t.cols, cy: t.heights[row] || s.cy / t.rows };
}

let cellEditor = null;

/// Type into a cell: a field over it; Tab and Shift+Tab move on, Escape puts it back.
/// `whole` selects the cell's text, as Tab does in PowerPoint; a click puts the caret
/// at its end instead, so a key typed after it adds to the text (`UX-015`).
async function editCell(s, row, col, whole = true) {
  await closeCellEditor(true);
  const t = await tableShape(s.node);
  if (row >= t.rows || col >= t.cols) return;
  activeCell = { row, col };
  const index = state.current;
  const text = await ask("table_cell_text", [index, s.node, row, col]);
  const b = cellBox(s, t, row, col);
  const k = byId("slide-frame").clientWidth / state.deck.width;
  const area = el("textarea", "op-cell-editor");
  area.value = text;
  area.setAttribute("aria-label", `Row ${row + 1}, column ${col + 1}`);
  Object.assign(area.style, {
    left: `${b.x * k}px`, top: `${b.y * k}px`,
    width: `${Math.max(24, b.cx * k)}px`, height: `${Math.max(18, b.cy * k)}px`,
  });
  byId("slide-frame").appendChild(area);
  const me = { area, node: s.node, row, col, index, original: text };
  cellEditor = me;
  area.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.key === "Escape") {
      area.value = me.original;
      closeCellEditor(false);
      byId("stage").focus();
    } else if (e.key === "Tab") {
      e.preventDefault();
      const n = row * t.cols + col + (e.shiftKey ? -1 : 1);
      if (n < 0 || n >= t.rows * t.cols) { closeCellEditor(true); return; }
      editCell(s, Math.floor(n / t.cols), n % t.cols);
    }
  });
  area.addEventListener("blur", () => { if (cellEditor === me) closeCellEditor(true); });
  area.focus();
  if (whole) area.select();
  else area.setSelectionRange(area.value.length, area.value.length);
}

async function closeCellEditor(commit) {
  const c = cellEditor;
  if (!c) return;
  cellEditor = null;
  const value = c.area.value;
  c.area.remove();
  if (!commit || value === c.original) return;
  await guarded("type in the cell", async () => {
    await call("set_table_cell_text", [c.index, c.node, c.row, c.col, value]);
    await redraw(c.node);
  });
}

function tableEdit(what, place) {
  return guarded("change the table", async () => {
    const s = tableSel();
    if (!s) return;
    await call("edit_table", [state.current, s.node, what, place(activeCell)]);
    if (what === 1) activeCell.row = Math.max(0, activeCell.row - 1);
    if (what === 3) activeCell.col = Math.max(0, activeCell.col - 1);
    await redraw(s.node);
  });
}

/// Table Design > Table Styles: the deck's own styles, the table's marked.
function stylesPicker() {
  return guarded("read the table styles", async () => {
    const s = tableSel();
    if (!s) return;
    const v = await ask("table_styles");
    const now = await ask("table_style", [state.current, s.node]);
    showModal("Table Styles", (body) => {
      const grid = el("div", "op-layouts");
      for (let i = 0; i + 1 < v.length; i += 2) {
        const id = v[i];
        const b = button(v[i + 1], "op-btn op-layout");
        if (id === now) { b.classList.add("primary"); b.setAttribute("aria-pressed", "true"); }
        b.addEventListener("click", () => {
          closeModal();
          guarded("apply the style", async () => {
            await call("set_table_style", [state.current, s.node, id]);
            await redraw(s.node);
          });
        });
        grid.appendChild(b);
      }
      body.appendChild(grid);
    });
  });
}

/// PowerPoint's Medium Style 2 row, by the built-in style ids the ribbon's tiles name.
const TILE_STYLES = {
  "Medium Style 2 - Accent 1": "{5C22544A-7EE6-4342-B048-85BDC9FD1C3A}",
  "Medium Style 2 - Accent 2": "{21E4AEA4-8DFA-4A89-87EB-49C32662AFE8}",
  "Medium Style 2 - Accent 3": "{F5AB1C69-6EDB-4FF4-983F-18BD219EF322}",
  "Medium Style 2 - Accent 4": "{00A15C55-8517-42AA-B614-E9B94910E393}",
  "Medium Style 2 - Accent 5": "{7DF18680-E054-41AD-8BC1-D1AEF772440D}",
  "Medium Style 2 - Accent 6": "{93296810-A885-4BE3-A3E7-6D5BEEA58F35}",
};

function pickTableStyle(name) {
  const id = TILE_STYLES[name];
  if (!id) return stylesPicker();
  return guarded("apply the style", async () => {
    const s = tableSel();
    if (!s) return;
    await call("set_table_style", [state.current, s.node, id]);
    await redraw(s.node);
  });
}

/* ── Charts: Edit Data ──────────────────────────────────────────────────── */

/// The data sheet: categories down, series across, as PowerPoint's Edit Data.
function editData(nodeArg) {
  return guarded("read the chart's data", async () => {
    const node = nodeArg ?? chartSel()?.node;
    if (node == null) return;
    const index = state.current;
    const v = await ask("chart_data", [index, node]);
    const n = Number(v[1]), m = Number(v[2]);
    const cats = v.slice(3, 3 + n);
    const series = [];
    for (let j = 0; j < m; j++) {
      const at = 3 + n + j * (n + 1);
      series.push({ name: v[at], values: v.slice(at + 1, at + 1 + n) });
    }
    showModal("Edit Data", (body) => {
      const apply = button("OK", "op-btn primary");
      const wrap = el("div", "op-sheet-wrap");
      const sheet = el("table", "op-sheet");
      const field = (value, label, onInput) => {
        const i = el("input", "op-sheet-field");
        i.type = "text";
        i.value = value;
        i.setAttribute("aria-label", label);
        i.addEventListener("input", () => onInput(i.value));
        i.addEventListener("keydown", (e) => { if (e.key === "Enter") { e.preventDefault(); apply.click(); } });
        return i;
      };
      const render = () => {
        sheet.textContent = "";
        const head = el("tr");
        head.appendChild(el("th"));
        series.forEach((s, j) => {
          const th = el("th");
          th.appendChild(field(s.name, `Series ${j + 1} name`, (x) => { s.name = x; }));
          head.appendChild(th);
        });
        sheet.appendChild(head);
        cats.forEach((c, k) => {
          const tr = el("tr");
          const th = el("th");
          th.appendChild(field(c, `Category ${k + 1}`, (x) => { cats[k] = x; }));
          tr.appendChild(th);
          series.forEach((s, j) => {
            const td = el("td");
            const g = field(s.values[k] ?? "", `Series ${j + 1}, category ${k + 1}`, (x) => { s.values[k] = x; });
            g.inputMode = "decimal";
            td.appendChild(g);
            tr.appendChild(td);
          });
          sheet.appendChild(tr);
        });
      };
      render();
      wrap.appendChild(sheet);
      body.appendChild(wrap);
      const actions = el("div", "op-confirm-actions");
      const addCat = button("Add Category");
      addCat.addEventListener("click", () => { cats.push(`Category ${cats.length + 1}`); for (const s of series) s.values.push(""); render(); });
      const addSer = button("Add Series");
      addSer.disabled = v[0] === "3";
      addSer.addEventListener("click", () => { series.push({ name: `Series ${series.length + 1}`, values: cats.map(() => "") }); render(); });
      apply.addEventListener("click", () => {
        closeModal();
        guarded("change the chart's data", async () => {
          const out = [v[0], String(cats.length), String(series.length), ...cats];
          for (const s of series) out.push(s.name, ...cats.map((_, k) => String(s.values[k] ?? "")));
          await call("set_chart_data", [index, node, out]);
          await redraw(node);
          status("Chart data changed");
        });
      });
      const cancel = button("Cancel");
      cancel.addEventListener("click", closeModal);
      body.appendChild(el("p", "op-muted", "A chart from the file keeps its number of categories and series, and its embedded workbook as it was."));
      actions.append(addCat, addSer, cancel, apply);
      body.appendChild(actions);
    });
  });
}

/* ── Wiring ─────────────────────────────────────────────────────────────── */

/// A double click on a selected table types in the cell under it; on a chart, Edit Data.
function onDoubleClick(e) {
  const s = sel();
  if (!s || !(s.flags & (TABLE | CHART)) || !state.deck) return;
  const r = byId("slide-frame").getBoundingClientRect();
  const k = byId("slide-frame").clientWidth / state.deck.width;
  const x = (e.clientX - r.left) / k, y = (e.clientY - r.top) / k;
  if (x < s.x || y < s.y || x > s.x + s.cx || y > s.y + s.cy) return;
  e.preventDefault();
  e.stopPropagation();
  if (s.flags & CHART) { editData(s.node); return; }
  tableShape(s.node).then((t) => { const c = cellAt(s, t, x, y); editCell(s, c.row, c.col); });
}

export function init() {
  register("insert.table", { label: "Table", enabled: has, run: tablePicker });
  register("insert.chart", { label: "Chart", enabled: has, run: chartPicker });
  for (const [label, kind] of CHART_KINDS) {
    register(`insert.chart.${label.toLowerCase()}`, { label: `${label} Chart`, enabled: has, run: () => insertChart(kind) });
  }
  register("chart.edit-data", { label: "Edit Data", enabled: chartSelected, run: () => editData() });
  // A gallery tile applies its style on one click (`UX-003`); More lists the deck's.
  register("table.styles", { label: "Table Styles", enabled: tableSelected, run: stylesPicker, pick: pickTableStyle });
  register("table.edit-cell", { label: "Edit Cell Text", enabled: tableSelected, run: () => { const s = tableSel(); if (s) editCell(s, activeCell.row, activeCell.col); } });
  register("table.insert-above", { label: "Insert Above", enabled: tableSelected, run: () => tableEdit(0, (c) => c.row) });
  register("table.insert-below", { label: "Insert Below", enabled: tableSelected, run: () => tableEdit(0, (c) => c.row + 1) });
  register("table.insert-left", { label: "Insert Left", enabled: tableSelected, run: () => tableEdit(2, (c) => c.col) });
  register("table.insert-right", { label: "Insert Right", enabled: tableSelected, run: () => tableEdit(2, (c) => c.col + 1) });
  register("table.delete-rows", { label: "Delete Rows", enabled: tableSelected, run: () => tableEdit(1, (c) => c.row) });
  register("table.delete-columns", { label: "Delete Columns", enabled: tableSelected, run: () => tableEdit(3, (c) => c.col) });
  byId("slide-frame").addEventListener("dblclick", onDoubleClick, true);
  // One click inside a table types in the cell under it, as PowerPoint (`UX-015`).
  // Keys typed while the cell opens are kept and typed into it, as `editor.text.js`
  // keeps them for a text box.
  selection.onTablePress(async (s, [x, y]) => {
    const held = [];
    const hold = (e) => {
      if (e.ctrlKey || e.metaKey || e.altKey) return;
      if (e.key.length === 1) held.push(e.key);
      else if (e.key === "Enter") held.push("\n");
      else if (e.key === "Backspace") held.pop();
      else return;
      e.preventDefault();
      e.stopPropagation();
    };
    document.addEventListener("keydown", hold, true);
    try {
      const t = await tableShape(s.node);
      const c = cellAt(s, t, x, y);
      await editCell(s, c.row, c.col, false);
    } finally {
      document.removeEventListener("keydown", hold, true);
    }
    const area = cellEditor?.area;
    if (area && held.length) {
      area.value += held.join("");
      area.setSelectionRange(area.value.length, area.value.length);
    }
  });
}
