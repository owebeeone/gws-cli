#[cfg(test)]
use clap::CommandFactory;

pub(crate) const STATUS_AFTER: &str = "\
Examples:
  gwz status
  gwz status --no-combined
  gwz status --porcelain
  gwz --member mem_app status";
