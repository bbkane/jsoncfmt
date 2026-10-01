use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use insta_cmd::{assert_cmd_snapshot, get_cargo_bin};
use tempfile::TempDir;

fn command() -> Command {
    Command::new(get_cargo_bin("jsoncfmt"))
}

fn run(command: &mut Command) -> Output {
    command.output().expect("command should run")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn temp_file(contents: &str) -> (TempDir, PathBuf) {
    let directory = TempDir::with_prefix("jsoncfmt-").expect("temporary directory should exist");
    let path = directory.path().join("input.jsonc");
    fs::write(&path, contents).expect("fixture should be written");
    (directory, path)
}

#[test]
fn snapshots_cli_metadata() {
    assert_cmd_snapshot!("help", command().arg("--help"));
    assert_cmd_snapshot!("format_help", command().args(["format", "--help"]));
    assert_cmd_snapshot!("version", command().arg("version"));
}

#[test]
fn snapshots_formatting() {
    assert_cmd_snapshot!(
        "default_format",
        command().args(["format", "tests/fixtures/comments.jsonc"])
    );
    assert_cmd_snapshot!(
        "preserve_key_order",
        command().args([
            "format",
            "--sort-keys",
            "false",
            "tests/fixtures/comments.jsonc",
        ])
    );
    assert_cmd_snapshot!(
        "four_space_indent",
        command().args([
            "format",
            "--indent-width",
            "4",
            "tests/fixtures/comments.jsonc",
        ])
    );
}

#[test]
fn snapshots_errors() {
    assert_cmd_snapshot!(
        "malformed_jsonc",
        command().args(["format", "tests/fixtures/malformed.jsonc"])
    );
    assert_cmd_snapshot!(
        "missing_file",
        command().args(["format", "tests/fixtures/does-not-exist.jsonc"])
    );
    assert_cmd_snapshot!(
        "invalid_boolean",
        command().args([
            "format",
            "--sort-keys",
            "sometimes",
            "tests/fixtures/comments.jsonc",
        ])
    );
    assert_cmd_snapshot!(
        "invalid_indent",
        command().args([
            "format",
            "--indent-width",
            "0",
            "tests/fixtures/comments.jsonc",
        ])
    );
}

#[test]
fn in_place_is_idempotent_and_has_no_output() {
    let (_directory, path) = temp_file(r#"{"z":1,"a":[{"d":4,"c":3},2]}"#);

    let first = run(command()
        .arg("format")
        .arg("--in-place")
        .arg("true")
        .arg(&path));
    assert_success(&first);
    assert!(first.stdout.is_empty());
    assert!(first.stderr.is_empty());
    let once = fs::read(&path).expect("formatted file should be readable");

    let second = run(command()
        .arg("format")
        .arg("--in-place")
        .arg("true")
        .arg(&path));
    assert_success(&second);
    assert!(second.stdout.is_empty());
    assert!(second.stderr.is_empty());
    let twice = fs::read(&path).expect("formatted file should be readable");

    assert_eq!(once, twice);
    assert_eq!(
        String::from_utf8(once).expect("formatter output should be UTF-8"),
        concat!(
            "{\n",
            "  \"a\": [\n",
            "    {\n",
            "      \"c\": 3,\n",
            "      \"d\": 4,\n",
            "    },\n",
            "    2,\n",
            "  ],\n",
            "  \"z\": 1,\n",
            "}\n",
        )
    );
    assert_eq!(
        fs::read_dir(_directory.path()).unwrap().count(),
        1,
        "successful formatting should not leave a temporary file"
    );
}

#[test]
fn failure_does_not_modify_the_file() {
    let original = b"{ \"broken\": }";
    let (directory, path) = temp_file(std::str::from_utf8(original).unwrap());

    let output = run(command()
        .arg("format")
        .arg("--in-place")
        .arg("true")
        .arg(&path));

    assert!(!output.status.success());
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(
        fs::read_dir(directory.path()).unwrap().count(),
        1,
        "failed formatting should not leave a temporary file"
    );
}

#[cfg(unix)]
#[test]
fn in_place_preserves_permissions_and_symlinks() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let directory = TempDir::with_prefix("jsoncfmt-").unwrap();
    let target = directory.path().join("target.jsonc");
    let link = directory.path().join("link.jsonc");
    fs::write(&target, r#"{"b":2,"a":1}"#).unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
    symlink(&target, &link).unwrap();

    let output = run(command()
        .arg("format")
        .arg("--in-place")
        .arg("true")
        .arg(&link));

    assert_success(&output);
    assert!(
        fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let metadata = fs::metadata(&target).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o777, 0o640);
}

#[test]
fn strict_jsonc_rejects_json5_extensions() {
    for input in [
        "{unquoted: 1}",
        "{'single': 1}",
        r#"{"hex": 0xFF}"#,
        r#"{"missing": 1 "comma": 2}"#,
        r#"{"infinity": Infinity}"#,
    ] {
        let (_directory, path) = temp_file(input);
        let output = run(command().arg("format").arg(&path));
        assert!(
            !output.status.success(),
            "JSON5 input should be rejected: {input}"
        );
    }
}

#[test]
fn stdout_mode_does_not_modify_the_file() {
    let input = r#"{"b":2,"a":1}"#;
    let (_directory, path) = temp_file(input);

    let output = run(command().arg("format").arg(&path));

    assert_success(&output);
    assert_eq!(fs::read_to_string(&path).unwrap(), input);
    assert!(!output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn handles_large_documents_and_rejects_excessive_nesting() {
    let properties = (0..500)
        .rev()
        .map(|index| format!(r#""key{index:03}":{index}"#))
        .collect::<Vec<_>>()
        .join(",");
    let large_document = format!("{{{properties}}}");
    let (_directory, path) = temp_file(&large_document);
    let output = run(command().arg("format").arg(&path));
    assert_success(&output);
    let formatted = String::from_utf8(output.stdout).unwrap();
    assert!(formatted.contains(r#""key000": 0,"#));
    assert!(formatted.contains(r#""key499": 499,"#));

    let mut deep_document = r#"{"leaf":true}"#.to_string();
    for _ in 0..128 {
        deep_document = format!(r#"{{"nested":[{deep_document}]}}"#);
    }
    let (_directory, path) = temp_file(&deep_document);
    let output = run(command().arg("format").arg(&path));

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("container nesting exceeds the supported maximum of 254")
    );
}
