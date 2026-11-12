//! The handoff note in an issue body (`DES-HANDOFF`): the section whose
//! heading line is `## Handoff` or `## Handoff (<timestamp>)`, running to the
//! next level 1 or 2 heading or to the end of the body. Headings inside
//! fenced code blocks do not count.

use std::ops::Range;

use serde::Serialize;

use crate::error::{Error, ErrorCode, Result};
use crate::markdown::{self, joined, stored};
use crate::timestamp::Timestamp;

/// A handoff note as `show` returns it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Handoff {
    /// The time in the heading, or `None` for a heading without one.
    pub written_at: Option<Timestamp>,
    /// The section after its heading, in stored form.
    pub text: String,
}

impl Handoff {
    /// The note in `body`: its first handoff section, if it has one.
    // @mara implements DES-HANDOFF
    pub fn of(body: &str) -> Option<Self> {
        let section = sections(body).into_iter().next()?;
        let text = &body[section.text.clone()];
        Some(Self {
            written_at: section.written_at,
            text: stored(text.trim_start_matches(['\r', '\n'])),
        })
    }
}

/// `body` with its handoff section replaced in place by a note holding
/// `text`, written at `now`, or with the note appended after a blank line.
// @mara implements DES-HANDOFF
pub(crate) fn write(body: &str, text: &str, now: &Timestamp) -> Result<String> {
    let text = stored(text);
    if text.is_empty() {
        return Err(Error::usage(
            "the handoff text must not be empty; remove the note instead",
        ));
    }
    if let Some((_, line)) = markdown::lines_outside_fences(&text).find(|(_, line)| ends(line)) {
        return Err(invalid(format!(
            "the handoff text contains the heading {line:?}, which would end the note early; \
             use headings of level 3 or lower"
        )));
    }
    if markdown::ends_in_fence(&text) {
        return Err(invalid(
            "the handoff text opens a fenced code block that it does not close, \
             which would carry the note past the next heading",
        ));
    }
    let note = format!("## Handoff ({now})\n\n{text}");
    Ok(match only(body)? {
        None => joined(body, &note),
        Some(old) if old.end == body.len() => format!("{}{note}", &body[..old.start]),
        Some(old) => format!("{}{note}\n{}", &body[..old.start], &body[old.end..]),
    })
}

/// `body` without its handoff section. When the section ends the body, the
/// blank lines before it go too, so the body ends as it did before the note
/// was appended.
// @mara implements DES-HANDOFF
pub(crate) fn remove(body: &str) -> Result<String> {
    Ok(match only(body)? {
        None => body.to_owned(),
        Some(old) if old.end == body.len() => stored(&body[..old.start]),
        Some(old) => format!("{}{}", &body[..old.start], &body[old.end..]),
    })
}

/// A handoff section of a body.
struct Section {
    /// The bytes of the section, from its heading line to the next level 1
    /// or 2 heading or the end of the body.
    range: Range<usize>,
    /// The bytes after the heading line's text.
    text: Range<usize>,
    written_at: Option<Timestamp>,
}

/// The handoff section of `body`, which must not have several.
fn only(body: &str) -> Result<Option<Range<usize>>> {
    let mut sections = sections(body).into_iter();
    let first = sections.next();
    if sections.next().is_some() {
        return Err(invalid(
            "the body has more than one handoff section; merge them by hand first",
        ));
    }
    Ok(first.map(|section| section.range))
}

/// Every handoff section of `body`, in order.
fn sections(body: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut open: Option<Section> = None;
    for (at, line) in markdown::lines_outside_fences(body) {
        if !ends(line) {
            continue;
        }
        if let Some(mut section) = open.take() {
            section.range.end = at;
            section.text.end = at;
            sections.push(section);
        }
        if let Some(written_at) = markdown::heading(line).and_then(|(_, text)| handoff(text)) {
            let text_start = at + line.len();
            open = Some(Section {
                range: at..body.len(),
                text: text_start..body.len(),
                written_at,
            });
        }
    }
    sections.extend(open);
    sections
}

/// Whether `line` is a level 1 or 2 heading, which ends a section.
fn ends(line: &str) -> bool {
    markdown::heading(line).is_some_and(|(level, _)| level <= 2)
}

/// The time of a heading text that names a handoff: `Some(None)` for
/// `Handoff`, and `Some(Some(time))` for `Handoff (<time>)`.
fn handoff(heading: &str) -> Option<Option<Timestamp>> {
    let rest = heading.strip_prefix("Handoff")?;
    if rest.is_empty() {
        return Some(None);
    }
    let time = rest.strip_prefix(" (")?.strip_suffix(')')?;
    Timestamp::parse(time).map(Some)
}

