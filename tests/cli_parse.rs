use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

fn run_bin(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_igniscope"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn run_bin_os(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_igniscope"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn fixture_path(file_name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("example-files")
        .join(file_name)
}

#[test]
fn summarize_project_export_reports_core_summary_fields() {
    let archive = fixture_path("Template_v8.3_example.zip");
    let out = run_bin_os(&[std::ffi::OsStr::new("summarize"), archive.as_os_str()]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("archive_kind: project_export"));
    assert!(stdout.contains("projects_total: 1"));
    assert!(stdout.contains("selected_project_roots: [\"\"]"));
}

#[test]
fn summarize_verbose_includes_per_project_lines() {
    let archive = fixture_path("multi-project.gwbk");
    let out = run_bin_os(&[
        std::ffi::OsStr::new("-v"),
        std::ffi::OsStr::new("summarize"),
        archive.as_os_str(),
    ]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("project: root=projects/IADemo/"));
    assert!(stdout.contains("project: root=projects/samplequickstart/"));
}

#[test]
fn analyze_writes_analytics_and_report_files() {
    let archive = fixture_path("multi-project.gwbk");
    let out_dir = unique_out_dir("analyze_writes");

    let out = run_bin_os(&[
        std::ffi::OsStr::new("analyze"),
        archive.as_os_str(),
        std::ffi::OsStr::new("--out-dir"),
        out_dir.as_os_str(),
    ]);
    assert!(out.status.success());

    let analytics_path = out_dir.join("analytics.json");
    let report_path = out_dir.join("report.md");
    assert!(analytics_path.exists(), "analytics.json should exist");
    assert!(report_path.exists(), "report.md should exist");

    let analytics = read_json(&analytics_path);
    assert_eq!(analytics["schema_version"], "0.1.0");
    assert_eq!(analytics["input"]["archive_kind"], "gateway_backup");
    assert_eq!(analytics["summary"]["projects_total"], 8);
    assert_eq!(
        analytics["projects"].as_array().map(|value| value.len()),
        Some(8)
    );

    let report = fs::read_to_string(&report_path).expect("report should be readable");
    assert!(report.contains("# igniscope report"));
    assert!(report.contains("## Input Summary"));
    assert!(report.contains("## Overall Aggregate Summary"));
    assert!(report.contains("## Per-Project Details"));
    assert!(report.contains("### Project `projects/IADemo/`"));
    assert!(report.contains("### Project `projects/samplequickstart/`"));

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("archive_kind: gateway_backup"));
    assert!(stdout.contains("analytics_json:"));
    assert!(stdout.contains("report_md:"));

    cleanup_out_dir(&out_dir);
}

#[test]
fn analyze_outputs_are_deterministic_for_same_input_and_out_dir() {
    let archive = fixture_path("Template_v8.3_example.zip");
    let out_dir = unique_out_dir("analyze_deterministic");

    let first = run_bin_os(&[
        std::ffi::OsStr::new("analyze"),
        archive.as_os_str(),
        std::ffi::OsStr::new("--out-dir"),
        out_dir.as_os_str(),
    ]);
    assert!(first.status.success());

    let analytics_path = out_dir.join("analytics.json");
    let report_path = out_dir.join("report.md");
    let analytics_first = fs::read_to_string(&analytics_path).expect("analytics should exist");
    let report_first = fs::read_to_string(&report_path).expect("report should exist");

    let second = run_bin_os(&[
        std::ffi::OsStr::new("analyze"),
        archive.as_os_str(),
        std::ffi::OsStr::new("--out-dir"),
        out_dir.as_os_str(),
    ]);
    assert!(second.status.success());

    let analytics_second =
        fs::read_to_string(&analytics_path).expect("analytics should exist after rerun");
    let report_second = fs::read_to_string(&report_path).expect("report should exist after rerun");

    assert_eq!(analytics_first, analytics_second);
    assert_eq!(report_first, report_second);

    cleanup_out_dir(&out_dir);
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

#[test]
fn summarize_unknown_archive_shape_fails_with_project_root_detection_exit_code() {
    let archive = fixture_path("data_center_industry_pack.1.1.0.zip");
    let out = run_bin_os(&[std::ffi::OsStr::new("summarize"), archive.as_os_str()]);

    assert_eq!(out.status.code(), Some(11));

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Could not detect project roots"));
}

/// Returns a unique output directory path under the system temp directory.
fn unique_out_dir(prefix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    path.push(format!("igniscope-{prefix}-{}-{now}", std::process::id()));
    path
}

/// Reads and parses a JSON file.
fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).expect("json file should be readable");
    serde_json::from_str(&text).expect("json should parse")
}

/// Cleans up a test output directory if it exists.
fn cleanup_out_dir(out_dir: &Path) {
    if out_dir.exists() {
        fs::remove_dir_all(out_dir).expect("test output directory should be removable");
    }
}
