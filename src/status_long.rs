#[cfg(test)]
use clap::CommandFactory;

pub(crate) const STATUS_LONG: &str = "\
Show git status across workspace members.

The default mode requests a combined workspace status: file paths are reported
relative to the workspace and prefixed by member path when file entries are
available. Use `--no-combined` for per-member summaries. Use `--porcelain` when
another tool needs stable script-oriented output.";
