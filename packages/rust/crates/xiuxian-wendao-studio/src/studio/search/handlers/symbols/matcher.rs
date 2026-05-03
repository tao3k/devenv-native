use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::studio::types::UiProjectConfig;

pub(super) fn build_project_glob_matcher(projects: &[UiProjectConfig]) -> Option<GlobSet> {
    let patterns = projects
        .iter()
        .flat_map(|project| project.dirs.iter())
        .filter(|dir| is_glob_pattern(dir.as_str()))
        .collect::<Vec<_>>();
    if patterns.is_empty() {
        return None;
    }

    let mut builder = GlobSetBuilder::new();
    let mut has_pattern = false;
    for pattern in patterns {
        let Ok(glob) = Glob::new(pattern.as_str()) else {
            continue;
        };
        builder.add(glob);
        has_pattern = true;
    }

    if !has_pattern {
        return None;
    }

    builder.build().ok()
}

fn is_glob_pattern(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[')
}
