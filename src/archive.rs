use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;
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

/// A discovered project resource and its normalized metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Resource {
    pub section: String,
    pub type_key: String,
    pub path: String,
    pub resource_json_path: String,
    pub binary_only: bool,
    pub attributes: BTreeMap<String, Value>,
    pub files: Vec<ResourceFile>,
}

/// A single file that belongs to a discovered resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceFile {
    pub file_kind: String,
    pub file_zip_path: String,
}

/// Resource inventory for a single selected project root.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectResourceInventory {
    pub project_root: String,
    pub resources: Vec<Resource>,
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

/// Discovers and validates resources for all selected project roots.
///
/// This validates:
/// - a resource exists only when `<folder>/resource.json` exists
/// - `resource.json.files` must be a string array
/// - every declared file must exist in the archive
/// - undeclared `data.bin` is appended once at the end when present
pub fn discover_resources_for_roots(
    archive_path: &Path,
    selected_project_roots: &[String],
) -> Result<Vec<ProjectResourceInventory>, AppError> {
    let entries = list_archive_entries(archive_path)?;
    let entry_set: BTreeSet<String> = entries.iter().cloned().collect();

    let file = File::open(archive_path).map_err(|err| {
        AppError::archive_read(archive_path, format!("could not open file: {err}"))
    })?;

    let mut archive = ZipArchive::new(file).map_err(|err| {
        AppError::archive_read(archive_path, format!("not a valid zip archive: {err}"))
    })?;

    let mut inventories = Vec::with_capacity(selected_project_roots.len());
    for project_root in selected_project_roots {
        let resources = discover_resources_for_root_in_archive(
            &mut archive,
            archive_path,
            project_root,
            &entries,
            &entry_set,
        )?;

        inventories.push(ProjectResourceInventory {
            project_root: project_root.clone(),
            resources,
        });
    }

    Ok(inventories)
}

/// Discovers and validates resources for a single project root.
pub fn discover_resources_for_root(
    archive_path: &Path,
    project_root: &str,
) -> Result<Vec<Resource>, AppError> {
    let inventories = discover_resources_for_roots(archive_path, &[project_root.to_string()])?;
    Ok(inventories
        .into_iter()
        .next()
        .map(|inventory| inventory.resources)
        .unwrap_or_default())
}

/// Discovers and validates root-scoped resources from an opened archive.
fn discover_resources_for_root_in_archive(
    archive: &mut ZipArchive<File>,
    archive_path: &Path,
    project_root: &str,
    entries: &[String],
    entry_set: &BTreeSet<String>,
) -> Result<Vec<Resource>, AppError> {
    let mut resource_json_paths = Vec::new();
    for entry in entries {
        if is_resource_json_for_root(entry, project_root) {
            resource_json_paths.push(entry.clone());
        }
    }

    let mut resources = Vec::with_capacity(resource_json_paths.len());
    for resource_json_path in resource_json_paths {
        let mut member = archive.by_name(&resource_json_path).map_err(|err| {
            AppError::archive_read(
                archive_path,
                format!("missing expected `{resource_json_path}`: {err}"),
            )
        })?;

        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).map_err(|err| {
            AppError::archive_read(
                archive_path,
                format!("could not read `{resource_json_path}`: {err}"),
            )
        })?;

        let resource = parse_resource(archive_path, &resource_json_path, &bytes)?;
        let files = build_resource_files(
            archive_path,
            &resource_json_path,
            &resource.files,
            entry_set,
        )?;
        let resource_path = resource_path_for_project_root(&resource_json_path, project_root);

        resources.push(Resource {
            section: "Other".to_string(),
            type_key: "unknown".to_string(),
            path: resource_path,
            resource_json_path,
            binary_only: is_binary_only_resource(&files),
            attributes: resource.attributes,
            files,
        });
    }

    resources.sort_by(|left, right| {
        (&left.section, &left.path, &left.resource_json_path).cmp(&(
            &right.section,
            &right.path,
            &right.resource_json_path,
        ))
    });

    Ok(resources)
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

/// Parses `resource.json` and validates shape.
fn parse_resource(
    archive_path: &Path,
    resource_json_path: &str,
    bytes: &[u8],
) -> Result<RawResource, AppError> {
    let value: Value = serde_json::from_slice(bytes).map_err(|err| {
        AppError::json_parse(
            archive_path,
            resource_json_path,
            format!("invalid JSON payload: {err}"),
        )
    })?;

    let object = value.as_object().ok_or_else(|| {
        AppError::json_parse(
            archive_path,
            resource_json_path,
            "expected JSON object at resource root",
        )
    })?;

    let files_value = object.get("files").ok_or_else(|| {
        AppError::resource_integrity(format!(
            "Missing required `files` key in `{resource_json_path}` of `{}`",
            archive_path.display()
        ))
    })?;

    let files_array = files_value.as_array().ok_or_else(|| {
        AppError::resource_integrity(format!(
            "Expected `files` array in `{resource_json_path}` of `{}`",
            archive_path.display()
        ))
    })?;

    let mut files = Vec::with_capacity(files_array.len());
    for (index, entry) in files_array.iter().enumerate() {
        let file_name = entry.as_str().ok_or_else(|| {
            AppError::resource_integrity(format!(
                "Expected string at `files[{index}]` in `{resource_json_path}` of `{}`",
                archive_path.display()
            ))
        })?;

        if file_name.is_empty() {
            return Err(AppError::resource_integrity(format!(
                "Found empty file name at `files[{index}]` in `{resource_json_path}` of `{}`",
                archive_path.display()
            )));
        }

        files.push(file_name.to_string());
    }

    let mut attributes = BTreeMap::new();
    for (key, value) in object {
        if key != "files" {
            attributes.insert(key.clone(), value.clone());
        }
    }

    Ok(RawResource { files, attributes })
}

