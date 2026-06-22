# Releasing gwz-cli

gwz-cli is the CLI binary. It depends on **gwz-core**, and that dependency is the only
thing that differs between branches:

- **`main` (dev):** `gwz-core = { path = "../gwz-core" }` — builds against the local
  sibling checkout, so you must have `../gwz-core` checked out next to this repo. The
  crate `version` carries a `-dev` suffix. **Do not cut release tags here.**
- **`release`:** `gwz-core = { git = "…/gwz-core", tag = "vX.Y.Z" }` — pinned to a
  published gwz-core release, so it is standalone-buildable and reproducible.
  **Release tags are cut off `release`.**

## Process

1. **Release the matching gwz-core first** — tag it off its `main` (see
   [gwz-core/RELEASE.md](../gwz-core/RELEASE.md)) and note the new tag `vX.Y.Z`.
2. `git switch release && git merge main`.
3. Reconcile the one intentional branch difference: the `gwz-core` dependency must stay
   in the **`git` + `tag`** form and point at the gwz-core tag that contains the code this
   release relies on:
   `gwz-core = { git = "https://github.com/owebeeone/gwz-core", tag = "vX.Y.Z" }`
   (NOT the `path` form that lives on `main`).
4. Set the real release `version` (drop the `-dev` suffix), run `cargo build` to refresh
   `Cargo.lock`, then `cargo test`.
5. Commit and tag **off `release`**: `git tag gwz-cli-vA.B.C`. Push `release` and the tag.

## The merge gotcha

`main` always carries the `path` dependency; `release` always carries the `git` + `tag`
dependency. As long as `main` never edits that dependency line, merging `main` → `release`
resolves it cleanly to release's form. The release-time job is to make sure the pinned
gwz-core tag is the one that actually contains the gwz-core code this gwz-cli release uses
(per step 1) — bump it every release.
