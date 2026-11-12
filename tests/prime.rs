//! `VER-AGENT-GUIDE`: the agent guide that `zban prime` prints, through the
//! real `zban` binary.

mod common;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use common::{stderr, stdout, temp_dir, temp_dir_outside_projects, write_files, zban};
use serde_json::{Value, json};
use zban::{Id, IssueType, NotReady, Prefix, Priority, Status};

/// The expected output of `zban prime`. Run the tests with
/// `ZBAN_UPDATE_SNAPSHOTS=1` to rewrite it after an intended change.
const SNAPSHOT: &str = "tests/snapshots/prime.md";

/// Commands the guide leaves out: `init` is a one-time human action, and
/// `prime` and `mcp` set up agents rather than serve them.
const SETUP_COMMANDS: &[&str] = &["init", "prime", "mcp", "help"];

/// The sections of `DES-AGENT-GUIDE`, except the handoff section, which
/// comes with the handoff options.
const SECTIONS: &[&str] = &[
    "## Track work in zban",
    "## Before starting",
    "## While working",
    "## Finishing",
    "## Committing",
    "## Finding work",
    "## Reference",
];

fn project(name: &str) -> PathBuf {
    let root = temp_dir(&format!("prime-{name}"));
    write_files(
        &root,
        &[(".zban/config.toml", "format_version = 1\nprefix = \"zb\"\n")],
    );
    root
}

fn succeeded(output: &Output) -> String {
    assert!(output.status.success(), "{}", stderr(output));
    assert_eq!(stderr(output), "");
    stdout(output)
}

/// The guide as `zban prime` prints it inside a new project named `name`.
fn guide(name: &str) -> String {
    succeeded(&zban(&project(name), &["prime"]))
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
    let root = project("inside");
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
    let outside = temp_dir_outside_projects("prime-outside");
    let project = project("from-outside");

    let output = zban(&outside, &["prime"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "");
    assert_eq!(stderr(&output), "");

    let json: Value =
        serde_json::from_str(&succeeded(&zban(&outside, &["prime", "--json"]))).unwrap();
    assert_eq!(json, json!({ "guide": null }));

    let expected = guide("outside-expected");
    let directed = succeeded(&zban(&outside, &["prime", "-C", project.to_str().unwrap()]));
    assert_eq!(directed, expected);
    let json: Value = serde_json::from_str(&succeeded(&zban(
        &outside,
        &["-C", project.to_str().unwrap(), "prime", "--json"],
    )))
    .unwrap();
    assert_eq!(json, json!({ "guide": expected }));
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn prime_counts_a_project_with_an_invalid_configuration() {
    let root = temp_dir("prime-invalid-config");
    write_files(&root, &[(".zban/config.toml", "format_version = 99\n")]);
    assert_eq!(
        succeeded(&zban(&root, &["prime"])),
        guide("invalid-config-expected")
    );

    let list = zban(&root, &["list"]);
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
    let outside = temp_dir_outside_projects("prime-missing");
    let output = zban(&outside, &["prime", "--json", "-C", "missing"]);
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["error"]["code"], "io");
}

// @mara implements VER-AGENT-GUIDE
#[test]
fn guide_vocabulary_equals_the_code() {
    let guide = guide("vocabulary");
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
    for invocation in code_spans(&guide("names")) {
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
    let guide = guide("commands");
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
    let guide = guide("sections");
    let headings: Vec<_> = guide
        .lines()
        .filter(|line| line.starts_with("## "))
        .collect();
    assert_eq!(headings, SECTIONS);
    let lines = guide.lines().count();
    assert!(lines <= 150, "the guide has {lines} lines");
}

/// For `VAL-DOGFOOD`: this repository's `AGENTS.md` loads the guide with the
/// line that `docs/agents.mara.md` recommends, and `zban prime` prints the
/// guide here.
#[test]
fn this_repository_loads_the_guide() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let documented = fs::read_to_string(root.join("docs/agents.mara.md")).unwrap();
    let line = documented
        .lines()
        .find(|line| line.starts_with("**IMPORTANT**") && line.contains("`zban prime`"))
        .expect("docs/agents.mara.md recommends a `zban prime` line");
    let agents = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(
        agents.lines().any(|agents_line| agents_line == line),
        "AGENTS.md lacks {line:?}"
    );

    let expected = fs::read_to_string(root.join(SNAPSHOT)).unwrap();
    assert_eq!(succeeded(&zban(root, &["prime"])), expected);
}
