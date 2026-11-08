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
// Building (`VS-08`): `{ op: "blank" }` opens the built-in new presentation, with
// `template: n` in built-in theme n (`PAR-18`), and
// `{ op: "build", fn, args }` runs one slide or insert call; undo and redo also
// answer the slide count, since they may add or remove slides.
// Text (`VS-05`): `{ op: "text", fn, args }`, one of the bridge's `text_*` calls by name.
// The slide show (`VS-07`): `{ op: "plan" }`, each slide's hidden flag and transition,
// and `{ op: "notes", index }`, a slide's speaker notes as text. Design (`VS-12`):
// `{ op: "design", fn, args }` and `{ op: "themes" }`. Shapes (`SHAPE-01`): `{ op: "icons",
// size }`, every preset's name and icon path. Out: `{ id, ok: true, ... }` or
// `{ id, ok: false, error }`, where `error` is the engine's message with its code.
// Nothing here fetches anything but the engine itself and the CJK font slices under
// `fonts/cjk/` a drawn slide asks for (`T2D1-04`), from this origin.

import init, {
  open, slide_count, slide_size, render_slide_png, slide_report, save_deck, export_pdf, version,
  slide_shapes, group_shapes, group_selection, ungroup_shape, shape_alt_text, set_shape_alt_text, set_shape_transform, delete_shape, undo, redo, undo_state,
  text_hit, text_offset_near, text_caret, text_selection, text_navigate, text_length,
  text_insert, text_delete, text_toggle, text_format_state, edit_break_typing,
  show_plan, show_advance, slide_transition, transition_direction, slide_notes,
  design_themes, design_apply_theme, design_slide_size, design_background,
  open_blank, open_template, slide_layouts, slide_layout, new_slide, duplicate_slide, delete_slide, move_slide,
  hidden_slides, set_slide_hidden, insert_shape, insert_picture,
  preset_names, preset_icon, insert_preset, shape_handles, drag_shape_handle,
  set_shape_fill, set_shape_outline, set_shape_style, slide_palette,
  transition_choice, set_slide_transition, set_transition_advance, transition_to_all, slide_animations, shape_name,
  add_animation, replace_animation, remove_animation, move_animation, set_animation_start,
  set_animation, paint_animation,
  render_slide_png_hiding,
  insert_table, table_shape, table_cell_text, set_table_cell_text, edit_table, table_styles,
  table_style, set_table_style, insert_chart, chart_data, set_chart_data,
  collab_begin, collab_end, collab_flush, collab_acknowledge, collab_unacknowledged,
  collab_receive, collab_replay, collab_map_offset, collab_presence, protocol_version,
  text_run_state, text_set_run, text_grow, text_set_paragraphs, text_slice, text_paste_rich, text_change_case, text_set_font,
  slide_outline, text_replace_all,
  font_families, font_css_stack, font_face_bytes,
  copy_shapes, clipboard_shapes, paste_shapes, duplicate_shapes, set_shape_transforms, delete_shapes,
  copy_slides, clipboard_slides, paste_slides,
  picture_crop, set_picture_crop, reset_picture, set_picture_style,
  insert_wordart, wordart_styles, set_shape_link, shape_link,
  slide_sections, section_add, section_rename, section_remove, header_footer,
  arrange_order, arrange_flip, find_text, replace_text_all, replace_text_one, reset_slide,
  slide_comments, comment_counts, add_comment, reply_comment, resolve_comment, delete_comment,
  notes_body, text_body_runs, selection_list, rename_shape, set_shape_hidden, move_shape_z,
  document_info, set_document_property, set_custom_property, remove_custom_property,
  save_deck_as, edits_applied,
  slide_font_needs, register_font_slice, set_cjk_default_region,
} from "./pkg/casual_slide_wasm.js";

// The File tab (`FILE-01`, `editor.file.js`): `{ id, op: "file", fn, args }` calls one of
// these by name and answers `{ value, undo, edits }`; File > Info's properties are edits
// with an undo step each. `{ op: "save-as", ext, dropMacros }` answers the bytes, and
// `{ op: "edits" }` the edit counter autosave polls (`SAVE-01`).
const FILE = { document_info, set_document_property, set_custom_property, remove_custom_property };

// Co-editing (`COL-04`): what `collab.js` asks of the engine.
const COLLAB = {
  collab_begin, collab_end, collab_flush, collab_acknowledge, collab_unacknowledged,
  collab_receive, collab_replay, collab_map_offset, collab_presence, protocol_version,
};

