use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::archive::{
    AnalyticsBundle, build_analytics_bundle, discover_resources_for_roots, inspect_archive,
    parse_project_metadata,
};
use crate::error::AppError;

const GENERATED_AT_PLACEHOLDER: &str = "1970-01-01T00:00:00Z";
const ANALYTICS_FILE_NAME: &str = "analytics.json";
const REPORT_FILE_NAME: &str = "report.md";

/// Handles the `summarize` command for archive inspection and project parsing.
pub fn run_summarize(archive_path: &Path, verbose: u8) -> Result<(), AppError> {
    let analytics = run_pipeline(archive_path)?;
    let output = render_summary_text(archive_path, &analytics, verbose);
    print!("{output}");
    Ok(())
}

/// Handles the `analyze` command for archive inspection and project parsing.
pub fn run_analyze(archive_path: &Path, out_dir: &Path, verbose: u8) -> Result<(), AppError> {
    let analytics = run_pipeline(archive_path)?;
    let output_paths = write_outputs(out_dir, &analytics)?;
    let summary_output = render_summary_text(archive_path, &analytics, verbose);
    print!("{summary_output}");
    println!("analytics_json: {}", output_paths.analytics_json.display());
    println!("report_md: {}", output_paths.report_md.display());
    Ok(())
}

/// Runs the full inspection and analytics pipeline for one archive.
fn run_pipeline(archive_path: &Path) -> Result<AnalyticsBundle, AppError> {
    let inspection = inspect_archive(archive_path)?;
    let project_metadata =
        parse_project_metadata(archive_path, &inspection.selected_project_roots)?;
    let resource_inventories =
        discover_resources_for_roots(archive_path, &inspection.selected_project_roots)?;
    build_analytics_bundle(
        archive_path,
        GENERATED_AT_PLACEHOLDER,
        &inspection,
        &project_metadata,
        &resource_inventories,
    )
}

/// Writes deterministic output artifacts to the requested directory.
fn write_outputs(out_dir: &Path, analytics: &AnalyticsBundle) -> Result<OutputPaths, AppError> {
    fs::create_dir_all(out_dir).map_err(|err| {
        AppError::internal(format!(
            "could not create output directory `{}`: {err}",
            out_dir.display()
        ))
    })?;

    let analytics_json_path = out_dir.join(ANALYTICS_FILE_NAME);
    let report_path = out_dir.join(REPORT_FILE_NAME);

    let analytics_json = serde_json::to_string_pretty(analytics).map_err(|err| {
        AppError::internal(format!(
            "could not serialize analytics output to JSON: {err}"
        ))
    })?;
    fs::write(&analytics_json_path, format!("{analytics_json}\n")).map_err(|err| {
        AppError::internal(format!(
            "could not write `{}`: {err}",
            analytics_json_path.display()
        ))
    })?;

    let report = render_report_markdown(analytics);
    fs::write(&report_path, report).map_err(|err| {
        AppError::internal(format!(
            "could not write `{}`: {err}",
            report_path.display()
        ))
    })?;

    Ok(OutputPaths {
        analytics_json: analytics_json_path,
        report_md: report_path,
    })
}

/// Renders deterministic `summarize` stdout text from analytics output.
fn render_summary_text(archive_path: &Path, analytics: &AnalyticsBundle, verbose: u8) -> String {
    let mut output = String::new();
    let _ = writeln!(&mut output, "archive_path: {}", archive_path.display());
    let _ = writeln!(
        &mut output,
        "archive_kind: {}",
        analytics.input.archive_kind
    );
    let _ = writeln!(
        &mut output,
        "projects_total: {}",
        analytics.summary.projects_total
    );
    let _ = writeln!(
        &mut output,
        "selected_project_roots: {:?}",
        analytics.input.selected_project_roots
    );
    let _ = writeln!(
        &mut output,
        "resources_total: {}",
        analytics.summary.resources_total
    );
    let _ = writeln!(
        &mut output,
        "files_total: {}",
        analytics.summary.files_total
    );
    let _ = writeln!(
        &mut output,
        "unknown_ratio: {:.6}",
        analytics.summary.unknown_ratio
    );

    if verbose > 0 {
        for project in &analytics.projects {
            let _ = writeln!(
                &mut output,
                "project: root={} title={} resources={} unknown_ratio={:.6}",
                display_project_root(&project.project_root),
                project.project.title,
                project.counts.resources_total,
                project.coverage.unknown_ratio
            );
        }
    }

    output
}

