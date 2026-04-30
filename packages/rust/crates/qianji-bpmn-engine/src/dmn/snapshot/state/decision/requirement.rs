use super::{
    BytesStart, DmnRequirementReferenceSnapshot, DmnSourceFile, Reader, Result, attribute_value,
};

pub(in crate::dmn::snapshot::state) struct TempRequirementReferenceSnapshot {
    requirement_kind: String,
    reference_kind: String,
    href: Option<String>,
}

impl From<TempRequirementReferenceSnapshot> for DmnRequirementReferenceSnapshot {
    fn from(value: TempRequirementReferenceSnapshot) -> Self {
        Self {
            requirement_kind: value.requirement_kind,
            reference_kind: value.reference_kind,
            href: value.href,
        }
    }
}

impl TempRequirementReferenceSnapshot {
    pub(in crate::dmn::snapshot::state) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        requirement_kind: &str,
        reference_kind: &str,
    ) -> Result<Self> {
        Ok(Self {
            requirement_kind: requirement_kind.to_string(),
            reference_kind: reference_kind.to_string(),
            href: attribute_value(source, reader, event, "href")?,
        })
    }
}
