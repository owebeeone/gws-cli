# GWZ Member Git: Selection, Commit, and Git Execution

Status: draft

Scope: running git (and git-shaped) work on **subsets** of workspace members with
GWZ-consistent policy. Covers the libgit2 vs `git` CLI split, `gwz commit`, and
the `gwz run` escape hatch (west `forall` / vcstool `custom`).

## Problem

GWZ today uses **libgit2** (`Git2Backend`) for workspace-coordinated operations
(clone, fetch, ff, checkout, push, status). That is right for deterministic,
testable workspace semantics — but it cannot replace the full git CLI (hooks,
GPG, credential UI, arbitrary porcelain).

Users also need:

- **Subset execution** on member repos (not always all members).
- **`gwz commit`** — coordinated commits across a selection (missing today).
- **Occasional passthrough** for ad hoc commands without GWZ reimplementing git.

GWZ must **not** parse or emulate the entire git CLI. It should expose a small
set of structured workspace commands plus one controlled escape hatch.

## Goals

- One **selection model** shared by status, pull, push, stash, commit, and run.
- **`gwz commit`** as a first-class coordinated command.
- **`gwz run`** to execute a shell command in each selected member (west
  `forall` / `vcs custom`).
- **Hybrid git execution**: libgit2 for workspace kernel ops; `git` subprocess
  for member-local porcelain that users already rely on.
- Atomic-by-default preflight for mutating multi-member commands (existing GWZ
  discipline).

## Non-Goals

- `gwz` as a git argv parser (`gwz git log --oneline` forwarding any git
  subcommand tree).
- Replacing libgit2 for materialize/pull/push kernel paths in v0.
- Manifest **groups** (west `-g`) in v0 — defer; explicit ids/paths first.

## Selection model (v0)

GWZ already has global `--member`, `--member-path`, and `--all`. This spec makes
that the **only** subset mechanism for v0 and applies it uniformly.

| Input | Resolution |
|-------|------------|
| *(none)* | All **active** members |
| `--all` | All active members (cannot combine with filters) |
| `--member <id>` … | Union of listed member ids |
| `--member-path <path>` … | Union of listed member paths |

Rules (already in core for status; MUST apply everywhere):

- Unknown, inactive, or ambiguous members → typed error before work.
- Deterministic sort order for execution and output.
- Selection is resolved once; per-member outcomes reference the same set.

### Comparison to west / vcstool

| Tool | Subset mechanism | GWZ v0 equivalent |
|------|------------------|-----------------|
| west | `[PROJECT …]` names/paths | `--member` / `--member-path` |
| west | `-g group` | deferred |
| vcstool | path roots (recursive discover) | `--member-path` only (manifest-known) |
| vcstool | `--git` type filter | implicit (git members only) |

Future: manifest **groups** and `gwz.conf` default selection — not v0.

## Git execution: two layers

```text
┌─────────────────────────────────────────────────────────┐
│  GWZ workspace layer (gwz-core)                         │
│  materialize, pull, push, lock, snapshot, status scan   │
│  → libgit2 GitBackend (deterministic, no shell)         │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│  Member porcelain layer (new)                           │
│  commit, run, (later: diff, log, branch)                │
│  → git subprocess per member (`git -C <path> …`)        │
└─────────────────────────────────────────────────────────┘
```

### Why subprocess for porcelain

| Concern | libgit2 | `git` subprocess |
|---------|---------|------------------|
| Hooks (pre-commit, commit-msg) | limited / different | native |
| GPG signing | painful | native |
| User config (`commit.gpgsign`, etc.) | bypassed | honored |
| Credential helpers / SSH | partial | native |
| Testability in CI | excellent | needs git installed |

**Rule:** workspace kernel stays on `GitBackend`; member porcelain spawns
`git` with a pinned, documented argv template per GWZ command.

### Subprocess contract

- Binary: `git` on `PATH` (or `core.git` from `~/.gwzconfig` later).
- Working directory: member repo root (`workspace_root / member.path`).
- Environment: inherit caller env; optional `GWZ_*` markers for hooks.
- No shell unless `gwz run` explicitly requests it.
- stdout/stderr captured per member for aggregation; TTY attach only when
  `--interactive` / single-member / progress spec says so.
- Failures map to typed per-member errors (`git_command_failed`).

`GitBackend` MAY gain a thin `run_git(path, args) -> GitRunResult` used by
porcelain commands only.

## `gwz commit`

Coordinated `git commit` across a selection. Structured GWZ command — **not**
open-ended git parsing.

### CLI

```text
gwz commit -m <message> [--member …] [--member-path …] [--all]
gwz commit -m <message> -a
gwz commit --amend -m <message>   # v0.1 optional
```

Global selection flags apply. Additional flags (v0 minimal set):

