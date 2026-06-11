use std::collections::BTreeMap;
use std::fs;
use std::io::Error as IoError;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use serde::Serialize;
use xiuxian_julia_core::integration_support::{
    JuliaServiceGuard, spawn_wendaosearch_all_parser_summary_service,
};
use xiuxian_wendao::analyzers::{
    DocRecord, ModuleRecord, RelationKind, RelationRecord, RepoSymbolKind,
    RepositoryAnalysisOutput, RepositoryRecord, SymbolRecord, analyze_repository_from_config,
};

use super::repo_fixture;
use super::repo_parser_summary;
use repo_parser_summary::{FakeParserSummaryServiceGuard, spawn_fake_julia_parser_summary_service};

pub type TestResultPath = repo_fixture::TestResultPath;

const WENDAO_INTELLIGENCE_FORCE_FAKE_PARSER_SUMMARY: &str =
    "WENDAO_INTELLIGENCE_FORCE_FAKE_PARSER_SUMMARY";
const WENDAO_INTELLIGENCE_FORCE_REAL_PARSER_SUMMARY: &str =
    "WENDAO_INTELLIGENCE_FORCE_REAL_PARSER_SUMMARY";

struct RepoIntelligenceParserSummaryService {
    base_url: String,
    _guard: Mutex<RepoIntelligenceParserSummaryGuard>,
}

enum RepoIntelligenceParserSummaryGuard {
    Real {
        _guard: JuliaServiceGuard,
    },
    Fake {
        _guard: FakeParserSummaryServiceGuard,
    },
}

static REPO_INTELLIGENCE_PARSER_SUMMARY_SERVICE: OnceLock<
    Result<RepoIntelligenceParserSummaryService, String>,
> = OnceLock::new();
static REPO_ANALYSIS_CACHE: OnceLock<
    Mutex<BTreeMap<String, Result<RepositoryAnalysisOutput, String>>>,
> = OnceLock::new();
static REPO_INTELLIGENCE_PARSER_SUMMARY_TEST_FAKE: OnceLock<bool> = OnceLock::new();

fn preserve_fast_fake_parser_summary_for_tests() -> bool {
    *REPO_INTELLIGENCE_PARSER_SUMMARY_TEST_FAKE.get_or_init(|| {
        if std::env::var_os(WENDAO_INTELLIGENCE_FORCE_REAL_PARSER_SUMMARY).is_some() {
            return false;
        }
        if std::env::var_os(WENDAO_INTELLIGENCE_FORCE_FAKE_PARSER_SUMMARY).is_some() {
            return true;
        }

        true
    })
}

fn io_error_to_box(error: impl std::fmt::Display) -> Box<dyn std::error::Error> {
    Box::new(IoError::other(error.to_string()))
}

fn repo_analysis_cache()
-> &'static Mutex<BTreeMap<String, Result<RepositoryAnalysisOutput, String>>> {
    REPO_ANALYSIS_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn cache_key_for_analysis(repo_id: &str, config_path: Option<&Path>, cwd: &Path) -> String {
    let cache_path = config_path
        .and_then(|path| path.canonicalize().ok())
        .or_else(|| cwd.canonicalize().ok())
        .unwrap_or_else(|| {
            config_path
                .map(|path| path.to_path_buf())
                .unwrap_or_else(|| cwd.to_path_buf())
        });
    format!(
        "repo-analysis|repo:{}|config:{}",
        repo_id,
        cache_path.display(),
    )
}

