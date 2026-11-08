// Co-editing in the editor (`COL-04`): Share, who is here, and where they are.
//
// Shaped after OpenCalc's `webapp/editor.presence.js` at `b23c67c` (webapp/README.md):
// the Share dialog makes an invite link that carries the room, not a credential;
// the roster and each participant's colour come from the server; another
// participant's selection and caret are drawn over the slide in their colour and
// never enter the document or the undo history. Changed for slides: a room is
// served by the same server as this page, so the socket is this origin and the
// page's `connect-src 'self'` holds; the link is this page with `?room=`.
//
// Remote edits go through the same engine `apply` as local ones (the bridge's
// shared session); here they only redraw what they touched and move the caret.

import { register } from "./commands.js";
import { showModal, closeModal } from "./editor.dialogs.js";
import { collaborate } from "./collab.js";
import { state, request, status, changed, reopen, deckBytes } from "./editor.core.js";
import * as slides from "./editor.slides.js";
import * as selection from "./editor.selection.js";
import * as text from "./editor.text.js";

const SVG = "http://www.w3.org/2000/svg";
const byId = (id) => document.getElementById(id);

/// How often this participant's own presence is compared and, if changed, sent.
const PRESENCE_MS = 200;

let session = null;     // the `collaborate` handle while sharing
let room = "";
let me = { client: 0, colour: "" };
let connection = "";    // the last status
const roster = new Map(); // client -> { name, colour, slide, shapes, caret }
let overlay = null;
let lastSent = "";
let presenceTimer = 0;
let drawToken = 0;

const engine = {
  async call(fn, ...args) {
    const r = await request({ op: "collab", fn, args }, { priority: 9 });
    return r.value;
  },
  load: (bytes) => reopen(bytes),
};

function el(tag, cls, textValue) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (textValue != null) e.textContent = textValue;
  return e;
}

function newRoomKey() {
  if (globalThis.crypto?.randomUUID) return globalThis.crypto.randomUUID();
  const b = new Uint8Array(16);
  globalThis.crypto.getRandomValues(b);
  return Array.from(b, (x) => x.toString(16).padStart(2, "0")).join("");
}

function inviteLink(key) {
  const link = new URL(location.origin + location.pathname);
  link.searchParams.set("room", key);
  return link.toString();
}

function socketUrl() {
  const u = new URL("/collab", location.href);
  u.protocol = location.protocol === "https:" ? "wss:" : "ws:";
  return u.toString();
}

/* ── Who is here ─────────────────────────────────────────────────────────── */

function showRoster() {
  // The bar's promise that nothing leaves the browser is not true while sharing.
  const local = document.querySelector(".bottom-status");
  if (local) local.hidden = !!session;
  const box = byId("collab-status");
  if (!box) return;
  box.replaceChildren();
  box.hidden = !session;
  if (!session) return;
  const dot = (colour, label) => {
    const d = el("span", "collab-dot");
    d.style.backgroundColor = colour;
    d.title = label;
    return d;
  };
  if (me.colour) box.appendChild(dot(me.colour, "You"));
  for (const p of roster.values()) box.appendChild(dot(p.colour, p.name));
  const names = ["You", ...Array.from(roster.values(), (p) => p.name)];
  const live = connection === "live" ? "" : ` (${connection})`;
  box.appendChild(el("span", "collab-names", `Sharing: ${names.join(", ")}${live}`));
}

/* ── Others' selection and caret ─────────────────────────────────────────── */

function ensureOverlay() {
  if (overlay) return overlay;
  overlay = document.createElementNS(SVG, "svg");
  overlay.setAttribute("class", "presence-overlay");
  overlay.setAttribute("aria-hidden", "true");
  overlay.setAttribute("preserveAspectRatio", "none");
  byId("slide-frame").appendChild(overlay);
  return overlay;
}

