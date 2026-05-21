//! Logic Gate: Incremental XSD Validator for streaming agent output.
//!
//! This module provides real-time validation of XML fragments as they stream in,
//! enforcing the project's "Physical Law" (XSD schema) without waiting for
//! complete documents. This is the primary defense against AI hallucinations.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};
use thiserror::Error;

type SharedStr = Arc<str>;
type SharedStrSet = HashSet<SharedStr>;
type RequiredAttrMap = HashMap<SharedStr, Vec<SharedStr>>;
type ChildTagMap = HashMap<SharedStr, SharedStrSet>;
type EnumAttrMap = HashMap<(SharedStr, SharedStr), SharedStrSet>;
type XmlAttributes = Vec<(SharedStr, SharedStr)>;

/// Validation errors detected during incremental parsing.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LogicGateError {
    /// Tag is not in the XSD whitelist.
    #[error("tag <{tag}> is not allowed by the contract")]
    ForbiddenTag { tag: Arc<str> },
    /// Attribute value violates XSD constraints.
    #[error("attribute '{attr}' has invalid value '{value}'")]
    InvalidAttribute { attr: Arc<str>, value: Arc<str> },
    /// Step number is not sequential.
    #[error("step number {actual} violates linearity (expected {expected})")]
    NonSequentialStep { expected: u32, actual: u32 },
    /// Required attribute is missing.
    #[error("required attribute '{attr}' is missing on <{tag}>")]
    MissingAttribute { tag: Arc<str>, attr: Arc<str> },
    /// Content validation failed.
    #[error("content validation failed: {reason}")]
    InvalidContent { reason: Arc<str> },
    /// Malformed XML fragment.
    #[error("malformed XML: {message}")]
    MalformedXml { message: Arc<str> },
}

/// Allowed tag names extracted from XSD schema (zero-copy).
#[derive(Debug, Clone)]
pub struct XsdConstraintMap {
    /// Tags allowed at the root level (zero-copy).
    root_tags: SharedStrSet,
    /// Tags allowed inside each parent tag (zero-copy).
    child_tags: ChildTagMap,
    /// Required attributes for each tag (zero-copy).
    required_attrs: RequiredAttrMap,
    /// Enumerated attribute values (zero-copy).
    enum_attrs: EnumAttrMap,
}

impl XsdConstraintMap {
    /// Create constraint map for `qianji_plan.xsd`.
    #[must_use]
    pub fn qianji_audit_plan() -> Self {
        Self {
            root_tags: qianji_root_tags(),
            child_tags: qianji_child_tags(),
            required_attrs: qianji_required_attrs(),
            enum_attrs: qianji_enum_attrs(),
        }
    }

    /// Check if a tag is allowed as a root element.
    #[must_use]
    pub fn is_root_tag(&self, tag: &str) -> bool {
        self.root_tags.contains(tag)
    }

    /// Check if a tag is allowed as a child of the given parent.
    #[must_use]
    pub fn is_allowed_child(&self, parent: &str, child: &str) -> bool {
        self.child_tags
            .get(parent)
            .is_some_and(|children| children.contains(child))
    }

    /// Check if an attribute value is valid for the given tag/attribute pair.
    /// Uses borrow-based lookup to avoid Arc allocation on hot path.
    #[must_use]
    pub fn is_valid_enum(&self, tag: &str, attr: &str, value: &str) -> bool {
        // Find by iterating to avoid allocating Arc on lookup
        self.enum_attrs
            .iter()
            .find(|((t, a), _)| &**t == tag && &**a == attr)
            .is_none_or(|(_, values)| values.contains(value)) // If not an enum, any value is allowed
    }

    /// Get required attributes for a tag.
    #[must_use]
    pub fn required_attributes(&self, tag: &str) -> &[Arc<str>] {
        self.required_attrs.get(tag).map_or(&[], Vec::as_slice)
    }
}

fn qianji_root_tags() -> SharedStrSet {
    let mut root_tags = HashSet::new();
    root_tags.insert(Arc::from("qianji-audit-plan"));
    root_tags
}

