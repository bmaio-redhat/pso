use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use tokio::process::Command;
use tokio::task::JoinSet;

use crate::cli::Cli;
use crate::report::{ShardOutcome, ShardResult, RunSummary};

/// Build the environment variables map shared by every shard.
pub fn build_env(cli: &Cli) -> HashMap<String, String> {
    let mut env: HashMap<String, String> = std::env::vars().collect();

    env.insert("IS_SHARDED".into(), "true".into());
    env.insert("WORKERS".into(), cli.workers.to_string());
    env.insert(
        "NODE_OPTIONS".into(),
        format!("--max-old-space-size={}", cli.memory_mb),
    );

    if cli.headed {
        env.insert("HEADED".into(), "1".into());
    }

    for pair in &cli.env {
        if let Some((k, v)) = pair.split_once('=') {
            env.insert(k.to_string(), v.to_string());
        }
    }

    env
}

/// Build the Playwright CLI arguments for a single shard.
pub fn build_args(cli: &Cli, shard_index: u32) -> Vec<String> {
    let mut args = vec![
        "test".into(),
        format!("--config={}", cli.config),
        format!("--shard={}/{}", shard_index, cli.shards),
        format!("--workers={}", cli.workers),
    ];

    if let Some(ref grep) = cli.grep {
        args.push(format!("--grep={grep}"));
    }
    if let Some(ref grep_invert) = cli.grep_invert {
        args.push(format!("--grep-invert={grep_invert}"));
    }
    if let Some(retries) = cli.retries {
        args.push(format!("--retries={retries}"));
    }
    if let Some(timeout) = cli.timeout {
        args.push(format!("--timeout={timeout}"));
    }
    if cli.headed {
        args.push("--headed".into());
    }
    if let Some(ref file) = cli.file {
        args.push(file.clone());
    }

    args
}

/// Resolve the `npx` binary path — prefer the project-local `node_modules`.
fn playwright_bin(project_dir: &Path) -> String {
    let local = project_dir.join("node_modules/.bin/playwright");
    if local.exists() {
        return local.to_string_lossy().into_owned();
    }
    "npx".into()
}

/// Spawn a single shard process and capture its output.
async fn run_shard(
    shard_index: u32,
    cli: &Cli,
    env: &HashMap<String, String>,
) -> ShardResult {
    let start = Instant::now();
    let bin = playwright_bin(&cli.project_dir);
    let args = build_args(cli, shard_index);

    let cmd_display = if bin.contains("npx") {
        format!("npx playwright {}", args.join(" "))
    } else {
        format!("playwright {}", args.join(" "))
    };

    eprintln!("[shard {}/{}] starting: {}", shard_index, cli.shards, cmd_display);

    let mut cmd = if bin.contains("npx") {
        let mut c = Command::new("npx");
        c.arg("playwright");
        c.args(&args);
        c
    } else {
        let mut c = Command::new(&bin);
        c.args(&args);
        c
    };

    cmd.current_dir(&cli.project_dir);
    cmd.envs(env);
    cmd.env("SHARD_INDEX", shard_index.to_string());
    cmd.env("SHARD_TOTAL", cli.shards.to_string());

    let output = cmd.output().await;
    let elapsed = start.elapsed();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let code = out.status.code().unwrap_or(-1);

            ShardResult {
                shard_index,
                outcome: if out.status.success() {
                    ShardOutcome::Passed
                } else {
                    ShardOutcome::Failed { exit_code: code }
                },
                duration: elapsed,
                stdout,
                stderr,
            }
        }
        Err(e) => ShardResult {
            shard_index,
            outcome: ShardOutcome::Error {
                message: e.to_string(),
            },
            duration: elapsed,
            stdout: String::new(),
            stderr: e.to_string(),
        },
    }
}

/// Orchestrate all shards in parallel and return a summary.
pub async fn orchestrate(cli: &Cli) -> RunSummary {
    let total_start = Instant::now();
    let env = build_env(cli);

    eprintln!(
        "\n=== PSO: Launching {} shard(s), {} worker(s) each, {}MB per shard ===\n",
        cli.shards, cli.workers, cli.memory_mb
    );

    let mut join_set = JoinSet::new();

    for i in 1..=cli.shards {
        let cli_shards = cli.shards;
        let cli_workers = cli.workers;
        let cli_memory_mb = cli.memory_mb;
        let cli_headed = cli.headed;
        let cli_quiet = cli.quiet;
        let cli_project_dir = cli.project_dir.clone();
        let cli_config = cli.config.clone();
        let cli_grep = cli.grep.clone();
        let cli_grep_invert = cli.grep_invert.clone();
        let cli_file = cli.file.clone();
        let cli_retries = cli.retries;
        let cli_timeout = cli.timeout;
        let cli_env_pairs = cli.env.clone();
        let env = env.clone();

        join_set.spawn(async move {
            let owned_cli = Cli {
                shards: cli_shards,
                workers: cli_workers,
                memory_mb: cli_memory_mb,
                headed: cli_headed,
                quiet: cli_quiet,
                project_dir: cli_project_dir,
                config: cli_config,
                grep: cli_grep,
                grep_invert: cli_grep_invert,
                file: cli_file,
                retries: cli_retries,
                timeout: cli_timeout,
                env: cli_env_pairs,
            };
            run_shard(i, &owned_cli, &env).await
        });
    }

    let mut results = Vec::with_capacity(cli.shards as usize);
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(shard_result) => {
                if !cli.quiet {
                    print_shard_output(&shard_result);
                }
                results.push(shard_result);
            }
            Err(e) => {
                eprintln!("[pso] shard task panicked: {e}");
            }
        }
    }

    results.sort_by_key(|r| r.shard_index);

    RunSummary {
        total_shards: cli.shards,
        workers_per_shard: cli.workers,
        memory_mb: cli.memory_mb,
        total_duration: total_start.elapsed(),
        results,
    }
}

fn print_shard_output(result: &ShardResult) {
    let marker = match &result.outcome {
        ShardOutcome::Passed => "PASS",
        ShardOutcome::Failed { .. } => "FAIL",
        ShardOutcome::Error { .. } => "ERR ",
    };
    eprintln!(
        "\n--- [{marker}] shard {} ({:.1}s) ---",
        result.shard_index,
        result.duration.as_secs_f64()
    );
    if !result.stdout.is_empty() {
        eprintln!("{}", result.stdout);
    }
    if !result.stderr.is_empty() {
        eprintln!("{}", result.stderr);
    }
}
