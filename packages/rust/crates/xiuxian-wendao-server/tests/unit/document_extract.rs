use xiuxian_wendao_server::transport::{
    DocumentExtractMode, decode_document_extract_source_path_utf8_hex,
    encode_document_extract_source_path_utf8_hex,
};

#[test]
fn document_extract_mode_parses_auto() -> Result<(), String> {
    assert_eq!(
        DocumentExtractMode::parse("auto")?,
        DocumentExtractMode::Auto
    );
    assert_eq!(DocumentExtractMode::parse("")?, DocumentExtractMode::Auto);
    Ok(())
}

#[test]
fn document_extract_mode_parses_audio_shards() -> Result<(), String> {
    assert_eq!(
        DocumentExtractMode::parse("audio-shards")?,
        DocumentExtractMode::AudioShards
    );
    assert_eq!(
        DocumentExtractMode::parse("audio_shards")?,
        DocumentExtractMode::AudioShards
    );
    Ok(())
}

#[test]
fn document_extract_source_path_utf8_hex_roundtrips_non_ascii_paths() {
    let source_path = "private-fixtures/audio-\u{97f3}\u{9891}.mp3";

    let encoded = encode_document_extract_source_path_utf8_hex(source_path);

    assert!(encoded.is_ascii());
    assert_eq!(
        decode_document_extract_source_path_utf8_hex(encoded.as_str()),
        Ok(source_path.to_string())
    );
}
