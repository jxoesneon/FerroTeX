use super::{BuildEngine, BuildLog, BuildRequest, BuildStatus};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

/// Implementation of `BuildEngine` using the `latexmk` command-line tool.
///
/// Handles spawning `latexmk` with appropriate flags for PDF generation and interaction modes.
#[derive(Debug)]
pub struct LatexmkAdapter {
    binary: String,
}

impl LatexmkAdapter {
    pub fn new() -> Self {
        Self {
            binary: "latexmk".to_string(),
        }
    }

    pub fn with_binary(binary: &str) -> Self {
        Self {
            binary: binary.to_string(),
        }
    }
}

impl Default for LatexmkAdapter {
    fn default() -> Self {
        Self::new()
    }
}

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
        let mut cmd = self.create_command(&file_path, &out_dir);

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(parent_dir) // Run in file's directory
            .spawn()
            .context(format!("Failed to spawn {}. Ensure it is installed and in your PATH (e.g. /Library/TeX/texbin).", self.binary))?;

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

impl LatexmkAdapter {
    pub fn create_command(
        &self,
        file_path: &std::path::Path,
        out_dir: &std::path::Path,
    ) -> Command {
        // PATH Augmentation for macOS (MacTeX)
        let mut cmd = Command::new(&self.binary);

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

        cmd.arg("-pdf")
            .arg("-interaction=nonstopmode")
            .arg("-halt-on-error")
            .arg("-file-line-error")
            .arg(format!("-outdir={}", out_dir.to_string_lossy()))
            .arg(file_path);

        cmd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_latexmk_command_creation() {
        let adapter = LatexmkAdapter::new();
        let file = Path::new("main.tex");
        let out = Path::new("build");
        let cmd = adapter.create_command(file, out);
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("latexmk"));
        assert!(debug_str.contains("-pdf"));
        assert!(debug_str.contains("main.tex"));
        assert!(debug_str.contains("-outdir=build"));
    }

    #[tokio::test]
    async fn test_build_failure() {
        let adapter = LatexmkAdapter::with_binary("nonexistent_latexmk");
        let temp_dir = tempfile::tempdir().unwrap();
        let tex_path = temp_dir.path().join("test.tex");
        // Create file so path existence checks pass if any (none currently in build() before spawn)
        tokio::fs::File::create(&tex_path).await.unwrap();

        let uri = url::Url::from_file_path(&tex_path).unwrap();
        let request = BuildRequest {
            document_uri: uri,
            workspace_root: None,
        };

        // Expect failure because latexmk is not in path
        let result = adapter.build(&request, None).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err
            .to_string()
            .contains("Failed to spawn nonexistent_latexmk"));
    }

    #[tokio::test]
    async fn test_build_success() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("mock_latexmk.sh");
        let tex_path = temp_dir.path().join("main.tex");
        let build_dir = temp_dir.path().join("build");

        // Create dummy tex file
        tokio::fs::write(&tex_path, "\\documentclass{article}")
            .await
            .unwrap();

        // Create mock latexmk script
        let script_content = format!(
            r#"#!/bin/sh
# Mock latexmk
mkdir -p "{}"
touch "{}/main.pdf"
echo "Mock latexmk build success"
exit 0
"#,
            build_dir.to_string_lossy(),
            build_dir.to_string_lossy()
        );

        tokio::fs::write(&script_path, script_content)
            .await
            .unwrap();

        let mut perms = tokio::fs::metadata(&script_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&script_path, perms)
            .await
            .unwrap();

        let adapter = LatexmkAdapter::with_binary(script_path.to_str().unwrap());

        let uri = url::Url::from_file_path(&tex_path).unwrap();
        let request = BuildRequest {
            document_uri: uri,
            workspace_root: None,
        };

        let result = adapter.build(&request, None).await;
        assert!(result.is_ok());
        if let BuildStatus::Success(path) = result.unwrap() {
            assert_eq!(path, build_dir.join("main.pdf"));
            assert!(path.exists());
        } else {
            panic!("Expected BuildStatus::Success");
        }
    }

    #[tokio::test]
    async fn test_build_success_with_logs() {
        use std::os::unix::fs::PermissionsExt;
        use std::sync::{Arc, Mutex};

        let temp_dir = tempfile::tempdir().unwrap();
        let script_path = temp_dir.path().join("mock_latexmk_logs.sh");
        let tex_path = temp_dir.path().join("main.tex");
        let build_dir = temp_dir.path().join("build");

        tokio::fs::write(&tex_path, "\\documentclass{article}")
            .await
            .unwrap();

        // Mock script that outputs to stdout and stderr
        let script_content = format!(
            r#"#!/bin/sh
mkdir -p "{}"
touch "{}/main.pdf"
echo "Log stdout line 1"
echo "Log stderr line 1" >&2
echo "Log stdout line 2"
exit 0
"#,
            build_dir.to_string_lossy(),
            build_dir.to_string_lossy()
        );

        tokio::fs::write(&script_path, script_content)
            .await
            .unwrap();

        let mut perms = tokio::fs::metadata(&script_path)
            .await
            .unwrap()
            .permissions();
        perms.set_mode(0o755);
        tokio::fs::set_permissions(&script_path, perms)
            .await
            .unwrap();

        let adapter = LatexmkAdapter::with_binary(script_path.to_str().unwrap());

        let uri = url::Url::from_file_path(&tex_path).unwrap();
        let request = BuildRequest {
            document_uri: uri,
            workspace_root: None,
        };

        let logs = Arc::new(Mutex::new(Vec::new()));
        let logs_clone = logs.clone();
        let callback = Box::new(move |msg: String| {
            logs_clone.lock().unwrap().push(msg);
        });

        let result = adapter.build(&request, Some(callback)).await;
        assert!(result.is_ok());

        let captured_logs = logs.lock().unwrap();
        assert!(captured_logs
            .iter()
            .any(|l| l.contains("[stdout] Log stdout line 1")));
        assert!(captured_logs
            .iter()
            .any(|l| l.contains("[stderr] Log stderr line 1")));
        assert!(captured_logs
            .iter()
            .any(|l| l.contains("[stdout] Log stdout line 2")));
    }
}
