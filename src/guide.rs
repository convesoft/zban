//! The agent guide (`DES-AGENT-GUIDE`): one template compiled into the
//! binary, whose vocabulary lists come from the code (`REQ-AGENT-GUIDE`).

use std::fmt;

use crate::id::{Id, Prefix};
use crate::issue::{IssueType, Priority, Status};
use crate::relations::NotReady;

/// The guide, with `{{name}}` slots for the vocabulary lists.
const TEMPLATE: &str = include_str!("guide.md");

/// The guide as the command line delivers it.
// @mara implements DES-AGENT-GUIDE
pub fn render() -> String {
    let id = Id::from_bits(&Prefix::default(), 0);
    let reasons = [
        NotReady::Status,
        NotReady::Epic,
        NotReady::BlockedBy(id.clone()),
        NotReady::AncestorClosed(id.clone()),
        NotReady::AncestorBlocked(id.clone()),
    ]
    .map(|reason| reason.to_string().replace(id.as_str(), "<id>"));
    TEMPLATE
        .replace("{{types}}", &words(IssueType::ALL))
        .replace("{{statuses}}", &words(Status::ALL))
        .replace("{{priorities}}", &words(Priority::ALL))
        .replace("{{not_ready}}", &words(&reasons))
}

/// `values` as code spans, separated by commas.
fn words<T: fmt::Display>(values: &[T]) -> String {
    values
        .iter()
        .map(|value| format!("`{value}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slot_is_filled() {
        let guide = render();
        assert!(!guide.contains("{{"), "{guide}");
        assert!(guide.contains("`blocked_by:<id>`, `ancestor_closed:<id>`"));
    }
}
