//! Relations between issues (`REQ-RELATIONS`) and readiness
//! (`DES-READINESS`), computed over one load of the project.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;

use petgraph::algo::{astar, tarjan_scc};
use petgraph::graphmap::DiGraphMap;

use crate::error::{Error, ErrorCode, Result};
use crate::id::Id;
use crate::issue::{Issue, IssueType, Problem, ProblemCode, Status};
use crate::store::IssueFile;

/// A readable issue whose ID names exactly one file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IssueRef<'a> {
    pub id: &'a Id,
    pub file: &'a IssueFile,
    pub issue: &'a Issue,
}

/// What an ID names in a loaded project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry<'a> {
    Readable(IssueRef<'a>),
    /// A file that cannot be read (`REQ-DAMAGED-DATA`).
    Unreadable(&'a IssueFile),
    /// The ID names a file in both `.zban/` and `.zban/archive/`.
    Ambiguous,
}

/// Why an issue is not ready (`DES-READINESS`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NotReady {
    /// The status is not `todo`.
    Status,
    /// The issue is an epic.
    Epic,
    /// This blocker is open, unknown or unreadable.
    BlockedBy(Id),
    /// This ancestor is closed.
    AncestorClosed(Id),
    /// This ancestor has an open blocker.
    AncestorBlocked(Id),
}

impl fmt::Display for NotReady {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status => f.write_str("status"),
            Self::Epic => f.write_str("epic"),
            Self::BlockedBy(id) => write!(f, "blocked_by:{id}"),
            Self::AncestorClosed(id) => write!(f, "ancestor_closed:{id}"),
            Self::AncestorBlocked(id) => write!(f, "ancestor_blocked:{id}"),
        }
    }
}

impl serde::Serialize for NotReady {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// The relations that other issues hold to one issue, each in listing order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Reverse<'a> {
    /// Issues whose `parent` is this one.
    pub children: Vec<IssueRef<'a>>,
    /// Issues whose `blocked_by` names this one.
    pub blocks: Vec<IssueRef<'a>>,
    /// Issues that store a `related` link to this one.
    pub related_from: Vec<IssueRef<'a>>,
    /// Issues whose `discovered_from` is this one.
    pub discovered: Vec<IssueRef<'a>>,
}

/// The issues of one project load, indexed by ID.
// @mara implements REQ-RELATIONS
#[derive(Debug)]
pub struct Graph<'a> {
    files: &'a [IssueFile],
    entries: HashMap<&'a Id, Entry<'a>>,
}

impl<'a> Graph<'a> {
    pub fn new(files: &'a [IssueFile]) -> Self {
        let mut entries = HashMap::new();
        for file in files {
            let Some(id) = &file.id else { continue };
            let entry = match &file.issue {
                Some(issue) => Entry::Readable(IssueRef { id, file, issue }),
                None => Entry::Unreadable(file),
            };
            entries
                .entry(id)
                .and_modify(|known| *known = Entry::Ambiguous)
                .or_insert(entry);
        }
        Self { files, entries }
    }

