use super::build_compiler;
use super::manifests::{
    ANNOTATION_DERIVED_AFFINITY_MANIFEST, ANNOTATION_EXPLICIT_AFFINITY_MANIFEST,
    FORMAL_AUDIT_NATIVE_MANIFEST, FORMAL_AUDIT_NATIVE_WITH_MAX_RETRIES_MANIFEST,
};
use super::manifests::{FORMAL_AUDIT_LLM_MANIFEST, LLM_TASK_MANIFEST};

#[test]
fn compiler_dispatches_annotation_and_keeps_explicit_execution_affinity()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(ANNOTATION_EXPLICIT_AFFINITY_MANIFEST)?;

    assert_eq!(engine.graph.node_count(), 1);
    let node_index = engine
        .graph
        .node_indices()
        .next()
        .unwrap_or_else(|| panic!("compiled graph should contain one node"));
    let node = &engine.graph[node_index];
    assert_eq!(
        node.execution_affinity.agent_id.as_deref(),
        Some("agent-alpha")
    );
    assert_eq!(
        node.execution_affinity.role_class.as_deref(),
        Some("planner")
    );
    Ok(())
}

#[test]
fn compiler_dispatches_annotation_and_derives_role_class_from_persona_id()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(ANNOTATION_DERIVED_AFFINITY_MANIFEST)?;

    assert_eq!(engine.graph.node_count(), 1);
    let node_index = engine
        .graph
        .node_indices()
        .next()
        .unwrap_or_else(|| panic!("compiled graph should contain one node"));
    let node = &engine.graph[node_index];
    assert_eq!(node.execution_affinity.agent_id, None);
    assert_eq!(
        node.execution_affinity.role_class.as_deref(),
        Some("steward")
    );
    Ok(())
}

#[test]
fn compiler_dispatches_formal_audit_native_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(FORMAL_AUDIT_NATIVE_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_rejects_native_formal_audit_with_max_retries_without_llm_controller()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let error = compiler
        .compile(FORMAL_AUDIT_NATIVE_WITH_MAX_RETRIES_MANIFEST)
        .err()
        .unwrap_or_else(|| panic!("native formal_audit with max_retries should fail"));
    let message = error.to_string();
    assert!(message.contains("formal_audit.max_retries"));
    assert!(message.contains("native formal_audit"));
    Ok(())
}

#[test]
fn compiler_rejects_llm_augmented_formal_audit_without_llm_feature()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let error = compiler
        .compile(FORMAL_AUDIT_LLM_MANIFEST)
        .err()
        .unwrap_or_else(|| panic!("manifest should fail without llm feature"));
    let message = error.to_string();
    assert!(message.contains("Task type `formal_audit`"));
    assert!(message.contains("feature `llm`"));
    Ok(())
}

#[test]
fn compiler_rejects_llm_task_without_llm_feature() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let error = compiler
        .compile(LLM_TASK_MANIFEST)
        .err()
        .unwrap_or_else(|| panic!("llm task manifest should fail without llm feature"));
    let message = error.to_string();
    assert!(message.contains("Task type 'llm'"));
    assert!(message.contains("feature 'llm'"));
    Ok(())
}
