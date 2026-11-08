// Recent presentations (`PAR-19`), as PowerPoint's File > Open and New list them.
//
// A browser cannot reopen a file by its path, so a deck opened from this device is
// kept, bytes and all, in this browser's own IndexedDB, and opens again from there:
// nothing is uploaded and nothing here fetches. The store is OpenCalc's draft store's
// shape (`webapp/editor.drafts.js` at `b23c67c`, copied into our `editor.drafts.js`):
// one object store for the list, one for the bytes, so listing never reads a deck.
// OpenCalc has no recent list of its own to copy.
//
// The list keeps every pinned deck and the twelve most recent others, each at most
// 50 MB; reopening a deck moves it to the top. Pin, Unpin and Remove act on one row.
// Names are document text, shown with `textContent` only.

import { state, status, openDeck } from "./editor.core.js";
import { icon } from "./ribbon.js";

const DB_NAME = "openpresent-recent";
const DB_VERSION = 1;
/// How many unpinned decks the list keeps.
export const MAX_RECENT = 12;
/// The largest deck kept, in bytes.
export const MAX_RECENT_BYTES = 50 * 1024 * 1024;

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
    req.onerror = () => reject(req.error || new Error("IndexedDB would not open"));
  });
  dbPromise.catch(() => { dbPromise = null; });
  return dbPromise;
}

function tx(db, stores, mode, fn) {
  return new Promise((resolve, reject) => {
    const t = db.transaction(stores, mode);
    let out;
    t.oncomplete = () => resolve(out);
    t.onerror = () => reject(t.error);
    t.onabort = () => reject(t.error || new Error("aborted"));
    out = fn(t);
  });
}

function all(store) {
  return new Promise((resolve, reject) => {
    const r = store.getAll();
    r.onsuccess = () => resolve(r.result || []);
    r.onerror = () => reject(r.error);
  });
}

/// Every kept deck: pinned first, then the most recently opened.
export async function listRecent() {
  const db = await openDb();
  let rows = [];
  await tx(db, ["meta"], "readonly", (t) => all(t.objectStore("meta")).then((r) => { rows = r; }));
  return rows
    .filter((r) => r && typeof r.id === "string" && typeof r.name === "string")
    .sort((a, b) => (b.pinned ? 1 : 0) - (a.pinned ? 1 : 0) || b.at - a.at);
}

/// Keep `bytes`, opened as `name`, at the top of the list. One row a name: opening
/// the same name again replaces its bytes. Never throws.
export async function remember(name, bytes) {
  try {
    if (!(bytes instanceof Uint8Array) || bytes.length > MAX_RECENT_BYTES) return;
    const db = await openDb();
    const rows = await listRecent();
    const same = rows.find((r) => r.name === name);
    const id = same ? same.id : `r-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    const meta = { id, name, at: Date.now(), size: bytes.length, pinned: !!same?.pinned };
    const drop = rows.filter((r) => r.id !== id && !r.pinned).slice(MAX_RECENT - 1).map((r) => r.id);
    await tx(db, ["meta", "bytes"], "readwrite", (t) => {
      t.objectStore("meta").put(meta);
      t.objectStore("bytes").put({ id, bytes });
      for (const d of drop) { t.objectStore("meta").delete(d); t.objectStore("bytes").delete(d); }
    });
  } catch { /* the list is a convenience: a full or missing store loses nothing */ }
}

async function bytesOf(id) {
  const db = await openDb();
  let rec = null;
  await tx(db, ["bytes"], "readonly", (t) => {
    const r = t.objectStore("bytes").get(id);
    r.onsuccess = () => { rec = r.result || null; };
  });
  return rec && rec.bytes instanceof Uint8Array ? rec.bytes : null;
}

async function setPinned(id, pinned) {
  const db = await openDb();
  await tx(db, ["meta"], "readwrite", (t) => {
    const s = t.objectStore("meta");
    const r = s.get(id);
    r.onsuccess = () => { if (r.result) s.put({ ...r.result, pinned }); };
  });
}

async function removeRecent(id) {
  const db = await openDb();
  await tx(db, ["meta", "bytes"], "readwrite", (t) => {
    t.objectStore("meta").delete(id);
    t.objectStore("bytes").delete(id);
  });
}

/// Open a kept deck, as if picked again.
export async function openRecent(id) {
  const rows = await listRecent();
  const meta = rows.find((r) => r.id === id);
  const bytes = meta && await bytesOf(id);
  if (!bytes) { status("That presentation is no longer kept in this browser"); return; }
  await openDeck(meta.name, bytes.slice().buffer);
  if (state.deck?.name === meta.name) remember(meta.name, bytes);
}

/* ── The list on File > Open and New ────────────────────────────────────── */

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

function when(ms) {
  const d = new Date(ms);
  if (!Number.isFinite(d.getTime())) return "";
  return new Date().toDateString() === d.toDateString()
    ? d.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" })
    : d.toLocaleDateString();
}

async function draw(page, close) {
  let host = page.querySelector(".op-recent");
  if (!host) {
    host = h("div", "rb-section op-recent");
    page.appendChild(host);
  }
  let rows;
  try { rows = await listRecent(); } catch { rows = []; }
  host.replaceChildren(h("h2", "rb-section-title", "Recent"));
  if (!rows.length) {
    host.appendChild(h("p", "op-recent-empty", "Presentations you open appear here. They are kept in this browser only."));
    return;
  }
  const list = h("div", "rb-page-list");
  for (const r of rows) {
    const row = h("div", "op-recent-row");
    const open = h("button", "rb-page-item op-recent-open");
    open.type = "button";
    const text = h("span", "rb-item-text");
    text.append(h("span", "rb-item-label", r.name), h("span", "rb-item-desc", `${when(r.at)}${r.pinned ? " · Pinned" : ""}`));
    open.append(icon("document", 20), text);
    open.addEventListener("click", () => { close(); openRecent(r.id); });
    const pin = h("button", "op-recent-act", r.pinned ? "Unpin" : "Pin");
    pin.type = "button";
    pin.setAttribute("aria-label", `${r.pinned ? "Unpin" : "Pin"} ${r.name}`);
    pin.addEventListener("click", async () => { await setPinned(r.id, !r.pinned); draw(page, close); });
    const del = h("button", "op-recent-act", "Remove");
    del.type = "button";
    del.setAttribute("aria-label", `Remove ${r.name} from the list`);
    del.addEventListener("click", async () => { await removeRecent(r.id); draw(page, close); });
    row.append(open, pin, del);
    list.appendChild(row);
  }
  host.appendChild(list);
}

/// Draw the list whenever File > Open or New shows; `close` leaves the File page.
export function initRecent(close) {
  document.addEventListener("op-backstage-page", (e) => {
    const { id, page } = e.detail || {};
    if ((id === "open" || id === "new") && page) draw(page, close);
  });
}
