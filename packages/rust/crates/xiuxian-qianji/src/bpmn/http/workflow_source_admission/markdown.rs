#[derive(Debug, Clone)]
pub(super) struct WorkflowTask {
    pub id: String,
    pub name: String,
    pub documentation: String,
}

pub(super) fn markdown_workflow_tasks(
    workflow_name: &str,
    workflow_description: &str,
    markdown: &str,
) -> Result<Vec<WorkflowTask>, WorkflowSourceCompileError> {
    let mut tasks = Vec::new();
    let mut current: Option<WorkflowTaskBuilder> = None;
    for line in markdown.lines() {
        if let Some((step_number, title)) = parse_step_heading(line) {
            if let Some(builder) = current.take() {
                tasks.push(builder.build(workflow_name, workflow_description));
            }
            current = Some(WorkflowTaskBuilder {
                id: format!("step-{step_number}"),
                name: title.to_owned(),
                body_lines: Vec::new(),
            });
            continue;
        }
        if let Some(builder) = current.as_mut() {
            builder.body_lines.push(line.to_owned());
        }
    }
    if let Some(builder) = current {
        tasks.push(builder.build(workflow_name, workflow_description));
    }
    if tasks.is_empty() {
        return Err(WorkflowSourceCompileError::MarkdownStepsMissing);
    }
    Ok(tasks)
}

#[derive(Debug, Clone)]
struct WorkflowTaskBuilder {
    id: String,
    name: String,
    body_lines: Vec<String>,
}

impl WorkflowTaskBuilder {
    fn build(self, workflow_name: &str, workflow_description: &str) -> WorkflowTask {
        let body = self.body_lines.join("\n").trim().to_owned();
        WorkflowTask {
            id: self.id,
            name: self.name.clone(),
            documentation: workflow_documentation(
                workflow_name,
                workflow_description,
                Some(self.name.as_str()),
                (!body.is_empty()).then_some(body.as_str()),
            ),
        }
    }
}

fn workflow_documentation(
    workflow_name: &str,
    workflow_description: &str,
    task_name: Option<&str>,
    body: Option<&str>,
) -> String {
    let mut parts = vec![format!("Workflow: {workflow_name}")];
    if !workflow_description.is_empty() {
        parts.push(format!("Workflow goal: {workflow_description}"));
    }
    if let Some(task_name) = task_name {
        parts.push(format!("Task: {task_name}"));
    }
    if let Some(body) = body {
        parts.push(format!("Instructions:\n{body}"));
    }
    parts.join("\n\n")
}

fn parse_step_heading(line: &str) -> Option<(&str, &str)> {
    let heading = line.trim().strip_prefix("##")?.trim_start();
    let heading = heading.strip_prefix("Step ")?;
    let (step_number, title) = heading.split_once(':')?;
    let step_number = step_number.trim();
    let title = title.trim();
    if step_number.is_empty()
        || title.is_empty()
        || !step_number
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return None;
    }
    Some((step_number, title))
}
use super::compile::WorkflowSourceCompileError;
