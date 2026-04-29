use super::Arc;

/// One bounded direct DMN literal expression.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnLiteralExpression {
    /// Stable literal-expression identifier when present in source.
    pub expression_id: Option<Arc<str>>,
    /// Optional DMN `typeRef` metadata on the expression.
    pub type_ref: Option<Arc<str>>,
    /// Source-level expression body.
    pub text: Arc<str>,
}

impl DmnLiteralExpression {
    /// Creates one bounded literal-expression snapshot.
    #[must_use]
    pub fn new(
        expression_id: Option<impl AsRef<str>>,
        type_ref: Option<impl AsRef<str>>,
        text: impl AsRef<str>,
    ) -> Self {
        Self {
            expression_id: expression_id.map(|value| Arc::<str>::from(value.as_ref())),
            type_ref: type_ref.map(|value| Arc::<str>::from(value.as_ref())),
            text: Arc::<str>::from(text.as_ref()),
        }
    }
}

/// One bounded direct DMN list expression.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnListExpression {
    /// Stable list identifier when present in source.
    pub list_id: Option<Arc<str>>,
    /// Ordered direct literal-expression items.
    pub items: Vec<DmnLiteralExpression>,
}

impl DmnListExpression {
    /// Creates one bounded list-expression snapshot.
    #[must_use]
    pub fn new(list_id: Option<impl AsRef<str>>, items: Vec<DmnLiteralExpression>) -> Self {
        Self {
            list_id: list_id.map(|value| Arc::<str>::from(value.as_ref())),
            items,
        }
    }
}

/// One bounded direct DMN context entry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnContextEntry {
    /// Stable context-entry identifier when present in source.
    pub entry_id: Option<Arc<str>>,
    /// Stable variable identifier when present in source.
    pub variable_id: Option<Arc<str>>,
    /// Optional variable name. A missing name marks the final result entry.
    pub variable_name: Option<Arc<str>>,
    /// Bounded literal-expression body for this context entry.
    pub expression: DmnLiteralExpression,
}

impl DmnContextEntry {
    /// Creates one bounded context-entry snapshot.
    #[must_use]
    pub fn new(
        entry_id: Option<impl AsRef<str>>,
        variable_id: Option<impl AsRef<str>>,
        variable_name: Option<impl AsRef<str>>,
        expression: DmnLiteralExpression,
    ) -> Self {
        Self {
            entry_id: entry_id.map(|value| Arc::<str>::from(value.as_ref())),
            variable_id: variable_id.map(|value| Arc::<str>::from(value.as_ref())),
            variable_name: variable_name.map(|value| Arc::<str>::from(value.as_ref())),
            expression,
        }
    }
}

/// One bounded direct DMN context expression.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnContextExpression {
    /// Stable context identifier when present in source.
    pub context_id: Option<Arc<str>>,
    /// Ordered context entries.
    pub entries: Vec<DmnContextEntry>,
}

impl DmnContextExpression {
    /// Creates one bounded context-expression snapshot.
    #[must_use]
    pub fn new(context_id: Option<impl AsRef<str>>, entries: Vec<DmnContextEntry>) -> Self {
        Self {
            context_id: context_id.map(|value| Arc::<str>::from(value.as_ref())),
            entries,
        }
    }
}

/// One bounded direct DMN relation column.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnRelationColumn {
    /// Stable column identifier.
    pub column_id: Arc<str>,
    /// Optional output name. Falls back to `column_id` when omitted.
    pub name: Option<Arc<str>>,
    /// Optional DMN `typeRef` metadata on the column.
    pub type_ref: Option<Arc<str>>,
}

impl DmnRelationColumn {
    /// Creates one bounded relation-column snapshot.
    #[must_use]
    pub fn new(
        column_id: impl AsRef<str>,
        name: Option<impl AsRef<str>>,
        type_ref: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            column_id: Arc::<str>::from(column_id.as_ref()),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            type_ref: type_ref.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }

    /// Returns the stable output key used for evaluated row objects.
    #[must_use]
    pub fn output_key(&self) -> &str {
        self.name.as_deref().unwrap_or(self.column_id.as_ref())
    }
}

/// One bounded direct DMN relation row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnRelationRow {
    /// Stable row identifier when present in source.
    pub row_id: Option<Arc<str>>,
    /// Ordered direct literal-expression cell values.
    pub cells: Vec<DmnLiteralExpression>,
}

impl DmnRelationRow {
    /// Creates one bounded relation-row snapshot.
    #[must_use]
    pub fn new(row_id: Option<impl AsRef<str>>, cells: Vec<DmnLiteralExpression>) -> Self {
        Self {
            row_id: row_id.map(|value| Arc::<str>::from(value.as_ref())),
            cells,
        }
    }
}

/// One bounded direct DMN relation expression.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnRelationExpression {
    /// Stable relation identifier when present in source.
    pub relation_id: Option<Arc<str>>,
    /// Ordered direct relation columns.
    pub columns: Vec<DmnRelationColumn>,
    /// Ordered direct relation rows.
    pub rows: Vec<DmnRelationRow>,
}

impl DmnRelationExpression {
    /// Creates one bounded relation-expression snapshot.
    #[must_use]
    pub fn new(
        relation_id: Option<impl AsRef<str>>,
        columns: Vec<DmnRelationColumn>,
        rows: Vec<DmnRelationRow>,
    ) -> Self {
        Self {
            relation_id: relation_id.map(|value| Arc::<str>::from(value.as_ref())),
            columns,
            rows,
        }
    }
}
