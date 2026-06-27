# Root Workspaces

GWZ can manage the development root workspace and other GWZ-managed roots. A
root workspace is the repository that owns `gwz.conf/` and coordinates member
repositories through the manifest, lock, snapshots, and tags.

In the `gwz-dev` development workspace, the main member directories are:

- `gwz-cli/`: the `gwz` binary, CLI docs, release scripts, and CLI tests.
- `gwz-core/`: the embeddable Rust engine, workspace artifacts, Git backend,
  and protocol types.
- `taut/`: schema, compiler, and protocol tooling used by GWZ.
- `gwz.conf/`: tracked GWZ root workspace metadata.
- `dev-docs/`: planning documents.

## Inspect The Workspace

List members:

```sh
gwz ls --local
```

Check status before mutating anything:

```sh
gwz status
```

For scripts:

```sh
gwz status --porcelain
gwz --json ls
```

## Run Member-Local Commands

Run a command in every member:

```sh
gwz forall -- git status --short
```

Run in specific members:

```sh
gwz forall gwz-cli gwz-core -- cargo test
```

Use `--partial` when you want all selected members attempted even if one fails:

```sh
gwz --partial forall -- cargo test
```

## Manage Workspace State

Capture a checkpoint:

```sh
gwz snapshot before-maintenance
```

Move to the lock:

```sh
gwz materialize --lock
```

Move to heads:

```sh
gwz pull --head
```

Preview first when the operation may touch many members:

```sh
gwz --dry-run pull --head
gwz --dry-run materialize --lock
```

## Stage, Commit, And Push

Stage all changed files across repositories:

```sh
gwz add -A
```

Commit with one message across repositories that have staged changes:

```sh
gwz commit -m "Update workspace"
```

Push root and member refs:

```sh
gwz push
```

Push configured members only:

```sh
gwz --all --no-target @root push
```

## Tag A Coordinated State

Create an annotated tag:

```sh
gwz tag v0.3.0 -m "GWZ v0.3.0"
```

Push the tag to member remotes:

```sh
gwz tag --push v0.3.0
```

Check out that tag later:

```sh
gwz materialize --tag v0.3.0
```

## Practical Rules

- Start with `gwz status` before operations that mutate members or metadata.
- Use snapshots before broad changes.
- Prefer member selectors for targeted work.
- Use `--dry-run` for materialize, pull, push, tag, add, and other mutations
  when the effect is not obvious.
- Keep root metadata changes committed when they are meant to be shared.
