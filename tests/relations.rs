//! `VER-RELATIONS-READY`: relations and readiness through the core library.

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use common::{temp_dir, write_files};
use tempfile::TempDir;
use zban::operations::{self, ListRequest};
use zban::{ErrorCode, Graph, Id, Issue, IssueFile, IssueRef, Project, Status, Timestamp};

const CONFIG: &str = "format_version = 1\nprefix = \"zb\"\n";

// Fixture IDs: one letter repeated, named after the issue's role.
const A: &str = "zb-aaaaaa"; // blocks B
const B: &str = "zb-bbbbbb"; // blocked by A
const C: &str = "zb-cccccc"; // independent, high priority, stores `related: A`
const D: &str = "zb-dddddd"; // child of the blocked B
const E: &str = "zb-eeeeee"; // epic
const E1: &str = "zb-eeeee1"; // child of the epic E
const F: &str = "zb-ffffff"; // draft
const T: &str = "zb-tttttt"; // todo child of the draft F
const G: &str = "zb-gggggg"; // task under epic H
const H: &str = "zb-hhhhhh"; // epic inside epic K
const K: &str = "zb-kkkkkk"; // epic blocked by the open epic J
const J: &str = "zb-jjjjjj"; // open epic
const R: &str = "zb-rrrrrr"; // critical, discovered from A
const M: &str = "zb-mmmmmm"; // low, same age as N
const N: &str = "zb-nnnnnn"; // low, same age as M
const W: &str = "zb-wwwwww"; // in progress
const Z: &str = "zb-zzzzzz"; // done, archived
const Q: &str = "zb-qqqqqq"; // child of the closed Z
const Y: &str = "zb-yyyyyy"; // unreadable file
const V: &str = "zb-vvvvvv"; // blocked by the unreadable Y
const S: &str = "zb-ssssss"; // child of the unreadable Y
const P: &str = "zb-pppppp"; // blocked by an unknown issue
const UNKNOWN: &str = "zb-000000";

/// Header text of a hand-written issue created `second` seconds into 2026.
fn issue(title: &str, status: &str, kind: &str, relations: &str, second: u32) -> String {
    format!(
        "---\ntitle: {title}\nstatus: {status}\ntype: {kind}\n{relations}created_at: 2026-01-01T00:00:{second:02}Z\n---\n"
    )
}

fn fixture() -> (TempDir, Project) {
    let dir = temp_dir();
    let root = dir.path();
    let open = |id: &str| format!(".zban/{id}.md");
    let files = [
        (open(A), issue("A", "todo", "task", "", 1)),
        (
            open(B),
            issue("B", "todo", "task", &format!("blocked_by:\n  - {A}\n"), 2),
        ),
        (
            open(C),
            issue(
                "C",
                "todo",
                "task",
                &format!("priority: high\nrelated:\n  - {A}\n"),
                3,
            ),
        ),
        (
            open(D),
            issue("D", "todo", "task", &format!("parent: {B}\n"), 4),
        ),
        (open(E), issue("E", "todo", "epic", "", 5)),
        (
            open(E1),
            issue("E1", "todo", "task", &format!("parent: {E}\n"), 5),
        ),
        (open(F), issue("F", "draft", "feature", "", 6)),
        (
            open(G),
            issue("G", "todo", "task", &format!("parent: {H}\n"), 7),
        ),
        (
            open(H),
            issue("H", "todo", "epic", &format!("parent: {K}\n"), 8),
        ),
        (
            open(K),
            issue("K", "todo", "epic", &format!("blocked_by:\n  - {J}\n"), 9),
        ),
        (open(J), issue("J", "in-progress", "epic", "", 10)),
        (
            open(T),
            issue("T", "todo", "task", &format!("parent: {F}\n"), 11),
        ),
        (
            open(R),
            issue(
                "R",
                "todo",
                "bug",
                &format!("priority: critical\ndiscovered_from: {A}\n"),
                12,
            ),
        ),
        (open(M), issue("M", "todo", "task", "priority: low\n", 13)),
        (open(N), issue("N", "todo", "task", "priority: low\n", 13)),
        (
            open(Q),
            issue("Q", "todo", "task", &format!("parent: {Z}\n"), 14),
        ),
        (
            open(S),
            issue("S", "todo", "task", &format!("parent: {Y}\n"), 15),
        ),
        (
            open(V),
            issue("V", "todo", "task", &format!("blocked_by:\n  - {Y}\n"), 16),
        ),
        (
            open(P),
            issue(
                "P",
                "todo",
                "task",
                &format!("blocked_by:\n  - {UNKNOWN}\n"),
                17,
            ),
        ),
        (open(W), issue("W", "in-progress", "task", "", 18)),
        (
            format!(".zban/archive/{Z}.md"),
            "---\ntitle: Z\nstatus: done\ntype: feature\ncreated_at: 2026-01-01T00:00:19Z\nclosed_at: 2026-01-02T00:00:00Z\n---\n".to_owned(),
        ),
        (
            open(Y),
            "---\n<<<<<<< HEAD\ntitle: Ours\n=======\ntitle: Theirs\n>>>>>>> branch\nstatus: todo\ntype: task\ncreated_at: 2026-01-01T00:00:20Z\n---\n".to_owned(),
        ),
    ];
    write_files(root, &[(".zban/config.toml", CONFIG)]);
    let files: Vec<_> = files
        .iter()
        .map(|(path, text)| (path.as_str(), text.as_str()))
        .collect();
    write_files(root, &files);
    let project = Project::open(root).unwrap();
    (dir, project)
}

