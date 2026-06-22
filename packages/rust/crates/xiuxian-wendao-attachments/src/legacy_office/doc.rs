use std::fs::File;
use std::path::Path;

use litchi::ole::OleFile;
use litchi::ole::doc::parts::fib::FileInformationBlock;
use litchi::ole::doc::parts::text::TextExtractor;

use super::panic_guard;

pub(crate) fn extract_text(path: &Path) -> Result<String, String> {
    panic_guard::run("Word 97-2003 document", path, || extract_text_inner(path))
}

fn extract_text_inner(path: &Path) -> Result<String, String> {
    let file = File::open(path)
        .map_err(|error| format!("open legacy Word document `{}`: {error}", path.display()))?;
    let mut ole = OleFile::open(file).map_err(|error| {
        format!(
            "open legacy Word OLE container `{}`: {error}",
            path.display()
        )
    })?;
    let word_document = ole
        .open_stream(&["WordDocument"])
        .map_err(|error| format!("read WordDocument stream `{}`: {error}", path.display()))?;
    let fib = FileInformationBlock::parse(&word_document)
        .map_err(|error| format!("parse Word FIB `{}`: {error}", path.display()))?;
    let table_stream_name = if fib.which_table_stream() {
        "1Table"
    } else {
        "0Table"
    };
    let table_stream = ole.open_stream(&[table_stream_name]).map_err(|error| {
        format!(
            "read Word table stream `{table_stream_name}` from `{}`: {error}",
            path.display()
        )
    })?;
    TextExtractor::new(&fib, &word_document, &table_stream)
        .and_then(|extractor| extractor.extract_all_text())
        .map_err(|error| format!("extract legacy Word text `{}`: {error}", path.display()))
}
