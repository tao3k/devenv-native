#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RawTaskIoSpec {
    pub(crate) declarations: Vec<RawTaskIoDeclaration>,
    pub(crate) property_ids: Vec<String>,
    pub(crate) inputs: Vec<RawTaskInputBinding>,
    pub(crate) outputs: Vec<RawTaskOutputBinding>,
    pub(crate) active_association: Option<RawTaskIoAssociation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTaskIoDeclaration {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) kind: RawTaskIoDeclarationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawTaskIoDeclarationKind {
    DataInput,
    DataOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTaskInputBinding {
    pub(crate) name: String,
    pub(crate) source: RawTaskInputSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RawTaskInputSource {
    Variable { source_ref: String },
    Literal { value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTaskOutputBinding {
    pub(crate) name: String,
    pub(crate) target_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawTaskIoAssociation {
    pub(crate) kind: RawTaskIoAssociationKind,
    pub(crate) source_refs: Vec<String>,
    pub(crate) target_ref: Option<String>,
    pub(crate) assignment_from: Option<String>,
    pub(crate) assignment_to: Option<String>,
}

impl RawTaskIoAssociation {
    pub(crate) fn new(kind: RawTaskIoAssociationKind) -> Self {
        Self {
            kind,
            source_refs: Vec::new(),
            target_ref: None,
            assignment_from: None,
            assignment_to: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawTaskIoAssociationKind {
    DataInput,
    DataOutput,
}
