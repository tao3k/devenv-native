//! Object-model validation for Episteme extension contracts.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::ontology::manifest::{EpistemeOntologyManifest, resolve_ontology_artifact_path};

use super::{
    pathing::{has_cjk, read_to_string},
    rdf::ExtensionRdfTerms,
};

pub(super) struct ExtensionObjectModelReport {
    pub(super) objects: usize,
    pub(super) properties: usize,
    pub(super) links: usize,
    pub(super) actions: usize,
    pub(super) queries: usize,
}

#[derive(Debug, Deserialize)]
struct ObjectModelContract {
    schema_version: u32,
    ontology: String,
    compatibility: String,
    object_model_compatibility: String,
    boundaries: ObjectModelBoundaries,
    #[serde(default)]
    object_types: Vec<ObjectType>,
    #[serde(default)]
    property_types: Vec<PropertyType>,
    #[serde(default)]
    link_types: Vec<LinkType>,
    #[serde(default)]
    action_types: Vec<ActionType>,
    #[serde(default)]
    query_types: Vec<QueryType>,
    #[serde(default)]
    interface_types: Vec<InterfaceType>,
    #[serde(default)]
    object_set_recipes: Vec<ObjectSetRecipe>,
}

#[derive(Debug, Deserialize)]
struct ObjectModelBoundaries {
    artifact_mode: String,
    mutation_allowed: ContractFlag,
    runtime_compilation_owner: String,
    sdk_generation_owner: String,
    rdf_source_authority: ContractFlag,
    object_model_source_authority: ContractFlag,
    runtime_object_mutation_allowed: ContractFlag,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(from = "bool")]
enum ContractFlag {
    Enabled,
    Disabled,
}

impl From<bool> for ContractFlag {
    fn from(value: bool) -> Self {
        if value { Self::Enabled } else { Self::Disabled }
    }
}

#[derive(Debug, Deserialize)]
struct ObjectType {
    domain: String,
    api_name: String,
    display_name: String,
    plural_display_name: String,
    status: String,
    rdf_class: String,
    primary_key: Vec<String>,
    display_name_property: String,
    title_property: String,
    interfaces: Vec<String>,
    visibility: String,
}

#[derive(Debug, Deserialize)]
struct PropertyType {
    domain: String,
    object_type: String,
    api_name: String,
    display_name: String,
    value_type: String,
    required: bool,
    indexed: bool,
    search_policy: String,
}

#[derive(Debug, Deserialize)]
struct LinkType {
    domain: String,
    api_name: String,
    display_name: String,
    status: String,
    rdf_property: String,
    from_object_type: String,
    to_object_type: String,
    cardinality: String,
    from_api_name: String,
    to_api_name: String,
    inverse_api_name: String,
    foreign_key_property: String,
}

#[derive(Debug, Deserialize)]
struct ActionType {
    domain: String,
    api_name: String,
    display_name: String,
    status: String,
    affected_object_types: Vec<String>,
    requires_evidence: bool,
    validation_rules: Vec<String>,
    parameters: Vec<String>,
    operations: Vec<String>,
    tool_description: String,
}

#[derive(Debug, Deserialize)]
struct QueryType {
    domain: String,
    api_name: String,
    parameters: Vec<String>,
    returns: String,
    returns_kind: String,
    object_set_recipe: String,
}

