#[cfg(test)]
use clap::CommandFactory;

pub(crate) fn workspace_git_status_json(
    status: &gwz_core::WorkspaceGitStatus,
) -> serde_json::Value {
    serde_json::json!({
        "clean": status.clean,
        "root_status": status.root_status.as_ref().map(root_git_status_json),
        "root_file_changes": status.root_file_changes.iter().map(root_file_change_json).collect::<Vec<_>>(),
        "file_changes": status.file_changes.iter().map(file_change_json).collect::<Vec<_>>(),
        "branches": status.branches.iter().map(branch_status_json).collect::<Vec<_>>(),
        "branch_groups": status.branch_groups.iter().map(branch_group_json).collect::<Vec<_>>(),
        "branch_differences": status.branch_differences.iter().map(branch_difference_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn root_git_status_json(status: &gwz_core::WorkspaceRootGitStatus) -> serde_json::Value {
    serde_json::json!({
        "branch": status.branch,
        "detached": status.detached,
        "head": status.head,
        "staged": status.staged,
        "unstaged": status.unstaged,
        "untracked": status.untracked,
        "dirty": status.dirty,
        "unborn": status.unborn,
    })
}

pub(crate) fn root_file_change_json(
    change: &gwz_core::WorkspaceRootFileChange,
) -> serde_json::Value {
    serde_json::json!({
        "repo_path": change.repo_path,
        "workspace_path": change.workspace_path,
        "index_status": change.index_status,
        "worktree_status": change.worktree_status,
        "original_repo_path": change.original_repo_path,
    })
}

pub(crate) fn file_change_json(change: &gwz_core::GitFileChange) -> serde_json::Value {
    serde_json::json!({
        "member_id": change.member_id,
        "member_path": change.member_path,
        "repo_path": change.repo_path,
        "workspace_path": change.workspace_path,
        "index_status": change.index_status,
        "worktree_status": change.worktree_status,
        "original_repo_path": change.original_repo_path,
    })
}

pub(crate) fn branch_status_json(status: &gwz_core::GitMemberBranchStatus) -> serde_json::Value {
    serde_json::json!({
        "member_id": status.member_id,
        "member_path": status.member_path,
        "label": status.label,
        "branch": status.branch,
        "detached": status.detached,
        "unborn": status.unborn,
        "head": status.head,
        "upstream": status.upstream,
        "ahead": status.ahead,
        "behind": status.behind,
    })
}

pub(crate) fn branch_group_json(group: &gwz_core::GitBranchGroup) -> serde_json::Value {
    serde_json::json!({
        "label": group.label,
        "member_ids": group.member_ids,
        "member_paths": group.member_paths,
    })
}

pub(crate) fn branch_difference_json(
    difference: &gwz_core::GitBranchDifference,
) -> serde_json::Value {
    serde_json::json!({
        "label": difference.label,
        "majority_label": difference.majority_label,
        "member_ids": difference.member_ids,
        "member_paths": difference.member_paths,
        "message": difference.message,
    })
}
