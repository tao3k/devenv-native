use std::{env, fs, path::PathBuf};

use super::{
    SearchStrategyFlowConfiguredMarkdownCorpusRow,
    audit_configured_search_strategy_flow_markdown_corpus,
    configured_search_strategy_flow_markdown_replay_families,
    configured_search_strategy_flow_markdown_replay_families_with_limit,
};

#[test]
fn configured_markdown_corpus_audit_follows_root_wendao_toml_imports()
-> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();
    fs::create_dir_all(root.join("assets/wendao"))?;
    fs::create_dir_all(root.join("docs"))?;
    fs::create_dir_all(root.join("docs/target"))?;
    fs::create_dir_all(root.join("semantic"))?;
    fs::write(
        root.join("wendao.toml"),
        r#"
imports = ["assets/wendao/imported.toml"]

[link_graph]
include_dirs = ["docs"]

[link_graph.projects.local]
plugins = ["julia"]
"#,
    )?;
    fs::write(
        root.join("assets/wendao/imported.toml"),
        r#"
[link_graph]
include_dirs = ["semantic"]

[link_graph.projects.remote]
plugins = ["julia"]
"#,
    )?;
    fs::write(
        root.join("docs/scenario.md"),
        "# Root Scenario\n\n## Precision Gate\n\nConfigured Markdown corpus.\n",
    )?;
    fs::write(
        root.join("docs/target/ignored.md"),
        "# Ignored Build Output\n\nThis must not enter the configured corpus audit.\n",
    )?;
    fs::write(
        root.join("semantic/working_knowledge.md"),
        "# Working Knowledge\n\nSemantic Markdown scenario.\n",
    )?;

    let audit = audit_configured_search_strategy_flow_markdown_corpus(root)?;
    assert_eq!(audit.config_surface, "root-wendao.toml");
    assert_eq!(audit.configured_project_count, 2);
    assert_eq!(audit.include_dir_count, 2);
    assert_eq!(audit.markdown_file_count, 2);
    assert_eq!(audit.heading_count, 3);
    assert_eq!(
        corpus_row(&audit.rows, "docs"),
        &SearchStrategyFlowConfiguredMarkdownCorpusRow {
            include_dir: "docs".to_owned(),
            markdown_file_count: 1,
            heading_count: 2,
        }
    );
    assert_eq!(
        corpus_row(&audit.rows, "semantic"),
        &SearchStrategyFlowConfiguredMarkdownCorpusRow {
            include_dir: "semantic".to_owned(),
            markdown_file_count: 1,
            heading_count: 1,
        }
    );
    Ok(())
}

#[test]
fn configured_markdown_replay_families_follow_root_wendao_toml_imports()
-> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path();
    fs::create_dir_all(root.join("assets/wendao"))?;
    fs::create_dir_all(root.join("docs"))?;
    fs::create_dir_all(root.join("semantic"))?;
    fs::write(
        root.join("wendao.toml"),
        r#"
imports = ["assets/wendao/imported.toml"]

[link_graph]
include_dirs = ["docs"]

[link_graph.projects.local]
plugins = ["julia"]
"#,
    )?;
    fs::write(
        root.join("assets/wendao/imported.toml"),
        r#"
[link_graph]
include_dirs = ["semantic"]

[link_graph.projects.remote]
plugins = ["julia"]
"#,
    )?;
    fs::write(
        root.join("docs/scenario.md"),
        "# Search Strategy Flow\n\n## Precision Gate\n\nConfigured Markdown corpus.\n",
    )?;
    fs::write(
        root.join("semantic/working_knowledge.md"),
        "# Working Knowledge\n\n## Evidence Ownership\n\nSearchStrategyFlow semantic proof.\n",
    )?;

    let families =
        configured_search_strategy_flow_markdown_replay_families(root, "SearchStrategyFlow proof")?;

    assert_eq!(families.len(), 2);
    assert_replay_family(&families, "docs", "docs/scenario.md");
    assert_replay_family(&families, "semantic", "semantic/working_knowledge.md");
    Ok(())
}

