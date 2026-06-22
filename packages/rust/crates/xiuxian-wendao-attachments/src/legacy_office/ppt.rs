use std::path::Path;

use super::panic_guard;

pub(crate) fn extract_text(path: &Path) -> Result<String, String> {
    panic_guard::run("PowerPoint 97-2003 presentation", path, || {
        litchi::Presentation::open(path)
            .and_then(|presentation| presentation.text())
            .map_err(|error| {
                format!(
                    "parse legacy PowerPoint presentation `{}`: {error}",
                    path.display()
                )
            })
    })
}
