use predicates::prelude::*;
use tempfile::TempDir;

fn ubt() -> assert_cmd::Command {
    #[allow(deprecated)]
    assert_cmd::Command::cargo_bin("ubt").unwrap()
}

// ── Completions ─────────────────────────────────────────────────────────

#[test]
fn completions_bash() {
    ubt()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn completions_zsh() {
    ubt()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef"));
}

#[test]
fn completions_fish() {
    ubt()
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ── Tool list ───────────────────────────────────────────────────────────

#[test]
fn tool_list_shows_plugins() {
    ubt()
        .args(["tool", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("go"))
        .stdout(predicate::str::contains("node"))
        .stdout(predicate::str::contains("python"))
        .stdout(predicate::str::contains("rust"));
}

// ── Info in a Go project ────────────────────────────────────────────────

#[test]
fn info_in_go_project() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module example.com/foo").unwrap();

    ubt()
        .arg("info")
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("go"))
        .stdout(predicate::str::contains("Go projects"));
}

// ── Info in a Node/pnpm project ─────────────────────────────────────────

#[test]
fn info_in_node_pnpm_project() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("package.json"), "{}").unwrap();
    std::fs::write(dir.path().join("pnpm-lock.yaml"), "").unwrap();

    ubt()
        .arg("info")
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("node"))
        .stdout(predicate::str::contains("pnpm"));
}

// ── Config show ─────────────────────────────────────────────────────────

#[test]
fn config_show_with_config() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("ubt.toml"),
        "[project]\ntool = \"go\"\n[commands]\ntest = \"custom-test\"\n",
    )
    .unwrap();

    ubt()
        .args(["config", "show"])
        .current_dir(dir.path())
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("go"))
        .stdout(predicate::str::contains("custom-test"));
}

#[test]
fn config_show_without_config() {
    let dir = TempDir::new().unwrap();

    ubt()
        .args(["config", "show"])
        .current_dir(dir.path())
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("No ubt.toml found"));
}

// ── Commands override via ubt.toml ──────────────────────────────────────

#[test]
fn config_commands_override_is_used() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname=\"x\"\nversion=\"0.1.0\"\nedition=\"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("ubt.toml"),
        "[project]\ntool = \"cargo\"\n[commands]\nbuild = \"echo custom-build-ran\"\n",
    )
    .unwrap();

    ubt()
        .arg("build")
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("custom-build-ran"));
}

// ── Aliases from ubt.toml ───────────────────────────────────────────────

#[test]
fn alias_runs_command() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("ubt.toml"),
        "[aliases]\nhello = \"echo alias-ran\"\n",
    )
    .unwrap();

    ubt()
        .arg("hello")
        .current_dir(dir.path())
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("alias-ran"));
}

#[test]
fn alias_with_args() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("ubt.toml"),
        "[aliases]\ngreet = \"echo {{args}}\"\n",
    )
    .unwrap();

    ubt()
        .args(["greet", "hello", "world"])
        .current_dir(dir.path())
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn unknown_command_errors() {
    let dir = TempDir::new().unwrap();

    ubt()
        .arg("nonexistent")
        .current_dir(dir.path())
        .env_remove("UBT_CONFIG")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown command"));
}

// ── Build in empty dir → error with guidance ────────────────────────────

#[test]
fn build_in_empty_dir_errors() {
    let dir = TempDir::new().unwrap();

    ubt()
        .arg("build")
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not detect project type"));
}

// ── Init creates config ─────────────────────────────────────────────────

#[test]
fn init_creates_config() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module foo").unwrap();

    ubt()
        .arg("init")
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    let content = std::fs::read_to_string(dir.path().join("ubt.toml")).unwrap();
    assert!(content.contains("go"));
}

// ── Verbose flag shows trace ────────────────────────────────────────────

#[test]
fn verbose_shows_detection_trace() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module foo").unwrap();

    // Use info command since it won't try to exec a tool
    ubt()
        .args(["-v", "info"])
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stderr(predicate::str::contains("ubt: detected go"));
}

// ── --tool override ─────────────────────────────────────────────────────

#[test]
fn tool_override_via_flag() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module foo").unwrap();

    ubt()
        .args(["--tool", "node", "info"])
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("node"));
}

// ── Help and version ────────────────────────────────────────────────────

#[test]
fn help_exits_success() {
    ubt().arg("--help").assert().success();
}

#[test]
fn version_exits_success() {
    ubt()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ubt"));
}

// ── Tool doctor ──────────────────────────────────────────────────────────

#[test]
fn tool_doctor_in_rust_project() {
    // cargo is guaranteed to be in PATH when running `cargo test`
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    ubt()
        .args(["tool", "doctor"])
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success();
}

#[test]
fn tool_doctor_in_empty_dir_fails() {
    let dir = TempDir::new().unwrap();

    ubt()
        .args(["tool", "doctor"])
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .failure()
        .stderr(predicate::str::contains("[fail]"));
}

#[test]
fn tool_doctor_warns_on_unknown_alias_target() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("ubt.toml"),
        "[aliases]\nmyalias = \"unknown-command-xyz\"\n",
    )
    .unwrap();

    ubt()
        .args(["tool", "doctor"])
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("[warn]"))
        .stdout(predicate::str::contains("unknown-command-xyz"));
}

// ── Quiet mode ───────────────────────────────────────────────────────────

#[test]
fn tool_list_quiet_produces_no_output() {
    ubt()
        .args(["--quiet", "tool", "list"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn info_quiet_produces_no_stdout() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module foo").unwrap();

    ubt()
        .args(["--quiet", "info"])
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// ── Init command ─────────────────────────────────────────────────────────

#[test]
fn init_already_exists_prints_message() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("ubt.toml"), "[project]\ntool = \"go\"\n").unwrap();

    ubt()
        .arg("init")
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("already exists"));
}

// ── Config show with aliases ─────────────────────────────────────────────

#[test]
fn config_show_with_aliases_section() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("ubt.toml"),
        "[project]\ntool = \"go\"\n\n[aliases]\nhello = \"echo hello\"\n",
    )
    .unwrap();

    ubt()
        .args(["config", "show"])
        .current_dir(dir.path())
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("Aliases:"))
        .stdout(predicate::str::contains("hello"));
}

// ── Verbose output ───────────────────────────────────────────────────────

#[test]
fn verbose_shows_config_source() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module foo").unwrap();
    std::fs::write(dir.path().join("ubt.toml"), "[project]\ntool = \"go\"\n").unwrap();

    ubt()
        .args(["-v", "info"])
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stderr(predicate::str::contains("ubt: config:"));
}

#[test]
fn verbose_shows_tool_source_cli_flag() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module foo").unwrap();

    // "tool source" is printed for regular commands (build/test/etc.), not for `info`
    // Use `build` so the verbose path is exercised; don't assert success since
    // `go build` may not be installed on the test machine.
    ubt()
        .args(["-v", "--tool", "go", "build"])
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .stderr(predicate::str::contains(
            "ubt: tool source: CLI --tool flag",
        ));
}

// ── Tool docs ────────────────────────────────────────────────────────────

#[test]
fn tool_docs_prints_homepage_url() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("go.mod"), "module foo").unwrap();

    ubt()
        .args(["tool", "docs"])
        .current_dir(dir.path())
        .env_remove("UBT_TOOL")
        .env_remove("UBT_CONFIG")
        .assert()
        .success()
        .stdout(predicate::str::contains("go.dev"));
}
