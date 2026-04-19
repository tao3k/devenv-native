use std::sync::Arc;

use xiuxian_zhenfa::{ZhenfaRegistry, ZhenfaTool};

use super::{
    WendaoDocsGetDocumentNodeTool, WendaoDocsGetDocumentSegmentTool, WendaoDocsGetDocumentTool,
    WendaoDocsGetNavigationTool, WendaoDocsGetPageIndexOutlineTool, WendaoDocsGetPageIndexTool,
    WendaoDocsGetPageIndexTreeTool, WendaoDocsGetRetrievalContextTool,
    WendaoDocsGetTocDocumentsTool, WendaoDocsSearchPageIndexTool, WendaoDocsSearchTool,
};

/// Register the docs-native Wendao tools into one zhenfa registry.
pub fn register_wendao_docs_native_tools(registry: &mut ZhenfaRegistry) {
    registry.register(Arc::new(WendaoDocsSearchTool) as Arc<dyn ZhenfaTool>);
    registry.register(Arc::new(WendaoDocsGetDocumentTool) as Arc<dyn ZhenfaTool>);
    registry.register(Arc::new(WendaoDocsGetPageIndexTreeTool) as Arc<dyn ZhenfaTool>);
    registry.register(Arc::new(WendaoDocsGetPageIndexOutlineTool) as Arc<dyn ZhenfaTool>);
    registry.register(Arc::new(WendaoDocsGetPageIndexTool) as Arc<dyn ZhenfaTool>);
    registry.register(Arc::new(WendaoDocsGetDocumentSegmentTool) as Arc<dyn ZhenfaTool>);
    registry.register(Arc::new(WendaoDocsSearchPageIndexTool) as Arc<dyn ZhenfaTool>);
    registry.register(Arc::new(WendaoDocsGetDocumentNodeTool) as Arc<dyn ZhenfaTool>);
    registry.register(Arc::new(WendaoDocsGetTocDocumentsTool) as Arc<dyn ZhenfaTool>);
    registry.register(Arc::new(WendaoDocsGetNavigationTool) as Arc<dyn ZhenfaTool>);
    registry.register(Arc::new(WendaoDocsGetRetrievalContextTool) as Arc<dyn ZhenfaTool>);
}

#[cfg(test)]
#[path = "../../../../tests/unit/zhenfa_router/native/docs/mod.rs"]
mod tests;
