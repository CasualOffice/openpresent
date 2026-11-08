// Autosave and crash recovery (`SAVE-01`).
//
// Copied from OpenCalc's `webapp/editor.drafts.js` (`SAVE-03`, its docs/83 §4 and
// §8 Phase B; webapp/README.md), with what a deck needs changed and nothing else:
//
// - **A draft is insurance, not a save.** A copy of the deck written to this
//   browser's own IndexedDB, offered back on the next open, never applied by itself
//   and never uploaded. Nothing here fetches.
// - **The cadence is OpenCalc's, which is its collaboration server's**: write after
//   5 s of quiet, at least every 60 s of steady editing, or every 200 edits; and
//   once more when the tab is hidden, the last moment a page is reliably given.
// - **The dirty signal is the engine's edit counter** (`edits_applied`), which counts
//   up and never down, so an undo is an edit; a draft is written only when it moved
//   since the last draft, and the tab's own draft is deleted once the deck is saved.
// - **One slot a tab**, leased over a `BroadcastChannel`, and a slot holding a draft
//   the bar is offering is never written over.
//
// Changed for OpenPresent: the engine runs in a worker, so the counter and the bytes
// are requests (`{ op: "edits" }`, `{ op: "save" }`); Restore opens a draft in this
// tab while the deck on screen is unedited, and in a new tab otherwise, so nothing on
// screen is replaced behind the user's back; and the recovery bar's buttons are our
// own modal's, never a browser `alert` or `confirm`.
//
// What it does not keep: the undo history. `History` holds model operations with no
// serialized form, so a recovered deck opens with an empty undo stack, and the bar
// says so (`SAVE-09`).

import { state, request, status, openDeck } from "./editor.core.js";
import { showModal, closeModal } from "./editor.dialogs.js";

const byId = (id) => document.getElementById(id);

const POLICY = Object.freeze({ quiesceMs: 5_000, ceilingMs: 60_000, everyEdits: 200, pollMs: 1_000 });

/* ── The store ─────────────────────────────────────────────────────────── */

const DB_NAME = "openpresent-drafts";
const DB_VERSION = 1;
/// The record shape's version. A record from a newer build is listed and left alone.
const SCHEMA = 1;

let storageFault = "";
let dbPromise = null;

function openDb() {
  if (dbPromise) return dbPromise;
  dbPromise = new Promise((resolve, reject) => {
    let req;
    try {
      if (typeof indexedDB === "undefined" || !indexedDB) throw new Error("no IndexedDB in this browser");
      req = indexedDB.open(DB_NAME, DB_VERSION);
    } catch (e) {
      reject(e);
      return;
    }
    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains("meta")) db.createObjectStore("meta", { keyPath: "id" });
      if (!db.objectStoreNames.contains("bytes")) db.createObjectStore("bytes", { keyPath: "id" });
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error || new Error("IndexedDB refused to open"));
    req.onblocked = () => reject(new Error("another tab is holding an older draft store open"));
  });
  dbPromise.catch(() => {});
  return dbPromise;
}

function txDone(tx) {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error || new Error("draft transaction failed"));
    tx.onabort = () => reject(tx.error || new Error("draft transaction aborted"));
  });
}

function reqDone(req) {
  return new Promise((resolve, reject) => {
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error || new Error("draft request failed"));
  });
}

/// Every draft's record, newest first.
export async function listDrafts() {
  const db = await openDb();
  const rows = await reqDone(db.transaction("meta", "readonly").objectStore("meta").getAll());
  return (rows || []).filter((r) => r && typeof r.id === "string").sort((a, b) => (b.at || 0) - (a.at || 0));
}

async function readDraftBytes(id) {
  const db = await openDb();
  const row = await reqDone(db.transaction("bytes", "readonly").objectStore("bytes").get(id));
  return row ? row.bytes : null;
}

async function deleteDraft(id) {
  const db = await openDb();
  const tx = db.transaction(["meta", "bytes"], "readwrite");
  tx.objectStore("meta").delete(id);
  tx.objectStore("bytes").delete(id);
  await txDone(tx);
}

