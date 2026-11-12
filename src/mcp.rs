//! The MCP adapter (`DES-MCP-TOOLS`): `zban mcp` serves the operations as
//! tools over stdio. Like the command line, it only turns tool arguments
//! into operation requests and results into the JSON documents of `DES-CLI`.
//! It is the only part of zban that starts the `tokio` runtime.

use std::path::{Path, PathBuf};

use rmcp::handler::server::tool::schema_for_input;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ListToolsResult, PaginatedRequestParams, ServerCapabilities, ServerConfig, Tool,
    ToolAnnotations,
};
use rmcp::schemars::JsonSchema;
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, ServiceExt as _};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use zban::operations::{self, CreateRequest, ListRequest, Replacement, UpdateRequest};
use zban::{Error, ErrorCode, Project, Result, Surface, guide};

/// Serves MCP over stdio until the client disconnects. Without a
/// `directory` argument, tools search for the project from `start`: the
/// directory given with `-C`, or the working directory.
// @mara implements REQ-MCP-STDIO
pub fn serve(start: &Path) -> Result<()> {
    let instructions = operations::guide(start, Surface::Mcp)?
        .guide
        .unwrap_or_else(|| guide::NOTE.to_owned());
    let server = Server {
        start: start.to_owned(),
        instructions,
    };
    let failed = |error: &dyn std::fmt::Display| {
        Error::new(ErrorCode::Io, format!("the MCP server failed: {error}"))
    };
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| failed(&error))?;
    runtime.block_on(async {
        let running = server
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|error| failed(&error))?;
        running.waiting().await.map_err(|error| failed(&error))?;
        Ok(())
    })
}

// @mara implements DES-MCP-TOOLS
struct Server {
    start: PathBuf,
    /// The agent guide, or the short note when no project was found at
    /// startup (`DES-AGENT-GUIDE`).
    instructions: String,
}

impl ServerHandler for Server {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("zban", env!("CARGO_PKG_VERSION")))
            .with_instructions(self.instructions.clone())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult::with_all_items(tools()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let arguments = request.arguments.unwrap_or_default();
        let outcome = match request.name.as_ref() {
            "list_issues" => parse(arguments).and_then(|args| self.list(args)),
            "show_issues" => parse(arguments).and_then(|args| self.show(args)),
            "create_issue" => parse(arguments).and_then(|args| self.create(args)),
            "update_issue" => parse(arguments).and_then(|args| self.update(args)),
            "delete_issue" => parse(arguments).and_then(|args| self.delete(args)),
            "check_issues" => parse(arguments).and_then(|args| self.check(args)),
            name => {
                return Err(ErrorData::invalid_params(
                    format!("zban has no tool `{name}`"),
                    None,
                ));
            }
        };
        Ok(result(outcome).into())
    }
}

/// The tools of `DES-MCP-TOOLS`. Their input schemas come from the argument
/// types below, which also parse the arguments.
// @mara implements DES-MCP-TOOLS
fn tools() -> Vec<Tool> {
    let read_only = || ToolAnnotations::new().read_only(true);
    vec![
        tool::<ListArgs>(
            "list_issues",
            "List open issues in priority order, or the issues matching the filters.",
        )
        .with_annotations(read_only()),
        tool::<ShowArgs>(
            "show_issues",
            "Show issues in full, with their body and the issues that refer to them.",
        )
        .with_annotations(read_only()),
        tool::<CreateArgs>("create_issue", "Create an issue."),
        tool::<UpdateArgs>(
            "update_issue",
            "Change an issue's fields, relations or body. The whole update is applied or nothing is.",
        ),
        tool::<DeleteArgs>(
            "delete_issue",
            "Delete an issue that no other issue refers to.",
        ),
        tool::<CheckArgs>(
            "check_issues",
            "Report problems in `.zban/`, each with a severity and a stable code.",
        ),
    ]
}

fn tool<T: JsonSchema + 'static>(name: &'static str, description: &'static str) -> Tool {
    let schema = schema_for_input::<T>().expect("argument types are objects");
    Tool::new(name, description, schema)
}

/// Tool arguments of the wrong shape are a `usage` failure, as on the
/// command line.
fn parse<T: DeserializeOwned>(arguments: Map<String, Value>) -> Result<T> {
    serde_json::from_value(Value::Object(arguments))
        .map_err(|error| Error::usage(format!("invalid arguments: {error}")))
}

/// A JSON document as the command line prints it with `--json`.
struct Document {
    text: String,
    value: Value,
}

impl Document {
    fn new(document: &impl Serialize) -> Self {
        Self {
            text: serde_json::to_string(document).expect("documents serialize to JSON"),
            value: serde_json::to_value(document).expect("documents serialize to JSON"),
        }
    }
}

