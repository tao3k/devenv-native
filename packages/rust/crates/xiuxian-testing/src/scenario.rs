//! Reusable scenario testing framework with Insta snapshot support.
//!
//! This module provides a standardized way to define and run scenario-based
//! snapshot tests.
//!
//! # Architecture
//!
//! - `Scenario`: Represents a test scenario with its configuration
//! - `ScenarioRunner`: Trait for category-specific test execution
//! - `ScenarioFramework`: Registry and executor for runners
//!
//! # Directory Structure
//!
//! ```text
//! tests/
//! ├── scenarios/                    # Scenario definitions (you write)
//! │   └── 001_page_index_hierarchy/
//! │       ├── scenario.toml         # Metadata + assertions
//! │       └── input/                # Input files
//! │           └── docs/alpha.md
//! └── snapshots/                    # Snapshots (insta manages)
//!     └── scenarios__*.snap
//! ```
//!
//! # scenario.toml Format
//!
//! ```toml
//! [scenario]
//! id = "001_page_index_hierarchy"
//! name = "Page Index Hierarchy"
//! category = "page_index"
//! description = "Build hierarchical page index"
//!
//! [input]
//! type = "markdown_tree"
//!
//! [runner]
//! build_page_index = true
//!
//! [assert]  # Optional declarative assertions
//! node_count = 3
//! root_title = "Alpha"
//! ```

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================================================
// Configuration Types
// ============================================================================

/// Scenario configuration from scenario.toml
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioConfig {
    /// Scenario metadata (id, name, description, category).
    pub scenario: ScenarioMeta,
    /// Input configuration for the scenario.
    pub input: InputConfig,
    /// Expected output configuration (optional - snapshots managed by insta).
    #[serde(default)]
    pub expected: Option<ExpectedConfig>,
    /// Runner-specific configuration options.
    #[serde(default)]
    pub runner: RunnerConfig,
    /// Declarative assertions (optional).
    #[serde(default)]
    pub assert: AssertConfig,
}

/// Scenario metadata from the `[scenario]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct ScenarioMeta {
    /// Unique scenario identifier per crate (e.g., "`001_routing_keywords_merge`").
    pub id: String,
    /// Human-readable scenario name.
    pub name: String,
    /// Detailed description of what the scenario tests.
    pub description: String,
    /// Category for runner selection (e.g., "`skill_definition`", "`page_index`").
    pub category: String,
}

/// Input configuration from the `[input]` section.
#[derive(Debug, Clone, Deserialize)]
pub struct InputConfig {
    /// Type of input: "`markdown_tree`", "`arrow_batch`", "`json`"
    #[serde(rename = "type")]
    pub input_type: String,
    /// Paths relative to scenario directory
    #[serde(default)]
    pub paths: Vec<String>,
}

/// Expected output configuration from the `[expected]` section.
///
/// This is optional when using insta snapshots, which are managed automatically
/// in `tests/snapshots/scenarios/`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExpectedConfig {
    /// Type of expected output: "`json_snapshot`", "`text_snapshot`"
    #[serde(rename = "type", default)]
    pub output_type: String,
    /// Expected files to compare (optional)
    #[serde(default)]
    pub files: Vec<String>,
}

/// Runner-specific configuration options.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RunnerConfig {
    /// Whether to build page indices during scenario execution.
    pub build_page_index: Option<bool>,
    /// Whether to collect links during scenario execution.
    pub collect_links: Option<bool>,
}

/// Declarative assertions from the `[assert]` section.
///
/// These are optional and provide human-readable validation rules
/// that complement insta snapshots.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AssertConfig {
    /// Expected node count (for tree structures).
    pub node_count: Option<usize>,
    /// Expected root title.
    pub root_title: Option<String>,
    /// Whether the result should have children.
    pub has_children: Option<bool>,
    /// Minimum token count.
    pub min_token_count: Option<usize>,
    /// Custom assertions as key-value pairs.
    #[serde(flatten)]
    pub custom: HashMap<String, Value>,
}

