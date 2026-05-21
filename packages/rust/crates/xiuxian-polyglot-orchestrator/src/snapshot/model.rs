//! Read-only control-plane snapshots.

use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::admission::{AdmissionBudget, AdmissionDecision};
use crate::evidence::LaneEvidence;
use crate::lanes::PolyglotLane;
use crate::refs::{ContractOwner, RouteProfileRef};

/// Inert snapshot of polyglot lane refs, admission budgets, and evidence.
///
/// This type is intentionally read-only. It does not dispatch work, probe
/// workers, mutate queues, or select routes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PolyglotControlSnapshot {
    route_refs: Vec<RouteProfileRef>,
    admission_budgets: Vec<AdmissionBudget>,
    lane_evidence: Vec<LaneEvidence>,
}

impl PolyglotControlSnapshot {
    /// Creates an empty snapshot.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            route_refs: Vec::new(),
            admission_budgets: Vec::new(),
            lane_evidence: Vec::new(),
        }
    }

    /// Creates a snapshot from already materialized owner facts.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotInvariantError`] when refs are duplicated, or when
    /// more than one admission budget or evidence envelope exists for a lane.
    pub fn from_parts(
        route_refs: Vec<RouteProfileRef>,
        admission_budgets: Vec<AdmissionBudget>,
        lane_evidence: Vec<LaneEvidence>,
    ) -> Result<Self, SnapshotInvariantError> {
        let snapshot = Self {
            route_refs,
            admission_budgets,
            lane_evidence,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    /// Adds one route/profile reference and validates the resulting snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotInvariantError`] when the reference duplicates an
    /// existing reference.
    pub fn with_route_ref(
        mut self,
        reference: RouteProfileRef,
    ) -> Result<Self, SnapshotInvariantError> {
        self.route_refs.push(reference);
        self.validate()?;
        Ok(self)
    }

    /// Adds one admission budget and validates the resulting snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotInvariantError`] when a budget for the same lane
    /// already exists.
    pub fn with_admission_budget(
        mut self,
        budget: AdmissionBudget,
    ) -> Result<Self, SnapshotInvariantError> {
        self.admission_budgets.push(budget);
        self.validate()?;
        Ok(self)
    }

    /// Adds one lane evidence envelope and validates the resulting snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotInvariantError`] when evidence for the same lane
    /// already exists.
    pub fn with_lane_evidence(
        mut self,
        evidence: LaneEvidence,
    ) -> Result<Self, SnapshotInvariantError> {
        self.lane_evidence.push(evidence);
        self.validate()?;
        Ok(self)
    }

    /// Returns all route/profile references.
    #[must_use]
    pub fn route_refs(&self) -> &[RouteProfileRef] {
        &self.route_refs
    }

    /// Returns all admission budgets.
    #[must_use]
    pub fn admission_budgets(&self) -> &[AdmissionBudget] {
        &self.admission_budgets
    }

    /// Returns all lane evidence envelopes.
    #[must_use]
    pub fn lane_evidence(&self) -> &[LaneEvidence] {
        &self.lane_evidence
    }

    /// Returns route/profile refs for one lane.
    pub fn route_refs_for_lane(
        &self,
        lane: PolyglotLane,
    ) -> impl Iterator<Item = &RouteProfileRef> {
        self.route_refs
            .iter()
            .filter(move |reference| reference.lane == lane)
    }

    /// Returns route/profile refs for one contract owner.
    pub fn route_refs_for_owner(
        &self,
        owner: ContractOwner,
    ) -> impl Iterator<Item = &RouteProfileRef> {
        self.route_refs
            .iter()
            .filter(move |reference| reference.owner == owner)
    }

    /// Returns the admission budget for one lane.
    #[must_use]
    pub fn admission_budget_for_lane(&self, lane: PolyglotLane) -> Option<&AdmissionBudget> {
        self.admission_budgets
            .iter()
            .find(|budget| budget.lane == lane)
    }

    /// Returns the admission decision for one lane when a budget exists.
    #[must_use]
    pub fn admission_decision_for_lane(&self, lane: PolyglotLane) -> Option<AdmissionDecision> {
        self.admission_budget_for_lane(lane)
            .map(|budget| budget.decide())
    }

    /// Returns the evidence envelope for one lane.
    #[must_use]
    pub fn evidence_for_lane(&self, lane: PolyglotLane) -> Option<&LaneEvidence> {
        self.lane_evidence
            .iter()
            .find(|evidence| evidence.lane == lane)
    }

    /// Validates snapshot invariants.
    ///
    /// # Errors
    ///
    /// Returns [`SnapshotInvariantError`] when refs are duplicated, or when
    /// more than one admission budget or evidence envelope exists for a lane.
    pub fn validate(&self) -> Result<(), SnapshotInvariantError> {
        validate_unique_route_refs(&self.route_refs)?;
        validate_unique_budget_lanes(&self.admission_budgets)?;
        validate_unique_evidence_lanes(&self.lane_evidence)?;
        Ok(())
    }
}

/// Snapshot invariant failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotInvariantError {
    /// A route/profile reference was repeated exactly.
    RouteRef {
        /// Lane carrying the duplicated reference.
        lane: PolyglotLane,
        /// Owner carrying the duplicated reference.
        owner: ContractOwner,
        /// Duplicated route.
        route: String,
        /// Duplicated optional profile id.
        profile: Option<String>,
        /// Duplicated optional schema version.
        schema_version: Option<String>,
    },
    /// More than one admission budget exists for a lane.
    AdmissionBudget {
        /// Duplicated lane.
        lane: PolyglotLane,
    },
    /// More than one evidence envelope exists for a lane.
    LaneEvidence {
        /// Duplicated lane.
        lane: PolyglotLane,
    },
}

