use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "pso",
    about = "Playwright Shard Orchestrator – run Playwright test shards in parallel",
    long_about = "Spawns N parallel Playwright processes, each running a distinct shard \
                  (--shard=i/N). Supports per-shard worker counts, resource limits, and \
                  automatic IS_SHARDED environment injection so the test framework knows \
                  it is running under the orchestrator."
)]
pub struct Cli {
    /// Number of shards to split the test suite into
    #[arg(short, long)]
    pub shards: u32,

    /// Number of Playwright workers inside each shard
    #[arg(short, long, default_value_t = 1)]
    pub workers: u32,

    /// Root of the project containing the Playwright config
    #[arg(short, long, default_value = "/home/bmaio/Developer/Projects/kubevirt-ui")]
    pub project_dir: PathBuf,

    /// Playwright config path relative to project_dir
    #[arg(long, default_value = "playwright/playwright.config.ts")]
    pub config: String,

    /// Optional grep filter passed to Playwright (e.g. "@tier1", "@smoke")
    #[arg(short, long)]
    pub grep: Option<String>,

    /// Optional grep-invert filter
    #[arg(long)]
    pub grep_invert: Option<String>,

    /// Test file or glob to run (e.g. "checkups.spec.ts")
    #[arg(short, long)]
    pub file: Option<String>,

    /// Number of retries for failed tests within each shard
    #[arg(short, long)]
    pub retries: Option<u32>,

    /// Per-shard memory limit in MB (sets NODE_OPTIONS --max-old-space-size)
    #[arg(long, default_value_t = 4096)]
    pub memory_mb: u32,

    /// Run browsers in headed mode
    #[arg(long, default_value_t = false)]
    pub headed: bool,

    /// Extra environment variables to pass to each shard (KEY=VALUE), repeatable
    #[arg(short, long)]
    pub env: Vec<String>,

    /// Timeout per test in milliseconds (overrides config default)
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Suppress per-shard stdout, only show the summary
    #[arg(long, default_value_t = false)]
    pub quiet: bool,
}

impl Cli {
    #[allow(dead_code)]
    pub fn fixture() -> Self {
        Self {
            shards: 4,
            workers: 2,
            project_dir: PathBuf::from("/tmp/fake-project"),
            config: "playwright/playwright.config.ts".into(),
            grep: None,
            grep_invert: None,
            file: None,
            retries: None,
            memory_mb: 4096,
            headed: false,
            env: vec![],
            timeout: None,
            quiet: false,
        }
    }
}