// ============================================================================
// Scenario
// ============================================================================

/// Represents a single test scenario.
pub struct Scenario {
    /// The directory containing the scenario
    pub dir: PathBuf,
    /// The loaded configuration
    pub config: ScenarioConfig,
}

impl Scenario {
    /// Load a scenario from a directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the scenario.toml file cannot be read or parsed.
    pub fn load(dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        let config_path = dir.join("scenario.toml");
        let content = fs::read_to_string(&config_path)?;
        let config: ScenarioConfig = toml::from_str(&content)?;
        Ok(Self { dir, config })
    }

    /// Get the scenario ID.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.config.scenario.id
    }

    /// Get the scenario name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.config.scenario.name
    }

    /// Get the scenario category.
    #[must_use]
    pub fn category(&self) -> &str {
        &self.config.scenario.category
    }

    /// Get the input path (first path in config).
    #[must_use]
    pub fn input_path(&self) -> Option<PathBuf> {
        self.config.input.paths.first().map(|p| self.dir.join(p))
    }

    /// Check if this scenario has input files.
    #[must_use]
    pub fn has_input(&self) -> bool {
        !self.config.input.paths.is_empty()
    }
}

// ============================================================================
// ScenarioRunner Trait
// ============================================================================

/// Trait for category-specific scenario execution.
///
/// Each category of tests (`page_index`, `search`, `graph`, etc.) implements
/// this trait to define how to run scenarios and generate snapshots.
pub trait ScenarioRunner: Send + Sync {
    /// Get the category name this runner handles.
    fn category(&self) -> &str;

    /// Get additional categories this runner handles (for multi-category runners).
    fn additional_categories(&self) -> Vec<&str> {
        vec![]
    }

    /// Check if this runner handles the given category.
    fn handles_category(&self, category: &str) -> bool {
        self.category() == category || self.additional_categories().contains(&category)
    }

    /// Run the scenario and return the result as JSON for snapshot comparison.
    ///
    /// # Errors
    ///
    /// Returns an error if the scenario execution fails.
    fn run(&self, scenario: &Scenario, temp_dir: &Path) -> Result<Value, Box<dyn Error>>;
}

// ============================================================================
// Snapshot Policy
// ============================================================================

/// Shared Insta policy for scenario snapshots.
///
/// This keeps snapshot settings centralized so all scenario execution paths
/// use the same determinism, metadata, and redaction rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioSnapshotPolicy {
    sort_maps: bool,
    metadata: ScenarioSnapshotMetadataPolicy,
    redactions: Vec<ScenarioSnapshotRedaction>,
}

