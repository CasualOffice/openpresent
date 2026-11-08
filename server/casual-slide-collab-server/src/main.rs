//! The runnable collaboration server.
//!
//! `OPENPRESENT_COLLAB_BIND` names the address, `127.0.0.1:8787` by default: on this
//! machine's loopback only, unless an operator says otherwise. There is no TLS here;
//! put it behind a proxy that terminates TLS before exposing it.

use casual_slide_collab_server::{Limits, serve};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let bind =
        std::env::var("OPENPRESENT_COLLAB_BIND").unwrap_or_else(|_| "127.0.0.1:8787".to_owned());
    let listener = match tokio::net::TcpListener::bind(&bind).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("casual-slide-collab-server: cannot listen on {bind}: {e}");
            return std::process::ExitCode::FAILURE;
        }
    };
    eprintln!("casual-slide-collab-server: listening on {bind}");
    // `OPENPRESENT_COLLAB_FILES`: a folder of the editor (`webapp/` with its `pkg/`)
    // to serve beside the session, so a room link opens the editor from here.
    let files = std::env::var_os("OPENPRESENT_COLLAB_FILES").map(std::path::PathBuf::from);
    match serve(listener, Limits::default(), files).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("casual-slide-collab-server: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
