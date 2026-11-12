//! The project directory and the issue files in it (`DES-PROJECT-LAYOUT`).
//!
//! Every write goes through a temporary file in the target directory that is
//! renamed or linked into place, so an interrupted write never leaves a
//! partial issue file (`REQ-PRESERVE-CONTENT`). Replacing a file first checks
//! that it still holds the bytes that were read (`REQ-NO-LOST-UPDATES`), under
//! a lock on `.zban/` that lasts until the file is replaced. Reading the
//! project holds the same lock, shared, from listing the directories until
//! the listed files have been read, so that it never overlaps a rename
//! (`ADR-WRITE-LOCK`).

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{Error, ErrorCode, Result};
use crate::id::{Id, Prefix};
use crate::issue::{Issue, Problem, ProblemCode};
use crate::relations::Graph;

/// The project directory inside the project root.
pub const DIR: &str = ".zban";
/// The directory of closed issues inside [`DIR`].
pub const ARCHIVE: &str = "archive";
/// The configuration file inside [`DIR`].
pub const CONFIG: &str = "config.toml";

/// How often `create` draws an ID before giving up. Only a broken random
/// source can exhaust this, given 2^30 IDs per prefix.
const MAX_ID_ATTEMPTS: usize = 100;

/// A zban project: the directory holding `.zban/` and its configuration.
// @mara implements DES-PROJECT-LAYOUT
#[derive(Debug, Clone)]
pub struct Project {
    root: PathBuf,
    config: Config,
}

/// One issue file as read from disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueFile {
    pub path: PathBuf,
    /// Whether the file is in `.zban/archive/`.
    pub archived: bool,
    /// The ID from the filename, or `None` when the name is not a valid ID.
    pub id: Option<Id>,
    /// The bytes read, which a later replace compares against.
    pub bytes: Vec<u8>,
    /// The issue, or `None` when the file is unreadable.
    pub issue: Option<Issue>,
    /// Problems with this file, including warnings on readable issues.
    pub problems: Vec<Problem>,
}

impl IssueFile {
    /// The `unreadable` error for an operation that needs this file's issue,
    /// naming the first problem that makes it unreadable.
    pub(crate) fn unreadable(&self) -> Error {
        let reason = self.problems.iter().find(|problem| problem.code.is_error());
        Error::new(
            ErrorCode::Unreadable,
            format!(
                "{} cannot be read{}; fix it by hand first",
                self.path.display(),
                reason.map_or_else(String::new, |problem| format!(": {}", problem.message))
            ),
        )
    }
}

impl Project {
    /// Creates `.zban/` and its configuration in `directory`. An invalid
    /// prefix is rejected before anything is created.
    pub fn init(directory: &Path, prefix: Option<&str>) -> Result<Self> {
        Self::init_linking(directory, prefix, hard_link)
    }

