//! What every command prints (`REQ-JSON-OUTPUT`): with `--json` exactly one
//! JSON document on standard output, otherwise text for humans with warnings
//! and errors on standard error. Failures print the error object of
//! `DES-CLI`, and each error code maps to an exit status here.

use std::ffi::OsString;
use std::io::{self, Write as _};
use std::process::ExitCode;

use clap::error::ErrorKind;
use serde::Serialize;
use zban::operations::Warning;
use zban::{Error, ErrorCode, Result};

/// What a command that ran prints.
pub struct Reply {
    /// The JSON document printed with `--json`.
    json: String,
    /// The text printed without `--json`.
    text: String,
    /// Warnings printed to standard error without `--json`. With `--json`
    /// they are part of the document.
    warnings: Vec<Warning>,
    /// Whether the command exits with status 1 although it ran, as `check`
    /// does when it finds errors.
    failed: bool,
}

impl Reply {
    /// A reply printing `document` as JSON, or `text` for humans.
    pub fn new(document: &impl Serialize, text: String) -> Self {
        Self {
            json: serde_json::to_string(document).expect("documents serialize to JSON"),
            text,
            warnings: Vec::new(),
            failed: false,
        }
    }

    /// Prints `warnings` to standard error in human output.
    pub fn warnings(mut self, warnings: &[Warning]) -> Self {
        self.warnings = warnings.to_vec();
        self
    }

    /// Makes the command exit with status 1 after printing its output.
    #[allow(dead_code, reason = "`check` exits with status 1 when it finds errors")]
    pub fn failed(mut self, failed: bool) -> Self {
        self.failed = failed;
        self
    }
}

/// The exit status of a failure: 2 for invalid usage, 1 otherwise.
fn exit_status(code: ErrorCode) -> u8 {
    match code {
        ErrorCode::Usage => 2,
        _ => 1,
    }
}

/// Prints the outcome of a command and returns its exit status.
pub fn finish(json: bool, outcome: Result<Reply>) -> ExitCode {
    let reply = match outcome {
        Ok(reply) => reply,
        Err(error) => return fail(json, &error),
    };
    if json {
        print_json(&reply.json);
    } else {
        let mut stderr = io::stderr().lock();
        for warning in &reply.warnings {
            let _ = writeln!(
                stderr,
                "warning: skipped {}: {} [{}]",
                warning.path, warning.message, warning.code
            );
        }
        print_text(&reply.text);
    }
    if reply.failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Prints `error` and returns its exit status.
pub fn fail(json: bool, error: &Error) -> ExitCode {
    if json {
        #[derive(Serialize)]
        struct Document<'a> {
            error: &'a Error,
        }
        print_json(&serde_json::to_string(&Document { error }).expect("errors serialize"));
    } else {
        let _ = writeln!(io::stderr().lock(), "error: {error} [{}]", error.code);
    }
    ExitCode::from(exit_status(error.code))
}

/// Handles arguments that clap rejected. Help and version requests print as
/// usual. With `--json`, other errors print the `usage` error object.
pub fn rejected(error: &clap::Error) -> ExitCode {
    let informational = matches!(
        error.kind(),
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
    );
    if !informational && json_requested(std::env::args_os()) {
        // The first paragraph, without the usage summary that follows it.
        let rendered = error.render().to_string();
        let paragraph: Vec<_> = rendered
            .lines()
            .map(str::trim)
            .take_while(|line| !line.is_empty())
            .collect();
        let message = paragraph.join(" ");
        let message = message.strip_prefix("error: ").unwrap_or(&message);
        return fail(true, &Error::usage(message));
    }
    let _ = error.print();
    ExitCode::from(u8::try_from(error.exit_code()).unwrap_or(2))
}

/// Whether `--json` appears among the options of arguments that failed to
/// parse.
fn json_requested(args: impl Iterator<Item = OsString>) -> bool {
    args.skip(1)
        .take_while(|arg| arg != "--")
        .any(|arg| arg == "--json")
}

/// Prints to standard output, ignoring a closed pipe such as `| head`.
fn print_json(document: &str) {
    let _ = writeln!(io::stdout().lock(), "{document}");
}

fn print_text(text: &str) {
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(text.as_bytes());
    if !text.is_empty() && !text.ends_with('\n') {
        let _ = stdout.write_all(b"\n");
    }
}
