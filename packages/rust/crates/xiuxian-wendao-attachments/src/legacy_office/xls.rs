use std::collections::BTreeMap;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;

use litchi::ole::OleFile;
use litchi::ole::xls::records::{
    BoolErrValue, CellRecord, FormulaValue, Record, RecordIter, XlsEncoding,
};

use super::panic_guard;

type SheetRows = BTreeMap<u16, BTreeMap<u16, String>>;

pub(crate) fn extract_text(path: &Path) -> Result<String, String> {
    panic_guard::run("Excel 97-2003 workbook", path, || extract_text_inner(path))
}

fn extract_text_inner(path: &Path) -> Result<String, String> {
    let file = File::open(path)
        .map_err(|error| format!("open legacy Excel workbook `{}`: {error}", path.display()))?;
    let mut ole = OleFile::open(file).map_err(|error| {
        format!(
            "open legacy Excel OLE container `{}`: {error}",
            path.display()
        )
    })?;
    let workbook_data = ole
        .open_stream(&["Workbook"])
        .or_else(|_| ole.open_stream(&["Book"]))
        .map_err(|error| format!("read Excel Workbook stream `{}`: {error}", path.display()))?;
    extract_text_from_workbook_stream(workbook_data.as_slice())
        .map_err(|error| format!("extract legacy Excel text `{}`: {error}", path.display()))
}

fn extract_text_from_workbook_stream(workbook_data: &[u8]) -> Result<String, String> {
    let records = read_records(workbook_data)?;
    let encoding = workbook_encoding(records.as_slice())?;
    let shared_strings = shared_strings(records.as_slice(), &encoding)?;
    let sheets = collect_sheet_rows(records.as_slice(), &encoding, shared_strings.as_slice())?;
    Ok(render_sheets(sheets.as_slice()))
}

fn read_records(workbook_data: &[u8]) -> Result<Vec<Record>, String> {
    let iter = RecordIter::new(Cursor::new(workbook_data))
        .map_err(|error| format!("open BIFF record stream: {error}"))?;
    let mut records = Vec::new();
    for record in iter {
        records.push(record.map_err(|error| format!("read BIFF record: {error}"))?);
    }
    Ok(records)
}

fn workbook_encoding(records: &[Record]) -> Result<XlsEncoding, String> {
    let mut encoding = XlsEncoding::from_codepage(1252)
        .map_err(|error| format!("initialize default XLS codepage: {error}"))?;
    for record in records {
        if record.header.record_type == 0x0042 && record.data.len() >= 2 {
            let codepage = u16::from_le_bytes([record.data[0], record.data[1]]);
            encoding = XlsEncoding::from_codepage(codepage)
                .map_err(|error| format!("parse XLS codepage {codepage}: {error}"))?;
        }
    }
    Ok(encoding)
}

fn shared_strings(records: &[Record], encoding: &XlsEncoding) -> Result<Vec<String>, String> {
    let Some(index) = records
        .iter()
        .position(|record| record.header.record_type == 0x00FC)
    else {
        return Ok(Vec::new());
    };
    let end = records[index + 1..]
        .iter()
        .position(|record| record.header.record_type != 0x003C)
        .map_or(records.len(), |offset| index + 1 + offset);
    parse_sst_records(&records[index..end], encoding)
        .map_err(|error| format!("parse XLS shared string table: {error}"))
}

fn parse_sst_records(records: &[Record], encoding: &XlsEncoding) -> Result<Vec<String>, String> {
    let first = records
        .first()
        .ok_or_else(|| "missing SST record".to_string())?;
    if first.header.record_type != 0x00FC {
        return Err("first shared string record is not SST".to_string());
    }
    if first.data.len() < 8 {
        return Err(format!(
            "SST record is too short: expected at least 8 bytes, found {}",
            first.data.len()
        ));
    }
    let unique_count = read_u32_le(first.data.as_slice(), 4)? as usize;
    let mut segments = Vec::with_capacity(records.len());
    segments.push(&first.data[8..]);
    for record in &records[1..] {
        if record.header.record_type == 0x003C {
            segments.push(record.data.as_slice());
        }
    }
    let mut cursor = SegmentCursor::new(segments);
    let mut strings = Vec::with_capacity(unique_count.min(10000));
    for _ in 0..unique_count {
        if !cursor.has_remaining() {
            break;
        }
        strings.push(parse_sst_string(&mut cursor, encoding)?);
    }
    Ok(strings)
}

