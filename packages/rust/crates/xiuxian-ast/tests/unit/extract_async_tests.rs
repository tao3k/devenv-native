use xiuxian_ast::{
    Lang, extract_items, extract_items_for_patterns, extract_skeleton, get_skeleton_patterns,
};

#[test]
fn test_extract_rust_async_function_skeletons() {
    let content = r#"
async fn fetch_local() -> String {
    "local".to_string()
}

pub async fn fetch_public() -> String {
    "public".to_string()
}

pub(crate) async fn fetch_crate() -> String {
    "crate".to_string()
}
"#;

    let private_results = extract_items(content, "async fn $NAME", Lang::Rust, Some(vec!["NAME"]));
    let private_names: Vec<&str> = private_results
        .iter()
        .map(|result| result.captures["NAME"].as_str())
        .collect();
    assert_eq!(private_names, vec!["fetch_local"]);

    let public_results = extract_items(
        content,
        "pub async fn $NAME",
        Lang::Rust,
        Some(vec!["NAME"]),
    );
    let public_names: Vec<&str> = public_results
        .iter()
        .map(|result| result.captures["NAME"].as_str())
        .collect();
    assert_eq!(public_names, vec!["fetch_public", "fetch_crate"]);
}

#[test]
fn test_extract_rust_items_for_patterns_parses_once_and_matches_all_patterns() {
    let content = r#"
pub struct RepoCodeSearchOutcome {
    count: usize,
}

pub async fn search_repo_code_outcome_for_query() -> RepoCodeSearchOutcome {
    RepoCodeSearchOutcome { count: 1 }
}
"#;

    let patterns = get_skeleton_patterns(Lang::Rust);
    let results = extract_items_for_patterns(content, patterns, Lang::Rust, Some(vec!["NAME"]));
    let names = results
        .iter()
        .map(|result| result.captures["NAME"].as_str())
        .collect::<Vec<_>>();

    assert!(names.contains(&"RepoCodeSearchOutcome"));
    assert!(names.contains(&"search_repo_code_outcome_for_query"));
}

#[test]
fn test_extract_skeleton_rust_async_functions() {
    let content = r#"
pub async fn search_repo_code_outcome_for_query(
    search_plane: &SearchPlaneService,
) -> Result<(), String> {
    search_plane.flush().await
}
"#;

    let skeleton = extract_skeleton(content, Lang::Rust);

    assert!(
        skeleton.contains("pub async fn search_repo_code_outcome_for_query"),
        "Should contain async Rust function signature"
    );
    assert!(
        !skeleton.contains("flush().await"),
        "Should not contain async function body"
    );
}
