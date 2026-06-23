use clap::Args;

#[derive(Clone, Debug, Args)]
pub(crate) struct NameArgs {
    #[arg(value_name = "name", help = "Workspace-level name to record (omit to list existing)")]
    pub(crate) name: Option<String>,

    #[arg(long, help = "List existing entries instead of recording one")]
    pub(crate) list: bool,
}
