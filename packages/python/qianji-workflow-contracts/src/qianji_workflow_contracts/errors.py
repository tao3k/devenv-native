"""Workflow contract error types."""

from __future__ import annotations


class ContractError(Exception):
    """Base error for workflow contract handling."""


class ContractValidationError(ContractError, ValueError):
    """Raised when a payload violates the expected contract shape."""


class ContractVersionError(ContractValidationError):
    """Raised when a payload carries an unsupported contract version."""
