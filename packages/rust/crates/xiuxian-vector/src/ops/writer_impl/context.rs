// Writer method context shared by ingest and dataset lifecycle operations.

use lance::dataset::WriteParams;
use lance::deps::arrow_array::types::Int32Type;

/// Write params for new tables and appends: `V2_1` storage for better encoding/compression.
fn default_write_params() -> WriteParams {
    WriteParams {
        data_storage_version: Some(lance_file::version::LanceFileVersion::V2_1),
        ..WriteParams::default()
    }
}

/// Build dictionary-encoded columns for low-cardinality `SKILL_NAME` and `CATEGORY`.
fn build_dictionary_columns(
    skill_names: &[String],
    categories: &[String],
) -> Result<
    (
        lance::deps::arrow_array::DictionaryArray<Int32Type>,
        lance::deps::arrow_array::DictionaryArray<Int32Type>,
    ),
    VectorStoreError,
> {
    use lance::deps::arrow_array::{DictionaryArray, Int32Array, StringArray};

    let mut uniq_skill: Vec<String> = Vec::new();
    let mut map_skill: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    for s in skill_names {
        if !map_skill.contains_key(s) {
            let idx = i32::try_from(uniq_skill.len()).map_err(|_| {
                VectorStoreError::General(
                    "skill_name dictionary exceeds i32 key capacity".to_string(),
                )
            })?;
            map_skill.insert(s.clone(), idx);
            uniq_skill.push(s.clone());
        }
    }
    let keys_skill: Vec<i32> = skill_names
        .iter()
        .map(|s| {
            map_skill.get(s).copied().ok_or_else(|| {
                VectorStoreError::General(format!("missing skill_name dictionary key for '{s}'"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let values_skill = StringArray::from(uniq_skill);
    let skill_name_array = DictionaryArray::<Int32Type>::try_new(
        Int32Array::from(keys_skill),
        std::sync::Arc::new(values_skill),
    )
    .map_err(VectorStoreError::Arrow)?;

    let mut uniq_cat: Vec<String> = Vec::new();
    let mut map_cat: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    for c in categories {
        if !map_cat.contains_key(c) {
            let idx = i32::try_from(uniq_cat.len()).map_err(|_| {
                VectorStoreError::General(
                    "category dictionary exceeds i32 key capacity".to_string(),
                )
            })?;
            map_cat.insert(c.clone(), idx);
            uniq_cat.push(c.clone());
        }
    }
    let keys_cat: Vec<i32> = categories
        .iter()
        .map(|c| {
            map_cat.get(c).copied().ok_or_else(|| {
                VectorStoreError::General(format!("missing category dictionary key for '{c}'"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let values_cat = StringArray::from(uniq_cat);
    let category_array = DictionaryArray::<Int32Type>::try_new(
        Int32Array::from(keys_cat),
        std::sync::Arc::new(values_cat),
    )
    .map_err(VectorStoreError::Arrow)?;

    Ok((skill_name_array, category_array))
}

/// Build a single dictionary-encoded column from string values (e.g. `TOOL_NAME`).
fn build_string_dictionary(
    values: &[String],
) -> Result<lance::deps::arrow_array::DictionaryArray<Int32Type>, VectorStoreError> {
    use lance::deps::arrow_array::{DictionaryArray, Int32Array, StringArray};

    let mut uniq: Vec<String> = Vec::new();
    let mut map: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    for s in values {
        if !map.contains_key(s) {
            let idx = i32::try_from(uniq.len()).map_err(|_| {
                VectorStoreError::General(
                    "tool_name dictionary exceeds i32 key capacity".to_string(),
                )
            })?;
            map.insert(s.clone(), idx);
            uniq.push(s.clone());
        }
    }
    let keys: Vec<i32> = values
        .iter()
        .map(|s| {
            map.get(s).copied().ok_or_else(|| {
                VectorStoreError::General(format!("missing tool_name dictionary key for '{s}'"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let value_arr = StringArray::from(uniq);
    DictionaryArray::<Int32Type>::try_new(Int32Array::from(keys), std::sync::Arc::new(value_arr))
        .map_err(VectorStoreError::Arrow)
}

/// Parse JSON for metadata extraction.
#[inline]
fn parse_metadata_extract(s: &str) -> MetadataExtract {
    serde_json::from_str(s).unwrap_or_default()
}

/// Parse JSON to `Value`.
#[inline]
fn parse_metadata_value(s: &str) -> Option<serde_json::Value> {
    serde_json::from_str(s).ok()
}

/// Single-pass metadata extraction for Arrow-native columns (avoids full Value tree).
#[derive(serde::Deserialize, Default)]
struct MetadataExtract {
    #[serde(default)]
    skill_name: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    file_path: Option<String>,
    #[serde(default)]
    routing_keywords: Vec<String>,
    #[serde(default)]
    intents: Vec<String>,
}

fn has_lance_data(path: &std::path::Path) -> bool {
    if !path.exists() {
        return false;
    }
    path.join("_versions").exists() || path.join("data").exists()
}

struct ParsedMetadataColumns {
    skill_name: lance::deps::arrow_array::DictionaryArray<Int32Type>,
    category: lance::deps::arrow_array::DictionaryArray<Int32Type>,
    tool_name: lance::deps::arrow_array::DictionaryArray<Int32Type>,
    file_path: lance::deps::arrow_array::StringArray,
    routing_keywords: lance::deps::arrow_array::ListArray,
    intents: lance::deps::arrow_array::ListArray,
}

fn validate_document_batch_inputs(
    ids_len: usize,
    vectors: &[Vec<f32>],
    contents_len: usize,
    metadatas_len: usize,
    dimension: usize,
) -> Result<i32, VectorStoreError> {
    if ids_len == 0 {
        return Err(VectorStoreError::General(
            "Cannot build record batch from empty ids".to_string(),
        ));
    }
    if ids_len != vectors.len() || ids_len != contents_len || ids_len != metadatas_len {
        return Err(VectorStoreError::General(
            "Mismatched input lengths for ids/vectors/contents/metadatas".to_string(),
        ));
    }
    if vectors[0].len() != dimension {
        return Err(VectorStoreError::InvalidDimension {
            expected: dimension,
            actual: vectors[0].len(),
        });
    }
    if vectors.iter().any(|vector| vector.len() != dimension) {
        return Err(VectorStoreError::General(
            "All vectors must match store dimension".to_string(),
        ));
    }
    i32::try_from(dimension).map_err(|_| {
        VectorStoreError::General(format!("vector dimension {dimension} exceeds i32 range"))
    })
}

fn build_vector_list_array(
    vectors: Vec<Vec<f32>>,
    list_dimension: i32,
) -> Result<lance::deps::arrow_array::FixedSizeListArray, VectorStoreError> {
    use lance::deps::arrow_array::{FixedSizeListArray, Float32Array};

    let flat_values: Vec<f32> = vectors.into_iter().flatten().collect();
    FixedSizeListArray::try_new(
        Arc::new(lance::deps::arrow_schema::Field::new(
            "item",
            lance::deps::arrow_schema::DataType::Float32,
            true,
        )),
        list_dimension,
        Arc::new(Float32Array::from(flat_values)),
        None,
    )
    .map_err(VectorStoreError::Arrow)
}

fn parse_document_metadata_columns(
    metadatas: &[String],
    ids: &[String],
) -> Result<ParsedMetadataColumns, VectorStoreError> {
    use lance::deps::arrow_array::StringArray;
    use lance::deps::arrow_array::builder::{ListBuilder, StringBuilder};

    let extracts: Vec<MetadataExtract> = metadatas
        .iter()
        .map(|metadata| parse_metadata_extract(metadata))
        .collect();
    let skill_names: Vec<String> = extracts
        .iter()
        .map(|extract| extract.skill_name.clone().unwrap_or_default())
        .collect();
    let categories: Vec<String> = extracts
        .iter()
        .map(|extract| extract.category.clone().unwrap_or_default())
        .collect();
    let (skill_name, category) = build_dictionary_columns(&skill_names, &categories)?;

    let tool_names: Vec<String> = extracts
        .iter()
        .zip(ids.iter())
        .map(|(extract, id)| {
            extract
                .tool_name
                .clone()
                .or_else(|| extract.command.clone())
                .unwrap_or_else(|| id.clone())
        })
        .collect();
    let tool_name = build_string_dictionary(&tool_names)?;

    let file_path = StringArray::from(
        extracts
            .iter()
            .map(|extract| extract.file_path.clone().unwrap_or_default())
            .collect::<Vec<_>>(),
    );
    let mut routing_builder = ListBuilder::new(StringBuilder::new());
    for extract in &extracts {
        for keyword in &extract.routing_keywords {
            routing_builder.values().append_value(keyword.as_str());
        }
        routing_builder.append(true);
    }
    let routing_keywords = routing_builder.finish();

    let mut intents_builder = ListBuilder::new(StringBuilder::new());
    for extract in &extracts {
        for intent in &extract.intents {
            intents_builder.values().append_value(intent.as_str());
        }
        intents_builder.append(true);
    }
    let intents = intents_builder.finish();

    Ok(ParsedMetadataColumns {
        skill_name,
        category,
        tool_name,
        file_path,
        routing_keywords,
        intents,
    })
}
