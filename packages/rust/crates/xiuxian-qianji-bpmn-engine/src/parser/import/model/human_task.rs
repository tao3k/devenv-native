#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskFormSpec {
    pub(crate) interaction_type: String,
    pub(crate) question_ref: Option<String>,
    pub(crate) question_text: Option<String>,
    pub(crate) choices_ref: Option<String>,
    pub(crate) choices: Vec<RawHumanTaskChoiceSpec>,
    pub(crate) free_text_fields: Vec<RawHumanTaskFreeTextSpec>,
    pub(crate) result_output: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskChoiceSpec {
    pub(crate) value: String,
    pub(crate) label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskFreeTextSpec {
    pub(crate) name: String,
    pub(crate) optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct RawHumanTaskNativeIoSpec {
    pub(crate) documentation_text: Option<String>,
    pub(crate) declarations: Vec<RawHumanTaskIoDeclaration>,
    pub(crate) interaction_type: Option<String>,
    pub(crate) question_ref: Option<String>,
    pub(crate) question_text: Option<String>,
    pub(crate) choices_ref: Option<String>,
    pub(crate) choices: Vec<RawHumanTaskChoiceSpec>,
    pub(crate) free_text_fields: Vec<RawHumanTaskFreeTextSpec>,
    pub(crate) result_output: Option<String>,
    pub(crate) active_association: Option<RawHumanTaskIoAssociation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskIoDeclaration {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) kind: RawHumanTaskIoDeclarationKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawHumanTaskIoDeclarationKind {
    DataInput,
    DataOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskIoAssociation {
    pub(crate) kind: RawHumanTaskIoAssociationKind,
    pub(crate) source_refs: Vec<String>,
    pub(crate) target_ref: Option<String>,
    pub(crate) assignment_from: Option<String>,
    pub(crate) assignment_to: Option<String>,
}

impl RawHumanTaskIoAssociation {
    pub(crate) fn new(kind: RawHumanTaskIoAssociationKind) -> Self {
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
pub(crate) enum RawHumanTaskIoAssociationKind {
    DataInput,
    DataOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskAssignmentSpec {
    pub(crate) human_performers: Vec<RawHumanTaskResourceRoleSpec>,
    pub(crate) potential_owners: Vec<RawHumanTaskResourceRoleSpec>,
    pub(crate) last_role_kind: Option<RawHumanTaskResourceRoleKind>,
}

impl RawHumanTaskAssignmentSpec {
    pub(crate) fn new() -> Self {
        Self {
            human_performers: Vec::new(),
            potential_owners: Vec::new(),
            last_role_kind: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RawHumanTaskResourceRoleKind {
    HumanPerformer,
    PotentialOwner,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawHumanTaskResourceRoleSpec {
    pub(crate) name: Option<String>,
    pub(crate) resource_ref: Option<String>,
    pub(crate) assignment_expression: Option<String>,
}
