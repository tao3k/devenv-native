use std::fs::File;
use std::path::Path;

use arrow::ipc::reader::FileReader;
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;
#[cfg(feature = "document-extract-pdf-source-range")]
use std::io::Cursor;

pub(crate) fn read_arrow_file(path: &Path) -> Result<Vec<RecordBatch>, String> {
    let file = File::open(path)
        .map_err(|error| format!("open Arrow IPC file `{}`: {error}", path.display()))?;
    read_arrow_reader(file, &format!("Arrow IPC file `{}`", path.display()))
}

#[cfg(feature = "document-extract-pdf-source-range")]
pub(crate) fn read_arrow_bytes(bytes: &[u8]) -> Result<Vec<RecordBatch>, String> {
    read_arrow_reader(Cursor::new(bytes), "Arrow IPC bytes")
}

fn read_arrow_reader<R: std::io::Read + std::io::Seek>(
    reader: R,
    label: &str,
) -> Result<Vec<RecordBatch>, String> {
    let reader =
        FileReader::try_new(reader, None).map_err(|error| format!("decode {label}: {error}"))?;
    let mut batches = Vec::new();
    for batch in reader {
        batches.push(batch.map_err(|error| format!("read Arrow IPC batch from {label}: {error}"))?);
    }
    Ok(batches)
}

pub(crate) fn write_arrow_file(path: &Path, batches: &[RecordBatch]) -> Result<(), String> {
    let Some(first) = batches.first() else {
        return Err(format!(
            "cannot write empty Arrow IPC file `{}`",
            path.display()
        ));
    };
    let file = File::create(path)
        .map_err(|error| format!("create Arrow IPC file `{}`: {error}", path.display()))?;
    let mut writer = FileWriter::try_new(file, first.schema().as_ref())
        .map_err(|error| format!("create Arrow IPC writer `{}`: {error}", path.display()))?;
    for batch in batches {
        writer
            .write(batch)
            .map_err(|error| format!("write Arrow IPC batch `{}`: {error}", path.display()))?;
    }
    writer
        .finish()
        .map_err(|error| format!("finish Arrow IPC file `{}`: {error}", path.display()))
}

#[cfg(feature = "document-extract-pdf-source-range")]
pub(crate) fn write_arrow_bytes(batches: &[RecordBatch]) -> Result<Vec<u8>, String> {
    let Some(first) = batches.first() else {
        return Err("cannot write empty Arrow IPC bytes".to_string());
    };
    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = FileWriter::try_new(&mut cursor, first.schema().as_ref())
            .map_err(|error| format!("create Arrow IPC byte writer: {error}"))?;
        for batch in batches {
            writer
                .write(batch)
                .map_err(|error| format!("write Arrow IPC byte batch: {error}"))?;
        }
        writer
            .finish()
            .map_err(|error| format!("finish Arrow IPC bytes: {error}"))?;
    }
    Ok(cursor.into_inner())
}