fn id(value: &str) -> Id {
    Id::parse(&Default::default(), value).unwrap()
}

fn ids(issues: &[IssueRef]) -> Vec<String> {
    issues.iter().map(|found| found.id.to_string()).collect()
}

/// The issues that `list --ready` returns, in its order.
fn ready(project: &Project) -> Vec<String> {
    let request = ListRequest {
        ready: true,
        ..Default::default()
    };
    let listed = operations::list(project, &request).unwrap();
    listed
        .issues
        .iter()
        .map(|listed| listed.issue.id.to_string())
        .collect()
}

/// The reasons of every readable issue that is not ready.
fn reasons(project: &Project) -> BTreeMap<String, Vec<String>> {
    let files = project.load().unwrap();
    let graph = Graph::new(&files);
    graph
        .issues()
        .into_iter()
        .map(|found| {
            let reasons = graph.not_ready(found.id, found.issue);
            (
                found.id.to_string(),
                reasons.iter().map(ToString::to_string).collect::<Vec<_>>(),
            )
        })
        .filter(|(_, reasons)| !reasons.is_empty())
        .collect()
}

fn load(project: &Project, value: &str) -> IssueFile {
    project
        .load()
        .unwrap()
        .into_iter()
        .find(|file| file.id.as_ref().map(Id::as_str) == Some(value))
        .unwrap_or_else(|| panic!("{value} is loaded"))
}

/// Saves issue `value` changed by `change` as an update does, following
/// `DES-LIFECYCLE`: a closed issue moves to `.zban/archive/`.
fn save(project: &Project, value: &str, force: bool, change: impl FnOnce(&mut Issue)) -> IssueFile {
    let files = project.load().unwrap();
    let file = files
        .iter()
        .find(|file| file.id.as_ref().map(Id::as_str) == Some(value))
        .unwrap_or_else(|| panic!("{value} is loaded"));
    let mut issue = file.issue.clone().unwrap();
    change(&mut issue);
    project.save(&files, file, issue, force).unwrap()
}

fn update(project: &Project, value: &str, change: impl FnOnce(&mut Issue)) {
    save(project, value, false, change);
}

fn close(project: &Project, value: &str) {
    let closed = save(project, value, false, |issue| issue.status = Status::Done);
    assert!(closed.archived, "{value} moved to the archive");
}

