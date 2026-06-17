#[cfg(test)]
use clap::CommandFactory;

pub(crate) fn new_operation_id() -> String {
    format!("op_{}", unique_suffix())
}

pub(crate) fn unique_suffix() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{}_{}", std::process::id(), millis)
}
