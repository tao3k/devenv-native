use std::fs;
use std::path::Path;

use crate::error::QianjiError;

pub(super) fn ensure_output_dir_is_safe(output_dir: &Path) -> Result<(), QianjiError> {
    if !output_dir.exists() {
        return Ok(());
    }

    if !output_dir.is_dir() {
        return Err(QianjiError::Topology(format!(
            "materialize target `{}` exists but is not a directory",
            output_dir.display()
        )));
    }

    let mut entries = fs::read_dir(output_dir).map_err(|error| {
        QianjiError::Topology(format!(
            "Failed to inspect materialize target `{}`: {error}",
            output_dir.display()
        ))
    })?;
    if entries.next().is_some() {
        return Err(QianjiError::Topology(format!(
            "materialize target `{}` must be empty",
            output_dir.display()
        )));
    }

    Ok(())
}