    /// [`Project::init`], giving the configuration its name with `link`.
    fn init_linking(directory: &Path, prefix: Option<&str>, link: Link) -> Result<Self> {
        let config = Config {
            prefix: prefix.map(Prefix::parse).transpose()?.unwrap_or_default(),
        };
        let root = fs::canonicalize(directory).map_err(|error| Error::io(directory, &error))?;
        let dir = root.join(DIR);
        match fs::create_dir(&dir) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                return Err(Error::new(
                    ErrorCode::ProjectExists,
                    format!("{} already exists", dir.display()),
                ));
            }
            Err(error) => return Err(Error::io(&dir, &error)),
        }
        let project = Self { root, config };
        let path = dir.join(CONFIG);
        let mut staged = Staged::write(&dir, project.config.render().as_bytes())?;
        if !project.add_file(&mut staged, &path, None, link)? {
            return Err(Error::io(&path, &io::ErrorKind::AlreadyExists.into()));
        }
        Ok(project)
    }

    /// Finds the project whose `.zban/config.toml` is nearest above `start`.
    pub fn discover(start: &Path) -> Result<Self> {
        Self::open(&Self::find_root(start)?)
    }

    /// The nearest directory at or above `start` that holds
    /// `.zban/config.toml`. Fails with `project_not_found` when there is none.
    pub(crate) fn find_root(start: &Path) -> Result<PathBuf> {
        let start = fs::canonicalize(start).map_err(|error| Error::io(start, &error))?;
        if let Some(root) = nearest_root(&start) {
            return Ok(root.to_path_buf());
        }
        Err(Error::new(
            ErrorCode::ProjectNotFound,
            format!(
                "no {DIR}/{CONFIG} in {} or any parent directory; run `zban init` to create a project",
                start.display()
            ),
        ))
    }

    /// Opens the project rooted at `root`.
    pub fn open(root: &Path) -> Result<Self> {
        Self::try_open(root)?.map_err(|message| {
            Error::new(
                ErrorCode::InvalidConfig,
                format!("{}: {message}", config_path(root).display()),
            )
        })
    }

    /// Opens the project rooted at `root`. The inner error says why its
    /// configuration is invalid, without naming the file.
    pub(crate) fn try_open(root: &Path) -> Result<Result<Self, String>> {
        let path = config_path(root);
        let bytes = fs::read(&path).map_err(|error| Error::io(&path, &error))?;
        let config = String::from_utf8(bytes)
            .map_err(|_| "not valid UTF-8".to_owned())
            .and_then(|text| Config::parse(&text));
        Ok(config.map(|config| Self {
            root: root.to_path_buf(),
            config,
        }))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn prefix(&self) -> &Prefix {
        &self.config.prefix
    }

    /// The `.zban/` directory, which holds open issues.
    pub fn dir(&self) -> PathBuf {
        self.root.join(DIR)
    }

    /// The `.zban/` directory of the nearest project above this one's root.
    /// Commands run inside this project no longer find that project.
    pub fn enclosing_dir(&self) -> Option<PathBuf> {
        nearest_root(self.root.parent()?).map(|root| root.join(DIR))
    }

    /// The `.zban/archive/` directory, which holds closed issues.
    pub fn archive_dir(&self) -> PathBuf {
        self.dir().join(ARCHIVE)
    }

    /// Reads every issue file. A file that cannot be read or parsed is
    /// returned with its problems instead of failing the load
    /// (`REQ-DAMAGED-DATA`). Files are ordered by path.
    ///
    /// The directories are listed and the files read under one shared lock
    /// on `.zban/`, so that no zban process replaces, moves or deletes a file
    /// in between: a file that a close or reopen moves into the other
    /// directory after the listing would be missed. The files are parsed
    /// after the lock is released.
    pub fn load(&self) -> Result<Vec<IssueFile>> {
        let read = self.shared(|| {
            let mut read = Vec::new();
            for (path, archived) in self.entries()? {
                if is_issue_file(&path) {
                    let bytes = fs::read(&path);
                    read.push((path, archived, bytes));
                }
            }
            Ok(read)
        })?;
        let mut files: Vec<_> = read
            .into_iter()
            .map(|(path, archived, bytes)| self.parse(path, archived, bytes))
            .collect();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }

    /// The temporary files that interrupted writes left in `.zban/` or
    /// `.zban/archive/`, ordered by path (`DES-CHECK`). They are listed
    /// under a shared lock on `.zban/`, as [`Project::load`] does.
    pub(crate) fn temporary_files(&self) -> Result<Vec<PathBuf>> {
        let mut paths = self.shared(|| {
            Ok(self
                .entries()?
                .into_iter()
                .map(|(path, _)| path)
                .filter(|path| is_temporary_file(path))
                .collect::<Vec<_>>())
        })?;
        paths.sort();
        Ok(paths)
    }

    /// Every entry directly inside `.zban/` and `.zban/archive/`, with
    /// whether it is in the archive. A missing archive has no entries.
    ///
    /// Callers hold a shared lock on `.zban/`, because a listing that
    /// overlaps a rename over one of its entries can leave that entry out,
    /// as on btrfs.
    fn entries(&self) -> Result<Vec<(PathBuf, bool)>> {
        let mut paths = Vec::new();
        for (dir, archived) in [(self.dir(), false), (self.archive_dir(), true)] {
            let entries = match fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(error) if archived && error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(Error::io(&dir, &error)),
            };
            for entry in entries {
                let path = entry.map_err(|error| Error::io(&dir, &error))?.path();
                paths.push((path, archived));
            }
        }
        Ok(paths)
    }

    /// The issue file at `path`, from `read`, the result of reading it.
    fn parse(&self, path: PathBuf, archived: bool, read: io::Result<Vec<u8>>) -> IssueFile {
        match read {
            Ok(bytes) => self.file(path, archived, bytes),
            Err(error) => {
                let id = self.filename_id(&path);
                IssueFile {
                    path,
                    archived,
                    id,
                    bytes: Vec::new(),
                    issue: None,
                    problems: vec![Problem::new(
                        ProblemCode::Unparseable,
                        format!("cannot read the file: {error}"),
                    )],
                }
            }
        }
    }

    fn filename_id(&self, path: &Path) -> Option<Id> {
        let stem = path.file_stem()?.to_str()?;
        Id::parse(self.prefix(), stem)
    }

    fn file(&self, path: PathBuf, archived: bool, bytes: Vec<u8>) -> IssueFile {
        let id = self.filename_id(&path);
        let mut problems = Vec::new();
        if id.is_none() {
            problems.push(Problem::new(
                ProblemCode::InvalidFilename,
                format!(
                    "the filename is not `<id>.md` with an ID like {}-a1b2c3",
                    self.prefix()
                ),
            ));
        }
        let (issue, parse_problems) = match std::str::from_utf8(&bytes) {
            Ok(text) => Issue::parse(self.prefix(), text),
            Err(_) => (
                None,
                vec![Problem::new(ProblemCode::Unparseable, "not valid UTF-8")],
            ),
        };
        problems.extend(parse_problems);
        IssueFile {
            issue: issue.filter(|_| id.is_some()),
            path,
            archived,
            id,
            bytes,
            problems,
        }
    }

    /// Creates a new issue file with a fresh random ID. Kept for tests, which
    /// write exact issues that a `CreateRequest` cannot describe.
    pub fn create(&self, issue: &Issue) -> Result<IssueFile> {
        self.create_with(issue, || Id::generate(self.prefix()))
    }

    /// Creates a new issue file, drawing IDs from `next_id` until one is
    /// unused in both `.zban/` and `.zban/archive/` (`DES-ISSUE-ID`). The file
    /// is added as [`Project::add_file`] describes. Its relations are checked
    /// against the project first (`REQ-RELATIONS`).
    /// Kept for tests, which choose the IDs to draw.
    pub fn create_with(
        &self,
        issue: &Issue,
        next_id: impl FnMut() -> Result<Id>,
    ) -> Result<IssueFile> {
        self.create_among(&Graph::new(&self.load()?), issue, next_id)
    }

    /// [`Project::create_with`], checking relations against `graph`, a load
    /// of this project.
    pub(crate) fn create_among(
        &self,
        graph: &Graph,
        issue: &Issue,
        next_id: impl FnMut() -> Result<Id>,
    ) -> Result<IssueFile> {
        self.create_linking(graph, issue, next_id, hard_link)
    }

    /// [`Project::create_among`], giving the file its name with `link`.
    fn create_linking(
        &self,
        graph: &Graph,
        issue: &Issue,
        mut next_id: impl FnMut() -> Result<Id>,
        link: Link,
    ) -> Result<IssueFile> {
        let bytes = self.encode(issue)?;
        graph.check(None, None, issue)?;
        let archived = issue.status.is_closed();
        let (dir, other) = if archived {
            (self.archive_dir(), self.dir())
        } else {
            (self.dir(), self.archive_dir())
        };
        if archived {
            fs::create_dir_all(&dir).map_err(|error| Error::io(&dir, &error))?;
        }
        let mut staged = Staged::write(&dir, &bytes)?;
        for _ in 0..MAX_ID_ATTEMPTS {
            let name = format!("{}.md", next_id()?);
            let path = dir.join(&name);
            if self.add_file(&mut staged, &path, Some(&other.join(&name)), link)? {
                return Ok(self.file(path, archived, bytes));
            }
        }
        Err(Error::new(
            ErrorCode::Io,
            format!("found no unused issue ID in {MAX_ID_ATTEMPTS} attempts"),
        ))
    }

    /// Gives `staged` the new name `target`, or returns `false`, changing
    /// nothing, when `target` or `twin` is taken. It links the file there,
    /// which fails rather than replacing a file. Where the file system has no
    /// hard links, it instead checks both names and renames the file while
    /// holding the exclusive lock, so that no other zban process takes the
    /// name in between (`ADR-CREATE-WITHOUT-HARD-LINKS`).
    // @mara implements DES-SHARED-CORE
    fn add_file(
        &self,
        staged: &mut Staged,
        target: &Path,
        twin: Option<&Path>,
        link: Link,
    ) -> Result<bool> {
        let taken = |path: &Path| fs::symlink_metadata(path).is_ok();
        if twin.is_some_and(taken) {
            return Ok(false);
        }
        match link(&staged.path, target) {
            Ok(()) => return Ok(true),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => return Ok(false),
            Err(error) if !linking_unsupported(&error) => return Err(Error::io(target, &error)),
            Err(_) => {}
        }
        self.locked(|| {
            if taken(target) || twin.is_some_and(taken) {
                return Ok(false);
            }
            staged.rename(target)?;
            Ok(true)
        })
    }

    /// Replaces a file read earlier with `issue`, in place. Fails with
    /// `conflict`, leaving the file alone, if it no longer holds the bytes
    /// that were read. The file stays in its directory even when the status
    /// now belongs in the other one; [`Project::place`] moves it, and
    /// [`Project::save`] applies the whole of `DES-LIFECYCLE`.
    pub fn replace(&self, file: &IssueFile, issue: &Issue) -> Result<IssueFile> {
        let bytes = self.encode(issue)?;
        let dir = file.path.parent().unwrap_or(Path::new("."));
        let mut staged = Staged::write(dir, &bytes)?;
        if let Ok(metadata) = fs::metadata(&file.path) {
            // Keep the file's permissions; a failure here is not worth failing for.
            let _ = fs::set_permissions(&staged.path, metadata.permissions());
        }
        self.locked(|| {
            unchanged(file)?;
            staged.rename(&file.path)
        })?;
        Ok(self.file(file.path.clone(), file.archived, bytes))
    }

    /// Renames a file into the directory its status requires: `.zban/archive/`
    /// when closed, `.zban/` when open (`DES-PROJECT-LAYOUT`). A file already
    /// there, or one without a readable issue, is returned unchanged. Fails
    /// with `ambiguous_id`, moving nothing, when the target name is taken.
    // @mara implements DES-LIFECYCLE
    pub fn place(&self, file: &IssueFile) -> Result<IssueFile> {
        let Some(issue) = &file.issue else {
            return Ok(file.clone());
        };
        self.locked(|| {
            let Some(target) = self.destination(file, issue.status.is_closed())? else {
                return Ok(file.clone());
            };
            if let Some(dir) = target.parent() {
                fs::create_dir_all(dir).map_err(|error| Error::io(dir, &error))?;
            }
            if let Err(error) = fs::rename(&file.path, &target) {
                if error.kind() == io::ErrorKind::NotFound && !file.path.exists() {
                    return Err(conflict(file, "was moved or deleted"));
                }
                return Err(Error::io(&target, &error));
            }
            Ok(IssueFile {
                path: target,
                archived: !file.archived,
                ..file.clone()
            })
        })
    }

    /// The path `file` moves to so that it is archived exactly when `closed`,
    /// or `None` when it is in that directory already. Fails with
    /// `ambiguous_id` when the path is taken.
    pub(crate) fn destination(&self, file: &IssueFile, closed: bool) -> Result<Option<PathBuf>> {
        if closed == file.archived {
            return Ok(None);
        }
        let dir = if closed {
            self.archive_dir()
        } else {
            self.dir()
        };
        let target = dir.join(file.path.file_name().unwrap_or_default());
        if fs::symlink_metadata(&target).is_ok() {
            return Err(Error::new(
                ErrorCode::AmbiguousId,
                format!(
                    "{} cannot move to {}, which already exists; keep one of the two files",
                    file.path.display(),
                    target.display()
                ),
            ));
        }
        Ok(Some(target))
    }

    /// Deletes issue `id`, resolving it and its referrers in `graph`, a load
    /// of this project. Fails with `referenced`, deleting nothing, while
    /// another issue names it in a relation, and with `conflict` if the file
    /// changes while it is checked.
    // @mara implements REQ-RELATIONS
    pub(crate) fn delete<'a>(&self, graph: &Graph<'a>, id: &Id) -> Result<&'a IssueFile> {
        let file = graph.resolve(id)?.file;
        let referrers = graph.referrers(id);
        if !referrers.is_empty() {
            let names: Vec<_> = referrers.iter().map(|id| id.as_str()).collect();
            return Err(Error::new(
                ErrorCode::Referenced,
                format!(
                    "{id} is referenced by {}; remove those relations first",
                    names.join(", ")
                ),
            ));
        }
        self.locked(|| {
            unchanged(file)?;
            fs::remove_file(&file.path).map_err(|error| Error::io(&file.path, &error))
        })?;
        Ok(file)
    }

    /// Runs `write` while holding an exclusive advisory lock on `.zban/`, so
    /// that no other zban process replaces, moves or deletes an issue file
    /// between the checks in `write` and its change, or reads the project
    /// meanwhile (`ADR-WRITE-LOCK`).
    // @mara implements DES-SHARED-CORE
    fn locked<T>(&self, write: impl FnOnce() -> Result<T>) -> Result<T> {
        self.with_lock(File::lock, write)
    }

    /// Runs `read` while holding a shared advisory lock on `.zban/`, so that
    /// no zban process replaces, moves or deletes an issue file meanwhile
    /// (`ADR-WRITE-LOCK`). Other reads can hold it at the same time.
    // @mara implements DES-SHARED-CORE
    fn shared<T>(&self, read: impl FnOnce() -> Result<T>) -> Result<T> {
        self.with_lock(File::lock_shared, read)
    }

    /// Runs `step` while holding the lock that `lock` takes on a new handle
    /// of `.zban/`. The lock waits for a conflicting holder and is released
    /// when `step` returns. Where the file system has no advisory locks,
    /// `step` runs unlocked, which leaves the byte comparison as the only
    /// guard of a write (`ADR-WRITE-LOCK`). Any other failure to take the
    /// lock fails the step.
    ///
    /// Locks on separate handles conflict even within one process, so
    /// `step` must not take the lock again: that would wait forever.
    fn with_lock<T>(
        &self,
        lock: fn(&File) -> io::Result<()>,
        step: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let dir = self.dir();
        let handle = File::open(&dir).map_err(|error| Error::io(&dir, &error))?;
        if let Err(error) = lock(&handle)
            && !unsupported(&error)
        {
            return Err(Error::io(&dir, &error));
        }
        step()
    }

    /// Renders an issue for writing, after checking that it is valid and
    /// reads back unchanged, so a write never leaves a file that zban would
    /// read differently.
    fn encode(&self, issue: &Issue) -> Result<Vec<u8>> {
        issue.validate()?;
        let text = issue.render();
        if Issue::parse(self.prefix(), &text).0.as_ref() != Some(issue) {
            return Err(Error::invalid_value(
                "the issue does not read back unchanged; check its IDs and unknown header entries",
            ));
        }
        Ok(text.into_bytes())
    }
}

