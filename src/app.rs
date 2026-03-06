use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

#[derive(Debug)]
pub struct AppError {
    message: String,
    exit_code: i32,
}

impl AppError {
    pub fn new(message: impl Into<String>, exit_code: i32) -> Self {
        Self {
            message: message.into(),
            exit_code,
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for AppError {}

pub fn run_summarize(archive_path: &Path, verbose: u8) -> Result<(), AppError> {
    println!("Parsed `summarize` command.");
    println!("archive_path: {}", archive_path.display());
    println!("verbose: {verbose}");
    println!("Note: CLI parsing and dispatch only for now.");
    Ok(())
}

pub fn run_analyze(archive_path: &Path, out_dir: &Path, verbose: u8) -> Result<(), AppError> {
    println!("Parsed `analyze` command.");
    println!("archive_path: {}", archive_path.display());
    println!("out_dir: {}", out_dir.display());
    println!("verbose: {verbose}");
    println!("Note: CLI parsing and dispatch only for now.");
    Ok(())
}

pub fn exit_code_for_error(err: &AppError) -> i32 {
    err.exit_code
}
