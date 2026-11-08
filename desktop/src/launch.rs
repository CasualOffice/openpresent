//! A file the operating system hands to this application.
//!
//! Copied from OpenCalc's `desktop/src/launch.rs` at `5d42a07`
//! (desktop/PROVENANCE.md). Changed: which files open is a fixed list, the
//! extensions the web viewer's file picker accepts (`webapp/editor.html`),
//! because this shell depends on no engine crate to ask.
//!
//! Declaring `bundle.fileAssociations` alone is worse than nothing: a
//! registered association the app cannot honour turns a double-click into an
//! empty window. So the opening works first, and the association is declared
//! against it. Three ways in:
//!
//! * **Windows and Linux** put the path in `argv`. A `.desktop` file's `Exec`
//!   line may spell it `%F` (a path) or `%U` (a `file://` URL), so both are
//!   accepted.
//! * **macOS never uses `argv` for this.** Finder sends `application:openURLs:`,
//!   which Tauri surfaces as `RunEvent::Opened`, and it can arrive **before**
//!   there is a window, so a path is queued in [`PendingOpen`] until the webview
//!   says it is ready.
//! * **A second activation** of a running app, another file double-clicked
//!   while the window is open, arrives the same way and must not be dropped.
//!
//! **Nothing here reads a file.** A path may enter the process only from the
//! platform, and never goes back out to the webview.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The extensions this app opens, lower-case and without a leading dot: the
/// same list the web viewer's picker accepts, so the desktop opens what the web
/// opens. `bundle.fileAssociations` in `tauri.conf.json` declares these.
pub const OPENABLE_EXTENSIONS: [&str; 6] = ["pptx", "pptm", "ppsx", "ppsm", "potx", "potm"];

/// Does `path` name a file this app opens? Asked of the extension only; whether
/// the bytes really are a presentation is the engine's to decide, in the page.
#[must_use]
pub fn opens_here(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            OPENABLE_EXTENSIONS
                .iter()
                .any(|known| known.eq_ignore_ascii_case(ext))
        })
}

/// The file named on the command line, if it names one.
///
/// * `argv[0]` is skipped: it is this binary.
/// * Anything beginning with `-` is skipped: Finder's `-psn_…`, or a flag.
/// * A `file://` argument is decoded, because a `.desktop` entry written with
///   `%U` delivers one.
/// * The first argument that survives and [`opens_here`] wins: one window
///   shows one deck.
#[must_use]
pub fn path_from_args<I: IntoIterator<Item = OsString>>(args: I) -> Option<PathBuf> {
    args.into_iter()
        .skip(1)
        .filter_map(|arg| {
            // The UTF-8 question is asked of the flag and URL rules only: a
            // Linux filename is bytes and need not be UTF-8.
            match arg.to_str() {
                Some(text) if text.starts_with('-') => None,
                Some(text) if text.starts_with("file:") => strip_file_url(text).map(PathBuf::from),
                _ => Some(PathBuf::from(arg)),
            }
        })
        .find(|path| opens_here(path))
}

/// `file:///Users/a/b.pptx` as a path, or `None` for a URL naming another
/// machine.
fn strip_file_url(text: &str) -> Option<String> {
    let rest = text.strip_prefix("file://")?;
    let rest = rest
        .strip_prefix("localhost/")
        .map(|r| format!("/{r}"))
        .unwrap_or_else(|| rest.to_owned());
    if !rest.starts_with('/') {
        return None;
    }
    Some(percent_decode(&rest))
}

/// `%20` back to a space. A `%` not followed by two hex digits is kept, since a
/// filename with a percent sign is likelier than a truncated escape.
fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        if b == b'%' {
            let hi = bytes.get(i + 1).and_then(|c| (*c as char).to_digit(16));
            let lo = bytes.get(i + 2).and_then(|c| (*c as char).to_digit(16));
            if let (Some(hi), Some(lo)) = (hi, lo) {
                // Two hex digits are at most 255.
                out.push(u8::try_from(hi * 16 + lo).unwrap_or(b'%'));
                i += 3;
                continue;
            }
        }
        out.push(b);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A file the platform handed over, waiting for a window to open it in.