fn qianji_child_tags() -> ChildTagMap {
    let mut child_tags = HashMap::new();

    // qianji-audit-plan children
    let mut plan_children = HashSet::new();
    plan_children.insert(Arc::from("summary"));
    plan_children.insert(Arc::from("implementation-steps"));
    plan_children.insert(Arc::from("risk-assessment"));
    plan_children.insert(Arc::from("verification-strategy"));
    child_tags.insert(Arc::from("qianji-audit-plan"), plan_children);

    // summary children
    let mut summary_children = HashSet::new();
    summary_children.insert(Arc::from("intent"));
    summary_children.insert(Arc::from("total-steps"));
    child_tags.insert(Arc::from("summary"), summary_children);

    // implementation-steps children
    let mut steps_children = HashSet::new();
    steps_children.insert(Arc::from("step"));
    child_tags.insert(Arc::from("implementation-steps"), steps_children);

    // step children
    let mut single_step_children = HashSet::new();
    single_step_children.insert(Arc::from("title"));
    single_step_children.insert(Arc::from("file-target"));
    single_step_children.insert(Arc::from("rationale"));
    single_step_children.insert(Arc::from("content"));
    child_tags.insert(Arc::from("step"), single_step_children);

    // content children
    let mut content_children = HashSet::new();
    content_children.insert(Arc::from("description"));
    content_children.insert(Arc::from("code-snippet"));
    child_tags.insert(Arc::from("content"), content_children);

    // risk-assessment children
    let mut risk_children = HashSet::new();
    risk_children.insert(Arc::from("risk-pair"));
    child_tags.insert(Arc::from("risk-assessment"), risk_children);

    // risk-pair children
    let mut risk_pair_children = HashSet::new();
    risk_pair_children.insert(Arc::from("potential-issue"));
    risk_pair_children.insert(Arc::from("mitigation-strategy"));
    child_tags.insert(Arc::from("risk-pair"), risk_pair_children);

    // verification-strategy children
    let mut verify_children = HashSet::new();
    verify_children.insert(Arc::from("test-command"));
    verify_children.insert(Arc::from("expected-outcome"));
    child_tags.insert(Arc::from("verification-strategy"), verify_children);
    child_tags
}

fn qianji_required_attrs() -> RequiredAttrMap {
    // Required attributes
    let mut required_attrs = HashMap::new();
    required_attrs.insert(Arc::from("step"), vec![Arc::from("number")]);
    required_attrs.insert(
        Arc::from("file-target"),
        vec![Arc::from("path"), Arc::from("action")],
    );
    required_attrs.insert(Arc::from("risk-pair"), vec![Arc::from("severity")]);
    required_attrs
}

fn qianji_enum_attrs() -> EnumAttrMap {
    // Enumerated attributes
    let mut enum_attrs = HashMap::new();

    let mut action_values = HashSet::new();
    action_values.insert(Arc::from("create"));
    action_values.insert(Arc::from("modify"));
    action_values.insert(Arc::from("delete"));
    action_values.insert(Arc::from("research"));
    enum_attrs.insert(
        (Arc::from("file-target"), Arc::from("action")),
        action_values,
    );

    let mut severity_values = HashSet::new();
    severity_values.insert(Arc::from("low"));
    severity_values.insert(Arc::from("medium"));
    severity_values.insert(Arc::from("high"));
    severity_values.insert(Arc::from("critical"));
    enum_attrs.insert(
        (Arc::from("risk-pair"), Arc::from("severity")),
        severity_values,
    );
    enum_attrs
}

impl Default for XsdConstraintMap {
    fn default() -> Self {
        Self::qianji_audit_plan()
    }
}

/// Global static constraint map for zero initialization overhead.
/// This is lazily initialized on first access and shared across all `LogicGate` instances.
static QIANJI_AUDIT_PLAN_CONSTRAINTS: OnceLock<XsdConstraintMap> = OnceLock::new();

/// The Logic Gate validator for incremental XML parsing.
#[derive(Debug)]
pub struct LogicGate {
    /// Constraint map derived from XSD (shared reference to static).
    constraints: &'static XsdConstraintMap,
    /// Stack of open tags for context (zero-copy).
    tag_stack: Vec<Arc<str>>,
    /// Next expected step number.
    next_step_number: u32,
    /// Current text buffer for content validation.
    text_buffer: String,
}

