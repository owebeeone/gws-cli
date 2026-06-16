#[cfg(test)]
use clap::CommandFactory;
use clap::{Args, Parser, Subcommand, ValueEnum};

const CLI_LONG: &str = "\
GWZ manages a local workspace made from multiple git repositories.

A workspace records its member repositories and exact revisions under the
tracked `workspace/` directory. Commands operate on the workspace as a whole,
so a single request can initialize, inspect, snapshot, materialize, pull, or
push a coordinated set of repositories.";

const CLI_AFTER: &str = "\
Examples:
  gwz init git@github.com:org/app.git git@github.com:org/lib.git
  gwz status
  gwz snapshot before-refactor
  gwz pull --head";

const INIT_LONG: &str = "\
Create a workspace or initialize one from source URLs.

A GWZ workspace is a local directory that owns a tracked `workspace/` metadata
directory. `workspace/gwz.yml` describes the workspace and its repository
members. `workspace/gwz.lock.yml` records the exact revisions that make the
workspace reproducible.

Running `gwz init` with no URLs creates an empty workspace at `--root` or the
current directory. Passing one or more URLs creates the workspace and adds those
repositories as initial members, materialized from their heads.";

const INIT_AFTER: &str = "\
Examples:
  gwz init
  gwz --root /work/demo init
  gwz init git@github.com:org/app.git git@github.com:org/lib.git";

const ADD_LONG: &str = "\
Add an existing local git repository to the workspace.

Use this when a repository already exists on disk and should become a workspace
member. GWZ records the repository as a member; it does not clone a new copy.
Use `gwz repo create` instead when the member should be created from scratch.";

const ADD_AFTER: &str = "\
Examples:
  gwz add repos/app
  gwz --root /work/demo add /src/local-lib";

const REPO_LONG: &str = "\
Manage repository members inside a workspace.

Repository commands create or update the local member repositories that make up
the workspace. For v0, this group exposes member creation; use top-level
commands such as `gwz add`, `gwz status`, `gwz pull`, and `gwz push` for
workspace-wide operations.";

const REPO_AFTER: &str = "\
Examples:
  gwz repo create repos/new-service
  gwz help repo create";

const REPO_CREATE_LONG: &str = "\
Create a new local repository member and register it with the workspace.

The repository is created immediately at the requested member path and can be
pushed to a remote later. This supports the GWZ workflow where a workspace can
grow new repositories locally before deciding where they should be published.";

const REPO_CREATE_AFTER: &str = "\
Examples:
  gwz repo create repos/new-service
  gwz --root /work/demo repo create packages/experiment";

const STATUS_LONG: &str = "\
Show git status across workspace members.

The default mode requests a combined workspace status: file paths are reported
relative to the workspace and prefixed by member path when file entries are
available. Use `--no-combined` for per-member summaries. Use `--porcelain` when
another tool needs stable script-oriented output.";

const STATUS_AFTER: &str = "\
Examples:
  gwz status
  gwz status --no-combined
  gwz status --porcelain
  gwz --member mem_app status";

const SNAPSHOT_LONG: &str = "\
Record the current workspace selection as a named snapshot.

A snapshot captures the current member revisions so the workspace can later be
materialized back to the same coordinated state. Use snapshots before risky
multi-repository changes, before sharing a reproducible work area, or before
pulling all members forward.";

const SNAPSHOT_AFTER: &str = "\
Examples:
  gwz snapshot before-refactor
  gwz --all snapshot integration-baseline";

const TAG_LONG: &str = "\
Record a named GWZ workspace tag.

A GWZ tag is workspace metadata, not a git tag inside each member repository.
It stores the workspace-level mapping from member to revision, so the same tag
name can be meaningful inside this workspace without colliding with tags in
other workspaces or child repositories.";

const TAG_AFTER: &str = "\
Examples:
  gwz tag release-2026-06
  gwz materialize --tag release-2026-06";

const MATERIALIZE_LONG: &str = "\
Materialize workspace members to an explicit target.

Materialization makes the local repositories match a workspace target. It is not
raw `git pull`; GWZ plans the workspace operation first and applies the selected
target across members. With no target flag, `gwz materialize` uses the workspace
lock. Use `--head`, `--snapshot`, or `--tag` for a different target.";

const MATERIALIZE_AFTER: &str = "\
Examples:
  gwz materialize
  gwz materialize --lock
  gwz materialize --snapshot before-refactor
  gwz --force materialize --tag release-2026-06";

const PULL_LONG: &str = "\
Move workspace members forward to an explicit target.

`gwz pull` is a workspace operation, not a direct wrapper around `git pull`.
The default target is `--head`, and the default sync policy is fast-forward only.
If any selected member cannot update cleanly, the operation is rejected before
partial mutation unless `--partial` or another explicit policy changes that
behavior.";

const PULL_AFTER: &str = "\
Examples:
  gwz pull --head
  gwz pull --snapshot integration-baseline
  gwz --sync fetch-only pull --head
  gwz --partial pull --head";

const PUSH_LONG: &str = "\
Push workspace member refs to their configured remotes.

`gwz push` applies one push request across the selected workspace members. Use
`--remote` to choose a remote name and selection flags such as `--member`,
`--path`, or `--all` to control which members participate.";

const PUSH_AFTER: &str = "\
Examples:
  gwz push
  gwz push --remote origin
  gwz --member mem_app push";

