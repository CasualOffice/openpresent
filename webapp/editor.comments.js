// The Comments pane (`UX-011`): modern threaded comments as PowerPoint for the web
// shows them. Review > New Comment (Ctrl+Alt+M), Show Comments, and the Comments
// buttons at the top right and in the status bar open it.
//
// A comment is on the slide, or on the shape selected when it was made; it has
// replies, and can be resolved, reopened and deleted, a reply on its own too. An
// @mention is the text typed and is sent nowhere: this build has no people to notify.
// Each verb is one engine call through the modern-comment model (Tier 1M), one undo
// step, and a save writes the comment part. Markers on the slide show where each
// thread is while the pane is open; a click on one picks its thread.
//
// Shaped after OpenCalc's notes panel (`buildNotePanel` in its `editor.core.js`): the
// thread is read back from the engine after every change rather than patched in the
// DOM, so the pane never drifts from what is stored, and a draft is never overwritten
// by a redraw. Comment text, names and times reach the DOM through `textContent`.

import { state, request, status } from "./editor.core.js";
import { register } from "./commands.js";
import { icon } from "./ribbon.js";
import * as host from "./editor.panehost.js";
import * as selection from "./editor.selection.js";
import * as slides from "./editor.slides.js";
import * as author from "./editor.author.js";
import { errorDialog } from "./editor.dialogs.js";

const byId = (id) => document.getElementById(id);
const has = () => !!state.deck && state.deck.count > 0;

let body = null;        // the pane's content, while it is open
let threads = [];       // the current slide's, as the engine last said
let lastJson = "";
let readFor = -1;
let active = null;      // the chosen thread's id
let composing = null;   // { node, x, y } while a new comment is being written

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

function button(label, cls, onClick, ic) {
  const b = h("button", cls);
  b.type = "button";
  if (ic) b.appendChild(icon(ic, 16));
  b.appendChild(h("span", null, label));
  b.addEventListener("click", onClick);
  return b;
}

async function call(fn, args) {
  const r = await request({ op: "review", fn, args }, { priority: 9 });
  if (r.undo) selection.noteUndoState(r.undo);
  return r.value;
}

/// "3 minutes ago", or the date; the full time stays in the title.
function relative(iso) {
  const t = Date.parse(iso);
  if (!Number.isFinite(t)) return "";
  const s = Math.round((Date.now() - t) / 1000);
  if (s < 45) return "Just now";
  if (s < 3600) return `${Math.round(s / 60)} min ago`;
  if (s < 86400) return `${Math.round(s / 3600)} h ago`;
  return new Date(t).toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" });
}

/// Text with each @mention in its own span, all through `textContent`.
function textWithMentions(text) {
  const box = h("div", "cm-text");
  for (const part of String(text).split(/(@[\p{L}\p{N}_.-]+(?: [\p{Lu}][\p{L}-]+)?)/u)) {
    if (!part) continue;
    box.appendChild(part.startsWith("@") ? h("span", "cm-mention", part) : document.createTextNode(part));
  }
  return box;
}

function avatar(a) {
  const name = a?.name || "Author";
  const ini = a?.initials || name.split(/\s+/).map((w) => w[0] || "").join("").slice(0, 2).toUpperCase();
  const e = h("span", "cm-avatar", ini);
  e.setAttribute("aria-hidden", "true");
  return e;
}

function entry(item, isRoot, thread) {
  const box = h("div", isRoot ? "cm-entry cm-root" : "cm-entry");
  const head = h("div", "cm-head");
  head.appendChild(avatar(item.author));
  const who = h("div", "cm-who");
  who.appendChild(h("span", "cm-name", item.author?.name || "Author"));
  const when = h("span", "cm-when", relative(item.created));
  if (item.created) when.title = item.created;
  who.appendChild(when);
  head.appendChild(who);
  if (!isRoot) {
    const x = button("", "cm-icon", (e) => { e.stopPropagation(); remove(thread.id, item.id); }, "dismiss");
    x.title = "Delete reply";
    x.setAttribute("aria-label", "Delete reply");
    head.appendChild(x);
  }
  box.append(head, textWithMentions(item.text));
  return box;
}

