#[cfg(test)]
use clap::CommandFactory;

pub(crate) const CLI_AFTER: &str = "\
Examples:
  gwz init git@github.com:org/app.git git@github.com:org/lib.git
  gwz status
  gwz snapshot before-refactor
  gwz pull --head";