/// The configuration file of the project rooted at `root`.
pub(crate) fn config_path(root: &Path) -> PathBuf {
    root.join(DIR).join(CONFIG)
}

/// The nearest directory at or above `start` that holds `.zban/config.toml`.
fn nearest_root(start: &Path) -> Option<&Path> {
    start.ancestors().find(|root| config_path(root).is_file())
}

/// Fails with `conflict` unless `file` still holds the bytes that were read
/// (`REQ-NO-LOST-UPDATES`).
fn unchanged(file: &IssueFile) -> Result<()> {
    match fs::read(&file.path) {
        Ok(current) if current == file.bytes => Ok(()),
        Ok(_) => Err(conflict(file, "changed")),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Err(conflict(file, "was moved or deleted"))
        }
        Err(error) => Err(Error::io(&file.path, &error)),
    }
}

/// The `conflict` error for a file that `what` happened to since it was read.
fn conflict(file: &IssueFile, what: &str) -> Error {
    Error::new(
        ErrorCode::Conflict,
        format!(
            "{} {what} since it was read; show the issue again and retry",
            file.path.display()
        ),
    )
}

/// Whether `error` says that the file system does not support the call,
/// such as taking an advisory lock (`ADR-WRITE-LOCK`).
fn unsupported(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::Unsupported
        || error
            .raw_os_error()
            .is_some_and(|code| UNSUPPORTED.contains(&code))
}