fn parse_sst_string(
    cursor: &mut SegmentCursor<'_>,
    encoding: &XlsEncoding,
) -> Result<String, String> {
    let char_count = usize::from(cursor.read_u16()?);
    let flags = cursor.read_u8()?;
    let rich_run_count = if flags & 0x08 != 0 {
        usize::from(cursor.read_u16()?)
    } else {
        0
    };
    let phonetic_size = if flags & 0x04 != 0 {
        cursor.read_u32()? as usize
    } else {
        0
    };
    let mut text = String::new();
    let mut remaining_chars = char_count;
    let mut is_unicode = flags & 0x01 != 0;
    while remaining_chars > 0 {
        if cursor.remaining_in_segment() == 0 {
            cursor.advance_segment()?;
            is_unicode = cursor.read_u8()? & 0x01 != 0;
            continue;
        }
        if is_unicode {
            let available_chars = remaining_chars.min(cursor.remaining_in_segment() / 2);
            if available_chars == 0 {
                cursor.advance_segment()?;
                is_unicode = cursor.read_u8()? & 0x01 != 0;
                continue;
            }
            let bytes = cursor.read_current_segment_bytes(available_chars * 2)?;
            let units = bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect::<Vec<_>>();
            text.push_str(String::from_utf16_lossy(units.as_slice()).as_str());
            remaining_chars -= available_chars;
        } else {
            let available_chars = remaining_chars.min(cursor.remaining_in_segment());
            let bytes = cursor.read_current_segment_bytes(available_chars)?;
            text.push_str(
                encoding
                    .decode(bytes)
                    .map_err(|error| format!("decode SST compressed string: {error}"))?
                    .as_str(),
            );
            remaining_chars -= available_chars;
        }
    }
    cursor.skip(rich_run_count.saturating_mul(4))?;
    cursor.skip(phonetic_size)?;
    Ok(text)
}

fn collect_sheet_rows(
    records: &[Record],
    encoding: &XlsEncoding,
    shared_strings: &[String],
) -> Result<Vec<SheetRows>, String> {
    let mut sheets = Vec::<SheetRows>::new();
    let mut current_sheet: Option<usize> = None;
    for record in records {
        match record.header.record_type {
            0x0809 if is_worksheet_bof(record.data.as_slice()) => {
                sheets.push(BTreeMap::new());
                current_sheet = Some(sheets.len() - 1);
            }
            0x000A => {
                current_sheet = None;
            }
            0x0203 | 0x0204 | 0x0205 | 0x027E | 0x00FD | 0x0006 => {
                let cell = CellRecord::parse(record.header.record_type, &record.data, encoding)
                    .map_err(|error| {
                        format!(
                            "parse XLS cell record 0x{:04X}: {error}",
                            record.header.record_type
                        )
                    })?;
                let Some(text) = cell_text(&cell, shared_strings)? else {
                    continue;
                };
                let sheet_index = current_sheet.unwrap_or_else(|| {
                    if sheets.is_empty() {
                        sheets.push(BTreeMap::new());
                    }
                    sheets.len() - 1
                });
                sheets[sheet_index]
                    .entry(cell.row())
                    .or_default()
                    .insert(cell.col(), text);
            }
            _ => {}
        }
    }
    Ok(sheets)
}

fn is_worksheet_bof(data: &[u8]) -> bool {
    data.get(2..4)
        .is_some_and(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]) == 0x0010)
}

