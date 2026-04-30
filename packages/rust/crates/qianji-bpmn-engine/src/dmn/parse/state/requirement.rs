use super::model::{TempInformationRequirementReference, TempKnowledgeRequirementReference};
use crate::dmn_model_api::{DmnInformationRequirementReference, DmnKnowledgeRequirementReference};

impl From<TempInformationRequirementReference> for DmnInformationRequirementReference {
    fn from(value: TempInformationRequirementReference) -> Self {
        Self::new(value.reference_kind, value.href)
    }
}

impl From<TempKnowledgeRequirementReference> for DmnKnowledgeRequirementReference {
    fn from(value: TempKnowledgeRequirementReference) -> Self {
        Self::new(value.reference_kind, value.href)
    }
}
