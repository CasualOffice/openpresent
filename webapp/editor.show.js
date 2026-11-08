// The slide show and Presenter View (`VS-07`): presenting as PowerPoint does.
//
// From Beginning (F5) and From Current Slide (Shift+F5) fill the screen with one
// slide; Presenter View (Alt+F5) adds a second window with the current and next
// slide, the speaker notes, a timer and the slide count. The keys are
// PowerPoint's: next by click, Space, Enter, N, Right, Down or Page Down; back by
// P, Backspace, Left, Up or Page Up; a number then Enter jumps to that slide; B
// or `.` blacks the screen and W or `,` whites it; Esc or `-` ends. Hidden slides
// (`@show="0"`) are skipped, as PowerPoint skips them, unless jumped to by number.
//
// Transitions (`VS-10`, `UX-004`): fade, push, wipe, split, cover, uncover, zoom and
// reveal play as themselves in the direction the file states, over the transition's
// own duration, and a cut changes at once. Any other effect is played as a fade, and
// when the show ends the status bar names which (`VS-07`). Advance Slide After moves
// on by itself once the time has passed, playing the slide's clicks first; with
// On Mouse Click off, a click on the slide does not advance it (keys still do).
//
// Animations (`VS-10`): a slide's main sequence plays click by click. A shape whose
// first effect is an entrance starts hidden; each click plays its effects, step by
// step for After Previous. Appear and Disappear are instant, Fade and Fade Out fade
// the shape's part of the slide, Fly In moves it up from below, Pulse grows and
// shrinks it. The engine draws each state with the right shapes hidden; a moving
// shape is its box cut from that drawing, unrotated. Effects this build does not
// write play as the change of state at once, and are named at the end. Going back
// undoes a click at once.
//
// The deck stays in the engine. This module asks the worker for pictures, the
// show plan and the notes, and keeps no model. The notes, document text, reach
// the presenter window through `textContent` only (`presenter.js`).

import { state, request, status, changed } from "./editor.core.js";

const byId = (id) => document.getElementById(id);
const MAX_WIDTH = 4096;
const SHOW_PRIORITY = 6;
const PRESENTER_PRIORITY = 4;

/// The show in progress, or `null`. `index` past the last slide is the black
/// "end of slide show" screen.
let show = null;
let token = 0;
let presenter = null;   // the Presenter View window, while it is open

const dpr = () => Math.min(window.devicePixelRatio || 1, 3);
const aspect = () => state.deck.height / state.deck.width;

export function showing() { return show !== null; }

async function picture(index, width, priority) {
  try {
    const r = await request({ op: "png", index, width: Math.max(1, Math.min(MAX_WIDTH, Math.round(width))) }, { priority });
    return URL.createObjectURL(new Blob([r.png], { type: "image/png" }));
  } catch (e) {
    if (e.message !== "superseded") status(`Slide ${index + 1} could not be drawn: ${e.message}`);
    return null;
  }
}

/* ── The plan ───────────────────────────────────────────────────────────── */

/// Which slides are hidden, and each one's transition, from the engine.
async function loadPlan() {
  const count = state.deck.count;
  const plan = Array.from({ length: count }, () => ({ hidden: false, transition: false, asIs: true, ms: 0, effect: "" }));
  try {
    const r = await request({ op: "plan" }, { priority: 9 });
    for (let i = 0; i < count; i++) {
      const flags = r.plan[i * 2] | 0;
      const after = r.advance ? r.advance[i * 2 + 1] | 0 : 0;
      plan[i] = {
        hidden: !!(flags & 1), transition: !!(flags & 2), asIs: !!(flags & 4), ms: r.plan[i * 2 + 1] | 0, effect: r.effects?.[i] || "", dir: r.directions?.[i] || "",
        onClick: r.advance ? r.advance[i * 2] !== 0 : true, after: after > 0 ? after - 1 : null,
      };
    }
  } catch (e) {
    // Without the plan the show still runs: every slide, no transitions.
    status(`The slide show could not read which slides are hidden: ${e.message}`);
  }
  return plan;
}

const visible = (i) => show && i >= 0 && i < show.plan.length && !show.plan[i].hidden;

