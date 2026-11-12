//! `VER-SURFACE-PARITY`: an MCP client talks to the real `zban mcp` over
//! stdio, and its tool results equal the command line's JSON output.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::mcp::{Called, Client};
use common::{snapshot, stderr, stdout, temp_dir, temp_dir_outside_projects, write_files, zban};
use serde_json::{Value, json};

const CONFIG: &str = "format_version = 1\nprefix = \"zb\"\n";

/// The tools of `DES-MCP-TOOLS`.
const TOOLS: &[&str] = &[
    "list_issues",
    "show_issues",
    "create_issue",
    "update_issue",
    "delete_issue",
    "check_issues",
];

const A: &str = "zb-aaaaaa"; // low task, blocks B
const B: &str = "zb-bbbbbb"; // task with an unchecked item, blocked by A
const E: &str = "zb-eeeeee"; // epic

fn fixture(root: &Path) {
    write_files(
        root,
        &[
            (".zban/config.toml", CONFIG),
            (
                ".zban/zb-aaaaaa.md",
                "---\ntitle: Existing task\nstatus: todo\ntype: task\npriority: low\n\
                 created_at: 2026-01-01T00:00:01Z\n---\n",
            ),
            (
                ".zban/zb-bbbbbb.md",
                "---\ntitle: Blocked task\nstatus: in-progress\ntype: task\nblocked_by:\n  - zb-aaaaaa\n\
                 created_at: 2026-01-01T00:00:02Z\n---\n\n- [ ] Step\n",
            ),
            (
                ".zban/zb-eeeeee.md",
                "---\ntitle: Epic\nstatus: todo\ntype: epic\ncreated_at: 2026-01-01T00:00:03Z\n---\n",
            ),
        ],
    );
}

/// The same fixture twice: one project for the command line and one for an
/// MCP server started in it.
struct Surfaces {
    cli: PathBuf,
    mcp: PathBuf,
    client: Client,
    /// The IDs of the issues created in both projects, as (CLI, MCP).
    created: Vec<(String, String)>,
}

impl Surfaces {
    fn new(name: &str) -> Self {
        let cli = temp_dir(&format!("mcp-{name}-cli"));
        let mcp = temp_dir(&format!("mcp-{name}-mcp"));
        fixture(&cli);
        fixture(&mcp);
        let client = Client::start(&mcp, &[]);
        Self {
            cli: fs::canonicalize(cli).unwrap(),
            mcp: fs::canonicalize(mcp).unwrap(),
            client,
            created: Vec::new(),
        }
    }

    /// Runs `args` with `--json` and the tool call with `arguments`, which
    /// name created issues by their CLI IDs, and checks that both give the
    /// same document. Returns the CLI's exit status and document.
    fn both(&mut self, args: &[&str], tool: &str, arguments: Value) -> (i32, Value) {
        let output = zban(&self.cli, &[&["--json"], args].concat());
        let status = output.status.code().unwrap();
        let document: Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{args:?}: {error}: {}", stderr(&output)));

        let mut arguments = arguments.to_string();
        for (cli, mcp) in &self.created {
            arguments = arguments
                .replace(cli.as_str(), mcp)
                .replace(&cli.to_uppercase(), &mcp.to_uppercase());
        }
        let called = self
            .client
            .call(tool, serde_json::from_str(&arguments).unwrap());
        // Only failures are tool errors, not a check that finds errors.
        let failed = document.get("error").is_some();
        assert_eq!(called.is_error, failed, "{args:?}: {called:?}");

        if tool == "create_issue" && status == 0 {
            let id = |document: &Value| document["id"].as_str().unwrap().to_owned();
            self.created.push((id(&document), id(&called.document)));
        }
        let mut text = called
            .text
            .replace(self.mcp.to_str().unwrap(), self.cli.to_str().unwrap());
        for (cli, mcp) in &self.created {
            text = text.replace(mcp.as_str(), cli);
        }
        let mut from_mcp: Value = serde_json::from_str(&text).unwrap();
        let mut from_cli = document.clone();
        // Issues created or closed during the test carry their own times.
        unify_times(&mut from_mcp);
        unify_times(&mut from_cli);
        if from_cli["error"]["code"] == "usage" {
            // The command line and MCP parse arguments differently, so only
            // the code is the same.
            assert_eq!(from_mcp["error"]["code"], "usage", "{args:?}: {text}");
        } else {
            assert_eq!(from_mcp, from_cli, "{args:?} and {tool}");
        }
        (status, document)
    }

