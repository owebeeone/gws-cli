pub(crate) const MATERIALIZE_LONG: &str = "\
Materialize workspace members to an explicit target.

Materialization makes the local repositories match a workspace target. It is not
raw `git pull`; GWZ plans the workspace operation first and applies the selected
target across members. With no target flag, `gwz materialize` uses the workspace
lock. Use `--head`, `--snapshot`, `--tag`, or `--switch` for a different
target.";
