// The PowerPoint shell: Quick Access Toolbar and search in the title bar, the
// tab strip, the ribbon and the File page.
//
// Copied from OpenCalc's `webapp/ribbon.js` at `b23c67c` (webapp/README.md, owner
// instruction 42) and cut to one page shell: OpenCalc's instances per embedded
// editor (shadow roots), combos, toolbar popups and the phone command bar are
// not here yet, and arrive with the features that need them. KeyTips are
// OpenCalc's (Alt shows a letter on File and every tab; the letter opens it),
// carried one level further as PowerPoint does: a tab's letter shows a badge on
// each of its buttons, typing one runs it, and Esc steps back (`UX-003`). What is
// kept is its rule that matters: **this chrome owns its controls**, every button
// drawn from `ribbon.model.js` and run through the command registry
// (`commands.js`), and a button's state read from the registry, never mirrored.
//
// One change, by the owner (2026-11-08): a command the registry knows but cannot
// run is drawn **disabled**, not hidden, so the ribbon shows PowerPoint's whole
// shape while saying plainly what works today.

import * as signer from "./editor.author.js";
import { command, run, all as allCommands } from "./commands.js";
import { TABS, FILE_RAIL, PRESENT_MENU } from "./ribbon.model.js";

const LAYOUT_KEY = "op.ribbon.layout";
const DISPLAY_KEY = "op.ribbon.display";
// A new key: the old one held OpenCalc's default, captions only, for everyone.
const LABELS_KEY = "op.ribbon.labels.v2";
// PowerPoint's classic ribbon names its groups and labels its buttons (docs/53),
// so both are on; OpenCalc's owner chose captions only for Excel (2026-10-14).
const LABELS_DEFAULT = "full";

const store = {
  get(k, d) { try { return localStorage.getItem(k) || d; } catch { return d; } },
  set(k, v) { try { localStorage.setItem(k, v); } catch { /* private mode */ } },
};

/* ── Icons ──────────────────────────────────────────────────────────────── */

/// Put the Fluent sprite into the page, once. Parsed as SVG with `DOMParser`
/// and imported, rather than OpenCalc's `innerHTML`: the result is the same
/// `<symbol>`s, and no string reaches the DOM as HTML. The file is ours, from
/// this origin; a failure leaves the buttons labelled but without icons.
async function ensureSprite() {
  if (document.getElementById("oc-fluent-icons")) return;
  try {
    const url = new URL("./icons.fluent.svg", import.meta.url);
    const r = await fetch(url);
    if (!r.ok) throw new Error(`sprite ${r.status}`);
    const doc = new DOMParser().parseFromString(await r.text(), "image/svg+xml");
    const svg = doc.documentElement;
    if (svg.nodeName !== "svg") throw new Error("sprite is not SVG");
    svg.removeAttribute("style");   // a style attribute is refused by the CSP; CSS hides it
    svg.setAttribute("class", "op-sprite");
    document.body.insertBefore(document.importNode(svg, true), document.body.firstChild);
  } catch (e) {
    console.warn("[shell] icon sprite failed to load", e);
  }
}

const SVG = "http://www.w3.org/2000/svg";
export function icon(name, size) {
  const svg = document.createElementNS(SVG, "svg");
  svg.setAttribute("viewBox", "0 0 20 20");
  svg.setAttribute("width", String(size));
  svg.setAttribute("height", String(size));
  svg.setAttribute("aria-hidden", "true");
  svg.setAttribute("class", "rb-ico");
  const use = document.createElementNS(SVG, "use");
  use.setAttribute("href", `#fi-${name}`);
  svg.appendChild(use);
  return svg;
}
const caret = () => { const c = icon("chevron_down", 10); c.classList.add("rb-caret"); return c; };

