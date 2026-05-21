use crate::studio::StudioState;
use crate::studio::pathing::{studio_display_path, studio_project_name};
use crate::studio::types::StudioNavigationTarget;

pub(crate) fn resolve_navigation_target(state: &StudioState, path: &str) -> StudioNavigationTarget {
    let normalized = studio_display_path(state, path);
    let project_name = studio_project_name(state, normalized.as_str());

    StudioNavigationTarget {
        path: normalized.into(),
        category: "file".into(),
        project_name,
        root_label: None,
        line: None,
        line_end: None,
        column: None,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/vfs/navigation.rs"]
mod tests;
