//! Una sola interfaz de comandos, sin operaciones privilegiadas.

use crate::core::catalog::{all_modules, profile_modules};
use crate::core::minimal_simulation::MinimalSimulation;
use crate::modules::{backups, docker, health, preflight, projects, services};
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