/// The tool result: the document as structured content and as text, or the
/// error object as a tool error.
fn result(outcome: Result<Document>) -> CallToolResult {
    let (document, failed) = match outcome {
        Ok(document) => (document, false),
        Err(error) => {
            #[derive(Serialize)]
            struct Failure<'a> {
                error: &'a Error,
            }
            (Document::new(&Failure { error: &error }), true)
        }
    };
    let mut result = if failed {
        CallToolResult::structured_error(document.value)
    } else {
        CallToolResult::structured(document.value)
    };
    result.content = vec![ContentBlock::text(document.text)];
    result
}

impl Server {
    /// The directory to search for the project from: the call's absolute
    /// `directory`, or else the server's own.
    fn directory(&self, directory: Option<PathBuf>) -> Result<PathBuf> {
        match directory {
            None => Ok(self.start.clone()),
            Some(directory) if directory.is_absolute() => Ok(directory),
            Some(directory) => Err(Error::usage(format!(
                "`directory` must be an absolute path, not {:?}",
                directory.display().to_string()
            ))),
        }
    }

    /// Runs `operation` with the directory to search for the project from,
    /// resolved afresh on every call.
    fn run<T: Serialize>(
        &self,
        directory: Option<PathBuf>,
        operation: impl FnOnce(&Path) -> Result<T>,
    ) -> Result<Document> {
        let given = directory.is_some();
        let start = self.directory(directory)?;
        match operation(&start) {
            Ok(document) => Ok(Document::new(&document)),
            Err(error) if error.code == ErrorCode::ProjectNotFound && !given => Err(Error::new(
                error.code,
                format!(
                    "{}. To work on a project elsewhere, pass `directory`: \
                     the absolute path of a directory inside it",
                    error.message
                ),
            )),
            Err(error) => Err(error),
        }
    }

    fn list(&self, args: ListArgs) -> Result<Document> {
        let request = ListRequest {
            statuses: args.status,
            types: args.issue_type,
            priorities: args.priority,
            tags: args.tag,
            parent: args.parent,
            ready: args.ready,
            search: args.search,
            all: args.all,
        };
        self.run(args.directory, |start| {
            operations::list(&Project::discover(start)?, &request)
        })
    }

    fn show(&self, args: ShowArgs) -> Result<Document> {
        self.run(args.directory, |start| {
            operations::show(&Project::discover(start)?, &args.ids)
        })
    }

    fn create(&self, args: CreateArgs) -> Result<Document> {
        let request = CreateRequest {
            title: args.title,
            issue_type: args.issue_type,
            status: args.status,
            priority: args.priority,
            tags: args.tag,
            parent: args.parent,
            blocked_by: args.blocked_by,
            related: args.related,
            discovered_from: args.discovered_from,
            body: args.body,
        };
        self.run(args.directory, |start| {
            operations::create(&Project::discover(start)?, &request)
        })
    }

    fn update(&self, args: UpdateArgs) -> Result<Document> {
        if args.replace_old.len() != args.replace_new.len() {
            return Err(Error::usage(
                "give each `replace_old` text exactly one `replace_new` text, at the same position",
            ));
        }
        let replacements = args
            .replace_old
            .into_iter()
            .zip(args.replace_new)
            .map(|(old, new)| Replacement { old, new })
            .collect();
        let request = UpdateRequest {
            id: args.id,
            title: args.title,
            status: args.status,
            issue_type: args.issue_type,
            priority: args.priority,
            add_tags: args.add_tag,
            remove_tags: args.remove_tag,
            parent: removable("parent", args.parent, args.no_parent)?,
            add_blocked_by: args.add_blocked_by,
            remove_blocked_by: args.remove_blocked_by,
            add_related: args.add_related,
            remove_related: args.remove_related,
            discovered_from: removable(
                "discovered_from",
                args.discovered_from,
                args.no_discovered_from,
            )?,
            body: args.body,
            replacements,
            append: args.append,
            force: args.force,
        };
        self.run(args.directory, |start| {
            operations::update(&Project::discover(start)?, &request)
        })
    }

    fn delete(&self, args: DeleteArgs) -> Result<Document> {
        self.run(args.directory, |start| {
            operations::delete(&Project::discover(start)?, &args.id)
        })
    }

    fn check(&self, args: CheckArgs) -> Result<Document> {
        self.run(args.directory, |start| operations::check(start, args.fix))
    }
}

