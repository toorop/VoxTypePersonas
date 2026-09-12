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
fn persisted_draft_active_profile_falls_back_to_raw_output() {
    let root = tempfile::tempdir().expect("temporary XDG root should exist");
    let first_run = run_process(&root, &["process"], b"");
    assert!(first_run.status.success());

    let config_path = root.path().join("config/voxtype-personas/config.toml");
    let configuration = fs::read_to_string(&config_path).expect("configuration should exist");
    fs::write(
        &config_path,
        configuration.replacen(
            "active_profile = \"raw\"",
            "active_profile = \"example\"",
            1,
        ),
    )
    .expect("fixture configuration should be written");
    let input = b"private dictated content";

    let output = run_process(&root, &["process"], input);

    assert!(output.status.success());
    assert_eq!(output.stdout, input);
    let diagnostic = String::from_utf8(output.stderr).expect("diagnostic should be UTF-8");
    assert_eq!(
        diagnostic,
        "voxtype-personas: selected profile is not ready; returned Raw profile output\n"
    );
    assert!(!diagnostic.contains("private dictated content"));
}

#[test]
fn profiles_validate_reads_catalog_files_without_creating_configuration() {
    let root = tempfile::tempdir().expect("temporary XDG root should exist");
    let example = format!("{}/../profiles/example.md", env!("CARGO_MANIFEST_DIR"));

    let output = run_process(&root, &["profiles", "validate", &example], b"");

    assert!(output.status.success());
    assert_eq!(output.stdout, b"Validated 1 portable profile file(s).\n");
    assert!(output.stderr.is_empty());
    assert!(
        !root
            .path()
            .join("config/voxtype-personas/config.toml")
            .exists()
    );
}

#[test]
fn profiles_validate_rejects_duplicate_catalog_ids_without_creating_configuration() {
    let root = tempfile::tempdir().expect("temporary XDG root should exist");
    let fixture_root = format!("{}/tests/fixtures/catalog", env!("CARGO_MANIFEST_DIR"));
    let first = format!("{fixture_root}/duplicate-a.md");
    let second = format!("{fixture_root}/duplicate-b.md");

    let output = run_process(&root, &["profiles", "validate", &first, &second], b"");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"voxtype-personas: portable profile identifier 'duplicate' is duplicated\n"
    );
    assert!(
        !root
            .path()
            .join("config/voxtype-personas/config.toml")
            .exists()
    );
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
