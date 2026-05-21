//! Schema benchmark evidence contracts.

use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Candidate schema strategy for heterogeneous document or compute tables.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaStrategyCandidate {
    /// One stable Arrow schema per compute or document profile.
    ProfileSpecific,
    /// A normalized long-table shape with field descriptors.
    NormalizedLongTable,
    /// Nested or struct-heavy Arrow shapes that preserve hierarchy.
    NestedStructHeavy,
    /// A wide sparse global schema. This remains a benchmark candidate only.
    GlobalSuperSchema,
}

impl SchemaStrategyCandidate {
    /// Returns the stable machine-readable candidate identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProfileSpecific => "profile_specific",
            Self::NormalizedLongTable => "normalized_long_table",
            Self::NestedStructHeavy => "nested_struct_heavy",
            Self::GlobalSuperSchema => "global_super_schema",
        }
    }
}

/// Relative preference between two supplied benchmark evidence rows.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaStrategyPreference {
    /// The left-hand evidence has a lower advisory cost.
    Left,
    /// Both evidence rows have the same advisory cost.
    Tie,
    /// The right-hand evidence has a lower advisory cost.
    Right,
}

/// Encoded Arrow payload size in bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EncodedByteSize(u64);

impl EncodedByteSize {
    /// Creates encoded byte-size evidence.
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Returns the byte count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Cache pressure evidence in bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CachePressureBytes(u64);

impl CachePressureBytes {
    /// Creates cache pressure evidence.
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Returns the byte count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Memory pressure evidence in bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MemoryPressureBytes(u64);

impl MemoryPressureBytes {
    /// Creates memory pressure evidence.
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Returns the byte count.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Inert benchmark evidence supplied for one schema strategy candidate.
///
/// The evidence records supplied observations only. It does not benchmark live
/// data, mutate Arrow schemas, select routes, or choose a production default.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SchemaBenchmarkEvidence {
    /// Candidate strategy described by this evidence.
    pub candidate: SchemaStrategyCandidate,
    /// Owner-supplied validation cost units.
    pub validation_cost_units: u64,
    /// Owner-supplied row count.
    pub row_count: u64,
    /// Owner-supplied encoded byte size.
    pub encoded_bytes: EncodedByteSize,
    /// Owner-supplied null cell count.
    pub null_cells: u64,
    /// Owner-supplied total comparable cell count.
    pub total_cells: u64,
    /// Owner-supplied cache pressure in bytes.
    pub cache_pressure_bytes: CachePressureBytes,
    /// Owner-supplied memory pressure in bytes.
    pub memory_pressure_bytes: MemoryPressureBytes,
    /// Owner-supplied schema evolution cost units.
    pub schema_evolution_cost_units: u64,
    /// Owner-supplied lossy projection count.
    pub lossy_projection_count: u64,
}

impl SchemaBenchmarkEvidence {
    /// Creates empty evidence for one schema strategy candidate.
    #[must_use]
    pub const fn new(candidate: SchemaStrategyCandidate) -> Self {
        Self {
            candidate,
            validation_cost_units: 0,
            row_count: 0,
            encoded_bytes: EncodedByteSize::new(0),
            null_cells: 0,
            total_cells: 0,
            cache_pressure_bytes: CachePressureBytes::new(0),
            memory_pressure_bytes: MemoryPressureBytes::new(0),
            schema_evolution_cost_units: 0,
            lossy_projection_count: 0,
        }
    }

    /// Creates profile-specific candidate evidence.
    #[must_use]
    pub const fn profile_specific() -> Self {
        Self::new(SchemaStrategyCandidate::ProfileSpecific)
    }

    /// Creates normalized long-table candidate evidence.
    #[must_use]
    pub const fn normalized_long_table() -> Self {
        Self::new(SchemaStrategyCandidate::NormalizedLongTable)
    }

    /// Creates nested or struct-heavy candidate evidence.
    #[must_use]
    pub const fn nested_struct_heavy() -> Self {
        Self::new(SchemaStrategyCandidate::NestedStructHeavy)
    }

    /// Creates global super-schema candidate evidence.
    #[must_use]
    pub const fn global_super_schema() -> Self {
        Self::new(SchemaStrategyCandidate::GlobalSuperSchema)
    }