/// Checks the change of issue `value` that `change` makes, as an update
/// would before writing.
fn check_update(
    project: &Project,
    value: &str,
    change: impl FnOnce(&mut Issue),
) -> zban::Result<()> {
    let files = project.load().unwrap();
    let graph = Graph::new(&files);
    let found = graph.resolve(&id(value)).unwrap();
    let mut issue = found.issue.clone();
    change(&mut issue);
    graph.check(Some(found.id), Some(found.issue), &issue)
}

/// Makes `issue` name `target` in the relation `key`.
fn refer(issue: &mut Issue, key: &str, target: Id) {
    match key {
        "parent" => issue.parent = Some(target),
        "blocked_by" => issue.blocked_by.push(target),
        "related" => issue.related.push(target),
        "discovered_from" => issue.discovered_from = Some(target),
        _ => unreachable!("{key} is not a relation"),
    }
}

fn snapshot(project: &Project) -> Vec<(String, Vec<u8>)> {
    project
        .load()
        .unwrap()
        .into_iter()
        .map(|file| (file.path.display().to_string(), file.bytes))
        .collect()
}

// @mara implements VER-RELATIONS-READY
#[test]
fn ready_list_is_ordered_by_priority_age_and_id() {
    let (_dir, project) = fixture();
    // R is critical, C high; A, E1, T and S normal by age; M and N low and
    // the same age, so by ID. E1's parent is an epic and T's a draft, which
    // hold nothing back. S's parent is unreadable, which ends its chain.
    assert_eq!(ready(&project), [R, C, A, E1, T, S, M, N]);
}

// @mara implements VER-RELATIONS-READY
#[test]
fn each_issue_that_is_not_ready_reports_every_reason() {
    let (_dir, project) = fixture();
    let expected: BTreeMap<String, Vec<String>> = [
        (B, vec![format!("blocked_by:{A}")]),
        (D, vec![format!("ancestor_blocked:{B}")]),
        (E, vec!["epic".to_owned()]),
        (F, vec!["status".to_owned()]),
        (G, vec![format!("ancestor_blocked:{K}")]),
        (H, vec!["epic".to_owned(), format!("ancestor_blocked:{K}")]),
        (J, vec!["status".to_owned(), "epic".to_owned()]),
        (K, vec!["epic".to_owned(), format!("blocked_by:{J}")]),
        (P, vec![format!("blocked_by:{UNKNOWN}")]),
        (Q, vec![format!("ancestor_closed:{Z}")]),
        (V, vec![format!("blocked_by:{Y}")]),
        (W, vec!["status".to_owned()]),
        (Z, vec!["status".to_owned()]),
    ]
    .into_iter()
    .map(|(id, reasons)| (id.to_owned(), reasons))
    .collect();
    assert_eq!(reasons(&project), expected);

    // A closed ancestor that is also blocked reports both, and the walk
    // goes on above it. Cancelling H with its child G open needs force.
    let cancelled = save(&project, H, true, |issue| issue.status = Status::Cancelled);
    assert!(cancelled.archived);
    assert_eq!(
        reasons(&project)[G],
        [
            format!("ancestor_closed:{H}"),
            format!("ancestor_blocked:{K}")
        ]
    );
}

// @mara implements VER-RELATIONS-READY
#[test]
fn closing_blockers_makes_dependents_ready() {
    let (_dir, project) = fixture();
    close(&project, A);
    // B is no longer blocked, and neither is its child D.
    assert_eq!(ready(&project), [R, C, B, D, E1, T, S, M, N]);
    assert!(!reasons(&project).contains_key(B));

    close(&project, J);
    // K is no longer blocked, so G, two levels below it, becomes ready.
    // The epics H and K stay out of the list.
    assert_eq!(ready(&project), [R, C, B, D, E1, G, T, S, M, N]);
    assert_eq!(reasons(&project)[H], ["epic"]);
    assert_eq!(reasons(&project)[K], ["epic"]);
}

