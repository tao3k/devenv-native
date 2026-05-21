use super::{
    Result, VectorStore, VectorStoreError, default_write_params, parse_metadata_value,
    validate_document_batch_inputs,
};

impl VectorStore {
    /// Batch add documents with vectors to a table.
    ///
    /// Positional boundary: this public writer API keeps the long-standing
    /// document component surface where callers provide parallel id, vector,
    /// content, and metadata arrays.
    ///
    /// # Errors
    ///
    /// Returns an error when input validation fails, dataset create/append fails,
    /// or `Arrow` batch construction fails.
    pub async fn add_documents(
        &self,
        table_name: &str,
        ids: Vec<String>,
        vectors: Vec<Vec<f32>>,
        contents: Vec<String>,
        metadatas: Vec<String>,
    ) -> Result<(), VectorStoreError> {
        use lance::deps::arrow_array::RecordBatchIterator;

        if ids.is_empty() {
            return Ok(());
        }

        let (schema, batch) = self.build_document_batch(ids, vectors, contents, metadatas)?;

        let (mut dataset, created) = self
            .get_or_create_dataset(table_name, false, Some((schema.clone(), batch.clone())))
            .await?;
        if !created {
            dataset
                .append(
                    Box::new(RecordBatchIterator::new(vec![Ok(batch)], schema)),
                    Some(default_write_params()),
                )
                .await?;
        }

        self.invalidate_cached_table(table_name).await;
        Ok(())
    }

    /// Add documents with rows grouped by a partition column so fragments align by partition
    /// (enables partition pruning at read). Partition value is read from each row's metadata JSON.
    ///
    /// Positional boundary: partitioned ingest mirrors [`Self::add_documents`]
    /// and adds only the explicit partition selector needed by Lance writes.
    ///
    /// # Errors
    ///
    /// Returns an error when input validation fails, partitioned append fails,
    /// or `Arrow` batch construction fails.
    pub async fn add_documents_partitioned(
        &self,
        table_name: &str,
        partition_by: &str,
        ids: Vec<String>,
        vectors: Vec<Vec<f32>>,
        contents: Vec<String>,
        metadatas: Vec<String>,
    ) -> Result<(), VectorStoreError> {
        if ids.is_empty() {
            return Ok(());
        }
        validate_document_batch_inputs(
            ids.len(),
            vectors.as_slice(),
            contents.len(),
            metadatas.len(),
            self.dimension,
        )?;

        let (mut dataset, _) = self.get_or_create_dataset(table_name, false, None).await?;
        let schema = self.create_schema();
        let groups = partitioned_document_groups(partition_by, metadatas.as_slice());
        self.append_partitioned_document_groups(
            &mut dataset,
            schema,
            PartitionedDocumentColumns {
                ids,
                vectors,
                contents,
                metadatas,
            },
            groups,
        )
        .await?;

        self.invalidate_cached_table(table_name).await;
        Ok(())
    }

    async fn append_partitioned_document_groups(
        &self,
        dataset: &mut lance::dataset::Dataset,
        schema: std::sync::Arc<lance::deps::arrow_schema::Schema>,
        columns: PartitionedDocumentColumns,
        groups: std::collections::BTreeMap<String, Vec<usize>>,
    ) -> Result<(), VectorStoreError> {
        use lance::deps::arrow_array::RecordBatchIterator;

        for indices in groups.into_values() {
            let batch = self.partitioned_document_batch(&columns, indices.as_slice())?;
            let batches: Vec<Result<_, crate::error::ArrowError>> = vec![Ok(batch)];
            dataset
                .append(
                    Box::new(RecordBatchIterator::new(batches, schema.clone())),
                    Some(default_write_params()),
                )
                .await?;
        }
        Ok(())
    }

    fn partitioned_document_batch(
        &self,
        columns: &PartitionedDocumentColumns,
        indices: &[usize],
    ) -> Result<lance::deps::arrow_array::RecordBatch, VectorStoreError> {
        let part_ids = indices
            .iter()
            .map(|&index| columns.ids[index].clone())
            .collect();
        let part_vectors = indices
            .iter()
            .map(|&index| columns.vectors[index].clone())
            .collect();
        let part_contents = indices
            .iter()
            .map(|&index| columns.contents[index].clone())
            .collect();
        let part_metadatas = indices
            .iter()
            .map(|&index| columns.metadatas[index].clone())
            .collect();
        self.build_document_batch(part_ids, part_vectors, part_contents, part_metadatas)
            .map(|(_, batch)| batch)
    }
}

struct PartitionedDocumentColumns {
    ids: Vec<String>,
    vectors: Vec<Vec<f32>>,
    contents: Vec<String>,
    metadatas: Vec<String>,
}

fn partitioned_document_groups(
    partition_by: &str,
    metadatas: &[String],
) -> std::collections::BTreeMap<String, Vec<usize>> {
    metadatas.iter().enumerate().fold(
        std::collections::BTreeMap::new(),
        |mut groups, (index, metadata)| {
            groups
                .entry(partition_value(partition_by, metadata))
                .or_default()
                .push(index);
            groups
        },
    )
}

fn partition_value(partition_by: &str, metadata: &str) -> String {
    parse_metadata_value(metadata)
        .and_then(|value| {
            value
                .get(partition_by)
                .and_then(|field| field.as_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "_unknown".to_string())
}
