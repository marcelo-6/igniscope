use std::path::Path;

use crate::archive::inspect_archive;
use crate::error::AppError;

pub fn run_summarize(archive_path: &Path, verbose: u8) -> Result<(), AppError> {
    let inspection = inspect_archive(archive_path)?;

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
    Ok(())
}

pub fn run_analyze(archive_path: &Path, out_dir: &Path, verbose: u8) -> Result<(), AppError> {
    let inspection = inspect_archive(archive_path)?;

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
    Ok(())
}
