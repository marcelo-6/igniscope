use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "ign-inspect",
    version,
    about = "Deterministic analyzer for Ignition project exports and gateway backups",
    long_about = None,
    arg_required_else_help = true,
    after_help = "Examples:\n  ign-inspect summarize ./backup.gwbk\n  ign-inspect analyze ./project.zip --out-dir ./out\n  ign-inspect -vv summarize ./project.zip"
)]
pub struct Cli {
    /// Increase output verbosity (-v, -vv, -vvv).
    #[arg(short, long, action = ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print a short summary for an Ignition archive.
    Summarize {
        /// Path to an Ignition project export (.zip) or gateway backup (.gwbk).
        #[arg(value_name = "ARCHIVE_PATH")]
        archive_path: PathBuf,
    },
    /// Build analysis artifacts in an output directory.
    Analyze {
        /// Path to an Ignition project export (.zip) or gateway backup (.gwbk).
        #[arg(value_name = "ARCHIVE_PATH")]
        archive_path: PathBuf,
        /// Directory where generated files will be written.
        #[arg(long, value_name = "OUT_DIR")]
        out_dir: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;
    use clap::error::ErrorKind;

    use super::{Cli, Command};

    fn parse_ok(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("CLI parse should succeed")
    }

    #[test]
    fn summarize_parses_with_required_archive_path() {
        let cli = parse_ok(&["ign-inspect", "summarize", "sample.zip"]);
        assert_eq!(cli.verbose, 0);

        match cli.command {
            Command::Summarize { archive_path } => {
                assert_eq!(archive_path, PathBuf::from("sample.zip"));
            }
            other => panic!("expected summarize command, got: {other:?}"),
        }
    }

    #[test]
    fn analyze_parses_with_required_out_dir() {
        let cli = parse_ok(&[
            "ign-inspect",
            "analyze",
            "sample.gwbk",
            "--out-dir",
            "./out",
        ]);
        assert_eq!(cli.verbose, 0);

        match cli.command {
            Command::Analyze {
                archive_path,
                out_dir,
            } => {
                assert_eq!(archive_path, PathBuf::from("sample.gwbk"));
                assert_eq!(out_dir, PathBuf::from("./out"));
            }
            other => panic!("expected analyze command, got: {other:?}"),
        }
    }

    #[test]
    fn analyze_missing_out_dir_returns_usage_error() {
        let err = Cli::try_parse_from(["ign-inspect", "analyze", "sample.zip"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn unknown_subcommand_returns_usage_error() {
        let err = Cli::try_parse_from(["ign-inspect", "inspect", "sample.zip"]).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn verbose_count_supports_multiple_v_flags() {
        let v1 = parse_ok(&["ign-inspect", "-v", "summarize", "sample.zip"]);
        assert_eq!(v1.verbose, 1);

        let v2 = parse_ok(&["ign-inspect", "-vv", "summarize", "sample.zip"]);
        assert_eq!(v2.verbose, 2);

        let v3 = parse_ok(&["ign-inspect", "-vvv", "summarize", "sample.zip"]);
        assert_eq!(v3.verbose, 3);
    }
}
