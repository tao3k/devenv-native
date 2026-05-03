mod batch;
mod provider;
mod response;
#[cfg(test)]
#[path = "../../../../../tests/unit/gateway/studio/search/handlers/attachments/mod.rs"]
mod tests;

pub(crate) use provider::StudioAttachmentSearchFlightRouteProvider;
#[cfg(test)]
pub(crate) use response::load_attachment_search_response_from_studio;
