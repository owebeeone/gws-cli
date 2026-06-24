use clap::Args;

#[derive(Clone, Debug, Args)]
pub(crate) struct LsArgs {
    #[arg(
        long,
        help = "Print workspace-relative paths instead of absolute paths"
    )]
    pub(crate) local: bool,

    #[arg(
        long,
        help = "Include configured-but-unmaterialized members"
    )]
    pub(crate) unmaterialized: bool,
}