fn main() {
    let cli = Cli::parse();
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            eprintln!("gwz: {error}");
            std::process::exit(1);
        }
    };

    match invocation_from_cli(cli, &new_request_id(), &cwd) {
        Ok(invocation) => match execute_invocation(&invocation) {
            Ok(response) => {
                println!("{}", render_response(&response, invocation.output));
                std::process::exit(exit_code_for_response(&response));
            }
            Err(error) => {
                eprintln!("gwz: {}", error.message);
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("gwz: {}", error.message);
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
fn usage_text() -> String {
    Cli::command().render_help().to_string()
}

#[derive(Clone, Debug, Parser)]
#[command(
    name = "gwz",
    version,
    about = "Manage GWZ multi-repository workspaces",
    long_about = CLI_LONG,
    after_long_help = CLI_AFTER,
    arg_required_else_help = true,
    subcommand_required = true
)]
struct Cli {
    #[command(flatten, next_help_heading = "Global Options")]
    global: GlobalArgs,

    #[command(subcommand)]
    command: CommandArgs,
}

#[derive(Clone, Debug, Default, Args)]
struct GlobalArgs {
    #[arg(
        long,
        global = true,
        value_name = "path",
        help = "Workspace root",
        long_help = "Workspace root. Defaults to the current directory when not supplied."
    )]
    root: Option<String>,

    #[arg(
        long = "member",
        global = true,
        value_name = "member-id",
        help = "Select a workspace member by id",
        long_help = "Select a workspace member by id. May be supplied more than once."
    )]
    members: Vec<String>,

    #[arg(
        long = "path",
        global = true,
        value_name = "member-path",
        help = "Select a workspace member by path",
        long_help = "Select a workspace member by path. May be supplied more than once."
    )]
    paths: Vec<String>,

    #[arg(
        long,
        global = true,
        help = "Select all workspace members",
        long_help = "Select all workspace members. Cannot be combined with `--member` or `--path`."
    )]
    all: bool,

    #[arg(
        long,
        global = true,
        help = "Plan the operation without mutating state",
        long_help = "Plan the operation without mutating workspace metadata or member repositories."
    )]
    dry_run: bool,

    #[arg(
        long,
        global = true,
        help = "Allow operations to complete partially",
        long_help = "Allow operations to complete for members that can proceed even when another selected member fails."
    )]
    partial: bool,

    #[arg(
        long,
        global = true,
        help = "Allow destructive behavior when required",
        long_help = "Allow destructive behavior when required. GWZ refuses destructive changes unless this is explicit."
    )]
    force: bool,

    #[arg(
        long,
        global = true,
        value_enum,
        value_name = "mode",
        help = "Select workspace sync behavior",
        long_help = "Select workspace sync behavior. The default policy is fast-forward only."
    )]
    sync: Option<SyncArg>,

    #[arg(
        long,
        global = true,
        value_name = "name",
        help = "Select the git remote name",
        long_help = "Select the git remote name used by operations that contact remotes."
    )]
    remote: Option<String>,

    #[arg(
        long,
        global = true,
        value_name = "n",
        value_parser = parse_positive_i64,
        help = "Maximum number of repos to process concurrently",
        long_help = "Maximum number of workspace member repositories to process concurrently."
    )]
    jobs: Option<i64>,

    #[arg(
        long,
        global = true,
        help = "Render one JSON response",
        long_help = "Render one structured JSON response for the operation."
    )]
    json: bool,

    #[arg(
        long,
        global = true,
        help = "Render newline-delimited JSON events",
        long_help = "Render newline-delimited JSON records for streaming operation consumers."
    )]
    jsonl: bool,
}

#[derive(Clone, Debug, Subcommand)]
enum CommandArgs {
    #[command(
        about = "Create a workspace or initialize one from source URLs",
        long_about = INIT_LONG,
        after_long_help = INIT_AFTER
    )]
    Init(InitArgs),
    #[command(
        about = "Add an existing git repository to the workspace",
        long_about = ADD_LONG,
        after_long_help = ADD_AFTER
    )]
    Add(AddArgs),
    #[command(
        about = "Manage workspace repositories",
        long_about = REPO_LONG,
        after_long_help = REPO_AFTER
    )]
    Repo(RepoArgs),
    #[command(
        about = "Show workspace git status",
        long_about = STATUS_LONG,
        after_long_help = STATUS_AFTER
    )]
    Status(StatusArgs),
    #[command(
        about = "Record the current workspace selection",
        long_about = SNAPSHOT_LONG,
        after_long_help = SNAPSHOT_AFTER
    )]
    Snapshot(NameArgs),
    #[command(
        about = "Record a named workspace tag",
        long_about = TAG_LONG,
        after_long_help = TAG_AFTER
    )]
    Tag(NameArgs),
    #[command(
        about = "Materialize workspace members to a target",
        long_about = MATERIALIZE_LONG,
        after_long_help = MATERIALIZE_AFTER
    )]
    Materialize(MaterializeArgs),
    #[command(
        about = "Update workspace members to an explicit target",
        long_about = PULL_LONG,
        after_long_help = PULL_AFTER
    )]
    Pull(PullArgs),
    #[command(
        about = "Push workspace member refs",
        long_about = PUSH_LONG,
        after_long_help = PUSH_AFTER
    )]
    Push,
}

#[derive(Clone, Debug, Args)]
struct InitArgs {
    #[arg(
        value_name = "url",
        help = "Git source URL to add as an initial workspace member",
        long_help = "Git source URL to add as an initial workspace member. May be supplied more than once."
    )]
    urls: Vec<String>,
}

#[derive(Clone, Debug, Args)]
struct AddArgs {
    #[arg(
        value_name = "repo-path",
        help = "Path to an existing local git repository"
    )]
    repo_path: String,
}

#[derive(Clone, Debug, Args)]
struct RepoArgs {
    #[command(subcommand)]
    command: RepoCommandArgs,
}

