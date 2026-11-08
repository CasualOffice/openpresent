// Arrange (`UX-006`): Bring to Front, Bring Forward, Send Backward, Send to Back
// (Ctrl+Shift+], Ctrl+], Ctrl+[, Ctrl+Shift+[ as PowerPoint for the web), and Rotate:
// Rotate Right 90°, Rotate Left 90°, Flip Vertical, Flip Horizontal. Each is one engine
// op and one undo step, on the selected shape; the menu parents (Arrange, Align,
// Rotate) open their menus and run nothing of their own.

import { register } from "./commands.js";
import { state, request, status } from "./editor.core.js";
import { errorDialog } from "./editor.dialogs.js";
import * as selection from "./editor.selection.js";
import * as slides from "./editor.slides.js";

const ORDER = { "bring-to-front": 0, "bring-forward": 1, "send-backward": 2, "send-to-back": 3 };
const ROT_UNIT = 60000;

const one = () => !!state.deck && selection.selectedNodes().length === 1 && selection.selectedNode() != null;

async function edit(what, msg) {
  const index = state.current;
  try {
    const r = await request(msg, { priority: 9 });
    if (r.undo) selection.noteUndoState(r.undo);
    if (r.value === false) status(`${what}: already there`);
    await slides.redrawSlide(index);
    await selection.refresh();
  } catch (e) {
    errorDialog(`OpenPresent could not ${what.toLowerCase()}.`, e.message);
  }
}

function order(how) {
  const node = selection.selectedNode();
  return edit("Arrange the shape", { op: "build", fn: "arrange_order", args: [state.current, node, ORDER[how]] });
}

function flip(horizontal) {
  const node = selection.selectedNode();
  return edit("Flip the shape", { op: "build", fn: "arrange_flip", args: [state.current, node, horizontal] });
}

function rotate(by) {
  const s = selection.selectedShape();
  if (!s) return;
  const rot = (((s.rot || 0) + by) % 360 + 360) % 360;
  return edit("Rotate the shape", {
    op: "transform", index: state.current, node: s.node,
    x: Math.round(s.x), y: Math.round(s.y), cx: Math.round(s.cx), cy: Math.round(s.cy), rot: Math.round(rot * ROT_UNIT),
  });
}

export function init() {
  for (const [how, label, key] of [["bring-to-front", "Bring to Front", "Ctrl+Shift+]"], ["bring-forward", "Bring Forward", "Ctrl+]"], ["send-backward", "Send Backward", "Ctrl+["], ["send-to-back", "Send to Back", "Ctrl+Shift+["]]) {
    register(`arrange.${how}`, { label, shortcut: key, enabled: one, run: () => order(how) });
  }
  register("arrange.rotate-right", { label: "Rotate Right 90°", enabled: one, run: () => rotate(90) });
  register("arrange.rotate-left", { label: "Rotate Left 90°", enabled: one, run: () => rotate(-90) });
  register("arrange.flip-vertical", { label: "Flip Vertical", enabled: one, run: () => flip(false) });
  register("arrange.flip-horizontal", { label: "Flip Horizontal", enabled: one, run: () => flip(true) });
}

/// Ctrl+] and Ctrl+[ (with Shift, to the front or back), from the editor's keys.
export function onKey(e) {
  const mod = e.ctrlKey || e.metaKey;
  if (!mod || e.altKey || !one()) return false;
  const how = e.code === "BracketRight" ? (e.shiftKey ? "bring-to-front" : "bring-forward")
    : e.code === "BracketLeft" ? (e.shiftKey ? "send-to-back" : "send-backward") : null;
  if (!how) return false;
  e.preventDefault();
  order(how);
  return true;
}
