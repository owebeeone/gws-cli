use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

pub(crate) fn progress_event(
    kind: gwz_core::EventKind,
    member_path: Option<&str>,
    progress: Option<gwz_core::GitTransferProgress>,
) -> gwz_core::OperationEvent {
    gwz_core::OperationEvent {
        operation_id: "op".to_owned(),
        request_id: "req".to_owned(),
        sequence: 0,
        timestamp_ms: 0,
        kind,
        severity: gwz_core::Severity::Info,
        member_id: member_path.map(|_| "m".to_owned()),
        member_path: member_path.map(str::to_owned),
        message: None,
        member: None,
        error: None,
        attribution: None,
        progress,
    }
}

pub(crate) fn receiving(recv: i64, total: i64, bytes: i64) -> gwz_core::GitTransferProgress {
    gwz_core::GitTransferProgress {
        phase: gwz_core::GitProgressPhase::Receiving,
        received_objects: Some(recv),
        total_objects: Some(total),
        received_bytes: Some(bytes),
        indexed_deltas: None,
        total_deltas: None,
    }
}

#[test]
pub(crate) fn progress_model_folds_member_lifecycle() {
    use gwz_core::EventKind;
    let mut model = ProgressModel::new("cloning");

    assert!(model.apply(&progress_event(
        EventKind::MemberStarted,
        Some("repos/foo"),
        None
    )));
    assert!(model.apply(&progress_event(
        EventKind::MemberStarted,
        Some("repos/bar"),
        None
    )));
    assert_eq!((model.started, model.finished, model.active()), (2, 0, 2));

    assert!(model.apply(&progress_event(
        EventKind::MemberProgress,
        Some("repos/foo"),
        Some(receiving(10, 100, 2048)),
    )));
    assert_eq!(model.current_path.as_deref(), Some("repos/foo"));
    assert!(model.current_progress.is_some());

    // Finishing the current member clears the surfaced detail.
    assert!(model.apply(&progress_event(
        EventKind::MemberFinished,
        Some("repos/foo"),
        None
    )));
    assert_eq!((model.finished, model.active()), (1, 1));
    assert_eq!(model.current_path, None);
    assert!(model.current_progress.is_none());

    // Finishing a non-current member only moves the counts.
    model.current_path = Some("repos/bar".to_owned());
    assert!(model.apply(&progress_event(
        EventKind::MemberFinished,
        Some("repos/baz"),
        None
    )));
    assert_eq!((model.finished, model.active()), (2, 0));
    assert_eq!(model.current_path.as_deref(), Some("repos/bar"));
}

#[test]
pub(crate) fn progress_model_ignores_non_member_events() {
    use gwz_core::EventKind;
    let mut model = ProgressModel::new("materializing");
    assert!(!model.apply(&progress_event(EventKind::OperationStarted, None, None)));
    assert!(!model.apply(&progress_event(EventKind::ArtifactWritten, None, None)));
    assert!(!model.apply(&progress_event(EventKind::OperationFinished, None, None)));
    assert_eq!((model.started, model.finished), (0, 0));
}

#[test]
pub(crate) fn render_progress_line_shows_counts_and_receiving_detail() {
    let model = ProgressModel {
        label: "cloning".to_owned(),
        started: 3,
        finished: 1,
        current_path: Some("repos/app.git".to_owned()),
        current_progress: Some(receiving(1234, 2730, 3_400_000)),
    };
    assert_eq!(
        render_progress_line(&model, 0),
        "⠋ cloning: 1 done, 2 active · app receiving 45% (1234/2730), 3.2 MiB"
    );
}
