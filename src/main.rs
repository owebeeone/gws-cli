use clap::Parser;

mod add_after;
mod add_long;
mod append_branch_summary;
mod branch_status_json;
mod cli_after;
mod cli_long;
mod clirequest;
mod clone_after;
mod clone_long;
mod git_status_json;
mod git_transfer_progress_json;
mod globalargs;
mod impl_from_syncarg_for_gwz_core_syncbehavior;
mod init_after;
mod init_long;
mod materialize_after;
mod materialize_long;
mod nameargs;
mod parse_non_negative_i64;
mod parse_positive_i64;
mod progress_detail;
mod pull_after;
mod pull_long;
mod push_after;
mod push_long;
mod repo_after;
mod repo_create_after;
mod repo_create_long;
mod repo_long;
mod response_meta_json;
mod snapshot_after;
mod snapshot_long;
mod stage_after;
mod stage_long;
mod status_after;
mod status_long;
mod statusargs;
mod tag_after;
mod tag_long;
#[cfg(test)]
mod tests;
mod unique_suffix;

pub(crate) use add_after::*;
pub(crate) use add_long::*;
pub(crate) use append_branch_summary::*;
pub(crate) use branch_status_json::*;
pub(crate) use cli_after::*;
pub(crate) use cli_long::*;
pub(crate) use clirequest::*;
pub(crate) use clone_after::*;
pub(crate) use clone_long::*;
pub(crate) use git_status_json::*;
pub(crate) use git_transfer_progress_json::*;
pub(crate) use globalargs::*;
pub(crate) use init_after::*;
pub(crate) use init_long::*;
pub(crate) use materialize_after::*;
pub(crate) use materialize_long::*;
pub(crate) use nameargs::*;
pub(crate) use parse_non_negative_i64::*;
pub(crate) use parse_positive_i64::*;
pub(crate) use progress_detail::*;
pub(crate) use pull_after::*;
pub(crate) use pull_long::*;
pub(crate) use push_after::*;
pub(crate) use push_long::*;
pub(crate) use repo_after::*;
pub(crate) use repo_create_after::*;
pub(crate) use repo_create_long::*;
pub(crate) use repo_long::*;
pub(crate) use response_meta_json::*;
pub(crate) use snapshot_after::*;
pub(crate) use snapshot_long::*;
pub(crate) use stage_after::*;
pub(crate) use stage_long::*;
pub(crate) use status_after::*;
pub(crate) use status_long::*;
pub(crate) use statusargs::*;
pub(crate) use tag_after::*;
pub(crate) use tag_long::*;
pub(crate) use unique_suffix::*;

fn main() {
    let cli = Cli::parse();
    // Bound stalled SSH/network reads (libssh2 has no timeout by default, so a missing
    // ssh-agent identity or unreachable host would hang forever). Set once, before any
    // operation spawns threads. `--ssh-timeout` is in seconds (0 disables); default 3s.
    let ssh_timeout_ms = (cli.global.ssh_timeout.unwrap_or(3).saturating_mul(1000))
        .clamp(0, i32::MAX as i64) as i32;
    gwz_core::git::set_server_timeout_ms(ssh_timeout_ms);
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
                std::process::exit(exit_code_for_response(&response.envelope));
            }
            Err(error) => {
                // F9: structured machine output keeps errors on the same channel
                // and shape as success; human/porcelain stay on stderr.
                match invocation.output {
                    OutputMode::Json | OutputMode::Jsonl => {
                        println!("{}", render_error_json(&error));
                    }
                    OutputMode::Human | OutputMode::Porcelain => {
                        eprintln!("gwz: {}", error.human_message());
                    }
                }
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("gwz: {}", error.human_message());
            std::process::exit(2);
        }
    }
}
