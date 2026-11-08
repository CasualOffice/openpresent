// The notes pane under the slide, editable (`UX-011`): click into it, type, delete,
// select, Bold, Italic and Underline, Undo and Redo, as PowerPoint's.
//
// The engine keeps the notes text, in the slide's notes slide, and every keystroke is
// one of the text operations every shape uses (`text_insert`, `text_delete`,
// `text_toggle`), on the notes body placeholder, so it is undone with everything else
// and a save patches the notes slide. A slide with no notes slide gets one on the
// first keystroke, in the same undo step's neighbour (`notes_body` with `create`).
//
// The pane is a `contenteditable` that never edits itself: every `beforeinput` is
// cancelled and turned into an engine call, and the paragraphs and runs are drawn again
// from what the engine then holds, with `textContent` only, so no document string is
// parsed as HTML. The caret is the browser's, put back at the offset the edit left.
// Composition (IME) is let through and committed when it ends.

import { state, request, status } from "./editor.core.js";
import { run } from "./commands.js";
import * as selection from "./editor.selection.js";

const byId = (id) => document.getElementById(id);
const PRIORITY = 8;
const len = (s) => Array.from(s).length;

let box = null;
let target = null;      // [part, node] of the notes body, or null
let forIndex = -1;
let lastJson = "";
let token = 0;
let chain = Promise.resolve();
let composeAt = null;
let busy = 0;           // edits queued and not yet drawn
let logical = [0, 0];   // where the caret will be once they are

async function call(op, fn, args) {
  const r = await request({ op, fn, args }, { priority: PRIORITY });
  if (r.undo) selection.noteUndoState(r.undo);
  return r.value;
}

/* ── Drawing ────────────────────────────────────────────────────────────── */

function render(paragraphs) {
  box.textContent = "";
  let at = 0;
  for (const runs of paragraphs) {
    const p = document.createElement("div");
    p.className = "np-para";
    p.dataset.start = String(at);
    for (const [text, bits] of runs) {
      const s = document.createElement("span");
      s.dataset.start = String(at);
      s.dataset.len = String(len(text));
      if (bits & 1) s.classList.add("np-b");
      if (bits & 2) s.classList.add("np-i");
      if (bits & 4) s.classList.add("np-u");
      // A line break (U+000B) shows as one; it counts one character, as in the engine.
      s.textContent = text.replaceAll("\u000b", "\n");
      if (text) p.appendChild(s);
      at += len(text);
    }
    if (!p.childNodes.length) p.appendChild(document.createElement("br"));
    p.dataset.end = String(at);
    box.appendChild(p);
    at += 1;   // the paragraph break
  }
  box.classList.toggle("is-empty", paragraphs.length === 0 || (paragraphs.length === 1 && paragraphs[0].every(([t]) => !t)));
}

/// Read the notes of the current slide again. With `keep`, the caret stays where it was.
export async function draw(keep = false) {
  if (!box) return;
  const t = ++token;
  const index = state.current;
  if (!state.deck) { box.textContent = ""; return; }
  try {
    const body = await call("review", "notes_body", [index, false]);
    if (t !== token) return;
    const caret = keep && forIndex === index && document.activeElement === box ? offsets() : null;
    forIndex = index;
    target = body.length === 2 ? body : null;
    if (!target) { lastJson = ""; render([]); return; }
    const json = await call("review", "text_body_runs", target);
    if (t !== token) return;
    if (json === lastJson && box.childNodes.length) return;
    lastJson = json;
    render(JSON.parse(json));
    if (caret) place(caret[0], caret[1]);
  } catch {
    if (t === token) { target = null; render([]); }
  }
}

/* ── Offsets ────────────────────────────────────────────────────────────── */

