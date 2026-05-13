//! Shared typed public identifiers for Zhenfa APIs.

use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! zhenfa_string_newtype {
    ($type_name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $type_name(String);

        impl $type_name {
            /// Create the typed string value.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Borrow the inner string.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Return the owned inner string.
            #[must_use]
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl From<String> for $type_name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl From<&str> for $type_name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl AsRef<str> for $type_name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl fmt::Display for $type_name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl PartialEq<&str> for $type_name {
            fn eq(&self, other: &&str) -> bool {
                self.as_str() == *other
            }
        }
    };
}

zhenfa_string_newtype!(ZhenfaSessionId, "Runtime session identifier.");
zhenfa_string_newtype!(ZhenfaTraceId, "Runtime trace or correlation identifier.");
zhenfa_string_newtype!(ZhenfaToolId, "Native Zhenfa tool identifier.");
zhenfa_string_newtype!(ZhenfaSignalType, "External or notification signal type.");
zhenfa_string_newtype!(ZhenfaXmlLiteTagName, "XML-Lite tag name.");

/// Borrowed native tool identifier accepted by public registry and dispatch APIs.
pub type ZhenfaToolIdRef<'a> = &'a str;

/// Dispatch elapsed milliseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ZhenfaElapsedMillis(u128);

impl ZhenfaElapsedMillis {
    /// Create an elapsed-milliseconds value.
    #[must_use]
    pub const fn new(value: u128) -> Self {
        Self(value)
    }

    /// Return the raw millisecond count.
    #[must_use]
    pub const fn as_u128(self) -> u128 {
        self.0
    }
}

impl From<u128> for ZhenfaElapsedMillis {
    fn from(value: u128) -> Self {
        Self::new(value)
    }
}
