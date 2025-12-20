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