/// An optional value that an update sets with `key` or removes with
/// `no_<key>`.
fn removable(key: &str, value: Option<String>, remove: bool) -> Result<Option<Option<String>>> {
    match (value, remove) {
        (Some(_), true) => Err(Error::usage(format!(
            "`{key}` and `no_{key}` exclude each other"
        ))),
        (_, true) => Ok(Some(None)),
        (value, false) => Ok(value.map(Some)),
    }
}

// The argument types. Each mirrors the options of its command, with list
// values for repeatable options and `true` for flags (`REQ-MCP-STDIO`).
// Every tool also takes `directory`.

/// Repeated values of one filter match any of them; different filters must
/// all match.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
struct ListArgs {
    /// Absolute path of a directory inside the project, usually your working
    /// directory. Without it, the project where the server started is used.
    directory: Option<PathBuf>,
    /// draft, todo, in-progress, done or cancelled. Naming a closed status
    /// includes closed issues.
    #[serde(default)]
    status: Vec<String>,
    /// epic, feature, bug or task.
    #[serde(default, rename = "type")]
    issue_type: Vec<String>,
    /// critical, high, normal, low or deferred.
    #[serde(default)]
    priority: Vec<String>,
    #[serde(default)]
    tag: Vec<String>,
    /// Direct children of this issue.
    parent: Option<String>,
    /// Issues that can be started now.
    #[serde(default)]
    ready: bool,
    /// Text to find in the title or body, ignoring case.
    search: Option<String>,
    /// Include closed issues.
    #[serde(default)]
    all: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
struct ShowArgs {
    /// Absolute path of a directory inside the project, usually your working
    /// directory. Without it, the project where the server started is used.
    directory: Option<PathBuf>,
    /// The issues to show, in this order.
    ids: Vec<String>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
struct CreateArgs {
    /// Absolute path of a directory inside the project, usually your working
    /// directory. Without it, the project where the server started is used.
    directory: Option<PathBuf>,
    /// A single line.
    title: String,
    /// epic, feature, bug or task [default: task]
    #[serde(rename = "type")]
    issue_type: Option<String>,
    /// draft, todo, in-progress, done or cancelled [default: todo]
    status: Option<String>,
    /// critical, high, normal, low or deferred [default: normal]
    priority: Option<String>,
    #[serde(default)]
    tag: Vec<String>,
    parent: Option<String>,
    #[serde(default)]
    blocked_by: Vec<String>,
    #[serde(default)]
    related: Vec<String>,
    discovered_from: Option<String>,
    body: Option<String>,
}

/// Body edits apply in order: `body`, each replacement, then `append`.
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
struct UpdateArgs {
    /// Absolute path of a directory inside the project, usually your working
    /// directory. Without it, the project where the server started is used.
    directory: Option<PathBuf>,
    id: String,
    /// draft, todo, in-progress, done or cancelled.
    status: Option<String>,
    /// epic, feature, bug or task.
    #[serde(rename = "type")]
    issue_type: Option<String>,
    /// critical, high, normal, low or deferred.
    priority: Option<String>,
    title: Option<String>,
    #[serde(default)]
    add_tag: Vec<String>,
    #[serde(default)]
    remove_tag: Vec<String>,
    parent: Option<String>,
    /// Remove the parent.
    #[serde(default)]
    no_parent: bool,
    #[serde(default)]
    add_blocked_by: Vec<String>,
    #[serde(default)]
    remove_blocked_by: Vec<String>,
    #[serde(default)]
    add_related: Vec<String>,
    #[serde(default)]
    remove_related: Vec<String>,
    discovered_from: Option<String>,
    /// Remove `discovered_from`.
    #[serde(default)]
    no_discovered_from: bool,
    /// Replace the body.
    body: Option<String>,
    /// Append to the body after a blank line.
    append: Option<String>,
    /// Texts that each occur exactly once in the body, replaced by the
    /// `replace_new` text at the same position.
    #[serde(default)]
    replace_old: Vec<String>,
    /// The replacement for the `replace_old` text at the same position.
    #[serde(default)]
    replace_new: Vec<String>,
    /// Close even with unchecked task items or open children.
    #[serde(default)]
    force: bool,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
struct DeleteArgs {
    /// Absolute path of a directory inside the project, usually your working
    /// directory. Without it, the project where the server started is used.
    directory: Option<PathBuf>,
    id: String,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(crate = "rmcp::schemars")]
struct CheckArgs {
    /// Absolute path of a directory inside the project, usually your working
    /// directory. Without it, the project where the server started is used.
    directory: Option<PathBuf>,
    /// Move misplaced files into the directory their status requires, and
    /// delete temporary files left by interrupted writes.
    #[serde(default)]
    fix: bool,
}
