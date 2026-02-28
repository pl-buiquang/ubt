//! End-to-end Docker tests.
//!
//! These tests require Docker to be installed and running.
//! Run with: `cargo test --test e2e -- --ignored`
//!
//! Each test builds a Docker image with a tiny project and the UBT binary,
//! then runs core UBT commands inside the container.

use std::process::Command;

fn docker_available() -> bool {
    Command::new("docker")
        .arg("info")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn build_ubt_binary() -> String {
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .output()
        .expect("failed to build ubt");
    assert!(output.status.success(), "cargo build --release failed");

    let binary = std::env::current_dir()
        .unwrap()
        .join("target/release/ubt");
    assert!(binary.exists(), "ubt binary not found at {}", binary.display());
    binary.to_string_lossy().to_string()
}

fn run_docker_build(ecosystem: &str) -> bool {
    let context_dir = format!("tests/e2e/docker/{ecosystem}");
    let tag = format!("ubt-e2e-{ecosystem}");

    // Copy the ubt binary into the Docker build context
    let ubt_binary = build_ubt_binary();
    let dest = format!("{context_dir}/ubt");
    std::fs::copy(&ubt_binary, &dest).expect("failed to copy ubt binary");

    let output = Command::new("docker")
        .args(["build", "-t", &tag, &context_dir])
        .output()
        .expect("failed to run docker build");

    // Clean up the copied binary
    let _ = std::fs::remove_file(&dest);

    if !output.status.success() {
        eprintln!(
            "Docker build failed for {ecosystem}:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    output.status.success()
}

#[test]
#[ignore]
fn e2e_go() {
    if !docker_available() {
        eprintln!("Docker not available, skipping");
        return;
    }
    assert!(run_docker_build("go"), "Go E2E test failed");
}

#[test]
#[ignore]
fn e2e_node_npm() {
    if !docker_available() {
        eprintln!("Docker not available, skipping");
        return;
    }
    assert!(run_docker_build("node-npm"), "Node/npm E2E test failed");
}

#[test]
#[ignore]
fn e2e_node_pnpm() {
    if !docker_available() {
        eprintln!("Docker not available, skipping");
        return;
    }
    assert!(run_docker_build("node-pnpm"), "Node/pnpm E2E test failed");
}

#[test]
#[ignore]
fn e2e_rust() {
    if !docker_available() {
        eprintln!("Docker not available, skipping");
        return;
    }
    assert!(run_docker_build("rust"), "Rust E2E test failed");
}

#[test]
#[ignore]
fn e2e_python() {
    if !docker_available() {
        eprintln!("Docker not available, skipping");
        return;
    }
    assert!(run_docker_build("python"), "Python E2E test failed");
}

#[test]
#[ignore]
fn e2e_ruby() {
    if !docker_available() {
        eprintln!("Docker not available, skipping");
        return;
    }
    assert!(run_docker_build("ruby"), "Ruby E2E test failed");
}
