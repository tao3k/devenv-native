use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioConfig {
    pub scenario: ScenarioMeta,
    pub input: InputConfig,
    #[serde(default)]
    pub expected: Option<ExpectedConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InputConfig {
    #[serde(rename = "type")]
    pub input_type: String,
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExpectedConfig {
    #[serde(rename = "type", default)]
    pub output_type: String,
    #[serde(default)]
    pub files: Vec<String>,
}

pub struct Scenario {
    pub dir: PathBuf,
    pub config: ScenarioConfig,
}

impl Scenario {
    pub fn load(dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        let config_path = dir.join("scenario.toml");
        let content = fs::read_to_string(&config_path)?;
        let config = toml::from_str::<ScenarioConfig>(&content)?;
        Ok(Self { dir, config })
    }

    pub fn id(&self) -> &str {
        &self.config.scenario.id
    }

    pub fn name(&self) -> &str {
        &self.config.scenario.name
    }

    pub fn category(&self) -> &str {
        &self.config.scenario.category
    }

    pub fn input_path(&self) -> Option<PathBuf> {
        self.config
            .input
            .paths
            .first()
            .map(|path| self.dir.join(path))
    }

    pub fn has_input(&self) -> bool {
        !self.config.input.paths.is_empty()
    }
}

pub trait ScenarioRunner: Send + Sync {
    fn category(&self) -> &str;

    fn additional_categories(&self) -> Vec<&str> {
        vec![]
    }

    fn handles_category(&self, category: &str) -> bool {
        self.category() == category || self.additional_categories().contains(&category)
    }

    fn run(&self, scenario: &Scenario, temp_dir: &Path) -> Result<Value, Box<dyn Error>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioSnapshotPolicy {
    sort_maps: bool,
    include: ScenarioSnapshotInclusions,
    redactions: Vec<ScenarioSnapshotRedaction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScenarioSnapshotInclusions {
    description: bool,
    info: bool,
    input_file: bool,
}

impl ScenarioSnapshotInclusions {
    fn none() -> Self {
        Self {
            description: false,
            info: false,
            input_file: false,
        }
    }

    fn all() -> Self {
        Self {
            description: true,
            info: true,
            input_file: true,
        }
    }
}

impl ScenarioSnapshotPolicy {
    pub fn new() -> Self {
        Self {
            sort_maps: true,
            include: ScenarioSnapshotInclusions::none(),
            redactions: Vec::new(),
        }
    }

    pub fn recommended() -> Self {
        let mut policy = Self {
            include: ScenarioSnapshotInclusions::all(),
            ..Self::new()
        };
        policy
            .add_redaction_preset(ScenarioSnapshotRedactionPreset::portable_paths())
            .add_redaction_preset(ScenarioSnapshotRedactionPreset::runtime_volatility());
        policy
    }

    pub fn add_redaction_preset(&mut self, preset: ScenarioSnapshotRedactionPreset) -> &mut Self {
        self.redactions.extend(preset.redactions());
        self
    }

    fn settings_for(&self, snapshot_path: &Path, scenario: &Scenario) -> insta::Settings {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(snapshot_path);
        settings.set_prepend_module_to_snapshot(false);
        settings.set_sort_maps(self.sort_maps);

        if self.include.description {
            settings.set_description(format!(
                "Scenario {} [{}]: {}",
                scenario.id(),
                scenario.category(),
                scenario.name()
            ));
        } else {
            settings.remove_description();
        }

        if self.include.input_file {
            settings.set_input_file(scenario.dir.join("scenario.toml"));
        } else {
            settings.remove_input_file();
        }

        for redaction in &self.redactions {
            redaction.apply(&mut settings);
        }

        if self.include.info {
            settings.set_info(&ScenarioSnapshotInfo {
                id: scenario.id(),
                name: scenario.name(),
                category: scenario.category(),
                description: &scenario.config.scenario.description,
                input_type: &scenario.config.input.input_type,
                input_paths: &scenario.config.input.paths,
                expected_output_type: scenario
                    .config
                    .expected
                    .as_ref()
                    .map(|expected| expected.output_type.as_str())
                    .filter(|value| !value.is_empty()),
            });
        } else {
            settings.remove_info();
        }

        settings
    }
}

impl Default for ScenarioSnapshotPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioSnapshotRedactionPreset {
    PortablePaths,
    RuntimeVolatility,
}

impl ScenarioSnapshotRedactionPreset {
    pub fn portable_paths() -> Self {
        Self::PortablePaths
    }

    pub fn runtime_volatility() -> Self {
        Self::RuntimeVolatility
    }

    fn redactions(self) -> Vec<ScenarioSnapshotRedaction> {
        match self {
            Self::PortablePaths => vec![
                ScenarioSnapshotRedaction::normalize_path(".**.path"),
                ScenarioSnapshotRedaction::normalize_path(".**.file_path"),
                ScenarioSnapshotRedaction::normalize_path(".**.input_path"),
                ScenarioSnapshotRedaction::normalize_path(".**.output_path"),
                ScenarioSnapshotRedaction::normalize_path(".**.source_path"),
                ScenarioSnapshotRedaction::normalize_path(".**.target_path"),
                ScenarioSnapshotRedaction::normalize_path(".**.temp_dir"),
                ScenarioSnapshotRedaction::normalize_path(".**.workspace_root"),
                ScenarioSnapshotRedaction::normalize_path(".**.cwd"),
                ScenarioSnapshotRedaction::normalize_path(".**.input_paths[]"),
                ScenarioSnapshotRedaction::normalize_path(".**.output_paths[]"),
            ],
            Self::RuntimeVolatility => vec![
                ScenarioSnapshotRedaction::replace(".**.request_id", "[request-id]"),
                ScenarioSnapshotRedaction::replace(".**.trace_id", "[trace-id]"),
                ScenarioSnapshotRedaction::replace(".**.session_id", "[session-id]"),
                ScenarioSnapshotRedaction::replace(".**.run_id", "[run-id]"),
                ScenarioSnapshotRedaction::replace(".**.correlation_id", "[correlation-id]"),
                ScenarioSnapshotRedaction::replace(".**.timestamp", "[timestamp]"),
                ScenarioSnapshotRedaction::replace(".**.created_at", "[created-at]"),
                ScenarioSnapshotRedaction::replace(".**.updated_at", "[updated-at]"),
                ScenarioSnapshotRedaction::replace(".**.started_at", "[started-at]"),
                ScenarioSnapshotRedaction::replace(".**.finished_at", "[finished-at]"),
                ScenarioSnapshotRedaction::replace(".**.completed_at", "[completed-at]"),
                ScenarioSnapshotRedaction::replace(".**.generated_at", "[generated-at]"),
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScenarioSnapshotRedaction {
    Replace {
        selector: String,
        replacement: String,
    },
    NormalizePath {
        selector: String,
    },
}

impl ScenarioSnapshotRedaction {
    fn replace(selector: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self::Replace {
            selector: selector.into(),
            replacement: replacement.into(),
        }
    }

    fn normalize_path(selector: impl Into<String>) -> Self {
        Self::NormalizePath {
            selector: selector.into(),
        }
    }

    fn apply(&self, settings: &mut insta::Settings) {
        match self {
            Self::Replace {
                selector,
                replacement,
            } => settings.add_redaction(selector, replacement.as_str()),
            Self::NormalizePath { selector } => settings.add_redaction(
                selector,
                insta::dynamic_redaction(|value, _path| match value.as_str() {
                    Some(raw) => normalize_snapshot_path(raw).into(),
                    None => value,
                }),
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ScenarioSnapshotInfo<'a> {
    id: &'a str,
    name: &'a str,
    category: &'a str,
    description: &'a str,
    input_type: &'a str,
    input_paths: &'a [String],
    expected_output_type: Option<&'a str>,
}

pub struct ScenarioFramework {
    runners: HashMap<String, Box<dyn ScenarioRunner>>,
    snapshot_path: PathBuf,
    snapshot_policy: ScenarioSnapshotPolicy,
}

impl ScenarioFramework {
    pub fn with_snapshot_path(snapshot_path: impl Into<PathBuf>) -> Self {
        Self {
            runners: HashMap::new(),
            snapshot_path: snapshot_path.into(),
            snapshot_policy: ScenarioSnapshotPolicy::default(),
        }
    }

    pub fn with_snapshot_policy(mut self, snapshot_policy: ScenarioSnapshotPolicy) -> Self {
        self.snapshot_policy = snapshot_policy;
        self
    }

    pub fn register(&mut self, runner: Box<dyn ScenarioRunner>) {
        self.runners.insert(runner.category().to_string(), runner);
    }

    fn find_runner(&self, category: &str) -> Option<&dyn ScenarioRunner> {
        self.runners
            .values()
            .find(|runner| runner.handles_category(category))
            .map(std::convert::AsRef::as_ref)
    }

    pub fn run_all_at(&self, scenarios_root: &Path) -> Result<usize, Box<dyn Error>> {
        let scenarios = load_scenarios_at(scenarios_root)?;
        ensure_unique_scenario_ids(&scenarios)?;
        let mut count = 0;

        for scenario in scenarios {
            let runner = self.find_runner(scenario.category()).ok_or_else(|| {
                io::Error::other(format!(
                    "No runner registered for scenario category '{}' (scenario: {})",
                    scenario.category(),
                    scenario.id()
                ))
            })?;
            count += 1;

            let temp_dir = tempfile::TempDir::new()?;
            if let Some(input_path) = scenario.input_path()
                && input_path.exists()
            {
                copy_dir_recursive(&input_path, temp_dir.path())?;
            }

            let result = runner.run(&scenario, temp_dir.path())?;
            self.assert_scenario_snapshot(&scenario, &result);
        }

        Ok(count)
    }

    fn assert_scenario_snapshot(&self, scenario: &Scenario, result: &Value) {
        let snapshot_name = format!("scenarios__{}", scenario.id());
        let settings = self
            .snapshot_policy
            .settings_for(&self.snapshot_path, scenario);
        settings.bind(|| {
            insta::assert_json_snapshot!(snapshot_name, result);
        });
    }
}

fn load_scenarios_at(root: &Path) -> Result<Vec<Scenario>, Box<dyn Error>> {
    let mut scenarios = Vec::new();

    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() && path.join("scenario.toml").exists() {
            scenarios.push(Scenario::load(path)?);
        }
    }

    scenarios.sort_by(|left, right| left.dir.cmp(&right.dir));
    Ok(scenarios)
}

fn ensure_unique_scenario_ids(scenarios: &[Scenario]) -> Result<(), io::Error> {
    let mut seen = HashMap::new();

    for scenario in scenarios {
        if let Some(existing_dir) = seen.insert(scenario.id().to_string(), scenario.dir.clone()) {
            return Err(io::Error::other(format!(
                "Duplicate scenario id '{}' found in '{}' and '{}'; scenario ids must be unique to avoid snapshot collisions",
                scenario.id(),
                existing_dir.display(),
                scenario.dir.display(),
            )));
        }
    }

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            fs::create_dir_all(&dest_path)?;
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

pub fn find_first_doc_name(dir: &Path) -> Result<String, Box<dyn Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Ok(name) = find_first_doc_name(&path) {
                return Ok(name);
            }
        } else if path.extension().is_some_and(|ext| ext == "md") {
            let stem = path
                .file_stem()
                .ok_or("missing file stem")?
                .to_string_lossy()
                .to_string();
            return Ok(stem);
        }
    }
    Err("no markdown file found".into())
}

fn normalize_snapshot_path(raw: &str) -> String {
    let normalized = raw.replace('\\', "/");
    let prefixes = [
        (workspace_root(), "[workspace]"),
        (home_dir(), "[home]"),
        (Some(std::env::temp_dir()), "[temp]"),
    ];

    for (prefix, placeholder) in prefixes {
        if let Some(prefix) = prefix
            && let Some(rewritten) = rewrite_path_prefix(&normalized, &prefix, placeholder)
        {
            return rewritten;
        }
    }

    normalized
}

fn rewrite_path_prefix(raw: &str, prefix: &Path, placeholder: &str) -> Option<String> {
    let normalized_prefix = prefix.to_string_lossy().replace('\\', "/");
    let suffix = raw
        .strip_prefix(&normalized_prefix)?
        .trim_start_matches('/');

    if suffix.is_empty() {
        Some(placeholder.to_string())
    } else {
        Some(format!("{placeholder}/{suffix}"))
    }
}

fn workspace_root() -> Option<PathBuf> {
    static WORKSPACE_ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    WORKSPACE_ROOT.get_or_init(detect_workspace_root).clone()
}

fn detect_workspace_root() -> Option<PathBuf> {
    let mut current = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    loop {
        let manifest_path = current.join("Cargo.toml");
        if let Ok(manifest) = fs::read_to_string(&manifest_path)
            && manifest.contains("[workspace]")
        {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}
