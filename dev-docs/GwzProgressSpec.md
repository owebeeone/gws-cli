# GWZ CLI Progress Reporting

Status: draft

Scope: terminal progress for long-running `gwz` workspace operations. Applies to
`gwz-cli` rendering and the `gwz-core` event/Git surfaces it consumes.

## Problem

Many GWZ commands block until completion and print nothing until the final human
summary. For multi-repository init, materialize, pull, and push this can mean
tens of seconds with no feedback — especially painful over SSH or slow remotes.

The protocol already defines `OperationEvent` and an async operation runtime, but
today:

- `gwz-cli` calls `gwz-core` handlers synchronously and renders only the final
  response.
- Handlers emit no `member_started` / `member_progress` / `member_finished`
  events during Git work.
- `Git2Backend` does not wire libgit2 transfer-progress callbacks.

## Goals

- When `gwz` runs attached to an interactive terminal, long-running operations
  MUST show progress on **stderr**.
- On acceptance, print one immediate summary line (e.g. `Cloning 3
  repositories…`).
- While work continues, refresh progress at most every **500 ms** with:
  - workspace-level counts (members done / total, active members when `--jobs` > 1),
  - per-active-member phase and Git transfer stats when available.
- Behavior MUST follow Git’s conventions for when progress is on vs off.
- `--json` / `--jsonl` / `--porcelain` consumers MUST NOT get spinner/progress
  lines mixed into structured output.

## Non-Goals

- Changing final human result format on stdout (status summaries, per-member
  lines, file lists).
- Progress for fast, read-only commands (`status`, `snapshot`, `tag`, empty
  `init`).
- A TUI, percentage bars in log files, or web UI widgets.
- Replacing `--jsonl` as the machine integration surface.

## Git Precedents (research)

Git’s behavior is the reference for terminal UX and gating.

| Git behavior | GWZ equivalent |
|--------------|----------------|
| Progress writes to **stderr**, normal output to stdout | Progress on stderr; final GWZ summary stays on stdout |
| Progress default **on** when stderr is a TTY | Same |
| Progress **off** when stderr is not a TTY (piped, CI) | Same |
| `--quiet` / `-q` suppresses progress | Global `--quiet` suppresses progress |
| `--progress` forces progress even without TTY | Optional `--progress` (may still omit `\r` refresh when not a TTY) |
| Sideband progress during fetch/clone/push | Map libgit2 `transfer_progress` / sideband to `member_progress` events |
| Phases: counting → compressing → receiving → resolving deltas (clone/fetch) | Surface as `phase` + object counts in progress events |
| Checkout shows file progress for large tree checkouts | Optional checkout phase on materialize/pull-to-snapshot |
| Carriage-return (`\r`) overwrites the current progress line | Same for live updates; emit `\n` before final stdout |

Example Git clone stderr (abbreviated):

```text
Cloning into 'repo'...
remote: Enumerating objects: 1234, done.
remote: Counting objects: 100% (1234/1234), done.
Receiving objects:  67% (826/1234), 2.10 MiB | 1.20 MiB/s
Resolving deltas: 100% (456/456), done.
```

GWZ adds a **workspace shell** above this: one line that names how many member
repositories are involved, then nests the active member’s Git phase underneath.

## Terminal Detection

Progress mode is **active** when all of the following hold:

1. Output mode is `Human` (not `--json`, `--jsonl`, or `--porcelain`).
2. `--quiet` is not set.
3. Either stderr is a TTY, or `--progress` is explicitly set.

When progress mode is inactive, behavior stays as today: block, then print final
output once.

Implementation note: use `std::io::IsTerminal` on stderr (Rust 1.70+). Do not use
stdout alone — Git keys off stderr so progress can coexist with stdout piping.

## Commands and Acknowledgment Lines

Long-running commands that MUST support progress when active:

| Command | Acknowledgment template (examples) |
|---------|-------------------------------------|
| `gwz init <url>…` | `Initializing workspace: cloning N repositories…` |
| `gwz materialize` | `Materializing N repositories to <target>…` |
| `gwz pull --head` | `Pulling N repositories to head…` |
| `gwz pull --snapshot <name>` | `Pulling N repositories to snapshot <name>…` |
| `gwz push` | `Pushing N repositories…` |

`<target>` is one of `lock`, `head`, snapshot name, or tag name derived from
flags.

