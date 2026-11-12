//! The `zban` binary: the command line and MCP adapters over the core library.

use clap::Parser;

/// A file-based issue tracker for you and your coding agents.
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Cli {}

fn main() {
    Cli::parse();
}
