//! Agent journal recording helpers.

use crate::{
    AgentDecision, AgentProposal, ControlError, ControlEvent, ControlEventKind, ControlEventRecord,
    ControlLedger, ControlResult, RunId, StepId,
};

/// Journal scope for Agent proposal and decision facts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "scope", rename_all = "snake_case")]
pub enum AgentJournalScope {
    /// Record the Agent fact at run scope.
    Run,
    /// Record the Agent fact at step scope.
    Step {
        /// Owning step id.
        step_id: StepId,
    },
}

impl AgentJournalScope {
    /// Creates a run-scoped journal scope.
    #[must_use]
    pub const fn run() -> Self {
        Self::Run
    }

    /// Creates a step-scoped journal scope.
    #[must_use]
    pub const fn step(step_id: StepId) -> Self {
        Self::Step { step_id }
    }

    fn step_id(&self) -> Option<&StepId> {
        match self {
            Self::Run => None,
            Self::Step { step_id } => Some(step_id),
        }
    }

    fn into_event(
        self,
        run_id: RunId,
        occurred_at_ms: u64,
        kind: ControlEventKind,
    ) -> ControlEvent {
        match self {
            Self::Run => ControlEvent::run(run_id, occurred_at_ms, kind),
            Self::Step { step_id } => ControlEvent::step(run_id, step_id, occurred_at_ms, kind),
        }
    }
}

/// Named request for recording one Agent proposal journal fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AgentProposalJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Journal scope.
    pub scope: AgentJournalScope,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Agent proposal payload.
    pub proposal: AgentProposal,
}

impl AgentProposalJournalRecord {
    /// Creates a proposal journal record request.
    #[must_use]
    pub const fn new(
        run_id: RunId,
        scope: AgentJournalScope,
        occurred_at_ms: u64,
        proposal: AgentProposal,
    ) -> Self {
        Self {
            run_id,
            scope,
            occurred_at_ms,
            proposal,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            run_id,
            scope,
            occurred_at_ms,
            proposal,
        } = self;
        scope.into_event(
            run_id,
            occurred_at_ms,
            ControlEventKind::AgentProposalRecorded { proposal },
        )
    }
}

/// Named request for recording one Agent decision journal fact.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AgentDecisionJournalRecord {
    /// Owning run id.
    pub run_id: RunId,
    /// Journal scope.
    pub scope: AgentJournalScope,
    /// Event timestamp supplied by caller.
    pub occurred_at_ms: u64,
    /// Agent decision payload.
    pub decision: AgentDecision,
}

impl AgentDecisionJournalRecord {
    /// Creates a decision journal record request.
    #[must_use]
    pub const fn new(
        run_id: RunId,
        scope: AgentJournalScope,
        occurred_at_ms: u64,
        decision: AgentDecision,
    ) -> Self {
        Self {
            run_id,
            scope,
            occurred_at_ms,
            decision,
        }
    }

    /// Converts this record into its replayable control event.
    #[must_use]
    pub fn into_event(self) -> ControlEvent {
        let Self {
            run_id,
            scope,
            occurred_at_ms,
            decision,
        } = self;
        scope.into_event(
            run_id,
            occurred_at_ms,
            ControlEventKind::AgentDecisionRecorded { decision },
        )
    }
}

/// Records one Agent proposal as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the proposal is invalid, the step-scoped
/// journal scope does not match the proposal step, or the ledger append fails.
pub fn record_agent_proposal<L>(
    ledger: &L,
    request: AgentProposalJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    request.proposal.validate()?;
    validate_proposal_scope(&request.scope, &request.proposal)?;
    ledger.append_event(request.into_event())
}

/// Records one deterministic Agent decision as an append-only control event.
///
/// # Errors
///
/// Returns a control error when the decision is invalid or the ledger append
/// fails.
pub fn record_agent_decision<L>(
    ledger: &L,
    request: AgentDecisionJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    request.decision.validate()?;
    ledger.append_event(request.into_event())
}

fn validate_proposal_scope(
    scope: &AgentJournalScope,
    proposal: &AgentProposal,
) -> ControlResult<()> {
    let Some(step_id) = scope.step_id() else {
        return Ok(());
    };
    if step_id == &proposal.step_id {
        return Ok(());
    }
    Err(ControlError::InvalidEventSequence {
        message: format!(
            "agent proposal scope step `{}` does not match proposal step `{}`",
            step_id.as_str(),
            proposal.step_id.as_str()
        ),
    })
}
