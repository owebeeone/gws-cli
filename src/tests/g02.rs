use super::*;

#[test]
pub(crate) fn error_path_renders_structured_json_envelope() {
    // F9: a top-level error carrying a gwz-core code renders envelope-consistent JSON.
    let error = CliError {
        message: "member has uncommitted changes".to_owned(),
        code: Some(gwz_core::model::ErrorCode::DirtyMember),
    };
    let json: serde_json::Value = serde_json::from_str(&render_error_json(&error)).unwrap();
    assert_eq!(json["kind"], "response");
    assert!(json["members"].as_array().unwrap().is_empty());
    assert_eq!(
        json["errors"][0]["message"],
        "member has uncommitted changes"
    );
    assert!(
        json["errors"][0]["code"]
            .as_str()
            .unwrap()
            .contains("Dirty")
    );
    assert_eq!(
        error.human_message(),
        "DirtyMember: member has uncommitted changes"
    );

    // A CLI validation error (no gwz-core code) still renders structured, code null.
    let plain = CliError::new("--json and --jsonl are mutually exclusive");
    let json: serde_json::Value = serde_json::from_str(&render_error_json(&plain)).unwrap();
    assert!(json["errors"][0]["code"].is_null());
    assert_eq!(
        plain.human_message(),
        "--json and --jsonl are mutually exclusive"
    );
}

#[test]
pub(crate) fn json_renderer_outputs_structured_response() {
    let response = CliResponse::envelope(sample_response(
        gwz_core::AggregateStatus::Ok,
        gwz_core::MemberStatus::Ok,
    ));

    let json: serde_json::Value =
        serde_json::from_str(&render_response(&response, OutputMode::Json)).unwrap();

    assert_eq!(json["kind"], "response");
    assert_eq!(json["meta"]["aggregate_status"], "Ok");
    assert_eq!(json["members"][0]["member_id"], "mem_app");
    assert_eq!(json["members"][0]["status"], "Ok");
}

#[test]
pub(crate) fn jsonl_renderer_emits_response_event_and_result_in_order() {
    let response = sample_response(
        gwz_core::AggregateStatus::Accepted,
        gwz_core::MemberStatus::Planned,
    );
    let event = sample_event();
    let result = sample_result();

    let lines = render_jsonl_stream(&CliResponse::envelope(response), &[event], Some(&result))
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0]["kind"], "response");
    assert_eq!(lines[1]["kind"], "event");
    assert_eq!(lines[2]["kind"], "result");
}

#[test]
pub(crate) fn human_renderer_smoke_covers_success_rejection_and_member_failure() {
    let success = render_response(
        &CliResponse::envelope(sample_response(
            gwz_core::AggregateStatus::Ok,
            gwz_core::MemberStatus::Ok,
        )),
        OutputMode::Human,
    );
    assert!(success.contains("status: Ok"));

    let rejected = render_response(
        &CliResponse::envelope(sample_response(
            gwz_core::AggregateStatus::Rejected,
            gwz_core::MemberStatus::Rejected,
        )),
        OutputMode::Human,
    );
    assert!(rejected.contains("status: Rejected"));

    let failed = render_response(
        &CliResponse::envelope(sample_response(
            gwz_core::AggregateStatus::Failed,
            gwz_core::MemberStatus::Failed,
        )),
        OutputMode::Human,
    );
    assert!(failed.contains("RemoteRejected"));
}

#[test]
pub(crate) fn exit_code_mapping_distinguishes_success_rejected_and_failed() {
    assert_eq!(
        exit_code_for_response(&sample_response(
            gwz_core::AggregateStatus::Ok,
            gwz_core::MemberStatus::Ok,
        )),
        0
    );
    assert_eq!(
        exit_code_for_response(&sample_response(
            gwz_core::AggregateStatus::Rejected,
            gwz_core::MemberStatus::Rejected,
        )),
        2
    );
    assert_eq!(
        exit_code_for_response(&sample_response(
            gwz_core::AggregateStatus::Failed,
            gwz_core::MemberStatus::Failed,
        )),
        1
    );
    // F5: a dirty workspace is normal -- exit 0.
    assert_eq!(
        exit_code_for_response(&sample_response(
            gwz_core::AggregateStatus::Dirty,
            gwz_core::MemberStatus::Ok,
        )),
        0
    );
    // A conflicted workspace needs action -- exit 1.
    assert_eq!(
        exit_code_for_response(&sample_response(
            gwz_core::AggregateStatus::Conflicted,
            gwz_core::MemberStatus::Conflicted,
        )),
        1
    );
}