function h(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

/// "Bold (Ctrl+B)" → "Bold".
const shortLabel = (s) => s.replace(/\s*\(.*\)\s*$/, "");

/// The tooltip of a disabled command says so, rather than nothing.
const tip = (label, c) => (c && c.enabled && !c.enabled() && c.reason ? `${label}: ${c.reason}` : label);

/* ── Popovers ───────────────────────────────────────────────────────────── */

let openPop = null;
function closePop() {
  if (!openPop) return;
  const { el, owner, onClose } = openPop;
  openPop = null;
  el.remove();
  owner?.setAttribute("aria-expanded", "false");
  onClose?.();
}

/// Below `anchor` if it fits, above if not, clamped to the window.
function place(el, anchor) {
  const r = anchor.getBoundingClientRect();
  el.style.left = "0px";
  el.style.top = "0px";
  const w = el.offsetWidth, ht = el.offsetHeight;
  let left = r.left;
  if (left + w > window.innerWidth - 4) left = Math.max(4, window.innerWidth - 4 - w);
  let top = r.bottom + 2;
  if (top + ht > window.innerHeight - 4) top = r.top - 2 - ht >= 4 ? r.top - 2 - ht : Math.max(4, window.innerHeight - 4 - ht);
  el.style.left = `${left}px`;
  el.style.top = `${top}px`;
}

function popover(owner, build, onClose) {
  closePop();
  const el = h("div", "popmenu rb-pop");
  el.setAttribute("role", "menu");
  build(el);
  document.body.appendChild(el);
  place(el, owner);
  owner.setAttribute("aria-expanded", "true");
  openPop = { el, owner, onClose };
  el.querySelector("button:not(:disabled), input")?.focus({ preventScroll: true });
  return el;
}

/// A dropdown listing commands by id, each run through the registry. A
/// disabled one is shown disabled in place.
function commandMenu(owner, ids) {
  popover(owner, (el) => {
    for (const id of ids) {
      const c = command(id);
      if (!c || (c.hidden && c.hidden())) continue;
      const b = h("button", "rb-pop-item", c.label || id);
      b.type = "button";
      b.dataset.cmd = id;
      b.setAttribute("role", "menuitem");
      if (c.shortcut) b.appendChild(h("span", "rb-pop-key", c.shortcut));
      if (c.enabled && !c.enabled()) { b.disabled = true; b.title = tip(c.label || id, c); }
      if (c.checked && c.checked()) b.setAttribute("aria-checked", "true");
      b.addEventListener("click", (e) => { e.stopPropagation(); closePop(); run(id); queueSync(); });
      el.appendChild(b);
    }
  });
}

/* ── Controls ───────────────────────────────────────────────────────────── */

const sync = new Set();   // functions that refresh a control's state

function button(spec, size) {
  const b = h("button", `rb-btn rb-${size}`);
  b.type = "button";
  b.dataset.cmd = spec.id;
  b.dataset.opProxy = spec.id;   // forwards to this command; never owns it
  b.title = spec.label;
  b.setAttribute("aria-label", shortLabel(spec.label));
  b.appendChild(icon(spec.icon, size === "l" ? 24 : 16));
  if (size === "l" || spec.text) b.appendChild(h("span", "rb-label", shortLabel(spec.label)));
  if (spec.text) b.classList.add("rb-text");
  if (spec.menu) { b.appendChild(caret()); b.setAttribute("aria-haspopup", "true"); b.setAttribute("aria-expanded", "false"); }
  b.addEventListener("click", (e) => {
    e.stopPropagation();
    if (spec.menu) commandMenu(b, spec.menu);
    else { closePop(); run(spec.id); queueSync(); }
  });
  sync.add(() => {
    const cc = command(spec.id);
    // A dropdown is live while anything it drops is; otherwise the command's own state.
    const live = spec.menu
      ? spec.menu.some((id) => { const m = command(id); return m && (!m.enabled || m.enabled()); })
      : !!cc && (!cc.enabled || cc.enabled());
    b.hidden = !!cc && !!cc.hidden && cc.hidden();
    if (b.disabled !== !live) b.disabled = !live;
    b.title = tip(spec.label, cc);
    if (cc?.checked) {
      const p = cc.checked() ? "true" : "false";
      if (b.getAttribute("aria-pressed") !== p) b.setAttribute("aria-pressed", p);
    }
  });
  return b;
}

function rows(spec) {
  const col = h("div", "rb-rows");
  col.dataset.rows = String(spec.rows.length);
  for (const r of spec.rows) {
    const row = h("div", "rb-row");
    for (const it of r) row.appendChild(it.kind === "combo" ? combo(it) : button(it, "s"));
    col.appendChild(row);
  }
  return col;
}

/// A combo box (Font, Font Size). Its value is the command's `value()` when the
/// registry gives one; with no working command it is drawn disabled, as a button is.
function combo(spec) {
  const wrap = h("div", "rb-combo");
  wrap.style.width = `${spec.width}px`;   // CSSOM, which the CSP allows
  const b = h("button", "rb-combo-btn");
  b.type = "button";
  b.dataset.cmd = spec.id;
  b.dataset.opProxy = spec.id;
  b.title = spec.label;
  b.setAttribute("aria-label", spec.label);
  b.setAttribute("aria-haspopup", "listbox");
  const val = h("span", "rb-combo-val", "");
  b.append(val, caret());
  b.addEventListener("click", (e) => { e.stopPropagation(); closePop(); run(spec.id); queueSync(); });
  wrap.appendChild(b);
  sync.add(() => {
    const c = command(spec.id);
    const live = !!c && (!c.enabled || c.enabled());
    if (b.disabled !== !live) b.disabled = !live;
    b.title = tip(spec.label, c);
    const v = c && c.value ? c.value() : "";
    if (val.textContent !== v) val.textContent = v;
  });
  return wrap;
}

/// An in-ribbon gallery: a row of tiles (our own swatches, or icons) and the
/// More arrow. The gallery is one command, enabled or not as that command is.
function gallery(spec) {
  const g = h("div", "rb-gallery");
  g.setAttribute("role", "listbox");
  g.setAttribute("aria-label", spec.label);
  const tiles = h("div", "rb-gal-tiles");
  const btns = [];
  for (const t of spec.tiles) {
    const b = h("button", "rb-gal-tile");
    b.type = "button";
    b.setAttribute("role", "option");
    b.title = t[0];
    if (t[1].startsWith("#")) {
      // A swatch is our own preview, not a named theme: a click opens the
      // command's own chooser, so the tile says that rather than a name.
      b.title = `${spec.label}…`;
      const sw = h("span", "rb-gal-swatch");
      sw.style.background = t[1];
      const aa = h("span", "rb-gal-aa", "Aa");
      aa.style.color = t[2];
      const bar = h("span", "rb-gal-bar");
      bar.style.background = t[2];
      sw.append(aa, bar);
      b.appendChild(sw);
    } else {
      b.append(icon(t[1], 24), h("span", "rb-gal-name", t[0]));
    }
    // A command that takes a tile (`pick`, `VS-10`) gets the tile's name; any
    // other opens its chooser.
    b.addEventListener("click", (e) => {
      e.stopPropagation(); closePop();
      const c = command(spec.id);
      if (c && c.pick && (!c.enabled || c.enabled())) c.pick(t[0]); else run(spec.id);
      queueSync();
    });
    b.dataset.tile = t[0];
    tiles.appendChild(b);
    btns.push(b);
  }
  const more = h("button", "rb-gal-more");
  more.type = "button";
  more.title = `More ${spec.label}`;
  more.setAttribute("aria-label", `More ${spec.label}`);
  more.appendChild(icon("chevron_down", 12));
  more.addEventListener("click", (e) => { e.stopPropagation(); closePop(); run(spec.id); queueSync(); });
  btns.push(more);
  g.append(tiles, more);
  sync.add(() => {
    const c = command(spec.id);
    const live = !!c && (!c.enabled || c.enabled());
    for (const b of btns) if (b.disabled !== !live) b.disabled = !live;
    const on = c && c.selected ? c.selected() : null;
    for (const b of btns) if (b.dataset.tile) b.setAttribute("aria-selected", b.dataset.tile === on ? "true" : "false");
    // A tile its command applies on one click (`pick`, `UX-003`) is named for what it applies.
    if (c && c.pick) for (const b of btns) if (b.dataset.tile && b.title !== b.dataset.tile) b.title = b.dataset.tile;
    more.title = tip(`More ${spec.label}`, c);
  });
  return g;
}

function group(spec) {
  const g = h("div", "rb-group");
  g.setAttribute("role", "group");
  g.setAttribute("aria-label", spec.label);
  g.dataset.fold = String(spec.fold ?? 99);
  const items = h("div", "rb-items");
  for (const it of spec.items) {
    items.appendChild(it.kind === "large" ? button(it, "l") : it.kind === "gallery" ? gallery(it) : rows(it));
  }
  const cap = h("div", "rb-cap");
  cap.appendChild(h("span", "rb-cap-text", spec.label));
  g.append(items, cap);
  // The dialog launcher at the group's bottom right, as PowerPoint's.
  if (spec.launcher) {
    const lb = h("button", "rb-launch");
    lb.type = "button";
    lb.dataset.cmd = spec.launcher;
    lb.appendChild(icon("arrow_down", 12));
    lb.addEventListener("click", (e) => { e.stopPropagation(); closePop(); run(spec.launcher); queueSync(); });
    sync.add(() => {
      const c = command(spec.launcher);
      lb.disabled = !c || (c.enabled && !c.enabled());
      const name = shortLabel(c?.label || spec.label);
      lb.title = tip(name, c);
      lb.setAttribute("aria-label", name);
    });
    g.appendChild(lb);
  }
  // The folded form: one large button that drops the group's own contents.
  const folded = h("button", "rb-btn rb-l rb-folded");
  folded.type = "button";
  folded.title = spec.label;
  folded.setAttribute("aria-label", spec.label);
  folded.setAttribute("aria-haspopup", "true");
  folded.setAttribute("aria-expanded", "false");
  folded.append(icon(spec.icon || "more_horizontal", 24), h("span", "rb-label", spec.label), caret());
  folded.addEventListener("click", (e) => {
    e.stopPropagation();
    if (openPop?.owner === folded) { closePop(); return; }
    popover(folded, (el) => { el.classList.add("rb-fold-pop"); el.appendChild(items); }, () => g.insertBefore(items, cap));
  });
  g.insertBefore(folded, items);
  return g;
}

/* ── Build ──────────────────────────────────────────────────────────────── */

let el = null;          // the ribbon
let backstage = null;
let syncQueued = false;

/// Refresh every control's state from the registry, once per frame at most.
export function queueSync() {
  if (syncQueued) return;
  syncQueued = true;
  requestAnimationFrame(() => {
    syncQueued = false;
    for (const f of sync) f();
  });
}

/// Fold groups, lowest `fold` first, until the active panel fits.
function fit(panel) {
  if (!panel) return;
  for (const g of panel.querySelectorAll(".rb-group.is-folded")) g.classList.remove("is-folded");
  const groups = [...panel.querySelectorAll(".rb-group")].sort((a, b) => a.dataset.fold - b.dataset.fold);
  const over = () => panel.scrollWidth > panel.clientWidth + 1;
  for (const g of groups) {
    if (!over()) break;
    g.classList.add("is-folded");
  }
  // Folding a narrow group saves little, so open again, highest rank first,
  // every group that fits now that others are folded.
  for (const g of groups.filter((x) => x.classList.contains("is-folded")).reverse()) {
    g.classList.remove("is-folded");
    if (over()) g.classList.add("is-folded");
  }
}

function buildTitleBar() {
  const header = document.querySelector(".app-header");
  if (!header) return;
  const logo = header.querySelector(".brand-logo");
  // Quick Access Toolbar: Save, Undo, Redo, PowerPoint's default three, and
  // Start From Beginning, which PowerPoint adds for a presentation.
  const qat = h("div", "rb-qat");
  qat.setAttribute("role", "toolbar");
  qat.setAttribute("aria-label", "Quick Access Toolbar");
  for (const [id, ic, label] of [["file.save", "save", "Save (Ctrl+S)"], ["edit.undo", "arrow_undo", "Undo (Ctrl+Z)"], ["edit.redo", "arrow_redo", "Redo (Ctrl+Y)"], ["slideshow.from-beginning", "play", "Start From Beginning (F5)"]]) {
    qat.appendChild(button({ id, icon: ic, label }, "s"));
  }
  (logo ? logo.after(qat) : header.prepend(qat));

  // Search (Alt+Q): every command the registry can run, by name.
  const search = h("div", "rb-search");
  const input = h("input", "rb-search-input");
  input.type = "search";
  input.placeholder = "Search (Alt+Q)";
  input.setAttribute("aria-label", "Search commands");
  search.append(icon("search", 16), input);
  const results = (q) => {
    const f = q.trim().toLowerCase();
    if (!f) { closePop(); return; }
    const words = f.split(/\s+/);
    const hits = allCommands()
      .filter((c) => c.label && words.every((w) => c.label.toLowerCase().includes(w) || c.id.includes(w)))
      .slice(0, 8);
    popover(search, (el2) => {
      el2.classList.add("rb-search-results");
      if (!hits.length) el2.appendChild(h("div", "rb-pop-empty", "No matching commands"));
      for (const c of hits) {
        const b = h("button", "rb-pop-item", c.label);
        b.type = "button";
        b.setAttribute("role", "menuitem");
        if (c.shortcut) b.appendChild(h("span", "rb-pop-key", c.shortcut));
        if (c.enabled && !c.enabled()) { b.disabled = true; b.title = tip(c.label, c); }
        b.addEventListener("mousedown", (e) => e.preventDefault());
        b.addEventListener("click", (e) => { e.stopPropagation(); closePop(); input.value = ""; run(c.id); queueSync(); });
        el2.appendChild(b);
      }
    });
    input.focus();
  };
  input.addEventListener("input", () => results(input.value));
  input.addEventListener("keydown", (e) => {
    e.stopPropagation();
    if (e.key === "Enter") { e.preventDefault(); openPop?.el.querySelector(".rb-pop-item:not(:disabled)")?.click(); }
    else if (e.key === "Escape") { e.preventDefault(); closePop(); input.value = ""; }
    else if (e.key === "ArrowDown") { e.preventDefault(); openPop?.el.querySelector(".rb-pop-item")?.focus(); }
  });
  const right = header.querySelector(".header-right");
  header.insertBefore(search, right);
  document.addEventListener("keydown", (e) => {
    if (e.altKey && (e.key === "q" || e.key === "Q" || e.code === "KeyQ")) { e.preventDefault(); e.stopPropagation(); input.focus(); }
  }, true);

}

/// The right end of the tab row, as PowerPoint for the web: Comments, the
/// editing mode, Present (a split button) and Share (docs/53).
function buildTabRowEnd(strip) {
  const end = h("div", "rb-tabs-end");
  const plain = (id, ic, label, cls) => {
    const b = h("button", `rb-end-btn ${cls || ""}`.trim());
    b.type = "button";
    b.dataset.opProxy = id;
    b.append(icon(ic, 16), h("span", "rb-end-label", label));
    b.addEventListener("click", (e) => { e.stopPropagation(); closePop(); run(id); queueSync(); });
    sync.add(() => {
      const c = command(id);
      b.disabled = !c || (c.enabled && !c.enabled());
      b.title = tip(label, c);
    });
    return b;
  };
  end.appendChild(plain("review.show-comments", "comment", "Comments"));
  // Editing mode: this build edits; Reviewing and Viewing arrive with comments.
  const mode = h("button", "rb-end-btn rb-mode");
  mode.type = "button";
  mode.title = "Editing mode";
  mode.setAttribute("aria-haspopup", "true");
  mode.append(icon("edit", 16), h("span", "rb-end-label", "Editing"), caret());
  mode.addEventListener("click", (e) => { e.stopPropagation(); commandMenu(mode, ["view.mode.editing", "view.mode.reviewing", "view.mode.viewing"]); });
  end.appendChild(mode);
  // Present: the main part plays from the current slide, the arrow drops the rest.
  const present = h("div", "rb-present");
  const main = h("button", "rb-present-main");
  main.type = "button";
  main.dataset.opProxy = "slideshow.from-current";
  main.append(icon("presenter", 16), h("span", "rb-end-label", "Present"));
  main.addEventListener("click", (e) => { e.stopPropagation(); closePop(); run("slideshow.from-current"); });
  const drop = h("button", "rb-present-drop");
  drop.type = "button";
  drop.title = "Presenting options";
  drop.setAttribute("aria-label", "Presenting options");
  drop.setAttribute("aria-haspopup", "true");
  drop.appendChild(caret());
  drop.addEventListener("click", (e) => { e.stopPropagation(); if (openPop?.owner === drop) closePop(); else commandMenu(drop, PRESENT_MENU); });
  present.append(main, drop);
  sync.add(() => {
    const c = command("slideshow.from-current");
    main.disabled = !c || (c.enabled && !c.enabled());
    main.title = tip("Present from the current slide (Shift+F5)", c);
  });
  end.appendChild(present);
  end.appendChild(plain("file.share", "share", "Share", "rb-share"));
  strip.appendChild(end);
}

/// A right-click menu at a point: command ids run through the registry, `null`
/// a separator. A verb this build lacks is shown disabled, as on the ribbon.
export function contextMenu(x, y, ids) {
  const anchor = { getBoundingClientRect: () => ({ left: x, right: x, top: y, bottom: y }), setAttribute() {} };
  popover(anchor, (el) => {
    el.classList.add("rb-ctx");
    let sep = false;
    for (const id of ids) {
      if (id === null) { sep = true; continue; }
      const c = command(id);
      if (!c || (c.hidden && c.hidden())) continue;
      if (sep && el.childElementCount) el.appendChild(h("div", "rb-pop-sep"));
      sep = false;
      const b = h("button", "rb-pop-item", shortLabel(c.label || id));
      b.type = "button";
      b.dataset.cmd = id;
      b.setAttribute("role", "menuitem");
      if (c.shortcut) b.appendChild(h("span", "rb-pop-key", c.shortcut));
      if (c.enabled && !c.enabled()) { b.disabled = true; b.title = tip(c.label || id, c); }
      b.addEventListener("click", (e) => { e.stopPropagation(); closePop(); run(id); queueSync(); });
      el.appendChild(b);
    }
  });
  // Keyboard: arrows move between items, Esc closes (the document's handler).
  openPop?.el.addEventListener("keydown", (e) => {
    const items = [...openPop.el.querySelectorAll(".rb-pop-item:not(:disabled)")];
    const i = items.indexOf(document.activeElement);
    const step = e.key === "ArrowDown" ? 1 : e.key === "ArrowUp" ? -1 : 0;
    if (!step || !items.length) return;
    e.preventDefault();
    e.stopPropagation();
    items[(i + step + items.length) % items.length].focus();
  });
}

/// What decides whether a contextual tab shows: `(kind) => boolean`, set by the
/// editor (a shape selected shows Shape Format).
let contextual = () => false;
export function setContextual(fn) { contextual = fn; queueSync(); }

function buildBackstage() {
  const bs = h("div", "rb-backstage");
  bs.hidden = true;
  bs.setAttribute("role", "dialog");
  bs.setAttribute("aria-label", "File");
  const rail = h("nav", "rb-rail");
  const back = h("button", "rb-back");
  back.type = "button";
  back.title = "Back (Esc)";
  back.setAttribute("aria-label", "Back to the presentation");
  back.appendChild(icon("chevron_left", 18));
  back.addEventListener("click", () => closeBackstage());
  rail.appendChild(back);
  const pane = h("div", "rb-pane");
  bs.append(rail, pane);

  const pages = [];
  const show = (id) => {
    for (const [pid, item, page] of pages) {
      const on = pid === id;
      item.classList.toggle("is-on", on);
      item.setAttribute("aria-current", on ? "page" : "false");
      page.hidden = !on;
    }
  };
  for (const entry of FILE_RAIL) {
    const page = h("section", "rb-page");
    page.hidden = true;
    page.appendChild(h("h1", "rb-page-title", entry.label));
    for (const sec of entry.sections || []) {
      const block = h("div", "rb-section");
      if (sec.title) block.appendChild(h("h2", "rb-section-title", sec.title));
      // A row of tiles, as PowerPoint's New page shows Blank Presentation.
      if (sec.tiles) {
        const row = h("div", "rb-tiles");
        for (const it of sec.items) {
          const b = h("button", "rb-tile");
          b.type = "button";
          b.dataset.opProxy = it.id;
          b.title = it.desc;
          const art = h("span", "rb-tile-art rb-tile-slide");
          art.append(h("span", "rb-tile-title", "Click to add title"), h("span", "rb-tile-sub", "Click to add subtitle"));
          b.append(art, h("span", "rb-tile-label", it.label));
          b.addEventListener("click", () => { closeBackstage(); run(it.id); queueSync(); });
          row.appendChild(b);
        }
        block.appendChild(row);
        page.appendChild(block);
        continue;
      }
      const list = h("div", "rb-page-list");
      for (const it of sec.items) {
        const b = h("button", "rb-page-item");
        b.type = "button";
        b.dataset.opProxy = it.id;
        const text = h("span", "rb-item-text");
        text.append(h("span", "rb-item-label", it.label), h("span", "rb-item-desc", it.desc));
        b.append(icon(it.icon, 20), text);
        b.addEventListener("click", () => { closeBackstage(); run(it.id); queueSync(); });
        sync.add(() => {
          const c = command(it.id);
          b.disabled = !c || (c.enabled && !c.enabled());
          b.title = tip(it.label, c);
        });
        list.appendChild(b);
      }
      block.appendChild(list);
      page.appendChild(block);
    }
    const item = h("button", "rb-rail-item");
    item.type = "button";
    item.append(icon(entry.icon, 18), h("span", null, entry.label));
    item.addEventListener("click", () => show(entry.id));
    rail.appendChild(item);
    pane.appendChild(page);
    pages.push([entry.id, item, page]);
  }
  // Options: the ribbon's own settings, as OpenCalc's Options page.
  const opts = h("section", "rb-page");
  opts.hidden = true;
  opts.appendChild(h("h1", "rb-page-title", "Options"));
  opts.appendChild(optionsPage());
  const optItem = h("button", "rb-rail-item");
  optItem.type = "button";
  optItem.append(icon("settings", 18), h("span", null, "Options"));
  optItem.addEventListener("click", () => show("options"));
  rail.appendChild(optItem);
  pane.appendChild(opts);
  pages.push(["options", optItem, opts]);

  bs.addEventListener("keydown", (e) => { if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); closeBackstage(); } });
  show(pages[0][0]);
  return bs;
}

