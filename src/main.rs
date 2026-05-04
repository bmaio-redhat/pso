mod cli;
mod report;
mod shard;

use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use clap::Parser;

use cli::Cli;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.shards == 0 {
        eprintln!("error: --shards must be >= 1");
        return ExitCode::FAILURE;
    }

    let pids: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));
    let pids_for_signal = Arc::clone(&pids);

    tokio::select! {
        summary = shard::orchestrate(&cli, pids) => {
            summary.print();
            if summary.all_passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("\n[pso] interrupted – killing all shards and their subprocesses");
            shard::kill_all_shards(&pids_for_signal);
            ExitCode::from(130)
        }
    }
}
