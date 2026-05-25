use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{BpmnEngineError, BpmnSourceFile, snapshot_bpmn_source};

#[test]
fn bpmn_snapshot_reports_invalid_xml_as_bpmn_xml_error() {
    let source = BpmnSourceFile::new("broken.bpmn", "<definitions><process></definitions>");

    let error = snapshot_bpmn_source(&source).must_err("invalid XML should be rejected");

    let BpmnEngineError::InvalidXml {
        source_id, offset, ..
    } = error
    else {
        panic!("invalid XML should return InvalidXml");
    };
    assert_eq!(source_id, "broken.bpmn");
    assert!(
        offset.is_some(),
        "XML reader should report an error byte offset"
    );
}
