use super::support::*;

#[test]
fn docs_tool_service_context_extension_requires_presence() {
    let ctx = ZhenfaContext::default();
    assert!(ctx.docs_tool_service().is_err());
    assert!(resolve_docs_tool_runtime(&ctx).is_err());
}

#[test]
fn docs_tool_runtime_falls_back_to_docs_tool_service_extension() {
    let mut ctx = ZhenfaContext::default();
    let _ = ctx.insert_extension(DocsToolService::from_project_root(".", TEST_REPO_ID));

    assert!(resolve_docs_tool_runtime(&ctx).is_ok());
}
