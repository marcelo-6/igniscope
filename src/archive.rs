use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use zip::ZipArchive;

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveKind {
    ProjectExport,
    GatewayBackup,
    Unknown,
}

impl ArchiveKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProjectExport => "project_export",
            Self::GatewayBackup => "gateway_backup",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectSelection {
    Single { root: String },
    Multiple { roots: Vec<String> },
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveInspection {
    pub archive_kind: ArchiveKind,
    pub project_selection: ProjectSelection,
    pub detected_project_roots: Vec<String>,
    pub selected_project_roots: Vec<String>,
}

/// Metadata extracted from a `project.json` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectMetadata {
    pub project_root: String,
    pub title: String,
    pub description: Option<String>,
    pub parent: Option<String>,
    pub enabled: bool,
    pub inheritable: bool,
}

#[derive(Debug, Deserialize)]
struct RawProjectFile {
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default)]
    inheritable: bool,
}

/// Default for missing `enabled` keys in `project.json`.
/// # TODO is there a better way for constants + serde?
const fn default_enabled() -> bool {
    true
}

/// Inspects an archive and returns its kind plus selected project.
pub fn inspect_archive(archive_path: &Path) -> Result<ArchiveInspection, AppError> {
    let entries = list_archive_entries(archive_path)?;
    inspect_entries(archive_path, &entries)
}

/// Lists archive entries in deterministic (sorted, deduplicated) order.
pub fn list_archive_entries(archive_path: &Path) -> Result<Vec<String>, AppError> {
    let file = File::open(archive_path).map_err(|err| {
        AppError::archive_read(archive_path, format!("could not open file: {err}"))
    })?;

    let mut archive = ZipArchive::new(file).map_err(|err| {
        AppError::archive_read(archive_path, format!("not a valid zip archive: {err}"))
    })?;

    let mut entries = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let zip_entry = archive.by_index(index).map_err(|err| {
            AppError::archive_read(
                archive_path,
                format!("could not read zip entry at index {index}: {err}"),
            )
        })?;

        let normalized = normalize_zip_entry_name(zip_entry.name());
        if !normalized.is_empty() {
            entries.push(normalized);
        }
    }

    entries.sort();
    entries.dedup();
    Ok(entries)
}

/// Parses `project.json` for each selected root, preserving root order.
pub fn parse_project_metadata(
    archive_path: &Path,
    selected_project_roots: &[String],
) -> Result<Vec<ProjectMetadata>, AppError> {
    let file = File::open(archive_path).map_err(|err| {
        AppError::archive_read(archive_path, format!("could not open file: {err}"))
    })?;

    let mut archive = ZipArchive::new(file).map_err(|err| {
        AppError::archive_read(archive_path, format!("not a valid zip archive: {err}"))
    })?;

    let mut projects = Vec::with_capacity(selected_project_roots.len());
    for project_root in selected_project_roots {
        let project_json_path = project_json_member_path(project_root);
        let mut member = archive.by_name(&project_json_path).map_err(|err| {
            AppError::archive_read(
                archive_path,
                format!("missing expected `{project_json_path}`: {err}"),
            )
        })?;

        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).map_err(|err| {
            AppError::archive_read(
                archive_path,
                format!("could not read `{project_json_path}`: {err}"),
            )
        })?;

        let project = parse_project_json_bytes(archive_path, &project_json_path, &bytes)?;
        projects.push(project.with_root(project_root.clone()));
    }

    Ok(projects)
}

/// Parses `project.json` into project metadata fields.
fn parse_project_json_bytes(
    archive_path: &Path,
    project_json_path: &str,
    bytes: &[u8],
) -> Result<RawProjectParsed, AppError> {
    let parsed: RawProjectFile = serde_json::from_slice(bytes).map_err(|err| {
        AppError::json_parse(
            archive_path,
            project_json_path,
            format!("invalid JSON payload: {err}"),
        )
    })?;

    Ok(RawProjectParsed {
        title: parsed.title,
        description: parsed.description,
        parent: parsed.parent,
        enabled: parsed.enabled,
        inheritable: parsed.inheritable,
    })
}

/// Builds the path leading to a project's `project.json` file.
/// TODO remove later
fn project_json_member_path(project_root: &str) -> String {
    if project_root.is_empty() {
        "project.json".to_string()
    } else {
        format!("{project_root}project.json")
    }
}

