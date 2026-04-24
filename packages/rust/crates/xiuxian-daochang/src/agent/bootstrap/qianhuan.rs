use super::service_mount::{ServiceMountCatalog, ServiceMountMeta};
use crate::config::XiuxianConfig;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use xiuxian_qianhuan::{PersonaProfile, PersonaProvider, PersonaRegistry};
use xiuxian_wendao::enhancer::WendaoResourceRegistry;

const PERSONA_CONFIG_TYPE: &str = "persona";

#[derive(Debug, Clone)]
struct EmbeddedWendaoPersonaProvider {
    registry: WendaoResourceRegistry,
}

impl EmbeddedWendaoPersonaProvider {
    fn new(registry: WendaoResourceRegistry) -> Self {
        Self { registry }
    }
}

impl PersonaProvider for EmbeddedWendaoPersonaProvider {
    fn fetch_persona(&self, id: &str) -> Option<PersonaProfile> {
        let block = self.registry.get(id)?;
        if !block.config_type.eq_ignore_ascii_case(PERSONA_CONFIG_TYPE) {
            return None;
        }
        parse_persona_profile(id, block.content.as_str()).ok()
    }
}

static EMBEDDED_PERSONA_PROVIDER: OnceLock<Option<Arc<dyn PersonaProvider>>> = OnceLock::new();

pub(crate) struct LoadedPersonaRegistries {
    pub(crate) internal: Arc<PersonaRegistry>,
}

impl LoadedPersonaRegistries {
    #[must_use]
    pub(crate) fn internal_len(&self) -> usize {
        self.internal.len()
    }
}

/// Initializes persona registries in declarative mode.
///
/// The thin-agent runtime no longer loads personas/templates from arbitrary
/// filesystem directories and does not eagerly preload built-in personas.
/// Persona lookup is delegated to provider-backed semantic resolution on
/// cache miss.
pub(crate) fn init_persona_registries(
    _project_root: &Path,
    _xiuxian_cfg: &XiuxianConfig,
    mounts: &mut ServiceMountCatalog,
) -> LoadedPersonaRegistries {
    mounts.skipped(
        "qianhuan.template_dirs",
        "orchestration",
        ServiceMountMeta::default().detail("disabled(declarative_skill_runtime)"),
    );
    let internal_registry = init_internal_persona_registry(mounts);
    mounts.skipped(
        "qianhuan.persona_registry.user",
        "orchestration",
        ServiceMountMeta::default().detail("disabled(declarative_skill_runtime)"),
    );

    LoadedPersonaRegistries {
        internal: internal_registry,
    }
}

fn init_internal_persona_registry(mounts: &mut ServiceMountCatalog) -> Arc<PersonaRegistry> {
    let mut internal_registry = PersonaRegistry::new();
    mount_graph_persona_provider(&mut internal_registry, mounts);

    let internal_registry = Arc::new(internal_registry);
    mounts.mounted(
        "qianhuan.persona_registry.internal",
        "orchestration",
        ServiceMountMeta::default().detail(format!(
            "personas={}, source=provider_cache_only(declarative_skill_runtime)",
            internal_registry.len()
        )),
    );

    internal_registry
}

fn mount_graph_persona_provider(registry: &mut PersonaRegistry, mounts: &mut ServiceMountCatalog) {
    if let Some(provider) = shared_embedded_persona_provider() {
        registry.set_provider(provider);
        mounts.mounted(
            "qianhuan.persona_registry.graph_provider",
            "orchestration",
            ServiceMountMeta::default()
                .detail("source=zhixing.wendao.embedded_registry,mode=cache_miss_fetch"),
        );
    } else {
        mounts.failed(
            "qianhuan.persona_registry.graph_provider",
            "orchestration",
            ServiceMountMeta::default()
                .detail("failed to build embedded zhixing wendao registry for persona provider"),
        );
    }
}

fn shared_embedded_persona_provider() -> Option<Arc<dyn PersonaProvider>> {
    EMBEDDED_PERSONA_PROVIDER
        .get_or_init(|| build_embedded_wendao_persona_provider().ok())
        .clone()
}

fn build_embedded_wendao_persona_provider() -> Result<Arc<dyn PersonaProvider>, String> {
    let registry = xiuxian_wendao::build_embedded_wendao_registry()
        .map_err(|error| format!("failed to build embedded zhixing wendao registry: {error}"))?;
    Ok(Arc::new(EmbeddedWendaoPersonaProvider::new(registry)))
}

fn parse_persona_profile(id: &str, persona_toml: &str) -> Result<PersonaProfile, String> {
    let mut payload: toml::Value = toml::from_str(persona_toml)
        .map_err(|error| format!("failed to parse persona TOML for `{id}`: {error}"))?;
    let table = payload
        .as_table_mut()
        .ok_or_else(|| format!("persona payload for `{id}` must be a TOML table"))?;
    table.insert("id".to_string(), toml::Value::String(id.to_string()));
    payload.try_into().map_err(|error| {
        format!("failed to deserialize persona profile for `{id}` from wendao registry: {error}")
    })
}