// Design (`VS-12`): `{ id, op: "design", fn, args }` calls one of these by name and
// answers `{ value, undo, size }`; `{ op: "themes" }` lists the built-in themes.
// Comments, notes and the Selection Pane (`UX-011`). Document text goes back as JSON
// strings, which the page sets with `textContent` only.
const REVIEW = {
  slide_comments, comment_counts, add_comment, reply_comment, resolve_comment, delete_comment,
  notes_body, text_body_runs, selection_list, rename_shape, set_shape_hidden, move_shape_z,
};

const DESIGN = { design_apply_theme, design_slide_size, design_background };

// Text editing (`VS-05`): `{ id, op: "text", fn, args }` calls one of these by name and
// answers `{ value, undo }`, `undo` as for the other edits.
const TEXT = {
  text_hit, text_offset_near, text_caret, text_selection, text_navigate, text_length,
  text_insert, text_delete, text_toggle, text_format_state, edit_break_typing,
  // The Home tab's Font and Paragraph groups, and Copy of text (`UX-003`).
  text_run_state, text_set_run, text_grow, text_set_paragraphs, text_slice, text_paste_rich, text_change_case, text_set_font,
  // Outline View, Find and Replace (`UX-006`).
  slide_outline, text_replace_all, find_text, replace_text_all, replace_text_one,
};

// Building a deck (`VS-08`): `{ id, op: "build", fn, args }` calls one of these by name
// and answers `{ value, count, undo }`, `count` being the deck's slides afterwards.
// Rich objects (`VS-11`) go the same way: tables and charts.
const BUILD = {
  slide_layouts, slide_layout, new_slide, duplicate_slide, delete_slide, move_slide,
  hidden_slides, set_slide_hidden, insert_shape, insert_picture,
  // Shapes (`SHAPE-01`): insert a preset, its yellow handles, Shape Format.
  insert_preset, shape_handles, drag_shape_handle, set_shape_fill, set_shape_outline,
  set_shape_style, slide_palette, edit_break_typing,
  // The shape clipboard and multiple selection (`UX-003`).
  copy_shapes, clipboard_shapes, paste_shapes, duplicate_shapes, set_shape_transforms, delete_shapes,
  copy_slides, clipboard_slides, paste_slides,
  // Picture Format (`UX-003`).
  picture_crop, set_picture_crop, reset_picture, set_picture_style,
  // Arrange, Reset Slide and WordArt (`UX-006`).
  arrange_order, arrange_flip, reset_slide, insert_wordart, wordart_styles,
  // Insert > Link on a shape (`UX-006`): stored, never fetched.
  set_shape_link, shape_link,
  // Home > Section, Insert > Header & Footer (`UX-006`).
  slide_sections, section_add, section_rename, section_remove, header_footer,
  // A second click inside a group (`UX-015`); Group and Ungroup (`PAR-03`).
  group_shapes, group_selection, ungroup_shape,
  // The Alt Text pane (`PAR-44`).
  shape_alt_text, set_shape_alt_text,
  // Tables and charts (`VS-11`): `editor.objects.js` asks for them as `build`; they sat
  // under `animate`, so every table and chart call was refused (`UX-015`).
  insert_table, table_shape, table_cell_text, set_table_cell_text, edit_table, table_styles,
  table_style, set_table_style, insert_chart, chart_data, set_chart_data,
};

/// Arguments that cross as typed arrays: the bridge takes `Vec<u32>` and `Vec<f64>`.
const NODE_LISTS = new Set(["copy_shapes", "duplicate_shapes", "delete_shapes", "group_selection"]);

// Transitions and animations (`VS-10`): `{ id, op: "animate", fn, args }` calls one of
// these by name and answers `{ value, undo }`; `{ op: "png-hiding", index, width, nodes }`
// draws a slide with those shapes hidden, for the show's animation steps.
const ANIMATE = {
  transition_choice, set_slide_transition, set_transition_advance, transition_to_all, slide_animations, shape_name,
  add_animation, replace_animation, remove_animation, move_animation, set_animation_start,
  set_animation, paint_animation,
};

const ready = init({ module_or_path: new URL("./pkg/casual_slide_wasm_bg.wasm", import.meta.url) })
  // Han text in a Latin font takes the viewer's region, as OpenCalc's editor sets it.
  .then((r) => { try { set_cjk_default_region(self.navigator?.language || ""); } catch {} return r; });

