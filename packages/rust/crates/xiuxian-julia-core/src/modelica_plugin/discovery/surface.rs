//! Modelica repository surface classification.

use crate::modelica_plugin::pathing::path_components;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RepositorySurface {
    Api,
    Example,
    Documentation,
    Support,
}

pub(crate) fn repository_surface(relative_path: &str) -> RepositorySurface {
    let components = path_components(relative_path);
    if components.contains(&"UsersGuide") {
        return RepositorySurface::Documentation;
    }
    if components.contains(&"Internal") {
        return RepositorySurface::Support;
    }
    if let Some(examples_index) = components
        .iter()
        .position(|component| *component == "Examples")
    {
        if components
            .iter()
            .skip(examples_index + 1)
            .any(|component| matches!(*component, "ExampleUtilities" | "Utilities"))
        {
            return RepositorySurface::Support;
        }
        return RepositorySurface::Example;
    }
    RepositorySurface::Api
}

pub(crate) fn is_api_surface_path(relative_path: &str) -> bool {
    repository_surface(relative_path) == RepositorySurface::Api
}
