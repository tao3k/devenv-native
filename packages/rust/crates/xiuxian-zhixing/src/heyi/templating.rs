use super::ZhixingHeyi;
use serde_json::{Map, Value, json};

const DEFAULT_ZHIXING_DOMAIN: &str = "zhixing.agenda";

impl ZhixingHeyi {
    pub(super) fn render_with_manifestation_context(
        &self,
        template_name: &str,
        payload: Value,
        state_context: &str,
    ) -> crate::Result<String> {
        let mut root = match payload {
            Value::Object(map) => map,
            value => {
                let mut map = Map::new();
                map.insert("payload".to_string(), value);
                map
            }
        };

        let persona_json = self.active_persona.as_ref().map(|persona| {
            json!({
                "id": persona.id,
                "name": persona.name,
                "voice_tone": persona.voice_tone,
                "style_anchors": persona.style_anchors,
            })
        });

        root.insert(
            "manifestation".to_string(),
            json!({
                "state_context": state_context,
                "injected_context": self.manifestation.inject_context(state_context),
                "persona": persona_json,
                "persona_id": self.active_persona.as_ref().map(|persona| persona.id.as_str()),
                "domain": DEFAULT_ZHIXING_DOMAIN,
            }),
        );

        self.manifestation
            .render_template(template_name, Value::Object(root))
            .map_err(|error| {
                crate::Error::Internal(format!(
                    "Failed to render template `{template_name}` with manifestation context: {error}"
                ))
            })
    }
}
