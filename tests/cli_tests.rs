use std::process::Command;

fn pso_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pso"))
}

#[test]
fn missing_shards_flag_prints_error() {
    let out = pso_bin().output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--shards"),
        "expected missing --shards hint, got: {stderr}"
    );
}

#[test]
fn help_flag_succeeds() {
    let out = pso_bin().arg("--help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Playwright processes"));
    assert!(stdout.contains("--shards"));
    assert!(stdout.contains("--workers"));
    assert!(stdout.contains("--memory-mb"));
}

#[test]
fn shards_zero_exits_with_failure() {
    let out = pso_bin().args(["--shards", "0"]).output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--shards must be >= 1"),
        "expected validation error, got: {stderr}"
    );
}

#[test]
fn invalid_shards_value_rejected() {
    let out = pso_bin().args(["--shards", "abc"]).output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn env_flag_parsed_correctly() {
    let out = pso_bin()
        .args([
            "--shards", "1",
            "--workers", "1",
            "-e", "FOO=bar",
            "-e", "BAZ=qux",
            "--project-dir", "/nonexistent",
            "--quiet",
        ])
        .output()
        .unwrap();
    // Will fail because /nonexistent doesn't have playwright, but that's
    // after arg parsing succeeds — we just verify it didn't reject the flags.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("PSO: Launching 1 shard"),
        "expected launch message, got: {stderr}"
    );
}