async function drawPresence() {
  const token = ++drawToken;
  const svg = ensureOverlay();
  if (!state.deck) { svg.replaceChildren(); byId("presence-names")?.replaceChildren(); return; }
  svg.setAttribute("viewBox", `0 0 ${state.deck.width} ${state.deck.height}`);
  const shapes = selection.allShapes();
  const nodes = [];
  const labels = [];
  const px = state.deck.width / Math.max(1, byId("slide-frame").clientWidth);   // EMU per pixel
  for (const [client, p] of roster) {
    if (p.slide !== state.current) continue;
    for (const n of p.shapes || []) {
      const s = shapes.find((x) => x.node === n);
      if (!s) continue;
      const r = document.createElementNS(SVG, "rect");
      r.setAttribute("x", String(s.x));
      r.setAttribute("y", String(s.y));
      r.setAttribute("width", String(s.cx));
      r.setAttribute("height", String(s.cy));
      r.setAttribute("transform", `rotate(${s.rot} ${s.x + s.cx / 2} ${s.y + s.cy / 2})`);
      r.setAttribute("class", "presence-outline");
      r.setAttribute("stroke", p.colour);
      r.setAttribute("stroke-width", String(2 * px));
      nodes.push(r);
      // The name as a page element: SVG text at slide scale is too large a font
      // for Chrome to draw.
      const label = el("span", "presence-name", p.name);
      label.style.left = `${(s.x / state.deck.width) * 100}%`;
      label.style.top = `${(s.y / state.deck.height) * 100}%`;
      label.style.backgroundColor = p.colour;
      labels.push(label);
    }
    const c = p.caret;
    const s = c && shapes.find((x) => x.node === c.node);
    if (s && s.part != null) {
      try {
        const index = state.current;
        if (c.anchor !== c.offset) {
          const quads = (await request({ op: "text", fn: "text_selection", args: [index, s.part, c.node, Math.min(c.anchor, c.offset), Math.max(c.anchor, c.offset)] }, { priority: 2 })).value;
          for (let i = 0; i + 8 <= quads.length; i += 8) {
            const poly = document.createElementNS(SVG, "polygon");
            poly.setAttribute("points", quads.slice(i, i + 8).join(" "));
            poly.setAttribute("fill", p.colour);
            poly.setAttribute("class", "presence-selection");
            nodes.push(poly);
          }
        }
        const [x1, y1, x2, y2] = (await request({ op: "text", fn: "text_caret", args: [index, s.part, c.node, c.offset] }, { priority: 2 })).value;
        const line = document.createElementNS(SVG, "line");
        for (const [k, v] of [["x1", x1], ["y1", y1], ["x2", x2], ["y2", y2]]) line.setAttribute(k, String(v));
        line.setAttribute("stroke", p.colour);
        line.setAttribute("stroke-width", String(2 * px));
        line.setAttribute("class", "presence-caret");
        line.dataset.client = String(client);
        nodes.push(line);
      } catch { /* the caret is past what this copy has yet */ }
    }
  }
  if (token !== drawToken) return;
  svg.replaceChildren(...nodes);
  let names = byId("presence-names");
  if (!names) {
    names = el("div", "presence-names");
    names.id = "presence-names";
    byId("slide-frame").appendChild(names);
  }
  names.replaceChildren(...labels);
}

/* ── This participant's own presence ─────────────────────────────────────── */

function sendPresence(force = false) {
  if (!session || !state.deck) return;
  const node = selection.selectedNode();
  const caret = text.caretState();
  const mine = {
    slide: state.current,
    shapes: node != null ? [node] : caret ? [caret.node] : [],
    caret: caret ? { node: caret.node, offset: caret.focus, anchor: caret.anchor } : null,
  };
  const key = JSON.stringify(mine);
  if (!force && key === lastSent) return;
  lastSent = key;
  session.present(mine.slide, mine.shapes, mine.caret).catch(() => {});
}

/* ── Arrivals ────────────────────────────────────────────────────────────── */

async function onDocument(d) {
  if (d.reason === "joined") {
    me = { client: d.client, colour: d.colour };
    lastSent = "";
    showRoster();
    sendPresence(true);
    changed();
    return;
  }
  if (d.reason !== "remote") return;
  const touched = d.touched || [];
  // The caret first, while the arrival is the last one the engine saw.
  const caret = text.caretState();
  if (caret) {
    const map = (o) => engine.call("collab_map_offset", caret.part, caret.node, o);
    const [anchor, focus] = await Promise.all([map(caret.anchor), map(caret.focus)]);
    if (anchor !== caret.anchor || focus !== caret.focus) text.remap(anchor, focus);
  }
  for (const i of touched) if (i >= 0) slides.redrawSlide(i);
  if (touched.includes(state.current) || touched.includes(-2)) {
    await selection.refresh();
    text.refresh();
  }
  drawPresence();
}

function onPresence(p) {
  if (p.kind === "gone") {
    roster.delete(p.client);
  } else {
    // Someone new: tell them where this participant is, since presence is only
    // sent when it changes.
    if (!roster.has(p.client)) setTimeout(() => sendPresence(true), 0);
    roster.set(p.client, {
      name: p.name || `Guest ${p.client}`,
      colour: p.colour,
      slide: p.slide ?? 0,
      shapes: p.shapes || [],
      caret: p.caret || null,
    });
  }
  showRoster();
  drawPresence();
}

function onStatus({ state: s, detail }) {
  connection = s;
  if (s === "live") status("Sharing: changes are live");
  else if (s === "lost") status("Reconnected; edits made while away were not kept");
  else if (s === "stopped") status(`Sharing stopped${detail ? ` (${detail})` : ""}`);
  else if (s === "refused") status(`The server refused that edit${detail ? ` (${detail})` : ""}`);
  else if (s === "reconnecting") status("Reconnecting…");
  if (s === "stopped") stop();
  showRoster();
}

