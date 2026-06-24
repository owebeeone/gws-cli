use clap::Args;

#[derive(Clone, Debug, Args)]
pub(crate) struct NameArgs {
    #[arg(value_name = "name", help = "Workspace-level name to record (omit to list existing)")]
    pub(crate) name: Option<String>,

    #[arg(long, help = "List existing entries instead of recording one")]
    pub(crate) list: bool,
}

#[derive(Clone, Debug, clap::Args)]
#[command(group(
    clap::ArgGroup::new("tag_action")
        .args(["list", "delete", "push", "fetch"])
        .multiple(false)
))]
pub(crate) struct TagArgs {
    #[arg(value_name = "name", help = "Tag name (omit to list)")]
    pub(crate) name: Option<String>,

    #[arg(long, help = "List tags (the default with no name)")]
    pub(crate) list: bool,

    #[arg(long, help = "Delete the named tag")]
    pub(crate) delete: bool,

    #[arg(long, help = "Push tags to a remote (a named tag, or all gwz tags if no name)")]
    pub(crate) push: bool,

    #[arg(long, help = "Fetch gwz tags from a remote")]
    pub(crate) fetch: bool,

    #[arg(short = 'm', value_name = "message", help = "Annotated tag message")]
    pub(crate) message: Option<String>,

    #[arg(short = 's', long = "sign", help = "Create a signed tag")]
    pub(crate) signed: bool,
}
