use crate::bootstrap_builtin_registry;
use xiuxian_wendao_core::repo_intelligence::{
    AnalysisContext, RegisteredRepository, RepositoryPluginConfig,
};

#[test]
fn bootstrap_builtin_registry_registers_julia_line_plugins_by_default() {
    let registry = bootstrap_builtin_registry()
        .unwrap_or_else(|error| panic!("builtin registry bootstrap should succeed: {error}"));

    assert!(
        registry.get("julia-code-parser").is_some(),
        "default builtin registry should include the external Julia plugin"
    );
    assert!(
        registry.get("modelica").is_some(),
        "builtin Julia line should also include the Modelica plugin"
    );
    assert!(
        registry.get("markdown-parser").is_some(),
        "default builtin registry should include the Markdown parser plugin"
    );
}

#[test]
fn markdown_parser_plugin_discovers_page_index_ready_markdown() {
    let registry = bootstrap_builtin_registry()
        .unwrap_or_else(|error| panic!("builtin registry bootstrap should succeed: {error}"));
    let plugin = registry
        .get("markdown-parser")
        .unwrap_or_else(|| panic!("markdown-parser plugin should be registered"));
    let root = unique_temp_dir("xiuxian-wendao-builtin-markdown-parser");
    std::fs::create_dir_all(root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::write(root.join("README.md"), "# Root Readme\n")
        .unwrap_or_else(|error| panic!("write README: {error}"));
    std::fs::write(root.join("docs").join("search.md"), "# Search Strategy\n")
        .unwrap_or_else(|error| panic!("write docs/search.md: {error}"));
    std::fs::create_dir_all(root.join("notes"))
        .unwrap_or_else(|error| panic!("create notes dir: {error}"));
    std::fs::write(root.join("notes").join("intent.md"), "# Intent Flow\n")
        .unwrap_or_else(|error| panic!("write notes/intent.md: {error}"));
    std::fs::write(root.join("src.jl"), "module Demo\nend\n")
        .unwrap_or_else(|error| panic!("write src.jl: {error}"));

    let repository = RegisteredRepository {
        id: "knowledge".to_string(),
        path: Some(root.clone()),
        plugins: vec![RepositoryPluginConfig::Id("markdown-parser".to_string())],
        ..RegisteredRepository::default()
    };
    let output = plugin
        .analyze_repository(
            &AnalysisContext {
                repository,
                repository_root: root.clone(),
            },
            root.as_path(),
        )
        .unwrap_or_else(|error| panic!("analyze Markdown files: {error}"));

    let paths = output
        .docs
        .iter()
        .map(|doc| doc.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec!["README.md", "docs/search.md", "notes/intent.md"]
    );
    assert_eq!(output.docs[2].title, "Intent Flow");

    std::fs::remove_dir_all(root).ok();
}

fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|error| panic!("system clock before epoch: {error}"))
            .as_nanos()
    ));
    std::fs::create_dir_all(path.as_path())
        .unwrap_or_else(|error| panic!("create temp dir `{}`: {error}", path.display()));
    path
}
