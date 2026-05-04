mod cli;
mod report;
mod shard;

use std::process::ExitCode;

use clap::Parser;

use cli::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.shards == 0 {
        eprintln!("error: --shards must be >= 1");
        return ExitCode::FAILURE;
    }

    let summary = shard::orchestrate(&cli).await;
    summary.print();

    if summary.all_passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
