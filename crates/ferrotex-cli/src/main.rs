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
struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI subcommands.
#[derive(Subcommand)]
enum Commands {
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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
            #[cfg(feature = "tectonic-engine")]
            {
                ferrotex_dap::run_tectonic_session()?;
            }
            #[cfg(not(feature = "tectonic-engine"))]
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
    } else {
        println!("\n⚠️ Build integrity verification failed!");
        std::process::exit(1);
    }

    Ok(())
}

/// Watches a log file for changes and prints new events as JSON.
///
/// This function tails the file, similar to `tail -f`, but parses the content
/// using `LogParser` to emit structured events.
///
/// # Arguments
///
/// * `path` - The path to the log file to watch.
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
                    // Check if file grew
                    let current_len = file.metadata()?.len();
                    if current_len > pos {
                        file.seek(SeekFrom::Start(pos))?;
                        let mut buffer = String::new();
                        // Read only the new part
                        // We use read_to_string which reads until EOF.
                        // Since we seeked to `pos`, it reads from `pos` to end.
                        // Note: This assumes valid UTF-8 appending.
                        file.read_to_string(&mut buffer)?;
                        let events = parser.update(&buffer);
                        for event in events {
                            println!("{}", serde_json::to_string(&event)?);
                        }
                        pos = current_len;
                    } else if current_len < pos {
                        // File truncated? Reset.
                        eprintln!("File truncated, resetting parser.");
                        parser = LogParser::new();
                        file.seek(SeekFrom::Start(0))?;
                        // Read everything again
                        let mut buffer = String::new();
                        file.read_to_string(&mut buffer)?;
                        pos = file.metadata()?.len();
                        let events = parser.update(&buffer);
                        for event in events {
                            println!("{}", serde_json::to_string(&event)?);
                        }
                    }
                }
            }
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }

    Ok(())
}
