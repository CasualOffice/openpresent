//! The desktop app (VS-03): a window around the web viewer, the platform's own
//! Open panel, and the files the operating system hands it.
//!
//! Copied from OpenCalc's `desktop/src/main.rs` at `5d42a07`
//! (desktop/PROVENANCE.md) and cut to what a viewer needs: the bridge, the
//! native Open, the hand-over of a double-clicked file, and `--self-check`.
//! Not taken yet, because nothing in the viewer needs them: the native menu bar
//! built from the editor's model, save and save-target, the clipboard, and the
//! close-with-unsaved-work question.
//!
//! **The engine is the web's.** The webview loads the same `webapp/` the site
//! serves, and the engine runs in it as WebAssembly, so the desktop draws what
//! the web draws because it is the same code, not a port of it.
//!
//! **Bytes cross the bridge, never paths** (OpenCalc's invariant). A path may
//! enter this process only from the platform, from an Open panel or from the
//! operating system opening a file with this app, and none goes back out. No
//! command takes a path, so the page cannot ask this process to read a file
//! the user did not choose.

// The window is the point of this binary; there is nothing to print.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use openpresent_desktop::base_name;
use openpresent_desktop::launch::{self, OPENABLE_EXTENSIONS, PendingOpen};
use serde::Serialize;
use tauri::ipc::Response;
use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_dialog::DialogExt;

/// What the shell holds between commands: hand-offs, taken rather than read,
/// so a cancelled panel leaves nothing behind.
#[derive(Default)]
struct Shell {
    /// Bytes a panel or a hand-over just read, waiting for the page.
    opened: Mutex<Option<Vec<u8>>>,
    /// A file the operating system asked this app to open.
    pending: Mutex<PendingOpen>,
}

/// A poisoned lock is a panic somewhere else; say so rather than panic too.
fn locked<T>(what: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, String> {
    what.lock()
        .map_err(|_| "the shell's state was left inconsistent by an earlier panic".to_owned())
}

/// What an open found, minus the bytes, which come back raw through
/// [`take_opened_bytes`]: a `Vec<u8>` in a JSON response is one number per byte.
#[derive(Serialize)]
struct Opened {
    name: String,
    size: usize,
}

/// Read `path` and stage its bytes for the page.
fn stage(app: &AppHandle, path: &Path) -> Result<Opened, String> {
    let bytes = std::fs::read(path).map_err(|why| format!("could not read the file: {why}"))?;
    let opened = Opened {
        name: base_name(&path.to_string_lossy()),
        size: bytes.len(),
    };
    *locked(&app.state::<Shell>().opened)? = Some(bytes);
    Ok(opened)
}

/// Raise the platform's Open panel and read what the user picked.
///
/// `async` because the panel is modal, and blocking on the main thread would
/// deadlock the window it is modal to. `Ok(None)` when the user cancels, which
/// is not an error.
#[tauri::command]
async fn native_open(app: AppHandle) -> Result<Option<Opened>, String> {
    let Some(chosen) = app
        .dialog()
        .file()
        .set_title("Open")
        .add_filter("PowerPoint Presentation", &OPENABLE_EXTENSIONS)
        .blocking_pick_file()
    else {
        return Ok(None);
    };
    let path = chosen.into_path().map_err(|why| why.to_string())?;
    stage(&app, &path).map(Some)
}

/// Collect the bytes the last open staged, as a raw `ArrayBuffer`.
#[tauri::command]
fn take_opened_bytes(window: WebviewWindow) -> Result<Response, String> {
    let bytes = locked(&window.state::<Shell>().opened)?
        .take()
        .ok_or_else(|| "no opened file is waiting to be collected".to_owned())?;
    Ok(Response::new(bytes))
}

/// Collect the file the operating system asked this app to open, if any.
///
/// Called once when the bridge is ready, which marks the shell ready so a
/// later file is nudged rather than queued, and again on each nudge.
/// `Ok(None)` is the ordinary answer: most launches open nothing.
#[tauri::command]
async fn take_pending_open(app: AppHandle) -> Result<Option<Opened>, String> {
    let Some(path) = locked(&app.state::<Shell>().pending)?.take() else {
        return Ok(None);
    };
    stage(&app, &path).map(Some)
}

/// The result of `--self-check`, reported from inside the page, and the exit
/// code CI reads. `process::exit`, as OpenCalc found that `AppHandle::exit`
/// left the status at 0 after a failure.
#[tauri::command]
fn self_check_report(failures: Vec<String>) {
    use std::io::Write;
    let code = if failures.is_empty() {
        println!("SELF-CHECK ok");
        0
    } else {
        for f in &failures {
            println!("SELF-CHECK FAILED {f}");
        }
        1
    };
    let _ = std::io::stdout().flush();
    std::process::exit(code);
}

/// The page's half of a file handed over after the page is running: another
/// deck double-clicked while the window is open.
const OPEN_HANDED_OVER: &str = "try { window.__openpresentNative.openHandedOver() } \
     catch (e) { console.error('[openpresent] open-file', e) }";

/// The operating system has asked this app to open `path`. Both platform routes
/// end here, so there is one policy. A path that is not a presentation is
/// dropped in silence: nobody chose it in this app.
fn hand_over(app: &AppHandle, path: PathBuf) {
    if !launch::opens_here(&path) {
        return;
    }
    let shell = app.state::<Shell>();
    let nudge = {
        let Ok(mut pending) = shell.pending.lock() else {
            return;
        };
        pending.queue(path)
    };
    if nudge && let Some(window) = app.get_webview_window("main") {
        let _ = window.eval(OPEN_HANDED_OVER);
    }
}

/// The bridge, `window.__openpresentNative`, injected on every page load so a
/// reload gets it too, and idempotent.
///
/// `open()` raises the native panel; the editor's `file.open` command and
/// Ctrl+O call it when it is present (`webapp/editor.files.js`), and the
/// browser's picker otherwise. `openHandedOver()` collects a double-clicked
/// file. Both end in the editor's own `openDeck(name, bytes)`, the same call a
/// picked or dropped file makes on the web.
const BOOTSTRAP: &str = r#"(function () {
  const invoke = window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke;
  if (!invoke) return;
  const editor = () => window.openpresentEditor;
  // A fresh, exactly-sized buffer: the editor transfers it to the engine's worker.
  const bytesOf = async () => new Uint8Array(await invoke("take_opened_bytes")).slice().buffer;
  const native = {
    async open() {
      const opened = await invoke("native_open");
      if (!opened) return null;
      return { name: opened.name, bytes: await bytesOf() };
    },
    async openHandedOver() {
      const e = editor();
      if (!e || !e.openDeck) return;
      const handed = await invoke("take_pending_open");
      if (!handed) return;
      await e.openDeck(handed.name, await bytesOf());
    },
  };
  window.__openpresentNative = native;
  // Collect a file handed over at launch once the editor has loaded. This first
  // call is also what marks the shell ready to nudge later files.
  let tries = 0;
  const ready = () => {
    if (editor()) {
      native.openHandedOver().catch((e) => console.error("[openpresent] open-file", e));
    } else if (tries++ < 200) {
      setTimeout(ready, 50);
    }
  };
  ready();
})();"#;

