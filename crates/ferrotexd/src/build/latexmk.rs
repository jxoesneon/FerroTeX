use super::{BuildEngine, BuildLog, BuildRequest, BuildStatus};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;

pub struct LatexmkAdapter;

#[async_trait]
impl BuildEngine for LatexmkAdapter {
    fn name(&self) -> &str {
        "latexmk"
    }

    async fn build(&self, request: &BuildRequest) -> Result<BuildStatus> {
        let file_path = request
            .document_uri
            .to_file_path()
            .map_err(|_| anyhow::anyhow!("Invalid URI scheme"))?;

        let parent_dir = file_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."));

        // We will output to a 'build' directory relative to the file to avoid clutter
        let out_dir = parent_dir.join("build");

        // Ensure out_dir exists
        tokio::fs::create_dir_all(&out_dir).await?;

        // latexmk -pdf -interaction=nonstopmode -halt-on-error -file-line-error -outdir=<dist> <file>
        let output = Command::new("latexmk")
            .arg("-pdf")
            .arg("-interaction=nonstopmode")
            .arg("-halt-on-error")
            .arg("-file-line-error")
            .arg(format!("-outdir={}", out_dir.to_string_lossy()))
            .arg(&file_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(parent_dir) // Run in file's directory
            .spawn()
            .context("Failed to spawn latexmk")?
            .wait_with_output()
            .await
            .context("Failed to wait for latexmk")?;

        if output.status.success() {
            // Artifact name typically replaces extension with .pdf
            let file_stem = file_path.file_stem().unwrap_or_default();
            let mut artifact = out_dir.join(file_stem);
            artifact.set_extension("pdf");

            Ok(BuildStatus::Success(artifact))
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            Ok(BuildStatus::Failure(BuildLog { stdout, stderr }))
        }
    }
}
