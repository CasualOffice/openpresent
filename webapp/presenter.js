// Presenter View (`VS-07`): the second window of a slide show, as PowerPoint's.
// The current slide, the next one, the speaker notes, a timer and the slide count.
//
// The editor's show (`editor.show.js`) owns everything. It calls `update(view)` here
// each time the slide changes, and this window hands keys and buttons back through
// `window.opener.openpresentShow`. No model and no engine here: the pictures are
// `blob:` URLs the editor drew, and the notes, document text, are set with
// `textContent` only.

const byId = (id) => document.getElementById(id);
const bridge = () => {
  try { return window.opener && !window.opener.closed ? window.opener.openpresentShow : null; } catch { return null; }
};

let started = Date.now();
let paused = 0;          // when the timer was paused, or 0
let blank = null;

function clock(ms) {
  const s = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return `${h}:${String(m).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
}

function tick() {
  byId("pv-timer").textContent = clock((paused || Date.now()) - started);
  byId("pv-clock").textContent = new Date().toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
}

function setPicture(id, url, alt) {
  const img = byId(id);
  if (url) { img.src = url; img.alt = alt; img.hidden = false; }
  else { img.hidden = true; img.removeAttribute("src"); }
}

/// What the show calls on each change: first at once with the count (`full`
/// false), then again with the pictures and notes (`full` true).
function update(v) {
  if (v.started && v.started !== update.started) { update.started = v.started; started = v.started; paused = 0; }
  blank = v.blank;
  byId("pv-count").textContent = v.ended ? `End of slide show (${v.count} slides)` : `Slide ${v.index + 1} of ${v.count}`;
  byId("pv-next-label").textContent = `Next: ${v.nextLabel}`;
  const b = byId("pv-blank");
  b.hidden = !v.blank;
  b.className = `pv-blank is-${v.blank || "none"}`;
  byId("pv-black").textContent = v.blank === "black" ? "Unblack screen" : "Black screen";
  const e = byId("pv-effect");
  e.hidden = !v.effect;
  e.textContent = v.effect ? `The ${v.effect} transition is shown as a fade.` : "";
  const n = byId("pv-notes");
  if (v.full) {
    setPicture("pv-current", v.current, `Slide ${v.index + 1}`);
    setPicture("pv-next", v.next, v.nextLabel);
    n.textContent = v.ended ? "" : v.notes || "No notes for this slide.";
    n.classList.toggle("is-empty", !v.notes);
    n.scrollTop = 0;
  } else if (update.index !== v.index) {
    // Another slide: never leave its predecessor's notes up while these load.
    n.textContent = "";
  }
  update.index = v.index;
  tick();
}

globalThis.openpresentPresenter = { update };

document.addEventListener("keydown", (e) => {
  if (e.ctrlKey || e.metaKey || e.altKey) return;
  // Space and Enter on a focused button press it.
  if (e.target.tagName === "BUTTON" && (e.key === " " || e.key === "Enter")) return;
  const b = bridge();
  if (!b) return;
  e.preventDefault();
  b.key(e.key);
});

byId("pv-prev").addEventListener("click", () => bridge()?.previous());
byId("pv-next-btn").addEventListener("click", () => bridge()?.next());
byId("pv-end").addEventListener("click", () => bridge()?.end());
byId("pv-black").addEventListener("click", () => bridge()?.key("b"));
byId("pv-pause").addEventListener("click", () => {
  if (paused) { started += Date.now() - paused; paused = 0; }
  else paused = Date.now();
  byId("pv-pause").textContent = paused ? "Resume" : "Pause";
  tick();
});
byId("pv-reset").addEventListener("click", () => { started = Date.now(); if (paused) paused = started; tick(); });

setInterval(tick, 500);
tick();
bridge()?.ready();