/// Renders deterministic markdown report output for `analyze`.
fn render_report_markdown(analytics: &AnalyticsBundle) -> String {
    let mut report = String::new();

    let _ = writeln!(&mut report, "# igniscope report");
    let _ = writeln!(&mut report);

    let _ = writeln!(&mut report, "## Input Summary");
    let _ = writeln!(&mut report);
    let _ = writeln!(
        &mut report,
        "- archive_kind: `{}`",
        analytics.input.archive_kind
    );
    let _ = writeln!(
        &mut report,
        "- projects_total: {}",
        analytics.summary.projects_total
    );
    let _ = writeln!(&mut report, "- selected_project_roots:");
    for project_root in &analytics.input.selected_project_roots {
        let _ = writeln!(&mut report, "  - `{}`", display_project_root(project_root));
    }
    let _ = writeln!(&mut report);

    let _ = writeln!(&mut report, "## Overall Aggregate Summary");
    let _ = writeln!(&mut report);
    let _ = writeln!(
        &mut report,
        "- resources_total: {}",
        analytics.summary.resources_total
    );
    let _ = writeln!(
        &mut report,
        "- files_total: {}",
        analytics.summary.files_total
    );
    let _ = writeln!(
        &mut report,
        "- binary_only_resources: {}",
        analytics.summary.binary_only_resources
    );
    let _ = writeln!(&mut report);

    write_count_map_section(
        &mut report,
        "## Counts By Section",
        &analytics.summary.resources_by_section,
    );
    write_count_map_section(
        &mut report,
        "## Counts By Type Key",
        &analytics.summary.resources_by_type,
    );
    write_count_map_section(
        &mut report,
        "## File Kind Breakdown",
        &analytics.summary.files_by_kind,
    );

    let _ = writeln!(&mut report, "## Coverage Summary");
    let _ = writeln!(&mut report);
    let _ = writeln!(
        &mut report,
        "- unknown_resources: {}",
        analytics.summary.unknown_resources
    );
    let _ = writeln!(
        &mut report,
        "- unknown_ratio: {:.6}",
        analytics.summary.unknown_ratio
    );
    let _ = writeln!(&mut report);

    let _ = writeln!(&mut report, "## Per-Project Details");
    let _ = writeln!(&mut report);
    for project in &analytics.projects {
        let _ = writeln!(
            &mut report,
            "### Project `{}`",
            display_project_root(&project.project_root)
        );
        let _ = writeln!(&mut report);
        let _ = writeln!(&mut report, "- title: {}", project.project.title);
        let _ = writeln!(
            &mut report,
            "- description: {:?}",
            project.project.description
        );
        let _ = writeln!(&mut report, "- parent: {:?}", project.project.parent);
        let _ = writeln!(&mut report, "- enabled: {}", project.project.enabled);
        let _ = writeln!(
            &mut report,
            "- inheritable: {}",
            project.project.inheritable
        );
        let _ = writeln!(
            &mut report,
            "- resources_total: {}",
            project.counts.resources_total
        );
        let _ = writeln!(&mut report, "- files_total: {}", project.counts.files_total);
        let _ = writeln!(
            &mut report,
            "- binary_only_resources: {}",
            project.counts.binary_only_resources
        );
        let _ = writeln!(
            &mut report,
            "- unknown_resources: {}",
            project.coverage.unknown_resources
        );
        let _ = writeln!(
            &mut report,
            "- unknown_ratio: {:.6}",
            project.coverage.unknown_ratio
        );
        let _ = writeln!(&mut report);

        write_project_count_subsection(
            &mut report,
            "#### Counts By Section",
            &project.counts.resources_by_section,
        );
        write_project_count_subsection(
            &mut report,
            "#### Counts By Type Key",
            &project.counts.resources_by_type,
        );
        write_project_count_subsection(
            &mut report,
            "#### File Kind Breakdown",
            &project.counts.files_by_kind,
        );

        if project.issues.is_empty() {
            let _ = writeln!(&mut report, "- issues: no issues");
        } else {
            let _ = writeln!(&mut report, "- issues:");
            for issue in &project.issues {
                let _ = writeln!(&mut report, "  - {issue}");
            }
        }
        let _ = writeln!(&mut report);
    }

    let _ = writeln!(&mut report, "## Issues");
    let _ = writeln!(&mut report);
    if analytics.issues.is_empty() {
        let _ = writeln!(&mut report, "No issues.");
    } else {
        for issue in &analytics.issues {
            let _ = writeln!(&mut report, "- {issue}");
        }
    }
    report
}

/// Writes a top-level summary count section as deterministic markdown bullets.
fn write_count_map_section(
    report: &mut String,
    heading: &str,
    counts: &std::collections::BTreeMap<String, usize>,
) {
    let _ = writeln!(report, "{heading}");
    let _ = writeln!(report);
    if counts.is_empty() {
        let _ = writeln!(report, "- none");
    } else {
        for (key, value) in counts {
            let _ = writeln!(report, "- `{key}`: {value}");
        }
    }
    let _ = writeln!(report);
}

