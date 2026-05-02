use std::collections::HashSet;
use std::path::Path;

use serde_json::json;
use xiuxian_ast::Lang;

use super::{excluded_ast_languages_for_repository, supported_ast_lang};
use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig};

#[test]
fn supported_ast_lang_accepts_toml_when_not_excluded() {
    assert_eq!(
        supported_ast_lang(Path::new("Cargo.toml"), &HashSet::new()),
        Some(Lang::Toml)
    );
}

#[test]
fn supported_ast_lang_skips_language_owned_by_plugin_window() {
    let excluded_languages = HashSet::from(["toml".to_string()]);
    assert_eq!(
        supported_ast_lang(Path::new("Cargo.toml"), &excluded_languages),
        None
    );
}

#[test]
fn excluded_ast_languages_for_repository_uses_plugin_ids_and_explicit_options() {
    let repository = RegisteredRepository {
        id: "alpha/repo".to_string(),
        plugins: vec![
            RepositoryPluginConfig::Id("julia".to_string()),
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
