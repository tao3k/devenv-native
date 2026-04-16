use super::*;

#[test]
fn test_strip_option() {
    assert_eq!(strip_option(""), None);
    assert_eq!(strip_option("value"), Some("value".to_string()));
    assert_eq!(strip_option(" value "), Some("value".to_string()));
}

#[test]
fn repo_navigation_target_prefixes_repo_root_for_relative_paths() {
    let target = repo_navigation_target("mcl", "Modelica/package.mo", None, None, None);
    assert_eq!(target.path, "mcl/Modelica/package.mo");
    assert_eq!(target.category, "repo_code");
    assert_eq!(target.project_name.as_deref(), Some("mcl"));
    assert_eq!(target.root_label.as_deref(), Some("mcl"));
}

#[test]
fn repo_navigation_target_does_not_duplicate_existing_repo_root_prefix() {
    let target = repo_navigation_target("mcl", "mcl/Modelica/package.mo", None, None, None);
    assert_eq!(target.path, "mcl/Modelica/package.mo");
}

#[test]
fn parse_content_search_line_parses_ripgrep_output() {
    let parsed = parse_content_search_line(
        "/tmp/repo/src/DifferentialEquations.jl:42:@reexport using SciMLBase",
    );
    let Some((path, line_number, snippet)) = parsed else {
        panic!("expected ripgrep output to parse");
    };

    assert_eq!(path, "/tmp/repo/src/DifferentialEquations.jl");
    assert_eq!(line_number, 42);
    assert_eq!(snippet, "@reexport using SciMLBase");
}

#[test]
fn supported_code_extension_includes_julia_and_modelica() {
    assert!(is_supported_code_extension("src/Foo.jl"));
    assert!(is_supported_code_extension("Modelica/package.mo"));
    assert!(!is_supported_code_extension("docs/readme.md"));
}

#[test]
fn truncate_content_search_snippet_limits_output_length() {
    let value = "abcdefghijklmnopqrstuvwxyz";
    let truncated = truncate_content_search_snippet(value, 8);
    assert_eq!(truncated, "abcdefgh...");
}

#[test]
fn code_content_globs_do_not_exclude_cache_root() {
    assert!(!CODE_CONTENT_EXCLUDE_GLOBS.contains(&"!.cache/**"));
}

#[test]
fn language_filter_matches_julia_path_extensions() {
    let mut filters = std::collections::HashSet::new();
    filters.insert("julia".to_string());

    assert!(path_matches_language_filters(
        "src/BaseModelica.jl",
        &filters
    ));
    assert!(path_matches_language_filters(
        "src/generated/parser.julia",
        &filters
    ));
    assert!(!path_matches_language_filters("docs/index.md", &filters));
}
