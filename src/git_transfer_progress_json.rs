#[cfg(test)]
use clap::CommandFactory;

pub(crate) fn git_transfer_progress_json(
    progress: &gwz_core::GitTransferProgress,
) -> serde_json::Value {
    serde_json::json!({
        "phase": format!("{:?}", progress.phase),
        "received_objects": progress.received_objects,
        "total_objects": progress.total_objects,
        "received_bytes": progress.received_bytes,
        "indexed_deltas": progress.indexed_deltas,
        "total_deltas": progress.total_deltas,
    })
}
