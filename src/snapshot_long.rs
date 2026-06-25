pub(crate) const SNAPSHOT_LONG: &str = "\
Record the current workspace selection as a named snapshot.

A snapshot captures the current member revisions so the workspace can later be
materialized back to the same coordinated state. Use snapshots before risky
multi-repository changes, before sharing a reproducible work area, or before
pulling all members forward.

By default, GWZ snapshots the observed selected member heads. Use bare
`--branch` to snapshot each member's current attached branch, or
`--branch <name>` to snapshot a named branch across members.";
