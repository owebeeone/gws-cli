use clap::Args;
#[cfg(test)]
use clap::CommandFactory;

#[derive(Clone, Debug, Args)]
pub(crate) struct NameArgs {
    #[arg(value_name = "name", help = "Workspace-level name to record")]
    pub(crate) name: String,
}