/// The engine offset of a DOM position inside the pane.
function offsetOf(node, off) {
  if (node === box) {
    const p = box.childNodes[Math.min(off, box.childNodes.length - 1)];
    if (!p) return 0;
    return off >= box.childNodes.length ? Number(p.dataset.end) : Number(p.dataset.start);
  }
  const span = node.nodeType === Node.TEXT_NODE ? node.parentElement : node;
  if (span?.dataset?.len != null && node.nodeType === Node.TEXT_NODE) {
    return Number(span.dataset.start) + len(node.data.slice(0, off));
  }
  const para = (span?.closest && span.closest(".np-para")) || null;
  if (!para) return 0;
  if (span === para) {
    let at = Number(para.dataset.start);
    for (let i = 0; i < off && i < para.childNodes.length; i++) at += Number(para.childNodes[i].dataset?.len || 0);
    return at;
  }
  // Inside a span but not its text: its start, or its end past the text.
  return Number(span.dataset.start) + (off > 0 ? Number(span.dataset.len) : 0);
}

/// The selection as `[anchor, focus]` engine offsets.
function offsets() {
  const sel = getSelection();
  if (!sel || !sel.rangeCount || !box.contains(sel.anchorNode)) return [0, 0];
  return [offsetOf(sel.anchorNode, sel.anchorOffset), offsetOf(sel.focusNode, sel.focusOffset)];
}

function domAt(o) {
  const paras = [...box.querySelectorAll(".np-para")];
  const p = paras.find((x) => o <= Number(x.dataset.end)) || paras[paras.length - 1];
  if (!p) return [box, 0];
  for (const s of p.querySelectorAll("span")) {
    const start = Number(s.dataset.start), n = Number(s.dataset.len);
    if (o <= start + n && s.firstChild) {
      const chars = Array.from(s.firstChild.data);
      return [s.firstChild, chars.slice(0, Math.max(0, o - start)).join("").length];
    }
  }
  return [p, p.childNodes.length && p.firstChild.nodeName === "BR" ? 0 : p.childNodes.length];
}

function place(anchor, focus = anchor) {
  const sel = getSelection();
  const [an, ao] = domAt(anchor);
  const [fn, fo] = domAt(focus);
  try { sel.setBaseAndExtent(an, ao, fn, fo); } catch { /* the pane was drawn again meanwhile */ }
}

/* ── Editing ────────────────────────────────────────────────────────────── */

/// Run edits one after another, in the order they were typed.
function queue(fn) {
  busy++;
  chain = chain.then(fn).catch((e) => status(`The notes could not be edited: ${e.message}`)).finally(() => { busy--; });
  return chain;
}

/// The selection as engine offsets: the DOM's when everything typed is drawn, or where
/// the queued edits will leave it while keys arrive faster than the engine answers.
function current() { return busy ? logical : offsets(); }

async function ensureTarget() {
  if (target) return target;
  const body = await call("review", "notes_body", [state.current, true]);
  target = body.length === 2 ? body : null;
  if (!target) throw new Error("this slide's notes cannot be edited");
  return target;
}

async function redrawAt(caret) {
  if (busy > 1) return;   // more is queued: draw once, after the last
  lastJson = "";
  const json = await call("review", "text_body_runs", target);
  lastJson = json;
  render(JSON.parse(json));
  place(caret);
}

function replace(text, [a, b], typing) {
  logical = [a + len(text), a + len(text)];
  return queue(async () => {
    const [part, node] = await ensureTarget();
    if (b > a) await call("text", "text_delete", [part, node, a, b]);
    if (text) await call("text", "text_insert", [part, node, a, text, typing && b === a]);
    await redrawAt(a + len(text));
  });
}

function remove(a, b) {
  if (b <= a || !target) return;
  logical = [Math.max(0, a), Math.max(0, a)];
  queue(async () => {
    const [part, node] = target;
    await call("text", "text_delete", [part, node, Math.max(0, a), b]);
    await redrawAt(Math.max(0, a));
  });
}

function toggle(which) {
  const [x, y] = current();
  const [a, b] = [Math.min(x, y), Math.max(x, y)];
  if (b <= a || !target) { status("Select some notes text first"); return; }
  queue(async () => {
    const [part, node] = target;
    await call("text", "text_toggle", [part, node, a, b, which]);
    lastJson = "";
    const json = await call("review", "text_body_runs", target);
    lastJson = json;
    render(JSON.parse(json));
    place(x, y);
  });
}