#[derive(Clone, Debug, Subcommand)]
enum RepoCommandArgs {
    #[command(
        about = "Create a new repository member",
        long_about = REPO_CREATE_LONG,
        after_long_help = REPO_CREATE_AFTER
    )]
    Create(RepoCreateArgs),
}

#[derive(Clone, Debug, Args)]
struct RepoCreateArgs {
    #[arg(
        value_name = "member-path",
        help = "Workspace-relative path for the new repository member"
    )]
    member_path: String,
}

#[derive(Clone, Debug, Args)]
struct StatusArgs {
    #[arg(
        long,
        help = "Render combined workspace status",
        long_help = "Render combined workspace status. This is the default mode."
    )]
    combined: bool,

    #[arg(
        long = "no-combined",
        help = "Render per-member status summaries",
        long_help = "Render per-member status summaries instead of one combined workspace view."
    )]
    no_combined: bool,

    #[arg(
        long,
        help = "Render porcelain output",
        long_help = "Render stable script-oriented output instead of human-readable text."
    )]
    porcelain: bool,

    #[arg(
        long = "no-files",
        help = "Omit file changes from combined status",
        long_help = "Omit file changes from combined status while keeping branch summaries."
    )]
    no_files: bool,

    #[arg(
        long = "no-branches",
        help = "Omit branch summaries from combined status",
        long_help = "Omit branch summaries from combined status while keeping file changes."
    )]
    no_branches: bool,
}

#[derive(Clone, Debug, Args)]
struct NameArgs {
    #[arg(value_name = "name", help = "Workspace-level name to record")]
    name: String,
}

#[derive(Clone, Debug, Default, Args)]
struct MaterializeArgs {
    #[arg(
        long,
        help = "Materialize the workspace lock",
        long_help = "Materialize the workspace lock. This is the default target."
    )]
    lock: bool,

    #[arg(long, help = "Materialize repository heads")]
    head: bool,

    #[arg(long, value_name = "name", help = "Materialize a workspace snapshot")]
    snapshot: Option<String>,

    #[arg(long, value_name = "name", help = "Materialize a workspace tag")]
    tag: Option<String>,
}

#[derive(Clone, Debug, Default, Args)]
struct PullArgs {
    #[arg(
        long,
        help = "Pull repository heads",
        long_help = "Pull repository heads. This is the default target."
    )]
    head: bool,

    #[arg(long, value_name = "name", help = "Pull a workspace snapshot")]
    snapshot: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum SyncArg {
    FetchOnly,
    FfOnly,
    Merge,
    Rebase,
    Reset,
    DriverSelected,
}