/// Both stores in one transaction: a record without its bytes would be an entry the
/// bar offers and cannot open.
async function putDraft(meta, bytes) {
  const db = await openDb();
  const tx = db.transaction(["meta", "bytes"], "readwrite");
  tx.objectStore("bytes").put({ id: meta.id, bytes });
  tx.objectStore("meta").put(meta);
  await txDone(tx);
}

/* ── The lease ─────────────────────────────────────────────────────────── */

const CHANNEL = "openpresent-drafts";
const LEASE_WAIT_MS = 150;
let channel = null;
let mySlot = null;
let leasePromise = null;
let reservedSlots = new Set();
let openedDraftId = null;

function slotOf(id) {
  const m = /^slot-(\d+)$/.exec(String(id || ""));
  return m ? Number(m[1]) : null;
}

function claimSlot() {
  if (leasePromise) return leasePromise;
  const adopted = slotOf(openedDraftId);
  if (adopted !== null) {
    mySlot = adopted;
    leasePromise = Promise.resolve(adopted);
    return leasePromise;
  }
  leasePromise = new Promise((resolve) => {
    const held = new Set();
    try { channel = new BroadcastChannel(CHANNEL); } catch { channel = null; }
    const pick = () => {
      let slot = 0;
      while (held.has(slot) || reservedSlots.has(slot)) slot += 1;
      mySlot = slot;
      resolve(slot);
    };
    if (!channel) { pick(); return; }
    channel.onmessage = (e) => {
      const msg = e && e.data;
      if (!msg) return;
      if (msg.type === "who" && mySlot !== null) channel.postMessage({ type: "holding", slot: mySlot });
      else if (msg.type === "holding" && typeof msg.slot === "number") held.add(msg.slot);
    };
    channel.postMessage({ type: "who" });
    setTimeout(() => {
      pick();
      try { channel.postMessage({ type: "holding", slot: mySlot }); } catch { /* closed */ }
    }, LEASE_WAIT_MS);
  });
  return leasePromise;
}

/* ── Writing ───────────────────────────────────────────────────────────── */

/// The counter at the last draft this tab wrote; `null` for none yet.
let draftedAtEdits = null;
/// The counter at the last save, or at open: the deck's "nothing to keep" state.
let savedAtEdits = 0;
let lastDraftAt = 0;
let lastEditsSeen = null;
let lastEditAt = 0;
let lastDeck = null;
let timer = null;
let writing = false;
let lastReason = "";

async function editsApplied() {
  try {
    return (await request({ op: "edits" }, { priority: -5 })).edits;
  } catch {
    return Number.NaN;
  }
}

async function writeDraft(reason, at) {
  if (writing || storageFault || !state.deck) return "";
  writing = true;
  try {
    let bytes;
    try {
      bytes = new Uint8Array((await request({ op: "save" }, { priority: -4 })).bytes);
    } catch (e) {
      faultAutosave(`Autosave off: the presentation could not be prepared (${e.message})`);
      return "";
    }
    const slot = await claimSlot();
    const meta = {
      id: `slot-${slot}`,
      schema: SCHEMA,
      name: state.deck.name || "Presentation.pptx",
      edits: at,
      ahead: Math.max(0, at - savedAtEdits),
      at: Date.now(),
      size: bytes.length,
    };
    try {
      await putDraft(meta, bytes);
    } catch (e) {
      faultAutosave(e && (e.name === "QuotaExceededError" || /quota/i.test(String(e.message || e)))
        ? "Autosave off: no storage space"
        : `Autosave off: this browser is not storing drafts (${e.message || e})`);
      return "";
    }
    draftedAtEdits = at;
    lastDraftAt = Date.now();
    lastReason = reason;
    try { navigator.storage?.persist?.().catch(() => {}); } catch { /* not offered */ }
    return reason;
  } finally {
    writing = false;
  }
}

/// A standing indicator in the status bar, not a toast: autosave never stops quietly.
function faultAutosave(message) {
  if (storageFault === message) return;
  storageFault = message;
  const el = byId("autosave-state");
  if (el) { el.textContent = message; el.hidden = false; }
  stopScheduler();
}

