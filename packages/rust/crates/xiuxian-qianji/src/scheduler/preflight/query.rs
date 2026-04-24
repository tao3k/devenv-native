use super::wendao_uri::resolve_wendao_uri_with_zhenfa;
use xiuxian_wendao::embedded_discover_canonical_uris;

/// Attempts to resolve one semantic query into URI hits and returns aggregated
/// XML-Lite payload when any hit is found.
///
/// Returns `Ok(None)` when no URI can be discovered from the query.
pub(crate) fn resolve_dynamic_query_with_uri_expansion(
    query_expression: &str,
) -> Result<Option<String>, String> {
    let mut uris = embedded_discover_canonical_uris(query_expression)
        .map_err(|error| format!("semantic discovery failed for `{query_expression}`: {error}"))?;
    if uris.is_empty() {
        return Ok(None);
    }
    uris.sort();
    uris.dedup();

    let mut resources = Vec::with_capacity(uris.len());
    for uri in uris {
        let content = resolve_wendao_uri_with_zhenfa(uri.as_str())?;
        resources.push((uri, content));
    }

    let mut xml = String::new();
    xml.push_str("<wendao_query_result>");
    xml.push_str("<query>");
    xml.push_str(escape_xml_lite(query_expression).as_str());
    xml.push_str("</query>");
    xml.push_str("<hit_count>");
    xml.push_str(resources.len().to_string().as_str());
    xml.push_str("</hit_count>");
    xml.push_str("<resources>");
    for (uri, content) in resources {
        xml.push_str("<resource>");
        xml.push_str("<uri>");
        xml.push_str(escape_xml_lite(uri.as_str()).as_str());
        xml.push_str("</uri>");
        xml.push_str("<content>");
        xml.push_str(escape_xml_lite(content.as_str()).as_str());
        xml.push_str("</content>");
        xml.push_str("</resource>");
    }
    xml.push_str("</resources>");
    xml.push_str("</wendao_query_result>");

    Ok(Some(xml))
}

fn escape_xml_lite(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
