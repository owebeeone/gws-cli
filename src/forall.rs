use std::process::{Command, ExitStatus};

use clap::Args;

use gwz_core::{ExecMode, ExecRequest, ExecResult, MemberEntry};

#[derive(Clone, Debug, Args)]
pub(crate) struct ForallArgs {
    #[arg(
        value_name = "projects",
        help = "Members to run in (id or path); empty = all. Put the command after `--`."
    )]
    pub(crate) projects: Vec<String>,

    #[arg(
        last = true,
        value_name = "cmd",
        help = "Command + args, run directly without a shell (portable). Use after `--`."
    )]
    pub(crate) command: Vec<String>,

    #[arg(
        short = 'c',
        long = "command-string",
        value_name = "string",
        help = "Run a shell command string (sh -c / cmd /C) instead of an argv"
    )]
    pub(crate) command_string: Option<String>,

    #[arg(long = "no-banner", help = "Suppress the per-member `=== <path> ===` banner")]
    pub(crate) no_banner: bool,
}

/// Resolve members + run the command, returning a `CliResponse` whose envelope carries the
/// aggregate status (for the exit code) and whose `summary` is the trailing `N/M failed: …` line
/// (empty when all succeed). Used by the `forall` execute arm.
#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_forall(
    start: &std::path::Path,
    meta: &gwz_core::RequestMeta,
    projects: &[String],
    mode: ExecMode,
    command: &[String],
    continue_on_fail: bool,
    no_banner: bool,
    operation_id: String,
) -> Result<crate::CliResponse, gwz_core::model::ModelError> {
    let root = gwz_core::workspace_ops::resolve_workspace_root(start, meta.workspace.as_ref())?;
    // Resolve the member list via the ls op, then narrow by the positional projects.
    let listed = gwz_core::workspace_ops::handle_ls(
        start,
        gwz_core::LsRequest {
            meta: meta.clone(),
            include_unmaterialized: None,
        },
        operation_id.clone(),
    )?;
    let members = filter_projects(listed.members.unwrap_or_default(), projects)?;

    let request = ExecRequest {
        meta: meta.clone(),
        mode,
        command: command.to_vec(),
        members,
        continue_on_fail: continue_on_fail.then_some(true),
    };
    let results = run_forall(&request, no_banner, root.to_string_lossy().as_ref());

    let failures: Vec<String> = results
        .iter()
        .filter(|result| !is_success(result))
        .map(|result| result.id.clone())
        .collect();
    let aggregate = if failures.is_empty() {
        gwz_core::AggregateStatus::Ok
    } else {
        gwz_core::AggregateStatus::Failed
    };
    let summary = if failures.is_empty() {
        String::new()
    } else {
        format!("{}/{} failed: {}", failures.len(), results.len(), failures.join(", "))
    };
    let envelope = gwz_core::operation::response_envelope_for(
        meta,
        gwz_core::operation::ActionKind::Forall,
        operation_id,
        aggregate,
        Vec::new(),
    )?;
    Ok(crate::CliResponse {
        envelope,
        workspace_git_status: None,
        status_mode: None,
        listing: None,
        branch_repos: None,
        stash_bundles: None,
        summary: Some(summary),
    })
}

/// Narrow `members` to those whose `id` or `path` matches a requested project. An unknown project
/// is an error (resolved here, in execute — parse can't see the manifest). Empty `projects` = all.
pub(crate) fn filter_projects(
    members: Vec<MemberEntry>,
    projects: &[String],
) -> Result<Vec<MemberEntry>, gwz_core::model::ModelError> {
    if projects.is_empty() {
        return Ok(members);
    }
    let mut selected: Vec<MemberEntry> = Vec::new();
    for project in projects {
        match members
            .iter()
            .find(|member| &member.id == project || &member.path == project)
        {
            Some(member) if !selected.iter().any(|chosen| chosen.id == member.id) => {
                selected.push(member.clone());
            }
            Some(_) => {}
            None => {
                return Err(gwz_core::model::ModelError::new(
                    gwz_core::model::ErrorCode::MemberNotFound,
                    format!("unknown project '{project}' (not a member id or path)"),
                ));
            }
        }
    }
    Ok(selected)
}

/// Run `req.command` in each member: cwd = member abspath, with `GWZ_MEMBER_*`/`GWZ_ROOT` env and
/// (argv mode) `{@}` → member path. Child stdio is **inherited** (streamed live); a
/// `=== <path> ===` banner goes to **stderr** per member unless `no_banner`. Stops at the first
/// failing member unless `req.continue_on_fail` (the `--partial` global). Returns one `ExecResult`
/// per member actually run.
pub(crate) fn run_forall(req: &ExecRequest, no_banner: bool, root: &str) -> Vec<ExecResult> {
    let continue_on_fail = req.continue_on_fail.unwrap_or(false);
    let mut results = Vec::with_capacity(req.members.len());
    for member in &req.members {
        if !no_banner {
            eprintln!("=== {} ===", member.path);
        }
        let result = run_one(req, member, root);
        let failed = !is_success(&result);
        results.push(result);
        if failed && !continue_on_fail {
            break;
        }
    }
    results
}

