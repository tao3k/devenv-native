/// Request-scoped surface bundle used to constrain XML authoring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurfaceBundle {
    pub(crate) project_root: String,
    pub(crate) catalog_table_name: String,
    pub(crate) column_catalog_table_name: String,
    pub(crate) view_source_catalog_table_name: String,
    pub(crate) policy: SurfacePolicy,
    pub(crate) objects: Vec<SurfaceObject>,
}

impl SurfaceBundle {
    pub(crate) fn find_object(&self, target: &str) -> Option<&SurfaceObject> {
        self.objects
            .iter()
            .find(|object| object.name.eq_ignore_ascii_case(target))
    }
}

/// Deterministic authoring policy shipped to the LLM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurfacePolicy {
    pub(crate) max_limit: usize,
    pub(crate) allowed_ops: Vec<String>,
    pub(crate) require_filter_for: Vec<String>,
}

impl SurfacePolicy {
    pub(crate) fn allows_op(&self, op: &str) -> bool {
        self.allowed_ops
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(op))
    }

    pub(crate) fn requires_filter_for(&self, object_name: &str) -> bool {
        self.require_filter_for
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(object_name))
    }
}

/// SQL-visible object exposed to the authoring loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurfaceObject {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) scope: String,
    pub(crate) corpus: String,
    pub(crate) repo_id: Option<String>,
    pub(crate) source_count: usize,
    pub(crate) columns: Vec<SurfaceColumn>,
}

impl SurfaceObject {
    pub(crate) fn find_column(&self, target: &str) -> Option<&SurfaceColumn> {
        self.columns
            .iter()
            .find(|column| column.name.eq_ignore_ascii_case(target))
    }
}

/// SQL-visible column metadata exposed to the authoring loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurfaceColumn {
    pub(crate) name: String,
    pub(crate) data_type: String,
    pub(crate) nullable: bool,
    pub(crate) ordinal_position: usize,
    pub(crate) origin_kind: String,
}

/// Strongly typed XML authoring contract emitted by `Author` or `Repair`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlAuthorSpec {
    pub(crate) target_object: String,
    pub(crate) projection: Vec<String>,
    pub(crate) filters: Vec<SqlFilter>,
    pub(crate) order_by: Vec<SqlOrderTerm>,
    pub(crate) limit: usize,
    pub(crate) sql_draft: Option<String>,
}

/// One constrained SQL filter clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlFilter {
    pub(crate) column: String,
    pub(crate) op: String,
    pub(crate) value: String,
}

/// One constrained SQL ordering term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlOrderTerm {
    pub(crate) column: String,
    pub(crate) direction: String,
}