#[test]
fn configured_markdown_corpus_audit_covers_real_wendao_toml_surface()
-> Result<(), Box<dyn std::error::Error>> {
    let audit = audit_configured_search_strategy_flow_markdown_corpus(repository_root().as_path())?;

    assert_eq!(audit.config_surface, "root-wendao.toml");
    assert!(
        audit.configured_project_count >= 177,
        "root wendao.toml plus imports should expose the configured repo surface, got {}",
        audit.configured_project_count
    );
    assert!(audit.include_dir_count >= 5);
    assert!(
        audit.markdown_file_count >= 400,
        "configured local Markdown corpus is the primary real-scenario surface, got {}",
        audit.markdown_file_count
    );
    assert!(
        audit.heading_count >= audit.markdown_file_count,
        "configured Markdown corpus should expose heading anchors for strategy flow replay"
    );

    assert!(corpus_row(&audit.rows, "docs").markdown_file_count >= 100);
    assert!(corpus_row(&audit.rows, "semantic").markdown_file_count >= 10);
    assert!(
        corpus_row(&audit.rows, "packages/rust/crates/xiuxian-wendao").markdown_file_count >= 300
    );
    assert!(
        corpus_row(
            &audit.rows,
            "packages/python/wendao-knowledge-retrieval-benchmark"
        )
        .markdown_file_count
            >= 5
    );
    Ok(())
}

#[test]
fn configured_markdown_replay_families_cover_real_wendao_toml_surface()
-> Result<(), Box<dyn std::error::Error>> {
    let families = configured_search_strategy_flow_markdown_replay_families(
        repository_root().as_path(),
        "SearchStrategyFlow PageIndex LinkGraph validation",
    )?;

    assert!(families.len() >= 4);
    for include_dir in [
        "docs",
        "semantic",
        "packages/rust/crates/xiuxian-wendao",
        "packages/python/wendao-knowledge-retrieval-benchmark",
    ] {
        let family = replay_family(&families, include_dir);
        assert!(family.markdown_file_count > 0);
        assert!(family.heading_count >= family.markdown_file_count);
        assert_eq!(family.batch.source, "rust-markdown-headings");
        assert!((1..=12).contains(&family.batch.row_count));
        assert_eq!(family.batch.row_count, family.batch.tsv.lines().count());
        assert!(
            family
                .batch
                .tsv
                .lines()
                .all(|line| line.starts_with(include_dir)),
            "candidate paths must stay repository-relative for {include_dir}"
        );
    }
    Ok(())
}

#[test]
fn configured_markdown_replay_families_support_scaled_candidate_limits()
-> Result<(), Box<dyn std::error::Error>> {
    let limit = 24;
    let families = configured_search_strategy_flow_markdown_replay_families_with_limit(
        repository_root().as_path(),
        "SearchStrategyFlow PageIndex LinkGraph validation",
        Some(limit),
    )?;

    for include_dir in [
        "docs",
        "semantic",
        "packages/rust/crates/xiuxian-wendao",
        "packages/python/wendao-knowledge-retrieval-benchmark",
    ] {
        let family = replay_family(&families, include_dir);
        assert!(family.markdown_file_count > 0);
        assert!(family.heading_count >= family.markdown_file_count);
        assert_eq!(family.batch.source, "rust-markdown-headings");
        assert_eq!(family.batch.row_count, family.batch.tsv.lines().count());
        assert_eq!(
            family.batch.row_count,
            family.heading_count.min(limit),
            "scaled replay limit should use the configured family heading count for {include_dir}"
        );
    }
    Ok(())
}

fn corpus_row<'a>(
    rows: &'a [SearchStrategyFlowConfiguredMarkdownCorpusRow],
    include_dir: &str,
) -> &'a SearchStrategyFlowConfiguredMarkdownCorpusRow {
    rows.iter()
        .find(|row| row.include_dir == include_dir)
        .unwrap_or_else(|| panic!("configured Markdown corpus row `{include_dir}` should exist"))
}

fn assert_replay_family(
    families: &[super::SearchStrategyFlowConfiguredMarkdownReplayFamily],
    include_dir: &str,
    expected_path: &str,
) {
    let family = replay_family(families, include_dir);
    assert_eq!(family.markdown_file_count, 1);
    assert_eq!(family.batch.source, "rust-markdown-headings");
    assert!(family.batch.row_count > 0);
    assert!(family.batch.tsv.contains(expected_path));
}

fn replay_family<'a>(
    families: &'a [super::SearchStrategyFlowConfiguredMarkdownReplayFamily],
    include_dir: &str,
) -> &'a super::SearchStrategyFlowConfiguredMarkdownReplayFamily {
    families
        .iter()
        .find(|family| family.include_dir == include_dir)
        .unwrap_or_else(|| panic!("configured Markdown replay family `{include_dir}` should exist"))
}

fn repository_root() -> PathBuf {
    env::var_os("PRJ_ROOT").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(4)
                .unwrap_or_else(|| panic!("resolve repository root from Cargo manifest"))
                .to_path_buf()
        },
        PathBuf::from,
    )
}