    /// The CLI ID of the `n`th issue created.
    fn created(&self, n: usize) -> String {
        self.created[n].0.clone()
    }
}

fn unify_times(value: &mut Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object.iter_mut() {
                let time = ["created_at", "closed_at", "written_at", "handoff_at"];
                if time.contains(&key.as_str()) && value.is_string() {
                    *value = json!("<time>");
                } else if let (Value::String(body), "body") = (&mut *value, key.as_str()) {
                    *body = without_handoff_time(body);
                } else {
                    unify_times(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(unify_times),
        _ => {}
    }
}

/// `body` with the time in its handoff heading replaced.
fn without_handoff_time(body: &str) -> String {
    match body.split_once("## Handoff (") {
        Some((before, rest)) => {
            let after = &rest[rest.find(')').unwrap()..];
            format!("{before}## Handoff (<time>{after}")
        }
        None => body.to_owned(),
    }
}

fn ids(document: &Value) -> Vec<&str> {
    document["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["id"].as_str().unwrap())
        .collect()
}

fn error_code(called: &Called) -> &str {
    assert!(called.is_error, "{called:?}");
    called.document["error"]["code"].as_str().unwrap()
}

// @mara implements VER-SURFACE-PARITY
#[test]
fn the_tool_list_matches_the_design() {
    let root = temp_dir("mcp-tools");
    fixture(&root);
    let mut client = Client::start(&root, &[]);
    assert_eq!(client.initialized["serverInfo"]["name"], "zban");
    let capabilities = client.initialized["capabilities"].as_object().unwrap();
    assert_eq!(
        capabilities.keys().collect::<Vec<_>>(),
        ["tools"],
        "no resources or prompts"
    );
    let tools = client.tools();
    let names: Vec<_> = tools
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, TOOLS);
    for tool in &tools {
        let schema = &tool["inputSchema"];
        assert_eq!(schema["type"], "object", "{tool}");
        assert_eq!(schema["additionalProperties"], false, "{tool}");
        assert!(schema["properties"]["directory"].is_object(), "{tool}");
        let required = schema["required"].as_array().cloned().unwrap_or_default();
        assert!(!required.contains(&json!("directory")), "{tool}");
        assert!(!tool["description"].as_str().unwrap().is_empty(), "{tool}");
    }
    assert!(
        client
            .try_request(
                "tools/call",
                json!({"name": "init_project", "arguments": {}})
            )
            .is_err(),
        "no tool for init"
    );
    assert!(client.finish().success());
}

// @mara implements VER-SURFACE-PARITY
#[test]
fn a_create_block_ready_update_close_check_flow_matches_the_cli() {
    let mut s = Surfaces::new("flow");

    // Create, and block a second issue on the first.
    let (_, parse) = s.both(
        &[
            "create",
            "Parse files",
            "-t",
            "feature",
            "-p",
            "high",
            "--tag",
            "storage",
            "--parent",
            E,
            "--body",
            "- [ ] Read\n- [ ] Write",
        ],
        "create_issue",
        json!({"title": "Parse files", "type": "feature", "priority": "high",
               "tag": ["storage"], "parent": E, "body": "- [ ] Read\n- [ ] Write"}),
    );
    assert_eq!(parse["ready"], true);
    let parse = s.created(0);
    s.both(
        &[
            "create",
            "Write docs",
            "--blocked-by",
            &parse,
            "--discovered-from",
            A,
        ],
        "create_issue",
        json!({"title": "Write docs", "blocked_by": [parse], "discovered_from": A}),
    );
    let docs = s.created(1);

    // Ready work, in priority order, and why the rest is not ready.
    let (_, ready) = s.both(&["list", "--ready"], "list_issues", json!({"ready": true}));
    assert_eq!(ids(&ready), [parse.as_str(), A]);
    s.both(&["list"], "list_issues", json!({}));
    s.both(
        &[
            "list", "--type", "task", "--type", "epic", "--search", "TASK",
        ],
        "list_issues",
        json!({"type": ["task", "epic"], "search": "TASK"}),
    );
    s.both(
        &["list", "--parent", E, "--tag", "storage"],
        "list_issues",
        json!({"parent": E, "tag": ["storage"]}),
    );

    // Work on the first issue.
    s.both(
        &["update", &parse, "-s", "in-progress", "--add-tag", "parser"],
        "update_issue",
        json!({"id": parse, "status": "in-progress", "add_tag": ["parser"]}),
    );
    s.both(
        &[
            "update",
            &parse,
            "--replace-old",
            "- [ ] Read",
            "--replace-new",
            "- [x] Read",
        ],
        "update_issue",
        json!({"id": parse, "replace_old": ["- [ ] Read"], "replace_new": ["- [x] Read"]}),
    );
    // Hand the work off and read the note back.
    s.both(
        &[
            "update",
            &parse,
            "--handoff",
            "- Next: write\n- Rejected: streaming",
        ],
        "update_issue",
        json!({"id": parse, "handoff": "- Next: write\n- Rejected: streaming"}),
    );
    let (_, listed) = s.both(&["list"], "list_issues", json!({}));
    assert!(listed["issues"][0]["handoff_at"].is_string(), "{listed}");
    let (_, shown) = s.both(&["show", &parse], "show_issues", json!({"ids": [parse]}));
    assert_eq!(
        shown["issues"][0]["handoff"]["text"],
        "- Next: write\n- Rejected: streaming\n"
    );

    let (status, refused) = s.both(
        &["update", &parse, "-s", "done"],
        "update_issue",
        json!({"id": parse, "status": "done"}),
    );
    assert_eq!(
        (status, &refused["error"]["code"]),
        (1, &json!("unchecked_items"))
    );

    // Close it, replacing the handoff with a summary.
    let (_, closed) = s.both(
        &[
            "update",
            &parse,
            "--replace-old",
            "- [ ] Write",
            "--replace-new",
            "- [x] Write",
            "--append",
            "## Summary of Changes\n\nDone.",
            "--no-handoff",
            "-s",
            "done",
            "--no-parent",
        ],
        "update_issue",
        json!({"id": parse, "replace_old": ["- [ ] Write"], "replace_new": ["- [x] Write"],
               "append": "## Summary of Changes\n\nDone.", "no_handoff": true,
               "status": "done", "no_parent": true}),
    );
    assert!(closed["path"].as_str().unwrap().contains("/archive/"));
    let (_, ready) = s.both(&["list", "--ready"], "list_issues", json!({"ready": true}));
    assert_eq!(ids(&ready), [docs.as_str(), A]);
    s.both(
        &["list", "--status", "done"],
        "list_issues",
        json!({"status": ["done"]}),
    );
    s.both(&["list", "--all"], "list_issues", json!({"all": true}));
    let (_, shown) = s.both(
        &["show", &parse, &docs.to_uppercase(), A],
        "show_issues",
        json!({"ids": [parse, docs.to_uppercase(), A]}),
    );
    assert_eq!(shown["issues"][0]["blocks"], json!([docs]));
    assert_eq!(shown["issues"][0]["handoff"], Value::Null);
    let body = shown["issues"][0]["body"].as_str().unwrap();
    assert!(
        body.ends_with("\n\n## Summary of Changes\n\nDone.\n"),
        "{body}"
    );
    assert!(!body.contains("Rejected"), "{body}");

    // Check, then remove what the flow created.
    let (_, checked) = s.both(&["check"], "check_issues", json!({}));
    assert_eq!(checked, json!({"findings": []}));
    s.both(&["check", "--fix"], "check_issues", json!({"fix": true}));
    s.both(&["delete", &docs], "delete_issue", json!({"id": docs}));
    s.both(&["delete", &parse], "delete_issue", json!({"id": parse}));

    let files = |root: &Path| snapshot(&root.join(".zban"), &[]);
    assert_eq!(files(&s.cli), files(&s.mcp));
}

// @mara implements VER-SURFACE-PARITY
#[test]
fn invalid_arguments_give_the_same_error_codes() {
    let mut s = Surfaces::new("errors");
    let before = snapshot(&s.cli.join(".zban"), &[]);
    let cases: &[(&[&str], &str, Value, &str)] = &[
        (
            &["list", "--status", "nope"],
            "list_issues",
            json!({"status": ["nope"]}),
            "invalid_value",
        ),
        (
            &["list", "--nope"],
            "list_issues",
            json!({"nope": true}),
            "usage",
        ),
        (
            &["list", "--ready=yes"],
            "list_issues",
            json!({"ready": "yes"}),
            "usage",
        ),
        (&["show"], "show_issues", json!({}), "usage"),
        (
            &["show", "zb-zzzzzz"],
            "show_issues",
            json!({"ids": ["zb-zzzzzz"]}),
            "not_found",
        ),
        (
            &["show", "nonsense"],
            "show_issues",
            json!({"ids": ["nonsense"]}),
            "not_found",
        ),
        (&["create"], "create_issue", json!({}), "usage"),
        (
            &["create", "Two\nlines"],
            "create_issue",
            json!({"title": "Two\nlines"}),
            "invalid_value",
        ),
        (
            &["create", "T", "-p", "urgent"],
            "create_issue",
            json!({"title": "T", "priority": "urgent"}),
            "invalid_value",
        ),
        (
            &["create", "T", "--parent", "zb-zzzzzz"],
            "create_issue",
            json!({"title": "T", "parent": "zb-zzzzzz"}),
            "invalid_reference",
        ),
        (
            &["update", A, "--add-blocked-by", A],
            "update_issue",
            json!({"id": A, "add_blocked_by": [A]}),
            "invalid_reference",
        ),
        (
            &["update", A, "--add-blocked-by", B],
            "update_issue",
            json!({"id": A, "add_blocked_by": [B]}),
            "cycle",
        ),
        (
            &["update", B, "-s", "done"],
            "update_issue",
            json!({"id": B, "status": "done"}),
            "unchecked_items",
        ),
        (
            &["update", B, "--replace-old", "absent", "--replace-new", "x"],
            "update_issue",
            json!({"id": B, "replace_old": ["absent"], "replace_new": ["x"]}),
            "text_not_found",
        ),
        (
            &["update", B, "--replace-old", " ", "--replace-new", "x"],
            "update_issue",
            json!({"id": B, "replace_old": [" "], "replace_new": ["x"]}),
            "text_ambiguous",
        ),
        (
            &["update", B, "--replace-old", "Step"],
            "update_issue",
            json!({"id": B, "replace_old": ["Step"]}),
            "usage",
        ),
        (
            &["update", B, "--handoff", "- x\n## Next"],
            "update_issue",
            json!({"id": B, "handoff": "- x\n## Next"}),
            "invalid_handoff",
        ),
        (
            &["update", B, "--handoff", "x", "--no-handoff"],
            "update_issue",
            json!({"id": B, "handoff": "x", "no_handoff": true}),
            "usage",
        ),
        (
            &["update", B, "--add-tag", "x", "--remove-tag", "x"],
            "update_issue",
            json!({"id": B, "add_tag": ["x"], "remove_tag": ["x"]}),
            "usage",
        ),
        (
            &["update", B, "--parent", E, "--no-parent"],
            "update_issue",
            json!({"id": B, "parent": E, "no_parent": true}),
            "usage",
        ),
        (
            &["update", B, "--discovered-from", A, "--no-discovered-from"],
            "update_issue",
            json!({"id": B, "discovered_from": A, "no_discovered_from": true}),
            "usage",
        ),
        (
            &["delete", A],
            "delete_issue",
            json!({"id": A}),
            "referenced",
        ),
        (&["delete"], "delete_issue", json!({}), "usage"),
    ];
    for (args, tool, arguments, expected) in cases {
        let (status, document) = s.both(args, tool, arguments.clone());
        assert_eq!(document["error"]["code"], *expected, "{args:?}: {document}");
        assert_eq!(status, if *expected == "usage" { 2 } else { 1 }, "{args:?}");
    }
    let files = |root: &Path| snapshot(&root.join(".zban"), &[]);
    assert_eq!(files(&s.cli), before);
    assert_eq!(files(&s.mcp), before);
}

// @mara implements VER-SURFACE-PARITY
#[test]
fn a_check_that_finds_errors_is_a_result() {
    let mut s = Surfaces::new("check");
    let closed = "---\ntitle: Closed\nstatus: done\ntype: task\n\
                  created_at: 2026-01-01T00:00:04Z\nclosed_at: 2026-01-02T00:00:00Z\n---\n";
    for root in [&s.cli, &s.mcp] {
        write_files(root, &[(".zban/zb-cccccc.md", closed)]);
    }
    let (status, checked) = s.both(&["check"], "check_issues", json!({}));
    assert_eq!(status, 1);
    assert_eq!(checked["findings"][0]["code"], "misplaced", "{checked}");
    let (status, fixed) = s.both(&["check", "--fix"], "check_issues", json!({"fix": true}));
    assert_eq!((status, fixed), (0, json!({"findings": []})));
    assert!(s.mcp.join(".zban/archive/zb-cccccc.md").is_file());
}

/// Usage failures that only MCP has: a relative `directory`, and tools that
/// do not exist.
// @mara implements VER-SURFACE-PARITY
#[test]
fn a_relative_directory_is_rejected() {
    let root = temp_dir("mcp-relative");
    fixture(&root);
    fs::create_dir_all(root.join("sub")).unwrap();
    let before = snapshot(&root, &[]);
    let mut client = Client::start(&root, &[]);
    for (tool, arguments) in [
        ("list_issues", json!({"directory": "sub"})),
        ("list_issues", json!({"directory": "."})),
        ("create_issue", json!({"directory": "sub", "title": "Lost"})),
        ("check_issues", json!({"directory": "../x"})),
    ] {
        let called = client.call(tool, arguments);
        assert_eq!(error_code(&called), "usage", "{tool}");
        assert!(called.text.contains("absolute"), "{}", called.text);
    }
    assert_eq!(snapshot(&root, &[]), before);
}

// @mara implements VER-SURFACE-PARITY
#[test]
fn outside_a_project_every_tool_needs_a_directory() {
    let outside = temp_dir_outside_projects("mcp-outside");
    let project = temp_dir("mcp-outside-project");
    fixture(&project);
    let project = fs::canonicalize(project).unwrap();

    let mut client = Client::start(&outside, &[]);
    assert_eq!(client.instructions(), zban::guide::NOTE);
    for (tool, arguments) in [
        ("list_issues", json!({})),
        ("show_issues", json!({"ids": [A]})),
        ("create_issue", json!({"title": "Lost"})),
        ("update_issue", json!({"id": A, "status": "done"})),
        ("delete_issue", json!({"id": A})),
        ("check_issues", json!({})),
    ] {
        let called = client.call(tool, arguments.clone());
        assert_eq!(error_code(&called), "project_not_found", "{tool}");
        let message = called.document["error"]["message"].as_str().unwrap();
        assert!(message.contains("`directory`"), "{tool}: {message}");

        // The same call with a `directory` inside the project works.
        let mut arguments = arguments;
        arguments["directory"] = json!(project.join(".zban").to_str().unwrap());
        let called = client.call(tool, arguments);
        let code = called.document["error"]["code"].as_str();
        assert!(
            !called.is_error || code == Some("referenced"),
            "{tool}: {:?}",
            called.document
        );
    }
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    assert!(client.finish().success());
    fs::remove_dir_all(outside).unwrap();
}

// @mara implements VER-SURFACE-PARITY
#[test]
fn directory_chooses_the_project_on_every_call() {
    // The server starts in `main`; `worktree` is a copy of it, as a Git
    // worktree would be, and `other` is an unrelated project.
    let main = temp_dir("mcp-main");
    let worktree = temp_dir("mcp-worktree");
    let other = temp_dir("mcp-other");
    fixture(&main);
    fixture(&worktree);
    write_files(
        &other,
        &[(".zban/config.toml", "format_version = 1\nprefix = \"ot\"\n")],
    );
    fs::create_dir_all(worktree.join("src/deep")).unwrap();
    let main_before = snapshot(&main, &[]);
    let path = |dir: &Path| fs::canonicalize(dir).unwrap().to_str().unwrap().to_owned();

    let mut client = Client::start(&main, &[]);
    let in_worktree = path(&worktree.join("src/deep"));
    let created = client.call(
        "create_issue",
        json!({"directory": in_worktree, "title": "Only in the worktree", "blocked_by": [A]}),
    );
    assert!(!created.is_error, "{:?}", created.document);
    let id = created.document["id"].as_str().unwrap().to_owned();
    assert!(worktree.join(format!(".zban/{id}.md")).is_file());
    let updated = client.call(
        "update_issue",
        json!({"directory": in_worktree, "id": A, "status": "done"}),
    );
    assert!(
        updated.document["path"]
            .as_str()
            .unwrap()
            .starts_with(&path(&worktree))
    );
    assert!(worktree.join(".zban/archive/zb-aaaaaa.md").is_file());

    // Without `directory`, the server's own project, which is unchanged.
    let listed = client.call("list_issues", json!({"all": true}));
    assert!(!ids(&listed.document).contains(&id.as_str()));
    assert_eq!(snapshot(&main, &[]), main_before);
    let listed = client.call("list_issues", json!({"directory": path(&worktree)}));
    assert!(ids(&listed.document).contains(&id.as_str()));

    // Another project, with its own prefix.
    let created = client.call(
        "create_issue",
        json!({"directory": path(&other), "title": "Other"}),
    );
    assert!(created.document["id"].as_str().unwrap().starts_with("ot-"));
    let shown = client.call(
        "show_issues",
        json!({"directory": path(&other), "ids": [A]}),
    );
    assert_eq!(error_code(&shown), "not_found");
    assert_eq!(snapshot(&main, &[]), main_before);
    assert!(client.finish().success());

    // A server started elsewhere with `-C` uses that project by default.
    let outside = temp_dir_outside_projects("mcp-directory-option");
    let mut client = Client::start(&outside, &["-C", &path(&worktree)]);
    let listed = client.call("list_issues", json!({}));
    assert!(ids(&listed.document).contains(&id.as_str()));
    drop(client);
    fs::remove_dir_all(outside).unwrap();
}

// @mara implements VER-SURFACE-PARITY
#[test]
fn mcp_takes_no_json_option() {
    let root = temp_dir("mcp-json");
    let output = zban(&root, &["mcp", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let document: Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(document["error"]["code"], "usage");
}
