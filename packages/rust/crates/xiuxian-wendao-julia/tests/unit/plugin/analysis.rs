type TestResult = Result<(), Box<dyn std::error::Error>>;

use std::fs;
use std::path::Path;

use serde_json::json;
use tempfile::TempDir;
use xiuxian_wendao_core::repo_intelligence::RepositoryPluginConfig;
use xiuxian_wendao_core::repo_intelligence::{AnalysisContext, RegisteredRepository};

use super::{
    analyze_repository, load_modelica_repository_context_for_source, preflight_repository,
};
use crate::julia_plugin_test_support::common::ensure_linked_modelica_parser_summary_service;

#[test]
fn analyze_repository_keeps_top_level_package_paths() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("package.mo").as_path(),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Example.mo").as_path(),
        "within DemoLib;\nmodel Example\nend Example;\n",
    )?;

    let output = analyze_repository(&analysis_context("demo", tempdir.path()), tempdir.path())?;

    assert!(
        output
            .modules
            .iter()
            .any(|module| module.path == "package.mo" && module.qualified_name == "DemoLib")
    );
    assert!(
        output
            .symbols
            .iter()
            .any(|symbol| symbol.path == "Example.mo" && symbol.qualified_name == "DemoLib.Example")
    );
    assert_eq!(output.diagnostics[0].path, "package.mo");
    Ok(())
}

#[test]
fn analyze_repository_lexically_collects_safe_root_package_imports_and_docs() -> TestResult {
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("package.mo").as_path(),
        "within ;\npackage DemoLib\n  import SI = Modelica.Units.SI;\n  import Modelica.Math;\n  import Modelica.Math.*;\n  annotation(Documentation(info = \"doc\"));\nend DemoLib;\n",
    )?;

    let output = analyze_repository(&analysis_context("demo", tempdir.path()), tempdir.path())?;

    assert!(
        output
            .modules
            .iter()
            .any(|module| module.path == "package.mo" && module.qualified_name == "DemoLib"),
        "modules: {:?}",
        output.modules
    );
    assert!(output.symbols.is_empty(), "symbols: {:?}", output.symbols);
    assert!(
        output.imports.iter().any(|import| {
            import.path == "package.mo"
                && import.module_id == "repo:demo:module:DemoLib"
                && import.import_name == "SI"
                && import.target_package == "Modelica"
                && import.source_module == "Modelica.Units.SI"
        }),
        "imports: {:?}",
        output.imports
    );
    assert!(
        output.imports.iter().any(|import| {
            import.path == "package.mo"
                && import.module_id == "repo:demo:module:DemoLib"
                && import.import_name == "Math"
                && import.target_package == "Modelica"
                && import.source_module == "Modelica.Math"
        }),
        "imports: {:?}",
        output.imports
    );
    assert!(
        output
            .docs
            .iter()
            .any(|doc| doc.path == "package.mo#annotation.documentation"),
        "docs: {:?}",
        output.docs
    );
    assert!(
        output.relations.iter().any(|relation| {
            relation.source_id == "repo:demo:doc:package.mo#annotation.documentation"
                && relation.target_id == "repo:demo:module:DemoLib"
        }),
        "relations: {:?}",
        output.relations
    );
    Ok(())
}

#[test]
fn analyze_repository_preserves_root_package_nested_declarations_via_parser_summary_fallback()
-> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("package.mo").as_path(),
        "within ;\npackage DemoLib\n  model Controller\n  end Controller;\nend DemoLib;\n",
    )?;

    let output = analyze_repository(&analysis_context("demo", tempdir.path()), tempdir.path())?;

    assert!(
        output.symbols.iter().any(|symbol| {
            symbol.path == "package.mo" && symbol.qualified_name == "DemoLib.Controller"
        }),
        "symbols: {:?}",
        output.symbols
    );
    Ok(())
}

#[test]
fn analyze_repository_lexically_collects_safe_nested_package_imports_and_docs() -> TestResult {
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("package.mo").as_path(),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Blocks/package.mo").as_path(),
        "within DemoLib;\npackage Blocks\n  import Modelica.Math;\n  annotation(Documentation(info = \"doc\"));\nend Blocks;\n",
    )?;

    let output = analyze_repository(&analysis_context("demo", tempdir.path()), tempdir.path())?;

    assert!(
        output.modules.iter().any(|module| {
            module.path == "Blocks/package.mo"
                && module.qualified_name == "DemoLib.Blocks"
                && module.module_id == "repo:demo:module:DemoLib.Blocks"
        }),
        "modules: {:?}",
        output.modules
    );
    assert!(
        !output
            .symbols
            .iter()
            .any(|symbol| symbol.path == "Blocks/package.mo"),
        "symbols: {:?}",
        output.symbols
    );
    assert!(
        output.imports.iter().any(|import| {
            import.path == "Blocks/package.mo"
                && import.module_id == "repo:demo:module:DemoLib.Blocks"
                && import.import_name == "Math"
                && import.target_package == "Modelica"
                && import.source_module == "Modelica.Math"
        }),
        "imports: {:?}",
        output.imports
    );
    assert!(
        output
            .docs
            .iter()
            .any(|doc| doc.path == "Blocks/package.mo#annotation.documentation"),
        "docs: {:?}",
        output.docs
    );
    assert!(
        output.relations.iter().any(|relation| {
            relation.source_id == "repo:demo:doc:Blocks/package.mo#annotation.documentation"
                && relation.target_id == "repo:demo:module:DemoLib.Blocks"
        }),
        "relations: {:?}",
        output.relations
    );
    Ok(())
}