    /// Returns this evidence with validation cost units.
    #[must_use]
    pub const fn with_validation_cost(mut self, validation_cost_units: u64) -> Self {
        self.validation_cost_units = validation_cost_units;
        self
    }

    /// Returns this evidence with row count.
    #[must_use]
    pub const fn with_row_count(mut self, row_count: u64) -> Self {
        self.row_count = row_count;
        self
    }

    /// Returns this evidence with encoded byte size.
    #[must_use]
    pub const fn with_encoded_bytes(mut self, encoded_bytes: u64) -> Self {
        self.encoded_bytes = EncodedByteSize::new(encoded_bytes);
        self
    }

    /// Returns this evidence with null-density counters.
    #[must_use]
    pub const fn with_null_density(mut self, null_cells: u64, total_cells: u64) -> Self {
        self.null_cells = null_cells;
        self.total_cells = total_cells;
        self
    }

    /// Returns this evidence with cache and memory pressure.
    #[must_use]
    pub const fn with_pressure_bytes(
        mut self,
        cache_pressure_bytes: u64,
        memory_pressure_bytes: u64,
    ) -> Self {
        self.cache_pressure_bytes = CachePressureBytes::new(cache_pressure_bytes);
        self.memory_pressure_bytes = MemoryPressureBytes::new(memory_pressure_bytes);
        self
    }

    /// Returns this evidence with schema evolution cost units.
    #[must_use]
    pub const fn with_schema_evolution_cost(mut self, schema_evolution_cost_units: u64) -> Self {
        self.schema_evolution_cost_units = schema_evolution_cost_units;
        self
    }

    /// Returns this evidence with lossy projection count.
    #[must_use]
    pub const fn with_lossy_projection_count(mut self, lossy_projection_count: u64) -> Self {
        self.lossy_projection_count = lossy_projection_count;
        self
    }

    /// Returns null density in basis points.
    #[must_use]
    pub const fn null_density_basis_points(self) -> u64 {
        if self.total_cells == 0 {
            return 0;
        }
        self.null_cells.saturating_mul(10_000) / self.total_cells
    }

    /// Returns a deterministic advisory cost score.
    ///
    /// Lower scores are better. The score is only an ordering helper for
    /// supplied evidence; it is not a production schema-selection policy.
    #[must_use]
    pub const fn advisory_cost_score(self) -> u64 {
        self.validation_cost_units
            .saturating_add(kib(self.encoded_bytes.get()))
            .saturating_add(kib(self.cache_pressure_bytes.get()))
            .saturating_add(kib(self.memory_pressure_bytes.get()))
            .saturating_add(self.null_density_basis_points())
            .saturating_add(self.schema_evolution_cost_units)
            .saturating_add(self.lossy_projection_count.saturating_mul(10_000))
    }

    /// Compares this evidence against another supplied evidence row.
    #[must_use]
    pub const fn preference_against(self, other: Self) -> SchemaStrategyPreference {
        let left = self.advisory_cost_score();
        let right = other.advisory_cost_score();
        if left < right {
            SchemaStrategyPreference::Left
        } else if left > right {
            SchemaStrategyPreference::Right
        } else {
            SchemaStrategyPreference::Tie
        }
    }
}

/// Metadata for one reproducible schema benchmark case.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SchemaBenchmarkCase {
    /// Stable benchmark case identifier.
    pub case_id: String,
    /// Human-readable workload label.
    pub workload_label: String,
    /// Owner-supplied input row count for the case.
    pub input_rows: u64,
    /// Owner-supplied input byte count for the case.
    pub input_bytes: u64,
}

impl SchemaBenchmarkCase {
    /// Creates benchmark case metadata.
    #[must_use]
    pub fn new(case_id: impl Into<String>, workload_label: impl Into<String>) -> Self {
        Self {
            case_id: case_id.into(),
            workload_label: workload_label.into(),
            input_rows: 0,
            input_bytes: 0,
        }
    }

