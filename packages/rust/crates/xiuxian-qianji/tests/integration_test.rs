//! Cargo entry point for dormant `xiuxian-qianji` integration suites.

xiuxian_testing::crate_test_policy_harness!();

#[path = "integration/executors_annotation.rs"]
mod executors_annotation;
#[path = "integration/executors_formal_audit.rs"]
mod executors_formal_audit;
#[path = "integration/llm_augmented_formal_audit/mod.rs"]
mod llm_augmented_formal_audit;
#[path = "integration/manifest_requires_llm.rs"]
mod manifest_requires_llm;
#[path = "integration/runtime_live_llm.rs"]
mod runtime_live_llm;
#[path = "integration/test_agenda_validation_pipeline.rs"]
mod test_agenda_validation_pipeline;
#[path = "integration/test_bootcamp_api.rs"]
mod test_bootcamp_api;
#[path = "integration/test_compiler_dispatch_routes/mod.rs"]
mod test_compiler_dispatch_routes;
#[path = "integration/test_compiler_dispatch_routes_llm.rs"]
mod test_compiler_dispatch_routes_llm;
#[path = "integration/test_consensus.rs"]
mod test_consensus;
#[path = "integration/test_context_isolation_and_concurrency.rs"]
mod test_context_isolation_and_concurrency;
#[path = "integration/test_formal_adversarial_audit.rs"]
mod test_formal_adversarial_audit;
#[path = "integration/test_invocation_call_mechanisms.rs"]
mod test_invocation_call_mechanisms;
#[path = "integration/test_layout_bpmn.rs"]
mod test_layout_bpmn;
#[path = "integration/test_memory_promotion_pipeline.rs"]
mod test_memory_promotion_pipeline;
#[path = "integration/test_probabilistic_routing.rs"]
mod test_probabilistic_routing;
#[path = "integration/test_qianji_master_research.rs"]
mod test_qianji_master_research;
#[path = "integration/test_qianji_precision_research.rs"]
mod test_qianji_precision_research;
#[path = "integration/test_qianji_qianhuan_binding.rs"]
mod test_qianji_qianhuan_binding;
#[path = "integration/test_qianji_trinity_integration.rs"]
mod test_qianji_trinity_integration;
#[path = "integration/test_qianji_yaml_orchestration.rs"]
mod test_qianji_yaml_orchestration;
#[path = "integration/test_scheduler_affinity_failover.rs"]
mod test_scheduler_affinity_failover;
#[path = "integration/test_scheduler_checkpoint.rs"]
mod test_scheduler_checkpoint;
#[path = "integration/test_scheduler_preflight.rs"]
mod test_scheduler_preflight;
#[path = "integration/test_schema_contracts.rs"]
mod test_schema_contracts;
#[path = "integration/test_smart_commit_integration.rs"]
mod test_smart_commit_integration;
#[path = "integration/test_swarm_discovery.rs"]
mod test_swarm_discovery;
#[path = "integration/test_swarm_orchestration.rs"]
mod test_swarm_orchestration;
#[path = "integration/test_wendao_ingester_mechanism.rs"]
mod test_wendao_ingester_mechanism;
#[path = "integration/test_wendao_refresh_mechanism.rs"]
mod test_wendao_refresh_mechanism;
#[path = "integration/test_wendao_sql_mechanism.rs"]
mod test_wendao_sql_mechanism;
#[path = "integration/test_write_file_mechanism.rs"]
mod test_write_file_mechanism;
