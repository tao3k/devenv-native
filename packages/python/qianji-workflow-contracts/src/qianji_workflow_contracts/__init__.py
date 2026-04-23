"""Pure workflow payload contracts for Qianji BPMN/DMN integrations."""

from __future__ import annotations

from .dataset_ref import DATASET_REF_KIND, WorkflowDatasetRef
from .decision import (
    DECISION_BATCH_KIND,
    DECISION_OUTCOME_KIND,
    DecisionBatch,
    DecisionOutcome,
    DecisionOutcomeRow,
    DecisionReason,
    DecisionRow,
)
from .envelope import WorkflowEnvelope, make_envelope, parse_envelope
from .errors import ContractError, ContractValidationError, ContractVersionError
from .execution import (
    EXECUTION_REF_KIND,
    EXECUTION_STATUS_KIND,
    WorkflowExecutionRef,
    WorkflowExecutionStatus,
)
from .host_work import (
    ALLOWED_HOST_WORK_STATUSES,
    HOST_WORK_REQUEST_KIND,
    HOST_WORK_RESULT_KIND,
    HostWorkRequest,
    HostWorkResult,
)
from .version import CONTRACT_VERSION, SUPPORTED_VERSIONS

_cached_version: str | None = None

__all__ = [
    "ALLOWED_HOST_WORK_STATUSES",
    "CONTRACT_VERSION",
    "DATASET_REF_KIND",
    "DECISION_BATCH_KIND",
    "DECISION_OUTCOME_KIND",
    "EXECUTION_REF_KIND",
    "EXECUTION_STATUS_KIND",
    "HOST_WORK_REQUEST_KIND",
    "HOST_WORK_RESULT_KIND",
    "SUPPORTED_VERSIONS",
    "ContractError",
    "ContractValidationError",
    "ContractVersionError",
    "DecisionBatch",
    "DecisionOutcome",
    "DecisionOutcomeRow",
    "DecisionReason",
    "DecisionRow",
    "HostWorkRequest",
    "HostWorkResult",
    "WorkflowDatasetRef",
    "WorkflowEnvelope",
    "WorkflowExecutionRef",
    "WorkflowExecutionStatus",
    "make_envelope",
    "parse_envelope",
]


def _get_version() -> str:
    """Lazy package-version lookup."""

    global _cached_version
    if _cached_version is None:
        from importlib.metadata import PackageNotFoundError, version

        try:
            _cached_version = version("qianji-workflow-contracts")
        except PackageNotFoundError:
            _cached_version = "0.0.0-dev"
    return _cached_version


def __getattr__(name: str):
    if name == "__version__":
        return _get_version()
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