/// Builds a deterministically ordered file list for a resource.
fn build_resource_files(
    archive_path: &Path,
    resource_json_path: &str,
    resource_files: &[String],
    entry_set: &BTreeSet<String>,
) -> Result<Vec<ResourceFile>, AppError> {
    let resource_folder = resource_folder_path(resource_json_path).ok_or_else(|| {
        AppError::internal(format!(
            "invalid resource json path without suffix: {resource_json_path}"
        ))
    })?;

    let mut files = Vec::with_capacity(resource_files.len() + 2);
    files.push(ResourceFile {
        file_kind: "resource.json".to_string(),
        file_zip_path: resource_json_path.to_string(),
    });

    let mut has_data_bin_declared = false;
    for declared in resource_files {
        let file_zip_path = format!("{resource_folder}{declared}");
        if !entry_set.contains(&file_zip_path) {
            return Err(AppError::resource_integrity(format!(
                "Declared file `{declared}` is missing for `{resource_json_path}` in `{}`",
                archive_path.display()
            )));
        }

        if declared == "data.bin" {
            has_data_bin_declared = true;
        }

        files.push(ResourceFile {
            file_kind: file_kind_from_declared_name(declared),
            file_zip_path,
        });
    }

    let data_bin_path = format!("{resource_folder}data.bin");
    if !has_data_bin_declared && entry_set.contains(&data_bin_path) {
        files.push(ResourceFile {
            file_kind: "data.bin".to_string(),
            file_zip_path: data_bin_path,
        });
    }

    Ok(files)
}

/// Determines whether an entry path is a `resource.json` under a project root.
fn is_resource_json_for_root(entry: &str, project_root: &str) -> bool {
    if !entry.ends_with("/resource.json") {
        return false;
    }

    if project_root.is_empty() {
        return true;
    }

    entry.starts_with(project_root)
}

/// Returns the normalized resource path relative to a project root.
fn resource_path_for_project_root(resource_json_path: &str, project_root: &str) -> String {
    let folder = resource_folder_path(resource_json_path)
        .unwrap_or(resource_json_path)
        .trim_end_matches('/')
        .to_string();

    if project_root.is_empty() {
        return folder;
    }

    folder
        .strip_prefix(project_root)
        .unwrap_or(&folder)
        .to_string()
}

/// Returns the folder prefix for a `.../resource.json` path.
fn resource_folder_path(resource_json_path: &str) -> Option<&str> {
    resource_json_path.strip_suffix("resource.json")
}

/// Maps a declared resource filename to a normalized file-kind label.
fn file_kind_from_declared_name(declared_file_name: &str) -> String {
    let file_name = declared_file_name
        .rsplit('/')
        .next()
        .unwrap_or(declared_file_name);

    if file_name.ends_with(".py") {
        "script".to_string()
    } else {
        file_name.to_string()
    }
}

