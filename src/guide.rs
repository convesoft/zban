//! The agent guide (`DES-AGENT-GUIDE`): one template compiled into the
//! binary and rendered for one surface, naming the operations the way that
//! surface does: commands for the command line, tools for MCP
//! (`REQ-AGENT-GUIDE`).
//!
//! The template `guide.md` is Markdown with tags in double braces:
//!
//! - `{{update id=ID status=in-progress}}` calls an operation, rendered as
//!   `zban update ID --status in-progress` or as
//!   `update_issue {"id": ID, "status": "in-progress"}`. Uppercase values
//!   are placeholders and other values literals; a quoted literal keeps its
//!   spaces, and its quotes on the command line. A text parameter given a
//!   placeholder reads standard input on the command line (`--body -`).
//! - `{{synopsis update}}` gives the operation with all its parameters.
//! - `{{param force}}` names a parameter: `--force` or `force`.
//! - `{{cli}}`, `{{mcp}}` and `{{end}}` enclose text for one surface. A tag
//!   alone on its line removes the line.
//! - `{{types}}`, `{{statuses}}`, `{{priorities}}` and `{{not_ready}}` list
//!   the vocabulary of the code.
//!
//! Every operation and parameter comes from [`OPERATIONS`], one table for
//! both surfaces. Tests check the rendered guides against the command-line
//! definitions and the MCP tool schemas in both directions, so the table
//! cannot drift from either.

use std::fmt;

use crate::id::{Id, Prefix};
use crate::issue::{IssueType, Priority, Status};
use crate::relations::NotReady;

/// Where the guide is delivered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// `zban prime`: operations are commands with options.
    Cli,
    /// The MCP server instructions: operations are tools with parameters.
    Mcp,
}

/// The MCP server instructions when no project is found where the server
/// starts.
// @mara implements DES-AGENT-GUIDE
pub const NOTE: &str = "\
zban tracks issues as Markdown files in `.zban/` directories. No zban project
was found where this server started, so pass `directory`, the absolute path
of a directory inside the project, to every tool. Run `zban prime -C <dir>`
to read the full zban workflow guide for that project.
";

const TEMPLATE: &str = include_str!("guide.md");

/// The guide as `surface` delivers it.
// @mara implements DES-AGENT-GUIDE
pub fn render(surface: Surface) -> String {
    let mut guide = String::new();
    // The surface that the text being read is for, inside `{{cli}}` or
    // `{{mcp}}`.
    let mut only = None;
    let mut at = 0;
    while let Some(found) = TEMPLATE[at..].find("{{") {
        let start = at + found;
        let end = start + TEMPLATE[start..].find("}}").expect("tags are closed");
        let shown = only.is_none_or(|only| only == surface);
        if shown {
            guide.push_str(&TEMPLATE[at..start]);
        }
        at = end + 2;
        let tag = &TEMPLATE[start + 2..end];
        let block = match tag {
            "cli" => Some(Some(Surface::Cli)),
            "mcp" => Some(Some(Surface::Mcp)),
            "end" => Some(None),
            _ => None,
        };
        if let Some(block) = block {
            assert!(
                only.is_some() != block.is_some(),
                "unbalanced {{{{{tag}}}}}"
            );
            only = block;
            if TEMPLATE[..start].ends_with('\n') && TEMPLATE[at..].starts_with('\n') {
                at += 1;
            }
        } else {
            // Expanded on both surfaces, so that every tag is checked.
            let text = expand(tag, surface);
            if shown {
                guide.push_str(&text);
            }
        }
    }
    assert!(only.is_none(), "a block is not closed");
    guide.push_str(&TEMPLATE[at..]);
    guide
}

