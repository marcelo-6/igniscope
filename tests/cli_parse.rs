use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn run_bin(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ign-inspect"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn run_bin_os(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ign-inspect"))
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
fn summarize_project_export_reports_detected_archive_kind_and_root() {
    let archive = fixture_path("Template_v8.3_example.zip");
    let out = run_bin_os(&[std::ffi::OsStr::new("summarize"), archive.as_os_str()]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Parsed `summarize` command."));
    assert!(stdout.contains("archive_kind: project_export"));
    assert!(stdout.contains("detected_project_roots: [\"\"]"));
    assert!(stdout.contains("selected_project_roots: [\"\"]"));
}

#[test]
fn analyze_gateway_backup_reports_all_project_roots() {
    let archive = fixture_path("multi-project.gwbk");
    let out = run_bin_os(&[
        std::ffi::OsStr::new("analyze"),
        archive.as_os_str(),
        std::ffi::OsStr::new("--out-dir"),
        std::ffi::OsStr::new("./out"),
    ]);
    assert!(out.status.success());

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Parsed `analyze` command."));
    assert!(stdout.contains("archive_kind: gateway_backup"));
    assert!(stdout.contains("projects/IADemo/"));
    assert!(stdout.contains("projects/OnlineDemo/"));
    assert!(stdout.contains("projects/TagDashboard/"));
    assert!(stdout.contains("projects/building-management-system-demo/"));
    assert!(stdout.contains("projects/global/"));
    assert!(stdout.contains("projects/oil-and-gas-demo/"));
    assert!(stdout.contains("projects/prepared-foods-line-demo/"));
    assert!(stdout.contains("projects/samplequickstart/"));
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

#[test]
fn summarize_unknown_archive_shape_fails_with_project_root_detection_exit_code() {
    let archive = fixture_path("data_center_industry_pack.1.1.0.zip");
    let out = run_bin_os(&[std::ffi::OsStr::new("summarize"), archive.as_os_str()]);

    assert_eq!(out.status.code(), Some(11));

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Could not detect project roots"));
}
