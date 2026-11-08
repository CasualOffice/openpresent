// Cut, Copy, Paste and Duplicate (`UX-003`); formatted paste from other applications
// and Paste Special (`PAR-11`); copy and paste of whole slides (`PAR-10`).
//
// Copied in approach from OpenCalc's `webapp/editor.clipboard.js` at `b23c67c`
// (webapp/README.md, owner instruction 42): the engine keeps its own copy of what was
// copied, which carries everything the system clipboard's plain text cannot, and the
// page writes a plain-text stand-in to the system clipboard. A paste uses the
// engine's copy while the system clipboard still holds that stand-in (or cannot be
// read), and the system clipboard's text otherwise, as OpenCalc's `doPaste` decides
// between `session_clip_paste` and `session_paste_tsv`. What changes for slides: the
// copy is shapes, slides or text, not cells; text pasted from another application
// while no box is open becomes a new text box, as PowerPoint's; and the engine's copy
// lives outside the open deck, so shapes and slides paste into a new one.
//
// Keys come as the browser's own `copy`, `cut` and `paste` events, which carry the
// system clipboard without asking for a permission; the ribbon's buttons use the
// async clipboard API for writing only. Paste from the ribbon uses the engine's copy,
// since reading the system clipboard there would ask the user for permission.
//
// Formatted text (`PAR-11`): the clipboard's `text/html` is read as OpenCalc's
// `cellsFromClipboardHtml` (`webapp/editor.core.js` at `b23c67c`, docs/68 there) reads
// it: `DOMParser` gives a document with no browsing context, so nothing in it runs or
// fetches; its nodes are never put in the page; only text and the `style` attribute,
// read as text, are consulted. Bold, italic, underline, size, colour and lists pass,
// one engine call and one undo step. Paste Special re-pastes the last paste as Keep
// Source Formatting, Use Destination Theme (bold, italic, underline and lists, no
// size or colour) or Keep Text Only; Ctrl+Shift+V pastes text only. A picture on the
// clipboard from another application pastes as a picture.
//
// Slides (`PAR-10`): with the thumbnail pane the last place pressed, Copy and Cut take
// the current slide and any added with Ctrl+click or Shift+click there, and Paste puts
// the engine's slides after the current one: in this deck as copies, in another as
// PowerPoint's Use Destination Theme, its layouts by name.

import { state, request, status, changed } from "./editor.core.js";
import { register } from "./commands.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";
import * as text from "./editor.text.js";
import { errorDialog } from "./editor.dialogs.js";

const byId = (id) => document.getElementById(id);
/// A new text box for pasted text: PowerPoint's default, 4 by 0.4 inches.
const PASTED_BOX = [3657600, 365760];
/// The most HTML a paste reads, in characters: past this the plain text is pasted.
const MAX_HTML = 2_000_000;

let lastClipText = null;   // the stand-in written for the engine's last copy
let held = 0;              // shapes the engine's clipboard holds
let heldSlides = 0;        // slides it holds
let clipKind = "shapes";   // what the engine's last copy was: "shapes" or "slides"
let paneActive = false;    // the thumbnail pane was the last place pressed
const extraSlides = new Set();   // slides added to the pane's selection
/// The last paste from another application, so Paste Special can redo it another way.
let lastPaste = null;      // { html, text, mode, range: the selection it replaced, count }
let plainNext = false;     // Ctrl+Shift+V: the coming paste is text only

/// Whether keys and events belong to the slide editor, not a dialog or a field.
function ours(target) {
  if (!state.deck || !byId("op-modal").hidden || slides.showing()) return false;
  if (text.editing()) return true;
  const t = target || document.activeElement;
  if (t && !t.classList?.contains("text-input") && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName))) return false;
  return !t?.closest?.(".task-pane, .rb-pop, .shp-pop");
}

async function call(fn, args) {
  return request({ op: "build", fn, args }, { priority: 9 });
}

/// The text of the selected shapes, joined by line feeds: the system clipboard's
/// stand-in for them, as PowerPoint puts a shape's text there.
async function shapesText(shapes) {
  const parts = [];
  for (const s of shapes) {
    if (s.part == null) continue;
    try {
      const n = (await request({ op: "text", fn: "text_length", args: [s.part, s.node] }, { priority: 9 })).value;
      if (n) parts.push((await request({ op: "text", fn: "text_slice", args: [s.part, s.node, 0, n] }, { priority: 9 })).value.replace(/\v/g, "\n"));
    } catch { /* no text */ }
  }
  return parts.join("\n");
}

