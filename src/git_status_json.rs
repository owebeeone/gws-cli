#[cfg(test)]
use clap::CommandFactory;

pub(crate) fn member_json(member: &gwz_core::MemberResponse) -> serde_json::Value {
    serde_json::json!({
        "member_id": member.member_id,
        "member_path": member.member_path,
        "source_kind": format!("{:?}", member.source_kind),
        "status": format!("{:?}", member.status),
        "error": member.error.as_ref().map(error_json),
        "planned": member.planned.as_ref().map(planned_json),
        "state": member.state.as_ref().map(member_state_json),
        "git_status": member.git_status.as_ref().map(git_status_json),
        "lock_match": member.lock_match.map(|lock_match| format!("{:?}", lock_match)),
    })
}

pub(crate) fn member_state_json(state: &gwz_core::ResolvedMemberState) -> serde_json::Value {
    serde_json::json!({
        "member_id": state.member_id,
        "path": state.path,
        "source_id": state.source_id,
        "source_kind": format!("{:?}", state.source_kind),
        "commit": state.commit,
        "branch": state.branch,
        "detached": state.detached,
        "upstream": state.upstream,
        "dirty": state.dirty,
        "materialized": state.materialized,
        "remotes": state.remotes.iter().map(remote_spec_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn remote_spec_json(remote: &gwz_core::RemoteSpec) -> serde_json::Value {
    serde_json::json!({
        "name": remote.name,
        "url": remote.url,
        "fetch": remote.fetch,
        "push": remote.push,
    })
}

pub(crate) fn git_status_json(status: &gwz_core::GitStatus) -> serde_json::Value {
    serde_json::json!({
        "member_id": status.member_id,
        "branch": status.branch,
        "detached": status.detached,
        "head": status.head,
        "upstream": status.upstream,
        "ahead": status.ahead,
        "behind": status.behind,
        "staged": status.staged,
        "unstaged": status.unstaged,
        "untracked": status.untracked,
        "dirty": status.dirty,
    })
}

pub(crate) fn planned_json(planned: &gwz_core::PlannedChange) -> serde_json::Value {
    serde_json::json!({
        "action": format!("{:?}", planned.action),
        "from_ref": planned.from_ref,
        "to_ref": planned.to_ref,
        "message": planned.message,
    })
}

pub(crate) fn error_json(error: &gwz_core::GwzError) -> serde_json::Value {
    serde_json::json!({
        "code": format!("{:?}", error.code),
        "message": error.message,
        "member_id": error.member_id,
        "member_path": error.member_path,
        "detail": error.detail,
    })
}