/* ── Starting and stopping ───────────────────────────────────────────────── */

function start(key, name) {
  if (session) return;
  room = key;
  roster.clear();
  session = collaborate({
    url: socketUrl(),
    room: key,
    name,
    seed: deckBytes,
    engine,
    onStatus,
    onDocument,
    onPresence,
  });
  clearInterval(presenceTimer);
  presenceTimer = setInterval(() => sendPresence(), PRESENCE_MS);
  // The address bar carries the room, so a reload rejoins it.
  history.replaceState(null, "", inviteLink(key));
  showRoster();
  changed();
}

function stop() {
  clearInterval(presenceTimer);
  session?.close();
  session = null;
  roster.clear();
  me = { client: 0, colour: "" };
  drawPresence();
  history.replaceState(null, "", location.pathname);
  showRoster();
  changed();
}

function nameField(value) {
  const wrap = el("label", "share-field");
  wrap.appendChild(el("span", "share-field-label", "Your name"));
  const input = document.createElement("input");
  input.type = "text";
  input.id = "share-name";
  input.maxLength = 64;
  input.value = value;
  input.autocomplete = "off";
  input.spellcheck = false;
  wrap.appendChild(input);
  return wrap;
}

/// The name this tab goes by: its own if given (two tabs can be two people), else
/// the one this browser last used.
function storedName() {
  try { return sessionStorage.getItem("op.name") || localStorage.getItem("op.name") || ""; } catch { return ""; }
}

function shareDialog() {
  showModal("Share", (body) => {
    if (session) {
      const others = roster.size;
      body.appendChild(el("p", "op-confirm-text", others === 0
        ? "Sharing. Nobody else has joined yet."
        : `Sharing with ${Array.from(roster.values(), (p) => p.name).join(", ")}.`));
      const wrap = el("label", "share-field");
      wrap.appendChild(el("span", "share-field-label", "Link to this presentation"));
      const link = document.createElement("input");
      link.type = "text";
      link.id = "share-link";
      link.readOnly = true;
      link.value = inviteLink(room);
      wrap.appendChild(link);
      body.appendChild(wrap);
      body.appendChild(el("p", "op-muted", "Anyone with the link can edit while someone is in the room. The presentation goes through the sharing server, not a third party."));
      const actions = el("div", "op-confirm-actions");
      const copy = el("button", "op-btn primary", "Copy link");
      copy.type = "button";
      copy.id = "share-copy";
      copy.addEventListener("click", async () => {
        try { await navigator.clipboard.writeText(link.value); copy.textContent = "Copied"; }
        catch { link.select(); copy.textContent = "Press Ctrl+C"; }
      });
      const stopBtn = el("button", "op-btn", "Stop sharing");
      stopBtn.type = "button";
      stopBtn.id = "share-stop";
      stopBtn.addEventListener("click", () => { stop(); status("Stopped sharing"); closeModal(); });
      actions.append(copy, stopBtn);
      body.appendChild(actions);
      return;
    }
    body.appendChild(el("p", "op-confirm-text", "Edit this presentation with others at the same time. Starting makes a link to send them."));
    body.appendChild(nameField(storedName()));
    const actions = el("div", "op-confirm-actions");
    const go = el("button", "op-btn primary", "Start sharing");
    go.type = "button";
    go.id = "share-start";
    go.addEventListener("click", () => {
      const name = (byId("share-name").value || "").trim() || "Guest";
      try { localStorage.setItem("op.name", name); sessionStorage.setItem("op.name", name); } catch { /* private mode */ }
      start(newRoomKey(), name);
      closeModal();
      // Show the link once the room is live.
      setTimeout(shareDialog, 600);
    });
    const cancel = el("button", "op-btn", "Cancel");
    cancel.type = "button";
    cancel.addEventListener("click", closeModal);
    actions.append(go, cancel);
    body.appendChild(actions);
  });
}

/// A page opened from an invite link joins its room.
export function joinFromLink() {
  const key = new URL(location.href).searchParams.get("room");
  if (!key || !/^[A-Za-z0-9-]{8,64}$/.test(key)) return;
  start(key, storedName() || "Guest");
}

export function sharing() { return !!session; }

/// Who else is here and where, as last heard: for the status and for tests.
export function participants() {
  return Array.from(roster, ([client, p]) => ({ client, ...p }));
}

export function init() {
  register("file.share", { label: "Share", enabled: () => !!state.deck, checked: () => !!session, run: shareDialog });
  const box = el("span", "collab-status");
  box.id = "collab-status";
  box.hidden = true;
  box.setAttribute("aria-live", "polite");
  byId("tb-status").after(box);
  // Others' marks follow the slide on the stage.
  document.addEventListener("op-changed", () => { if (session) drawPresence(); });
}
