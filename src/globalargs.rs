#[cfg(test)]
use clap::CommandFactory;
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::*;

#[cfg(test)]
pub(crate) fn usage_text() -> String {
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
pub(crate) struct Cli {
    #[command(flatten, next_help_heading = "Global Options")]
    pub(crate) global: GlobalArgs,

    #[command(subcommand)]
    pub(crate) command: CommandArgs,
}

#[derive(Clone, Debug, Default, Args)]
pub(crate) struct GlobalArgs {
    #[arg(
        long,
        global = true,
        value_name = "path",
        help = "Workspace root",
        long_help = "Workspace root. Defaults to the current directory when not supplied."
    )]
    pub(crate) root: Option<String>,

    #[arg(
        long = "member",
        global = true,
        value_name = "member-id",
        help = "Select a workspace member by id",
        long_help = "Select a workspace member by id. May be supplied more than once."
    )]
    pub(crate) members: Vec<String>,

    #[arg(
        long = "member-path",
        global = true,
        value_name = "member-path",
        help = "Select a workspace member by path",
        long_help = "Select a workspace member by path. May be supplied more than once."
    )]
    pub(crate) paths: Vec<String>,

    #[arg(
        long,
        global = true,
        help = "Select all workspace members",
        long_help = "Select all workspace members. Cannot be combined with `--member` or `--member-path`."
    )]
    pub(crate) all: bool,

    #[arg(
        long,
        global = true,
        help = "Plan the operation without mutating state",
        long_help = "Plan the operation without mutating workspace metadata or member repositories."
    )]
    pub(crate) dry_run: bool,

    #[arg(
        long,
        global = true,
        help = "Allow operations to complete partially",
        long_help = "Allow operations to complete for members that can proceed even when another selected member fails."
    )]
    pub(crate) partial: bool,

    #[arg(
        long,
        global = true,
        help = "Allow destructive behavior when required",
        long_help = "Allow destructive behavior when required. GWZ refuses destructive changes unless this is explicit."
    )]
    pub(crate) force: bool,

    #[arg(
        long,
        global = true,
        value_enum,
        value_name = "mode",
        help = "Select workspace sync behavior",
        long_help = "Select workspace sync behavior. The default policy is fast-forward only."
    )]
    pub(crate) sync: Option<SyncArg>,

    #[arg(
        long,
        global = true,
        value_name = "name",
        help = "Select the git remote name",
        long_help = "Select the git remote name used by operations that contact remotes."
    )]
    pub(crate) remote: Option<String>,

    #[arg(
        long,
        global = true,
        value_name = "n",
        value_parser = parse_positive_i64,
        help = "Global ceiling on concurrent member operations (default 50)",
        long_help = "Global ceiling on the total number of member repositories processed concurrently across all hosts. Defaults to 50. Per-host concurrency is bounded separately by --max-per-host."
    )]
    pub(crate) jobs: Option<i64>,

    #[arg(
        long = "max-per-host",
        global = true,
        value_name = "n",
        value_parser = parse_positive_i64,
        help = "Max concurrent connections to any one host (default 8)",
        long_help = "Maximum concurrent network operations against a single remote host, so a host is not overloaded. Members whose host cannot be parsed (e.g. local paths) are bounded only by --jobs. Defaults to 8."
    )]
    pub(crate) max_per_host: Option<i64>,

    #[arg(
        long = "progress-interval",
        global = true,
        value_name = "ms",
        value_parser = parse_non_negative_i64,
        help = "Min milliseconds between progress events per repo (0 = every update)",
        long_help = "Minimum milliseconds between member progress events per repository. Coalesces high-frequency Git transfer updates; 0 emits every update. Defaults to 100."
    )]
    pub(crate) progress_interval: Option<i64>,

    #[arg(
        long,
        global = true,
        help = "Render one JSON response",
        long_help = "Render one structured JSON response for the operation."
    )]
    pub(crate) json: bool,

    #[arg(
        long,
        global = true,
        help = "Render newline-delimited JSON events",
        long_help = "Render newline-delimited JSON records for streaming operation consumers."
    )]
    pub(crate) jsonl: bool,

    #[arg(
        long = "ssh-timeout",
        global = true,
        value_name = "secs",
        value_parser = parse_non_negative_i64,
        help = "Abort a stalled SSH/network read after N seconds (0 = no timeout, default 3)",
        long_help = "Maximum seconds to wait on a stalled SSH/network read before failing. libssh2 has no timeout by default, so a missing ssh-agent identity or an unreachable host would otherwise hang forever. 0 disables the timeout. Defaults to 3."
    )]
    pub(crate) ssh_timeout: Option<i64>,
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum CommandArgs {
    #[command(
        about = "Create a workspace or initialize one from source URLs",
        long_about = INIT_LONG,
        after_long_help = INIT_AFTER
    )]
    Init(InitArgs),
    #[command(
        about = "Clone a workspace and materialize its members",
        long_about = CLONE_LONG,
        after_long_help = CLONE_AFTER
    )]
    Clone(CloneArgs),
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
    #[command(about = "Record the live worktree state into the lock (no mutation)")]
    Capture,
    #[command(about = "Commit staged changes across members and the workspace root")]
    Commit(CommitArgs),
}

