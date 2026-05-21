#!/usr/bin/env python3
"""Compatibility facade for Telegram-specific config resolution helpers."""

from __future__ import annotations

import config_resolver_telegram_acl as _acl
import config_resolver_telegram_partition as _partition
import config_resolver_telegram_webhook as _webhook

telegram_webhook_secret_token = _webhook.telegram_webhook_secret_token
telegram_webhook_bind = _webhook.telegram_webhook_bind
telegram_webhook_port = _webhook.telegram_webhook_port
default_telegram_webhook_url = _webhook.default_telegram_webhook_url

telegram_session_partition_mode = _partition.telegram_session_partition_mode

allowed_users_from_settings = _acl.allowed_users_from_settings
username_from_settings = _acl.username_from_settings