/// Derives selected project entry names.
fn inspect_entries(archive_path: &Path, entries: &[String]) -> Result<ArchiveInspection, AppError> {
    let kind = detect_archive_kind(entries);
    let gateway_roots = detect_gateway_project_roots(entries);

    let (detected_project_roots, selected_project_roots) = match kind {
        ArchiveKind::ProjectExport => (vec![String::new()], vec![String::new()]),
        ArchiveKind::GatewayBackup => (gateway_roots.clone(), gateway_roots),
        ArchiveKind::Unknown => {
            return Err(AppError::project_root_detection(
                archive_path,
                "expected `project.json` at archive root or one/more `projects/<name>/project.json` roots",
            ));
        }
    };

    let project_selection = match selected_project_roots.len() {
        0 => ProjectSelection::None,
        1 => ProjectSelection::Single {
            root: selected_project_roots[0].clone(),
        },
        _ => ProjectSelection::Multiple {
            roots: selected_project_roots.clone(),
        },
    };

    Ok(ArchiveInspection {
        archive_kind: kind,
        project_selection,
        detected_project_roots,
        selected_project_roots,
    })
}

/// Detects archive kind from normalized entry names.
pub(crate) fn detect_archive_kind(entries: &[String]) -> ArchiveKind {
    let has_root_project = entries.iter().any(|entry| entry == "project.json");
    if has_root_project {
        return ArchiveKind::ProjectExport;
    }

    let gateway_roots = detect_gateway_project_roots(entries);
    if gateway_roots.is_empty() {
        ArchiveKind::Unknown
    } else {
        ArchiveKind::GatewayBackup
    }
}

/// Extracts gateway project roots from entry names.
pub(crate) fn detect_gateway_project_roots(entries: &[String]) -> Vec<String> {
    let mut roots = BTreeSet::new();

    for entry in entries {
        if let Some(project_name) = gateway_project_name(entry) {
            roots.insert(format!("projects/{project_name}/"));
        }
    }

    roots.into_iter().collect()
}

/// Returns a gateway project name when entry matches `projects/<name>/project.json`.
fn gateway_project_name(entry: &str) -> Option<&str> {
    let rest = entry.strip_prefix("projects/")?;
    let name = rest.strip_suffix("/project.json")?;
    if name.is_empty() || name.contains('/') {
        return None;
    }
    Some(name)
}

/// Normalizes entry names to forward slashes and strips leading separators.
fn normalize_zip_entry_name(name: &str) -> String {
    name.replace('\\', "/").trim_start_matches('/').to_string()
}

#[derive(Debug)]
struct RawProjectParsed {
    title: String,
    description: Option<String>,
    parent: Option<String>,
    enabled: bool,
    inheritable: bool,
}

