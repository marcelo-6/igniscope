use std::path::Path;

use crate::archive::{discover_resources_for_roots, inspect_archive, parse_project_metadata};
use crate::error::AppError;

/// Handles the `summarize` command for archive inspection and project parsing.
pub fn run_summarize(archive_path: &Path, verbose: u8) -> Result<(), AppError> {
    let inspection = inspect_archive(archive_path)?;
    let project_metadata =
        parse_project_metadata(archive_path, &inspection.selected_project_roots)?;
    let resource_inventories =
        discover_resources_for_roots(archive_path, &inspection.selected_project_roots)?;

    println!("Parsed `summarize` command.");
    println!("archive_path: {}", archive_path.display());
    println!("verbose: {verbose}");
    println!("archive_kind: {}", inspection.archive_kind.as_str());
    println!("project_selection: {:#?}", inspection.project_selection);
    println!(
        "detected_project_roots: {:?}",
        inspection.detected_project_roots
    );
    println!(
        "selected_project_roots: {:?}",
        inspection.selected_project_roots
    );
    println!("project_metadata: {:#?}", project_metadata);
    println!(
        "resource_counts_by_project: {:?}",
        resource_inventories
            .iter()
            .map(|inventory| (inventory.project_root.clone(), inventory.resources.len()))
            .collect::<Vec<_>>()
    );
    Ok(())
}

/// Handles the `analyze` command for archive inspection and project parsing.
pub fn run_analyze(archive_path: &Path, out_dir: &Path, verbose: u8) -> Result<(), AppError> {
    let inspection = inspect_archive(archive_path)?;
    let project_metadata =
        parse_project_metadata(archive_path, &inspection.selected_project_roots)?;
    let resource_inventories =
        discover_resources_for_roots(archive_path, &inspection.selected_project_roots)?;

    println!("Parsed `analyze` command.");
    println!("archive_path: {}", archive_path.display());
    println!("out_dir: {}", out_dir.display());
    println!("verbose: {verbose}");
    println!("archive_kind: {}", inspection.archive_kind.as_str());
    println!("project_selection: {:#?}", inspection.project_selection);
    println!(
        "detected_project_roots: {:?}",
        inspection.detected_project_roots
    );
    println!(
        "selected_project_roots: {:?}",
        inspection.selected_project_roots
    );
    println!("project_metadata: {:#?}", project_metadata);
    println!(
        "resource_counts_by_project: {:?}",
        resource_inventories
            .iter()
            .map(|inventory| (inventory.project_root.clone(), inventory.resources.len()))
            .collect::<Vec<_>>()
    );
    Ok(())
}
