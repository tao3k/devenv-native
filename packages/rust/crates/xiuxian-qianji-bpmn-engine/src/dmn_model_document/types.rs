//! Typed scalar boundaries for public DMN snapshot DTOs.

use std::ops::{Deref, Not};

/// DMN identifier text preserved from source XML.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct DmnSnapshotId(String);

impl DmnSnapshotId {
    /// Borrows the serialized identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DmnSnapshotId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for DmnSnapshotId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for DmnSnapshotId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for DmnSnapshotId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// DMN kind/tag text preserved from source XML.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct DmnSnapshotKind(String);

impl DmnSnapshotKind {
    /// Borrows the serialized kind.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DmnSnapshotKind {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for DmnSnapshotKind {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for DmnSnapshotKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for DmnSnapshotKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// DMN type URI or catalog text preserved from source XML.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct DmnSnapshotType(String);

impl DmnSnapshotType {
    /// Borrows the serialized type.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for DmnSnapshotType {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for DmnSnapshotType {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for DmnSnapshotType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for DmnSnapshotType {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// DMN boolean marker preserved from source XML.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct DmnSnapshotFlag(bool);

impl DmnSnapshotFlag {
    /// Returns the serialized boolean value.
    #[must_use]
    pub const fn get(self) -> bool {
        self.0
    }
}

impl From<bool> for DmnSnapshotFlag {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<DmnSnapshotFlag> for bool {
    fn from(value: DmnSnapshotFlag) -> Self {
        value.0
    }
}

impl Not for DmnSnapshotFlag {
    type Output = bool;

    fn not(self) -> Self::Output {
        !self.0
    }
}

impl std::fmt::Display for DmnSnapshotFlag {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
