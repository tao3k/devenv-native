//! Reusable Arrow table-schema contracts for db-store payload boundaries.

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::BuildHasher;
use std::io::Cursor;
use std::sync::Arc;

use arrow::datatypes::{DataType, Field, Schema};
use arrow::error::ArrowError;
use arrow::ipc::reader::StreamReader;
use arrow::record_batch::RecordBatch;

/// Canonical metadata key used to label an Arrow payload with its logical table.
pub const WENDAO_TABLE_METADATA_KEY: &str = "wendao.table";

/// Small logical data-type vocabulary for shared Arrow table contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowSchemaDataType {
    /// Arrow `Utf8`.
    Utf8,
    /// Arrow `Int32`.
    Int32,
    /// Arrow `Int64`.
    Int64,
    /// Arrow `UInt64`.
    UInt64,
    /// Arrow `Float64`.
    Float64,
    /// Arrow `Boolean`.
    Boolean,
    /// Arrow `Timestamp(Millisecond, None)`.
    TimestampMillisecond,
    /// Arrow `Binary`.
    Binary,
    /// Arrow `List(Utf8)`.
    Utf8List,
    /// Binary payload columns that may arrive as binary arrays or byte lists.
    BinaryPayload,
}

impl ArrowSchemaDataType {
    fn arrow_data_type(self) -> DataType {
        match self {
            Self::Utf8 => DataType::Utf8,
            Self::Int32 => DataType::Int32,
            Self::Int64 => DataType::Int64,
            Self::UInt64 => DataType::UInt64,
            Self::Float64 => DataType::Float64,
            Self::Boolean => DataType::Boolean,
            Self::TimestampMillisecond => {
                DataType::Timestamp(arrow::datatypes::TimeUnit::Millisecond, None)
            }
            Self::Binary | Self::BinaryPayload => DataType::Binary,
            Self::Utf8List => DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Utf8 => "Utf8",
            Self::Int32 => "Int32",
            Self::Int64 => "Int64",
            Self::UInt64 => "UInt64",
            Self::Float64 => "Float64",
            Self::Boolean => "Boolean",
            Self::TimestampMillisecond => "Timestamp(Millisecond)",
            Self::Binary => "Binary",
            Self::Utf8List => "List(Utf8)",
            Self::BinaryPayload => "Binary, LargeBinary, List(UInt8), or LargeList(UInt8)",
        }
    }

    fn matches_arrow_data_type(self, actual: &DataType) -> bool {
        match self {
            Self::Utf8 => matches!(actual, DataType::Utf8),
            Self::Int32 => matches!(actual, DataType::Int32),
            Self::Int64 => matches!(actual, DataType::Int64),
            Self::UInt64 => matches!(actual, DataType::UInt64),
            Self::Float64 => matches!(actual, DataType::Float64),
            Self::Boolean => matches!(actual, DataType::Boolean),
            Self::TimestampMillisecond => {
                matches!(
                    actual,
                    DataType::Timestamp(arrow::datatypes::TimeUnit::Millisecond, None)
                )
            }
            Self::Binary => matches!(actual, DataType::Binary),
            Self::Utf8List => {
                matches!(actual, DataType::List(field) if field.data_type() == &DataType::Utf8)
            }
            Self::BinaryPayload => {
                matches!(
                    actual,
                    DataType::Binary
                        | DataType::LargeBinary
                        | DataType::List(_)
                        | DataType::LargeList(_)
                )
            }
        }
    }
}

/// One column in a logical Arrow table contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrowSchemaColumn {
    name: &'static str,
    data_type: ArrowSchemaDataType,
    nullable: bool,
}

impl ArrowSchemaColumn {
    /// Create a column contract.
    #[must_use]
    pub const fn new(name: &'static str, data_type: ArrowSchemaDataType) -> Self {
        Self {
            name,
            data_type,
            nullable: false,
        }
    }