/* ── Slides in the thumbnail pane (`PAR-10`) ────────────────────────────── */

/// The slides the pane has selected, in order: the current one and any added.
function paneSlides() {
  const all = new Set([state.current, ...extraSlides]);
  return [...all].filter((i) => i >= 0 && i < (state.deck?.count || 0)).sort((a, b) => a - b);
}

/// Copy and Cut act on slides while the pane was the last place pressed and no text
/// box is open.
const slidesFocused = () => paneActive && !text.editing() && !!state.deck?.count;

function markExtra() {
  const list = byId("thumbs");
  for (const li of list.children) {
    const i = Number(li.dataset.index);
    const on = extraSlides.has(i) && i !== state.current;
    li.classList.toggle("is-also", on);
    if (i !== state.current) li.setAttribute("aria-selected", on ? "true" : "false");
  }
}

async function copySlides() {
  const indices = paneSlides();
  try {
    const r = await call("copy_slides", [indices]);
    heldSlides = r.value;
  } catch (e) {
    errorDialog("OpenPresent cannot copy those slides.", e.message);
    return null;
  }
  clipKind = "slides";
  lastClipText = await shapesText(selection.allShapes());
  status(heldSlides === 1 ? "Copied 1 slide" : `Copied ${heldSlides} slides`);
  return lastClipText;
}

async function deleteSlides(indices) {
  let r = null;
  for (const i of [...indices].sort((a, b) => b - a)) r = await call("delete_slide", [i]);
  extraSlides.clear();
  if (r) {
    selection.noteUndoState(r.undo);
    slides.reloadList(r.count, Math.min(indices[0], r.count - 1));
  }
}

async function pasteSlides() {
  const at = state.deck.count ? state.current + 1 : 0;
  try {
    const r = await call("paste_slides", [at]);
    if (!r.value.length) { status("The clipboard is empty"); return; }
    const [first, n, skipped] = r.value;
    selection.noteUndoState(r.undo);
    extraSlides.clear();
    slides.reloadList(r.count, first);
    const what = n === 1 ? "Pasted 1 slide" : `Pasted ${n} slides`;
    status(skipped ? `${what}; ${skipped} object${skipped === 1 ? "" : "s"} could not be carried to this deck yet` : what);
  } catch (e) {
    errorDialog("OpenPresent cannot paste those slides here.", e.message);
  }
}

/* ── Copy, Cut, Paste ───────────────────────────────────────────────────── */

/// Copy: the open box's selected text, the pane's slides, or the selected shapes.
/// Returns the stand-in text for the system clipboard, or null when nothing was copied.
async function copy() {
  if (text.editing()) {
    const t = await text.selectedText();
    if (!t) { status("Select text to copy it."); return null; }
    lastClipText = null;
    return t;
  }
  if (slidesFocused()) return copySlides();
  const shapes = selection.selectedShapes();
  if (!shapes.length) return null;
  try {
    const r = await call("copy_shapes", [state.current, shapes.map((s) => s.node)]);
    held = r.value;
  } catch (e) {
    errorDialog("OpenPresent cannot copy that yet.", e.message);
    return null;
  }
  clipKind = "shapes";
  lastClipText = await shapesText(shapes);
  status(held === 1 ? "Copied 1 shape" : `Copied ${held} shapes`);
  return lastClipText;
}

async function cut() {
  const editing = text.editing();
  const onSlides = slidesFocused();
  const indices = paneSlides();
  const t = await copy();
  if (t == null) return null;
  if (editing) text.cutSelection();
  else if (onSlides) await deleteSlides(indices);
  else await selection.deleteSelection();
  return t;
}

/// Paste the engine's shapes onto the current slide and select them.
async function pasteShapes() {
  const index = state.current;
  try {
    const r = await call("paste_shapes", [index]);
    if (!r.value.length) { status("The clipboard is empty"); return; }
    selection.noteUndoState(r.undo);
    selection.selectNodes(r.value);
    await slides.redrawSlide(index);
    status(r.value.length === 1 ? "Pasted 1 shape" : `Pasted ${r.value.length} shapes`);
  } catch (e) {
    errorDialog("OpenPresent cannot paste that here yet.", e.message);
  }
}

