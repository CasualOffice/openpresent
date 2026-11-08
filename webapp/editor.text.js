// Text editing on the stage (`VS-05`): click into a text box, see a caret, type,
// delete, select, Bold, Italic and Underline, undo and redo.
//
// The engine keeps the text and lays it out; this module keeps only where the
// caret is and draws it. As PowerPoint, one click inside a shape that holds text
// puts the caret there; `editor.selection.js` (`VS-06`) routes the press, since it
// knows the shapes, and a press on a border selects and drags the shape instead.
// A click becomes a hit test in the engine, the caret and the selection come back
// as slide coordinates and are drawn in an SVG over the slide picture, and
// keystrokes go to a hidden textarea so the browser's own input methods work.
// Every edit is one engine call, applied to the deck's model through
// `casual-slide-transaction`, so it can be undone.
// No document string reaches the DOM: the textarea is emptied after each input.
//
// Fluid typing: keys typed while the engine is busy are joined into one insert,
// the stage is redrawn once per insert and the caret drawn with the new picture,
// and the thumbnail and the shape list wait until typing pauses.
//
// Shaped after OpenCalc's in-cell editor (`webapp/editor.edit.js` at `b23c67c`):
// a hidden input owns focus and IME, and the canvas only draws.

import { state, request, changed, status } from "./editor.core.js";
import { register, run } from "./commands.js";
import * as slides from "./editor.slides.js";
import * as shapes from "./editor.selection.js";
import * as keys from "./editor.keys.js";

const SVG = "http://www.w3.org/2000/svg";
const byId = (id) => document.getElementById(id);

/// Text calls run in order, ahead of thumbnails, behind nothing but opening.
const TEXT_PRIORITY = 8;
/// How long typing pauses before the thumbnail and the shape list are read again.
const SETTLE_MS = 400;

let overlay = null;    // <svg> over the slide
let input = null;      // hidden <textarea>
let formatBits = 0;    // bit 1 bold, 2 italic, 4 underline, 8 strike, 16 superscript, 32 subscript
/// The pressed state of the Font group's toggles at the caret (`PAR-20`).
export const caretFormatBits = () => (selection.editing ? formatBits : 0);
let runState = null;   // [font, size, align, bulleted] at the caret (`UX-003`)
let composing = false;
let dragging = false;
let blink = 0;
let settleTimer = 0;

/// The open text box: the shape as `[part, node]` and the caret's anchor and focus,
/// character offsets in its body (`docs/24`). No document text.
const selection = { object: null, editing: false, anchor: 0, focus: 0 };
const select = (patch) => Object.assign(selection, patch);
const clearSelection = () => select({ object: null, editing: false, anchor: 0, focus: 0 });
const textRange = () => [Math.min(selection.anchor, selection.focus), Math.max(selection.anchor, selection.focus)];

/// Whether a text box is open, for the shape selection to stand aside.
export const editing = () => selection.editing;

/// Where the caret is, for presence (`COL-04`): the shape and both ends, or null.
export function caretState() {
  if (!selection.editing || !obj()) return null;
  const [part, node] = obj();
  return { part, node, anchor: selection.anchor, focus: selection.focus };
}

/// Move the caret's ends after another participant's edit moved the text under
/// them (`COL-04`), then draw it again.
export function remap(anchor, focus) {
  if (!selection.editing) return;
  select({ anchor, focus });
  refresh();
}

async function call(fn, ...args) {
  const r = await request({ op: "text", fn, args }, { priority: TEXT_PRIORITY });
  if (r.undo && EDITS.has(fn)) shapes.noteUndoState(r.undo);
  return r.value;
}

const obj = () => selection.object;
const EDITS = new Set(["text_insert", "text_delete", "text_toggle", "text_set_run", "text_grow", "text_set_paragraphs", "text_paste_rich", "text_change_case"]);

/* ── Drawing the caret and selection ────────────────────────────────────── */

