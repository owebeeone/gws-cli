#[cfg(test)]
use clap::CommandFactory;

pub(crate) const PUSH_LONG: &str = "\
Push workspace member refs to their configured remotes.

`gwz push` applies one push request across the selected workspace members. Use
`--remote` to choose a remote name and selection flags such as `--member`,
`--member-path`, or `--all` to control which members participate.";
