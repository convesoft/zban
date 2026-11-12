//! The YAML layer of issue headers (`ADR-DEPENDENCIES`).
//!
//! `saphyr-parser` reads the header as events. This module accepts only the
//! plain subset of YAML that `DES-ISSUE-FILE` allows, turns the values into a
//! small tree, and splits the header text into one verbatim entry per top-level
//! key so that unknown keys can be written back unchanged. Writing never goes
//! through a generic serializer: [`scalar`] formats each string itself.

use std::collections::HashSet;
use std::fmt::Write as _;

use saphyr_parser::{Event, Marker, Parser, ScalarStyle, Span};

/// A header value. Mapping contents are not needed: only unknown keys may hold
/// mappings, and those are kept as text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Node {
    Scalar { text: String, plain: bool },
    Sequence(Vec<Node>),
    Mapping,
}

impl Node {
    /// Whether a YAML 1.2 core schema parser reads this value as null.
    pub(crate) fn is_null(&self) -> bool {
        matches!(self, Self::Scalar { text, plain: true } if is_null(text))
    }
}

/// One top-level key of a header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) key: String,
    pub(crate) value: Node,
    /// The entry's lines as written, from the key to the end of its value,
    /// with the mapping's own indentation removed. Ends with a newline.
    pub(crate) text: String,
}

/// Parses header text, the lines between the opening and closing `---`.
/// An empty header has no entries. The error describes why the header is
/// unparseable; its line numbers count from `first_line`, the file line on
/// which the header text starts.
pub(crate) fn parse_header(text: &str, first_line: usize) -> Result<Vec<Entry>, String> {
    let line_offset = first_line - 1;
    let events = Parser::new_from_str(text)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "invalid YAML on line {}: {}",
                error.marker().line() + line_offset,
                error.info()
            )
        })?;
    let mut reader = Reader {
        events: &events,
        position: 0,
        line_offset,
        content_end: Marker::new(0, 1, 0),
    };
    reader.expect(|event| matches!(event, Event::StreamStart))?;
    if matches!(reader.peek(), Event::StreamEnd) {
        return Ok(Vec::new());
    }
    reader.expect(|event| matches!(event, Event::DocumentStart(_)))?;
    let (event, span) = reader.next()?;
    let Event::MappingStart(anchor, tag) = event else {
        return Err("the header must be a YAML mapping".into());
    };
    reader.plain_node(span, *anchor, tag.is_some())?;
    if !is_block(span) {
        return Err("the header must be a block mapping, not a flow mapping".into());
    }
    let lines: Vec<&str> = text.split_terminator('\n').collect();
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    while !matches!(reader.peek(), Event::MappingEnd) {
        let (key_line, column) = reader.start()?;
        let key = reader.key(&mut seen)?;
        let value = reader.node()?;
        // The entry runs from its key to the line where its last content
        // ends; blank and comment lines after that belong to no entry.
        let end = reader.content_end;
        let last_line = if end.col() == 0 {
            end.line() - 1
        } else {
            end.line()
        };
        let entry_lines = lines
            .get(key_line - 1..last_line)
            .ok_or("each top-level key must start its own line")?;
        let mut text = String::new();
        for line in entry_lines {
            text.push_str(dedent(line, column));
            text.push('\n');
        }
        entries.push(Entry { key, value, text });
    }
    reader.next()?;
    reader.expect(|event| matches!(event, Event::DocumentEnd))?;
    if !matches!(reader.peek(), Event::StreamEnd) {
        return Err("the header must be a single YAML document".into());
    }
    Ok(entries)
}

struct Reader<'a, 'input> {
    events: &'a [(Event<'input>, Span)],
    position: usize,
    /// Added to parser line numbers to give file line numbers.
    line_offset: usize,
    /// Where the last scalar or flow collection read so far ends. Block
    /// collections have no closing characters, so their end events do not
    /// count.
    content_end: Marker,
}

impl<'a, 'input> Reader<'a, 'input> {
    fn peek(&self) -> &'a Event<'input> {
        // The parser always ends with `StreamEnd`, which no rule consumes.
        &self.events[self.position.min(self.events.len() - 1)].0
    }

