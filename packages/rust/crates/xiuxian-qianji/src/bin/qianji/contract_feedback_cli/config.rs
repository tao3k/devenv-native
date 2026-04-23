use std::time::{SystemTime, UNIX_EPOCH};

use xiuxian_testing::{AdvisoryAuditPolicy, ContractRunConfig, FindingSeverity};

use super::types::{REST_DOCS_PACK_ID, RestDocsCliCommand};

pub(crate) fn build_contract_feedback_config(command: &RestDocsCliCommand) -> ContractRunConfig {
    let mut config = ContractRunConfig {
        generated_at: generated_at_string(),
        ..ContractRunConfig::default()
    };

    if command.live_advisory || !command.roles.is_empty() {
        config.set_advisory_policy_for_pack(
            REST_DOCS_PACK_ID,
            AdvisoryAuditPolicy {
                enabled: true,
                requested_roles: command.roles.clone(),
                min_severity: FindingSeverity::Warning,
            },
        );
    }

    config
}

fn generated_at_string() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_error| "0".to_string(),
        |duration| duration.as_millis().to_string(),
    )
}