/// Text from another application, with no box open: a new text box holding it,
/// formatted as `rich` says when given.
async function pasteTextBox(t, rich) {
  const index = state.current;
  const [cx, cy] = PASTED_BOX;
  const x = Math.max(0, Math.round((state.deck.width - cx) / 2));
  const y = Math.max(0, Math.round((state.deck.height - cy) / 2));
  try {
    const r = await call("insert_shape", [index, 0, x, y, cx, cy]);
    const node = r.value;
    const hit = await request({ op: "text", fn: "text_hit", args: [index, x + cx / 2, y + cy / 2] }, { priority: 9 });
    const part = hit.value[0];
    const done = rich
      ? await request({ op: "text", fn: "text_paste_rich", args: [part, node, 0, 0, rich.texts, rich.looks, rich.lists] }, { priority: 9 })
      : await request({ op: "text", fn: "text_insert", args: [part, node, 0, t.replace(/\r\n?/g, "\n"), false] }, { priority: 9 });
    selection.noteUndoState(done.undo);
    selection.selectNode(node);
    await slides.redrawSlide(index);
    status("Pasted as a text box");
  } catch (e) {
    errorDialog("OpenPresent could not paste that text.", e.message);
  }
}

/// A picture from another application: inserted at its own proportions, centred.
async function pastePicture(file) {
  const index = state.current;
  try {
    const bytes = await file.arrayBuffer();
    let ratio = 1;
    try {
      const bmp = await createImageBitmap(new Blob([bytes]));
      ratio = bmp.height / Math.max(1, bmp.width);
      bmp.close();
    } catch { /* the engine refuses what is not a picture */ }
    const W = state.deck.width, H = state.deck.height;
    let cx = W * 0.6, cy = cx * ratio;
    if (cy > H * 0.8) { cy = H * 0.8; cx = cy / ratio; }
    const box = [(W - cx) / 2, (H - cy) / 2, cx, cy].map(Math.round);
    const r = await request({ op: "build", fn: "insert_picture", args: [index, bytes, ...box] }, { priority: 9, transfer: [bytes] });
    selection.noteUndoState(r.undo);
    selection.selectNode(r.value);
    await slides.redrawSlide(index);
    status("Pasted as a picture");
  } catch (e) {
    errorDialog("OpenPresent could not paste that picture.", e.message);
  }
}

/* ── Formatted text from other applications (`PAR-11`) ──────────────────── */

/// A colour as `RRGGBB`, from `#rgb`, `#rrggbb` or `rgb()`. OpenCalc's `hex` in
/// `cellsFromClipboardHtml`: named colours are dropped rather than guessed.
function hex(value) {
  if (!value) return null;
  const t = String(value).trim().toLowerCase();
  let m = /^#?([0-9a-f]{6})$/.exec(t);
  if (m) return m[1].toUpperCase();
  m = /^#?([0-9a-f]{3})$/.exec(t);
  if (m) return m[1].split("").map((c) => c + c).join("").toUpperCase();
  m = /^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/.exec(t);
  if (m) return [1, 2, 3].map((i) => Math.min(255, Number(m[i])).toString(16).padStart(2, "0")).join("").toUpperCase();
  return null;
}

/// An element's `style` attribute as declarations, read as text: OpenCalc's
/// `declarations`. Never `getComputedStyle`, which needs the node in the page.
function declarations(el) {
  const out = {};
  for (const part of (el.getAttribute("style") ?? "").split(";")) {
    const at = part.indexOf(":");
    if (at < 0) continue;
    out[part.slice(0, at).trim().toLowerCase()] = part.slice(at + 1).trim();
  }
  return out;
}

/// A CSS font size in hundredths of a point, or null.
function fontSize(value) {
  const m = /^(\d+(?:\.\d+)?)\s*(pt|px)?/.exec(String(value || "").trim());
  if (!m) return null;
  const pt = m[2] === "px" ? Number(m[1]) * 0.75 : Number(m[1]);
  return pt >= 1 && pt <= 4000 ? Math.round(pt * 2) * 50 : null;
}

