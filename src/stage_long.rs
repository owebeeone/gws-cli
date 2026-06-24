
pub(crate) const STAGE_LONG: &str = "\
Stage file contents across the workspace's member repositories.

`gwz add` is the multi-repo `git add`: each pathspec is resolved relative to the
current directory, routed to the member (or workspace root) repository that owns
it, and staged there. Pair it with `gwz commit`. To register an existing
repository as a workspace member, use `gwz repo add` instead.";
