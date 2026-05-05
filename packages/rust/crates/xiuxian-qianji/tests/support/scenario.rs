use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ScenarioConfig {
    pub scenario: ScenarioMeta,
    #[serde(default)]
    pub input: InputConfig,
    #[serde(default)]
    pub expected: Option<ExpectedConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ScenarioMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct InputConfig {
    #[serde(rename = "type", default)]
    pub input_type: String,
    #[serde(default)]
    pub paths: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct ExpectedConfig {
    #[serde(rename = "type", default)]
    pub output_type: String,
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
}

pub trait ScenarioRunner: Send + Sync {
    fn category(&self) -> &str;

    fn run(&self, scenario: &Scenario, temp_dir: &Path) -> Result<Value, Box<dyn Error>>;
}

pub struct ScenarioFramework {
    runners: HashMap<String, Box<dyn ScenarioRunner>>,
    snapshot_path: PathBuf,
}

impl ScenarioFramework {
    pub fn with_snapshot_path(snapshot_path: impl Into<PathBuf>) -> Self {
        Self {
            runners: HashMap::new(),
            snapshot_path: snapshot_path.into(),
        }
    }

    pub fn register(&mut self, runner: Box<dyn ScenarioRunner>) {
        self.runners.insert(runner.category().to_string(), runner);
    }

    pub fn run_all_at(&self, scenarios_root: &Path) -> Result<usize, Box<dyn Error>> {
        let scenarios = load_scenarios_at(scenarios_root)?;
        ensure_unique_scenario_ids(&scenarios)?;
        let mut count = 0;

        for scenario in scenarios {
            let runner = self.runners.get(scenario.category()).ok_or_else(|| {
                io::Error::other(format!(
                    "No runner registered for scenario category '{}' (scenario: {})",
                    scenario.category(),
                    scenario.id()
                ))
            })?;
            let temp_dir = tempfile::TempDir::new()?;
            let result = runner.run(&scenario, temp_dir.path())?;
            self.assert_scenario_snapshot(&scenario, &result);
            count += 1;
        }

        Ok(count)
    }

    fn assert_scenario_snapshot(&self, scenario: &Scenario, result: &Value) {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(&self.snapshot_path);
        settings.set_prepend_module_to_snapshot(false);
        settings.set_sort_maps(true);
        settings.set_description(format!(
            "Scenario {} [{}]: {}",
            scenario.id(),
            scenario.category(),
            scenario.name()
        ));
        settings.set_input_file(scenario.dir.join("scenario.toml"));
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
        settings.bind(|| {
            insta::assert_json_snapshot!(format!("scenarios__{}", scenario.id()), result);
        });
    }
}

#[derive(Debug, serde::Serialize)]
struct ScenarioSnapshotInfo<'a> {
    id: &'a str,
    name: &'a str,
    category: &'a str,
    description: &'a str,
    input_type: &'a str,
    input_paths: &'a [PathBuf],
    expected_output_type: Option<&'a str>,
}

fn load_scenarios_at(root: &Path) -> Result<Vec<Scenario>, Box<dyn Error>> {
    let mut dirs = fs::read_dir(root)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    dirs.sort();

    dirs.into_iter()
        .filter(|path| path.is_dir())
        .map(Scenario::load)
        .collect()
}

fn ensure_unique_scenario_ids(scenarios: &[Scenario]) -> Result<(), io::Error> {
    let mut seen = HashSet::new();
    for scenario in scenarios {
        if !seen.insert(scenario.id().to_string()) {
            return Err(io::Error::other(format!(
                "duplicate scenario id: {}",
                scenario.id()
            )));
        }
    }
    Ok(())
}
