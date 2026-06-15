# gws-cli

`gws-cli` provides the `gws` command-line driver for `gws-core`.

The CLI is intentionally thin: it parses argv, builds GWS requests, calls
`gws-core`, and renders responses/events.

## Current Commands

```text
gws init
gws init <url>...
gws add <repo-path>
gws repo create <member-path>
gws status
gws status --no-combined
gws status --porcelain
gws snapshot <name>
gws tag <name>
gws materialize --lock
gws materialize --snapshot <name>
gws materialize --tag <name>
gws pull --head
gws pull --snapshot <name>
gws push
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
gws --root /tmp/ws init /tmp/source.git
gws --root /tmp/ws status --json
gws --root /tmp/ws status --no-combined --json
gws --root /tmp/ws snapshot snap_one
gws --root /tmp/ws pull --head
gws --root /tmp/ws push --remote origin
```

## Development

```text
cargo fmt
cargo test
cargo fmt --check
cargo run -- --version
```
