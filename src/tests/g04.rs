use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[test]
pub(crate) fn render_progress_line_without_current_member_is_just_counts() {
    let model = ProgressModel {
        label: "pulling".to_owned(),
        started: 2,
        finished: 2,
        current_path: None,
        current_progress: None,
    };
    assert_eq!(
        render_progress_line(&model, 0),
        "⠋ pulling: 2 done, 0 active"
    );
    // The spinner advances with the tick.
    assert!(render_progress_line(&model, 1).starts_with("⠙ "));
}
