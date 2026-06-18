
use crate::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CliResponse {
    pub(crate) envelope: gwz_core::ResponseEnvelope,
    pub(crate) workspace_git_status: Option<gwz_core::WorkspaceGitStatus>,
    pub(crate) status_mode: Option<gwz_core::StatusMode>,
}

impl CliResponse {
    pub(crate) fn envelope(response: gwz_core::ResponseEnvelope) -> Self {
        Self {
            envelope: response,
            workspace_git_status: None,
            status_mode: None,
        }
    }
}

/// Streams each operation event to stdout as a JSON line, flushed immediately,
/// so `--jsonl` consumers see records live as the operation runs instead of
/// batched at the end. stdout is block-buffered when piped, hence the flush.
pub(crate) struct JsonlSink;

impl gwz_core::operation::EventSink for JsonlSink {
    fn deliver(&self, event: gwz_core::OperationEvent) {
        use std::io::Write;
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "{}", event_json(&event));
        let _ = out.flush();
    }
}

pub(crate) fn render_human_response(response: &CliResponse) -> String {
    if let Some(workspace_status) = &response.workspace_git_status {
        return render_human_status_response(response, workspace_status);
    }

    let mut lines = vec![format!(
        "status: {:?}",
        response.envelope.meta.aggregate_status
    )];
    for member in &response.envelope.members {
        let mut line = format!(
            "{} {} {:?}",
            member.member_id, member.member_path, member.status
        );
        if let Some(error) = &member.error {
            line.push_str(&format!(" {:?}: {}", error.code, error.message));
        }
        if let Some(message) = member
            .planned
            .as_ref()
            .and_then(|planned| planned.message.as_ref())
        {
            line.push_str(&format!(" {message}"));
        }
        lines.push(line);
    }
    for error in &response.envelope.errors {
        lines.push(format!("{:?}: {}", error.code, error.message));
    }
    lines.join("\n")
}

pub(crate) fn render_human_status_response(
    response: &CliResponse,
    workspace_status: &gwz_core::WorkspaceGitStatus,
) -> String {
    let per_repo = response.status_mode == Some(gwz_core::StatusMode::Summary);
    let mut lines = Vec::new();
    append_branch_summary(&mut lines, workspace_status);
    if per_repo {
        append_per_repo_status(&mut lines, response, workspace_status);
    } else {
        let mut changes = root_human_changes(workspace_status);
        changes.extend(member_human_changes(workspace_status, None));
        append_change_sections(&mut lines, &changes);
    }
    append_unmaterialized_notice(&mut lines, response);
    append_status_issues(&mut lines, response);
    append_suppressed_dirty_summary(&mut lines, response, workspace_status);
    if lines.is_empty() {
        lines.push("nothing to commit, working tree clean".to_owned());
    }
    lines.join("\n")
}

pub(crate) fn is_unmaterialized(member: &gwz_core::MemberResponse) -> bool {
    member
        .state
        .as_ref()
        .is_some_and(|state| !state.materialized)
}

pub(crate) fn append_unmaterialized_notice(lines: &mut Vec<String>, response: &CliResponse) {
    let unmaterialized = response
        .envelope
        .members
        .iter()
        .filter(|member| is_unmaterialized(member))
        .collect::<Vec<_>>();
    if unmaterialized.is_empty() {
        return;
    }
    push_blank(lines);
    lines.push(
        "Members not materialized (run `gwz materialize --lock` to complete the clone):".to_owned(),
    );
    lines.extend(
        unmaterialized
            .into_iter()
            .map(|member| format!("  {}", member.member_path)),
    );
}

pub(crate) fn append_branch_summary(
    lines: &mut Vec<String>,
    workspace_status: &gwz_core::WorkspaceGitStatus,
) {
    let mut groups = workspace_status
        .branch_groups
        .iter()
        .map(|group| (group.label.clone(), group.member_paths.clone()))
        .collect::<Vec<_>>();

    let Some(root_status) = workspace_status.root_status.as_ref() else {
        if groups.is_empty() {
            lines.push("Workspace status".to_owned());
        } else if groups.len() == 1 {
            lines.push(branch_group_sentence(&groups[0].0));
        } else {
            append_branch_groups(lines, &groups);
        }
        return;
    };

    if let Some(label) = root_branch_label(root_status) {
        add_branch_group_path(&mut groups, label, ".".to_owned());
    }

    if groups.is_empty() {
        lines.push("Workspace status".to_owned());
    } else {
        if groups.len() == 1 {
            lines.push(branch_group_sentence(&groups[0].0));
        } else {
            append_branch_groups(lines, &groups);
        }
    }

    if root_status.unborn {
        lines.push("No commits yet".to_owned());
    }
}

