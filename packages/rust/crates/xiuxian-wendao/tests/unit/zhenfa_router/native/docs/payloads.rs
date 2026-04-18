use super::support::*;

#[test]
fn get_document_tool_returns_serialized_page_payload() -> TestResult {
    let ctx = docs_context_with_fake_runtime();
    let runtime = current_thread_runtime()?;
    let output = runtime.block_on(
        WendaoDocsGetDocumentTool.call_native(&ctx, json!({ "page_id": TEST_PAGE_ID })),
    )?;

    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["page"]["title"], "Projectionica.Controllers.PI");
    assert_eq!(payload["page"]["page_id"], TEST_PAGE_ID);
    Ok(())
}

#[test]
fn search_tool_returns_serialized_projected_page_payload() -> TestResult {
    let ctx = docs_context_with_fake_runtime();
    let runtime = current_thread_runtime()?;
    let output = runtime.block_on(
        WendaoDocsSearchTool.call_native(&ctx, json!({ "query": "solver", "limit": 4 })),
    )?;

    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["repo_id"], TEST_REPO_ID);
    assert_eq!(payload["pages"][0]["page_id"], TEST_PAGE_ID);
    assert_eq!(payload["pages"][0]["title"], "SOLVER");
    Ok(())
}

#[test]
fn get_toc_documents_tool_returns_serialized_page_index_documents_payload() -> TestResult {
    let ctx = docs_context_with_fake_runtime();
    let runtime = current_thread_runtime()?;
    let output = runtime.block_on(WendaoDocsGetTocDocumentsTool.call_native(&ctx, json!({})))?;

    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["repo_id"], TEST_REPO_ID);
    assert_eq!(payload["documents"][0]["page_id"], TEST_PAGE_ID);
    Ok(())
}

#[test]
fn get_document_node_tool_returns_serialized_node_payload() -> TestResult {
    let ctx = docs_context_with_fake_runtime();
    let runtime = current_thread_runtime()?;
    let output = runtime.block_on(
        WendaoDocsGetDocumentNodeTool
            .call_native(&ctx, json!({ "page_id": TEST_PAGE_ID, "node_id": "0007" })),
    )?;

    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["repo_id"], TEST_REPO_ID);
    assert_eq!(payload["hit"]["page_id"], TEST_PAGE_ID);
    assert_eq!(payload["hit"]["node_id"], "0007");
    assert_eq!(payload["hit"]["node_title"], "Anchors");
    Ok(())
}

#[test]
fn get_document_structure_outline_tool_returns_text_free_tree_payload() -> TestResult {
    let ctx = docs_context_with_fake_runtime();
    let runtime = current_thread_runtime()?;
    let output = runtime.block_on(
        WendaoDocsGetDocumentStructureOutlineTool
            .call_native(&ctx, json!({ "page_id": TEST_PAGE_ID })),
    )?;

    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["repo_id"], TEST_REPO_ID);
    assert_eq!(payload["tree"]["page_id"], TEST_PAGE_ID);
    assert_eq!(payload["tree"]["roots"][0]["text"], "");
    assert_eq!(payload["tree"]["roots"][0]["summary"], "Anchor summary");
    Ok(())
}

#[test]
fn get_document_structure_catalog_tool_returns_text_free_trees_payload() -> TestResult {
    let ctx = docs_context_with_fake_runtime();
    let runtime = current_thread_runtime()?;
    let output =
        runtime.block_on(WendaoDocsGetDocumentStructureCatalogTool.call_native(&ctx, json!({})))?;

    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["repo_id"], TEST_REPO_ID);
    assert_eq!(payload["trees"][0]["page_id"], TEST_PAGE_ID);
    assert_eq!(payload["trees"][0]["roots"][0]["text"], "");
    assert_eq!(payload["trees"][0]["roots"][0]["summary"], "Anchor summary");
    Ok(())
}

#[test]
fn get_document_segment_tool_returns_serialized_segment_payload() -> TestResult {
    let ctx = docs_context_with_fake_runtime();
    let runtime = current_thread_runtime()?;
    let output = runtime.block_on(WendaoDocsGetDocumentSegmentTool.call_native(
        &ctx,
        json!({ "page_id": TEST_PAGE_ID, "line_start": 12, "line_end": 18 }),
    ))?;

    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["repo_id"], TEST_REPO_ID);
    assert_eq!(payload["page_id"], TEST_PAGE_ID);
    assert_eq!(payload["line_range"][0], 12);
    assert_eq!(payload["line_range"][1], 18);
    assert_eq!(payload["content"], "## Anchors\nBody");
    Ok(())
}

#[test]
fn search_document_structure_tool_returns_serialized_hits_payload() -> TestResult {
    let ctx = docs_context_with_fake_runtime();
    let runtime = current_thread_runtime()?;
    let output = runtime.block_on(WendaoDocsSearchDocumentStructureTool.call_native(
        &ctx,
        json!({ "query": "anchors", "kind": "reference", "limit": 3 }),
    ))?;

    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["repo_id"], TEST_REPO_ID);
    assert_eq!(payload["hits"][0]["page_id"], TEST_PAGE_ID);
    assert_eq!(payload["hits"][0]["node_title"], "ANCHORS");
    assert_eq!(payload["hits"][0]["node_id"], "search:3");
    Ok(())
}