/// The overlay's coordinates are the slide's, in EMU. Set on every draw: a deck of
/// another size in a frame of the same shape (a 10-inch 16:9 deck after the 13.33-inch
/// blank one) resizes nothing, so an observer alone left the old size (`UX-013`).
function fitOverlay() {
  const box = state.deck && `0 0 ${state.deck.width} ${state.deck.height}`;
  if (overlay && box && overlay.getAttribute("viewBox") !== box) overlay.setAttribute("viewBox", box);
}

function clearOverlay() {
  if (overlay) overlay.replaceChildren();
}

/// Where the caret and the selection are, from the engine: numbers only.
async function geometry() {
  if (!selection.editing || !obj() || !state.deck) return null;
  const [part, node] = obj();
  const index = state.current;
  const [start, end] = textRange();
  const quads = start !== end ? await call("text_selection", index, part, node, start, end) : [];
  const caret = await call("text_caret", index, part, node, selection.focus);
  formatBits = await call("text_format_state", part, node, selection.focus);
  runState = await call("text_run_state", part, node, selection.focus).catch(() => null);
  return { quads, caret };
}

/// Draw what `geometry` found: the selection, then a blinking caret.
function draw(g) {
  clearOverlay();
  clearInterval(blink);
  if (!g || !selection.editing) return;
  fitOverlay();
  const { quads, caret } = g;
  for (let i = 0; i + 8 <= quads.length; i += 8) {
    const poly = document.createElementNS(SVG, "polygon");
    poly.setAttribute("points", quads.slice(i, i + 8).join(" "));
    poly.setAttribute("class", "text-sel");
    overlay.appendChild(poly);
  }
  const [x1, y1, x2, y2] = caret;
  const line = document.createElementNS(SVG, "line");
  for (const [k, v] of [["x1", x1], ["y1", y1], ["x2", x2], ["y2", y2]]) line.setAttribute(k, String(v));
  line.setAttribute("class", "text-caret");
  line.setAttribute("vector-effect", "non-scaling-stroke");
  overlay.appendChild(line);
  // The textarea follows the caret, so an IME's window opens beside it.
  const r = byId("slide-frame").getBoundingClientRect();
  input.style.left = `${(x1 / state.deck.width) * r.width}px`;
  input.style.top = `${(y1 / state.deck.height) * r.height}px`;
  let on = true;
  blink = setInterval(() => { on = !on; line.style.visibility = on ? "visible" : "hidden"; }, 530);
  // A ribbon button took focus: the keys come back to the text.
  if (document.activeElement !== input) input.focus({ preventScroll: true });
}

async function paint() {
  const deck = state.deck, at = obj();
  try {
    draw(await geometry());
  } catch (e) {
    // A deck opened, or the box closed, while the engine answered: the question was
    // about a shape no longer open, so there is nothing to report (`UX-013`).
    if (state.deck === deck && obj() === at) status(`The caret could not be placed: ${e.message}`);
  }
  changed();
}

/// After an edit: redraw the slide, and draw the caret with the new picture. The
/// thumbnail and the shape list follow once typing pauses.
let edits = 0;   // edits made through this module
/// How many edits this module has made: Paste Special redoes a paste only while it is
/// the last one (`PAR-11`).
export const editCount = () => edits;

async function afterEdit() {
  edits++;
  const [part, node] = obj();
  const [g] = await Promise.all([geometry().catch(() => null), slides.redraw({ quiet: true })]);
  draw(g);
  changed();
  const length = await call("text_length", part, node).catch(() => 1);
  shapes.noteEmpty(node, length === 0);
  clearTimeout(settleTimer);
  const index = state.current;
  settleTimer = setTimeout(() => {
    slides.refreshThumb(index);
    if (index === state.current) shapes.refresh();
  }, SETTLE_MS);
}

/* ── Entering and leaving a text box ────────────────────────────────────── */

/// Close the open text box, if any. The shape stays selected, as PowerPoint's.
let opens = 0;   // moves on with each open and close, so a late open is dropped

