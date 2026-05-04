use pso::cli::Cli;
use pso::shard::{build_args, build_env};

#[test]
fn build_env_sets_is_sharded() {
    let cli = Cli::fixture();
    let env = build_env(&cli);
    assert_eq!(env.get("IS_SHARDED").unwrap(), "true");
}

#[test]
fn build_env_sets_workers() {
    let cli = Cli::fixture();
    let env = build_env(&cli);
    assert_eq!(env.get("WORKERS").unwrap(), "2");
}

#[test]
fn build_env_sets_node_options() {
    let cli = Cli::fixture();
    let env = build_env(&cli);
    assert_eq!(
        env.get("NODE_OPTIONS").unwrap(),
        "--max-old-space-size=4096"
    );
}

#[test]
fn build_env_custom_memory() {
    let mut cli = Cli::fixture();
    cli.memory_mb = 8192;
    let env = build_env(&cli);
    assert_eq!(
        env.get("NODE_OPTIONS").unwrap(),
        "--max-old-space-size=8192"
    );
}

#[test]
fn build_env_does_not_include_shard_index() {
    let cli = Cli::fixture();
    let env = build_env(&cli);
    assert!(!env.contains_key("SHARD_INDEX"));
    assert!(!env.contains_key("SHARD_TOTAL"));
}

#[test]
fn build_env_headed_sets_env() {
    let mut cli = Cli::fixture();
    cli.headed = true;
    let env = build_env(&cli);
    assert_eq!(env.get("HEADED").unwrap(), "1");
}

#[test]
fn build_env_no_headed_by_default() {
    let cli = Cli::fixture();
    let env = build_env(&cli);
    assert!(!env.contains_key("HEADED"));
}

#[test]
fn build_env_custom_env_pairs() {
    let mut cli = Cli::fixture();
    cli.env = vec![
        "BRIDGE_BASE_ADDRESS=https://console.example.com".into(),
        "MY_VAR=123".into(),
    ];
    let env = build_env(&cli);
    assert_eq!(
        env.get("BRIDGE_BASE_ADDRESS").unwrap(),
        "https://console.example.com"
    );
    assert_eq!(env.get("MY_VAR").unwrap(), "123");
}

#[test]
fn build_env_malformed_pair_ignored() {
    let mut cli = Cli::fixture();
    cli.env = vec!["NO_EQUALS_SIGN".into()];
    let env = build_env(&cli);
    assert!(!env.contains_key("NO_EQUALS_SIGN"));
}

#[test]
fn build_args_basic_shard() {
    let cli = Cli::fixture();
    let args = build_args(&cli, 1);
    assert_eq!(args[0], "test");
    assert_eq!(args[1], "--config=playwright/playwright.config.ts");
    assert_eq!(args[2], "--shard=1/4");
    assert_eq!(args[3], "--workers=2");
    assert_eq!(args.len(), 4);
}

#[test]
fn build_args_different_shard_indices() {
    let cli = Cli::fixture();
    for i in 1..=cli.shards {
        let args = build_args(&cli, i);
        assert_eq!(args[2], format!("--shard={}/{}", i, cli.shards));
    }
}

#[test]
fn build_args_with_grep() {
    let mut cli = Cli::fixture();
    cli.grep = Some("@tier1".into());
    let args = build_args(&cli, 1);
    assert!(args.contains(&"--grep=@tier1".to_string()));
}

#[test]
fn build_args_with_grep_invert() {
    let mut cli = Cli::fixture();
    cli.grep_invert = Some("@flaky".into());
    let args = build_args(&cli, 1);
    assert!(args.contains(&"--grep-invert=@flaky".to_string()));
}

#[test]
fn build_args_with_retries() {
    let mut cli = Cli::fixture();
    cli.retries = Some(3);
    let args = build_args(&cli, 1);
    assert!(args.contains(&"--retries=3".to_string()));
}

#[test]
fn build_args_with_timeout() {
    let mut cli = Cli::fixture();
    cli.timeout = Some(60000);
    let args = build_args(&cli, 1);
    assert!(args.contains(&"--timeout=60000".to_string()));
}

#[test]
fn build_args_with_headed() {
    let mut cli = Cli::fixture();
    cli.headed = true;
    let args = build_args(&cli, 1);
    assert!(args.contains(&"--headed".to_string()));
}

#[test]
fn build_args_with_file() {
    let mut cli = Cli::fixture();
    cli.file = Some("checkups.spec.ts".into());
    let args = build_args(&cli, 1);
    assert_eq!(args.last().unwrap(), "checkups.spec.ts");
}

#[test]
fn build_args_all_options() {
    let mut cli = Cli::fixture();
    cli.grep = Some("@smoke".into());
    cli.grep_invert = Some("@skip".into());
    cli.retries = Some(2);
    cli.timeout = Some(120000);
    cli.headed = true;
    cli.file = Some("vm.spec.ts".into());

    let args = build_args(&cli, 3);

    assert_eq!(args[0], "test");
    assert!(args.contains(&"--shard=3/4".to_string()));
    assert!(args.contains(&"--grep=@smoke".to_string()));
    assert!(args.contains(&"--grep-invert=@skip".to_string()));
    assert!(args.contains(&"--retries=2".to_string()));
    assert!(args.contains(&"--timeout=120000".to_string()));
    assert!(args.contains(&"--headed".to_string()));
    assert_eq!(args.last().unwrap(), "vm.spec.ts");
}
