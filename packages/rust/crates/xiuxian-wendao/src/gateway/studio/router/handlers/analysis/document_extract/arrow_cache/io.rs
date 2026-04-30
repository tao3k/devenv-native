use std::fs::File;
use std::path::Path;

use arrow::ipc::reader::FileReader;
use arrow::ipc::writer::FileWriter;
use arrow::record_batch::RecordBatch;

pub(in super::super) fn read_arrow_file(path: &Path) -> Result<Vec<RecordBatch>, String> {
    let file = File::open(path)
        .map_err(|error| format!("open Arrow IPC file `{}`: {error}", path.display()))?;
    let reader = FileReader::try_new(file, None)
        .map_err(|error| format!("decode Arrow IPC file `{}`: {error}", path.display()))?;
    let mut batches = Vec::new();
    for batch in reader {
        batches.push(
            batch.map_err(|error| format!("read Arrow IPC batch `{}`: {error}", path.display()))?,
        );
    }
    Ok(batches)
}

pub(in super::super) fn write_arrow_file(
    path: &Path,
    batches: &[RecordBatch],
) -> Result<(), String> {
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
