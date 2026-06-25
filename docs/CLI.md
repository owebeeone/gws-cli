# CLI Reference

This reference is derived from:

```sh
cargo run -q -p gwz -- --help
cargo run -q -p gwz -- help <command>
```

Use terminal help as the source of truth for exact parser behavior.

## Usage

```text
gwz [OPTIONS] <COMMAND>
```

## Commands

| Command | Purpose |
| --- | --- |
| `init` | Create a workspace or initialize one from source URLs. |
| `clone` | Clone a workspace and materialize its members. |
| `add` | Stage file contents across workspace repositories. |
| `repo` | Manage workspace repositories. |
| `status` | Show workspace Git status. |
| `ls` | List workspace members. |
| `forall` | Run a command in each selected member. |
| `snapshot` | Record or list workspace snapshots. |
| `tag` | Manage Git tags across workspace repositories. |
| `branch` | Manage local Git branches across workspace members. |
| `stash` | Manage coordinated Git stash bundles across workspace members. |
| `materialize` | Materialize workspace members to a target. |
| `pull` | Update workspace members to an explicit target. |
| `push` | Push workspace member refs. |
| `capture` | Record the live worktree state into the lock. |
| `commit` | Commit staged changes across members and the workspace root. |

## Global Options

| Option | Meaning |
| --- | --- |
| `--root <path>` | Workspace root override. |
| `--member <member-id>` | Select a member by id. May be supplied more than once. |
| `--member-path <member-path>` | Select a member by path. May be supplied more than once. |
| `--all` | Select all members. Cannot be combined with member selectors. |
| `--dry-run` | Plan without mutating workspace metadata or member repositories. |
| `--partial` | Allow operations to continue for members that can proceed. |
| `--force` | Allow destructive behavior when required. |
| `--sync <mode>` | Select sync behavior. Values: `fetch-only`, `ff-only`, `merge`, `rebase`, `reset`, `driver-selected`. |
| `--remote <name>` | Select the Git remote name for remote operations. |
| `--jobs <n>` | Global ceiling for concurrent member operations. Default is 50. |
| `--max-per-host <n>` | Maximum concurrent network operations per remote host. Default is 8. |
| `--progress-interval <ms>` | Minimum milliseconds between member progress events. Default is 100; `0` emits every update. |
| `--json` | Render one structured JSON response. |
| `--jsonl` | Render newline-delimited JSON records for streaming consumers. |
| `--ssh-timeout <secs>` | Timeout for stalled SSH/network reads. Default is 3; `0` disables it. |
| `-h`, `--help` | Print help. |
| `-V`, `--version` | Print version. |

`--json` and `--jsonl` are mutually exclusive. `gwz status --porcelain` cannot
be combined with either machine output flag.

## Command Synopsis

### `gwz init`

```text
gwz init [OPTIONS] [url]...
```

Options:

| Option | Meaning |
| --- | --- |
| `--path <path-prefix>` | Workspace-relative prefix for initialized source repositories. Defaults to an empty prefix. |

See [commands/init.md](commands/init.md).

### `gwz clone`

```text
gwz clone [OPTIONS] <url> [directory]
```

Arguments:

| Argument | Meaning |
| --- | --- |
| `<url>` | Git URL of the workspace root repository. |
| `[directory]` | Target directory. If omitted, derived from the URL. |

See [commands/clone.md](commands/clone.md).

### `gwz repo`

```text
gwz repo [OPTIONS] <COMMAND>
gwz repo add [OPTIONS] <repo-path>
gwz repo create [OPTIONS] <member-path>
gwz repo sync [member-path]
```

Subcommands:

| Subcommand | Meaning |
| --- | --- |
| `add` | Add an existing local Git repository as a member. |
| `create` | Create a new local repository member. |
| `sync` | Refresh manifest metadata from local Git config. |

`repo sync` does not fetch, push, check out branches, or rewrite the lock.

See [commands/repo.md](commands/repo.md).

### `gwz add`

```text
gwz add [OPTIONS] [pathspec]...
```

Options:

| Option | Meaning |
| --- | --- |
| `-A`, `--all` | Stage all changes across every workspace repository. |

See [commands/add.md](commands/add.md).

### `gwz commit`

```text
gwz commit [OPTIONS] --message <message>
```

Options:

| Option | Meaning |
| --- | --- |
| `-m`, `--message <message>` | Commit message applied to every committed repository. |
| `-a`, `--all` | Stage tracked modifications first, like `git commit -a`. |

See [commands/commit.md](commands/commit.md).

