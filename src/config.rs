//! The project configuration in `.zban/config.toml`.
//!
//! The file has exactly two keys, so zban reads it with a small parser for the
//! TOML it needs instead of a TOML crate (`ADR-DEPENDENCIES`). Anything outside
//! that subset is reported as invalid configuration.

use crate::id::Prefix;

/// The only `.zban/` format this version of zban reads and writes.
pub const FORMAT_VERSION: i64 = 1;

/// The parsed project configuration.
// @mara implements DES-PROJECT-LAYOUT
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub prefix: Prefix,
}

#[derive(Debug, PartialEq)]
enum Value {
    Integer(i64),
    String(String),
}

impl Config {
    /// Parses the configuration text. The error is a message without the path.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut format_version = None;
        let mut prefix = None;
        for (index, line) in text.split('\n').enumerate() {
            let Some((key, value)) =
                parse_line(line).map_err(|message| format!("line {}: {message}", index + 1))?
            else {
                continue;
            };
            let slot = match key.as_str() {
                "format_version" => &mut format_version,
                "prefix" => &mut prefix,
                other => return Err(format!("unknown key `{other}`")),
            };
            if slot.replace(value).is_some() {
                return Err(format!("duplicate key `{key}`"));
            }
        }
        match format_version {
            Some(Value::Integer(FORMAT_VERSION)) => {}
            Some(Value::Integer(other)) => {
                return Err(format!(
                    "unsupported format_version {other}; this zban reads format {FORMAT_VERSION}"
                ));
            }
            Some(Value::String(_)) => return Err("`format_version` must be an integer".into()),
            None => return Err("missing key `format_version`".into()),
        }
        let prefix = match prefix {
            Some(Value::String(prefix)) => Prefix::parse(&prefix).map_err(|error| error.message)?,
            Some(Value::Integer(_)) => return Err("`prefix` must be a string".into()),
            None => return Err("missing key `prefix`".into()),
        };
        Ok(Self { prefix })
    }

    /// The configuration file's content.
    pub fn render(&self) -> String {
        format!(
            "format_version = {FORMAT_VERSION}\nprefix = \"{}\"\n",
            self.prefix
        )
    }
}

/// Parses one line: blank, a comment, or `key = value` with an optional comment.
fn parse_line(line: &str) -> Result<Option<(String, Value)>, String> {
    let line = line.strip_suffix('\r').unwrap_or(line);
    let rest = trim_space(line);
    if rest.is_empty() || rest.starts_with('#') {
        return Ok(None);
    }
    if rest.starts_with('[') {
        return Err("tables are not allowed".into());
    }
    let (key, rest) = match rest.chars().next() {
        Some(quote @ ('"' | '\'')) => quoted(rest, quote)?,
        _ => {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
                .unwrap_or(rest.len());
            if end == 0 {
                return Err("expected a key".into());
            }
            (rest[..end].to_owned(), &rest[end..])
        }
    };
    let rest = trim_space(rest);
    if rest.starts_with('.') {
        return Err("dotted keys are not allowed".into());
    }
    let Some(rest) = rest.strip_prefix('=') else {
        return Err(format!("expected `=` after `{key}`"));
    };
    let rest = trim_space(rest);
    let (value, rest) = match rest.chars().next() {
        Some(quote @ ('"' | '\'')) => {
            let (text, rest) = quoted(rest, quote)?;
            (Value::String(text), rest)
        }
        _ => {
            let end = rest.find([' ', '\t', '#']).unwrap_or(rest.len());
            let token = &rest[..end];
            let value = integer(token).ok_or_else(|| format!("unsupported value `{token}`"))?;
            (Value::Integer(value), &rest[end..])
        }
    };
    let rest = trim_space(rest);
    if rest.is_empty() || rest.starts_with('#') {
        Ok(Some((key, value)))
    } else {
        Err(format!("unexpected `{rest}` after the value"))
    }
}

fn trim_space(text: &str) -> &str {
    text.trim_start_matches([' ', '\t'])
}

