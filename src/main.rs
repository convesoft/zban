//! The `zban` binary: the command line and MCP adapters over the core library.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use zban::Project;

/// A file-based issue tracker for you and your coding agents.
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Cli {
    /// Run as if zban was started in DIR.
    #[arg(short = 'C', long = "directory", value_name = "DIR", global = true)]
    directory: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a project: `.zban/` and its configuration in this directory.
    Init {
        /// ID prefix: 1-8 lowercase ASCII letters or digits, starting with a letter.
        #[arg(long, value_name = "P")]
        prefix: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let directory = cli.directory.unwrap_or_else(|| PathBuf::from("."));
    let result = match cli.command {
        Command::Init { prefix } => Project::init(&directory, prefix.as_deref()).map(|project| {
            println!(
                "Created {} with ID prefix {}",
                project.dir().display(),
                project.prefix()
            );
        }),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error} [{}]", error.code);
            ExitCode::FAILURE
        }
    }
}