impl fmt::Display for SnapshotInvariantError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RouteRef {
                lane,
                owner,
                route,
                profile,
                schema_version,
            } => write!(
                formatter,
                "duplicate route/profile ref for lane `{}`, owner `{:?}`, route `{}`, profile `{:?}`, schema `{:?}`",
                lane.as_str(),
                owner,
                route,
                profile,
                schema_version
            ),
            Self::AdmissionBudget { lane } => write!(
                formatter,
                "duplicate admission budget for lane `{}`",
                lane.as_str()
            ),
            Self::LaneEvidence { lane } => {
                write!(
                    formatter,
                    "duplicate lane evidence for lane `{}`",
                    lane.as_str()
                )
            }
        }
    }
}

impl Error for SnapshotInvariantError {}

fn validate_unique_route_refs(
    references: &[RouteProfileRef],
) -> Result<(), SnapshotInvariantError> {
    for (index, reference) in references.iter().enumerate() {
        if references[..index]
            .iter()
            .any(|existing| existing == reference)
        {
            return Err(SnapshotInvariantError::RouteRef {
                lane: reference.lane,
                owner: reference.owner,
                route: reference.route.clone(),
                profile: reference.profile.clone(),
                schema_version: reference.schema_version.clone(),
            });
        }
    }
    Ok(())
}

fn validate_unique_budget_lanes(budgets: &[AdmissionBudget]) -> Result<(), SnapshotInvariantError> {
    for (index, budget) in budgets.iter().enumerate() {
        if budgets[..index]
            .iter()
            .any(|existing| existing.lane == budget.lane)
        {
            return Err(SnapshotInvariantError::AdmissionBudget { lane: budget.lane });
        }
    }
    Ok(())
}

fn validate_unique_evidence_lanes(evidence: &[LaneEvidence]) -> Result<(), SnapshotInvariantError> {
    for (index, current) in evidence.iter().enumerate() {
        if evidence[..index]
            .iter()
            .any(|existing| existing.lane == current.lane)
        {
            return Err(SnapshotInvariantError::LaneEvidence { lane: current.lane });
        }
    }
    Ok(())
}
