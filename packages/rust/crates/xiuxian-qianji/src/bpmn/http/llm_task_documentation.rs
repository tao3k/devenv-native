//! BPMN task documentation extraction for server-owned LLM prompts.

use std::fs;

pub(super) fn bpmn_task_documentation(
    bpmn_source_ref: Option<&str>,
    activity_id: Option<&str>,
) -> Option<String> {
    let bpmn_source_ref = bpmn_source_ref?;
    let activity_id = activity_id?;
    let path = bpmn_source_ref
        .strip_prefix("file://")
        .unwrap_or(bpmn_source_ref);
    let xml = fs::read_to_string(path).ok()?;
    extract_activity_documentation(&xml, activity_id)
}

pub(super) fn extract_activity_documentation(xml: &str, activity_id: &str) -> Option<String> {
    for pattern in [
        format!("id=\"{activity_id}\""),
        format!("id='{activity_id}'"),
    ] {
        let mut search_from = 0;
        while let Some(relative_pos) = xml[search_from..].find(&pattern) {
            let id_pos = search_from + relative_pos;
            let Some(tag_start) = xml[..id_pos].rfind('<') else {
                search_from = id_pos + pattern.len();
                continue;
            };
            let Some(tag_end_relative) = xml[id_pos..].find('>') else {
                return None;
            };
            let tag_end = id_pos + tag_end_relative;
            let start_tag = &xml[tag_start + 1..tag_end];
            if start_tag.starts_with('/') || start_tag.ends_with('/') {
                search_from = id_pos + pattern.len();
                continue;
            }
            let Some(tag_name) = start_tag.split_whitespace().next() else {
                search_from = id_pos + pattern.len();
                continue;
            };
            let close_tag = format!("</{tag_name}>");
            let body_start = tag_end + 1;
            let Some(body_end_relative) = xml[body_start..].find(&close_tag) else {
                search_from = id_pos + pattern.len();
                continue;
            };
            let body_end = body_start + body_end_relative;
            if let Some(documentation) =
                extract_first_documentation_text(&xml[body_start..body_end])
            {
                return Some(documentation);
            }
            search_from = id_pos + pattern.len();
        }
    }
    None
}

fn extract_first_documentation_text(xml: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(open_relative) = xml[search_from..].find('<') {
        let open = search_from + open_relative;
        let Some(tag_end_relative) = xml[open..].find('>') else {
            return None;
        };
        let tag_end = open + tag_end_relative;
        let start_tag = xml[open + 1..tag_end].trim();
        if start_tag.starts_with('/') {
            search_from = tag_end + 1;
            continue;
        }
        let Some(tag_name) = start_tag.split_whitespace().next() else {
            search_from = tag_end + 1;
            continue;
        };
        if tag_name
            .rsplit(':')
            .next()
            .is_some_and(|local_name| local_name == "documentation")
        {
            let close_tag = format!("</{tag_name}>");
            let text_start = tag_end + 1;
            let text_end = xml[text_start..].find(&close_tag)? + text_start;
            let text = xml_text_unescape(&strip_xml_tags(&xml[text_start..text_end]));
            let text = text.trim();
            return (!text.is_empty()).then(|| text.to_owned());
        }
        search_from = tag_end + 1;
    }
    None
}

fn strip_xml_tags(xml: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in xml.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

fn xml_text_unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