    /// What `id` names, or `None` for an unknown ID.
    pub fn get(&self, id: &Id) -> Option<Entry<'a>> {
        self.entries.get(id).copied()
    }

    /// The readable issue `id`. Fails with `not_found`, `ambiguous_id` or
    /// `unreadable` otherwise.
    pub fn resolve(&self, id: &Id) -> Result<IssueRef<'a>> {
        match self.get(id) {
            Some(Entry::Readable(found)) => Ok(found),
            Some(Entry::Unreadable(file)) => Err(file.unreadable()),
            Some(Entry::Ambiguous) => Err(Error::new(
                ErrorCode::AmbiguousId,
                format!(
                    "{id} names a file in both .zban/ and .zban/archive/; remove one and run `zban check`"
                ),
            )),
            None => Err(Error::new(
                ErrorCode::NotFound,
                format!("issue {id} does not exist"),
            )),
        }
    }

    /// Every readable issue with a unique ID, in listing order: priority,
    /// then `created_at`, then ID.
    pub fn issues(&self) -> Vec<IssueRef<'a>> {
        let mut issues: Vec<_> = self.readable().collect();
        issues.sort_by(listing_order);
        issues
    }

    fn readable(&self) -> impl Iterator<Item = IssueRef<'a>> {
        self.entries.values().filter_map(|entry| match entry {
            Entry::Readable(found) => Some(*found),
            _ => None,
        })
    }

    /// The issues that can start now, in listing order.
    pub fn ready(&self) -> Vec<IssueRef<'a>> {
        self.issues()
            .into_iter()
            .filter(|found| self.not_ready(found.id, found.issue).is_empty())
            .collect()
    }

    /// Every reason why issue `id`, in the state `issue`, is not ready. The
    /// issue is ready when there is none. Reasons come grouped by kind in the
    /// order of [`NotReady`]: blockers in `blocked_by` order, ancestors nearest
    /// first.
    // @mara implements DES-READINESS
    pub fn not_ready(&self, id: &Id, issue: &Issue) -> Vec<NotReady> {
        let mut reasons = Vec::new();
        if issue.status != Status::Todo {
            reasons.push(NotReady::Status);
        }
        if issue.issue_type == IssueType::Epic {
            reasons.push(NotReady::Epic);
        }
        reasons.extend(
            self.open_blockers(issue)
                .map(|blocker| NotReady::BlockedBy(blocker.clone())),
        );
        let ancestors = self.ancestors(id, issue);
        reasons.extend(
            ancestors
                .iter()
                .filter(|ancestor| ancestor.issue.status.is_closed())
                .map(|ancestor| NotReady::AncestorClosed(ancestor.id.clone())),
        );
        reasons.extend(
            ancestors
                .iter()
                .filter(|ancestor| self.open_blockers(ancestor.issue).next().is_some())
                .map(|ancestor| NotReady::AncestorBlocked(ancestor.id.clone())),
        );
        reasons
    }

    /// The blockers of `issue` that are not closed. Unknown and unreadable
    /// issues count as open.
    fn open_blockers<'b>(&'b self, issue: &'b Issue) -> impl Iterator<Item = &'b Id> {
        issue.blocked_by.iter().filter(|blocker| {
            !matches!(self.get(blocker), Some(Entry::Readable(found)) if found.issue.status.is_closed())
        })
    }

    /// The readable ancestors of issue `id`, nearest first. An unknown or
    /// unreadable parent ends the chain, and so does a repeated one, which
    /// only hand-edited data can contain.
    fn ancestors(&self, id: &Id, issue: &Issue) -> Vec<IssueRef<'a>> {
        let mut seen = HashSet::from([id]);
        let mut chain = Vec::new();
        let mut next = issue.parent.as_ref();
        while let Some(parent) = next {
            let Some(Entry::Readable(found)) = self.get(parent) else {
                break;
            };
            if !seen.insert(found.id) {
                break;
            }
            chain.push(found);
            next = found.issue.parent.as_ref();
        }
        chain
    }

    /// The issues that relate to issue `id`, read in the reverse direction.
    // @mara implements REQ-RELATIONS
    pub fn reverse(&self, id: &Id) -> Reverse<'a> {
        let mut reverse = Reverse::default();
        for found in self.issues() {
            let issue = found.issue;
            if issue.parent.as_ref() == Some(id) {
                reverse.children.push(found);
            }
            if issue.blocked_by.contains(id) {
                reverse.blocks.push(found);
            }
            if issue.related.contains(id) {
                reverse.related_from.push(found);
            }
            if issue.discovered_from.as_ref() == Some(id) {
                reverse.discovered.push(found);
            }
        }
        reverse
    }

    /// The other issues that name `id` in any relation, sorted by ID. An
    /// issue whose ID names two files counts too.
    pub fn referrers(&self, id: &Id) -> Vec<&'a Id> {
        let mut referrers: Vec<_> = self
            .files
            .iter()
            .filter_map(|file| {
                let (other, issue) = (file.id.as_ref()?, file.issue.as_ref()?);
                let refers = other != id && references(issue).any(|(_, target)| target == id);
                refers.then_some(other)
            })
            .collect();
        referrers.sort();
        referrers.dedup();
        referrers
    }

    /// Checks the relations that a create or update gives an issue
    /// (`REQ-RELATIONS`). `after` is the issue as the request leaves it,
    /// `before` the issue as loaded and `id` its ID; both are `None` for a new
    /// issue. Only what the request adds is checked, so problems already in
    /// the project do not block unrelated changes (`REQ-DAMAGED-DATA`).
    ///
    /// Fails with `invalid_reference` for a reference to an unknown issue or
    /// to the issue itself, `unreadable` or `ambiguous_id` for a reference to
    /// such an issue, and `cycle` for a cycle through `parent` and
    /// `blocked_by` or a `blocked_by` on an ancestor.
    // @mara implements REQ-RELATIONS
    pub fn check(&self, id: Option<&Id>, before: Option<&Issue>, after: &Issue) -> Result<()> {
        let added = |key: &str, target: &Id| {
            before.is_none_or(|before| !references(before).any(|old| old == (key, target)))
        };
        for (key, target) in references(after).filter(|&(key, target)| added(key, target)) {
            if Some(target) == id {
                return Err(Error::new(
                    ErrorCode::InvalidReference,
                    format!("`{key}` names {target}, the issue itself"),
                ));
            }
            if let Err(error) = self.resolve(target) {
                return Err(match error.code {
                    ErrorCode::NotFound => Error::new(
                        ErrorCode::InvalidReference,
                        format!("`{key}` names {target}, which does not exist"),
                    ),
                    _ => error,
                });
            }
        }
        if let Some(id) = id {
            self.check_cycles(id, after, |key, target| added(key, target))?;
        }
        self.check_blocked_by_ancestor(id, before, after)
    }

    /// Fails when an edge that issue `id` gains closes a cycle through
    /// `parent` and `blocked_by`. A new issue cannot be on a cycle, because
    /// nothing refers to it yet.
    fn check_cycles(
        &self,
        id: &Id,
        after: &Issue,
        added: impl Fn(&str, &Id) -> bool,
    ) -> Result<()> {
        let mut graph = DiGraphMap::<&Id, ()>::new();
        for found in self.readable() {
            if found.id != id {
                for (_, target) in edges(found.issue) {
                    graph.add_edge(found.id, target, ());
                }
            }
        }
        for (_, target) in edges(after) {
            graph.add_edge(id, target, ());
        }
        for (key, target) in edges(after).filter(|&(key, target)| added(key, target)) {
            let back = astar(&graph, target, |node| node == id, |_| 1, |_| 0);
            if let Some((_, path)) = back {
                let cycle: Vec<_> = std::iter::once(id).chain(path).map(Id::as_str).collect();
                return Err(Error::new(
                    ErrorCode::Cycle,
                    format!(
                        "`{key}` {target} would form a cycle through `parent` and `blocked_by`: {}",
                        cycle.join(" -> ")
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Fails when the request makes an issue blocked by one of its own
    /// ancestors: issue `id` itself through a new blocker or parent, or one of
    /// its descendants through a new parent of `id`.
    fn check_blocked_by_ancestor(
        &self,
        id: Option<&Id>,
        before: Option<&Issue>,
        after: &Issue,
    ) -> Result<()> {
        let Some(id) = id else {
            let chain = self.chain(None, after, None);
            return match after.blocked_by.iter().find(|b| chain.contains(b)) {
                Some(blocker) => Err(blocked_by_ancestor("the new issue", blocker)),
                None => Ok(()),
            };
        };
        let stored = |other: &Id| match self.get(other) {
            Some(Entry::Readable(found)) => Some(found.issue),
            _ => None,
        };
        let before = before.or_else(|| stored(id));
        // The ancestors of `id`'s descendants change only with its parent.
        let mut subjects = vec![id];
        if before.map(|before| &before.parent) != Some(&after.parent) {
            let mut children: HashMap<&Id, Vec<&Id>> = HashMap::new();
            for found in self.readable() {
                if let Some(parent) = &found.issue.parent {
                    children.entry(parent).or_default().push(found.id);
                }
            }
            let mut seen = HashSet::from([id]);
            let mut index = 0;
            while let Some(&subject) = subjects.get(index) {
                index += 1;
                let below = children.get(subject).into_iter().flatten();
                subjects.extend(below.filter(|&&child| seen.insert(child)));
            }
        }
        for subject in subjects {
            let (now, then) = if subject == id {
                (Some(after), before)
            } else {
                (stored(subject), stored(subject))
            };
            let Some(now) = now else { continue };
            let chain = self.chain(Some(subject), now, Some((id, after)));
            let old_chain = match (then, before) {
                (Some(then), Some(before)) => self.chain(Some(subject), then, Some((id, before))),
                _ => Vec::new(),
            };
            let was_blocked = |blocker: &Id| {
                then.is_some_and(|then| then.blocked_by.contains(blocker))
                    && old_chain.contains(&blocker)
            };
            if let Some(blocker) = now
                .blocked_by
                .iter()
                .find(|&blocker| chain.contains(&blocker) && !was_blocked(blocker))
            {
                return Err(blocked_by_ancestor(subject.as_str(), blocker));
            }
        }
        Ok(())
    }

    /// The IDs in the `parent` chain above `issue`, nearest first, with issue
    /// `replaced.0` taken in the state `replaced.1`. Unlike readiness, the
    /// chain includes an unknown or unreadable parent before it ends.
    fn chain<'b>(
        &'b self,
        subject: Option<&'b Id>,
        issue: &'b Issue,
        replaced: Option<(&'b Id, &'b Issue)>,
    ) -> Vec<&'b Id> {
        let mut seen: HashSet<&Id> = subject.into_iter().collect();
        let mut chain = Vec::new();
        let mut next = issue.parent.as_ref();
        while let Some(parent) = next {
            if !seen.insert(parent) {
                break;
            }
            chain.push(parent);
            next = match replaced {
                Some((id, state)) if id == parent => state.parent.as_ref(),
                _ => match self.get(parent) {
                    Some(Entry::Readable(found)) => found.issue.parent.as_ref(),
                    _ => None,
                },
            };
        }
        chain
    }

    /// Adds a `related` link from issue `id`, in the state `issue`, to
    /// `other`, unless either issue already stores it (`DES-ISSUE-FILE`).
    /// [`Graph::check`] then validates the new reference.
    pub fn add_related(&self, id: &Id, issue: &mut Issue, other: &Id) {
        if !issue.related.contains(other) && !self.stores_related(other, id) {
            issue.related.push(other.clone());
        }
    }

    /// Removes the `related` link between issue `id`, in the state `issue`,
    /// and `other` from whichever issue stores it. When `other` stores it,
    /// returns that issue and its state without the link, for the caller to
    /// write.
    pub fn remove_related(
        &self,
        id: &Id,
        issue: &mut Issue,
        other: &Id,
    ) -> Option<(IssueRef<'a>, Issue)> {
        issue.related.retain(|related| related != other);
        let Some(Entry::Readable(found)) = self.get(other) else {
            return None;
        };
        if !found.issue.related.contains(id) {
            return None;
        }
        let mut changed = found.issue.clone();
        changed.related.retain(|related| related != id);
        Some((found, changed))
    }

    fn stores_related(&self, id: &Id, other: &Id) -> bool {
        matches!(self.get(id), Some(Entry::Readable(found)) if found.issue.related.contains(other))
    }

    /// The relation findings of `DES-CHECK` across the whole project, each
    /// with the file it concerns: references to unknown issues or to the issue
    /// itself, blockers on an ancestor, cycles through `parent` and
    /// `blocked_by`, `related` links stored on both issues, and open issues
    /// under a closed parent. A reference to an unreadable or duplicated
    /// issue is not reported here, because that file has its own finding.
    // @mara implements DES-CHECK
    pub(crate) fn problems(&self) -> Vec<(&'a IssueFile, Problem)> {
        let mut problems = Vec::new();
        for file in self.files {
            let (Some(id), Some(issue)) = (&file.id, &file.issue) else {
                continue;
            };
            let mut add = |code, message| problems.push((file, Problem::new(code, message)));
            for (key, target) in references(issue) {
                if target == id {
                    let message = format!("`{key}` names {target}, the issue itself");
                    add(ProblemCode::InvalidReference, message);
                } else if self.get(target).is_none() {
                    let message = format!("`{key}` names {target}, which does not exist");
                    add(ProblemCode::InvalidReference, message);
                }
            }
            let chain = self.chain(Some(id), issue, None);
            for blocker in issue.blocked_by.iter().filter(|b| chain.contains(b)) {
                let message = format!(
                    "blocked by {blocker}, one of its own ancestors, which cannot close before it"
                );
                add(ProblemCode::Cycle, message);
            }
            // Reported once, on the issue with the later ID.
            for other in issue.related.iter().filter(|&other| other < id) {
                if self.stores_related(other, id) {
                    let message = format!(
                        "{other} stores the same `related` link to this issue; keep it on one of the two"
                    );
                    add(ProblemCode::RelatedBothSides, message);
                }
            }
            if let (false, Some(parent)) = (issue.status.is_closed(), &issue.parent)
                && let Some(Entry::Readable(found)) = self.get(parent)
                && found.issue.status.is_closed()
            {
                let message = format!(
                    "the parent {parent} is {}, but this issue is still {}",
                    found.issue.status, issue.status
                );
                add(ProblemCode::OpenUnderClosed, message);
            }
        }
        problems.extend(self.cycles());
        problems
    }

    /// A `cycle` finding for each issue on a cycle through `parent` and
    /// `blocked_by`, naming the shortest such cycle through it. A reference to
    /// the issue itself is an invalid reference instead.
    fn cycles(&self) -> Vec<(&'a IssueFile, Problem)> {
        // Built in listing order, so the cycle each message names is stable.
        let issues = self.issues();
        let mut graph = DiGraphMap::<&Id, ()>::new();
        for found in &issues {
            graph.add_node(found.id);
            for (_, target) in edges(found.issue).filter(|&(_, target)| target != found.id) {
                graph.add_edge(found.id, target, ());
            }
        }
        let mut problems = Vec::new();
        for component in tarjan_scc(&graph) {
            if component.len() < 2 {
                continue;
            }
            for &id in &component {
                // Unknown and unreadable issues have no edges, so every issue
                // on a cycle is readable.
                let Some(Entry::Readable(found)) = self.get(id) else {
                    continue;
                };
                let back = graph
                    .neighbors(id)
                    .filter(|next| component.contains(next))
                    .filter_map(|next| astar(&graph, next, |node| node == id, |_| 1, |_| 0))
                    .min_by_key(|(cost, _)| *cost);
                let Some((_, path)) = back else { continue };
                let cycle: Vec<_> = std::iter::once(id).chain(path).map(Id::as_str).collect();
                let message = format!(
                    "on a cycle through `parent` and `blocked_by`: {}",
                    cycle.join(" -> ")
                );
                problems.push((found.file, Problem::new(ProblemCode::Cycle, message)));
            }
        }
        problems
    }
}

fn listing_order(a: &IssueRef, b: &IssueRef) -> Ordering {
    (a.issue.priority, &a.issue.created_at, a.id).cmp(&(
        b.issue.priority,
        &b.issue.created_at,
        b.id,
    ))
}

/// The `parent` and `blocked_by` references of an issue, which readiness and
/// cycles follow.
fn edges(issue: &Issue) -> impl Iterator<Item = (&'static str, &Id)> {
    let parent = issue.parent.iter().map(|id| ("parent", id));
    let blocked_by = issue.blocked_by.iter().map(|id| ("blocked_by", id));
    parent.chain(blocked_by)
}

/// Every reference of an issue, with the key that holds it.
fn references(issue: &Issue) -> impl Iterator<Item = (&'static str, &Id)> {
    let related = issue.related.iter().map(|id| ("related", id));
    let discovered_from = issue
        .discovered_from
        .iter()
        .map(|id| ("discovered_from", id));
    edges(issue).chain(related).chain(discovered_from)
}

fn blocked_by_ancestor(subject: &str, blocker: &Id) -> Error {
    Error::new(
        ErrorCode::Cycle,
        format!(
            "{subject} would be blocked by {blocker}, one of its own ancestors, which cannot close before it"
        ),
    )
}
