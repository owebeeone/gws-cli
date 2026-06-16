# GWZ Stash Implementation Plan

Status: draft

Related spec: `dev-docs/GwzStashSpec.md`

Goal: add coordinated workspace stash support while keeping each implementation
step reviewable. Each step SHOULD target no more than about 500 LOC of net code
change. This is advisory, not a hard limit.

## Scope Decisions

- GWZ MUST record workspace stash bundles in local runtime metadata under
  `.gwz/stash/bundles/`.
- Native Git stash entries MUST remain the storage for actual file changes.
- Bundle metadata MUST be local runtime state, not tracked workspace intent.
  Store it under `.gwz/`, not `gwz.conf/`.
- v0 SHOULD stash workspace members only. The workspace root repository SHOULD
  be excluded from stash operations until root lifecycle and first-commit
  behavior are deliberately specified.
- Bundle-wide `pop` MUST be the default.
- Per-member `pop` MAY be supported only when explicit through member selection,
  and MUST update bundle metadata so partial restoration is visible.
- `stash@{n}` MUST be advisory only. Lookup MUST prefer the GWZ message prefix
  and stash commit identity when available.

## Target Commands

```text
gwz stash push [-u|-a] [-m <message>] [selection flags]
gwz stash list [--no-combined] [--json]
gwz stash pop [stash-id] [selection flags]
gwz stash apply [stash-id] [selection flags]
gwz stash drop <stash-id> [selection flags]
```

Flag meanings:

- default push: tracked changes only, matching `git stash push`
- `-u`, `--include-untracked`: include untracked files
- `-a`, `--all`: include untracked and ignored files
- `-m`: user message suffix; GWZ prepends `gwz:<stash_id>:`

## Bundle Metadata

Bundle files SHOULD use this shape:

```yaml
schema: gwz.stash-bundle/v0
workspace_id: ws_default
stash_id: stsh_01JZABC123
message: before-refactor
created_at: "2026-06-17T10:00:00Z"
include_untracked: true
include_ignored: false
members:
  mem_gwz_core:
    path: gwz-core
    participation: stashed
    restore_state: pending
    stash_message: "gwz:stsh_01JZABC123: before-refactor"
    git_stash_oid: "..."
    git_stash_ref: "stash@{0}"
    branch_before: main
    head_before: "..."
    dirty_summary_before:
      staged: 0
      unstaged: 1
      untracked: 2
  mem_taut:
    path: taut
    participation: empty
    restore_state: noop
    branch_before: main
    head_before: "..."
    dirty_summary_before:
      staged: 0
      unstaged: 0
      untracked: 0
```

Participation values:

- `stashed`: native Git stash entry was created
- `empty`: member participated but had nothing to stash
- `skipped`: member was not part of this explicit operation

Restore state values:

- `pending`: stashed member has not been restored or dropped
- `applied`: stash was applied but kept
- `popped`: stash was popped and removed from Git stash
- `dropped`: stash was dropped without applying
- `noop`: clean member with no Git stash payload
- `missing`: registry entry no longer has a matching Git stash payload

## Step 1: Spec Tightening

Files:

- `dev-docs/GwzStashSpec.md`
- `dev-docs/GwzStashPlan.md`

Work:

- Clarify that the registry is authoritative for bundle membership, not for
  payload existence.
- Clarify default tracked-only behavior, `-u`, and `-a`.
- Clarify whether `pop` uses `git stash pop --index`. Recommendation: v0 SHOULD
  preserve index state when possible and report a typed conflict when not.
- State that root repository stash support is deferred.
- State that explicit member selection on `pop`, `apply`, and `drop` creates a
  partial bundle and updates `restore_state`.

Verification:

- Manual review.
- The stale project spelling guard MUST report no matches in `dev-docs`.

## Step 2: Core Data Model And Registry I/O

Files:

- `gwz-core/src/stash/`
- `gwz-core/src/lib.rs`
- `gwz-core/src/workspace/` or runtime path helpers
- focused tests in `gwz-core`

