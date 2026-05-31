#[path = "../../src/legacy_office/panic_guard.rs"]
mod panic_guard;

#[path = "../../src/legacy_office/xls.rs"]
mod xls;

use litchi::ole::xls::records::{FormulaValue, Record, RecordHeader, XlsEncoding};
use xls::{SheetRows, formula_text, parse_sst_records, render_sheets};

#[test]
fn render_sheets_preserves_sparse_cell_order() {
    let mut rows = SheetRows::new();
    rows.entry(2).or_default().insert(1, "name".to_string());
    rows.entry(2).or_default().insert(3, "value".to_string());
    assert_eq!(render_sheets(&[rows]), "\tname\t\tvalue\n");
}

#[test]
fn formula_text_prefers_cached_value() {
    assert_eq!(formula_text(&FormulaValue::Number(12.0), &[1, 2, 3]), "12");
    assert_eq!(
        formula_text(&FormulaValue::Empty, &[1, 2, 3]),
        "Formula(3 bytes)"
    );
}

#[test]
fn parse_sst_records_handles_unicode_continuation_flags() {
    let mut first = Vec::new();
    first.extend_from_slice(&1_u32.to_le_bytes());
    first.extend_from_slice(&1_u32.to_le_bytes());
    first.extend_from_slice(&3_u16.to_le_bytes());
    first.push(1);
    first.extend_from_slice(&utf16('你'));
    let mut continuation = vec![1];
    continuation.extend_from_slice(&utf16('好'));
    continuation.extend_from_slice(&utf16('啊'));
    let records = vec![
        Record {
            header: RecordHeader {
                record_type: 0x00FC,
                data_len: first.len() as u16,
            },
            data: first,
        },
        Record {
            header: RecordHeader {
                record_type: 0x003C,
                data_len: continuation.len() as u16,
            },
            data: continuation,
        },
    ];
    let encoding = match XlsEncoding::from_codepage(1252) {
        Ok(encoding) => encoding,
        Err(error) => panic!("test encoding failed: {error}"),
    };
    let strings = match parse_sst_records(&records, &encoding) {
        Ok(strings) => strings,
        Err(error) => panic!("parse SST failed: {error}"),
    };
    assert_eq!(strings, ["你好啊"]);
}

fn utf16(ch: char) -> [u8; 2] {
    let value = u16::try_from(ch as u32).unwrap_or(0);
    value.to_le_bytes()
}
