use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

#[test]
fn help_flags_print_usage() {
    let temp = TempDir::new("help");
    for flag in ["--help", "-h"] {
        let output = gwz(temp.path()).arg(flag).output().unwrap();

        assert_success(&output);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Usage: gwz"));
        assert!(stdout.contains("-h, --help"));
        assert!(stdout.contains("init"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn help_command_prints_detailed_subcommand_usage() {
    let temp = TempDir::new("help-command");
    let output = gwz(temp.path()).args(["help", "status"]).output().unwrap();

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: gwz status"));
    assert!(stdout.contains("--no-combined"));
    assert!(stdout.contains("--porcelain"));
    assert!(output.stderr.is_empty());
}

#[test]
fn subcommand_help_prints_detailed_subcommand_usage() {
    let temp = TempDir::new("subcommand-help");
    let output = gwz(temp.path())
        .args(["status", "--help"])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: gwz status"));
    assert!(stdout.contains("--no-files"));
    assert!(stdout.contains("--no-branches"));
    assert!(output.stderr.is_empty());
}

#[test]
fn init_help_explains_gwz_workspace_basics() {
    let temp = TempDir::new("init-help");
    let output = gwz(temp.path()).args(["help", "init"]).output().unwrap();

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A GWZ workspace is a local directory"));
    assert!(stdout.contains("gwz.conf/gwz.yml"));
    assert!(stdout.contains("gwz.conf/gwz.lock.yml"));
    assert!(stdout.contains("Examples:"));
    assert!(stdout.contains("gwz init git@github.com:org/app.git"));
}

#[test]
fn every_command_help_has_semantics_and_examples() {
    let temp = TempDir::new("all-help");
    for command in [
        &["init"][..],
        &["add"][..],
        &["repo"][..],
        &["repo", "create"][..],
        &["status"][..],
        &["snapshot"][..],
        &["tag"][..],
        &["materialize"][..],
        &["pull"][..],
        &["push"][..],
    ] {
        let mut args = vec!["help"];
        args.extend_from_slice(command);
        let output = gwz(temp.path()).args(args).output().unwrap();

        assert_success(&output);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Examples:"),
            "missing examples for {:?}:\n{}",
            command,
            stdout
        );
        assert!(
            stdout.contains("workspace"),
            "missing workspace semantics for {:?}:\n{}",
            command,
            stdout
        );
    }
}

#[test]
fn init_status_snapshot_tag_and_materialize_targets_work() {
    let temp = TempDir::new("init-status");
    let remote = RemoteFixture::new("init-status-source");
    let first = remote.commit_and_push("README.md", "one", "initial");

    let init = gwz(temp.path())
        .args([
            "--root",
            temp.path_str(),
            "--jsonl",
            "init",
            "--path",
            "repos",
            remote.url(),
        ])
        .output()
        .unwrap();
    assert_success(&init);
    // Events stream live first; the response record is the terminal line.
    let init_lines = json_lines(&init);
    let init_response = init_lines
        .iter()
        .find(|line| line["kind"] == "response")
        .expect("jsonl carries a response record");
    assert_eq!(init_response["meta"]["aggregate_status"], "Ok");

    let status_root = gwz(temp.path())
        .args([
            "--root",
            temp.path_str(),
            "--json",
            "status",
            "--no-combined",
        ])
        .output()
        .unwrap();
    assert_success(&status_root);
    assert_eq!(json(&status_root)["members"][0]["status"], "Ok");

    let status_member = gwz(&temp.path().join("repos/remote"))
        .args(["--json", "status", "--no-combined"])
        .output()
        .unwrap();
    assert_success(&status_member);
    assert_eq!(json(&status_member)["meta"]["aggregate_status"], "Ok");

    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "snapshot", "snap_first"])
            .output()
            .unwrap(),
    );
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "tag", "tag_first"])
            .output()
            .unwrap(),
    );

    fs::remove_dir_all(temp.path().join("repos/remote")).unwrap();
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "materialize", "--lock"])
            .output()
            .unwrap(),
    );
    assert_eq!(
        repo_head(&temp.path().join("repos/remote")),
        Some(first.clone())
    );

    let second = remote.commit_and_push("README.md", "two", "second");
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "pull", "--head"])
            .output()
            .unwrap(),
    );
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "snapshot", "snap_second"])
            .output()
            .unwrap(),
    );
    assert_success(
        &gwz(temp.path())
            .args([
                "--root",
                temp.path_str(),
                "--force",
                "materialize",
                "--snapshot",
                "snap_first",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(repo_head(&temp.path().join("repos/remote")), Some(first));
    assert_success(
        &gwz(temp.path())
            .args([
                "--root",
                temp.path_str(),
                "--force",
                "pull",
                "--snapshot",
                "snap_second",
            ])
            .output()
            .unwrap(),
    );
    assert_eq!(repo_head(&temp.path().join("repos/remote")), Some(second));
}

#[test]
fn status_reports_unmaterialized_members_with_completion_hint() {
    let temp = TempDir::new("status-unmaterialized");
    let remote = RemoteFixture::new("status-unmaterialized-source");
    remote.commit_and_push("README.md", "one", "initial");

    assert_success(
        &gwz(temp.path())
            .args([
                "--root",
                temp.path_str(),
                "init",
                "--path",
                "repos",
                remote.url(),
            ])
            .output()
            .unwrap(),
    );

    // Simulate a naive `git clone` of the workspace root: gwz.conf is present
    // but the member working tree was never materialized.
    fs::remove_dir_all(temp.path().join("repos/remote")).unwrap();

    let status = gwz(temp.path())
        .args(["--root", temp.path_str(), "status"])
        .output()
        .unwrap();
    assert_success(&status);
    let stdout = String::from_utf8_lossy(&status.stdout);
    assert!(
        stdout.contains("not materialized"),
        "expected a not-materialized notice, got:\n{stdout}"
    );
    assert!(
        stdout.contains("gwz materialize --lock"),
        "expected a completion hint, got:\n{stdout}"
    );

    let status_json = gwz(temp.path())
        .args(["--root", temp.path_str(), "--json", "status"])
        .output()
        .unwrap();
    assert_success(&status_json);
    let member = &json(&status_json)["members"][0];
    assert_eq!(member["status"], "Noop");
    assert_eq!(member["lock_match"], "Missing");
    assert_eq!(member["state"]["materialized"], false);
}

#[test]
fn pull_head_and_push_work_with_local_remote() {
    let temp = TempDir::new("pull-push");
    let remote = RemoteFixture::new("pull-push-source");
    let first = remote.commit_and_push("README.md", "one", "initial");
    assert_success(
        &gwz(temp.path())
            .args([
                "--root",
                temp.path_str(),
                "init",
                "--path",
                "repos",
                remote.url(),
            ])
            .output()
            .unwrap(),
    );

    let second = remote.commit_and_push("README.md", "two", "second");
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "pull", "--head"])
            .output()
            .unwrap(),
    );
    assert_eq!(repo_head(&temp.path().join("repos/remote")), Some(second));

    let local = commit_file(
        &temp.path().join("repos/remote"),
        "LOCAL.md",
        "local",
        "local",
    );
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "push", "--remote", "origin"])
            .output()
            .unwrap(),
    );
    assert_ne!(local, first);
    assert_eq!(
        repo_ref(Path::new(remote.url()), "refs/heads/main"),
        Some(local)
    );
}