Work:

- Add stash bundle structs.
- Add read/write/list/delete helpers for `.gwz/stash/bundles/*.yml`.
- Use atomic writes consistent with existing artifact writes.
- Validate schema, workspace id, stash id, member ids, member paths, and state
  enum values.
- Keep `.gwz/` reserved and local.

Tests first:

- Round-trip one bundle with `stashed`, `empty`, and partial restore states.
- Reject unsupported schema versions.
- Reject duplicate or invalid member records.
- Atomic write replaces existing bundle cleanly.
- Listing returns newest first by `created_at`.

Exit criteria:

- Core tests pass for registry I/O.
- No Git stash behavior yet.

## Step 3: Git Backend Stash Primitives

Files:

- `gwz-core/src/git/`
- git backend tests

Work:

- Extend `GitBackend` with stash methods:

```text
stash_push(path, options, message) -> GitStashResult
stash_list(path) -> Vec<GitStashEntry>
stash_apply(path, selector, preserve_index) -> GitStashApplyResult
stash_pop(path, selector, preserve_index) -> GitStashPopResult
stash_drop(path, selector) -> GitStashDropResult
```

- Selector MUST support message prefix and, when available, stash object id.
- Result SHOULD include stash oid, advisory stash ref, message, and timestamp
  when Git exposes them.

Tests first:

- Push tracked-only changes and verify working tree is clean afterward.
- Push with `--include-untracked` and verify untracked files are captured.
- Push with `--all` and verify ignored files are captured.
- List finds GWZ-prefixed stash entries.
- Apply restores but keeps stash entry.
- Pop restores and removes only the matching GWZ stash.
- Drop removes only the matching GWZ stash.
- Non-GWZ stashes are not touched.

Exit criteria:

- Git stash primitives work in isolated local repos.

## Step 4: Protocol And Operation Types

Files:

- `gwz-core/protocol/gwz.taut.py`
- generated protocol files
- protocol corpus
- protocol tests

Work:

- Add request/response messages for stash push, list, apply, pop, and drop.
- Add typed status records for stash bundle and member participation.
- Add error codes if needed:
  - `StashNotFound`
  - `StashIncomplete`
  - `StashConflict`
- Regenerate Rust protocol and corpus.

Tests first:

- CBOR round-trip for each stash request and response.
- Error code wire values pinned.
- Generated protocol test remains current.

Exit criteria:

- Protocol tests pass.
- No CLI parser yet.

## Step 5: `stash push` Orchestration

Files:

- `gwz-core/src/stash/`
- core operation tests

Work:

- Resolve workspace and selection.
- Generate a stash id.
- Preflight all selected active Git members.
- For each selected member:
  - record `empty` when no included changes exist
  - run native stash when changes exist
  - record message, oid/ref, branch/head, and dirty summary
- Write bundle after member stashes complete.

Atomicity notes:

- Full atomicity is not guaranteed once native Git mutations start.
- Preflight MUST reduce predictable failures.
- If mutation fails mid-operation, GWZ MUST write a recovery bundle for any
  completed member stashes and return a typed partial failure.

Tests first:

- Dirty members create native stash entries with the shared `gwz:<stash_id>:`
  prefix.
- Clean members are recorded as `empty`.
- Mixed clean/dirty selection creates one bundle.
- Default push ignores untracked files.
- `-u` includes untracked files.
- `-a` includes ignored files.
- Mid-operation failure records recoverable partial metadata.

Exit criteria:

- Core `stash push` can be called without CLI.

## Step 6: `stash list`

Files:

- `gwz-core/src/stash/`
- `gwz-cli/src/main.rs`
- CLI workflow tests

Work:

- Core loads bundle registry and optionally reconciles against member `git stash
  list`.
- Combined human output shows one line per bundle.
- `--no-combined` expands per-member participation and restore state.
- JSON output exposes full bundle metadata.