impl RawProjectParsed {
    /// Attaches a project root to parsed `project.json` fields.
    fn with_root(self, project_root: String) -> ProjectMetadata {
        ProjectMetadata {
            project_root,
            title: self.title,
            description: self.description,
            parent: self.parent,
            enabled: self.enabled,
            inheritable: self.inheritable,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        ArchiveKind, ProjectSelection, detect_archive_kind, detect_gateway_project_roots,
        inspect_archive, parse_project_json_bytes, parse_project_metadata,
    };
    use crate::error::AppError;

    fn fixture_path(file_name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("example-files")
            .join(file_name)
    }

    #[test]
    fn archive_kind_detects_project_export_fixture() {
        let archive = inspect_archive(&fixture_path("Template_v8.3_example.zip"))
            .expect("fixture should be inspectable");
        assert_eq!(archive.archive_kind, ArchiveKind::ProjectExport);
    }

    #[test]
    fn archive_kind_detects_gateway_backup_fixture() {
        let archive = inspect_archive(&fixture_path("multi-project.gwbk"))
            .expect("fixture should be inspectable");
        assert_eq!(archive.archive_kind, ArchiveKind::GatewayBackup);
    }

    #[test]
    fn archive_kind_detects_unknown_fixture_as_error() {
        let err = inspect_archive(&fixture_path("data_center_industry_pack.1.1.0.zip"))
            .expect_err("wrapper archive should fail root detection");

        match err {
            AppError::ProjectRootDetection { .. } => {}
            other => panic!("expected project root detection error, got: {other:?}"),
        }
    }

    #[test]
    fn project_roots_for_project_export_is_root_only() {
        let archive = inspect_archive(&fixture_path("Template_v8.3_example.zip"))
            .expect("fixture should be inspectable");
        assert_eq!(archive.detected_project_roots, vec![String::new()]);
        assert_eq!(archive.selected_project_roots, vec![String::new()]);
        assert_eq!(
            archive.project_selection,
            ProjectSelection::Single {
                root: String::new()
            }
        );
    }

    #[test]
    fn project_roots_for_multi_project_gateway_are_sorted() {
        let archive = inspect_archive(&fixture_path("multi-project.gwbk"))
            .expect("fixture should be inspectable");
        assert_eq!(
            archive.detected_project_roots,
            vec![
                "projects/IADemo/".to_string(),
                "projects/OnlineDemo/".to_string(),
                "projects/TagDashboard/".to_string(),
                "projects/building-management-system-demo/".to_string(),
                "projects/global/".to_string(),
                "projects/oil-and-gas-demo/".to_string(),
                "projects/prepared-foods-line-demo/".to_string(),
                "projects/samplequickstart/".to_string(),
            ]
        );
        assert_eq!(
            archive.detected_project_roots,
            archive.selected_project_roots
        );
        assert_eq!(
            archive.project_selection,
            ProjectSelection::Multiple {
                roots: vec![
                    "projects/IADemo/".to_string(),
                    "projects/OnlineDemo/".to_string(),
                    "projects/TagDashboard/".to_string(),
                    "projects/building-management-system-demo/".to_string(),
                    "projects/global/".to_string(),
                    "projects/oil-and-gas-demo/".to_string(),
                    "projects/prepared-foods-line-demo/".to_string(),
                    "projects/samplequickstart/".to_string(),
                ]
            }
        );
    }

    #[test]
    fn archive_kind_prefers_root_project_when_both_shapes_exist() {
        let entries = vec![
            "project.json".to_string(),
            "projects/alpha/project.json".to_string(),
        ];
        assert_eq!(detect_archive_kind(&entries), ArchiveKind::ProjectExport);
    }

    #[test]
    fn project_roots_ignore_invalid_gateway_layouts() {
        let entries = vec![
            "projects//project.json".to_string(),
            "projects/alpha/nested/project.json".to_string(),
            "Projects/uppercase/project.json".to_string(),
            "projects/valid/project.json".to_string(),
        ];

        assert_eq!(
            detect_gateway_project_roots(&entries),
            vec!["projects/valid/".to_string()]
        );
    }

    #[test]
    fn project_meta_project_export_fixture_yields_single_record() {
        let archive_path = fixture_path("Template_v8.3_example.zip");
        let inspection = inspect_archive(&archive_path).expect("fixture should be inspectable");
        let project_meta =
            parse_project_metadata(&archive_path, &inspection.selected_project_roots).unwrap();

        assert_eq!(project_meta.len(), 1);
        assert_eq!(project_meta[0].project_root, "");
        assert_eq!(project_meta[0].title, "Good template");
        assert_eq!(project_meta[0].enabled, true);
        assert_eq!(project_meta[0].inheritable, false);
    }

    #[test]
    fn project_meta_multi_project_fixture_preserves_selected_root_order() {
        let archive_path = fixture_path("multi-project.gwbk");
        let selected_roots = vec![
            "projects/TagDashboard/".to_string(),
            "projects/IADemo/".to_string(),
        ];
        let project_meta = parse_project_metadata(&archive_path, &selected_roots).unwrap();

        assert_eq!(project_meta.len(), 2);
        assert_eq!(project_meta[0].project_root, "projects/TagDashboard/");
        assert_eq!(project_meta[0].title, "IIoT Demo");
        assert_eq!(project_meta[1].project_root, "projects/IADemo/");
        assert_eq!(project_meta[1].title, "Vision Demo");
    }

    #[test]
    fn project_meta_invalid_json_returns_json_parse_error() {
        let err = parse_project_json_bytes(
            Path::new("synthetic.zip"),
            "project.json",
            br#"{"title":"bad","enabled":"not_a_bool"}"#,
        )
        .expect_err("invalid JSON shape should fail");

        match err {
            AppError::JsonParse { .. } => {}
            other => panic!("expected json parse error, got: {other:?}"),
        }
    }
}