N is the count of **selected active members** that will do work (exclude members
planned as `noop` / `skipped` when known at accept time). If N is 0, skip
progress (emit final result only).

Short commands (`status`, `add`, `snapshot`, `tag`, `repo create`, empty `init`)
MAY omit progress entirely.

### Acknowledgment timing

The acknowledgment line MUST appear as soon as the operation is accepted — i.e.
when the handler knows the selection and plan, before Git network I/O begins.

Today handlers return a final `Ok` envelope synchronously. Implementation
requires one of:

1. **Preferred:** route long-running commands through `OperationRuntime::submit`,
   print acknowledgment from the immediate `Accepted` response (member plans),
   subscribe to events, then `wait` for the final result; or
2. **Interim:** emit a planning-phase acknowledgment from synchronous handlers
   before Git calls, then run Git work (no mid-flight events until core emits
   them).

The preferred path aligns with the existing protocol and `--jsonl` streaming.

## Progress Model

Two layers, aggregated by the CLI renderer.

### Layer 1 — Workspace

Tracked state:

```text
total_members      # selected members that will execute (not skipped/noop)
finished_members   # member_finished with Ok/Failed/Skipped
active_members     # up to concurrency limit, currently running
current_phase      # highest-level verb: cloning | fetching | pushing | checking-out | writing-lock
```

Rendered fragment examples:

```text
Cloning 3 repositories: 1/3 done
Pulling 3 repositories: 2/3 done, 1 active
Pushing 3 repositories: 0/3 done, 2 active
```

When `--jobs` > 1, show active count; when `jobs == 1`, show the single active
member path instead of “1 active”.

### Layer 2 — Active member Git transfer

When a member is in a network or checkout phase, include Git-style stats from
core events:

```text
gwz-core: receiving objects 67% (826/1234), 2.1 MiB
taut: writing objects 45% (90/200), 512 KiB
```

Phases (map from libgit2 / sideband where possible):

| Phase | Typical operation |
|-------|-------------------|
| `enumerating` | Remote counting objects |
| `counting` | Local enumeration |
| `compressing` | Push pack generation |
| `receiving` | Clone/fetch pack receive |
| `resolving` | Delta resolution |
| `checking-out` | Tree checkout to worktree |
| `writing` | Push upload |

If only unstructured text is available (sideband line from remote), pass it
through as `member_progress.message` and show the latest line for that member.

## Protocol and Core Events

Use existing `OperationEvent` kinds; extend only when string messages are
insufficient.

### Required event sequence (per member)

```text
operation_started
member_started        (member_id, member_path, planned action summary)
member_progress       (0..n, throttled in core or CLI)
member_finished       (terminal member status)
operation_finished
```

Handlers for init, materialize, pull, and push MUST emit `member_started` before
Git work and `member_finished` after.

### `member_progress` content (v0)

v0 MAY use the existing optional `message` field with a stable, parseable
prefix-free human string, e.g.:

```text
receiving objects 67% (826/1234), 2.1 MiB
```

v1 SHOULD add a structured payload to avoid string parsing (taut extension):

```python
Msg("GitTransferProgress",
    F("phase", 1, Ref("GitProgressPhase")),
    F("received_objects", 2, INT, optional=True),
    F("total_objects", 3, INT, optional=True),
    F("received_bytes", 4, INT, optional=True),
    F("indexed_deltas", 5, INT, optional=True),
    F("total_deltas", 6, INT, optional=True)),
```

Add optional `progress=F(..., Ref("GitTransferProgress"))` on `OperationEvent`.
Until then, CLI parses v0 messages with a small fixed grammar or displays
`message` verbatim under the workspace line.

### Git backend

Extend `GitBackend` network methods to accept an optional progress sink:

```text
clone_repo(..., progress: Option<&mut dyn GitProgressSink>)
fetch(..., progress: Option<&mut dyn GitProgressSink>)
push(..., progress: Option<&mut dyn GitProgressSink>)
```

`Git2Backend` MUST register libgit2 `RemoteCallbacks::transfer_progress` and
forward sideband progress when a sink is present. Checkout progress MAY use
`checkout_progress` on `CheckoutBuilder` for large checkouts.

The sink forwards into the operation `EventSink` as `MemberProgress`, coalesced
to at most one event per member per **100 ms** in core (CLI still renders at
500 ms).