    /// Create a nullable column contract.
    #[must_use]
    pub const fn nullable(name: &'static str, data_type: ArrowSchemaDataType) -> Self {
        Self {
            name,
            data_type,
            nullable: true,
        }
    }

    /// Logical column name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Logical column data type.
    #[must_use]
    pub const fn data_type(self) -> ArrowSchemaDataType {
        self.data_type
    }

    /// Whether generated Arrow schemas should mark the column nullable.
    #[must_use]
    pub const fn is_nullable(self) -> bool {
        self.nullable
    }
}

/// A logical Arrow table contract owned by a domain crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrowSchemaContract {
    table_name: &'static str,
    exact_column_set: bool,
    columns: Vec<ArrowSchemaColumn>,
}

impl ArrowSchemaContract {
    /// Create a table contract.
    #[must_use]
    pub fn new(
        table_name: &'static str,
        exact_column_set: bool,
        columns: Vec<ArrowSchemaColumn>,
    ) -> Self {
        Self {
            table_name,
            exact_column_set,
            columns,
        }
    }

    /// Logical table name.
    #[must_use]
    pub const fn table_name(&self) -> &'static str {
        self.table_name
    }

    /// Whether validation requires the exact column count and order.
    #[must_use]
    pub const fn exact_column_set(&self) -> bool {
        self.exact_column_set
    }

    /// Required columns for the logical table.
    #[must_use]
    pub fn columns(&self) -> &[ArrowSchemaColumn] {
        self.columns.as_slice()
    }
}

/// Nullability compatibility policy for Arrow schema validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowSchemaNullabilityPolicy {
    /// Do not validate field nullability.
    Ignore,
    /// Require exact nullability parity.
    Exact,
    /// Allow SQL engines to widen non-nullable fields to nullable fields.
    AllowWidening,
}

impl ArrowSchemaNullabilityPolicy {
    const fn allows(self, expected_nullable: bool, actual_nullable: bool) -> bool {
        match self {
            Self::Ignore => true,
            Self::Exact => expected_nullable == actual_nullable,
            Self::AllowWidening => {
                expected_nullable == actual_nullable || (!expected_nullable && actual_nullable)
            }
        }
    }
}

/// Options for shared Arrow schema contract validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrowSchemaValidationOptions {
    nullability_policy: ArrowSchemaNullabilityPolicy,
}

impl ArrowSchemaValidationOptions {
    /// Create validation options with default compatibility rules.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nullability_policy: ArrowSchemaNullabilityPolicy::Ignore,
        }
    }

    /// Return validation options with a different nullability policy.
    #[must_use]
    pub const fn with_nullability_policy(
        mut self,
        nullability_policy: ArrowSchemaNullabilityPolicy,
    ) -> Self {
        self.nullability_policy = nullability_policy;
        self
    }

    const fn nullability_policy(self) -> ArrowSchemaNullabilityPolicy {
        self.nullability_policy
    }
}

