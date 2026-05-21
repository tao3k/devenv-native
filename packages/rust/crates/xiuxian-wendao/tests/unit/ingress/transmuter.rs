use super::{IngressTransmuterError, resolve_and_wash};

#[test]
fn xml_lite_validation_accepts_comments_and_processing_instructions() {
    let content = r#"<?xml version="1.0"?><root><!-- skip --><item /></root>"#;

    let washed = match resolve_and_wash("fixture.xml", content) {
        Ok(washed) => washed,
        Err(error) => panic!("valid XML-lite content failed: {error}"),
    };

    assert_eq!(washed, content);
}

#[test]
fn xml_lite_validation_reports_mismatched_tags() {
    let error = match resolve_and_wash("fixture.xml", "<root><item></root>") {
        Ok(washed) => panic!("mismatched XML-lite content passed: {washed}"),
        Err(error) => error,
    };

    assert_eq!(
        error.to_string(),
        IngressTransmuterError::MismatchedClosingTag {
            expected: "item".to_string(),
            found: "root".to_string(),
        }
        .to_string()
    );
}

#[test]
fn xml_lite_validation_reports_unclosed_comments() {
    let error = match resolve_and_wash("fixture.xml", "<root><!-- open</root>") {
        Ok(washed) => panic!("unclosed XML-lite comment passed: {washed}"),
        Err(error) => error,
    };

    assert_eq!(
        error.to_string(),
        IngressTransmuterError::UnclosedTag {
            tag: "!--".to_string(),
        }
        .to_string()
    );
}
