#[cfg(test)]
use clap::CommandFactory;

pub(crate) const PULL_LONG: &str = "\
Move workspace members forward to an explicit target.

`gwz pull` is a workspace operation, not a direct wrapper around `git pull`.
The default target is `--head`, and the default sync policy is fast-forward only.
If any selected member cannot update cleanly, the operation is rejected before
partial mutation unless `--partial` or another explicit policy changes that
behavior.";
