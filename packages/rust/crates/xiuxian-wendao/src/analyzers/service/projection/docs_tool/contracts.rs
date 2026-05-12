//! `analyzers::service::projection::docs_tool::contracts` owns Wendao projection docs tool contracts behavior.

use std::collections::BTreeMap;

#[cfg(test)]
use anyhow::{Context, Result};
use schemars::JsonSchema;
#[cfg(test)]
use schemars::schema_for;
use serde::{Deserialize, Serialize};

use crate::analyzers::ProjectionPageKind;

#[cfg(test)]
use super::options::{DEFAULT_DOCS_FAMILY_LIMIT, DEFAULT_DOCS_RELATED_LIMIT};

/// Stable contract identifier for the docs search capability.
pub const DOCS_SEARCH_CONTRACT_ID: &str = "wendao.docs.search";
/// Stable contract identifier for the docs document capability.
pub const DOCS_DOCUMENT_CONTRACT_ID: &str = "wendao.docs.document";
/// Stable contract identifier for the docs page-index-tree capability.
pub const DOCS_PAGE_INDEX_TREE_CONTRACT_ID: &str = "wendao.docs.page_index_tree";
/// Stable contract identifier for the docs navigation capability.
pub const DOCS_NAVIGATION_CONTRACT_ID: &str = "wendao.docs.navigation";
/// Stable contract identifier for the docs retrieval-context capability.
pub const DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID: &str = "wendao.docs.retrieval_context";
/// Ordered Wendao docs contracts exposed for Qianji consumption.
pub const DOCS_CONTRACT_IDS: &[&str] = &[
    DOCS_SEARCH_CONTRACT_ID,
    DOCS_DOCUMENT_CONTRACT_ID,
    DOCS_PAGE_INDEX_TREE_CONTRACT_ID,
    DOCS_NAVIGATION_CONTRACT_ID,
    DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID,
];

#[cfg(test)]
const CONTRACTS_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/resources/contracts");

#[cfg(test)]
const DOCS_SEARCH_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/manifests/wendao.docs.search.toml"
));
const DOCS_SEARCH_CONTRACT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.docs.search/contract.toml"
));
const DOCS_SEARCH_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.docs.search/schema.json"
));

#[cfg(test)]
const DOCS_DOCUMENT_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/manifests/wendao.docs.document.toml"
));
const DOCS_DOCUMENT_CONTRACT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.docs.document/contract.toml"
));
const DOCS_DOCUMENT_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.docs.document/schema.json"
));

#[cfg(test)]
const DOCS_PAGE_INDEX_TREE_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/manifests/wendao.docs.page_index_tree.toml"
));
const DOCS_PAGE_INDEX_TREE_CONTRACT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.docs.page_index_tree/contract.toml"
));
const DOCS_PAGE_INDEX_TREE_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.docs.page_index_tree/schema.json"
));

#[cfg(test)]
const DOCS_NAVIGATION_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/manifests/wendao.docs.navigation.toml"
));
const DOCS_NAVIGATION_CONTRACT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.docs.navigation/contract.toml"
));
const DOCS_NAVIGATION_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.docs.navigation/schema.json"
));

#[cfg(test)]
const DOCS_RETRIEVAL_CONTEXT_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/manifests/wendao.docs.retrieval_context.toml"
));
const DOCS_RETRIEVAL_CONTEXT_CONTRACT_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.docs.retrieval_context/contract.toml"
));
const DOCS_RETRIEVAL_CONTEXT_SCHEMA_JSON: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/contracts/snapshots/wendao.docs.retrieval_context/schema.json"
));

/// Stable tool arguments for docs search native-tool execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsSearchToolArgs {
    /// User-provided docs-facing projected page search string.
    pub query: String,
    /// Optional projected-page family filter.
    #[serde(default)]
    pub kind: Option<ProjectionPageKind>,
    /// Optional result-limit override.
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Stable tool arguments for docs document native-tool execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsDocumentToolArgs {
    /// Stable docs-facing page identifier.
    pub page_id: String,
}

