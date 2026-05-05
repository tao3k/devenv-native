use super::build_compiler;
use super::manifests::{
    CLI_CALL_MANIFEST, CLI_CALL_UNKNOWN_FLAG_MANIFEST, HTTP_CALL_INVALID_PATH_MANIFEST,
    HTTP_CALL_MANIFEST,
};

#[test]
fn compiler_dispatches_http_call_task_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(HTTP_CALL_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_cli_call_task_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(CLI_CALL_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_rejects_http_call_when_contract_path_drifts() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let Err(error) = compiler.compile(HTTP_CALL_INVALID_PATH_MANIFEST) else {
        panic!("invalid HTTP path should fail contract validation");
    };
    assert!(error.to_string().contains("/api/docs/navigation"));
    Ok(())
}

#[test]
fn compiler_rejects_cli_call_when_flag_is_unknown() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let Err(error) = compiler.compile(CLI_CALL_UNKNOWN_FLAG_MANIFEST) else {
        panic!("unknown CLI flag should fail contract validation");
    };
    assert!(error.to_string().contains("--nope"));
    Ok(())
}
