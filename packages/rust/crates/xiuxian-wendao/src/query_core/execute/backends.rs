//! `query_core::execute::backends` owns Wendao query core execute backends behavior.

use std::{collections::HashMap, sync::Arc};

use arrow::array::{ArrayRef, StringArray, UInt64Array};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

use crate::link_graph::{LinkGraphDirection, LinkGraphIndex};
use crate::query_core::context::{GraphBackend, RetrievalBackend};
use crate::query_core::operators::{
    GraphDirection, GraphNeighborsOp, PayloadFetchOp, RetrievalCorpus, VectorSearchOp,
};
use crate::query_core::types::{WendaoQueryCoreError, WendaoRelation};
use crate::search::SearchPlaneService;
use crate::search::contracts::SearchHit;

const GRAPH_NEIGHBORS_RELATION_TABLE: &str = "query_core_graph_neighbors";

/// Retrieval backend that delegates to the existing Wendao search plane.
pub struct SearchPlaneRetrievalBackend {
    service: Arc<SearchPlaneService>,
}

impl SearchPlaneRetrievalBackend {
    /// Create a retrieval adapter over the existing search-plane service.
    #[must_use]
    pub fn new(service: Arc<SearchPlaneService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl RetrievalBackend for SearchPlaneRetrievalBackend {
    async fn vector_search(
        &self,
        op: &VectorSearchOp,
    ) -> Result<WendaoRelation, WendaoQueryCoreError> {
        let hits = match op.corpus {
            RetrievalCorpus::RepoContent => self
                .service
                .search_repo_content_chunks(
                    op.repo_id.as_str(),
                    op.search_term.as_str(),
                    &op.language_filters,
                    op.limit,
                )
                .await
                .map_err(|error| WendaoQueryCoreError::Backend(error.to_string()))?,
            RetrievalCorpus::RepoEntity => self
                .service
                .search_repo_entities(
                    op.repo_id.as_str(),
                    op.search_term.as_str(),
                    &op.language_filters,
                    &op.kind_filters,
                    op.limit,
                )
                .await
                .map_err(|error| WendaoQueryCoreError::Backend(error.to_string()))?,
        };
        let rows = hits
            .into_iter()
            .map(|hit| retrieval_row_from_search_hit(&hit, op.repo_id.as_str()))
            .collect::<Vec<_>>();
        let batch = xiuxian_db_store::retrieval_rows_to_record_batch(&rows)?;
        Ok(WendaoRelation::new(batch.schema(), vec![batch]))
    }

    async fn payload_fetch(
        &self,
        relation: &WendaoRelation,
        op: &PayloadFetchOp,
    ) -> Result<WendaoRelation, WendaoQueryCoreError> {
        let batches = relation
            .batches()
            .iter()
            .map(|batch| {
                xiuxian_db_store::payload_fetch_record_batch(batch, &op.columns, op.ids.as_ref())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let schema = batches
            .first()
            .map(xiuxian_db_store::EngineRecordBatch::schema)
            .ok_or_else(|| WendaoQueryCoreError::InvalidRelation("missing payload batch".into()))?;
        Ok(WendaoRelation::new(schema, batches))
    }
}

/// Graph backend that delegates to the existing `link_graph` index.
pub struct LinkGraphNeighborsBackend {
    index: Arc<LinkGraphIndex>,
}

impl LinkGraphNeighborsBackend {
    /// Create a graph adapter over an existing `LinkGraphIndex`.
    #[must_use]
    pub fn new(index: Arc<LinkGraphIndex>) -> Self {
        Self { index }
    }
}

#[async_trait]
impl GraphBackend for LinkGraphNeighborsBackend {
    async fn graph_neighbors(
        &self,
        op: &GraphNeighborsOp,
    ) -> Result<WendaoRelation, WendaoQueryCoreError> {
        let direction = match op.direction {
            GraphDirection::Incoming => LinkGraphDirection::Incoming,
            GraphDirection::Outgoing => LinkGraphDirection::Outgoing,
            GraphDirection::Both => LinkGraphDirection::Both,
        };
        let center = self.index.metadata(op.node_id.as_str()).ok_or_else(|| {
            WendaoQueryCoreError::Backend(format!("graph node `{}` not found", op.node_id))
        })?;
        let neighbors = self
            .index
            .neighbors(op.node_id.as_str(), direction, op.hops, op.limit);

        let rows = std::iter::once((
            op.node_id.clone(),
            center.path.clone(),
            Some(center.title.clone()),
            0_u64,
            "center".to_string(),
        ))
        .chain(neighbors.into_iter().map(|neighbor| {
            (
                neighbor.stem,
                neighbor.path,
                Some(neighbor.title),
                u64::try_from(neighbor.distance).unwrap_or(u64::MAX),
                graph_direction_label(op.direction).to_string(),
            )
        }))
        .collect::<Vec<_>>();
        let node_ids = rows
            .iter()
            .map(|(node_id, _, _, _, _)| node_id.clone())
            .collect::<Vec<_>>();
        let paths = rows
            .iter()
            .map(|(_, path, _, _, _)| path.clone())
            .collect::<Vec<_>>();
        let titles = rows
            .iter()
            .map(|(_, _, title, _, _)| title.clone())
            .collect::<Vec<_>>();
        let distances = rows
            .iter()
            .map(|(_, _, _, distance, _)| *distance)
            .collect::<Vec<_>>();
        let directions = rows
            .iter()
            .map(|(_, _, _, _, direction)| direction.clone())
            .collect::<Vec<_>>();

        let contract = graph_neighbors_relation_contract();
        let schema = graph_neighbors_relation_schema_ref(&contract);
        let batch = RecordBatch::try_new(
            Arc::clone(&schema),
            vec![
                Arc::new(StringArray::from(node_ids)) as ArrayRef,
                Arc::new(StringArray::from(paths)) as ArrayRef,
                Arc::new(StringArray::from(titles)) as ArrayRef,
                Arc::new(UInt64Array::from(distances)) as ArrayRef,
                Arc::new(StringArray::from(directions)) as ArrayRef,
            ],
        )?;
        validate_graph_neighbors_relation_batch(&batch, &contract)?;
        Ok(WendaoRelation::new(schema, vec![batch]))
    }
}

fn graph_neighbors_relation_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        GRAPH_NEIGHBORS_RELATION_TABLE,
        true,
        vec![
            utf8_column("node_id"),
            utf8_column("path"),
            nullable_utf8_column("title"),
            uint64_column("distance"),
            utf8_column("direction"),
        ],
    )
}

fn graph_neighbors_relation_schema_ref(contract: &ArrowSchemaContract) -> SchemaRef {
    let mut metadata = HashMap::new();
    metadata.insert(
        WENDAO_TABLE_METADATA_KEY.to_string(),
        contract.table_name().to_string(),
    );
    Arc::new(build_arrow_schema(contract, metadata))
}

fn validate_graph_neighbors_relation_batch(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
) -> Result<(), WendaoQueryCoreError> {
    validate_record_batch_schema_with_options(
        batch,
        contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| {
        WendaoQueryCoreError::InvalidRelation(format!(
            "validate query-core graph-neighbor relation schema contract: {error}"
        ))
    })
}

fn utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

fn nullable_utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

fn uint64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::UInt64)
}

