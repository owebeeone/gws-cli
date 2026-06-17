#[cfg(test)]
use clap::CommandFactory;

pub(crate) const CLI_LONG: &str = "\
GWZ manages a local workspace made from multiple git repositories.

A workspace records its member repositories and exact revisions under the
tracked `gwz.conf/` directory. Commands operate on the workspace as a whole,
so a single request can initialize, inspect, snapshot, materialize, pull, or
push a coordinated set of repositories.";
