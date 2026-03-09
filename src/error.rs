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
        details: String,
    },
    // TODO better naming
    ManifestIntegrity {
        details: String,
    },
    Internal {
        details: String,
    },
}

impl AppError {
    pub fn archive_read(archive_path: &Path, details: impl Into<String>) -> Self {
        Self::ArchiveRead {
            archive_path: archive_path.to_path_buf(),
            details: details.into(),
        }
    }

    pub fn project_root_detection(archive_path: &Path, details: impl Into<String>) -> Self {
        Self::ProjectRootDetection {
            archive_path: archive_path.to_path_buf(),
            details: details.into(),
        }
    }

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
            Self::JsonParse { details } => write!(f, "JSON parse error: {details}"),
            Self::ManifestIntegrity { details } => write!(f, "Manifest integrity error: {details}"),
            Self::Internal { details } => write!(f, "Internal error: {details}"),
        }
    }
}

impl Error for AppError {}

// TODO define better error codes
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
}
