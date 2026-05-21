//! Org agenda-backed agent planning projection.

use std::path::PathBuf;

use orgize::Org;
use orgize::ast::{AgendaDate, AgendaQuery, AgentPlanningQuery};

use super::OrgizeToolError;
use super::io::{collect_org_paths, join_projection_text, read_to_string};

/// Options for agent planning snapshots derived from Org agenda syntax.
///
/// Raw DTO boundary: these flags mirror CLI/query toggles and are not stored
/// as a long-lived domain model.
#[derive(Clone, Debug)]
pub struct OrgizeAgentPlanningRequest {
    /// Files or directories to inspect.
    pub paths: Vec<PathBuf>,
    /// Inclusive start date in `YYYY-MM-DD` form.
    pub start_date: String,
    /// Optional inclusive end date in `YYYY-MM-DD` form.
    pub end_date: Option<String>,
    /// Include DONE-state tasks.
    pub include_done: bool,
    /// Include archived tasks.
    pub include_archived: bool,
    /// Include COMMENT tasks.
    pub include_comments: bool,
    /// Optional Org agenda match expression.
    pub match_expression: Option<String>,
}

/// Renders agent planning cards from Org agenda semantics.
///
/// # Errors
///
/// Returns an error when a path cannot be read, a date is invalid, or a match
/// expression cannot be parsed.
pub fn render_agent_planning(
    request: &OrgizeAgentPlanningRequest,
) -> Result<String, OrgizeToolError> {
    let files = collect_org_paths(&request.paths)?;
    let start = parse_agenda_date(&request.start_date)?;
    let end = request
        .end_date
        .as_deref()
        .map(parse_agenda_date)
        .transpose()?
        .unwrap_or(start);
    let mut agenda = AgendaQuery::new(start, end)
        .include_done(request.include_done)
        .include_archived(request.include_archived)
        .include_comments(request.include_comments);
    if let Some(expression) = request.match_expression.as_deref() {
        agenda = agenda.match_expression(expression).map_err(|error| {
            OrgizeToolError::InvalidMatchExpression {
                expression: expression.to_string(),
                message: error.to_string(),
            }
        })?;
    }
    let base_query = AgentPlanningQuery::new(agenda);

    let mut rendered = Vec::new();
    for path in files {
        let source = read_to_string(&path)?;
        let document = Org::parse(&source).document();
        let snapshot = document.agent_planning_snapshot(&AgentPlanningQuery::new(
            base_query
                .agenda
                .clone()
                .source_file(path.display().to_string()),
        ));
        rendered.push(snapshot.to_compact_text(&path.display().to_string()));
    }
    Ok(join_projection_text(
        rendered,
        "[ok] orgize agent planning\n",
    ))
}

fn parse_agenda_date(value: &str) -> Result<AgendaDate, OrgizeToolError> {
    let parts = value.split('-').collect::<Vec<_>>();
    let invalid = || OrgizeToolError::InvalidDate {
        value: value.to_string(),
    };
    let [year, month, day] = parts.as_slice() else {
        return Err(invalid());
    };
    let year = year.parse::<u16>().map_err(|_| invalid())?;
    let month = month.parse::<u8>().map_err(|_| invalid())?;
    let day = day.parse::<u8>().map_err(|_| invalid())?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(invalid());
    }
    Ok(AgendaDate::new(year, month, day))
}
