//! `VER-AGENT-GUIDE`: the agent guide that `zban prime` prints and that
//! `zban mcp` sends as its instructions, through the real `zban` binary.

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use common::mcp::Client;
use common::{stderr, stdout, temp_dir, write_files, zban};
use serde_json::{Value, json};
use tempfile::TempDir;
use zban::{Id, IssueType, NotReady, Prefix, Priority, Status};

/// The expected output of `zban prime`. Run the tests with
/// `ZBAN_UPDATE_SNAPSHOTS=1` to rewrite it after an intended change.
const SNAPSHOT: &str = "tests/snapshots/prime.md";

/// The expected MCP server instructions inside a project, updated the same
/// way.
const MCP_SNAPSHOT: &str = "tests/snapshots/mcp.md";

/// The most characters the MCP server instructions may hold
/// (`DES-AGENT-GUIDE`), with a margin under the 4,096 at which Claude Code
/// truncates them.
const MCP_BUDGET: usize = 3_800;

/// Commands the guide leaves out: `init` is a one-time human action, and
/// `prime` and `mcp` set up agents rather than serve them.
const SETUP_COMMANDS: &[&str] = &["init", "prime", "mcp", "help"];

/// The sections of `DES-AGENT-GUIDE`.
const SECTIONS: &[&str] = &[
    "## Track work in zban",
    "## Before starting",
    "## While working",
    "## Finishing",
    "## Committing",
    "## Handoffs",
    "## Finding work",
    "## Reference",
];

/// A new project, in a directory that lasts as long as the returned
/// [`TempDir`], and its root.
fn project() -> (TempDir, PathBuf) {
    let dir = temp_dir();
    let root = dir.path().to_owned();
    write_files(
        &root,
        &[(".zban/config.toml", "format_version = 1\nprefix = \"zb\"\n")],
    );
    (dir, root)
}

fn succeeded(output: &Output) -> String {
    assert!(output.status.success(), "{}", stderr(output));
    assert_eq!(stderr(output), "");
    stdout(output)
}

/// The guide as `zban prime` prints it inside a new project.
fn guide() -> String {
    let (_dir, root) = project();
    succeeded(&zban(&root, &["prime"]))
}

/// The instructions of an MCP server started in `dir` with `args`.
fn instructions(dir: &Path, args: &[&str]) -> String {
    let client = Client::start(dir, args);
    let instructions = client.instructions().to_owned();
    assert!(client.finish().success());
    instructions
}

/// The guide as the MCP server sends it inside a new project.
fn mcp_guide() -> String {
    let (_dir, root) = project();
    instructions(&root, &[])
}

/// Both renderings of the guide.
fn guides() -> [String; 2] {
    [guide(), mcp_guide()]
}

/// The input schema of each tool, by name.
fn tool_schemas() -> BTreeMap<String, Value> {
    let (_dir, root) = project();
    let mut client = Client::start(&root, &[]);
    client
        .tools()
        .into_iter()
        .map(|tool| {
            (
                tool["name"].as_str().unwrap().to_owned(),
                tool["inputSchema"].clone(),
            )
        })
        .collect()
}

/// Checks `text` against the snapshot at `path`, after rewriting it when
/// `ZBAN_UPDATE_SNAPSHOTS` is set.
fn assert_snapshot(path: &str, text: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    if std::env::var_os("ZBAN_UPDATE_SNAPSHOTS").is_some() {
        fs::write(&path, text).unwrap();
    }
    let expected = fs::read_to_string(&path).unwrap();
    assert_eq!(
        text,
        expected,
        "the guide differs from {}; rerun with ZBAN_UPDATE_SNAPSHOTS=1 if intended",
        path.display()
    );
}

/// The JSON type a schema property takes, besides `null`.
fn schema_type(property: &Value) -> &str {
    match &property["type"] {
        Value::String(kind) => kind,
        Value::Array(kinds) => kinds
            .iter()
            .filter_map(Value::as_str)
            .find(|kind| *kind != "null")
            .unwrap(),
        other => panic!("no type: {other}"),
    }
}