pub(crate) fn root_branch_label(root_status: &gwz_core::WorkspaceRootGitStatus) -> Option<String> {
    if let Some(branch) = &root_status.branch {
        Some(branch.clone())
    } else if root_status.detached {
        Some(
            root_status
                .head
                .as_ref()
                .map(|head| format!("detached@{}", head.chars().take(12).collect::<String>()))
                .unwrap_or_else(|| "detached".to_owned()),
        )
    } else if root_status.unborn {
        Some("unborn".to_owned())
    } else {
        None
    }
}

pub(crate) fn add_branch_group_path(
    groups: &mut Vec<(String, Vec<String>)>,
    label: String,
    path: String,
) {
    if let Some(index) = groups
        .iter()
        .position(|(group_label, _)| group_label == &label)
    {
        let (label, mut paths) = groups.remove(index);
        paths.insert(0, path);
        groups.insert(0, (label, paths));
    } else {
        groups.insert(0, (label, vec![path]));
    }
}

pub(crate) fn append_branch_groups(lines: &mut Vec<String>, groups: &[(String, Vec<String>)]) {
    for (label, paths) in groups {
        lines.push(format!(
            "{} {}",
            paths.join(", "),
            branch_group_phrase(label)
        ));
    }
}

pub(crate) fn branch_group_sentence(label: &str) -> String {
    let phrase = branch_group_phrase(label);
    let mut chars = phrase.chars();
    let Some(first) = chars.next() else {
        return phrase;
    };
    format!("{}{}", first.to_uppercase(), chars.collect::<String>())
}

pub(crate) fn branch_group_phrase(label: &str) -> String {
    if label == "unborn" {
        "have no commits yet".to_owned()
    } else if label == "detached" {
        "HEAD detached".to_owned()
    } else if let Some(commit) = label.strip_prefix("detached@") {
        format!("detached at {commit}")
    } else {
        format!("on branch {label}")
    }
}

pub(crate) fn append_per_repo_status(
    lines: &mut Vec<String>,
    response: &CliResponse,
    workspace_status: &gwz_core::WorkspaceGitStatus,
) {
    let root_changes = root_human_changes(workspace_status);
    if !root_changes.is_empty() {
        push_blank(lines);
        lines.push("Workspace root".to_owned());
        append_change_sections(lines, &root_changes);
    }

    for member in &response.envelope.members {
        if is_unmaterialized(member) {
            continue;
        }
        let changes = member_human_changes(workspace_status, Some(&member.member_id));
        if changes.is_empty() && member.status == gwz_core::MemberStatus::Ok {
            continue;
        }
        push_blank(lines);
        lines.push(format_member_status_heading(member));
        append_change_sections(lines, &changes);
    }
}

/// F16: a dirty tree whose per-file detail was suppressed (`--no-files`) must not vanish
/// from the human output — the counts (`dirty`/staged/unstaged/untracked) are first-class
/// on `GitStatus`, independent of the file list. Surface a count summary for the root and
/// each member that is dirty but produced no rendered file changes.
pub(crate) fn append_suppressed_dirty_summary(
    lines: &mut Vec<String>,
    response: &CliResponse,
    workspace_status: &gwz_core::WorkspaceGitStatus,
) {
    let mut summary = Vec::new();
    if let Some(root) = workspace_status.root_status.as_ref()
        && root.dirty
        && root_human_changes(workspace_status).is_empty()
    {
        summary.push(format!(
            "  workspace root: {} staged, {} unstaged, {} untracked",
            root.staged, root.unstaged, root.untracked
        ));
    }
    for member in &response.envelope.members {
        if is_unmaterialized(member) {
            continue;
        }
        let Some(status) = member.git_status.as_ref() else {
            continue;
        };
        if status.dirty
            && member_human_changes(workspace_status, Some(&member.member_id)).is_empty()
        {
            summary.push(format!(
                "  {}: {} staged, {} unstaged, {} untracked",
                member.member_path, status.staged, status.unstaged, status.untracked
            ));
        }
    }
    if summary.is_empty() {
        return;
    }
    push_blank(lines);
    lines.push("Uncommitted changes (file detail omitted; re-run without --no-files):".to_owned());
    lines.extend(summary);
}

