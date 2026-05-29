//! Arrow schema column helpers for catalog registration.

use xiuxian_db_store::{ArrowSchemaColumn, ArrowSchemaDataType};

pub(in crate::search::queries::sql::registration) const fn utf8_column(
    name: &'static str,
) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

pub(in crate::search::queries::sql::registration) const fn nullable_utf8_column(
    name: &'static str,
) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

pub(in crate::search::queries::sql::registration) const fn boolean_column(
    name: &'static str,
) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Boolean)
}

pub(in crate::search::queries::sql::registration) const fn uint64_column(
    name: &'static str,
) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::UInt64)
}
