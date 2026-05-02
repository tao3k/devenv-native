use crate::bin_support::wendao::types::{AuditArgs, Cli};
use crate::link_graph::LinkGraphIndex;
use crate::zhenfa_router::native::semantic_check::{
    WendaoSemanticCheckArgs, wendao_semantic_check,
};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use xiuxian_zhenfa::ZhenfaContext;

const DEFAULT_EPISTEME_DIR: &str = "wendao-episteme";

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct EpistemeTemplateManifest {
    imports: TemplateManifestImports,
    authoring_templates: Vec<AuthoringTemplateEntry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct TemplateManifestImports {
    policy_manifests: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AuthoringTemplateEntry {
    id: String,
    framework: String,
    path: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AuthoringTemplate {
    id: String,
    framework: String,
    mode: String,
    coordinate_source: String,
    topology_source: String,
    authority: String,
    llm_role: String,
    authority_rule: String,
    paths: TemplatePaths,
    #[serde(alias = "deterministic_diagnostics")]
    diagnostics: Vec<TemplateDiagnostic>,
    llm_inputs: Vec<NamedDescription>,
    llm_outputs: Vec<NamedDescription>,
    examples: Vec<TemplateExample>,
    verification: Vec<VerificationStep>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct TemplatePaths {
    category_directory_shape: String,
    note_file_shape: String,
    coordinate_shape: String,
    inbox_directory: String,
    inbox_policy: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct TemplateDiagnostic {
    code: String,
    severity: String,
    meaning: String,
    llm_action: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct NamedDescription {
    name: String,
    description: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct TemplateExample {
    kind: String,
    path: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct VerificationStep {
    phase: String,
    command: String,
}

pub(super) fn handle(cli: &Cli, args: &AuditArgs, index: Option<&LinkGraphIndex>) -> Result<()> {
    if args.template.is_some() {
        print_authoring_template(cli, args)?;
        return Ok(());
    }

    let mut ctx = ZhenfaContext::default();

    // Inject the index into context extensions if available
    if let Some(idx) = index {
        ctx.insert_extension(idx.clone());
    } else {
        anyhow::bail!(
            "LinkGraphIndex must be provided for audit (check your environment or --scope)"
        );
    }

    // Convert CLI args to the Tool args
    let check_args = WendaoSemanticCheckArgs {
        doc: Some(args.target.clone()),
        checks: None,
        include_warnings: Some(true),
        source_paths: args.source.as_ref().map(|s| vec![s.clone()]),
        fuzzy_confidence_threshold: Some(args.threshold),
        episteme_load: args.load.clone(),
    };

    let result = wendao_semantic_check(&ctx, check_args)
        .map_err(|e| anyhow::anyhow!("Audit failed: {e:?}"))?;

    println!("{result}");
    Ok(())
}

fn print_authoring_template(cli: &Cli, args: &AuditArgs) -> Result<()> {
    let rendered = render_authoring_template(cli, args)?;
    print!("{rendered}");
    if !rendered.ends_with('\n') {
        println!();
    }
    Ok(())
}

fn render_authoring_template(cli: &Cli, args: &AuditArgs) -> Result<String> {
    let (template_path, template_text) = load_authoring_template_text(cli, args)?;
    let template: AuthoringTemplate = toml::from_str(&template_text).with_context(|| {
        format!(
            "failed to parse authoring template `{}`",
            template_path.display()
        )
    })?;
    validate_authoring_template(&template)
        .with_context(|| format!("invalid authoring template `{}`", template_path.display()))?;
    Ok(render_authoring_template_compact(&template))
}

fn validate_authoring_template(template: &AuthoringTemplate) -> Result<()> {
    let mut missing = Vec::new();
    collect_template_scalar_missing(template, &mut missing);
    collect_template_path_missing(&template.paths, &mut missing);
    collect_template_diagnostic_missing(&template.diagnostics, &mut missing);
    collect_named_description_missing("llm_inputs", &template.llm_inputs, &mut missing);
    collect_named_description_missing("llm_outputs", &template.llm_outputs, &mut missing);
    collect_template_example_missing(&template.examples, &mut missing);
    collect_template_verification_missing(&template.verification, &mut missing);

    if missing.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "authoring template is missing required LLM contract fields: {}",
            missing.join(", ")
        );
    }
}

fn collect_template_scalar_missing(template: &AuthoringTemplate, missing: &mut Vec<String>) {
    push_missing(missing, "id", template.id.as_str());
    push_missing(missing, "framework", template.framework.as_str());
    push_missing(missing, "mode", template.mode.as_str());
    push_missing(
        missing,
        "coordinate_source",
        template.coordinate_source.as_str(),
    );
    push_missing(
        missing,
        "topology_source",
        template.topology_source.as_str(),
    );
    push_missing(missing, "authority_rule", template.authority_rule.as_str());
    push_missing(missing, "authority", template.authority.as_str());
    push_missing(missing, "llm_role", template.llm_role.as_str());
}

fn collect_template_path_missing(paths: &TemplatePaths, missing: &mut Vec<String>) {
    push_missing(
        missing,
        "paths.category_directory_shape",
        paths.category_directory_shape.as_str(),
    );
    push_missing(
        missing,
        "paths.note_file_shape",
        paths.note_file_shape.as_str(),
    );
    push_missing(
        missing,
        "paths.coordinate_shape",
        paths.coordinate_shape.as_str(),
    );
    push_missing(
        missing,
        "paths.inbox_directory",
        paths.inbox_directory.as_str(),
    );
    push_missing(missing, "paths.inbox_policy", paths.inbox_policy.as_str());
}

fn collect_template_diagnostic_missing(
    diagnostics: &[TemplateDiagnostic],
    missing: &mut Vec<String>,
) {
    if diagnostics.is_empty() {
        missing.push("diagnostics".to_string());
    }
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        push_missing(
            missing,
            format!("diagnostics[{index}].code"),
            diagnostic.code.as_str(),
        );
        push_missing(
            missing,
            format!("diagnostics[{index}].severity"),
            diagnostic.severity.as_str(),
        );
        push_missing(
            missing,
            format!("diagnostics[{index}].meaning"),
            diagnostic.meaning.as_str(),
        );
        push_missing(
            missing,
            format!("diagnostics[{index}].llm_action"),
            diagnostic.llm_action.as_str(),
        );
    }
}

fn collect_named_description_missing(
    field: &str,
    items: &[NamedDescription],
    missing: &mut Vec<String>,
) {
    if items.is_empty() {
        missing.push(field.to_string());
    }
    for (index, item) in items.iter().enumerate() {
        push_missing(
            missing,
            format!("{field}[{index}].name"),
            item.name.as_str(),
        );
        push_missing(
            missing,
            format!("{field}[{index}].description"),
            item.description.as_str(),
        );
    }
}

fn collect_template_example_missing(examples: &[TemplateExample], missing: &mut Vec<String>) {
    if examples.is_empty() {
        missing.push("examples".to_string());
    }
    for (index, example) in examples.iter().enumerate() {
        push_missing(
            missing,
            format!("examples[{index}].kind"),
            example.kind.as_str(),
        );
        push_missing(
            missing,
            format!("examples[{index}].path"),
            example.path.as_str(),
        );
    }
}

fn collect_template_verification_missing(
    verification: &[VerificationStep],
    missing: &mut Vec<String>,
) {
    if verification.is_empty() {
        missing.push("verification".to_string());
    }
    for (index, step) in verification.iter().enumerate() {
        push_missing(
            missing,
            format!("verification[{index}].phase"),
            step.phase.as_str(),
        );
        push_missing(
            missing,
            format!("verification[{index}].command"),
            step.command.as_str(),
        );
    }
}

fn push_missing(missing: &mut Vec<String>, field: impl Into<String>, value: &str) {
    if value.trim().is_empty() {
        missing.push(field.into());
    }
}

fn load_authoring_template_text(cli: &Cli, args: &AuditArgs) -> Result<(PathBuf, String)> {
    let requested_template = args
        .template
        .as_deref()
        .context("missing audit template framework")?;
    let (manifest_path, episteme_root) = resolve_template_manifest_path(cli, args)?;
    let manifest_text = fs::read_to_string(&manifest_path).with_context(|| {
        format!(
            "failed to read episteme manifest `{}`",
            manifest_path.display()
        )
    })?;
    let manifest: EpistemeTemplateManifest = toml::from_str(&manifest_text).with_context(|| {
        format!(
            "failed to parse episteme manifest `{}`",
            manifest_path.display()
        )
    })?;
    let authoring_templates =
        collect_authoring_templates(&episteme_root, manifest_path.as_path(), manifest)?;
    let template_entry = authoring_templates
        .iter()
        .find(|entry| entry.framework == requested_template || entry.id == requested_template)
        .with_context(|| {
            format!(
                "missing authoring template `{requested_template}` in `{}`",
                manifest_path.display()
            )
        })?;
    let template_path = validate_template_path(&episteme_root, template_entry)?;
    let template_text = fs::read_to_string(&template_path).with_context(|| {
        format!(
            "failed to read authoring template `{}`",
            template_path.display()
        )
    })?;
    Ok((template_path, template_text))
}

fn collect_authoring_templates(
    episteme_root: &Path,
    manifest_path: &Path,
    manifest: EpistemeTemplateManifest,
) -> Result<Vec<AuthoringTemplateEntry>> {
    if !manifest.authoring_templates.is_empty() {
        anyhow::bail!(
            "episteme root manifest must not declare inline authoring_templates; move the entries into distributed policy manifests"
        );
    }
    if manifest.imports.policy_manifests.is_empty() {
        anyhow::bail!(
            "episteme manifest `{}` must declare imports.policy_manifests",
            manifest_path.display()
        );
    }

    let mut authoring_templates = Vec::new();
    for policy_manifest_path in manifest.imports.policy_manifests {
        let policy_manifest_path = validate_import_path(
            episteme_root,
            "policy manifest",
            policy_manifest_path.as_str(),
        )?;
        let policy_manifest_text =
            fs::read_to_string(&policy_manifest_path).with_context(|| {
                format!(
                    "failed to read policy manifest `{}`",
                    policy_manifest_path.display()
                )
            })?;
        let mut policy_manifest: EpistemeTemplateManifest = toml::from_str(&policy_manifest_text)
            .with_context(|| {
            format!(
                "failed to parse policy manifest `{}` imported by `{}`",
                policy_manifest_path.display(),
                manifest_path.display()
            )
        })?;
        let policy_manifest_root = policy_manifest_path
            .parent()
            .unwrap_or_else(|| Path::new("."));
        for template in &mut policy_manifest.authoring_templates {
            template.path = normalize_imported_template_path(
                episteme_root,
                policy_manifest_root,
                &template.id,
                template.path.as_str(),
            )?;
        }
        authoring_templates.extend(policy_manifest.authoring_templates);
    }
    Ok(authoring_templates)
}

fn validate_import_path(episteme_root: &Path, kind: &str, path: &str) -> Result<PathBuf> {
    let declared_path = Path::new(path);
    if declared_path.is_absolute() {
        anyhow::bail!("{kind} import declares absolute path `{path}`");
    }
    let resolved_path = episteme_root.join(declared_path);
    if !resolved_path.is_file() {
        anyhow::bail!("{kind} import is missing at `{}`", resolved_path.display());
    }
    Ok(resolved_path)
}

fn normalize_imported_template_path(
    episteme_root: &Path,
    policy_manifest_root: &Path,
    id: &str,
    path: &str,
) -> Result<String> {
    let declared_path = Path::new(path);
    if declared_path.is_absolute() {
        anyhow::bail!("authoring template `{id}` declares absolute path `{path}`");
    }
    let resolved_path = policy_manifest_root.join(declared_path);
    if !resolved_path.is_file() {
        anyhow::bail!(
            "authoring template `{id}` file is missing at `{}`",
            resolved_path.display()
        );
    }
    Ok(resolved_path
        .strip_prefix(episteme_root)
        .unwrap_or(resolved_path.as_path())
        .to_string_lossy()
        .to_string())
}

fn render_authoring_template_compact(template: &AuthoringTemplate) -> String {
    let mut rendered = String::new();
    let id = fallback(template.id.as_str(), "unknown-template");

    let _ = writeln!(rendered, "Wendao audit template for LLM: {id}");
    append_field(&mut rendered, "framework", template.framework.as_str());
    append_field(&mut rendered, "mode", template.mode.as_str());
    append_field(
        &mut rendered,
        "coordinate_source",
        template.coordinate_source.as_str(),
    );
    append_field(
        &mut rendered,
        "topology_source",
        template.topology_source.as_str(),
    );
    append_field(&mut rendered, "authority", template.authority.as_str());
    append_field(&mut rendered, "llm_role", template.llm_role.as_str());

    if !template.authority_rule.trim().is_empty() {
        let _ = writeln!(rendered, "\nauthority rule:");
        let _ = writeln!(rendered, "  {}", template.authority_rule);
    }

    append_path_contract(&mut rendered, &template.paths);
    append_diagnostics(&mut rendered, &template.diagnostics);
    append_named_descriptions(&mut rendered, "llm inputs", &template.llm_inputs);
    append_named_descriptions(&mut rendered, "llm outputs", &template.llm_outputs);
    append_examples(&mut rendered, &template.examples);
    append_verification(&mut rendered, &template.verification);

    rendered
}

fn append_field(rendered: &mut String, label: &str, value: &str) {
    if !value.trim().is_empty() {
        let _ = writeln!(rendered, "{label}: {value}");
    }
}

fn append_path_contract(rendered: &mut String, paths: &TemplatePaths) {
    let fields = [
        (
            "category_directory_shape",
            paths.category_directory_shape.as_str(),
        ),
        ("note_file_shape", paths.note_file_shape.as_str()),
        ("coordinate_shape", paths.coordinate_shape.as_str()),
        ("inbox_directory", paths.inbox_directory.as_str()),
        ("inbox_policy", paths.inbox_policy.as_str()),
    ];
    if fields.iter().all(|(_label, value)| value.trim().is_empty()) {
        return;
    }

    let _ = writeln!(rendered, "\npath contract:");
    for (label, value) in fields {
        if !value.trim().is_empty() {
            let _ = writeln!(rendered, "  {label}: {value}");
        }
    }
}

fn append_diagnostics(rendered: &mut String, diagnostics: &[TemplateDiagnostic]) {
    if diagnostics.is_empty() {
        return;
    }

    let _ = writeln!(rendered, "\ndeterministic diagnostics:");
    for diagnostic in diagnostics {
        let severity = fallback(diagnostic.severity.as_str(), "diagnostic");
        let code = fallback(diagnostic.code.as_str(), "unknown");
        let meaning = fallback(
            diagnostic.meaning.as_str(),
            "No diagnostic meaning declared.",
        );
        let _ = writeln!(rendered, "{severity}[{code}]: {meaning}");
        if !diagnostic.llm_action.trim().is_empty() {
            let _ = writeln!(rendered, "  help: {}", diagnostic.llm_action);
        }
    }
}

fn append_named_descriptions(rendered: &mut String, heading: &str, items: &[NamedDescription]) {
    if items.is_empty() {
        return;
    }

    let _ = writeln!(rendered, "\n{heading}:");
    for item in items {
        let name = fallback(item.name.as_str(), "unknown");
        let description = fallback(item.description.as_str(), "No description declared.");
        let _ = writeln!(rendered, "  {name}: {description}");
    }
}

fn append_examples(rendered: &mut String, examples: &[TemplateExample]) {
    if examples.is_empty() {
        return;
    }

    let _ = writeln!(rendered, "\nexamples:");
    for example in examples {
        let kind = fallback(example.kind.as_str(), "example");
        let path = fallback(example.path.as_str(), "unknown");
        let _ = writeln!(rendered, "  {kind}: {path}");
    }
}

fn append_verification(rendered: &mut String, verification: &[VerificationStep]) {
    if verification.is_empty() {
        return;
    }

    let _ = writeln!(rendered, "\nverification:");
    for step in verification {
        let command = fallback(step.command.as_str(), "unknown command");
        if step.phase.trim().is_empty() {
            let _ = writeln!(rendered, "  {command}");
        } else {
            let _ = writeln!(rendered, "  {}: {command}", step.phase);
        }
    }
}

fn fallback<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    let value = value.trim();
    if value.is_empty() { fallback } else { value }
}

fn resolve_template_manifest_path(cli: &Cli, args: &AuditArgs) -> Result<(PathBuf, PathBuf)> {
    let load_path = args
        .load
        .as_deref()
        .map_or_else(|| default_episteme_dir(cli.root.as_path()), PathBuf::from);
    let manifest_path = if load_path.is_dir() {
        load_path.join("episteme.toml")
    } else {
        load_path
    };
    if !manifest_path.is_file() {
        anyhow::bail!(
            "missing episteme manifest for audit template at `{}`; pass --load <episteme>",
            manifest_path.display()
        );
    }
    let episteme_root = manifest_path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    Ok((manifest_path, episteme_root))
}

fn default_episteme_dir(cli_root: &Path) -> PathBuf {
    let root_candidate = cli_root.join(DEFAULT_EPISTEME_DIR);
    if root_candidate.exists() {
        return root_candidate;
    }
    PathBuf::from(DEFAULT_EPISTEME_DIR)
}

fn validate_template_path(
    episteme_root: &Path,
    template_entry: &AuthoringTemplateEntry,
) -> Result<PathBuf> {
    let declared_path = Path::new(&template_entry.path);
    if declared_path.is_absolute() {
        anyhow::bail!(
            "authoring template `{}` declares absolute path `{}`",
            template_entry.id,
            declared_path.display()
        );
    }
    let template_path = episteme_root.join(declared_path);
    if !template_path.is_file() {
        anyhow::bail!(
            "authoring template `{}` file is missing at `{}`",
            template_entry.id,
            template_path.display()
        );
    }
    Ok(template_path)
}

#[cfg(test)]
#[path = "../../../../tests/unit/bin/wendao/execute/audit.rs"]
mod tests;
