//! `VER-ID-GENERATION`: IDs are random, merge-safe and never reused locally.

mod common;

use std::cell::Cell;
use std::collections::HashSet;
use std::fs;

use common::{stderr, temp_dir, zban};
use zban::id::{ALPHABET, RANDOM_LEN};
use zban::{Id, Issue, Prefix, Project, Status, Timestamp};

/// Matches `<prefix>-[0-9a-hjkmnp-tv-z]{6}`, written out independently of
/// the alphabet constant.
fn matches_id_pattern(prefix: &str, id: &str) -> bool {
    let in_class = |c: char| matches!(c, '0'..='9' | 'a'..='h' | 'j' | 'k' | 'm' | 'n' | 'p'..='t' | 'v'..='z');
    id.strip_prefix(prefix)
        .and_then(|rest| rest.strip_prefix('-'))
        .is_some_and(|random| random.chars().count() == 6 && random.chars().all(in_class))
}

// @mara implements VER-ID-GENERATION
#[test]
fn generated_ids_match_the_pattern() {
    for prefix in ["zb", "web", "a1b2c3d4"] {
        let parsed = Prefix::parse(prefix).unwrap();
        let mut seen_characters = HashSet::new();
        for _ in 0..5000 {
            let id = Id::generate(&parsed).unwrap();
            assert!(matches_id_pattern(prefix, id.as_str()), "{id}");
            seen_characters.extend(id.as_str()[prefix.len() + 1..].chars());
        }
        assert_eq!(seen_characters.len(), 32, "every character is drawn");
    }

    let dir = temp_dir();
    let project = Project::init(dir.path(), Some("web")).unwrap();
    for _ in 0..20 {
        let file = project
            .create(&Issue::new("Generated", Timestamp::now()))
            .unwrap();
        let name = file.path.file_name().unwrap().to_str().unwrap();
        let id = name.strip_suffix(".md").unwrap();
        assert!(matches_id_pattern("web", id), "{name}");
        assert_eq!(file.id.unwrap().as_str(), id);
    }
}

// @mara implements VER-ID-GENERATION
#[test]
fn create_draws_again_when_the_id_exists_in_either_directory() {
    let dir = temp_dir();
    let project = Project::init(dir.path(), None).unwrap();
    let id = |value: &str| Id::parse(project.prefix(), value).unwrap();
    let open = Issue::new("Open", Timestamp::now());
    let closed = Issue {
        status: Status::Done,
        closed_at: Some(Timestamp::now()),
        ..Issue::new("Closed", Timestamp::now())
    };
    let open_file = project.create_with(&open, || Ok(id("zb-00000a"))).unwrap();
    let closed_file = project
        .create_with(&closed, || Ok(id("zb-00000b")))
        .unwrap();
    assert!(!open_file.archived && closed_file.archived);

    for (new, expected) in [(&open, "zb-00000c"), (&closed, "zb-00000d")] {
        let draws = ["zb-00000a", "zb-00000b", expected];
        let calls = Cell::new(0);
        let created = project
            .create_with(new, || {
                let next = id(draws[calls.get()]);
                calls.set(calls.get() + 1);
                Ok(next)
            })
            .unwrap();
        assert_eq!(calls.get(), 3, "existing open and archived IDs are skipped");
        assert_eq!(created.id, Some(id(expected)));
        assert_eq!(created.archived, new.status.is_closed());
    }
    assert_eq!(fs::read(&open_file.path).unwrap(), open_file.bytes);
    assert_eq!(fs::read(&closed_file.path).unwrap(), closed_file.bytes);
    assert_eq!(project.load().unwrap().len(), 4);
}

// @mara implements VER-ID-GENERATION
#[test]
fn init_without_prefix_writes_zb() {
    let dir = temp_dir();
    let root = dir.path();
    let output = zban(root, &["init"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fs::read_to_string(root.join(".zban/config.toml")).unwrap(),
        "format_version = 1\nprefix = \"zb\"\n"
    );

    let dir = temp_dir();
    let root = dir.path();
    let output = zban(root, &["init", "--prefix", "web2"]);
    assert!(output.status.success(), "{}", stderr(&output));
    assert_eq!(
        fs::read_to_string(root.join(".zban/config.toml")).unwrap(),
        "format_version = 1\nprefix = \"web2\"\n"
    );
}

// @mara implements VER-ID-GENERATION
#[test]
fn init_rejects_an_invalid_prefix_without_creating_zban() {
    let dir = temp_dir();
    let root = dir.path();
    for prefix in ["", "Zb", "2b", "z-b", "z_b", "toolongpx", "zé"] {
        let output = zban(root, &["init", "--prefix", prefix]);
        assert_eq!(output.status.code(), Some(1), "{prefix:?}");
        assert!(stderr(&output).contains("invalid_value"), "{prefix:?}");
        assert!(!root.join(".zban").exists(), "{prefix:?}");
    }
}

/// `RISK-ID-COLLISION`: 2^30 IDs per prefix, so a thousand issues created on
/// each of two branches collide with a probability of about 0.1%.
// @mara implements VER-ID-GENERATION
#[test]
fn collision_analysis_holds_for_the_alphabet_and_length() {
    let distinct: HashSet<_> = ALPHABET.iter().collect();
    assert_eq!(distinct.len(), 32);
    assert_eq!(RANDOM_LEN, 6);
    let values = (ALPHABET.len() as f64).powi(RANDOM_LEN as i32);
    assert_eq!(values, 2f64.powi(30));

    let per_branch = 1000.0;
    let no_collision = (1.0 - per_branch / values).powf(per_branch);
    let collision = 1.0 - no_collision;
    assert!((0.0009..0.0011).contains(&collision), "{collision}");
}
