# `gwz branch`

Manage local Git branches across selected workspace members.

```text
gwz branch [OPTIONS]
```

With no operation flag, `gwz branch` lists local branches for the selected
members. Branch operations apply to workspace members only; the workspace root
repository is not included.

## Options

| Option | Meaning |
| --- | --- |
| `--list` | List local branches across selected members. This is the default. |
| `--create <name>` | Create a local branch across selected members. |
| `--from <ref>` | Start point for `--create`. Defaults to `HEAD`. |
| `--switch` | Switch selected members to the branch after `--create`. |
| `--delete <name>` | Delete a local branch across selected members. |
| `--merge <source>` | Merge a source ref into each selected member's current attached branch. |

Only one of `--list`, `--create`, `--delete`, or `--merge` may be supplied.
`--from` and `--switch` require `--create`.

## Examples

List branches:

```sh
gwz branch
```

Create a branch from each selected member's `HEAD`:

```sh
gwz branch --create feature/refactor
```

Create and switch to a branch:

```sh
gwz branch --create feature/refactor --switch
```

Create from a named start ref:

```sh
gwz branch --create release/v1 --from origin/main
```

Delete a non-current branch:

```sh
gwz branch --delete feature/old
```

Merge a source ref into each selected member's current branch:

```sh
gwz branch --merge feature/refactor
```

## Notes

- Create rejects before mutation if any selected member lacks the start ref or
  already has the branch at a different commit.
- Create with `--switch` requires clean selected member worktrees and rewrites
  the lock from observed post-switch state.
- Delete refuses to delete the current branch. Deleting a non-current branch
  does not require a clean worktree.
- Merge requires each selected member to be materialized, clean, on a current
  attached branch, and free of in-progress merge/rebase state before mutation.
- Merge conflicts are reported as `conflicted` with per-member conflict paths.
  The native Git merge state is left in place for resolution, and successful
  earlier member merges are not rolled back.
- Branch mutations acquire the workspace-wide mutator lock shared with
  `gwz stash`.