// ---------------------------------------------------------------------------
// Japanese, Chinese and Korean faces (`T2D1-04`)
//
// From OpenCalc's `editor.paint.js` and its PDF export's font loop: the faces are
// WOFF2 slices under `fonts/cjk/`, never in the WebAssembly binary, and the
// manifest says which characters each slice holds. The engine draws the slide here,
// so after a drawing the worker asks which slices it wanted (`slide_font_needs`),
// fetches those files, registers them and draws again. A deployment without the
// slices draws as before, with the characters left blank and reported.
// ---------------------------------------------------------------------------

const CJK_BASE = new URL("./fonts/cjk/", import.meta.url);
let cjkManifest = null;
const cjkRegistered = new Set();
const cjkTried = new Set();
const cjkRanges = new Map();

function loadCjkManifest() {
  cjkManifest ??= fetch(new URL("manifest.json", CJK_BASE))
    .then((r) => (r.ok ? r.json() : null))
    .catch(() => null);
  return cjkManifest;
}

/// The slice of `region` holding code point `cp`, or -1: a binary search over the
/// manifest's ranges, parsed once per region.
function sliceFor(manifest, region, cp) {
  let spans = cjkRanges.get(region);
  if (!spans) {
    spans = [];
    (manifest.regions?.[region] || []).forEach((line, i) => {
      for (const part of String(line || "").split(",")) {
        const [a, b] = part.split("-");
        const lo = parseInt(a, 16);
        const hi = b === undefined ? lo : parseInt(b, 16);
        if (Number.isFinite(lo) && Number.isFinite(hi) && lo <= hi) spans.push([lo, hi, i]);
      }
    });
    spans.sort((x, y) => x[0] - y[0]);
    cjkRanges.set(region, spans);
  }
  let lo = 0, hi = spans.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (spans[mid][0] <= cp) lo = mid + 1; else hi = mid;
  }
  const s = spans[lo - 1];
  return s && cp <= s[1] ? s[2] : -1;
}

/// Fetch and register the slices `needs` names (`[{chain, chars}]`). Each character
/// is looked for once per variant, so a slice the deployment lacks is not asked for
/// on every drawing. Whether anything new was registered.
async function loadCjkFor(needs) {
  const m = await loadCjkManifest();
  if (!m || !m.variants) return false;
  const files = new Map();
  for (const { chain, chars } of needs) {
    if (!Array.isArray(chain) || typeof chars !== "string") continue;
    for (const ch of chars) {
      const asked = chain[0] + "\u0000" + ch;
      if (cjkTried.has(asked)) continue;
      cjkTried.add(asked);
      const cp = ch.codePointAt(0);
      for (const key of chain) {
        const v = m.variants[key];
        if (!v) continue;
        const i = sliceFor(m, v.region, cp);
        if (i < 0) continue;
        const file = `${key}/${String(i).padStart(3, "0")}`;
        if (!cjkRegistered.has(file)) files.set(file, [key, i]);
        break;
      }
    }
  }
  let added = false;
  await Promise.all([...files].map(async ([file, [key, i]]) => {
    try {
      const sha = m.variants[key]?.sha256_16?.[i] ?? "";
      const r = await fetch(new URL(`${file}.woff2${sha ? `?h=${sha}` : ""}`, CJK_BASE));
      if (!r.ok) return;
      register_font_slice(key, new Uint8Array(await r.arrayBuffer()));
      cjkRegistered.add(file);
      added = true;
    } catch { /* left blank and reported, as before */ }
  }));
  return added;
}

/// Draw with `draw`, and once more if the drawing wanted CJK slices that have now
/// been loaded.
async function drawWithFonts(draw) {
  const png = draw();
  let needs = [];
  try { needs = JSON.parse(slide_font_needs()); } catch { needs = []; }
  if (needs.length && (await loadCjkFor(needs))) return draw();
  return png;
}

function failure(e) {
  // A `JsError` from the bridge carries `[CS-…] what happened`; anything else
  // (an engine trap) is named as such, without its stack.
  if (e && typeof e.message === "string") return e.message;
  return "the engine stopped unexpectedly";
}

