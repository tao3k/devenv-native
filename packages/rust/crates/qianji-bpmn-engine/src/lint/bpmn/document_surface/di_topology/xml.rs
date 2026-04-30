use quick_xml::Reader;
use quick_xml::events::BytesStart;

pub(super) fn local_name(raw: &[u8]) -> Option<&str> {
    let name = std::str::from_utf8(raw).ok()?;
    Some(name.rsplit_once(':').map_or(name, |(_, local)| local))
}

pub(super) fn attribute_value(
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
    name: &str,
) -> Option<String> {
    for attribute in event.attributes().flatten() {
        if local_name(attribute.key.as_ref()) != Some(name) {
            continue;
        }
        if let Ok(value) = attribute.decode_and_unescape_value(reader.decoder()) {
            return Some(value.into_owned());
        }
    }
    None
}