/// The next slide shown after `i` in direction `d`, skipping hidden ones; the
/// end screen (`count`) past the last, and `-1` before the first.
function step(i, d) {
  const n = show.plan.length;
  for (let j = i + d; j >= 0 && j < n; j += d) if (visible(j)) return j;
  return d > 0 ? n : -1;
}

/* ── Drawing ────────────────────────────────────────────────────────────── */

const layers = () => [byId("show-img"), byId("show-fade")];

function clearLayer(img) {
  img.hidden = true;
  img.classList.remove("is-in");
  img.removeAttribute("src");
}

/// The bottom picture becomes `url`; the one it showed before is released.
function setBase(url, alt) {
  const base = byId("show-img");
  if (show.baseUrl && show.baseUrl !== url) URL.revokeObjectURL(show.baseUrl);
  show.baseUrl = url;
  base.src = url;
  base.alt = alt;
  base.hidden = false;
}

/// Drop a fade in progress, and release its picture.
function clearOver() {
  if (show.overUrl && show.overUrl !== show.baseUrl) URL.revokeObjectURL(show.overUrl);
  show.overUrl = null;
  clearLayer(byId("show-fade"));
}

/// Show `show.index`; `fade` plays the slide's transition into it.
async function draw(fade) {
  const box = byId("show");
  const [base, over] = layers();
  const t = ++token;
  box.classList.toggle("is-end", show.index >= show.plan.length);
  if (show.index >= show.plan.length) {
    clearOver();
    if (show.baseUrl) URL.revokeObjectURL(show.baseUrl);
    show.baseUrl = null;
    clearLayer(base);
    updatePresenter();
    return;
  }
  const index = show.index;
  const w = Math.min(box.clientWidth || window.innerWidth, (box.clientHeight || window.innerHeight) / aspect());
  show.width = w * dpr();
  // A slide entered forward starts before its first click; backward, after its last.
  const kept = show.anim && show.anim.index === index ? show.anim.at : 0;
  const anim = await loadAnimations(index);
  if (t !== token || !show) return;
  show.anim = anim;
  show.playing = false;
  if (anim) {
    anim.index = index;
    const n = anim.clicks.length;
    anim.at = fade || show.entering === "start" ? 0 : show.entering === "keep" ? Math.min(kept, n) : n;
  }
  show.entering = null;
  const url = anim ? await stateAt(index, anim, anim.at) : await picture(index, show.width, SHOW_PRIORITY);
  if (!url || t !== token || !show) { if (url) URL.revokeObjectURL(url); return; }
  const p = show.plan[index];
  if (p.transition && !p.asIs && p.effect) show.standIns.add(p.effect);
  const alt = `Slide ${index + 1}`;
  clearOver();
  const autoStart = () => { if (anim && anim.at === 0 && anim.auto && t === token && show) playClick(); };
  if (fade && p.transition && p.ms > 0 && !base.hidden) {
    show.overUrl = url;
    over.src = url;
    over.alt = alt;
    over.hidden = false;
    const done = () => {
      if (!show || show.overUrl !== url) return;
      setBase(url, alt);
      show.overUrl = null;
      clearLayer(over);
      over.style.transitionDuration = "";
      autoStart();
    };
    const motion = p.asIs ? transitionFrames(p.effect, p.dir) : null;
    if (motion) {
      // The new slide moves in, or is uncovered, over the old; for uncover the old
      // slide is drawn on top and moves away.
      over.style.transitionDuration = "0ms";
      over.classList.add("is-in");
      const opts = { duration: p.ms, easing: "ease-in-out", fill: "forwards" };
      if (motion.baseOnTop) base.style.zIndex = "2";
      const a = over.animate(motion.over, opts);
      const b = motion.base ? base.animate(motion.base, opts) : null;
      const finish = () => {
        if (b) base.style.visibility = "hidden";
        a.cancel();
        b?.cancel();
        base.style.zIndex = "";
        done();
        if (b) base.decode().catch(() => {}).then(() => { base.style.visibility = ""; });
      };
      let finished = false;
      const once = () => { if (!finished) { finished = true; finish(); } };
      a.finished.then(once, () => {});
      setTimeout(once, p.ms + 200);
    } else {
      // A fade, or the fade a stand-in plays: the new slide fades in over the old.
      over.style.transitionDuration = `${p.ms}ms`;
      void over.offsetWidth;          // start from transparent
      over.classList.add("is-in");
      over.addEventListener("transitionend", done, { once: true });
      setTimeout(done, p.ms + 100);   // a fade the browser skipped still ends
    }
  } else {
    setBase(url, alt);
    autoStart();
  }
  arm(p.ms);
  updatePresenter();
}