/// A text box that posts on Ctrl+Enter, with Post and Cancel.
function composer(placeholder, onPost, onCancel, autofocus) {
  const wrap = h("div", "cm-compose");
  const ta = h("textarea", "cm-input");
  ta.rows = 2;
  ta.placeholder = placeholder;
  ta.setAttribute("aria-label", placeholder);
  if (autofocus) ta.dataset.autofocus = "";
  const row = h("div", "cm-actions");
  const post = button("Post", "op-btn primary cm-post", () => send());
  post.disabled = true;
  post.title = "Post (Ctrl+Enter)";
  const send = async () => {
    const text = ta.value.trim();
    if (!text) return;
    post.disabled = true;
    // Emptied first, so the redraw after posting carries no draft back.
    ta.value = "";
    if (!(await onPost(text))) ta.value = text;
    post.disabled = !ta.value.trim();
  };
  ta.addEventListener("input", () => { post.disabled = !ta.value.trim(); });
  ta.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); send(); }
    else if (e.key === "Escape" && onCancel) { e.preventDefault(); onCancel(); }
  });
  row.appendChild(post);
  if (onCancel) row.appendChild(button("Cancel", "op-btn", onCancel));
  wrap.append(ta, row);
  return wrap;
}

/// The first-use name prompt, in the pane: never a browser dialog.
function namePrompt(after) {
  const box = h("div", "cm-name-prompt");
  box.appendChild(h("p", "cm-hint", "Your name is shown on the comments you add. It is kept in this browser; change it in File > Options."));
  const input = h("input", "fp-input cm-name-input");
  input.type = "text";
  input.placeholder = "Your name";
  input.setAttribute("aria-label", "Your name");
  input.dataset.autofocus = "";
  input.maxLength = 255;
  const ok = button("OK", "op-btn primary", () => { author.setName(input.value); after(); });
  input.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.key === "Enter") { e.preventDefault(); ok.click(); }
  });
  const row = h("div", "cm-actions");
  row.append(input, ok);
  box.appendChild(row);
  return box;
}

function threadCard(t) {
  const card = h("article", "cm-thread");
  card.dataset.id = t.id;
  card.tabIndex = 0;
  card.classList.toggle("is-active", t.id === active);
  card.classList.toggle("is-resolved", !!t.resolved);
  if (t.node >= 0) card.appendChild(h("div", "cm-on", "On a shape"));
  if (t.resolved) card.appendChild(h("div", "cm-resolved", "Resolved"));
  card.appendChild(entry(t, true, t));
  for (const r of t.replies) card.appendChild(entry(r, false, t));
  const tools = h("div", "cm-tools");
  tools.append(
    button(t.resolved ? "Reopen" : "Resolve", "op-btn cm-resolve", (e) => { e.stopPropagation(); resolve(t.id, !t.resolved); }, t.resolved ? "arrow_undo" : "checkmark"),
    button("Delete thread", "op-btn cm-delete", (e) => { e.stopPropagation(); remove(t.id, ""); }, "comment_dismiss"),
  );
  card.appendChild(tools);
  if (!t.resolved) card.appendChild(composer("Reply", (text) => reply(t.id, text), null, false));
  card.addEventListener("click", (e) => choose(t.id, !e.target.closest("textarea, input, button")));
  card.addEventListener("focusin", () => { if (active !== t.id) choose(t.id, false); });
  return card;
}