export function stop() {
  opens++;
  if (!selection.editing) return;
  clearSelection();
  call("edit_break_typing").catch(() => {});
  clearOverlay();
  clearInterval(blink);
  input.blur();
  shapes.repaint();
  changed();
}

/// A press inside text shape `[part, node]` at slide point `p`, routed here by the
/// shape selection: open the box with the caret under the pointer, or move the
/// caret in the open one; `extend` keeps the anchor, as Shift does.
export function pointerDown(part, node, p, extend) {
  const same = selection.editing && obj()[0] === part && obj()[1] === node;
  const clicks = countClick(part, node, p);
  // Focus now, inside the press, so the keys that follow are not lost.
  input.focus({ preventScroll: true });
  dragging = true;
  const ticket = ++opens;
  opening++;
  serial(async () => {
    const at = await call("text_offset_near", state.current, part, node, p[0], p[1]);
    if (ticket !== opens) return;
    if (!same) {
      await call("edit_break_typing");
      select({ object: [part, node], editing: true, anchor: at, focus: at });
      shapes.repaint();
    } else {
      select(extend ? { focus: at } : { anchor: at, focus: at });
    }
    // A double click selects the word, a triple click the paragraph (`PAR-12`).
    if (clicks > 1 && !extend) {
      const t = await plain();
      const [a, b] = clicks === 2 ? wordAt(t, at) : paragraphAt(t, at);
      select({ anchor: a, focus: b });
      dragging = false;
    }
    await paint();
  }).finally(() => { opening--; });
}

/* ── Words and paragraphs (`PAR-12`) ────────────────────────────────────── */

/// How long and how near, in milliseconds and screen pixels, a press counts as the
/// next click of a double or triple click.
const MULTI_CLICK_MS = 500;
const MULTI_CLICK_PX = 4;
let lastClick = { at: 0, count: 0, key: "", p: [0, 0] };

/// The press's place in a run of clicks on one box: 1, 2 or 3 (and back to 1).
function countClick(part, node, p) {
  const now = performance.now();
  const key = `${part}:${node}`;
  const k = byId("slide-frame").clientWidth / (state.deck?.width || 1);
  const near = Math.hypot(p[0] - lastClick.p[0], p[1] - lastClick.p[1]) * k <= MULTI_CLICK_PX;
  const count = now - lastClick.at <= MULTI_CLICK_MS && key === lastClick.key && near ? (lastClick.count % 3) + 1 : 1;
  lastClick = { at: now, count, key, p };
  return count;
}

/// The open box's text, paragraphs joined by line feeds and line breaks as vertical
/// tabs, one character an offset. Held in this module only, never put in the page.
async function plain() {
  const [part, node] = obj();
  const length = await call("text_length", part, node);
  return [...(await call("text_slice", part, node, 0, length))];
}

const segmenter = typeof Intl !== "undefined" && Intl.Segmenter ? new Intl.Segmenter(undefined, { granularity: "word" }) : null;

/// The words of `chars` as `[start, end, wordLike]` in character offsets: the
/// browser's word breaks, which split Chinese and Japanese by dictionary as
/// PowerPoint does, or runs of letters and digits where there are none.
function words(chars) {
  const out = [];
  if (segmenter) {
    let at = 0;
    for (const seg of segmenter.segment(chars.join(""))) {
      const n = [...seg.segment].length;
      out.push([at, at + n, !!seg.isWordLike]);
      at += n;
    }
    return out;
  }
  const kind = (c) => (/[\p{L}\p{N}_]/u.test(c) ? 1 : /\s/.test(c) ? 0 : 2);
  for (let i = 0; i < chars.length;) {
    let j = i + 1;
    while (j < chars.length && kind(chars[j]) === kind(chars[i]) && kind(chars[i]) !== 2) j++;
    out.push([i, j, kind(chars[i]) === 1]);
    i = j;
  }
  return out;
}

const BREAKS = new Set(["\n", "\v"]);

