//! Transmuter structural validation and normalization coverage.

use xiuxian_zhenfa::{
    ZhenfaResolveAndWashError, ZhenfaTransmuter, ZhenfaTransmuterError, ZhenfaXmlLiteTagName,
};

#[test]
fn validate_structure_accepts_balanced_xml_lite() {
    let result = ZhenfaTransmuter::validate_structure(
        "<report><decision>run</decision><score>0.8</score></report>",
    );
    assert!(result.is_ok());
}

#[test]
fn validate_structure_rejects_mismatched_tag() {
    let result = ZhenfaTransmuter::validate_structure("<report><score>0.8</report>");
    assert_eq!(
        result,
        Err(ZhenfaTransmuterError::MismatchedClosingTag {
            expected: "score".to_string(),
            found: "report".to_string(),
        })
    );
}

#[test]
fn refine_for_caller_normalizes_payload() {
    let refined = ZhenfaTransmuter::refine_for_caller("line 1 \r\n\r\n\r\nline 2\0");
    assert_eq!(refined, "line 1\n\n\nline 2");
}

#[test]
fn check_semantic_integrity_requires_reference_type_suffix() {
    assert!(!ZhenfaTransmuter::check_semantic_integrity(
        "- [[references/teacher.md]]"
    ));
    assert!(ZhenfaTransmuter::check_semantic_integrity(
        "- [[references/teacher.md#persona]]"
    ));
}

#[test]
fn validate_and_refine_rejects_unclosed_tag() {
    let result = ZhenfaTransmuter::validate_and_refine("<score>0.8");
    assert_eq!(
        result,
        Err(ZhenfaTransmuterError::UnclosedTag {
            tag: ZhenfaXmlLiteTagName::from("score"),
        })
    );
}

#[test]
fn resolve_and_wash_returns_refined_payload() {
    let resolved =
        match ZhenfaTransmuter::resolve_and_wash("wendao://skills/demo/references/a.md", |_| {
            Some("line 1 \r\n\r\n\r\nline 2".to_string())
        }) {
            Ok(resolved) => resolved,
            Err(error) => panic!("semantic URI should resolve and be refined: {error}"),
        };
    assert_eq!(resolved, "line 1\n\n\nline 2");
}

#[test]
fn resolve_and_wash_returns_resource_not_found_for_missing_uri() {
    let result =
        ZhenfaTransmuter::resolve_and_wash("wendao://skills/demo/references/missing.md", |_| None);
    assert_eq!(
        result,
        Err(ZhenfaResolveAndWashError::ResourceNotFound {
            uri: "wendao://skills/demo/references/missing.md".to_string(),
        })
    );
}

#[test]
fn resolve_and_wash_validates_xml_for_xml_assets() {
    let result =
        ZhenfaTransmuter::resolve_and_wash("wendao://skills/demo/references/report.xml", |_| {
            Some("<report><score>0.8".to_string())
        });
    assert_eq!(
        result,
        Err(ZhenfaResolveAndWashError::Transmuter(
            ZhenfaTransmuterError::UnclosedTag {
                tag: ZhenfaXmlLiteTagName::from("score"),
            }
        ))
    );
}
