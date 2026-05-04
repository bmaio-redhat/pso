use std::time::Duration;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum ShardOutcome {
    Passed,
    Failed { exit_code: i32 },
    Error { message: String },
}

#[derive(Debug, Serialize)]
pub struct ShardResult {
    pub shard_index: u32,
    pub outcome: ShardOutcome,
    pub duration: Duration,
    #[serde(skip)]
    pub stdout: String,
    #[serde(skip)]
    pub stderr: String,
}

#[derive(Debug, Serialize)]
pub struct RunSummary {
    pub total_shards: u32,
    pub workers_per_shard: u32,
    pub memory_mb: u32,
    pub total_duration: Duration,
    pub results: Vec<ShardResult>,
}

impl RunSummary {
    pub fn all_passed(&self) -> bool {
        self.results
            .iter()
            .all(|r| matches!(r.outcome, ShardOutcome::Passed))
    }

    pub fn print(&self) {
        eprintln!("\n{}", "=".repeat(60));
        eprintln!("  PSO Run Summary");
        eprintln!("{}", "=".repeat(60));
        eprintln!(
            "  Shards: {}  |  Workers/shard: {}  |  Memory/shard: {}MB",
            self.total_shards, self.workers_per_shard, self.memory_mb
        );
        eprintln!(
            "  Wall time: {:.1}s",
            self.total_duration.as_secs_f64()
        );
        eprintln!("{}", "-".repeat(60));

        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut errored = 0u32;

        for r in &self.results {
            let (icon, label) = match &r.outcome {
                ShardOutcome::Passed => {
                    passed += 1;
                    ("\x1b[32m✓\x1b[0m", "passed".to_string())
                }
                ShardOutcome::Failed { exit_code } => {
                    failed += 1;
                    ("\x1b[31m✗\x1b[0m", format!("failed (exit {})", exit_code))
                }
                ShardOutcome::Error { message } => {
                    errored += 1;
                    ("\x1b[33m!\x1b[0m", format!("error: {}", message))
                }
            };
            eprintln!(
                "  {icon} shard {:<3} {:<30} ({:.1}s)",
                r.shard_index,
                label,
                r.duration.as_secs_f64()
            );
        }

        eprintln!("{}", "-".repeat(60));
        eprintln!(
            "  \x1b[32m{passed} passed\x1b[0m, \x1b[31m{failed} failed\x1b[0m, \x1b[33m{errored} errors\x1b[0m"
        );
        eprintln!("{}\n", "=".repeat(60));
    }
}