    /// Returns this case with owner-supplied input size facts.
    #[must_use]
    pub const fn with_input_size(mut self, input_rows: u64, input_bytes: u64) -> Self {
        self.input_rows = input_rows;
        self.input_bytes = input_bytes;
        self
    }
}

/// Advisory report over supplied schema benchmark evidence rows.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SchemaBenchmarkReport {
    case: SchemaBenchmarkCase,
    evidence: Vec<SchemaBenchmarkEvidence>,
}

impl SchemaBenchmarkReport {
    /// Creates a schema benchmark report from supplied evidence.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaBenchmarkReportError`] when evidence is empty or when
    /// more than one row exists for the same candidate.
    pub fn new(
        case: SchemaBenchmarkCase,
        evidence: Vec<SchemaBenchmarkEvidence>,
    ) -> Result<Self, SchemaBenchmarkReportError> {
        let report = Self { case, evidence };
        report.validate()?;
        Ok(report)
    }

    /// Returns the benchmark case metadata.
    #[must_use]
    pub const fn case(&self) -> &SchemaBenchmarkCase {
        &self.case
    }

    /// Returns supplied evidence rows.
    #[must_use]
    pub fn evidence(&self) -> &[SchemaBenchmarkEvidence] {
        &self.evidence
    }

    /// Returns evidence for a candidate.
    #[must_use]
    pub fn evidence_for_candidate(
        &self,
        candidate: SchemaStrategyCandidate,
    ) -> Option<&SchemaBenchmarkEvidence> {
        self.evidence
            .iter()
            .find(|evidence| evidence.candidate == candidate)
    }

    /// Returns the uniquely preferred evidence row when one exists.
    ///
    /// Ties return `None` because this report is advisory evidence, not a
    /// production schema-default selector.
    #[must_use]
    pub fn preferred_evidence(&self) -> Option<&SchemaBenchmarkEvidence> {
        let mut best: Option<&SchemaBenchmarkEvidence> = None;
        let mut tied = false;

        for evidence in &self.evidence {
            let Some(current_best) = best else {
                best = Some(evidence);
                continue;
            };

            let score = evidence.advisory_cost_score();
            let best_score = current_best.advisory_cost_score();
            if score < best_score {
                best = Some(evidence);
                tied = false;
            } else if score == best_score {
                tied = true;
            }
        }

        if tied { None } else { best }
    }

    /// Returns the uniquely preferred candidate when one exists.
    #[must_use]
    pub fn preferred_candidate(&self) -> Option<SchemaStrategyCandidate> {
        self.preferred_evidence().map(|evidence| evidence.candidate)
    }

    /// Validates report invariants.
    ///
    /// # Errors
    ///
    /// Returns [`SchemaBenchmarkReportError`] when evidence is empty or when
    /// more than one row exists for the same candidate.
    pub fn validate(&self) -> Result<(), SchemaBenchmarkReportError> {
        if self.evidence.is_empty() {
            return Err(SchemaBenchmarkReportError::EmptyEvidence {
                case_id: self.case.case_id.clone(),
            });
        }

        for (index, evidence) in self.evidence.iter().enumerate() {
            if self.evidence[..index]
                .iter()
                .any(|existing| existing.candidate == evidence.candidate)
            {
                return Err(SchemaBenchmarkReportError::DuplicateCandidate {
                    case_id: self.case.case_id.clone(),
                    candidate: evidence.candidate,
                });
            }
        }

        Ok(())
    }
}

/// Schema benchmark report invariant failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SchemaBenchmarkReportError {
    /// The report contains no supplied evidence rows.
    EmptyEvidence {
        /// Benchmark case identifier.
        case_id: String,
    },
    /// The report contains more than one row for a candidate.
    DuplicateCandidate {
        /// Benchmark case identifier.
        case_id: String,
        /// Duplicated candidate.
        candidate: SchemaStrategyCandidate,
    },
}

impl fmt::Display for SchemaBenchmarkReportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyEvidence { case_id } => {
                write!(
                    formatter,
                    "schema benchmark case `{case_id}` has no evidence"
                )
            }
            Self::DuplicateCandidate { case_id, candidate } => write!(
                formatter,
                "schema benchmark case `{}` repeats candidate `{}`",
                case_id,
                candidate.as_str()
            ),
        }
    }
}

impl Error for SchemaBenchmarkReportError {}

const fn kib(bytes: u64) -> u64 {
    bytes / 1024
}
