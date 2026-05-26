//! `WendaoGraph` evidence table contract types.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::Schema;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, build_arrow_schema,
};

/// Scalar Arrow type used by a `WendaoGraph` evidence table column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WendaoGraphEvidenceColumnType {
    /// UTF-8 string column.
    Utf8,
    /// 64-bit integer column.
    Int64,
    /// 64-bit float column.
    Float64,
    /// Boolean column.
    Boolean,
}

impl WendaoGraphEvidenceColumnType {
    const fn arrow_schema_data_type(self) -> ArrowSchemaDataType {
        match self {
            Self::Utf8 => ArrowSchemaDataType::Utf8,
            Self::Int64 => ArrowSchemaDataType::Int64,
            Self::Float64 => ArrowSchemaDataType::Float64,
            Self::Boolean => ArrowSchemaDataType::Boolean,
        }
    }
}

/// One column in a `WendaoGraph` evidence table contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WendaoGraphEvidenceColumnContract {
    /// Canonical column name.
    pub name: &'static str,
    /// Canonical Arrow scalar type.
    pub data_type: WendaoGraphEvidenceColumnType,
}

impl WendaoGraphEvidenceColumnContract {
    const fn arrow_schema_column(self) -> ArrowSchemaColumn {
        ArrowSchemaColumn::new(self.name, self.data_type.arrow_schema_data_type())
    }
}

/// Whether a table belongs to the request or response side of the contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WendaoGraphEvidenceTableKind {
    /// Host-to-Julia request table.
    Request,
    /// Julia-to-host response table.
    Response,
}

/// One table in the `WendaoGraph` evidence contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WendaoGraphEvidenceTableContract {
    /// Canonical table name.
    pub table_name: &'static str,
    /// Request or response side.
    pub kind: WendaoGraphEvidenceTableKind,
    /// Whether the table must be present in a request bundle.
    pub required: bool,
    /// Canonical ordered columns.
    pub columns: &'static [WendaoGraphEvidenceColumnContract],
}

impl WendaoGraphEvidenceTableContract {
    /// Materialize the Arrow schema for this table contract.
    #[must_use]
    pub fn schema(self) -> Arc<Schema> {
        Arc::new(build_arrow_schema(
            &self.arrow_schema_contract(),
            HashMap::<String, String>::new(),
        ))
    }

    pub(super) fn arrow_schema_contract(self) -> ArrowSchemaContract {
        ArrowSchemaContract::new(
            self.table_name,
            true,
            self.columns
                .iter()
                .copied()
                .map(WendaoGraphEvidenceColumnContract::arrow_schema_column)
                .collect(),
        )
    }
}

pub(super) const fn column(
    name: &'static str,
    data_type: WendaoGraphEvidenceColumnType,
) -> WendaoGraphEvidenceColumnContract {
    WendaoGraphEvidenceColumnContract { name, data_type }
}
