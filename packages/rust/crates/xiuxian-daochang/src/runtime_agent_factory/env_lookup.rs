use xiuxian_macros::env_non_empty;

pub(in crate::runtime_agent_factory) fn non_empty_env(name: &str) -> Option<String> {
    env_non_empty!(name)
}
