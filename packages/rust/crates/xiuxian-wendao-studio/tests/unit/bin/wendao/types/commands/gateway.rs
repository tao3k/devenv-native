use super::{GatewayArgs, GatewayCommand, GatewayStartArgs, gateway};
use crate::bin_support::wendao::types::Command;

#[test]
fn test_gateway_start_args_default() {
    let args = GatewayStartArgs::default();
    assert!(args.port.is_none());
}

#[test]
fn test_gateway_start_args_custom_port() {
    let args = GatewayStartArgs { port: Some(8080) };
    assert_eq!(args.port, Some(8080));
}

#[test]
fn test_gateway_command_creation() {
    let start_args = GatewayStartArgs { port: Some(8080) };
    let args = GatewayArgs {
        command: GatewayCommand::Start(start_args),
    };
    let cmd = gateway(&args);
    match cmd {
        Command::Gateway(ga) => match &ga.command {
            GatewayCommand::Start(sa) => assert_eq!(sa.port, Some(8080)),
        },
        _ => panic!("Expected Gateway command"),
    }
}

#[test]
fn test_gateway_args_clone() {
    let args = GatewayArgs {
        command: GatewayCommand::Start(GatewayStartArgs { port: Some(9517) }),
    };
    let cloned = args.clone();
    assert!(matches!(cloned.command, GatewayCommand::Start(_)));
}

#[test]
fn test_gateway_args_debug() {
    let args = GatewayArgs {
        command: GatewayCommand::Start(GatewayStartArgs { port: Some(9517) }),
    };
    let debug_str = format!("{args:?}");
    assert!(debug_str.contains("9517"));
}
