use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

#[test]
pub(crate) fn progress_detail_covers_resolving_and_counting_phases() {
    let resolving = gwz_core::GitTransferProgress {
        phase: gwz_core::GitProgressPhase::Resolving,
        received_objects: None,
        total_objects: None,
        received_bytes: None,
        indexed_deltas: Some(980),
        total_deltas: Some(1254),
    };
    assert_eq!(progress_detail(&resolving), "78% (980/1254)");

    let counting = gwz_core::GitTransferProgress {
        phase: gwz_core::GitProgressPhase::Counting,
        received_objects: None,
        total_objects: Some(500),
        received_bytes: None,
        indexed_deltas: None,
        total_deltas: None,
    };
    assert_eq!(progress_detail(&counting), "500");
}
