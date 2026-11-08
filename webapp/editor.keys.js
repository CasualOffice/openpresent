// PowerPoint's keyboard shortcuts (`PAR-13`): one table from keys to command ids,
// run through the registry, so a key does exactly what its ribbon button does and
// another session's command (Find, Replace, Print, Hyperlink, Group) is bound the
// moment it registers. A key whose command this build has not built yet still
// belongs to PowerPoint, not to the browser: it is taken and the status bar says the
// command is not available, as its greyed button does.
//
// Shaped after OpenCalc's `bindKeys` in `webapp/editor.core.js` at `b23c67c`: keys
// are matched on `KeyboardEvent.code` for punctuation, which is the same key on every
// layout with Shift held or not, and on `key` for letters. Ctrl and Cmd are one.
// The browser keeps what it never hands a page (Ctrl+N, Ctrl+T, Ctrl+W).
//
// `editor.core.js` calls `handle` for the stage and the panes, after the shape
// selection's and New Slide's own keys; `editor.text.js` calls it for keys typed in
// an open text box that it does not use itself.

import { status } from "./editor.core.js";
import { command, run } from "./commands.js";
import * as text from "./editor.text.js";

/// Keys to commands. A value may be a function of whether a text box is open, for a
/// key whose verb depends on it, as Ctrl+] does.
const SHORTCUTS = new Map([
  ["Ctrl+F", "edit.find"],
  ["Ctrl+H", "edit.replace"],
  ["Ctrl+P", "file.print"],
  ["Ctrl+K", "insert.link"],
  ["Ctrl+G", "arrange.group"],
  ["Ctrl+Shift+G", "arrange.ungroup"],
  // In text, the desktop's font size; on shapes, the web's layer order (docs/53).
  ["Ctrl+]", (editing) => (editing ? "font.increase" : "arrange.bring-forward")],
  ["Ctrl+[", (editing) => (editing ? "font.decrease" : "arrange.send-backward")],
  ["Ctrl+Shift+]", "arrange.bring-to-front"],
  ["Ctrl+Shift+[", "arrange.send-to-back"],
  ["Ctrl+Shift+.", "font.increase"],
  ["Ctrl+Shift+,", "font.decrease"],
  ["Ctrl+E", "para.center"],
  ["Ctrl+L", "para.left"],
  ["Ctrl+R", "para.right"],
  ["Ctrl+J", "para.justify"],
  ["Ctrl+B", "font.bold"],
  ["Ctrl+I", "font.italic"],
  ["Ctrl+U", "font.underline"],
  ["Ctrl+Space", "font.clear"],
  ["Ctrl+=", "font.subscript"],
  ["Ctrl+Shift+=", "font.superscript"],
  ["Ctrl+Shift+F", "font.dialog"],
  ["Ctrl+Shift+P", "font.dialog"],
  ["Ctrl+Shift+C", "home.format-painter"],
  ["Ctrl+Alt+V", "edit.paste-keep-source"],
  ["Ctrl+Alt+M", "review.new-comment"],
  ["Ctrl+D", "edit.duplicate"],
  ["Ctrl+M", "home.new-slide"],
  ["Ctrl+S", "file.save"],
  ["Ctrl+O", "file.open"],
  ["Ctrl+Shift+S", "file.save-copy"],
  ["F12", "file.save-copy"],
  ["F7", "review.spelling"],
  ["F5", "slideshow.from-beginning"],
  ["Shift+F5", "slideshow.from-current"],
  ["Alt+F5", "slideshow.presenter-view"],
  ["Alt+Shift+ArrowRight", "para.indent-more"],
  ["Alt+Shift+ArrowLeft", "para.indent-less"],
]);

const CODES = { BracketRight: "]", BracketLeft: "[", Period: ".", Comma: ",", Space: "Space", Equal: "=" };

/// The key as the table names it: modifiers in the order Ctrl, Alt, Shift, then the key.
export function combo(e) {
  const mods = [];
  if (e.ctrlKey || e.metaKey) mods.push("Ctrl");
  if (e.altKey) mods.push("Alt");
  if (e.shiftKey) mods.push("Shift");
  let key = CODES[e.code];
  if (!key) key = /^Key[A-Z]$/.test(e.code || "") ? e.code.slice(3) : e.key.length === 1 ? e.key.toUpperCase() : e.key;
  return [...mods, key].join("+");
}

/// The command a key runs, or null: for the table and for a test.
export function commandFor(e, editing = text.editing()) {
  const v = SHORTCUTS.get(combo(e));
  if (!v) return null;
  return typeof v === "function" ? v(editing) : v;
}

/// Run the key's command, if it has one, and keep the key from the browser. A
/// command this build has not built is named in the status bar instead.
export function handle(e) {
  const id = commandFor(e);
  if (!id) return false;
  e.preventDefault();
  e.stopPropagation();
  const c = command(id);
  if (!c) { status(`${combo(e)} is not available in this build`); return true; }
  if (c.enabled && !c.enabled()) {
    const label = (c.label || id).replace(/\s*\(.*\)\s*$/, "");
    status(c.reason ? `${label} is not available yet` : `${label} is not available here`);
    return true;
  }
  run(id);
  return true;
}

/// Every bound key and its command id, for the Keyboard Shortcuts list.
export function bindings() {
  return [...SHORTCUTS].map(([k, v]) => [k, typeof v === "function" ? `${v(true)} / ${v(false)}` : v]);
}
