pub(crate) const BRANCH_LONG: &str = "\
Manage local Git branches across the workspace's selected member repositories.

The CLI only builds a BranchRequest; validation, repository inspection, locking,
and mutation are handled by gwz-core.

  list     gwz branch [--list]
  create   gwz branch --create <name> [--from <ref>] [--switch]
  delete   gwz branch --delete <name>
  merge    gwz branch --merge <source-ref>";