#[test]
fn pull_head_skips_member_without_fetch_remote_and_streams_events() {
    let temp = TempDir::new("pull-no-fetch-jsonl");
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "init"])
            .output()
            .unwrap(),
    );

    create_repo_with_commit(&temp.path().join("local-repo"));
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "add", "./local-repo"])
            .output()
            .unwrap(),
    );

    let remote = RemoteFixture::new("pull-no-fetch-source");
    remote.commit_and_push("README.md", "one", "initial");
    git2::Repository::clone(remote.url(), temp.path().join("remote")).unwrap();
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "add", "./remote"])
            .output()
            .unwrap(),
    );

    let human = gwz(temp.path())
        .args(["--root", temp.path_str(), "pull", "--head"])
        .output()
        .unwrap();
    assert_success(&human);
    let stdout = String::from_utf8_lossy(&human.stdout);
    assert!(
        stdout.contains("local-repo"),
        "missing local member path:\n{stdout}"
    );
    assert!(
        stdout.contains("no fetch remote configured; skipping pull"),
        "missing no-remote skip message:\n{stdout}"
    );

    let jsonl = gwz(temp.path())
        .args(["--root", temp.path_str(), "--jsonl", "pull", "--head"])
        .output()
        .unwrap();
    assert_success(&jsonl);
    assert_jsonl_lifecycle(&jsonl);
    let lines = json_lines(&jsonl);
    let response = lines
        .iter()
        .find(|line| line["kind"] == "response")
        .expect("jsonl carries a response record");
    let members = response["members"].as_array().unwrap();
    let local = members
        .iter()
        .find(|member| member["member_path"] == "local-repo")
        .expect("local repo member is reported");
    assert_eq!(local["status"], "Noop");
    assert_eq!(
        local["planned"]["message"],
        "no fetch remote configured; skipping pull"
    );
    assert!(
        members
            .iter()
            .any(|member| member["member_path"] == "remote"),
        "remote member is still reported: {members:?}"
    );
}