function segmented(label, options) {
  const row = h("div", "rb-option");
  const name = h("span", "rb-option-label", label);
  const seg = h("div", "rb-seg");
  seg.setAttribute("role", "radiogroup");
  seg.setAttribute("aria-label", label);
  const buttons = [];
  const paintAll = () => { for (const [b, isOn] of buttons) b.setAttribute("aria-checked", isOn() ? "true" : "false"); };
  for (const o of options) {
    const b = h("button", "rb-seg-btn", o.label);
    b.type = "button";
    b.setAttribute("role", "radio");
    b.addEventListener("click", () => { o.pick(); paintAll(); });
    buttons.push([b, o.on]);
    seg.appendChild(b);
  }
  paintAll();
  row.append(name, seg);
  return row;
}

function optionsPage() {
  const wrap = h("div", "rb-options");
  const appearance = h("div", "rb-section");
  appearance.appendChild(h("h2", "rb-section-title", "Appearance"));
  const theme = () => document.documentElement.dataset.theme || "auto";
  appearance.appendChild(segmented("Theme", [["auto", "Automatic"], ["light", "Light"], ["dark", "Dark"]]
    .map(([v, label]) => ({ label, pick: () => run(`view.theme.${v}`), on: () => theme() === v }))));
  wrap.appendChild(appearance);
  const ribbon = h("div", "rb-section");
  ribbon.appendChild(h("h2", "rb-section-title", "Ribbon"));
  ribbon.appendChild(segmented("Labels", [["none", "Icons only"], ["captions", "Group names"], ["full", "Names and labels"]]
    .map(([v, label]) => ({ label, pick: () => applyLabels(v), on: () => store.get(LABELS_KEY, LABELS_DEFAULT) === v }))));
  wrap.appendChild(ribbon);
  // The name comments are signed with (`UX-011`), kept in this browser.
  const you = h("div", "rb-section");
  you.appendChild(h("h2", "rb-section-title", "Comments"));
  const row = h("label", "rb-option-row");
  row.appendChild(h("span", null, "Your name"));
  const input = h("input", "fp-input");
  input.type = "text";
  input.maxLength = 255;
  input.value = signer.author().name;
  input.addEventListener("change", () => signer.setName(input.value));
  input.addEventListener("keydown", (e) => e.stopPropagation());
  row.appendChild(input);
  you.appendChild(row);
  wrap.appendChild(you);
  return wrap;
}

