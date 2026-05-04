use std::time::Duration;

use pso::report::{RunSummary, ShardOutcome, ShardResult};

fn make_result(index: u32, outcome: ShardOutcome) -> ShardResult {
    ShardResult {
        shard_index: index,
        outcome,
        duration: Duration::from_secs(10),
        stdout: String::new(),
        stderr: String::new(),
    }
}

#[test]
fn all_passed_when_all_shards_pass() {
    let summary = RunSummary {
        total_shards: 3,
        workers_per_shard: 2,
        memory_mb: 4096,
        total_duration: Duration::from_secs(30),
        results: vec![
            make_result(1, ShardOutcome::Passed),
            make_result(2, ShardOutcome::Passed),
            make_result(3, ShardOutcome::Passed),
        ],
    };
    assert!(summary.all_passed());
}

#[test]
fn not_all_passed_when_one_fails() {
    let summary = RunSummary {
        total_shards: 3,
        workers_per_shard: 2,
        memory_mb: 4096,
        total_duration: Duration::from_secs(30),
        results: vec![
            make_result(1, ShardOutcome::Passed),
            make_result(2, ShardOutcome::Failed { exit_code: 1 }),
            make_result(3, ShardOutcome::Passed),
        ],
    };
    assert!(!summary.all_passed());
}

#[test]
fn not_all_passed_when_one_errors() {
    let summary = RunSummary {
        total_shards: 2,
        workers_per_shard: 1,
        memory_mb: 4096,
        total_duration: Duration::from_secs(20),
        results: vec![
            make_result(1, ShardOutcome::Passed),
            make_result(
                2,
                ShardOutcome::Error {
                    message: "spawn failed".into(),
                },
            ),
        ],
    };
    assert!(!summary.all_passed());
}

#[test]
fn all_passed_with_empty_results() {
    let summary = RunSummary {
        total_shards: 0,
        workers_per_shard: 1,
        memory_mb: 4096,
        total_duration: Duration::from_secs(0),
        results: vec![],
    };
    assert!(summary.all_passed());
}

#[test]
fn summary_serializes_to_json() {
    let summary = RunSummary {
        total_shards: 2,
        workers_per_shard: 1,
        memory_mb: 2048,
        total_duration: Duration::from_millis(15500),
        results: vec![
            make_result(1, ShardOutcome::Passed),
            make_result(2, ShardOutcome::Failed { exit_code: 1 }),
        ],
    };
    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("\"total_shards\":2"));
    assert!(json.contains("\"Passed\""));
    assert!(json.contains("\"exit_code\":1"));
}
