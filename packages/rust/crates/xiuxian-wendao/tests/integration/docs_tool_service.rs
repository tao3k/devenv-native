//! Integration tests for the crate-local docs capability facade.
#![cfg(feature = "julia")]

use std::fs;
use std::path::Path;

use crate::support::linked_parser_summary::linked_modelica_parser_summary_base_url;
use crate::support::repo_intelligence::create_sample_modelica_repo;
use crate::support::wendao_command;
use xiuxian_wendao::analyzers::{
    DocsNavigationOptions, DocsPageIndexTreeResult, DocsToolService, ProjectedPageIndexNode,
    ProjectedPageIndexTree, ProjectionPageKind,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const MODELICA_DOCS_TOOL_PAGE_ID: &str = "repo:modelica-docs-tool:projection:reference:symbol:repo:modelica-docs-tool:symbol:Projectionica.Controllers.PI";
const MODELICA_DOCS_CLI_PAGE_ID: &str = "repo:modelica-docs-cli:projection:reference:symbol:repo:modelica-docs-cli:symbol:Projectionica.Controllers.PI";

fn write_modelica_docs_config(
    config_path: &Path,
    repo_id: &str,
    repo_dir: &Path,
    parser_summary_base_url: Option<&str>,
) -> TestResult {
    let plugin = parser_summary_base_url.map_or_else(
        || "\"modelica\"".to_string(),
        |base_url| {
            format!(
                "{{ id = \"modelica\", parser_summary_transport = {{ base_url = \"{base_url}\", file_summary = {{ schema_version = \"v3\" }} }} }}"
            )
        },
    );
    fs::write(
        config_path,
        format!(
            r#"[link_graph.projects.{repo_id}]
root = "{}"
plugins = [{plugin}]
"#,
            repo_dir.display()
        ),
    )?;
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn docs_tool_service_opens_outline_tree_search_navigation_node_and_context() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-tool-service.wendao.toml");
    write_modelica_docs_config(&config_path, "modelica-docs-tool", &repo_dir, None)?;

    let service = DocsToolService::from_project_root(temp.path(), "modelica-docs-tool")
        .with_optional_config_path(Some(config_path.clone()));

    let page = service.get_document(MODELICA_DOCS_TOOL_PAGE_ID)?;
    let catalog = service.get_page_index()?;
    let structure = service.get_page_index_tree(MODELICA_DOCS_TOOL_PAGE_ID)?;
    let outline = service.get_page_index_outline(MODELICA_DOCS_TOOL_PAGE_ID)?;
    let node_id = anchors_node_id(&structure)?;
    let node = service.get_document_node(MODELICA_DOCS_TOOL_PAGE_ID, &node_id)?;
    let node_range = anchors_line_range(&service, MODELICA_DOCS_TOOL_PAGE_ID, &node_id)?;
    let segment =
        service.get_document_segment(MODELICA_DOCS_TOOL_PAGE_ID, node_range.0, node_range.1)?;
    let search = service.search_page_index("anchors", Some(ProjectionPageKind::Reference), 3)?;
    let toc = service.get_toc_documents()?;
    let navigation = service.get_navigation_with_options(
        MODELICA_DOCS_TOOL_PAGE_ID,
        DocsNavigationOptions {
            node_id: None,
            family_kind: None,
            related_limit: 2,
            family_limit: 0,
        },
    )?;
    let context = service.get_retrieval_context(MODELICA_DOCS_TOOL_PAGE_ID, None)?;

    assert_eq!(page.page.title, "Projectionica.Controllers.PI");
    assert_eq!(catalog.repo_id, "modelica-docs-tool");
    assert!(
        catalog
            .trees
            .iter()
            .any(|tree| tree.page_id == MODELICA_DOCS_TOOL_PAGE_ID),
        "repo-scoped page-index should include the requested page"
    );
    assert_eq!(
        catalog
            .trees
            .iter()
            .find(|tree| tree.page_id == MODELICA_DOCS_TOOL_PAGE_ID)
            .and_then(|tree| tree.roots.first())
            .map(|node| node.text.as_str()),
        Some("")
    );
    assert_eq!(
        structure.tree.as_ref().map(|tree| tree.title.as_str()),
        Some("Projectionica.Controllers.PI")
    );
    assert_eq!(
        outline
            .tree
            .as_ref()
            .and_then(|tree| tree.roots.first())
            .map(|node| node.text.as_str()),
        Some("")
    );
    assert_eq!(
        node.hit.as_ref().map(|hit| hit.node_title.as_str()),
        Some("Anchors")
    );
    assert_eq!(segment.page_id, MODELICA_DOCS_TOOL_PAGE_ID);
    assert_eq!(segment.line_range, node_range);
    assert!(segment.content.contains("Anchors"));
    assert_eq!(search.repo_id, "modelica-docs-tool");
    assert_eq!(
        search.hits.first().map(|hit| hit.node_title.as_str()),
        Some("Anchors")
    );
    assert_eq!(toc.repo_id, "modelica-docs-tool");
    assert!(
        toc.documents
            .iter()
            .any(|document| document.page_id == MODELICA_DOCS_TOOL_PAGE_ID),
        "repo-scoped TOC documents should include the requested page"
    );
    assert_eq!(
        navigation
            .center
            .as_ref()
            .map(|center| center.page.title.as_str()),
        Some("Projectionica.Controllers.PI")
    );
    assert!(
        navigation.tree.is_some(),
        "navigation should include the page tree"
    );
    assert_eq!(context.center.page.title, "Projectionica.Controllers.PI");
    assert!(
        context.node_context.is_none(),
        "retrieval context should stay page-scoped when node_id is absent"
    );
    assert!(
        context.related_pages.len() <= 5,
        "default retrieval context limit should remain bounded"
    );
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn cli_docs_page_index_outline_returns_text_free_tree_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-docs-tree-outline-cli.wendao.toml");
    let parser_summary_base_url = linked_modelica_parser_summary_base_url()?;
    write_modelica_docs_config(
        &config_path,
        "modelica-docs-cli",
        &repo_dir,
        Some(parser_summary_base_url.as_str()),
    )?;

    let output = wendao_command()
        .current_dir(temp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("docs")
        .arg("tree-outline")
        .arg("--repo")
        .arg("modelica-docs-cli")
        .arg("--page-id")
        .arg(MODELICA_DOCS_CLI_PAGE_ID)
        .output()?;

    assert!(output.status.success(), "{output:?}");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(payload["repo_id"], "modelica-docs-cli");
    assert_eq!(payload["tree"]["page_id"], MODELICA_DOCS_CLI_PAGE_ID);
    assert_eq!(payload["tree"]["roots"][0]["text"], "");
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn cli_docs_page_index_returns_text_free_trees_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-page-index-cli.wendao.toml");
    let parser_summary_base_url = linked_modelica_parser_summary_base_url()?;
    write_modelica_docs_config(
        &config_path,
        "modelica-docs-cli",
        &repo_dir,
        Some(parser_summary_base_url.as_str()),
    )?;

    let output = wendao_command()
        .current_dir(temp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("docs")
        .arg("page-index")
        .arg("--repo")
        .arg("modelica-docs-cli")
        .output()?;

    assert!(output.status.success(), "{output:?}");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(payload["repo_id"], "modelica-docs-cli");
    let trees = payload["trees"]
        .as_array()
        .ok_or("expected serialized page-index trees")?;
    let target_tree = trees
        .iter()
        .find(|tree| tree["page_id"] == MODELICA_DOCS_CLI_PAGE_ID)
        .ok_or("expected requested page in page-index")?;
    assert_eq!(target_tree["roots"][0]["text"], "");
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn cli_docs_segment_returns_serialized_segment_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-segment-cli.wendao.toml");
    let parser_summary_base_url = linked_modelica_parser_summary_base_url()?;
    write_modelica_docs_config(
        &config_path,
        "modelica-docs-cli",
        &repo_dir,
        Some(parser_summary_base_url.as_str()),
    )?;
    let service = DocsToolService::from_project_root(temp.path(), "modelica-docs-cli")
        .with_optional_config_path(Some(config_path.clone()));
    let structure = service.get_page_index_tree(MODELICA_DOCS_CLI_PAGE_ID)?;
    let node_id = anchors_node_id(&structure)?;
    let node = service.get_document_node(MODELICA_DOCS_CLI_PAGE_ID, &node_id)?;
    let line_range = node
        .hit
        .as_ref()
        .map(|hit| hit.line_range)
        .ok_or("expected node hit for segment reopen")?;

    let output = wendao_command()
        .current_dir(temp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("docs")
        .arg("segment")
        .arg("--repo")
        .arg("modelica-docs-cli")
        .arg("--page-id")
        .arg(MODELICA_DOCS_CLI_PAGE_ID)
        .arg("--line-start")
        .arg(line_range.0.to_string())
        .arg("--line-end")
        .arg(line_range.1.to_string())
        .output()?;

    assert!(output.status.success(), "{output:?}");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(payload["repo_id"], "modelica-docs-cli");
    assert_eq!(payload["page_id"], MODELICA_DOCS_CLI_PAGE_ID);
    assert_eq!(payload["line_range"][0], line_range.0);
    assert_eq!(payload["line_range"][1], line_range.1);
    assert!(
        payload["content"]
            .as_str()
            .is_some_and(|content| content.contains("Anchors")),
        "expected serialized segment content"
    );
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn cli_docs_search_page_index_returns_serialized_hits_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-docs-search-structure-cli.wendao.toml");
    let parser_summary_base_url = linked_modelica_parser_summary_base_url()?;
    write_modelica_docs_config(
        &config_path,
        "modelica-docs-cli",
        &repo_dir,
        Some(parser_summary_base_url.as_str()),
    )?;
    let service = DocsToolService::from_project_root(temp.path(), "modelica-docs-cli")
        .with_optional_config_path(Some(config_path.clone()));
    let expected = service.search_page_index("anchors", Some(ProjectionPageKind::Reference), 3)?;
    let first_hit = expected
        .hits
        .first()
        .ok_or("expected at least one page-index search hit")?;

    let output = wendao_command()
        .current_dir(temp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("docs")
        .arg("search-structure")
        .arg("--repo")
        .arg("modelica-docs-cli")
        .arg("--query")
        .arg("anchors")
        .arg("--kind")
        .arg("reference")
        .arg("--limit")
        .arg("3")
        .output()?;

    assert!(output.status.success(), "{output:?}");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(payload["repo_id"], "modelica-docs-cli");
    assert_eq!(payload["hits"][0]["page_id"], first_hit.page_id);
    assert_eq!(payload["hits"][0]["node_id"], first_hit.node_id);
    assert_eq!(payload["hits"][0]["node_title"], first_hit.node_title);
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn cli_docs_node_returns_serialized_node_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-node-cli.wendao.toml");
    let parser_summary_base_url = linked_modelica_parser_summary_base_url()?;
    write_modelica_docs_config(
        &config_path,
        "modelica-docs-cli",
        &repo_dir,
        Some(parser_summary_base_url.as_str()),
    )?;

    let service = DocsToolService::from_project_root(temp.path(), "modelica-docs-cli")
        .with_optional_config_path(Some(config_path.clone()));
    let structure = service.get_page_index_tree(MODELICA_DOCS_CLI_PAGE_ID)?;
    let node_id = anchors_node_id(&structure)?;

    let output = wendao_command()
        .current_dir(temp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("docs")
        .arg("node")
        .arg("--repo")
        .arg("modelica-docs-cli")
        .arg("--page-id")
        .arg(MODELICA_DOCS_CLI_PAGE_ID)
        .arg("--node-id")
        .arg(&node_id)
        .output()?;

    assert!(output.status.success(), "{output:?}");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(payload["repo_id"], "modelica-docs-cli");
    assert_eq!(payload["hit"]["page_id"], MODELICA_DOCS_CLI_PAGE_ID);
    assert_eq!(payload["hit"]["node_id"], node_id);
    assert_eq!(payload["hit"]["node_title"], "Anchors");
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn cli_docs_page_returns_serialized_page_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-cli.wendao.toml");
    let parser_summary_base_url = linked_modelica_parser_summary_base_url()?;
    write_modelica_docs_config(
        &config_path,
        "modelica-docs-cli",
        &repo_dir,
        Some(parser_summary_base_url.as_str()),
    )?;

    let output = wendao_command()
        .current_dir(temp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("docs")
        .arg("page")
        .arg("--repo")
        .arg("modelica-docs-cli")
        .arg("--page-id")
        .arg(MODELICA_DOCS_CLI_PAGE_ID)
        .output()?;

    assert!(output.status.success(), "{output:?}");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(payload["page"]["title"], "Projectionica.Controllers.PI");
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn cli_docs_toc_returns_serialized_toc_payload() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-toc-cli.wendao.toml");
    let parser_summary_base_url = linked_modelica_parser_summary_base_url()?;
    write_modelica_docs_config(
        &config_path,
        "modelica-docs-cli",
        &repo_dir,
        Some(parser_summary_base_url.as_str()),
    )?;

    let output = wendao_command()
        .current_dir(temp.path())
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("docs")
        .arg("toc")
        .arg("--repo")
        .arg("modelica-docs-cli")
        .output()?;

    assert!(output.status.success(), "{output:?}");
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(payload["repo_id"], "modelica-docs-cli");
    assert!(
        payload["documents"]
            .as_array()
            .is_some_and(|documents| !documents.is_empty()),
        "expected serialized TOC documents"
    );
    Ok(())
}

fn find_node_id(nodes: &[ProjectedPageIndexNode], title: &str) -> Option<String> {
    for node in nodes {
        if node.title == title {
            return Some(node.node_id.clone());
        }
        if let Some(node_id) = find_node_id(node.children.as_slice(), title) {
            return Some(node_id);
        }
    }
    None
}

fn tree_roots(tree: Option<&ProjectedPageIndexTree>) -> &[ProjectedPageIndexNode] {
    tree.map_or(&[], |tree| tree.roots.as_slice())
}

fn anchors_node_id(
    structure: &DocsPageIndexTreeResult,
) -> Result<String, Box<dyn std::error::Error>> {
    find_node_id(tree_roots(structure.tree.as_ref()), "Anchors")
        .ok_or_else(|| "expected a projected page-index node titled `Anchors`".into())
}

fn anchors_line_range(
    service: &DocsToolService,
    page_id: &str,
    node_id: &str,
) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    service
        .get_document_node(page_id, node_id)?
        .hit
        .as_ref()
        .map(|hit| hit.line_range)
        .ok_or_else(|| "expected node hit for segment reopen".into())
}