// @mara implements VER-RELATIONS-READY
#[test]
fn show_lists_reverse_relations() {
    let (_dir, project) = fixture();
    let files = project.load().unwrap();
    let graph = Graph::new(&files);

    let a = graph.reverse(&id(A));
    assert_eq!(ids(&a.children), [] as [&str; 0]);
    assert_eq!(ids(&a.blocks), [B]);
    assert_eq!(ids(&a.related_from), [C]);
    assert_eq!(ids(&a.discovered), [R]);

    assert_eq!(ids(&graph.reverse(&id(B)).children), [D]);
    assert_eq!(ids(&graph.reverse(&id(K)).children), [H]);
    assert_eq!(ids(&graph.reverse(&id(J)).blocks), [K]);
    // Archived issues take part in relations.
    assert_eq!(ids(&graph.reverse(&id(Z)).children), [Q]);
    // Children are listed in listing order.
    update(&project, M, |issue| issue.parent = Some(id(F)));
    let files = project.load().unwrap();
    assert_eq!(ids(&Graph::new(&files).reverse(&id(F)).children), [T, M]);
}

// @mara implements VER-RELATIONS-READY
#[test]
fn related_links_are_stored_once() {
    let (_dir, project) = fixture();
    let before = snapshot(&project);

    // C stores `related: A`. Adding the link from A's side changes nothing,
    // so an update writes no file.
    let files = project.load().unwrap();
    let graph = Graph::new(&files);
    let a = graph.resolve(&id(A)).unwrap();
    let mut changed = a.issue.clone();
    graph.add_related(a.id, &mut changed, &id(C));
    assert_eq!(&changed, a.issue);
    graph.check(Some(a.id), Some(a.issue), &changed).unwrap();
    assert_eq!(snapshot(&project), before);

    // Adding a new link stores it on the issue being updated.
    let mut changed = a.issue.clone();
    graph.add_related(a.id, &mut changed, &id(M));
    assert_eq!(changed.related, [id(M)]);

    // Removing it from A's side edits C, which stores it.
    let mut changed = a.issue.clone();
    let (other, other_issue) = graph.remove_related(a.id, &mut changed, &id(C)).unwrap();
    assert_eq!(&changed, a.issue);
    assert_eq!(other.id, &id(C));
    project
        .save(&files, other.file, other_issue, false)
        .unwrap();
    assert!(load(&project, C).issue.unwrap().related.is_empty());
    assert_eq!(load(&project, A).bytes, a.file.bytes);

    // Removing it from the side that stores it edits that issue only.
    update(&project, C, |issue| issue.related.push(id(A)));
    let files = project.load().unwrap();
    let graph = Graph::new(&files);
    let c = graph.resolve(&id(C)).unwrap();
    let mut changed = c.issue.clone();
    assert_eq!(graph.remove_related(c.id, &mut changed, &id(A)), None);
    assert!(changed.related.is_empty());

    // A link stored on both issues is removed from both.
    update(&project, A, |issue| issue.related.push(id(C)));
    let files = project.load().unwrap();
    let graph = Graph::new(&files);
    let a = graph.resolve(&id(A)).unwrap();
    let mut changed = a.issue.clone();
    let (other, other_issue) = graph.remove_related(a.id, &mut changed, &id(C)).unwrap();
    assert!(changed.related.is_empty());
    assert_eq!(other.id, &id(C));
    assert!(other_issue.related.is_empty());
}