/// What a parameter takes.
#[derive(Debug, Clone, Copy)]
enum Kind {
    /// Nothing: an option without a value, or `true`.
    Flag,
    /// A value, shown as this placeholder.
    Value(&'static str),
    /// Text, which the command line also reads from standard input.
    Text,
}

/// A parameter of an operation.
#[derive(Debug, Clone, Copy)]
struct Param {
    /// The MCP parameter. The command-line option is `--` followed by the
    /// name with hyphens for underscores.
    name: &'static str,
    kind: Kind,
    /// Whether it takes several values: a repeatable option, or a list.
    many: bool,
}

const fn flag(name: &'static str) -> Param {
    Param {
        name,
        kind: Kind::Flag,
        many: false,
    }
}

const fn one(name: &'static str, placeholder: &'static str) -> Param {
    Param {
        name,
        kind: Kind::Value(placeholder),
        many: false,
    }
}

const fn many(name: &'static str, placeholder: &'static str) -> Param {
    Param {
        name,
        kind: Kind::Value(placeholder),
        many: true,
    }
}

const fn text(name: &'static str) -> Param {
    Param {
        name,
        kind: Kind::Text,
        many: false,
    }
}

/// One item of a synopsis.
#[derive(Debug)]
enum Part {
    /// A positional argument on the command line, a required parameter over
    /// MCP.
    Required(Param),
    Optional(Param),
    /// Two parameters that exclude each other.
    Either(Param, Param),
    /// Two lists whose values pair up by position.
    Pair(Param, Param),
}

/// An operation as the guide names it.
#[derive(Debug)]
struct Operation {
    /// The command, which also names the operation in the template.
    command: &'static str,
    tool: &'static str,
    parts: &'static [Part],
}

use Part::{Either, Optional, Pair, Required};

/// The operations that agents use, with every parameter of each.
const OPERATIONS: &[Operation] = &[
    Operation {
        command: "list",
        tool: "list_issues",
        parts: &[
            Optional(many("status", "S")),
            Optional(many("type", "T")),
            Optional(many("priority", "P")),
            Optional(many("tag", "T")),
            Optional(one("parent", "ID")),
            Optional(flag("ready")),
            Optional(one("search", "TEXT")),
            Optional(flag("all")),
        ],
    },
    Operation {
        command: "show",
        tool: "show_issues",
        parts: &[Required(many("ids", "ID"))],
    },
    Operation {
        command: "create",
        tool: "create_issue",
        parts: &[
            Required(one("title", "TITLE")),
            Optional(one("type", "T")),
            Optional(one("status", "S")),
            Optional(one("priority", "P")),
            Optional(many("tag", "T")),
            Optional(one("parent", "ID")),
            Optional(many("blocked_by", "ID")),
            Optional(many("related", "ID")),
            Optional(one("discovered_from", "ID")),
            Optional(text("body")),
        ],
    },
    Operation {
        command: "update",
        tool: "update_issue",
        parts: &[
            Required(one("id", "ID")),
            Optional(one("status", "S")),
            Optional(one("type", "T")),
            Optional(one("priority", "P")),
            Optional(one("title", "TEXT")),
            Optional(many("add_tag", "T")),
            Optional(many("remove_tag", "T")),
            Either(one("parent", "ID"), flag("no_parent")),
            Optional(many("add_blocked_by", "ID")),
            Optional(many("remove_blocked_by", "ID")),
            Optional(many("add_related", "ID")),
            Optional(many("remove_related", "ID")),
            Either(one("discovered_from", "ID"), flag("no_discovered_from")),
            Optional(text("body")),
            Optional(text("append")),
            Pair(many("replace_old", "TEXT"), many("replace_new", "TEXT")),
            Optional(flag("force")),
        ],
    },
    Operation {
        command: "delete",
        tool: "delete_issue",
        parts: &[Required(one("id", "ID"))],
    },
    Operation {
        command: "check",
        tool: "check_issues",
        parts: &[Optional(flag("fix"))],
    },
];

impl Operation {
    fn named(command: &str) -> &'static Self {
        OPERATIONS
            .iter()
            .find(|operation| operation.command == command)
            .unwrap_or_else(|| panic!("the guide names an unknown operation `{command}`"))
    }

    /// Every parameter, and whether it is required.
    fn params(&self) -> impl Iterator<Item = (Param, bool)> {
        self.parts.iter().flat_map(|part| match *part {
            Required(param) => vec![(param, true)],
            Optional(param) => vec![(param, false)],
            Either(a, b) | Pair(a, b) => vec![(a, false), (b, false)],
        })
    }

    fn param(&self, name: &str) -> (Param, bool) {
        self.params()
            .find(|(param, _)| param.name == name)
            .unwrap_or_else(|| panic!("`{}` has no parameter `{name}`", self.command))
    }
}

