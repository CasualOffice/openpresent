// The selection: what on the current slide the user has picked. Shaped after
// OpenCalc's `webapp/editor.selection.js` (owner instruction 42): one module owns
// it, and the modules that act on it listen for changes rather than keeping a copy.
//
// Shared by text editing (`VS-05`) and moving shapes (`VS-06`). `object` is the
// picked shape as `[part, node]`, the engine's `ObjectId`; `editing` says its text
// is open with a caret, `anchor` and `focus` being character offsets in its body
// (`docs/24`). Nothing here holds document text.

const listeners = new Set();

export const selection = {
  slide: -1,
  object: null,     // [part, node] or null
  editing: false,
  anchor: 0,
  focus: 0,
};

/// Change the selection and tell every listener.
export function select(patch) {
  Object.assign(selection, patch);
  for (const fn of listeners) fn(selection);
}

/// Pick nothing.
export function clearSelection() {
  select({ object: null, editing: false, anchor: 0, focus: 0 });
}

export function onSelectionChange(fn) {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

/// The text range as `[start, end)`, start first.
export function textRange() {
  return [Math.min(selection.anchor, selection.focus), Math.max(selection.anchor, selection.focus)];
}
