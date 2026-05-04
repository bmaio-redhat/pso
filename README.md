# PSO – Playwright Shard Orchestrator

A Rust CLI tool that runs Playwright test shards in parallel. It spawns N independent
Playwright processes — each executing a distinct `--shard=i/N` slice of the test suite —
with configurable worker counts, memory limits, and automatic `IS_SHARDED` environment
injection so the test framework knows it is running under the orchestrator.

## Project layout

```
pso/
├── Cargo.toml              # Crate manifest and dependencies
├── Dockerfile              # Multi-stage build (Rust compile → slim Debian runtime)
├── src/
│   ├── main.rs             # Entry point, argument parsing, exit code
│   ├── lib.rs              # Re-exports modules for integration tests
│   ├── cli.rs              # CLI argument definitions (clap derive)
│   ├── shard.rs            # Shard orchestration: env building, arg building, process spawning
│   └── report.rs           # ShardResult / RunSummary types and terminal summary printer
└── tests/
    ├── cli_tests.rs        # Binary-level tests (help, validation, flag parsing)
    ├── shard_tests.rs      # Unit tests for build_env() and build_args()
    └── report_tests.rs     # Unit tests for RunSummary (all_passed, serialization)
```

## Prerequisites

- Rust 1.85+ (edition 2024)
- A project with Playwright installed (the target project must have `node_modules/.bin/playwright` or `npx` available)

## Building

```bash
# Debug build
cargo build

# Optimised release build
cargo build --release

# The binary lands at target/release/pso
```

## Usage

```
pso --shards <N> [OPTIONS]
```

### Required

| Flag | Description |
|------|-------------|
| `-s, --shards <N>` | Number of shards to split the test suite into |

### Optional

| Flag | Default | Description |
|------|---------|-------------|
| `-w, --workers <N>` | `1` | Playwright workers inside each shard |
| `-p, --project-dir <PATH>` | current kubevirt-ui path | Root of the project containing the Playwright config |
| `--config <PATH>` | `playwright/playwright.config.ts` | Playwright config path relative to project dir |
| `-g, --grep <PATTERN>` | — | Grep filter passed to Playwright (e.g. `@tier1`, `@smoke`) |
| `--grep-invert <PATTERN>` | — | Exclude tests matching this pattern |
| `-f, --file <GLOB>` | — | Test file or glob to run (e.g. `checkups.spec.ts`) |
| `-r, --retries <N>` | — | Number of retries for failed tests within each shard |
| `--memory-mb <MB>` | `4096` | Per-shard memory limit (sets `NODE_OPTIONS --max-old-space-size`) |
| `--headed` | `false` | Run browsers in headed mode |
| `-e, --env <KEY=VALUE>` | — | Extra env vars passed to each shard (repeatable) |
| `--timeout <MS>` | — | Timeout per test in milliseconds |
| `-v, --verbose` | `false` | Show full per-shard stdout/stderr (default: only start and summary) |

### Examples

```bash
# 4 shards, 2 workers each, default 4GB memory
pso --shards 4 --workers 2

# Run only tier1 tests with retries
pso --shards 3 --workers 1 --grep "@tier1" --retries 2

# Custom project directory and extra env vars
pso -s 2 -w 3 -p /path/to/project \
  -e "BRIDGE_BASE_ADDRESS=https://console.example.com" \
  -e "BRIDGE_KUBEADMIN_PASSWORD=secret"

# Single shard with higher memory, verbose to see full output
pso --shards 1 --workers 4 --memory-mb 8192 --verbose

# Specific test file across 2 shards
pso --shards 2 -f "checkups.spec.ts"
```

## Environment variables injected per shard

Every spawned Playwright process receives the following environment variables automatically:

| Variable | Value | Purpose |
|----------|-------|---------|
| `IS_SHARDED` | `true` | Signals the test framework that it is running under the shard orchestrator |
| `SHARD_INDEX` | `1`, `2`, … `N` | The 1-based index of this shard, for shard-specific file paths |
| `SHARD_TOTAL` | value of `--shards` | Total number of shards in this run |
| `WORKERS` | value of `--workers` | Consumed by the Playwright config to set worker count |
| `NODE_OPTIONS` | `--max-old-space-size=<memory-mb>` | Limits Node.js heap memory per shard |
| `HEADED` | `1` (only when `--headed` is set) | Runs browsers in headed mode |

Any additional variables passed via `-e KEY=VALUE` are forwarded as-is.

### Shard-specific files

Each shard knows its own index via `SHARD_INDEX`. Use this in your test framework
to isolate per-shard resources:

```typescript
const shardIndex = process.env.SHARD_INDEX ?? '0';

const kubeconfig = `testconfig_${shardIndex}`;
const sharedState = `shared_state_${shardIndex}.json`;
```

This prevents shards from colliding on shared files like kubeconfigs or state JSON.

## How sharding works

1. PSO receives `--shards=N` and spawns N tokio tasks concurrently.
2. Each task runs `playwright test --shard=i/N --workers=W` as a child process
   with the injected environment.
3. Playwright internally splits the test file list into N buckets and only runs
   bucket `i`, so there is no test overlap between shards.
4. As each shard completes, its stdout/stderr is printed (unless `--quiet`).
5. After all shards finish, a summary table is printed showing pass/fail status,
   exit codes, and per-shard duration.
6. PSO exits with code 0 only if every shard passed; otherwise it exits with code 1.

## Docker

The included Dockerfile produces a minimal image containing only the `pso` binary.
It is designed to be composed alongside a Playwright image that provides Node.js and
browser binaries.

```bash
# Build the image
docker build -t pso .

# Run (mount the test project in)
docker run --rm -v /path/to/project:/project pso --shards 4 -w 2 -p /project
```

## Testing

```bash
# Run all tests
cargo test

# Run a specific test suite
cargo test --test cli_tests
cargo test --test shard_tests
cargo test --test report_tests
```

### Test suites

| Suite | Count | Coverage |
|-------|-------|----------|
| `cli_tests` | 5 | Binary-level: help output, missing flags, validation, env flag parsing |
| `shard_tests` | 17 | `build_env()` and `build_args()`: IS_SHARDED, WORKERS, NODE_OPTIONS, grep, retries, timeout, headed, file, custom env pairs |
| `report_tests` | 5 | `RunSummary::all_passed()` for pass/fail/error/empty, JSON serialization |
