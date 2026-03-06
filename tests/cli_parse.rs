use std::process::{Command, Output};

fn run_bin(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ign-inspect"))
        .args(args)
        .output()
        .expect("binary should execute")
}

#[test]
fn summarize_runs_and_prints_parse_stub_output() {
    let out = run_bin(&["summarize", "sample.zip"]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Parsed `summarize` command."));
    assert!(stdout.contains("archive_path: sample.zip"));
}

#[test]
fn analyze_runs_and_prints_parse_stub_output() {
    let out = run_bin(&["analyze", "sample.gwbk", "--out-dir", "./out"]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Parsed `analyze` command."));
    assert!(stdout.contains("archive_path: sample.gwbk"));
    assert!(stdout.contains("out_dir: ./out"));
}

#[test]
fn analyze_missing_out_dir_fails_with_usage_exit_code() {
    let out = run_bin(&["analyze", "sample.zip"]);
    assert_eq!(out.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("required arguments were not provided"));
    assert!(stderr.contains("--out-dir"));
}

#[test]
fn unknown_subcommand_fails_with_usage_exit_code() {
    let out = run_bin(&["inspect", "sample.zip"]);
    assert_eq!(out.status.code(), Some(2));

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("unrecognized subcommand"));
}
