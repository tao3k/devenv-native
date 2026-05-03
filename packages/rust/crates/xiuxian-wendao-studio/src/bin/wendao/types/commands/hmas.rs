use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub(crate) enum HmasCommand {
    /// Validate markdown blackboard protocol blocks from a file.
    Validate {
        #[arg(long)]
        file: PathBuf,
    },
}