/// The JSON type of a value in the guide.
fn value_type(value: &Value) -> &'static str {
    match value {
        Value::Array(_) => "array",
        Value::Bool(_) => "boolean",
        Value::String(_) => "string",
        other => panic!("unexpected value {other}"),
    }
}

/// The arguments of a tool call in the guide, such as
/// `{"id": ID, "status": "done"}`, with placeholders made strings.
fn call_arguments(text: &str) -> serde_json::Map<String, Value> {
    let mut json = String::new();
    let (mut in_string, mut escaped, mut placeholder) = (false, false, false);
    for c in text.chars() {
        // Quotes open before the first letter of a placeholder and close
        // after its last.
        let letter = !in_string && c.is_ascii_uppercase();
        if letter != placeholder {
            json.push('"');
        }
        placeholder = letter;
        if in_string {
            in_string = escaped || c != '"';
            escaped = !escaped && c == '\\';
        } else {
            in_string = c == '"';
        }
        json.push(c);
    }
    serde_json::from_str(&json).unwrap_or_else(|error| panic!("{text}: {error}"))
}

/// The tool references in the MCP guide: each tool with the rest of its code
/// span, such as `{"id": ID}`.
fn tool_spans(guide: &str) -> Vec<(String, String)> {
    code_spans(guide)
        .into_iter()
        .filter_map(|span| {
            let (tool, rest) = span.split_once(' ').unwrap_or((&span, ""));
            let named = tool.ends_with("_issue") || tool.ends_with("_issues");
            named.then(|| (tool.to_owned(), rest.to_owned()))
        })
        .collect()
}

