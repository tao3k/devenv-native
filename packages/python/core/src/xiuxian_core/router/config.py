"""Router configuration schema and loader.

This module defines the remaining router-search configuration helpers used by
surviving protocol-neutral utilities.
"""

from __future__ import annotations

import json
from pathlib import Path

from pydantic import BaseModel, ConfigDict, Field, model_validator

from xiuxian_foundation.api.schema_locator import resolve_schema_file_path
from xiuxian_foundation.config.settings import get_setting, get_settings


def _coerce_int(value: object, *, default: int) -> int:
    """Parse integer config with safe default fallback."""
    if value is None:
        return default
    try:
        return int(value)
    except (TypeError, ValueError):
        return default


def _coerce_float(value: object, *, default: float) -> float:
    """Parse float config with safe default fallback."""
    if value is None:
        return default
    try:
        return float(value)
    except (TypeError, ValueError):
        return default


def _coerce_bool(value: object, *, default: bool) -> bool:
    """Parse bool config with string-safe handling."""
    if value is None:
        return default
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        normalized = value.strip().lower()
        if normalized in {"1", "true", "yes", "on"}:
            return True
        if normalized in {"0", "false", "no", "off"}:
            return False
        return default
    return bool(value)


class RouterConfidenceProfile(BaseModel):
    """Validated confidence profile values."""

    model_config = ConfigDict(frozen=True, extra="forbid")

    high_threshold: float = Field(0.75, ge=0.0, le=1.0)
    medium_threshold: float = Field(0.5, ge=0.0, le=1.0)
    high_base: float = Field(0.90, ge=0.0, le=1.0)
    high_scale: float = Field(0.05, ge=0.0, le=1.0)
    high_cap: float = Field(0.99, ge=0.0, le=1.0)
    medium_base: float = Field(0.60, ge=0.0, le=1.0)
    medium_scale: float = Field(0.30, ge=0.0, le=1.0)
    medium_cap: float = Field(0.89, ge=0.0, le=1.0)
    low_floor: float = Field(0.10, ge=0.0, le=1.0)

    @model_validator(mode="after")
    def _validate_profile(self) -> RouterConfidenceProfile:
        if self.high_threshold < self.medium_threshold:
            raise ValueError(
                "profile.high_threshold must be >= profile.medium_threshold"
            )
        if self.high_cap < self.high_base:
            raise ValueError("profile.high_cap must be >= profile.high_base")
        if self.medium_cap < self.medium_base:
            raise ValueError("profile.medium_cap must be >= profile.medium_base")
        return self


def _default_profiles() -> dict[str, dict[str, float]]:
    return {
        "balanced": {
            "high_threshold": 0.75,
            "medium_threshold": 0.50,
            "high_base": 0.90,
            "high_scale": 0.05,
            "high_cap": 0.99,
            "medium_base": 0.60,
            "medium_scale": 0.30,
            "medium_cap": 0.89,
            "low_floor": 0.10,
        },
        "precision": {
            "high_threshold": 0.82,
            "medium_threshold": 0.58,
            "high_base": 0.92,
            "high_scale": 0.04,
            "high_cap": 0.99,
            "medium_base": 0.62,
            "medium_scale": 0.24,
            "medium_cap": 0.88,
            "low_floor": 0.10,
        },
        "recall": {
            "high_threshold": 0.68,
            "medium_threshold": 0.42,
            "high_base": 0.88,
            "high_scale": 0.06,
            "high_cap": 0.99,
            "medium_base": 0.56,
            "medium_scale": 0.35,
            "medium_cap": 0.90,
            "low_floor": 0.08,
        },
    }


