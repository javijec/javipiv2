//! Preflight conservador: no abre rutas de almacenamiento ni ejecuta comandos.
use crate::core::catalog::profile_modules;
use std::io::{self, Read};

fn dietpi_evidence(os: &str, marker: Option<&str>) -> bool {
    os.lines()
        .any(|line| matches!(line.trim(), "ID=dietpi" | "ID=\"dietpi\""))
        || marker.is_some_and(|text| !text.trim().is_empty())
}

pub fn run(fixture: bool) -> Result<(), String> {
    let (dietpi, arch, storage) = if fixture {
        let mut bytes = Vec::new();
        io::stdin()
            .take(1025)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 1024 {
            return Err("fixture demasiado grande".into());
        }
        let input = String::from_utf8(bytes).map_err(|_| "fixture inválida")?;
        let lines: Vec<_> = input.lines().collect();
        if lines.len() != 3
            || !matches!(lines[0], "dietpi" | "unknown" | "debian" | "debian-marker")
            || !matches!(lines[1], "aarch64" | "unknown")
            || !matches!(lines[2], "mounted" | "unknown")
        {
            return Err("fixture inválida".into());
        }
        println!("Evidencia simulada (fixture, no describe el host):");
        (
            dietpi_evidence(
                if lines[0] == "dietpi" {
                    "ID=dietpi"
                } else {
                    "ID=debian"
                },
                if lines[0] == "debian-marker" {
                    Some("G_DIETPI_VERSION_CORE=9")
                } else {
                    None
                },
            ),
            lines[1] == "aarch64",
            lines[2] == "mounted",
        )
    } else {
        let os = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
        let marker = std::fs::read_to_string("/boot/dietpi/.version").ok();
        let dietpi = dietpi_evidence(&os, marker.as_deref());
        println!("Preflight de solo lectura (arquitectura del binario):");
        (dietpi, std::env::consts::ARCH == "aarch64", false)
    };
    println!(
        "DietPi: {}",
        if dietpi { "verificado" } else { "desconocido" }
    );
    println!("ARM64: {}", if arch { "verificado" } else { "desconocido" });
    println!(
        "Almacenamiento: {}",
        if storage {
            "montaje informado por fixture; capacidad desconocida"
        } else {
            "desconocido"
        }
    );
    println!("Dependencias del sistema: desconocidas; no se inspeccionan ejecutables.");
    println!("Vista ordenada; no es un plan exacto ni instala nada:");
    for module in profile_modules("minimal")? {
        let deps = if module.dependencies.is_empty() {
            "ninguna".to_owned()
        } else {
            module.dependencies.join(", ")
        };
        println!(
            "{}: dependencias: {}; presencia: desconocida; acciones: desconocidas",
            module.id, deps
        );
    }
    println!("Idempotencia y reanudación: pendientes; no se escribe estado.");
    Ok(())
}