const BLOCKS = new Set(["P", "DIV", "LI", "H1", "H2", "H3", "H4", "H5", "H6", "TR", "BLOCKQUOTE", "PRE", "UL", "OL", "TABLE", "SECTION", "ARTICLE", "HEADER", "FOOTER"]);
const SKIP = new Set(["SCRIPT", "STYLE", "HEAD", "TITLE", "META", "LINK", "TEMPLATE", "NOSCRIPT", "SVG", "MATH", "OBJECT", "IFRAME"]);

/// The clipboard's HTML as runs and paragraphs: `{ runs: [{ text, b, i, u, size,
/// color }], paras: [[list, level]] }`, list 0 plain, 1 bulleted, 2 numbered. Null
/// when it holds no text.
export function richFromHtml(html) {
  let doc;
  try { doc = new DOMParser().parseFromString(html, "text/html"); } catch { return null; }
  const runs = [];
  const paras = [[0, 0]];
  let atStart = true;   // nothing yet in the current paragraph
  const newParagraph = (list, level) => {
    if (atStart && runs.length) { paras[paras.length - 1] = [list, level]; return; }
    if (!runs.length) { paras[0] = [list, level]; return; }
    runs.push({ text: "\n" });
    paras.push([list, level]);
    atStart = true;
  };
  const walk = (node, look, lists, pre) => {
    if (node.nodeType === 3) {
      let t = node.nodeValue || "";
      if (!pre) {
        t = t.replace(/[\t\n\r ]+/g, " ");
        if (atStart) t = t.replace(/^ +/, "");
      }
      if (!t) return;
      runs.push({ text: t, ...look });
      atStart = false;
      return;
    }
    if (node.nodeType !== 1) return;
    const tag = node.tagName.toUpperCase();
    if (SKIP.has(tag)) return;
    if (tag === "BR") { runs.push({ text: "\v", ...look }); atStart = false; return; }
    if (tag === "IMG") return;
    const css = declarations(node);
    const next = { ...look };
    if (tag === "B" || tag === "STRONG" || /^H[1-6]$/.test(tag) || tag === "TH") next.b = true;
    if (tag === "I" || tag === "EM" || tag === "CITE") next.i = true;
    if (tag === "U" || tag === "INS") next.u = true;
    const weight = css["font-weight"];
    if (weight) next.b = weight === "bold" || weight === "bolder" || Number(weight) >= 600;
    const style = css["font-style"];
    if (style) next.i = /italic|oblique/.test(style);
    const deco = `${css["text-decoration"] ?? ""} ${css["text-decoration-line"] ?? ""}`;
    if (/underline/.test(deco)) next.u = true;
    else if (/\bnone\b/.test(deco)) next.u = false;
    const size = fontSize(css["font-size"]);
    if (size) next.size = size;
    const color = hex(css.color) ?? (tag === "FONT" ? hex(node.getAttribute("color")) : null);
    if (color) next.color = color;
    let inner = lists;
    if (tag === "UL" || tag === "OL") inner = [...lists, tag === "OL" ? 2 : 1];
    const block = BLOCKS.has(tag) && tag !== "UL" && tag !== "OL" && tag !== "TABLE";
    if (block) {
      const inList = tag === "LI" && lists.length;
      newParagraph(inList ? lists[lists.length - 1] : 0, inList ? Math.min(8, lists.length - 1) : 0);
    }
    for (const child of node.childNodes) walk(child, next, inner, pre || tag === "PRE");
    if (block && !atStart) newParagraph(0, 0);
  };
  walk(doc.body || doc.documentElement, {}, [], false);
  // No empty paragraph left at the end, as PowerPoint pastes.
  while (runs.length && runs[runs.length - 1].text === "\n") { runs.pop(); paras.pop(); }
  if (!runs.some((r) => r.text.trim())) return null;
  return { runs, paras };
}

