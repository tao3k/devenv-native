use std::fmt::Write as _;

use crate::bpmn_cli::deps::QianjiBpmnWorkflowInstancesReport;
use crate::bpmn_cli::types::{BpmnCliOutput, BpmnInstancesCliCommand};

use super::support::{bpmn_checkpoint_backend_label, bpmn_lifecycle_label};

pub(crate) fn render_bpmn_instances_output(
    _command: &BpmnInstancesCliCommand,
    report: &QianjiBpmnWorkflowInstancesReport,
) -> BpmnCliOutput {
    let mut rendered = format!(
        "# BPMN Instances\n\nCheckpoint backend: {}\nInstance count: {}\n",
        bpmn_checkpoint_backend_label(&report.checkpoint_store),
        report.instances.len(),
    );

    if !report.instances.is_empty() {
        let _ = writeln!(rendered, "\n## Instances");
        for instance in &report.instances {
            let _ = writeln!(
                rendered,
                "- {} | lifecycle={} | process={} | package={} | checkpoint_sequence={} | state_sequence={} | updated_at_ms={} | active_tokens={} | pending_host={} | waits={}",
                instance.instance_id,
                bpmn_lifecycle_label(&instance.lifecycle),
                instance.process_id,
                instance.package_id,
                instance.checkpoint_sequence,
                instance.state_sequence,
                instance.updated_at_ms,
                instance.active_token_count,
                instance.pending_host_work_count,
                instance.wait_registration_count,
            );
        }
    }

    BpmnCliOutput {
        rendered,
        exit_code: 0,
    }
}
