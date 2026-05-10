//! Transparent semantic carriers for Studio public DTO fields.

use serde::{Deserialize, Serialize};
use specta::Type;

macro_rules! string_carrier {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(
            Debug,
            Clone,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
            Type,
        )]
        #[serde(transparent)]
        #[specta(transparent)]
        pub struct $name(String);

        impl $name {
            /// Returns the inner string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            /// Consumes the carrier and returns the inner string.
            #[must_use]
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    };
}

macro_rules! numeric_carrier {
    ($name:ident, $inner:ty, $doc:literal, $method:ident) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(
            Debug,
            Clone,
            Copy,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
            Type,
        )]
        #[serde(transparent)]
        #[specta(transparent)]
        pub struct $name($inner);

        impl $name {
            /// Returns the inner numeric value.
            #[must_use]
            pub const fn $method(self) -> $inner {
                self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl PartialEq<$inner> for $name {
            fn eq(&self, other: &$inner) -> bool {
                self.0 == *other
            }
        }
    };
}

string_carrier!(
    StudioContractId,
    "Stable identifier carried by a Studio public DTO."
);
string_carrier!(
    StudioContractPath,
    "Path-like value carried by a Studio public DTO."
);
string_carrier!(
    StudioContractUrl,
    "URL-like value carried by a Studio public DTO."
);
string_carrier!(
    StudioContractKind,
    "Kind label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractStatus,
    "Status label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractMode,
    "Mode label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractState,
    "State label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractCategory,
    "Category label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractContentType,
    "Content type label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractMimeType,
    "MIME type label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractSemanticType,
    "Semantic type label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractNodeKind,
    "AST node kind label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractDocType,
    "Document type label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractRelationType,
    "Relation type label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractTag,
    "Tag label carried by a Studio public DTO."
);
string_carrier!(
    StudioContractToken,
    "Token-like value carried by a Studio public DTO."
);

numeric_carrier!(
    StudioContractMillisecondsI64,
    i64,
    "Epoch or duration milliseconds carried by a Studio public DTO.",
    value
);
numeric_carrier!(
    StudioContractMillisecondsU64,
    u64,
    "Non-negative duration milliseconds carried by a Studio public DTO.",
    value
);
numeric_carrier!(
    StudioContractSecondsU64,
    u64,
    "Non-negative duration seconds carried by a Studio public DTO.",
    value
);

/// Boolean flag indicating that a graph node is the center of a response.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(transparent)]
#[specta(transparent)]
pub struct StudioContractCenterFlag(bool);

impl StudioContractCenterFlag {
    /// Returns the inner flag.
    #[must_use]
    pub const fn is_center(self) -> bool {
        self.0
    }
}

impl From<bool> for StudioContractCenterFlag {
    fn from(value: bool) -> Self {
        Self(value)
    }
}