function renderList() {
  if (!body) return;
  const list = body.querySelector(".cm-list");
  // Keep what is being typed: a draft in a reply box survives a redraw of its thread.
  const drafts = new Map();
  for (const c of list.querySelectorAll(".cm-thread")) {
    const ta = c.querySelector(".cm-input");
    if (ta && ta.value) drafts.set(c.dataset.id, ta.value);
  }
  const focused = document.activeElement?.closest?.(".cm-thread")?.dataset.id;
  list.textContent = "";
  if (!threads.length && !composing) list.appendChild(h("p", "fp-none", "There are no comments on this slide. Select New to add one."));
  for (const t of threads) {
    const card = threadCard(t);
    const ta = card.querySelector(".cm-input");
    if (ta && drafts.has(t.id)) { ta.value = drafts.get(t.id); card.querySelector(".cm-post").disabled = false; }
    list.appendChild(card);
  }
  if (focused) list.querySelector(`.cm-thread[data-id="${CSS.escape(focused)}"] .cm-input`)?.focus({ preventScroll: true });
  drawMarkers();
}

function renderTop() {
  const top = body.querySelector(".cm-top");
  top.textContent = "";
  if (!author.hasName()) { top.appendChild(namePrompt(() => renderTop())); top.querySelector("input")?.focus(); return; }
  if (composing) {
    const where = composing.node >= 0 ? "Comment on the selected shape" : "Comment on this slide";
    top.appendChild(h("div", "cm-on", where));
    top.appendChild(composer("Start a conversation. Use @ to mention someone.", (text) => create(text), () => { composing = null; renderTop(); renderList(); }, true));
    top.querySelector(".cm-input")?.focus({ preventScroll: true });
  } else {
    top.appendChild(button("New", "op-btn primary cm-new", () => newComment(), "comment_add"));
  }
}

function build(b, opts = {}) {
  body = h("div", "cm-pane");
  body.append(h("div", "cm-top"), h("div", "cm-list"));
  b.appendChild(body);
  if (opts.compose) startComposing();
  renderTop();
  lastJson = "";
  read(true);
}

async function read(force = false) {
  if (!body || !has()) return;
  const index = state.current;
  try {
    const json = await call("slide_comments", [index]);
    if (!force && json === lastJson && readFor === index) return;
    if (readFor !== -1 && readFor !== index) { composing = null; active = null; renderTop(); }
    readFor = index;
    lastJson = json;
    threads = JSON.parse(json);
    renderList();
  } catch (e) {
    status(`The comments could not be read: ${e.message}`);
  }
}

function startComposing() {
  const s = selection.selectedShape();
  const d = state.deck;
  composing = s
    ? { node: s.node, x: Math.max(0, s.x + s.cx), y: Math.max(0, s.y) }
    : { node: -1, x: Math.round((d?.width || 0) * 0.04), y: Math.round((d?.height || 0) * 0.06) };
}

/// Review > New Comment: on the selected shape, or the slide.
function newComment() {
  if (!has()) return;
  startComposing();
  if (!host.isOpen("comments")) { host.open("comments", { compose: true }); return; }
  renderTop();
  renderList();
}

async function guarded(what, fn) {
  try { return await fn(); } catch (e) { errorDialog(`OpenPresent could not ${what}.`, e.message); return false; }
}

async function edited() {
  await read(true);
  slides.refreshThumb?.(state.current);
}

function create(text) {
  const c = composing;
  if (!c) return false;
  return guarded("add the comment", async () => {
    const me = author.author();
    const id = author.newGuid();
    await call("add_comment", [state.current, c.node, c.x, c.y, text, me.id, me.name || "Author", id, author.now()]);
    composing = null;
    active = id;
    renderTop();
    await edited();
    status("Comment added");
    return true;
  });
}

function reply(thread, text) {
  return guarded("reply", async () => {
    const me = author.author();
    if (!author.hasName()) { renderTop(); }
    await call("reply_comment", [state.current, thread, text, me.id, me.name || "Author", author.newGuid(), author.now()]);
    active = thread;
    await edited();
    return true;
  });
}

function resolve(thread, on) {
  return guarded(on ? "resolve the thread" : "reopen the thread", async () => {
    await call("resolve_comment", [state.current, thread, on]);
    await edited();
    status(on ? "Thread resolved" : "Thread reopened");
  });
}

