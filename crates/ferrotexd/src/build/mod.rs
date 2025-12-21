#![allow(dead_code)]

use anyhow::Result;
use async_trait::async_trait;

/// Search query parameters for a build request.
#[derive(Debug, Clone)]
pub struct BuildRequest {
    /// The URI of the document to build.
    pub document_uri: tower_lsp::lsp_types::Url,
    /// The root directory of the workspace (optional).
    pub workspace_root: Option<std::path::PathBuf>,
}

/// Start/End logs from a build execution.
#[derive(Debug)]
pub struct BuildLog {
    pub stdout: String,
    pub stderr: String,
}

/// The outcome of a build attempt.
#[derive(Debug)]
pub enum BuildStatus {
    /// Build succeeded, producing an artifact at the given path.
    Success(std::path::PathBuf), 
    /// Build failed, with captured logs.
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
