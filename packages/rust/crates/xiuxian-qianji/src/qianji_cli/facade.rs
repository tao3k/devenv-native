//! Facade surface for `xiuxian-qianji`.

use super::dispatch;
use std::error::Error;
use std::fmt;

/// Typed public error for the `qianji` command-line facade.
#[derive(Debug)]
pub struct QianjiCliError {
    source: Box<dyn Error>,
}

impl From<Box<dyn Error>> for QianjiCliError {
    fn from(source: Box<dyn Error>) -> Self {
        Self { source }
    }
}

impl fmt::Display for QianjiCliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "qianji CLI failed: {}", self.source)
    }
}

impl Error for QianjiCliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// Runs the `qianji` command-line interface.
///
/// # Errors
/// Returns an error if argument parsing, environment resolution, compilation, or execution fails.
pub async fn run_qianji_cli() -> Result<(), QianjiCliError> {
    Box::pin(dispatch::run()).await.map_err(Into::into)
}
