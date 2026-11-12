//! The `zban` binary: the command line and MCP adapters over the core library.

mod cli;
mod mcp;

fn main() -> std::process::ExitCode {
    cli::run()
}
