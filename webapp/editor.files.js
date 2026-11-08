// Opening a file: the picker, and drag and drop onto the window.
//
// Shaped after OpenCalc's `webapp/editor.files.js` at `b23c67c` (webapp/README.md):
// the file is read in the page with `File.arrayBuffer()` and handed to the
// engine; nothing is uploaded and nothing is fetched. What is a presentation is
// the engine's to decide, so any dropped file is passed on and a refusal comes
// back with the engine's code rather than being guessed here from its name.

const byId = (id) => document.getElementById(id);
let onOpen = null;

/// Show the file picker: the platform's own Open panel in the desktop app
/// (`desktop/src/main.rs`, `VS-03`), the browser's picker otherwise.
export function openFilePicker() {
  const native = globalThis.__openpresentNative;
  if (native && native.open) {
    native.open()
      .then((o) => o && onOpen && onOpen(o.name, o.bytes))
      .catch((e) => console.error("[openpresent] open", e));
    return;
  }
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
