use std::sync::Arc;

#[cfg(test)]
use std::io::Cursor;

use crate::studio::arrow_types::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray, LanceUInt64Array,
};
#[cfg(test)]
use arrow::datatypes::{DataType, Field, Schema};
#[cfg(test)]
use arrow::ipc::writer::StreamWriter;

use crate::studio::types::{RetrievalChunk, RetrievalChunkSurface};

pub(crate) fn build_retrieval_chunks_flight_batch(
    chunks: &[RetrievalChunk],
) -> Result<LanceRecordBatch, String> {
    let owner_ids = chunks
        .iter()
        .map(|chunk| chunk.owner_id.clone())
        .collect::<Vec<_>>();
    let chunk_ids = chunks
        .iter()
        .map(|chunk| chunk.chunk_id.clone())
        .collect::<Vec<_>>();
    let semantic_types = chunks
        .iter()
        .map(|chunk| chunk.semantic_type.clone())
        .collect::<Vec<_>>();
    let fingerprints = chunks
        .iter()
        .map(|chunk| chunk.fingerprint.clone())
        .collect::<Vec<_>>();
    let token_estimates = chunks
        .iter()
        .map(|chunk| {
            u64::try_from(chunk.token_estimate)
                .map_err(|_| "tokenEstimate exceeds u64 range".to_string())
        })
        .collect::<Result<Vec<u64>, _>>()?;
    let display_labels = chunks
        .iter()
        .map(|chunk| chunk.display_label.clone())
        .collect::<Vec<_>>();
    let excerpts = chunks
        .iter()
        .map(|chunk| chunk.excerpt.clone())
        .collect::<Vec<_>>();
    let line_starts = chunks
        .iter()
        .map(|chunk| {
            chunk
                .line_start
                .map(|value| {
                    u64::try_from(value).map_err(|_| "lineStart exceeds u64 range".to_string())
                })
                .transpose()
        })
        .collect::<Result<Vec<Option<u64>>, _>>()?;
    let line_ends = chunks
        .iter()
        .map(|chunk| {
            chunk
                .line_end
                .map(|value| {
                    u64::try_from(value).map_err(|_| "lineEnd exceeds u64 range".to_string())
                })
                .transpose()
        })
        .collect::<Result<Vec<Option<u64>>, _>>()?;
    let surfaces = chunks
        .iter()
        .map(|chunk| {
            chunk
                .surface
                .map(retrieval_surface_label)
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    let attributes_json = chunks
        .iter()
        .map(|chunk| {
            if chunk.attributes.is_empty() {
                Ok(None)
            } else {
                serde_json::to_string(&chunk.attributes)
                    .map(Some)
                    .map_err(|error| error.to_string())
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    LanceRecordBatch::try_new(
        retrieval_chunks_flight_schema(),
        vec![
            Arc::new(LanceStringArray::from(owner_ids)),
            Arc::new(LanceStringArray::from(chunk_ids)),
            Arc::new(LanceStringArray::from(semantic_types)),
            Arc::new(LanceStringArray::from(fingerprints)),
            Arc::new(LanceUInt64Array::from(token_estimates)),
            Arc::new(LanceStringArray::from(display_labels)),
            Arc::new(LanceStringArray::from(excerpts)),
            Arc::new(LanceUInt64Array::from(line_starts)),
            Arc::new(LanceUInt64Array::from(line_ends)),
            Arc::new(LanceStringArray::from(surfaces)),
            Arc::new(LanceStringArray::from(attributes_json)),
        ],
    )
    .map_err(|error| error.to_string())
}

fn retrieval_chunks_flight_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new("ownerId", LanceDataType::Utf8, false),
        LanceField::new("chunkId", LanceDataType::Utf8, false),
        LanceField::new("semanticType", LanceDataType::Utf8, false),
        LanceField::new("fingerprint", LanceDataType::Utf8, false),
        LanceField::new("tokenEstimate", LanceDataType::UInt64, false),
        LanceField::new("displayLabel", LanceDataType::Utf8, true),
        LanceField::new("excerpt", LanceDataType::Utf8, true),
        LanceField::new("lineStart", LanceDataType::UInt64, true),
        LanceField::new("lineEnd", LanceDataType::UInt64, true),
        LanceField::new("surface", LanceDataType::Utf8, true),
        LanceField::new("attributesJson", LanceDataType::Utf8, true),
    ]))
}

#[cfg(test)]
pub(crate) fn encode_retrieval_chunks_ipc(chunks: &[RetrievalChunk]) -> Result<Vec<u8>, String> {
    let batch = build_retrieval_chunks_flight_batch(chunks)?;
    let schema = Schema::new(vec![
        Field::new("ownerId", DataType::Utf8, false),
        Field::new("chunkId", DataType::Utf8, false),
        Field::new("semanticType", DataType::Utf8, false),
        Field::new("fingerprint", DataType::Utf8, false),
        Field::new("tokenEstimate", DataType::UInt64, false),
        Field::new("displayLabel", DataType::Utf8, true),
        Field::new("excerpt", DataType::Utf8, true),
        Field::new("lineStart", DataType::UInt64, true),
        Field::new("lineEnd", DataType::UInt64, true),
        Field::new("surface", DataType::Utf8, true),
        Field::new("attributesJson", DataType::Utf8, true),
    ]);
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer =
            StreamWriter::try_new(&mut buffer, &schema).map_err(|error| error.to_string())?;
        writer.write(&batch).map_err(|error| error.to_string())?;
        writer.finish().map_err(|error| error.to_string())?;
    }
    Ok(buffer.into_inner())
}

const fn retrieval_surface_label(surface: RetrievalChunkSurface) -> &'static str {
    match surface {
        RetrievalChunkSurface::Document => "document",
        RetrievalChunkSurface::Section => "section",
        RetrievalChunkSurface::CodeBlock => "codeblock",
        RetrievalChunkSurface::Table => "table",
        RetrievalChunkSurface::Math => "math",
        RetrievalChunkSurface::Observation => "observation",
        RetrievalChunkSurface::Declaration => "declaration",
        RetrievalChunkSurface::Block => "block",
        RetrievalChunkSurface::Symbol => "symbol",
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/gateway/studio/router/retrieval_arrow.rs"]
mod tests;
