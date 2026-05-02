use crate::channels::managed_runtime::parsing::matching::eq_any_ignore_ascii;
use crate::channels::managed_runtime::parsing::normalize::{
    normalize_command_input, slice_original_command_suffix,
};
use crate::channels::managed_runtime::parsing::{JobStatusCommand, OutputFormat, ResumeCommand};

pub(crate) fn parse_help_command(input: &str) -> Option<OutputFormat> {
    let normalized = normalize_command_input(input);
    let mut parts = normalized.split_whitespace();
    let command = parts.next()?;
    let arg1 = parts.next();
    let arg2 = parts.next();
    if parts.next().is_some() {
        return None;
    }

    match (command, arg1, arg2) {
        ("help" | "commands", None, None) => Some(OutputFormat::Dashboard),
        ("help" | "commands", Some(fmt), None) if eq_any_ignore_ascii(fmt, &["json"]) => {
            Some(OutputFormat::Json)
        }
        ("slash", Some(sub), None) if eq_any_ignore_ascii(sub, &["help"]) => {
            Some(OutputFormat::Dashboard)
        }
        ("slash", Some(sub), Some(fmt))
            if eq_any_ignore_ascii(sub, &["help"]) && eq_any_ignore_ascii(fmt, &["json"]) =>
        {
            Some(OutputFormat::Json)
        }
        _ => None,
    }
}

pub(crate) fn parse_background_prompt(input: &str) -> Option<String> {
    let normalized = normalize_command_input(input);
    let lower = normalized.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("bg ") {
        return slice_original_command_suffix(normalized, rest).map(ToString::to_string);
    }
    if let Some(rest) = lower.strip_prefix("research ") {
        let original = slice_original_command_suffix(normalized, rest)?;
        return Some(format!("research {}", original.trim()));
    }
    None
}

pub(crate) fn parse_job_status_command(input: &str) -> Option<JobStatusCommand> {
    let normalized = normalize_command_input(input);
    let mut parts = normalized.split_whitespace();
    let cmd = parts.next()?;
    if !eq_any_ignore_ascii(cmd, &["job"]) {
        return None;
    }

    let id = parts.next()?.trim();
    if id.is_empty() {
        return None;
    }
    let format = match parts.next() {
        None => OutputFormat::Dashboard,
        Some(value) if eq_any_ignore_ascii(value, &["json"]) => OutputFormat::Json,
        Some(_) => return None,
    };
    if parts.next().is_some() {
        return None;
    }
    Some(JobStatusCommand {
        job_id: id.to_string(),
        format,
    })
}

pub(crate) fn parse_jobs_summary_command(input: &str) -> Option<OutputFormat> {
    let normalized = normalize_command_input(input);
    let mut parts = normalized.split_whitespace();
    let cmd = parts.next()?;
    if !eq_any_ignore_ascii(cmd, &["jobs"]) {
        return None;
    }

    let format = match parts.next() {
        None => OutputFormat::Dashboard,
        Some(value) if eq_any_ignore_ascii(value, &["json"]) => OutputFormat::Json,
        Some(_) => return None,
    };
    if parts.next().is_some() {
        return None;
    }
    Some(format)
}

pub(crate) fn is_reset_context_command(input: &str) -> bool {
    let normalized = normalize_command_input(input);
    eq_any_ignore_ascii(normalized, &["reset", "clear"])
}

pub(crate) fn is_stop_command(input: &str) -> bool {
    let normalized = normalize_command_input(input);
    eq_any_ignore_ascii(normalized, &["stop", "cancel", "interrupt"])
}

pub(crate) fn parse_resume_context_command(input: &str) -> Option<ResumeCommand> {
    let normalized = normalize_command_input(input);
    let mut parts = normalized.split_whitespace();
    let cmd = parts.next()?;
    if !eq_any_ignore_ascii(cmd, &["resume"]) {
        return None;
    }
    match (parts.next(), parts.next()) {
        (None, None) => Some(ResumeCommand::Restore),
        (Some(sub), None) if eq_any_ignore_ascii(sub, &["status", "stats", "info"]) => {
            Some(ResumeCommand::Status)
        }
        (Some(sub), None) if eq_any_ignore_ascii(sub, &["drop", "discard"]) => {
            Some(ResumeCommand::Drop)
        }
        _ => None,
    }
}
