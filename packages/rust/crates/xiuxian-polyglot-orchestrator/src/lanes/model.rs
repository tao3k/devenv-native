//! Lane identity and capability classification.

use serde::{Deserialize, Serialize};

/// A polyglot execution lane coordinated by Rust.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolyglotLane {
    /// Python-backed document extraction and OCR work.
    PythonDocling,
    /// Julia-backed numerical and profile compute work.
    JuliaCompute,
}

impl PolyglotLane {
    /// Returns the stable machine-readable lane identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PythonDocling => "python_docling",
            Self::JuliaCompute => "julia_compute",
        }
    }

    /// Returns true when this lane is implemented by the Python analyzer.
    #[must_use]
    pub const fn is_python_worker(self) -> bool {
        matches!(self, Self::PythonDocling)
    }

    /// Returns true when this lane is implemented by Julia compute.
    #[must_use]
    pub const fn is_julia_compute(self) -> bool {
        matches!(self, Self::JuliaCompute)
    }
}

/// A capability class exposed by a polyglot lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneCapability {
    /// Full-document extraction through the existing analyzer service.
    DocumentExtraction,
    /// OCR shard extraction through the existing analyzer service.
    OcrShardExtraction,
    /// Audio shard transcription through the existing analyzer service.
    AudioShardTranscription,
    /// Julia-side graph evidence projection.
    GraphEvidenceCompute,
    /// Julia-side graph search, structural rerank, and constraint filtering.
    GraphSearchCompute,
    /// Julia-side scientific or relational compute.
    ScientificCompute,
    /// Julia memory-family profile compute.
    MemoryProfileCompute,
}

impl LaneCapability {
    /// Returns the lane that currently owns this capability.
    #[must_use]
    pub const fn owning_lane(self) -> PolyglotLane {
        match self {
            Self::DocumentExtraction | Self::OcrShardExtraction | Self::AudioShardTranscription => {
                PolyglotLane::PythonDocling
            }
            Self::GraphEvidenceCompute
            | Self::GraphSearchCompute
            | Self::ScientificCompute
            | Self::MemoryProfileCompute => PolyglotLane::JuliaCompute,
        }
    }
}
