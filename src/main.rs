use clap::Parser;
use igniscope::app;
use igniscope::cli::{Cli, Command};
use igniscope::error::exit_code_for_error;

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Command::Summarize { archive_path } => app::run_summarize(archive_path, cli.verbose),
        Command::Analyze {
            archive_path,
            out_dir,
        } => app::run_analyze(archive_path, out_dir, cli.verbose),
    };

    if let Err(err) = result {
        eprintln!("{err}");
        std::process::exit(exit_code_for_error(&err));
    }
}