#[test]
fn push_streams_member_lifecycle_events_as_jsonl() {
    let temp = TempDir::new("push-jsonl");
    let remote = RemoteFixture::new("push-jsonl-source");
    remote.commit_and_push("README.md", "one", "initial");
    assert_success(
        &gwz(temp.path())
            .args([
                "--root",
                temp.path_str(),
                "init",
                "--path",
                "repos",
                remote.url(),
            ])
            .output()
            .unwrap(),
    );
    let local = commit_file(
        &temp.path().join("repos/remote"),
        "LOCAL.md",
        "local",
        "local",
    );

    let jsonl = gwz(temp.path())
        .args([
            "--root",
            temp.path_str(),
            "--jsonl",
            "push",
            "--remote",
            "origin",
        ])
        .output()
        .unwrap();

    assert_success(&jsonl);
    assert_jsonl_lifecycle(&jsonl);
    let lines = json_lines(&jsonl);
    let response = lines
        .iter()
        .find(|line| line["kind"] == "response")
        .expect("jsonl carries a response record");
    assert_eq!(response["meta"]["aggregate_status"], "Ok");
    assert_eq!(
        repo_ref(Path::new(remote.url()), "refs/heads/main"),
        Some(local)
    );
}

#[test]
fn add_create_and_dry_run_commands_work() {
    let temp = TempDir::new("add-create");
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "init"])
            .output()
            .unwrap(),
    );

    let existing = temp.path().join("repos/existing");
    create_repo_with_commit(&existing);
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "add", existing.to_str().unwrap()])
            .output()
            .unwrap(),
    );
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "repo", "create", "repos/empty"])
            .output()
            .unwrap(),
    );
    let dry_run = gwz(temp.path())
        .args([
            "--root",
            temp.path_str(),
            "--dry-run",
            "--json",
            "materialize",
            "--lock",
        ])
        .output()
        .unwrap();
    assert_success(&dry_run);
    assert_eq!(json(&dry_run)["meta"]["aggregate_status"], "Accepted");
}

#[test]
fn add_existing_repo_accepts_plain_relative_path_inside_workspace() {
    let temp = TempDir::new("add-relative");
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "init"])
            .output()
            .unwrap(),
    );
    create_repo_with_commit(&temp.path().join("local-repo"));

    let output = gwz(temp.path())
        .args(["add", "./local-repo"])
        .output()
        .unwrap();

    assert_success(&output);
    let status_json = gwz(temp.path())
        .args(["--root", temp.path_str(), "--json", "status"])
        .output()
        .unwrap();
    assert_success(&status_json);
    assert_eq!(
        json(&status_json)["members"][0]["member_path"],
        "local-repo"
    );
}

#[test]
fn combined_status_succeeds_by_default() {
    let temp = TempDir::new("combined-status");
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "init"])
            .output()
            .unwrap(),
    );

    let output = gwz(temp.path())
        .args(["--root", temp.path_str(), "status"])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Workspace root on branch main"), "{stdout}");
}