fn retrieval_row_from_search_hit(hit: &SearchHit, repo_id: &str) -> xiuxian_db_store::RetrievalRow {
    xiuxian_db_store::RetrievalRow {
        id: hit.stem.clone(),
        path: hit.path.clone(),
        repo: Some(repo_id.to_string()),
        title: hit.title.clone(),
        score: Some(hit.score),
        source: "legacy-search-plane".to_string(),
        snippet: hit.best_section.clone(),
        #[cfg(feature = "vector-store")]
        doc_type: hit.doc_type.clone(),
        #[cfg(not(feature = "vector-store"))]
        doc_type: hit.doc_type.clone().map(Into::into),
        match_reason: hit.match_reason.clone(),
        best_section: hit.best_section.clone(),
        language: hit
            .tags
            .iter()
            .find_map(|tag| tag.strip_prefix("lang:").map(str::to_string)),
        line: hit
            .navigation_target
            .as_ref()
            .and_then(|target| target.line)
            .map(|line| u64::try_from(line).unwrap_or(u64::MAX)),
    }
}

fn graph_direction_label(direction: GraphDirection) -> &'static str {
    match direction {
        GraphDirection::Incoming => "incoming",
        GraphDirection::Outgoing => "outgoing",
        GraphDirection::Both => "both",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GRAPH_NEIGHBORS_RELATION_TABLE, graph_neighbors_relation_contract,
        graph_neighbors_relation_schema_ref,
    };
    use xiuxian_db_store::WENDAO_TABLE_METADATA_KEY;

    #[test]
    fn graph_neighbors_relation_schema_uses_db_store_table_metadata() {
        let contract = graph_neighbors_relation_contract();
        let schema = graph_neighbors_relation_schema_ref(&contract);

        assert_eq!(
            schema
                .metadata()
                .get(WENDAO_TABLE_METADATA_KEY)
                .map(String::as_str),
            Some(GRAPH_NEIGHBORS_RELATION_TABLE)
        );
        assert_eq!(schema.field(0).name(), "node_id");
    }
}