///
/// Readiness lives beside the queue, under the same lock, and is set by the
/// webview collecting rather than by the shell guessing, so every path is
/// delivered by exactly one of two routes: the webview's collection when it
/// becomes ready, or the nudge [`queue`](Self::queue) asks for when it already
/// was.
#[derive(Debug, Default)]
pub struct PendingOpen {
    queued: Option<PathBuf>,
    webview_ready: bool,
}

impl PendingOpen {
    /// Hold `path` for the window. Returns `true` when the webview is already
    /// collecting, so the caller must nudge it. A second file replaces the
    /// first: one window shows one deck.
    pub fn queue(&mut self, path: PathBuf) -> bool {
        self.queued = Some(path);
        self.webview_ready
    }

    /// The webview is ready and collects whatever is waiting. Marks readiness
    /// whether or not anything is queued, so a later file can be nudged.
    pub fn take(&mut self) -> Option<PathBuf> {
        self.webview_ready = true;
        self.queued.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(raw: &[&str]) -> Vec<OsString> {
        raw.iter().map(OsString::from).collect()
    }

    #[test]
    fn the_path_comes_out_of_argv() {
        assert_eq!(
            path_from_args(args(&["openpresent-desktop", "/tmp/deck.pptx"])),
            Some(PathBuf::from("/tmp/deck.pptx")),
        );
        assert_eq!(path_from_args(args(&["openpresent-desktop"])), None);
        assert_eq!(
            path_from_args(args(&["openpresent-desktop", "-psn_0_774242"])),
            None
        );
        assert_eq!(
            path_from_args(args(&[
                "openpresent-desktop",
                "--self-check",
                "/tmp/a.pptx"
            ])),
            Some(PathBuf::from("/tmp/a.pptx")),
        );
        assert_eq!(
            path_from_args(args(&["openpresent-desktop", "--open=/tmp/a.pptx"])),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_filename_that_is_not_utf8_still_opens() {
        use std::os::unix::ffi::OsStringExt;
        let raw = OsString::from_vec(b"/tmp/caf\xff.pptx".to_vec());
        assert!(raw.to_str().is_none());
        assert_eq!(
            path_from_args(vec![OsString::from("openpresent-desktop"), raw.clone()]),
            Some(PathBuf::from(raw)),
        );
    }

    #[test]
    fn only_presentations_open() {
        for ext in OPENABLE_EXTENSIONS {
            assert!(opens_here(Path::new(&format!("/tmp/deck.{ext}"))), ".{ext}");
        }
        assert!(opens_here(Path::new("/tmp/DECK.PPTX")));
        for ext in ["ppt", "key", "odp", "pdf", "exe", "xlsx"] {
            assert!(
                !opens_here(Path::new(&format!("/tmp/deck.{ext}"))),
                ".{ext}"
            );
        }
        assert!(!opens_here(Path::new("/tmp/deck")));
    }

    #[test]
    fn a_file_url_from_a_desktop_entry_is_a_path() {
        assert_eq!(
            path_from_args(args(&["app", "file:///home/a/Q3%20review.pptx"])),
            Some(PathBuf::from("/home/a/Q3 review.pptx")),
        );
        assert_eq!(
            path_from_args(args(&["app", "file://localhost/home/a/b.pptx"])),
            Some(PathBuf::from("/home/a/b.pptx")),
        );
        assert_eq!(
            path_from_args(args(&["app", "file://server/share/b.pptx"])),
            None
        );
        // A stray percent is kept, not dropped.
        assert_eq!(
            path_from_args(args(&["app", "file:///home/a/100%.pptx"])),
            Some(PathBuf::from("/home/a/100%.pptx")),
        );
    }

    #[test]
    fn a_file_arriving_before_the_window_waits_and_a_later_one_nudges() {
        let mut pending = PendingOpen::default();
        // macOS first launch: queued before the webview exists, no nudge.
        assert!(!pending.queue(PathBuf::from("/a.pptx")));
        assert_eq!(pending.take(), Some(PathBuf::from("/a.pptx")));
        // Second activation: the webview is collecting, so it must be nudged.
        assert!(pending.queue(PathBuf::from("/b.pptx")));
        assert_eq!(pending.take(), Some(PathBuf::from("/b.pptx")));
        assert_eq!(pending.take(), None);
    }
}
