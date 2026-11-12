//! The little Markdown that zban reads and writes in issue bodies, which are
//! otherwise opaque text (`DES-ISSUE-FILE`).

/// Text as stored in a body: ending with exactly one LF, or empty
/// (`DES-ISSUE-FILE`).
pub(crate) fn stored(text: &str) -> String {
    let text = text.trim_end_matches(['\n', '\r']);
    if text.is_empty() {
        String::new()
    } else {
        format!("{text}\n")
    }
}

/// `text`, in stored form, appended to `body` after exactly one blank line
/// (`DES-ISSUE-FILE`).
pub(crate) fn joined(body: &str, text: &str) -> String {
    let text = stored(text);
    let kept = body.trim_end();
    if text.is_empty() {
        body.to_owned()
    } else if kept.is_empty() {
        text
    } else {
        format!("{kept}\n\n{text}")
    }
}

/// The lines of `body` that lie outside fenced code blocks, each with the
/// byte offset where it starts and without its line ending.
pub(crate) fn lines_outside_fences(body: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut fences = Fences::default();
    lines(body).filter(move |(_, line)| !fences.fenced(line))
}

/// Whether `text` ends inside a fenced code block that it does not close.
pub(crate) fn ends_in_fence(text: &str) -> bool {
    let mut fences = Fences::default();
    for (_, line) in lines(text) {
        fences.fenced(line);
    }
    fences.open.is_some()
}

/// The lines of `text` with their byte offsets, without line endings.
fn lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut start = 0;
    text.split_inclusive('\n').map(move |line| {
        let at = start;
        start += line.len();
        let line = line.strip_suffix('\n').unwrap_or(line);
        (at, line.strip_suffix('\r').unwrap_or(line))
    })
}

/// Fenced code blocks, followed line by line.
///
/// A fence is a line of at least three backticks or tildes, at any
/// indentation, so fences inside list items count too. A backtick fence's
/// info string cannot contain a backtick. The block ends at a line of at
/// least as many of the same character with nothing else after it, or at
/// the end of the text. The fence lines belong to the block.
#[derive(Default)]
struct Fences {
    /// The character and length of the open fence.
    open: Option<(u8, usize)>,
}

impl Fences {
    /// Whether `line`, the next line, belongs to a fenced code block.
    fn fenced(&mut self, line: &str) -> bool {
        match self.open {
            None => {
                self.open = fence(line).map(|(marker, len, _)| (marker, len));
                self.open.is_some()
            }
            Some((marker, len)) => {
                if let Some((closing, closing_len, info)) = fence(line)
                    && closing == marker
                    && closing_len >= len
                    && info.trim().is_empty()
                {
                    self.open = None;
                }
                true
            }
        }
    }
}

/// The level and the text of an ATX heading such as `## Handoff`: one to
/// six `#` after at most three spaces, then a space, a tab or the line end.
/// The text is trimmed. Setext headings are not recognised.
pub(crate) fn heading(line: &str) -> Option<(usize, &str)> {
    let indent = line.bytes().take_while(|&b| b == b' ').count();
    if indent > 3 {
        return None;
    }
    let text = &line[indent..];
    let level = text.bytes().take_while(|&b| b == b'#').count();
    let rest = &text[level..];
    let valid = (1..=6).contains(&level) && (rest.is_empty() || rest.starts_with([' ', '\t']));
    valid.then(|| (level, rest.trim()))
}

/// The fence character, the fence length and the text after the fence, when
/// `line` is a code fence.
fn fence(line: &str) -> Option<(u8, usize, &str)> {
    let text = line.trim_start_matches([' ', '\t']);
    let marker = *text.as_bytes().first()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let len = text.bytes().take_while(|&b| b == marker).count();
    let info = &text[len..];
    (len >= 3 && !(marker == b'`' && info.contains('`'))).then_some((marker, len, info))
}

