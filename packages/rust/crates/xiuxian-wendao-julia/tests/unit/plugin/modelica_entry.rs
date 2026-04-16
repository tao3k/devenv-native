type TestResult = Result<(), Box<dyn std::error::Error>>;

use std::fs;
use std::path::Path;

use tempfile::TempDir;
use xiuxian_wendao_core::repo_intelligence::{
    AnalysisContext, RegisteredRepository, RepoIntelligencePlugin, RepoSourceFile,
    RepositoryPluginConfig,
};

use super::ModelicaRepoIntelligencePlugin;
use crate::julia_plugin_test_support::common::{
    ensure_linked_modelica_parser_summary_service, repo_root,
};

#[test]
#[serial_test::serial(modelica_live)]
fn analyze_file_emits_modelica_module_and_symbols() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "Controllers/PI.mo".to_string(),
            contents: "within Demo.Controllers;\nmodel PI\n  parameter Real k = 1;\n  parameter Real Ti = 0.1;\n  Real y;\nequation\n  y = k;\nend PI;\n".to_string(),
        },
    )?;

    assert!(
        output
            .modules
            .iter()
            .any(|module| module.path == "Controllers/PI.mo" && module.qualified_name == "PI")
    );
    assert!(
        output.symbols.iter().any(|symbol| {
            symbol.path == "Controllers/PI.mo"
                && symbol.qualified_name == "PI"
                && symbol.name == "PI"
                && symbol.module_id.as_deref() == Some("repo:demo:module:PI")
        }),
        "symbols: {:?}",
        output.symbols
    );
    let model = output
        .symbols
        .iter()
        .find(|symbol| symbol.name == "PI")
        .unwrap_or_else(|| panic!("symbols: {:?}", output.symbols));
    assert!(
        model
            .attributes
            .get("class_name")
            .is_some_and(|value| value == "PI"),
        "model attrs: {:?}",
        model.attributes
    );
    assert!(
        model
            .attributes
            .get("restriction")
            .is_some_and(|value| value == "model"),
        "model attrs: {:?}",
        model.attributes
    );
    assert!(
        model
            .attributes
            .get("top_level")
            .is_some_and(|value| value == "true"),
        "model attrs: {:?}",
        model.attributes
    );
    Ok(())
}

#[test]
fn analyze_file_supports_modelica_standard_library_package_via_process_managed_parser_summary()
-> TestResult {
    if std::env::var_os("RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST").is_none() {
        eprintln!("skipping process-managed Modelica analyze_file proof");
        return Ok(());
    }

    let source_path = repo_root().join(
        ".data/xiuxian-wendao/repo-intelligence/repos/github.com/modelica/ModelicaStandardLibrary/Modelica/Blocks/package.mo",
    );
    if !source_path.is_file() {
        eprintln!(
            "skipping process-managed Modelica analyze_file proof; missing {}",
            source_path.display()
        );
        return Ok(());
    }

    let tempdir = TempDir::new()?;
    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("mcl-live", tempdir.path()),
        &RepoSourceFile {
            path: "Modelica/Blocks/package.mo".to_string(),
            contents: fs::read_to_string(&source_path)?,
        },
    )?;

    assert!(
        output
            .modules
            .iter()
            .any(|module| module.path == "Modelica/Blocks/package.mo"
                && module.qualified_name == "Blocks"),
        "modules: {:?}",
        output.modules
    );
    assert!(
        output
            .symbols
            .iter()
            .any(|symbol| symbol.path == "Modelica/Blocks/package.mo" && symbol.name == "Init"),
        "symbols: {:?}",
        output.symbols
    );
    Ok(())
}

#[test]
#[serial_test::serial(modelica_live)]
fn analyze_file_supports_modelica_standard_library_leaf_via_nested_root_context() -> TestResult {
    if std::env::var_os("RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST").is_none() {
        eprintln!("skipping process-managed Modelica leaf analyze_file proof");
        return Ok(());
    }

    ensure_linked_modelica_parser_summary_service()?;
    let repository_root = repo_root().join(
        ".data/xiuxian-wendao/repo-intelligence/repos/github.com/modelica/ModelicaStandardLibrary",
    );
    let source_path = repository_root.join("Modelica/Clocked/Types/SolverMethod.mo");
    if !source_path.is_file() {
        eprintln!(
            "skipping process-managed Modelica leaf analyze_file proof; missing {}",
            source_path.display()
        );
        return Ok(());
    }

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("mcl", repository_root.as_path()),
        &RepoSourceFile {
            path: "Modelica/Clocked/Types/SolverMethod.mo".to_string(),
            contents: fs::read_to_string(&source_path)?,
        },
    )?;

    assert!(output.modules.is_empty(), "modules: {:?}", output.modules);
    assert!(
        output.symbols.iter().any(|symbol| {
            symbol.path == "Modelica/Clocked/Types/SolverMethod.mo"
                && symbol.name == "SolverMethod"
                && symbol.qualified_name == "Modelica.Clocked.Types.SolverMethod"
                && symbol.module_id.as_deref() == Some("repo:mcl:module:Modelica.Clocked.Types")
        }),
        "symbols: {:?}",
        output.symbols
    );
    assert!(output.imports.is_empty(), "imports: {:?}", output.imports);
    Ok(())
}

