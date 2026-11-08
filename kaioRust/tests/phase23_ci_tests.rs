use std::fs;
use std::path::PathBuf;

fn get_repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // If manifest_dir is kaioRust, parent is repo root
    if manifest_dir.ends_with("kaioRust") {
        manifest_dir.parent().unwrap().to_path_buf()
    } else {
        manifest_dir
    }
}

#[test]
fn test_ci_workflow_file_exists_and_is_valid() {
    let repo_root = get_repo_root();
    let ci_file = repo_root.join(".github").join("workflows").join("ci.yml");
    assert!(
        ci_file.exists(),
        "ci.yml must exist at .github/workflows/ci.yml"
    );

    let content = fs::read_to_string(&ci_file).expect("Failed to read ci.yml");
    assert!(!content.is_empty(), "ci.yml should not be empty");
    assert!(content.contains("name: CI"));
    assert!(content.contains("push:"));
    assert!(content.contains("pull_request:"));
    assert!(content.contains("branches: [main, master, develop]"));
    assert!(content.contains("working-directory: kaioRust"));
}

#[test]
fn test_ci_workflow_has_required_jobs() {
    let repo_root = get_repo_root();
    let ci_file = repo_root.join(".github").join("workflows").join("ci.yml");
    let content = fs::read_to_string(&ci_file).expect("Failed to read ci.yml");

    // All 5 core jobs specified in doc 18
    assert!(content.contains("lint:"), "ci.yml must contain lint job");
    assert!(content.contains("build:"), "ci.yml must contain build job");
    assert!(content.contains("test:"), "ci.yml must contain test job");
    assert!(
        content.contains("security:"),
        "ci.yml must contain security job"
    );
    assert!(
        content.contains("health:"),
        "ci.yml must contain health job"
    );
}

#[test]
fn test_ci_lint_job_specification() {
    let repo_root = get_repo_root();
    let ci_file = repo_root.join(".github").join("workflows").join("ci.yml");
    let content = fs::read_to_string(&ci_file).expect("Failed to read ci.yml");

    assert!(
        content.contains("cargo fmt -- --check"),
        "Lint job must check formatting"
    );
    assert!(
        content.contains("cargo clippy --all-targets --all-features -- -D warnings"),
        "Lint job must enforce zero Clippy warnings"
    );
    assert!(
        content.contains("components: clippy, rustfmt"),
        "Toolchain step must install clippy and rustfmt"
    );
}

#[test]
fn test_ci_build_matrix_specification() {
    let repo_root = get_repo_root();
    let ci_file = repo_root.join(".github").join("workflows").join("ci.yml");
    let content = fs::read_to_string(&ci_file).expect("Failed to read ci.yml");

    assert!(
        content.contains("os: [ubuntu-latest, windows-latest, macos-latest]"),
        "Build job matrix must target ubuntu, windows, and macos"
    );
    assert!(
        content.contains("cargo build --verbose"),
        "Build job must run cargo build"
    );
    assert!(
        content.contains("cargo build --release"),
        "Build job must build release for main/master"
    );
}

#[test]
fn test_ci_test_job_specification() {
    let repo_root = get_repo_root();
    let ci_file = repo_root.join(".github").join("workflows").join("ci.yml");
    let content = fs::read_to_string(&ci_file).expect("Failed to read ci.yml");

    assert!(
        content.contains("os: [ubuntu-latest, windows-latest]"),
        "Test job matrix must cover ubuntu and windows"
    );
    assert!(
        content.contains("cargo test --verbose -- --nocapture"),
        "Test job must run cargo test"
    );
    assert!(
        content.contains("cargo test --release"),
        "Test job must run release tests on ubuntu"
    );
    assert!(
        content.contains("needs: [lint]"),
        "Test job must depend on lint job passing first"
    );
}

#[test]
fn test_ci_health_audit_job_specification() {
    let repo_root = get_repo_root();
    let ci_file = repo_root.join(".github").join("workflows").join("ci.yml");
    let content = fs::read_to_string(&ci_file).expect("Failed to read ci.yml");

    assert!(
        content.contains("./target/release/kaioken memory init"),
        "Health job must initialize memory"
    );
    assert!(
        content.contains("./target/release/kaioken memory scan --no-llm"),
        "Health job must execute deterministic scan"
    );
    assert!(
        content.contains("./target/release/kaioken schedule --daily --json"),
        "Health job must generate daily health report"
    );
    assert!(
        content.contains("actions/upload-artifact@v4"),
        "Health job must upload health-report artifact"
    );
}

#[test]
fn test_maintenance_workflow_specification() {
    let repo_root = get_repo_root();
    let maintenance_file = repo_root
        .join(".github")
        .join("workflows")
        .join("maintenance.yml");
    assert!(
        maintenance_file.exists(),
        "maintenance.yml must exist at .github/workflows/maintenance.yml"
    );

    let content = fs::read_to_string(&maintenance_file).expect("Failed to read maintenance.yml");
    assert!(
        content.contains("cron: '0 6 * * 1'"),
        "Maintenance workflow must trigger on Monday 6 AM UTC"
    );
    assert!(
        content.contains("Full repository scan"),
        "Maintenance must perform full scan"
    );
    assert!(
        content.contains("Health audit"),
        "Maintenance must run health audit"
    );
    assert!(
        content.contains("cargo audit"),
        "Maintenance must run cargo audit"
    );
}

#[test]
fn test_cargo_toml_profiles_and_lints() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = manifest_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml).expect("Failed to read Cargo.toml");

    assert!(
        content.contains("[profile.test]"),
        "Cargo.toml must define [profile.test]"
    );
    assert!(
        content.contains("[profile.dev]"),
        "Cargo.toml must define [profile.dev]"
    );
    assert!(
        content.contains("[profile.release]"),
        "Cargo.toml must define [profile.release]"
    );
    assert!(
        content.contains("[lints.clippy]"),
        "Cargo.toml must configure [lints.clippy]"
    );
}

#[test]
fn test_readme_badges() {
    let repo_root = get_repo_root();
    let root_readme = repo_root.join("README.md");
    let docs_readme = repo_root.join("kaioRust").join("docs").join("README.md");

    assert!(root_readme.exists(), "Root README.md must exist");
    assert!(docs_readme.exists(), "kaioRust/docs/README.md must exist");

    let root_content = fs::read_to_string(&root_readme).expect("Failed to read root README.md");
    let docs_content = fs::read_to_string(&docs_readme).expect("Failed to read docs README.md");

    let expected_ci_badge = "https://github.com/BABTICH/kaioken/actions/workflows/ci.yml/badge.svg";
    let expected_security_badge =
        "https://github.com/BABTICH/kaioken/actions/workflows/ci.yml/badge.svg?event=schedule";

    assert!(
        root_content.contains(expected_ci_badge),
        "Root README.md must contain CI badge"
    );
    assert!(
        root_content.contains(expected_security_badge),
        "Root README.md must contain Security Audit badge"
    );
    assert!(
        docs_content.contains(expected_ci_badge),
        "docs/README.md must contain CI badge"
    );
    assert!(
        docs_content.contains(expected_security_badge),
        "docs/README.md must contain Security Audit badge"
    );
}