fn cell_text(cell: &CellRecord, shared_strings: &[String]) -> Result<Option<String>, String> {
    let text = match cell {
        CellRecord::Blank { .. } => return Ok(None),
        CellRecord::Number { value, .. } | CellRecord::Rk { value, .. } => format_number(*value),
        CellRecord::Label { value, .. } => value.clone(),
        CellRecord::BoolErr { value, .. } => match value {
            BoolErrValue::Bool(value) => {
                if *value {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            BoolErrValue::Error(code) => format!("Error {code}"),
        },
        CellRecord::LabelSst { sst_index, .. } => shared_strings
            .get(*sst_index as usize)
            .cloned()
            .ok_or_else(|| format!("XLS shared string index {sst_index} is out of bounds"))?,
        CellRecord::Formula { value, formula, .. } => formula_text(value, formula.as_slice()),
    };
    let text = text.trim().to_string();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

fn formula_text(value: &FormulaValue, formula: &[u8]) -> String {
    match value {
        FormulaValue::Number(value) => format_number(*value),
        FormulaValue::String(value) => value.clone(),
        FormulaValue::Bool(value) => {
            if *value {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        FormulaValue::Error(code) => format!("Error {code}"),
        FormulaValue::Empty => format!("Formula({} bytes)", formula.len()),
    }
}

fn format_number(value: f64) -> String {
    if value.is_finite() && value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn render_sheets(sheets: &[SheetRows]) -> String {
    let mut output = String::new();
    for (sheet_index, rows) in sheets.iter().filter(|rows| !rows.is_empty()).enumerate() {
        if !output.is_empty() {
            output.push('\n');
        }
        if sheets.len() > 1 {
            output.push_str(&format!("Sheet {}\n", sheet_index + 1));
        }
        for cells in rows.values() {
            let Some(max_col) = cells.keys().next_back().copied() else {
                continue;
            };
            let mut row = Vec::with_capacity(usize::from(max_col) + 1);
            for column in 0..=max_col {
                row.push(cells.get(&column).map_or("", String::as_str));
            }
            output.push_str(row.join("\t").trim_end());
            output.push('\n');
        }
    }
    output
}

struct SegmentCursor<'a> {
    segments: Vec<&'a [u8]>,
    segment_index: usize,
    offset: usize,
}

impl<'a> SegmentCursor<'a> {
    fn new(segments: Vec<&'a [u8]>) -> Self {
        Self {
            segments,
            segment_index: 0,
            offset: 0,
        }
    }

    fn has_remaining(&self) -> bool {
        if self.segment_index >= self.segments.len() {
            return false;
        }
        self.segments[self.segment_index]
            .get(self.offset..)
            .is_some_and(|remaining| !remaining.is_empty())
            || self.segments[self.segment_index + 1..]
                .iter()
                .any(|segment| !segment.is_empty())
    }

    fn remaining_in_segment(&self) -> usize {
        self.segments
            .get(self.segment_index)
            .map_or(0, |segment| segment.len().saturating_sub(self.offset))
    }

    fn advance_segment(&mut self) -> Result<(), String> {
        self.segment_index = self.segment_index.saturating_add(1);
        self.offset = 0;
        while self
            .segments
            .get(self.segment_index)
            .is_some_and(|segment| segment.is_empty())
        {
            self.segment_index = self.segment_index.saturating_add(1);
        }
        if self.segment_index >= self.segments.len() {
            return Err("SST continuation ended unexpectedly".to_string());
        }
        Ok(())
    }

    fn read_u8(&mut self) -> Result<u8, String> {
        if self.remaining_in_segment() == 0 {
            self.advance_segment()?;
        }
        let byte = self.segments[self.segment_index][self.offset];
        self.offset += 1;
        Ok(byte)
    }

    fn read_u16(&mut self) -> Result<u16, String> {
        let bytes = [self.read_u8()?, self.read_u8()?];
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32(&mut self) -> Result<u32, String> {
        let bytes = [
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
            self.read_u8()?,
        ];
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_current_segment_bytes(&mut self, len: usize) -> Result<&'a [u8], String> {
        if len > self.remaining_in_segment() {
            return Err("SST parser attempted to read past current segment".to_string());
        }
        let start = self.offset;
        self.offset += len;
        Ok(&self.segments[self.segment_index][start..start + len])
    }

    fn skip(&mut self, mut len: usize) -> Result<(), String> {
        while len > 0 {
            if self.remaining_in_segment() == 0 {
                self.advance_segment()?;
            }
            let step = len.min(self.remaining_in_segment());
            self.offset += step;
            len -= step;
        }
        Ok(())
    }
}

fn read_u32_le(data: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = data
        .get(offset..offset + 4)
        .ok_or_else(|| format!("expected 4 bytes at offset {offset}"))?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        use litchi::ole::xls::records::RecordHeader;

        fn utf16(ch: char) -> [u8; 2] {
            let value = u16::try_from(ch as u32).unwrap_or(0);
            value.to_le_bytes()
        }

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
}