### `gwz status`

```text
gwz status [OPTIONS]
```

Options:

| Option | Meaning |
| --- | --- |
| `--combined` | Render combined workspace status. This is the default. |
| `--no-combined` | Render per-repository status. |
| `--porcelain` | Render stable script-oriented status output. |
| `--no-files` | Omit file changes from combined status. |
| `--no-branches` | Omit branch summaries from combined status. |

See [commands/status.md](commands/status.md).

### `gwz ls`

```text
gwz ls [OPTIONS]
```

Options:

| Option | Meaning |
| --- | --- |
| `--local` | Print workspace-relative paths instead of absolute paths. |
| `--unmaterialized` | Include configured members that are not materialized. |

See [commands/ls.md](commands/ls.md).

### `gwz forall`

```text
gwz forall [OPTIONS] [projects]... [-- <cmd>...]
```

Arguments and options:

| Item | Meaning |
| --- | --- |
| `[projects]...` | Members to run in by id or path. Empty means all. |
| `[cmd]...` | Command and args, run directly without a shell. Use after `--`. |
| `-c`, `--command-string <string>` | Run a shell command string. |
| `--no-banner` | Suppress the per-member banner on stderr. |

See [commands/forall.md](commands/forall.md).

### `gwz snapshot`

```text
gwz snapshot [OPTIONS] [name]
```

Options:

| Option | Meaning |
| --- | --- |
| `--list` | List existing snapshots instead of recording one. |
| `--branch[=<name>]` | Snapshot the current attached branch for all selected members, or a named local branch without switching. |

Omitting `[name]` also lists snapshots.

See [commands/snapshot.md](commands/snapshot.md).

### `gwz tag`

```text
gwz tag [OPTIONS] [name]
```

Options:

| Option | Meaning |
| --- | --- |
| `--list` | List tags. Default with no name. |
| `--delete` | Delete the named tag. |
| `--push` | Push one named tag, or all tags when no name is supplied. |
| `--fetch` | Fetch tags from a remote. |
| `-m <message>` | Annotated tag message. |
| `-s`, `--sign` | Create a signed tag. |

See [commands/tag.md](commands/tag.md).

### `gwz materialize`

```text
gwz materialize [OPTIONS]
```

Target options:

| Option | Meaning |
| --- | --- |
| `--lock` | Materialize the workspace lock. This is the default. |
| `--head` | Materialize repository heads. |
| `--snapshot <name>` | Materialize a workspace snapshot. |
| `--tag <name>` | Materialize a workspace tag. |
| `--switch <branch>` | Switch selected members to an existing local branch and rewrite the lock from observed state. |

Only one target flag may be supplied.

See [commands/materialize.md](commands/materialize.md).

### `gwz branch`

```text
gwz branch [OPTIONS]
```

Options:

| Option | Meaning |
| --- | --- |
| `--list` | List local branches across selected members. This is the default. |
| `--create <name>` | Create a local branch across selected members. |
| `--from <ref>` | Start point for `--create`. Defaults to `HEAD`. |
| `--switch` | Switch selected members to the created branch after `--create`. |
| `--delete <name>` | Delete a non-current local branch across selected members. |
| `--merge <source>` | Merge a source ref into each selected member's current attached branch. |

See [commands/branch.md](commands/branch.md).

### `gwz stash`

```text
gwz stash <COMMAND>
```

Subcommands:

| Subcommand | Meaning |
| --- | --- |
| `push [-u|-a] [-m <message>]` | Push a coordinated stash bundle for selected members. |
| `list [--expanded]` | List local coordinated stash bundles. |
| `apply [stash-id]` | Apply a bundle and keep native stash payloads. |
| `pop [stash-id]` | Apply a bundle and remove native stash payloads. |
| `drop <stash-id>` | Remove native stash payloads without applying them. |

See [commands/stash.md](commands/stash.md).

### `gwz pull`

```text
gwz pull [OPTIONS]
```

Target options:

| Option | Meaning |
| --- | --- |
| `--head` | Pull repository heads. This is the default. |
| `--snapshot <name>` | Pull a workspace snapshot. |

Only one target flag may be supplied.

See [commands/pull.md](commands/pull.md).

### `gwz push`

```text
gwz push [OPTIONS]
```

`gwz push` has no command-specific options. Use global selectors and
`--remote <name>`.

See [commands/push.md](commands/push.md).

### `gwz capture`

```text
gwz capture [OPTIONS]
```

`gwz capture` has no command-specific options.

See [commands/capture.md](commands/capture.md).