function onBeforeInput(e) {
  if (!state.deck || state.reading) { e.preventDefault(); return; }
  if (e.inputType === "insertCompositionText") return;   // committed at compositionend
  e.preventDefault();
  const [x, y] = current();
  const range = [Math.min(x, y), Math.max(x, y)];
  const [a, b] = range;
  switch (e.inputType) {
    case "insertText":
    case "insertReplacementText":
      replace(e.data ?? e.dataTransfer?.getData("text/plain") ?? "", range, true);
      break;
    case "insertParagraph": replace("\n", range, false); break;
    case "insertLineBreak": replace("\u000b", range, false); break;
    case "insertFromPaste":
    case "insertFromDrop": {
      const text = (e.dataTransfer?.getData("text/plain") || "").replace(/\r\n?/g, "\n");
      if (text) replace(text, range, false);
      break;
    }
    case "deleteContentBackward":
    case "deleteWordBackward":
    case "deleteSoftLineBackward":
    case "deleteHardLineBackward":
      if (b > a) remove(a, b); else remove(a - 1, a);
      break;
    case "deleteContentForward":
    case "deleteWordForward":
    case "deleteSoftLineForward":
    case "deleteHardLineForward":
      if (b > a) remove(a, b); else remove(a, a + 1);
      break;
    case "deleteByCut":
    case "deleteByDrag":
    case "deleteContent":
      remove(a, b);
      break;
    case "formatBold": toggle(0); break;
    case "formatItalic": toggle(1); break;
    case "formatUnderline": toggle(2); break;
    case "historyUndo": run("edit.undo"); break;
    case "historyRedo": run("edit.redo"); break;
    default: break;
  }
}

export function init() {
  const old = byId("notes-text");
  box = document.createElement("div");
  box.id = "notes-text";
  box.className = "notes-text notes-edit";
  box.contentEditable = "true";
  box.spellcheck = true;
  box.setAttribute("role", "textbox");
  box.setAttribute("aria-multiline", "true");
  box.setAttribute("aria-label", "Notes");
  box.dataset.placeholder = "Click to add notes";
  old.replaceWith(box);
  box.addEventListener("beforeinput", onBeforeInput);
  box.addEventListener("compositionstart", () => { composeAt = current(); });
  box.addEventListener("compositionend", (e) => {
    const at = composeAt;
    composeAt = null;
    if (!at) return;
    const range = [Math.min(...at), Math.max(...at)];
    if (e.data) replace(e.data, range, false); else draw();
  });
  box.addEventListener("keydown", (e) => {
    // The slide's own keys (Delete a shape, arrows to nudge) stay out of the notes.
    const mod = e.ctrlKey || e.metaKey;
    if (e.key === "Escape") { e.preventDefault(); byId("stage")?.focus({ preventScroll: true }); return; }
    if (e.key === "F6" || (e.key === "Tab" && mod)) return;
    if (mod && !e.altKey && (e.key === "z" || e.key === "Z")) { e.preventDefault(); run(e.shiftKey ? "edit.redo" : "edit.undo"); }
    else if (mod && !e.altKey && (e.key === "y" || e.key === "Y")) { e.preventDefault(); run("edit.redo"); }
    else if (mod && !e.altKey && ["b", "i", "u"].includes(e.key.toLowerCase())) { e.preventDefault(); toggle({ b: 0, i: 1, u: 2 }[e.key.toLowerCase()]); }
    e.stopPropagation();
  });
  box.addEventListener("focus", () => { if (state.deck && !target) draw(); });
  // An undo or redo, another slide: read the notes again; the same text draws nothing.
  let pending = 0;
  document.addEventListener("op-changed", () => {
    clearTimeout(pending);
    pending = setTimeout(() => { if (!busy) chain = chain.then(() => draw(true)).catch(() => {}); }, 80);
  });
}
