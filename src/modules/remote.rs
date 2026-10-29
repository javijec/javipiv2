//! Observación local de unidades conocidas, sin inferir conectividad.
use super::services;

#[derive(Debug)]
pub struct Observation {
    name: &'static str,
    unit: &'static str,
    load: Option<String>,
    active: Option<String>,
}

impl Observation {
    pub fn line(&self) -> String {
        format!(
            "{} ({}): carga={}, actividad={}",
            self.name,
            self.unit,
            self.load.as_deref().unwrap_or("desconocido"),
            self.active.as_deref().unwrap_or("desconocido")
        )
    }
}

pub fn status_with(
    mut query: impl FnMut(&str) -> Result<services::Service, String>,
) -> Vec<Observation> {
    [("SSH", "ssh.service"), ("Tailscale", "tailscaled.service")]
        .into_iter()
        .map(|(name, unit)| {
            let result = query(unit).ok().filter(|service| service.name == unit);
            Observation {
                name,
                unit,
                load: result
                    .as_ref()
                    .and_then(|s| s.load.clone())
                    .filter(|s| !s.is_empty() && s != "-"),
                active: result
                    .and_then(|s| s.active)
                    .filter(|s| !s.is_empty() && s != "-"),
            }
        })
        .collect()
}

pub fn status() -> Vec<Observation> {
    status_with(services::status)
}
