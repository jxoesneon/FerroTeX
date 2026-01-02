use super::{BuildEngine, BuildLog, BuildRequest, BuildStatus};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

/// Implementation of `BuildEngine` using the `tectonic` command-line tool.
pub struct TectonicAdapter;

#[async_trait]
impl BuildEngine for TectonicAdapter {
    fn name(&self) -> &str {
        "tectonic"
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
        
        // Tectonic auto-downloads packages, so we just run it on the file.
        // tectonic -outdir <build> <file>
        // Note: Tectonic default interface is chatty, we want to capture stdout/stderr.
        
        let out_dir = parent_dir.join("build");
        tokio::fs::create_dir_all(&out_dir).await?;

        let mut child = Command::new("tectonic")
            .arg("-o")
            .arg(&out_dir)
            .arg("--synctex")
            .arg(&file_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(parent_dir)
            .spawn()
            .context("Failed to spawn tectonic. Ensure it is installed and in your PATH.")?;

        let stdout = child.stdout.take().context("Failed to open stdout")?;
        let stderr = child.stderr.take().context("Failed to open stderr")?;

        if let Some(callback) = log_callback {
             let cb_stdout = std::sync::Arc::new(callback);
             let cb_stderr = cb_stdout.clone();

             let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
             let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

             let stdout_handle = tokio::spawn(async move {
                 let mut acc = String::new();
                 while let Ok(Some(line)) = stdout_reader.next_line().await {
                     cb_stdout(format!("[stdout] {}\n", line));
                     acc.push_str(&line);
                     acc.push('\n');
                 }
                 acc
             });

             let stderr_handle = tokio::spawn(async move {
                 let mut acc = String::new();
                 while let Ok(Some(line)) = stderr_reader.next_line().await {
                     cb_stderr(format!("[stderr] {}\n", line));
                     acc.push_str(&line);
                     acc.push('\n');
                 }
                 acc
             });

             let status = child.wait().await?;
             let (stdout_res, stderr_res) = tokio::join!(stdout_handle, stderr_handle);
             let stdout = stdout_res.unwrap_or_default();
             let stderr = stderr_res.unwrap_or_default();

             if status.success() {
                 let file_stem = file_path.file_stem().unwrap_or_default();
                 let mut artifact = out_dir.join(file_stem);
                 artifact.set_extension("pdf");
                 Ok(BuildStatus::Success(artifact))
             } else {
                 Ok(BuildStatus::Failure(BuildLog {
                     stdout,
                     stderr,
                 }))
             }

        } else {
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
