//! Typed scalar boundaries for public BPMN snapshot DTOs.

use std::ops::{Deref, Not};

/// Stable BPMN snapshot identifier value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct BpmnSnapshotId(String);

impl BpmnSnapshotId {
    /// Returns this identifier as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for BpmnSnapshotId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for BpmnSnapshotId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for BpmnSnapshotId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for BpmnSnapshotId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// BPMN catalog kind value preserved from source XML.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct BpmnSnapshotKind(String);

impl BpmnSnapshotKind {
    /// Returns this kind as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for BpmnSnapshotKind {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for BpmnSnapshotKind {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for BpmnSnapshotKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for BpmnSnapshotKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// BPMN catalog type value preserved from source XML.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct BpmnSnapshotType(String);

impl BpmnSnapshotType {
    /// Returns this type as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for BpmnSnapshotType {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Deref for BpmnSnapshotType {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for BpmnSnapshotType {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<&str> for BpmnSnapshotType {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// BPMN boolean marker preserved from source XML.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct BpmnSnapshotFlag(bool);

impl BpmnSnapshotFlag {
    /// Returns the raw boolean marker.
    #[must_use]
    pub const fn get(self) -> bool {
        self.0
    }
}

impl From<bool> for BpmnSnapshotFlag {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl PartialEq<bool> for BpmnSnapshotFlag {
    fn eq(&self, other: &bool) -> bool {
        self.0 == *other
    }
}

impl From<BpmnSnapshotFlag> for bool {
    fn from(value: BpmnSnapshotFlag) -> Self {
        value.0
    }
}

impl Not for BpmnSnapshotFlag {
    type Output = bool;

    fn not(self) -> Self::Output {
        !self.0
    }
}
