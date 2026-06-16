# gwz-cli

`gwz-cli` provides the `gwz` command-line driver for `gwz-core`.

The CLI is intentionally thin: it parses argv, builds GWZ requests, calls
`gwz-core`, and renders responses/events.

## Current Commands

```text
gwz init
gwz init <url>...
gwz add <repo-path>
gwz repo create <member-path>
gwz status
gwz status --no-combined
gwz status --porcelain
gwz snapshot <name>
gwz tag <name>
gwz materialize --lock
gwz materialize --snapshot <name>
gwz materialize --tag <name>
gwz pull --head
gwz pull --snapshot <name>
gwz push
```

Common flags:

```text
--root <path>
--member <member-id>
--path <member-path>
--all
--dry-run
--partial
--force
--sync <fetch-only|ff-only|merge|rebase|reset|driver-selected>
--remote <name>
--jobs <n>
--json
--jsonl
```

Status-specific flags:

```text
--combined
--no-combined
--porcelain
--no-files
--no-branches
```

Examples:

```text
gwz --root /tmp/ws init /tmp/source.git
gwz --root /tmp/ws status --json
gwz --root /tmp/ws status --no-combined --json
gwz --root /tmp/ws snapshot snap_one
gwz --root /tmp/ws pull --head
gwz --root /tmp/ws push --remote origin
```

## Development

```text
cargo fmt
cargo test
cargo fmt --check
cargo run -- --version
```

## CLI Help And Docs

CLI help is generated from the command parser. The `clap` command definitions
SHOULD be the source of truth for terminal help and generated Markdown reference
docs such as `docs/CLI.md`.

## License

`gwz-cli` is licensed under GPL-2.0-only, the same license family used by Git.
