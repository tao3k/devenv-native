use std::collections::BTreeMap;

pub(super) type Row = BTreeMap<String, String>;

#[derive(Debug, Clone, Copy)]
pub(super) struct Column {
    pub(super) name: &'static str,
    pub(super) data_type: ColumnDataType,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ColumnDataType {
    String,
    Int64,
}

impl Column {
    pub(super) const fn string(name: &'static str) -> Self {
        Self {
            name,
            data_type: ColumnDataType::String,
        }
    }

    pub(super) const fn int64(name: &'static str) -> Self {
        Self {
            name,
            data_type: ColumnDataType::Int64,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ProjectionStateRow {
    pub(super) projection: String,
    pub(super) status: String,
    pub(super) staleness: String,
    pub(super) source_object_count: i64,
    pub(super) source_relation_count: i64,
    pub(super) source_evidence_count: i64,
}

pub(super) fn required_value<'a>(
    row: &'a Row,
    column_name: &str,
    table_name: &str,
    row_number: usize,
    artifact_label: &str,
) -> Result<&'a str, String> {
    row.get(column_name).map(String::as_str).ok_or_else(|| {
        let row_label = if row_number == 0 {
            "unknown".to_string()
        } else {
            row_number.to_string()
        };
        format!("{artifact_label} `{table_name}` row {row_label} is missing `{column_name}`")
    })
}
