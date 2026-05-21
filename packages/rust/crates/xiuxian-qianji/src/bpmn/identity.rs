//! Domain identity newtypes for the `qianji` BPMN boundary.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

macro_rules! define_bpmn_string_identity {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Creates a typed BPMN identity from an owned or borrowed string.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Borrows the raw identity string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            /// Returns the owned raw identity string.
            #[must_use]
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<&String> for $name {
            fn from(value: &String) -> Self {
                Self::new(value.clone())
            }
        }

        impl From<&$name> for $name {
            fn from(value: &$name) -> Self {
                value.clone()
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.into_string()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl PartialEq<&str> for $name {
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }

        impl PartialEq<$name> for &str {
            fn eq(&self, other: &$name) -> bool {
                *self == other.as_str()
            }
        }

        impl PartialEq<String> for $name {
            fn eq(&self, other: &String) -> bool {
                self.as_str() == other.as_str()
            }
        }

        impl PartialEq<$name> for String {
            fn eq(&self, other: &$name) -> bool {
                self.as_str() == other.as_str()
            }
        }
    };
}

define_bpmn_string_identity!(
    /// BPMN process identifier carried across run, checkpoint, and HTTP seams.
    QianjiBpmnProcessId
);
define_bpmn_string_identity!(
    /// Workflow instance identifier carried across checkpoint-backed BPMN seams.
    QianjiBpmnWorkflowInstanceId
);
define_bpmn_string_identity!(
    /// BPMN activity identifier for pending host-work and task-completion seams.
    QianjiBpmnActivityId
);
define_bpmn_string_identity!(
    /// BPMN package identifier reported by persisted workflow summaries.
    QianjiBpmnPackageId
);
define_bpmn_string_identity!(
    /// Synthetic start-at BPMN node identifier for controlled test runs.
    QianjiBpmnStartAtNodeId
);
define_bpmn_string_identity!(
    /// Checkpoint lease owner token reported by backend conflict errors.
    QianjiBpmnLeaseOwnerToken
);