function applyLabels(v) {
  store.set(LABELS_KEY, v);
  if (!el) return;
  el.classList.toggle("rb-labels-captions", v === "captions" || v === "full");
  el.classList.toggle("rb-labels-full", v === "full");
  fit(el.querySelector(".rb-panel:not([hidden])"));
}

export function openBackstage(page) {
  if (!backstage) return;
  closePop();
  queueSync();
  backstage.hidden = false;
  document.documentElement.classList.add("op-backstage-open");
  if (page) [...backstage.querySelectorAll(".rb-rail-item")].find((b) => b.textContent === page)?.click();
  backstage.querySelector(".rb-rail-item.is-on, .rb-rail-item")?.focus();
}

function closeBackstage() {
  if (!backstage || backstage.hidden) return;
  backstage.hidden = true;
  document.documentElement.classList.remove("op-backstage-open");
  el?.querySelector(".rb-file")?.focus();
}

/* ── KeyTips ────────────────────────────────────────────────────────────── */

/// PowerPoint's own KeyTips for the buttons a user reaches for most, by command
/// id. Every other button gets one made from its label (`assignKeys`).
const KNOWN_TIPS = {
  "edit.paste": "V", "edit.cut": "X", "edit.copy": "C", "home.format-painter": "FP",
  "home.new-slide": "I", "home.layout": "L", "home.reset": "R", "home.section": "T",
  "font.name": "FF", "font.size": "FS", "font.increase": "FG", "font.decrease": "FK",
  "font.clear": "E", "font.bold": "1", "font.italic": "2", "font.underline": "3",
  "font.strike": "4", "font.color": "FC", "font.highlight": "TC",
  "para.bullets": "U", "para.numbering": "N", "para.left": "AL", "para.center": "AC",
  "para.right": "AR", "para.justify": "AJ", "para.indent-less": "AO", "para.indent-more": "AI",
  "insert.picture": "P", "insert.table": "T", "insert.textbox": "X", "insert.shapes": "SH",
  "picture.crop": "C", "picture.reset": "Q", "picture.style": "K",
};