class RouterSearchConfig(BaseModel):
    """Validated router search configuration."""

    model_config = ConfigDict(frozen=True, extra="forbid")

    active_profile: str = "balanced"
    auto_profile_select: bool = True
    profiles: dict[str, RouterConfidenceProfile] = Field(
        default_factory=_default_profiles
    )
    default_limit: int = Field(10, ge=1)
    default_threshold: float = Field(0.2, ge=0.0, le=1.0)
    rerank: bool = True
    semantic_weight: float = Field(0.7, ge=0.0, le=1.0)
    keyword_weight: float = Field(0.3, ge=0.0, le=1.0)
    adaptive_threshold_step: float = Field(0.15, ge=0.0, le=1.0)
    adaptive_max_attempts: int = Field(3, ge=1)

    @model_validator(mode="after")
    def _validate_threshold_order(self) -> RouterSearchConfig:
        if not self.profiles:
            raise ValueError("router.search.profiles must not be empty")
        if self.active_profile not in self.profiles:
            raise ValueError(
                "router.search.active_profile must reference an existing entry "
                "in router.search.profiles"
            )
        return self

    @property
    def active_confidence_profile(self) -> RouterConfidenceProfile:
        """Return the selected confidence profile."""
        return self.profiles[self.active_profile]


def load_router_search_config(
    *,
    semantic_weight: float | None = None,
    keyword_weight: float | None = None,
    adaptive_threshold_step: float | None = None,
    adaptive_max_attempts: int | None = None,
) -> RouterSearchConfig:
    """Load and validate router search settings.

    Explicit arguments override settings values.
    """

    active_profile = str(
        get_setting("router.search.active_profile", "balanced") or "balanced"
    )
    profiles = (
        get_setting("router.search.profiles", _default_profiles())
        or _default_profiles()
    )

    return RouterSearchConfig(
        active_profile=active_profile,
        auto_profile_select=_coerce_bool(
            get_setting("router.search.auto_profile_select", True),
            default=True,
        ),
        profiles=profiles,
        default_limit=_coerce_int(
            get_setting("router.search.default_limit", 10), default=10
        ),
        default_threshold=_coerce_float(
            get_setting("router.search.default_threshold", 0.2),
            default=0.2,
        ),
        rerank=_coerce_bool(get_setting("router.search.rerank", True), default=True),
        semantic_weight=(
            _coerce_float(
                get_setting("router.search.semantic_weight", 0.7), default=0.7
            )
            if semantic_weight is None
            else semantic_weight
        ),
        keyword_weight=(
            _coerce_float(get_setting("router.search.keyword_weight", 0.3), default=0.3)
            if keyword_weight is None
            else keyword_weight
        ),
        adaptive_threshold_step=(
            _coerce_float(
                get_setting("router.search.adaptive_threshold_step", 0.15), default=0.15
            )
            if adaptive_threshold_step is None
            else adaptive_threshold_step
        ),
        adaptive_max_attempts=(
            _coerce_int(
                get_setting("router.search.adaptive_max_attempts", 3), default=3
            )
            if adaptive_max_attempts is None
            else adaptive_max_attempts
        ),
    )


def router_search_json_schema() -> dict:
    """Export JSON schema for router search settings."""
    return RouterSearchConfig.model_json_schema()


_ROUTER_SEARCH_SCHEMA_NAME = "xiuxian.router.search_config.v1.schema.json"


def resolve_router_schema_path(schema_path: str | Path | None = None) -> Path:
    """Resolve path to router search config schema file.

    Priority:
    1. Explicit `schema_path` argument
    2. `router.search.schema_file` from settings
    3. Default: resolved schema in Rust crate resources (`xiuxian-wendao/resources`)

    Paths starting with "packages/" are resolved relative to project root;
    other relative paths are resolved relative to conf_dir (--conf).
    """
    if schema_path is not None:
        return Path(schema_path)

    configured = get_setting("router.search.schema_file")
    if configured:
        configured_path = Path(str(configured))
        if configured_path.is_absolute():
            return configured_path
        if str(configured_path).replace("\\", "/").startswith("packages/"):
            from xiuxian_foundation.runtime.gitops import get_project_root

            return get_project_root() / configured_path
        conf_dir = Path(get_settings().conf_dir)
        return conf_dir / configured_path

    resolved = resolve_schema_file_path(
        _ROUTER_SEARCH_SCHEMA_NAME,
        preferred_crates=("xiuxian-wendao",),
    )
    if resolved.exists():
        return resolved
    conf_dir = Path(get_settings().conf_dir)
    return conf_dir / _ROUTER_SEARCH_SCHEMA_NAME


def write_router_search_json_schema(schema_path: str | Path | None = None) -> Path:
    """Write router search JSON schema to config home (or explicit path)."""
    output = resolve_router_schema_path(schema_path)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(router_search_json_schema(), indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return output