pub fn analyze_repository_from_config_cached(
    repo_id: &str,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepositoryAnalysisOutput, Box<dyn std::error::Error>> {
    preserve_fast_fake_parser_summary_for_tests();
    let mut cache = repo_analysis_cache()
        .lock()
        .map_err(|error| io_error_to_box(format!("repo analysis cache lock failed: {error}")))?;
    let key = cache_key_for_analysis(repo_id, config_path, cwd);
    if let Some(cached) = cache.get(&key) {
        return cached
            .as_ref()
            .map(|value| value.clone())
            .map_err(|error| io_error_to_box(error.clone()));
    }

    let output = analyze_repository_from_config(repo_id, config_path, cwd)
        .map_err(|error| io_error_to_box(error.to_string()))?;
    cache.insert(key, Ok(output.clone()));

    Ok(output)
}

pub fn create_sample_julia_repo(
    base: &Path,
    package_name: &str,
    expected_root: bool,
) -> TestResultPath {
    repo_fixture::create_sample_julia_repo(base, package_name, expected_root)
}

pub fn create_cached_sample_julia_repo(
    fixture_name: &str,
    package_name: &str,
    expected_root: bool,
    extra_files: &[(&str, &str)],
) -> TestResultPath {
    repo_fixture::create_cached_sample_julia_repo(
        fixture_name,
        package_name,
        expected_root,
        extra_files,
    )
}

pub fn create_sample_modelica_repo(base: &Path, package_name: &str) -> TestResultPath {
    repo_fixture::create_sample_modelica_repo(base, package_name)
}

pub fn assert_repo_json_snapshot(name: &str, value: impl Serialize) {
    insta::with_settings!({
        snapshot_path => "../snapshots/repo_intelligence",
        prepend_module_to_snapshot => false,
        sort_maps => true,
    }, {
        insta::assert_json_snapshot!(name, value);
    });
}

pub fn write_repo_config(base: &Path, repo_dir: &Path, repo_id: &str) -> TestResultPath {
    preserve_fast_fake_parser_summary_for_tests();
    let parser_summary_base_url = repo_intelligence_parser_summary_base_url()?;
    let config_path = base.join(format!("{repo_id}.wendao.toml"));
    fs::write(
        &config_path,
        format!(
            r#"[link_graph.projects.{repo_id}]
root = "{}"
plugins = [
  {{ id = "julia-code-parser", parser_summary_transport = {{ base_url = "{parser_summary_base_url}", file_summary = {{ schema_version = "v3" }}, root_summary = {{ schema_version = "v3" }} }} }}
]
"#,
            repo_dir.display(),
        ),
    )?;
    Ok(config_path)
}

fn repo_intelligence_parser_summary_base_url() -> Result<String, Box<dyn std::error::Error>> {
    let service = REPO_INTELLIGENCE_PARSER_SUMMARY_SERVICE.get_or_init(|| {
        let (base_url, guard) = spawn_repo_intelligence_parser_summary_service()?;
        Ok(RepoIntelligenceParserSummaryService {
            base_url,
            _guard: Mutex::new(guard),
        })
    });
    match service {
        Ok(service) => Ok(service.base_url.clone()),
        Err(message) => {
            Err(Box::new(IoError::other(message.clone())) as Box<dyn std::error::Error>)
        }
    }
}

fn spawn_repo_intelligence_parser_summary_service()
-> Result<(String, RepoIntelligenceParserSummaryGuard), String> {
    if !real_repo_intelligence_parser_summary_service_is_available() {
        return spawn_fake_julia_parser_summary_service().map(|(base_url, guard)| {
            (
                base_url,
                RepoIntelligenceParserSummaryGuard::Fake { _guard: guard },
            )
        });
    }
    match spawn_real_repo_intelligence_parser_summary_service() {
        Ok((base_url, guard)) => Ok((
            base_url,
            RepoIntelligenceParserSummaryGuard::Real { _guard: guard },
        )),
        Err(_) => spawn_fake_julia_parser_summary_service().map(|(base_url, guard)| {
            (
                base_url,
                RepoIntelligenceParserSummaryGuard::Fake { _guard: guard },
            )
        }),
    }
}

fn real_repo_intelligence_parser_summary_service_is_available() -> bool {
    if preserve_fast_fake_parser_summary_for_tests() {
        return false;
    }
    if std::env::var_os(WENDAO_INTELLIGENCE_FORCE_FAKE_PARSER_SUMMARY).is_some() {
        return false;
    }
    std::env::var_os("WENDAO_CODE_PARSER_PACKAGE_DIR")
        .filter(|value| !value.is_empty())
        .is_some_and(|path| Path::new(&path).exists())
        || Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .is_some_and(|root| root.join(".data").join("WendaoCodeParser.jl").is_dir())
}

fn spawn_real_repo_intelligence_parser_summary_service()
-> Result<(String, JuliaServiceGuard), String> {
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| error.to_string())?;
        Ok::<(String, JuliaServiceGuard), String>(
            runtime.block_on(spawn_wendaosearch_all_parser_summary_service()),
        )
    })
    .join()
    .map_err(|_| "repo-intelligence parser-summary service thread panicked".to_string())?
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn sample_projection_analysis(repo_id: &str) -> RepositoryAnalysisOutput {
    let module_id = format!("repo:{repo_id}:module:ProjectionPkg");
    let solve_symbol_id = format!("repo:{repo_id}:symbol:ProjectionPkg.solve");
    let problem_symbol_id = format!("repo:{repo_id}:symbol:ProjectionPkg.Problem");
    let readme_doc_id = format!("repo:{repo_id}:doc:README.md");
    let solve_doc_id = format!("repo:{repo_id}:doc:src/ProjectionPkg.jl#symbol:solve");
    let problem_doc_id = format!("repo:{repo_id}:doc:src/ProjectionPkg.jl#symbol:Problem");

    RepositoryAnalysisOutput {
        repository: Some(RepositoryRecord {
            repo_id: repo_id.to_string().into(),
            name: "ProjectionPkg".to_string(),
            path: format!("/virtual/repos/{repo_id}").into(),
            url: None,
            revision: Some("fixture".to_string()),
            version: Some("0.1.0".to_string()),
            uuid: None,
            dependencies: Vec::new(),
        }),
        modules: vec![ModuleRecord {
            repo_id: repo_id.to_string().into(),
            module_id: module_id.clone().into(),
            qualified_name: "ProjectionPkg".to_string(),
            path: "src/ProjectionPkg.jl".into(),
        }],
        symbols: vec![
            SymbolRecord {
                repo_id: repo_id.to_string().into(),
                symbol_id: solve_symbol_id.clone().into(),
                module_id: Some(module_id.clone().into()),
                name: "solve".to_string(),
                qualified_name: "ProjectionPkg.solve".to_string(),
                kind: RepoSymbolKind::Function,
                path: "src/ProjectionPkg.jl".into(),
                line_start: None,
                line_end: None,
                signature: Some("solve(problem::Problem)".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            },
            SymbolRecord {
                repo_id: repo_id.to_string().into(),
                symbol_id: problem_symbol_id.clone().into(),
                module_id: Some(module_id.clone().into()),
                name: "Problem".to_string(),
                qualified_name: "ProjectionPkg.Problem".to_string(),
                kind: RepoSymbolKind::Type,
                path: "src/ProjectionPkg.jl".into(),
                line_start: None,
                line_end: None,
                signature: Some("struct Problem".to_string()),
                audit_status: None,
                verification_state: None,
                attributes: BTreeMap::new(),
            },
        ],
        imports: Vec::new(),
        examples: Vec::new(),
        docs: vec![
            DocRecord {
                repo_id: repo_id.to_string().into(),
                doc_id: readme_doc_id.clone().into(),
                title: "README.md".to_string(),
                path: "README.md".into(),
                format: Some("md".to_string()),
                doc_target: None,
            },
            DocRecord {
                repo_id: repo_id.to_string().into(),
                doc_id: problem_doc_id.clone().into(),
                title: "Problem".to_string(),
                path: "src/ProjectionPkg.jl#symbol:Problem".into(),
                format: Some("julia_docstring".to_string()),
                doc_target: None,
            },
            DocRecord {
                repo_id: repo_id.to_string().into(),
                doc_id: solve_doc_id.clone().into(),
                title: "solve".to_string(),
                path: "src/ProjectionPkg.jl#symbol:solve".into(),
                format: Some("julia_docstring".to_string()),
                doc_target: None,
            },
        ],
        relations: vec![
            RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: readme_doc_id,
                target_id: module_id.clone(),
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: problem_doc_id,
                target_id: problem_symbol_id,
                kind: RelationKind::Documents,
            },
            RelationRecord {
                repo_id: repo_id.to_string().into(),
                source_id: solve_doc_id,
                target_id: solve_symbol_id,
                kind: RelationKind::Documents,
            },
        ],
        diagnostics: Vec::new(),
    }
}