#[derive(Clone, Debug, PartialEq)]
struct CliInvocation {
    request: CliRequest,
    output: OutputMode,
    start_dir: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
enum CliRequest {
    CreateWorkspace(gwz_core::CreateWorkspaceRequest),
    InitFromSources(gwz_core::InitFromSourcesRequest),
    AddExistingRepo(gwz_core::AddExistingRepoRequest),
    CreateRepo(gwz_core::CreateRepoRequest),
    Materialize(gwz_core::MaterializeRequest),
    Status(gwz_core::StatusRequest),
    Snapshot(gwz_core::SnapshotRequest),
    Tag(gwz_core::TagRequest),
    PullHead(gwz_core::PullHeadRequest),
    PullSnapshot(gwz_core::PullSnapshotRequest),
    Push(gwz_core::PushRequest),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputMode {
    Human,
    Json,
    Jsonl,
    Porcelain,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CliError {
    message: String,
}

impl CliError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[cfg(test)]
fn parse_args_with_request_id(
    args: Vec<String>,
    request_id: &str,
    current_dir: &std::path::Path,
) -> Result<CliInvocation, CliError> {
    let cli = Cli::try_parse_from(std::iter::once("gwz".to_owned()).chain(args))
        .map_err(|error| CliError::new(error.to_string()))?;
    invocation_from_cli(cli, request_id, current_dir)
}

fn invocation_from_cli(
    cli: Cli,
    request_id: &str,
    current_dir: &std::path::Path,
) -> Result<CliInvocation, CliError> {
    cli.validate()?;
    let output = cli.output_mode();
    let meta = cli.request_meta(request_id);
    let workspace_root = cli
        .global
        .root
        .clone()
        .unwrap_or_else(|| current_dir.to_string_lossy().into_owned());
    let request = cli.command_request(meta, workspace_root)?;
    Ok(CliInvocation {
        request,
        output,
        start_dir: current_dir.to_path_buf(),
    })
}

fn execute_invocation(invocation: &CliInvocation) -> Result<gwz_core::ResponseEnvelope, CliError> {
    let backend = gwz_core::git::Git2Backend::new();
    let operation_id = new_operation_id();
    let start = invocation.start_dir.as_path();
    let response = match &invocation.request {
        CliRequest::CreateWorkspace(request) => {
            gwz_core::workspace_ops::handle_create_workspace(request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::InitFromSources(request) => gwz_core::workspace_ops::handle_init_from_sources(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::AddExistingRepo(request) => gwz_core::workspace_ops::handle_add_existing_repo(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::CreateRepo(request) => gwz_core::workspace_ops::handle_create_repo(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::Materialize(request) => gwz_core::workspace_ops::handle_materialize(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::Status(request) => {
            gwz_core::status::handle_status(&backend, start, request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::Snapshot(request) => {
            gwz_core::workspace_ops::handle_snapshot(start, request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::Tag(request) => {
            gwz_core::workspace_ops::handle_tag(start, request.clone(), operation_id)
                .map(|response| response.response)
        }
        CliRequest::PullHead(request) => gwz_core::workspace_ops::handle_pull_head(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::PullSnapshot(request) => gwz_core::workspace_ops::handle_pull_snapshot(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| response.response),
        CliRequest::Push(request) => {
            gwz_core::workspace_ops::handle_push(&backend, start, request.clone(), operation_id)
                .map(|response| response.response)
        }
    };
    response.map_err(|error| CliError::new(error.to_string()))
}

fn render_response(response: &gwz_core::ResponseEnvelope, output: OutputMode) -> String {
    match output {
        OutputMode::Human => render_human_response(response),
        OutputMode::Json => response_json(response).to_string(),
        OutputMode::Jsonl => render_jsonl_stream(response, &[], None),
        OutputMode::Porcelain => render_porcelain_response(response),
    }
}

fn render_human_response(response: &gwz_core::ResponseEnvelope) -> String {
    let mut lines = vec![format!("status: {:?}", response.meta.aggregate_status)];
    for member in &response.members {
        let mut line = format!(
            "{} {} {:?}",
            member.member_id, member.member_path, member.status
        );
        if let Some(error) = &member.error {
            line.push_str(&format!(" {:?}: {}", error.code, error.message));
        }
        lines.push(line);
    }
    for error in &response.errors {
        lines.push(format!("{:?}: {}", error.code, error.message));
    }
    lines.join("\n")
}

fn render_porcelain_response(response: &gwz_core::ResponseEnvelope) -> String {
    response
        .members
        .iter()
        .filter(|member| member.status != gwz_core::MemberStatus::Ok)
        .map(|member| format!("!! {}", member.member_path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_jsonl_stream(
    response: &gwz_core::ResponseEnvelope,
    events: &[gwz_core::OperationEvent],
    result: Option<&gwz_core::OperationResult>,
) -> String {
    let mut lines = Vec::with_capacity(1 + events.len() + usize::from(result.is_some()));
    lines.push(response_json(response).to_string());
    lines.extend(events.iter().map(|event| event_json(event).to_string()));
    if let Some(result) = result {
        lines.push(result_json(result).to_string());
    }
    lines.join("\n")
}

fn exit_code_for_response(response: &gwz_core::ResponseEnvelope) -> i32 {
    match response.meta.aggregate_status {
        gwz_core::AggregateStatus::Accepted
        | gwz_core::AggregateStatus::Ok
        | gwz_core::AggregateStatus::Noop => 0,
        gwz_core::AggregateStatus::Rejected => 2,
        gwz_core::AggregateStatus::Partial | gwz_core::AggregateStatus::Failed => 1,
    }
}

fn response_json(response: &gwz_core::ResponseEnvelope) -> serde_json::Value {
    serde_json::json!({
        "kind": "response",
        "meta": response_meta_json(&response.meta),
        "members": response.members.iter().map(member_json).collect::<Vec<_>>(),
        "errors": response.errors.iter().map(error_json).collect::<Vec<_>>(),
    })
}

fn result_json(result: &gwz_core::OperationResult) -> serde_json::Value {
    serde_json::json!({
        "kind": "result",
        "operation_id": result.operation_id,
        "request_id": result.request_id,
        "action": format!("{:?}", result.action),
        "aggregate_status": format!("{:?}", result.aggregate_status),
        "started_at_ms": result.started_at_ms,
        "finished_at_ms": result.finished_at_ms,
        "members": result.members.iter().map(member_json).collect::<Vec<_>>(),
        "errors": result.errors.iter().map(error_json).collect::<Vec<_>>(),
    })
}

fn event_json(event: &gwz_core::OperationEvent) -> serde_json::Value {
    serde_json::json!({
        "kind": "event",
        "operation_id": event.operation_id,
        "request_id": event.request_id,
        "sequence": event.sequence,
        "timestamp_ms": event.timestamp_ms,
        "event_kind": format!("{:?}", event.kind),
        "severity": format!("{:?}", event.severity),
        "member_id": event.member_id,
        "member_path": event.member_path,
        "message": event.message,
        "member": event.member.as_ref().map(member_json),
        "error": event.error.as_ref().map(error_json),
    })
}

fn response_meta_json(meta: &gwz_core::ResponseMeta) -> serde_json::Value {
    serde_json::json!({
        "request_id": meta.request_id,
        "schema_version": meta.schema_version,
        "action": format!("{:?}", meta.action),
        "aggregate_status": format!("{:?}", meta.aggregate_status),
        "operation_id": meta.operation_id,
        "message": meta.message,
    })
}

fn member_json(member: &gwz_core::MemberResponse) -> serde_json::Value {
    serde_json::json!({
        "member_id": member.member_id,
        "member_path": member.member_path,
        "source_kind": format!("{:?}", member.source_kind),
        "status": format!("{:?}", member.status),
        "error": member.error.as_ref().map(error_json),
        "planned": member.planned.as_ref().map(planned_json),
        "lock_match": member.lock_match.map(|lock_match| format!("{:?}", lock_match)),
    })
}

fn planned_json(planned: &gwz_core::PlannedChange) -> serde_json::Value {
    serde_json::json!({
        "action": format!("{:?}", planned.action),
        "from_ref": planned.from_ref,
        "to_ref": planned.to_ref,
        "message": planned.message,
    })
}

fn error_json(error: &gwz_core::GwzError) -> serde_json::Value {
    serde_json::json!({
        "code": format!("{:?}", error.code),
        "message": error.message,
        "member_id": error.member_id,
        "member_path": error.member_path,
        "detail": error.detail,
    })
}

impl Cli {
    fn validate(&self) -> Result<(), CliError> {
        if self.global.json && self.global.jsonl {
            return Err(CliError::new("--json and --jsonl are mutually exclusive"));
        }
        if self.global.all && (!self.global.members.is_empty() || !self.global.paths.is_empty()) {
            return Err(CliError::new(
                "--all cannot be combined with --member or --path",
            ));
        }
        if let CommandArgs::Status(status) = &self.command {
            status.validate(&self.global)?;
        }
        Ok(())
    }

    fn output_mode(&self) -> OutputMode {
        if matches!(&self.command, CommandArgs::Status(status) if status.porcelain) {
            OutputMode::Porcelain
        } else if self.global.json {
            OutputMode::Json
        } else if self.global.jsonl {
            OutputMode::Jsonl
        } else {
            OutputMode::Human
        }
    }

    fn request_meta(&self, request_id: &str) -> gwz_core::RequestMeta {
        gwz_core::RequestMeta {
            request_id: request_id.to_owned(),
            schema_version: "gwz.protocol/v0".to_owned(),
            workspace: self
                .global
                .root
                .as_ref()
                .map(|root| gwz_core::WorkspaceRef {
                    root: Some(root.clone()),
                    workspace_id: None,
                }),
            selection: self.selection(),
            policy: self.policy(),
            dry_run: self.global.dry_run.then_some(true),
            ..Default::default()
        }
    }

    fn selection(&self) -> Option<gwz_core::Selection> {
        if self.global.all || !self.global.members.is_empty() || !self.global.paths.is_empty() {
            Some(gwz_core::Selection {
                all: self.global.all.then_some(true),
                member_ids: self.global.members.clone(),
                paths: self.global.paths.clone(),
            })
        } else {
            None
        }
    }

    fn policy(&self) -> Option<gwz_core::OperationPolicy> {
        if self.global.partial
            || self.global.force
            || self.global.sync.is_some()
            || self.global.remote.is_some()
            || self.global.jobs.is_some()
        {
            Some(gwz_core::OperationPolicy {
                partial: self
                    .global
                    .partial
                    .then_some(gwz_core::PartialBehavior::Partial),
                destructive: self
                    .global
                    .force
                    .then_some(gwz_core::DestructiveBehavior::Allow),
                sync: self.global.sync.map(Into::into),
                remote: self.global.remote.clone(),
                concurrency: self.global.jobs,
                ..Default::default()
            })
        } else {
            None
        }
    }

    fn command_request(
        &self,
        meta: gwz_core::RequestMeta,
        workspace_root: String,
    ) -> Result<CliRequest, CliError> {
        match &self.command {
            CommandArgs::Init(args) => args.request(meta, workspace_root),
            CommandArgs::Add(args) => args.request(meta),
            CommandArgs::Repo(args) => args.request(meta),
            CommandArgs::Status(args) => args.request(meta),
            CommandArgs::Snapshot(args) => Ok(CliRequest::Snapshot(gwz_core::SnapshotRequest {
                meta,
                snapshot_id: args.name.clone(),
            })),
            CommandArgs::Tag(args) => Ok(CliRequest::Tag(gwz_core::TagRequest {
                meta,
                tag_name: args.name.clone(),
            })),
            CommandArgs::Materialize(args) => args.request(meta),
            CommandArgs::Pull(args) => args.request(meta),
            CommandArgs::Push => Ok(CliRequest::Push(gwz_core::PushRequest {
                remote: self.global.remote.clone(),
                refspec: None,
                meta,
            })),
        }
    }
}

impl InitArgs {
    fn request(
        &self,
        meta: gwz_core::RequestMeta,
        workspace_root: String,
    ) -> Result<CliRequest, CliError> {
        if self.urls.is_empty() {
            Ok(CliRequest::CreateWorkspace(
                gwz_core::CreateWorkspaceRequest {
                    meta,
                    workspace_root,
                    workspace_id: None,
                },
            ))
        } else {
            Ok(CliRequest::InitFromSources(
                gwz_core::InitFromSourcesRequest {
                    meta,
                    workspace_root,
                    sources: self
                        .urls
                        .iter()
                        .cloned()
                        .map(|url| gwz_core::SourceUrl {
                            url,
                            path: None,
                            remote_name: None,
                            branch: None,
                        })
                        .collect(),
                    target: Some(gwz_core::MaterializeTarget {
                        kind: gwz_core::MaterializeTargetKind::Head,
                        name: None,
                        commit: None,
                    }),
                    workspace_id: None,
                },
            ))
        }
    }
}

impl AddArgs {
    fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        Ok(CliRequest::AddExistingRepo(
            gwz_core::AddExistingRepoRequest {
                meta,
                repository_path: self.repo_path.clone(),
                member_path: None,
                member_id: None,
                source_id: None,
            },
        ))
    }
}

impl RepoArgs {
    fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        match &self.command {
            RepoCommandArgs::Create(args) => {
                Ok(CliRequest::CreateRepo(gwz_core::CreateRepoRequest {
                    meta,
                    member_path: args.member_path.clone(),
                    initial_branch: None,
                    member_id: None,
                    source_id: None,
                }))
            }
        }
    }
}

impl StatusArgs {
    fn validate(&self, global: &GlobalArgs) -> Result<(), CliError> {
        if self.porcelain && (global.json || global.jsonl) {
            return Err(CliError::new(
                "--porcelain cannot be combined with --json or --jsonl",
            ));
        }
        if self.no_files && self.no_branches {
            return Err(CliError::new(
                "--no-files and --no-branches cannot both be supplied",
            ));
        }
        if self.combined && self.no_combined {
            return Err(CliError::new(
                "--combined and --no-combined cannot both be supplied",
            ));
        }
        if self.porcelain && self.no_combined {
            return Err(CliError::new(
                "--porcelain cannot be combined with --no-combined",
            ));
        }
        if self.no_combined && (self.no_files || self.no_branches) {
            return Err(CliError::new(
                "--no-files and --no-branches can only be used with combined status",
            ));
        }
        Ok(())
    }

    fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        let combined = !self.no_combined;
        Ok(CliRequest::Status(gwz_core::StatusRequest {
            meta,
            mode: Some(if combined {
                gwz_core::StatusMode::Combined
            } else {
                gwz_core::StatusMode::Summary
            }),
            include_file_changes: if combined { Some(!self.no_files) } else { None },
            include_branch_summary: if combined {
                Some(!self.no_branches)
            } else {
                None
            },
            path_style: combined.then_some(gwz_core::StatusPathStyle::WorkspaceRelative),
        }))
    }
}

impl MaterializeArgs {
    fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        Ok(CliRequest::Materialize(gwz_core::MaterializeRequest {
            meta,
            target: self.target()?,
        }))
    }

    fn target(&self) -> Result<gwz_core::MaterializeTarget, CliError> {
        let targets = usize::from(self.lock)
            + usize::from(self.head)
            + usize::from(self.snapshot.is_some())
            + usize::from(self.tag.is_some());
        if targets > 1 {
            return Err(CliError::new("only one target flag may be supplied"));
        }
        if self.head {
            Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Head,
                name: None,
                commit: None,
            })
        } else if let Some(name) = &self.snapshot {
            Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Snapshot,
                name: Some(name.clone()),
                commit: None,
            })
        } else if let Some(name) = &self.tag {
            Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Tag,
                name: Some(name.clone()),
                commit: None,
            })
        } else {
            Ok(gwz_core::MaterializeTarget {
                kind: gwz_core::MaterializeTargetKind::Lock,
                name: None,
                commit: None,
            })
        }
    }
}

impl PullArgs {
    fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        match (self.head, self.snapshot.as_ref()) {
            (true, Some(_)) => Err(CliError::new("only one target flag may be supplied")),
            (_, Some(name)) => Ok(CliRequest::PullSnapshot(gwz_core::PullSnapshotRequest {
                meta,
                snapshot_id: name.clone(),
            })),
            _ => Ok(CliRequest::PullHead(gwz_core::PullHeadRequest { meta })),
        }
    }
}

impl From<SyncArg> for gwz_core::SyncBehavior {
    fn from(value: SyncArg) -> Self {
        match value {
            SyncArg::FetchOnly => gwz_core::SyncBehavior::FetchOnly,
            SyncArg::FfOnly => gwz_core::SyncBehavior::FfOnly,
            SyncArg::Merge => gwz_core::SyncBehavior::Merge,
            SyncArg::Rebase => gwz_core::SyncBehavior::Rebase,
            SyncArg::Reset => gwz_core::SyncBehavior::Reset,
            SyncArg::DriverSelected => gwz_core::SyncBehavior::DriverSelected,
        }
    }
}

fn parse_positive_i64(value: &str) -> Result<i64, String> {
    let parsed = value
        .parse::<i64>()
        .map_err(|_| "--jobs requires an integer".to_owned())?;
    if parsed < 1 {
        return Err("--jobs must be greater than zero".to_owned());
    }
    Ok(parsed)
}

fn new_request_id() -> String {
    format!("req_{}", unique_suffix())
}

fn new_operation_id() -> String {
    format!("op_{}", unique_suffix())
}

fn unique_suffix() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    format!("{}_{}", std::process::id(), millis)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn usage_text_covers_standard_help_and_commands() {
        let usage = usage_text();

        assert!(usage.contains("Usage: gwz"));
        assert!(usage.contains("-h, --help"));
        assert!(usage.contains("init"));
        assert!(usage.contains("status"));
    }

    #[test]
    fn parses_init_workspace_with_root() {
        let invocation = parse_args_with_request_id(
            strings(["--root", "/tmp/gwz-test", "init"]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        assert_eq!(invocation.output, OutputMode::Human);
        let CliRequest::CreateWorkspace(request) = invocation.request else {
            panic!("expected create workspace");
        };
        assert_eq!(request.workspace_root, "/tmp/gwz-test");
        assert_eq!(request.meta.request_id, "req_test");
    }

    #[test]
    fn parses_init_sources_from_plain_urls() {
        let invocation = parse_args_with_request_id(
            strings([
                "init",
                "git@github.com:org/repo-a.git",
                "https://github.com/org/repo-b",
            ]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        let CliRequest::InitFromSources(request) = invocation.request else {
            panic!("expected init from sources");
        };
        assert_eq!(request.workspace_root, "/cwd");
        assert_eq!(request.sources[0].url, "git@github.com:org/repo-a.git");
        assert_eq!(request.sources[0].path, None);
        assert_eq!(request.sources[1].url, "https://github.com/org/repo-b");
    }

    #[test]
    fn parses_global_selection_policy_and_output_flags() {
        let invocation = parse_args_with_request_id(
            strings([
                "--root",
                "/ws",
                "--member",
                "mem_app",
                "--path",
                "repos/lib",
                "--dry-run",
                "--partial",
                "--force",
                "--sync",
                "reset",
                "--remote",
                "origin",
                "--jobs",
                "4",
                "--json",
                "status",
            ]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        assert_eq!(invocation.output, OutputMode::Json);
        let CliRequest::Status(request) = invocation.request else {
            panic!("expected status");
        };
        let workspace = request.meta.workspace.unwrap();
        assert_eq!(workspace.root, Some("/ws".to_owned()));
        let selection = request.meta.selection.unwrap();
        assert_eq!(selection.member_ids, vec!["mem_app"]);
        assert_eq!(selection.paths, vec!["repos/lib"]);
        let policy = request.meta.policy.unwrap();
        assert_eq!(policy.partial, Some(gwz_core::PartialBehavior::Partial));
        assert_eq!(
            policy.destructive,
            Some(gwz_core::DestructiveBehavior::Allow)
        );
        assert_eq!(policy.sync, Some(gwz_core::SyncBehavior::Reset));
        assert_eq!(policy.remote, Some("origin".to_owned()));
        assert_eq!(policy.concurrency, Some(4));
        assert_eq!(request.meta.dry_run, Some(true));
    }

    #[test]
    fn parses_combined_status_flags() {
        let invocation = parse_args_with_request_id(
            strings(["status", "--porcelain", "--no-branches"]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        assert_eq!(invocation.output, OutputMode::Porcelain);
        let CliRequest::Status(request) = invocation.request else {
            panic!("expected status");
        };
        assert_eq!(request.mode, Some(gwz_core::StatusMode::Combined));
        assert_eq!(request.include_file_changes, Some(true));
        assert_eq!(request.include_branch_summary, Some(false));
        assert_eq!(
            request.path_style,
            Some(gwz_core::StatusPathStyle::WorkspaceRelative)
        );
    }

    #[test]
    fn parses_status_as_combined_by_default() {
        let invocation =
            parse_args_with_request_id(strings(["status"]), "req_test", Path::new("/cwd")).unwrap();

        let CliRequest::Status(request) = invocation.request else {
            panic!("expected status");
        };
        assert_eq!(request.mode, Some(gwz_core::StatusMode::Combined));
        assert_eq!(request.include_file_changes, Some(true));
        assert_eq!(request.include_branch_summary, Some(true));
        assert_eq!(
            request.path_style,
            Some(gwz_core::StatusPathStyle::WorkspaceRelative)
        );
    }

    #[test]
    fn parses_no_combined_status_as_summary_mode() {
        let invocation = parse_args_with_request_id(
            strings(["status", "--no-combined"]),
            "req_test",
            Path::new("/cwd"),
        )
        .unwrap();

        let CliRequest::Status(request) = invocation.request else {
            panic!("expected status");
        };
        assert_eq!(request.mode, Some(gwz_core::StatusMode::Summary));
        assert_eq!(request.include_file_changes, None);
        assert_eq!(request.include_branch_summary, None);
        assert_eq!(request.path_style, None);
    }

    #[test]
    fn parses_command_matrix() {
        assert!(matches!(
            parse(strings(["add", "repos/app"])).request,
            CliRequest::AddExistingRepo(_)
        ));
        assert!(matches!(
            parse(strings(["repo", "create", "repos/app"])).request,
            CliRequest::CreateRepo(_)
        ));
        assert!(matches!(
            parse(strings(["materialize", "--lock"])).request,
            CliRequest::Materialize(_)
        ));
        assert!(matches!(
            parse(strings(["materialize", "--snapshot", "snap_one"])).request,
            CliRequest::Materialize(_)
        ));
        assert!(matches!(
            parse(strings(["pull", "--head"])).request,
            CliRequest::PullHead(_)
        ));
        assert!(matches!(
            parse(strings(["pull", "--snapshot", "snap_one"])).request,
            CliRequest::PullSnapshot(_)
        ));
        assert!(matches!(
            parse(strings(["snapshot", "snap_one"])).request,
            CliRequest::Snapshot(_)
        ));
        assert!(matches!(
            parse(strings(["tag", "release_one"])).request,
            CliRequest::Tag(_)
        ));
        assert!(matches!(
            parse(strings(["push"])).request,
            CliRequest::Push(_)
        ));
    }

    #[test]
    fn rejects_invalid_command_combinations_before_core_execution() {
        assert!(parse_result(strings(["--json", "--jsonl", "status"])).is_err());
        assert!(parse_result(strings(["--all", "--member", "mem_app", "status"])).is_err());
        assert!(parse_result(strings(["status", "--no-files", "--no-branches"])).is_err());
        assert!(parse_result(strings(["status", "--combined", "--no-combined"])).is_err());
        assert!(parse_result(strings(["status", "--porcelain", "--no-combined"])).is_err());
        assert!(parse_result(strings(["status", "--no-combined", "--no-files"])).is_err());
        assert!(parse_result(strings(["push", "--combined"])).is_err());
        assert!(parse_result(strings(["push", "--no-combined"])).is_err());
        assert!(parse_result(strings(["materialize", "--snapshot"])).is_err());
        assert!(parse_result(strings(["pull", "--lock"])).is_err());
        assert!(parse_result(strings(["unknown"])).is_err());
    }

    #[test]
    fn can_call_core_status_in_process() {
        let temp = TempDir::new("cli-status");
        gwz_core::workspace_ops::handle_create_workspace(
            gwz_core::CreateWorkspaceRequest {
                meta: request_meta("req_setup"),
                workspace_root: temp.path().to_string_lossy().into_owned(),
                workspace_id: Some("ws_cli".to_owned()),
            },
            "op_setup",
        )
        .unwrap();
        let invocation = parse_args_with_request_id(
            strings([
                "--root",
                temp.path().to_str().unwrap(),
                "status",
                "--no-combined",
            ]),
            "req_status",
            temp.path(),
        )
        .unwrap();

        let response = execute_invocation(&invocation).unwrap();

        assert_eq!(
            response.meta.aggregate_status,
            gwz_core::AggregateStatus::Ok
        );
        assert!(response.members.is_empty());
    }

    #[test]
    fn json_renderer_outputs_structured_response() {
        let response = sample_response(gwz_core::AggregateStatus::Ok, gwz_core::MemberStatus::Ok);

        let json: serde_json::Value =
            serde_json::from_str(&render_response(&response, OutputMode::Json)).unwrap();

        assert_eq!(json["kind"], "response");
        assert_eq!(json["meta"]["aggregate_status"], "Ok");
        assert_eq!(json["members"][0]["member_id"], "mem_app");
        assert_eq!(json["members"][0]["status"], "Ok");
    }

    #[test]
    fn jsonl_renderer_emits_response_event_and_result_in_order() {
        let response = sample_response(
            gwz_core::AggregateStatus::Accepted,
            gwz_core::MemberStatus::Planned,
        );
        let event = sample_event();
        let result = sample_result();

        let lines = render_jsonl_stream(&response, &[event], Some(&result))
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0]["kind"], "response");
        assert_eq!(lines[1]["kind"], "event");
        assert_eq!(lines[2]["kind"], "result");
    }

    #[test]
    fn human_renderer_smoke_covers_success_rejection_and_member_failure() {
        let success = render_response(
            &sample_response(gwz_core::AggregateStatus::Ok, gwz_core::MemberStatus::Ok),
            OutputMode::Human,
        );
        assert!(success.contains("status: Ok"));

        let rejected = render_response(
            &sample_response(
                gwz_core::AggregateStatus::Rejected,
                gwz_core::MemberStatus::Rejected,
            ),
            OutputMode::Human,
        );
        assert!(rejected.contains("status: Rejected"));

        let failed = render_response(
            &sample_response(
                gwz_core::AggregateStatus::Failed,
                gwz_core::MemberStatus::Failed,
            ),
            OutputMode::Human,
        );
        assert!(failed.contains("RemoteRejected"));
    }

    #[test]
    fn exit_code_mapping_distinguishes_success_rejected_and_failed() {
        assert_eq!(
            exit_code_for_response(&sample_response(
                gwz_core::AggregateStatus::Ok,
                gwz_core::MemberStatus::Ok,
            )),
            0
        );
        assert_eq!(
            exit_code_for_response(&sample_response(
                gwz_core::AggregateStatus::Rejected,
                gwz_core::MemberStatus::Rejected,
            )),
            2
        );
        assert_eq!(
            exit_code_for_response(&sample_response(
                gwz_core::AggregateStatus::Failed,
                gwz_core::MemberStatus::Failed,
            )),
            1
        );
    }

    fn parse(args: Vec<String>) -> CliInvocation {
        parse_result(args).unwrap()
    }

    fn parse_result(args: Vec<String>) -> Result<CliInvocation, CliError> {
        parse_args_with_request_id(args, "req_test", Path::new("/cwd"))
    }

    fn strings<const N: usize>(items: [&str; N]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn request_meta(request_id: &str) -> gwz_core::RequestMeta {
        gwz_core::RequestMeta {
            request_id: request_id.to_owned(),
            schema_version: "gwz.protocol/v0".to_owned(),
            ..Default::default()
        }
    }

    fn sample_response(
        aggregate_status: gwz_core::AggregateStatus,
        member_status: gwz_core::MemberStatus,
    ) -> gwz_core::ResponseEnvelope {
        let error = (member_status == gwz_core::MemberStatus::Failed
            || member_status == gwz_core::MemberStatus::Rejected)
            .then(|| gwz_core::GwzError {
                code: gwz_core::GwzErrorCode::RemoteRejected,
                message: "remote rejected".to_owned(),
                member_id: Some("mem_app".to_owned()),
                member_path: Some("repos/app".to_owned()),
                detail: None,
            });
        gwz_core::ResponseEnvelope {
            meta: gwz_core::ResponseMeta {
                request_id: "req_render".to_owned(),
                schema_version: "gwz.protocol/v0".to_owned(),
                action: gwz_core::ActionKind::Status,
                aggregate_status,
                operation_id: Some("op_render".to_owned()),
                message: None,
                attribution: None,
            },
            members: vec![gwz_core::MemberResponse {
                member_id: "mem_app".to_owned(),
                member_path: "repos/app".to_owned(),
                source_kind: gwz_core::SourceKind::Git,
                status: member_status,
                error,
                planned: None,
                state: None,
                git_status: None,
                lock_match: None,
            }],
            errors: Vec::new(),
        }
    }

    fn sample_event() -> gwz_core::OperationEvent {
        gwz_core::OperationEvent {
            operation_id: "op_render".to_owned(),
            request_id: "req_render".to_owned(),
            sequence: 0,
            timestamp_ms: 1,
            kind: gwz_core::EventKind::OperationStarted,
            severity: gwz_core::Severity::Info,
            member_id: None,
            member_path: None,
            message: Some("started".to_owned()),
            member: None,
            error: None,
            attribution: None,
        }
    }

    fn sample_result() -> gwz_core::OperationResult {
        gwz_core::OperationResult {
            operation_id: "op_render".to_owned(),
            request_id: "req_render".to_owned(),
            action: gwz_core::ActionKind::Status,
            aggregate_status: gwz_core::AggregateStatus::Ok,
            started_at_ms: 1,
            finished_at_ms: 2,
            members: Vec::new(),
            errors: Vec::new(),
            attribution: None,
        }
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir()
                .join(format!("gwz-cli-{prefix}-{}-{unique}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
