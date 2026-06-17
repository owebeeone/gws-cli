#[cfg(test)]
use clap::CommandFactory;

pub(crate) const TAG_LONG: &str = "\
Record a named GWZ workspace tag.

A GWZ tag is workspace metadata, not a git tag inside each member repository.
It stores the workspace-level mapping from member to revision, so the same tag
name can be meaningful inside this workspace without colliding with tags in
other workspaces or child repositories.";