/// What `text_paste_rich` takes for `rich` pasted as `mode`: `keep` (Keep Source
/// Formatting), `theme` (Use Destination Theme) or `text` (Keep Text Only).
export function richArgs(rich, mode) {
  const texts = [], looks = [], lists = [];
  const flag = (v) => (v === undefined ? (mode === "keep" ? 0 : -1) : v ? 1 : 0);
  for (const r of rich.runs) {
    texts.push(r.text);
    if (mode === "text") { looks.push(-1, -1, -1, -1, -1); continue; }
    looks.push(flag(r.b), flag(r.i), flag(r.u),
      mode === "keep" && r.size ? r.size : -1,
      mode === "keep" && r.color ? parseInt(r.color, 16) : -1);
  }
  if (mode !== "text") for (const [list, level] of rich.paras) lists.push(list, level);
  return { texts, looks, lists };
}

const PASTE_LABELS = { keep: "Keep Source Formatting", theme: "Use Destination Theme", text: "Keep Text Only" };

/// Paste text from another application: formatted from its HTML unless `mode` is
/// `text` or there is none.
async function pasteForeign(html, plain, mode) {
  const rich = html && html.length <= MAX_HTML ? richFromHtml(html) : null;
  const args = rich ? richArgs(rich, mode) : null;
  lastPaste = null;
  if (text.editing()) {
    const range = text.openRange();
    if (args) { if (!(await text.pasteRich(args))) return; }
    else if (plain) { text.pasteText(plain); return; }
    else return;
    // Paste Special redoes this paste only while it is the box's last edit.
    lastPaste = { html, text: plain, mode, range, count: text.editCount() };
  } else if (args || plain) {
    await pasteTextBox(plain || "", args);
  } else return;
  if (args && mode !== "keep") status(`Pasted: ${PASTE_LABELS[mode]}`);
}

let pendingMode = null;   // Paste Special chosen with nothing to redo: the next paste's mode

/// Paste Special: paste the last paste again as `mode`, in its place, as PowerPoint's
/// Paste Options do; with no paste to redo, the next paste is made as `mode`.
async function pasteAs(mode) {
  const last = lastPaste;
  const range = text.openRange();
  if (last && range && range[0] === last.range[0] && range[1] === last.range[1] && text.editCount() === last.count) {
    if (last.mode === mode) { status(`Pasted: ${PASTE_LABELS[mode]}`); return; }
    await selection.undoStep();
    await text.setRange(last.range[2], last.range[3]);
    await pasteForeign(last.html, last.text, mode);
    status(`Pasted: ${PASTE_LABELS[mode]}`);
    return;
  }
  pendingMode = mode;
  status(`${PASTE_LABELS[mode]}: press Ctrl+V to paste`);
}

/// Paste: `osText` is the system clipboard's text, or null when it could not be read;
/// `html` its HTML, and `image` a picture file on it.
async function paste(osText, html = "", image = null) {
  const mode = plainNext ? "text" : pendingMode || "keep";
  plainNext = false;
  pendingMode = null;
  const fromUs = osText == null || osText === "" || osText === lastClipText;
  if (text.editing()) {
    if (fromUs && !html) { if (osText) text.pasteText(osText); return; }
    await pasteForeign(html, osText, mode);
    return;
  }
  if (clipKind === "slides" && heldSlides && fromUs && !image) { await pasteSlides(); return; }
  if (held && fromUs && !image) { await pasteShapes(); return; }
  if (image && !(html && richFromHtml(html))) { await pastePicture(image); return; }
  if (osText || html) { await pasteForeign(html, osText, mode); return; }
  status("The clipboard is empty");
}

async function duplicate() {
  const nodes = selection.selectedNodes();
  if (!nodes.length) return;
  const index = state.current;
  try {
    const r = await call("duplicate_shapes", [index, nodes]);
    selection.noteUndoState(r.undo);
    selection.selectNodes(r.value);
    await slides.redrawSlide(index);
    status("Duplicated");
  } catch (e) {
    errorDialog("OpenPresent cannot duplicate that yet.", e.message);
  }
}

/// The ribbon's Copy and Cut: the system clipboard is written through the async
/// API, which needs no permission inside a click.
async function viaRibbon(fn) {
  const t = await fn();
  if (t == null) return;
  try { await navigator.clipboard.writeText(t); } catch { status("Copied here; the system clipboard was refused, so other applications will not see it"); }
  changed();
}