/// The text for the tag `{{tag}}`.
fn expand(tag: &str, surface: Surface) -> String {
    let (word, rest) = tag.split_once(' ').unwrap_or((tag, ""));
    match (word, rest) {
        ("types", "") => words(IssueType::ALL),
        ("statuses", "") => words(Status::ALL),
        ("priorities", "") => words(Priority::ALL),
        ("not_ready", "") => {
            let id = Id::from_bits(&Prefix::default(), 0);
            words(&[
                NotReady::Status,
                NotReady::Epic,
                NotReady::BlockedBy(id.clone()),
                NotReady::AncestorClosed(id.clone()),
                NotReady::AncestorBlocked(id.clone()),
            ])
            .replace(id.as_str(), "<id>")
        }
        ("synopsis", command) => synopsis(Operation::named(command), surface),
        ("param", name) => mention(name, surface),
        (command, arguments) => call(Operation::named(command), arguments, surface),
    }
}

/// `values` as code spans, separated by commas.
fn words<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The command-line option of `param`.
fn option(param: Param) -> String {
    format!("--{}", param.name.replace('_', "-"))
}

/// A parameter named in prose.
fn mention(name: &str, surface: Surface) -> String {
    let param = OPERATIONS
        .iter()
        .flat_map(Operation::params)
        .find(|(param, required)| param.name == name && !required)
        .unwrap_or_else(|| panic!("the guide names an unknown parameter `{name}`"))
        .0;
    match surface {
        Surface::Cli => option(param),
        Surface::Mcp => param.name.to_owned(),
    }
}

/// The operation with all its parameters, such as `zban show ID...` or
/// `show_issues {ids: [ID]}`.
fn synopsis(operation: &Operation, surface: Surface) -> String {
    match surface {
        Surface::Cli => {
            let usage = |param: Param| match param.kind {
                Kind::Flag => option(param),
                Kind::Value(placeholder) => format!("{} {placeholder}", option(param)),
                Kind::Text => format!("{} TEXT|-", option(param)),
            };
            let repeated = |param: Param| if param.many { "..." } else { "" };
            let mut text = format!("zban {}", operation.command);
            for part in operation.parts {
                let part = match *part {
                    Required(param) => format!("{}{}", placeholder(param), repeated(param)),
                    Optional(param) => format!("[{}]{}", usage(param), repeated(param)),
                    Either(a, b) => format!("[{}|{}]", usage(a), usage(b)),
                    Pair(a, b) => format!("[{} {}]{}", usage(a), usage(b), repeated(a)),
                };
                text.push(' ');
                text.push_str(&part);
            }
            text
        }
        Surface::Mcp => {
            let fields: Vec<_> = operation
                .params()
                .map(|(param, required)| {
                    let value = match param.kind {
                        Kind::Flag => "true",
                        Kind::Value(_) | Kind::Text => placeholder(param),
                    };
                    let value = if param.many {
                        format!("[{value}]")
                    } else {
                        value.to_owned()
                    };
                    let optional = if required { "" } else { "?" };
                    format!("{}{optional}: {value}", param.name)
                })
                .collect();
            format!("{} {{{}}}", operation.tool, fields.join(", "))
        }
    }
}

fn placeholder(param: Param) -> &'static str {
    match param.kind {
        Kind::Value(placeholder) => placeholder,
        Kind::Text => "TEXT",
        Kind::Flag => panic!("`{}` takes no value", param.name),
    }
}

