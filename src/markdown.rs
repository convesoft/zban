//! The little Markdown that zban reads in issue bodies, which are otherwise
//! opaque text (`DES-ISSUE-FILE`).

/// The lines of `body` that lie outside fenced code blocks, without their
/// line endings.
///
/// A fence is a line of at least three backticks or tildes, at any
/// indentation, so fences inside list items count too. A backtick fence's
/// info string cannot contain a backtick. The block ends at a line of at
/// least as many of the same character with nothing else after it, or at
/// the end of the body. The fence lines belong to the block.
pub(crate) fn lines_outside_fences(body: &str) -> impl Iterator<Item = &str> {
    let mut open: Option<(u8, usize)> = None;
    body.lines().filter(move |line| match open {
        None => match fence(line) {
            Some((marker, len, _)) => {
                open = Some((marker, len));
                false
            }
            None => true,
        },
        Some((marker, len)) => {
            if let Some((closing, closing_len, info)) = fence(line)
                && closing == marker
                && closing_len >= len
                && info.trim().is_empty()
            {
                open = None;
            }
            false
        }
    })
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
        lines_outside_fences(body).collect()
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
