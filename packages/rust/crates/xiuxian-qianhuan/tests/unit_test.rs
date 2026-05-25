//! Canonical unit test harness for `xiuxian-qianhuan`.

#[path = "unit/contracts.rs"]
mod contracts;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/test_ccs_gating_integration.rs"]
mod test_ccs_gating_integration;
#[path = "unit/test_dynamic_template_loading.rs"]
mod test_dynamic_template_loading;
#[path = "unit/test_embedded_template_catalog.rs"]
mod test_embedded_template_catalog;
#[path = "unit/test_hot_reload_backend.rs"]
mod test_hot_reload_backend;
#[path = "unit/test_hot_reload_policy.rs"]
mod test_hot_reload_policy;
#[path = "unit/test_hot_reload_runtime.rs"]
mod test_hot_reload_runtime;
#[path = "unit/test_manifestation_manager.rs"]
mod test_manifestation_manager;
#[path = "unit/test_markdown_config_bridge.rs"]
mod test_markdown_config_bridge;
#[cfg(feature = "artifact-cache")]
#[path = "unit/test_prompt_context_artifacts.rs"]
mod test_prompt_context_artifacts;
#[path = "unit/test_thousand_faces.rs"]
mod test_thousand_faces;
#[path = "unit/test_window.rs"]
mod test_window;
#[path = "unit/test_xml_escape_hardening.rs"]
mod test_xml_escape_hardening;
#[path = "unit/test_zhenfa_native_tools.rs"]
mod test_zhenfa_native_tools;
#[path = "unit/test_zhenfa_router.rs"]
mod test_zhenfa_router;
#[path = "unit/unit_ccs_gating.rs"]
mod unit_ccs_gating;
#[path = "unit/unit_ccs_refinement.rs"]
mod unit_ccs_refinement;
#[path = "unit/unit_orchestrator.rs"]
mod unit_orchestrator;
#[path = "unit/unit_persona.rs"]
mod unit_persona;
#[path = "unit/unit_transmuter.rs"]
mod unit_transmuter;
#[path = "unit/unit_xml_validation.rs"]
mod unit_xml_validation;