/// The picture a paste carries from another application, if any.
function imageOf(data) {
  for (const f of data?.files || []) if (/^image\/(png|jpeg)$/.test(f.type)) return f;
  return null;
}

export function init() {
  const canCopy = () => !!state.deck && (text.editing() || selection.hasSelection() || slidesFocused());
  register("edit.copy", { label: "Copy", shortcut: "Ctrl+C", enabled: canCopy, run: () => viaRibbon(copy) });
  register("edit.cut", { label: "Cut", shortcut: "Ctrl+X", enabled: canCopy, run: () => viaRibbon(cut) });
  const canPaste = () => !!state.deck && (held > 0 || heldSlides > 0 || text.editing());
  register("edit.paste", { label: "Paste", shortcut: "Ctrl+V", enabled: canPaste, run: () => paste(null) });
  register("edit.duplicate", { label: "Duplicate", shortcut: "Ctrl+D", enabled: () => !!state.deck && selection.hasSelection() && !text.editing(), run: duplicate });
  // Paste Special (`PAR-11`).
  const canSpecial = () => !!state.deck;
  register("edit.paste-keep-source", { label: "Keep Source Formatting", enabled: canSpecial, run: () => pasteAs("keep") });
  register("edit.paste-dest-theme", { label: "Use Destination Theme", enabled: canSpecial, run: () => pasteAs("theme") });
  register("edit.paste-text", { label: "Keep Text Only", shortcut: "Ctrl+Shift+V", enabled: canSpecial, run: () => pasteAs("text") });

  // The keys: the browser's own events, which carry the system clipboard.
  document.addEventListener("copy", (e) => {
    if (!ours(e.target)) return;
    if (!text.editing() && !selection.hasSelection() && !slidesFocused()) return;
    e.preventDefault();
    // The event's clipboard must be written now; the engine answers later, so
    // the stand-in follows through the async API.
    copy().then((t) => { if (t != null) navigator.clipboard.writeText(t).catch(() => {}); changed(); });
  });
  document.addEventListener("cut", (e) => {
    if (!ours(e.target)) return;
    if (!text.editing() && !selection.hasSelection() && !slidesFocused()) return;
    e.preventDefault();
    cut().then((t) => { if (t != null) navigator.clipboard.writeText(t).catch(() => {}); changed(); });
  });
  document.addEventListener("paste", (e) => {
    if (!ours(e.target)) return;
    e.preventDefault();
    e.stopImmediatePropagation();
    const d = e.clipboardData;
    paste(d ? d.getData("text/plain") : null, d ? d.getData("text/html") : "", imageOf(d));
  }, true);
  // Ctrl+Shift+V: the browser fires a paste; it pastes text only, as `PAR-11` asks.
  document.addEventListener("keydown", (e) => {
    const mod = (e.ctrlKey || e.metaKey) && !e.altKey;
    if (mod && e.shiftKey && (e.key === "v" || e.key === "V")) plainNext = true;
  }, true);

  // The thumbnail pane's selection (`PAR-10`): a press there makes Copy and Paste
  // act on slides; Ctrl+click adds or removes a slide, Shift+click a range.
  document.addEventListener("pointerdown", (e) => {
    if (!e.target?.closest) return;
    // A press in the pane closes the open text box, as PowerPoint's.
    if (e.target.closest("#slide-pane")) { paneActive = true; if (text.editing()) text.stop(); }
    else if (e.target.closest("#stage")) paneActive = false;
  }, true);
  const list = byId("thumbs");
  list.addEventListener("click", (e) => {
    const li = e.target.closest?.(".thumb");
    if (!li) return;
    const i = Number(li.dataset.index);
    if (e.ctrlKey || e.metaKey) {
      e.stopPropagation();
      if (i === state.current) return;
      if (extraSlides.has(i)) extraSlides.delete(i); else extraSlides.add(i);
      markExtra();
    } else if (e.shiftKey) {
      e.stopPropagation();
      extraSlides.clear();
      const [a, b] = [Math.min(i, state.current), Math.max(i, state.current)];
      for (let k = a; k <= b; k++) extraSlides.add(k);
      markExtra();
    } else {
      extraSlides.clear();
      queueMicrotask(markExtra);
    }
  }, true);
  document.addEventListener("op-slides-built", () => { extraSlides.clear(); });
}
