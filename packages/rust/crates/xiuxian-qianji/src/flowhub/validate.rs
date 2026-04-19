use std::collections::BTreeSet;
use std::path::Path;

use crate::contracts::{
    FlowhubGraphContract, FlowhubGraphNodeContract, FlowhubGraphSurfaceContract,
    FlowhubModuleManifest, FlowhubRootManifest, FlowhubScenarioManifest, FlowhubStructureContract,
    FlowhubTemplateComposition, FlowhubValidationKind, TemplateLinkRef, WorkdirManifest,
    WorkdirPlan,
};
use crate::error::QianjiError;
use crate::workdir::validate_workdir_manifest;

pub(super) fn validate_flowhub_module_manifest(
    manifest: &FlowhubModuleManifest,
) -> Result<(), QianjiError> {
    if manifest.version != 1 {
        return Err(QianjiError::Topology(format!(
            "unsupported Flowhub module manifest version `{}`: expected `1`",
            manifest.version
        )));
    }

    if manifest.module.name.trim().is_empty() {
        return Err(QianjiError::Topology(
            "Flowhub module manifest requires a non-empty `module.name`".to_string(),
        ));
    }

    if manifest.exports.entry.trim().is_empty() {
        return Err(QianjiError::Topology(
            "Flowhub module manifest requires a non-empty `exports.entry`".to_string(),
        ));
    }

    if manifest.exports.ready.trim().is_empty() {
        return Err(QianjiError::Topology(
            "Flowhub module manifest requires a non-empty `exports.ready`".to_string(),
        ));
    }

    if let Some(template) = &manifest.template {
        validate_template_composition(template, "Flowhub composite module manifest", true, true)?;
    }

    if let Some(contract) = &manifest.contract {
        validate_structure_contract(contract, "Flowhub module manifest", false)?;
        if let Some(template) = &manifest.template {
            let registered = contract
                .register
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            let templated = template
                .use_entries
                .iter()
                .map(|entry| entry.module_ref.as_str())
                .collect::<BTreeSet<_>>();
            if registered != templated {
                return Err(QianjiError::Topology(
                    "Flowhub module manifest `contract.register` must match `template.use` module refs for owned child nodes"
                        .to_string(),
                ));
            }
        }
    }

    validate_graph_contracts(manifest)?;

    for rule in &manifest.validation {
        if rule.path.trim().is_empty() {
            return Err(QianjiError::Topology(
                "Flowhub module manifest contains a `[[validation]]` rule with an empty `path`"
                    .to_string(),
            ));
        }

        if rule.kind != FlowhubValidationKind::Glob && rule.min_matches.is_some() {
            return Err(QianjiError::Topology(format!(
                "Flowhub module validation path `{}` uses `min_matches`, but only `kind = \"glob\"` supports it",
                rule.path
            )));
        }
    }

    Ok(())
}

fn validate_graph_contracts(manifest: &FlowhubModuleManifest) -> Result<(), QianjiError> {
    if manifest.graph.is_empty() {
        return Ok(());
    }

    let Some(contract) = &manifest.contract else {
        return Err(QianjiError::Topology(
            "Flowhub module manifest requires `[contract]` when `[[graph]]` entries are declared"
                .to_string(),
        ));
    };

    let required_entries = contract
        .required
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut graph_paths = BTreeSet::new();
    for graph in &manifest.graph {
        validate_graph_contract_path(graph)?;
        validate_graph_contract_name(graph)?;
        validate_graph_workdir_contract(graph)?;
        validate_graph_node_contracts(graph)?;
        let path = graph.path.as_str();
        if !graph_paths.insert(path) {
            return Err(QianjiError::Topology(format!(
                "Flowhub module manifest contains duplicate `[[graph]]` path `{path}`"
            )));
        }
        if !required_entries.contains(path) {
            return Err(QianjiError::Topology(format!(
                "Flowhub module manifest `[[graph]] path = \"{path}\"` must also be declared in `contract.required`"
            )));
        }
    }

    Ok(())
}

