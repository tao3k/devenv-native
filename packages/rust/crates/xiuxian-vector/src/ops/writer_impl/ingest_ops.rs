impl VectorStore {
    /// Batch add documents with vectors to a table.
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
        use lance::deps::arrow_array::RecordBatchIterator;
        use std::collections::BTreeMap;

        if ids.is_empty() {
            return Ok(());
        }
        if ids.len() != vectors.len() || ids.len() != contents.len() || ids.len() != metadatas.len()
        {
            return Err(VectorStoreError::General(
                "Mismatched input lengths for ids/vectors/contents/metadatas".to_string(),
            ));
        }

        let partition_values: Vec<String> = metadatas
            .iter()
            .map(|s| {
                parse_metadata_value(s)
                    .and_then(|v| {
                        v.get(partition_by)
                            .and_then(|x| x.as_str())
                            .map(String::from)
                    })
                    .unwrap_or_else(|| "_unknown".to_string())
            })
            .collect();

        let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, pv) in partition_values.into_iter().enumerate() {
            groups.entry(pv).or_default().push(i);
        }

        let (mut dataset, _) = self.get_or_create_dataset(table_name, false, None).await?;
        let schema = self.create_schema();

        for (_partition_value, indices) in groups {
            let part_ids: Vec<String> = indices.iter().map(|&i| ids[i].clone()).collect();
            let part_vectors: Vec<Vec<f32>> = indices.iter().map(|&i| vectors[i].clone()).collect();
            let part_contents: Vec<String> = indices.iter().map(|&i| contents[i].clone()).collect();
            let part_metadatas: Vec<String> =
                indices.iter().map(|&i| metadatas[i].clone()).collect();

            let (_, batch) =
                self.build_document_batch(part_ids, part_vectors, part_contents, part_metadatas)?;
            let batches: Vec<Result<_, crate::error::ArrowError>> = vec![Ok(batch)];
            dataset
                .append(
                    Box::new(RecordBatchIterator::new(batches, schema.clone())),
                    Some(default_write_params()),
                )
                .await?;
        }

        self.invalidate_cached_table(table_name).await;
        Ok(())
    }
}
