use clap::Parser;
use ign_inspect::app;
use ign_inspect::cli::{Cli, Command};

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
        std::process::exit(app::exit_code_for_error(&err));
    }
}
