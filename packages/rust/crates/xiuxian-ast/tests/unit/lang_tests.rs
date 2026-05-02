use xiuxian_ast::Lang;

#[test]
fn test_lang_all_returns_supported_languages_in_stable_order() {
    let languages = Lang::all()
        .iter()
        .copied()
        .map(Lang::as_str)
        .collect::<Vec<_>>();

    assert_eq!(
        languages,
        vec![
            "python",
            "rust",
            "javascript",
            "typescript",
            "bash",
            "go",
            "java",
            "c",
            "cpp",
            "csharp",
            "ruby",
            "swift",
            "kotlin",
            "lua",
            "php",
            "json",
            "yaml",
            "toml",
            "markdown",
            "dockerfile",
            "html",
            "css",
            "sql",
        ]
    );
}