/// Stable tool arguments for docs page-index-tree native-tool execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPageIndexTreeToolArgs {
    /// Stable docs-facing page identifier.
    pub page_id: String,
}

/// Stable tool arguments for docs navigation native-tool execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsNavigationToolArgs {
    /// Stable docs-facing page identifier.
    pub page_id: String,
    /// Optional stable page-index node identifier.
    #[serde(default)]
    pub node_id: Option<String>,
    /// Optional family cluster expansion kind.
    #[serde(default)]
    pub family_kind: Option<ProjectionPageKind>,
    /// Optional related-page limit override.
    #[serde(default)]
    pub related_limit: Option<usize>,
    /// Optional family-cluster limit override.
    #[serde(default)]
    pub family_limit: Option<usize>,
}

/// Stable tool arguments for docs retrieval-context native-tool execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsRetrievalContextToolArgs {
    /// Stable docs-facing page identifier.
    pub page_id: String,
    /// Optional stable page-index node identifier.
    #[serde(default)]
    pub node_id: Option<String>,
    /// Optional related-page limit override.
    #[serde(default)]
    pub related_limit: Option<usize>,
}

/// Raw checked-in assets for one Wendao docs contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocsCapabilityContractAssets {
    /// Checked-in invocation-first `contract.toml` content.
    pub contract_toml: &'static str,
    /// Checked-in `schema.json` content for strict input validation.
    pub schema_json: &'static str,
}

/// Minimal invocation contract snapshot consumed by downstream runtimes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocsCapabilityContractSnapshot {
    /// Stable contract identifier.
    pub id: String,
    /// Checked-in contract version.
    pub version: u32,
    /// Supported Qianji node kinds for this contract.
    pub task_types: Vec<String>,
    /// HTTP invocation surface.
    pub http: DocsHttpContractSnapshot,
    /// CLI invocation surface.
    pub cli: DocsCliContractSnapshot,
    /// Native tool surface.
    pub tool: DocsToolContractSnapshot,
    /// Canonical parameter list.
    pub params: Vec<DocsContractParamSnapshot>,
}

/// HTTP invocation surface for one docs capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocsHttpContractSnapshot {
    /// HTTP method.
    pub method: String,
    /// Stable gateway path.
    pub path: String,
    /// Ordered query parameter names.
    pub query: Vec<String>,
}

/// CLI invocation surface for one docs capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocsCliContractSnapshot {
    /// Fixed command argv prefix.
    pub argv: Vec<String>,
    /// Canonical parameter to CLI flag mapping.
    pub flags: BTreeMap<String, String>,
}

/// Native tool surface for one docs capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocsToolContractSnapshot {
    /// Native tool identifier.
    pub name: String,
    /// Sibling schema asset filename.
    pub schema: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Parameters injected by the runtime instead of the tool caller.
    pub runtime_injected: Vec<String>,
}

/// Canonical parameter description for the invocation contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct DocsContractParamSnapshot {
    /// Canonical parameter name.
    pub name: String,
    #[serde(rename = "type")]
    /// Minimal scalar type hint used by the contract surface.
    pub value_type: String,
    #[serde(default)]
    /// Whether the parameter is mandatory for authored invocations.
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional literal default value.
    pub default: Option<DocsContractDefaultValue>,
}