impl LogicGate {
    /// Create a new Logic Gate with the default Qianji audit plan constraints.
    ///
    /// Uses a globally cached constraint map for zero initialization overhead
    /// on subsequent calls.
    #[must_use]
    pub fn new() -> Self {
        let constraints =
            QIANJI_AUDIT_PLAN_CONSTRAINTS.get_or_init(XsdConstraintMap::qianji_audit_plan);
        Self {
            constraints,
            tag_stack: Vec::new(),
            next_step_number: 1,
            text_buffer: String::new(),
        }
    }

    /// Create a Logic Gate with custom constraints.
    #[cfg(test)]
    #[must_use]
    pub fn with_constraints(constraints: XsdConstraintMap) -> Self {
        Self {
            constraints: Box::leak(Box::new(constraints)),
            tag_stack: Vec::new(),
            next_step_number: 1,
            text_buffer: String::new(),
        }
    }

    /// Perform hot validation on a text chunk.
    ///
    /// This method parses XML incrementally and validates each tag/attribute
    /// as it appears, without waiting for the document to complete.
    ///
    /// # Errors
    ///
    /// Returns `LogicGateError` when a constraint violation is detected.
    pub fn hot_validate(&mut self, chunk: &str) -> Result<Vec<LogicGateEvent>, LogicGateError> {
        let mut events = Vec::new();
        self.text_buffer.push_str(chunk);

        // Process complete tags in the buffer
        let text = std::mem::take(&mut self.text_buffer);
        let mut cursor = 0usize;
        let mut last_complete = 0usize;

        while cursor < text.len() {
            let remaining = &text[cursor..];
            if !remaining.starts_with('<') {
                cursor += 1;
                continue;
            }

            // Find the end of the tag
            let Some(tag_end) = remaining.find('>') else {
                // Incomplete tag, wait for more data
                break;
            };

            let tag_content = &remaining[1..tag_end];
            cursor += tag_end + 1;
            last_complete = cursor;

            // Skip comments and processing instructions
            if tag_content.starts_with('!') || tag_content.starts_with('?') {
                continue;
            }

            // Parse the tag
            let is_closing = tag_content.starts_with('/');
            let tag_body = if is_closing {
                &tag_content[1..]
            } else {
                tag_content
            };

            let tag_name = tag_body.split_whitespace().next().unwrap_or("");

            if tag_name.is_empty() {
                continue;
            }

            if is_closing {
                Self::validate_closing_tag(&mut self.tag_stack, tag_name)?;
                events.push(LogicGateEvent::TagClosed {
                    tag: Arc::from(tag_name),
                });
            } else {
                let is_self_closing = tag_body.ends_with('/');
                let attrs = Self::parse_attributes_static(tag_body);

                self.validate_opening_tag(tag_name, &attrs)?;
                events.push(LogicGateEvent::TagOpened {
                    tag: Arc::from(tag_name),
                    attributes: attrs.clone(),
                });

                if is_self_closing {
                    Self::validate_closing_tag(&mut self.tag_stack, tag_name)?;
                    events.push(LogicGateEvent::TagClosed {
                        tag: Arc::from(tag_name),
                    });
                }
            }
        }

        // Keep unprocessed content in buffer
        self.text_buffer = text[last_complete..].to_string();

        Ok(events)
    }