/// What a double click at `at` selects: the word there and the spaces after it, as
/// PowerPoint's; on a space or a mark, that run alone.
export function wordAt(chars, at) {
  const list = words(chars);
  const i = list.findIndex(([a, b]) => at >= a && at < b);
  const k = i >= 0 ? i : list.findIndex(([a, b]) => at === b);
  if (k < 0) return [at, at];
  let [a, b, word] = list[k];
  if (BREAKS.has(chars[a])) return [at, at];
  if (word) while (b < chars.length && /[ \t\u00a0\u3000]/.test(chars[b])) b++;
  return [a, b];
}

/// What a triple click at `at` selects: its paragraph, without the break that ends it.
export function paragraphAt(chars, at) {
  let a = Math.min(at, chars.length), b = a;
  while (a > 0 && chars[a - 1] !== "\n") a--;
  while (b < chars.length && chars[b] !== "\n") b++;
  return [a, b];
}

/// Where Ctrl+Left (`-1`) or Ctrl+Right (`1`) takes the caret from `at`: the start of
/// the word before, or of the next, stopping at each paragraph's ends, as PowerPoint.
export function wordStep(chars, at, direction) {
  const stops = new Set([0, chars.length]);
  for (const [a, , word] of words(chars)) if (word) stops.add(a);
  chars.forEach((c, i) => { if (c === "\n") { stops.add(i); stops.add(i + 1); } });
  const sorted = [...stops].sort((x, y) => x - y);
  if (direction < 0) return sorted.filter((x) => x < at).pop() ?? 0;
  return sorted.find((x) => x > at) ?? chars.length;
}

/// Where Ctrl+Up (`-1`) or Ctrl+Down (`1`) takes the caret: the start of this
/// paragraph or the one before, or the start of the next.
export function paragraphStep(chars, at, direction) {
  const [a, b] = paragraphAt(chars, at);
  if (direction < 0) return at > a ? a : paragraphAt(chars, Math.max(0, a - 1))[0];
  return b < chars.length ? b + 1 : chars.length;
}

/// Move by word or paragraph, or to either end of the box; `extend` keeps the anchor.
function jump(how, extend) {
  return serial(async () => {
    if (!selection.editing) return;
    const chars = await plain();
    const at = selection.focus;
    const focus = how === "start" ? 0 : how === "end" ? chars.length
      : how === "word-left" ? wordStep(chars, at, -1) : how === "word-right" ? wordStep(chars, at, 1)
      : how === "para-up" ? paragraphStep(chars, at, -1) : paragraphStep(chars, at, 1);
    await call("edit_break_typing");
    select(extend ? { focus } : { anchor: focus, focus });
    await paint();
  });
}

/// Tab: a list level down at a paragraph's start or over a selection, else a tab.
function tab() {
  return serial(async () => {
    if (!selection.editing) return;
    const [start, end] = textRange();
    let level = start !== end;
    if (!level) {
      const chars = await plain();
      level = start === 0 || chars[start - 1] === "\n";
    }
    if (level) await levelNow("up"); else await insertNow("\t", true);
  });
}

/// Shift+Tab, and Alt+Shift+Left and Right: a list level up or down, in turn with
/// the keys before it.
function level(way) {
  return serial(() => levelNow(way));
}

async function levelNow(way) {
  if (!selection.editing) return;
  const [part, node] = obj();
  const [start, end] = textRange();
  if (await call("text_set_paragraphs", part, node, start, end, 3, way)) await afterEdit();
  status(way === "up" ? "Increase List Level" : "Decrease List Level");
}

/// Ctrl+Backspace and Ctrl+Delete: delete to the start of the word, or to the next.
function removeWord(direction) {
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    let [start, end] = textRange();
    if (start === end) {
      const chars = await plain();
      if (direction < 0) start = wordStep(chars, start, -1); else end = wordStep(chars, end, 1);
      if (start === end) return;
    }
    await call("text_delete", part, node, start, end);
    select({ anchor: start, focus: start });
    await afterEdit();
  });
}

