//! Smoke tests that run the real `zban` binary.

use std::process::{Command, Output};

fn zban(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zban"))
        .args(args)
        .output()
        .expect("zban runs")
}

#[test]
fn version_prints_the_package_version() {
    let output = zban(&["--version"]);
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("zban {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn unknown_option_is_invalid_usage() {
    let output = zban(&["--no-such-option"]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn no_arguments_prints_help_as_invalid_usage() {
    let output = zban(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("Usage: zban")
    );
}