/// A KeyTip for each of `items` (`[id, label]`), unique and prefix-free, so
/// typing one never stops short at another: PowerPoint's where it has one, else
/// the label's first letter, its words' initials, then its first letter and each
/// other in turn, then a number.
function assignKeys(items, taken = []) {
  const used = [...taken];
  const free = (k) => k && !used.some((u) => u.startsWith(k) || k.startsWith(u));
  const out = new Map();
  const claim = (id, k) => { used.push(k); out.set(id, k); };
  for (const [id] of items) if (KNOWN_TIPS[id] && free(KNOWN_TIPS[id])) claim(id, KNOWN_TIPS[id]);
  for (const [id, label] of items) {
    if (out.has(id)) continue;
    const letters = shortLabel(label).toUpperCase().replace(/[^A-Z0-9 ]/g, "");
    const words = letters.split(/\s+/).filter(Boolean);
    const first = words[0]?.[0] || "";
    const cands = [first, words.slice(0, 2).map((w) => w[0]).join("")];
    for (const c of letters.replace(/\s+/g, "").slice(1)) cands.push(first + c);
    for (const a of "ABCDEFGHIJKLMNOPQRSTUVWXYZ") cands.push(first + a);
    // Every two-letter pair as a last resort: a bounded list, so a full set of
    // keys leaves a button without a tip rather than looping.
    for (const a of "ABCDEFGHIJKLMNOPQRSTUVWXYZ") for (const b of "ABCDEFGHIJKLMNOPQRSTUVWXYZ") cands.push(a + b);
    const k = cands.find((c) => c.length && free(c));
    if (k) claim(id, k);
  }
  return out;
}

