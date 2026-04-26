use std::io;

use super::invalid_input;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TemplateCliCommand {
    Bpmn,
    Dmn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TemplateCliOutput {
    pub(crate) rendered: String,
}

pub(super) fn handle_template_command(command: &TemplateCliCommand) {
    let output = run_template_command(command);
    println!("{}", output.rendered);
}

pub(super) fn run_template_command(command: &TemplateCliCommand) -> TemplateCliOutput {
    let rendered = match command {
        TemplateCliCommand::Bpmn => bpmn_template(),
        TemplateCliCommand::Dmn => dmn_template(),
    };
    TemplateCliOutput {
        rendered: rendered.to_string(),
    }
}

pub(super) fn parse_template_command(args: &[String]) -> io::Result<Option<TemplateCliCommand>> {
    let Some(command_name) = args.get(1).map(String::as_str) else {
        return Ok(None);
    };
    if command_name != "template" {
        return Ok(None);
    }

    let mut index = 2;
    let mut bpmn = false;
    let mut dmn = false;
    while index < args.len() {
        match args[index].as_str() {
            "--bpmn" => bpmn = true,
            "--dmn" => dmn = true,
            other => {
                return Err(invalid_input(format!(
                    "unsupported `template` option `{other}`"
                )));
            }
        }

        index += 1;
    }

    match (bpmn, dmn) {
        (true, false) => Ok(Some(TemplateCliCommand::Bpmn)),
        (false, true) => Ok(Some(TemplateCliCommand::Dmn)),
        (false, false) => Err(invalid_input(
            "missing `--bpmn` or `--dmn` for `template` command",
        )),
        (true, true) => Err(invalid_input(
            "`template` command requires exactly one of `--bpmn` or `--dmn`",
        )),
    }
}

fn bpmn_template() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<definitions xmlns="http://www.omg.org/spec/BPMN/20100524/MODEL"
             xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
             xmlns:qianji="https://qianji.dev/bpmn/extensions"
             id="Definitions_1"
             targetNamespace="https://qianji.dev">
  <process id="Process_1" name="Skill Workflow" isExecutable="true">
    <startEvent id="Start_1" name="Start"/>
    <serviceTask id="Task_1" name="Do focused work" implementation="${environment.services.runAgent}">
      <extensionElements>
        <qianji:config>
          <qianji:prompt>Perform one focused step from the skill and return the declared outputs as JSON.</qianji:prompt>
          <qianji:tools>bash</qianji:tools>
          <qianji:inputs></qianji:inputs>
          <qianji:outputs>result</qianji:outputs>
        </qianji:config>
      </extensionElements>
    </serviceTask>
    <endEvent id="End_1" name="End"/>
    <sequenceFlow id="Flow_1" sourceRef="Start_1" targetRef="Task_1"/>
    <sequenceFlow id="Flow_2" sourceRef="Task_1" targetRef="End_1"/>
  </process>
</definitions>"#
}

fn dmn_template() -> &'static str {
    r#"<definitions xmlns="https://www.omg.org/spec/DMN/20191111/MODEL/"
  id="Definitions_skill_decision"
  name="Skill Decision"
  namespace="https://qianji.dev/dmn">
  <decision id="skill-decision" name="Skill Decision">
    <decisionTable id="decision_table_1" hitPolicy="UNIQUE">
      <input id="input_1" label="input">
        <inputExpression id="input_expression_1" typeRef="string">
          <text>input</text>
        </inputExpression>
      </input>
      <output id="output_1" name="decision" label="decision" typeRef="string" />
      <rule id="rule_1">
        <inputEntry id="input_entry_1">
          <text>-</text>
        </inputEntry>
        <outputEntry id="output_entry_1">
          <text>"review"</text>
        </outputEntry>
      </rule>
    </decisionTable>
  </decision>
</definitions>"#
}
