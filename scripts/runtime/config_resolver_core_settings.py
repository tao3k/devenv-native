#!/usr/bin/env python3
"""Compatibility facade for channel resolver settings parsing helpers."""

from __future__ import annotations

import config_resolver_core_settings_acl as _acl
import config_resolver_core_settings_telegram as _telegram

repo_root_from = _telegram.repo_root_from
settings_candidates = _telegram.settings_candidates
read_telegram_key_from_toml = _telegram.read_telegram_key_from_toml
read_telegram_acl_allow_users = _acl.read_telegram_acl_allow_users
