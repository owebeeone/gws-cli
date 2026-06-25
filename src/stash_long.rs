pub(crate) const STASH_LONG: &str = "\
Manage coordinated Git stashes across the workspace's selected member repositories.

The CLI only builds a StashRequest; stash discovery, registry I/O, Git operations,
and conflict handling are owned by gwz-core.

  push    gwz stash push [-u|-a] [-m <message>]
  list    gwz stash list [--expanded]
  apply   gwz stash apply [stash-id]
  pop     gwz stash pop [stash-id]
  drop    gwz stash drop <stash-id>";
