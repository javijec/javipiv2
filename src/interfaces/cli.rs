//! Una sola interfaz de comandos, sin operaciones privilegiadas.

use crate::core::catalog::{all_modules, profile_modules};
use crate::core::minimal_simulation::MinimalSimulation;
use crate::modules::{
    agents, backups, docker, health, preflight, projects, remote, services, storage, updates,
};
use std::ffi::OsString;
use std::io::Read;

fn help() {
    println!(
        "JaviPiV2 {} — preview de solo lectura",
        env!("CARGO_PKG_VERSION")
    );
    println!("Uso: javipiv2 <comando>");
    println!("  status                 Host, uptime, memoria, carga y temperatura");
    println!(
        "  tui                    Interfaz interactiva de solo lectura (pendiente de validación)"
    );
    println!("  module list            Catálogo (no instalaciones reales)");
    println!("  service list           Servicios conocidos por systemd (solo lectura)");
    println!("  service status <nombre>  Estado de un servicio (solo lectura)");
    println!("  service plan <start|stop|restart> <unidad.service> --allow <unidad.service>");
    println!("                         Solo plan; --allow autoriza esa unidad para esta consulta");
    println!("  backup inspect --stdin  Observar snapshots JSON suministrados por stdin");
    println!("  storage status         Montajes relevantes (observación pasiva)");
    println!("  updates check          Comparar componentes con upstream (requiere red)");
    println!("  agents status          Observar ejecutables y versiones npm locales (sin red)");
    println!("  remote status          Observar unidades y TCP local sin atribución (sin red)");
    println!("  docker status          Consultar daemon Docker local");
    println!("  docker list            Listar contenedores locales");
    println!("  docker inspect <nombre>  Información básica de un contenedor");
    println!("  project list           Inventario Git de solo lectura");
    println!("  project status <nombre>  Información Git del proyecto");
    println!(
        "  JAVIPIV2_WORKSPACES    Ruta configurable bajo /srv (predeterminada: /srv/workspaces)"
    );
    println!("  plan <perfil>          Módulos previstos: minimal, developer, full");
    println!("  simulate minimal       Progreso simulado de solo lectura, sin instalar");
    println!("  simulate minimal --fixture  Simulación con evidencia de stdin");
    println!("  preflight minimal      Diagnóstico y dependencias de solo lectura");
    println!("  preflight minimal --fixture  Diagnóstico simulado desde stdin");
    println!("  version                Versión del proyecto");
    println!("  help                   Esta ayuda");
}