    fn validate_opening_tag(
        &mut self,
        tag: &str,
        attrs: &[(Arc<str>, Arc<str>)],
    ) -> Result<(), LogicGateError> {
        // Check if this is a root tag
        if self.tag_stack.is_empty() {
            if !self.constraints.is_root_tag(tag) {
                return Err(LogicGateError::ForbiddenTag {
                    tag: Arc::from(tag),
                });
            }
        } else {
            // Check if tag is allowed as child of current parent
            let Some(parent) = self.tag_stack.last() else {
                return Err(LogicGateError::MalformedXml {
                    message: Arc::from("tag stack was unexpectedly empty"),
                });
            };
            if !self.constraints.is_allowed_child(parent, tag) {
                return Err(LogicGateError::ForbiddenTag {
                    tag: Arc::from(tag),
                });
            }
        }

        // Check required attributes
        for required in self.constraints.required_attributes(tag) {
            if !attrs.iter().any(|(k, _)| k.as_ref() == &**required) {
                return Err(LogicGateError::MissingAttribute {
                    tag: Arc::from(tag),
                    attr: required.clone(),
                });
            }
        }

        // Validate enum attributes
        for (attr, value) in attrs {
            if !self.constraints.is_valid_enum(tag, attr, value) {
                return Err(LogicGateError::InvalidAttribute {
                    attr: attr.clone(),
                    value: value.clone(),
                });
            }
        }

        // Check step linearity
        if tag == "step"
            && let Some((_, num_str)) = attrs.iter().find(|(k, _)| k.as_ref() == "number")
            && let Ok(num) = num_str.parse::<u32>()
        {
            if num != self.next_step_number {
                return Err(LogicGateError::NonSequentialStep {
                    expected: self.next_step_number,
                    actual: num,
                });
            }
            self.next_step_number += 1;
        }

        self.tag_stack.push(Arc::from(tag));
        Ok(())
    }

    fn validate_closing_tag(
        tag_stack: &mut Vec<Arc<str>>,
        tag: &str,
    ) -> Result<(), LogicGateError> {
        match tag_stack.pop() {
            Some(expected) if expected.as_ref() == tag => Ok(()),
            Some(expected) => Err(LogicGateError::MalformedXml {
                message: Arc::from(
                    format!("mismatched closing tag: expected </{expected}>, found </{tag}>")
                        .as_str(),
                ),
            }),
            None => Err(LogicGateError::MalformedXml {
                message: Arc::from(format!("unexpected closing tag </{tag}>").as_str()),
            }),
        }
    }

    fn parse_attributes_static(tag_body: &str) -> XmlAttributes {
        let mut attrs = Vec::new();
        let mut in_value = false;
        let mut current_key = String::new();
        let mut current_value = String::new();

        // Skip the tag name
        let mut chars = tag_body.chars().skip_while(|c| !c.is_whitespace());

        while let Some(c) = chars.next() {
            if c == '=' && !in_value {
                // Found attribute name - keep it in current_key for now
                // Expect quote
                let quote = chars.next();
                if quote == Some('"') || quote == Some('\'') {
                    in_value = true;
                }
                continue;
            }

            if in_value && (c == '"' || c == '\'') {
                // End of value - save the attribute (zero-copy)
                attrs.push((
                    Arc::from(current_key.trim()),
                    Arc::from(current_value.as_str()),
                ));
                current_key.clear();
                current_value.clear();
                in_value = false;
                continue;
            }

            if in_value {
                current_value.push(c);
            } else if c == '/' {
                // Self-closing indicator, stop parsing
                break;
            } else if !c.is_whitespace() {
                current_key.push(c);
            }
        }

        attrs
    }

    /// Reset the parser state for a new document.
    pub fn reset(&mut self) {
        self.tag_stack.clear();
        self.next_step_number = 1;
        self.text_buffer.clear();
    }

    /// Check if the parser has finished processing.
    #[cfg(test)]
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.tag_stack.is_empty()
    }

    /// Get the current tag stack depth.
    #[cfg(test)]
    #[must_use]
    pub fn depth(&self) -> usize {
        self.tag_stack.len()
    }

    /// Get the current parent tag.
    #[cfg(test)]
    #[must_use]
    pub fn current_parent(&self) -> Option<&str> {
        self.tag_stack.last().map(|s| &**s)
    }
}

impl Default for LogicGate {
    fn default() -> Self {
        Self::new()
    }
}

/// Events emitted during incremental validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicGateEvent {
    /// A tag was opened.
    TagOpened {
        /// Tag name (zero-copy).
        tag: Arc<str>,
        /// Attributes parsed from the tag (zero-copy keys/values).
        attributes: Vec<(Arc<str>, Arc<str>)>,
    },
    /// A tag was closed.
    TagClosed {
        /// Tag name (zero-copy).
        tag: Arc<str>,
    },
}

#[cfg(test)]
#[path = "../../../tests/unit/transmuter/streaming/logic_gate.rs"]
mod tests;