impl ScenarioSnapshotPolicy {
    /// Create a policy with stable JSON ordering and no metadata churn.
    ///
    /// This keeps the default path deterministic while avoiding automatic
    /// snapshot header changes for existing consumer crates.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sort_maps: true,
            metadata: ScenarioSnapshotMetadataPolicy::new(),
            redactions: Vec::new(),
        }
    }

    /// Create the richer recommended policy for generated scenario snapshots.
    ///
    /// This enables snapshot description, structured info metadata, an
    /// input-file backlink to the scenario definition, and shared stability
    /// presets for portable paths plus common runtime volatility fields.
    #[must_use]
    pub fn recommended() -> Self {
        let mut policy = Self {
            metadata: ScenarioSnapshotMetadataPolicy::recommended(),
            ..Self::new()
        };
        policy
            .add_redaction_preset(ScenarioSnapshotRedactionPreset::portable_paths())
            .add_redaction_preset(ScenarioSnapshotRedactionPreset::runtime_volatility());
        policy
    }

    /// Create a profile for snapshots that must remain portable across CI,
    /// mirrored workspaces, or fixture roots that are not part of the public
    /// test contract.
    ///
    /// This keeps the richer metadata envelope from `recommended()` while
    /// omitting the `input_file` header.
    #[must_use]
    pub fn portable_ci() -> Self {
        let mut policy = Self::recommended();
        policy.set_include_input_file(false);
        policy
    }

    /// Create a profile for snapshots that contain runtime-oriented metrics.
    ///
    /// This builds on `recommended()` and rounds common timing fields so
    /// runtime-heavy suites can stay stable without bespoke selector lists.
    #[must_use]
    pub fn runtime_heavy() -> Self {
        let mut policy = Self::recommended();
        policy.add_redaction_preset(ScenarioSnapshotRedactionPreset::timing_noise());
        policy
    }

    /// Return whether serialized maps are sorted before snapshotting.
    #[must_use]
    pub fn sort_maps(&self) -> bool {
        self.sort_maps
    }

    /// Set whether serialized maps are sorted before snapshotting.
    pub fn set_sort_maps(&mut self, value: bool) -> &mut Self {
        self.sort_maps = value;
        self
    }

    /// Return whether snapshot descriptions are emitted.
    #[must_use]
    pub fn include_description(&self) -> bool {
        self.metadata.include_description()
    }

    /// Set whether snapshot descriptions are emitted.
    pub fn set_include_description(&mut self, value: bool) -> &mut Self {
        self.metadata.set_include_description(value);
        self
    }

    /// Return whether structured snapshot info metadata is emitted.
    #[must_use]
    pub fn include_info(&self) -> bool {
        self.metadata.include_info()
    }

    /// Set whether structured snapshot info metadata is emitted.
    pub fn set_include_info(&mut self, value: bool) -> &mut Self {
        self.metadata.set_include_info(value);
        self
    }

    /// Return whether the scenario definition path is attached as snapshot metadata.
    #[must_use]
    pub fn include_input_file(&self) -> bool {
        self.metadata.include_input_file()
    }

    /// Set whether the scenario definition path is attached as snapshot metadata.
    pub fn set_include_input_file(&mut self, value: bool) -> &mut Self {
        self.metadata.set_include_input_file(value);
        self
    }

    /// Add a reusable redaction rule to the policy.
    pub fn add_redaction(&mut self, redaction: ScenarioSnapshotRedaction) -> &mut Self {
        self.redactions.push(redaction);
        self
    }

    /// Add a shared preset of redactions for common snapshot-noise patterns.
    pub fn add_redaction_preset(&mut self, preset: ScenarioSnapshotRedactionPreset) -> &mut Self {
        self.redactions.extend(preset.redactions());
        self
    }

    /// Return the configured redaction rules.
    #[must_use]
    pub fn redactions(&self) -> &[ScenarioSnapshotRedaction] {
        &self.redactions
    }

    fn settings_for(&self, snapshot_path: &Path, scenario: &Scenario) -> insta::Settings {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(snapshot_path);
        settings.set_prepend_module_to_snapshot(false);
        settings.set_sort_maps(self.sort_maps);

        if self.metadata.include_description() {
            settings.set_description(Self::description_for(scenario));
        } else {
            settings.remove_description();
        }

        if self.metadata.include_input_file() {
            settings.set_input_file(scenario.dir.join("scenario.toml"));
        } else {
            settings.remove_input_file();
        }

        for redaction in &self.redactions {
            redaction.apply(&mut settings);
        }

        if self.metadata.include_info() {
            let info = Self::info_for(scenario);
            settings.set_info(&info);
        } else {
            settings.remove_info();
        }

        settings
    }

    fn description_for(scenario: &Scenario) -> String {
        format!(
            "Scenario {} [{}]: {}",
            scenario.id(),
            scenario.category(),
            scenario.name()
        )
    }

    fn info_for(scenario: &Scenario) -> ScenarioSnapshotInfo<'_> {
        ScenarioSnapshotInfo {
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
        }
    }
}

