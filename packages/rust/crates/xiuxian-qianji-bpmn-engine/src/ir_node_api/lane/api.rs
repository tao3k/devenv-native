//! Public ir node api lane contracts for BPMN/DMN engine integration.

use std::sync::Arc;

/// Passive BPMN lane membership metadata attached to one flow node.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnLaneMembershipSpec {
    /// Optional source-level lane-set identifier.
    #[serde(
        default,
        rename = "lane_set_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub set_id: Option<Arc<str>>,
    /// Optional source-level lane-set name.
    #[serde(
        default,
        rename = "lane_set_name",
        skip_serializing_if = "Option::is_none"
    )]
    pub set_name: Option<Arc<str>>,
    /// Optional source-level lane identifier.
    #[serde(default, rename = "lane_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<Arc<str>>,
    /// Optional source-level lane name.
    #[serde(default, rename = "lane_name", skip_serializing_if = "Option::is_none")]
    pub name: Option<Arc<str>>,
}

impl BpmnLaneMembershipSpec {
    /// Creates an empty passive lane membership snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self {
            set_id: None,
            set_name: None,
            id: None,
            name: None,
        }
    }

    /// Attaches a source-level lane-set identifier.
    #[must_use]
    pub fn with_lane_set_id(mut self, lane_set_id: impl AsRef<str>) -> Self {
        self.set_id = Some(Arc::<str>::from(lane_set_id.as_ref()));
        self
    }

    /// Attaches a source-level lane-set name.
    #[must_use]
    pub fn with_lane_set_name(mut self, lane_set_name: impl AsRef<str>) -> Self {
        self.set_name = Some(Arc::<str>::from(lane_set_name.as_ref()));
        self
    }

    /// Attaches a source-level lane identifier.
    #[must_use]
    pub fn with_lane_id(mut self, lane_id: impl AsRef<str>) -> Self {
        self.id = Some(Arc::<str>::from(lane_id.as_ref()));
        self
    }

    /// Attaches a source-level lane name.
    #[must_use]
    pub fn with_lane_name(mut self, lane_name: impl AsRef<str>) -> Self {
        self.name = Some(Arc::<str>::from(lane_name.as_ref()));
        self
    }
}

impl Default for BpmnLaneMembershipSpec {
    fn default() -> Self {
        Self::new()
    }
}
