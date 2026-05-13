//! Flowhub show rendering branch for root, module, and scenario views.

#[path = "root.rs"]
mod root;
#[path = "scenario.rs"]
mod scenario;
pub(crate) use root::render_flowhub_root_module_section_lines;
pub(crate) use scenario::render_scenario_case_section_lines;
#[cfg(test)]
pub(crate) use scenario::render_scenario_case_summary_block;
#[path = "facade.rs"]
mod facade;

#[cfg(test)]
pub(crate) use facade::render_flowhub_module_contract_section_lines;
pub(crate) use facade::{
    FLOWHUB_ROOT_MODULE_TEMPLATE_NAME, FLOWHUB_SCENARIO_CASE_TEMPLATE_NAME, module_kind_label,
    render_embedded_flowhub_block, render_flowhub_show_impl,
};
