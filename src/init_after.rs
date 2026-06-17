#[cfg(test)]
use clap::CommandFactory;

pub(crate) const INIT_AFTER: &str = "\
Examples:
  gwz init
  gwz --root /work/demo init
  gwz init --path repos git@github.com:org/app.git
  gwz init git@github.com:org/app.git git@github.com:org/lib.git";
