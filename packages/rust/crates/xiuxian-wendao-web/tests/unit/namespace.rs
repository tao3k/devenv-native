const WEB_MANIFEST: &str = include_str!("../../Cargo.toml");

#[test]
fn web_manifest_does_not_own_studio_or_domain_dependencies() {
    let forbidden_dependencies = [
        "xiuxian-wendao =",
        "xiuxian-wendao-julia",
        "xiuxian-wendao-parsers",
        "xiuxian-wendao-attachments",
        "xiuxian-wendao-runtime",
        "xiuxian-zhenfa",
        "xiuxian-db-store",
        "xiuxian-git-repo",
        "xiuxian-config-core",
        "xiuxian-ast",
        "axum =",
        "duckdb =",
        "comrak =",
        "notify =",
        "reqwest =",
    ];
    let leaked_dependencies = forbidden_dependencies
        .iter()
        .copied()
        .filter(|dependency| WEB_MANIFEST.contains(dependency))
        .collect::<Vec<_>>();

    assert!(
        leaked_dependencies.is_empty(),
        "xiuxian-wendao-web still owns non-transport dependencies: {leaked_dependencies:?}"
    );
}