/// What `--self-check` runs in the page: the bridge is installed, the editor
/// loaded, and the deck named on the command line was opened through the
/// hand-over route and **drawn** by the engine in this webview. CI runs it on
/// the built Linux binary under xvfb with a fixture deck.
const SELF_CHECK: &str = r#"(async () => {
  const fail = [];
  const invoke = window.__TAURI__.core.invoke;
  const until = async (ok, ms) => {
    const end = Date.now() + ms;
    while (Date.now() < end) { if (ok()) return true; await new Promise((r) => setTimeout(r, 100)); }
    return ok();
  };
  try {
    if (!(await until(() => window.__openpresentNative && window.openpresentEditor, 15000))) {
      if (!window.__openpresentNative) fail.push("window.__openpresentNative is absent: BOOTSTRAP did not run");
      if (!window.openpresentEditor) fail.push("window.openpresentEditor is absent: the editor did not load");
    } else {
      for (const m of ["open", "openHandedOver"]) {
        if (typeof window.__openpresentNative[m] !== "function") fail.push("the bridge has no " + m + "()");
      }
      const img = document.getElementById("slide-img");
      const drawn = await until(() => img && img.src.startsWith("blob:") && img.complete && img.naturalWidth > 0, 60000);
      if (!drawn) {
        const name = document.getElementById("doc-name");
        fail.push("no slide was drawn on the stage within 60 s; the title reads "
                  + JSON.stringify(name ? name.textContent : null)
                  + " and the status " + JSON.stringify((document.getElementById("tb-status") || {}).textContent));
      } else {
        console.log("[openpresent] self-check drew " + img.naturalWidth + "x" + img.naturalHeight);
      }
    }
  } catch (e) {
    fail.push("threw: " + (e && e.message ? e.message : String(e)));
  }
  await invoke("self_check_report", { failures: fail });
})()"#;

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Shell::default())
        .invoke_handler(tauri::generate_handler![
            native_open,
            take_opened_bytes,
            take_pending_open,
            self_check_report,
        ])
        .on_page_load(|webview, payload| {
            if payload.event() == PageLoadEvent::Finished {
                let _ = webview.eval(BOOTSTRAP);
                if std::env::args().any(|a| a == "--self-check") {
                    let _ = webview.eval(SELF_CHECK);
                }
            }
        })
        .setup(|app| {
            // Windows and Linux: a double-click puts the path in `argv`. Queued;
            // the page collects it when it is ready.
            if let Some(path) = launch::path_from_args(std::env::args_os()) {
                hand_over(app.handle(), path);
            }
            Ok(())
        })
        .build(tauri::generate_context!());
    let app = match app {
        Ok(app) => app,
        Err(why) => {
            eprintln!("OpenPresent could not start: {why}");
            std::process::exit(1);
        }
    };
    // `build` + `run` rather than `Builder::run`: `RunEvent` is reachable only
    // from this form, and macOS delivers double-clicked files through it.
    app.run(|_app, _event| {
        // macOS's only route, on first launch and on every later double-click.
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Opened { urls } = &_event {
            for url in urls {
                if let Ok(path) = url.to_file_path() {
                    hand_over(_app, path);
                }
            }
        }
    });
}