pub(super) fn validate_flowhub_root_manifest(
    manifest: &FlowhubRootManifest,
) -> Result<(), QianjiError> {
    if manifest.version != 1 {
        return Err(QianjiError::Topology(format!(
            "unsupported Flowhub root manifest version `{}`: expected `1`",
            manifest.version
        )));
    }

    if manifest.flowhub.name.trim().is_empty() {
        return Err(QianjiError::Topology(
            "Flowhub root manifest requires a non-empty `flowhub.name`".to_string(),
        ));
    }

    validate_structure_contract(&manifest.contract, "Flowhub root manifest", true)?;

    Ok(())
}

pub(super) fn validate_flowhub_scenario_manifest(
    manifest: &FlowhubScenarioManifest,
) -> Result<(), QianjiError> {
    if manifest.version != 1 {
        return Err(QianjiError::Topology(format!(
            "unsupported Flowhub scenario manifest version `{}`: expected `1`",
            manifest.version
        )));
    }

    if manifest.planning.name.trim().is_empty() {
        return Err(QianjiError::Topology(
            "Flowhub scenario manifest requires a non-empty `planning.name`".to_string(),
        ));
    }

    validate_template_composition(&manifest.template, "Flowhub scenario manifest", true, false)?;

    Ok(())
}

fn validate_template_composition(
    template: &FlowhubTemplateComposition,
    context: &str,
    require_use_entries: bool,
    allow_local_link_refs: bool,
) -> Result<(), QianjiError> {
    if require_use_entries && template.use_entries.is_empty() {
        return Err(QianjiError::Topology(format!(
            "{context} requires at least one `template.use` entry"
        )));
    }

    let mut aliases = BTreeSet::new();
    for use_entry in &template.use_entries {
        if !aliases.insert(use_entry.alias.as_str()) {
            return Err(QianjiError::Topology(format!(
                "duplicate template.use alias `{}`",
                use_entry.alias
            )));
        }
    }

    for link in &template.link {
        validate_link_alias(&aliases, &link.from, allow_local_link_refs)?;
        validate_link_alias(&aliases, &link.to, allow_local_link_refs)?;
    }

    Ok(())
}

fn validate_link_alias(
    aliases: &BTreeSet<&str>,
    link_ref: &TemplateLinkRef,
    allow_local_link_refs: bool,
) -> Result<(), QianjiError> {
    match &link_ref.alias {
        Some(alias) if aliases.contains(alias.as_str()) => Ok(()),
        Some(alias) => Err(QianjiError::Topology(format!(
            "unknown template.link alias `{alias}` in reference `{link_ref}`"
        ))),
        None if allow_local_link_refs => Ok(()),
        None => Err(QianjiError::Topology(format!(
            "scenario template.link reference `{link_ref}` must use `<alias>::<symbol>`"
        ))),
    }
}

fn validate_structure_contract(
    contract: &FlowhubStructureContract,
    context: &str,
    require_register: bool,
) -> Result<(), QianjiError> {
    if require_register && contract.register.is_empty() {
        return Err(QianjiError::Topology(format!(
            "{context} requires at least one `contract.register` entry"
        )));
    }

    if require_register && contract.required.is_empty() {
        return Err(QianjiError::Topology(format!(
            "{context} requires at least one `contract.required` entry"
        )));
    }

    let mut registered = BTreeSet::new();
    for entry in &contract.register {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            return Err(QianjiError::Topology(format!(
                "{context} contains an empty `contract.register` entry"
            )));
        }
        validate_registered_child_ref(trimmed, context)?;
        if !registered.insert(trimmed) {
            return Err(QianjiError::Topology(format!(
                "{context} contains duplicate `contract.register` entry `{trimmed}`"
            )));
        }
    }

    let mut required = BTreeSet::new();
    for entry in &contract.required {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            return Err(QianjiError::Topology(format!(
                "{context} contains an empty `contract.required` entry"
            )));
        }
        validate_required_pattern(trimmed, context)?;
        if !required.insert(trimmed) {
            return Err(QianjiError::Topology(format!(
                "{context} contains duplicate `contract.required` entry `{trimmed}`"
            )));
        }
    }

    Ok(())
}

fn validate_registered_child_ref(entry: &str, context: &str) -> Result<(), QianjiError> {
    if entry.starts_with('/') {
        return Err(QianjiError::Topology(format!(
            "{context} `contract.register` entry `{entry}` must stay relative"
        )));
    }

    for segment in entry.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(QianjiError::Topology(format!(
                "{context} `contract.register` entry `{entry}` contains an invalid path segment"
            )));
        }
        if segment
            .chars()
            .any(|character| matches!(character, '*' | '?' | '[' | ']'))
        {
            return Err(QianjiError::Topology(format!(
                "{context} `contract.register` entry `{entry}` must not contain glob syntax"
            )));
        }
    }

    Ok(())
}

