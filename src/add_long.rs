#[cfg(test)]
use clap::CommandFactory;

pub(crate) const ADD_LONG: &str = "\
Add an existing local git repository to the workspace.

Use this when a repository already exists on disk and should become a workspace
member. GWZ records the repository as a member; it does not clone a new copy.
Use `gwz repo create` instead when the member should be created from scratch.";
