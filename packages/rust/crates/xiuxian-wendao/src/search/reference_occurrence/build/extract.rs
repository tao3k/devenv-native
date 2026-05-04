use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;
use xiuxian_ast::Lang;

use crate::search::contracts::{ReferenceSearchHit, SearchProjectConfig, StudioNavigationTarget};
use crate::search::contracts::{infer_crate_name, project_metadata_for_path};
use crate::search::{ProjectScannedFile, SearchPlaneService};

static REFERENCE_TOKEN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").unwrap_or_else(|error| {
        panic!("reference token regex must compile: {error}");
    })
});

pub(crate) fn build_reference_occurrences_for_file(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    file: &ProjectScannedFile,
) -> Vec<ReferenceSearchHit> {
    let normalized_path_ref = Path::new(file.normalized_path.as_str());
    let Some(language) = reference_scan_lang(normalized_path_ref) else {
        return Vec::new();
    };
    let metadata = project_metadata_for_path(
        project_root,
        config_root,
        projects,
        file.normalized_path.as_str(),
    );
    let crate_name = infer_crate_name(normalized_path_ref);
    let snapshot = service.shared_source_snapshot_entry(project_root, file);
    let definition_locations = snapshot
        .ast_hits
        .iter()
        .cloned()
        .map(|hit| (hit.name.to_ascii_lowercase(), hit.path, hit.line_start))
        .collect::<HashSet<_>>();

    let mut hits = Vec::new();
    for (line_idx, line_text) in snapshot.content.lines().enumerate() {
        let line_number = line_idx + 1;
        let mut seen_tokens = HashSet::new();
        for mat in REFERENCE_TOKEN_PATTERN.find_iter(line_text) {
            let token = mat.as_str();
            let token_folded = token.to_ascii_lowercase();
            if !seen_tokens.insert(token_folded.clone()) {
                continue;
            }
            if definition_locations.contains(&(
                token_folded,
                file.normalized_path.clone(),
                line_number,
            )) {
                continue;
            }

            let column = line_text[..mat.start()].chars().count() + 1;
            hits.push(ReferenceSearchHit {
                name: token.to_string(),
                path: file.normalized_path.clone(),
                language: language.to_string(),
                crate_name: crate_name.clone(),
                project_name: metadata.project_name.clone(),
                root_label: metadata.root_label.clone(),
                navigation_target: reference_navigation_target(
                    file.normalized_path.as_str(),
                    crate_name.as_str(),
                    metadata.project_name.as_deref(),
                    metadata.root_label.as_deref(),
                    line_number,
                    column,
                ),
                line: line_number,
                column,
                line_text: line_text.trim().to_string(),
                score: 0.0,
            });
        }
    }
    hits
}

fn reference_scan_lang(path: &Path) -> Option<&'static str> {
    match Lang::from_path(path)? {
        Lang::Python => Some("python"),
        Lang::Rust => Some("rust"),
        Lang::JavaScript => Some("javascript"),
        Lang::TypeScript => Some("typescript"),
        Lang::Bash => Some("bash"),
        Lang::Go => Some("go"),
        Lang::Java => Some("java"),
        Lang::C => Some("c"),
        Lang::Cpp => Some("cpp"),
        Lang::CSharp => Some("csharp"),
        Lang::Ruby => Some("ruby"),
        Lang::Swift => Some("swift"),
        Lang::Kotlin => Some("kotlin"),
        Lang::Lua => Some("lua"),
        Lang::Php => Some("php"),
        _ => None,
    }
}

fn reference_navigation_target(
    path: &str,
    crate_name: &str,
    project_name: Option<&str>,
    root_label: Option<&str>,
    line: usize,
    column: usize,
) -> StudioNavigationTarget {
    StudioNavigationTarget {
        path: path.to_string(),
        category: "doc".to_string(),
        project_name: project_name
            .map(ToString::to_string)
            .or_else(|| Some(crate_name.to_string())),
        root_label: root_label.map(ToString::to_string),
        line: Some(line),
        line_end: Some(line),
        column: Some(column),
    }
}
