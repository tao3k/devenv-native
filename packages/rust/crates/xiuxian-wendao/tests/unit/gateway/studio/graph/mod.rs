use std::sync::Arc;

use super::*;
use crate::gateway::studio::test_support::assert_studio_json_snapshot;
use crate::gateway::studio::types::UiConfig;
use crate::gateway::studio::{GatewayState, StudioState};
use serde::Deserialize;
use serde_json::json;
use tempfile::tempdir;

mod configured_projects;
mod live_neighbors;
mod markdown_analysis;
mod pathing;
mod support;
mod topology;
