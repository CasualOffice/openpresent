// Opening a file: the picker, and drag and drop onto the window.
//
// Shaped after OpenCalc's `webapp/editor.files.js` at `b23c67c` (webapp/README.md):
// the file is read in the page with `File.arrayBuffer()` and handed to the
// engine; nothing is uploaded and nothing is fetched. What is a presentation is
// the engine's to decide, so any dropped file is passed on and a refusal comes
// back with the engine's code rather than being guessed here from its name.

const byId = (id) => document.getElementById(id);
let onOpen = null;

/// Hand a produced file to whatever can put it somewhere (`VS-04`).
///
/// Copied from OpenCalc's `download` in `webapp/editor.clipboard.js` at `b23c67c`
/// (webapp/README.md), with its native hook renamed: Save and every Export funnel
/// through here, so the desktop shell intercepts at this one point. When the
/// shell has installed `window.__openpresentNative`, the bytes and the extension
/// cross to it, never a path, and its `save` decides where the file goes; its
/// promise resolves to the written name, or `null` if the user cancelled. In a
/// browser tab it is an anchor download, and resolves at once.
export function download(data, name, type) {
  const native = window.__openpresentNative;
  if (native && typeof native.save === "function") {
    const dot = name.lastIndexOf(".");
    const ext = dot === -1 ? "" : name.slice(dot + 1);
    return Promise.resolve(native.save(data, ext, name));
  }
  const blob = new Blob([data], { type });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = name;
  a.click();
  // Revoked on the next task, once the click has started the download.
  setTimeout(() => URL.revokeObjectURL(a.href), 0);
  return Promise.resolve(name);
}

/// Show the file picker.
export function openFilePicker() {
  const input = byId("file-input");
  input.value = "";
  input.click();
}

async function openFile(file) {
  if (!file || !onOpen) return;
  const bytes = await file.arrayBuffer();
  await onOpen(file.name, bytes);
}

/// Wire the picker, the empty state's button and the window's drop target.
/// `onOpen(name, arrayBuffer)` opens what the user chose.
export function initFiles(opts) {
  onOpen = opts.onOpen;
  byId("file-input").addEventListener("change", (e) => openFile(e.target.files?.[0]));
  byId("open-button").addEventListener("click", openFilePicker);

  // Drag and drop anywhere on the window. A counter, not a flag: `dragleave`
  // fires on every child the pointer crosses.
  const veil = byId("drop-veil");
  let depth = 0;
  const hasFiles = (e) => [...(e.dataTransfer?.types || [])].includes("Files");
  window.addEventListener("dragenter", (e) => {
    if (!hasFiles(e)) return;
    e.preventDefault();
    depth++;
    veil.hidden = false;
  });
  window.addEventListener("dragover", (e) => {
    if (!hasFiles(e)) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = "copy";
  });
  window.addEventListener("dragleave", (e) => {
    if (!hasFiles(e)) return;
    depth = Math.max(0, depth - 1);
    if (!depth) veil.hidden = true;
  });
  window.addEventListener("drop", (e) => {
    if (!hasFiles(e)) return;
    // Always prevented, so a dropped file never navigates the tab away to itself.
    e.preventDefault();
    depth = 0;
    veil.hidden = true;
    openFile(e.dataTransfer.files?.[0]);
  });
}
