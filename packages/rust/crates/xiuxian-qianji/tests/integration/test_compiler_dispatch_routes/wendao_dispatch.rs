use super::{build_compiler, manifests::*};

#[test]
fn compiler_dispatches_wendao_ingester_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(WENDAO_INGESTER_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_wendao_refresh_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(WENDAO_REFRESH_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_wendao_sql_discover_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(WENDAO_SQL_DISCOVER_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_wendao_sql_validate_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(WENDAO_SQL_VALIDATE_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}

#[test]
fn compiler_dispatches_wendao_sql_execute_via_leaf_lane() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let compiler = build_compiler(temp.path())?;
    let engine = compiler.compile(WENDAO_SQL_EXECUTE_MANIFEST)?;
    assert_eq!(engine.graph.node_count(), 1);
    Ok(())
}