/// Reads a single-line basic or literal string starting at `text`.
fn quoted(text: &str, quote: char) -> Result<(String, &str), String> {
    if text.starts_with(&format!("{quote}{quote}{quote}")) {
        return Err("multi-line strings are not supported".into());
    }
    let inner = &text[1..];
    let end = inner
        .find(quote)
        .ok_or_else(|| "unterminated string".to_owned())?;
    let content = &inner[..end];
    if quote == '"' && content.contains('\\') {
        return Err("escape sequences are not supported".into());
    }
    Ok((content.to_owned(), &inner[end + 1..]))
}

/// Parses a TOML decimal integer, such as `1`, `+1` or `1_000`.
fn integer(token: &str) -> Option<i64> {
    let digits = token.strip_prefix(['+', '-']).unwrap_or(token);
    let well_formed = digits.starts_with(|c: char| c.is_ascii_digit())
        && digits.ends_with(|c: char| c.is_ascii_digit())
        && !digits.contains("__")
        && digits.bytes().all(|b| b.is_ascii_digit() || b == b'_')
        && !(digits.len() > 1 && digits.starts_with('0'));
    if !well_formed {
        return None;
    }
    token.replace('_', "").parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prefix(text: &str) -> Result<String, String> {
        Config::parse(text).map(|config| config.prefix.as_str().to_owned())
    }

    #[test]
    fn renders_and_parses_the_default() {
        let text = Config::default().render();
        assert_eq!(text, "format_version = 1\nprefix = \"zb\"\n");
        assert_eq!(prefix(&text).unwrap(), "zb");
    }

    #[test]
    fn accepts_hand_written_toml() {
        let text = "# zban\r\n\n  prefix='web'   # ours\r\nformat_version = +1\n";
        assert_eq!(prefix(text).unwrap(), "web");
        assert_eq!(
            prefix("\"prefix\" = \"a1\"\nformat_version=1").unwrap(),
            "a1"
        );
    }

    #[test]
    fn rejects_anything_else() {
        for (text, message) in [
            ("format_version = 1\n", "missing key `prefix`"),
            ("prefix = \"zb\"\n", "missing key `format_version`"),
            (
                "format_version = 2\nprefix = \"zb\"\n",
                "unsupported format_version 2",
            ),
            (
                "format_version = \"1\"\nprefix = \"zb\"\n",
                "must be an integer",
            ),
            ("format_version = 1\nprefix = 1\n", "must be a string"),
            ("format_version = 1\nprefix = \"Zb\"\n", "invalid prefix"),
            (
                "format_version = 1\nprefix = \"zb\"\ncolor = \"red\"\n",
                "unknown key `color`",
            ),
            (
                "format_version = 1\nprefix = \"zb\"\nprefix = \"zc\"\n",
                "duplicate key `prefix`",
            ),
            ("format_version = 1\n[ids]\nprefix = \"zb\"\n", "tables"),
            ("format_version = 1\nids.prefix = \"zb\"\n", "dotted keys"),
            ("format_version = 1\nprefix \"zb\"\n", "expected `=`"),
            ("format_version = 1\nprefix = \"zb\n", "unterminated string"),
            (
                "format_version = 1\nprefix = \"\"\"zb\"\"\"\n",
                "multi-line",
            ),
            (
                "format_version = 1\nprefix = \"z\\u0062\"\n",
                "escape sequences",
            ),
            ("format_version = 1\nprefix = \"zb\" x\n", "unexpected `x`"),
            (
                "format_version = 01\nprefix = \"zb\"\n",
                "unsupported value",
            ),
            (
                "format_version = 1.0\nprefix = \"zb\"\n",
                "unsupported value",
            ),
            (
                "format_version = true\nprefix = \"zb\"\n",
                "unsupported value",
            ),
            ("= 1\n", "expected a key"),
        ] {
            let error = prefix(text).unwrap_err();
            assert!(error.contains(message), "{text:?}: {error}");
        }
    }
}
