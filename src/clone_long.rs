#[cfg(test)]
use clap::CommandFactory;

pub(crate) const CLONE_LONG: &str = "\
Clone a GWZ workspace from its root repository URL.

`gwz clone` is the one-shot form of `git clone <url>` followed by
`gwz materialize --lock`. It clones the workspace root repository (the one that
owns the tracked `gwz.conf/` directory) into a target directory, verifies it is
a GWZ workspace, then materializes every member: missing member repositories are
cloned and checked out at the commits recorded in `gwz.conf/gwz.lock.yml`.

If the target directory is omitted, it is derived from the URL.";