/// Advance Slide After (`UX-004`): once the slide's time has passed, play its next
/// click, or move on when none is left. `lead` is the transition's own time.
function arm(lead = 0) {
  clearTimeout(show.timer);
  const p = show.plan[show.index];
  if (!p || p.after == null) return;
  const index = show.index;
  show.timer = setTimeout(() => {
    if (!show || show.index !== index || show.blank) return;
    if (show.playing) { arm(); return; }
    if (show.anim && show.anim.at < show.anim.clicks.length) { playClick().then(() => { if (show && show.index === index) arm(); }); return; }
    next();
  }, lead + p.after);
}

/// Where a slide comes from for a direction token: `l` moves left, so it starts at
/// the right; corners combine two.
const FROM = { l: [100, 0], r: [-100, 0], u: [0, 100], d: [0, -100], lu: [100, 100], ru: [-100, 100], ld: [100, -100], rd: [-100, -100] };

/// Keyframes for the effects played as themselves, by the direction the file
/// states; `null` for anything else, which plays the fade. Also the editor's
/// Preview (`UX-004`).
export function transitionFrames(effect, dir) {
  const d = (dir || "").split(" ");
  if (effect === "fade" && d[0] === "black") {
    return { over: [{ opacity: 0 }, { opacity: 0, offset: 0.5 }, { opacity: 1 }], base: [{ opacity: 1 }, { opacity: 0, offset: 0.5 }, { opacity: 0 }] };
  }
  if (effect === "cover") {
    const [x, y] = FROM[d[0]] || FROM.l;
    return { over: [{ transform: `translate(${x}%, ${y}%)` }, { transform: "translate(0, 0)" }] };
  }
  if (effect === "pull") {
    const [x, y] = FROM[d[0]] || FROM.l;
    return { over: [{ opacity: 1 }, { opacity: 1 }], base: [{ transform: "translate(0, 0)" }, { transform: `translate(${-x}%, ${-y}%)` }], baseOnTop: true };
  }
  if (effect === "zoom") {
    const from = d[0] === "in" ? 0.3 : 1.6;
    return { over: [{ opacity: 0, transform: `scale(${from})` }, { opacity: 1, transform: "scale(1)" }] };
  }
  if (effect === "reveal") {
    const x = d[0] === "r" ? -12 : 12;
    if (d[1] === "black") {
      return {
        over: [{ opacity: 0, transform: `translate(${x}%, 0)` }, { opacity: 0, transform: `translate(${x}%, 0)`, offset: 0.5 }, { opacity: 1, transform: "translate(0, 0)" }],
        base: [{ opacity: 1 }, { opacity: 0, offset: 0.5 }, { opacity: 0 }],
      };
    }
    return { over: [{ opacity: 0, transform: `translate(${x}%, 0)` }, { opacity: 1, transform: "translate(0, 0)" }], base: [{ opacity: 1 }, { opacity: 0 }] };
  }
  if (effect === "push") {
    const [x, y] = { l: [100, 0], r: [-100, 0], u: [0, 100], d: [0, -100] }[d[0]] || [100, 0];
    return {
      over: [{ transform: `translate(${x}%, ${y}%)` }, { transform: "translate(0, 0)" }],
      base: [{ transform: "translate(0, 0)" }, { transform: `translate(${-x}%, ${-y}%)` }],
    };
  }
  if (effect === "wipe") {
    const from = { l: "inset(0 0 0 100%)", r: "inset(0 100% 0 0)", u: "inset(100% 0 0 0)", d: "inset(0 0 100% 0)" }[d[0]] || "inset(0 0 0 100%)";
    return { over: [{ clipPath: from }, { clipPath: "inset(0 0 0 0)" }] };
  }
  if (effect === "split") {
    const vert = d[0] === "vert";
    const closed = vert ? "inset(0 50% 0 50%)" : "inset(50% 0 50% 0)";
    const edges = vert ? "polygon(0 0, 0 0, 0 100%, 0 100%, 100% 100%, 100% 100%, 100% 0, 100% 0)" : "polygon(0 0, 100% 0, 100% 0, 0 0, 0 100%, 100% 100%, 100% 100%, 0 100%)";
    if (d[1] === "in") {
      // In: the new slide closes in from both edges over the old.
      const full = vert ? "polygon(0 0, 50% 0, 50% 100%, 0 100%, 100% 100%, 50% 100%, 50% 0, 100% 0)" : "polygon(0 0, 100% 0, 100% 50%, 0 50%, 0 100%, 100% 100%, 100% 50%, 0 50%)";
      return { over: [{ clipPath: edges }, { clipPath: full }] };
    }
    return { over: [{ clipPath: closed }, { clipPath: "inset(0 0 0 0)" }] };
  }
  return null;
}

