use super::support::*;

#[test]
fn test_xml_escape() {
    assert_eq!(xml_escape("a<b>c&d"), "a&lt;b&gt;c&amp;d");
    assert_eq!(xml_escape("\"quoted\""), "&quot;quoted&quot;");
    assert_eq!(xml_escape("'single'"), "&apos;single&apos;");
    assert_eq!(xml_escape("normal text"), "normal text");
}
