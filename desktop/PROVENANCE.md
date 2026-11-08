# Provenance

Copied from OpenCalc (`../opencalc/desktop` and the `desktop` job of its
`.github/workflows/ci.yml`) at commit `5d42a07` on 2026-11-08, same owner, Apache-2.0
(owner instruction 42), for `VS-03`.

| File | From OpenCalc | Changed |
| --- | --- | --- |
| `Cargo.toml` | `desktop/Cargo.toml` | Renamed; no engine crate, since the engine runs in the webview; no clipboard plugin |
| `build.rs` | `desktop/build.rs` | `OPENPRESENT_SKIP_ASSET_CHECK`; no CJK font slices to leave out |
| `src/assets.rs` | `desktop/src/assets.rs` | This engine's file names, and the worker |
| `src/launch.rs` | `desktop/src/launch.rs` | Openable extensions are the web picker's list, not asked of an SDK; `PendingOpen` cut to `queue` and `take`; tests rewritten for `.pptx` |
| `src/lib.rs` | `desktop/src/lib.rs`, `dialog::base_name` | No session or document: only `base_name` |
| `src/main.rs` | `desktop/src/main.rs` | Cut to the bridge, `native_open`, `take_opened_bytes`, `take_pending_open`, the hand-over and `--self-check`. Not taken yet: the native menu, save, the clipboard, close confirmation. The self-check now requires a drawn slide |
| `tauri.conf.json` | `desktop/tauri.conf.json` | OpenPresent's name and identifier; PowerPoint associations; `dragDropEnabled: false` so the page's own drop works as on the web; no `?chrome=native` |
| `desktop-entry.desktop` | `desktop/desktop-entry.desktop` | None |
| `.github/workflows/desktop.yml` | `ci.yml`, job `desktop` | Its own workflow; three platforms on pull requests too; the self-check is handed a fixture deck |

`icons/icon.png` and `icons/icon.ico` are new, drawn by a script for this app, not OpenCalc's.
