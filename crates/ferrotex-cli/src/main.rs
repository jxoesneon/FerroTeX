use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use ferrotex_log::LogParser;

#[derive(Parser)]
#[command(name = "ferrotex")]
#[command(about = "FerroTeX CLI tools", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a TeX log file and emit JSON IR
    Parse {
        /// Path to the .log file
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
    }
    Ok(())
}
