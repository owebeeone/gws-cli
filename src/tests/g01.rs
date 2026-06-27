use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[test]
pub(crate) fn parses_init_workspace_with_root() {
    let invocation = parse_args_with_request_id(
        strings(["--root", "/tmp/gwz-test", "init"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    assert_eq!(invocation.output, OutputMode::Human);
    let CliRequest::CreateWorkspace(request) = invocation.request else {
        panic!("expected create workspace");
    };
    assert_eq!(request.workspace_root, "/tmp/gwz-test");
    assert_eq!(request.meta.request_id, "req_test");
}

#[test]
pub(crate) fn parses_init_update_bootstrap_with_root() {
    let invocation = parse_args_with_request_id(
        strings(["--root", "/tmp/gwz-test", "init", "--update"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    assert_eq!(invocation.output, OutputMode::Human);
    let CliRequest::UpdateBootstrap { meta } = invocation.request else {
        panic!("expected update bootstrap");
    };
    assert_eq!(meta.request_id, "req_test");
    assert_eq!(
        meta.workspace.unwrap().root,
        Some("/tmp/gwz-test".to_owned())
    );
}

#[test]
pub(crate) fn init_update_rejects_sources_and_path_prefix() {
    let with_source = parse_args_with_request_id(
        strings(["init", "--update", "git@github.com:org/repo.git"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap_err();
    assert!(
        with_source
            .message
            .contains("--update cannot be combined with source URLs")
    );

    let with_path = parse_args_with_request_id(
        strings(["init", "--update", "--path", "repos"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap_err();
    assert!(
        with_path
            .message
            .contains("--update cannot be combined with --path")
    );
}

#[test]
pub(crate) fn parses_init_sources_from_plain_urls() {
    let invocation = parse_args_with_request_id(
        strings([
            "init",
            "git@github.com:org/repo-a.git",
            "https://github.com/org/repo-b",
        ]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::InitFromSources(request) = invocation.request else {
        panic!("expected init from sources");
    };
    assert_eq!(request.workspace_root, "/cwd");
    assert_eq!(request.sources[0].url, "git@github.com:org/repo-a.git");
    assert_eq!(request.sources[0].path, None);
    assert_eq!(request.sources[1].url, "https://github.com/org/repo-b");
}

#[test]
pub(crate) fn parses_clone_with_explicit_and_derived_target() {
    let with_dir = parse_args_with_request_id(
        strings(["clone", "git@github.com:org/workspace.git", "work/demo"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();
    let CliRequest::CloneWorkspace { url, target, .. } = with_dir.request else {
        panic!("expected clone workspace");
    };
    assert_eq!(url, "git@github.com:org/workspace.git");
    assert_eq!(target, "work/demo");

    let derived = parse_args_with_request_id(
        strings(["clone", "https://github.com/org/workspace.git"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();
    let CliRequest::CloneWorkspace { target, .. } = derived.request else {
        panic!("expected clone workspace");
    };
    assert_eq!(target, "workspace");
}

#[test]
pub(crate) fn clone_rejects_dry_run() {
    let error = parse_args_with_request_id(
        strings(["--dry-run", "clone", "https://github.com/org/workspace.git"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap_err();
    assert!(
        error
            .message
            .contains("--dry-run is not supported for clone")
    );
}

#[test]
pub(crate) fn parses_init_path_prefix_for_initial_sources() {
    let invocation = parse_args_with_request_id(
        strings([
            "init",
            "--path",
            "repos",
            "git@github.com:org/repo-a.git",
            "https://github.com/org/repo-b",
        ]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::InitFromSources(request) = invocation.request else {
        panic!("expected init from sources");
    };
    assert_eq!(request.sources[0].path, Some("repos/repo-a".to_owned()));
    assert_eq!(request.sources[1].path, Some("repos/repo-b".to_owned()));
}

#[test]
pub(crate) fn parses_global_selection_policy_and_output_flags() {
    let invocation = parse_args_with_request_id(
        strings([
            "--root",
            "/ws",
            "--member",
            "mem_app",
            "--member-path",
            "repos/lib",
            "--dry-run",
            "--partial",
            "--force",
            "--sync",
            "reset",
            "--remote",
            "origin",
            "--jobs",
            "4",
            "--json",
            "status",
        ]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    assert_eq!(invocation.output, OutputMode::Json);
    let CliRequest::Status(request) = invocation.request else {
        panic!("expected status");
    };
    let workspace = request.meta.workspace.unwrap();
    assert_eq!(workspace.root, Some("/ws".to_owned()));
    let selection = request.meta.selection.unwrap();
    assert_eq!(selection.targets, vec!["mem_app", "repos/lib"]);
    assert!(selection.exclude_targets.is_empty());
    assert!(selection.member_ids.is_empty());
    assert!(selection.paths.is_empty());
    let policy = request.meta.policy.unwrap();
    assert_eq!(policy.partial, Some(gwz_core::PartialBehavior::Partial));
    assert_eq!(
        policy.destructive,
        Some(gwz_core::DestructiveBehavior::Allow)
    );
    assert_eq!(policy.sync, Some(gwz_core::SyncBehavior::Reset));
    assert_eq!(policy.remote, Some("origin".to_owned()));
    assert_eq!(policy.concurrency, Some(4));
    assert_eq!(request.meta.dry_run, Some(true));
}

#[test]
pub(crate) fn capture_verb_parses_with_selection() {
    let invocation = parse_args_with_request_id(
        strings(["--root", "/ws", "--member", "mem_app", "capture"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::Capture(request) = invocation.request else {
        panic!("expected capture");
    };
    assert_eq!(request.meta.workspace.unwrap().root, Some("/ws".to_owned()));
    assert_eq!(request.meta.selection.unwrap().targets, vec!["mem_app"]);
}

#[test]
pub(crate) fn parses_all_with_target_exclusion_for_ls() {
    let invocation = parse_args_with_request_id(
        strings(["--all", "--no-target", "@root", "ls"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::Ls { request, .. } = invocation.request else {
        panic!("expected ls");
    };
    let selection = request.meta.selection.unwrap();
    assert_eq!(selection.targets, vec!["@all"]);
    assert_eq!(selection.exclude_targets, vec!["@root"]);
    assert_eq!(selection.all, None);
    assert!(selection.member_ids.is_empty());
    assert!(selection.paths.is_empty());
}

#[test]
pub(crate) fn parses_target_aliases_into_selector_fields() {
    let invocation = parse_args_with_request_id(
        strings([
            "--target",
            "@root",
            "--member",
            "mem_app",
            "--member-path",
            "repos/lib",
            "--no-target",
            "@default",
            "--no-member",
            "mem_old",
            "--no-member-path",
            "repos/old",
            "status",
        ]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::Status(request) = invocation.request else {
        panic!("expected status");
    };
    let selection = request.meta.selection.unwrap();
    assert_eq!(selection.targets, vec!["@root", "mem_app", "repos/lib"]);
    assert_eq!(
        selection.exclude_targets,
        vec!["@default", "mem_old", "repos/old"]
    );
    assert_eq!(selection.all, None);
    assert!(selection.member_ids.is_empty());
    assert!(selection.paths.is_empty());
}

#[test]
pub(crate) fn parses_combined_status_flags() {
    let invocation = parse_args_with_request_id(
        strings(["status", "--porcelain", "--no-branches"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    assert_eq!(invocation.output, OutputMode::Porcelain);
    let CliRequest::Status(request) = invocation.request else {
        panic!("expected status");
    };
    assert_eq!(request.mode, Some(gwz_core::StatusMode::Combined));
    assert_eq!(request.include_file_changes, Some(true));
    assert_eq!(request.include_branch_summary, Some(false));
    assert_eq!(
        request.path_style,
        Some(gwz_core::StatusPathStyle::WorkspaceRelative)
    );
}

#[test]
pub(crate) fn parses_status_as_combined_by_default() {
    let invocation =
        parse_args_with_request_id(strings(["status"]), "req_test", Path::new("/cwd")).unwrap();

    let CliRequest::Status(request) = invocation.request else {
        panic!("expected status");
    };
    assert_eq!(request.mode, Some(gwz_core::StatusMode::Combined));
    assert_eq!(request.include_file_changes, Some(true));
    assert_eq!(request.include_branch_summary, Some(true));
    assert_eq!(
        request.path_style,
        Some(gwz_core::StatusPathStyle::WorkspaceRelative)
    );
}

#[test]
pub(crate) fn parses_no_combined_status_as_summary_mode() {
    let invocation = parse_args_with_request_id(
        strings(["status", "--no-combined"]),
        "req_test",
        Path::new("/cwd"),
    )
    .unwrap();

    let CliRequest::Status(request) = invocation.request else {
        panic!("expected status");
    };
    assert_eq!(request.mode, Some(gwz_core::StatusMode::Summary));
    assert_eq!(request.include_file_changes, Some(true));
    assert_eq!(request.include_branch_summary, Some(true));
    assert_eq!(
        request.path_style,
        Some(gwz_core::StatusPathStyle::WorkspaceRelative)
    );
}

#[test]
pub(crate) fn parses_command_matrix() {
    assert!(matches!(
        parse(strings(["repo", "add", "repos/app"])).request,
        CliRequest::AddExistingRepo(_)
    ));
    assert!(matches!(
        parse(strings(["add", "src/foo.rs"])).request,
        CliRequest::Stage(_)
    ));
    // The verb collapses to CliRequest::Tag; the operation lives in the inner TagRequest.op.
    assert!(matches!(
        parse(strings(["tag"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::List)
    ));
    assert!(matches!(
        parse(strings(["tag", "--list"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::List)
    ));
    assert!(matches!(
        parse(strings(["tag", "v1"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::Create)
    ));
    assert!(matches!(
        parse(strings(["tag", "--delete", "v1"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::Delete)
    ));
    assert!(matches!(
        parse(strings(["tag", "--push"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::Push)
    ));
    assert!(matches!(
        parse(strings(["tag", "--fetch"])).request,
        CliRequest::Tag(ref r) if matches!(r.op, gwz_core::TagOp::Fetch)
    ));
    assert!(matches!(
        parse(strings(["tag", "--list", "--remote", "origin"])).request,
        CliRequest::Tag(ref r) if r.remote.as_deref() == Some("origin")
    ));
    assert!(matches!(
        parse(strings(["branch"])).request,
        CliRequest::Branch(ref r) if matches!(r.op, gwz_core::BranchOp::List)
    ));
    assert!(matches!(
        parse(strings(["branch", "--list"])).request,
        CliRequest::Branch(ref r) if matches!(r.op, gwz_core::BranchOp::List)
    ));
    assert!(matches!(
        parse(strings(["branch", "--create", "feature/login"])).request,
        CliRequest::Branch(ref r)
            if matches!(r.op, gwz_core::BranchOp::Create)
                && r.name.as_deref() == Some("feature/login")
                && r.start_ref.as_deref() == Some("HEAD")
                && r.switch_after_create.is_none()
    ));
    assert!(matches!(
        parse(strings([
            "branch",
            "--create",
            "feature/login",
            "--from",
            "main",
            "--switch"
        ]))
        .request,
        CliRequest::Branch(ref r)
            if matches!(r.op, gwz_core::BranchOp::Create)
                && r.name.as_deref() == Some("feature/login")
                && r.start_ref.as_deref() == Some("main")
                && r.switch_after_create == Some(true)
    ));
    assert!(matches!(
        parse(strings(["branch", "--delete", "feature/login"])).request,
        CliRequest::Branch(ref r)
            if matches!(r.op, gwz_core::BranchOp::Delete)
                && r.name.as_deref() == Some("feature/login")
    ));
    assert!(matches!(
        parse(strings(["branch", "--merge", "feature/source"])).request,
        CliRequest::Branch(ref r)
            if matches!(r.op, gwz_core::BranchOp::Merge)
                && r.name.is_none()
                && r.start_ref.as_deref() == Some("feature/source")
    ));
    assert!(matches!(
        parse(strings(["stash", "push"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Push)
                && r.include_untracked.is_none()
                && r.include_ignored.is_none()
    ));
    assert!(matches!(
        parse(strings(["stash", "push", "-u", "-m", "wip"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Push)
                && r.include_untracked == Some(true)
                && r.message.as_deref() == Some("wip")
    ));
    assert!(matches!(
        parse(strings(["stash", "push", "-a"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Push)
                && r.include_ignored == Some(true)
                && r.include_untracked.is_none()
    ));
    assert!(matches!(
        parse(strings(["stash", "list", "--expanded"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::List)
                && r.expanded == Some(true)
    ));
    assert!(matches!(
        parse(strings(["stash", "apply"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Apply)
                && r.stash_id.is_none()
    ));
    assert!(matches!(
        parse(strings(["stash", "pop", "stash_one"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Pop)
                && r.stash_id.as_deref() == Some("stash_one")
    ));
    assert!(matches!(
        parse(strings(["stash", "drop", "stash_one"])).request,
        CliRequest::Stash(ref r)
            if matches!(r.op, gwz_core::StashOp::Drop)
                && r.stash_id.as_deref() == Some("stash_one")
    ));
    assert!(matches!(
        parse(strings(["snapshot"])).request,
        CliRequest::ListSnapshots(_)
    ));
    assert!(matches!(
        parse(strings(["snapshot", "snap"])).request,
        CliRequest::Snapshot(ref r) if r.source.is_none()
    ));
    assert!(matches!(
        parse(strings(["snapshot", "snap", "--branch"])).request,
        CliRequest::Snapshot(ref r)
            if matches!(
                r.source.as_ref(),
                Some(gwz_core::SnapshotSource {
                    kind: gwz_core::SnapshotSourceKind::Current,
                    branch: None,
                })
            )
    ));
    assert!(matches!(
        parse(strings(["snapshot", "snap", "--branch", "main"])).request,
        CliRequest::Snapshot(ref r)
            if matches!(
                r.source.as_ref(),
                Some(gwz_core::SnapshotSource {
                    kind: gwz_core::SnapshotSourceKind::Branch,
                    branch: Some(branch),
                }) if branch == "main"
            )
    ));
    assert!(matches!(
        parse(strings(["ls"])).request,
        CliRequest::Ls { local: false, ref request } if request.include_unmaterialized.is_none()
    ));
    assert!(matches!(
        parse(strings(["ls", "--local"])).request,
        CliRequest::Ls { local: true, .. }
    ));
    assert!(matches!(
        parse(strings(["ls", "--unmaterialized"])).request,
        CliRequest::Ls { ref request, .. } if request.include_unmaterialized == Some(true)
    ));
    assert!(matches!(
        parse(strings(["forall", "--", "git", "status"])).request,
        CliRequest::Forall { mode: gwz_core::ExecMode::Argv, ref command, .. } if command.len() == 2
    ));
    assert!(matches!(
        parse(strings(["forall", "-c", "git status"])).request,
        CliRequest::Forall {
            mode: gwz_core::ExecMode::Shell,
            ..
        }
    ));
    assert!(matches!(
        parse(strings(["forall", "app", "lib", "--", "git"])).request,
        CliRequest::Forall { ref projects, .. } if projects.len() == 2
    ));
    assert!(matches!(
        parse(strings(["repo", "create", "repos/app"])).request,
        CliRequest::CreateRepo(_)
    ));
    assert!(matches!(
        parse(strings(["repo", "sync"])).request,
        CliRequest::RepoSync(_)
    ));
    assert!(matches!(
        parse(strings(["repo", "sync", "repos/app"])).request,
        CliRequest::RepoSync(ref request)
            if request.meta.selection.as_ref().unwrap().targets == vec!["repos/app"]
    ));
    assert!(matches!(
        parse(strings(["materialize", "--lock"])).request,
        CliRequest::Materialize(_)
    ));
    assert!(matches!(
        parse(strings(["materialize", "--snapshot", "snap_one"])).request,
        CliRequest::Materialize(_)
    ));
    assert!(matches!(
        parse(strings(["materialize", "--switch", "feature/login"])).request,
        CliRequest::Materialize(ref r)
            if r.target.kind == gwz_core::MaterializeTargetKind::Branch
                && r.target.name.as_deref() == Some("feature/login")
                && r.target.commit.is_none()
    ));
    assert!(matches!(
        parse(strings(["pull", "--head"])).request,
        CliRequest::PullHead(_)
    ));
    assert!(matches!(
        parse(strings(["pull", "--snapshot", "snap_one"])).request,
        CliRequest::PullSnapshot(_)
    ));
    assert!(matches!(
        parse(strings(["snapshot", "snap_one"])).request,
        CliRequest::Snapshot(_)
    ));
    assert!(matches!(
        parse(strings(["tag", "release_one"])).request,
        CliRequest::Tag(_)
    ));
    assert!(matches!(
        parse(strings(["push"])).request,
        CliRequest::Push(_)
    ));
}

#[test]
pub(crate) fn rejects_invalid_command_combinations_before_core_execution() {
    assert!(parse_result(strings(["--json", "--jsonl", "status"])).is_err());
    assert!(parse_result(strings(["--all", "--member", "mem_app", "status"])).is_ok());
    assert!(parse_result(strings(["--path", "repos/lib", "status"])).is_err());
    assert!(parse_result(strings(["status", "--no-files", "--no-branches"])).is_err());
    assert!(parse_result(strings(["status", "--combined", "--no-combined"])).is_err());
    assert!(parse_result(strings(["status", "--porcelain", "--no-combined"])).is_err());
    assert!(parse_result(strings(["status", "--no-combined", "--no-files"])).is_err());
    assert!(parse_result(strings(["push", "--combined"])).is_err());
    // forall: no command, both -c and --, and --json/--jsonl are all rejected at parse.
    assert!(parse_result(strings(["forall"])).is_err());
    assert!(parse_result(strings(["forall", "-c", "x", "--", "y"])).is_err());
    assert!(parse_result(strings(["--json", "forall", "--", "echo"])).is_err());
    assert!(parse_result(strings(["--jsonl", "forall", "--", "echo"])).is_err());
    assert!(parse_result(strings(["push", "--no-combined"])).is_err());
    assert!(
        parse_result(strings([
            "--member",
            "mem_app",
            "repo",
            "sync",
            "repos/app"
        ]))
        .is_err()
    );
    assert!(parse_result(strings(["materialize", "--snapshot"])).is_err());
    assert!(
        parse_result(strings([
            "materialize",
            "--lock",
            "--switch",
            "feature/login"
        ]))
        .is_err()
    );
    assert!(parse_result(strings(["materialize", "--switch"])).is_err());
    assert!(parse_result(strings(["snapshot", "--branch"])).is_err());
    assert!(parse_result(strings(["snapshot", "--list", "--branch"])).is_err());
    assert!(parse_result(strings(["branch", "--list", "--create", "work"])).is_err());
    assert!(parse_result(strings(["branch", "--create", "work", "--delete", "work"])).is_err());
    assert!(parse_result(strings(["branch", "--merge", "source", "--create", "work"])).is_err());
    assert!(parse_result(strings(["branch", "--merge", "source", "--delete", "work"])).is_err());
    assert!(parse_result(strings(["branch", "--merge", "source", "--list"])).is_err());
    assert!(parse_result(strings(["branch", "--merge", "source", "--switch"])).is_err());
    assert!(parse_result(strings(["branch", "--delete", "work", "--switch"])).is_err());
    assert!(parse_result(strings(["branch", "--from", "main"])).is_err());
    assert!(parse_result(strings(["stash", "push", "-u", "-a"])).is_err());
    assert!(parse_result(strings(["stash", "drop"])).is_err());
    assert!(parse_result(strings(["pull", "--lock"])).is_err());
    assert!(parse_result(strings(["unknown"])).is_err());
}

#[test]
pub(crate) fn can_call_core_status_in_process() {
    let temp = TempDir::new("cli-status");
    gwz_core::workspace_ops::handle_create_workspace(
        gwz_core::CreateWorkspaceRequest {
            meta: request_meta("req_setup"),
            workspace_root: temp.path().to_string_lossy().into_owned(),
            workspace_id: Some("ws_cli".to_owned()),
        },
        "op_setup",
    )
    .unwrap();
    let invocation = parse_args_with_request_id(
        strings([
            "--root",
            temp.path().to_str().unwrap(),
            "status",
            "--no-combined",
        ]),
        "req_status",
        temp.path(),
    )
    .unwrap();

    let response = execute_invocation(&invocation).unwrap();

    assert_eq!(
        response.envelope.meta.aggregate_status,
        gwz_core::AggregateStatus::Ok
    );
    assert!(response.envelope.members.is_empty());
}

pub(crate) fn parse(args: Vec<String>) -> CliInvocation {
    parse_result(args).unwrap()
}

pub(crate) fn parse_result(args: Vec<String>) -> Result<CliInvocation, CliError> {
    parse_args_with_request_id(args, "req_test", Path::new("/cwd"))
}

pub(crate) fn strings<const N: usize>(items: [&str; N]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

pub(crate) fn request_meta(request_id: &str) -> gwz_core::RequestMeta {
    gwz_core::RequestMeta {
        request_id: request_id.to_owned(),
        schema_version: "gwz.protocol/v0".to_owned(),
        ..Default::default()
    }
}

pub(crate) struct TempDir {
    pub(crate) path: PathBuf,
}

impl TempDir {
    pub(crate) fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("gwz-cli-{prefix}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
