//! Embedded manifestation template catalog adapter.

use std::sync::OnceLock;

use serde_json::Value;

use crate::{ManifestationInterface, ManifestationManager};

/// Lazy shared catalog for embedded manifestation templates.
pub struct EmbeddedManifestationTemplateCatalog {
    init_error_context: &'static str,
    templates: &'static [(&'static str, &'static str)],
    renderer: OnceLock<Result<ManifestationManager, String>>,
}

impl EmbeddedManifestationTemplateCatalog {
    /// Build one embedded template catalog with lazy manifestation-manager initialization.
    #[must_use]
    pub const fn new(
        init_error_context: &'static str,
        templates: &'static [(&'static str, &'static str)],
    ) -> Self {
        Self {
            init_error_context,
            templates,
            renderer: OnceLock::new(),
        }
    }

    /// Render one embedded template into raw text.
    ///
    /// # Errors
    ///
    /// Returns an error when the catalog cannot initialize or the template
    /// render fails.
    pub fn render_text(&self, template_name: &str, payload: Value) -> Result<String, String> {
        self.renderer()?
            .render_template(template_name, payload)
            .map_err(|error| {
                format!("failed to render `{template_name}` through qianhuan: {error}")
            })
    }

    /// Render one embedded template and split it into lines.
    ///
    /// # Errors
    ///
    /// Returns an error when the catalog cannot initialize or the template
    /// render fails.
    pub fn render_lines(&self, template_name: &str, payload: Value) -> Result<Vec<String>, String> {
        self.render_text(template_name, payload)
            .map(|rendered| rendered.lines().map(str::to_string).collect())
    }

    fn renderer(&self) -> Result<&ManifestationManager, String> {
        self.renderer
            .get_or_init(|| {
                ManifestationManager::new_with_embedded_templates(&[], self.templates).map_err(
                    |error| format!("failed to initialize {}: {error}", self.init_error_context),
                )
            })
            .as_ref()
            .map_err(Clone::clone)
    }
}