/// Whether linking a new file into place failed with `error` because the
/// file system has no hard links: unsupported, or `EPERM` on Linux, which
/// `link(2)` documents for such a file system
/// (`ADR-CREATE-WITHOUT-HARD-LINKS`).
fn linking_unsupported(error: &io::Error) -> bool {
    unsupported(error) || (cfg!(target_os = "linux") && error.raw_os_error() == Some(1))
}

/// `ENOSYS`, `ENOTSUP` and `EOPNOTSUPP` on the platforms of
/// `ADR-RUST-BINARY`, which the standard library may not report as
/// `Unsupported`; it does not for macOS's `ENOTSUP`. Elsewhere only
/// `Unsupported` counts.
const UNSUPPORTED: &[i32] = if cfg!(target_vendor = "apple") {
    &[78, 45, 102]
} else if cfg!(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
)) {
    &[38, 95]
} else {
    &[]
};

/// Every `*.md` file is an issue, whatever its name.
fn is_issue_file(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.as_encoded_bytes().ends_with(b".md"))
        && fs::metadata(path).is_ok_and(|metadata| metadata.is_file())
}

/// Whether `path` is a file named like the temporary files of [`Staged`],
/// `.zban-*.tmp`.
fn is_temporary_file(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        name.as_encoded_bytes()
            .strip_prefix(TEMPORARY_PREFIX.as_bytes())
            .is_some_and(|rest| rest.ends_with(TEMPORARY_SUFFIX.as_bytes()))
    }) && fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file())
}

