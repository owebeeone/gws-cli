# gwz

`gwz` is the command-line driver for `gwz-core`.

The CLI is intentionally thin: it parses argv, builds GWZ requests, calls
`gwz-core`, and renders responses/events.

## Current Commands

```text
gwz init
gwz init --path <path-prefix> <url>...
gwz init <url>...
gwz clone <url> [directory]
gwz repo add <repo-path>
gwz repo create <member-path>
gwz add <pathspec>...
gwz add -A
gwz commit -m <message>
gwz commit -a -m <message>
gwz status
gwz status --no-combined
gwz status --porcelain
gwz snapshot <name>
gwz tag <name>
gwz tag --list [--remote <name>]
gwz tag --delete <name> [--remote <name>]
gwz tag --push [<name>] [--remote <name>]
gwz tag --fetch [--remote <name>]
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
--member-path <member-path>
--all
--dry-run
--partial
--force
--sync <fetch-only|ff-only|merge|rebase|reset|driver-selected>
--remote <name>
--jobs <n>
--max-per-host <n>
--progress-interval <ms>
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
gwz --root /tmp/ws init --path repos /tmp/source.git
gwz clone /tmp/ws.git /tmp/ws-clone
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

## Install

Install the latest release on macOS or Linux:

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh | sh
```

Install the latest release on Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1 | iex"
```

The `latest` URLs point at the newest non-prerelease GitHub Release. If you want
a pinned install, replace `latest` with a concrete tag such as `v0.1.0`:

```text
https://github.com/owebeeone/gwz-cli/releases/download/v0.1.0/gwz-installer.sh
```

Users who already have Rust can install from source:

```sh
cargo install --git https://github.com/owebeeone/gwz-cli
```

### Smoke Test Installers

Test the Unix installer without modifying `PATH`:

```sh
tmp="$(mktemp -d)"

curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.sh \
  -o "${tmp}/gwz-installer.sh"

GWZ_UNMANAGED_INSTALL="${tmp}/bin" \
GWZ_NO_MODIFY_PATH=1 \
sh "${tmp}/gwz-installer.sh"

"${tmp}/bin/gwz" --version
"${tmp}/bin/gwz" --help
```

Test the Windows installer without modifying `PATH`:

```powershell
$ErrorActionPreference = "Stop"

$tmp = Join-Path $env:TEMP "gwz-test-$([guid]::NewGuid())"
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

$installer = Join-Path $tmp "gwz-installer.ps1"
Invoke-WebRequest `
  "https://github.com/owebeeone/gwz-cli/releases/latest/download/gwz-installer.ps1" `
  -OutFile $installer

$env:GWZ_UNMANAGED_INSTALL = Join-Path $tmp "bin"
$env:GWZ_NO_MODIFY_PATH = "1"

Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
& $installer

$exe = Join-Path $env:GWZ_UNMANAGED_INSTALL "gwz.exe"
& $exe --version
& $exe --help
```

Release assets are checksummed and have GitHub artifact attestations. The
installers are convenience scripts; users who want stronger verification should
download the release asset, verify the attestation, compare the SHA-256 checksum,
and then install.

## CLI Help And Docs

CLI help is generated from the command parser. The `clap` command definitions
SHOULD be the source of truth for terminal help and generated Markdown reference
docs such as `docs/CLI.md`.

## License

`gwz` is licensed under GPL-2.0-only, the same license family used by Git.
