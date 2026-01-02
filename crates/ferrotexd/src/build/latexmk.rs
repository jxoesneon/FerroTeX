use super::{BuildEngine, BuildLog, BuildRequest, BuildStatus};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

/// Implementation of `BuildEngine` using the `latexmk` command-line tool.
///
/// Handles spawning `latexmk` with appropriate flags for PDF generation and interaction modes.
pub struct LatexmkAdapter;

#[async_trait]
impl BuildEngine for LatexmkAdapter {
    fn name(&self) -> &str {
        "latexmk"
    }

    async fn build(
        &self,
        request: &BuildRequest,
        log_callback: Option<Box<dyn Fn(String) + Send + Sync>>,
    ) -> Result<BuildStatus> {
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
        // PATH Augmentation for macOS (MacTeX)
        let mut cmd = Command::new("latexmk");
        
        #[cfg(target_os = "macos")]
        {
            let current_path = std::env::var("PATH").unwrap_or_default();
            // Common MacTeX path
            let mactex_path = "/Library/TeX/texbin";
            if std::path::Path::new(mactex_path).exists() && !current_path.contains(mactex_path) {
                let new_path = format!("{}:{}", current_path, mactex_path);
                cmd.env("PATH", new_path);
            }
        }

        let mut child = cmd
            .arg("-pdf")
            .arg("-synctex=1")
            .arg("-interaction=nonstopmode")
            .arg("-halt-on-error")
            .arg("-file-line-error")
            .arg(format!("-outdir={}", out_dir.to_string_lossy()))
            .arg(&file_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(parent_dir) // Run in file's directory
            .spawn()
            .context("Failed to spawn latexmk. Ensure it is installed and in your PATH (e.g. /Library/TeX/texbin).")?;

        let stdout = child.stdout.take().context("Failed to open stdout")?;
        let stderr = child.stderr.take().context("Failed to open stderr")?;

        // If a callback is provided, we need to stream logs in real-time.
        // We spawn tasks to read stdout/stderr concurrently.
        if let Some(callback) = log_callback {
            let cb_stdout = std::sync::Arc::new(callback);
            let cb_stderr = cb_stdout.clone();

            let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
            let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

            let stdout_handle = tokio::spawn(async move {
                while let Ok(Some(line)) = stdout_reader.next_line().await {
                    cb_stdout(format!("[stdout] {}\n", line));
                }
            });

            let stderr_handle = tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_reader.next_line().await {
                    cb_stderr(format!("[stderr] {}\n", line));
                }
            });

            // Wait for process to finish
            let status = child.wait().await?;
            
            // Wait for IO streams to finish
            let _ = tokio::join!(stdout_handle, stderr_handle);

            if status.success() {
                let file_stem = file_path.file_stem().unwrap_or_default();
                let mut artifact = out_dir.join(file_stem);
                artifact.set_extension("pdf");
                Ok(BuildStatus::Success(artifact))
            } else {
                 Ok(BuildStatus::Failure(BuildLog {
                    stdout: "See realtime logs".into(),
                    stderr: "See realtime logs".into(),
                }))
            }
        } else {
             // Buffered mode (same as before)
             let output = child.wait_with_output().await?;
             if output.status.success() {
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
}
