pub(super) fn local_name(raw: &[u8]) -> Option<&str> {
    let name = std::str::from_utf8(raw).ok()?;
    Some(name.rsplit_once(':').map_or(name, |(_, local)| local))
}