/// A value given in a call.
enum Value<'a> {
    /// Uppercase, such as `ID`: the same on both surfaces.
    Placeholder(&'a str),
    Word(&'a str),
    Quoted(&'a str),
}

/// The arguments of a call such as `id=ID replace_old="- [ ] Step" force`,
/// in the order given.
fn arguments(mut text: &str) -> Vec<(&str, Option<Value<'_>>)> {
    let mut arguments = Vec::new();
    loop {
        text = text.trim_start();
        if text.is_empty() {
            return arguments;
        }
        let end = text.find(['=', ' ']).unwrap_or(text.len());
        let name = &text[..end];
        text = &text[end..];
        let value = text.strip_prefix('=').map(|rest| {
            if let Some(quoted) = rest.strip_prefix('"') {
                let close = quoted.find('"').expect("quotes are closed");
                text = &quoted[close + 1..];
                Value::Quoted(&quoted[..close])
            } else {
                let end = rest.find(' ').unwrap_or(rest.len());
                text = &rest[end..];
                let word = &rest[..end];
                if word.chars().all(|c| c.is_ascii_uppercase()) {
                    Value::Placeholder(word)
                } else {
                    Value::Word(word)
                }
            }
        });
        arguments.push((name, value));
    }
}

/// A call of the operation with `arguments`, such as
/// `zban update ID --status done` or `update_issue {"id": ID, "status": "done"}`.
fn call(operation: &Operation, arguments_text: &str, surface: Surface) -> String {
    let mut parts = Vec::new();
    for (name, value) in arguments(arguments_text) {
        let (param, required) = operation.param(name);
        let value = match (param.kind, value) {
            (Kind::Flag, None) => None,
            (Kind::Flag, Some(_)) => panic!("`{name}` takes no value"),
            (_, None) => panic!("`{name}` needs a value"),
            (_, Some(value)) => Some(value),
        };
        parts.push(match surface {
            Surface::Cli => {
                let value = value.map(|value| match value {
                    Value::Placeholder(_) if matches!(param.kind, Kind::Text) => "-".to_owned(),
                    Value::Placeholder(word) | Value::Word(word) => word.to_owned(),
                    Value::Quoted(text) => format!("\"{text}\""),
                });
                match (required, value) {
                    (true, Some(value)) => value,
                    (_, None) => option(param),
                    (false, Some(value)) => format!("{} {value}", option(param)),
                }
            }
            Surface::Mcp => {
                let value = match value {
                    None => "true".to_owned(),
                    Some(Value::Placeholder(word)) => word.to_owned(),
                    Some(Value::Word(text) | Value::Quoted(text)) => {
                        serde_json::to_string(text).expect("strings serialize")
                    }
                };
                let value = if param.many {
                    format!("[{value}]")
                } else {
                    value
                };
                format!("\"{name}\": {value}")
            }
        });
    }
    match surface {
        Surface::Cli => {
            let mut text = format!("zban {}", operation.command);
            for part in parts {
                text.push(' ');
                text.push_str(&part);
            }
            text
        }
        Surface::Mcp if parts.is_empty() => operation.tool.to_owned(),
        Surface::Mcp => format!("{} {{{}}}", operation.tool, parts.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tag_is_filled_on_both_surfaces() {
        for surface in [Surface::Cli, Surface::Mcp] {
            let guide = render(surface);
            assert!(!guide.contains("{{"), "{guide}");
            assert!(guide.contains("`blocked_by:<id>`, `ancestor_closed:<id>`"));
        }
    }

    #[test]
    fn calls_render_for_each_surface() {
        let both = |tag: &str| (expand(tag, Surface::Cli), expand(tag, Surface::Mcp));
        assert_eq!(
            both("update id=ID status=in-progress force"),
            (
                "zban update ID --status in-progress --force".to_owned(),
                r#"update_issue {"id": ID, "status": "in-progress", "force": true}"#.to_owned()
            )
        );
        assert_eq!(
            both(r#"create title="A \ title" body=TEXT"#),
            (
                r#"zban create "A \ title" --body -"#.to_owned(),
                r#"create_issue {"title": "A \\ title", "body": TEXT}"#.to_owned()
            )
        );
        assert_eq!(
            both("show ids=ID"),
            (
                "zban show ID".to_owned(),
                r#"show_issues {"ids": [ID]}"#.to_owned()
            )
        );
        assert_eq!(
            both("check"),
            ("zban check".to_owned(), "check_issues".to_owned())
        );
        assert_eq!(
            both("synopsis show"),
            (
                "zban show ID...".to_owned(),
                "show_issues {ids: [ID]}".to_owned()
            )
        );
        assert_eq!(
            both("param no_parent"),
            ("--no-parent".to_owned(), "no_parent".to_owned())
        );
    }

    #[test]
    #[should_panic(expected = "`list` has no parameter `id`")]
    fn unknown_parameters_are_rejected() {
        expand("list id=ID", Surface::Mcp);
    }
}
