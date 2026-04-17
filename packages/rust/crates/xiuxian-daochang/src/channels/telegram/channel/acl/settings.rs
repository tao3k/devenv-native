use super::group_overrides::parse_group_overrides;
use super::normalization::{
    normalize_allowed_group_entries, normalize_allowed_user_entries_with_context,
    normalize_control_command_policy, normalize_group_allow_from, normalize_slash_command_policy,
};
use super::parsing::{
    parse_comma_entries, parse_optional_comma_entries, parse_semicolon_entries,
    resolve_bool_env_or_setting, resolve_optional_env_or_setting, resolve_string_env_or_setting,
};
use super::slash_policy::build_slash_command_policy;
use super::types::{
    TELEGRAM_ACL_FIELD_ADMIN_COMMAND_RULES, TELEGRAM_ACL_FIELD_ALLOWED_USERS, TelegramAclConfig,
};
use crate::TelegramAclOverrides;
use crate::TelegramSlashCommandPolicy;
use crate::build_telegram_acl_overrides_from_settings;
use crate::channels::control_command_authorization::ControlCommandPolicy;
use crate::channels::telegram::channel::admin_rules::parse_admin_command_rule_specs;
use crate::channels::telegram::channel::group_policy::{
    TelegramGroupPolicyConfig, parse_group_policy_mode,
};
use crate::config::TelegramSettings;

struct ResolvedSlashPolicyRaw {
    global: Option<String>,
    session_status: Option<String>,
    session_budget: Option<String>,
    session_memory: Option<String>,
    session_feedback: Option<String>,
    job_status: Option<String>,
    jobs_summary: Option<String>,
    background_submit: Option<String>,
}

struct ResolvedAclRaw {
    allowed_users: String,
    allowed_groups: String,
    session_admin_persist: bool,
    group_policy: String,
    group_allow_from: Option<String>,
    require_mention: bool,
    admin_users: String,
    control_command_allow_from: Option<String>,
    admin_command_rules: String,
    slash_policy: ResolvedSlashPolicyRaw,
}

pub(in crate::channels::telegram::channel) fn resolve_acl_config_from_settings(
    settings: TelegramSettings,
) -> anyhow::Result<TelegramAclConfig> {
    let acl_overrides = build_telegram_acl_overrides_from_settings(&settings)?;
    let resolved = resolve_acl_raw(&settings, &acl_overrides);

    let allowed_users = normalize_allowed_user_entries_with_context(
        parse_comma_entries(resolved.allowed_users.as_str()),
        TELEGRAM_ACL_FIELD_ALLOWED_USERS,
    );
    let allowed_groups =
        normalize_allowed_group_entries(parse_comma_entries(resolved.allowed_groups.as_str()));
    let group_policy =
        parse_group_policy_mode(resolved.group_policy.as_str(), "telegram.group_policy")
            .unwrap_or_default();
    let group_allow_from =
        normalize_group_allow_from(parse_optional_comma_entries(resolved.group_allow_from));
    let admin_users = parse_comma_entries(resolved.admin_users.as_str());
    let control_command_allow_from =
        parse_optional_comma_entries(resolved.control_command_allow_from);
    let admin_command_rules = parse_admin_command_rules(
        resolved.admin_command_rules.as_str(),
        acl_overrides.control_command_rules,
    )?;

    let slash_command_policy = TelegramSlashCommandPolicy {
        global: parse_optional_comma_entries(resolved.slash_policy.global),
        session_status: parse_optional_comma_entries(resolved.slash_policy.session_status),
        session_budget: parse_optional_comma_entries(resolved.slash_policy.session_budget),
        session_memory: parse_optional_comma_entries(resolved.slash_policy.session_memory),
        session_feedback: parse_optional_comma_entries(resolved.slash_policy.session_feedback),
        job_status: parse_optional_comma_entries(resolved.slash_policy.job_status),
        jobs_summary: parse_optional_comma_entries(resolved.slash_policy.jobs_summary),
        background_submit: parse_optional_comma_entries(resolved.slash_policy.background_submit),
    };

    let control_command_policy = normalize_control_command_policy(ControlCommandPolicy::new(
        admin_users.clone(),
        control_command_allow_from,
        admin_command_rules,
    ));
    let slash_command_policy = normalize_slash_command_policy(build_slash_command_policy(
        admin_users,
        slash_command_policy,
    ));
    let group_policy_config = TelegramGroupPolicyConfig {
        group_policy,
        group_allow_from,
        require_mention: resolved.require_mention,
        groups: parse_group_overrides(settings.groups.unwrap_or_default()),
    };

    Ok(TelegramAclConfig {
        allowed_users,
        allowed_groups,
        control_command_policy,
        slash_command_policy,
        group_policy_config,
        session_admin_persist: resolved.session_admin_persist,
    })
}