/// Writes a per-project count subsection as deterministic markdown bullets.
fn write_project_count_subsection(
    report: &mut String,
    heading: &str,
    counts: &std::collections::BTreeMap<String, usize>,
) {
    let _ = writeln!(report, "{heading}");
    let _ = writeln!(report);
    if counts.is_empty() {
        let _ = writeln!(report, "- none");
    } else {
        for (key, value) in counts {
            let _ = writeln!(report, "- `{key}`: {value}");
        }
    }
    let _ = writeln!(report);
}

/// Normalizes project-root display to avoid empty-string output in reports.
fn display_project_root(project_root: &str) -> &str {
    if project_root.is_empty() {
        "(root)"
    } else {
        project_root
    }
}

#[derive(Debug)]
struct OutputPaths {
    analytics_json: PathBuf,
    report_md: PathBuf,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use crate::archive::{
        AnalyticsInput, AnalyticsSummary, CoverageMetrics, ProjectAnalytics, ProjectCounts,
        ProjectMetadata,
    };

    use super::{display_project_root, render_report_markdown, render_summary_text};

    fn synthetic_analytics() -> crate::archive::AnalyticsBundle {
        crate::archive::AnalyticsBundle {
            schema_version: "0.1.0".to_string(),
            generated_at: "1970-01-01T00:00:00Z".to_string(),
            input: AnalyticsInput {
                archive_path: "/tmp/synthetic.zip".to_string(),
                archive_kind: "project_export".to_string(),
                detected_project_roots: vec!["".to_string()],
                selected_project_roots: vec!["".to_string()],
            },
            summary: AnalyticsSummary {
                projects_total: 1,
                resources_total: 3,
                files_total: 5,
                binary_only_resources: 1,
                resources_by_section: BTreeMap::from([("Perspective".to_string(), 3usize)]),
                resources_by_type: BTreeMap::from([("perspective.view".to_string(), 3usize)]),
                files_by_kind: BTreeMap::from([
                    ("resource.json".to_string(), 3usize),
                    ("view.json".to_string(), 2usize),
                ]),
                unknown_resources: 0,
                unknown_ratio: 0.0,
            },
            projects: vec![ProjectAnalytics {
                project_root: "".to_string(),
                project: ProjectMetadata {
                    project_root: "".to_string(),
                    title: "Synthetic".to_string(),
                    description: None,
                    parent: None,
                    enabled: true,
                    inheritable: false,
                },
                counts: ProjectCounts {
                    resources_total: 3,
                    files_total: 5,
                    binary_only_resources: 1,
                    resources_by_section: BTreeMap::from([("Perspective".to_string(), 3usize)]),
                    resources_by_type: BTreeMap::from([("perspective.view".to_string(), 3usize)]),
                    files_by_kind: BTreeMap::from([
                        ("resource.json".to_string(), 3usize),
                        ("view.json".to_string(), 2usize),
                    ]),
                },
                coverage: CoverageMetrics {
                    unknown_resources: 0,
                    unknown_ratio: 0.0,
                },
                issues: vec![],
            }],
            issues: vec![],
            gateway_meta: None,
        }
    }

    #[test]
    fn summarize_output_includes_core_fields() {
        let analytics = synthetic_analytics();
        let output = render_summary_text(Path::new("/tmp/synthetic.zip"), &analytics, 0);
        assert!(output.contains("archive_kind: project_export"));
        assert!(output.contains("projects_total: 1"));
        assert!(output.contains("selected_project_roots: [\"\"]"));
    }

    #[test]
    fn summarize_output_includes_project_lines_in_verbose_mode() {
        let analytics = synthetic_analytics();
        let output = render_summary_text(Path::new("/tmp/synthetic.zip"), &analytics, 1);
        assert!(output.contains("project: root=(root) title=Synthetic resources=3"));
    }

    #[test]
    fn report_markdown_contains_required_sections() {
        let analytics = synthetic_analytics();
        let report = render_report_markdown(&analytics);
        assert!(report.contains("# igniscope report"));
        assert!(report.contains("## Input Summary"));
        assert!(report.contains("## Overall Aggregate Summary"));
        assert!(report.contains("## Counts By Section"));
        assert!(report.contains("## Counts By Type Key"));
        assert!(report.contains("## File Kind Breakdown"));
        assert!(report.contains("## Coverage Summary"));
        assert!(report.contains("## Per-Project Details"));
        assert!(report.contains("## Issues"));
    }

    #[test]
    fn root_display_is_human_readable() {
        assert_eq!(display_project_root(""), "(root)");
        assert_eq!(display_project_root("projects/alpha/"), "projects/alpha/");
    }
}