let moveQueued = null;
/// The pointer moved with the button down in a text box: extend the selection.
export function pointerMove(p) {
  if (!dragging || !selection.editing) return;
  // Only the latest point matters; the engine is asked once per answer.
  const pending = moveQueued;
  moveQueued = p;
  if (pending) return;
  serial(async () => {
    const q = moveQueued;
    moveQueued = null;
    if (!q || !selection.editing) return;
    const [part, node] = obj();
    const focus = await call("text_offset_near", state.current, part, node, q[0], q[1]);
    if (focus !== selection.focus) { select({ focus }); await paint(); }
  });
}

export function pointerUp() { dragging = false; }

/// A key typed while a text shape is selected but not open: open it at the end
/// of its text and type the key there, as PowerPoint does.
export function typeInto(part, node, key) {
  input.focus({ preventScroll: true });
  const ticket = ++opens;
  opening++;
  serial(async () => {
    const length = await call("text_length", part, node);
    if (ticket !== opens) return;
    await call("edit_break_typing");
    select({ object: [part, node], editing: true, anchor: length, focus: length });
    shapes.repaint();
  }).finally(() => { opening--; });
  insert(key);
}

/// What was typed after a press that is waiting for the slide's shapes to be read,
/// as actions to run once the box is open; null when none is waiting.
let held = null;
/// Opens begun and not yet finished: keys typed meanwhile run after them.
let opening = 0;

/// A press is waiting for the shapes (`VS-19`): take focus now and keep the keys
/// typed meanwhile, so none is lost.
export function hold() {
  held = held ?? [];
  input.focus({ preventScroll: true });
}

/// The waiting press was replayed: run the held keys in the box it opened, or drop
/// them if it opened none.
export function release(opened) {
  const keys = held || [];
  held = null;
  if (opened) for (const k of keys) k();
  else if (document.activeElement === input) byId("stage").focus({ preventScroll: true });
}

/// Run `action` for a key now, or after the waiting press is replayed.
function act(action) {
  if (held !== null) held.push(action); else action();
}

/* ── Edits ──────────────────────────────────────────────────────────────── */

let queue = Promise.resolve();
let tail = null;   // the typing job last queued, while nothing has been queued after it
/// Edits run one after another, each seeing the caret the last one left.
function serial(fn) {
  tail = null;
  queue = queue.then(fn).catch((e) => status(`The edit could not be made: ${e.message}`));
  return queue;
}

/// Replace the selection with `text`; `typing` joins keystrokes in one undo step.
/// Keys typed while an edit is in flight join the typing queued last, so the stage
/// is drawn once for them all; a key queued after Enter, Tab or any other edit
/// starts its own, so nothing typed overtakes what came before it.
function insert(text, typing = true) {
  if (typing) {
    if (tail && !tail.started) { tail.text += text; return queue; }
    const job = { text, started: false };
    serial(async () => {
      job.started = true;
      await insertNow(job.text, true);
    });
    tail = job;
    return queue;
  }
  return serial(() => insertNow(text, false));
}

async function insertNow(text, typing) {
  if (!selection.editing || !text) return;
  const [part, node] = obj();
  let [start, end] = textRange();
  if (start !== end) {
    await call("text_delete", part, node, start, end);
    typing = false;
  }
  await call("text_insert", part, node, start, text, typing);
  // The caret moves past what was inserted; dropped control characters are
  // found by the length, which is the engine's.
  const length = await call("text_length", part, node);
  const at = Math.min(length, start + [...text].length);
  select({ anchor: at, focus: at });
  await afterEdit();
}

function remove(direction) {
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    let [start, end] = textRange();
    if (start === end) {
      const length = await call("text_length", part, node);
      if (direction < 0) start = Math.max(0, start - 1); else end = Math.min(length, end + 1);
      if (start === end) return;
    }
    await call("text_delete", part, node, start, end);
    select({ anchor: start, focus: start });
    await afterEdit();
  });
}