impl Default for ArrowSchemaValidationOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors produced by shared Arrow schema contract validation.
#[derive(Debug)]
#[non_exhaustive]
pub enum ArrowSchemaContractError {
    /// The Arrow IPC payload was empty.
    EmptyIpcPayload {
        /// Logical table name expected for the payload.
        table_name: &'static str,
    },
    /// The Arrow IPC stream schema could not be decoded.
    IpcSchemaDecode {
        /// Logical table name expected for the payload.
        table_name: &'static str,
        /// Arrow decode error returned by the stream reader.
        source: ArrowError,
    },
    /// One Arrow IPC batch could not be decoded.
    IpcBatchDecode {
        /// Logical table name expected for the payload.
        table_name: &'static str,
        /// Arrow decode error returned for one batch.
        source: ArrowError,
    },
    /// The Arrow IPC stream had no record batches.
    EmptyIpcStream {
        /// Logical table name expected for the payload.
        table_name: &'static str,
    },
    /// The optional `wendao.table` metadata did not match the contract.
    TableMetadataMismatch {
        /// Logical table name required by the contract.
        expected_table_name: &'static str,
        /// Logical table name found in Arrow schema metadata.
        actual_table_name: String,
    },
    /// An exact table contract had a different number of columns.
    ColumnCountMismatch {
        /// Logical table name being validated.
        table_name: &'static str,
        /// Number of columns required by the contract.
        expected_count: usize,
        /// Number of columns present in the Arrow schema.
        actual_count: usize,
    },
    /// A required column was missing.
    MissingRequiredColumn {
        /// Logical table name being validated.
        table_name: &'static str,
        /// Required column that was absent from the Arrow schema.
        column_name: &'static str,
    },
    /// An exact table contract had the right column but in the wrong position.
    ColumnOrderMismatch {
        /// Logical table name being validated.
        table_name: &'static str,
        /// Zero-based column position that failed the order check.
        column_index: usize,
        /// Column name required at the failing position.
        expected_column_name: &'static str,
        /// Column name found at the failing position.
        actual_column_name: String,
    },
    /// A column type did not match the contract.
    DataTypeMismatch {
        /// Logical table name being validated.
        table_name: &'static str,
        /// Column whose Arrow data type failed validation.
        column_name: &'static str,
        /// Human-readable data type required by the contract.
        expected_data_type: &'static str,
        /// Arrow data type found in the schema.
        actual_data_type: DataType,
    },
    /// A column nullability policy did not match the contract.
    NullabilityMismatch {
        /// Logical table name being validated.
        table_name: &'static str,
        /// Column whose nullability mismatched.
        column_name: &'static str,
        /// Nullability required by the contract.
        expected_nullable: bool,
        /// Nullability found in the Arrow schema.
        actual_nullable: bool,
    },
}

impl Display for ArrowSchemaContractError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyIpcPayload { table_name } => {
                write!(
                    formatter,
                    "`{table_name}` Arrow IPC payload must not be empty"
                )
            }
            Self::IpcSchemaDecode { table_name, source } => {
                write!(
                    formatter,
                    "`{table_name}` Arrow IPC schema decode failed: {source}"
                )
            }
            Self::IpcBatchDecode { table_name, source } => {
                write!(
                    formatter,
                    "`{table_name}` Arrow IPC batch decode failed: {source}"
                )
            }
            Self::EmptyIpcStream { table_name } => {
                write!(
                    formatter,
                    "`{table_name}` Arrow IPC payload contained no batches"
                )
            }
            Self::TableMetadataMismatch {
                expected_table_name,
                actual_table_name,
            } => write!(
                formatter,
                "table metadata must be `{expected_table_name}` but was `{actual_table_name}`"
            ),
            Self::ColumnCountMismatch {
                table_name,
                expected_count,
                actual_count,
            } => write!(
                formatter,
                "`{table_name}` must have {expected_count} columns but had {actual_count}"
            ),
            Self::MissingRequiredColumn {
                table_name,
                column_name,
            } => write!(
                formatter,
                "`{table_name}` missing required column `{column_name}`"
            ),
            Self::ColumnOrderMismatch {
                table_name,
                column_index,
                expected_column_name,
                actual_column_name,
            } => write!(
                formatter,
                "`{table_name}` column {column_index} must be `{expected_column_name}` but was `{actual_column_name}`"
            ),
            Self::DataTypeMismatch {
                table_name,
                column_name,
                expected_data_type,
                actual_data_type,
            } => write!(
                formatter,
                "`{table_name}` column `{column_name}` must be {expected_data_type} but was {actual_data_type:?}"
            ),
            Self::NullabilityMismatch {
                table_name,
                column_name,
                expected_nullable,
                actual_nullable,
            } => write!(
                formatter,
                "`{table_name}` column `{column_name}` expected nullable={expected_nullable} but received nullable={actual_nullable}"
            ),
        }
    }
}

