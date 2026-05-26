//! Thin SQL data-plane adapter over db-store Arrow schema contracts.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::{DataType, Schema, SchemaRef};
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaContractError, ArrowSchemaDataType,
    ArrowSchemaNullabilityPolicy, ArrowSchemaValidationOptions, build_arrow_schema,
    validate_schema_against_contract_with_options,
};

/// Stable subset of Arrow data types used by Wendao SQL table contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArrowFieldType {
    /// UTF-8 text.
    Utf8,
    /// Signed 64-bit integer.
    Int64,
    /// 64-bit floating point value.
    Float64,
}

impl ArrowFieldType {
    const fn schema_data_type(self) -> ArrowSchemaDataType {
        match self {
            Self::Utf8 => ArrowSchemaDataType::Utf8,
            Self::Int64 => ArrowSchemaDataType::Int64,
            Self::Float64 => ArrowSchemaDataType::Float64,
        }
    }

    fn arrow_data_type(self) -> DataType {
        match self {
            Self::Utf8 => DataType::Utf8,
            Self::Int64 => DataType::Int64,
            Self::Float64 => DataType::Float64,
        }
    }
}

/// One field in a stable Arrow table contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArrowFieldContract {
    name: &'static str,
    field_type: ArrowFieldType,
    nullable: bool,
}

impl ArrowFieldContract {
    /// Create one field contract.
    #[must_use]
    pub(crate) const fn new(
        name: &'static str,
        field_type: ArrowFieldType,
        nullable: bool,
    ) -> Self {
        Self {
            name,
            field_type,
            nullable,
        }
    }

    fn schema_column(self) -> ArrowSchemaColumn {
        if self.nullable {
            ArrowSchemaColumn::nullable(self.name, self.field_type.schema_data_type())
        } else {
            ArrowSchemaColumn::new(self.name, self.field_type.schema_data_type())
        }
    }
}

/// One stable Arrow table contract for a `RecordBatch` data-plane payload.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ArrowTableContract {
    schema_id: &'static str,
    schema_version: &'static str,
    table_name: &'static str,
    fields: &'static [ArrowFieldContract],
}

impl ArrowTableContract {
    /// Create one table contract.
    #[must_use]
    pub(crate) const fn new(
        schema_id: &'static str,
        schema_version: &'static str,
        table_name: &'static str,
        fields: &'static [ArrowFieldContract],
    ) -> Self {
        Self {
            schema_id,
            schema_version,
            table_name,
            fields,
        }
    }

    /// Stable table name.
    #[must_use]
    pub(crate) const fn table_name(self) -> &'static str {
        self.table_name
    }

    /// Build the Arrow schema for this table contract.
    #[must_use]
    pub(crate) fn schema(self) -> SchemaRef {
        Arc::new(build_arrow_schema(&self.schema_contract(), self.metadata()))
    }

    /// Validate an Arrow schema using data-plane compatibility rules.
    ///
    /// This mode still requires stable field order, field names, and Arrow data
    /// types. It allows SQL engines to widen non-null fields to nullable because
    /// query engines commonly lose source NOT NULL proofs for SELECT outputs.
    ///
    /// # Errors
    ///
    /// Returns an error when the schema is not compatible with the contract.
    pub(crate) fn validate_compatible_schema(self, schema: &Schema) -> Result<(), String> {
        validate_schema_against_contract_with_options(
            schema,
            &self.schema_contract(),
            ArrowSchemaValidationOptions::new()
                .with_nullability_policy(ArrowSchemaNullabilityPolicy::AllowWidening),
        )
        .map_err(|error| self.compatibility_error(error))
    }

    fn schema_contract(self) -> ArrowSchemaContract {
        ArrowSchemaContract::new(
            self.table_name,
            true,
            self.fields
                .iter()
                .map(|field| field.schema_column())
                .collect(),
        )
    }

    fn metadata(self) -> HashMap<String, String> {
        HashMap::from([
            ("wendao.schema.id".to_string(), self.schema_id.to_string()),
            (
                "wendao.schema.version".to_string(),
                self.schema_version.to_string(),
            ),
            ("wendao.table.name".to_string(), self.table_name.to_string()),
            (
                "wendao.contract.surface".to_string(),
                "arrow-record-batch".to_string(),
            ),
        ])
    }

    fn compatibility_error(self, error: ArrowSchemaContractError) -> String {
        match error {
            ArrowSchemaContractError::ColumnCountMismatch {
                expected_count,
                actual_count,
                ..
            } => format!(
                "{} expected {} columns but received {}",
                self.table_name, expected_count, actual_count
            ),
            ArrowSchemaContractError::ColumnOrderMismatch {
                column_index,
                expected_column_name,
                actual_column_name,
                ..
            } => format!(
                "{} column {} expected `{}` but received `{}`",
                self.table_name, column_index, expected_column_name, actual_column_name
            ),
            ArrowSchemaContractError::DataTypeMismatch {
                column_name,
                actual_data_type,
                ..
            } => {
                let expected_type = self
                    .fields
                    .iter()
                    .find(|field| field.name == column_name)
                    .map_or(DataType::Null, |field| field.field_type.arrow_data_type());
                format!(
                    "{} column `{}` expected Arrow type `{}` but received `{}`",
                    self.table_name, column_name, expected_type, actual_data_type
                )
            }
            ArrowSchemaContractError::NullabilityMismatch {
                column_name,
                expected_nullable,
                actual_nullable,
                ..
            } => format!(
                "{} column `{}` expected nullable={} but received nullable={}",
                self.table_name, column_name, expected_nullable, actual_nullable
            ),
            other => other.to_string(),
        }
    }
}