// @mara implements VER-RELATIONS-READY
#[test]
fn unknown_and_self_references_fail_with_invalid_reference() {
    let (_dir, project) = fixture();
    let unknown = id(UNKNOWN);
    // Each relation key, naming an unknown issue and then C itself.
    for key in ["parent", "blocked_by", "related", "discovered_from"] {
        for target in [UNKNOWN, C] {
            let error =
                check_update(&project, C, |issue| refer(issue, key, id(target))).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidReference, "{key}: {error}");
            assert!(error.message.contains(key), "{error}");
        }
    }

    // Create checks the same before writing anything.
    let before = snapshot(&project);
    let mut new = Issue::new("New", Timestamp::now());
    new.blocked_by.push(unknown.clone());
    let error = project.create(&new).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidReference);
    assert!(error.message.contains(UNKNOWN), "{error}");
    assert_eq!(snapshot(&project), before);

    // Naming an unreadable issue fails with `unreadable`.
    let error = check_update(&project, C, |issue| issue.related.push(id(Y))).unwrap_err();
    assert_eq!(error.code, ErrorCode::Unreadable);
    let mut new = Issue::new("New", Timestamp::now());
    new.parent = Some(id(Y));
    assert_eq!(
        project.create(&new).unwrap_err().code,
        ErrorCode::Unreadable
    );

    // References that are already broken do not block unrelated changes.
    check_update(&project, P, |issue| issue.title = "Renamed".into()).unwrap();
    check_update(&project, V, |issue| issue.blocked_by.push(id(C))).unwrap();
    // Existing references can be kept or removed; only additions are checked.
    check_update(&project, P, |issue| issue.blocked_by.clear()).unwrap();

    // Valid references are accepted, by create too.
    let mut new = Issue::new("New", Timestamp::now());
    new.parent = Some(id(E));
    new.blocked_by.push(id(A));
    new.related.push(id(Z));
    new.discovered_from = Some(id(C));
    let created = project.create(&new).unwrap();
    assert_eq!(created.issue, Some(new));
}

// @mara implements VER-RELATIONS-READY
#[test]
fn cycles_and_blockers_on_ancestors_fail_with_cycle() {
    let (_dir, project) = fixture();
    let cycle = |value: &str, change: &dyn Fn(&mut Issue)| {
        let error = check_update(&project, value, change).unwrap_err();
        assert_eq!(error.code, ErrorCode::Cycle, "{value}: {error}");
        error.message
    };

    // Through `parent` only: K would become a child of its own child H.
    let message = cycle(K, &|issue| issue.parent = Some(id(H)));
    assert!(message.contains(&format!("{K} -> {H} -> {K}")), "{message}");
    // Through `blocked_by` only: A would wait for B, which waits for A.
    let message = cycle(A, &|issue| issue.blocked_by.push(id(B)));
    assert!(message.contains(&format!("{A} -> {B} -> {A}")), "{message}");
    // Through both: an epic blocked by its own child.
    let message = cycle(E, &|issue| issue.blocked_by.push(id(E1)));
    assert!(
        message.contains(&format!("{E} -> {E1} -> {E}")),
        "{message}"
    );
    // Through both over several issues: G -> H -> K (parents), K -> J
    // (blocker), and J -> G (new parent).
    let message = cycle(J, &|issue| issue.parent = Some(id(G)));
    assert!(
        message.contains(&format!("{J} -> {G} -> {H} -> {K} -> {J}")),
        "{message}"
    );

    // A blocker on an ancestor, the parent or one further up.
    cycle(G, &|issue| issue.blocked_by.push(id(H)));
    cycle(G, &|issue| issue.blocked_by.push(id(K)));
    // A new parent that puts an existing blocker above the issue.
    cycle(B, &|issue| issue.parent = Some(id(A)));
    // A new parent that puts a descendant's blocker above it: D is blocked
    // by C, and C would become D's grandparent.
    update(&project, D, |issue| issue.blocked_by.push(id(C)));
    let message = cycle(B, &|issue| issue.parent = Some(id(C)));
    assert!(message.contains(D), "{message}");

    // Create checks the same before writing anything.
    let before = snapshot(&project);
    let mut new = Issue::new("New", Timestamp::now());
    new.parent = Some(id(H));
    new.blocked_by.push(id(K));
    assert_eq!(project.create(&new).unwrap_err().code, ErrorCode::Cycle);
    new.blocked_by = vec![id(H)];
    assert_eq!(project.create(&new).unwrap_err().code, ErrorCode::Cycle);
    assert_eq!(snapshot(&project), before);

    // Relations that only look similar are fine: a blocker elsewhere in the
    // tree, and a parent whose chain leads to the issue's blocker's sibling.
    check_update(&project, G, |issue| issue.blocked_by.push(id(J))).unwrap();
    check_update(&project, T, |issue| issue.blocked_by.push(id(E1))).unwrap();
    check_update(&project, M, |issue| issue.parent = Some(id(G))).unwrap();
}

