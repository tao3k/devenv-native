//! Linkage markers for offline `SearchStrategyFlow` audit entrypoints.

use std::path::Path;

use super::{code_inventory, registry};

pub(crate) fn link_search_strategy_flow_offline_audit_entrypoints() {
    let _: fn(&Path) -> Result<code_inventory::SearchStrategyFlowCodeInventoryAudit, String> =
        code_inventory::audit_search_strategy_flow_code_intelligence_inventory;
    let _: fn(&Path) -> Result<registry::SearchStrategyFlowRegistryAuthorityAudit, String> =
        registry::audit_search_strategy_flow_registry_authority;
}