/// The start and end of a temporary file's name.
const TEMPORARY_PREFIX: &str = ".zban-";
const TEMPORARY_SUFFIX: &str = ".tmp";

/// A fully written temporary file, removed on drop unless it was renamed.
/// Its `.tmp` name is never taken for an issue.
#[derive(Debug)]
struct Staged {
    path: PathBuf,
    renamed: bool,
}

impl Staged {
    fn write(dir: &Path, bytes: &[u8]) -> Result<Self> {
        loop {
            let random = getrandom::u64().map_err(|error| {
                Error::new(
                    ErrorCode::Io,
                    format!("cannot read operating-system randomness: {error}"),
                )
            })?;
            let path = dir.join(format!("{TEMPORARY_PREFIX}{random:016x}{TEMPORARY_SUFFIX}"));
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut file) => {
                    let staged = Self {
                        path,
                        renamed: false,
                    };
                    file.write_all(bytes)
                        .and_then(|()| file.sync_all())
                        .map_err(|error| Error::io(&staged.path, &error))?;
                    return Ok(staged);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(Error::io(&path, &error)),
            }
        }
    }

    /// Atomically replaces `target` with this file.
    fn rename(&mut self, target: &Path) -> Result<()> {
        fs::rename(&self.path, target).map_err(|error| Error::io(target, &error))?;
        self.renamed = true;
        Ok(())
    }
}

