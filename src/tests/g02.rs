use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

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
