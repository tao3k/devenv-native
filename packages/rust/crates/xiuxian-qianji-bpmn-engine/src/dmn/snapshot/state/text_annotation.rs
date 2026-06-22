use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{DmnSourceFile, DmnTextAnnotationSnapshot};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempTextAnnotationSnapshot {
    text_annotation_id: Option<String>,
    text: String,
}

impl From<TempTextAnnotationSnapshot> for DmnTextAnnotationSnapshot {
    fn from(value: TempTextAnnotationSnapshot) -> Self {
        let text = if value.text.is_empty() {
            None
        } else {
            Some(value.text)
        };
        Self {
            text_annotation_id: value.text_annotation_id,
            text,
        }
    }
}

impl TempTextAnnotationSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            text_annotation_id: attribute_value(source, reader, event, "id")?,
            text: String::new(),
        })
    }

    pub(super) fn append_text(&mut self, text: &str) {
        self.text.push_str(text);
    }
}
