//! Package manager abstraction for TeX distributions.
//!
//! ## Overview
//!
//! This module provides a unified interface for interacting with different TeX package
//! managers (tlmgr, MiKTeX) through the [`PackageManager`] facade and the [`PackageBackend`]
//! trait. The abstraction enables FerroTeX tools to install packages and search for
//! package information without knowing which specific distribution is installed.
//!
//! ## Architecture
//!
//! The design uses the **Strategy pattern** with dependency injection for testability:
//!
//! ```text
//! ┌─────────────────┐
//! │ PackageManager  │  ← High-level facade
//! └────────┬────────┘
//!          │
//!          │ Arc<dyn PackageBackend>
//!          ▼
//! ┌──────────────────┐
//! │ PackageBackend   │  ← Trait defining backend interface
//! │   (trait)        │
//! └────────┬─────────┘
//!          │
//!    ┌─────┴──────┬─────────────┬──────────────┐
//!    │            │             │              │
//!    │            │             │              │
//! TlmgrBackend MiktexBackend NoOpBackend  (MockBackend)
//! ```
//!
//! ### Command Execution Abstraction
//!
//! To enable unit testing without invoking actual system commands, all backends
//! use the [`CommandExecutor`] trait:
//!
//! - **Production**: [`RealCommandExecutor`] uses `std::process::Command`
//! - **Testing**: [`MockCommandExecutor`] returns pre-configured outputs
//!
//! This allows comprehensive testing of error handling, parsing logic, and edge cases
//! without requiring a TeX distribution to be installed.
//!
//! ## CTAN Database Integration
//!
//! The [`ctan_db`] sub-module provides a compiled-in mapping of file names to CTAN
//! package names. This enables IDE features like "click to view package documentation"
//! when a missing file is detected.
//!
//! ## Examples
//!
//! ### Auto-detecting and Installing a Package
//!
//! ```no_run
//! use ferrotex_core::package_manager::PackageManager;
//!
//! let pm = PackageManager::new(); // Auto-detects tlmgr or mpm
//!
//! if !pm.is_available() {
//!     eprintln!("No TeX package manager found!");
//!     return;
//! }
//!
//! match pm.install("tikz") {
//!     Ok(status) => {
//!         println!("Package '{}' installation: {:?}", status.name, status.state);
//!     }
//!     Err(e) => {
//!         eprintln!("Installation error: {}", e);
//!     }
//! }
//! ```
//!
//! ### Using a Custom Backend (for testing)
//!
//! ```
//! use ferrotex_core::package_manager::{PackageManager, NoOpBackend};
//! use std::sync::Arc;
//!
//! let backend = Arc::new(NoOpBackend);
//! let pm = PackageManager::with_backend(backend);
//!
//! // This won't actually install anything
//! let result = pm.install("amsmath");
//! assert!(result.is_ok());
//! ```
//!
//! ### Looking up CTAN Package Information
//!
//! ```
//! use ferrotex_core::package_manager::PackageManager;
//!
//! if let Some(url) = PackageManager::get_ctan_link("tikz.sty") {
//!     println!("Documentation: {}", url);
//!     // Output: Documentation: https://ctan.org/pkg/pgf
//! }
//! ```

use anyhow::{anyhow, Result};
use log::{info, warn};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Database of CTAN packages and file mappings.
pub mod ctan_db;

/// The state of a package installation operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallState {
    /// The package was successfully installed.
    Complete,
    /// The installation is in progress (not currently used).
    Pending,
    /// The installation failed (see [`InstallStatus::message`] for details).
    Failed,
    /// The installation state is unknown (e.g., no package manager available).
    Unknown,
}

/// The result of a package installation attempt.
///
/// Contains both the outcome ([`state`](Self::state)) and optional diagnostic
/// information ([`message`](Self::message)) for failures.
#[derive(Debug, Clone)]
pub struct InstallStatus {
    /// The name of the package that was installed (or attempted).
    pub name: String,
    /// The outcome of the installation operation.
    pub state: InstallState,
    /// Optional error message or diagnostic information.
    /// Typically populated when `state` is [`InstallState::Failed`].
    pub message: Option<String>,
}

/// Trait for executing system commands.
/// This allows us to mock `std::process::Command` in tests.
pub trait CommandExecutor: Send + Sync + std::fmt::Debug {
    /// Executes a system command with the given arguments.
    ///
    /// # Arguments
    ///
    /// * `program` - The path to the executable.
    /// * `args` - A list of arguments to pass to the executable.
    ///
    /// # Returns
    ///
    /// The output of the command (stdout/stderr/exit code).
    fn execute(&self, program: &Path, args: &[&str]) -> Result<std::process::Output>;
}

/// Default implementation of [`CommandExecutor`] using `std::process::Command`.
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
///
/// This struct is used to simulate command execution in tests, allowing inspection
/// of command arguments and returning pre-defined output.
#[cfg(test)]
#[derive(Debug)]
pub struct MockCommandExecutor {
    /// The string to return as standard output.
    pub stdout: String,
    /// The string to return as standard error.
    pub stderr: String,
    /// The exit code to simulate (0 for success).
    pub status_code: i32,
}