/// Minimal literal default value surface kept in `contract.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DocsContractDefaultValue {
    /// Integer literal default.
    Integer(usize),
    /// String literal default.
    String(String),
    /// Boolean literal default.
    Boolean(bool),
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct DocsCapabilityManifest {
    id: String,
    version: u32,
    task_types: Vec<String>,
    http: DocsHttpManifest,
    cli: DocsCliManifest,
    tool: DocsToolManifest,
    params: Vec<DocsContractParamManifest>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct DocsHttpManifest {
    method: String,
    path: String,
    query: Vec<String>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct DocsCliManifest {
    argv: Vec<String>,
    flags: BTreeMap<String, String>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct DocsToolManifest {
    name: String,
    schema_provider: String,
    #[serde(default)]
    runtime_injected: Vec<String>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct DocsContractParamManifest {
    name: String,
    #[serde(rename = "type")]
    value_type: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    default: Option<DocsContractDefaultValue>,
}

/// Resolve the raw checked-in assets for one docs contract id.
#[must_use]
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn docs_capability_contract_assets(contract_id: &str) -> Option<DocsCapabilityContractAssets> {
    match contract_id {
        DOCS_SEARCH_CONTRACT_ID => Some(DocsCapabilityContractAssets {
            contract_toml: DOCS_SEARCH_CONTRACT_TOML,
            schema_json: DOCS_SEARCH_SCHEMA_JSON,
        }),
        DOCS_DOCUMENT_CONTRACT_ID => Some(DocsCapabilityContractAssets {
            contract_toml: DOCS_DOCUMENT_CONTRACT_TOML,
            schema_json: DOCS_DOCUMENT_SCHEMA_JSON,
        }),
        DOCS_PAGE_INDEX_TREE_CONTRACT_ID => Some(DocsCapabilityContractAssets {
            contract_toml: DOCS_PAGE_INDEX_TREE_CONTRACT_TOML,
            schema_json: DOCS_PAGE_INDEX_TREE_SCHEMA_JSON,
        }),
        DOCS_NAVIGATION_CONTRACT_ID => Some(DocsCapabilityContractAssets {
            contract_toml: DOCS_NAVIGATION_CONTRACT_TOML,
            schema_json: DOCS_NAVIGATION_SCHEMA_JSON,
        }),
        DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID => Some(DocsCapabilityContractAssets {
            contract_toml: DOCS_RETRIEVAL_CONTEXT_CONTRACT_TOML,
            schema_json: DOCS_RETRIEVAL_CONTEXT_SCHEMA_JSON,
        }),
        _ => None,
    }
}

/// Resolve the raw checked-in `contract.toml` for one docs contract id.
#[must_use]
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn docs_capability_contract_snapshot(contract_id: &str) -> Option<&'static str> {
    docs_capability_contract_assets(contract_id).map(|assets| assets.contract_toml)
}

/// Resolve the raw checked-in `schema.json` for one docs contract id.
#[must_use]
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub fn docs_capability_schema_snapshot(contract_id: &str) -> Option<&'static str> {
    docs_capability_contract_assets(contract_id).map(|assets| assets.schema_json)
}

#[cfg(test)]
fn docs_capability_manifest(contract_id: &str) -> Option<&'static str> {
    match contract_id {
        DOCS_SEARCH_CONTRACT_ID => Some(DOCS_SEARCH_MANIFEST),
        DOCS_DOCUMENT_CONTRACT_ID => Some(DOCS_DOCUMENT_MANIFEST),
        DOCS_PAGE_INDEX_TREE_CONTRACT_ID => Some(DOCS_PAGE_INDEX_TREE_MANIFEST),
        DOCS_NAVIGATION_CONTRACT_ID => Some(DOCS_NAVIGATION_MANIFEST),
        DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID => Some(DOCS_RETRIEVAL_CONTEXT_MANIFEST),
        _ => None,
    }
}

#[cfg(test)]
fn parse_manifest(contract_id: &str) -> Result<DocsCapabilityManifest> {
    let raw = docs_capability_manifest(contract_id)
        .with_context(|| format!("missing docs contract manifest for `{contract_id}`"))?;
    toml::from_str(raw)
        .with_context(|| format!("failed to parse docs contract manifest `{contract_id}`"))
}

#[cfg(test)]
fn build_snapshot(manifest: &DocsCapabilityManifest) -> DocsCapabilityContractSnapshot {
    DocsCapabilityContractSnapshot {
        id: manifest.id.clone(),
        version: manifest.version,
        task_types: manifest.task_types.clone(),
        http: DocsHttpContractSnapshot {
            method: manifest.http.method.clone(),
            path: manifest.http.path.clone(),
            query: manifest.http.query.clone(),
        },
        cli: DocsCliContractSnapshot {
            argv: manifest.cli.argv.clone(),
            flags: manifest.cli.flags.clone(),
        },
        tool: DocsToolContractSnapshot {
            name: manifest.tool.name.clone(),
            schema: "schema.json".to_string(),
            runtime_injected: manifest.tool.runtime_injected.clone(),
        },
        params: manifest
            .params
            .iter()
            .map(|param| DocsContractParamSnapshot {
                name: param.name.clone(),
                value_type: param.value_type.clone(),
                required: param.required,
                default: param.default.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
fn generate_snapshot_contract_toml(contract_id: &str) -> Result<String> {
    let manifest = parse_manifest(contract_id)?;
    validate_manifest(&manifest)?;
    let mut rendered = toml::to_string_pretty(&build_snapshot(&manifest))
        .with_context(|| format!("failed to serialize contract snapshot `{contract_id}`"))?;
    if !rendered.ends_with('\n') {
        rendered.push('\n');
    }
    Ok(rendered)
}

#[cfg(test)]
fn generate_schema_json(contract_id: &str) -> Result<String> {
    let manifest = parse_manifest(contract_id)?;
    let schema = match manifest.tool.schema_provider.as_str() {
        "DocsSearchToolArgs" => serde_json::to_string_pretty(&schema_for!(DocsSearchToolArgs))
            .context("failed to serialize DocsSearchToolArgs schema")?,
        "DocsDocumentToolArgs" => serde_json::to_string_pretty(&schema_for!(DocsDocumentToolArgs))
            .context("failed to serialize DocsDocumentToolArgs schema")?,
        "DocsPageIndexTreeToolArgs" => {
            serde_json::to_string_pretty(&schema_for!(DocsPageIndexTreeToolArgs))
                .context("failed to serialize DocsPageIndexTreeToolArgs schema")?
        }
        "DocsNavigationToolArgs" => {
            serde_json::to_string_pretty(&schema_for!(DocsNavigationToolArgs))
                .context("failed to serialize DocsNavigationToolArgs schema")?
        }
        "DocsRetrievalContextToolArgs" => {
            serde_json::to_string_pretty(&schema_for!(DocsRetrievalContextToolArgs))
                .context("failed to serialize DocsRetrievalContextToolArgs schema")?
        }
        other => anyhow::bail!("unknown docs schema provider `{other}`"),
    };
    Ok(format!("{schema}\n"))
}

#[cfg(test)]
fn validate_manifest(manifest: &DocsCapabilityManifest) -> Result<()> {
    let expected = expected_contract_shape(manifest.id.as_str())
        .with_context(|| format!("unsupported docs contract `{}`", manifest.id))?;
    if manifest.version != 1 {
        anyhow::bail!(
            "docs contract `{}` must stay on version 1, got {}",
            manifest.id,
            manifest.version
        );
    }
    if manifest.task_types != expected.task_types {
        anyhow::bail!("docs contract `{}` task_types drifted", manifest.id);
    }
    if manifest.http != expected.http {
        anyhow::bail!("docs contract `{}` http surface drifted", manifest.id);
    }
    if manifest.cli != expected.cli {
        anyhow::bail!("docs contract `{}` cli surface drifted", manifest.id);
    }
    if manifest.tool != expected.tool {
        anyhow::bail!("docs contract `{}` tool surface drifted", manifest.id);
    }
    if manifest.params != expected.params {
        anyhow::bail!("docs contract `{}` params drifted", manifest.id);
    }
    Ok(())
}

#[cfg(test)]
fn expected_contract_shape(contract_id: &str) -> Option<DocsCapabilityManifest> {
    match contract_id {
        DOCS_SEARCH_CONTRACT_ID => Some(search_contract_shape()),
        DOCS_DOCUMENT_CONTRACT_ID => Some(document_contract_shape()),
        DOCS_PAGE_INDEX_TREE_CONTRACT_ID => Some(page_index_tree_contract_shape()),
        DOCS_NAVIGATION_CONTRACT_ID => Some(navigation_contract_shape()),
        DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID => Some(retrieval_context_contract_shape()),
        _ => None,
    }
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct DocsContractShapeSpec<'a> {
    id: &'a str,
    http_path: &'a str,
    http_query: &'a [&'a str],
    cli_argv: &'a [&'a str],
    cli_flags: &'a [(&'a str, &'a str)],
    tool_name: &'a str,
    schema_provider: &'a str,
}

#[cfg(test)]
fn search_contract_shape() -> DocsCapabilityManifest {
    docs_contract_manifest(
        DocsContractShapeSpec {
            id: DOCS_SEARCH_CONTRACT_ID,
            http_path: crate::gateway::API_DOCS_SEARCH_OPENAPI_PATH,
            http_query: &["repo", "query", "kind", "limit"],
            cli_argv: &["wendao", "docs", "search"],
            cli_flags: &[
                ("kind", "--kind"),
                ("limit", "--limit"),
                ("query", "--query"),
                ("repo", "--repo"),
            ],
            tool_name: "wendao.docs.search",
            schema_provider: "DocsSearchToolArgs",
        },
        vec![
            required_string_param("repo"),
            required_string_param("query"),
            optional_string_param("kind"),
            optional_integer_param("limit", 10),
        ],
    )
}

#[cfg(test)]
fn document_contract_shape() -> DocsCapabilityManifest {
    docs_contract_manifest(
        DocsContractShapeSpec {
            id: DOCS_DOCUMENT_CONTRACT_ID,
            http_path: crate::gateway::API_DOCS_PAGE_OPENAPI_PATH,
            http_query: &["repo", "page_id"],
            cli_argv: &["wendao", "docs", "page"],
            cli_flags: &[("page_id", "--page-id"), ("repo", "--repo")],
            tool_name: "wendao.docs.get_document",
            schema_provider: "DocsDocumentToolArgs",
        },
        vec![
            required_string_param("repo"),
            required_string_param("page_id"),
        ],
    )
}

#[cfg(test)]
fn page_index_tree_contract_shape() -> DocsCapabilityManifest {
    docs_contract_manifest(
        DocsContractShapeSpec {
            id: DOCS_PAGE_INDEX_TREE_CONTRACT_ID,
            http_path: crate::gateway::API_DOCS_PAGE_INDEX_TREE_OPENAPI_PATH,
            http_query: &["repo", "page_id"],
            cli_argv: &["wendao", "docs", "tree"],
            cli_flags: &[("page_id", "--page-id"), ("repo", "--repo")],
            tool_name: "wendao.docs.get_page_index_tree",
            schema_provider: "DocsPageIndexTreeToolArgs",
        },
        vec![
            required_string_param("repo"),
            required_string_param("page_id"),
        ],
    )
}

#[cfg(test)]
fn navigation_contract_shape() -> DocsCapabilityManifest {
    docs_contract_manifest(
        DocsContractShapeSpec {
            id: DOCS_NAVIGATION_CONTRACT_ID,
            http_path: crate::gateway::API_DOCS_NAVIGATION_OPENAPI_PATH,
            http_query: &[
                "repo",
                "page_id",
                "node_id",
                "family_kind",
                "related_limit",
                "family_limit",
            ],
            cli_argv: &["wendao", "docs", "navigation"],
            cli_flags: &[
                ("family_kind", "--family-kind"),
                ("family_limit", "--family-limit"),
                ("node_id", "--node-id"),
                ("page_id", "--page-id"),
                ("related_limit", "--related-limit"),
                ("repo", "--repo"),
            ],
            tool_name: "wendao.docs.get_navigation",
            schema_provider: "DocsNavigationToolArgs",
        },
        vec![
            required_string_param("repo"),
            required_string_param("page_id"),
            optional_string_param("node_id"),
            optional_string_param("family_kind"),
            optional_integer_param("related_limit", DEFAULT_DOCS_RELATED_LIMIT),
            optional_integer_param("family_limit", DEFAULT_DOCS_FAMILY_LIMIT),
        ],
    )
}

#[cfg(test)]
fn retrieval_context_contract_shape() -> DocsCapabilityManifest {
    docs_contract_manifest(
        DocsContractShapeSpec {
            id: DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID,
            http_path: crate::gateway::API_DOCS_RETRIEVAL_CONTEXT_OPENAPI_PATH,
            http_query: &["repo", "page_id", "node_id", "related_limit"],
            cli_argv: &["wendao", "docs", "context"],
            cli_flags: &[
                ("node_id", "--node-id"),
                ("page_id", "--page-id"),
                ("related_limit", "--related-limit"),
                ("repo", "--repo"),
            ],
            tool_name: "wendao.docs.get_retrieval_context",
            schema_provider: "DocsRetrievalContextToolArgs",
        },
        vec![
            required_string_param("repo"),
            required_string_param("page_id"),
            optional_string_param("node_id"),
            optional_integer_param("related_limit", DEFAULT_DOCS_RELATED_LIMIT),
        ],
    )
}

#[cfg(test)]
fn docs_contract_manifest(
    spec: DocsContractShapeSpec<'_>,
    params: Vec<DocsContractParamManifest>,
) -> DocsCapabilityManifest {
    DocsCapabilityManifest {
        id: spec.id.to_string(),
        version: 1,
        task_types: vec!["http_call".to_string(), "cli_call".to_string()],
        http: DocsHttpManifest {
            method: "GET".to_string(),
            path: spec.http_path.to_string(),
            query: spec
                .http_query
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        },
        cli: DocsCliManifest {
            argv: spec
                .cli_argv
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            flags: spec
                .cli_flags
                .iter()
                .map(|(name, flag)| ((*name).to_string(), (*flag).to_string()))
                .collect(),
        },
        tool: DocsToolManifest {
            name: spec.tool_name.to_string(),
            schema_provider: spec.schema_provider.to_string(),
            runtime_injected: vec!["repo".to_string()],
        },
        params,
    }
}

#[cfg(test)]
fn required_string_param(name: &str) -> DocsContractParamManifest {
    DocsContractParamManifest {
        name: name.to_string(),
        value_type: "string".to_string(),
        required: true,
        default: None,
    }
}

#[cfg(test)]
fn optional_string_param(name: &str) -> DocsContractParamManifest {
    DocsContractParamManifest {
        name: name.to_string(),
        value_type: "string".to_string(),
        required: false,
        default: None,
    }
}

#[cfg(test)]
fn optional_integer_param(name: &str, default: usize) -> DocsContractParamManifest {
    DocsContractParamManifest {
        name: name.to_string(),
        value_type: "integer".to_string(),
        required: false,
        default: Some(DocsContractDefaultValue::Integer(default)),
    }
}

#[cfg(test)]
fn contract_snapshot_path(contract_id: &str) -> String {
    format!("{CONTRACTS_ROOT}/snapshots/{contract_id}/contract.toml")
}

#[cfg(test)]
fn schema_snapshot_path(contract_id: &str) -> String {
    format!("{CONTRACTS_ROOT}/snapshots/{contract_id}/schema.json")
}

#[cfg(test)]
fn snapshot_root_path() -> &'static str {
    concat!(env!("CARGO_MANIFEST_DIR"), "/resources/contracts/snapshots")
}

#[cfg(test)]
#[path = "../../../../../tests/unit/analyzers/service/projection/docs_tool/contracts.rs"]
mod tests;
