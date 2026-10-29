//! Diagnóstico pasivo: las señales ambientales no prueban instalación.
use super::{health, services, storage};
use crate::core::catalog::profile_modules;
use std::fmt;

/// Reservados para una fuente de instalación confiable aún inexistente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationState {
    Installed,
    Absent,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Available,
    Unknown,
    Error,
}

#[derive(Debug)]
pub struct Diagnosis {
    pub requirements: Vec<(&'static str, InstallationState)>,
    pub health: Signal,
    pub storage: Signal,
    pub services: Signal,
}

pub fn diagnose_with(
    health_query: impl FnOnce() -> Result<health::SystemStatus, std::io::Error>,
    storage_query: impl FnOnce() -> Result<storage::StorageStatus, String>,
    services_query: impl FnOnce() -> Result<Vec<services::Service>, String>,
) -> Result<Diagnosis, String> {
    let requirements = profile_modules("minimal")?
        .into_iter()
        .map(|m| (m.id, InstallationState::Unknown))
        .collect();
    let health = match health_query() {
        Ok(_) => Signal::Available,
        Err(_) => Signal::Error,
    };
    let storage = match storage_query() {
        Ok(s) if s.autofs || s.srv_ext4 || !s.docker_overlays.is_empty() => Signal::Available,
        Ok(_) => Signal::Unknown,
        Err(_) => Signal::Error,
    };
    let services = match services_query() {
        Ok(s)
            if s.iter().any(|s| {
                [&s.load, &s.active, &s.sub].iter().any(|v| {
                    v.as_deref()
                        .is_some_and(|v| !v.is_empty() && v != "-" && v != "desconocido")
                })
            }) =>
        {
            Signal::Available
        }
        Ok(_) => Signal::Unknown,
        Err(_) => Signal::Error,
    };
    Ok(Diagnosis {
        requirements,
        health,
        storage,
        services,
    })
}

pub fn diagnose() -> Result<Diagnosis, String> {
    diagnose_with(
        health::read_status,
        storage::read_status_fallible,
        services::list,
    )
}

impl fmt::Display for Diagnosis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Perfil minimal: requisitos de instalación")?;
        for (id, state) in &self.requirements {
            let label = match state {
                InstallationState::Installed => "instalado",
                InstallationState::Absent => "ausente",
                InstallationState::Unknown => "no verificado",
            };
            writeln!(f, "{id}: {label}")?;
        }
        writeln!(f, "Instalado y ausente están reservados: ninguna fuente actual los confirma. El código disponible no acredita instalación.")?;
        writeln!(
            f,
            "Señales ambientales independientes (no prueban instalación):"
        )?;
        for (name, signal) in [
            ("Health", self.health),
            ("Storage", self.storage),
            ("Services", self.services),
        ] {
            writeln!(
                f,
                "{name}: {}",
                match signal {
                    Signal::Available => "disponible",
                    Signal::Unknown => "desconocido",
                    Signal::Error => "error de consulta",
                }
            )?;
        }
        write!(f, "Plan de instalación ejecutable: no definido")
    }
}
