# GWZ Config and Aliases (`.gwzconfig`)

Status: draft

Scope: user configuration and command aliases for `gwz-cli`. Mirrors Git’s
`gitconfig` model (`~/.gitconfig`, `.git/config`); aliases expand **before**
clap parsing so they never appear in default `--help` output.

## Problem

Users want personal shortcuts (`gwz s` → `gwz status`) and stable defaults
(`--jobs`, progress behavior) without forking the CLI or wrapping scripts.
Registering aliases as clap subcommands would pollute generated usage and
completion.

## Goals

- Layered config files in Git’s INI style (`[section]`, `key = value`).
- `[alias]` entries that rewrite argv before clap runs.
- Built-in `gwz --help` lists **only** real commands and flags.
- `gwz config` / `gwz help aliases` lists user-defined aliases and values.
- Workspace-local overrides plus global user config, with naming parallel to Git.

## Non-Goals

- Shell-escape aliases (`!sh -c …`) in v0 (Git’s `!` prefix — defer).
- Remote or shared config sync.
- Aliases that replace `gwz-core` protocol actions (CLI-only sugar).
- A new config syntax (no TOML/YAML for v0 — use gitconfig INI).

## Config files

### Locations (lowest → highest precedence)

Mirrors Git’s `git config` layering:

| Layer | GWZ path | Git analogue |
|-------|----------|--------------|
| System | `/etc/gwzconfig` | `/etc/gitconfig` |
| Global | `~/.gwzconfig` | `~/.gitconfig` |
| XDG global | `$XDG_CONFIG_HOME/gwz/config` | `$XDG_CONFIG_HOME/git/config` |
| Workspace local | `gwz.conf/config` | `.git/config` |
| Extra include | `$GWZ_CONFIG` (file path) | `$GIT_CONFIG` |
| Command line | `gwz -c key=value` | `git -c key=value` |

Later layers override earlier keys. Workspace local applies only after a
workspace root is discovered (`gwz.conf/gwz.yml`).

Precedence within global: if `~/.gwzconfig` exists, it wins over the XDG file;
if only XDG exists, use that. (Same practical rule as Git.)

### Format

Git-compatible INI subset:

```ini
# ~/.gwzconfig
[core]
    jobs = 4
    progress = auto

[alias]
    s = status
    st = status --no-combined
    ph = pull --head
    ps = push
    wip = stash push -a -m wip

[pull]
    sync = ff-only

[progress]
    interval-ms = 500
```

Rules:

- `#` and `;` start comments.
- Section and key names are case-sensitive.
- Quoted values follow gitconfig quoting rules for values with spaces.
- Unknown sections/keys are ignored (forward compatible).

### Reserved paths

`gwz.conf/config` is workspace metadata (like `.git/config`), not a member
repository. It lives under tracked `gwz.conf/` and MAY be committed with the
workspace when teams want shared aliases or defaults.

## Aliases

### Definition

```ini
[alias]
    s = status
    up = pull --head
```

Maps the first **command token** after global options to a replacement token
list.

### Expansion (pre-clap)

```text
raw argv
  -> load merged config
  -> skip global flags + values (see below)
  -> if next token matches [alias], replace with shlex-split expansion
  -> repeat until no alias or max depth
  -> Cli::try_parse_from(expanded argv)
```

**Aliases MUST NOT be registered in clap.** Only argv rewriting.

### Precedence vs built-in commands

If an alias name collides with a built-in subcommand (`status`, `init`, …),
**built-in wins** (unlike Git, where alias can shadow). Rationale: predictable
help and scripts; users can pick non-colliding short names (`st`, `s`).

### Recursion and limits

- Max alias expansion depth: **20** (Git uses similar guard).
- Circular aliases (`a = b`, `b = a`) MUST error with a clear message.
- Expansion is token substitution only in v0 — no nested `!shell`.

### Global flags before alias

GWZ globals that MUST be skipped before resolving the command token:

| Flag | Takes value |
|------|-------------|
| `--root` | yes |
| `--member` | yes (repeatable) |
| `--member-path` | yes (repeatable) |
| `--all` | no |
| `--dry-run` | no |
| `--partial` | no |
| `--force` | no |
| `--sync` | yes |
| `--remote` | yes |
| `--jobs` | yes |
| `--json` / `--jsonl` | no |
| `--quiet` / `--progress` | no |
| `-c` / `--config` | yes (when implemented) |

Example:

```text
gwz --root /ws --member mem_app s
  -> gwz --root /ws --member mem_app status
```

### Examples

```text
gwz s                    # status
gwz ph                   # pull --head
gwz wip                  # stash push -a -m wip
gwz config --get alias.s # show expansion (git-style)
```

## Non-alias config (v0)

Keys the CLI reads after merge (extensible):

| Section | Key | Type | Default | Effect |
|---------|-----|------|---------|--------|
| `core` | `jobs` | int | — | Default `--jobs` when flag omitted |
| `core` | `progress` | `auto\|on\|off` | `auto` | Progress mode (`GwzProgressSpec.md`) |
| `pull` | `sync` | sync enum | `ff-only` | Default `--sync` for pull |
| `progress` | `interval-ms` | int | `500` | stderr refresh cadence |

Unset keys leave clap defaults unchanged. Explicit CLI flags always override
config.

## `gwz config`

Git-shaped introspection and editing:

```text
gwz config <name>                    # get last-wins value
gwz config --get <name>              # same (git alias)
gwz config --list                    # all keys
gwz config --list alias              # aliases only
gwz config --show-origin <name>
gwz config --global <name> <value>   # write ~/.gwzconfig
gwz config --workspace <name> <value>  # write gwz.conf/config
```

`name` uses section.key form: `alias.s`, `core.jobs`, `pull.sync`.

Writing MUST preserve comments in file when practical; v0 MAY rewrite whole
file atomically for simplicity.

## Help and usage

| Command | Shows |
|---------|--------|
| `gwz --help` | Built-in subcommands and flags only (clap-generated) |
| `gwz help aliases` | All `alias.*` from merged config with expansions |
| `gwz config --list alias` | Same as above, scriptable |
| `gwz <alias> --help` | Expand alias, then show help for resolved command |

Aliases MUST NOT appear in `gwz --help` subcommand list or shell completion for
built-in commands (completion MAY offer configured aliases separately).

## Implementation

### `gwz-cli` (primary)

1. `config` module: load/merge INI files, `get(section, key)`.
2. `alias` module: `expand_aliases(args, config) -> Vec<String>`.
3. `main`: `let args = expand_aliases(env::args(), &config)?; Cli::try_parse_from(args)`.
4. `gwz config` subcommand (v0: list/get; set in v0.1 if needed).
5. Tests: expansion with globals, depth limit, built-in shadowing, no help leak.

### `gwz-core`

No requirement for v0. Config stays driver-local unless later operations need
defaults from workspace config (then pass resolved policy in requests).

### Dependencies

- INI parser: small crate (e.g. `git-config` or `config` with custom INI) or
  minimal hand-rolled gitconfig subset.
- `shlex` (or equivalent) for alias value splitting.

Estimated effort: **one focused PR** for load + alias expand + tests; second
small PR for `gwz config` list/get.

## Errors

| Case | Message shape |
|------|----------------|
| Alias loop | `gwz: alias loop detected for 'a'` |
| Max depth | `gwz: alias expansion exceeded 20 steps` |
| Bad config file | `gwz: bad config file ~/.gwzconfig: …` |
| Unknown `gwz config` key | exit 1, no value printed |

## Security

- No `!` shell aliases in v0.
- Config files are trusted local input; do not execute values.
- `credential` or secret keys are out of scope (see `gwz-core` attribution).

## Open decisions

- Whether `gwz config set` is v0 or v0.1.
- Include/exclude aliases from shell completion via `gwz completion`.
- Whether workspace `gwz.conf/config` is gitignored by default or committed for
  team aliases.

## Related

- `GwzProgressSpec.md` — `core.progress`, `progress.interval-ms`
- `GwzStashSpec.md` — alias example `wip = stash push …`
- `gwz-cli/src/main.rs` — clap `Parser` / `try_parse_from` entry point