/// The text of every code span in `markdown`, with line breaks as spaces.
fn code_spans(markdown: &str) -> Vec<String> {
    markdown
        .split('`')
        .skip(1)
        .step_by(2)
        .map(|span| span.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

/// The commands that `zban --help` lists.
fn commands() -> Vec<String> {
    let help = succeeded(&zban(Path::new("."), &["--help"]));
    help.lines()
        .skip_while(|line| *line != "Commands:")
        .skip(1)
        .take_while(|line| !line.is_empty())
        .map(|line| line.split_whitespace().next().unwrap().to_owned())
        .collect()
}

/// Whether `help` documents `option`, such as `--status` in
/// `  -s, --status <STATUS>  Description`.
fn documents(help: &str, option: &str) -> bool {
    help.lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with('-'))
        .any(|line| {
            line.split([' ', ','])
                .take_while(|word| word.is_empty() || word.starts_with(['-', '<']))
                .any(|word| word == option)
        })
}

/// The options named in a `zban` invocation or synopsis such as
/// `zban list [--status S]... [--parent ID|--no-parent]`, each with the word
/// that follows it.
fn options(invocation: &str) -> Vec<(String, Option<String>)> {
    let words: Vec<_> = invocation
        .split_whitespace()
        .flat_map(|word| word.trim_matches(['[', ']', '.']).split('|'))
        .collect();
    words
        .iter()
        .enumerate()
        .filter(|(_, word)| {
            word.starts_with('-')
                && word
                    .trim_start_matches('-')
                    .starts_with(char::is_alphabetic)
        })
        .map(|(i, word)| ((*word).to_owned(), words.get(i + 1).map(|&w| w.to_owned())))
        .collect()
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn prime_prints_the_guide_inside_a_project() {
    let (_dir, root) = project();
    let printed = succeeded(&zban(&root, &["prime"]));
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOT);
    if std::env::var_os("ZBAN_UPDATE_SNAPSHOTS").is_some() {
        fs::write(&path, &printed).unwrap();
    }
    let expected = fs::read_to_string(&path).unwrap();
    assert_eq!(
        printed, expected,
        "the guide differs from {SNAPSHOT}; rerun with ZBAN_UPDATE_SNAPSHOTS=1 if intended"
    );

    write_files(&root, &[("src/deep/file.txt", "")]);
    let below = succeeded(&zban(&root.join("src/deep"), &["prime"]));
    assert_eq!(below, expected, "found by searching upward");

    let json: Value = serde_json::from_str(&succeeded(&zban(&root, &["prime", "--json"]))).unwrap();
    assert_eq!(json, json!({ "guide": expected }));
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn prime_prints_nothing_outside_a_project() {
    let outside = temp_dir();
    let (_dir, project) = project();

    let output = zban(outside.path(), &["prime"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");

    let json: Value =
        serde_json::from_str(&succeeded(&zban(outside.path(), &["prime", "--json"]))).unwrap();
    assert_eq!(json, json!({ "guide": null }));

    let expected = guide();
    let directed = succeeded(&zban(
        outside.path(),
        &["prime", "-C", project.to_str().unwrap()],
    ));
    assert_eq!(directed, expected);
    let json: Value = serde_json::from_str(&succeeded(&zban(
        outside.path(),
        &["-C", project.to_str().unwrap(), "prime", "--json"],
    )))
    .unwrap();
    assert_eq!(json, json!({ "guide": expected }));
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn prime_counts_a_project_with_an_invalid_configuration() {
    let dir = temp_dir();
    let root = dir.path();
    write_files(root, &[(".zban/config.toml", "format_version = 99\n")]);
    assert_eq!(succeeded(&zban(root, &["prime"])), guide());

    let list = zban(root, &["list"]);
    assert_eq!(list.status.code(), Some(1));
    assert!(
        stderr(&list).contains("[invalid_config]"),
        "{}",
        stderr(&list)
    );
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn prime_fails_for_a_missing_directory() {
    let outside = temp_dir();
    let output = zban(outside.path(), &["prime", "--json", "-C", "missing"]);
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["error"]["code"], "io");
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn guide_vocabulary_equals_the_code() {
    for guide in guides() {
        vocabulary_equals_the_code(&guide);
    }
}

fn vocabulary_equals_the_code(guide: &str) {
    let words = |words: Vec<String>| {
        words
            .iter()
            .map(|word| format!("`{word}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let types = words(IssueType::ALL.iter().map(|t| t.to_string()).collect());
    let statuses = words(Status::ALL.iter().map(|s| s.to_string()).collect());
    let priorities = words(Priority::ALL.iter().map(|p| p.to_string()).collect());
    let id = Id::from_bits(&Prefix::default(), 1);
    let reasons = words(
        [
            NotReady::Status,
            NotReady::Epic,
            NotReady::BlockedBy(id.clone()),
            NotReady::AncestorClosed(id.clone()),
            NotReady::AncestorBlocked(id.clone()),
        ]
        .iter()
        .map(|reason| reason.to_string().replace(id.as_str(), "<id>"))
        .collect(),
    );
    assert!(guide.contains(&format!("Types: {types}.")), "{guide}");
    assert!(guide.contains(&format!("Statuses: {statuses}.")), "{guide}");
    assert!(
        guide.contains(&format!("Priorities, highest first: {priorities}.")),
        "{guide}"
    );
    assert!(
        guide.contains(&format!("The reasons in `not_ready` are {reasons}.")),
        "{guide}"
    );
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn guide_names_only_commands_and_options_that_exist() {
    let commands = commands();
    let (mut invocations, mut options_checked) = (0, 0);
    for invocation in code_spans(&guide()) {
        let Some(rest) = invocation.strip_prefix("zban ") else {
            continue;
        };
        let command = rest.split_whitespace().next().unwrap();
        assert!(
            commands.iter().any(|known| known == command),
            "`{invocation}` names an unknown command; zban has {commands:?}"
        );
        let help = succeeded(&zban(Path::new("."), &[command, "--help"]));
        for (option, value) in options(rest) {
            assert!(
                documents(&help, &option),
                "`{invocation}` names {option}, which `zban {command}` does not take"
            );
            // Example values must be valid words; placeholders are uppercase.
            let value = value.filter(|value| value.starts_with(char::is_lowercase));
            if let Some(value) = value {
                let valid = match option.as_str() {
                    "--status" => value.parse::<Status>().is_ok(),
                    "--type" => value.parse::<IssueType>().is_ok(),
                    "--priority" => value.parse::<Priority>().is_ok(),
                    _ => true,
                };
                assert!(valid, "`{invocation}` gives {option} the invalid {value:?}");
            }
            options_checked += 1;
        }
        invocations += 1;
    }
    assert!(invocations >= 10, "only {invocations} invocations found");
    assert!(
        options_checked >= 40,
        "only {options_checked} options found"
    );
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn guide_lists_every_command_agents_use() {
    let guide = guide();
    let named: BTreeSet<_> = code_spans(&guide)
        .iter()
        .filter_map(|span| span.strip_prefix("zban "))
        .filter_map(|rest| rest.split_whitespace().next())
        .map(str::to_owned)
        .collect();
    for command in commands() {
        if !SETUP_COMMANDS.contains(&command.as_str()) {
            assert!(
                guide.contains(&format!("\n- `zban {command} ")),
                "the reference lists no `zban {command}`"
            );
            assert!(named.contains(&command));
        }
    }
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn guide_covers_its_sections_within_150_lines() {
    for guide in guides() {
        let headings: Vec<_> = guide
            .lines()
            .filter(|line| line.starts_with("## "))
            .collect();
        assert_eq!(headings, SECTIONS);
        let lines = guide.lines().count();
        assert!(lines <= 150, "the guide has {lines} lines");
    }
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn mcp_guide_fits_the_instructions_budget() {
    let length = mcp_guide().chars().count();
    assert!(
        length <= MCP_BUDGET,
        "the MCP guide has {length} characters, more than {MCP_BUDGET}"
    );
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn mcp_server_sends_the_guide_inside_a_project() {
    let (_dir, root) = project();
    let sent = instructions(&root, &[]);
    assert_snapshot(MCP_SNAPSHOT, &sent);
    assert_ne!(sent, guide(), "rendered for MCP");

    write_files(&root, &[("src/deep/file.txt", "")]);
    assert_eq!(instructions(&root.join("src/deep"), &[]), sent);
    let outside = temp_dir();
    assert_eq!(
        instructions(outside.path(), &["-C", root.to_str().unwrap()]),
        sent
    );

    let invalid = temp_dir();
    write_files(
        invalid.path(),
        &[(".zban/config.toml", "format_version = 99\n")],
    );
    assert_eq!(instructions(invalid.path(), &[]), sent);
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn mcp_server_sends_a_short_note_outside_a_project() {
    let outside = temp_dir();
    let note = instructions(outside.path(), &[]);
    assert!(note.contains("`directory`"), "{note}");
    assert!(note.contains("`zban prime -C <dir>`"), "{note}");
    assert!(note.contains(".zban/"), "{note}");
    assert!(note.lines().count() <= 6, "{note}");
    assert!(!note.contains("## "), "{note}");
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn guide_names_only_tools_and_parameters_that_exist() {
    let schemas = tool_schemas();
    let (mut calls, mut params_checked) = (0, 0);
    for (tool, rest) in tool_spans(&mcp_guide()) {
        let schema = schemas
            .get(&tool)
            .unwrap_or_else(|| panic!("`{tool}` is not a tool; zban has {:?}", schemas.keys()));
        let properties = &schema["properties"];
        let named: Vec<(String, &str)> = if rest.is_empty() {
            Vec::new()
        } else {
            calls += 1;
            call_arguments(&rest)
                .iter()
                .map(|(name, value)| (name.clone(), value_type(value)))
                .collect()
        };
        for (name, kind) in named {
            let property = properties.get(&name).unwrap_or_else(|| {
                panic!("`{tool} {rest}` names `{name}`, which it does not take")
            });
            assert_eq!(schema_type(property), kind, "`{tool} {rest}`: `{name}`");
            params_checked += 1;
        }
    }
    assert!(calls >= 10, "only {calls} calls found");
    assert!(
        params_checked >= 20,
        "only {params_checked} parameters found"
    );
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn guide_lists_every_option_of_each_command() {
    let guide = guide();
    let reference = guide.split("\n## Reference\n").nth(1).unwrap();
    let schemas = tool_schemas();
    for command in commands() {
        if SETUP_COMMANDS.contains(&command.as_str()) {
            continue;
        }
        let prefix = format!("\n- `zban {command}");
        let at = reference
            .find(&prefix)
            .unwrap_or_else(|| panic!("the reference lists no `zban {command}`"));
        let line = &reference[at + 4..];
        let synopsis = &line[..line.find('`').unwrap()];
        let listed: BTreeSet<String> = options(synopsis).into_iter().map(|(o, _)| o).collect();

        let help = succeeded(&zban(Path::new("."), &[&command, "--help"]));
        let offered: BTreeSet<String> = help
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with('-'))
            .filter_map(|line| line.split([' ', ',']).find(|word| word.starts_with("--")))
            .filter(|option| !["--help", "--json", "--directory"].contains(option))
            .map(str::to_owned)
            .collect();
        assert_eq!(listed, offered, "`zban {command}`");

        // The tool takes the same options as parameters, named with
        // underscores, besides `directory` and the positional arguments.
        let (tool, schema) = schemas
            .iter()
            .find(|(tool, _)| tool.split('_').next() == Some(command.as_str()))
            .unwrap_or_else(|| panic!("no tool for `zban {command}`"));
        let parameters: BTreeSet<String> = schema["properties"]
            .as_object()
            .unwrap()
            .keys()
            .map(|name| format!("--{}", name.replace('_', "-")))
            .collect();
        let unlisted: Vec<_> = parameters.difference(&listed).collect();
        assert!(listed.is_subset(&parameters), "`{tool}` lacks options");
        assert!(
            unlisted
                .iter()
                .all(|name| ["--directory", "--id", "--ids", "--title"].contains(&name.as_str())),
            "`{tool}` takes {unlisted:?}"
        );

        // Positional arguments, such as `<ID>...` in `Usage: zban show [OPTIONS] <ID>...`.
        let usage = help
            .lines()
            .find(|line| line.starts_with("Usage: "))
            .unwrap();
        let arguments: Vec<String> = usage
            .split_whitespace()
            .filter(|word| word.starts_with('<'))
            .map(|word| word.replace(['<', '>'], ""))
            .collect();
        let mut depth = 0;
        let mut positional = Vec::new();
        for word in synopsis.split_whitespace().skip(2) {
            if depth == 0 && !word.starts_with('[') {
                positional.push(word.to_owned());
            }
            depth += word.matches('[').count();
            depth -= word.matches(']').count();
        }
        assert_eq!(positional, arguments, "`zban {command}`");
    }
}

/// The line that `docs/agents.mara.md` recommends for a project's agent
/// instructions: the Markdown block that starts with `**IMPORTANT**`, with its
/// final line break.
fn recommended_line() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/agents.mara.md");
    let documented = fs::read_to_string(path).unwrap();
    let fence = "```markdown\n";
    let start = documented
        .find(&format!("{fence}**IMPORTANT**"))
        .expect("docs/agents.mara.md recommends an **IMPORTANT** line")
        + fence.len();
    let end = start + documented[start..].find("\n```\n").unwrap() + 1;
    documented[start..end].to_owned()
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn setup_line_names_the_heading_the_guide_starts_with() {
    let line = recommended_line();
    assert!(line.contains("`# zban workflow`"), "{line}");
    assert!(line.contains("`zban prime`"), "{line}");
    for guide in guides() {
        assert!(guide.starts_with("# zban workflow\n"), "{guide}");
    }
    let outside = temp_dir();
    let note = instructions(outside.path(), &[]);
    assert!(!note.contains("# zban workflow"), "{note}");
}

/// For `VAL-DOGFOOD`: this repository's `AGENTS.md` loads the guide with the
/// line that `docs/agents.mara.md` recommends, as a paragraph of its own, and
/// `zban prime` prints the guide here.
#[test]
fn this_repository_loads_the_guide() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let line = recommended_line();
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(
        agents.contains(&format!("\n\n{line}\n")),
        "AGENTS.md lacks the recommended line:\n{line}"
    );

    let expected = fs::read_to_string(root.join(SNAPSHOT)).unwrap();
    assert_eq!(succeeded(&zban(root, &["prime"])), expected);
}