#[test]
fn analyze_repository_preserves_nested_package_declarations_via_parser_summary_fallback()
-> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("package.mo").as_path(),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Blocks/package.mo").as_path(),
        "within DemoLib;\npackage Blocks\n  model Controller\n  end Controller;\nend Blocks;\n",
    )?;

    let output = analyze_repository(&analysis_context("demo", tempdir.path()), tempdir.path())?;

    assert!(
        output.symbols.iter().any(|symbol| {
            symbol.path == "Blocks/package.mo"
                && symbol.qualified_name == "DemoLib.Blocks.Controller"
        }),
        "symbols: {:?}",
        output.symbols
    );
    Ok(())
}

#[test]
fn analyze_repository_supports_dominant_nested_root_package() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("Modelica/package.mo").as_path(),
        "within ;\npackage Modelica\nend Modelica;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Modelica/Blocks.mo").as_path(),
        "within Modelica;\nmodel Blocks\nend Blocks;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("ModelicaServices/package.mo").as_path(),
        "within ;\npackage ModelicaServices\nend ModelicaServices;\n",
    )?;

    preflight_repository(&analysis_context("mcl", tempdir.path()), tempdir.path())?;
    let output = analyze_repository(&analysis_context("mcl", tempdir.path()), tempdir.path())?;

    assert!(
        output
            .modules
            .iter()
            .any(|module| module.path == "Modelica/package.mo"
                && module.qualified_name == "Modelica")
    );
    assert!(
        output
            .symbols
            .iter()
            .any(|symbol| symbol.path == "Modelica/Blocks.mo"
                && symbol.qualified_name == "Modelica.Blocks")
    );
    assert_eq!(output.diagnostics[0].path, "Modelica/package.mo");
    Ok(())
}

#[test]
fn load_modelica_repository_context_prefers_source_hint_for_nested_root_package() -> TestResult {
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("Modelica/package.mo").as_path(),
        "within ;\npackage Modelica\nend Modelica;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Modelica/Blocks.mo").as_path(),
        "within Modelica;\nmodel Blocks\nend Blocks;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("ModelicaServices/package.mo").as_path(),
        "within ;\npackage ModelicaServices\nend ModelicaServices;\n",
    )?;

    let context = load_modelica_repository_context_for_source(
        &analysis_context("mcl", tempdir.path()).repository,
        tempdir.path(),
        "Modelica/Blocks.mo",
    )?;

    assert_eq!(context.package_root, tempdir.path().join("Modelica"));
    assert_eq!(context.root_package_name, "Modelica");
    assert_eq!(context.path_prefix.as_deref(), Some("Modelica"));
    Ok(())
}

#[test]
fn analyze_repository_preserves_import_backed_package_attributes() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    write_modelica_file(
        tempdir.path().join("Modelica/package.mo").as_path(),
        "within ;\npackage Modelica\nend Modelica;\n",
    )?;
    write_modelica_file(
        tempdir.path().join("Modelica/Blocks/package.mo").as_path(),
        "within Modelica;\npackage Blocks\n  import SI = Modelica.Units.SI;\n  import Modelica.Math;\n  import Modelica.Math.*;\nend Blocks;\n",
    )?;

    let output = analyze_repository(&analysis_context("mcl", tempdir.path()), tempdir.path())?;
    let payload = output
        .imports
        .iter()
        .map(|import| {
            json!({
                "module_id": import.module_id,
                "import_name": import.import_name,
                "target_package": import.target_package,
                "source_module": import.source_module,
                "kind": format!("{:?}", import.kind),
                "line_start": import.line_start,
                "resolved_id": import.resolved_id,
                "attributes": import.attributes,
            })
        })
        .collect::<Vec<_>>();

    insta::assert_json_snapshot!(
        "analyze_repository_preserves_import_backed_package_attributes",
        payload
    );
    Ok(())
}

fn analysis_context(repo_id: &str, repository_root: &Path) -> AnalysisContext {
    AnalysisContext {
        repository: RegisteredRepository {
            id: repo_id.to_string(),
            plugins: vec![RepositoryPluginConfig::Id("modelica".to_string())],
            ..RegisteredRepository::default()
        },
        repository_root: repository_root.to_path_buf(),
    }
}

fn write_modelica_file(path: &Path, contents: &str) -> TestResult {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}