/// Alt (released alone) or Alt+Period shows the tabs' KeyTips; a tab's letters
/// show its buttons'; Esc steps back a level, and a click or another key with a
/// modifier puts them away.
function installKeyTips(file, tablist, panels, select) {
  let mode = null;      // null, "tabs", or the tab id whose buttons show tips
  let typed = "";
  let targets = new Map();   // KeyTip → element to click
  const clear = () => {
    for (const t of document.querySelectorAll(".op-ribbon .rb-tip")) t.remove();
    el.classList.remove("rb-keytips");
    targets = new Map();
    typed = "";
  };
  const badge = (node, key) => {
    const t = h("span", "rb-tip", key);
    t.setAttribute("aria-hidden", "true");
    node.appendChild(t);
    targets.set(key, node);
  };
  const off = () => { mode = null; clear(); };
  const showTabs = () => {
    clear();
    mode = "tabs";
    el.classList.add("rb-keytips");
    badge(file, "F");
    for (const tab of TABS) {
      const t = tablist.querySelector(`#rb-tab-${tab.id}`);
      if (t && !t.hidden) badge(t, tab.key);
    }
  };
  const showButtons = (id) => {
    clear();
    mode = id;
    el.classList.add("rb-keytips");
    const panel = panels.get(id);
    if (!panel) return;
    const nodes = [...panel.querySelectorAll(".rb-btn, .rb-combo-btn, .rb-gal-more, .rb-launch")]
      .filter((b) => !b.hidden && b.getClientRects().length > 0);
    const items = nodes.map((b, i) => [b.dataset.cmd ? `${b.dataset.cmd}#${i}` : `#${i}`, b.getAttribute("aria-label") || b.title || "Z"]);
    // A command seen twice (a gallery's More, a launcher) keys by its first.
    const keyed = assignKeys(items.map(([k, l]) => [k.split("#")[0] || k, l]).filter(([k], i, a) => a.findIndex(([x]) => x === k) === i));
    nodes.forEach((b, i) => {
      const id0 = items[i][0].split("#")[0] || items[i][0];
      const k = keyed.get(id0);
      if (k && !targets.has(k)) badge(b, k);
    });
  };
  // Shown on the release of a lone Alt, as OpenCalc does, so Alt as a modifier
  // (Alt+Q, Alt+F5) never raises them or loses its keystroke.
  let altAlone = false;
  document.addEventListener("keyup", (e) => {
    if (e.key === "Alt" && altAlone) { altAlone = false; if (mode) off(); else showTabs(); }
  });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Alt") { if (!e.repeat) altAlone = true; return; }
    altAlone = false;
    // Alt+Period (Option+Period on a Mac, where Alt alone is taken): PowerPoint
    // for the web's way in. `code`, as Option+Period types "≥".
    if (e.altKey && !e.ctrlKey && !e.metaKey && e.code === "Period") {
      e.preventDefault(); e.stopPropagation();
      if (mode) off(); else showTabs();
      return;
    }
    if (!mode) return;
    if (e.key === "Escape") {
      e.preventDefault(); e.stopImmediatePropagation();
      if (mode === "tabs") off(); else showTabs();
      return;
    }
    if (e.ctrlKey || e.metaKey || e.altKey) { off(); return; }
    if (e.key === "Shift") return;
    const k = e.key.length === 1 ? e.key.toUpperCase() : "";
    e.preventDefault(); e.stopImmediatePropagation();
    if (!k) return;
    const next = typed + k;
    const hit = targets.get(next);
    if (hit) {
      if (mode === "tabs") {
        if (hit === file) { off(); hit.click(); return; }
        const tab = TABS.find((t) => `rb-tab-${t.id}` === hit.id);
        select(tab.id);
        showButtons(tab.id);
        return;
      }
      off();
      hit.click();
      return;
    }
    if ([...targets.keys()].some((t) => t.startsWith(next))) {
      typed = next;
      for (const [t, node] of targets) node.querySelector(".rb-tip")?.classList.toggle("rb-tip-dim", !t.startsWith(next));
    }
  }, true);
  document.addEventListener("mousedown", () => { if (mode) off(); }, true);
  window.addEventListener("blur", () => { if (mode) off(); });
}

