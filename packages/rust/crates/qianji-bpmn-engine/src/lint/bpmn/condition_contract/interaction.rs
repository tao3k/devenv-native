use super::{
    BytesStart, Event, HashMap, Reader, StaticInteractionChoiceOutput, Value,
    append_entity_reference, attribute_value, is_element, local_name,
};

mod api;
mod model;
mod task;
mod text;

use model::{
    ActiveNativeInteractionTask, NativeAssociationCapture, NativeInputAssociation,
    NativeOutputAssociation,
};
use text::{append_to_option, choice_values_from_assignment};

pub(super) use api::collect_static_interaction_choice_outputs;
