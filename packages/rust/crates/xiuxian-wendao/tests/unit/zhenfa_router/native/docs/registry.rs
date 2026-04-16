use super::support::*;

#[test]
fn docs_native_tools_register_all_capabilities() {
    let mut registry = ZhenfaRegistry::new();
    register_wendao_docs_native_tools(&mut registry);

    assert_eq!(registry.len(), 10);
    assert!(registry.contains("wendao.docs.get_document"));
    assert!(registry.contains("wendao.docs.get_document_structure"));
    assert!(registry.contains("wendao.docs.get_document_structure_outline"));
    assert!(registry.contains("wendao.docs.get_document_structure_catalog"));
    assert!(registry.contains("wendao.docs.get_document_segment"));
    assert!(registry.contains("wendao.docs.search_document_structure"));
    assert!(registry.contains("wendao.docs.get_document_node"));
    assert!(registry.contains("wendao.docs.get_toc_documents"));
    assert!(registry.contains("wendao.docs.get_navigation"));
    assert!(registry.contains("wendao.docs.get_retrieval_context"));
}