/// Returns `true` when all payload files are binary-only (`data.bin`) entries.
fn is_binary_only_resource(files: &[ResourceFile]) -> bool {
    let mut payload_count = 0usize;
    for file in files {
        if file.file_kind == "resource.json" {
            continue;
        }

        payload_count += 1;
        if file.file_kind != "data.bin" {
            return false;
        }
    }

    payload_count > 0
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

#[derive(Debug)]
struct RawResource {
    files: Vec<String>,
    attributes: BTreeMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    use super::{
        ArchiveKind, ProjectSelection, build_resource_files, detect_archive_kind,
        detect_gateway_project_roots, discover_resources_for_root, discover_resources_for_roots,
        inspect_archive, is_binary_only_resource, parse_project_json_bytes, parse_project_metadata,
        parse_resource,
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

    #[test]
    fn resource_discovery_project_export_counts_expected_resources() {
        let archive_path = fixture_path("Template_v8.3_example.zip");
        let resources = discover_resources_for_root(&archive_path, "").unwrap();

        assert_eq!(resources.len(), 88);
        assert!(
            resources
                .iter()
                .all(|resource| resource.files.first().unwrap().file_kind == "resource.json")
        );
    }

    #[test]
    fn resource_discovery_gateway_backup_counts_expected_resources_per_project() {
        let archive_path = fixture_path("multi-project.gwbk");
        let selected_roots = vec![
            "projects/IADemo/".to_string(),
            "projects/OnlineDemo/".to_string(),
            "projects/TagDashboard/".to_string(),
            "projects/building-management-system-demo/".to_string(),
            "projects/global/".to_string(),
            "projects/oil-and-gas-demo/".to_string(),
            "projects/prepared-foods-line-demo/".to_string(),
            "projects/samplequickstart/".to_string(),
        ];

        let inventory = discover_resources_for_roots(&archive_path, &selected_roots).unwrap();
        let counts: Vec<(String, usize)> = inventory
            .into_iter()
            .map(|entry| (entry.project_root, entry.resources.len()))
            .collect();

        assert_eq!(
            counts,
            vec![
                ("projects/IADemo/".to_string(), 135),
                ("projects/OnlineDemo/".to_string(), 688),
                ("projects/TagDashboard/".to_string(), 76),
                ("projects/building-management-system-demo/".to_string(), 216),
                ("projects/global/".to_string(), 10),
                ("projects/oil-and-gas-demo/".to_string(), 24),
                ("projects/prepared-foods-line-demo/".to_string(), 132),
                ("projects/samplequickstart/".to_string(), 243),
            ]
        );
    }

    #[test]
    fn resource_discovery_is_deterministic_between_runs() {
        let archive_path = fixture_path("Template_v8.3_example.zip");
        let first = discover_resources_for_root(&archive_path, "").unwrap();
        let second = discover_resources_for_root(&archive_path, "").unwrap();

        assert_eq!(first, second);
    }

    #[test]
    fn resource_validation_rejects_missing_files_key() {
        let err = parse_resource(
            Path::new("synthetic.zip"),
            "foo/resource.json",
            br#"{"scope":"A"}"#,
        )
        .expect_err("missing files key should fail");

        match err {
            AppError::ResourceIntegrity { .. } => {}
            other => panic!("expected manifest integrity error, got: {other:?}"),
        }
    }

    #[test]
    fn resource_validation_rejects_invalid_files_type() {
        let err = parse_resource(
            Path::new("synthetic.zip"),
            "foo/resource.json",
            br#"{"files":"not-an-array"}"#,
        )
        .expect_err("invalid files type should fail");

        match err {
            AppError::ResourceIntegrity { .. } => {}
            other => panic!("expected manifest integrity error, got: {other:?}"),
        }
    }

    #[test]
    fn resource_validation_rejects_missing_declared_file() {
        let entry_set: BTreeSet<String> = ["foo/resource.json".to_string()].into_iter().collect();
        let err = build_resource_files(
            Path::new("synthetic.zip"),
            "foo/resource.json",
            &["missing.py".to_string()],
            &entry_set,
        )
        .expect_err("missing declared file should fail");

        match err {
            AppError::ResourceIntegrity { .. } => {}
            other => panic!("expected manifest integrity error, got: {other:?}"),
        }
    }

    #[test]
    fn resource_validation_appends_data_bin_when_undeclared() {
        let entry_set: BTreeSet<String> = [
            "foo/resource.json".to_string(),
            "foo/view.json".to_string(),
            "foo/data.bin".to_string(),
        ]
        .into_iter()
        .collect();

        let files = build_resource_files(
            Path::new("synthetic.zip"),
            "foo/resource.json",
            &["view.json".to_string()],
            &entry_set,
        )
        .expect("build resource files should succeed");

        let ordered_paths: Vec<&str> = files
            .iter()
            .map(|file| file.file_zip_path.as_str())
            .collect();
        assert_eq!(
            ordered_paths,
            vec!["foo/resource.json", "foo/view.json", "foo/data.bin"]
        );
    }

    #[test]
    fn resource_validation_does_not_duplicate_declared_data_bin() {
        let entry_set: BTreeSet<String> =
            ["foo/resource.json".to_string(), "foo/data.bin".to_string()]
                .into_iter()
                .collect();

        let files = build_resource_files(
            Path::new("synthetic.zip"),
            "foo/resource.json",
            &["data.bin".to_string()],
            &entry_set,
        )
        .expect("build resource files should succeed");

        assert_eq!(files.len(), 2);
        assert_eq!(files[1].file_zip_path, "foo/data.bin");
    }

    #[test]
    fn resource_binary_only_detection_requires_only_data_bin_payload() {
        let entry_set: BTreeSet<String> =
            ["foo/resource.json".to_string(), "foo/data.bin".to_string()]
                .into_iter()
                .collect();
        let files = build_resource_files(
            Path::new("synthetic.zip"),
            "foo/resource.json",
            &["data.bin".to_string()],
            &entry_set,
        )
        .unwrap();
        assert!(is_binary_only_resource(&files));

        let entry_set_with_text: BTreeSet<String> = [
            "bar/resource.json".to_string(),
            "bar/data.bin".to_string(),
            "bar/view.json".to_string(),
        ]
        .into_iter()
        .collect();
        let files_with_text = build_resource_files(
            Path::new("synthetic.zip"),
            "bar/resource.json",
            &["view.json".to_string()],
            &entry_set_with_text,
        )
        .unwrap();
        assert!(!is_binary_only_resource(&files_with_text));
    }
}