impl Error for ArrowSchemaContractError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IpcSchemaDecode { source, .. } | Self::IpcBatchDecode { source, .. } => {
                Some(source)
            }
            Self::EmptyIpcPayload { .. }
            | Self::EmptyIpcStream { .. }
            | Self::TableMetadataMismatch { .. }
            | Self::ColumnCountMismatch { .. }
            | Self::MissingRequiredColumn { .. }
            | Self::ColumnOrderMismatch { .. }
            | Self::DataTypeMismatch { .. }
            | Self::NullabilityMismatch { .. } => None,
        }
    }
}

/// Build an Arrow `Schema` from a logical table contract and metadata.
#[must_use]
pub fn build_arrow_schema<S: BuildHasher>(
    contract: &ArrowSchemaContract,
    metadata: HashMap<String, String, S>,
) -> Schema {
    let metadata = metadata.into_iter().collect::<HashMap<_, _>>();
    Schema::new_with_metadata(arrow_fields_for_contract(contract), metadata)
}

/// Build Arrow fields from a logical table contract.
#[must_use]
pub fn arrow_fields_for_contract(contract: &ArrowSchemaContract) -> Vec<Field> {
    contract
        .columns()
        .iter()
        .copied()
        .map(arrow_field_for_column)
        .collect()
}

/// Build one Arrow field from a logical column contract.
#[must_use]
pub fn arrow_field_for_column(column: ArrowSchemaColumn) -> Field {
    Field::new(
        column.name(),
        column.data_type().arrow_data_type(),
        column.is_nullable(),
    )
}

/// Validate one `RecordBatch` schema against a logical table contract.
///
/// # Errors
///
/// Returns an error when optional table metadata mismatches, required columns
/// are absent, an exact contract has extra or reordered columns, or any
/// required column has an incompatible Arrow data type.
pub fn validate_record_batch_schema(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
) -> Result<(), ArrowSchemaContractError> {
    validate_record_batch_schema_with_options(
        batch,
        contract,
        ArrowSchemaValidationOptions::default(),
    )
}

/// Validate one `RecordBatch` schema against a logical table contract.
///
/// # Errors
///
/// Returns an error when optional table metadata mismatches, required columns
/// are absent, an exact contract has extra or reordered columns, any required
/// column has an incompatible Arrow data type, or the configured nullability
/// policy is violated.
pub fn validate_record_batch_schema_with_options(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
    options: ArrowSchemaValidationOptions,
) -> Result<(), ArrowSchemaContractError> {
    validate_table_metadata(
        batch.schema().metadata().get(WENDAO_TABLE_METADATA_KEY),
        contract.table_name(),
    )?;
    validate_schema_against_contract_with_options(batch.schema().as_ref(), contract, options)
}

/// Validate an Arrow `Schema` against a logical table contract.
///
/// # Errors
///
/// Returns an error when required columns are absent, an exact contract has
/// extra or reordered columns, or any required column has an incompatible Arrow
/// data type.
pub fn validate_schema_against_contract(
    schema: &Schema,
    contract: &ArrowSchemaContract,
) -> Result<(), ArrowSchemaContractError> {
    validate_schema_against_contract_with_options(
        schema,
        contract,
        ArrowSchemaValidationOptions::default(),
    )
}

