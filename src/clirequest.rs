use clap::{Args, Subcommand};

use crate::*;

// Default coalescing for member_progress events: at most one per member per
// 100 ms. Set as a request option so a driver can tune or disable it (0).
pub(crate) const DEFAULT_PROGRESS_MIN_INTERVAL_MS: i64 = 100;

#[derive(Clone, Debug, Args)]
pub(crate) struct CloneArgs {
    #[arg(value_name = "url", help = "Git URL of the workspace root repository")]
    pub(crate) url: String,

    #[arg(
        value_name = "directory",
        help = "Target directory for the cloned workspace",
        long_help = "Target directory for the cloned workspace. Defaults to a directory named after the workspace repository."
    )]
    pub(crate) dir: Option<String>,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct AddArgs {
    #[arg(
        value_name = "repo-path",
        help = "Path to an existing local git repository"
    )]
    pub(crate) repo_path: String,
}

#[derive(Clone, Debug, Subcommand)]
pub(crate) enum RepoCommandArgs {
    #[command(
        about = "Add an existing git repository as a member",
        long_about = ADD_LONG,
        after_long_help = ADD_AFTER
    )]
    Add(AddArgs),
    #[command(
        about = "Create a new repository member",
        long_about = REPO_CREATE_LONG,
        after_long_help = REPO_CREATE_AFTER
    )]
    Create(RepoCreateArgs),
}

#[derive(Clone, Debug, Args)]
pub(crate) struct RepoCreateArgs {
    #[arg(
        value_name = "member-path",
        help = "Workspace-relative path for the new repository member"
    )]
    pub(crate) member_path: String,
}

#[derive(Clone, Debug, Default, Args)]
pub(crate) struct MaterializeArgs {
    #[arg(
        long,
        help = "Materialize the workspace lock",
        long_help = "Materialize the workspace lock. This is the default target."
    )]
    pub(crate) lock: bool,

    #[arg(long, help = "Materialize repository heads")]
    pub(crate) head: bool,

    #[arg(long, value_name = "name", help = "Materialize a workspace snapshot")]
    pub(crate) snapshot: Option<String>,

    #[arg(long, value_name = "name", help = "Materialize a workspace tag")]
    pub(crate) tag: Option<String>,
}

#[derive(Clone, Debug, Default, Args)]
pub(crate) struct PullArgs {
    #[arg(
        long,
        help = "Pull repository heads",
        long_help = "Pull repository heads. This is the default target."
    )]
    pub(crate) head: bool,

    #[arg(long, value_name = "name", help = "Pull a workspace snapshot")]
    pub(crate) snapshot: Option<String>,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct CommitArgs {
    #[arg(
        short = 'm',
        long,
        value_name = "message",
        help = "Commit message applied to every committed repo"
    )]
    pub(crate) message: String,

    #[arg(
        short = 'a',
        long,
        help = "Stage tracked modifications first (git commit -a)"
    )]
    pub(crate) all: bool,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct StageArgs {
    #[arg(
        value_name = "pathspec",
        help = "Paths to stage; resolved relative to the current directory like `git add`"
    )]
    pub(crate) pathspecs: Vec<String>,

    #[arg(
        short = 'A',
        long = "all",
        help = "Stage all changes across every workspace repo (git add -A)"
    )]
    pub(crate) all: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CliInvocation {
    pub(crate) request: CliRequest,
    pub(crate) output: OutputMode,
    pub(crate) start_dir: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CliRequest {
    CreateWorkspace(gwz_core::CreateWorkspaceRequest),
    CloneWorkspace {
        meta: gwz_core::RequestMeta,
        url: String,
        target: String,
    },
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
    Capture(gwz_core::CaptureRequest),
    Commit(gwz_core::CommitRequest),
    Stage(gwz_core::StageRequest),
    ListSnapshots,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutputMode {
    Human,
    Json,
    Jsonl,
    Porcelain,
}

/// A short verb for the progress line, derived from the request kind. Only the
/// I/O-bound operations emit member events, so only those labels are ever seen.
pub(crate) fn operation_label(request: &CliRequest) -> &'static str {
    match request {
        CliRequest::CloneWorkspace { .. } => "cloning",
        CliRequest::Materialize(_) => "materializing",
        CliRequest::InitFromSources(_) => "initializing",
        CliRequest::PullSnapshot(_) => "pulling",
        _ => "working",
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CliError {
    pub(crate) message: String,
    pub(crate) code: Option<gwz_core::model::ErrorCode>,
}

impl CliError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
        }
    }

    /// Preserve a gwz-core error's code (so `--json`/`--jsonl` can emit it
    /// structured) alongside its message.
    pub(crate) fn from_model(error: gwz_core::model::ModelError) -> Self {
        Self {
            message: error.message,
            code: Some(error.code),
        }
    }

    /// Human rendering: prefix with the error code when present, matching
    /// gwz-core's `ModelError` Display.
    pub(crate) fn human_message(&self) -> String {
        match self.code {
            Some(code) => format!("{code:?}: {}", self.message),
            None => self.message.clone(),
        }
    }
}

impl Cli {
    pub(crate) fn validate(&self) -> Result<(), CliError> {
        if self.global.json && self.global.jsonl {
            return Err(CliError::new("--json and --jsonl are mutually exclusive"));
        }
        if self.global.all && (!self.global.members.is_empty() || !self.global.paths.is_empty()) {
            return Err(CliError::new(
                "--all cannot be combined with --member or --member-path",
            ));
        }
        if let CommandArgs::Status(status) = &self.command {
            status.validate(&self.global)?;
        }
        if matches!(&self.command, CommandArgs::Clone(_)) && self.global.dry_run {
            return Err(CliError::new("--dry-run is not supported for clone"));
        }
        Ok(())
    }

