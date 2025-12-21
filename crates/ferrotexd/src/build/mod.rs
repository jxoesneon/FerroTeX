use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct BuildRequest {
    pub document_uri: tower_lsp::lsp_types::Url,
    pub workspace_root: Option<std::path::PathBuf>,
}

#[derive(Debug)]
pub struct BuildLog {
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub enum BuildStatus {
    Success(std::path::PathBuf), // Path to artifact (PDF)
    Failure(BuildLog),
}

#[async_trait]
pub trait BuildEngine: Send + Sync {
    /// uniquely identifies the engine (e.g. "latexmk", "tectonic")
    fn name(&self) -> &str;

    /// Execute the build for the given request
    async fn build(&self, request: &BuildRequest) -> Result<BuildStatus>;
}

pub mod latexmk;
