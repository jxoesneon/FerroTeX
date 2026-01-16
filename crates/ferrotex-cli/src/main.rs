use clap::Parser;
use ferrotex_cli::{execute, Cli};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    execute(cli)
}
