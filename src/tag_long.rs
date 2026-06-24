
pub(crate) const TAG_LONG: &str = "\
Manage real git tags across the workspace's member repositories — the multi-repo
`git tag`, fanned out the way `gwz commit` fans out `git commit`.

Local operations (create, list, delete) span the selected members plus the workspace
root; remote operations (push, fetch, and list/delete against a --remote) span the
members only.

  create   gwz tag <name> [-m <message>] [-s]   lightweight / annotated / signed
  list     gwz tag                              local (or --list [--remote <name>])
  delete   gwz tag --delete <name> [--remote <name>]
  push     gwz tag --push [<name>] [--remote <name>]   one tag, or every tag
  fetch    gwz tag --fetch [--remote <name>]";
