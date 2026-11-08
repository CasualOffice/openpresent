// Dialogs: the one modal, and what it shows.
//
// OpenCalc's lightweight modal (`.oc-modal` in its `editor.css` and
// `editor.dialogs.js` at `b23c67c`, webapp/README.md), with one difference: its
// body is built from nodes with `textContent`, never from an HTML string, since
// an engine message can carry a part name read from the user's file.

import { icon } from "./ribbon.js";

const byId = (id) => document.getElementById(id);
let returnFocus = null;
let request = null;   // the engine client, set by `initDialogs` to avoid a cycle at load

function el(tag, cls, text) {
  const e = document.createElement(tag);
  if (cls) e.className = cls;
  if (text != null) e.textContent = text;
  return e;
}

/// Open the modal with `title` and a body built by `build(body)`.
export function showModal(title, build) {
  const modal = byId("op-modal");
  returnFocus = document.activeElement;
  byId("op-modal-title").textContent = title;
  const body = byId("op-modal-body");
  body.textContent = "";
  build(body);
  modal.hidden = false;
  (body.querySelector(".op-btn.primary") || byId("op-modal-x")).focus();
}

export function closeModal() {
  const modal = byId("op-modal");
  if (modal.hidden) return;
  modal.hidden = true;
  returnFocus?.focus?.({ preventScroll: true });
  returnFocus = null;
}

/// A file the engine refused, or anything else that went wrong. One message
/// box, titled with the product's name where PowerPoint's says "Microsoft
/// PowerPoint" (OpenCalc's docs/95 rule 8): what happened, then the engine's
/// own message with its code.
export function errorDialog(what, detail) {
  showModal("OpenPresent", (body) => {
    body.appendChild(el("p", "op-confirm-text", what));
    if (detail) body.appendChild(el("p", "op-muted", detail));
    const actions = el("div", "op-confirm-actions");
    const ok = el("button", "op-btn primary", "OK");
    ok.type = "button";
    ok.addEventListener("click", closeModal);
    actions.appendChild(ok);
    body.appendChild(actions);
  });
}

/// What slide `index` leaves out, from the engine's report.
export async function reportDialog(index) {
  if (!request) return;
  let lines;
  try {
    lines = (await request({ op: "report", index }, { priority: 4 })).lines;
  } catch (e) {
    errorDialog("The report for this slide could not be made.", e.message);
    return;
  }
  showModal(`Slide ${index + 1}: what is not drawn yet`, (body) => {
    if (!lines.length) {
      body.appendChild(el("p", null, "Everything on this slide that the engine reads is drawn."));
      return;
    }
    body.appendChild(el("p", null, "This build draws the slide without these. Each is a feature by name and count; the file itself is unchanged."));
    const list = el("ul", "op-report");
    for (const line of lines) list.appendChild(el("li", null, line));
    body.appendChild(list);
  });
}

const SHORTCUTS = [
  ["Open a presentation", "Ctrl+O"],
  ["Save", "Ctrl+S"],
  ["Next slide", "→ ↓ Page Down Space"],
  ["Previous slide", "← ↑ Page Up"],
  ["First and last slide", "Home, End"],
  ["Slide show from the beginning", "F5"],
  ["Slide show from the current slide", "Shift+F5"],
  ["Presenter View", "Alt+F5"],
  ["Next slide in a show", "Space, Enter, N, Right, Page Down or click"],
  ["Previous slide in a show", "P, Backspace, Left or Page Up"],
  ["Go to slide number", "Number, then Enter"],
  ["Black or white screen", "B or W"],
  ["End the slide show", "Esc"],
  ["Show or hide the ribbon", "Ctrl+F1"],
  ["Search commands", "Alt+Q"],
];

export function shortcutsDialog() {
  showModal("Keyboard shortcuts", (body) => {
    for (const [what, keys] of SHORTCUTS) {
      const row = el("div", "kb-row");
      row.append(el("span", null, what), el("kbd", null, keys));
      body.appendChild(row);
    }
  });
}

export function aboutDialog(version) {
  showModal("About OpenPresent", (body) => {
    body.appendChild(el("p", null, "An open-source presentation editor compatible with PowerPoint. This build opens .pptx files and shows their slides; editing arrives in later releases."));
    body.appendChild(el("p", "op-muted", `Engine ${version || "not loaded yet"}. Everything runs in this browser; your presentation is never uploaded.`));
  });
}

export function initDialogs(engineRequest) {
  request = engineRequest;
  const x = byId("op-modal-x");
  x.appendChild(icon("dismiss", 16));
  x.addEventListener("click", closeModal);
  const modal = byId("op-modal");
  modal.addEventListener("click", (e) => { if (e.target === modal) closeModal(); });
  modal.addEventListener("keydown", (e) => {
    if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); closeModal(); }
  });
}
