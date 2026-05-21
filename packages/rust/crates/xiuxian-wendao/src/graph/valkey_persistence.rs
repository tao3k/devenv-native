//! Valkey persistence for `KnowledgeGraph` entities and relations.
//!
//! Persists the full graph snapshot as JSON under a deterministic Valkey key
//! derived from caller-provided graph scope. This keeps wendao runtime-native
//! and avoids `LanceDB` coupling in the graph storage path.

use super::core::read_lock;
use super::{GraphError, KnowledgeGraph};
use crate::entity::{Entity, Relation};
use crate::settings::{get_setting_string, merged_wendao_settings};
use crate::valkey_common::{normalize_key_prefix, open_client};
use chrono::Utc;
use log::info;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use xiuxian_config_core::toml_first_named_string;
use xxhash_rust::xxh3::xxh3_64;

const GRAPH_VALKEY_URL_SETTING: &str = "graph.persistence.valkey_url";
const GRAPH_VALKEY_KEY_PREFIX_SETTING: &str = "graph.persistence.key_prefix";
const GRAPH_VALKEY_URL_ENV: &str = "XIUXIAN_WENDAO_GRAPH_VALKEY_URL";
const GRAPH_VALKEY_KEY_PREFIX_ENV: &str = "XIUXIAN_WENDAO_GRAPH_VALKEY_KEY_PREFIX";
const DEFAULT_GRAPH_VALKEY_KEY_PREFIX: &str = "xiuxian_wendao:graph";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GraphSnapshot {
    schema_version: u32,
    dimension: usize,
    saved_at_rfc3339: String,
    entities: Vec<Entity>,
    relations: Vec<Relation>,
}

fn resolve_graph_valkey_url() -> Result<String, GraphError> {
    let settings = merged_wendao_settings();
    resolve_graph_valkey_url_with_settings_and_lookup(&settings, &|name| std::env::var(name).ok())
}

fn resolve_graph_key_prefix() -> String {
    let settings = merged_wendao_settings();
    resolve_graph_key_prefix_with_settings_and_lookup(&settings, &|name| std::env::var(name).ok())
}

