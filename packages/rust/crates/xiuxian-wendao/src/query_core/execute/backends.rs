use std::sync::Arc;

use arrow::array::{ArrayRef, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use async_trait::async_trait;

use crate::link_graph::{LinkGraphDirection, LinkGraphIndex};
use crate::query_core::context::{GraphBackend, RetrievalBackend};
use crate::query_core::operators::{
    GraphDirection, GraphNeighborsOp, PayloadFetchOp, RetrievalCorpus, VectorSearchOp,
};
use crate::query_core::types::{WendaoQueryCoreError, WendaoRelation};
use crate::search::SearchPlaneService;
use crate::search::contracts::SearchHit;

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

        let mut node_ids = vec![op.node_id.clone()];
        let mut paths = vec![center.path.clone()];
        let mut titles = vec![Some(center.title.clone())];
        let mut distances = vec![0_u64];
        let mut directions = vec!["center".to_string()];

        for neighbor in neighbors {
            node_ids.push(neighbor.stem);
            paths.push(neighbor.path);
            titles.push(Some(neighbor.title));
            distances.push(u64::try_from(neighbor.distance).unwrap_or(u64::MAX));
            directions.push(graph_direction_label(op.direction).to_string());
        }

        let schema = Arc::new(Schema::new(vec![
            Field::new("node_id", DataType::Utf8, false),
            Field::new("path", DataType::Utf8, false),
            Field::new("title", DataType::Utf8, true),
            Field::new("distance", DataType::UInt64, false),
            Field::new("direction", DataType::Utf8, false),
        ]));
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
        Ok(WendaoRelation::new(schema, vec![batch]))
    }
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