#[derive(Debug, Deserialize)]
struct InterfaceType {
    api_name: String,
    implemented_by: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ObjectSetRecipe {
    domain: String,
    api_name: String,
    kind: String,
    #[serde(default)]
    object_type: Option<String>,
    #[serde(default)]
    base_object_type: Option<String>,
    #[serde(default)]
    link_type: Option<String>,
    #[serde(default)]
    target_object_type: Option<String>,
    #[serde(default)]
    allowed_methods: Vec<String>,
}

pub(super) fn validate_extension_object_models(
    episteme_root: &Path,
    manifest: &EpistemeOntologyManifest,
    rdf_terms: &ExtensionRdfTerms,
) -> Result<ExtensionObjectModelReport> {
    let domain_ids = manifest
        .domains
        .iter()
        .map(|domain| domain.id.as_str())
        .collect::<BTreeSet<_>>();
    let require_cjk_label = manifest
        .primary_language
        .as_deref()
        .is_some_and(|language| language.starts_with("zh"));
    let mut aggregate = ExtensionObjectModelReport {
        objects: 0,
        properties: 0,
        links: 0,
        actions: 0,
        queries: 0,
    };
    for domain in &manifest.domains {
        for contract_path in &domain.object_model_contracts {
            let path = resolve_ontology_artifact_path(
                episteme_root,
                contract_path,
                "object_model_contracts",
            )
            .map_err(|source| anyhow::anyhow!(source))?;
            let model = toml::from_str::<ObjectModelContract>(&read_to_string(path.as_path())?)
                .with_context(|| format!("failed to parse `{}`", path.display()))?;
            validate_object_model_shape(&model)?;
            validate_object_model_references(&model, &domain_ids, rdf_terms, require_cjk_label)?;
            aggregate.objects += model.object_types.len();
            aggregate.properties += model.property_types.len();
            aggregate.links += model.link_types.len();
            aggregate.actions += model.action_types.len();
            aggregate.queries += model.query_types.len();
        }
    }
    Ok(aggregate)
}

fn validate_object_model_shape(model: &ObjectModelContract) -> Result<()> {
    if model.schema_version != 1 {
        bail!(
            "unsupported object model schema_version: {}",
            model.schema_version
        );
    }
    if model.ontology.trim().is_empty() {
        bail!("object model ontology must not be blank");
    }
    if model.compatibility != "semantic_api_compatibility" {
        bail!("object model compatibility must be `semantic_api_compatibility`");
    }
    if model.object_model_compatibility != "foundry_style_object_model_v1" {
        bail!("object model compatibility must be `foundry_style_object_model_v1`");
    }
    if model.boundaries.artifact_mode != "extension_source_contract" {
        bail!("object model artifact_mode must be `extension_source_contract`");
    }
    if model.boundaries.mutation_allowed == ContractFlag::Enabled
        || model.boundaries.runtime_object_mutation_allowed == ContractFlag::Enabled
    {
        bail!("object model must not allow runtime/source mutation");
    }
    if model.boundaries.runtime_compilation_owner.trim().is_empty()
        || model.boundaries.sdk_generation_owner.trim().is_empty()
    {
        bail!("object model runtime and SDK owners must not be blank");
    }
    if model.boundaries.rdf_source_authority != ContractFlag::Enabled
        || model.boundaries.object_model_source_authority != ContractFlag::Enabled
    {
        bail!("object model must declare RDF and object-model source authority");
    }
    if model.object_types.is_empty() {
        bail!("object model must declare at least one object type");
    }
    Ok(())
}

fn validate_object_model_references(
    model: &ObjectModelContract,
    domain_ids: &BTreeSet<&str>,
    rdf_terms: &ExtensionRdfTerms,
    require_cjk_label: bool,
) -> Result<()> {
    let object_types = collect_object_types(model, domain_ids, rdf_terms, require_cjk_label)?;
    let properties_by_object =
        validate_property_types(model, &object_types, domain_ids, require_cjk_label)?;
    validate_object_property_bindings(&object_types, &properties_by_object)?;
    validate_links(
        model,
        &object_types,
        domain_ids,
        rdf_terms,
        require_cjk_label,
    )?;
    validate_actions(model, &object_types, domain_ids, require_cjk_label)?;
    validate_queries(model, &object_types, domain_ids)?;
    validate_interfaces(model, &object_types)?;
    validate_recipes(model, &object_types, domain_ids)?;
    Ok(())
}

fn collect_object_types<'model>(
    model: &'model ObjectModelContract,
    domain_ids: &BTreeSet<&str>,
    rdf_terms: &ExtensionRdfTerms,
    require_cjk_label: bool,
) -> Result<BTreeMap<&'model str, &'model ObjectType>> {
    let mut object_types = BTreeMap::<&str, &ObjectType>::new();
    for object in &model.object_types {
        validate_domain(domain_ids, object.domain.as_str(), "object_type.domain")?;
        require_api_name(object.api_name.as_str(), "object_type.api_name")?;
        require_label(
            object.display_name.as_str(),
            require_cjk_label,
            "object_type.display_name",
        )?;
        require_label(
            object.plural_display_name.as_str(),
            require_cjk_label,
            "object_type.plural_display_name",
        )?;
        require_known_status(object.status.as_str(), "object_type.status")?;
        if !rdf_terms.has_class(object.rdf_class.as_str()) {
            bail!(
                "object type `{}` references unknown RDF class `{}`",
                object.api_name,
                object.rdf_class
            );
        }
        if object.primary_key.is_empty() {
            bail!(
                "object type `{}` must declare a primary key",
                object.api_name
            );
        }
        if !matches!(object.visibility.as_str(), "public" | "private" | "hidden") {
            bail!(
                "object type `{}` has invalid visibility `{}`",
                object.api_name,
                object.visibility
            );
        }
        if object_types
            .insert(object.api_name.as_str(), object)
            .is_some()
        {
            bail!("duplicate object type api_name `{}`", object.api_name);
        }
    }
    Ok(object_types)
}

