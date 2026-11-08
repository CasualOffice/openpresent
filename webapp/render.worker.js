// The engine, off the page's thread. Started from OpenCalc's `webapp/file.worker.js`
// (webapp/README.md): the page posts a deck's bytes here and the WebAssembly
// engine reads them, so a large deck never freezes the window. Here the worker
// also keeps the deck open and draws its slides on request.
//
// Messages in: `{ id, op: "open", bytes }`, `{ id, op: "png", index, width }`,
// `{ id, op: "report", index }`, `{ id, op: "save" }` (the deck through the writer),
// `{ id, op: "text", fn, args }` (a text-editing call, `VS-05`)
// and `{ id, op: "pdf" }` (every slide as one PDF). Out: `{ id, ok: true, ... }` or
// `{ id, ok: false, error }`, where `error` is the engine's message with its code.
// Nothing here fetches anything but the engine itself, from this origin.

import init, {
  open, slide_count, slide_size, render_slide_png, slide_report, save_deck, export_pdf, version,
  text_hit, text_offset_near, text_caret, text_selection, text_navigate, text_length,
  text_insert, text_delete, text_toggle, text_format_state, edit_undo, edit_redo, edit_state,
  edit_break_typing,
} from "./pkg/casual_slide_wasm.js";

// Text editing (`VS-05`): `{ id, op: "text", fn, args }` calls one of these by name,
// and answers `{ value, undo }`, `undo` being `edit_state()` after the call.
const TEXT = {
  text_hit, text_offset_near, text_caret, text_selection, text_navigate, text_length,
  text_insert, text_delete, text_toggle, text_format_state, edit_undo, edit_redo,
  edit_break_typing,
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
    } else if (msg.op === "text") {
      const fn = Object.hasOwn(TEXT, msg.fn) ? TEXT[msg.fn] : null;
      if (!fn) throw new Error("unknown request");
      let value = fn(...(Array.isArray(msg.args) ? msg.args : []));
      if (ArrayBuffer.isView(value)) value = Array.from(value);
      self.postMessage({ id, ok: true, value, undo: edit_state() });
    } else if (msg.op === "report") {
      self.postMessage({ id, ok: true, lines: slide_report(msg.index) });
    } else {
      self.postMessage({ id, ok: false, error: "unknown request" });
    }
  } catch (e) {
    self.postMessage({ id, ok: false, error: failure(e) });
  }
};
