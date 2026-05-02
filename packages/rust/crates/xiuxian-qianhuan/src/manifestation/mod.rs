//! Template manifestation manager, request, and catalog boundary.

/// Embedded-template catalog helpers.
pub mod catalog;
/// Manifestation manager logic.
pub mod manager;
/// Manifestation render request models.
pub mod request;
/// Template helper logic.
pub mod templates;

pub use catalog::EmbeddedManifestationTemplateCatalog;
pub use manager::{ManifestationManager, MemoryTemplateRecord};
pub use request::{
    ManifestationRenderRequest, ManifestationRuntimeContext, ManifestationTemplateTarget,
};