fn validate_property_types<'model>(
    model: &'model ObjectModelContract,
    object_types: &BTreeMap<&str, &ObjectType>,
    domain_ids: &BTreeSet<&str>,
    require_cjk_label: bool,
) -> Result<BTreeMap<&'model str, BTreeSet<&'model str>>> {
    let mut properties_by_object = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut property_keys = BTreeSet::<(&str, &str)>::new();
    for property in &model.property_types {
        validate_domain(domain_ids, property.domain.as_str(), "property_type.domain")?;
        require_object(
            object_types,
            property.object_type.as_str(),
            "property_type.object_type",
        )?;
        require_label(
            property.display_name.as_str(),
            require_cjk_label,
            "property_type.display_name",
        )?;
        if property.api_name.trim().is_empty() {
            bail!("property type api_name must not be blank");
        }
        if !property_keys.insert((property.object_type.as_str(), property.api_name.as_str())) {
            bail!(
                "duplicate property `{}` for object type `{}`",
                property.api_name,
                property.object_type
            );
        }
        require_enum(
            property.value_type.as_str(),
            &[
                "string",
                "integer",
                "double",
                "boolean",
                "date",
                "timestamp",
                "attachment",
                "object_reference",
            ],
            "property_type.value_type",
        )?;
        require_enum(
            property.search_policy.as_str(),
            &["none", "exact", "full_text", "range", "vector"],
            "property_type.search_policy",
        )?;
        let _ = property.required;
        let _ = property.indexed;
        properties_by_object
            .entry(property.object_type.as_str())
            .or_default()
            .insert(property.api_name.as_str());
    }
    Ok(properties_by_object)
}

fn validate_object_property_bindings(
    object_types: &BTreeMap<&str, &ObjectType>,
    properties_by_object: &BTreeMap<&str, BTreeSet<&str>>,
) -> Result<()> {
    for object in object_types.values() {
        let Some(properties) = properties_by_object.get(object.api_name.as_str()) else {
            bail!(
                "object type `{}` has no property definitions",
                object.api_name
            );
        };
        for key in &object.primary_key {
            if !properties.contains(key.as_str()) {
                bail!(
                    "object type `{}` primary key `{key}` has no matching property",
                    object.api_name
                );
            }
        }
        for property in [&object.display_name_property, &object.title_property] {
            if !properties.contains(property.as_str()) {
                bail!(
                    "object type `{}` display/title property `{property}` has no matching property",
                    object.api_name
                );
            }
        }
    }
    Ok(())
}

fn validate_links(
    model: &ObjectModelContract,
    object_types: &BTreeMap<&str, &ObjectType>,
    domain_ids: &BTreeSet<&str>,
    rdf_terms: &ExtensionRdfTerms,
    require_cjk_label: bool,
) -> Result<()> {
    let mut link_types = BTreeSet::new();
    for link in &model.link_types {
        validate_domain(domain_ids, link.domain.as_str(), "link_type.domain")?;
        require_label(
            link.display_name.as_str(),
            require_cjk_label,
            "link_type.display_name",
        )?;
        require_known_status(link.status.as_str(), "link_type.status")?;
        require_object(
            object_types,
            link.from_object_type.as_str(),
            "link_type.from_object_type",
        )?;
        require_object(
            object_types,
            link.to_object_type.as_str(),
            "link_type.to_object_type",
        )?;
        require_enum(
            link.cardinality.as_str(),
            &["one_to_one", "one_to_many", "many_to_one", "many_to_many"],
            "link_type.cardinality",
        )?;
        for (field, value) in [
            ("link_type.from_api_name", link.from_api_name.as_str()),
            ("link_type.to_api_name", link.to_api_name.as_str()),
            ("link_type.inverse_api_name", link.inverse_api_name.as_str()),
            (
                "link_type.foreign_key_property",
                link.foreign_key_property.as_str(),
            ),
        ] {
            if value.trim().is_empty() {
                bail!("{field} must not be blank for link `{}`", link.api_name);
            }
        }
        if !rdf_terms.has_object_property(link.rdf_property.as_str()) {
            bail!(
                "link type `{}` references unknown RDF object property `{}`",
                link.api_name,
                link.rdf_property
            );
        }
        if !link_types.insert(link.api_name.as_str()) {
            bail!("duplicate link type api_name `{}`", link.api_name);
        }
    }
    Ok(())
}

