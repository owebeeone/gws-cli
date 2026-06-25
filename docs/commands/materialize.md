# `gwz materialize`

Materialize workspace members to an explicit target.

```text
gwz materialize [OPTIONS]
```

Materialization makes local member repositories match a workspace target. With
no target flag, GWZ uses the workspace lock.

## Targets

| Option | Meaning |
| --- | --- |
| `--lock` | Materialize `gwz.conf/gwz.lock.yml`. This is the default. |
| `--head` | Materialize repository heads. |
| `--snapshot <name>` | Materialize a workspace snapshot. |
| `--tag <name>` | Materialize a Git tag across selected members. |
| `--switch <branch>` | Switch selected members to an existing local branch. |

Only one target flag may be supplied.

## Examples

Materialize the lock:

```sh
gwz materialize
```

Materialize a snapshot:

```sh
gwz materialize --snapshot before-refactor
```

Materialize a tag:

```sh
gwz materialize --tag v0.3.0
```

Switch selected members to an existing local branch:

```sh
gwz materialize --switch feature/refactor
```

Allow required destructive behavior explicitly:

```sh
gwz --force materialize --tag v0.3.0
```

## Notes

- This is not a raw `git pull`; GWZ plans the workspace operation first.
- `--switch` does not create branches, fetch, detach, or move branch refs. It
  requires the branch to exist locally in every selected member and rewrites the
  lock from observed post-switch state.
- Use `gwz status` before materializing when you have local changes.
- Use `--dry-run` to preview planned member changes.
- Use `--partial` only when it is acceptable for some selected members to move
  while others fail.