## CLI Rendering

### Streams

| Stream | Content |
|--------|---------|
| stderr | Acknowledgment, live progress lines, errors before exit |
| stdout | Final human summary (unchanged) |

Before writing final stdout, print `\n` to stderr if the last progress line used
`\r`.

### Refresh cadence

- **500 ms** minimum interval between visible progress updates (user requirement).
- Between renders, drain all pending events and merge into one snapshot.
- Use `\r` + clear-to-EOL when stderr is a TTY; plain newline-separated lines
  when `--progress` is set but stderr is not a TTY.

### Example session

```text
$ gwz pull --head
# stderr (updates in place):
Pulling 3 repositories to head…
Pulling 3 repositories: 0/3 done, 1 active — gwz-core: receiving objects 12% (40/334), 180 KiB
# … 500 ms later …
Pulling 3 repositories: 1/3 done, 1 active — taut: receiving objects 55% (210/381), 900 KiB
# …
Pulling 3 repositories: 3/3 done
# stdout:
status: Ok
mem_gwz_cli gwz-cli Ok branch=main …
…
```

Dry-run (`--dry-run`): print acknowledgment from plan only, then final planned
response — no Git progress.

## Flags

| Flag | Effect |
|------|--------|
| `--quiet` / `-q` | No progress lines; final output unchanged |
| `--progress` | Enable progress even when stderr is not a TTY |
| `--json` / `--jsonl` / `--porcelain` | Implicit quiet for progress (structured modes only) |

`--jsonl` continues to emit response / event / result records; progress lines
MUST NOT appear on stderr when `--jsonl` is set (events carry progress instead).

## Implementation Plan

### Phase A — CLI shell (gwz-cli)

1. Detect progress mode (TTY + output mode + flags).
2. For long-running commands, switch to async execution:
   - submit via `OperationRuntime` (or equivalent public API on `gwz-core`),
   - print acknowledgment from accepted envelope,
   - loop: `subscribe` → merge events → maybe render (500 ms) → `try_result` /
     `wait`,
   - clear progress line, print final human response on stdout.
3. Add `--quiet` and optional `--progress` global flags.
4. Unit tests: TTY gating logic, throttle merge, acknowledgment text from fixture
   envelopes, no stderr progress when `--json`.

### Phase B — Core events (gwz-core)

1. Wire init / materialize / pull / push through `OperationRuntime::submit`.
2. Emit `member_started` / `member_finished` around each member execution.
3. Parallel member execution MUST still emit per-member events (ordering may
   interleave).

### Phase C — Git transfer progress (gwz-core)

1. Add `GitProgressSink` and wire libgit2 callbacks.
2. Emit `member_progress` during clone, fetch, push.
3. Optional checkout progress on `checkout_commit` for materialize/pull.

### Phase D — Structured progress (optional)

1. Extend taut schema with `GitTransferProgress`.
2. Teach CLI renderer to prefer structured fields over message parsing.

## Testing

| Area | Test |
|------|------|
| CLI | Progress mode off when stdout piped and stderr not TTY |
| CLI | Progress mode on when stderr is TTY |
| CLI | `--quiet` suppresses stderr progress |
| CLI | `--jsonl` has no stderr progress; events include member_progress |
| CLI | Throttle: many events in 50 ms → at most 2 renders in 600 ms |
| CLI | Acknowledgment strings for init/materialize/pull/push fixtures |
| Core | member_started/finished emitted for each member in pull fixture |
| Core | transfer_progress callback fires during clone in temp repo test |
| E2E | Local bare remote: `gwz pull --head` produces monotonic finished count |

Use scripted non-TTY tests for assertions; manual or PTY test for `\r` overwrite
smoke.

## Open Decisions

- Public API name for CLI async entry (`gwz_core::execute_async` vs exposing
  `OperationRuntime` directly).
- Whether acknowledgment uses `ResponseMeta.message` or only derived text from
  member plans.
- v0 structured progress vs message-only (recommend message-only first, structured
  in Phase D).

## Related Docs

- `gwz-core/protocol/gws.taut.py` — `OperationEvent`, `EventKind`
- `gwz-core/src/operation/mod.rs` — runtime, `EventSink`
- `gwz-cli/src/main.rs` — current synchronous execute/render path
