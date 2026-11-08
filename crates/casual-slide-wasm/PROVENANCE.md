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

Not taken: everything that edits, saves, collaborates or runs code. This is the first
vertical slice (owner instruction 44), which opens and shows a deck; OpenCalc's session
API arrives here piece by piece with the features that need it.

New here: `Viewer`, which keeps the bridge's logic in plain Rust so it is tested natively;
the width bound and the pixel bound; RGBA output decoded from the renderer's PNG; and the
per-slide report.