fn validate_actions(
    model: &ObjectModelContract,
    object_types: &BTreeMap<&str, &ObjectType>,
    domain_ids: &BTreeSet<&str>,
    require_cjk_label: bool,
) -> Result<()> {
    let mut action_types = BTreeSet::new();
    let link_types = model
        .link_types
        .iter()
        .map(|link| link.api_name.as_str())
        .collect::<BTreeSet<_>>();
    for action in &model.action_types {
        validate_domain(domain_ids, action.domain.as_str(), "action_type.domain")?;
        require_label(
            action.display_name.as_str(),
            require_cjk_label,
            "action_type.display_name",
        )?;
        require_known_status(action.status.as_str(), "action_type.status")?;
        if !action.requires_evidence {
            bail!("action type `{}` must require evidence", action.api_name);
        }
        if action.operations.is_empty() || action.parameters.is_empty() {
            bail!(
                "action type `{}` must declare parameters and operations",
                action.api_name
            );
        }
        if action.tool_description.trim().is_empty() {
            bail!(
                "action type `{}` tool_description must not be blank",
                action.api_name
            );
        }
        for object_type in &action.affected_object_types {
            require_object(
                object_types,
                object_type,
                "action_type.affected_object_types",
            )?;
        }
        for rule in &action.validation_rules {
            if rule.trim().is_empty() {
                bail!(
                    "action type `{}` has a blank validation rule",
                    action.api_name
                );
            }
        }
        for operation in &action.operations {
            validate_action_operation(operation, object_types, &link_types)?;
        }
        if !action_types.insert(action.api_name.as_str()) {
            bail!("duplicate action type api_name `{}`", action.api_name);
        }
    }
    Ok(())
}

fn validate_action_operation(
    operation: &str,
    object_types: &BTreeMap<&str, &ObjectType>,
    link_types: &BTreeSet<&str>,
) -> Result<()> {
    let Some((kind, target)) = operation.split_once(':') else {
        bail!("action operation must use `kind:target`: {operation}");
    };
    match kind {
        "create_object" => require_object(object_types, target, "action_type.operation"),
        "create_link" => {
            if link_types.contains(target) {
                Ok(())
            } else {
                bail!("action operation references unknown link type `{target}`")
            }
        }
        _ => bail!("unsupported action operation kind `{kind}`"),
    }
}

fn validate_queries(
    model: &ObjectModelContract,
    object_types: &BTreeMap<&str, &ObjectType>,
    domain_ids: &BTreeSet<&str>,
) -> Result<()> {
    let recipes = model
        .object_set_recipes
        .iter()
        .map(|recipe| recipe.api_name.as_str())
        .collect::<BTreeSet<_>>();
    let mut query_types = BTreeSet::new();
    for query in &model.query_types {
        validate_domain(domain_ids, query.domain.as_str(), "query_type.domain")?;
        if query.parameters.is_empty() {
            bail!("query type `{}` must declare parameters", query.api_name);
        }
        require_object(object_types, query.returns.as_str(), "query_type.returns")?;
        require_enum(
            query.returns_kind.as_str(),
            &["object", "object_set"],
            "query_type.returns_kind",
        )?;
        if !recipes.contains(query.object_set_recipe.as_str()) {
            bail!(
                "query type `{}` references unknown object_set_recipe `{}`",
                query.api_name,
                query.object_set_recipe
            );
        }
        if !query_types.insert(query.api_name.as_str()) {
            bail!("duplicate query type api_name `{}`", query.api_name);
        }
    }
    Ok(())
}

