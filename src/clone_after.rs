#[cfg(test)]
use clap::CommandFactory;

pub(crate) const CLONE_AFTER: &str = "\
Examples:
  gwz clone git@github.com:org/workspace.git
  gwz clone git@github.com:org/workspace.git work/demo

If you already ran a plain `git clone` on a workspace root, run
`gwz materialize --lock` inside it to complete the clone instead.";
