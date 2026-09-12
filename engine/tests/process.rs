use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn command(root: &tempfile::TempDir) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_voxtype-personas"));
    command
        .env("XDG_CONFIG_HOME", root.path().join("config"))
        .env("XDG_DATA_HOME", root.path().join("data"))
        .env("XDG_STATE_HOME", root.path().join("state"))
        .env("XDG_CACHE_HOME", root.path().join("cache"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn run_process(root: &tempfile::TempDir, arguments: &[&str], input: &[u8]) -> std::process::Output {
    let mut child = command(root)
        .args(arguments)
        .spawn()
        .expect("engine should start");
    child
        .stdin
        .take()
        .expect("standard input should be available")
        .write_all(input)
        .expect("test input should be written");
    child.wait_with_output().expect("engine should finish")
}

#[test]
fn raw_profile_preserves_input_bytes_and_keeps_stderr_empty() {
    let root = tempfile::tempdir().expect("temporary XDG root should exist");
    let input = b"  Keep\tthis exactly.\nSecond line.\n";

    let output = run_process(&root, &["process"], input);

    assert!(output.status.success());
    assert_eq!(output.stdout, input);
    assert!(output.stderr.is_empty());
}

#[test]
fn empty_input_is_returned_unchanged() {
    let root = tempfile::tempdir().expect("temporary XDG root should exist");

    let output = run_process(&root, &["process"], b"");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn explicit_raw_profile_does_not_change_the_persisted_selection() {
    let root = tempfile::tempdir().expect("temporary XDG root should exist");

    let output = run_process(&root, &["process", "--profile", "raw"], b"raw text");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"raw text");

    let config_path = root.path().join("config/voxtype-personas/config.toml");
    let configuration = fs::read_to_string(config_path).expect("configuration should exist");
    assert!(configuration.contains("active_profile = \"raw\""));
}

#[test]
fn profiles_list_exposes_draft_ready_and_active_state() {
    let root = tempfile::tempdir().expect("temporary XDG root should exist");

    let output = run_process(&root, &["profiles", "list"], b"");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("profile list should be UTF-8"),
        "  example\tExample profile\tDraft\n* raw\tRaw\tActive\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn draft_profile_activation_is_rejected_without_changing_configuration() {
    let root = tempfile::tempdir().expect("temporary XDG root should exist");

    let output = run_process(&root, &["profiles", "set-active", "example"], b"");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).expect("diagnostic should be UTF-8"),
        "voxtype-personas: profile 'example' is a draft and cannot be activated\n"
    );
    let config_path = root.path().join("config/voxtype-personas/config.toml");
    let configuration = fs::read_to_string(config_path).expect("configuration should exist");
    assert!(configuration.contains("active_profile = \"raw\""));
}

#[test]
fn unavailable_profile_preserves_raw_output_and_keeps_input_out_of_diagnostics() {
    let root = tempfile::tempdir().expect("temporary XDG root should exist");
    let input = b"private dictated content";

    let output = run_process(&root, &["process", "--profile", "example"], input);

    assert!(output.status.success());
    assert_eq!(output.stdout, input);
    assert!(!output.stderr.is_empty());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("private dictated content"));
}
