"""Stable protocol-version definitions for workflow contracts."""

from __future__ import annotations

from .errors import ContractValidationError, ContractVersionError

CONTRACT_VERSION = "0.1"
SUPPORTED_VERSIONS = (CONTRACT_VERSION,)


def contract_major(version: str) -> str:
    """Return the major component of a contract version."""

    normalized = version.strip()
    if not normalized:
        raise ContractValidationError("contract_version must be a non-empty string")
    return normalized.split(".", maxsplit=1)[0]


def ensure_supported_contract_version(version: str) -> str:
    """Accept exact versions and compatible future minor versions.

    The current rule accepts:

    1. exact supported versions
    2. future minor versions that keep the same major component

    Any version with a different major component is rejected.
    """

    normalized = version.strip()
    if not normalized:
        raise ContractValidationError("contract_version must be a non-empty string")
    if normalized in SUPPORTED_VERSIONS:
        return normalized

    supported_majors = {contract_major(candidate) for candidate in SUPPORTED_VERSIONS}
    incoming_major = contract_major(normalized)
    if incoming_major not in supported_majors:
        raise ContractVersionError(
            f"unsupported contract_version {normalized!r}; supported majors are "
            f"{sorted(supported_majors)!r}"
        )
    return normalized
