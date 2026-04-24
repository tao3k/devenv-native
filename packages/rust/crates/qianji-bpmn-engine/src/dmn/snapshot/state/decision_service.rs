use crate::dmn::snapshot::xml::attribute_value;
use crate::dmn_model_api::{
    DmnDecisionServiceReferenceSnapshot, DmnDecisionServiceSnapshot, DmnSourceFile,
};
use crate::error::Result;
use quick_xml::Reader;
use quick_xml::events::BytesStart;

#[derive(Debug)]
pub(super) struct TempDecisionServiceSnapshot {
    decision_service_id: Option<String>,
    name: Option<String>,
    output_decisions: Vec<TempDecisionServiceReferenceSnapshot>,
    encapsulated_decisions: Vec<TempDecisionServiceReferenceSnapshot>,
    input_decisions: Vec<TempDecisionServiceReferenceSnapshot>,
    input_data: Vec<TempDecisionServiceReferenceSnapshot>,
}

impl From<TempDecisionServiceSnapshot> for DmnDecisionServiceSnapshot {
    fn from(value: TempDecisionServiceSnapshot) -> Self {
        Self {
            decision_service_id: value.decision_service_id,
            name: value.name,
            output_decisions: value.output_decisions.into_iter().map(Into::into).collect(),
            encapsulated_decisions: value
                .encapsulated_decisions
                .into_iter()
                .map(Into::into)
                .collect(),
            input_decisions: value.input_decisions.into_iter().map(Into::into).collect(),
            input_data: value.input_data.into_iter().map(Into::into).collect(),
        }
    }
}

impl TempDecisionServiceSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
    ) -> Result<Self> {
        Ok(Self {
            decision_service_id: attribute_value(source, reader, event, "id")?,
            name: attribute_value(source, reader, event, "name")?,
            output_decisions: Vec::new(),
            encapsulated_decisions: Vec::new(),
            input_decisions: Vec::new(),
            input_data: Vec::new(),
        })
    }

    pub(super) fn push_output_decision(&mut self, reference: TempDecisionServiceReferenceSnapshot) {
        self.output_decisions.push(reference);
    }

    pub(super) fn push_encapsulated_decision(
        &mut self,
        reference: TempDecisionServiceReferenceSnapshot,
    ) {
        self.encapsulated_decisions.push(reference);
    }

    pub(super) fn push_input_decision(&mut self, reference: TempDecisionServiceReferenceSnapshot) {
        self.input_decisions.push(reference);
    }

    pub(super) fn push_input_data(&mut self, reference: TempDecisionServiceReferenceSnapshot) {
        self.input_data.push(reference);
    }
}

#[derive(Debug)]
pub(super) struct TempDecisionServiceReferenceSnapshot {
    href: Option<String>,
    reference_kind: String,
}

impl From<TempDecisionServiceReferenceSnapshot> for DmnDecisionServiceReferenceSnapshot {
    fn from(value: TempDecisionServiceReferenceSnapshot) -> Self {
        Self {
            href: value.href,
            reference_kind: value.reference_kind,
        }
    }
}

impl TempDecisionServiceReferenceSnapshot {
    pub(super) fn from_event(
        source: &DmnSourceFile,
        reader: &Reader<&[u8]>,
        event: &BytesStart<'_>,
        reference_kind: &str,
    ) -> Result<Self> {
        Ok(Self {
            href: attribute_value(source, reader, event, "href")?,
            reference_kind: reference_kind.to_string(),
        })
    }
}
