#[cfg(test)]
use clap::CommandFactory;

pub(crate) const REPO_CREATE_LONG: &str = "\
Create a new local repository member and register it with the workspace.

The repository is created immediately at the requested member path and can be
pushed to a remote later. This supports the GWZ workflow where a workspace can
grow new repositories locally before deciding where they should be published.";
