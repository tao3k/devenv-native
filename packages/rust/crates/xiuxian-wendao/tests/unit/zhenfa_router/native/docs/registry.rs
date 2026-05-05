use super::support::{ZhenfaRegistry, register_wendao_docs_native_tools};

#[test]
fn docs_native_tools_register_all_capabilities() {
    let mut registry = ZhenfaRegistry::new();
    register_wendao_docs_native_tools(&mut registry);

    assert_eq!(registry.len(), 11);
    assert!(registry.contains("wendao.docs.search"));
    assert!(registry.contains("wendao.docs.get_document"));
    assert!(registry.contains("wendao.docs.get_page_index_tree"));
    assert!(registry.contains("wendao.docs.get_page_index_outline"));
    assert!(registry.contains("wendao.docs.get_page_index"));
    assert!(registry.contains("wendao.docs.get_document_segment"));
    assert!(registry.contains("wendao.docs.search_page_index"));
    assert!(registry.contains("wendao.docs.get_document_node"));
    assert!(registry.contains("wendao.docs.get_toc_documents"));
    assert!(registry.contains("wendao.docs.get_navigation"));
    assert!(registry.contains("wendao.docs.get_retrieval_context"));
}