function toggle(which) {
  // From the ribbon, the button took focus: the next key is the text's again.
  if (selection.editing) input.focus({ preventScroll: true });
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    const [start, end] = textRange();
    if (start === end) { status("Select text to format it."); return; }
    if (await call("text_toggle", part, node, start, end, which)) await afterEdit();
  });
}

/* ── The Home tab's Font and Paragraph groups, and the clipboard (`UX-003`) ── */

/// What the Font and Size boxes, alignment and Bullets show at the caret, or null.
export const caretRunState = () => (selection.editing ? runState : null);

/// Run one formatting call over the open box's selection, `fn(part, node, start,
/// end)` returning whether it changed anything, and redraw. `whole`: the caret alone
/// also counts (alignment and bullets act on its paragraph).
export function formatSelection(fn, whole = false) {
  if (selection.editing) input.focus({ preventScroll: true });
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    const [start, end] = textRange();
    if (start === end && !whole) { status("Select text to format it."); return; }
    if (await fn(part, node, start, end)) await afterEdit();
  });
}

/// The selected text, for Copy; empty when nothing is selected.
export async function selectedText() {
  // After the keys before it: Ctrl+A then Ctrl+C copies what Ctrl+A selected.
  await queue.catch(() => {});
  if (!selection.editing) return "";
  const [start, end] = textRange();
  if (start === end) return "";
  const [part, node] = obj();
  return (await call("text_slice", part, node, start, end)).replace(/\v/g, "\n");
}

/// Delete the selection, for Cut.
export function cutSelection() {
  const [start, end] = textRange();
  if (start !== end) remove(-1);
}

/// Paste plain text at the caret, replacing the selection.
export function pasteText(text) {
  if (selection.editing && text) act(() => insert(text.replace(/\r\n?/g, "\n"), false));
}

/// Paste formatted text (`PAR-11`) at the caret, replacing the selection, as one undo
/// step: `rich` is `{ texts, looks, lists }` as the engine's `text_paste_rich` takes
/// them. Resolves to whether it pasted.
export function pasteRich(rich) {
  if (!selection.editing) return Promise.resolve(false);
  let done = false;
  act(() => serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    const [start, end] = textRange();
    await call("edit_break_typing");
    const at = await call("text_paste_rich", part, node, start, end, rich.texts, rich.looks, rich.lists);
    select({ anchor: at, focus: at });
    done = true;
    await afterEdit();
  }));
  return queue.then(() => done);
}

/// Select `[anchor, focus]` in the open box: Paste Special puts back the selection a
/// paste replaced.
export function setRange(anchor, focus) {
  return serial(async () => {
    if (!selection.editing) return;
    select({ anchor, focus });
    await paint();
  });
}

/// The open box's selection as `[part, node, start, end]`, or null.
export function openRange() {
  if (!selection.editing || !obj()) return null;
  return [...obj(), ...textRange()];
}

function history(id) {
  call("edit_break_typing").catch(() => {});
  run(id);
}

/// The stage was drawn again, after an undo, a redo or a move: keep the caret
/// inside the body, or close the box if its shape is gone.
export function refresh() {
  if (!selection.editing) return;
  serial(async () => {
    const [part, node] = obj();
    try {
      const length = await call("text_length", part, node);
      select({ anchor: Math.min(selection.anchor, length), focus: Math.min(selection.focus, length) });
      await paint();
    } catch {
      stop();
    }
  });
}

/// Move the caret; `extend` keeps the anchor, as Shift does.
function move(how, extend) {
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    const [start, end] = textRange();
    let focus;
    if (how === "left" || how === "right") {
      const length = await call("text_length", part, node);
      if (!extend && start !== end) focus = how === "left" ? start : end;
      else focus = Math.max(0, Math.min(length, selection.focus + (how === "left" ? -1 : 1)));
    } else {
      const code = { home: 0, end: 1, up: 2, down: 3 }[how];
      focus = await call("text_navigate", state.current, part, node, selection.focus, code);
    }
    await call("edit_break_typing");
    select(extend ? { focus } : { anchor: focus, focus });
    await paint();
  });
}

