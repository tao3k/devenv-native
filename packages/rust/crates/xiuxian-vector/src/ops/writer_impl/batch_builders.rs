impl VectorStore {
    fn build_document_batch(
        &self,
        ids: Vec<String>,
        vectors: Vec<Vec<f32>>,
        contents: Vec<String>,
        metadatas: Vec<String>,
    ) -> Result<
        (
            Arc<lance::deps::arrow_schema::Schema>,
            lance::deps::arrow_array::RecordBatch,
        ),
        VectorStoreError,
    > {
        use lance::deps::arrow_array::StringArray;

        let list_dimension = validate_document_batch_inputs(
            ids.len(),
            &vectors,
            contents.len(),
            metadatas.len(),
            self.dimension,
        )?;
        let metadata_columns = parse_document_metadata_columns(&metadatas, &ids)?;
        let id_array = StringArray::from(ids);
        let content_array = StringArray::from(contents);
        let vector_array = build_vector_list_array(vectors, list_dimension)?;
        let metadata_array = StringArray::from(metadatas);

        let schema = self.create_schema();
        let batch = lance::deps::arrow_array::RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array),
                Arc::new(vector_array),
                Arc::new(content_array),
                Arc::new(metadata_columns.skill_name),
                Arc::new(metadata_columns.category),
                Arc::new(metadata_columns.tool_name),
                Arc::new(metadata_columns.file_path),
                Arc::new(metadata_columns.routing_keywords),
                Arc::new(metadata_columns.intents),
                Arc::new(metadata_array),
            ],
        )
        .map_err(VectorStoreError::Arrow)?;
        Ok((schema, batch))
    }
}