function remove(thread, replyId) {
  return guarded("delete the comment", async () => {
    await call("delete_comment", [state.current, thread, replyId]);
    if (!replyId && active === thread) active = null;
    await edited();
    status(replyId ? "Reply deleted" : "Thread deleted");
  });
}

function choose(id, focus = true) {
  active = id;
  const t = threads.find((x) => x.id === id);
  if (t && t.node >= 0) selection.selectNode(t.node);
  for (const c of body?.querySelectorAll(".cm-thread") || []) c.classList.toggle("is-active", c.dataset.id === id);
  if (focus) body?.querySelector(`.cm-thread[data-id="${CSS.escape(id)}"]`)?.focus({ preventScroll: true });
  body?.querySelector(`.cm-thread[data-id="${CSS.escape(id)}"]`)?.scrollIntoView({ block: "nearest" });
  drawMarkers();
}

/* ── Markers on the slide ───────────────────────────────────────────────── */

function drawMarkers() {
  const frame = byId("slide-frame");
  let layer = byId("cm-markers");
  if (!layer) {
    layer = h("div", "cm-markers");
    layer.id = "cm-markers";
    frame.appendChild(layer);
  }
  layer.textContent = "";
  const d = state.deck;
  if (!body || !d || !d.width || readFor !== state.current) return;
  threads.forEach((t, i) => {
    const m = h("button", "cm-marker");
    m.type = "button";
    m.classList.toggle("is-active", t.id === active);
    m.classList.toggle("is-resolved", !!t.resolved);
    m.appendChild(icon("comment", 16));
    m.setAttribute("aria-label", `Comment ${i + 1} by ${t.author?.name || "Author"}`);
    const x = t.x ?? 0, y = t.y ?? 0;
    m.style.left = `${Math.min(97, (x / d.width) * 100)}%`;
    m.style.top = `${Math.min(95, (y / d.height) * 100)}%`;
    m.addEventListener("pointerdown", (e) => e.stopPropagation());
    m.addEventListener("click", (e) => { e.stopPropagation(); choose(t.id); });
    layer.appendChild(m);
  });
}

function step(dir) {
  if (!threads.length) return;
  const i = threads.findIndex((t) => t.id === active);
  const next = i < 0 ? (dir > 0 ? 0 : threads.length - 1) : Math.min(threads.length - 1, Math.max(0, i + dir));
  if (!host.isOpen("comments")) host.open("comments");
  choose(threads[next].id);
}

host.registerPane("comments", {
  title: "Comments",
  build,
  refresh: () => read(),
  closed: () => { body = null; composing = null; readFor = -1; drawMarkers(); },
});

export function init() {
  register("review.new-comment", { label: "New Comment", shortcut: "Ctrl+Alt+M", enabled: has, run: newComment });
  register("review.show-comments", { label: "Show Comments", enabled: has, checked: () => host.isOpen("comments"), run: () => host.toggle("comments") });
  register("review.delete-comment", { label: "Delete Comment", enabled: () => has() && host.isOpen("comments") && !!active, run: () => active && remove(active, "") });
  register("review.previous-comment", { label: "Previous", enabled: () => has() && host.isOpen("comments") && threads.length > 0, run: () => step(-1) });
  register("review.next-comment", { label: "Next", enabled: () => has() && host.isOpen("comments") && threads.length > 0, run: () => step(1) });
  document.addEventListener("keydown", (e) => {
    if (e.ctrlKey && e.altKey && (e.key === "m" || e.key === "M" || e.code === "KeyM")) { e.preventDefault(); newComment(); }
  });
  // Another slide, an undo or a redo: read again; the same JSON draws nothing.
  let pending = 0;
  document.addEventListener("op-changed", () => {
    if (!body) return;
    clearTimeout(pending);
    pending = setTimeout(() => read(readFor !== state.current), 60);
  });
  document.addEventListener("op-author-changed", () => { if (body) renderTop(); });
}