fn invalid(message: impl Into<String>) -> Error {
    Error::new(ErrorCode::InvalidHandoff, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: &str = "2026-11-12T18:40:00Z";

    fn now() -> Timestamp {
        Timestamp::parse(NOW).unwrap()
    }

    fn code(result: Result<String>) -> ErrorCode {
        result.unwrap_err().code
    }

    #[test]
    fn reads_the_first_section_up_to_the_next_heading() {
        let body = "Intro\n\n## Handoff (2026-11-12T18:00:00Z)\n\n- Next: parse\n\n\
                    ### Detail\n\n```\n## Not a heading\n```\n\n## Notes\n\nKept.\n";
        let handoff = Handoff::of(body).unwrap();
        assert_eq!(handoff.written_at.unwrap().as_str(), "2026-11-12T18:00:00Z");
        assert_eq!(
            handoff.text,
            "- Next: parse\n\n### Detail\n\n```\n## Not a heading\n```\n"
        );
        assert_eq!(Handoff::of("## Handoff\nNow.").unwrap().written_at, None);
        assert_eq!(Handoff::of("## Handoff\nNow.").unwrap().text, "Now.\n");
        assert_eq!(Handoff::of("## Handoff\n").unwrap().text, "");
    }

    #[test]
    fn only_handoff_headings_outside_fences_count() {
        for body in [
            "```\n## Handoff\n```\n",
            "### Handoff\n",
            "## Handoff notes\n",
            "## Handoff (yesterday)\n",
            "## Handoff (2026-11-12)\n",
            "##Handoff\n",
            "    ## Handoff\n",
        ] {
            assert_eq!(Handoff::of(body), None, "{body:?}");
        }
        assert!(Handoff::of("  ## Handoff  \n").is_some());
        assert!(Handoff::of("# Title\n## Handoff\r\nx\r\n").is_some());
    }

    #[test]
    fn writing_appends_or_replaces_in_place() {
        let written = write("Intro\n\n\n", "Next.\n\n", &now()).unwrap();
        assert_eq!(written, format!("Intro\n\n## Handoff ({NOW})\n\nNext.\n"));
        assert_eq!(
            write("", "Next.", &now()).unwrap(),
            format!("## Handoff ({NOW})\n\nNext.\n")
        );

        let body = "Intro\n\n## Handoff\nOld\n\n\n## Notes\n\nKept.\n";
        assert_eq!(
            write(body, "New", &now()).unwrap(),
            format!("Intro\n\n## Handoff ({NOW})\n\nNew\n\n## Notes\n\nKept.\n")
        );
        let body = "Intro\n\n## Handoff (2026-01-01T00:00:00Z)\n\nOld\n";
        assert_eq!(
            write(body, "New", &now()).unwrap(),
            format!("Intro\n\n## Handoff ({NOW})\n\nNew\n")
        );
    }

    #[test]
    fn removing_drops_the_section() {
        let body = "Intro\n\n## Handoff\nOld\n\n## Notes\n\nKept.\n";
        assert_eq!(remove(body).unwrap(), "Intro\n\n## Notes\n\nKept.\n");
        let body = "Intro\n\n## Handoff (2026-01-01T00:00:00Z)\n\nOld\n";
        assert_eq!(remove(body).unwrap(), "Intro\n");
        assert_eq!(remove("## Handoff\n\nOld\n").unwrap(), "");
        assert_eq!(remove("Intro\n").unwrap(), "Intro\n");
    }

    #[test]
    fn rejects_text_that_would_move_the_section_end() {
        for text in ["a\n## Next\n", "# Title\n", "a\n  ##\n", "```\n## Fenced\n"] {
            assert_eq!(
                code(write("", text, &now())),
                ErrorCode::InvalidHandoff,
                "{text:?}"
            );
        }
        let fenced = "```\n## Fenced\n```\n### Detail\n";
        assert!(write("", fenced, &now()).is_ok());
        assert_eq!(code(write("", "\n\n", &now())), ErrorCode::Usage);
    }

    #[test]
    fn several_sections_make_both_edits_fail() {
        let body = "## Handoff\n\nOne\n\n## Handoff (2026-01-01T00:00:00Z)\n\nTwo\n";
        assert_eq!(
            code(write(body, "Three", &now())),
            ErrorCode::InvalidHandoff
        );
        assert_eq!(code(remove(body)), ErrorCode::InvalidHandoff);
        assert_eq!(Handoff::of(body).unwrap().text, "One\n");
    }
}