    fn next(&mut self) -> Result<(&'a Event<'input>, &'a Span), String> {
        let (event, span) = self
            .events
            .get(self.position)
            .ok_or("unexpected end of YAML")?;
        self.position += 1;
        Ok((event, span))
    }

    /// The line (1-based) and column (0-based) where the next event starts.
    fn start(&self) -> Result<(usize, usize), String> {
        let (_, span) = self
            .events
            .get(self.position)
            .ok_or("unexpected end of YAML")?;
        Ok((span.start.line(), span.start.col()))
    }

    fn expect(&mut self, wanted: impl Fn(&Event<'input>) -> bool) -> Result<(), String> {
        let (event, _) = self.next()?;
        if wanted(event) {
            Ok(())
        } else {
            Err("the header must be a single YAML document".into())
        }
    }

    /// Reads a mapping key, which must be a scalar that the mapping has not
    /// used yet and is not a merge key.
    fn key(&mut self, seen: &mut HashSet<String>) -> Result<String, String> {
        let (line, _) = self.start()?;
        let line = line + self.line_offset;
        match self.node()? {
            Node::Scalar { text, plain } => {
                if plain && text == "<<" {
                    return Err(format!("merge keys are not allowed (line {line})"));
                }
                if !seen.insert(text.clone()) {
                    return Err(format!("duplicate key `{text}` (line {line})"));
                }
                Ok(text)
            }
            _ => Err(format!("mapping keys must be scalars (line {line})")),
        }
    }

    fn node(&mut self) -> Result<Node, String> {
        let (event, span) = self.next()?;
        match event {
            Event::Alias(_) => Err(format!(
                "aliases are not allowed (line {})",
                span.start.line() + self.line_offset
            )),
            Event::Scalar(text, style, anchor, tag) => {
                self.plain_node(span, *anchor, tag.is_some())?;
                self.content_end = span.end;
                Ok(Node::Scalar {
                    text: text.to_string(),
                    plain: *style == ScalarStyle::Plain,
                })
            }
            Event::SequenceStart(anchor, tag) => {
                self.plain_node(span, *anchor, tag.is_some())?;
                let mut items = Vec::new();
                while !matches!(self.peek(), Event::SequenceEnd) {
                    items.push(self.node()?);
                }
                self.end_collection()?;
                Ok(Node::Sequence(items))
            }
            Event::MappingStart(anchor, tag) => {
                self.plain_node(span, *anchor, tag.is_some())?;
                let mut seen = HashSet::new();
                while !matches!(self.peek(), Event::MappingEnd) {
                    self.key(&mut seen)?;
                    self.node()?;
                }
                self.end_collection()?;
                Ok(Node::Mapping)
            }
            _ => Err("unexpected YAML structure".into()),
        }
    }

    /// Consumes the end of a collection; a flow collection ends with its
    /// closing bracket.
    fn end_collection(&mut self) -> Result<(), String> {
        let (_, span) = self.next()?;
        if !is_block(span) {
            self.content_end = span.end;
        }
        Ok(())
    }

    /// Rejects anchors and tags, which the plain subset does not allow.
    fn plain_node(&self, span: &Span, anchor: usize, tagged: bool) -> Result<(), String> {
        let line = span.start.line() + self.line_offset;
        if anchor != 0 {
            Err(format!("anchors are not allowed (line {line})"))
        } else if tagged {
            Err(format!("tags are not allowed (line {line})"))
        } else {
            Ok(())
        }
    }
}

/// Block collections start with an empty span; flow collections span their
/// opening bracket.
fn is_block(span: &Span) -> bool {
    span.start.index() == span.end.index()
}

/// Removes up to `column` leading spaces.
fn dedent(line: &str, column: usize) -> &str {
    let spaces = line.bytes().take(column).take_while(|&b| b == b' ').count();
    &line[spaces..]
}

/// Formats a string as a YAML scalar: plain when a YAML 1.2 parser reads it
/// back as the same string, double-quoted otherwise.
// @mara implements DES-ISSUE-FILE
pub(crate) fn scalar(value: &str) -> String {
    if plain_round_trips(value) {
        value.to_owned()
    } else {
        double_quoted(value)
    }
}

/// Asks the parser whether `value`, written plain as a mapping value, reads
/// back as the same string under the YAML 1.2 core schema.
fn plain_round_trips(value: &str) -> bool {
    if is_core_non_string(value) || value.chars().any(needs_escape) {
        return false;
    }
    let document = format!("k: {value}\n");
    let Ok(events) = Parser::new_from_str(&document)
        .map(|result| result.map(|(event, _)| event))
        .collect::<Result<Vec<_>, _>>()
    else {
        return false;
    };
    matches!(
        events.as_slice(),
        [
            Event::StreamStart,
            Event::DocumentStart(false),
            Event::MappingStart(0, None),
            Event::Scalar(key, ScalarStyle::Plain, 0, None),
            Event::Scalar(read, ScalarStyle::Plain, 0, None),
            Event::MappingEnd,
            Event::DocumentEnd,
            Event::StreamEnd,
        ] if key == "k" && read == value
    )
}

fn double_quoted(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for c in value.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\t' => quoted.push_str("\\t"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\0' => quoted.push_str("\\0"),
            c if needs_escape(c) => {
                let code = u32::from(c);
                let _ = match code {
                    0..=0xff => write!(quoted, "\\x{code:02X}"),
                    0x100..=0xffff => write!(quoted, "\\u{code:04X}"),
                    _ => write!(quoted, "\\U{code:08X}"),
                };
            }
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// Characters that YAML does not allow unescaped in a scalar, plus line
/// breaks and the byte order mark.
fn needs_escape(c: char) -> bool {
    let printable = matches!(c,
        '\t' | '\u{20}'..='\u{7e}' | '\u{85}' | '\u{a0}'..='\u{d7ff}'
        | '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}');
    !printable || c == '\u{feff}'
}

fn is_null(text: &str) -> bool {
    matches!(text, "" | "~" | "null" | "Null" | "NULL")
}

/// Whether a plain scalar resolves to null, a boolean, an integer or a float
/// under the YAML 1.2 core schema.
fn is_core_non_string(text: &str) -> bool {
    let unsigned = text.strip_prefix(['-', '+']).unwrap_or(text);
    let digits = |s: &str, radix: u32| !s.is_empty() && s.chars().all(|c| c.is_digit(radix));
    is_null(text)
        || matches!(
            text,
            "true" | "True" | "TRUE" | "false" | "False" | "FALSE" | ".nan" | ".NaN" | ".NAN"
        )
        || matches!(unsigned, ".inf" | ".Inf" | ".INF")
        || digits(unsigned, 10)
        || text.strip_prefix("0o").is_some_and(|s| digits(s, 8))
        || text.strip_prefix("0x").is_some_and(|s| digits(s, 16))
        || is_core_float(unsigned)
}

/// Matches `( \. [0-9]+ | [0-9]+ ( \. [0-9]* )? ) ( [eE] [-+]? [0-9]+ )?`.
fn is_core_float(text: &str) -> bool {
    let (mantissa, exponent) = match text.find(['e', 'E']) {
        Some(at) => (&text[..at], Some(&text[at + 1..])),
        None => (text, None),
    };
    let all_digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    let mantissa_ok = match mantissa.split_once('.') {
        Some(("", fraction)) => !fraction.is_empty() && all_digits(fraction),
        Some((whole, fraction)) => all_digits(whole) && all_digits(fraction),
        None => !mantissa.is_empty() && all_digits(mantissa),
    };
    let exponent_ok = exponent.is_none_or(|exponent| {
        let digits = exponent.strip_prefix(['-', '+']).unwrap_or(exponent);
        !digits.is_empty() && all_digits(digits)
    });
    mantissa_ok && exponent_ok
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scalar_text(node: &Node) -> &str {
        match node {
            Node::Scalar { text, .. } => text,
            other => panic!("not a scalar: {other:?}"),
        }
    }

    #[test]
    fn splits_entries_and_keeps_their_text() {
        let header = "# leading comment\ntitle: 'Quoted' # trailing\nestimate:\n  low: 1\n# inside\n  high: 3\n\n# about tags\ntags: [a, b]\nnotes: |\n  text\n  # kept\n";
        let entries = parse_header(header, 2).unwrap();
        let keys: Vec<_> = entries.iter().map(|entry| entry.key.as_str()).collect();
        assert_eq!(keys, ["title", "estimate", "tags", "notes"]);
        assert_eq!(scalar_text(&entries[0].value), "Quoted");
        assert_eq!(entries[0].text, "title: 'Quoted' # trailing\n");
        assert_eq!(entries[1].value, Node::Mapping);
        assert_eq!(
            entries[1].text,
            "estimate:\n  low: 1\n# inside\n  high: 3\n"
        );
        assert_eq!(
            entries[2].value,
            Node::Sequence(vec![
                Node::Scalar {
                    text: "a".into(),
                    plain: true
                },
                Node::Scalar {
                    text: "b".into(),
                    plain: true
                },
            ])
        );
        assert_eq!(entries[3].text, "notes: |\n  text\n  # kept\n");
    }

    #[test]
    fn keeps_block_scalars_whole() {
        let header = "kept: |+\n  text\n\n\nfolded: >\n  a\n  # b\n\n# comment\nflow: [\n  a,\n  ]  # c\n  # d\n";
        let entries = parse_header(header, 2).unwrap();
        assert_eq!(
            entries[0].value,
            Node::Scalar {
                text: "text\n\n\n".into(),
                plain: false
            }
        );
        assert_eq!(entries[0].text, "kept: |+\n  text\n\n\n");
        assert_eq!(entries[1].text, "folded: >\n  a\n  # b\n\n");
        assert_eq!(entries[2].text, "flow: [\n  a,\n  ]  # c\n");
        let rewritten: String = entries.iter().map(|entry| entry.text.as_str()).collect();
        assert_eq!(parse_header(&rewritten, 2).unwrap(), entries);
    }

    #[test]
    fn removes_the_mapping_indentation() {
        let entries = parse_header("  title: a\n  extra:\n    - 1\n", 2).unwrap();
        assert_eq!(entries[1].text, "extra:\n  - 1\n");
    }

    #[test]
    fn errors_give_file_line_numbers() {
        let error = parse_header("title: a\nstatus: [b\n", 2).unwrap_err();
        assert!(error.starts_with("invalid YAML on line "), "{error}");
        let error = parse_header("title: a\nstatus: &x b\n", 2).unwrap_err();
        assert_eq!(error, "anchors are not allowed (line 3)");
        let error = parse_header("title: a\ntitle: b\n", 2).unwrap_err();
        assert_eq!(error, "duplicate key `title` (line 3)");
    }

    #[test]
    fn empty_header_has_no_entries() {
        assert!(parse_header("", 2).unwrap().is_empty());
        assert!(parse_header("# nothing\n", 2).unwrap().is_empty());
    }

    #[test]
    fn rejects_yaml_outside_the_plain_subset() {
        for (header, reason) in [
            ("title: &a x\n", "anchors"),
            ("title: x\nother: *a\n", "invalid YAML"),
            ("a: &x 1\nb: *x\n", "anchors"),
            ("title: !!str x\n", "tags"),
            ("title: !custom x\n", "tags"),
            ("tags: !!seq [a]\n", "tags"),
            ("<<: {a: 1}\n", "merge keys"),
            ("title: a\ntitle: b\n", "duplicate key `title`"),
            ("title: a\n'title': b\n", "duplicate key `title`"),
            ("extra:\n  a: 1\n  a: 2\n", "duplicate key `a`"),
            ("? [a]\n: b\n", "keys must be scalars"),
            ("{title: a}\n", "block mapping"),
            ("- a\n", "must be a YAML mapping"),
            ("just text\n", "must be a YAML mapping"),
            ("a: 1\n...\nb: 2\n", "single YAML document"),
            (
                "<<<<<<< HEAD\ntitle: a\n=======\ntitle: b\n>>>>>>> other\n",
                "",
            ),
            ("title: [a\n", "invalid YAML"),
        ] {
            let error = parse_header(header, 2).unwrap_err();
            assert!(error.contains(reason), "{header:?}: {error}");
        }
    }

    #[test]
    fn writes_plain_only_when_it_reads_back_the_same() {
        for plain in [
            "Parse issue files",
            "zb-n938cw",
            "2026-11-11T20:00:00Z",
            "yes",
            "no",
            "on",
            "-foo",
            "?x",
            ":x",
            "a:b",
            "a#b",
            "1.2.3",
            "0b101",
            "1_000",
            "C++ & Rust",
            "ünïcødé ✓",
            "a\tb",
            "back\\slash \"q\"",
        ] {
            assert_eq!(scalar(plain), plain);
        }
        for (value, quoted) in [
            ("", r#""""#),
            ("123", r#""123""#),
            ("-1", r#""-1""#),
            ("1.5", r#""1.5""#),
            ("1e3", r#""1e3""#),
            (".5", r#"".5""#),
            ("0x1F", r#""0x1F""#),
            ("0o17", r#""0o17""#),
            (".inf", r#"".inf""#),
            ("-.Inf", r#""-.Inf""#),
            (".NaN", r#"".NaN""#),
            ("true", r#""true""#),
            ("FALSE", r#""FALSE""#),
            ("null", r#""null""#),
            ("~", r#""~""#),
            ("a: b", r#""a: b""#),
            ("a #b", r#""a #b""#),
            ("trailing:", r#""trailing:""#),
            ("- item", r#""- item""#),
            ("[x]", r#""[x]""#),
            ("{x}", r#""{x}""#),
            ("'q'", r#""'q'""#),
            ("\"dq\"", r#""\"dq\"""#),
            ("#start", r##""#start""##),
            ("&anchor", r#""&anchor""#),
            ("*alias", r#""*alias""#),
            ("!tag", r#""!tag""#),
            ("|", r#""|""#),
            (">", r#"">""#),
            ("%x", r#""%x""#),
            ("@x", r#""@x""#),
            ("`x", r#""`x""#),
            (" padded", r#"" padded""#),
            ("padded ", r#""padded ""#),
            ("two\nlines", r#""two\nlines""#),
            ("cr\r", r#""cr\r""#),
            ("\"back\\slash\"", r#""\"back\\slash\"""#),
            ("bell\u{7}", r#""bell\x07""#),
            ("del\u{7f}", r#""del\x7F""#),
            ("bom\u{feff}", r#""bom\uFEFF""#),
            ("nul\0", r#""nul\0""#),
        ] {
            assert_eq!(scalar(value), quoted, "{value:?}");
        }
    }

    #[test]
    fn written_scalars_read_back_unchanged() {
        for value in [
            "123",
            "a: b",
            "a #b",
            "two\nlines",
            " padded ",
            "\u{1}\u{85}\u{2028}x",
            "yes",
            "- a",
            "\u{10ffff}",
            "x\u{fffe}",
        ] {
            let entries = parse_header(&format!("title: {}\n", scalar(value)), 2).unwrap();
            assert_eq!(scalar_text(&entries[0].value), value);
        }
    }

    #[test]
    fn core_schema_resolution() {
        for non_string in [
            "", "~", "null", "NULL", "true", "False", "0", "-12", "+3", "0o7", "0xFF", "1.", "1.5",
            ".5", "-1.5e-3", "1E10", ".inf", "+.INF", ".nan",
        ] {
            assert!(is_core_non_string(non_string), "{non_string:?}");
        }
        for string in [
            "yes",
            "nULL",
            "0o",
            "0x",
            "0xG",
            "1e",
            "e3",
            ".",
            "-",
            "1.2.3",
            "1_000",
            ".infinity",
            "+.nan",
            "TrUe",
        ] {
            assert!(!is_core_non_string(string), "{string:?}");
        }
    }
}