/// Whether `line` is a task item and, if so, whether it is checked: a list
/// line starting with `- [ ]`, `* [ ]` or `+ [ ]`, or the same with `[x]` or
/// `[X]`, at any indentation (`DES-LIFECYCLE`).
pub(crate) fn task_item(line: &str) -> Option<bool> {
    let text = line.trim_start_matches([' ', '\t']);
    let rest = text.strip_prefix(['-', '*', '+'])?.strip_prefix(' ')?;
    match rest.get(..3)? {
        "[ ]" => Some(false),
        "[x]" | "[X]" => Some(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outside(body: &str) -> Vec<&str> {
        lines_outside_fences(body).map(|(_, line)| line).collect()
    }

    #[test]
    fn fenced_code_blocks_are_skipped() {
        let body = "a\n```\nin\n```\nb\n~~~~ rust\nin\n~~~\nstill in\n~~~~~\nc\n";
        assert_eq!(outside(body), ["a", "b", "c"]);
    }

    #[test]
    fn closing_fence_needs_the_same_character_and_enough_length() {
        let body = "````\n```\n~~~~\n```` info\nin\n````\nout\n";
        assert_eq!(outside(body), ["out"]);
    }

    #[test]
    fn indented_fences_and_line_endings() {
        let body = "- item\r\n  ```sh\r\n  - [ ] in\r\n  ```\r\nout\r\n\t~~~\n";
        assert_eq!(outside(body), ["- item", "out"]);
    }

    #[test]
    fn stored_text_ends_with_one_line_feed() {
        assert_eq!(stored("a"), "a\n");
        assert_eq!(stored("a\n\n\n"), "a\n");
        assert_eq!(stored("a\r\n"), "a\n");
        assert_eq!(stored("\n\n"), "");
        assert_eq!(stored(""), "");
    }

    #[test]
    fn joining_leaves_one_blank_line() {
        assert_eq!(joined("a\n", "b"), "a\n\nb\n");
        assert_eq!(joined("a", "b\n\n"), "a\n\nb\n");
        assert_eq!(joined("a\n\n\n", "b"), "a\n\nb\n");
        assert_eq!(joined("", "b"), "b\n");
        assert_eq!(joined("a\n\n", ""), "a\n\n");
    }

    #[test]
    fn unclosed_fences_are_detected() {
        assert!(!ends_in_fence("a\n```\nb\n```\n"));
        assert!(ends_in_fence("a\n```\nb\n"));
        assert!(ends_in_fence("~~~~\n~~~\n"));
        assert!(!ends_in_fence(""));
    }

    #[test]
    fn lines_carry_their_offsets() {
        let body = "a\r\n```\nin\n```\nbc\nd";
        let lines: Vec<_> = lines_outside_fences(body).collect();
        assert_eq!(lines, [(0, "a"), (14, "bc"), (17, "d")]);
    }

    #[test]
    fn atx_headings() {
        for (line, expected) in [
            ("# Title", Some((1, "Title"))),
            ("## Handoff (x)  ", Some((2, "Handoff (x)"))),
            ("   ##\tTabbed", Some((2, "Tabbed"))),
            ("##", Some((2, ""))),
            ("###### Six", Some((6, "Six"))),
            ("####### Seven", None),
            ("##Handoff", None),
            ("    ## Indented code", None),
            ("\t## Tab", None),
            ("- ## In a list", None),
            ("", None),
        ] {
            assert_eq!(heading(line), expected, "{line:?}");
        }
    }

    #[test]
    fn unclosed_fence_runs_to_the_end() {
        assert_eq!(outside("a\n```\nb\n"), ["a"]);
    }

    #[test]
    fn short_runs_and_backticks_in_the_info_string_are_not_fences() {
        let body = "``\n``` a`b\n~~~ a`b\n";
        assert_eq!(outside(body), ["``", "``` a`b"]);
    }

    #[test]
    fn task_items() {
        for (line, expected) in [
            ("- [ ] open", Some(false)),
            ("* [ ] open", Some(false)),
            ("+ [ ]", Some(false)),
            ("- [x] done", Some(true)),
            ("- [X] done", Some(true)),
            ("    * [x] nested", Some(true)),
            ("\t+ [ ] tab", Some(false)),
            ("- [ ]no space after", Some(false)),
            ("- [y] other", None),
            ("-[ ] no space", None),
            ("-  [ ] two spaces", None),
            ("1. [ ] ordered", None),
            ("> - [ ] quoted", None),
            ("[ ] bare", None),
            ("- [", None),
            ("- é", None),
            ("", None),
        ] {
            assert_eq!(task_item(line), expected, "{line:?}");
        }
    }
}