// Every successful answer carries the engine's edit counter as it stands
// (`editsNow`), so the page knows an edit happened the moment it did, without a
// poll: the leave warning reads it (`PAR-01`). Before a deck is open there is none.
const post = self.postMessage.bind(self);
self.postMessage = (m, transfer) => {
  if (m && m.ok) {
    try { m.editsNow = edits_applied(); } catch { /* no deck yet */ }
  }
  post(m, transfer);
};

self.onmessage = async (event) => {
  const msg = event.data || {};
  const { id } = msg;
  try {
    await ready;
    if (msg.op === "open" || msg.op === "blank") {
      // `blank` is File > New, and the deck the editor starts with (`VS-08`).
      if (msg.op === "blank" && Number.isInteger(msg.template)) open_template(msg.template);
      else if (msg.op === "blank") open_blank(); else open(new Uint8Array(msg.bytes));
      const size = slide_size();
      self.postMessage({ id, ok: true, count: slide_count(), width: size[0], height: size[1], version: version() });
    } else if (msg.op === "png") {
      const png = await drawWithFonts(() => render_slide_png(msg.index, msg.width));
      self.postMessage({ id, ok: true, png: png.buffer }, [png.buffer]);
    } else if (msg.op === "png-hiding") {
      const nodes = new Uint32Array(msg.nodes || []);
      const png = await drawWithFonts(() => render_slide_png_hiding(msg.index, msg.width, nodes));
      self.postMessage({ id, ok: true, png: png.buffer }, [png.buffer]);
    } else if (msg.op === "animate") {
      const fn = Object.hasOwn(ANIMATE, msg.fn) ? ANIMATE[msg.fn] : null;
      if (!fn) throw new Error("unknown request");
      let value = fn(...(Array.isArray(msg.args) ? msg.args : []));
      if (ArrayBuffer.isView(value)) value = Array.from(value);
      self.postMessage({ id, ok: true, value, undo: Array.from(undo_state()) });
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
    } else if (msg.op === "review") {
      const fn = Object.hasOwn(REVIEW, msg.fn) ? REVIEW[msg.fn] : null;
      if (!fn) throw new Error("unknown request");
      let value = fn(...(Array.isArray(msg.args) ? msg.args : []));
      if (ArrayBuffer.isView(value)) value = Array.from(value);
      self.postMessage({ id, ok: true, value, undo: Array.from(undo_state()) });
    } else if (msg.op === "undo" || msg.op === "redo") {
      const slide = msg.op === "undo" ? undo() : redo();
      // -2: every slide changed, and perhaps the size with them (`VS-12`).
      self.postMessage({ id, ok: true, slide, undo: Array.from(undo_state()), size: Array.from(slide_size()) });
    } else if (msg.op === "design") {
      const fn = Object.hasOwn(DESIGN, msg.fn) ? DESIGN[msg.fn] : null;
      if (!fn) throw new Error("unknown request");
      const value = fn(...(Array.isArray(msg.args) ? msg.args : []));
      self.postMessage({ id, ok: true, value, undo: Array.from(undo_state()), size: Array.from(slide_size()) });
    } else if (msg.op === "fonts") {
      // The Font box's list (`UX-003`): OpenCalc's `font_families`, each with the CSS
      // stack it is shown in, and the bundled faces' bytes for the page to register.
      const list = JSON.parse(font_families());
      const faces = {};
      for (const f of list) {
        if (!faces[f.f]) faces[f.f] = font_face_bytes(f.f).buffer;
      }
      const stacks = list.map((f) => font_css_stack(f.n));
      self.postMessage({ id, ok: true, list, stacks, faces }, Object.values(faces));
    } else if (msg.op === "themes") {
      self.postMessage({ id, ok: true, themes: design_themes() });
    } else if (msg.op === "text") {
      const fn = Object.hasOwn(TEXT, msg.fn) ? TEXT[msg.fn] : null;
      if (!fn) throw new Error("unknown request");
      const args = Array.isArray(msg.args) ? msg.args.slice() : [];
      // Rich paste (`PAR-11`): the runs' looks and the paragraphs' lists as typed arrays.
      if (msg.fn === "text_paste_rich") { args[5] = new Int32Array(args[5] || []); args[6] = new Uint8Array(args[6] || []); }
      let value = fn(...args);
      if (ArrayBuffer.isView(value)) value = Array.from(value);
      self.postMessage({ id, ok: true, value, undo: Array.from(undo_state()) });
    } else if (msg.op === "plan") {
      // Two numbers a slide (`show_plan`), and the effect's name where the show
      // plays a fade in its place, for the report at the end.
      const plan = Array.from(show_plan());
      const effects = {}, directions = {};
      for (let i = 0; i * 2 < plan.length; i++) {
        if (plan[i * 2] & 2) { effects[i] = slide_transition(i); directions[i] = transition_direction(i); }
      }
      self.postMessage({ id, ok: true, plan, effects, directions, advance: Array.from(show_advance()) });
    } else if (msg.op === "notes") {
      self.postMessage({ id, ok: true, notes: slide_notes(msg.index) });
    } else if (msg.op === "icons") {
      // The Shapes gallery (`SHAPE-01`): every preset's name and its icon's path data,
      // at `size` units square, drawn by the geometry engine.
      const size = Number(msg.size) || 32;
      const names = Array.from(preset_names());
      self.postMessage({ id, ok: true, names, icons: names.map((n) => preset_icon(n, size, size)) });
    } else if (msg.op === "build") {
      const fn = Object.hasOwn(BUILD, msg.fn) ? BUILD[msg.fn] : null;
      if (!fn) throw new Error("unknown request");
      const args = Array.isArray(msg.args) ? msg.args.slice() : [];
      // A picture's bytes cross as an ArrayBuffer.
      if (msg.fn === "insert_picture") args[1] = new Uint8Array(args[1]);
      if (NODE_LISTS.has(msg.fn)) args[1] = new Uint32Array(args[1] || []);
      if (msg.fn === "set_shape_transforms") args[1] = new Float64Array(args[1] || []);
      // The thumbnail pane's slide clipboard (`PAR-10`).
      if (msg.fn === "copy_slides") args[0] = new Uint32Array(args[0] || []);
      let value = fn(...args);
      if (ArrayBuffer.isView(value)) value = Array.from(value);
      // An engine `i64` (`slide_layout`) arrives as a BigInt, which no page caller
      // expects and which a `u32` argument refuses: New Slide failed on every deck
      // with slides (`PAR-16`). Every such value fits a double.
      if (typeof value === "bigint") value = Number(value);
      self.postMessage({ id, ok: true, value, count: slide_count(), undo: Array.from(undo_state()) });
    } else if (msg.op === "undo" || msg.op === "redo") {
      const slide = msg.op === "undo" ? undo() : redo();
      // -2: every slide changed, and perhaps the size with them (`VS-12`).
      self.postMessage({ id, ok: true, slide, count: slide_count(), undo: Array.from(undo_state()), size: Array.from(slide_size()) });
    } else if (msg.op === "collab") {
      const fn = Object.hasOwn(COLLAB, msg.fn) ? COLLAB[msg.fn] : null;
      if (!fn) throw new Error("unknown request");
      const args = Array.isArray(msg.args) ? msg.args.slice() : [];
      if (msg.fn === "collab_presence") args[1] = new Uint32Array(args[1] || []);
      if (msg.fn === "collab_begin" || msg.fn === "collab_acknowledge") args[0] = BigInt(args[0]);
      if (msg.fn === "collab_begin" || msg.fn === "collab_acknowledge" || msg.fn === "collab_receive") args[1] = BigInt(args[1]);
      let value = fn(...args);
      if (ArrayBuffer.isView(value)) value = Array.from(value);
      self.postMessage({ id, ok: true, value, count: slide_count(), undo: Array.from(undo_state()) });
    } else if (msg.op === "file") {
      const fn = Object.hasOwn(FILE, msg.fn) ? FILE[msg.fn] : null;
      if (!fn) throw new Error("unknown request");
      const value = fn(...(Array.isArray(msg.args) ? msg.args : []));
      self.postMessage({ id, ok: true, value, undo: Array.from(undo_state()), edits: edits_applied() });
    } else if (msg.op === "save-as") {
      const bytes = save_deck_as(String(msg.ext || ""), !!msg.dropMacros);
      self.postMessage({ id, ok: true, bytes: bytes.buffer }, [bytes.buffer]);
    } else if (msg.op === "edits") {
      self.postMessage({ id, ok: true, edits: edits_applied() });
    } else if (msg.op === "report") {
      self.postMessage({ id, ok: true, lines: slide_report(msg.index) });
    } else {
      self.postMessage({ id, ok: false, error: "unknown request" });
    }
  } catch (e) {
    self.postMessage({ id, ok: false, error: failure(e) });
  }
};
