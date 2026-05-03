use chrono::Utc;

use crate::studio::types::UiProjectConfig;

pub(crate) fn fingerprint_projects(projects: &[UiProjectConfig]) -> String {
    projects
        .iter()
        .map(|project| {
            format!(
                "{}|{}|{}",
                project.name,
                project.root,
                project.dirs.join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("::")
}

pub(crate) fn timestamp_now() -> String {
    Utc::now().to_rfc3339()
}