#[test]
pub(crate) fn no_files_still_surfaces_dirty_member_counts() {
    // F16: `--no-files` suppresses the per-file list (empty `file_changes`), but a dirty
    // member must still surface via its first-class counts — not silently vanish.
    let mut envelope = sample_response(gwz_core::AggregateStatus::Ok, gwz_core::MemberStatus::Ok);
    envelope.members[0].git_status = Some(dirty_git_status("mem_app", 0, 2, 1));
    let cli = CliResponse::envelope(envelope);
    let workspace = empty_workspace_git_status();

    let mut lines = Vec::new();
    append_suppressed_dirty_summary(&mut lines, &cli, &workspace);
    let out = lines.join("\n");
    assert!(out.contains("file detail omitted"), "got: {out}");
    assert!(
        out.contains("repos/app: 0 staged, 2 unstaged, 1 untracked"),
        "got: {out}"
    );

    // A clean member contributes nothing.
    let mut clean = sample_response(gwz_core::AggregateStatus::Ok, gwz_core::MemberStatus::Ok);
    clean.members[0].git_status = Some(dirty_git_status("mem_app", 0, 0, 0));
    let mut clean_lines = Vec::new();
    append_suppressed_dirty_summary(&mut clean_lines, &CliResponse::envelope(clean), &workspace);
    assert!(clean_lines.is_empty());
}

pub(crate) fn dirty_git_status(
    member_id: &str,
    staged: i64,
    unstaged: i64,
    untracked: i64,
) -> gwz_core::GitStatus {
    gwz_core::GitStatus {
        member_id: member_id.to_owned(),
        branch: Some("main".to_owned()),
        detached: false,
        head: None,
        upstream: None,
        ahead: None,
        behind: None,
        staged,
        unstaged,
        untracked,
        dirty: staged + unstaged + untracked > 0,
    }
}

pub(crate) fn empty_workspace_git_status() -> gwz_core::WorkspaceGitStatus {
    gwz_core::WorkspaceGitStatus {
        clean: false,
        file_changes: Vec::new(),
        branches: Vec::new(),
        branch_groups: Vec::new(),
        branch_differences: Vec::new(),
        root_status: None,
        root_file_changes: Vec::new(),
    }
}

pub(crate) fn sample_response(
    aggregate_status: gwz_core::AggregateStatus,
    member_status: gwz_core::MemberStatus,
) -> gwz_core::ResponseEnvelope {
    let error = (member_status == gwz_core::MemberStatus::Failed
        || member_status == gwz_core::MemberStatus::Rejected)
        .then(|| gwz_core::GwzError {
            code: gwz_core::GwzErrorCode::RemoteRejected,
            message: "remote rejected".to_owned(),
            member_id: Some("mem_app".to_owned()),
            member_path: Some("repos/app".to_owned()),
            target_kind: None,
            detail: None,
        });
    gwz_core::ResponseEnvelope {
        meta: gwz_core::ResponseMeta {
            request_id: "req_render".to_owned(),
            schema_version: "gwz.protocol/v0".to_owned(),
            action: gwz_core::ActionKind::Status,
            aggregate_status,
            operation_id: Some("op_render".to_owned()),
            message: None,
            attribution: None,
        },
        members: vec![gwz_core::MemberResponse {
            member_id: "mem_app".to_owned(),
            member_path: "repos/app".to_owned(),
            source_kind: gwz_core::SourceKind::Git,
            status: member_status,
            error,
            planned: None,
            state: None,
            git_status: None,
            lock_match: None,
            target_kind: None,
        }],
        errors: Vec::new(),
    }
}

pub(crate) fn sample_event() -> gwz_core::OperationEvent {
    gwz_core::OperationEvent {
        operation_id: "op_render".to_owned(),
        request_id: "req_render".to_owned(),
        sequence: 0,
        timestamp_ms: 1,
        kind: gwz_core::EventKind::OperationStarted,
        severity: gwz_core::Severity::Info,
        member_id: None,
        member_path: None,
        target_kind: None,
        message: Some("started".to_owned()),
        member: None,
        error: None,
        attribution: None,
        progress: None,
    }
}

pub(crate) fn sample_result() -> gwz_core::OperationResult {
    gwz_core::OperationResult {
        operation_id: "op_render".to_owned(),
        request_id: "req_render".to_owned(),
        action: gwz_core::ActionKind::Status,
        aggregate_status: gwz_core::AggregateStatus::Ok,
        started_at_ms: 1,
        finished_at_ms: 2,
        members: Vec::new(),
        errors: Vec::new(),
        attribution: None,
    }
}
