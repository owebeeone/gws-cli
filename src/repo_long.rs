#[cfg(test)]
use clap::CommandFactory;

pub(crate) const REPO_LONG: &str = "\
Manage repository members inside a workspace.

Repository commands create or update the local member repositories that make up
the workspace. For v0, this group exposes member creation; use top-level
commands such as `gwz add`, `gwz status`, `gwz pull`, and `gwz push` for
workspace-wide operations.";
