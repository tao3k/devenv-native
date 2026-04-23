//! Qianji Graphical Style (QGS) Constants
//! Inspired by `PaperBanana` academic aesthetics.

const BG_COLOR: &str = "#F9FAFB";
const BORDER_COLOR: &str = "#374151";
const TASK_BG: &str = "#EFF6FF";
const GATEWAY_BG: &str = "#FFFBEB";
const FONT_FAMILY: &str = "Inter, sans-serif";
const STROKE_WIDTH: f32 = 1.5;

/// Represents a graphical theme for Qianji visualization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QgsTheme {
    /// Global background color.
    pub background: String,
    /// Global border/stroke color.
    pub border: String,
    /// Fill color for Task-type nodes.
    pub task_fill: String,
    /// Fill color for Gateway-type nodes.
    pub gateway_fill: String,
    /// Font family used in the diagram.
    pub font_family: String,
    /// Stroke width for all elements.
    pub stroke_width: f32,
}

impl Default for QgsTheme {
    fn default() -> Self {
        Self {
            background: BG_COLOR.into(),
            border: BORDER_COLOR.into(),
            task_fill: TASK_BG.into(),
            gateway_fill: GATEWAY_BG.into(),
            font_family: FONT_FAMILY.into(),
            stroke_width: STROKE_WIDTH,
        }
    }
}