/// Gives a file a second name, failing with `AlreadyExists` rather than
/// replacing a file there. It is [`hard_link`] except in tests, which make
/// it fail as a file system without hard links does.
type Link = fn(&Path, &Path) -> io::Result<()>;

/// Creates `link` as another name of the file at `original`.
fn hard_link(original: &Path, link: &Path) -> io::Result<()> {
    fs::hard_link(original, link)
}

impl Drop for Staged {
    fn drop(&mut self) {
        if !self.renamed {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::timestamp::Timestamp;

    /// A new project in a fresh directory, which lasts as long as the
    /// returned [`TempDir`].
    fn temp_project() -> (TempDir, Project) {
        let dir = tempfile::tempdir().unwrap();
        let project = Project::init(dir.path(), None).unwrap();
        (dir, project)
    }

    fn entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        names.sort();
        names
    }

    // @mara implements VER-ISSUE-FILES
    #[test]
    fn interrupted_write_leaves_the_original_and_no_partial_issue() {
        let (_dir, project) = temp_project();
        let created = project
            .create(&Issue::new("Original", Timestamp::now()))
            .unwrap();
        let before = entries(&project.dir());
        let original = fs::read(&created.path).unwrap();

        // An update whose process dies after writing the new content but
        // before the rename: the temporary file stays behind.
        let mut changed = created.issue.clone().unwrap();
        changed.title = "Changed".into();
        let staged = Staged::write(&project.dir(), changed.render().as_bytes()).unwrap();
        // A create interrupted before its link leaves only a temporary file too.
        let staged_new = Staged::write(&project.dir(), changed.render().as_bytes()).unwrap();
        std::mem::forget(staged);
        std::mem::forget(staged_new);

        assert_eq!(fs::read(&created.path).unwrap(), original);
        let after = entries(&project.dir());
        assert_eq!(after.len(), before.len() + 2);
        assert!(
            after
                .iter()
                .filter(|name| !before.contains(name))
                .all(|name| name.starts_with(".zban-") && name.ends_with(".tmp"))
        );
        let loaded = project.load().unwrap();
        assert_eq!(loaded, vec![created]);
    }

    // @mara implements VER-CHECK
    #[test]
    fn check_reports_and_fix_deletes_what_interrupted_writes_leave() {
        let (_dir, project) = temp_project();
        let created = project
            .create(&Issue::new("Original", Timestamp::now()))
            .unwrap();
        fs::create_dir_all(project.archive_dir()).unwrap();
        let before = (entries(&project.dir()), entries(&project.archive_dir()));
        let staged = [project.dir(), project.archive_dir()]
            .map(|dir| Staged::write(&dir, created.bytes.as_slice()).unwrap());
        let mut paths: Vec<_> = staged.iter().map(|staged| staged.path.clone()).collect();
        paths.sort();
        staged.into_iter().for_each(std::mem::forget);

        let findings = Project::check(project.root(), false).unwrap();
        let reported: Vec<_> = findings
            .iter()
            .map(|finding| (PathBuf::from(&finding.path), finding.code))
            .collect();
        let expected: Vec<_> = paths
            .into_iter()
            .map(|path| (path, ProblemCode::TemporaryFile))
            .collect();
        assert_eq!(reported, expected);

        assert_eq!(Project::check(project.root(), true).unwrap(), Vec::new());
        let after = (entries(&project.dir()), entries(&project.archive_dir()));
        assert_eq!(after, before);
    }

    #[test]
    fn failed_writes_remove_their_temporary_file() {
        let (_dir, project) = temp_project();
        let created = project
            .create(&Issue::new("Original", Timestamp::now()))
            .unwrap();
        fs::write(&created.path, "edited elsewhere").unwrap();
        let error = project
            .replace(&created, created.issue.as_ref().unwrap())
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::Conflict);
        let names = entries(&project.dir());
        assert!(
            names.iter().all(|name| !name.ends_with(".tmp")),
            "{names:?}"
        );
    }

