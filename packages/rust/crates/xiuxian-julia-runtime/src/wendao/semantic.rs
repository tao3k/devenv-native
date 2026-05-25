//! Transparent semantic carriers for Julia integration public DTO fields.

use serde::{Deserialize, Serialize};

macro_rules! string_carrier {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(
            Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
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

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.into_string()
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
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
    };
}

macro_rules! numeric_carrier {
    ($name:ident, $inner:ty, $doc:literal) => {
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
        )]
        #[serde(transparent)]
        pub struct $name($inner);

        impl $name {
            /// Returns the inner numeric value.
            #[must_use]
            pub const fn value(self) -> $inner {
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

        impl PartialEq<$name> for $inner {
            fn eq(&self, other: &$name) -> bool {
                *self == other.0
            }
        }
    };
}

macro_rules! bool_carrier {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(bool);

        impl $name {
            /// Returns the inner boolean value.
            #[must_use]
            pub const fn value(self) -> bool {
                self.0
            }
        }

        impl From<bool> for $name {
            fn from(value: bool) -> Self {
                Self(value)
            }
        }

        impl PartialEq<bool> for $name {
            fn eq(&self, other: &bool) -> bool {
                self.0 == *other
            }
        }

        impl PartialEq<$name> for bool {
            fn eq(&self, other: &$name) -> bool {
                *self == other.0
            }
        }
    };
}

string_carrier!(
    JuliaContractId,
    "Stable id carried by a Julia integration DTO."
);
string_carrier!(
    JuliaContractRoute,
    "Flight or health route carried by a Julia integration DTO."
);
string_carrier!(
    JuliaContractSchemaVersion,
    "Schema version carried by a Julia integration DTO."
);
string_carrier!(
    JuliaContractMode,
    "Mode label carried by a Julia integration DTO."
);
string_carrier!(
    JuliaContractPath,
    "Path-like value carried by a Julia integration DTO."
);
string_carrier!(
    JuliaContractUrl,
    "URL-like value carried by a Julia integration DTO."
);
string_carrier!(
    JuliaContractState,
    "State label carried by a Julia integration DTO."
);
string_carrier!(
    JuliaContractKind,
    "Kind label carried by a Julia integration DTO."
);
string_carrier!(
    JuliaContractTransport,
    "Transport label carried by a Julia integration DTO."
);
string_carrier!(
    JuliaContractReason,
    "Reason label carried by a Julia integration DTO."
);

numeric_carrier!(
    JuliaContractSecondsU64,
    u64,
    "Duration seconds carried by a Julia integration DTO."
);
numeric_carrier!(
    JuliaContractTimestampMsI64,
    i64,
    "Unix timestamp milliseconds carried by a Julia integration DTO."
);

bool_carrier!(
    JuliaContractEnabled,
    "Enabled flag carried by a Julia integration DTO."
);
