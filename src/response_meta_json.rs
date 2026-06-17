#[cfg(test)]
use clap::CommandFactory;

pub(crate) fn response_meta_json(meta: &gwz_core::ResponseMeta) -> serde_json::Value {
    serde_json::json!({
        "request_id": meta.request_id,
        "schema_version": meta.schema_version,
        "action": format!("{:?}", meta.action),
        "aggregate_status": format!("{:?}", meta.aggregate_status),
        "operation_id": meta.operation_id,
        "message": meta.message,
    })
}