/* ── Animations (`VS-10`) ───────────────────────────────────────────────── */

const ENTRANCE = 1, EXIT = 3;
const KIND = { appear: 0, fade: 1, fly: 2, pulse: 3, disappear: 4, fadeOut: 5 };

/// Slide `index`'s main sequence as clicks, each a list of steps, each a list of
/// effects; `null` for a slide with none, or one the engine cannot read.
async function loadAnimations(index) {
  let v;
  try {
    v = (await request({ op: "animate", fn: "slide_animations", args: [index] }, { priority: SHOW_PRIORITY })).value;
  } catch {
    return null;
  }
  const effects = [];
  for (let i = 0; i + 8 < v.length; i += 9) {
    effects.push({ node: v[i], kind: v[i + 1], cls: v[i + 2], start: v[i + 3], ms: v[i + 4], box: [v[i + 5], v[i + 6], v[i + 7], v[i + 8]] });
  }
  const shown = effects.filter((e) => e.node >= 0);
  if (!shown.length) return null;
  const clicks = [];
  for (const e of shown) {
    const last = clicks[clicks.length - 1];
    if (e.start === 0 || !last) clicks.push([[e]]);
    else if (e.start === 1) last[last.length - 1].push(e);
    else last.push([e]);
  }
  // Hidden before anything plays: each shape whose first effect is an entrance.
  const initial = new Set();
  const seen = new Set();
  for (const e of shown) {
    if (seen.has(e.node)) continue;
    seen.add(e.node);
    if (e.cls === ENTRANCE) initial.add(e.node);
  }
  return { clicks, initial, auto: shown[0].start !== 0, at: 0 };
}

function applyEffect(hidden, e) {
  if (e.cls === ENTRANCE) hidden.delete(e.node);
  else if (e.cls === EXIT) hidden.add(e.node);
}

/// The shapes hidden once the first `clicks` clicks have played.
function hiddenAfter(anim, clicks) {
  const hidden = new Set(anim.initial);
  for (const c of anim.clicks.slice(0, clicks)) for (const s of c) for (const e of s) applyEffect(hidden, e);
  return hidden;
}

function drawHiding(index, hidden) {
  const width = Math.max(1, Math.min(MAX_WIDTH, Math.round(show.width)));
  return request({ op: "png-hiding", index, width, nodes: [...hidden] }, { priority: SHOW_PRIORITY })
    .then((r) => URL.createObjectURL(new Blob([r.png], { type: "image/png" })), () => null);
}

/// The slide as it stands after `clicks` clicks.
function stateAt(index, anim, clicks) {
  return drawHiding(index, hiddenAfter(anim, clicks));
}

/// A shape's part of a slide picture, placed exactly over the slide shown.
function part(url, box) {
  const host = byId("show");
  const r = byId("show-img").getBoundingClientRect();
  const o = host.getBoundingClientRect();
  const img = document.createElement("img");
  img.className = "show-part";
  img.alt = "";
  img.style.visibility = "hidden";   // shown when its effect starts
  img.src = url;
  Object.assign(img.style, { left: `${r.left - o.left}px`, top: `${r.top - o.top}px`, width: `${r.width}px`, height: `${r.height}px` });
  const [x, y, cx, cy] = box;
  const W = state.deck.width, H = state.deck.height;
  if (cx > 0 || cy > 0) {
    const pct = (v, of) => `${Math.max(0, Math.min(100, (v / of) * 100))}%`;
    img.style.clipPath = `inset(${pct(y, H)} ${pct(W - x - cx, W)} ${pct(H - y - cy, H)} ${pct(x, W)})`;
    img.style.transformOrigin = `${((x + cx / 2) / W) * 100}% ${((y + cy / 2) / H) * 100}%`;
  }
  host.appendChild(img);
  return img;
}