#[test]
fn status_without_workspace_names_missing_config_file() {
    let temp = TempDir::new("missing-workspace");

    let output = gwz(temp.path()).arg("status").output().unwrap();

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("WorkspaceNotFound"), "{stderr}");
    assert!(stderr.contains("gwz.conf/gwz.yml missing"), "{stderr}");
}

#[test]
fn status_reports_branch_and_untracked_files() {
    let temp = TempDir::new("status-files");
    let remote = RemoteFixture::new("status-files-source");
    remote.commit_and_push("README.md", "one", "initial");
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "init", remote.url()])
            .output()
            .unwrap(),
    );
    fs::write(temp.path().join("remote/foo.txt"), "untracked").unwrap();

    let output = gwz(temp.path())
        .args(["--root", temp.path_str(), "status"])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Workspace root on branch main"), "{stdout}");
    assert!(stdout.contains("All members on branch main"), "{stdout}");
    assert!(stdout.contains("No commits yet"), "{stdout}");
    assert!(!stdout.contains("On workspace branch"), "{stdout}");
    assert!(!stdout.contains("workspace ."), "{stdout}");
    assert!(!stdout.contains("mem_remote"), "{stdout}");
    assert!(stdout.contains("Changes to be committed:"), "{stdout}");
    assert!(stdout.contains("Untracked files:"), "{stdout}");
    assert!(stdout.contains("A  gwz.conf/gwz.yml"), "{stdout}");
    assert!(stdout.contains("?? remote/foo.txt"), "{stdout}");
}

#[test]
fn combined_status_separates_workspace_root_branch_from_member_branches() {
    let temp = TempDir::new("status-branch-groups");
    let good = RemoteFixture::new_named("status-branch-good", "good");
    let bad = RemoteFixture::new_named("status-branch-bad", "bad");
    good.commit_and_push("README.md", "one", "initial");
    bad.commit_and_push("README.md", "one", "initial");
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "init", good.url(), bad.url()])
            .output()
            .unwrap(),
    );
    set_unborn_branch(temp.path(), "foo");
    checkout_branch(&temp.path().join("bad"), "foo");

    let output = gwz(temp.path())
        .args(["--root", temp.path_str(), "status"])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Workspace root on branch foo"), "{stdout}");
    assert!(stdout.contains("good on branch main"), "{stdout}");
    assert!(stdout.contains("bad on branch foo"), "{stdout}");
    assert!(!stdout.contains("On workspace branch foo"), "{stdout}");
}

#[test]
fn no_combined_status_reports_per_repo_file_changes() {
    let temp = TempDir::new("status-no-combined-files");
    let remote = RemoteFixture::new("status-no-combined-source");
    remote.commit_and_push("README.md", "one", "initial");
    assert_success(
        &gwz(temp.path())
            .args(["--root", temp.path_str(), "init", remote.url()])
            .output()
            .unwrap(),
    );
    fs::write(temp.path().join("remote/foo.txt"), "untracked").unwrap();

    let output = gwz(temp.path())
        .args(["--root", temp.path_str(), "status", "--no-combined"])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Workspace root"), "{stdout}");
    assert!(stdout.contains("Workspace root on branch main"), "{stdout}");
    assert!(!stdout.contains("workspace ."), "{stdout}");
    assert!(!stdout.contains("mem_remote"), "{stdout}");
    assert!(stdout.contains("Changes to be committed:"), "{stdout}");
    assert!(stdout.contains("A  gwz.conf/gwz.yml"), "{stdout}");
    assert!(stdout.contains("remote on branch main"), "{stdout}");
    assert!(stdout.contains("Untracked files:"), "{stdout}");
    assert!(stdout.contains("?? remote/foo.txt"), "{stdout}");
}