#[test]
#[serial_test::serial(modelica_live)]
fn analyze_file_uses_repository_module_context_for_safe_leaf_files() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "PI.mo".to_string(),
            contents: "within DemoLib;\nmodel PI\nend PI;\n".to_string(),
        },
    )?;

    assert!(output.modules.is_empty(), "modules: {:?}", output.modules);
    assert!(
        output.symbols.iter().any(|symbol| {
            symbol.path == "PI.mo"
                && symbol.qualified_name == "DemoLib.PI"
                && symbol.module_id.as_deref() == Some("repo:demo:module:DemoLib")
        }),
        "symbols: {:?}",
        output.symbols
    );
    Ok(())
}

#[test]
#[serial_test::serial(modelica_live)]
fn analyze_file_uses_repository_import_context_for_safe_leaf_files() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "PI.mo".to_string(),
            contents: "within DemoLib;\nmodel PI\n  import Modelica.Math;\nend PI;\n".to_string(),
        },
    )?;

    assert!(
        output.imports.iter().any(|import| {
            import.path == "PI.mo"
                && import.module_id == "repo:demo:module:DemoLib"
                && import.import_name == "Math"
                && import.target_package == "Modelica"
                && import.source_module == "Modelica.Math"
        }),
        "imports: {:?}",
        output.imports
    );
    Ok(())
}

#[test]
fn analyze_file_uses_repository_root_package_context_for_package_files() -> TestResult {
    let tempdir = TempDir::new()?;
    let contents = "within ;\npackage DemoLib\n  import Modelica.Math;\n  annotation(Documentation(info = \"doc\"));\nend DemoLib;\n";
    fs::write(tempdir.path().join("package.mo"), contents)?;

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "package.mo".to_string(),
            contents: contents.to_string(),
        },
    )?;

    assert!(
        output.modules.iter().any(|module| {
            module.path == "package.mo"
                && module.qualified_name == "DemoLib"
                && module.module_id == "repo:demo:module:DemoLib"
        }),
        "modules: {:?}",
        output.modules
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
        output
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.path == "package.mo" && diagnostic.severity == "info" }),
        "diagnostics: {:?}",
        output.diagnostics
    );
    Ok(())
}

#[test]
fn analyze_file_uses_repository_context_for_safe_nested_package_files() -> TestResult {
    let tempdir = TempDir::new()?;
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;
    fs::create_dir_all(tempdir.path().join("Blocks"))?;
    let contents = "within DemoLib;\npackage Blocks\n  import Modelica.Math;\n  annotation(Documentation(info = \"doc\"));\nend Blocks;\n";

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "Blocks/package.mo".to_string(),
            contents: contents.to_string(),
        },
    )?;

    assert!(
        output.modules.iter().any(|module| {
            module.path == "Blocks/package.mo"
                && module.qualified_name == "DemoLib.Blocks"
                && module.module_id == "repo:demo:module:DemoLib.Blocks"
        }),
        "modules: {:?}",
        output.modules
    );
    assert!(output.symbols.is_empty(), "symbols: {:?}", output.symbols);
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
    Ok(())
}

#[test]
#[serial_test::serial(modelica_live)]
fn analyze_file_preserves_nested_package_declarations_via_parser_summary_fallback() -> TestResult {
    ensure_linked_modelica_parser_summary_service()?;
    let tempdir = TempDir::new()?;
    fs::write(
        tempdir.path().join("package.mo"),
        "within ;\npackage DemoLib\nend DemoLib;\n",
    )?;

    let plugin = ModelicaRepoIntelligencePlugin;
    let output = plugin.analyze_file(
        &analysis_context("demo", tempdir.path()),
        &RepoSourceFile {
            path: "Blocks/package.mo".to_string(),
            contents: "within DemoLib;\npackage Blocks\n  model Init\n  end Init;\nend Blocks;\n"
                .to_string(),
        },
    )?;

    assert!(
        output
            .symbols
            .iter()
            .any(|symbol| symbol.path == "Blocks/package.mo" && symbol.name == "Init"),
        "symbols: {:?}",
        output.symbols
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
