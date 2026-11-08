// Who comments are signed as (`UX-011`). There is no account in this browser to read a
// name from, so it is a local setting: asked once inside the Comments pane, never in a
// browser dialog, and changed from File > Options. As OpenCalc's `commentAuthor`, an
// empty name is allowed and signs as "Author" rather than blocking the comment.
//
// The author's id is a GUID minted here once and kept, so the deck's author list names
// one person once however often they comment. Randomness is the host's to supply; the
// engine never samples it.

const KEY = "openpresent.author";

function read() {
  try {
    const v = JSON.parse(localStorage.getItem(KEY) || "null");
    if (v && typeof v.id === "string" && typeof v.name === "string") return v;
  } catch { /* storage off or damaged: a fresh identity */ }
  return null;
}

let me = read();

/// A GUID in braces, upper case, as PowerPoint writes comment and author ids.
export function newGuid() {
  return `{${crypto.randomUUID().toUpperCase()}}`;
}

/// Whether the user has given a name yet.
export const hasName = () => !!me && me.name.trim() !== "";

/// `{ id, name }`, minting the id on first use.
export function author() {
  if (!me) {
    me = { id: newGuid(), name: "" };
    try { localStorage.setItem(KEY, JSON.stringify(me)); } catch { /* kept for this tab */ }
  }
  return me;
}

export function setName(name) {
  const a = author();
  a.name = String(name || "").trim().slice(0, 255);
  try { localStorage.setItem(KEY, JSON.stringify(a)); } catch { /* kept for this tab */ }
  document.dispatchEvent(new CustomEvent("op-author-changed"));
}

/// The time now, as `xsd:dateTime` in UTC, which the engine is given and never reads.
export function now() { return new Date().toISOString(); }