#[test]
fn pull_head_dirty_member_blocks_partial_mutation() {
    let temp = TempDir::new("pull-atomic");
    let good = RemoteFixture::new_named("pull-atomic-good", "good");
    let good_first = good.commit_and_push("README.md", "one", "initial");
    let bad = RemoteFixture::new_named("pull-atomic-bad", "bad");
    bad.commit_and_push("README.md", "one", "initial");
    assert_success(
        &gwz(temp.path())
            .args([
                "--root",
                temp.path_str(),
                "init",
                "--path",
                "repos",
                good.url(),
                bad.url(),
            ])
            .output()
            .unwrap(),
    );
    let good_second = good.commit_and_push("README.md", "two", "second");
    fs::write(temp.path().join("repos/bad/README.md"), "dirty").unwrap();

    let output = gwz(temp.path())
        .args(["--root", temp.path_str(), "pull", "--head"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert_eq!(repo_head(&temp.path().join("repos/good")), Some(good_first));
    assert_ne!(
        repo_head(&temp.path().join("repos/good")),
        Some(good_second)
    );
}

#[test]
fn clone_completes_workspace_from_root_url() {
    let temp = TempDir::new("clone-from-url");
    let remote = RemoteFixture::new("clone-from-url-source");
    let commit = remote.commit_and_push("README.md", "one", "initial");

    // Create and commit a source workspace so its root repo can be cloned.
    let origin = temp.path().join("origin");
    fs::create_dir_all(&origin).unwrap();
    assert_success(
        &gwz(temp.path())
            .args([
                "--root",
                origin.to_str().unwrap(),
                "init",
                "--path",
                "repos",
                remote.url(),
            ])
            .output()
            .unwrap(),
    );
    commit_workspace_root(&origin);

    // Clone the whole workspace in one shot.
    let target = temp.path().join("target");
    let clone = gwz(temp.path())
        .args(["clone", origin.to_str().unwrap(), target.to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&clone);

    // gwz.conf came over with the clone and the member was materialized at the
    // locked commit.
    assert!(target.join("gwz.conf/gwz.yml").is_file());
    assert_eq!(repo_head(&target.join("repos/remote")), Some(commit));

    // A second clone into the same target must refuse rather than corrupt it.
    let again = gwz(temp.path())
        .args(["clone", origin.to_str().unwrap(), target.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!again.status.success());
}

#[test]
fn clone_streams_member_lifecycle_events_as_jsonl() {
    let temp = TempDir::new("clone-jsonl");
    let remote = RemoteFixture::new("clone-jsonl-source");
    remote.commit_and_push("README.md", "one", "initial");
    let origin = temp.path().join("origin");
    fs::create_dir_all(&origin).unwrap();
    assert_success(
        &gwz(temp.path())
            .args([
                "--root",
                origin.to_str().unwrap(),
                "init",
                "--path",
                "repos",
                remote.url(),
            ])
            .output()
            .unwrap(),
    );
    commit_workspace_root(&origin);

    let target = temp.path().join("target");
    let out = gwz(temp.path())
        .args([
            "--jsonl",
            "clone",
            origin.to_str().unwrap(),
            target.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out);

    // The stream carries the operation/member lifecycle as `event` records.
    // (Transfer-progress events only fire on network transports, not local
    // clones, so we assert the deterministic lifecycle here.)
    let event_kinds: Vec<String> = json_lines(&out)
        .iter()
        .filter(|line| line["kind"] == "event")
        .map(|line| line["event_kind"].as_str().unwrap().to_owned())
        .collect();
    for expected in [
        "OperationStarted",
        "MemberStarted",
        "MemberFinished",
        "OperationFinished",
    ] {
        assert!(
            event_kinds.iter().any(|kind| kind == expected),
            "missing {expected} in {event_kinds:?}"
        );
    }
}

fn commit_workspace_root(root: &Path) {
    let repo = git2::Repository::open(root).unwrap();
    let mut index = repo.index().unwrap();
    index
        .add_all(["."], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let signature = git2::Signature::now("GWZ Test", "gwz@example.invalid").unwrap();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "init workspace",
        &tree,
        &[],
    )
    .unwrap();
}

fn gwz(cwd: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_gwz"));
    command.current_dir(cwd);
    command
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

fn json_lines(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn assert_jsonl_lifecycle(output: &Output) {
    let event_kinds: Vec<String> = json_lines(output)
        .iter()
        .filter(|line| line["kind"] == "event")
        .map(|line| line["event_kind"].as_str().unwrap().to_owned())
        .collect();
    for expected in [
        "OperationStarted",
        "MemberStarted",
        "MemberFinished",
        "OperationFinished",
    ] {
        assert!(
            event_kinds.iter().any(|kind| kind == expected),
            "missing {expected} in {event_kinds:?}"
        );
    }
}

fn create_repo_with_commit(path: &Path) -> String {
    fs::create_dir_all(path).unwrap();
    let repo = git2::Repository::init_opts(path, init_opts()).unwrap();
    commit_in_repo(&repo, "README.md", "one", "initial")
}

fn commit_file(repo_path: &Path, relative_path: &str, content: &str, message: &str) -> String {
    let repo = git2::Repository::open(repo_path).unwrap();
    commit_in_repo(&repo, relative_path, content, message)
}

fn commit_in_repo(
    repo: &git2::Repository,
    relative_path: &str,
    content: &str,
    message: &str,
) -> String {
    let workdir = repo.workdir().unwrap();
    fs::write(workdir.join(relative_path), content).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new(relative_path)).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let signature = git2::Signature::now("GWZ Test", "gwz@example.invalid").unwrap();
    let parents = repo
        .head()
        .ok()
        .and_then(|head| head.target())
        .map(|oid| repo.find_commit(oid).unwrap())
        .into_iter()
        .collect::<Vec<_>>();
    let parent_refs = parents.iter().collect::<Vec<_>>();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parent_refs,
    )
    .unwrap()
    .to_string()
}

fn repo_head(repo_path: &Path) -> Option<String> {
    repo_ref(repo_path, "HEAD")
}

fn repo_ref(repo_path: &Path, ref_name: &str) -> Option<String> {
    git2::Repository::open(repo_path)
        .unwrap()
        .revparse_single(ref_name)
        .ok()
        .map(|object| object.id().to_string())
}

fn set_unborn_branch(repo_path: &Path, branch: &str) {
    git2::Repository::open(repo_path)
        .unwrap()
        .set_head(&format!("refs/heads/{branch}"))
        .unwrap();
}

fn checkout_branch(repo_path: &Path, branch: &str) {
    let repo = git2::Repository::open(repo_path).unwrap();
    let commit = repo.head().unwrap().peel_to_commit().unwrap();
    repo.branch(branch, &commit, false).unwrap();
    repo.set_head(&format!("refs/heads/{branch}")).unwrap();
    let mut checkout = git2::build::CheckoutBuilder::new();
    checkout.safe();
    repo.checkout_head(Some(&mut checkout)).unwrap();
}

fn init_opts() -> &'static mut git2::RepositoryInitOptions {
    Box::leak(Box::new({
        let mut opts = git2::RepositoryInitOptions::new();
        opts.initial_head("main");
        opts
    }))
}

struct RemoteFixture {
    _temp: TempDir,
    source: PathBuf,
    remote: PathBuf,
}

impl RemoteFixture {
    fn new(prefix: &str) -> Self {
        Self::new_named(prefix, "remote")
    }

    fn new_named(prefix: &str, repo_name: &str) -> Self {
        let temp = TempDir::new(prefix);
        let source = temp.path().join("source");
        let remote = temp.path().join(format!("{repo_name}.git"));
        fs::create_dir_all(&source).unwrap();
        git2::Repository::init_opts(&source, init_opts()).unwrap();
        init_bare_main(&remote);
        git2::Repository::open(&source)
            .unwrap()
            .remote("origin", remote.to_str().unwrap())
            .unwrap();
        Self {
            _temp: temp,
            source,
            remote,
        }
    }

    fn url(&self) -> &str {
        self.remote.to_str().unwrap()
    }

    fn commit_and_push(&self, relative_path: &str, content: &str, message: &str) -> String {
        let commit = commit_file(&self.source, relative_path, content, message);
        let repo = git2::Repository::open(&self.source).unwrap();
        let mut remote = repo.find_remote("origin").unwrap();
        remote
            .push(&["refs/heads/main:refs/heads/main"], None)
            .unwrap();
        commit
    }
}

fn init_bare_main(path: &Path) {
    let repo = git2::Repository::init_bare(path).unwrap();
    repo.set_head("refs/heads/main").unwrap();
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "gwz-cli-it-{prefix}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn path_str(&self) -> &str {
        self.path.to_str().unwrap()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