| Flag | Maps to |
|------|---------|
| `-m` / `--message` | `git commit -m` (required unless amend with kept message) |
| `-a` / `--all` | `git commit -a` (stage tracked modifications) |
| `--no-verify` | `git commit --no-verify` |

Defer v0: `-s` signoff, `--amend`, pathspec commits, empty-message opens
editor (use `-m` only in v0).

### Behavior

1. Resolve selection.
2. Preflight each selected git member:
   - is a repository;
   - has staged changes (or `-a` and committable tracked changes);
   - not in a conflicted merge state (if detectable).
3. Members with **nothing to commit** → `noop` (skipped), not an error.
4. If any member has **staged changes but commit would fail** → atomic reject
   before any commit (default).
5. Run `git -C <member> commit …` in parallel (respect `--jobs`).
6. Report per-member new commit oid + aggregate status.
7. Do **not** rewrite `gwz.conf/gwz.lock.yml` automatically (commit ≠
   workspace pin); user runs `gwz snapshot` when they want a workspace record.

Optional `--strict`: fail if any selected member is `noop` (all must commit).

### Output (human)

```text
status: Ok
mem_gwz_core gwz-core Ok commit=abc1234
mem_taut taut Noop nothing to commit
mem_gwz_cli gwz-cli Ok commit=def5678
```

### Protocol

Add `CommitRequest` / `CommitResponse` to taut (`ActionKind::commit`) with
`message`, `all` (stage tracked), `no_verify`, selection, policy.

## `gwz run`

> **Superseded (2026-06-24):** replaced by **`gwz forall`** — see
> `gwz-core/dev-docs/GWZForallPlan.md`. Net deltas: the verb is `forall`; the executor is
> **CLI-only** (no in-core `GitCliRunner`); v0 is **sequential** (`--jobs`/parallel deferred);
> `--partial` still means continue, default stops at the first failure. The text below is historical.

Controlled escape hatch — west `forall` / vcstool `custom`. Runs an arbitrary
command in each selected member directory.

### CLI

```text
gwz run -c "<command>" [--member …] [--all]
gwz run -- <command> …        # alternate form
```

Examples:

```text
gwz run -c "git log --oneline -n 5"
gwz --member mem_app run -c "npm test"
gwz run -c "git branch -vv"
```

### Behavior

- Execute via shell (`sh -c` on Unix) **in member repo root**.
- Selection + `--jobs` concurrency (default: core count).
- `--workers 1` / `--jobs 1` for interactive or stdin-driven commands (vcstool
  pattern).
- Per-member exit code captured; aggregate `failed` if any non-zero (default).
- `--partial` (existing global): continue on member failure.

**GWZ does not parse git** here — the user supplies the full command string.
GWZ only chooses **which repos** and **how parallel**.

### Safety

- Document that `run` is power-user / escape hatch.
- Prefer structured commands (`commit`, `status`, `pull`) for common flows.
- No `run` from config aliases that hide destructive commands (alias spec
  unchanged).

## Command matrix (subset + execution backend)

| Command | Subset flags | Backend v0 |
|---------|--------------|------------|
| `status` | yes | libgit2 |
| `pull` / `push` / `materialize` | yes | libgit2 |
| `stash` | yes | libgit2 + subprocess (stash push/pop) |
| `commit` | yes | **subprocess** |
| `run` | yes | **subprocess** (shell) |
| `snapshot` / `tag` | yes | artifact I/O + libgit2 read |

## Implementation phases

### Phase 1 — Selection uniformity

- Audit all handlers use shared `resolve_selection`.
- CLI: document globals on every multi-member command.
- Tests: subset resolution parity across status/push/pull.

### Phase 2 — `git run` helper + `gwz run`

- `GitBackend::run_git` or separate `GitCliRunner` in `gwz-core`.
- `gwz run -c` with jobs, per-member exit codes.
- Tests with temp repos + `git rev-parse`.

### Phase 3 — `gwz commit`

- Taut messages + handler.
- Preflight (staged/dirty/conflict), noop members, atomic default.
- Subprocess `git commit -m`.

### Phase 4 — Stash / GPG / editor (optional)

- Stash push/pop via subprocess (hooks, `-m` message).
- `commit` editor support (`-m` omitted → single shared editor session) — defer.

## Open decisions

- Whether `noop` members on `gwz commit` should print a warning on stderr.
- `core.git` config key for non-`PATH` git binary.
- Auto-update lock on commit: **no** for v0 (explicit snapshot).
- `gwz diff` / `gwz log` as structured commands vs `gwz run` only.

## Related

- `GwzStashSpec.md` — coordinated stash bundles
- `GwzRcSpec.md` — `alias.ph = pull --head`, etc.
- `GwzProgressSpec.md` — progress on long commit/push/pull
- `gwz-core/src/git/mod.rs` — `GitBackend` trait today
