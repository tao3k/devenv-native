pub(super) fn flow_id(index: usize) -> String {
    format!("Flow_{}", index + 1)
}

pub(super) fn gateway_id(source: &str) -> String {
    stable_xml_id("Gateway", source)
}

pub(super) fn node_ref(node: &str) -> String {
    match node {
        "start" => "Start_1".to_string(),
        "end" => "End_1".to_string(),
        other => other.to_string(),
    }
}

pub(super) fn stable_xml_id(prefix: &str, value: &str) -> String {
    let mut output = String::with_capacity(prefix.len() + value.len() + 1);
    output.push_str(prefix);
    output.push('_');
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            output.push(ch);
        } else {
            output.push('_');
        }
    }
    if output.ends_with('_') {
        output.push('1');
    }
    output
}
