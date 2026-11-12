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

/// Fenced code blocks, followed line by line (`DES-LIFECYCLE`).
///
/// A fence line is indented by at most three columns more than its margin:
/// the content column of the innermost list item holding it, or column 0.
/// The fence lines belong to the block. A block ends at a closing fence
/// line or at the end of the text. Lines inside a block neither start nor
/// end list items, and list items end at the first line that is neither
/// blank nor indented to their content, lazy continuation lines included.
// @mara implements DES-LIFECYCLE
// @mara implements DES-HANDOFF
#[derive(Default)]
struct Fences {
    /// The content columns of the list items holding the current line,
    /// innermost last, so in increasing order.
    items: Vec<usize>,
    /// The character, the length and the margin of the open fence.
    open: Option<(u8, usize, usize)>,
}

impl Fences {
    /// Whether `line`, the next line, belongs to a fenced code block.
    fn fenced(&mut self, line: &str) -> bool {
        let (mut column, mut text) = indented(0, line);
        if let Some((marker, len, margin)) = self.open {
            if column <= margin + 3
                && let Some((closing, closing_len, info)) = fence(text)
                && closing == marker
                && closing_len >= len
                && info.trim().is_empty()
            {
                self.open = None;
            }
            return true;
        }
        if text.is_empty() {
            return false;
        }
        self.items.retain(|&content| content <= column);
        loop {
            let margin = self.items.last().copied().unwrap_or(0);
            if column > margin + 3 {
                return false;
            }
            if let Some((marker, len, _)) = fence(text) {
                self.open = Some((marker, len, margin));
                return true;
            }
            let Some((width, rest)) = list_marker(text) else {
                return false;
            };
            let after_marker = column + width;
            (column, text) = indented(after_marker, rest);
            let gap = column - after_marker;
            let content = if text.is_empty() || gap > 4 {
                after_marker + 1
            } else {
                column
            };
            self.items.push(content);
        }
    }
}

/// The column that `text`, starting at `column`, reaches after its leading
/// spaces and tabs, with a tab advancing to the next multiple of four, and
/// the text after them.
fn indented(mut column: usize, text: &str) -> (usize, &str) {
    let rest = text.trim_start_matches([' ', '\t']);
    for byte in text[..text.len() - rest.len()].bytes() {
        column = if byte == b'\t' {
            column / 4 * 4 + 4
        } else {
            column + 1
        };
    }
    (column, rest)
}

/// The width of the list item marker that `text` starts with, and the text
/// after it: `-`, `*`, `+`, or one to nine digits and `.` or `)`, followed
/// by a space, a tab or the end of the line.
fn list_marker(text: &str) -> Option<(usize, &str)> {
    let digits = text.bytes().take_while(u8::is_ascii_digit).count();
    let width = match text.as_bytes().get(digits)? {
        b'-' | b'*' | b'+' if digits == 0 => 1,
        b'.' | b')' if (1..=9).contains(&digits) => digits + 1,
        _ => return None,
    };
    let rest = &text[width..];
    (rest.is_empty() || rest.starts_with([' ', '\t'])).then_some((width, rest))
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
/// `text`, without its indentation, is a code fence: at least three
/// backticks or tildes, and no backtick after a backtick fence.
fn fence(text: &str) -> Option<(u8, usize, &str)> {
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
    fn fences_in_list_items_and_line_endings() {
        let body = "- item\r\n  ```sh\r\n  - [ ] in\r\n  ```\r\nout\r\n";
        assert_eq!(outside(body), ["- item", "out"]);
    }

    #[test]
    fn fences_indented_by_up_to_three_columns_open_blocks() {
        for indent in ["", " ", "  ", "   "] {
            let body = format!("a\n{indent}```\nin\n{indent}```\nb\n");
            assert_eq!(outside(&body), ["a", "b"], "{indent:?}");
        }
    }

    #[test]
    fn fences_indented_by_four_columns_outside_list_items_open_nothing() {
        for indent in ["    ", "\t", "  \t", "      "] {
            let body = format!("Prose\n{indent}```\n- [ ] after\n");
            assert_eq!(
                outside(&body),
                ["Prose", &format!("{indent}```"), "- [ ] after"]
            );
        }
    }

    #[test]
    fn fences_follow_the_content_of_their_list_item() {
        for (body, expected) in [
            // Nested items: the fence is indented to the inner content.
            (
                "- a\n  - b\n    ```\n    in\n    ```\nout\n",
                &["- a", "  - b", "out"][..],
            ),
            (
                "1. a\n   - b\n     ~~~\n     in\n     ~~~\n",
                &["1. a", "   - b"],
            ),
            ("10. a\n    ```\n    in\n    ```\n", &["10. a"]),
            (
                "- a\n\n  - b\n\n       ```\n       in\n       ```\n",
                &["- a", "", "  - b", ""],
            ),
            // Up to three columns past the content, in an item that a tab
            // or extra spaces after the marker indent further.
            ("-\ta\n\t   ```\n\tin\n\t```\n", &["-\ta"]),
            ("-   a\n       ```\n    ```\n", &["-   a"]),
            // A fence or a nested item right after the marker.
            ("- ```\n  - [ ] in\n  ```\nout\n", &["out"]),
            ("- 1. ```\n     in\n     ```\nout\n", &["out"]),
            // Four columns past the content open nothing.
            (
                "- a\n      ```\n- [ ] b\n",
                &["- a", "      ```", "- [ ] b"],
            ),
            ("-      ```\n- [ ] b\n", &["-      ```", "- [ ] b"]),
            // A line indented less than the content ends the item.
            (
                "- a\n\nb\n    ```\n- [ ] c\n",
                &["- a", "", "b", "    ```", "- [ ] c"],
            ),
            (
                "- a\n  - b\n  c\n      ```\n",
                &["- a", "  - b", "  c", "      ```"],
            ),
            // Indented too far, a marker starts no item.
            ("a\n    - b\n      ```\n", &["a", "    - b", "      ```"]),
            // Not list item markers.
            ("-a\n  ```\n", &["-a"]),
            ("1234567890. a\n    ```\n", &["1234567890. a", "    ```"]),
            ("2026-11-12\n    ```\n", &["2026-11-12", "    ```"]),
        ] {
            assert_eq!(outside(body), expected, "{body:?}");
        }
    }

    #[test]
    fn closing_fences_keep_the_margin_of_the_opening_one() {
        for (body, expected) in [
            // Inside a block, lines start and end no list items.
            ("- a\n  ```\nb\n  ```\n     ```\nin\n", &["- a"][..]),
            ("```\n- a\n```\n     ```\nout\n", &["     ```", "out"]),
            // A closing fence may be indented less than the opening one.
            (
                "- a\n  - b\n    ```\n    in\n```\nout\n",
                &["- a", "  - b", "out"],
            ),
            ("```\n    ```\nin\n   ```\nout\n", &["out"]),
        ] {
            assert_eq!(outside(body), expected, "{body:?}");
        }
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
