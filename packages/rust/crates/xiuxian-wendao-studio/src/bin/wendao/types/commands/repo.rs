use clap::{Args, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub(crate) enum RepoSyncModeArg {
    #[default]
    Ensure,
    Refresh,
    Status,
}

#[derive(Args, Debug)]
pub(crate) struct RepoSyncArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long, value_enum, default_value_t = RepoSyncModeArg::Ensure)]
    pub mode: RepoSyncModeArg,
}

#[derive(Args, Debug)]
pub(crate) struct RepoOverviewArgs {
    #[arg(long)]
    pub repo: String,
}

#[derive(Args, Debug)]
pub(crate) struct RepoModuleSearchArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub query: String,
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub(crate) struct RepoSymbolSearchArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub query: String,
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub(crate) struct RepoExampleSearchArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub query: String,
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub(crate) struct RepoDocCoverageArgs {
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub module: Option<String>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum RepoCommand {
    /// Synchronize one registered repository source and return resolved paths.
    Sync(RepoSyncArgs),
    /// Return repository overview counts and metadata.
    Overview(RepoOverviewArgs),
    /// Search normalized modules for a repository.
    ModuleSearch(RepoModuleSearchArgs),
    /// Search normalized symbols for a repository.
    SymbolSearch(RepoSymbolSearchArgs),
    /// Search normalized examples for a repository.
    ExampleSearch(RepoExampleSearchArgs),
    /// Return deterministic documentation coverage for a repository or module.
    DocCoverage(RepoDocCoverageArgs),
}