fn resolve_acl_raw(
    settings: &TelegramSettings,
    acl_overrides: &TelegramAclOverrides,
) -> ResolvedAclRaw {
    ResolvedAclRaw {
        allowed_users: resolve_string_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_ALLOWED_USERS",
            Some(acl_overrides.allowed_users.join(",")),
            "",
        ),
        allowed_groups: resolve_string_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_ALLOWED_GROUPS",
            Some(acl_overrides.allowed_groups.join(",")),
            "",
        ),
        session_admin_persist: resolve_bool_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_SESSION_ADMIN_PERSIST",
            settings.session_admin_persist,
            false,
        ),
        group_policy: resolve_string_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_GROUP_POLICY",
            settings.group_policy.clone(),
            "open",
        ),
        group_allow_from: resolve_optional_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_GROUP_ALLOW_FROM",
            settings.group_allow_from.clone(),
        ),
        require_mention: resolve_bool_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_REQUIRE_MENTION",
            settings.require_mention,
            false,
        ),
        admin_users: resolve_string_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_ADMIN_USERS",
            Some(acl_overrides.admin_users.join(",")),
            "",
        ),
        control_command_allow_from: resolve_optional_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_CONTROL_COMMAND_ALLOW_FROM",
            acl_overrides
                .control_command_allow_from
                .as_ref()
                .map(|entries| entries.join(",")),
        ),
        admin_command_rules: resolve_string_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_ADMIN_COMMAND_RULES",
            None,
            "",
        ),
        slash_policy: resolve_slash_policy_raw(acl_overrides),
    }
}

fn resolve_slash_policy_raw(acl_overrides: &TelegramAclOverrides) -> ResolvedSlashPolicyRaw {
    ResolvedSlashPolicyRaw {
        global: resolve_optional_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_SLASH_COMMAND_ALLOW_FROM",
            acl_overrides
                .slash_command_allow_from
                .as_ref()
                .map(|entries| entries.join(",")),
        ),
        session_status: resolve_optional_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_SLASH_SESSION_STATUS_ALLOW_FROM",
            acl_overrides
                .slash_session_status_allow_from
                .as_ref()
                .map(|entries| entries.join(",")),
        ),
        session_budget: resolve_optional_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_SLASH_SESSION_BUDGET_ALLOW_FROM",
            acl_overrides
                .slash_session_budget_allow_from
                .as_ref()
                .map(|entries| entries.join(",")),
        ),
        session_memory: resolve_optional_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_SLASH_SESSION_MEMORY_ALLOW_FROM",
            acl_overrides
                .slash_session_memory_allow_from
                .as_ref()
                .map(|entries| entries.join(",")),
        ),
        session_feedback: resolve_optional_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_SLASH_SESSION_FEEDBACK_ALLOW_FROM",
            acl_overrides
                .slash_session_feedback_allow_from
                .as_ref()
                .map(|entries| entries.join(",")),
        ),
        job_status: resolve_optional_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_SLASH_JOB_ALLOW_FROM",
            acl_overrides
                .slash_job_allow_from
                .as_ref()
                .map(|entries| entries.join(",")),
        ),
        jobs_summary: resolve_optional_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_SLASH_JOBS_ALLOW_FROM",
            acl_overrides
                .slash_jobs_allow_from
                .as_ref()
                .map(|entries| entries.join(",")),
        ),
        background_submit: resolve_optional_env_or_setting(
            "XIUXIAN_DAOCHANG_TELEGRAM_SLASH_BG_ALLOW_FROM",
            acl_overrides
                .slash_bg_allow_from
                .as_ref()
                .map(|entries| entries.join(",")),
        ),
    }
}

fn parse_admin_command_rules(
    admin_command_rules_raw: &str,
    default_rules: Vec<crate::TelegramCommandAdminRule>,
) -> anyhow::Result<Vec<crate::TelegramCommandAdminRule>> {
    if admin_command_rules_raw.trim().is_empty() {
        return Ok(default_rules);
    }

    let admin_command_specs = parse_semicolon_entries(admin_command_rules_raw);
    parse_admin_command_rule_specs(admin_command_specs)
        .map_err(|error| anyhow::anyhow!("{TELEGRAM_ACL_FIELD_ADMIN_COMMAND_RULES}: {error}"))
}
