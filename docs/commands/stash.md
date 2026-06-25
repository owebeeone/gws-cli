# `gwz stash`

Manage coordinated native Git stash bundles across selected workspace members.

```text
gwz stash <COMMAND>
```

Each push creates one local bundle id and one native Git stash entry per dirty
selected member. Bundle metadata is stored under `.gwz/stash/bundles/`; native
stash payloads remain in member repositories. The workspace root repository is
not a stash participant.

## Subcommands

| Subcommand | Meaning |
| --- | --- |
| `push [-u|-a] [-m <message>]` | Push a coordinated stash bundle. |
| `list [--expanded]` | List local stash bundles. |
| `apply [stash-id]` | Apply a bundle and keep native stash payloads. |
| `pop [stash-id]` | Apply a bundle and remove native stash payloads. |
| `drop <stash-id>` | Remove native stash payloads without applying them. |

## Push

```text
gwz stash push [-u|-a] [-m <message>]
```

By default, stash push records tracked staged and unstaged changes. `-u`
includes untracked files. `-a` includes ignored files and also includes
untracked files. `-u` and `-a` are mutually exclusive.

Clean selected members are recorded in the bundle as no-op members. Dirty
members receive native stash entries with a `gwz:<stash-id>:` message prefix.

## Restore

`apply` and `pop` default to the newest eligible local bundle when no `stash-id`
is supplied. `drop` requires a `stash-id`.

Restore operations require clean destination member worktrees and preserve index
state by default. A partial restore requires an explicit member selection. Native
stash identity is resolved by object id first and GWZ message prefix second;
`stash@{n}` is display text only because native stash indices can move.

## Examples

Push tracked changes:

```sh
gwz stash push -m before-refactor
```

Push tracked and untracked changes:

```sh
gwz stash push -u -m before-refactor
```

List bundles with member detail:

```sh
gwz stash list --expanded
```

Apply the latest eligible bundle:

```sh
gwz stash apply
```

Pop a specific bundle:

```sh
gwz stash pop stash_2026_06_25T10_00_00Z
```

Drop a specific bundle:

```sh
gwz stash drop stash_2026_06_25T10_00_00Z
```

## Notes

- Bundle metadata is local runtime state and is not portable across clones.
- If `.gwz/` is removed, bundle grouping is lost. Native GWZ-prefixed stash
  entries may still be surfaced as orphan warnings by `gwz stash list`.
- `stash_incomplete` means local bundle metadata and native stash payloads no
  longer match, or a partial restore needs explicit selection.
- Stash mutations acquire the workspace-wide mutator lock shared with branch
  mutations.