/// Validate an Arrow `Schema` against a logical table contract.
///
/// # Errors
///
/// Returns an error when required columns are absent, an exact contract has
/// extra or reordered columns, any required column has an incompatible Arrow
/// data type, or the configured nullability policy is violated.
pub fn validate_schema_against_contract_with_options(
    schema: &Schema,
    contract: &ArrowSchemaContract,
    options: ArrowSchemaValidationOptions,
) -> Result<(), ArrowSchemaContractError> {
    if contract.exact_column_set() && schema.fields().len() != contract.columns().len() {
        return Err(ArrowSchemaContractError::ColumnCountMismatch {
            table_name: contract.table_name(),
            expected_count: contract.columns().len(),
            actual_count: schema.fields().len(),
        });
    }

    for (column_index, column) in contract.columns().iter().enumerate() {
        let field = schema.field_with_name(column.name()).map_err(|_| {
            ArrowSchemaContractError::MissingRequiredColumn {
                table_name: contract.table_name(),
                column_name: column.name(),
            }
        })?;

        if contract.exact_column_set() && schema.fields()[column_index].name() != column.name() {
            return Err(ArrowSchemaContractError::ColumnOrderMismatch {
                table_name: contract.table_name(),
                column_index,
                expected_column_name: column.name(),
                actual_column_name: schema.fields()[column_index].name().clone(),
            });
        }

        if !column
            .data_type()
            .matches_arrow_data_type(field.data_type())
        {
            return Err(ArrowSchemaContractError::DataTypeMismatch {
                table_name: contract.table_name(),
                column_name: column.name(),
                expected_data_type: column.data_type().label(),
                actual_data_type: field.data_type().clone(),
            });
        }
        if !options
            .nullability_policy()
            .allows(column.is_nullable(), field.is_nullable())
        {
            return Err(ArrowSchemaContractError::NullabilityMismatch {
                table_name: contract.table_name(),
                column_name: column.name(),
                expected_nullable: column.is_nullable(),
                actual_nullable: field.is_nullable(),
            });
        }
    }
    Ok(())
}

/// Validate an Arrow IPC stream payload against a logical table contract.
///
/// # Errors
///
/// Returns an error when the payload is empty, cannot be decoded as Arrow IPC,
/// contains no batches, or any batch schema violates the provided contract.
pub fn validate_arrow_ipc_stream(
    payload: &[u8],
    contract: &ArrowSchemaContract,
) -> Result<(), ArrowSchemaContractError> {
    validate_arrow_ipc_stream_with_options(
        payload,
        contract,
        ArrowSchemaValidationOptions::default(),
    )
}

/// Validate an Arrow IPC stream payload against a logical table contract.
///
/// # Errors
///
/// Returns an error when the payload is empty, cannot be decoded as Arrow IPC,
/// contains no batches, or any batch schema violates the provided contract and
/// validation options.
pub fn validate_arrow_ipc_stream_with_options(
    payload: &[u8],
    contract: &ArrowSchemaContract,
    options: ArrowSchemaValidationOptions,
) -> Result<(), ArrowSchemaContractError> {
    if payload.is_empty() {
        return Err(ArrowSchemaContractError::EmptyIpcPayload {
            table_name: contract.table_name(),
        });
    }

    let reader = StreamReader::try_new(Cursor::new(payload), None).map_err(|source| {
        ArrowSchemaContractError::IpcSchemaDecode {
            table_name: contract.table_name(),
            source,
        }
    })?;

    let mut batch_count = 0usize;
    for batch in reader {
        let batch = batch.map_err(|source| ArrowSchemaContractError::IpcBatchDecode {
            table_name: contract.table_name(),
            source,
        })?;
        validate_record_batch_schema_with_options(&batch, contract, options)?;
        batch_count = batch_count.saturating_add(1);
    }

    if batch_count == 0 {
        return Err(ArrowSchemaContractError::EmptyIpcStream {
            table_name: contract.table_name(),
        });
    }
    Ok(())
}

fn validate_table_metadata(
    actual_table_name: Option<&String>,
    expected_table_name: &'static str,
) -> Result<(), ArrowSchemaContractError> {
    if let Some(actual_table_name) = actual_table_name
        && actual_table_name != expected_table_name
    {
        return Err(ArrowSchemaContractError::TableMetadataMismatch {
            expected_table_name,
            actual_table_name: actual_table_name.clone(),
        });
    }
    Ok(())
}
