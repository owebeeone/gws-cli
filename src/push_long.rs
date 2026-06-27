pub(crate) const PUSH_LONG: &str = "\
Push workspace target refs to configured remotes.

`gwz push` applies one push request across selected workspace targets. By
default that includes the workspace root (`@root`) plus configured member
repositories. Use `--remote` to choose a remote name and selectors such as
`--target`, `--member`, `--member-path`, `--all`, and `--no-target @root` to
control which targets participate.";