#[derive(Clone, Debug, Args)]
pub(crate) struct InitArgs {
    #[arg(
        long = "path",
        default_value = "",
        value_name = "path-prefix",
        help = "Workspace-relative prefix for initialized source repositories",
        long_help = "Workspace-relative prefix for initialized source repositories. Defaults to an empty prefix, so repositories are created directly under the workspace root."
    )]
    pub(crate) path_prefix: String,

    #[arg(
        value_name = "url",
        help = "Git source URL to add as an initial workspace member",
        long_help = "Git source URL to add as an initial workspace member. May be supplied more than once."
    )]
    pub(crate) urls: Vec<String>,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct RepoArgs {
    #[command(subcommand)]
    pub(crate) command: RepoCommandArgs,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub(crate) enum SyncArg {
    FetchOnly,
    FfOnly,
    Merge,
    Rebase,
    Reset,
    DriverSelected,
}

#[cfg(test)]
pub(crate) fn parse_args_with_request_id(
    args: Vec<String>,
    request_id: &str,
    current_dir: &std::path::Path,
) -> Result<CliInvocation, CliError> {
    let cli = Cli::try_parse_from(std::iter::once("gwz".to_owned()).chain(args))
        .map_err(|error| CliError::new(error.to_string()))?;
    invocation_from_cli(cli, request_id, current_dir)
}

pub(crate) fn invocation_from_cli(
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

pub(crate) fn execute_invocation(invocation: &CliInvocation) -> Result<CliResponse, CliError> {
    let backend = gwz_core::git::Git2Backend::new();
    let operation_id = new_operation_id();
    let start = invocation.start_dir.as_path();
    // --jsonl streams machine records to stdout; Human renders a live progress
    // line to stderr (TTY-gated); Json/Porcelain stay quiet.
    let jsonl_sink = JsonlSink;
    let null_sink = gwz_core::operation::NullSink;
    let progress_sink = StderrProgressSink::new(operation_label(&invocation.request));
    let events: &dyn gwz_core::operation::EventSink = match invocation.output {
        OutputMode::Jsonl => &jsonl_sink,
        OutputMode::Human => &progress_sink,
        OutputMode::Json | OutputMode::Porcelain => &null_sink,
    };
    let response = match &invocation.request {
        CliRequest::CloneWorkspace { meta, url, target } => {
            gwz_core::workspace_ops::handle_clone_workspace(
                &backend,
                meta.clone(),
                url,
                target,
                operation_id,
                events,
            )
            .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::CreateWorkspace(request) => {
            gwz_core::workspace_ops::handle_create_workspace(request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::InitFromSources(request) => gwz_core::workspace_ops::handle_init_from_sources(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::AddExistingRepo(request) => gwz_core::workspace_ops::handle_add_existing_repo(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::CreateRepo(request) => gwz_core::workspace_ops::handle_create_repo(
            &backend,
            start,
            request.clone(),
            operation_id,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::Materialize(request) => gwz_core::workspace_ops::handle_materialize(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::Status(request) => {
            gwz_core::status::handle_status(&backend, start, request.clone(), operation_id).map(
                |response| CliResponse {
                    envelope: response.response,
                    workspace_git_status: response.workspace_git_status,
                    status_mode: request.mode,
                },
            )
        }
        CliRequest::Snapshot(request) => {
            gwz_core::workspace_ops::handle_snapshot(&backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Tag(request) => {
            gwz_core::workspace_ops::handle_tag(&backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::PullHead(request) => gwz_core::workspace_ops::handle_pull_head_with_events(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::PullSnapshot(request) => gwz_core::workspace_ops::handle_pull_snapshot(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::Push(request) => gwz_core::workspace_ops::handle_push_with_events(
            &backend,
            start,
            request.clone(),
            operation_id,
            events,
        )
        .map(|response| CliResponse::envelope(response.response)),
        CliRequest::Capture(request) => {
            gwz_core::workspace_ops::handle_capture(&backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
        CliRequest::Commit(request) => {
            gwz_core::workspace_ops::handle_commit(&backend, start, request.clone(), operation_id)
                .map(|response| CliResponse::envelope(response.response))
        }
    };
    response.map_err(CliError::from_model)
}

pub(crate) fn render_response(response: &CliResponse, output: OutputMode) -> String {
    match output {
        OutputMode::Human => render_human_response(response),
        OutputMode::Json => response_json(response).to_string(),
        OutputMode::Jsonl => render_jsonl_stream(response, &[], None),
        OutputMode::Porcelain => render_porcelain_response(response),
    }
}

pub(crate) fn exit_code_for_response(response: &gwz_core::ResponseEnvelope) -> i32 {
    match response.meta.aggregate_status {
        gwz_core::AggregateStatus::Accepted
        | gwz_core::AggregateStatus::Ok
        | gwz_core::AggregateStatus::Noop
        // F5/AD3: a dirty workspace is the normal resting state (like `git status`) — exit 0.
        | gwz_core::AggregateStatus::Dirty => 0,
        gwz_core::AggregateStatus::Rejected => 2,
        // A conflict needs developer action (resolve + continue) — exit non-zero, like `git rebase`.
        gwz_core::AggregateStatus::Partial
        | gwz_core::AggregateStatus::Failed
        | gwz_core::AggregateStatus::Conflicted => 1,
    }
}

impl StatusArgs {
    pub(crate) fn validate(&self, global: &GlobalArgs) -> Result<(), CliError> {
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

    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        let combined = !self.no_combined;
        Ok(CliRequest::Status(gwz_core::StatusRequest {
            meta,
            mode: Some(if combined {
                gwz_core::StatusMode::Combined
            } else {
                gwz_core::StatusMode::Summary
            }),
            include_file_changes: Some(if combined { !self.no_files } else { true }),
            include_branch_summary: if combined {
                Some(!self.no_branches)
            } else {
                Some(true)
            },
            path_style: Some(gwz_core::StatusPathStyle::WorkspaceRelative),
        }))
    }
}

pub(crate) fn new_request_id() -> String {
    format!("req_{}", unique_suffix())
}