    pub(crate) fn output_mode(&self) -> OutputMode {
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

    pub(crate) fn request_meta(&self, request_id: &str) -> gwz_core::RequestMeta {
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

    pub(crate) fn selection(&self) -> Option<gwz_core::Selection> {
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

    pub(crate) fn policy(&self) -> Option<gwz_core::OperationPolicy> {
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
            max_connections_per_host: self.global.max_per_host,
            progress_min_interval_ms: Some(
                self.global
                    .progress_interval
                    .unwrap_or(DEFAULT_PROGRESS_MIN_INTERVAL_MS),
            ),
            ..Default::default()
        })
    }

    pub(crate) fn command_request(
        &self,
        meta: gwz_core::RequestMeta,
        workspace_root: String,
        current_dir: &std::path::Path,
    ) -> Result<CliRequest, CliError> {
        match &self.command {
            CommandArgs::Init(args) => args.request(meta, workspace_root),
            CommandArgs::Clone(args) => args.request(meta),
            CommandArgs::Add(args) => args.request(meta, current_dir),
            CommandArgs::Repo(args) => args.request(meta),
            CommandArgs::Status(args) => args.request(meta),
            CommandArgs::Snapshot(args) => match args.name.clone() {
                Some(name) if !args.list => {
                    Ok(CliRequest::Snapshot(gwz_core::SnapshotRequest { meta, snapshot_id: name }))
                }
                _ => Ok(CliRequest::ListSnapshots),
            },
            CommandArgs::Tag(args) => {
                let op = if args.push {
                    gwz_core::TagOp::Push
                } else if args.fetch {
                    gwz_core::TagOp::Fetch
                } else if args.delete {
                    gwz_core::TagOp::Delete
                } else if args.list || args.name.is_none() {
                    gwz_core::TagOp::List
                } else {
                    gwz_core::TagOp::Create
                };
                Ok(CliRequest::Tag(gwz_core::TagRequest {
                    meta,
                    op,
                    name: args.name.clone(),
                    message: args.message.clone(),
                    signed: args.signed.then_some(true),
                    remote: self.global.remote.clone(),
                    all: None,
                }))
            }
            CommandArgs::Materialize(args) => args.request(meta),
            CommandArgs::Pull(args) => args.request(meta),
            CommandArgs::Push => Ok(CliRequest::Push(gwz_core::PushRequest {
                remote: self.global.remote.clone(),
                refspec: None,
                meta,
            })),
            CommandArgs::Capture => Ok(CliRequest::Capture(gwz_core::CaptureRequest { meta })),
            CommandArgs::Commit(args) => args.request(meta),
        }
    }
}

impl InitArgs {
    pub(crate) fn request(
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
                        .map(|url| {
                            Ok(gwz_core::SourceUrl {
                                path: init_source_path(&self.path_prefix, &url)?,
                                url,
                                remote_name: None,
                                branch: None,
                            })
                        })
                        .collect::<Result<Vec<_>, CliError>>()?,
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

impl CloneArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        let target = match &self.dir {
            Some(dir) => dir.clone(),
            None => repo_name_from_url(&self.url)?,
        };
        Ok(CliRequest::CloneWorkspace {
            meta,
            url: self.url.clone(),
            target,
        })
    }
}

impl AddArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
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
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        match &self.command {
            RepoCommandArgs::Add(args) => args.request(meta),
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

impl MaterializeArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        Ok(CliRequest::Materialize(gwz_core::MaterializeRequest {
            meta,
            target: self.target()?,
        }))
    }

    pub(crate) fn target(&self) -> Result<gwz_core::MaterializeTarget, CliError> {
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
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
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

impl CommitArgs {
    pub(crate) fn request(&self, meta: gwz_core::RequestMeta) -> Result<CliRequest, CliError> {
        Ok(CliRequest::Commit(gwz_core::CommitRequest {
            meta,
            message: self.message.clone(),
            all: self.all.then_some(true),
        }))
    }
}

impl StageArgs {
    pub(crate) fn request(
        &self,
        meta: gwz_core::RequestMeta,
        cwd: &std::path::Path,
    ) -> Result<CliRequest, CliError> {
        Ok(CliRequest::Stage(gwz_core::StageRequest {
            meta,
            cwd: cwd.to_string_lossy().into_owned(),
            pathspecs: self.pathspecs.clone(),
            all: self.all.then_some(true),
        }))
    }
}

pub(crate) fn init_source_path(path_prefix: &str, url: &str) -> Result<Option<String>, CliError> {
    let prefix = path_prefix
        .replace('\\', "/")
        .trim_matches(|value| value == '/')
        .to_owned();
    if prefix.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(format!("{prefix}/{}", repo_name_from_url(url)?)))
}

pub(crate) fn repo_name_from_url(url: &str) -> Result<String, CliError> {
    let trimmed = url.trim_end_matches(['/', '\\']);
    let segment = trimmed
        .rsplit(['/', '\\', ':'])
        .find(|part| !part.is_empty())
        .unwrap_or(trimmed);
    let name = segment.strip_suffix(".git").unwrap_or(segment);
    if name.is_empty() {
        Err(CliError::new(
            "source URL does not include a repository name",
        ))
    } else {
        Ok(name.to_owned())
    }
}
