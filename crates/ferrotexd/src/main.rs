//! # FerroTeX LSP Daemon
//!
//! This is the entry point for the FerroTeX Language Server Protocol (LSP) daemon.
//! It initializes the `Backend` and starts the LSP server using `tower-lsp`.

use dashmap::DashMap;
use ferrotexd::{workspace::Workspace, Backend};
use std::sync::{Arc, Mutex};
use tower_lsp::{LspService, Server};

/// The main entry point for the FerroTeX LSP daemon.
///
/// It initializes logging, sets up the LSP service with the `Backend`,
/// and starts the asynchronous server on stdin/stdout.
#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(DashMap::new()),
        workspace: Arc::new(Workspace::new()),
        root_uri: Arc::new(Mutex::new(None)),
        syntax_diagnostics: Arc::new(DashMap::new()),
        package_manager: Arc::new(Mutex::new(
            ferrotex_core::package_manager::PackageManager::new(),
        )),
        package_index: Arc::new(Mutex::new(None)),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
