use std::sync::Arc;

/// Standard BPMN human-task assignment metadata preserved for host dispatch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskAssignmentSpec {
    /// Human performer role declarations from the BPMN task.
    #[serde(default)]
    pub human_performers: Vec<BpmnHumanTaskResourceRoleSpec>,
    /// Potential owner role declarations from the BPMN task.
    #[serde(default)]
    pub potential_owners: Vec<BpmnHumanTaskResourceRoleSpec>,
}

impl BpmnHumanTaskAssignmentSpec {
    /// Creates an empty assignment metadata snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self {
            human_performers: Vec::new(),
            potential_owners: Vec::new(),
        }
    }

    /// Adds one human performer role.
    #[must_use]
    pub fn with_human_performer(mut self, role: BpmnHumanTaskResourceRoleSpec) -> Self {
        self.human_performers.push(role);
        self
    }

    /// Adds one potential owner role.
    #[must_use]
    pub fn with_potential_owner(mut self, role: BpmnHumanTaskResourceRoleSpec) -> Self {
        self.potential_owners.push(role);
        self
    }
}

impl Default for BpmnHumanTaskAssignmentSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// One standard BPMN resource role attached to a human task.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskResourceRoleSpec {
    /// Optional source-level role name.
    #[serde(default)]
    pub name: Option<Arc<str>>,
    /// Optional source-level `resourceRef` text.
    #[serde(default)]
    pub resource_ref: Option<Arc<str>>,
    /// Optional source-level `resourceAssignmentExpression/formalExpression` text.
    #[serde(default)]
    pub assignment_expression: Option<Arc<str>>,
}

impl BpmnHumanTaskResourceRoleSpec {
    /// Creates one standard BPMN resource role.
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: None,
            resource_ref: None,
            assignment_expression: None,
        }
    }

    /// Attaches a source-level role name.
    #[must_use]
    pub fn with_name(mut self, name: impl AsRef<str>) -> Self {
        self.name = Some(Arc::<str>::from(name.as_ref()));
        self
    }

    /// Attaches a source-level resource reference.
    #[must_use]
    pub fn with_resource_ref(mut self, resource_ref: impl AsRef<str>) -> Self {
        self.resource_ref = Some(Arc::<str>::from(resource_ref.as_ref()));
        self
    }

    /// Attaches a source-level assignment expression.
    #[must_use]
    pub fn with_assignment_expression(mut self, assignment_expression: impl AsRef<str>) -> Self {
        self.assignment_expression = Some(Arc::<str>::from(assignment_expression.as_ref()));
        self
    }
}

impl Default for BpmnHumanTaskResourceRoleSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// Bounded human-task form metadata preserved for host dispatch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskFormSpec {
    /// Native BPMN IO interaction type.
    pub interaction_type: Arc<str>,
    /// Optional variable reference containing the question prompt.
    #[serde(default)]
    pub question_ref: Option<Arc<str>>,
    /// Optional inline question prompt.
    #[serde(default)]
    pub question_text: Option<Arc<str>>,
    /// Optional variable reference containing choices.
    #[serde(default)]
    pub choices_ref: Option<Arc<str>>,
    /// Optional inline choices preserved from native BPMN IO metadata.
    #[serde(default)]
    pub choices: Vec<BpmnHumanTaskChoiceSpec>,
    /// Optional free-text fields preserved from native BPMN IO metadata.
    #[serde(default)]
    pub free_text_fields: Vec<BpmnHumanTaskFreeTextSpec>,
    /// Optional output variable that receives the primary interaction result.
    #[serde(default)]
    pub result_output: Option<Arc<str>>,
}

impl BpmnHumanTaskFormSpec {
    /// Creates one bounded human-task form metadata snapshot.
    #[must_use]
    pub fn new(interaction_type: impl AsRef<str>) -> Self {
        Self {
            interaction_type: Arc::<str>::from(interaction_type.as_ref()),
            question_ref: None,
            question_text: None,
            choices_ref: None,
            choices: Vec::new(),
            free_text_fields: Vec::new(),
            result_output: None,
        }
    }

    /// Attaches a dynamic question variable reference.
    #[must_use]
    pub fn with_question_ref(mut self, question_ref: impl AsRef<str>) -> Self {
        self.question_ref = Some(Arc::<str>::from(question_ref.as_ref()));
        self
    }

    /// Attaches an inline question prompt.
    #[must_use]
    pub fn with_question_text(mut self, question_text: impl AsRef<str>) -> Self {
        self.question_text = Some(Arc::<str>::from(question_text.as_ref()));
        self
    }

    /// Attaches a dynamic choices variable reference.
    #[must_use]
    pub fn with_choices_ref(mut self, choices_ref: impl AsRef<str>) -> Self {
        self.choices_ref = Some(Arc::<str>::from(choices_ref.as_ref()));
        self
    }

    /// Attaches inline choice metadata.
    #[must_use]
    pub fn with_choice(mut self, choice: BpmnHumanTaskChoiceSpec) -> Self {
        self.choices.push(choice);
        self
    }

    /// Attaches free-text field metadata.
    #[must_use]
    pub fn with_free_text_field(mut self, field: BpmnHumanTaskFreeTextSpec) -> Self {
        self.free_text_fields.push(field);
        self
    }

    /// Attaches the primary result output variable.
    #[must_use]
    pub fn with_result_output(mut self, result_output: impl AsRef<str>) -> Self {
        self.result_output = Some(Arc::<str>::from(result_output.as_ref()));
        self
    }
}

/// One inline choice for a bounded human-task form.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskChoiceSpec {
    /// Stable choice value submitted by the host.
    pub value: Arc<str>,
    /// Optional display label.
    #[serde(default)]
    pub label: Option<Arc<str>>,
}

impl BpmnHumanTaskChoiceSpec {
    /// Creates one inline choice.
    #[must_use]
    pub fn new(value: impl AsRef<str>) -> Self {
        Self {
            value: Arc::<str>::from(value.as_ref()),
            label: None,
        }
    }

    /// Attaches a display label.
    #[must_use]
    pub fn with_label(mut self, label: impl AsRef<str>) -> Self {
        self.label = Some(Arc::<str>::from(label.as_ref()));
        self
    }
}

/// One free-text field for a bounded human-task form.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnHumanTaskFreeTextSpec {
    /// Output field name.
    pub name: Arc<str>,
    /// Whether the host may omit this field.
    pub optional: bool,
}

impl BpmnHumanTaskFreeTextSpec {
    /// Creates one free-text field.
    #[must_use]
    pub fn new(name: impl AsRef<str>, optional: bool) -> Self {
        Self {
            name: Arc::<str>::from(name.as_ref()),
            optional,
        }
    }
}
