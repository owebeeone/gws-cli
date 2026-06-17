#[cfg(test)]
use clap::CommandFactory;

pub(crate) const PULL_AFTER: &str = "\
Examples:
  gwz pull --head
  gwz pull --snapshot integration-baseline
  gwz --sync fetch-only pull --head
  gwz --partial pull --head";
