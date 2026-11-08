# Provenance

Started from `crates/casual-calc-wasm` in OpenCalc (`../opencalc`) at commit `b23c67c`, on
2026-11-08 (owner instruction 42). Same owner, Apache-2.0.

Taken from it, and changed only where slides need it:

- The shape of the bridge: a `cdylib` built by `wasm-pack --target web`, one open document
  held in a thread-local session so the browser keeps no model of its own, free
  `#[wasm_bindgen]` functions rather than exported classes, and errors returned as
  `JsError` carrying the engine's stable code.
- `[package.metadata.wasm-pack.profile.release] wasm-opt = false`, for the reason OpenCalc
  gives: cached `wasm-opt` binaries reject the bulk-memory ops modern Rust emits.

Not taken at first: everything that edits, saves, collaborates or runs code; Save (`VS-04`) and shape edits (`VS-06`, below) have since arrived. This is the first
vertical slice (owner instruction 44), which opens and shows a deck; OpenCalc's session
API arrives here piece by piece with the features that need it.

New here: `Viewer`, which keeps the bridge's logic in plain Rust so it is tested natively;
the width bound and the pixel bound; RGBA output decoded from the renderer's PNG; and the
per-slide report.

The edit ops and their undo history live in `casual-slide-transaction` (its own
`PROVENANCE.md`): slice 6's shape ops (`VS-06`), first written here as `src/txn.rs`, moved
there beside slice 5's text ops (`VS-05`), so there is one op set and one undo stack.
`src/edit.rs` (`VS-05`) is new: hit-testing a click into a text box, caret and selection
geometry from `casual-slide-text`'s caret lines, and the text edits through the history.
