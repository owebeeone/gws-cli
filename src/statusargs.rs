use clap::Args;
#[cfg(test)]
use clap::CommandFactory;

#[derive(Clone, Debug, Args)]
pub(crate) struct StatusArgs {
    #[arg(
        long,
        help = "Render combined workspace status",
        long_help = "Render combined workspace status. This is the default mode."
    )]
    pub(crate) combined: bool,

    #[arg(
        long = "no-combined",
        help = "Render per-repo status with file changes",
        long_help = "Render per-repo status with file changes instead of one combined workspace view."
    )]
    pub(crate) no_combined: bool,

    #[arg(
        long,
        help = "Render porcelain output",
        long_help = "Render stable script-oriented output instead of human-readable text."
    )]
    pub(crate) porcelain: bool,

    #[arg(
        long = "no-files",
        help = "Omit file changes from combined status",
        long_help = "Omit file changes from combined status while keeping branch summaries."
    )]
    pub(crate) no_files: bool,

    #[arg(
        long = "no-branches",
        help = "Omit branch summaries from combined status",
        long_help = "Omit branch summaries from combined status while keeping file changes."
    )]
    pub(crate) no_branches: bool,
}