function selectAll() {
  return serial(async () => {
    if (!selection.editing) return;
    const [part, node] = obj();
    select({ anchor: 0, focus: await call("text_length", part, node) });
    await paint();
  });
}

/* ── Keys ───────────────────────────────────────────────────────────────── */

function onKeyDown(e) {
  // A box being opened takes its keys too: each action runs after the open, in order.
  if (!(selection.editing || opening > 0 || held !== null) || composing || e.isComposing) return;
  if (held !== null) {
    const copy = { key: e.key, shiftKey: e.shiftKey, ctrlKey: e.ctrlKey, metaKey: e.metaKey, altKey: e.altKey };
    if (keyAction(copy, true)) { e.preventDefault(); e.stopPropagation(); act(() => keyAction(copy, false)); }
    return;
  }
  if (keyAction(e, false)) { e.preventDefault(); e.stopPropagation(); return; }
  // PowerPoint's other shortcuts, ahead of the browser (`PAR-13`).
  keys.handle(e);
}

/// The editing action of key `e`, run unless `probe`; whether the key has one.
function keyAction(e, probe) {
  const mod = (e.ctrlKey || e.metaKey) && !e.altKey;
  const key = e.key.length === 1 ? e.key.toLowerCase() : e.key;
  let f = null;
  if (mod && key === "b") f = () => toggle(0);
  else if (mod && key === "i") f = () => toggle(1);
  else if (mod && key === "u") f = () => toggle(2);
  else if (mod && key === "z" && !e.shiftKey) f = () => history("edit.undo");
  else if (mod && (key === "y" || (key === "z" && e.shiftKey))) f = () => history("edit.redo");
  else if (mod && key === "a") f = selectAll;
  // The hidden input holds no selection, so the browser fires no copy or cut of its own.
  else if (mod && key === "c" && !e.shiftKey) f = () => run("edit.copy");
  else if (mod && key === "x" && !e.shiftKey) f = () => run("edit.cut");
  else if (mod && key === "e") f = () => run("para.center");
  else if (mod && key === "l") f = () => run("para.left");
  else if (mod && key === "r") f = () => run("para.right");
  else if (mod && key === "j") f = () => run("para.justify");
  else if (mod && e.shiftKey && (e.key === ">" || e.key === ".")) f = () => run("font.increase");
  else if (mod && e.shiftKey && (e.key === "<" || e.key === ",")) f = () => run("font.decrease");
  else if (mod && key === "s") f = () => run("file.save");
  else if (mod && e.key === "Backspace") f = () => removeWord(-1);
  else if (mod && e.key === "Delete") f = () => removeWord(1);
  else if (mod && e.key === "ArrowLeft") f = () => jump("word-left", e.shiftKey);
  else if (mod && e.key === "ArrowRight") f = () => jump("word-right", e.shiftKey);
  else if (mod && e.key === "ArrowUp") f = () => jump("para-up", e.shiftKey);
  else if (mod && e.key === "ArrowDown") f = () => jump("para-down", e.shiftKey);
  else if (mod && e.key === "Home") f = () => jump("start", e.shiftKey);
  else if (mod && e.key === "End") f = () => jump("end", e.shiftKey);
  else if (e.key === "Backspace") f = () => remove(-1);
  else if (e.key === "Delete") f = () => remove(1);
  else if (e.key === "Enter") f = () => insert(e.shiftKey ? "\v" : "\n", false);
  // Tab at a paragraph's start, or over a selection, demotes it a list level, and
  // Shift+Tab promotes, as PowerPoint (`PAR-06`); elsewhere Tab types a tab.
  else if (e.key === "Tab" && e.shiftKey && !(e.ctrlKey || e.metaKey)) f = () => level("down");
  else if (e.key === "Tab" && !(e.ctrlKey || e.metaKey)) f = tab;
  else if (e.altKey && e.shiftKey && e.key === "ArrowRight") f = () => level("up");
  else if (e.altKey && e.shiftKey && e.key === "ArrowLeft") f = () => level("down");
  else if (e.key === "ArrowLeft") f = () => move("left", e.shiftKey);
  else if (e.key === "ArrowRight") f = () => move("right", e.shiftKey);
  else if (e.key === "ArrowUp") f = () => move("up", e.shiftKey);
  else if (e.key === "ArrowDown") f = () => move("down", e.shiftKey);
  else if (e.key === "Home") f = () => move("home", e.shiftKey);
  else if (e.key === "End") f = () => move("end", e.shiftKey);
  else if (e.key === "Escape") f = () => { stop(); byId("stage").focus({ preventScroll: true }); };
  if (f && !probe) f();
  return !!f;
}

