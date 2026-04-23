use std::path::{Path, PathBuf};

use crate::contracts::FlowhubScenarioManifest;
use crate::flowhub::{
    ResolvedFlowhubModule, check_flowhub, load_flowhub_scenario_manifest,
    resolve_flowhub_scenario_modules,
};
use crate::markdown::{MarkdownDiagnostic, render_validation_failed, render_validation_pass};

/// One user-facing validation diagnostic for a Flowhub scenario check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubScenarioDiagnostic {
    /// Short diagnostic title.
    pub title: String,
    /// On-disk location of the failing surface.
    pub location: PathBuf,
    /// Concrete failing condition.
    pub problem: String,
    /// Why the issue blocks bounded execution.
    pub why_it_blocks: String,
    /// Concrete next action.
    pub fix: String,
}

/// Validation report for one Flowhub scenario directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowhubScenarioCheckReport {
    /// Stable scenario/plan name when known.
    pub plan_name: Option<String>,
    /// Scenario root directory.
    pub scenario_dir: PathBuf,
    /// Resolved Flowhub root used for module lookups.
    pub flowhub_root: PathBuf,
    /// Ordered leaf aliases exposed by the scenario when known.
    pub visible_aliases: Vec<String>,
    /// Collected blocking diagnostics.
    pub diagnostics: Vec<FlowhubScenarioDiagnostic>,
}

