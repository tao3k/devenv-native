use crate::studio::types::StudioNavigationTarget;

pub(super) fn source_symbol_navigation_target(
    path: &str,
    crate_name: &str,
    project_name: Option<&str>,
    root_label: Option<&str>,
    line_start: usize,
    line_end: usize,
) -> StudioNavigationTarget {
    StudioNavigationTarget {
        path: path.to_string().into(),
        category: "doc".into(),
        project_name: project_name
            .map(ToString::to_string)
            .or_else(|| Some(crate_name.to_string())),
        root_label: root_label.map(ToString::to_string),
        line: Some(line_start),
        line_end: Some(line_end),
        column: None,
    }
}