Tests first:

- Combined list sorts newest first.
- Combined list shows member counts, stashed count, clean count, and partial
  state when applicable.
- Expanded list shows per-member stash refs or clean/noop state.
- Drift is surfaced when a registry entry points at a missing Git stash.

Exit criteria:

- Users can inspect bundles before restore.

## Step 7: Bundle-Wide `stash apply` And `stash pop`

Files:

- `gwz-core/src/stash/`
- `gwz-cli/src/main.rs`
- tests in both crates

Work:

- Resolve newest eligible bundle when no stash id is supplied.
- For explicit stash id, use that bundle.
- Preflight all pending `stashed` members in the bundle.
- `apply` restores all pending stashed members and sets `restore_state: applied`
  when selection is explicit or keeps state pending when applying bundle-wide.
- `pop` restores all pending stashed members, drops native stash entries, and
  sets `restore_state: popped`.
- Delete the bundle file only after all members are terminal:
  `popped`, `dropped`, or `noop`.

Tests first:

- Bundle-wide pop restores all dirty members and deletes complete bundle.
- Bundle-wide apply restores all dirty members and keeps Git stash payloads.
- Pop never touches unrelated Git stashes.
- Missing native stash returns `StashIncomplete`.
- Dirty destination worktree returns `StashConflict` before mutation.

Exit criteria:

- Default restore behavior is bundle-wide and predictable.

## Step 8: Explicit Per-Member Restore

Files:

- `gwz-core/src/stash/`
- `gwz-cli/src/main.rs`
- workflow tests

Work:

- Allow member selection only when explicitly provided:

```text
gwz stash pop stsh_... --member mem_gwz_core
gwz stash apply stsh_... --member-path gwz-core
```

- The command MUST be rejected when selection is ambiguous or implicit.
- Update only selected member records.
- Keep bundle file while any selected member remains `pending`.
- `stash list` MUST show partial bundles.

Tests first:

- Explicit per-member pop restores one member and leaves others pending.
- Subsequent bundle-wide pop restores the remaining pending members.
- Explicit per-member apply marks or reports state without dropping the stash.
- Selection without stash id is rejected unless newest-bundle semantics are
  explicitly specified later.

Exit criteria:

- Per-member restore is possible but never accidental.

## Step 9: `stash drop`

Files:

- `gwz-core/src/stash/`
- `gwz-cli/src/main.rs`
- tests

Work:

- Drop native stash payloads for selected pending/applied members.
- Set restore state to `dropped`.
- Delete bundle file when all member states are terminal.

Tests first:

- Bundle-wide drop removes all matching native stashes.
- Per-member drop is explicit and leaves remaining members pending.
- Missing stash during drop marks `missing` or returns `StashIncomplete`,
  depending on policy chosen in Step 1.

Exit criteria:

- Users can clean up bundle metadata and native stash payloads.

## Step 10: CLI Polish And Release Verification

Files:

- `gwz-cli/src/main.rs`
- `gwz-cli/tests/local_workflows.rs`
- release workflow tests if needed

Work:

- Help text and examples for all stash subcommands.
- Human output consistency with current status style.
- JSON output stability.
- Publish workflow still runs core protocol tests with released taut package.

Verification:

- `cargo test --locked` in `gwz-core`
- `cargo test --locked` in `gwz-cli`
- `cargo fmt --check` in both repos
- `cargo clippy --all-targets -- -D warnings` in both repos
- `cargo build --release --locked` in `gwz-cli`

## Open Questions

- Should `apply` mark `restore_state: applied`, or should it leave state
  `pending` because the native stash still exists?
- Should `pop` preserve staged state by default with `--index`, or should this
  be opt-in?
- Should missing native stash on `drop` be terminal `missing`, or should the
  command fail until the user reconciles?
- Should newest-bundle resolution consider only bundles where all selected
  members are pending?
- Should root repository stash support be added after the workspace root has an
  initial commit flow?
