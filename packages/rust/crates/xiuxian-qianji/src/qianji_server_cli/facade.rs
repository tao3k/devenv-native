//! Facade surface for `xiuxian-qianji`.

use super::{cli, run};
use std::error::Error;
use std::fmt;

/// Typed public error for the `qianji-server` command-line facade.
#[derive(Debug)]
pub struct QianjiServerCliError {
    source: anyhow::Error,
}

impl From<anyhow::Error> for QianjiServerCliError {
    fn from(source: anyhow::Error) -> Self {
        Self { source }
    }
}

impl fmt::Display for QianjiServerCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "qianji-server CLI failed: {}", self.source)
    }
}

impl Error for QianjiServerCliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// Runs the `qianji-server` command-line interface.
///
/// # Errors
/// Returns an error if argument parsing, socket binding, or HTTP serving fails.
pub async fn run_qianji_server_cli<I, S>(args: I) -> Result<(), QianjiServerCliError>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let command = cli::parse_qianji_server_args(args)?;
    run::run_qianji_server(command).await.map_err(Into::into)
}