function onInput(e) {
  if (composing || e.isComposing) return;
  const text = input.value;
  input.value = "";
  if (text) act(() => insert(text));
}

/* ── Commands ───────────────────────────────────────────────────────────── */

/// Bold, Italic and Underline in the Home ribbon, through the registry. Undo and
/// Redo are `editor.selection.js`'s, over the one history. Called by the core
/// before it fills in the commands not built yet.
export function registerTextCommands() {
  const open = () => !!state.deck && selection.editing;
  register("font.bold", { label: "Bold", shortcut: "Ctrl+B", enabled: open, checked: () => open() && !!(formatBits & 1), run: () => toggle(0) });
  register("font.italic", { label: "Italic", shortcut: "Ctrl+I", enabled: open, checked: () => open() && !!(formatBits & 2), run: () => toggle(1) });
  register("font.underline", { label: "Underline", shortcut: "Ctrl+U", enabled: open, checked: () => open() && !!(formatBits & 4), run: () => toggle(2) });
}

/// A new deck, or another slide: no text is open.
export function reset() {
  if (selection.editing) input?.blur();
  clearSelection();
  clearOverlay();
  clearInterval(blink);
}

/* ── Wiring ─────────────────────────────────────────────────────────────── */

export function init() {
  const frame = byId("slide-frame");
  overlay = document.createElementNS(SVG, "svg");
  overlay.setAttribute("class", "text-overlay");
  overlay.setAttribute("aria-hidden", "true");
  overlay.setAttribute("preserveAspectRatio", "none");
  frame.appendChild(overlay);
  input = document.createElement("textarea");
  input.className = "text-input";
  input.setAttribute("aria-label", "Text");
  input.setAttribute("autocomplete", "off");
  input.setAttribute("autocorrect", "off");
  input.setAttribute("spellcheck", "false");
  frame.appendChild(input);

  new ResizeObserver(fitOverlay).observe(frame);
  new MutationObserver(fitOverlay).observe(frame, { attributes: true, attributeFilter: ["hidden"] });

  // Presses on the slide are routed by `editor.selection.js`; one on the grey
  // around it closes the box and clears the selection, as PowerPoint's.
  byId("stage").addEventListener("pointerdown", (e) => {
    if (e.button !== 0 || !e.target.isConnected || e.target.closest("#slide-frame")) return;
    stop();
    shapes.deselect();
  });
  window.addEventListener("pointerup", () => { dragging = false; });
  input.addEventListener("keydown", onKeyDown);
  input.addEventListener("input", onInput);
  input.addEventListener("compositionstart", () => { composing = true; });
  input.addEventListener("compositionend", (e) => {
    composing = false;
    input.value = "";
    const data = e.data;
    if (data) act(() => insert(data));
  });
  input.addEventListener("paste", (e) => {
    e.preventDefault();
    const text = e.clipboardData?.getData("text/plain");
    if (text) act(() => insert(text.replace(/\r\n?/g, "\n"), false));
  });
}