fn format_updates(observations: &[updates::Observation]) -> String {
    observations
        .iter()
        .map(|observation| observation.line())
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

fn format_storage_capacity(capacity: Option<storage::Capacity>) -> String {
    match capacity {
        Some(capacity) => format!(
            "Capacidad /srv: total={}, usada={}, disponible={}, reservada={}",
            format_bytes(capacity.total_bytes),
            format_bytes(capacity.used_bytes),
            format_bytes(capacity.available_bytes),
            format_bytes(capacity.reserved_bytes)
        ),
        None => "Capacidad /srv: desconocido".into(),
    }
}

fn print_service(service: &services::Service) {
    println!(
        "{}: carga={}, actividad={}, subestado={}, categoría={}",
        service.name,
        service.load.as_deref().unwrap_or("desconocido"),
        service.active.as_deref().unwrap_or("desconocido"),
        service.sub.as_deref().unwrap_or("desconocido"),
        service.category()
    );
}

fn print_container(container: &docker::Container) {
    println!(
        "{}: id={}, imagen={}, estado={}",
        container.name, container.id, container.image, container.state
    );
}

fn print_project(project: &projects::Project) {
    let state = match project.git {
        projects::GitState::Compatible => "compatible (validez no verificada)",
        projects::GitState::NotGit => "sin Git",
        projects::GitState::Unknown => "unknown",
    };
    println!(
        "{}: Git={}, rama={}, HEAD={}",
        project.name,
        state,
        project.branch.as_deref().unwrap_or("unknown"),
        project.head.as_deref().unwrap_or("unknown")
    );
}

fn projects() -> Result<projects::Projects, String> {
    projects::Projects::new(
        std::env::var_os("JAVIPIV2_WORKSPACES")
            .unwrap_or_else(|| OsString::from("/srv/workspaces")),
    )
}

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), String> {
    let args = args.into_iter().collect::<Vec<_>>();
    let parts = args
        .iter()
        .map(|s| s.to_str().ok_or("argumento no UTF-8"))
        .collect::<Result<Vec<_>, _>>()?;
    match parts.as_slice() {
        [] | ["help"] | ["--help"] | ["-h"] => help(),
        ["version"] | ["--version"] => println!("javipiv2 {}", env!("CARGO_PKG_VERSION")),
        ["tui"] => super::tui::run()?,
        ["status"] => {
            let status = health::read_status().map_err(|e| format!("consulta /proc: {e}"))?;
            println!("Host: {}", status.hostname);
            println!(
                "Uptime: {} h {} min",
                status.uptime_seconds / 3600,
                (status.uptime_seconds / 60) % 60
            );
            println!(
                "Memoria disponible: {} / {} MiB",
                status.memory_available_kib / 1024,
                status.memory_total_kib / 1024
            );
            let cpu = health::read_cpu_observation();
            match cpu.load_average {
                Some([one, five, fifteen]) => {
                    println!("Carga promedio (1, 5, 15 min): {one}, {five}, {fifteen}");
                }
                None => println!("Carga promedio (1, 5, 15 min): desconocida"),
            }
            match cpu.cpu_temperature_millicelsius {
                Some(temperature) => {
                    println!("Temperatura CPU: {:.3} °C", temperature as f64 / 1000.0);
                }
                None => println!("Temperatura CPU: desconocida"),
            }
        }
        ["backup", "inspect", "--stdin"] => {
            const MAX_INPUT: usize = 4 * 1024 * 1024;
            let mut input = Vec::new();
            std::io::stdin()
                .lock()
                .take((MAX_INPUT + 1) as u64)
                .read_to_end(&mut input)
                .map_err(|_| "no se pudo leer stdin")?;
            if input.len() > MAX_INPUT {
                return Err("entrada de snapshots demasiado grande".into());
            }
            let observation = backups::inspect(&input)?;
            println!("Snapshots: {}", observation.count);
            if let Some(snapshot) = observation.newest {
                println!("Fecha: {}", snapshot.time);
                println!("ID: {}", snapshot.id.as_deref().unwrap_or("desconocido"));
                println!(
                    "Perfil: {}",
                    snapshot.profile.as_deref().unwrap_or("desconocido")
                );
            } else {
                println!("Último snapshot: desconocido");
            }
            println!("Último intento: desconocido");
            println!("Resultado del último intento: desconocido");
        }
        ["updates", "check"] => println!("{}", format_updates(&updates::check())),
        ["remote", "status"] => {
            println!("Estado de servicios locales; no demuestra conectividad.");
            for observation in remote::status() {
                println!("{}", observation.line());
            }
        }
        ["agents", "status"] => {
            for observation in agents::status() {
                println!("{}", observation.line());
            }
        }
        ["storage", "status"] => {
            let (status, capacity) = storage::read_capacity().unwrap_or_default();
            println!(
                "/srv autofs: {}",
                if status.autofs {
                    "confirmado"
                } else {
                    "desconocido"
                }
            );
            println!(
                "/srv ext4: {}",
                if status.srv_ext4 {
                    "confirmado"
                } else {
                    "desconocido"
                }
            );
            if status.srv_ext4 {
                println!("Montaje de datos: /srv (ext4)");
            }
            if status.docker_overlays.is_empty() {
                println!("Overlay Docker: desconocido");
            }
            for path in status.docker_overlays {
                println!("Overlay Docker: {path} (overlay)");
            }
            println!("{}", format_storage_capacity(capacity));
        }
        ["docker", "status"] => {
            docker::status()?;
            println!("Daemon Docker local disponible");
        }
        ["docker", "list"] => {
            let containers = docker::list()?;
            if containers.is_empty() {
                println!("No hay contenedores");
            }
            for container in &containers {
                print_container(container);
            }
        }
        ["docker", "inspect", name] => print_container(&docker::inspect(name)?),
        ["project", "list"] => {
            let entries = projects()?.list()?;
            if entries.is_empty() {
                println!("No hay proyectos");
            }
            for entry in &entries {
                print_project(entry);
            }
        }
        ["project", "status", name] => print_project(&projects()?.status(name)?),
        ["service", "list"] => {
            println!("Servicios conocidos por systemd (no es una lista completa de unidades instaladas):");
            for service in services::list()? {
                print_service(&service);
            }
        }
        ["service", "status", name] => print_service(&services::status(name)?),
        ["service", "plan", operation, unit, "--allow", allowed] => {
            let operation = operation.parse::<services::ServiceOperation>()?;
            let allowlist = services::PlanAllowlist::new([(*allowed).to_owned()])?;
            let plan = allowlist.plan(operation, unit)?;
            println!(
                "Plan solicitado (no ejecutado): {} en {}",
                plan.operation.as_str(),
                plan.unit
            );
        }
        ["service", "plan", operation, unit] => {
            let operation = operation.parse::<services::ServiceOperation>()?;
            services::PlanAllowlist::default().plan(operation, unit)?;
        }
        ["module", "list"] => {
            for module in all_modules()? {
                println!(
                    "{:<15} {:<8} {}",
                    module.id, module.state, module.description
                );
            }
            println!("Nota: el catálogo no acredita instalación ni activación.");
        }
        ["simulate", "minimal"] | ["simulate", "minimal", "--fixture"] => {
            preflight::run(parts.len() == 3)?;
            println!("Componentes de código disponibles en este programa: catálogo core y consulta health; no acreditan instalación en el host.");
            println!("Operaciones previstas: acciones de instalación sin definir; no se ejecuta ninguna operación real.");
            println!("Pasos del perfil (selección, no instalación):");
            for (index, module) in profile_modules("minimal")?.iter().enumerate() {
                let availability = match module.id {
                    "core" => "código de catálogo y CLI disponible en este programa",
                    "health" => "consulta de salud disponible en este programa",
                    _ => "funcionalidad pendiente",
                };
                println!(
                    "{}. {}: {}; presencia instalada desconocida; instalación sin definir",
                    index + 1,
                    module.id,
                    availability
                );
            }
            println!("Comprobaciones pendientes: dependencias del sistema, presencia de módulos y capacidad de almacenamiento (desconocida).");
            println!("Antes de una futura operación que requiera capacidad deberá verificarse de forma segura; si no es posible, se rechazará sin modificar datos.");
            let mut simulation = MinimalSimulation::new()?;
            while let Some(id) = simulation.step()? {
                println!("Progreso simulado: {id} completado (no instalado)");
            }
            print!("{}", simulation.report());
        }
        ["preflight", "minimal"] => preflight::run(false)?,
        ["preflight", "minimal", "--fixture"] => preflight::run(true)?,
        ["plan", profile] => {
            println!("Perfil {profile} (propuesta, no ejecuta instalaciones):");
            for module in profile_modules(profile)? {
                println!("  {:<15} {}", module.id, module.state);
            }
        }
        _ => return Err("comando desconocido; usá javipiv2 help".into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::ffi::OsString;

    #[test]
    fn updates_cli_renders_synthetic_data_without_query() {
        let observations = crate::modules::updates::check_with(
            |_| Some(br#"{"version":"1.0.0"}"#.to_vec()),
            |_| Some(br#"{"version":"2.0.0","tag_name":"v2.0.0"}"#.to_vec()),
        );
        let text = super::format_updates(&observations);
        assert!(text
            .contains("Codex: instalada=1.0.0, upstream=2.0.0, estado=actualización disponible"));
        assert!(text.contains("Engram: instalada=desconocido, upstream=2.0.0, estado=desconocido"));
        assert_eq!(text.lines().count(), 6);
    }

    #[test]
    fn storage_capacity_rendering_is_exact() {
        use crate::modules::storage::Capacity;
        let capacity = Capacity {
            total_bytes: 1000,
            used_bytes: 600,
            available_bytes: 300,
            reserved_bytes: 100,
        };
        assert_eq!(
            super::format_storage_capacity(Some(capacity)),
            "Capacidad /srv: total=1000.0 B, usada=600.0 B, disponible=300.0 B, reservada=100.0 B"
        );
        assert_eq!(
            super::format_storage_capacity(None),
            "Capacidad /srv: desconocido"
        );
    }

    #[test]
    fn known_read_only_commands() {
        for args in [
            vec!["help"],
            vec!["version"],
            vec!["module", "list"],
            vec!["plan", "minimal"],
        ] {
            assert!(run(args.into_iter().map(OsString::from)).is_ok());
        }
    }

    #[test]
    fn service_plans_require_explicit_exact_allowlist() {
        let call = |args: Vec<&str>| run(args.into_iter().map(OsString::from));
        assert!(call(vec![
            "service",
            "plan",
            "restart",
            "demo.service",
            "--allow",
            "demo.service"
        ])
        .is_ok());
        assert!(call(vec!["service", "plan", "start", "demo.service"]).is_err());
        assert!(call(vec![
            "service",
            "plan",
            "stop",
            "demo.service",
            "--allow",
            "other.service"
        ])
        .is_err());
        assert!(call(vec![
            "service",
            "plan",
            "reload",
            "demo.service",
            "--allow",
            "demo.service"
        ])
        .is_err());
        assert!(call(vec![
            "service",
            "plan",
            "start",
            "../demo.service",
            "--allow",
            "../demo.service"
        ])
        .is_err());
    }

    #[test]
    fn rejects_unknown_commands() {
        assert!(run(vec![OsString::from("service"), OsString::from("restart")]).is_err());
    }
}
