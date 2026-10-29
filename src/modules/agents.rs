//! Observación local conservadora; no ejecuta agentes ni consulta red.
use super::updates;

#[derive(Debug, Clone)]
pub struct Observation {
    pub name: &'static str,
    pub version: Option<String>,
}
impl Observation {
    pub fn line(&self) -> String {
        format!(
            "{}: disponibilidad=desconocido, versión={}, estado=desconocido",
            self.name,
            self.version.as_deref().unwrap_or("desconocido")
        )
    }
}
pub fn status() -> Vec<Observation> {
    status_with(updates::local_file)
}
pub fn status_with(mut local: impl FnMut(&str) -> Option<Vec<u8>>) -> Vec<Observation> {
    [
        ("Pi Agent", Some("@earendil-works/pi-coding-agent")),
        ("Gentle Shell", Some("gentle-pi")),
        ("Codex", Some("@openai/codex")),
        ("Engram", None),
    ]
    .into_iter()
    .map(|(name, package)| Observation {
        name,
        version: package.and_then(|p| updates::local_version(p, &mut local)),
    })
    .collect()
}