fn run_one(req: &ExecRequest, member: &MemberEntry, root: &str) -> ExecResult {
    let mut command = match req.mode {
        ExecMode::Argv => {
            // Substitute `{@}` → member path in each argv element.
            let mut args = req.command.iter().map(|arg| arg.replace("{@}", &member.path));
            let Some(program) = args.next() else {
                return spawn_failure(member, "empty command");
            };
            let mut command = Command::new(program);
            command.args(args);
            command
        }
        ExecMode::Shell => {
            let script = req.command.first().cloned().unwrap_or_default();
            let (program, flag) = if cfg!(windows) { ("cmd", "/C") } else { ("sh", "-c") };
            let mut command = Command::new(program);
            command.arg(flag).arg(script);
            command
        }
    };
    command
        .current_dir(&member.abspath)
        .env("GWZ_MEMBER_ID", &member.id)
        .env("GWZ_MEMBER_PATH", &member.path)
        .env("GWZ_MEMBER_ABSPATH", &member.abspath)
        .env("GWZ_ROOT", root);
    match command.status() {
        Ok(status) => ExecResult {
            id: member.id.clone(),
            path: member.path.clone(),
            exit_code: status.code().map(i64::from),
            signal: signal_of(&status),
            spawn_error: None,
        },
        Err(error) => spawn_failure(member, &error.to_string()),
    }
}

fn spawn_failure(member: &MemberEntry, message: &str) -> ExecResult {
    ExecResult {
        id: member.id.clone(),
        path: member.path.clone(),
        exit_code: None,
        signal: None,
        spawn_error: Some(message.to_owned()),
    }
}

fn signal_of(status: &ExitStatus) -> Option<i64> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status.signal().map(i64::from)
    }
    #[cfg(not(unix))]
    {
        let _ = status;
        None
    }
}

/// A member succeeded iff it exited 0, with no signal and no spawn failure.
pub(crate) fn is_success(result: &ExecResult) -> bool {
    result.spawn_error.is_none() && result.signal.is_none() && result.exit_code == Some(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(id: &str, path: &str) -> MemberEntry {
        MemberEntry {
            id: id.to_owned(),
            path: path.to_owned(),
            abspath: std::env::temp_dir().to_string_lossy().into_owned(),
            materialized: true,
        }
    }

    fn request(mode: ExecMode, command: &[&str], members: Vec<MemberEntry>, continue_on_fail: bool) -> ExecRequest {
        ExecRequest {
            meta: gwz_core::RequestMeta::default(),
            mode,
            command: command.iter().map(|arg| arg.to_string()).collect(),
            members,
            continue_on_fail: continue_on_fail.then_some(true),
        }
    }

    #[test]
    fn argv_substitutes_at_token() {
        // `test repos/app = repos/app` → exit 0, proving `{@}` → member path.
        let results = run_forall(
            &request(ExecMode::Argv, &["test", "{@}", "=", "repos/app"], vec![member("mem_app", "repos/app")], false),
            true,
            "/root",
        );
        assert_eq!(results[0].exit_code, Some(0));
        assert!(is_success(&results[0]));
    }

    #[test]
    fn shell_sees_member_env() {
        let script = r#"[ "$GWZ_MEMBER_PATH" = "repos/app" ] && [ "$GWZ_MEMBER_ID" = "mem_app" ] && [ "$GWZ_ROOT" = "/root" ]"#;
        let results = run_forall(
            &request(ExecMode::Shell, &[script], vec![member("mem_app", "repos/app")], false),
            true,
            "/root",
        );
        assert_eq!(results[0].exit_code, Some(0), "GWZ_MEMBER_* / GWZ_ROOT are exported");
    }

    #[test]
    fn spawn_failure_is_recorded() {
        let results = run_forall(
            &request(ExecMode::Argv, &["definitely-not-a-real-binary-zzz"], vec![member("mem_app", "repos/app")], false),
            true,
            "/root",
        );
        assert!(results[0].spawn_error.is_some());
        assert_eq!(results[0].exit_code, None);
        assert!(!is_success(&results[0]));
    }

    #[test]
    fn non_zero_exit_is_a_failure() {
        let results = run_forall(
            &request(ExecMode::Argv, &["false"], vec![member("mem_app", "repos/app")], false),
            true,
            "/root",
        );
        assert_eq!(results[0].exit_code, Some(1));
        assert!(!is_success(&results[0]));
    }

    #[test]
    fn filter_projects_matches_id_or_path_and_errors_on_unknown() {
        let members = vec![member("mem_app", "repos/app"), member("mem_lib", "repos/lib")];
        assert_eq!(filter_projects(members.clone(), &[]).unwrap().len(), 2, "empty = all");
        let by_id = filter_projects(members.clone(), &["mem_app".to_owned()]).unwrap();
        assert_eq!(by_id.len(), 1);
        assert_eq!(by_id[0].id, "mem_app");
        let by_path = filter_projects(members.clone(), &["repos/lib".to_owned()]).unwrap();
        assert_eq!(by_path[0].id, "mem_lib", "matched by path");
        assert!(filter_projects(members, &["nope".to_owned()]).is_err(), "unknown project errors");
    }

    #[test]
    fn stops_at_first_failure_unless_continue() {
        let two = || vec![member("mem_a", "a"), member("mem_b", "b")];
        // First member runs `false` (exit 1); default stops → only one result.
        let stopped = run_forall(&request(ExecMode::Argv, &["false"], two(), false), true, "/root");
        assert_eq!(stopped.len(), 1, "stopped at the first failure");
        // continue_on_fail runs the rest.
        let all = run_forall(&request(ExecMode::Argv, &["false"], two(), true), true, "/root");
        assert_eq!(all.len(), 2, "continue_on_fail runs every member");
    }
}
