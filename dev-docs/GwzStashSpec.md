# GWZ Stash

Status: draft

Scope: coordinated `git stash` across workspace members. CLI surface in
`gwz-cli`; semantics and registry in `gwz-core`.

## Problem

`git stash` is per-repository. A workspace often has dirty state in several
members at once; stashing or restoring one repo at a time is easy to get wrong
and leaves no shared identity for “this stash point across the workspace.”

## Goals

- `gwz stash push` stashes the **selection** in one coordinated operation.
- Every participating stash shares a **GWZ stash id** embedded in the git stash
  message so refs can be correlated across repos.
- `gwz stash pop` restores a **bundle** — all members that share the same GWZ
  id — not a single repo in isolation (unless selection narrows members).
- `gwz stash list` shows workspace stash **points** (combined default) or
  per-member detail (`--no-combined`).
- Repos with **nothing to stash** still participate in the bundle via a workspace
  registry marker (no fake git stash required in the member repo).

## Non-Goals

- Replacing `git stash` inside a single member (users may still run it locally).
- Stash sync to remotes or other machines (local workspace only for v0).
- Auto-stash on `gwz pull` (separate feature).

## Concepts

### GWZ stash id (`stash_id`)

Opaque bundle identifier, e.g. `stsh_01JZ…` (ULID-style). One id per
`gwz stash push`.

### GWZ stash message prefix

User message `-m` is optional. GWZ always stores git stash entries with a
machine-parseable prefix:

```text
gwz:stsh_01JZABC123: <user message>
```

If `-m` is omitted, the suffix is `gwz stash`.

The prefix is how `gwz stash list` rediscovers bundles if the registry is stale,
and how `pop` matches the correct `stash@{n}` in each repo.

### Stash bundle

One logical workspace stash point:

- one `stash_id`
- one user message (suffix only, without prefix)
- one timestamp
- per-member participation record (see below)

### Participation

| Kind | Meaning |
|------|---------|
| `stashed` | Member had work to save; a git stash entry exists with this `stash_id` in its message |
| `empty` | Member was clean (and `-a` did not force otherwise); **no** git stash entry; bundle still lists the member |

`empty` exists so **pop** can restore the whole workspace snapshot of “who was
in this bundle” without requiring a dummy commit or empty git stash object.

## Registry (`.gwz/stash/`)

Workspace authority for bundles. Lives under internal runtime dir (not
`gwz.conf/` — local, not workspace intent).

```text
.gwz/stash/
  bundles/
    stsh_01JZABC123.yml
```

Example bundle file:

```yaml
schema: gwz.stash-bundle/v0
stash_id: stsh_01JZABC123
message: before-refactor
created_at: "2026-06-15T14:30:00Z"
members:
  mem_gwz_core:
    path: gwz-core
    participation: stashed
    git_stash_ref: stash@{2}
  mem_taut:
    path: taut
    participation: empty
  mem_gwz_cli:
    path: gwz-cli
    participation: stashed
    git_stash_ref: stash@{0}
```

Rules:

- Written atomically after a successful `stash push`.
- `git_stash_ref` is the ref at push time (`stash@{n}`); pop resolves by
  message prefix first, falls back to stored ref if ref moved.
- Registry MUST NOT be a workspace member path; `.gwz/` is already reserved.

## Commands

### `gwz stash push`

```text
gwz stash push [-a] [-m <message>] [--member …] [--member-path …] [--all]
```

Behavior:

1. Resolve selection (default: all active members).
2. Generate new `stash_id`.
3. Preflight every selected member (repo exists, stash possible).
4. For each member:
   - If dirty (tracked/untracked per `-a`):  
     `git stash push` with `-a` when set and message  
     `gwz:<stash_id>: <message or "gwz stash">`
   - If clean: record `participation: empty`; **do not** run `git stash push`.
5. Write bundle under `.gwz/stash/bundles/`.
6. Report per-member outcomes + aggregate status.

Flags:

| Flag | Maps to |
|------|---------|
| `-a` | `git stash push -a` on members that stash |
| `-m` | User suffix only; GWZ adds prefix |
| Selection globals | Same as `gwz status` / `gwz pull` |

Default: **atomic** — if any member that has changes cannot stash, reject before
mutating any member (same preflight discipline as other GWZ mutators).

### `gwz stash list`

```text
gwz stash list [--no-combined] [--json]
```

**Combined (default):** one line per bundle, newest first:

```text
stsh_01JZABC  2026-06-15  before-refactor  3 members (2 stashed, 1 clean)
stsh_01JZXYZ  2026-06-14  wip              3 members (3 stashed)
```

**`--no-combined`:** expand each bundle:

```text
stash stsh_01JZABC  2026-06-15  before-refactor
  gwz-core   stash@{2}  gwz:stsh_01JZABC: before-refactor
  taut       (clean — no git stash)
  gwz-cli    stash@{0}  gwz:stsh_01JZABC: before-refactor
```

List source: registry first; optionally reconcile against `git stash list` in
each member and warn on drift (orphan git stashes with `gwz:` prefix not in
registry, or registry entries whose git refs are missing).

### `gwz stash pop`

```text
gwz stash pop [stash-id] [--member …] [--all]
```

| Invocation | Behavior |
|--------------|----------|
| `gwz stash pop` | Pop the **newest** bundle that applies to the current selection |
| `gwz stash pop stsh_01JZABC` | Pop that specific bundle |

Pop rules:

1. Load bundle from registry (or fail `stash_not_found`).
2. Preflight **every** member in the bundle (not only selection intersection):
   - `stashed`: matching git stash exists; worktree can accept pop (clean or
     `--force` policy later).
   - `empty`: noop.
3. If any `stashed` member cannot pop cleanly, **reject before mutation**
   (atomic default).
4. Pop all `stashed` members (git stash pop matching `gwz:<stash_id>:` message).
5. Remove bundle file on full success; update registry if partial mode added
   later.

Important: pop is **bundle-scoped**. Popping `stsh_X` never pops a member’s
unrelated `git stash` entries. Users who want single-repo pop use `git stash`
inside that repo.

Selection narrows which bundles are eligible for “newest” resolution but a
explicit `stash-id` still pops the **whole** bundle (all members recorded in it).
Rationale: the bundle is the unit of consistency.

### `gwz stash drop` (v0 SHOULD)

```text
gwz stash drop <stash-id>
```

Drop git stashes for all `stashed` members in the bundle, delete registry entry.
Atomic preflight like pop.

## Empty-repo / clean-member strategy

**Do not** create empty git stash objects or dummy commits in clean repos.

Instead:

1. Bundle registry records `participation: empty` for that member.
2. Combined `stash list` shows `(N stashed, M clean)`.
3. `stash pop` skips that member.

This keeps member repos free of GWZ-internal git objects while still allowing
synced “workspace stash points” that mean: *these repos were stashed together;
these were clean at that moment.*

If a member later becomes dirty and the user runs `gwz stash push` again, a new
`stash_id` is issued; old bundles are unchanged.

## CLI / core split

| Layer | Responsibility |
|-------|----------------|
| `gwz-cli` | argv, selection flags, human/json list renderers |
| `gwz-core` | bundle id, registry I/O, per-member git stash via `GitBackend`, atomic orchestration |

`GitBackend` SHOULD gain:

```text
stash_push(path, all, message) -> GitStashResult
stash_list(path) -> Vec<GitStashEntry>
stash_pop(path, selector) -> GitStashPopResult   # selector = message prefix or ref
```

## Output modes

| Mode | Behavior |
|------|----------|
| Human | Combined list default; per-member with `--no-combined` |
| `--json` | Structured bundles + members |
| `--porcelain` | Not required v0 |

## Errors (illustrative)

| Code | When |
|------|------|
| `dirty_member` | Pop would conflict (worktree not clean) |
| `stash_not_found` | Unknown `stash-id` or no bundles |
| `stash_incomplete` | Registry says stashed but git ref missing |
| `member_not_found` | Selection resolution failure |

## Open decisions

- Whether `gwz stash apply` (non-destructive) is v0 or v0.1.
- `--force` on pop to allow conflicts (defer).
- Prune bundles when user manually `git stash drop`s in one repo (reconcile
  command vs lazy warn on list).

## Examples

```text
# Stash everything dirty across workspace
gwz stash push -a -m "before pull"

# List workspace stash points
gwz stash list

# Per-repo detail
gwz stash list --no-combined

# Restore latest coordinated stash
gwz stash pop

# Restore a specific workspace stash point
gwz stash pop stsh_01JZABC123
```

## Related

- `gwz-cli` status combined/`--no-combined` — same list UX pattern
- `gwz-core` `.gwz/` runtime dir — registry location
- `GwzProgressSpec.md` — progress on `stash push`/`pop` when slow (optional)
