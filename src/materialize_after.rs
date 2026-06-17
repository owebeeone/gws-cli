#[cfg(test)]
use clap::CommandFactory;

pub(crate) const MATERIALIZE_AFTER: &str = "\
Examples:
  gwz materialize
  gwz materialize --lock
  gwz materialize --snapshot before-refactor
  gwz --force materialize --tag release-2026-06";
