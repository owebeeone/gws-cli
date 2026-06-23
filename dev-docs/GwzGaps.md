# gwz — known gaps & deferred work

Tracked gaps that are intentionally not built yet. In-flight feature plans live in
`gwz-core/dev-docs/` (e.g. `GWZAddPlan.md`); this file collects the "not scheduled"
items so they aren't lost.

## `gwz add` (multi-repo staging)
- **Interactive / patch staging** — no `git add -p` equivalent (stage selected hunks).
- **Unstaging** — no `gwz restore --staged` / `gwz reset` equivalent to undo a stage.

(Implemented `gwz add` behavior and its other deferrals are recorded in
`gwz-core/dev-docs/GWZAddPlan.md`.)

## `gwz stash`
- Spec exists (`gwz-cli/dev-docs/GwzStashSpec.md` + `GwzStashPlan.md`), **not implemented**.