fn validate_required_pattern(entry: &str, context: &str) -> Result<(), QianjiError> {
    if entry.starts_with('/') {
        return Err(QianjiError::Topology(format!(
            "{context} `contract.required` entry `{entry}` must stay relative"
        )));
    }

    for segment in entry.split('/') {
        if segment.is_empty() {
            return Err(QianjiError::Topology(format!(
                "{context} `contract.required` entry `{entry}` contains an empty path segment"
            )));
        }
        if segment == ".." {
            return Err(QianjiError::Topology(format!(
                "{context} `contract.required` entry `{entry}` must stay inside the graph node directory"
            )));
        }
    }

    Ok(())
}

fn validate_graph_contract_path(graph: &FlowhubGraphContract) -> Result<(), QianjiError> {
    let path = graph.path.trim();
    if path.is_empty() {
        return Err(QianjiError::Topology(
            "Flowhub module manifest contains a `[[graph]]` entry with an empty `path`".to_string(),
        ));
    }
    if path.starts_with('/') {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{}\"` must stay relative",
            graph.path
        )));
    }
    if path.contains('/') {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{}\"` must target an immediate module-owned Mermaid file",
            graph.path
        )));
    }
    let has_mermaid_extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mmd"));
    if !has_mermaid_extension {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{}\"` must target a `.mmd` file",
            graph.path
        )));
    }
    if path
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
    {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{}\"` must not contain glob syntax",
            graph.path
        )));
    }

    Ok(())
}

fn validate_graph_contract_name(graph: &FlowhubGraphContract) -> Result<(), QianjiError> {
    let Some(name) = graph.name.as_deref() else {
        return Ok(());
    };
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]].name` must be non-empty for path `{}`",
            graph.path
        )));
    }
    if trimmed.contains('\n') || trimmed.contains('\r') {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]].name` must stay on one line for path `{}`",
            graph.path
        )));
    }

    Ok(())
}

fn validate_graph_node_contracts(graph: &FlowhubGraphContract) -> Result<(), QianjiError> {
    let mut labels = BTreeSet::new();
    for node in &graph.node {
        validate_graph_node_contract(node, &graph.path)?;
        if !labels.insert(node.label.as_str()) {
            return Err(QianjiError::Topology(format!(
                "Flowhub module manifest `[[graph]] path = \"{}\"` contains duplicate `[[graph.node]] label = \"{}\"`",
                graph.path, node.label
            )));
        }
    }

    Ok(())
}

fn validate_graph_workdir_contract(graph: &FlowhubGraphContract) -> Result<(), QianjiError> {
    let Some(workdir) = &graph.workdir else {
        return Ok(());
    };

    if let Some(note) = workdir.note.as_deref() {
        validate_graph_workdir_field(note, &graph.path, "[graph.workdir].note", "non-empty")?;
    }

    validate_graph_workdir_root(workdir.root.as_str(), &graph.path, "[graph.workdir].root")?;
    if let Some(target) = &workdir.target {
        validate_graph_workdir_surface(target, &graph.path, "[graph.workdir].target")?;
    }
    let plan_name = graph.resolved_workdir_name().ok_or_else(|| {
        QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{}\"` requires `[graph.workdir]` to derive a stable bounded workdir name from the graph path or declared graph name",
            graph.path
        ))
    })?;

    let manifest = WorkdirManifest {
        version: 1,
        plan: WorkdirPlan {
            name: plan_name.to_string(),
            surface: derive_graph_workdir_surface_entries(&workdir.check.require),
        },
        check: workdir.check.clone(),
    };
    validate_workdir_manifest(&manifest).map_err(|error| {
        QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{}\"` requires `[graph.workdir]` to derive a valid bounded work-surface manifest: {error}",
            graph.path
        ))
    })?;

    Ok(())
}

fn validate_graph_workdir_surface(
    surface: &FlowhubGraphSurfaceContract,
    graph_path: &str,
    field_name: &str,
) -> Result<(), QianjiError> {
    validate_graph_workdir_root(
        surface.root.as_str(),
        graph_path,
        &format!("{field_name}.root"),
    )?;
    validate_graph_workdir_paths(
        surface.paths.as_slice(),
        graph_path,
        &format!("{field_name}.paths"),
        true,
    )
}