    // @mara implements VER-ISSUE-FILES
    #[test]
    fn a_write_proceeds_unlocked_only_where_locking_is_unsupported() {
        if cfg!(any(target_os = "linux", target_os = "macos"))
            && cfg!(any(target_arch = "x86_64", target_arch = "aarch64"))
        {
            assert!(!UNSUPPORTED.is_empty());
        }
        for &code in UNSUPPORTED {
            let error = io::Error::from_raw_os_error(code);
            // The operating system's own message confirms each number. Its
            // letter case depends on the C library: glibc says "Operation not
            // supported" for 95, musl "Not supported".
            let message = error.to_string();
            let lowercase = message.to_lowercase();
            assert!(
                lowercase.contains("not supported") || lowercase.contains("not implemented"),
                "{code}: {message}"
            );
            assert!(unsupported(&error), "{message}");
        }
        assert!(unsupported(&io::ErrorKind::Unsupported.into()));

        let failures = [
            io::ErrorKind::PermissionDenied.into(),
            io::ErrorKind::Interrupted.into(),
            io::Error::other("lock failed"),
            // EIO and EACCES, which have these numbers on Linux and macOS.
            io::Error::from_raw_os_error(5),
            io::Error::from_raw_os_error(13),
        ];
        for error in failures {
            assert!(!unsupported(&error), "{error}");
        }
    }

    /// Fails as linking does on a file system without hard links: with
    /// `EPERM` on Linux and `ENOTSUP` on macOS.
    fn link_unsupported(_: &Path, _: &Path) -> io::Result<()> {
        let code = if cfg!(target_vendor = "apple") { 45 } else { 1 };
        Err(io::Error::from_raw_os_error(code))
    }

    /// Fails as linking does when the file system has hard links but the
    /// write fails, with `EIO`.
    fn link_broken(_: &Path, _: &Path) -> io::Result<()> {
        Err(io::Error::from_raw_os_error(5))
    }