#[cfg(test)]
impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, _program: &Path, _args: &[&str]) -> Result<std::process::Output> {
        #[cfg(unix)]
        let status = {
            use std::os::unix::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(self.status_code << 8)
        };
        #[cfg(windows)]
        let status = {
            use std::os::windows::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(self.status_code as u32)
        };

        Ok(std::process::Output {
            status,
            stdout: self.stdout.as_bytes().to_vec(),
            stderr: self.stderr.as_bytes().to_vec(),
        })
    }
}

/// Trait defining the interface for TeX package manager backends.
///
/// Implementors provide distribution-specific logic for installing packages
/// and searching for package information.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to support concurrent use by the
/// language server or other multi-threaded tools.
pub trait PackageBackend: std::fmt::Debug + Send + Sync {
    /// Installs the specified package.
    ///
    /// # Arguments
    ///
    /// * `package` - The name of the package to install (e.g., "tikz", "amsmath")
    ///
    /// # Returns
    ///
    /// An [`InstallStatus`] indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns an error if the package manager command fails to execute
    /// (e.g., command not found, permission denied).
    fn install(&self, package: &str) -> Result<InstallStatus>;

    /// Searches for packages or files matching the query.
    ///
    /// # Arguments
    ///
    /// * `query` - Search term (interpretation is backend-specific)
    ///
    /// # Returns
    ///
    /// A list of matching package names or file paths.
    ///
    /// # Errors
    ///
    /// Returns an error if the search command fails.
    fn search(&self, query: &str) -> Result<Vec<String>>;

    /// Returns a human-readable name for this backend (e.g., "tlmgr", "miktex").
    fn name(&self) -> &'static str;
}

/// Backend implementation for the TeX Live Manager (`tlmgr`).
#[derive(Debug)]
pub struct TlmgrBackend {
    path: PathBuf,
    executor: Box<dyn CommandExecutor>,
}

impl TlmgrBackend {
    /// Creates a new `TlmgrBackend` for the given executable path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            executor: Box::new(RealCommandExecutor),
        }
    }

    /// Creates a new `TlmgrBackend` with a custom executor (for testing).
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
        let output = self
            .executor
            .execute(&self.path, &["search", "--global", "--file", query])?;

        if !output.status.success() {
            return Err(anyhow!("tlmgr search failed"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let results = stdout
            .lines()
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

/// Backend implementation for the MiKTeX Package Manager (`mpm`).
#[derive(Debug)]
pub struct MiktexBackend {
    path: PathBuf,
    executor: Box<dyn CommandExecutor>,
}

impl MiktexBackend {
    /// Creates a new `MiktexBackend` for the given executable path.
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            executor: Box::new(RealCommandExecutor),
        }
    }

    /// Creates a new `MiktexBackend` with a custom executor (for testing).
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

/// A backend used when no package manager is detected.
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

/// High-level facade for TeX package management operations.
///
/// This struct auto-detects the available package manager on the system
/// (tlmgr for TeX Live, mpm for MiKTeX) and provides a unified interface
/// for package installation and search.
///
/// # Thread Safety
///
/// `PackageManager` is cheaply cloneable (uses `Arc` internally) and can be
/// safely shared across threads.
///
/// # Examples
///
/// ```no_run
/// use ferrotex_core::package_manager::PackageManager;
///
/// let pm = PackageManager::new();
///
/// if pm.is_available() {
///     let status = pm.install("tikz")?;
///     println!("Installation: {:?}", status.state);
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct PackageManager {
    backend: std::sync::Arc<dyn PackageBackend>,
}

impl Default for PackageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageManager {
    /// Creates a new `PackageManager` by detecting available system tools.
    ///
    /// It checks for `tlmgr` and `mpm` in the system PATH.
    pub fn new() -> Self {
        // Auto-detect
        if let Ok(path) = which::which("tlmgr") {
            info!("Detected tlmgr at {:?}", path);
            return Self {
                backend: std::sync::Arc::new(TlmgrBackend::new(path)),
            };
        }
        if let Ok(path) = which::which("mpm") {
            info!("Detected miktex (mpm) at {:?}", path);
            return Self {
                backend: std::sync::Arc::new(MiktexBackend::new(path)),
            };
        }

        warn!("No package manager detected");
        Self {
            backend: std::sync::Arc::new(NoOpBackend),
        }
    }

    /// Creates a new `PackageManager` with a specific backend (useful for testing).
    pub fn with_backend(backend: std::sync::Arc<dyn PackageBackend>) -> Self {
        Self { backend }
    }

    /// Installs a package using the active backend.
    pub fn install(&self, package: &str) -> Result<InstallStatus> {
        self.backend.install(package)
    }

    /// Searches for a package using the active backend.
    pub fn search(&self, query: &str) -> Result<Vec<String>> {
        self.backend.search(query)
    }

    /// Checks if a valid package manager backend is available.
    pub fn is_available(&self) -> bool {
        self.backend.name() != "none"
    }

    /// Returns a link to the package documentation on CTAN, if available.
    pub fn get_ctan_link(filename: &str) -> Option<String> {
        ctan_db::CTAN_DB
            .lookup(filename)
            .map(|pkg| format!("https://ctan.org/pkg/{}", pkg))
    }
}

#[cfg(test)]
mod tests;
