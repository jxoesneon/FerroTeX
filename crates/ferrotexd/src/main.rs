use dashmap::DashMap;
use ferrotexd::{workspace::Workspace, Backend};
use std::sync::{Arc, Mutex};
use tower_lsp::{LspService, Server};

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
