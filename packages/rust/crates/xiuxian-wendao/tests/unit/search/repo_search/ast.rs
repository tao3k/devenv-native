use std::collections::HashSet;
use std::path::Path;

use serde_json::json;

use super::{
    build_repo_ast_analysis_index_from_checkout, excluded_ast_languages_for_repository,
    supported_ast_lang,
};
use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig};

#[test]
fn supported_ast_lang_accepts_toml_when_not_excluded() {
    assert_eq!(
        supported_ast_lang(Path::new("src/lib.rs"), &HashSet::new()).map(|lang| lang.as_str()),
        Some("rust")
    );
}

#[test]
fn supported_ast_lang_skips_language_owned_by_plugin_window() {
    let excluded_languages = HashSet::from(["rust".to_string()]);
    assert_eq!(
        supported_ast_lang(Path::new("src/lib.rs"), &excluded_languages),
        None
    );
}

#[test]
fn excluded_ast_languages_for_repository_uses_plugin_ids_and_explicit_options() {
    let repository = RegisteredRepository {
        id: "alpha/repo".to_string(),
        plugins: vec![
            RepositoryPluginConfig::Id("julia-code-parser".to_string()),
            RepositoryPluginConfig::Id("TS".to_string()),
            RepositoryPluginConfig::Config {
                id: "custom-parser".to_string(),
                options: json!({
                    "language": "modelica",
                    "languages": ["sql", "yaml"],
                    "ast_grep_exclude_languages": ["toml", "md"],
                }),
            },
        ],
        ..RegisteredRepository::default()
    };

    let excluded_languages = excluded_ast_languages_for_repository(&repository);

    assert!(excluded_languages.contains("julia"));
    assert!(excluded_languages.contains("typescript"));
    assert!(excluded_languages.contains("modelica"));
    assert!(excluded_languages.contains("sql"));
    assert!(excluded_languages.contains("yaml"));
    assert!(excluded_languages.contains("toml"));
    assert!(excluded_languages.contains("markdown"));
    assert!(excluded_languages.contains("custom-parser"));
}

#[test]
fn repo_ast_analysis_index_preserves_scan_boundary_after_ast_retirement()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let source_dir = temp
        .path()
        .join("packages/rust/crates/xiuxian-wendao/src/search/repo_search");
    std::fs::create_dir_all(source_dir.as_path())?;
    std::fs::write(
        source_dir.join("orchestration.rs"),
        r"
pub struct RepoCodeSearchOutcome {
    pub count: usize,
}

pub async fn search_repo_code_outcome_for_query() -> RepoCodeSearchOutcome {
RepoCodeSearchOutcome { count: 1 }
}
",
    )?;
    std::fs::create_dir_all(temp.path().join("packages/rust/crates/other/src"))?;
    std::fs::write(
        temp.path().join("packages/rust/crates/other/src/lib.rs"),
        "pub async fn search_repo_code_outcome_for_query() {}\n",
    )?;
    let repository = RegisteredRepository {
        id: "alpha/repo".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("ast-grep".to_string())],
        ..RegisteredRepository::default()
    };

    let index = build_repo_ast_analysis_index_from_checkout(
        temp.path(),
        &repository,
        &["rust".to_string()],
        &["packages/rust/crates/xiuxian-wendao".to_string()],
        &[],
    );
    let hits = index.search(Some("search_repo_code_outcome_for_query"), 10);

    assert_eq!(index.file_count(), 1);
    assert_eq!(index.symbol_count(), 0);
    assert!(hits.is_empty());
    Ok(())
}
