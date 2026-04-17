//! Agent-side external tool dispatch helpers.

mod diagnostics;
mod dispatch;
mod helpers;
mod llm_tools;
mod soft_fail;
mod tool_types;

#[cfg(test)]
#[path = "../../../tests/unit/agent/tool_dispatch/mod.rs"]
mod tests;