let ticking = false;
async function tick() {
  if (ticking || !state.deck) return;
  ticking = true;
  try {
    const now = Date.now();
    // Another deck on screen (File > New, Open, a restore): the counter started again.
    if (state.deck !== lastDeck) {
      lastDeck = state.deck;
      draftedAtEdits = null;
      savedAtEdits = 0;
      lastEditsSeen = null;
      lastDraftAt = now;
    }
    const edits = await editsApplied();
    if (!Number.isFinite(edits)) return;
    if (edits !== lastEditsSeen) { lastEditsSeen = edits; lastEditAt = now; }
    if (edits === (draftedAtEdits === null ? savedAtEdits : draftedAtEdits)) return;
    let reason = "";
    if (now - lastEditAt >= POLICY.quiesceMs) reason = "quiesced";
    else if (now - lastDraftAt >= POLICY.ceilingMs) reason = "ceiling";
    else if (draftedAtEdits !== null && edits - draftedAtEdits >= POLICY.everyEdits) reason = "edits";
    if (reason) await writeDraft(reason, edits);
  } finally {
    ticking = false;
  }
}

function startScheduler() {
  if (timer) return;
  lastEditAt = Date.now();
  lastDraftAt = Date.now();
  timer = setInterval(() => { tick().catch(() => {}); }, POLICY.pollMs);
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState !== "hidden" || !timer || storageFault || !state.deck) return;
    if (lastEditsSeen !== null && lastEditsSeen !== (draftedAtEdits ?? savedAtEdits)) writeDraft("hidden", lastEditsSeen).catch(() => {});
  });
  // A completed save spends this tab's draft.
  document.addEventListener("op-saved", async () => {
    const at = await editsApplied();
    if (Number.isFinite(at)) savedAtEdits = at;
    if (draftedAtEdits === null || mySlot === null) return;
    draftedAtEdits = null;
    deleteDraft(`slot-${mySlot}`).then(refreshRecoveryBar).catch(() => {});
  });
}

function stopScheduler() {
  if (timer) clearInterval(timer);
  timer = null;
}

/* ── The recovery bar ──────────────────────────────────────────────────── */

const dismissed = new Set();

function timeLabel(ms) {
  const d = new Date(ms);
  if (!Number.isFinite(d.getTime())) return "an unknown time";
  const time = d.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
  return new Date().toDateString() === d.toDateString() ? `${time} today` : `${time} on ${d.toLocaleDateString()}`;
}

function button(label, cls, onClick) {
  const b = document.createElement("button");
  b.type = "button";
  b.className = cls;
  b.textContent = label;
  b.addEventListener("click", onClick);
  return b;
}

/// Draw the bar from the store; hidden when there is nothing to offer.
export async function refreshRecoveryBar() {
  const bar = byId("op-recovery");
  if (!bar) return [];
  let rows = [];
  try { rows = (await listDrafts()).filter((r) => r.id !== openedDraftId); } catch { rows = []; }
  const showing = rows.filter((r) => !dismissed.has(r.id));
  bar.replaceChildren();
  bar.hidden = !showing.length;
  for (const meta of showing) bar.appendChild(entryFor(meta));
  return rows;
}

function entryFor(meta) {
  const row = document.createElement("div");
  row.className = "op-recovery-row";
  row.dataset.draftId = meta.id;
  const strong = document.createElement("strong");
  strong.textContent = "Recovered unsaved work";
  const detail = document.createElement("span");
  detail.className = "op-recovery-detail";
  const ahead = typeof meta.ahead === "number" ? meta.ahead : 0;
  // `textContent`: the name came from a file.
  detail.textContent = ` ${meta.name || "Presentation"}, kept at ${timeLabel(meta.at)}, ${ahead} edit${ahead === 1 ? "" : "s"} after the last save.`;
  const note = document.createElement("span");
  note.className = "op-recovery-note";
  const newer = typeof meta.schema !== "number" || meta.schema > SCHEMA;
  note.textContent = newer ? " Written by a newer OpenPresent; download it to keep it." : " Undo history is not recovered.";
  const actions = document.createElement("span");
  actions.className = "op-recovery-actions";
  if (!newer) actions.appendChild(button("Restore", "op-btn primary", () => restore(meta)));
  actions.appendChild(button("Download", "op-btn", () => downloadDraft(meta)));
  actions.appendChild(button("Discard", "op-btn", () => discard(meta)));
  actions.appendChild(button("Not now", "op-btn", () => { dismissed.add(meta.id); refreshRecoveryBar(); }));
  row.append(strong, detail, note, actions);
  return row;
}