impl FlowhubScenarioCheckReport {
    /// Returns `true` when no blocking diagnostics were emitted.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Validate a Flowhub scenario directory against Flowhub contracts and the
/// derived bounded work-surface node graph.
#[must_use]
pub fn check_flowhub_scenario(
    flowhub_root: impl AsRef<Path>,
    scenario_dir: impl AsRef<Path>,
) -> FlowhubScenarioCheckReport {
    let flowhub_root = flowhub_root.as_ref();
    let scenario_dir = scenario_dir.as_ref();
    let manifest_path = scenario_dir.join("qianji.toml");

    let mut report = FlowhubScenarioCheckReport {
        plan_name: None,
        scenario_dir: scenario_dir.to_path_buf(),
        flowhub_root: flowhub_root.to_path_buf(),
        visible_aliases: Vec::new(),
        diagnostics: Vec::new(),
    };

    let Some(manifest) = load_manifest_or_record(&mut report, &manifest_path) else {
        return report;
    };
    let Some(resolved_modules) = resolve_modules_or_record(&mut report, flowhub_root, &manifest)
    else {
        return report;
    };

    collect_visible_aliases(&mut report, &manifest, &resolved_modules);
    validate_resolved_modules(&mut report, &resolved_modules);

    report
}

/// Render a scenario validation report into markdown diagnostics.
#[must_use]
pub fn render_flowhub_scenario_check_markdown(report: &FlowhubScenarioCheckReport) -> String {
    if report.is_valid() {
        let plan_name = report.plan_name.as_deref().unwrap_or("(unknown)");
        let mut summary_lines = vec![
            format!("Scenario: {plan_name}"),
            format!("Location: {}", report.scenario_dir.display()),
            format!("Flowhub: {}", report.flowhub_root.display()),
        ];
        if !report.visible_aliases.is_empty() {
            summary_lines.push(format!(
                "Visible surfaces: flowchart.mmd, {}",
                report.visible_aliases.join(", ")
            ));
        }
        return render_validation_pass(&summary_lines);
    }

    let diagnostics = report
        .diagnostics
        .iter()
        .map(|diagnostic| MarkdownDiagnostic {
            title: diagnostic.title.as_str(),
            location: diagnostic.location.display().to_string().into(),
            problem: diagnostic.problem.as_str(),
            why_it_blocks: diagnostic.why_it_blocks.as_str(),
            fix: diagnostic.fix.as_str(),
        })
        .collect::<Vec<_>>();

    render_validation_failed(
        &[
            format!(
                "Scenario: {}",
                report.plan_name.as_deref().unwrap_or("(unknown)")
            ),
            format!("Location: {}", report.scenario_dir.display()),
            format!("Flowhub: {}", report.flowhub_root.display()),
        ],
        &diagnostics,
    )
}

fn load_manifest_or_record(
    report: &mut FlowhubScenarioCheckReport,
    manifest_path: &Path,
) -> Option<FlowhubScenarioManifest> {
    match load_flowhub_scenario_manifest(manifest_path) {
        Ok(manifest) => {
            report.plan_name = Some(manifest.planning.name.clone());
            Some(manifest)
        }
        Err(error) => {
            report.diagnostics.push(FlowhubScenarioDiagnostic {
                title: "Invalid scenario manifest".to_string(),
                location: manifest_path.to_path_buf(),
                problem: error.to_string(),
                why_it_blocks: "Qianji cannot resolve the scenario into a bounded work surface"
                    .to_string(),
                fix: "repair the scenario-root `qianji.toml` so it satisfies the Flowhub contract"
                    .to_string(),
            });
            None
        }
    }
}

fn resolve_modules_or_record(
    report: &mut FlowhubScenarioCheckReport,
    flowhub_root: &Path,
    manifest: &FlowhubScenarioManifest,
) -> Option<Vec<ResolvedFlowhubModule>> {
    match resolve_flowhub_scenario_modules(flowhub_root, manifest) {
        Ok(modules) => Some(modules),
        Err(error) => {
            report.diagnostics.push(FlowhubScenarioDiagnostic {
                title: "Scenario resolve failed".to_string(),
                location: report.scenario_dir.join("qianji.toml"),
                problem: error.to_string(),
                why_it_blocks: "the scenario cannot select its declared Flowhub modules"
                    .to_string(),
                fix: "repair `template.use` so every selected module resolves under `qianji-flowhub/`"
                    .to_string(),
            });
            None
        }
    }
}

fn collect_visible_aliases(
    report: &mut FlowhubScenarioCheckReport,
    manifest: &FlowhubScenarioManifest,
    resolved_modules: &[ResolvedFlowhubModule],
) {
    report.visible_aliases = resolved_modules
        .iter()
        .filter(|module| module.manifest.template.is_none())
        .map(|module| module.alias.clone())
        .collect();

    if report.visible_aliases.is_empty() {
        report.diagnostics.push(FlowhubScenarioDiagnostic {
            title: "No visible leaf surfaces".to_string(),
            location: report.scenario_dir.join("qianji.toml"),
            problem: format!(
                "scenario `{}` does not expose any leaf nodes that can anchor a bounded work surface",
                manifest.planning.name
            ),
            why_it_blocks: "Codex would have no bounded surface to inspect or edit".to_string(),
            fix: "select at least one leaf node in `template.use` or expose a leaf node from the composite chain".to_string(),
        });
    }
}

fn validate_resolved_modules(
    report: &mut FlowhubScenarioCheckReport,
    resolved_modules: &[ResolvedFlowhubModule],
) {
    for module in resolved_modules {
        match check_flowhub(&module.module_dir) {
            Ok(module_report) => {
                for diagnostic in module_report.diagnostics {
                    report.diagnostics.push(FlowhubScenarioDiagnostic {
                        title: format!("{}: {}", module.module_ref, diagnostic.title),
                        location: diagnostic.location,
                        problem: diagnostic.problem,
                        why_it_blocks: format!(
                            "scenario guard-graph evaluation is blocked because node `{}` is invalid; {}",
                            module.module_ref, diagnostic.why_it_blocks
                        ),
                        fix: diagnostic.fix,
                    });
                }
            }
            Err(error) => report.diagnostics.push(FlowhubScenarioDiagnostic {
                title: format!("Module preflight failed: {}", module.module_ref),
                location: module.module_dir.clone(),
                problem: error.to_string(),
                why_it_blocks:
                    "the scenario cannot trust this Flowhub node before guard-graph evaluation"
                        .to_string(),
                fix: format!(
                    "repair `{}` so `qianji check --dir {}` succeeds",
                    module.module_ref,
                    module.module_dir.display()
                ),
            }),
        }
    }
}