impl Default for ScenarioSnapshotPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScenarioSnapshotMetadataPolicy {
    include_description: bool,
    include_info: bool,
    include_input_file: bool,
}

impl ScenarioSnapshotMetadataPolicy {
    fn new() -> Self {
        Self {
            include_description: false,
            include_info: false,
            include_input_file: false,
        }
    }

    fn recommended() -> Self {
        Self {
            include_description: true,
            include_info: true,
            include_input_file: true,
        }
    }

    fn include_description(&self) -> bool {
        self.include_description
    }

    fn set_include_description(&mut self, value: bool) {
        self.include_description = value;
    }

    fn include_info(&self) -> bool {
        self.include_info
    }

    fn set_include_info(&mut self, value: bool) {
        self.include_info = value;
    }

    fn include_input_file(&self) -> bool {
        self.include_input_file
    }

    fn set_include_input_file(&mut self, value: bool) {
        self.include_input_file = value;
    }
}

/// Shared groups of scenario redactions for common snapshot-stability patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioSnapshotRedactionPreset {
    /// Normalize common path-bearing fields so snapshots stay portable across
    /// workspaces, home directories, and temp roots.
    PortablePaths,
    /// Replace common runtime-generated identifiers and timestamps with stable
    /// placeholders.
    RuntimeVolatility,
    /// Round common timing and duration fields that tend to fluctuate between
    /// executions while preserving their overall scale.
    TimingNoise,
}

impl ScenarioSnapshotRedactionPreset {
    /// Preset for path-bearing fields in scenario output payloads.
    #[must_use]
    pub fn portable_paths() -> Self {
        Self::PortablePaths
    }

    /// Preset for runtime-generated identifiers and timestamps.
    #[must_use]
    pub fn runtime_volatility() -> Self {
        Self::RuntimeVolatility
    }

    /// Preset for common timing and duration metrics in runtime-heavy output.
    #[must_use]
    pub fn timing_noise() -> Self {
        Self::TimingNoise
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
            Self::TimingNoise => vec![
                ScenarioSnapshotRedaction::round(".**.latency_ms", 2),
                ScenarioSnapshotRedaction::round(".**.duration_ms", 2),
                ScenarioSnapshotRedaction::round(".**.elapsed_ms", 2),
                ScenarioSnapshotRedaction::round(".**.processing_ms", 2),
                ScenarioSnapshotRedaction::round(".**.total_ms", 2),
                ScenarioSnapshotRedaction::round(".**.latency_secs", 4),
                ScenarioSnapshotRedaction::round(".**.duration_secs", 4),
                ScenarioSnapshotRedaction::round(".**.elapsed_secs", 4),
                ScenarioSnapshotRedaction::round(".**.processing_secs", 4),
                ScenarioSnapshotRedaction::round(".**.total_secs", 4),
            ],
        }
    }
}

/// A reusable redaction rule for scenario snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioSnapshotRedaction {
    /// Replace the matched value with a static placeholder.
    Replace {
        /// Insta selector path such as `.request.id`.
        selector: String,
        /// Replacement placeholder written into the snapshot.
        replacement: String,
    },
    /// Sort a matched collection before storing it in the snapshot.
    Sort {
        /// Insta selector path such as `.flags`.
        selector: String,
    },
    /// Round a matched floating-point value to a fixed precision.
    Round {
        /// Insta selector path such as `.timings.latency_ms`.
        selector: String,
        /// Number of digits to preserve after the decimal point.
        decimals: usize,
    },
    /// Normalize a selected path string to a portable snapshot representation.
    NormalizePath {
        /// Insta selector path such as `.artifacts.output_path`.
        selector: String,
    },
}

impl ScenarioSnapshotRedaction {
    /// Create a static placeholder redaction.
    #[must_use]
    pub fn replace(selector: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self::Replace {
            selector: selector.into(),
            replacement: replacement.into(),
        }
    }