async function downloadDraft(meta) {
  const bytes = await readDraftBytes(meta.id);
  if (!bytes) { status(`The draft of ${meta.name} has no stored bytes`); return; }
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([bytes], { type: "application/octet-stream" }));
  a.download = meta.name || "Recovered.pptx";
  a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 10_000);
}

/// Restore: the draft opens in this tab while what is on screen is unedited, so
/// nothing is lost; otherwise in a new tab, beside it.
async function restore(meta) {
  const edits = await editsApplied();
  if (state.deck && Number.isFinite(edits) && edits !== savedAtEdits) {
    const url = new URL(location.href);
    url.searchParams.set("draft", meta.id);
    try { window.open(url.toString(), "_blank", "noopener"); } catch { /* blocked */ }
    status(`Opening the draft of ${meta.name} in a new tab`);
    return;
  }
  await loadHere(meta);
}

async function loadHere(meta) {
  const bytes = await readDraftBytes(meta.id);
  if (!bytes) { status(`The draft of ${meta.name} has no stored bytes`); return; }
  const buffer = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
  await openDeck(meta.name || "Recovered.pptx", buffer);
  if (state.deck?.name !== meta.name) return;
  openedDraftId = meta.id;
  // The restored deck has no file behind it: keep writing to the draft it came from.
  if (mySlot === null && leasePromise === null) claimSlot();
  status(`Recovered ${meta.name} from ${timeLabel(meta.at)}. Undo history is not recovered`);
  await refreshRecoveryBar();
}

function discard(meta) {
  showModal("Discard the recovered work?", (body) => {
    const p = document.createElement("p");
    p.className = "op-confirm-text";
    p.textContent = `The unsaved work in ${meta.name || "Presentation"} from ${timeLabel(meta.at)} is deleted from this browser and cannot be brought back. The presentation open here is not affected.`;
    const actions = document.createElement("div");
    actions.className = "op-confirm-actions";
    actions.append(
      button("Cancel", "op-btn", closeModal),
      button("Discard", "op-btn primary", async () => {
        closeModal();
        try { await deleteDraft(meta.id); } catch (e) { status(`Could not discard the draft: ${e.message}`); return; }
        const slot = slotOf(meta.id);
        if (slot !== null) reservedSlots.delete(slot);
        status(`Discarded the draft of ${meta.name}`);
        await refreshRecoveryBar();
      }),
    );
    body.append(p, actions);
  });
}

/* ── Boot ──────────────────────────────────────────────────────────────── */

/// Start autosave for this page. Never throws: a failure here must not stop the editor.
export async function initDrafts() {
  // Embedded in another page, the host owns the document: no copy in this browser.
  if (window.top !== window.self) return { autosave: false, why: "this editor is part of another page" };
  const asked = new URLSearchParams(location.search).get("draft");
  if (asked) {
    try {
      const meta = (await listDrafts()).find((r) => r.id === asked);
      if (meta) await loadHere(meta);
      else status("That draft is no longer in this browser");
    } catch (e) {
      status(`Could not read the draft: ${e.message}`);
    }
  }
  try {
    const offered = await refreshRecoveryBar();
    reservedSlots = new Set(offered.map((r) => slotOf(r.id)).filter((n) => n !== null));
  } catch { /* the bar stays hidden */ }
  try {
    await openDb();
  } catch (e) {
    faultAutosave(`Autosave off: this browser is not storing drafts (${e.message})`);
    return { autosave: false, why: storageFault };
  }
  startScheduler();
  return { autosave: true, why: "" };
}

/// For a check in a browser: what this module holds.
export function draftState() {
  return { fault: storageFault, slot: mySlot, reserved: [...reservedSlots], draftedAtEdits, savedAtEdits, reason: lastReason, running: !!timer, openedDraftId };
}
