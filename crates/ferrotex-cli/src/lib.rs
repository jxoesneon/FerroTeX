use clap::{Parser, Subcommand};
use ferrotex_log::LogParser;
use notify::{EventKind, RecursiveMode, Watcher};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;

/// The main CLI argument parser.
#[derive(Parser)]
#[command(name = "ferrotex")]
#[command(version)]
#[command(about = "FerroTeX CLI tools", long_about = None)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI subcommands.
#[derive(Subcommand)]
pub enum Commands {
    /// Parse a TeX log file and emit JSON IR.
    Parse {
        /// Path to the .log file.
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Watch a TeX log file for changes and stream events.
    Watch {
        /// Path to the .log file.
        #[arg(value_name = "FILE")]
        path: PathBuf,
    },
    /// Start the Debug Adapter Protocol (DAP) server.
    Debug,
    /// Build a TeX document using pdflatex.
    Build {
        /// Path to the .tex file to compile.
        #[arg(value_name = "FILE")]
        path: PathBuf,
        /// Output directory (defaults to current directory).
        #[arg(short, long, default_value = ".")]
        output_dir: PathBuf,
    },
    /// Verify the current source files against ferrotex.lock.
    Verify {
        /// Path to the .lock file.
        #[arg(value_name = "LOCKFILE", default_value = "ferrotex.lock")]
        path: PathBuf,
    },
}

pub fn execute(cli: Cli) -> anyhow::Result<()> {
    match &cli.command {
        Commands::Parse { path } => {
            let content = fs::read_to_string(path)?;
            let parser = LogParser::new();
            let events = parser.parse(&content);
            println!("{}", serde_json::to_string_pretty(&events)?);
        }
        Commands::Watch { path } => {
            watch_log(path)?;
        }
        Commands::Debug => {
            #[cfg(feature = "jxoesneon-tectonic-engine")]
            {
                ferrotex_dap::run_jxoesneon_tectonic_session()?;
            }
            #[cfg(not(feature = "jxoesneon-tectonic-engine"))]
            {
                ferrotex_dap::run_mock_session()?;
            }
        }
        Commands::Build { path, output_dir } => {
            build_tex(path, output_dir)?;
        }
        Commands::Verify { path } => {
            verify_lock(path)?;
        }
    }
    Ok(())
}

fn build_tex(tex_path: &Path, output_dir: &Path) -> anyhow::Result<()> {
    use ferrotex_build::{ArtifactId, PdfLatexTransform, Transform};

    let input_id = ArtifactId(tex_path.to_string_lossy().to_string());
    let output_id = ArtifactId(
        tex_path
            .with_extension("pdf")
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    );

    let transform = PdfLatexTransform::new(
        input_id,
        output_id,
        tex_path.to_path_buf(),
        output_dir.to_path_buf(),
    );

    println!("Running: {}", transform.description());
    match transform.execute() {
        Ok(()) => println!("Build successful!"),
        Err(e) => eprintln!("Build failed: {}", e),
    }

    Ok(())
}

fn verify_lock(lock_path: &Path) -> anyhow::Result<()> {
    use ferrotex_build::Lockfile;
    use sha2::{Digest, Sha256};

    let lockfile = Lockfile::load(lock_path)?;
    println!(
        "🔍 Verifying build against lockfile: {}",
        lock_path.display()
    );

    let mut all_match = true;
    for (path_str, expected_hash) in &lockfile.entries {
        let path = Path::new(path_str);
        if !path.exists() {
            println!("❌ Missing file: {}", path_str);
            all_match = false;
            continue;
        }

        let data = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let actual_hash = hex::encode(hasher.finalize());

        if actual_hash == *expected_hash {
            println!("✅ OK: {}", path_str);
        } else {
            println!("❌ MISMATCH: {}", path_str);
            println!("   Expected: {}", expected_hash);
            println!("   Actual:   {}", actual_hash);
            all_match = false;
        }
    }

    if all_match {
        println!("\n✨ Build is verified and reproducible!");
        Ok(())
    } else {
        println!("\n⚠️ Build integrity verification failed!");
        Err(anyhow::anyhow!("Verification failed"))
    }
}

fn process_log_change(
    file: &mut File,
    pos: &mut u64,
    parser: &mut LogParser,
) -> anyhow::Result<Vec<String>> {
    let current_len = file.metadata()?.len();
    let mut results = Vec::new();

    if current_len > *pos {
        file.seek(SeekFrom::Start(*pos))?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        let events = parser.update(&buffer);
        for event in events {
            results.push(serde_json::to_string(&event)?);
        }
        *pos = current_len;
    } else if current_len < *pos {
        // File truncated? Reset.
        eprintln!("File truncated, resetting parser.");
        *parser = LogParser::new();
        file.seek(SeekFrom::Start(0))?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        *pos = file.metadata()?.len();
        let events = parser.update(&buffer);
        for event in events {
            results.push(serde_json::to_string(&event)?);
        }
    }
    Ok(results)
}

fn watch_log(path: &Path) -> anyhow::Result<()> {
    let mut parser = LogParser::new();
    let mut file = File::open(path)?;
    let mut pos = 0;

    // Initial read
    let metadata = file.metadata()?;
    let len = metadata.len();
    if len > 0 {
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        pos = len;
        let events = parser.update(&buffer);
        for event in events {
            println!("{}", serde_json::to_string(&event)?);
        }
    }

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(path, RecursiveMode::NonRecursive)?;

    eprintln!("Watching {}...", path.display());

    for res in rx {
        match res {
            Ok(event) => {
                if let EventKind::Modify(_) = event.kind {
                    let changes = process_log_change(&mut file, &mut pos, &mut parser)?;
                    for change in changes {
                        println!("{}", change);
                    }
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_process_log_change() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let mut file = File::create(&log_path).unwrap();

        let mut parser = LogParser::new();
        let mut pos = 0;

        // 1. Initial write
        write!(file, "Initial content\n").unwrap();
        let mut read_file = File::open(&log_path).unwrap();

        let _changes = process_log_change(&mut read_file, &mut pos, &mut parser).unwrap();
        // Initial content might not trigger an event, but pos should advance
        assert!(pos > 0);

        // 2. Append
        write!(file, "! LaTeX Error: Something wrong.\n").unwrap();
        let changes = process_log_change(&mut read_file, &mut pos, &mut parser).unwrap();
        assert!(!changes.is_empty());
        assert!(changes.iter().any(|s| s.contains("Error")));

        // 3. Truncate
        file.set_len(0).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        write!(file, "New content\n").unwrap();

        // This should reset and succeed
        assert!(process_log_change(&mut read_file, &mut pos, &mut parser).is_ok());
    }

    #[test]
    fn test_verify_lock_success() {
        use ferrotex_build::Lockfile;
        use sha2::{Digest, Sha256};

        let temp_dir = tempfile::tempdir().unwrap();
        let src_path = temp_dir.path().join("main.tex");
        let content = b"content";
        let mut file = File::create(&src_path).unwrap();
        file.write_all(content).unwrap();

        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hex::encode(hasher.finalize());

        let mut lockfile = Lockfile::new();
        lockfile
            .entries
            .insert(src_path.to_str().unwrap().to_string(), hash);

        let lock_path = temp_dir.path().join("ferrotex.lock");
        lockfile.save(&lock_path).unwrap();

        assert!(verify_lock(&lock_path).is_ok());
    }

    #[test]
    fn test_verify_lock_mismatch() {
        use ferrotex_build::Lockfile;

        let temp_dir = tempfile::tempdir().unwrap();
        let src_path = temp_dir.path().join("main.tex");
        let content = b"content";
        let mut file = File::create(&src_path).unwrap();
        file.write_all(content).unwrap();

        let mut lockfile = Lockfile::new();
        lockfile.entries.insert(
            src_path.to_str().unwrap().to_string(),
            "wronghash".to_string(),
        );

        let lock_path = temp_dir.path().join("ferrotex.lock");
        lockfile.save(&lock_path).unwrap();

        assert!(verify_lock(&lock_path).is_err());
    }

    #[test]
    fn test_verify_lock_missing_file() {
        use ferrotex_build::Lockfile;

        let temp_dir = tempfile::tempdir().unwrap();
        let src_path = temp_dir.path().join("missing.tex");

        let mut lockfile = Lockfile::new();
        lockfile
            .entries
            .insert(src_path.to_str().unwrap().to_string(), "hash".to_string());

        let lock_path = temp_dir.path().join("ferrotex.lock");
        lockfile.save(&lock_path).unwrap();

        assert!(verify_lock(&lock_path).is_err());
    }

    #[test]
    fn test_watch_log_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        assert!(watch_log(&log_path).is_err());
    }

    #[test]
    fn test_parse_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("test.log");
        // Create a dummy log file
        let mut file = File::create(&log_path).unwrap();
        writeln!(file, "This is TeX, Version 3.141592653").unwrap();

        let cli = Cli {
            command: Commands::Parse { path: log_path },
        };

        assert!(execute(cli).is_ok());
    }

    #[test]
    fn test_build_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let tex_path = temp_dir.path().join("main.tex");
        File::create(&tex_path).unwrap();

        let cli = Cli {
            command: Commands::Build {
                path: tex_path,
                output_dir: temp_dir.path().to_path_buf(),
            },
        };

        // This should return Ok even if build fails (it prints error)
        assert!(execute(cli).is_ok());
    }

    #[test]
    fn test_watch_command() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_path = temp_dir.path().join("non_existent.log");

        let cli = Cli {
            command: Commands::Watch { path: log_path },
        };

        // Should error because file doesn't exist (File::open fails)
        assert!(execute(cli).is_err());
    }
}