    /// Create a sorting redaction for unstable sequence or map ordering.
    #[must_use]
    pub fn sort(selector: impl Into<String>) -> Self {
        Self::Sort {
            selector: selector.into(),
        }
    }

    /// Create a rounding redaction for floating-point noise.
    #[must_use]
    pub fn round(selector: impl Into<String>, decimals: usize) -> Self {
        Self::Round {
            selector: selector.into(),
            decimals,
        }
    }

    /// Normalize a selected path string to a portable snapshot representation.
    #[must_use]
    pub fn normalize_path(selector: impl Into<String>) -> Self {
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
            Self::Sort { selector } => settings.add_redaction(selector, insta::sorted_redaction()),
            Self::Round { selector, decimals } => {
                settings.add_redaction(selector, insta::rounded_redaction(*decimals));
            }
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

fn normalize_snapshot_path(raw: &str) -> String {
    let normalized = normalize_path_separators(raw);
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

fn normalize_path_separators(raw: &str) -> String {
    raw.replace('\\', "/")
}

fn rewrite_path_prefix(raw: &str, prefix: &Path, placeholder: &str) -> Option<String> {
    let normalized_prefix = normalize_path_separators(&prefix.to_string_lossy());
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

// ============================================================================
// ScenarioFramework
// ============================================================================

/// Framework for registering and running scenario tests.
pub struct ScenarioFramework {
    runners: HashMap<String, Box<dyn ScenarioRunner>>,
    snapshot_path: PathBuf,
    snapshot_policy: ScenarioSnapshotPolicy,
}

impl ScenarioFramework {
    /// Create a new empty framework with default snapshot path.
    ///
    /// The snapshot path is relative to the crate's manifest directory:
    /// `tests/snapshots/`
    #[must_use]
    pub fn new() -> Self {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        Self {
            runners: HashMap::new(),
            snapshot_path: manifest_dir.join("tests").join("snapshots"),
            snapshot_policy: ScenarioSnapshotPolicy::default(),
        }
    }

    /// Create a new framework with a custom snapshot path.
    #[must_use]
    pub fn with_snapshot_path(snapshot_path: impl Into<PathBuf>) -> Self {
        Self {
            runners: HashMap::new(),
            snapshot_path: snapshot_path.into(),
            snapshot_policy: ScenarioSnapshotPolicy::default(),
        }
    }

    /// Return the current snapshot policy.
    #[must_use]
    pub fn snapshot_policy(&self) -> &ScenarioSnapshotPolicy {
        &self.snapshot_policy
    }

    /// Return the current snapshot policy for in-place updates.
    pub fn snapshot_policy_mut(&mut self) -> &mut ScenarioSnapshotPolicy {
        &mut self.snapshot_policy
    }

    /// Replace the current snapshot policy.
    pub fn set_snapshot_policy(&mut self, snapshot_policy: ScenarioSnapshotPolicy) {
        self.snapshot_policy = snapshot_policy;
    }

    /// Replace the current snapshot policy and return the framework.
    #[must_use]
    pub fn with_snapshot_policy(mut self, snapshot_policy: ScenarioSnapshotPolicy) -> Self {
        self.snapshot_policy = snapshot_policy;
        self
    }

    /// Register a scenario runner.
    pub fn register(&mut self, runner: Box<dyn ScenarioRunner>) {
        let category = runner.category().to_string();
        self.runners.insert(category, runner);
    }

    /// Find the runner for a given category.
    #[must_use]
    pub fn find_runner(&self, category: &str) -> Option<&dyn ScenarioRunner> {
        self.runners
            .values()
            .find(|r| r.handles_category(category))
            .map(std::convert::AsRef::as_ref)
    }

    /// Run all scenarios across all categories.
    ///
    /// This discovers all scenarios and matches them to registered runners
    /// by their category. Insta handles snapshot management.
    ///
    /// # Errors
    ///
    /// Returns an error if any scenario execution fails.
    pub fn run_all(&self) -> Result<usize, Box<dyn Error>> {
        self.run_all_at(&scenarios_root())
    }

    /// Run all scenarios at a specific root directory.
    ///
    /// # Errors
    ///
    /// Returns an error if any scenario execution fails.
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

            // Create temp directory and copy input
            let temp_dir = tempfile::TempDir::new()?;
            if let Some(input_path) = scenario.input_path()
                && input_path.exists()
            {
                copy_dir_recursive(&input_path, temp_dir.path())?;
            }

            // Run the scenario
            let result = runner.run(&scenario, temp_dir.path())?;

            self.assert_scenario_snapshot(&scenario, &result);
        }

        Ok(count)
    }

    /// Run all scenarios in a category using the registered runner.
    ///
    /// # Errors
    ///
    /// Returns an error if no runner is registered for the category or if
    /// any scenario execution fails.
    pub fn run_category(&self, category: &str) -> Result<usize, Box<dyn Error>> {
        self.run_category_at(category, &scenarios_root())
    }

    /// Run all scenarios in a category at a specific root directory.
    ///
    /// # Errors
    ///
    /// Returns an error if no runner is registered for the category or if
    /// any scenario execution fails.
    pub fn run_category_at(
        &self,
        category: &str,
        scenarios_root: &Path,
    ) -> Result<usize, Box<dyn Error>> {
        let runner = self.find_runner(category).ok_or_else(|| {
            io::Error::other(format!("No runner registered for category: {category}"))
        })?;

        let scenarios = load_scenarios_at(scenarios_root)?;
        ensure_unique_scenario_ids(&scenarios)?;
        let mut count = 0;

        for scenario in scenarios {
            if !runner.handles_category(scenario.category()) {
                continue;
            }

            count += 1;

            // Create temp directory and copy input
            let temp_dir = tempfile::TempDir::new()?;
            if let Some(input_path) = scenario.input_path()
                && input_path.exists()
            {
                copy_dir_recursive(&input_path, temp_dir.path())?;
            }

            // Run the scenario and generate snapshot
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

impl Default for ScenarioFramework {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get the scenarios fixture root directory for the current crate.
///
/// This returns the crate-local `tests/scenarios` directory,
/// allowing each crate to define its own scenario tests.
#[must_use]
pub fn scenarios_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scenarios")
}

/// Get a custom scenarios root directory.
#[must_use]
pub fn scenarios_root_at(base: &Path) -> PathBuf {
    base.join("tests").join("scenarios")
}

/// Discover all scenario directories in the default location.
#[must_use]
pub fn discover_scenarios() -> Vec<PathBuf> {
    discover_scenarios_at(&scenarios_root())
}

/// Discover all scenario directories at a specific root.
#[must_use]
pub fn discover_scenarios_at(root: &Path) -> Vec<PathBuf> {
    let mut scenarios = Vec::new();

    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("scenario.toml").exists() {
                scenarios.push(path);
            }
        }
    }

    scenarios.sort();
    scenarios
}

fn load_scenarios_at(root: &Path) -> Result<Vec<Scenario>, Box<dyn Error>> {
    discover_scenarios_at(root)
        .into_iter()
        .map(Scenario::load)
        .collect()
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

/// Copy a directory recursively.
///
/// # Errors
///
/// Returns an error if any file operation fails.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), Box<dyn Error>> {
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

/// Find the first markdown file name in a directory tree.
///
/// # Errors
///
/// Returns an error if no markdown file is found.
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

/// Load expected JSON from a scenario's expected directory.
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed as JSON.
pub fn load_expected_json(scenario_dir: &Path, file: &str) -> Result<Value, Box<dyn Error>> {
    let path = scenario_dir.join("expected").join(file);
    let content = fs::read_to_string(&path)?;
    let value: Value = serde_json::from_str(&content)?;
    Ok(value)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[path = "../tests/unit/scenario/mod.rs"]
mod tests;