/// Play one step: its shapes leave the drawing and come back moving, as their effect.
async function playStep(index, before, hidden, effects) {
  const after = new Set(hidden);
  for (const e of effects) applyEffect(after, e);
  const moving = new Set(hidden);
  for (const e of effects) moving.add(e.node);
  const [mid, end] = await Promise.all([drawHiding(index, moving), drawHiding(index, after)]);
  if (!show || !mid || !end) { for (const u of [mid, end]) if (u) URL.revokeObjectURL(u); return after; }
  const parts = [];
  for (const e of effects) {
    if (e.kind === KIND.disappear || (e.cls === EXIT && e.kind !== KIND.fadeOut)) continue;
    parts.push([e, part(e.cls === EXIT ? before : end, e.box)]);
  }
  // Every part is decoded, and starts where its effect starts, before the drawing
  // without them goes under, so nothing blinks.
  await Promise.all(parts.map(([, img]) => img.decode().catch(() => {})));
  const plays = [];
  for (const [e, img] of parts) {
    img.style.visibility = "";
    const opts = { duration: Math.max(1, e.ms), easing: "ease-out", fill: "forwards" };
    if (e.kind === KIND.fade) plays.push(img.animate([{ opacity: 0 }, { opacity: 1 }], opts).finished);
    else if (e.kind === KIND.fadeOut) plays.push(img.animate([{ opacity: 1 }, { opacity: 0 }], opts).finished);
    else if (e.kind === KIND.fly) {
      const dy = ((state.deck.height - e.box[1]) / state.deck.height) * 100;
      plays.push(img.animate([{ transform: `translateY(${dy}%)` }, { transform: "translateY(0)" }], opts).finished);
    } else if (e.kind === KIND.pulse) {
      plays.push(img.animate([{ transform: "scale(1)" }, { transform: "scale(1.05)" }, { transform: "scale(1)" }], { ...opts, easing: "ease-in-out" }).finished);
    } else if (e.kind !== KIND.appear) {
      show.standIns.add("animations other than Appear, Fade, Fly In, Pulse, Disappear and Fade Out");
    }
  }
  show.baseUrl = null;            // `before` is still drawn from; released below
  setBase(mid, `Slide ${index + 1}`);
  const longest = Math.max(0, ...effects.map((e) => e.ms));
  await Promise.race([Promise.all(plays), new Promise((r) => setTimeout(r, longest + 150))]).catch(() => {});
  if (before) URL.revokeObjectURL(before);
  if (!show) { URL.revokeObjectURL(end); return after; }
  setBase(end, `Slide ${index + 1}`);
  for (const [, img] of parts) img.remove();
  return after;
}

/// Play the next click of the slide shown, and any click after it that starts by itself.
async function playClick() {
  const anim = show.anim;
  if (!anim || show.playing || anim.at >= anim.clicks.length) return;
  show.playing = true;
  const index = show.index;
  const t = token;
  let hidden = hiddenAfter(anim, anim.at);
  for (const effects of anim.clicks[anim.at]) {
    if (!show || t !== token) break;
    hidden = await playStep(index, show.baseUrl, hidden, effects);
  }
  if (!show) return;
  show.playing = false;
  if (t !== token) return;
  anim.at++;
  updatePresenter();
  const next = anim.clicks[anim.at];
  if (next && next[0][0].start !== 0) playClick();
}

/// Back one click: the slide as it stood before it, at once.
async function unplayClick() {
  const anim = show.anim;
  anim.at = Math.max(0, anim.at - 1);
  const t = ++token;
  const url = await stateAt(show.index, anim, anim.at);
  if (!url || t !== token || !show) { if (url) URL.revokeObjectURL(url); return; }
  for (const p of byId("show").querySelectorAll(".show-part")) p.remove();
  clearOver();
  setBase(url, `Slide ${show.index + 1}`);
}

function setBlank(colour) {
  show.blank = colour;
  const box = byId("show");
  box.classList.toggle("is-black", colour === "black");
  box.classList.toggle("is-white", colour === "white");
  updatePresenter();
}

