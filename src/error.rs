use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    ArchiveRead {
        archive_path: PathBuf,
        details: String,
    },
    ProjectRootDetection {
        archive_path: PathBuf,
        details: String,
    },
    JsonParse {
        archive_path: PathBuf,
        json_path: String,
        details: String,
    },
    ManifestIntegrity {
        details: String,
    },
    Internal {
        details: String,
    },
}

impl AppError {
    /// Creates an archive read error with context.
    pub fn archive_read(archive_path: &Path, details: impl Into<String>) -> Self {
        Self::ArchiveRead {
            archive_path: archive_path.to_path_buf(),
            details: details.into(),
        }
    }

    /// Creates a project detection error for an archive path.
    pub fn project_root_detection(archive_path: &Path, details: impl Into<String>) -> Self {
        Self::ProjectRootDetection {
            archive_path: archive_path.to_path_buf(),
            details: details.into(),
        }
    }

    /// Creates a JSON-parse error with added context (such as archive and path).
    pub fn json_parse(
        archive_path: &Path,
        json_path: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self::JsonParse {
            archive_path: archive_path.to_path_buf(),
            json_path: json_path.into(),
            details: details.into(),
        }
    }

    /// Creates a generic internal error for unexpected states.
    pub fn internal(details: impl Into<String>) -> Self {
        Self::Internal {
            details: details.into(),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArchiveRead {
                archive_path,
                details,
            } => {
                write!(
                    f,
                    "Failed to read archive `{}`: {}",
                    archive_path.display(),
                    details
                )
            }
            Self::ProjectRootDetection {
                archive_path,
                details,
            } => {
                write!(
                    f,
                    "Could not detect project roots in `{}`: {}",
                    archive_path.display(),
                    details
                )
            }
            Self::JsonParse {
                archive_path,
                json_path,
                details,
            } => write!(
                f,
                "Failed to parse JSON `{}` in `{}`: {}",
                json_path,
                archive_path.display(),
                details
            ),
            Self::ManifestIntegrity { details } => write!(f, "Manifest integrity error: {details}"),
            Self::Internal { details } => write!(f, "Internal error: {details}"),
        }
    }
}

impl Error for AppError {}

/// Maps typed errors to exit codes.
pub fn exit_code_for_error(err: &AppError) -> i32 {
    match err {
        AppError::ArchiveRead { .. } => 10,
        AppError::ProjectRootDetection { .. } => 11,
        AppError::JsonParse { .. } => 12,
        AppError::ManifestIntegrity { .. } => 13,
        AppError::Internal { .. } => 20,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{AppError, exit_code_for_error};

    #[test]
    fn archive_read_maps_to_exit_code_10() {
        let err = AppError::archive_read(Path::new("sample.zip"), "bad zip");
        assert_eq!(exit_code_for_error(&err), 10);
    }

    #[test]
    fn project_root_detection_maps_to_exit_code_11() {
        let err = AppError::project_root_detection(Path::new("sample.zip"), "no roots");
        assert_eq!(exit_code_for_error(&err), 11);
    }

    #[test]
    fn json_parse_maps_to_exit_code_12() {
        let err = AppError::json_parse(Path::new("sample.zip"), "project.json", "invalid");
        assert_eq!(exit_code_for_error(&err), 12);
    }
}
