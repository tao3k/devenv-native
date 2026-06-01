use super::{gateway_bearer_token_with_lookup, gateway_internal_principal_secret_with_lookup};

#[test]
fn gateway_bearer_token_defaults_to_disabled() {
    assert!(gateway_bearer_token_with_lookup(&|_| None).is_none());
}

#[test]
fn gateway_bearer_token_trims_non_empty_env_value() {
    assert_eq!(
        gateway_bearer_token_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_BEARER_TOKEN" => Some("  wd_test  ".to_string()),
            _ => None,
        })
        .as_deref(),
        Some("wd_test")
    );
}

#[test]
fn gateway_bearer_token_ignores_blank_env_value() {
    assert!(
        gateway_bearer_token_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_BEARER_TOKEN" => Some("   ".to_string()),
            _ => None,
        })
        .is_none()
    );
}

#[test]
fn gateway_internal_principal_secret_trims_non_empty_env_value() {
    assert_eq!(
        gateway_internal_principal_secret_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_INTERNAL_PRINCIPAL_SECRET" => {
                Some("  internal-secret  ".to_string())
            }
            _ => None,
        })
        .as_deref(),
        Some("internal-secret")
    );
}

#[test]
fn gateway_internal_principal_secret_ignores_blank_env_value() {
    assert!(
        gateway_internal_principal_secret_with_lookup(&|key| match key {
            "XIUXIAN_WENDAO_GATEWAY_INTERNAL_PRINCIPAL_SECRET" => Some("   ".to_string()),
            _ => None,
        })
        .is_none()
    );
}