/* ── Moving ─────────────────────────────────────────────────────────────── */

function next() {
  if (show.index >= show.plan.length) { end(); return; }
  if (show.playing) return;
  // A click plays the slide's next animation before it moves on (`VS-10`).
  if (show.anim && show.anim.at < show.anim.clicks.length) { playClick(); return; }
  show.index = step(show.index, 1);
  draw(true);
}

function previous() {
  if (show.index < show.plan.length && show.anim && show.anim.at > 0 && !show.playing) { unplayClick(); return; }
  const j = step(Math.min(show.index, show.plan.length), -1);
  if (j < 0) return;
  show.index = j;
  draw(false);
}

/// Jump to slide `n`, counted from 1, hidden or not, as PowerPoint does.
function jump(n) {
  if (!(n >= 1 && n <= show.plan.length)) return;
  show.index = n - 1;
  draw(true);
}

/// Start the slide show at slide `from` (from 0). From the beginning, the first
/// slide that is not hidden; from the current slide, that slide even if hidden.
export async function start(from, { presenterView = false } = {}) {
  if (!state.deck || show) return;
  // Opened before anything is awaited, so the browser counts it as the user's click.
  if (presenterView) openPresenter();
  const box = byId("show");
  show = { plan: [], index: 0, baseUrl: null, overUrl: null, blank: null, typed: "", standIns: new Set(), started: Date.now(), from, anim: null, playing: false, entering: "start", width: 0 };
  box.hidden = false;
  box.focus();
  changed();
  try { await box.requestFullscreen?.(); } catch { /* shown as a window-filling layer instead */ }
  const plan = await loadPlan();
  if (!show) return;
  show.plan = plan;
  show.index = from === 0 && plan.length && plan[0].hidden ? step(0, 1) : Math.min(from, plan.length);
  draw(false);
}

/// End the show, back to Normal view at the slide last shown.
export function end() {
  if (!show) return;
  const last = Math.min(show.index, show.plan.length - 1);
  const standIns = [...show.standIns].sort();
  const urls = [show.baseUrl, show.overUrl];
  clearTimeout(show.timer);
  show = null;
  token++;
  const box = byId("show");
  box.hidden = true;
  box.classList.remove("is-end", "is-black", "is-white");
  for (const img of layers()) clearLayer(img);
  for (const p of box.querySelectorAll(".show-part")) p.remove();
  for (const u of new Set(urls)) if (u) URL.revokeObjectURL(u);
  if (document.fullscreenElement) document.exitFullscreen?.().catch(() => {});
  closePresenter();
  onEnd(Math.max(0, last));
  changed();
  // What the show stood in for, so it is never silent (`VS-07`).
  if (standIns.length) status(`Shown as a simple fade, not drawn yet: ${standIns.join(", ")}`);
}

let onEnd = () => {};

/* ── Keys ───────────────────────────────────────────────────────────────── */

const NEXT = new Set(["ArrowRight", "ArrowDown", "PageDown", " ", "n", "N"]);
const BACK = new Set(["ArrowLeft", "ArrowUp", "PageUp", "Backspace", "p", "P"]);

/// One key, from the show itself or the presenter window. True if it was the show's.
export function key(e) {
  if (!show) return false;
  const k = e.key;
  if (k === "Escape" || k === "-") { end(); return true; }
  if (!show.plan.length) return true;   // the plan is still loading
  if (/^[0-9]$/.test(k)) { show.typed = (show.typed + k).slice(-6); return true; }
  if (k === "Enter" && show.typed) { const n = Number(show.typed); show.typed = ""; if (show.blank) setBlank(null); jump(n); return true; }
  show.typed = "";
  if (k === "b" || k === "B" || k === ".") { setBlank(show.blank === "black" ? null : "black"); return true; }
  if (k === "w" || k === "W" || k === ",") { setBlank(show.blank === "white" ? null : "white"); return true; }
  // On a blank screen any other key brings the slide back, and does nothing else.
  if (show.blank && (NEXT.has(k) || BACK.has(k) || k === "Enter" || k === "Home" || k === "End")) { setBlank(null); return true; }
  if (NEXT.has(k) || k === "Enter") { next(); return true; }
  if (BACK.has(k)) { previous(); return true; }
  if (k === "Home") { show.index = step(-1, 1); show.entering = "start"; draw(false); return true; }
  if (k === "End") { const j = step(show.plan.length, -1); if (j >= 0) { show.index = j; draw(false); } return true; }
  return false;
}