    // @mara implements VER-ISSUE-FILES
    #[test]
    fn linking_falls_back_only_where_the_file_system_has_no_hard_links() {
        let eperm = io::Error::from_raw_os_error(1);
        assert!(eperm.to_string().contains("not permitted"), "{eperm}");
        assert_eq!(linking_unsupported(&eperm), cfg!(target_os = "linux"));
        assert!(linking_unsupported(
            &link_unsupported(Path::new("a"), Path::new("b")).unwrap_err()
        ));
        assert!(linking_unsupported(&io::ErrorKind::Unsupported.into()));
        for &code in UNSUPPORTED {
            assert!(linking_unsupported(&io::Error::from_raw_os_error(code)));
        }

        let failures = [
            io::ErrorKind::PermissionDenied.into(),
            io::Error::other("link failed"),
            // EIO, EACCES, EEXIST, EXDEV and EMLINK, which have these
            // numbers on Linux and macOS.
            io::Error::from_raw_os_error(5),
            io::Error::from_raw_os_error(13),
            io::Error::from_raw_os_error(17),
            io::Error::from_raw_os_error(18),
            io::Error::from_raw_os_error(31),
        ];
        for error in failures {
            assert!(!linking_unsupported(&error), "{error}");
        }
    }

    // @mara implements VER-ISSUE-FILES
    #[test]
    fn init_and_create_rename_into_place_without_hard_links() {
        let dir = tempfile::tempdir().unwrap();
        let project = Project::init_linking(dir.path(), None, link_unsupported).unwrap();
        assert_eq!(entries(&project.dir()), [CONFIG]);
        assert_eq!(
            Project::open(project.root()).unwrap().prefix(),
            project.prefix()
        );

        let id = |value: &str| Id::parse(project.prefix(), value).unwrap();
        let closed = |title: &str| Issue {
            status: crate::Status::Done,
            closed_at: Some(Timestamp::now()),
            ..Issue::new(title, Timestamp::now())
        };
        let taken_open = project
            .create_with(&Issue::new("Open", Timestamp::now()), || {
                Ok(id("zb-00000a"))
            })
            .unwrap();
        let taken_closed = project
            .create_with(&closed("Closed"), || Ok(id("zb-00000b")))
            .unwrap();

        // Each create first draws the ID taken in its own directory, then
        // the one taken in the other directory.
        let files = project.load().unwrap();
        let graph = Graph::new(&files);
        let mut ids = ["zb-00000a", "zb-00000b", "zb-00000c"].map(id).into_iter();
        let open = project
            .create_linking(
                &graph,
                &Issue::new("New open", Timestamp::now()),
                || Ok(ids.next().unwrap()),
                link_unsupported,
            )
            .unwrap();
        let mut ids = ["zb-00000b", "zb-00000a", "zb-00000d"].map(id).into_iter();
        let closed = project
            .create_linking(
                &graph,
                &closed("New closed"),
                || Ok(ids.next().unwrap()),
                link_unsupported,
            )
            .unwrap();

        assert_eq!(open.path, project.dir().join("zb-00000c.md"));
        assert_eq!(closed.path, project.archive_dir().join("zb-00000d.md"));
        assert_eq!(fs::read(&taken_open.path).unwrap(), taken_open.bytes);
        assert_eq!(fs::read(&taken_closed.path).unwrap(), taken_closed.bytes);
        assert_eq!(
            entries(&project.dir()),
            [ARCHIVE, CONFIG, "zb-00000a.md", "zb-00000c.md"]
        );
        assert_eq!(
            entries(&project.archive_dir()),
            ["zb-00000b.md", "zb-00000d.md"]
        );
        let loaded = project.load().unwrap();
        assert!(loaded.contains(&open) && loaded.contains(&closed));
    }

    // @mara implements VER-ISSUE-FILES
    #[test]
    fn init_and_create_fail_with_io_on_any_other_linking_error() {
        let (_dir, project) = temp_project();
        let files = project.load().unwrap();
        let error = project
            .create_linking(
                &Graph::new(&files),
                &Issue::new("Never written", Timestamp::now()),
                || Id::generate(project.prefix()),
                link_broken,
            )
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::Io);
        assert_eq!(entries(&project.dir()), [CONFIG]);

        let dir = tempfile::tempdir().unwrap();
        let error = Project::init_linking(dir.path(), None, link_broken).unwrap_err();
        assert_eq!(error.code, ErrorCode::Io);
        // The empty `.zban/` that stays behind is zb-bjyq1s.
        assert!(entries(&dir.path().join(DIR)).is_empty());
    }
}