fn resolve_graph_valkey_url_with_settings_and_lookup(
    settings: &Value,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<String, GraphError> {
    toml_first_named_string(
        GRAPH_VALKEY_URL_SETTING,
        get_setting_string(settings, GRAPH_VALKEY_URL_SETTING),
        lookup,
        &[GRAPH_VALKEY_URL_ENV, "VALKEY_URL"],
    )
    .map(|candidate| candidate.value)
    .ok_or_else(|| {
        GraphError::InvalidRelation(
            GRAPH_VALKEY_URL_SETTING.to_string(),
            format!(
                "graph valkey url is required (set {GRAPH_VALKEY_URL_SETTING}, {GRAPH_VALKEY_URL_ENV}, or VALKEY_URL)"
            ),
        )
    })
}

fn resolve_graph_key_prefix_with_settings_and_lookup(
    settings: &Value,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> String {
    normalize_graph_key_prefix(
        toml_first_named_string(
            GRAPH_VALKEY_KEY_PREFIX_SETTING,
            get_setting_string(settings, GRAPH_VALKEY_KEY_PREFIX_SETTING),
            lookup,
            &[GRAPH_VALKEY_KEY_PREFIX_ENV],
        )
        .map(|candidate| candidate.value)
        .unwrap_or_default()
        .as_str(),
    )
}

fn normalize_graph_key_prefix(candidate: &str) -> String {
    normalize_key_prefix(candidate, DEFAULT_GRAPH_VALKEY_KEY_PREFIX)
}

fn graph_redis_client(valkey_url: &str) -> Result<redis::Client, GraphError> {
    open_client(valkey_url).map_err(|error| {
        GraphError::InvalidRelation("graph_valkey_client".to_string(), error.to_string())
    })
}

fn graph_snapshot_key(graph_scope: &str) -> String {
    let prefix = resolve_graph_key_prefix();
    let normalized_scope = graph_scope.trim();
    let hash = xxh3_64(normalized_scope.as_bytes());
    format!("{prefix}:snapshot:{hash:016x}")
}

impl KnowledgeGraph {
    /// Save graph snapshot to Valkey using blocking I/O.
    pub(crate) fn save_to_valkey_sync(
        &self,
        graph_scope: &str,
        dimension: usize,
    ) -> Result<(), GraphError> {
        let valkey_url = resolve_graph_valkey_url()?;
        let snapshot_key = graph_snapshot_key(graph_scope);
        let entities = {
            let guard = read_lock::<HashMap<String, Entity>>(&self.entities);
            guard.values().cloned().collect::<Vec<_>>()
        };
        let relations = {
            let guard = read_lock::<HashMap<String, Relation>>(&self.relations);
            guard.values().cloned().collect::<Vec<_>>()
        };
        let snapshot = GraphSnapshot {
            schema_version: 1,
            dimension,
            saved_at_rfc3339: Utc::now().to_rfc3339(),
            entities,
            relations,
        };
        let payload = serde_json::to_string(&snapshot).map_err(|error| {
            GraphError::InvalidRelation("graph_snapshot_serialize".to_string(), error.to_string())
        })?;

        let client = graph_redis_client(valkey_url.as_str())?;
        let mut conn = client.get_connection().map_err(|error| {
            GraphError::InvalidRelation("graph_valkey_connect".to_string(), error.to_string())
        })?;
        redis::cmd("SET")
            .arg(&snapshot_key)
            .arg(payload)
            .query::<()>(&mut conn)
            .map_err(|error| {
                GraphError::InvalidRelation("graph_valkey_set".to_string(), error.to_string())
            })?;

        let stats = self.get_stats();
        info!(
            "Knowledge graph saved to Valkey scope={} key={} ({} entities, {} relations)",
            graph_scope, snapshot_key, stats.total_entities, stats.total_relations
        );

        Ok(())
    }

    /// Save graph snapshot to Valkey.
    ///
    /// `graph_scope` is a logical namespace key; same scope overwrites the same snapshot.
    /// `dimension` is persisted for compatibility and diagnostics.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when required environment variables are missing, Valkey cannot be
    /// reached, or snapshot serialization fails.
    pub fn save_to_valkey(&self, graph_scope: &str, dimension: usize) -> Result<(), GraphError> {
        self.save_to_valkey_sync(graph_scope, dimension)
    }

    /// Load graph snapshot from Valkey using blocking I/O.
    pub(crate) fn load_from_valkey_sync(&mut self, graph_scope: &str) -> Result<(), GraphError> {
        let valkey_url = resolve_graph_valkey_url()?;
        let snapshot_key = graph_snapshot_key(graph_scope);

        let client = graph_redis_client(valkey_url.as_str())?;
        let mut conn = client.get_connection().map_err(|error| {
            GraphError::InvalidRelation("graph_valkey_connect".to_string(), error.to_string())
        })?;
        let payload: Option<String> = redis::cmd("GET")
            .arg(&snapshot_key)
            .query(&mut conn)
            .map_err(|error| {
                GraphError::InvalidRelation("graph_valkey_get".to_string(), error.to_string())
            })?;

        self.clear();
        let Some(payload) = payload else {
            return Ok(());
        };
        let snapshot: GraphSnapshot = serde_json::from_str(&payload).map_err(|error| {
            GraphError::InvalidRelation("graph_snapshot_parse".to_string(), error.to_string())
        })?;

        for entity in snapshot.entities {
            self.add_entity(entity)?;
        }
        for relation in snapshot.relations {
            self.add_relation(relation)?;
        }

        let stats = self.get_stats();
        info!(
            "Knowledge graph loaded from Valkey scope={} key={} ({} entities, {} relations)",
            graph_scope, snapshot_key, stats.total_entities, stats.total_relations
        );

        Ok(())
    }

    /// Load graph snapshot from Valkey.
    ///
    /// Replaces in-memory graph with stored snapshot if present.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when required environment variables are missing, Valkey operations
    /// fail, snapshot parsing fails, or restored graph entities/relations are invalid.
    pub fn load_from_valkey(&mut self, graph_scope: &str) -> Result<(), GraphError> {
        self.load_from_valkey_sync(graph_scope)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/graph/valkey_persistence.rs"]
mod tests;
