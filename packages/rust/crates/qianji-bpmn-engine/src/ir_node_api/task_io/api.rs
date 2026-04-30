use std::sync::Arc;

/// Bounded standard BPMN task IO bindings preserved for host dispatch.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnTaskIoSpec {
    /// Resolved input bindings supplied to the host request.
    #[serde(default)]
    pub inputs: Vec<BpmnTaskInputBinding>,
    /// Declared output bindings used to validate host completion data.
    #[serde(default)]
    pub outputs: Vec<BpmnTaskOutputBinding>,
}

impl BpmnTaskIoSpec {
    /// Creates an empty task IO binding snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    /// Adds one input binding.
    #[must_use]
    pub fn with_input(mut self, input: BpmnTaskInputBinding) -> Self {
        self.inputs.push(input);
        self
    }

    /// Adds one output binding.
    #[must_use]
    pub fn with_output(mut self, output: BpmnTaskOutputBinding) -> Self {
        self.outputs.push(output);
        self
    }
}

impl Default for BpmnTaskIoSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// One bounded BPMN task input binding.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnTaskInputBinding {
    /// Data input name exposed to the host.
    pub name: Arc<str>,
    /// Source used to materialize the input value.
    pub source: BpmnTaskInputSource,
}

impl BpmnTaskInputBinding {
    /// Creates one bounded task input binding.
    #[must_use]
    pub fn new(name: impl AsRef<str>, source: BpmnTaskInputSource) -> Self {
        Self {
            name: Arc::<str>::from(name.as_ref()),
            source,
        }
    }
}

/// Supported bounded BPMN task input source.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BpmnTaskInputSource {
    /// Reads a value from a workflow variable path.
    Variable {
        /// Source workflow variable path.
        source_ref: Arc<str>,
    },
    /// Uses an inline assignment literal.
    Literal {
        /// Raw assignment literal. Runtime parses JSON literals and otherwise
        /// preserves the text as a string value.
        value: Arc<str>,
    },
}

impl BpmnTaskInputSource {
    /// Creates a variable-backed input source.
    #[must_use]
    pub fn variable(source_ref: impl AsRef<str>) -> Self {
        Self::Variable {
            source_ref: Arc::<str>::from(source_ref.as_ref()),
        }
    }

    /// Creates a literal-backed input source.
    #[must_use]
    pub fn literal(value: impl AsRef<str>) -> Self {
        Self::Literal {
            value: Arc::<str>::from(value.as_ref()),
        }
    }
}

/// One bounded BPMN task output binding.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnTaskOutputBinding {
    /// Host completion field name.
    pub name: Arc<str>,
    /// Target workflow variable path.
    pub target_ref: Arc<str>,
    /// Whether completion must include this field.
    #[serde(default = "default_required_task_output")]
    pub required: bool,
}

impl BpmnTaskOutputBinding {
    /// Creates one bounded task output binding.
    #[must_use]
    pub fn new(name: impl AsRef<str>, target_ref: impl AsRef<str>) -> Self {
        Self {
            name: Arc::<str>::from(name.as_ref()),
            target_ref: Arc::<str>::from(target_ref.as_ref()),
            required: true,
        }
    }

    /// Marks this output as optional for programmatic IR fixtures.
    #[must_use]
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }
}

fn default_required_task_output() -> bool {
    true
}