// @mara implements VER-RELATIONS-READY
#[test]
fn existing_cycles_do_not_block_unrelated_changes() {
    // A cycle that arrived with a merge, which only `zban check` reports.
    let dir = temp_dir();
    let root = dir.path();
    write_files(
        root,
        &[
            (".zban/config.toml", CONFIG),
            (
                &format!(".zban/{A}.md"),
                &issue("A", "todo", "task", &format!("parent: {B}\n"), 1),
            ),
            (
                &format!(".zban/{B}.md"),
                &issue("B", "todo", "task", &format!("blocked_by:\n  - {A}\n"), 2),
            ),
            (&format!(".zban/{C}.md"), &issue("C", "todo", "task", "", 3)),
        ],
    );
    let project = Project::open(root).unwrap();
    check_update(&project, A, |issue| issue.title = "Renamed".into()).unwrap();
    check_update(&project, A, |issue| issue.blocked_by.push(id(C))).unwrap();
    // Readiness still terminates: B's ancestor chain loops back to B.
    assert_eq!(reasons(&project)[A], [format!("ancestor_blocked:{B}")]);
    assert_eq!(reasons(&project)[B], [format!("blocked_by:{A}")]);
    // An edge towards the cycle is fine, but one that closes another cycle
    // through it is refused.
    check_update(&project, C, |issue| issue.parent = Some(id(A))).unwrap();
    let error = check_update(&project, B, |issue| issue.parent = Some(id(A))).unwrap_err();
    assert_eq!(error.code, ErrorCode::Cycle);
}

// @mara implements VER-RELATIONS-READY
#[test]
fn deleting_a_referenced_issue_is_refused() {
    let (_dir, project) = fixture();
    let before = snapshot(&project);
    let delete = |value: &str| operations::delete(&project, value);
    // B blocks on A, C stores a related link to A, and R was discovered from A.
    let error = delete(A).unwrap_err();
    assert_eq!(error.code, ErrorCode::Referenced);
    for referrer in [B, C, R] {
        assert!(error.message.contains(referrer), "{error}");
    }
    // A parent, and an archived issue that a child names.
    assert_eq!(delete(F).unwrap_err().code, ErrorCode::Referenced);
    assert_eq!(delete(Z).unwrap_err().code, ErrorCode::Referenced);
    assert_eq!(snapshot(&project), before);

    // An issue that nothing references can be deleted, even when it
    // references others itself.
    let deleted = delete(R).unwrap();
    assert!(!Path::new(&deleted.path).exists());
    assert_eq!(snapshot(&project).len(), before.len() - 1);

    assert_eq!(delete(UNKNOWN).unwrap_err().code, ErrorCode::NotFound);
    assert_eq!(delete(Y).unwrap_err().code, ErrorCode::Unreadable);
}

// @mara implements VER-RELATIONS-READY
#[test]
fn an_id_with_two_files_is_ambiguous() {
    let (_dir, project) = fixture();
    let open = fs::read(project.dir().join(format!("{M}.md"))).unwrap();
    fs::write(project.archive_dir().join(format!("{M}.md")), open).unwrap();
    let files = project.load().unwrap();
    let graph = Graph::new(&files);
    assert_eq!(
        graph.resolve(&id(M)).unwrap_err().code,
        ErrorCode::AmbiguousId
    );
    assert!(!ready(&project).contains(&M.to_owned()));
    let error = check_update(&project, C, |issue| issue.blocked_by.push(id(M))).unwrap_err();
    assert_eq!(error.code, ErrorCode::AmbiguousId);
}
