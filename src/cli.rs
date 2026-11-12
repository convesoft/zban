//! The command line adapter (`DES-CLI`): it turns arguments into operation
//! requests and operation results into output. What gets printed, and with
//! which exit status, is decided in [`output`].

mod human;
mod output;

use std::io::{self, Read as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use zban::operations::{self, CreateRequest, ListRequest, Replacement, UpdateRequest};
use zban::{Error, ErrorCode, Project, Result};

use output::Reply;

/// A file-based issue tracker for you and your coding agents.
// @mara implements DES-CLI
#[derive(Parser)]
#[command(version, arg_required_else_help = true)]
struct Cli {
    /// Print one JSON document instead of text for humans.
    #[arg(long, global = true)]
    json: bool,

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
    /// Create an issue.
    Create(CreateArgs),
    /// List open issues in priority order, or the issues matching the filters.
    List(ListArgs),
    /// Show issues in full, with their body and the issues that refer to them.
    Show {
        #[arg(required = true, value_name = "ID")]
        ids: Vec<String>,
    },
    /// Change an issue's fields, relations or body.
    Update(UpdateArgs),
    /// Delete an issue that no other issue refers to.
    Delete {
        #[arg(value_name = "ID")]
        id: String,
    },
    /// Report problems in `.zban/`; exits with status 1 when any is an error.
    Check {
        /// Move misplaced files into the directory their status requires.
        #[arg(long)]
        fix: bool,
    },
    /// Print the workflow guide for coding agents; outside a project, print
    /// nothing.
    Prime,
}

#[derive(Args)]
struct CreateArgs {
    /// A single line.
    title: String,
    /// epic, feature, bug or task [default: task]
    #[arg(short = 't', long = "type", value_name = "TYPE")]
    issue_type: Option<String>,
    /// draft, todo, in-progress, done or cancelled [default: todo]
    #[arg(short, long)]
    status: Option<String>,
    /// critical, high, normal, low or deferred [default: normal]
    #[arg(short, long)]
    priority: Option<String>,
    #[arg(long = "tag", value_name = "T")]
    tags: Vec<String>,
    #[arg(long, value_name = "ID")]
    parent: Option<String>,
    #[arg(long, value_name = "ID")]
    blocked_by: Vec<String>,
    #[arg(long, value_name = "ID")]
    related: Vec<String>,
    #[arg(long, value_name = "ID")]
    discovered_from: Option<String>,
    /// The body, or `-` to read it from standard input.
    #[arg(long, value_name = "TEXT|-", allow_hyphen_values = true)]
    body: Option<String>,
}

/// Repeated values of one filter match any of them; different filters must
/// all match.
#[derive(Args)]
struct ListArgs {
    #[arg(short, long = "status", value_name = "S")]
    statuses: Vec<String>,
    #[arg(short = 't', long = "type", value_name = "T")]
    types: Vec<String>,
    #[arg(short, long = "priority", value_name = "P")]
    priorities: Vec<String>,
    #[arg(long = "tag", value_name = "T")]
    tags: Vec<String>,
    /// Direct children of this issue.
    #[arg(long, value_name = "ID")]
    parent: Option<String>,
    /// Issues that can be started now.
    #[arg(long)]
    ready: bool,
    /// Text to find in the title or body, ignoring case.
    #[arg(long, value_name = "TEXT", allow_hyphen_values = true)]
    search: Option<String>,
    /// Include closed issues.
    #[arg(long)]
    all: bool,
}

/// Body edits apply in order: `--body`, each replacement, then `--append`.
#[derive(Args)]
struct UpdateArgs {
    #[arg(value_name = "ID")]
    id: String,
    #[arg(short, long)]
    status: Option<String>,
    #[arg(short = 't', long = "type", value_name = "TYPE")]
    issue_type: Option<String>,
    #[arg(short, long)]
    priority: Option<String>,
    #[arg(long, allow_hyphen_values = true)]
    title: Option<String>,
    #[arg(long = "add-tag", value_name = "T")]
    add_tags: Vec<String>,
    #[arg(long = "remove-tag", value_name = "T")]
    remove_tags: Vec<String>,
    #[arg(long, value_name = "ID", conflicts_with = "no_parent")]
    parent: Option<String>,
    #[arg(long)]
    no_parent: bool,
    #[arg(long, value_name = "ID")]
    add_blocked_by: Vec<String>,
    #[arg(long, value_name = "ID")]
    remove_blocked_by: Vec<String>,
    #[arg(long, value_name = "ID")]
    add_related: Vec<String>,
    #[arg(long, value_name = "ID")]
    remove_related: Vec<String>,
    #[arg(long, value_name = "ID", conflicts_with = "no_discovered_from")]
    discovered_from: Option<String>,
    #[arg(long)]
    no_discovered_from: bool,
    /// Replace the body; `-` reads it from standard input.
    #[arg(long, value_name = "TEXT|-", allow_hyphen_values = true)]
    body: Option<String>,
    /// Append to the body after a blank line; `-` reads standard input.
    #[arg(long, value_name = "TEXT|-", allow_hyphen_values = true)]
    append: Option<String>,
    /// Text that occurs exactly once in the body; repeatable, paired in order
    /// with `--replace-new`.
    #[arg(
        long,
        value_name = "TEXT",
        allow_hyphen_values = true,
        requires = "replace_new"
    )]
    replace_old: Vec<String>,
    /// The replacement for the `--replace-old` at the same position.
    #[arg(
        long,
        value_name = "TEXT",
        allow_hyphen_values = true,
        requires = "replace_old"
    )]
    replace_new: Vec<String>,
    /// Close even with unchecked task items or open children.
    #[arg(long)]
    force: bool,
}

