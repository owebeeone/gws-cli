# `gwz push`

Push workspace target refs to configured remotes.

```text
gwz push [OPTIONS]
```

`gwz push` applies one push request across selected workspace targets. Plain
`gwz push` includes `@root` plus configured member repositories.

## Examples

Push default targets:

```sh
gwz push
```

Push configured members but not the workspace root:

```sh
gwz --all --no-target @root push
```

Push to an explicit remote:

```sh
gwz --remote origin push
```

Push one member by id:

```sh
gwz --member gwz-cli push
```

Push only the workspace root:

```sh
gwz --target @root push
```

Preview planned push behavior:

```sh
gwz --dry-run push
```

## Notes

- `gwz push` has no command-specific options.
- Use global selectors to control which targets participate.
- `--all --no-target @root` is the canonical all-members-only selector.
- Use `gwz tag --push` for tag push workflows.
- Network behavior is controlled by global options such as `--jobs`,
  `--max-per-host`, `--progress-interval`, and `--ssh-timeout`.
