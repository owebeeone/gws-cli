# `gwz snapshot`

Record or list workspace snapshots.

```text
gwz snapshot [OPTIONS] [name]
```

A snapshot captures selected member revisions under a workspace-level name so
the workspace can later be materialized or pulled back to that coordinated
state.

## Arguments And Options

| Item | Meaning |
| --- | --- |
| `[name]` | Snapshot name to record. Omit to list existing snapshots. |
| `--list` | List existing snapshots instead of recording one. |
| `--branch[=<name>]` | Snapshot branch heads instead of observed worktree state. |

## Examples

Record a snapshot:

```sh
gwz snapshot before-refactor
```

Record all members explicitly:

```sh
gwz --all snapshot integration-baseline
```

Record the current attached branch for all selected members:

```sh
gwz snapshot branch-baseline --branch
```

Record a named local branch without switching worktrees:

```sh
gwz snapshot feature-baseline --branch feature/refactor
```

List snapshots:

```sh
gwz snapshot --list
```

Materialize a snapshot:

```sh
gwz materialize --snapshot before-refactor
```

Pull to a snapshot:

```sh
gwz pull --snapshot integration-baseline
```

## Notes

- Use snapshots for reproducible workspace checkpoints that are not necessarily
  release tags.
- `--branch` with no value requires every selected member to be attached to a
  born branch with the same branch name. Detached, unborn, or mixed branch
  selections are rejected.
- `--branch <name>` resolves `refs/heads/<name>` in each selected member without
  switching worktrees and records that commit in the snapshot.
- Snapshot listings render as human text by default and as a listing object with
  `--json`.