/// Runs the command line and returns the exit status.
pub fn run() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => return output::rejected(&error),
    };
    let directory = cli.directory.as_deref().unwrap_or(Path::new("."));
    output::finish(cli.json, execute(cli.command, directory))
}

fn execute(command: Command, directory: &Path) -> Result<Reply> {
    let project = || Project::discover(directory);
    match command {
        Command::Init { prefix } => {
            let initialized = operations::init(directory, prefix.as_deref())?;
            Ok(Reply::new(&initialized, human::initialized(&initialized)))
        }
        Command::Create(args) => {
            let request = args.request(&mut Stdin::default())?;
            let issue = operations::create(&project()?, &request)?;
            Ok(Reply::new(&issue, human::changed("Created", &issue)))
        }
        Command::List(args) => {
            let listing = operations::list(&project()?, &args.request())?;
            Ok(Reply::new(&listing, human::list(&listing.issues)).warnings(&listing.warnings))
        }
        Command::Show { ids } => {
            let shown = operations::show(&project()?, &ids)?;
            Ok(Reply::new(&shown, human::show(&shown.issues)).warnings(&shown.warnings))
        }
        Command::Update(args) => {
            let request = args.request(&mut Stdin::default())?;
            let issue = operations::update(&project()?, &request)?;
            Ok(Reply::new(&issue, human::changed("Updated", &issue)))
        }
        Command::Delete { id } => {
            let issue = operations::delete(&project()?, &id)?;
            Ok(Reply::new(&issue, human::changed("Deleted", &issue)))
        }
        Command::Check { fix } => {
            let checked = operations::check(directory, fix)?;
            let text = human::findings(&checked.findings);
            Ok(Reply::new(&checked, text).failed(checked.has_errors()))
        }
        Command::Prime => {
            let guide = operations::guide(directory)?;
            let text = guide.guide.clone().unwrap_or_default();
            Ok(Reply::new(&guide, text))
        }
    }
}

impl CreateArgs {
    fn request(self, stdin: &mut Stdin) -> Result<CreateRequest> {
        Ok(CreateRequest {
            body: stdin.resolve("--body", self.body)?,
            title: self.title,
            issue_type: self.issue_type,
            status: self.status,
            priority: self.priority,
            tags: self.tags,
            parent: self.parent,
            blocked_by: self.blocked_by,
            related: self.related,
            discovered_from: self.discovered_from,
        })
    }
}

impl ListArgs {
    fn request(self) -> ListRequest {
        ListRequest {
            statuses: self.statuses,
            types: self.types,
            priorities: self.priorities,
            tags: self.tags,
            parent: self.parent,
            ready: self.ready,
            search: self.search,
            all: self.all,
        }
    }
}

impl UpdateArgs {
    fn request(self, stdin: &mut Stdin) -> Result<UpdateRequest> {
        if self.replace_old.len() != self.replace_new.len() {
            return Err(Error::usage(
                "give each --replace-old exactly one --replace-new, in the same order",
            ));
        }
        let replacements = self
            .replace_old
            .into_iter()
            .zip(self.replace_new)
            .map(|(old, new)| Replacement { old, new })
            .collect();
        Ok(UpdateRequest {
            body: stdin.resolve("--body", self.body)?,
            append: stdin.resolve("--append", self.append)?,
            id: self.id,
            title: self.title,
            status: self.status,
            issue_type: self.issue_type,
            priority: self.priority,
            add_tags: self.add_tags,
            remove_tags: self.remove_tags,
            parent: removable(self.parent, self.no_parent),
            add_blocked_by: self.add_blocked_by,
            remove_blocked_by: self.remove_blocked_by,
            add_related: self.add_related,
            remove_related: self.remove_related,
            discovered_from: removable(self.discovered_from, self.no_discovered_from),
            replacements,
            force: self.force,
        })
    }
}

/// An optional value that an update sets with `--<key>` or removes with
/// `--no-<key>`.
fn removable(value: Option<String>, remove: bool) -> Option<Option<String>> {
    if remove { Some(None) } else { value.map(Some) }
}

/// Standard input, which at most one option of a request may read.
#[derive(Default)]
struct Stdin {
    read_by: Option<&'static str>,
}

impl Stdin {
    /// The text of `option`: its value, or standard input when it is `-`.
    fn resolve(&mut self, option: &'static str, value: Option<String>) -> Result<Option<String>> {
        if value.as_deref() != Some("-") {
            return Ok(value);
        }
        if let Some(first) = self.read_by.replace(option) {
            return Err(Error::usage(format!(
                "{first} and {option} both read standard input; at most one option can"
            )));
        }
        let mut text = String::new();
        io::stdin().read_to_string(&mut text).map_err(|error| {
            Error::new(
                ErrorCode::Io,
                format!("cannot read standard input: {error}"),
            )
        })?;
        Ok(Some(text))
    }
}