/// Show tab `id`, as PowerPoint opens Shape Format on a shape just drawn. Before the
/// ribbon is built it does nothing.
export let selectTab = () => {};

/// Build the shell before `anchor` (the workspace). Returns the ribbon.
export async function start(anchor) {
  if (el) return el;
  await ensureSprite();
  el = h("div", "op-ribbon");
  const strip = h("div", "rb-tabs");
  const file = h("button", "rb-file", "File");
  file.type = "button";
  file.setAttribute("aria-haspopup", "dialog");
  file.addEventListener("click", () => openBackstage());
  const tablist = h("div", "rb-tablist");
  tablist.setAttribute("role", "tablist");
  tablist.setAttribute("aria-label", "Ribbon");
  strip.append(file, tablist);

  const band = h("div", "rb-band");
  const panels = new Map();
  for (const tab of TABS) {
    const t = h("button", "rb-tab", tab.label);
    t.type = "button";
    t.id = `rb-tab-${tab.id}`;
    if (tab.contextual) {
      t.classList.add("rb-tab-ctx");
      t.hidden = true;
      sync.add(() => {
        const on = contextual(tab.contextual);
        if (t.hidden === !on) return;
        t.hidden = !on;
        if (!on && t.classList.contains("is-on")) select("home");
      });
    }
    t.setAttribute("role", "tab");
    t.setAttribute("aria-controls", `rb-panel-${tab.id}`);
    t.addEventListener("click", () => select(tab.id));
    tablist.appendChild(t);
    const panel = h("div", "rb-panel");
    panel.id = `rb-panel-${tab.id}`;
    panel.setAttribute("role", "tabpanel");
    panel.setAttribute("aria-labelledby", t.id);
    panel.hidden = true;
    for (const g of tab.groups) panel.appendChild(group(g));
    band.appendChild(panel);
    panels.set(tab.id, panel);
  }
  const layout = h("button", "rb-layout");
  layout.type = "button";
  layout.appendChild(icon("chevron_down", 14));
  band.appendChild(layout);
  buildTabRowEnd(strip);
  el.append(strip, band);
  anchor.parentElement.insertBefore(el, anchor);

  buildTitleBar();
  backstage = buildBackstage();
  document.body.appendChild(backstage);
  document.documentElement.classList.add("op-chrome-ribbon");

  function select(id) {
    closePop();
    for (const [k, p] of panels) p.hidden = k !== id;
    for (const b of tablist.querySelectorAll(".rb-tab")) {
      const on = b.id === `rb-tab-${id}`;
      b.classList.toggle("is-on", on);
      b.setAttribute("aria-selected", on ? "true" : "false");
      b.tabIndex = on ? 0 : -1;
    }
    fit(panels.get(id));
    queueSync();
  }
  selectTab = (id) => { if (panels.has(id)) select(id); };

  const applyLayout = (v) => {
    el.classList.toggle("rb-simplified", v === "simplified");
    store.set(LAYOUT_KEY, v);
    fit(el.querySelector(".rb-panel:not([hidden])"));
  };
  applyLayout(store.get(LAYOUT_KEY, "classic"));   // DEC-10: Classic, Simplified a toggle

  layout.title = "Ribbon Display Options (Ctrl+F1 to show or hide the ribbon)";
  layout.setAttribute("aria-label", "Ribbon Display Options");
  layout.setAttribute("aria-haspopup", "true");
  const tabsOnly = () => el.classList.contains("rb-tabs-only");
  const peek = (on) => el.classList.toggle("rb-peek", !!on && tabsOnly());
  const applyDisplay = (v) => {
    el.classList.toggle("rb-tabs-only", v === "tabs");
    peek(false);
    store.set(DISPLAY_KEY, v);
    window.dispatchEvent(new Event("resize"));
  };
  applyDisplay(store.get(DISPLAY_KEY, "always"));
  layout.addEventListener("click", (e) => {
    e.stopPropagation();
    if (openPop?.owner === layout) { closePop(); return; }
    const items = [
      ["Always show Ribbon", () => !tabsOnly(), () => applyDisplay("always")],
      ["Show tabs only", tabsOnly, () => applyDisplay("tabs")],
      null,
      ["Classic ribbon", () => !el.classList.contains("rb-simplified"), () => applyLayout("classic")],
      ["Simplified ribbon", () => el.classList.contains("rb-simplified"), () => applyLayout("simplified")],
    ];
    popover(layout, (pop) => {
      for (const it of items) {
        if (!it) { pop.appendChild(h("div", "rb-pop-sep")); continue; }
        const [label, on, pick] = it;
        const b = h("button", "rb-pop-item", label);
        b.type = "button";
        b.setAttribute("role", "menuitemradio");
        b.setAttribute("aria-checked", on() ? "true" : "false");
        b.addEventListener("click", (ev) => { ev.stopPropagation(); closePop(); pick(); });
        pop.appendChild(b);
      }
    });
  });
  for (const t of tablist.querySelectorAll(".rb-tab")) {
    t.addEventListener("click", () => {
      if (!tabsOnly()) return;
      peek(!(t.classList.contains("is-on") && el.classList.contains("rb-peek")));
    });
    t.addEventListener("dblclick", () => applyDisplay(tabsOnly() ? "always" : "tabs"));
  }
  applyLabels(store.get(LABELS_KEY, LABELS_DEFAULT));

  tablist.addEventListener("keydown", (e) => {
    const tabs = [...tablist.querySelectorAll(".rb-tab")].filter((t) => !t.hidden);
    const i = tabs.indexOf(document.activeElement);
    const step = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
    if (i < 0 || !step) return;
    e.preventDefault();
    e.stopPropagation();
    const next = tabs[(i + step + tabs.length) % tabs.length];
    next.focus();
    next.click();
  });
  installKeyTips(file, tablist, panels, select);
  document.addEventListener("keydown", (e) => {
    if (e.key === "F1" && (e.ctrlKey || e.metaKey)) { e.preventDefault(); applyDisplay(tabsOnly() ? "always" : "tabs"); }
    if (e.key === "Escape") closePop();
  });
  document.addEventListener("click", (e) => {
    if (openPop && !openPop.el.contains(e.target)) closePop();
    if (el.classList.contains("rb-peek") && !e.target.closest?.(".op-ribbon")) peek(false);
  });
  window.addEventListener("resize", () => fit(el.querySelector(".rb-panel:not([hidden])")));

  select("home");
  return el;
}
