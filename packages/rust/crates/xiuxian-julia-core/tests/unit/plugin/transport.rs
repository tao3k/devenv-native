use xiuxian_wendao_core::{
    repo_intelligence::{RegisteredRepository, RepositoryPluginConfig},
    transport::PluginTransportKind,
};

use super::{build_flight_transport_binding, build_julia_flight_transport_client};

include!("transport/config.rs");
