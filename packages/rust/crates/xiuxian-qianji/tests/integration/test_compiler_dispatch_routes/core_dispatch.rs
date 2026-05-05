use super::build_compiler;
use super::manifests::{
    CALIBRATION_MANIFEST, COMMAND_MANIFEST, KNOWLEDGE_MANIFEST, MOCK_MANIFEST,
    ROUTER_INVALID_WEIGHT_MANIFEST, ROUTER_MANIFEST, ROUTER_SEMANTIC_GUARD_MANIFEST,
    SECURITY_SCAN_MANIFEST, SUSPEND_MANIFEST, UNKNOWN_TASK_MANIFEST, WRITE_FILE_MANIFEST,
};

#[test]
fn compiler_dispatches_knowledge_task_via_stateless_lane() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(KNOWLEDGE_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_command_task_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(COMMAND_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_write_file_task_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(WRITE_FILE_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_suspend_task_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(SUSPEND_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_router_task_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(ROUTER_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_semantic_guard_router_task_via_leaf_lane()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(ROUTER_SEMANTIC_GUARD_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_rejects_router_with_invalid_branch_weight() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let error = compiler
        .compile(ROUTER_INVALID_WEIGHT_MANIFEST)
        .err()
        .unwrap_or_else(|| panic!("router manifest should fail on invalid branch weight"));
    let message = error.to_string();
    assert!(message.contains("Router branch weight"));
    Ok(())
}

#[test]
fn compiler_dispatches_calibration_task_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(CALIBRATION_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_mock_task_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(MOCK_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_security_scan_task_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(SECURITY_SCAN_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_rejects_unknown_task_type_with_topology_error() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let error = compiler
        .compile(UNKNOWN_TASK_MANIFEST)
        .err()
        .unwrap_or_else(|| panic!("unknown task type manifest should fail"));
    let message = error.to_string();
    assert!(message.contains("Unknown task type: not_real_task"));
    Ok(())
}