fn derive_graph_workdir_surface_entries(require: &[String]) -> Vec<String> {
    let mut surfaces = Vec::new();
    let mut seen = BTreeSet::new();

    for entry in require {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }

        let top_level = trimmed.split('/').next().unwrap_or(trimmed);
        if top_level == "qianji.toml" {
            continue;
        }

        if seen.insert(top_level.to_string()) {
            surfaces.push(top_level.to_string());
        }
    }

    surfaces
}

fn validate_graph_node_contract(
    node: &FlowhubGraphNodeContract,
    graph_path: &str,
) -> Result<(), QianjiError> {
    validate_graph_node_contract_field(
        node.label.as_str(),
        graph_path,
        "[[graph.node]].label",
        "non-empty",
    )?;
    validate_graph_node_contract_field(
        node.kind.as_str(),
        graph_path,
        "[[graph.node]].kind",
        "non-empty",
    )?;
    validate_graph_node_contract_field(
        node.role.as_str(),
        graph_path,
        "[[graph.node]].role",
        "non-empty",
    )?;
    validate_graph_node_contract_field(
        node.agent_action.as_str(),
        graph_path,
        "[[graph.node]].agent_action",
        "non-empty",
    )?;
    Ok(())
}

fn validate_graph_workdir_paths(
    paths: &[String],
    graph_path: &str,
    field_name: &str,
    require_entries: bool,
) -> Result<(), QianjiError> {
    if require_entries && paths.is_empty() {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} to contain at least one path"
        )));
    }

    let mut seen = BTreeSet::new();
    for path in paths {
        let normalized = validate_graph_workdir_path(path, graph_path, field_name)?;
        if !seen.insert(normalized.clone()) {
            return Err(QianjiError::Topology(format!(
                "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` contains duplicate {field_name} entry `{normalized}`"
            )));
        }
    }

    Ok(())
}

fn validate_graph_workdir_root(
    value: &str,
    graph_path: &str,
    field_name: &str,
) -> Result<(), QianjiError> {
    validate_graph_workdir_field(value, graph_path, field_name, "non-empty single-line text")?;
    let trimmed = value.trim();
    if trimmed.starts_with('/') || trimmed.contains('\\') {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} to stay symbolic and relative"
        )));
    }
    if trimmed.ends_with('/') {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} without a trailing slash"
        )));
    }
    if trimmed == "." || trimmed == ".." {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} to be a stable symbolic root"
        )));
    }

    Ok(())
}

fn validate_graph_workdir_path(
    value: &str,
    graph_path: &str,
    field_name: &str,
) -> Result<String, QianjiError> {
    validate_graph_workdir_field(value, graph_path, field_name, "non-empty single-line text")?;
    let trimmed = value.trim();
    if trimmed.starts_with('/') || trimmed.contains('\\') {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} entries to stay relative"
        )));
    }
    if trimmed
        .chars()
        .any(|character| matches!(character, '*' | '?' | '[' | ']'))
    {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} entries without glob metacharacters"
        )));
    }

    let normalized = trimmed.trim_end_matches('/');
    if normalized.is_empty() {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} entries to be stable relative paths"
        )));
    }

    for segment in normalized.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(QianjiError::Topology(format!(
                "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} entries with valid relative path segments"
            )));
        }
    }

    Ok(normalized.to_string())
}

fn validate_graph_workdir_field(
    value: &str,
    graph_path: &str,
    field_name: &str,
    requirement: &str,
) -> Result<(), QianjiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} to be {requirement}"
        )));
    }
    if trimmed.contains('\n') || trimmed.contains('\r') {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} to stay on one line"
        )));
    }

    Ok(())
}

fn validate_graph_node_contract_field(
    value: &str,
    graph_path: &str,
    field_name: &str,
    requirement: &str,
) -> Result<(), QianjiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} to be {requirement}"
        )));
    }
    if trimmed.contains('\n') || trimmed.contains('\r') {
        return Err(QianjiError::Topology(format!(
            "Flowhub module manifest `[[graph]] path = \"{graph_path}\"` requires {field_name} to stay on one line"
        )));
    }

    Ok(())
}
