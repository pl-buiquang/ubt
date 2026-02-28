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
