use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{Result, anyhow};
use log::{info, warn};

pub mod ctan_db;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallState {
    Complete,
    Pending,
    Failed,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct InstallStatus {
    pub name: String,
    pub state: InstallState, 
    pub message: Option<String>,
}

/// Trait for executing system commands.
/// This allows us to mock `std::process::Command` in tests.
pub trait CommandExecutor: Send + Sync + std::fmt::Debug {
    fn execute(&self, program: &Path, args: &[&str]) -> Result<std::process::Output>;
}

#[derive(Debug)]
pub struct RealCommandExecutor;

impl CommandExecutor for RealCommandExecutor {
    fn execute(&self, program: &Path, args: &[&str]) -> Result<std::process::Output> {
        Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| anyhow!("Failed to execute command: {}", e))
    }
}

/// A mocked executor for testing that doesn't actually run system commands.
#[cfg(test)]
#[derive(Debug)]
pub struct MockCommandExecutor {
    pub stdout: String,
    pub stderr: String,
    pub status_code: i32,
}

#[cfg(test)]
impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, _program: &Path, _args: &[&str]) -> Result<std::process::Output> {
         use std::os::unix::process::ExitStatusExt;
         Ok(std::process::Output {
             status: std::process::ExitStatus::from_raw(self.status_code << 8), // 0 success, non-zero fail
             stdout: self.stdout.as_bytes().to_vec(),
             stderr: self.stderr.as_bytes().to_vec(),
         })
    }
}

pub trait PackageBackend: std::fmt::Debug + Send + Sync {
    fn install(&self, package: &str) -> Result<InstallStatus>;
    fn search(&self, query: &str) -> Result<Vec<String>>;
    fn name(&self) -> &'static str;
}

#[derive(Debug)]
pub struct TlmgrBackend {
    path: PathBuf,
    executor: Box<dyn CommandExecutor>,
}

impl TlmgrBackend {
    pub fn new(path: PathBuf) -> Self {
        Self { path, executor: Box::new(RealCommandExecutor) }
    }
    
    // For testing injection
    pub fn with_executor(path: PathBuf, executor: Box<dyn CommandExecutor>) -> Self {
         Self { path, executor }
    }
}

impl PackageBackend for TlmgrBackend {
    fn install(&self, package: &str) -> Result<InstallStatus> {
        // tlmgr install <package>
        let output = self.executor.execute(&self.path, &["install", package])?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Ok(InstallStatus {
                name: package.to_string(),
                state: InstallState::Failed,
                message: Some(stderr.to_string()),
            });
        }
        
        Ok(InstallStatus {
            name: package.to_string(),
            state: InstallState::Complete,
            message: None,
        })
    }

    fn search(&self, query: &str) -> Result<Vec<String>> {
        let output = self.executor.execute(&self.path, &["search", "--global", "--file", query])?;
        
        if !output.status.success() {
             return Err(anyhow!("tlmgr search failed"));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let results = stdout.lines()
            .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
            .filter(|s| !s.is_empty())
            .filter(|s| !s.ends_with(':')) // Filter out 'tlmgr:' lines if any
            .collect();
            
        Ok(results)
    }

    fn name(&self) -> &'static str {
        "tlmgr"
    }
}

#[derive(Debug)]
pub struct MiktexBackend {
    path: PathBuf,
    executor: Box<dyn CommandExecutor>,
}

impl MiktexBackend {
    pub fn new(path: PathBuf) -> Self {
        Self { path, executor: Box::new(RealCommandExecutor) }
    }
    
    pub fn with_executor(path: PathBuf, executor: Box<dyn CommandExecutor>) -> Self {
        Self { path, executor }
    }
}

impl PackageBackend for MiktexBackend {
    fn install(&self, package: &str) -> Result<InstallStatus> {
        // mpm --install <package>
        let output = self.executor.execute(&self.path, &["--install", package])?;

        if !output.status.success() {
             let stderr = String::from_utf8_lossy(&output.stderr);
             return Ok(InstallStatus {
                name: package.to_string(),
                state: InstallState::Failed,
                message: Some(stderr.to_string()),
            });
        }

        Ok(InstallStatus {
            name: package.to_string(),
            state: InstallState::Complete,
            message: None,
        })
    }

    fn search(&self, _query: &str) -> Result<Vec<String>> {
        // miktex search not easily standardized
        Ok(vec![])
    }

    fn name(&self) -> &'static str {
        "miktex"
    }
}

#[derive(Debug)]
pub struct NoOpBackend;
impl PackageBackend for NoOpBackend {
    fn install(&self, package: &str) -> Result<InstallStatus> {
        Ok(InstallStatus {
            name: package.to_string(),
            state: InstallState::Unknown,
            message: Some("No package manager found".into()),
        })
    }
    fn search(&self, _query: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }
    fn name(&self) -> &'static str {
        "none"
    }
}

pub struct PackageManager {
    backend: Box<dyn PackageBackend>,
}

impl PackageManager {
    pub fn new() -> Self {
        // Auto-detect
        if let Ok(path) = which::which("tlmgr") {
            info!("Detected tlmgr at {:?}", path);
            return Self { backend: Box::new(TlmgrBackend::new(path)) };
        }
        if let Ok(path) = which::which("mpm") {
            info!("Detected miktex (mpm) at {:?}", path);
            return Self { backend: Box::new(MiktexBackend::new(path)) };
        }
        
        warn!("No package manager detected");
        Self { backend: Box::new(NoOpBackend) }
    }

    pub fn with_backend(backend: Box<dyn PackageBackend>) -> Self {
        Self { backend }
    }

    pub fn install(&self, package: &str) -> Result<InstallStatus> {
        self.backend.install(package)
    }

    pub fn search(&self, query: &str) -> Result<Vec<String>> {
        self.backend.search(query)
    }
    
    pub fn is_available(&self) -> bool {
        self.backend.name() != "none"
    }
    
    pub fn get_ctan_link(filename: &str) -> Option<String> {
        ctan_db::CTAN_DB.lookup(filename).map(|pkg| format!("https://ctan.org/pkg/{}", pkg))
    }
}

#[cfg(test)]
mod tests;
