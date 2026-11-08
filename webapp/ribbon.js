// The PowerPoint shell: Quick Access Toolbar and search in the title bar, the
// tab strip, the ribbon and the File page.
//
// Copied from OpenCalc's `webapp/ribbon.js` at `b23c67c` (webapp/README.md, owner
// instruction 42) and cut to one page shell: OpenCalc's instances per embedded
// editor (shadow roots), combos, toolbar popups, KeyTips and the phone command
// bar are not here yet, and arrive with the features that need them. What is
// kept is its rule that matters: **this chrome owns its controls**, every button
// drawn from `ribbon.model.js` and run through the command registry
// (`commands.js`), and a button's state read from the registry, never mirrored.
//
// One change, by the owner (2026-11-08): a command the registry knows but cannot
// run is drawn **disabled**, not hidden, so the ribbon shows PowerPoint's whole
// shape while saying plainly what works today.

import { command, run, all as allCommands } from "./commands.js";
import { TABS, FILE_RAIL } from "./ribbon.model.js";

const LAYOUT_KEY = "op.ribbon.layout";
const DISPLAY_KEY = "op.ribbon.display";
const LABELS_KEY = "op.ribbon.labels";
// As OpenCalc (its owner, 2026-10-14): group names on, button labels off.
const LABELS_DEFAULT = "captions";

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
  if (size === "l") b.appendChild(h("span", "rb-label", shortLabel(spec.label)));
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
    for (const it of r) row.appendChild(button(it, "s"));
    col.appendChild(row);
  }
  return col;
}

function group(spec) {
  const g = h("div", "rb-group");
  g.setAttribute("role", "group");
  g.setAttribute("aria-label", spec.label);
  g.dataset.fold = String(spec.fold ?? 99);
  const items = h("div", "rb-items");
  for (const it of spec.items) items.appendChild(it.kind === "large" ? button(it, "l") : rows(it));
  const cap = h("div", "rb-cap");
  cap.appendChild(h("span", "rb-cap-text", spec.label));
  g.append(items, cap);
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
  for (const g of groups) {
    if (panel.scrollWidth <= panel.clientWidth + 1) break;
    g.classList.add("is-folded");
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

  if (right) {
    const share = h("button", "rb-share");
    share.type = "button";
    share.dataset.opProxy = "file.share";
    share.append(icon("share", 16), h("span", null, "Share"));
    share.addEventListener("click", () => run("file.share"));
    right.prepend(share);
    sync.add(() => {
      const c = command("file.share");
      share.disabled = !c || (c.enabled && !c.enabled());
      share.title = tip("Share", c);
    });
  }
}

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
    t.setAttribute("role", "tab");
    t.setAttribute("aria-controls", `rb-panel-${tab.id}`);
    t.addEventListener("click", () => select(tab.id));
    tablist.appendChild(t);
    // A contextual tab (Shape Format, `SHAPE-01`) is there only while its command,
    // `tab.contextual`, is enabled: PowerPoint shows it while a shape is selected.
    if (tab.contextual) {
      t.classList.add("rb-tab-contextual");
      t.hidden = true;
      sync.add(() => {
        const c = command(tab.contextual);
        const on = !!c && (!c.enabled || c.enabled());
        if (t.hidden === on) t.hidden = !on;
        if (!on && t.classList.contains("is-on")) select("home");
      });
    }
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
    const tabs = [...tablist.querySelectorAll(".rb-tab")];
    const i = tabs.indexOf(document.activeElement);
    const step = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
    if (i < 0 || !step) return;
    e.preventDefault();
    e.stopPropagation();
    const next = tabs[(i + step + tabs.length) % tabs.length];
    next.focus();
    next.click();
  });
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
