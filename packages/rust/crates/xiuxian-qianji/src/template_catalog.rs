//! Embedded template catalog for Qianji-owned markdown/control-plane surfaces.

use std::sync::OnceLock;

use serde_json::Value;
use tera::{Context, Tera};

/// Lazy shared catalog for embedded templates.
pub(crate) struct EmbeddedTemplateCatalog {
    init_error_context: &'static str,
    templates: &'static [(&'static str, &'static str)],
    renderer: OnceLock<Result<Tera, String>>,
}

impl EmbeddedTemplateCatalog {
    /// Build one embedded template catalog with lazy renderer initialization.
    #[must_use]
    pub(crate) const fn new(
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
    pub(crate) fn render_text(
        &self,
        template_name: &str,
        payload: Value,
    ) -> Result<String, String> {
        let context = Context::from_value(payload).map_err(|error| {
            format!("failed to build template context for `{template_name}`: {error}")
        })?;
        self.renderer()?
            .render(template_name, &context)
            .map_err(|error| format!("failed to render `{template_name}`: {error}"))
    }

    /// Render one embedded template and split it into lines.
    pub(crate) fn render_lines(
        &self,
        template_name: &str,
        payload: Value,
    ) -> Result<Vec<String>, String> {
        self.render_text(template_name, payload)
            .map(|rendered| rendered.lines().map(str::to_string).collect())
    }

    fn renderer(&self) -> Result<&Tera, String> {
        self.renderer
            .get_or_init(|| {
                let mut tera = Tera::default();
                for (name, source) in self.templates {
                    tera.add_raw_template(name, source).map_err(|error| {
                        format!(
                            "failed to initialize {} template `{name}`: {error}",
                            self.init_error_context
                        )
                    })?;
                }
                Ok(tera)
            })
            .as_ref()
            .map_err(Clone::clone)
    }
}