function click(forward) {
  if (!show || !show.plan.length) return;
  if (show.blank) { setBlank(null); return; }
  // On Mouse Click off (`@advClick="0"`): a click does not move the slide on.
  const p = show.plan[show.index];
  if (forward && p && p.onClick === false && !(show.anim && show.anim.at < show.anim.clicks.length)) return;
  if (forward) next(); else previous();
}

/* ── Presenter View ─────────────────────────────────────────────────────── */

let presenterUrls = [];

function openPresenter() {
  if (presenter && !presenter.closed) { presenter.focus(); return; }
  let w = null;
  try { w = window.open("./presenter.html", "openpresent-presenter", "popup,width=1100,height=700"); } catch { w = null; }
  if (!w) { status("Presenter View needs a second window, and this browser blocked it"); return; }
  presenter = w;
}

function closePresenter() {
  for (const u of presenterUrls.splice(0)) URL.revokeObjectURL(u);
  if (presenter && !presenter.closed) presenter.close();
  presenter = null;
}

let presenterToken = 0;

/// Bring the presenter window up to date: current and next slide, notes, count.
async function updatePresenter() {
  const api = presenter && !presenter.closed ? presenter.openpresentPresenter : null;
  if (!api || !show) return;
  const t = ++presenterToken;
  const n = show.plan.length;
  const i = show.index;
  const nextIndex = i < n ? step(i, 1) : n;
  const view = {
    started: show.started,
    index: i,
    count: n,
    ended: i >= n,
    blank: show.blank,
    effect: i < n && show.plan[i].transition && !show.plan[i].asIs ? show.plan[i].effect : "",
    full: false,
    current: null,
    next: null,
    nextLabel: nextIndex < n ? `Slide ${nextIndex + 1}` : "End of slide show",
    notes: "",
  };
  api.update(view);   // the count and timer at once; pictures and notes follow
  const [current, upcoming, notes] = await Promise.all([
    i < n ? picture(i, 960 * dpr(), PRESENTER_PRIORITY) : null,
    nextIndex < n ? picture(nextIndex, 480 * dpr(), PRESENTER_PRIORITY) : null,
    i < n ? request({ op: "notes", index: i }, { priority: PRESENTER_PRIORITY }).then((r) => r.notes, () => "") : "",
  ]);
  const fresh = [current, upcoming].filter(Boolean);
  if (t !== presenterToken || !presenter || presenter.closed || !show) { for (const u of fresh) URL.revokeObjectURL(u); return; }
  const old = presenterUrls;
  presenterUrls = fresh;
  presenter.openpresentPresenter?.update({ ...view, full: true, current, next: upcoming, notes });
  // Kept one update longer than shown, so the window never shows a revoked picture.
  setTimeout(() => { for (const u of old) URL.revokeObjectURL(u); }, 1000);
}

/// What `presenter.js` calls on the window that opened it.
const presenterBridge = {
  ready() { updatePresenter(); },
  key(k) { key({ key: k }); },
  next() { click(true); },
  previous() { click(false); },
  end() { end(); },
};

/* ── Wiring ─────────────────────────────────────────────────────────────── */

/// `after(index)` is called with the slide last shown when the show ends.
export function init(after) {
  onEnd = after;
  globalThis.openpresentShow = presenterBridge;
  const box = byId("show");
  box.addEventListener("click", () => click(true));
  box.addEventListener("contextmenu", (e) => { e.preventDefault(); click(false); });
  box.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    if (key(e)) e.preventDefault();
  });
  // Leaving full screen with the browser's own Escape ends the show too.
  // With Presenter View open, moving to its window may leave full screen; the show goes on.
  document.addEventListener("fullscreenchange", () => {
    if (!document.fullscreenElement && show && !(presenter && !presenter.closed)) end();
  });
  window.addEventListener("resize", () => { if (show && show.plan.length) { show.entering = "keep"; draw(false); } });
  window.addEventListener("pagehide", closePresenter);
}
