//! Diagnóstico pasivo: las señales ambientales no prueban instalación.
use super::installation_registry::{self, Availability};
use super::{health, services, storage};
use crate::core::catalog::profile_modules;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Available,
    Unknown,
    Error,
}

#[derive(Debug)]
pub struct Diagnosis {
    pub capabilities: Vec<&'static str>,
    pub binary: Availability,
    pub health: Signal,
    pub storage: Signal,
    pub services: Signal,
}

pub fn diagnose_with(
    health_query: impl FnOnce() -> Result<health::SystemStatus, std::io::Error>,
    storage_query: impl FnOnce() -> Result<storage::StorageStatus, String>,
    services_query: impl FnOnce() -> Result<Vec<services::Service>, String>,
) -> Result<Diagnosis, String> {
    diagnose_with_status(
        Availability::Unknown,
        health_query,
        storage_query,
        services_query,
    )
}

pub fn diagnose_with_status(
    binary: Availability,
    health_query: impl FnOnce() -> Result<health::SystemStatus, std::io::Error>,
    storage_query: impl FnOnce() -> Result<storage::StorageStatus, String>,
    services_query: impl FnOnce() -> Result<Vec<services::Service>, String>,
) -> Result<Diagnosis, String> {
    let capabilities = profile_modules("minimal")?
        .into_iter()
        .map(|m| m.id)
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
        capabilities,
        binary,
        health,
        storage,
        services,
    })
}

pub fn diagnose() -> Result<Diagnosis, String> {
    let verified = installation_registry::verify_production();
    let status = if verified == Availability::Unknown && cfg!(debug_assertions) {
        installation_registry::development_availability()
    } else {
        verified
    };
    diagnose_with_status(
        status,
        health::read_status,
        storage::read_status_fallible,
        services::list,
    )
}

impl fmt::Display for Diagnosis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Perfil minimal: un binario")?;
        writeln!(
            f,
            "Binario: {}",
            match self.binary {
                Availability::Verified => "instalado verificado (coincide con registro confiable)",
                Availability::AvailableInDevelopment =>
                    "disponible en desarrollo; instalación desconocida",
                Availability::Unknown => "instalación desconocida",
            }
        )?;
        writeln!(f, "Capacidades internas: {}", self.capabilities.join(", "))?;
        writeln!(
            f,
            "Las capacidades no son paquetes instalados por separado."
        )?;
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
