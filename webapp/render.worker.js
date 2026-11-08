// The engine, off the page's thread. Started from OpenCalc's `webapp/file.worker.js`
// (webapp/README.md): the page posts a deck's bytes here and the WebAssembly
// engine reads them, so a large deck never freezes the window. Here the worker
// also keeps the deck open and draws its slides on request.
//
// Messages in: `{ id, op: "open", bytes }`, `{ id, op: "png", index, width }`,
// `{ id, op: "report", index }`, `{ id, op: "save" }` (the deck through the writer)
// and `{ id, op: "pdf" }` (every slide as one PDF). Editing (`VS-06`):
// `{ op: "shapes", index }`, `{ op: "transform", index, node, x, y, cx, cy, rot }`,
// `{ op: "delete", index, node }`, `{ op: "undo" }` and `{ op: "redo" }`; each edit
// answers with `undo: [canUndo, canRedo]`, and undo and redo with the slide changed.
// Text (`VS-05`): `{ op: "text", fn, args }`, one of the bridge's `text_*` calls by name.
// The slide show (`VS-07`): `{ op: "plan" }`, each slide's hidden flag and transition,
// and `{ op: "notes", index }`, a slide's speaker notes as text. Design (`VS-12`):
// `{ op: "design", fn, args }` and `{ op: "themes" }`. Out: `{ id, ok: true, ... }` or
// `{ id, ok: false, error }`, where `error` is the engine's message with its code.
// Nothing here fetches anything but the engine itself, from this origin.

import init, {
  open, slide_count, slide_size, render_slide_png, slide_report, save_deck, export_pdf, version,
  slide_shapes, set_shape_transform, delete_shape, undo, redo, undo_state,
  text_hit, text_offset_near, text_caret, text_selection, text_navigate, text_length,
  text_insert, text_delete, text_toggle, text_format_state, edit_break_typing,
  show_plan, slide_transition, slide_notes,
  design_themes, design_apply_theme, design_slide_size, design_background,
} from "./pkg/casual_slide_wasm.js";

// Design (`VS-12`): `{ id, op: "design", fn, args }` calls one of these by name and
// answers `{ value, undo, size }`; `{ op: "themes" }` lists the built-in themes.
const DESIGN = { design_apply_theme, design_slide_size, design_background };

// Text editing (`VS-05`): `{ id, op: "text", fn, args }` calls one of these by name and
// answers `{ value, undo }`, `undo` as for the other edits.
const TEXT = {
  text_hit, text_offset_near, text_caret, text_selection, text_navigate, text_length,
  text_insert, text_delete, text_toggle, text_format_state, edit_break_typing,
};

const ready = init({ module_or_path: new URL("./pkg/casual_slide_wasm_bg.wasm", import.meta.url) });

function failure(e) {
  // A `JsError` from the bridge carries `[CS-…] what happened`; anything else
  // (an engine trap) is named as such, without its stack.
  if (e && typeof e.message === "string") return e.message;
  return "the engine stopped unexpectedly";
}

self.onmessage = async (event) => {
  const msg = event.data || {};
  const { id } = msg;
  try {
    await ready;
    if (msg.op === "open") {
      open(new Uint8Array(msg.bytes));
      const size = slide_size();
      self.postMessage({ id, ok: true, count: slide_count(), width: size[0], height: size[1], version: version() });
    } else if (msg.op === "png") {
      const png = render_slide_png(msg.index, msg.width);
      self.postMessage({ id, ok: true, png: png.buffer }, [png.buffer]);
    } else if (msg.op === "save") {
      const bytes = save_deck();
      self.postMessage({ id, ok: true, bytes: bytes.buffer }, [bytes.buffer]);
    } else if (msg.op === "pdf") {
      const bytes = export_pdf();
      self.postMessage({ id, ok: true, bytes: bytes.buffer }, [bytes.buffer]);
    } else if (msg.op === "shapes") {
      self.postMessage({ id, ok: true, shapes: Array.from(slide_shapes(msg.index)) });
    } else if (msg.op === "transform") {
      set_shape_transform(msg.index, msg.node, msg.x, msg.y, msg.cx, msg.cy, msg.rot);
      self.postMessage({ id, ok: true, undo: Array.from(undo_state()) });
    } else if (msg.op === "delete") {
      delete_shape(msg.index, msg.node);
      self.postMessage({ id, ok: true, undo: Array.from(undo_state()) });
    } else if (msg.op === "undo" || msg.op === "redo") {
      const slide = msg.op === "undo" ? undo() : redo();
      // -2: every slide changed, and perhaps the size with them (`VS-12`).
      self.postMessage({ id, ok: true, slide, undo: Array.from(undo_state()), size: Array.from(slide_size()) });
    } else if (msg.op === "design") {
      const fn = Object.hasOwn(DESIGN, msg.fn) ? DESIGN[msg.fn] : null;
      if (!fn) throw new Error("unknown request");
      const value = fn(...(Array.isArray(msg.args) ? msg.args : []));
      self.postMessage({ id, ok: true, value, undo: Array.from(undo_state()), size: Array.from(slide_size()) });
    } else if (msg.op === "themes") {
      self.postMessage({ id, ok: true, themes: design_themes() });
    } else if (msg.op === "text") {
      const fn = Object.hasOwn(TEXT, msg.fn) ? TEXT[msg.fn] : null;
      if (!fn) throw new Error("unknown request");
      let value = fn(...(Array.isArray(msg.args) ? msg.args : []));
      if (ArrayBuffer.isView(value)) value = Array.from(value);
      self.postMessage({ id, ok: true, value, undo: Array.from(undo_state()) });
    } else if (msg.op === "plan") {
      // Two numbers a slide (`show_plan`), and the effect's name where the show
      // plays a fade in its place, for the report at the end.
      const plan = Array.from(show_plan());
      const effects = {};
      for (let i = 0; i * 2 < plan.length; i++) {
        if ((plan[i * 2] & 6) === 2) effects[i] = slide_transition(i);
      }
      self.postMessage({ id, ok: true, plan, effects });
    } else if (msg.op === "notes") {
      self.postMessage({ id, ok: true, notes: slide_notes(msg.index) });
    } else if (msg.op === "report") {
      self.postMessage({ id, ok: true, lines: slide_report(msg.index) });
    } else {
      self.postMessage({ id, ok: false, error: "unknown request" });
    }
  } catch (e) {
    self.postMessage({ id, ok: false, error: failure(e) });
  }
};