pub(crate) fn append_status_issues(lines: &mut Vec<String>, response: &CliResponse) {
    let mut issues = Vec::new();
    for member in &response.envelope.members {
        if is_unmaterialized(member) {
            continue;
        }
        if member.status != gwz_core::MemberStatus::Ok || member.error.is_some() {
            let mut issue = format!("{}: {:?}", member.member_path, member.status);
            if let Some(error) = &member.error {
                issue.push_str(&format!(" {:?}: {}", error.code, error.message));
            }
            issues.push(issue);
        }
    }
    issues.extend(
        response
            .envelope
            .errors
            .iter()
            .map(|error| format!("{:?}: {}", error.code, error.message)),
    );
    if issues.is_empty() {
        return;
    }
    push_blank(lines);
    lines.push("Issues:".to_owned());
    lines.extend(issues.into_iter().map(|issue| format!("  {issue}")));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HumanChangeSection {
    Staged,
    Unstaged,
    Untracked,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HumanChange {
    pub(crate) section: HumanChangeSection,
    pub(crate) status: String,
    pub(crate) path: String,
}

pub(crate) fn root_human_changes(
    workspace_status: &gwz_core::WorkspaceGitStatus,
) -> Vec<HumanChange> {
    workspace_status
        .root_file_changes
        .iter()
        .map(|change| {
            human_change(
                &change.index_status,
                &change.worktree_status,
                &change.workspace_path,
            )
        })
        .collect()
}

pub(crate) fn member_human_changes(
    workspace_status: &gwz_core::WorkspaceGitStatus,
    member_id: Option<&str>,
) -> Vec<HumanChange> {
    workspace_status
        .file_changes
        .iter()
        .filter(|change| member_id.is_none_or(|member_id| change.member_id == member_id))
        .map(|change| {
            human_change(
                &change.index_status,
                &change.worktree_status,
                &change.workspace_path,
            )
        })
        .collect()
}

pub(crate) fn human_change(index_status: &str, worktree_status: &str, path: &str) -> HumanChange {
    let section = if index_status == " " && worktree_status == "?" {
        HumanChangeSection::Untracked
    } else if index_status != " " {
        HumanChangeSection::Staged
    } else {
        HumanChangeSection::Unstaged
    };
    HumanChange {
        section,
        status: format_status_pair(index_status, worktree_status),
        path: path.to_owned(),
    }
}

pub(crate) fn append_change_sections(lines: &mut Vec<String>, changes: &[HumanChange]) {
    append_change_section(
        lines,
        changes,
        HumanChangeSection::Staged,
        "Changes to be committed:",
    );
    append_change_section(
        lines,
        changes,
        HumanChangeSection::Unstaged,
        "Changes not staged for commit:",
    );
    append_change_section(
        lines,
        changes,
        HumanChangeSection::Untracked,
        "Untracked files:",
    );
}

pub(crate) fn append_change_section(
    lines: &mut Vec<String>,
    changes: &[HumanChange],
    section: HumanChangeSection,
    header: &str,
) {
    let section_changes = changes
        .iter()
        .filter(|change| change.section == section)
        .collect::<Vec<_>>();
    if section_changes.is_empty() {
        return;
    }
    push_blank(lines);
    lines.push(header.to_owned());
    lines.extend(
        section_changes
            .into_iter()
            .map(|change| format!("  {} {}", change.status, change.path)),
    );
}

pub(crate) fn push_blank(lines: &mut Vec<String>) {
    if !lines.is_empty() && !lines.last().is_some_and(|line| line.is_empty()) {
        lines.push(String::new());
    }
}

pub(crate) fn format_member_status_heading(member: &gwz_core::MemberResponse) -> String {
    let Some(git_status) = &member.git_status else {
        return member.member_path.clone();
    };
    if let Some(branch) = &git_status.branch {
        format!("{} on branch {}", member.member_path, branch)
    } else if git_status.detached {
        format!("{} detached", member.member_path)
    } else {
        member.member_path.clone()
    }
}

pub(crate) fn render_porcelain_response(response: &CliResponse) -> String {
    if let Some(workspace_status) = &response.workspace_git_status
        && !(workspace_status.root_file_changes.is_empty()
            && workspace_status.file_changes.is_empty())
    {
        return workspace_status
            .root_file_changes
            .iter()
            .map(format_root_file_change)
            .chain(workspace_status.file_changes.iter().map(format_file_change))
            .collect::<Vec<_>>()
            .join("\n");
    }
    response
        .envelope
        .members
        .iter()
        .filter(|member| member.status != gwz_core::MemberStatus::Ok)
        .map(|member| format!("!! {}", member.member_path))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn format_file_change(change: &gwz_core::GitFileChange) -> String {
    let status = format_status_pair(&change.index_status, &change.worktree_status);
    format!("{status} {}", change.workspace_path)
}

pub(crate) fn format_root_file_change(change: &gwz_core::WorkspaceRootFileChange) -> String {
    let status = format_status_pair(&change.index_status, &change.worktree_status);
    format!("{status} {}", change.workspace_path)
}

pub(crate) fn format_status_pair(index_status: &str, worktree_status: &str) -> String {
    if index_status == " " && worktree_status == "?" {
        "??".to_owned()
    } else {
        format!("{index_status}{worktree_status}")
    }
}

pub(crate) fn render_jsonl_stream(
    response: &CliResponse,
    events: &[gwz_core::OperationEvent],
    result: Option<&gwz_core::OperationResult>,
) -> String {
    let mut lines = Vec::with_capacity(1 + events.len() + usize::from(result.is_some()));
    lines.push(response_json(response).to_string());
    lines.extend(events.iter().map(|event| event_json(event).to_string()));
    if let Some(result) = result {
        lines.push(result_json(result).to_string());
    }
    lines.join("\n")
}

pub(crate) fn response_json(response: &CliResponse) -> serde_json::Value {
    serde_json::json!({
        "kind": "response",
        "meta": response_meta_json(&response.envelope.meta),
        "members": response.envelope.members.iter().map(member_json).collect::<Vec<_>>(),
        "errors": response.envelope.errors.iter().map(error_json).collect::<Vec<_>>(),
        "workspace_git_status": response.workspace_git_status.as_ref().map(workspace_git_status_json),
    })
}

/// F9: render a top-level CLI error as structured JSON, envelope-consistent with
/// `response_json` (same keys; the error sits in `errors`, no members).
pub(crate) fn render_error_json(error: &CliError) -> String {
    serde_json::json!({
        "kind": "response",
        "meta": serde_json::Value::Null,
        "members": [],
        "errors": [{
            "code": error
                .code
                .map(|code| format!("{:?}", gwz_core::GwzErrorCode::from(code))),
            "message": error.message,
            "member_id": serde_json::Value::Null,
            "member_path": serde_json::Value::Null,
            "detail": serde_json::Value::Null,
        }],
        "workspace_git_status": serde_json::Value::Null,
    })
    .to_string()
}

pub(crate) fn result_json(result: &gwz_core::OperationResult) -> serde_json::Value {
    serde_json::json!({
        "kind": "result",
        "operation_id": result.operation_id,
        "request_id": result.request_id,
        "action": format!("{:?}", result.action),
        "aggregate_status": format!("{:?}", result.aggregate_status),
        "started_at_ms": result.started_at_ms,
        "finished_at_ms": result.finished_at_ms,
        "members": result.members.iter().map(member_json).collect::<Vec<_>>(),
        "errors": result.errors.iter().map(error_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn event_json(event: &gwz_core::OperationEvent) -> serde_json::Value {
    serde_json::json!({
        "kind": "event",
        "operation_id": event.operation_id,
        "request_id": event.request_id,
        "sequence": event.sequence,
        "timestamp_ms": event.timestamp_ms,
        "event_kind": format!("{:?}", event.kind),
        "severity": format!("{:?}", event.severity),
        "member_id": event.member_id,
        "member_path": event.member_path,
        "message": event.message,
        "member": event.member.as_ref().map(member_json),
        "error": event.error.as_ref().map(error_json),
        "progress": event.progress.as_ref().map(git_transfer_progress_json),
    })
}
