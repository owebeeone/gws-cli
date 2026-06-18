const RELEASE_WORKFLOW: &str = include_str!("../.github/workflows/release.yml");
const DIST_WORKSPACE: &str = include_str!("../dist-workspace.toml");

#[test]
fn release_workflow_only_runs_for_explicit_releases() {
    assert!(RELEASE_WORKFLOW.contains("release:"));
    assert!(RELEASE_WORKFLOW.contains("types: [published]"));
    assert!(RELEASE_WORKFLOW.contains("workflow_dispatch"));
    assert!(!RELEASE_WORKFLOW.contains("pull_request:"));
    assert!(!RELEASE_WORKFLOW.contains("branches:"));
}

#[test]
fn release_workflow_builds_rust_split_platform_parity() {
    assert!(DIST_WORKSPACE.contains("aarch64-apple-darwin"));
    assert!(DIST_WORKSPACE.contains("x86_64-apple-darwin"));
    assert!(DIST_WORKSPACE.contains("aarch64-unknown-linux-gnu"));
    assert!(DIST_WORKSPACE.contains("x86_64-unknown-linux-gnu"));
    assert!(DIST_WORKSPACE.contains("x86_64-pc-windows-msvc"));
}

#[test]
fn release_workflow_uses_cargo_dist_installers() {
    assert!(RELEASE_WORKFLOW.contains("cargo-dist-installer.sh"));
    assert!(RELEASE_WORKFLOW.contains("dist host --steps=create"));
    assert!(RELEASE_WORKFLOW.contains("dist build"));
}

#[test]
fn release_workflow_uploads_and_attests_release_assets() {
    assert!(RELEASE_WORKFLOW.contains("actions/attest-build-provenance"));
    assert!(RELEASE_WORKFLOW.contains("artifacts/*.sha256"));
    assert!(RELEASE_WORKFLOW.contains("artifacts/sha256.sum"));
    assert!(RELEASE_WORKFLOW.contains("gh release edit"));
    assert!(RELEASE_WORKFLOW.contains("gh release upload"));
}