fn validate_interfaces(
    model: &ObjectModelContract,
    object_types: &BTreeMap<&str, &ObjectType>,
) -> Result<()> {
    let mut interfaces = BTreeSet::new();
    for interface in &model.interface_types {
        if !interfaces.insert(interface.api_name.as_str()) {
            bail!("duplicate interface type api_name `{}`", interface.api_name);
        }
        for object_type in &interface.implemented_by {
            require_object(object_types, object_type, "interface_type.implemented_by")?;
        }
    }
    for object in &model.object_types {
        for interface in &object.interfaces {
            if !interfaces.contains(interface.as_str()) {
                bail!(
                    "object type `{}` references unknown interface `{interface}`",
                    object.api_name
                );
            }
        }
    }
    Ok(())
}

fn validate_recipes(
    model: &ObjectModelContract,
    object_types: &BTreeMap<&str, &ObjectType>,
    domain_ids: &BTreeSet<&str>,
) -> Result<()> {
    let link_types = model
        .link_types
        .iter()
        .map(|link| link.api_name.as_str())
        .collect::<BTreeSet<_>>();
    let mut recipes = BTreeSet::new();
    for recipe in &model.object_set_recipes {
        validate_domain(
            domain_ids,
            recipe.domain.as_str(),
            "object_set_recipe.domain",
        )?;
        if recipe.allowed_methods.is_empty() {
            bail!(
                "object set recipe `{}` must declare allowed methods",
                recipe.api_name
            );
        }
        match recipe.kind.as_str() {
            "base" | "filter" => {
                let object_type = recipe.object_type.as_deref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "object set recipe `{}` must declare object_type",
                        recipe.api_name
                    )
                })?;
                require_object(object_types, object_type, "object_set_recipe.object_type")?;
            }
            "link" => {
                let base = recipe.base_object_type.as_deref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "object set recipe `{}` must declare base_object_type",
                        recipe.api_name
                    )
                })?;
                let target = recipe.target_object_type.as_deref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "object set recipe `{}` must declare target_object_type",
                        recipe.api_name
                    )
                })?;
                require_object(object_types, base, "object_set_recipe.base_object_type")?;
                require_object(object_types, target, "object_set_recipe.target_object_type")?;
                let link = recipe.link_type.as_deref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "object set recipe `{}` must declare link_type",
                        recipe.api_name
                    )
                })?;
                if !link_types.contains(link) {
                    bail!(
                        "object set recipe `{}` references unknown link type `{link}`",
                        recipe.api_name
                    );
                }
            }
            value => bail!(
                "object set recipe `{}` has unsupported kind `{value}`",
                recipe.api_name
            ),
        }
        if !recipes.insert(recipe.api_name.as_str()) {
            bail!("duplicate object set recipe api_name `{}`", recipe.api_name);
        }
    }
    Ok(())
}

fn validate_domain(domain_ids: &BTreeSet<&str>, value: &str, field: &str) -> Result<()> {
    if domain_ids.contains(value) {
        Ok(())
    } else {
        bail!("{field} references unknown domain `{value}`")
    }
}

fn require_object(
    object_types: &BTreeMap<&str, &ObjectType>,
    value: &str,
    field: &str,
) -> Result<()> {
    if object_types.contains_key(value) {
        Ok(())
    } else {
        bail!("{field} references unknown object type `{value}`")
    }
}

fn require_label(value: &str, require_cjk_label: bool, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{field} must not be blank");
    }
    if require_cjk_label && !has_cjk(value) {
        bail!("{field} must contain a Chinese label");
    }
    Ok(())
}

fn require_api_name(value: &str, field: &str) -> Result<()> {
    if value
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
        && value.chars().all(|ch| ch.is_ascii_alphanumeric())
    {
        Ok(())
    } else {
        bail!("{field} must be a PascalCase ASCII identifier: {value}")
    }
}

fn require_known_status(value: &str, field: &str) -> Result<()> {
    require_enum(value, &["active", "preview", "deprecated"], field)
}

fn require_enum(value: &str, allowed: &[&str], field: &str) -> Result<()> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        bail!("{field} has unsupported value `{value}`")
    }
}
