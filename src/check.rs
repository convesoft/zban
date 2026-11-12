//! The integrity check and its repair (`REQ-CHECK`, `DES-CHECK`).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::error::{ErrorCode, Result};
use crate::id::Id;
use crate::issue::{Problem, ProblemCode, Severity};
use crate::relations::Graph;
use crate::store::{self, ARCHIVE, DIR, IssueFile, Project};

/// One problem that `zban check` reports. In JSON it is
/// `{severity, code, path, message}` (`DES-CHECK`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub severity: Severity,
    pub code: ProblemCode,
    /// The absolute path of the file the problem is in.
    pub path: String,
    pub message: String,
}

impl Finding {
    fn new(path: &Path, problem: Problem) -> Self {
        Self {
            severity: problem.code.severity(),
            code: problem.code,
            path: path.display().to_string(),
            message: problem.message,
        }
    }
}

impl Project {
    /// Checks the project that `directory` is in and returns every finding,
    /// ordered by path, then code. When the configuration is invalid, that
    /// is the only finding.
    ///
    /// With `fix`, each misplaced file is first moved into the directory its
    /// status requires, unless a file already exists at the target path; a
    /// moved file is not reported. Nothing else is ever changed.
    ///
    /// Fails with `project_not_found` outside a project, and with `io` or
    /// `conflict` when a directory cannot be read or a move fails.
    // @mara implements DES-CHECK
    pub fn check(directory: &Path, fix: bool) -> Result<Vec<Finding>> {
        let root = Self::find_root(directory)?;
        let project = match Self::try_open(&root)? {
            Ok(project) => project,
            Err(message) => {
                let problem = Problem::new(ProblemCode::InvalidConfig, message);
                return Ok(vec![Finding::new(&store::config_path(&root), problem)]);
            }
        };
        let mut files = project.load()?;
        if fix {
            for file in &mut files {
                if is_misplaced(file) {
                    match project.place(file) {
                        Ok(placed) => *file = placed,
                        // The target path is taken; the finding remains.
                        Err(error) if error.code == ErrorCode::AmbiguousId => {}
                        Err(error) => return Err(error),
                    }
                }
            }
            files.sort_by(|a, b| a.path.cmp(&b.path));
        }
        Ok(project.findings(&files))
    }

    fn findings(&self, files: &[IssueFile]) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut by_id: BTreeMap<&Id, Vec<&IssueFile>> = BTreeMap::new();
        for file in files {
            let problems = file.problems.iter().cloned();
            findings.extend(problems.map(|problem| Finding::new(&file.path, problem)));
            if let Some(id) = &file.id {
                by_id.entry(id).or_default().push(file);
            }
            if let Some(problem) = self.misplaced(file) {
                findings.push(Finding::new(&file.path, problem));
            }
        }
        // A filename is unique in its directory, so a duplicated ID has one
        // file in `.zban/` and one in `.zban/archive/`.
        for (id, same) in by_id.into_iter().filter(|(_, same)| same.len() > 1) {
            for file in same {
                let other = if file.archived {
                    format!("{DIR}/")
                } else {
                    format!("{DIR}/{ARCHIVE}/")
                };
                let message = format!("{id} also has a file in {other}; keep one of the two");
                let problem = Problem::new(ProblemCode::DuplicateId, message);
                findings.push(Finding::new(&file.path, problem));
            }
        }
        let relations = Graph::new(files).problems();
        findings.extend(
            relations
                .into_iter()
                .map(|(file, problem)| Finding::new(&file.path, problem)),
        );
        // A stable sort keeps the order of findings with the same path and code.
        findings.sort_by(|a, b| (&a.path, a.code.as_str()).cmp(&(&b.path, b.code.as_str())));
        findings
    }

    /// The `misplaced` finding for a readable issue outside the directory its
    /// status requires (`DES-PROJECT-LAYOUT`).
    fn misplaced(&self, file: &IssueFile) -> Option<Problem> {
        let issue = file.issue.as_ref().filter(|_| is_misplaced(file))?;
        let closed = issue.status.is_closed();
        let dir = if closed {
            format!("{DIR}/{ARCHIVE}/")
        } else {
            format!("{DIR}/")
        };
        let message = match self.destination(file, closed) {
            Ok(_) => format!(
                "the status {} belongs in {dir}; `zban check --fix` moves the file",
                issue.status
            ),
            Err(_) => format!(
                "the status {} belongs in {dir}, but a file with this name already exists there",
                issue.status
            ),
        };
        Some(Problem::new(ProblemCode::Misplaced, message))
    }
}

/// Whether `file` holds a readable issue outside the directory its status
/// requires.
fn is_misplaced(file: &IssueFile) -> bool {
    file.issue
        .as_ref()
        .is_some_and(|issue| issue.status.is_closed() != file.archived)
}
