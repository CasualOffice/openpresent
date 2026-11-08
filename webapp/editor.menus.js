// Right-click menus and region keys, as PowerPoint's (`UX-003`, docs/53).
//
// One menu per thing under the pointer: a thumbnail, a shape, text being
// edited, or the slide itself. Each is a list of command ids in
// `ribbon.model.js`, run through the registry, so a verb this build lacks is
// shown disabled there exactly as it is on the ribbon. Shift+F10 and the Menu
// key open the menu for what has focus; Ctrl+F6 moves between the window's
// regions, as the web product does.
//
// A right-click on a shape that is not selected selects it first, as PowerPoint
// does, then shows the shape's menu (`selection.selectForMenu`).

import { state } from "./editor.core.js";
import { contextMenu } from "./ribbon.js";
import { CONTEXT_MENUS } from "./ribbon.model.js";
import { register } from "./commands.js";
import * as selection from "./editor.selection.js";
import * as text from "./editor.text.js";
import * as slides from "./editor.slides.js";

const byId = (id) => document.getElementById(id);

/// Which menu the stage gets now.
function stageMenu() {
  if (text.editing()) return CONTEXT_MENUS.text;
  if (selection.hasSelection()) return CONTEXT_MENUS.shape;
  return CONTEXT_MENUS.slide;
}

function onStage(e) {
  if (!state.deck || state.reading || !e.target.closest?.("#slide-frame, .stage")) return;
  e.preventDefault();
  selection.selectForMenu(e);
  contextMenu(e.clientX, e.clientY, stageMenu());
}

function onThumbs(e) {
  const li = e.target.closest?.(".thumb");
  if (!state.deck || !li) return;
  e.preventDefault();
  const i = Number(li.dataset.index);
  if (Number.isInteger(i) && i !== state.current) slides.go(i);
  contextMenu(e.clientX, e.clientY, CONTEXT_MENUS.thumb);
}

/// Shift+F10 or the Menu key: the menu for the focused region, at its centre
/// (thumbnails: at the current slide).
function menuFromKeyboard() {
  const a = document.activeElement;
  if (a?.closest?.("#slide-pane")) {
    const li = byId("thumbs").children[state.current];
    const r = (li || byId("thumbs")).getBoundingClientRect();
    contextMenu(r.left + 24, r.top + r.height / 2, CONTEXT_MENUS.thumb);
    return;
  }
  const r = byId("slide-frame").getBoundingClientRect();
  contextMenu(r.left + r.width / 2, r.top + r.height / 2, stageMenu());
}

/// Ctrl+F6 order on the web (docs/49): slide, notes, status bar, title bar,
/// ribbon, thumbnails.
const REGIONS = ["#stage", "#notes-pane", ".bottom-bar button:not(:disabled)", ".rb-search-input", ".rb-tab.is-on", "#thumbs"];
function cycleRegion(dir) {
  const els = REGIONS.map((s) => document.querySelector(s)).filter((n) => n && n.offsetParent !== null);
  if (!els.length) return;
  const i = els.findIndex((n) => n.contains(document.activeElement));
  const next = els[(i + dir + els.length) % els.length];
  if (next.id === "notes-pane" && !next.hasAttribute("tabindex")) next.tabIndex = -1;
  next.focus({ preventScroll: true });
}

export function registerMenuCommands() {
  register("view.context-menu", { label: "Show Context Menu", shortcut: "Shift+F10", enabled: () => !!state.deck, run: menuFromKeyboard });
  register("view.next-region", { label: "Next Region", shortcut: "Ctrl+F6", run: () => cycleRegion(1) });
}

export function init() {
  byId("stage").addEventListener("contextmenu", onStage);
  byId("slide-pane").addEventListener("contextmenu", onThumbs);
  document.addEventListener("keydown", (e) => {
    if (e.target.closest?.(".rb-pop, .rb-backstage, .op-modal")) return;
    if ((e.key === "F10" && e.shiftKey) || e.key === "ContextMenu") {
      if (!state.deck || /^(INPUT|TEXTAREA|SELECT)$/.test(e.target.tagName) && !text.editing()) return;
      e.preventDefault();
      menuFromKeyboard();
    } else if (e.key === "F6" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      cycleRegion(e.shiftKey ? -1 : 1);
    }
  }, true);
}
