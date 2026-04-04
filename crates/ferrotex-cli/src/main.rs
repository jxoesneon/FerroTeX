use clap::Parser;
use ferrotex_cli::{execute, Cli, FerroTeXResult};

fn main() -> FerroTeXResult<()> {
    let cli = Cli::parse();
    execute(cli)
}
