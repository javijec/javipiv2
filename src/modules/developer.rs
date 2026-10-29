//! Comparación offline de versiones suministradas por el usuario.
use super::agents;
use super::updates::{self, Comparison};

/// Present independent local evidence without probing additional paths.
pub fn status_from(observations: Vec<agents::Observation>) -> Vec<String> {
    observations.into_iter().map(|item| {
        let executable = item.executable.as_deref().map_or_else(
            || "desconocido".to_owned(),
            |path| format!("{path} (ruta consultada; destino regular con bits de ejecución; acceso no garantizado)"),
        );
        format!("{}: ejecutable={}, versión npm={} (fuente: metadatos npm locales; no vinculada al ejecutable), disponibilidad efectiva=desconocido, estado de ejecución=desconocido", item.name, executable, item.version.as_deref().unwrap_or("desconocido"))
    }).collect()
}

pub fn status() -> Vec<String> {
    status_from(agents::status())
}

pub fn compare(installed: &str, reference: &str) -> Comparison {
    fn valid(value: &str) -> bool {
        !value.starts_with('v') && updates::parse_version(value.as_bytes()).is_some()
    }
    if !valid(installed) || !valid(reference) {
        return Comparison::Unknown;
    }
    updates::compare(Some(installed), Some(reference))
}
