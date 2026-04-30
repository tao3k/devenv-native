#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawDataObjectSpec {
    pub(crate) id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawDataObjectReferenceSpec {
    pub(crate) id: String,
    pub(crate) data_object_ref: String,
}
